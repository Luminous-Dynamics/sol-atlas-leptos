// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Independent numerical KKT checks for the MATPOWER 8.1 makeBdc DC-OPF profile.
//!
//! This verifier consumes MATPOWER's primal and named dual output, reconstructs
//! the admitted equations from the original MATPOWER source, and checks dual
//! signs, stationarity and complementary slackness independently of the solver.
//! A floating-point pass is numerical evidence, not an exact-arithmetic proof.

use std::collections::{BTreeMap, BTreeSet};

use matpower_dc_opf_primal_auditor::{
    AuditOptions as PrimalAuditOptions, audit_matpower_dc_opf_candidate, parse_candidate_tsv,
};
use pglib_case_auditor::{MatpowerDcProjection, project_matpower_dc_case};

const MODEL_PROFILE_ID: &str = "matpower_8_1_makeBdc_dcopf";
const DUAL_SCHEMA_VERSION: &str = "1";
const DUAL_CONVENTION: &str = "lower_upper_nonnegative";
const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundDual {
    /// MATPOWER's non-negative multiplier for the lower bound.
    pub lower: f64,
    /// MATPOWER's non-negative multiplier for the upper bound.
    pub upper: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KktOptions {
    pub equality_residual_tolerance_pu: f64,
    pub flow_violation_tolerance_pu: f64,
    pub angle_violation_tolerance_rad: f64,
    pub generator_bound_tolerance_pu: f64,
    pub angle_reference_tolerance_rad: f64,
    pub dual_feasibility_tolerance: f64,
    pub stationarity_absolute_tolerance: f64,
    pub stationarity_relative_tolerance: f64,
    pub complementarity_tolerance: f64,
}

impl Default for KktOptions {
    fn default() -> Self {
        Self {
            equality_residual_tolerance_pu: 1.0e-7,
            flow_violation_tolerance_pu: 1.0e-7,
            angle_violation_tolerance_rad: 1.0e-8,
            generator_bound_tolerance_pu: 1.0e-7,
            angle_reference_tolerance_rad: 1.0e-8,
            dual_feasibility_tolerance: 1.0e-8,
            stationarity_absolute_tolerance: 1.0e-4,
            stationarity_relative_tolerance: 1.0e-8,
            complementarity_tolerance: 1.0e-4,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KktReport {
    pub schema_version: &'static str,
    pub model_profile_id: &'static str,
    pub source_git_blob_sha1: String,
    pub primal_audit_passed: bool,
    pub model_convex: bool,
    pub dual_coverage_complete: bool,
    pub primal_feasible_under_reconstructed_model: bool,
    pub dual_feasible: bool,
    pub stationarity_within_tolerance: bool,
    pub complementary_slackness_within_tolerance: bool,
    pub numerical_kkt_satisfied: bool,
    pub formal_global_optimality_proof: bool,
    pub bus_count: usize,
    pub online_generator_count: usize,
    pub online_branch_count: usize,
    pub flow_constraint_count: usize,
    pub angle_constraint_count: usize,
    pub max_pmis_residual_pu: f64,
    pub max_flow_violation_pu: f64,
    pub max_angle_violation_rad: f64,
    pub max_generator_bound_violation_pu: f64,
    pub max_reference_angle_violation_rad: f64,
    pub max_dual_sign_violation: f64,
    pub max_stationarity_va: f64,
    pub max_stationarity_pg: f64,
    pub max_complementarity_product: f64,
    pub sum_complementarity_products: f64,
    pub primal_objective: f64,
    pub lagrangian_at_primal: f64,
    pub numerical_dual_infimum_estimate: Option<f64>,
    pub numerical_primal_dual_gap_estimate: Option<f64>,
    pub numerical_gap_is_rigorous_lower_bound: bool,
    pub status: &'static str,
    pub claim_ceiling: &'static str,
}

impl KktReport {
    #[must_use]
    pub fn to_json(&self) -> String {
        let optional_number = |value: Option<f64>| match value {
            Some(number) if number.is_finite() => number.to_string(),
            _ => "null".to_owned(),
        };
        format!(
            concat!(
                "{{",
                "\"schema_version\":{},",
                "\"model_profile_id\":{},",
                "\"source_git_blob_sha1\":{},",
                "\"primal_audit_passed\":{},",
                "\"model_convex\":{},",
                "\"dual_coverage_complete\":{},",
                "\"primal_feasible_under_reconstructed_model\":{},",
                "\"dual_feasible\":{},",
                "\"stationarity_within_tolerance\":{},",
                "\"complementary_slackness_within_tolerance\":{},",
                "\"numerical_kkt_satisfied\":{},",
                "\"formal_global_optimality_proof\":{},",
                "\"bus_count\":{},",
                "\"online_generator_count\":{},",
                "\"online_branch_count\":{},",
                "\"flow_constraint_count\":{},",
                "\"angle_constraint_count\":{},",
                "\"max_pmis_residual_pu\":{},",
                "\"max_flow_violation_pu\":{},",
                "\"max_angle_violation_rad\":{},",
                "\"max_generator_bound_violation_pu\":{},",
                "\"max_reference_angle_violation_rad\":{},",
                "\"max_dual_sign_violation\":{},",
                "\"max_stationarity_va\":{},",
                "\"max_stationarity_pg\":{},",
                "\"max_complementarity_product\":{},",
                "\"sum_complementarity_products\":{},",
                "\"primal_objective\":{},",
                "\"lagrangian_at_primal\":{},",
                "\"numerical_dual_infimum_estimate\":{},",
                "\"numerical_primal_dual_gap_estimate\":{},",
                "\"numerical_gap_is_rigorous_lower_bound\":{},",
                "\"status\":{},",
                "\"claim_ceiling\":{}",
                "}}"
            ),
            quote(self.schema_version),
            quote(self.model_profile_id),
            quote(&self.source_git_blob_sha1),
            self.primal_audit_passed,
            self.model_convex,
            self.dual_coverage_complete,
            self.primal_feasible_under_reconstructed_model,
            self.dual_feasible,
            self.stationarity_within_tolerance,
            self.complementary_slackness_within_tolerance,
            self.numerical_kkt_satisfied,
            self.formal_global_optimality_proof,
            self.bus_count,
            self.online_generator_count,
            self.online_branch_count,
            self.flow_constraint_count,
            self.angle_constraint_count,
            number(self.max_pmis_residual_pu),
            number(self.max_flow_violation_pu),
            number(self.max_angle_violation_rad),
            number(self.max_generator_bound_violation_pu),
            number(self.max_reference_angle_violation_rad),
            number(self.max_dual_sign_violation),
            number(self.max_stationarity_va),
            number(self.max_stationarity_pg),
            number(self.max_complementarity_product),
            number(self.sum_complementarity_products),
            number(self.primal_objective),
            number(self.lagrangian_at_primal),
            optional_number(self.numerical_dual_infimum_estimate),
            optional_number(self.numerical_primal_dual_gap_estimate),
            self.numerical_gap_is_rigorous_lower_bound,
            quote(self.status),
            quote(self.claim_ceiling)
        )
    }
}

#[derive(Clone, Debug, Default)]
struct DualCertificate {
    buses: BTreeMap<i64, BoundDual>,
    va: BTreeMap<i64, BoundDual>,
    generators: BTreeMap<usize, BoundDual>,
    flows: BTreeMap<usize, BoundDual>,
    angles: BTreeMap<usize, BoundDual>,
}

#[derive(Clone, Copy)]
struct BranchModel {
    source_row: usize,
    from_index: usize,
    to_index: usize,
    susceptance: f64,
    phase_injection_pu: f64,
    rate_lower_pu: Option<f64>,
    rate_upper_pu: Option<f64>,
    angle_lower_rad: Option<f64>,
    angle_upper_rad: Option<f64>,
}

#[derive(Clone, Copy)]
struct GeneratorModel {
    source_row: usize,
    bus_index: usize,
    pg_pu: f64,
    lower_pu: f64,
    upper_pu: f64,
    quadratic_cost_pu: f64,
    linear_cost_pu: f64,
    constant_cost: f64,
}

struct Metrics {
    max_pmis: f64,
    max_flow_violation: f64,
    max_angle_violation: f64,
    max_generator_bound_violation: f64,
    max_reference_angle_violation: f64,
    max_dual_sign_violation: f64,
    max_stationarity_va: f64,
    max_stationarity_pg: f64,
    max_complementarity_product: f64,
    sum_complementarity_products: f64,
    primal_feasible: bool,
    dual_feasible: bool,
    stationarity_ok: bool,
    complementarity_ok: bool,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            primal_feasible: true,
            dual_feasible: true,
            stationarity_ok: true,
            complementarity_ok: true,
            ..Self::default_values()
        }
    }
}

impl Metrics {
    fn default_values() -> Self {
        Self {
            max_pmis: 0.0,
            max_flow_violation: 0.0,
            max_angle_violation: 0.0,
            max_generator_bound_violation: 0.0,
            max_reference_angle_violation: 0.0,
            max_dual_sign_violation: 0.0,
            max_stationarity_va: 0.0,
            max_stationarity_pg: 0.0,
            max_complementarity_product: 0.0,
            sum_complementarity_products: 0.0,
            primal_feasible: true,
            dual_feasible: true,
            stationarity_ok: true,
            complementarity_ok: true,
        }
    }

    fn observe_complementarity(&mut self, product: f64, tolerance: f64) {
        let magnitude = product.abs();
        self.max_complementarity_product = self.max_complementarity_product.max(magnitude);
        self.sum_complementarity_products += magnitude;
        if !magnitude.is_finite() || magnitude > tolerance {
            self.complementarity_ok = false;
        }
    }

    fn observe_dual(&mut self, dual: BoundDual, tolerance: f64) {
        for value in [dual.lower, dual.upper] {
            let violation = (-value).max(0.0);
            self.max_dual_sign_violation = self.max_dual_sign_violation.max(violation);
            if !value.is_finite() || violation > tolerance {
                self.dual_feasible = false;
            }
        }
    }
}

/// Parse a MATPOWER receipt containing the primal rows plus DUAL_* rows.
fn split_primal_and_dual(input: &str) -> Result<(String, DualCertificate), String> {
    let mut primal_lines = Vec::new();
    let mut duals = DualCertificate::default();
    let mut meta_seen = false;

    for (index, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        let line_number = index + 1;
        match fields.first().copied() {
            Some("DUAL_META") if fields.len() == 3 => {
                if meta_seen {
                    return Err(format!("duplicate DUAL_META at line {line_number}"));
                }
                if fields[1] != DUAL_SCHEMA_VERSION || fields[2] != DUAL_CONVENTION {
                    return Err(format!(
                        "unsupported dual schema/convention at line {line_number}"
                    ));
                }
                meta_seen = true;
            }
            Some("DUAL_BUS") if fields.len() == 4 => {
                insert_dual(
                    &mut duals.buses,
                    parse_i64(fields[1], "dual bus ID")?,
                    parse_dual(fields[2], fields[3], line_number)?,
                    line_number,
                    "bus",
                )?;
            }
            Some("DUAL_VA") if fields.len() == 4 => {
                insert_dual(
                    &mut duals.va,
                    parse_i64(fields[1], "dual voltage-angle bus ID")?,
                    parse_dual(fields[2], fields[3], line_number)?,
                    line_number,
                    "voltage-angle",
                )?;
            }
            Some("DUAL_GEN") if fields.len() == 4 => {
                insert_dual(
                    &mut duals.generators,
                    parse_source_row(fields[1], "dual generator source row")?,
                    parse_dual(fields[2], fields[3], line_number)?,
                    line_number,
                    "generator",
                )?;
            }
            Some("DUAL_FLOW") if fields.len() == 4 => {
                insert_dual(
                    &mut duals.flows,
                    parse_source_row(fields[1], "dual flow branch source row")?,
                    parse_dual(fields[2], fields[3], line_number)?,
                    line_number,
                    "flow branch",
                )?;
            }
            Some("DUAL_ANGLE") if fields.len() == 4 => {
                insert_dual(
                    &mut duals.angles,
                    parse_source_row(fields[1], "dual angle branch source row")?,
                    parse_dual(fields[2], fields[3], line_number)?,
                    line_number,
                    "angle branch",
                )?;
            }
            Some(kind) if kind.starts_with("DUAL_") => {
                return Err(format!(
                    "malformed or unknown dual row '{kind}' at line {line_number}"
                ));
            }
            _ => primal_lines.push(line),
        }
    }

    if !meta_seen {
        return Err("candidate receipt is missing DUAL_META".to_owned());
    }
    Ok((primal_lines.join("\n"), duals))
}

fn insert_dual<K: Ord + std::fmt::Display>(
    target: &mut BTreeMap<K, BoundDual>,
    key: K,
    dual: BoundDual,
    line_number: usize,
    family: &str,
) -> Result<(), String> {
    if target.insert(key, dual).is_some() {
        return Err(format!(
            "duplicate {family} dual identity at line {line_number}"
        ));
    }
    Ok(())
}

fn parse_dual(lower: &str, upper: &str, line: usize) -> Result<BoundDual, String> {
    Ok(BoundDual {
        lower: parse_finite(lower, &format!("lower dual at line {line}"))?,
        upper: parse_finite(upper, &format!("upper dual at line {line}"))?,
    })
}

fn parse_finite(value: &str, context: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|error| format!("invalid {context} '{value}': {error}"))?;
    if !parsed.is_finite() {
        return Err(format!("non-finite {context}"));
    }
    Ok(parsed)
}

fn parse_i64(value: &str, context: &str) -> Result<i64, String> {
    value
        .parse::<i64>()
        .map_err(|error| format!("invalid {context} '{value}': {error}"))
}

fn parse_source_row(value: &str, context: &str) -> Result<usize, String> {
    let row = value
        .parse::<usize>()
        .map_err(|error| format!("invalid {context} '{value}': {error}"))?;
    if row == 0 {
        return Err(format!("{context} must be one-based"));
    }
    Ok(row)
}

fn quote(value: &str) -> String {
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
            other => output.push(other),
        }
    }
    output.push('"');
    output
}

