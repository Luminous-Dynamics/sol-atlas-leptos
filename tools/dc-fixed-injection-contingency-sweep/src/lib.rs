// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Exhaustive fixed-injection single-branch outage screening for small DC networks.
//!
//! This crate treats each in-service branch outage as a separate scenario, runs
//! the bounded DC reference solver, and independently audits each returned
//! candidate. Solver errors and auditor rejection remain explicit unknowns;
//! neither is counted as a passing contingency.
//!
//! This is not a full N-1 security assessment: it does not redispatch generators,
//! enforce generator reserve/ramp limits, model corrective switching, AC physics,
//! voltage/reactive constraints, protection, stability, or operating authority.

use dc_power_flow_independent_auditor::{AuditOptions, AuditReport, audit_dc_power_flow};
use dc_power_flow_reference::{DcNetwork, SolveOptions, solve_dc_power_flow};

const MAX_CONTINGENCIES: usize = 512;
const AUDIT_ANGLE_TOLERANCE_RAD: f64 = 1.0e-9;

#[derive(Clone, Copy, Debug)]
pub struct SweepOptions {
    pub solve_options: SolveOptions,
    /// Hard cap on the number of in-service branch outages to evaluate.
    /// Exceeding this cap rejects the whole sweep; no partial report is returned.
    pub max_contingencies: usize,
}

impl Default for SweepOptions {
    fn default() -> Self {
        Self {
            solve_options: SolveOptions::default(),
            max_contingencies: 256,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SweepError {
    EmptyNetworkLabel,
    InvalidContingencyBudget { requested: usize, maximum: usize },
    ContingencyBudgetExceeded { required: usize, configured: usize },
}

impl std::fmt::Display for SweepError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyNetworkLabel => {
                formatter.write_str("network identity label must not be empty")
            }
            Self::InvalidContingencyBudget { requested, maximum } => write!(
                formatter,
                "contingency budget {requested} is invalid; maximum supported budget is {maximum}"
            ),
            Self::ContingencyBudgetExceeded {
                required,
                configured,
            } => write!(
                formatter,
                "network requires {required} branch-outage evaluations; configured budget is {configured}"
            ),
        }
    }
}

impl std::error::Error for SweepError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutagedBranch {
    /// Position in the immutable input network's branch vector.
    pub branch_index: usize,
    pub from_bus: i64,
    pub to_bus: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScenarioDisposition {
    /// A separate auditor recomputed the declared DC equations and all reported
    /// values; no declared nodal or Rate A violation was found.
    AuditedWithinDeclaredChecks { audit: AuditReport },
    /// The candidate receipt was internally consistent, but the independent
    /// audit found a declared nodal or branch-limit violation.
    AuditedWithDeclaredViolations { audit: AuditReport },
    /// The reference solver did not return a candidate. This is unknown, not pass.
    SolverDidNotReturnCandidate { reason: String },
    /// The solver returned a candidate that the independent auditor rejected.
    CandidateRejectedByAuditor { reason: String },
}

impl ScenarioDisposition {
    pub fn is_audited(&self) -> bool {
        matches!(
            self,
            Self::AuditedWithinDeclaredChecks { .. } | Self::AuditedWithDeclaredViolations { .. }
        )
    }

    pub fn has_declared_violation(&self) -> bool {
        matches!(self, Self::AuditedWithDeclaredViolations { .. })
    }