fn number(value: f64) -> String {
    if value.is_finite() {
        if value == 0.0 {
            "0".to_owned()
        } else {
            value.to_string()
        }
    } else {
        "null".to_owned()
    }
}

fn angle_limit_is_present(lower_deg: f64, upper_deg: f64) -> bool {
    let lower_nonzero = lower_deg != 0.0;
    let upper_nonzero = upper_deg != 0.0;
    (lower_nonzero && lower_deg > -360.0)
        || (upper_nonzero && upper_deg < 360.0)
        || (lower_nonzero && !upper_nonzero)
        || (!lower_nonzero && upper_nonzero)
}

fn exact_key_coverage<K: Ord + Clone + std::fmt::Display>(
    family: &str,
    expected: &BTreeSet<K>,
    actual: &BTreeMap<K, BoundDual>,
) -> Result<(), String> {
    let actual_keys = actual.keys().cloned().collect::<BTreeSet<_>>();
    if &actual_keys != expected {
        let missing = expected
            .difference(&actual_keys)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let extra = actual_keys
            .difference(expected)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        return Err(format!(
            "{family} dual coverage mismatch; missing={missing:?}, extra={extra:?}"
        ));
    }
    Ok(())
}

fn validate_options(options: KktOptions) -> Result<(), String> {
    let values = [
        options.equality_residual_tolerance_pu,
        options.flow_violation_tolerance_pu,
        options.angle_violation_tolerance_rad,
        options.generator_bound_tolerance_pu,
        options.angle_reference_tolerance_rad,
        options.dual_feasibility_tolerance,
        options.stationarity_absolute_tolerance,
        options.stationarity_relative_tolerance,
        options.complementarity_tolerance,
    ];
    if values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err("all KKT tolerances must be finite and non-negative".to_owned());
    }
    Ok(())
}