    pub fn is_unknown(&self) -> bool {
        matches!(
            self,
            Self::SolverDidNotReturnCandidate { .. } | Self::CandidateRejectedByAuditor { .. }
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScenarioResult {
    /// None identifies the intact base case; Some identifies one branch outage.
    pub outage: Option<OutagedBranch>,
    pub disposition: ScenarioDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SweepSummary {
    pub expected_branch_outages: usize,
    pub evaluated_branch_outages: usize,
    pub audited_within_declared_checks: usize,
    pub audited_with_declared_violations: usize,
    pub solver_unknowns: usize,
    pub auditor_rejections: usize,
    pub coverage_complete: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContingencySweepReport {
    /// Caller-supplied label only. It is not authenticated or hashed by this crate.
    pub network_identity_label: String,
    pub base_case: ScenarioResult,
    /// Ordered by the original network branch-vector index.
    pub branch_outages: Vec<ScenarioResult>,
    pub summary: SweepSummary,
    pub claim_ceiling: &'static str,
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
    if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

fn audit_to_json(audit: &AuditReport) -> String {
    let reference_buses = audit
        .reference_bus_ids
        .iter()
        .map(i64::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        concat!(
            "{{\n",
            "    \"checked_bus_count\": {},\n",
            "    \"checked_branch_count\": {},\n",
            "    \"energized_island_count\": {},\n",
            "    \"reference_bus_ids\": [{}],\n",
            "    \"max_branch_flow_error_mw\": {},\n",
            "    \"max_nodal_outflow_error_mw\": {},\n",
            "    \"max_abs_nodal_residual_mw\": {},\n",
            "    \"max_branch_overload_mw\": {},\n",
            "    \"within_nodal_tolerance\": {},\n",
            "    \"within_declared_branch_limits\": {},\n",
            "    \"independently_feasible_under_declared_checks\": {},\n",
            "    \"claim_ceiling\": {}\n",
            "  }}"
        ),
        audit.checked_bus_count,
        audit.checked_branch_count,
        audit.energized_island_count,
        reference_buses,
        json_number(audit.max_branch_flow_error_mw),
        json_number(audit.max_nodal_outflow_error_mw),
        json_number(audit.max_abs_nodal_residual_mw),
        json_number(audit.max_branch_overload_mw),
        audit.within_nodal_tolerance,
        audit.within_declared_branch_limits,
        audit.independently_feasible_under_declared_checks,
        json_quote(audit.claim_ceiling)
    )
}

fn scenario_to_json(scenario: &ScenarioResult) -> String {
    let outage = match &scenario.outage {
        None => "null".to_owned(),
        Some(branch) => format!(
            concat!(
                "{{\n",
                "    \"branch_index\": {},\n",
                "    \"from_bus\": {},\n",
                "    \"to_bus\": {}\n",
                "  }}"
            ),
            branch.branch_index, branch.from_bus, branch.to_bus
        ),
    };
    let disposition = match &scenario.disposition {
        ScenarioDisposition::AuditedWithinDeclaredChecks { audit } => format!(
            "{{\n    \"status\": \"audited_within_declared_checks\",\n    \"audit\": {}\n  }}",
            audit_to_json(audit)
        ),
        ScenarioDisposition::AuditedWithDeclaredViolations { audit } => format!(
            "{{\n    \"status\": \"audited_with_declared_violations\",\n    \"audit\": {}\n  }}",
            audit_to_json(audit)
        ),
        ScenarioDisposition::SolverDidNotReturnCandidate { reason } => format!(
            "{{\n    \"status\": \"solver_did_not_return_candidate\",\n    \"reason\": {}\n  }}",
            json_quote(reason)
        ),
        ScenarioDisposition::CandidateRejectedByAuditor { reason } => format!(
            "{{\n    \"status\": \"candidate_rejected_by_auditor\",\n    \"reason\": {}\n  }}",
            json_quote(reason)
        ),
    };
    format!(
        "{{\n  \"outage\": {},\n  \"disposition\": {}\n}}",
        outage, disposition
    )
}

impl ContingencySweepReport {
    /// Deterministic JSON receipt. The caller must separately bind the network
    /// label to exact source bytes, import receipt and model-profile version.
    pub fn to_json(&self) -> String {
        let contingencies = self
            .branch_outages
            .iter()
            .map(scenario_to_json)
            .collect::<Vec<_>>()
            .join(",\n");
        format!(
            concat!(
                "{{\n",
                "  \"schema_version\": \"dc-fixed-injection-contingency-sweep.v0.1.0\",\n",
                "  \"network_identity_label\": {},\n",
                "  \"base_case\": {},\n",
                "  \"branch_outages\": [\n{}\n  ],\n",
                "  \"summary\": {{\n",
                "    \"expected_branch_outages\": {},\n",
                "    \"evaluated_branch_outages\": {},\n",
                "    \"audited_within_declared_checks\": {},\n",
                "    \"audited_with_declared_violations\": {},\n",
                "    \"solver_unknowns\": {},\n",
                "    \"auditor_rejections\": {},\n",
                "    \"coverage_complete\": {}\n",
                "  }},\n",
                "  \"claim_ceiling\": {}\n",
                "}}"
            ),
            json_quote(&self.network_identity_label),
            scenario_to_json(&self.base_case),
            contingencies,
            self.summary.expected_branch_outages,
            self.summary.evaluated_branch_outages,
            self.summary.audited_within_declared_checks,
            self.summary.audited_with_declared_violations,
            self.summary.solver_unknowns,
            self.summary.auditor_rejections,
            self.summary.coverage_complete,
            json_quote(self.claim_ceiling)
        )
    }
}

fn audit_options_for(solve: SolveOptions) -> AuditOptions {
    AuditOptions {
        value_tolerance_mw: solve.residual_tolerance_mw,
        angle_tolerance_rad: AUDIT_ANGLE_TOLERANCE_RAD,
        residual_tolerance_mw: solve.residual_tolerance_mw,
        branch_limit_tolerance_mw: solve.branch_limit_tolerance_mw,
    }
}

fn evaluate_scenario(
    network: &DcNetwork,
    outage: Option<OutagedBranch>,
    solve: SolveOptions,
) -> ScenarioResult {
    let mut scenario_network = network.clone();
    if let Some(branch) = &outage {
        scenario_network.branches[branch.branch_index].in_service = false;
    }

    let disposition = match solve_dc_power_flow(&scenario_network, solve) {
        Err(error) => ScenarioDisposition::SolverDidNotReturnCandidate {
            reason: error.to_string(),
        },
        Ok(candidate) => {
            match audit_dc_power_flow(&scenario_network, &candidate, audit_options_for(solve)) {
                Err(error) => ScenarioDisposition::CandidateRejectedByAuditor {
                    reason: error.to_string(),
                },
                Ok(audit) if audit.independently_feasible_under_declared_checks => {
                    ScenarioDisposition::AuditedWithinDeclaredChecks { audit }
                }
                Ok(audit) => ScenarioDisposition::AuditedWithDeclaredViolations { audit },
            }
        }
    };

    ScenarioResult {
        outage,
        disposition,
    }
}

/// Evaluate the intact base case and every single in-service branch outage.
///
/// The sweep is exhaustive over the declared branch-outage set or returns a
/// budget error before producing any report. Every successful candidate is
/// checked by the independent receipt auditor. The input network is never
/// mutated; each scenario operates on a clone.
///
/// The network label is an external correlation key only. Bind this report to
/// the actual source data and admission receipt in the calling application.
pub fn sweep_fixed_injection_single_branch_outages(
    network_identity_label: &str,
    network: &DcNetwork,
    options: SweepOptions,
) -> Result<ContingencySweepReport, SweepError> {
    if network_identity_label.trim().is_empty() {
        return Err(SweepError::EmptyNetworkLabel);
    }
    if options.max_contingencies == 0 || options.max_contingencies > MAX_CONTINGENCIES {
        return Err(SweepError::InvalidContingencyBudget {
            requested: options.max_contingencies,
            maximum: MAX_CONTINGENCIES,
        });
    }

    let active_branches = network
        .branches
        .iter()
        .enumerate()
        .filter(|(_, branch)| branch.in_service)
        .map(|(index, branch)| OutagedBranch {
            branch_index: index,
            from_bus: branch.from_bus,
            to_bus: branch.to_bus,
        })
        .collect::<Vec<_>>();

    if active_branches.len() > options.max_contingencies {
        return Err(SweepError::ContingencyBudgetExceeded {
            required: active_branches.len(),
            configured: options.max_contingencies,
        });
    }

    let base_case = evaluate_scenario(network, None, options.solve_options);
    let branch_outages = active_branches
        .into_iter()
        .map(|outage| evaluate_scenario(network, Some(outage), options.solve_options))
        .collect::<Vec<_>>();

    let mut summary = SweepSummary {
        expected_branch_outages: branch_outages.len(),
        evaluated_branch_outages: branch_outages.len(),
        audited_within_declared_checks: 0,
        audited_with_declared_violations: 0,
        solver_unknowns: 0,
        auditor_rejections: 0,
        coverage_complete: true,
    };

    for scenario in &branch_outages {
        match &scenario.disposition {
            ScenarioDisposition::AuditedWithinDeclaredChecks { .. } => {
                summary.audited_within_declared_checks += 1;
            }
            ScenarioDisposition::AuditedWithDeclaredViolations { .. } => {
                summary.audited_with_declared_violations += 1;
            }
            ScenarioDisposition::SolverDidNotReturnCandidate { .. } => {
                summary.solver_unknowns += 1;
            }
            ScenarioDisposition::CandidateRejectedByAuditor { .. } => {
                summary.auditor_rejections += 1;
            }
        }
    }

    Ok(ContingencySweepReport {
        network_identity_label: network_identity_label.to_owned(),
        base_case,
        branch_outages,
        summary,
        claim_ceiling: "audited_fixed_injection_dc_single_branch_outage_screening_only",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dc_power_flow_reference::{DcBranch, DcBus};

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
                    specified_injection_mw: Some(0.0),
                },
                DcBus {
                    id: 3,
                    specified_injection_mw: Some(-60.0),
                },
            ],
            branches: vec![
                DcBranch {
                    from_bus: 1,
                    to_bus: 2,
                    reactance_pu: 0.1,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: Some(45.0),
                    in_service: true,
                },
                DcBranch {
                    from_bus: 2,
                    to_bus: 3,
                    reactance_pu: 0.1,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: Some(45.0),
                    in_service: true,
                },
                DcBranch {
                    from_bus: 1,
                    to_bus: 3,
                    reactance_pu: 0.2,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: Some(50.0),
                    in_service: true,
                },
            ],
        }
    }

    #[test]
    fn evaluates_base_case_and_every_active_branch_outage_in_stable_order() {
        let network = triangle();
        let report = sweep_fixed_injection_single_branch_outages(
            "pglib-case14-ieee:dc-profile-v0.1",
            &network,
            SweepOptions::default(),
        )
        .expect("bounded sweep");

        assert!(matches!(
            report.base_case.disposition,
            ScenarioDisposition::AuditedWithinDeclaredChecks { .. }
        ));
        assert_eq!(report.branch_outages.len(), 3);
        assert_eq!(
            report
                .branch_outages
                .iter()
                .map(|scenario| scenario.outage.as_ref().unwrap().branch_index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(report.summary.expected_branch_outages, 3);
        assert_eq!(report.summary.evaluated_branch_outages, 3);
        assert!(report.summary.coverage_complete);
    }

    #[test]
    fn overloads_are_audited_violations_not_silently_counted_as_passes() {
        let report = sweep_fixed_injection_single_branch_outages(
            "triangle-overload-panel-v0.1",
            &triangle(),
            SweepOptions::default(),
        )
        .expect("bounded sweep");

        assert_eq!(report.summary.audited_with_declared_violations, 3);
        assert_eq!(report.summary.audited_within_declared_checks, 0);
        assert_eq!(report.summary.solver_unknowns, 0);
        assert_eq!(report.summary.auditor_rejections, 0);
        for scenario in &report.branch_outages {
            assert!(matches!(
                &scenario.disposition,
                ScenarioDisposition::AuditedWithDeclaredViolations { .. }
            ));
        }
    }

    #[test]
    fn island_without_a_reference_is_an_explicit_unknown() {
        let network = DcNetwork {
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
                DcBus {
                    id: 3,
                    specified_injection_mw: Some(-30.0),
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
                    reactance_pu: 0.1,
                    tap_ratio: 0.0,
                    phase_shift_degrees: 0.0,
                    rate_a_mva: None,
                    in_service: true,
                },
            ],
        };

        let report = sweep_fixed_injection_single_branch_outages(
            "bridge-islanding-negative-control-v0.1",
            &network,
            SweepOptions::default(),
        )
        .expect("complete sweep with unsupported outcomes");

        assert_eq!(report.summary.evaluated_branch_outages, 2);
        assert_eq!(report.summary.solver_unknowns, 2);
        assert_eq!(report.summary.audited_within_declared_checks, 0);
        assert_eq!(report.summary.audited_with_declared_violations, 0);
        assert!(
            report
                .branch_outages
                .iter()
                .all(|scenario| scenario.disposition.is_unknown())
        );
    }

    #[test]
    fn out_of_service_branches_are_not_outaged_twice() {
        let mut network = triangle();
        network.branches.push(DcBranch {
            from_bus: 1,
            to_bus: 3,
            reactance_pu: 0.3,
            tap_ratio: 0.0,
            phase_shift_degrees: 0.0,
            rate_a_mva: None,
            in_service: false,
        });
        let report = sweep_fixed_injection_single_branch_outages(
            "out-of-service-exclusion-v0.1",
            &network,
            SweepOptions::default(),
        )
        .expect("bounded sweep");

        assert_eq!(report.summary.expected_branch_outages, 3);
        assert_eq!(report.branch_outages.len(), 3);
    }

    #[test]
    fn exceeding_the_configured_budget_returns_no_partial_report() {
        let error = sweep_fixed_injection_single_branch_outages(
            "budget-negative-control-v0.1",
            &triangle(),
            SweepOptions {
                solve_options: SolveOptions::default(),
                max_contingencies: 2,
            },
        )
        .expect_err("three outages exceed a two-outage budget");

        assert_eq!(
            error,
            SweepError::ContingencyBudgetExceeded {
                required: 3,
                configured: 2,
            }
        );
    }

    #[test]
    fn source_network_is_never_mutated() {
        let network = triangle();
        let original_status = network
            .branches
            .iter()
            .map(|branch| branch.in_service)
            .collect::<Vec<_>>();
        let _report = sweep_fixed_injection_single_branch_outages(
            "immutability-negative-control-v0.1",
            &network,
            SweepOptions::default(),
        )
        .expect("bounded sweep");
        assert_eq!(
            network
                .branches
                .iter()
                .map(|branch| branch.in_service)
                .collect::<Vec<_>>(),
            original_status
        );
    }

    #[test]
    fn json_receipt_is_deterministic_and_escapes_identity_labels() {
        let report = sweep_fixed_injection_single_branch_outages(
            "case \"quoted\"\\path\nline",
            &triangle(),
            SweepOptions::default(),
        )
        .expect("bounded sweep");

        let first = report.to_json();
        assert_eq!(first, report.to_json());
        assert!(first.contains("case \\\"quoted\\\"\\\\path\\nline"));
        assert!(first.contains("audited_with_declared_violations"));
        assert!(first.contains("coverage_complete"));
        assert!(first.ends_with("}"));
    }

    #[test]
    fn empty_identity_label_is_rejected() {
        assert_eq!(
            sweep_fixed_injection_single_branch_outages("  ", &triangle(), SweepOptions::default())
                .expect_err("empty source label must fail closed"),
            SweepError::EmptyNetworkLabel
        );
    }
}