/// Independently recompute MATPOWER 8.1 DC-OPF KKT conditions.
///
/// The solver's dual values are input observations. Their identity coverage,
/// non-negativity, stationarity and complementary slackness are independently
/// reconstructed from the original case source. No optimizer or solver status
/// is called by this function.
pub fn audit_matpower_dc_opf_kkt(
    source: &str,
    receipt_tsv: &str,
    primal_options: PrimalAuditOptions,
    kkt_options: KktOptions,
) -> Result<KktReport, String> {
    validate_options(kkt_options)?;
    let (primal_tsv, duals) = split_primal_and_dual(receipt_tsv)?;

    // Keep the existing independent primal auditor as a separate prerequisite.
    let primal = audit_matpower_dc_opf_candidate(source, &primal_tsv, primal_options)?;
    let candidate = parse_candidate_tsv(&primal_tsv)?;
    let projection = project_matpower_dc_case(source)
        .map_err(|error| format!("MATPOWER source projection: {error}"))?;

    audit_projection_kkt(&projection, &candidate, &duals, &primal, kkt_options)
}

fn audit_projection_kkt(
    projection: &MatpowerDcProjection,
    candidate: &matpower_dc_opf_primal_auditor::MatpowerDcOpfCandidate,
    duals: &DualCertificate,
    primal: &matpower_dc_opf_primal_auditor::AuditReport,
    options: KktOptions,
) -> Result<KktReport, String> {
    if projection.matpower_version != "8.1" {
        return Err(format!(
            "KKT profile requires MATPOWER 8.1, got {}",
            projection.matpower_version
        ));
    }
    if candidate.model_profile_id != MODEL_PROFILE_ID {
        return Err(format!(
            "unsupported model profile {}; expected {}",
            candidate.model_profile_id, MODEL_PROFILE_ID
        ));
    }
    if !primal.model_profile_matches || !primal.source_identity_matches {
        return Err("primal receipt source/profile identity does not match the source".to_owned());
    }

    let base = projection.base_mva;
    if !base.is_finite() || base <= 0.0 {
        return Err("source base MVA must be finite and positive".to_owned());
    }

    let bus_indices = projection
        .buses
        .iter()
        .enumerate()
        .map(|(index, bus)| (bus.id, index))
        .collect::<BTreeMap<_, _>>();
    if bus_indices.len() != projection.buses.len() {
        return Err("source projection has duplicate bus identities".to_owned());
    }
    let candidate_buses = &candidate.buses;
    let expected_bus_ids = projection
        .buses
        .iter()
        .map(|bus| bus.id)
        .collect::<BTreeSet<_>>();
    exact_key_coverage("Pmis", &expected_bus_ids, &duals.buses)?;
    exact_key_coverage("Va bound", &expected_bus_ids, &duals.va)?;

    let mut va = vec![0.0; projection.buses.len()];
    for (index, bus) in projection.buses.iter().enumerate() {
        let state = candidate_buses
            .get(&bus.id)
            .ok_or_else(|| format!("candidate is missing bus {}", bus.id))?;
        va[index] = state.voltage_angle_degrees * DEG_TO_RAD;
        if !va[index].is_finite() {
            return Err(format!("candidate angle is non-finite at bus {}", bus.id));
        }
    }

    let candidate_generators = candidate
        .generators
        .iter()
        .map(|generator| (generator.source_row, generator))
        .collect::<BTreeMap<_, _>>();
    let cost_by_row = projection
        .costs
        .iter()
        .map(|cost| (cost.source_row, cost))
        .collect::<BTreeMap<_, _>>();
    let mut generator_models = Vec::new();
    let mut expected_generator_rows = BTreeSet::new();
    for generator in &projection.generators {
        if !generator.in_service {
            continue;
        }
        let state = candidate_generators
            .get(&generator.source_row)
            .ok_or_else(|| {
                format!(
                    "candidate is missing online generator row {}",
                    generator.source_row
                )
            })?;
        if !state.in_service || state.bus_id != generator.bus_id {
            return Err(format!(
                "candidate generator identity/status mismatch at source row {}",
                generator.source_row
            ));
        }
        let bus_index = *bus_indices
            .get(&generator.bus_id)
            .ok_or_else(|| format!("generator references unknown bus {}", generator.bus_id))?;
        let cost = cost_by_row
            .get(&generator.source_row)
            .ok_or_else(|| format!("missing cost row for generator {}", generator.source_row))?;
        let (quadratic, linear, constant) = cost_triplet(cost)?;
        if quadratic < 0.0 {
            return Err(format!(
                "non-convex quadratic cost at generator row {}",
                generator.source_row
            ));
        }
        let pg_pu = state.pg_mw / base;
        generator_models.push(GeneratorModel {
            source_row: generator.source_row,
            bus_index,
            pg_pu,
            lower_pu: generator.p_min_mw / base,
            upper_pu: generator.p_max_mw / base,
            quadratic_cost_pu: quadratic * base * base,
            linear_cost_pu: linear * base,
            constant_cost: constant,
        });
        expected_generator_rows.insert(generator.source_row);
    }
    exact_key_coverage("Pg bound", &expected_generator_rows, &duals.generators)?;

    let mut branch_models = Vec::new();
    let mut expected_flow_rows = BTreeSet::new();
    let mut expected_angle_rows = BTreeSet::new();
    for branch in &projection.branches {
        if !branch.in_service {
            continue;
        }
        let from_index = *bus_indices
            .get(&branch.from_bus)
            .ok_or_else(|| format!("branch {} has unknown from-bus", branch.source_row))?;
        let to_index = *bus_indices
            .get(&branch.to_bus)
            .ok_or_else(|| format!("branch {} has unknown to-bus", branch.source_row))?;
        if !branch.reactance_pu.is_finite() || branch.reactance_pu.abs() <= f64::EPSILON {
            return Err(format!(
                "branch {} has invalid reactance",
                branch.source_row
            ));
        }
        let effective_tap = if branch.tap_ratio == 0.0 {
            1.0
        } else {
            branch.tap_ratio
        };
        if !effective_tap.is_finite() || effective_tap <= 0.0 {
            return Err(format!(
                "branch {} has invalid tap ratio",
                branch.source_row
            ));
        }
        let susceptance = 1.0 / (branch.reactance_pu * effective_tap);
        let phase_injection_pu = -susceptance * branch.phase_shift_degrees * DEG_TO_RAD;

        let has_rate_limit = branch.rate_a_mva != 0.0 && branch.rate_a_mva < 1.0e10;
        if has_rate_limit && branch.rate_a_mva <= 0.0 {
            return Err(format!("branch {} has invalid Rate A", branch.source_row));
        }
        let (rate_lower_pu, rate_upper_pu) = if has_rate_limit {
            expected_flow_rows.insert(branch.source_row);
            (
                Some(-branch.rate_a_mva / base - phase_injection_pu),
                Some(branch.rate_a_mva / base - phase_injection_pu),
            )
        } else {
            (None, None)
        };

        let has_angle_limit =
            angle_limit_is_present(branch.angle_min_degrees, branch.angle_max_degrees);
        let (angle_lower_rad, angle_upper_rad) = if has_angle_limit {
            expected_angle_rows.insert(branch.source_row);
            (
                Some(if branch.angle_min_degrees < -360.0 {
                    f64::NEG_INFINITY
                } else {
                    branch.angle_min_degrees * DEG_TO_RAD
                }),
                Some(if branch.angle_max_degrees > 360.0 {
                    f64::INFINITY
                } else {
                    branch.angle_max_degrees * DEG_TO_RAD
                }),
            )
        } else {
            (None, None)
        };

        branch_models.push(BranchModel {
            source_row: branch.source_row,
            from_index,
            to_index,
            susceptance,
            phase_injection_pu,
            rate_lower_pu,
            rate_upper_pu,
            angle_lower_rad,
            angle_upper_rad,
        });
    }
    exact_key_coverage("Pf", &expected_flow_rows, &duals.flows)?;
    exact_key_coverage("angle", &expected_angle_rows, &duals.angles)?;

    let mut metrics = Metrics::default();
    let mut pg_by_bus = vec![0.0; projection.buses.len()];
    for generator in &generator_models {
        pg_by_bus[generator.bus_index] += generator.pg_pu;
    }

    // Reconstruct Pmis and the source-derived phase-shift injection.
    let mut b_va = vec![0.0; projection.buses.len()];
    let mut pbus_injection = vec![0.0; projection.buses.len()];
    for branch in &branch_models {
        let angle_difference = va[branch.from_index] - va[branch.to_index];
        let branch_flow_without_phase = branch.susceptance * angle_difference;
        b_va[branch.from_index] += branch_flow_without_phase;
        b_va[branch.to_index] -= branch_flow_without_phase;
        pbus_injection[branch.from_index] += branch.phase_injection_pu;
        pbus_injection[branch.to_index] -= branch.phase_injection_pu;
    }

    let mut bmis = vec![0.0; projection.buses.len()];
    for (index, bus) in projection.buses.iter().enumerate() {
        bmis[index] = -(bus.demand_mw + bus.shunt_conductance_mw) / base - pbus_injection[index];
        let residual = b_va[index] - pg_by_bus[index] - bmis[index];
        metrics.max_pmis = metrics.max_pmis.max(residual.abs());
        if !residual.is_finite() || residual.abs() > options.equality_residual_tolerance_pu {
            metrics.primal_feasible = false;
        }
    }

    // Reconstruct network inequalities, angle inequalities and their slacks.
    for branch in &branch_models {
        let angle_difference = va[branch.from_index] - va[branch.to_index];
        let flow_expression = branch.susceptance * angle_difference;

        if let (Some(lower), Some(upper)) = (branch.rate_lower_pu, branch.rate_upper_pu) {
            let dual = duals.flows[&branch.source_row];
            metrics.observe_dual(dual, options.dual_feasibility_tolerance);
            let lower_slack = flow_expression - lower;
            let upper_slack = upper - flow_expression;
            let violation = (-lower_slack).max(0.0).max((-upper_slack).max(0.0));
            metrics.max_flow_violation = metrics.max_flow_violation.max(violation);
            if !violation.is_finite() || violation > options.flow_violation_tolerance_pu {
                metrics.primal_feasible = false;
            }
            metrics.observe_complementarity(
                dual.lower * lower_slack,
                options.complementarity_tolerance,
            );
            metrics.observe_complementarity(
                dual.upper * upper_slack,
                options.complementarity_tolerance,
            );
        }

        if let (Some(lower), Some(upper)) = (branch.angle_lower_rad, branch.angle_upper_rad) {
            let dual = duals.angles[&branch.source_row];
            metrics.observe_dual(dual, options.dual_feasibility_tolerance);
            let lower_slack = angle_difference - lower;
            let upper_slack = upper - angle_difference;
            let lower_violation = if lower.is_finite() {
                (-lower_slack).max(0.0)
            } else {
                0.0
            };
            let upper_violation = if upper.is_finite() {
                (-upper_slack).max(0.0)
            } else {
                0.0
            };
            let violation = lower_violation.max(upper_violation);
            metrics.max_angle_violation = metrics.max_angle_violation.max(violation);
            if !violation.is_finite() || violation > options.angle_violation_tolerance_rad {
                metrics.primal_feasible = false;
            }
            if lower.is_finite() {
                metrics.observe_complementarity(
                    dual.lower * lower_slack,
                    options.complementarity_tolerance,
                );
            } else if dual.lower.abs() > options.dual_feasibility_tolerance {
                metrics.dual_feasible = false;
            }
            if upper.is_finite() {
                metrics.observe_complementarity(
                    dual.upper * upper_slack,
                    options.complementarity_tolerance,
                );
            } else if dual.upper.abs() > options.dual_feasibility_tolerance {
                metrics.dual_feasible = false;
            }
        }
    }

    // Variable-bound feasibility and complementary slackness.
    for (index, bus) in projection.buses.iter().enumerate() {
        let dual = duals.va[&bus.id];
        metrics.observe_dual(dual, options.dual_feasibility_tolerance);
        if bus.bus_type == 3 {
            let reference = bus.voltage_angle_initial_degrees * DEG_TO_RAD;
            let violation = (va[index] - reference).abs();
            metrics.max_reference_angle_violation =
                metrics.max_reference_angle_violation.max(violation);
            if violation > options.angle_reference_tolerance_rad {
                metrics.primal_feasible = false;
            }
            metrics.observe_complementarity(
                dual.lower * (va[index] - reference),
                options.complementarity_tolerance,
            );
            metrics.observe_complementarity(
                dual.upper * (reference - va[index]),
                options.complementarity_tolerance,
            );
        } else if dual.lower.abs() > options.dual_feasibility_tolerance
            || dual.upper.abs() > options.dual_feasibility_tolerance
        {
            // Non-reference Va variables have no finite MATPOWER variable bounds.
            metrics.dual_feasible = false;
        }
    }

    for generator in &generator_models {
        let dual = duals.generators[&generator.source_row];
        metrics.observe_dual(dual, options.dual_feasibility_tolerance);
        let lower_slack = generator.pg_pu - generator.lower_pu;
        let upper_slack = generator.upper_pu - generator.pg_pu;
        let violation = (-lower_slack).max(0.0).max((-upper_slack).max(0.0));
        metrics.max_generator_bound_violation =
            metrics.max_generator_bound_violation.max(violation);
        if !violation.is_finite() || violation > options.generator_bound_tolerance_pu {
            metrics.primal_feasible = false;
        }
        metrics
            .observe_complementarity(dual.lower * lower_slack, options.complementarity_tolerance);
        metrics
            .observe_complementarity(dual.upper * upper_slack, options.complementarity_tolerance);
    }

    // The equality Pmis is represented by identical lower/upper bounds.
    for dual in duals.buses.values().copied() {
        metrics.observe_dual(dual, options.dual_feasibility_tolerance);
    }

    // Reconstruct stationarity in the actual MATPOWER variables:
    // Va in radians, Pg in per-unit, objective in MATPOWER's cost units.
    let mut stationarity_va = vec![0.0; projection.buses.len()];
    let mut stationarity_va_scale = vec![1.0; projection.buses.len()];
    let mut stationarity_pg = Vec::with_capacity(generator_models.len());
    let mut stationarity_pg_scale = Vec::with_capacity(generator_models.len());
    let mut pg_linear_for_dual = Vec::with_capacity(generator_models.len());

    for generator in &generator_models {
        let dual = duals.generators[&generator.source_row];
        let eq_dual = duals.buses[&projection.buses[generator.bus_index].id];
        let signed_eq = eq_dual.upper - eq_dual.lower;
        let signed_bound = dual.upper - dual.lower;
        let linear = generator.linear_cost_pu - signed_eq + signed_bound;
        let gradient = 2.0 * generator.quadratic_cost_pu * generator.pg_pu + linear;
        let scale = 1.0
            + (2.0 * generator.quadratic_cost_pu * generator.pg_pu).abs()
            + generator.linear_cost_pu.abs()
            + signed_eq.abs()
            + signed_bound.abs();
        stationarity_pg.push(gradient);
        stationarity_pg_scale.push(scale);
        pg_linear_for_dual.push(linear);
    }

    // Pmis Jacobian is Bbus for Va and -Cg for Pg. Assemble Bbus^T * lambda
    // branch-by-branch to avoid a dense bus-by-bus matrix.
    let signed_pmis = projection
        .buses
        .iter()
        .map(|bus| duals.buses[&bus.id].upper - duals.buses[&bus.id].lower)
        .collect::<Vec<_>>();
    for branch in &branch_models {
        let delta =
            branch.susceptance * (signed_pmis[branch.from_index] - signed_pmis[branch.to_index]);
        stationarity_va[branch.from_index] += delta;
        stationarity_va[branch.to_index] -= delta;
        stationarity_va_scale[branch.from_index] += delta.abs();
        stationarity_va_scale[branch.to_index] += delta.abs();
    }

    // Add every active Pf and angle-difference inequality Jacobian.
    for branch in &branch_models {
        if branch.rate_lower_pu.is_some() {
            let dual = duals.flows[&branch.source_row];
            let signed = dual.upper - dual.lower;
            let contribution = branch.susceptance * signed;
            stationarity_va[branch.from_index] += contribution;
            stationarity_va[branch.to_index] -= contribution;
            stationarity_va_scale[branch.from_index] += contribution.abs();
            stationarity_va_scale[branch.to_index] += contribution.abs();
        }
        if branch.angle_lower_rad.is_some() {
            let dual = duals.angles[&branch.source_row];
            let signed = dual.upper - dual.lower;
            stationarity_va[branch.from_index] += signed;
            stationarity_va[branch.to_index] -= signed;
            stationarity_va_scale[branch.from_index] += signed.abs();
            stationarity_va_scale[branch.to_index] += signed.abs();
        }
    }

    // Variable-bound Jacobians are +1 for the upper bound and -1 for the lower.
    for (index, bus) in projection.buses.iter().enumerate() {
        let dual = duals.va[&bus.id];
        let contribution = dual.upper - dual.lower;
        stationarity_va[index] += contribution;
        stationarity_va_scale[index] += contribution.abs();
    }

    let mut stationarity_ok = true;
    for (residual, scale) in stationarity_va.iter().zip(stationarity_va_scale.iter()) {
        metrics.max_stationarity_va = metrics.max_stationarity_va.max(residual.abs());
        let tolerance = options.stationarity_absolute_tolerance
            + options.stationarity_relative_tolerance * scale;
        if !residual.is_finite() || residual.abs() > tolerance {
            stationarity_ok = false;
        }
    }
    for (residual, scale) in stationarity_pg.iter().zip(stationarity_pg_scale.iter()) {
        metrics.max_stationarity_pg = metrics.max_stationarity_pg.max(residual.abs());
        let tolerance = options.stationarity_absolute_tolerance
            + options.stationarity_relative_tolerance * scale;
        if !residual.is_finite() || residual.abs() > tolerance {
            stationarity_ok = false;
        }
    }
    metrics.stationarity_ok = stationarity_ok;

    // Construct the dual function's numerical infimum estimate. For finite
    // precision, a small nonzero coefficient on a zero-curvature Va/linear-Pg
    // variable makes the exact infimum unbounded. We disclose this estimate as
    // non-rigorous and only emit it when the residual is within KKT tolerance.
    let mut dual_constant = 0.0;
    let mut primal_at_lagrangian = 0.0;
    for generator in &generator_models {
        dual_constant += generator.constant_cost;
        primal_at_lagrangian += generator.quadratic_cost_pu * generator.pg_pu * generator.pg_pu
            + generator.linear_cost_pu * generator.pg_pu
            + generator.constant_cost;
    }

    for (index, bus) in projection.buses.iter().enumerate() {
        let dual = duals.buses[&bus.id];
        let expression = b_va[index] - pg_by_bus[index];
        primal_at_lagrangian +=
            dual.lower * (bmis[index] - expression) + dual.upper * (expression - bmis[index]);
    }

    for (index, bus) in projection.buses.iter().enumerate() {
        let dual = duals.buses[&bus.id];
        dual_constant += (dual.lower - dual.upper) * bmis[index];
    }
    for branch in &branch_models {
        if let (Some(lower), Some(upper)) = (branch.rate_lower_pu, branch.rate_upper_pu) {
            let dual = duals.flows[&branch.source_row];
            dual_constant += dual.lower * lower - dual.upper * upper;
            let expression = branch.susceptance * (va[branch.from_index] - va[branch.to_index]);
            primal_at_lagrangian +=
                dual.lower * (lower - expression) + dual.upper * (expression - upper);
        }
        if let (Some(lower), Some(upper)) = (branch.angle_lower_rad, branch.angle_upper_rad) {
            let dual = duals.angles[&branch.source_row];
            if lower.is_finite() {
                dual_constant += dual.lower * lower;
            }
            if upper.is_finite() {
                dual_constant -= dual.upper * upper;
            }
            let expression = va[branch.from_index] - va[branch.to_index];
            if lower.is_finite() {
                primal_at_lagrangian += dual.lower * (lower - expression);
            }
            if upper.is_finite() {
                primal_at_lagrangian += dual.upper * (expression - upper);
            }
        }
    }

    for (index, bus) in projection.buses.iter().enumerate() {
        let dual = duals.va[&bus.id];
        if bus.bus_type == 3 {
            let reference = bus.voltage_angle_initial_degrees * DEG_TO_RAD;
            dual_constant += dual.lower * reference - dual.upper * reference;
            primal_at_lagrangian +=
                dual.lower * (reference - va[index]) + dual.upper * (va[index] - reference);
        }
    }
    for generator in &generator_models {
        let dual = duals.generators[&generator.source_row];
        dual_constant += dual.lower * generator.lower_pu - dual.upper * generator.upper_pu;
        primal_at_lagrangian += dual.lower * (generator.lower_pu - generator.pg_pu)
            + dual.upper * (generator.pg_pu - generator.upper_pu);
    }

    let mut dual_infimum = dual_constant;
    let mut dual_infimum_available = true;
    for (index, generator) in generator_models.iter().enumerate() {
        let q = generator.quadratic_cost_pu;
        let linear = pg_linear_for_dual[index];
        if q > 0.0 {
            dual_infimum -= linear * linear / (4.0 * q);
        } else if linear.abs()
            > options.stationarity_absolute_tolerance
                + options.stationarity_relative_tolerance * stationarity_pg_scale[index]
        {
            dual_infimum_available = false;
        }
    }
    for (residual, scale) in stationarity_va.iter().zip(stationarity_va_scale.iter()) {
        if residual.abs()
            > options.stationarity_absolute_tolerance
                + options.stationarity_relative_tolerance * scale
        {
            dual_infimum_available = false;
        }
    }

    let numerical_dual_infimum_estimate = if dual_infimum_available && dual_infimum.is_finite() {
        Some(dual_infimum)
    } else {
        None
    };
    let numerical_primal_dual_gap_estimate =
        numerical_dual_infimum_estimate.map(|dual| primal.recomputed_objective - dual);

    let stationarity_within_tolerance = metrics.stationarity_ok;
    let dual_feasible = metrics.dual_feasible;
    let primal_feasible = metrics.primal_feasible;
    let complementary_slackness_within_tolerance = metrics.complementarity_ok;
    let numerical_kkt_satisfied = primal.passed
        && primal_feasible
        && dual_feasible
        && stationarity_within_tolerance
        && complementary_slackness_within_tolerance;
    let status = if numerical_kkt_satisfied {
        "KktSatisfiedWithinTolerance"
    } else {
        "KktNotSatisfied"
    };

    Ok(KktReport {
        schema_version: "matpower-dc-opf-kkt.v0.1.0",
        model_profile_id: MODEL_PROFILE_ID,
        source_git_blob_sha1: projection.source_git_blob_sha1.clone(),
        primal_audit_passed: primal.passed,
        model_convex: true,
        dual_coverage_complete: true,
        primal_feasible_under_reconstructed_model: primal_feasible,
        dual_feasible,
        stationarity_within_tolerance,
        complementary_slackness_within_tolerance,
        numerical_kkt_satisfied,
        formal_global_optimality_proof: false,
        bus_count: projection.buses.len(),
        online_generator_count: generator_models.len(),
        online_branch_count: branch_models.len(),
        flow_constraint_count: expected_flow_rows.len(),
        angle_constraint_count: expected_angle_rows.len(),
        max_pmis_residual_pu: metrics.max_pmis,
        max_flow_violation_pu: metrics.max_flow_violation,
        max_angle_violation_rad: metrics.max_angle_violation,
        max_generator_bound_violation_pu: metrics.max_generator_bound_violation,
        max_reference_angle_violation_rad: metrics.max_reference_angle_violation,
        max_dual_sign_violation: metrics.max_dual_sign_violation,
        max_stationarity_va: metrics.max_stationarity_va,
        max_stationarity_pg: metrics.max_stationarity_pg,
        max_complementarity_product: metrics.max_complementarity_product,
        sum_complementarity_products: metrics.sum_complementarity_products,
        primal_objective: primal.recomputed_objective,
        lagrangian_at_primal: primal_at_lagrangian,
        numerical_dual_infimum_estimate,
        numerical_primal_dual_gap_estimate,
        numerical_gap_is_rigorous_lower_bound: false,
        status,
        claim_ceiling: "Numerical KKT verification within declared floating-point tolerances for the exact MATPOWER 8.1 convex DC-OPF model. This is not an exact-arithmetic proof, AC-OPF certificate, physical validation, security proof, or operational authorization.",
    })
}

fn cost_triplet(cost: &pglib_case_auditor::MatpowerDcCost) -> Result<(f64, f64, f64), String> {
    if cost.model != 2 {
        return Err(format!("unsupported generator cost model {}", cost.model));
    }
    match cost.coefficients.as_slice() {
        [constant] => Ok((0.0, 0.0, *constant)),
        [linear, constant] => Ok((0.0, *linear, *constant)),
        [quadratic, linear, constant] => Ok((*quadratic, *linear, *constant)),
        _ => Err(format!(
            "unsupported polynomial coefficient count {} at cost row {}",
            cost.coefficients.len(),
            cost.source_row
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_BUS: &str = include_str!("../../matpower-dc-opf-primal-auditor/fixtures/two_bus.m");

    fn valid_receipt(source: &str) -> String {
        let digest = pglib_case_auditor::git_blob_sha1(source.as_bytes());
        format!(
            concat!(
                "META\tsource_git_blob_sha1\t{}\n",
                "META\tmatpower_version\t8.1\n",
                "META\tmodel_profile_id\tmatpower_8_1_makeBdc_dcopf\n",
                "META\tbase_mva\t100\n",
                "META\tobjective\t2100\n",
                "BUS\t1\t0\n",
                "BUS\t2\t-5.729577951\n",
                "GEN\t1\t1\t1\t100\n",
                "BRANCH\t1\t1\t2\t1\t100\t-100\n",
                "DUAL_META\t1\tlower_upper_nonnegative\n",
                "DUAL_BUS\t1\t0\t2200\n",
                "DUAL_BUS\t2\t0\t2200\n",
                "DUAL_VA\t1\t0\t0\n",
                "DUAL_VA\t2\t0\t0\n",
                "DUAL_GEN\t1\t0\t0\n",
                "DUAL_FLOW\t1\t0\t0\n",
                "DUAL_ANGLE\t1\t0\t0\n"
            ),
            digest
        )
    }

    #[test]
    fn hand_calculated_two_bus_qp_satisfies_numerical_kkt_conditions() {
        let report = audit_matpower_dc_opf_kkt(
            TWO_BUS,
            &valid_receipt(TWO_BUS),
            PrimalAuditOptions::default(),
            KktOptions::default(),
        )
        .expect("two-bus KKT receipt should parse");
        assert!(report.primal_audit_passed, "{}", report.to_json());
        assert!(report.dual_feasible, "{}", report.to_json());
        assert!(report.stationarity_within_tolerance, "{}", report.to_json());
        assert!(
            report.complementary_slackness_within_tolerance,
            "{}",
            report.to_json()
        );
        assert!(report.numerical_kkt_satisfied, "{}", report.to_json());
        assert!(!report.formal_global_optimality_proof);
        assert!(!report.numerical_gap_is_rigorous_lower_bound);
        assert!((report.primal_objective - 2100.0).abs() < 1.0e-8);
        assert!(
            report
                .numerical_primal_dual_gap_estimate
                .is_some_and(|gap| gap.abs() < 1.0e-7)
        );
    }

    #[test]
    fn wrong_bus_multiplier_is_detected_by_stationarity() {
        let receipt =
            valid_receipt(TWO_BUS).replace("DUAL_BUS\t1\t0\t2200", "DUAL_BUS\t1\t0\t2100");
        let report = audit_matpower_dc_opf_kkt(
            TWO_BUS,
            &receipt,
            PrimalAuditOptions::default(),
            KktOptions::default(),
        )
        .expect("mutated dual should be auditable");
        assert!(!report.stationarity_within_tolerance);
        assert!(!report.numerical_kkt_satisfied);
    }

    #[test]
    fn negative_dual_is_not_accepted() {
        let receipt =
            valid_receipt(TWO_BUS).replace("DUAL_BUS\t1\t0\t2200", "DUAL_BUS\t1\t-0.1\t2200");
        let report = audit_matpower_dc_opf_kkt(
            TWO_BUS,
            &receipt,
            PrimalAuditOptions::default(),
            KktOptions::default(),
        )
        .expect("negative dual should produce an audit report");
        assert!(!report.dual_feasible);
        assert!(!report.numerical_kkt_satisfied);
    }

    #[test]
    fn missing_dual_identity_fails_closed() {
        let receipt = valid_receipt(TWO_BUS).replace("DUAL_VA\t2\t0\t0\n", "");
        let error = audit_matpower_dc_opf_kkt(
            TWO_BUS,
            &receipt,
            PrimalAuditOptions::default(),
            KktOptions::default(),
        )
        .expect_err("missing voltage-angle dual must be rejected");
        assert!(error.contains("Va bound dual coverage mismatch"));
    }

    const TWO_BUS_BINDING_LINE: &str = include_str!("../fixtures/two_bus_binding_line.m");

    fn valid_binding_line_receipt(source: &str) -> String {
        let digest = pglib_case_auditor::git_blob_sha1(source.as_bytes());
        format!(
            concat!(
                "META\\tsource_git_blob_sha1\\t{}\\n",
                "META\\tmatpower_version\\t8.1\\n",
                "META\\tmodel_profile_id\\tmatpower_8_1_makeBdc_dcopf\\n",
                "META\\tbase_mva\\t100\\n",
                "META\\tobjective\\t2200\\n",
                "BUS\\t1\\t0\\n",
                "BUS\\t2\\t-2.291831181\\n",
                "GEN\\t1\\t1\\t1\\t40\\n",
                "GEN\\t2\\t2\\t1\\t60\\n",
                "BRANCH\\t1\\t1\\t2\\t1\\t40\\t-40\\n",
                "DUAL_META\\t1\\tlower_upper_nonnegative\\n",
                "DUAL_BUS\\t1\\t0\\t1000\\n",
                "DUAL_BUS\\t2\\t0\\t3000\\n",
                "DUAL_VA\\t1\\t0\\t0\\n",
                "DUAL_VA\\t2\\t0\\t0\\n",
                "DUAL_GEN\\t1\\t0\\t0\\n",
                "DUAL_GEN\\t2\\t0\\t0\\n",
                "DUAL_FLOW\\t1\\t0\\t2000\\n",
                "DUAL_ANGLE\\t1\\t0\\t0\\n"
            ),
            digest
        )
    }

    #[test]
    fn binding_line_case_checks_nonzero_flow_dual_and_dispatch_stationarity() {
        let receipt = valid_binding_line_receipt(TWO_BUS_BINDING_LINE);
        let report = audit_matpower_dc_opf_kkt(
            TWO_BUS_BINDING_LINE,
            &receipt,
            PrimalAuditOptions::default(),
            KktOptions::default(),
        )
        .expect("binding-line KKT receipt should parse");

        assert!(report.primal_audit_passed, "{}", report.to_json());
        assert!(
            report.primal_feasible_under_reconstructed_model,
            "{}",
            report.to_json()
        );
        assert!(report.dual_feasible, "{}", report.to_json());
        assert!(
            report.complementary_slackness_within_tolerance,
            "{}",
            report.to_json()
        );
        assert!(report.numerical_kkt_satisfied, "{}", report.to_json());
        assert_eq!(report.flow_constraint_count, 1);
        assert!((report.primal_objective - 2200.0).abs() < 1.0e-8);
        assert!(
            report
                .numerical_primal_dual_gap_estimate
                .is_some_and(|gap| gap.abs() < 1.0e-6),
            "{}",
            report.to_json()
        );
    }

    #[test]
    fn wrong_binding_line_shadow_price_is_detected_by_stationarity() {
        let receipt = valid_binding_line_receipt(TWO_BUS_BINDING_LINE)
            .replace("DUAL_FLOW\\t1\\t0\\t2000", "DUAL_FLOW\\t1\\t0\\t1900");
        let report = audit_matpower_dc_opf_kkt(
            TWO_BUS_BINDING_LINE,
            &receipt,
            PrimalAuditOptions::default(),
            KktOptions::default(),
        )
        .expect("wrong line shadow price should produce an audit report");
        assert!(
            !report.stationarity_within_tolerance,
            "{}",
            report.to_json()
        );
        assert!(!report.numerical_kkt_satisfied, "{}", report.to_json());
    }

    #[test]
    fn upper_flow_multiplier_with_slack_violates_complementarity() {
        let receipt = valid_receipt(TWO_BUS).replace("DUAL_FLOW\t1\t0\t0", "DUAL_FLOW\t1\t0\t0.01");
        let report = audit_matpower_dc_opf_kkt(
            TWO_BUS,
            &receipt,
            PrimalAuditOptions::default(),
            KktOptions::default(),
        )
        .expect("complementarity mutation should produce a report");
        assert!(!report.complementary_slackness_within_tolerance);
        assert!(!report.numerical_kkt_satisfied);
    }
}
