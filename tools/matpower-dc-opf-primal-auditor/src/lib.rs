// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Independent primal-feasibility and objective audit for MATPOWER 8.1 DC-OPF.
//!
//! This crate does not optimize. It parses the original MATPOWER source through
//! the strict source-bound projection, parses a separately produced MATPOWER
//! candidate receipt, then independently reconstructs the stated MATPOWER DC
//! equations, generator costs, and declared operating limits.

use std::collections::BTreeMap;

use pglib_case_auditor::{MatpowerDcProjection, git_blob_sha1, project_matpower_dc_case};

pub const MATPOWER_VERSION: &str = "8.1";
pub const MODEL_PROFILE_ID: &str = "matpower_8_1_makeBdc_dcopf";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AuditOptions {
    pub power_tolerance_mw: f64,
    pub angle_tolerance_degrees: f64,
    pub objective_absolute_tolerance: f64,
    pub objective_relative_tolerance: f64,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            power_tolerance_mw: 1.0e-6,
            angle_tolerance_degrees: 1.0e-8,
            objective_absolute_tolerance: 1.0e-5,
            objective_relative_tolerance: 1.0e-9,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BusCandidate {
    pub bus_id: i64,
    pub voltage_angle_degrees: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratorCandidate {
    pub source_row: usize,
    pub bus_id: i64,
    pub in_service: bool,
    pub pg_mw: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BranchCandidate {
    pub source_row: usize,
    pub from_bus: i64,
    pub to_bus: i64,
    pub in_service: bool,
    pub pf_mw: f64,
    pub pt_mw: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatpowerDcOpfCandidate {
    pub source_git_blob_sha1: String,
    pub matpower_version: String,
    pub model_profile_id: String,
    pub base_mva: f64,
    pub objective: f64,
    pub buses: BTreeMap<i64, BusCandidate>,
    pub generators: Vec<GeneratorCandidate>,
    pub branches: Vec<BranchCandidate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuditViolation {
    pub family: String,
    pub identity: String,
    pub rule: String,
    pub observed_magnitude: f64,
    pub allowed_tolerance: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuditReport {
    pub source_git_blob_sha1: String,
    pub matpower_version: String,
    pub model_profile_id: String,
    pub bus_count: usize,
    pub generator_count: usize,
    pub branch_count: usize,
    pub field_comparison_count: usize,
    pub recomputed_objective: f64,
    pub reported_objective: f64,
    pub objective_absolute_error: f64,
    pub max_bus_balance_residual_mw: f64,
    pub max_branch_from_flow_error_mw: f64,
    pub max_branch_to_flow_error_mw: f64,
    pub max_generator_bound_violation_mw: f64,
    pub max_branch_rate_violation_mw: f64,
    pub max_branch_angle_violation_degrees: f64,
    pub source_identity_matches: bool,
    pub model_profile_matches: bool,
    pub primal_feasible: bool,
    pub objective_consistent: bool,
    pub passed: bool,
    pub violations: Vec<AuditViolation>,
    pub claim_ceiling: &'static str,
}

impl AuditReport {
    #[must_use]
    pub fn to_json(&self) -> String {
        let violations = self
            .violations
            .iter()
            .map(|violation| {
                format!(
                    concat!(
                        "{{\"family\":{},\"identity\":{},\"rule\":{},",
                        "\"observed_magnitude\":{},\"allowed_tolerance\":{}}}"
                    ),
                    quote(&violation.family),
                    quote(&violation.identity),
                    quote(&violation.rule),
                    number(violation.observed_magnitude),
                    number(violation.allowed_tolerance)
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!(
            concat!(
                "{{",
                "\"schema_version\":\"matpower-dc-opf-primal-audit.v0.1.0\",",
                "\"source_git_blob_sha1\":{},",
                "\"matpower_version\":{},\"model_profile_id\":{},",
                "\"bus_count\":{},\"generator_count\":{},\"branch_count\":{},",
                "\"field_comparison_count\":{},",
                "\"recomputed_objective\":{},\"reported_objective\":{},",
                "\"objective_absolute_error\":{},",
                "\"max_bus_balance_residual_mw\":{},",
                "\"max_branch_from_flow_error_mw\":{},",
                "\"max_branch_to_flow_error_mw\":{},",
                "\"max_generator_bound_violation_mw\":{},",
                "\"max_branch_rate_violation_mw\":{},",
                "\"max_branch_angle_violation_degrees\":{},",
                "\"source_identity_matches\":{},\"model_profile_matches\":{},",
                "\"primal_feasible\":{},\"objective_consistent\":{},\"passed\":{},",
                "\"violations\":[{}],",
                "\"claim_ceiling\":{}",
                "}}"
            ),
            quote(&self.source_git_blob_sha1),
            quote(&self.matpower_version),
            quote(&self.model_profile_id),
            self.bus_count,
            self.generator_count,
            self.branch_count,
            self.field_comparison_count,
            number(self.recomputed_objective),
            number(self.reported_objective),
            number(self.objective_absolute_error),
            number(self.max_bus_balance_residual_mw),
            number(self.max_branch_from_flow_error_mw),
            number(self.max_branch_to_flow_error_mw),
            number(self.max_generator_bound_violation_mw),
            number(self.max_branch_rate_violation_mw),
            number(self.max_branch_angle_violation_degrees),
            self.source_identity_matches,
            self.model_profile_matches,
            self.primal_feasible,
            self.objective_consistent,
            self.passed,
            violations,
            quote(self.claim_ceiling)
        )
    }
}

#[derive(Default)]
struct Metrics {
    field_comparison_count: usize,
    max_bus_balance_residual_mw: f64,
    max_branch_from_flow_error_mw: f64,
    max_branch_to_flow_error_mw: f64,
    max_generator_bound_violation_mw: f64,
    max_branch_rate_violation_mw: f64,
    max_branch_angle_violation_degrees: f64,
    violations: Vec<AuditViolation>,
}

impl Metrics {
    fn record(
        &mut self,
        family: &str,
        identity: impl ToString,
        rule: &str,
        magnitude: f64,
        tolerance: f64,
    ) {
        self.field_comparison_count += 1;
        if !magnitude.is_finite() || magnitude > tolerance {
            self.violations.push(AuditViolation {
                family: family.to_owned(),
                identity: identity.to_string(),
                rule: rule.to_owned(),
                observed_magnitude: magnitude,
                allowed_tolerance: tolerance,
            });
        }
    }

    fn record_power(
        &mut self,
        family: &str,
        identity: impl ToString,
        rule: &str,
        residual_mw: f64,
        tolerance_mw: f64,
    ) {
        self.record(family, identity, rule, residual_mw.abs(), tolerance_mw);
    }
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

fn parse_float(value: &str, context: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|error| format!("invalid {context} '{value}': {error}"))?;
    if !parsed.is_finite() {
        return Err(format!("non-finite {context} '{value}'"));
    }
    Ok(parsed)
}

fn parse_i64(value: &str, context: &str) -> Result<i64, String> {
    value
        .parse::<i64>()
        .map_err(|error| format!("invalid {context} '{value}': {error}"))
}

fn parse_row_index(value: &str, context: &str) -> Result<usize, String> {
    let parsed = parse_i64(value, context)?;
    usize::try_from(parsed).map_err(|_| format!("{context} must be positive"))
}

fn parse_status(value: &str, context: &str) -> Result<bool, String> {
    match parse_i64(value, context)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(format!("{context} must be 0 or 1, got {other}")),
    }
}

/// Parse the narrow, deterministic TSV emitted by the pinned MATPOWER reference script.
pub fn parse_candidate_tsv(input: &str) -> Result<MatpowerDcOpfCandidate, String> {
    let mut digest = None;
    let mut version = None;
    let mut profile = None;
    let mut base_mva = None;
    let mut objective = None;
    let mut buses = BTreeMap::new();
    let mut generators = Vec::new();
    let mut branches = Vec::new();
    let mut generator_rows = std::collections::BTreeSet::new();
    let mut branch_rows = std::collections::BTreeSet::new();

    for (line_index, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row = line.split('\t').collect::<Vec<_>>();
        let line_number = line_index + 1;

        match row.first().copied() {
            Some("META") if row.len() == 3 => match row[1] {
                "source_git_blob_sha1" => {
                    if digest.replace(row[2].to_owned()).is_some() {
                        return Err(format!("duplicate source digest at line {line_number}"));
                    }
                }
                "matpower_version" => {
                    if version.replace(row[2].to_owned()).is_some() {
                        return Err(format!("duplicate MATPOWER version at line {line_number}"));
                    }
                }
                "model_profile_id" => {
                    if profile.replace(row[2].to_owned()).is_some() {
                        return Err(format!("duplicate model profile at line {line_number}"));
                    }
                }
                "base_mva" => {
                    if base_mva
                        .replace(parse_float(row[2], "base MVA")?)
                        .is_some()
                    {
                        return Err(format!("duplicate base MVA at line {line_number}"));
                    }
                }
                "objective" => {
                    if objective
                        .replace(parse_float(row[2], "objective")?)
                        .is_some()
                    {
                        return Err(format!("duplicate objective at line {line_number}"));
                    }
                }
                unknown => {
                    return Err(format!("unknown META key '{unknown}' at line {line_number}"));
                }
            },
            Some("BUS") if row.len() == 3 => {
                let bus_id = parse_i64(row[1], "bus ID")?;
                let candidate = BusCandidate {
                    bus_id,
                    voltage_angle_degrees: parse_float(row[2], "bus angle")?,
                };
                if buses.insert(bus_id, candidate).is_some() {
                    return Err(format!("duplicate bus ID {bus_id} at line {line_number}"));
                }
            }
            Some("GEN") if row.len() == 5 => {
                let source_row = parse_row_index(row[1], "generator source row")?;
                if source_row == 0 || !generator_rows.insert(source_row) {
                    return Err(format!("invalid or duplicate generator row at line {line_number}"));
                }
                generators.push(GeneratorCandidate {
                    source_row,
                    bus_id: parse_i64(row[2], "generator bus ID")?,
                    in_service: parse_status(row[3], "generator status")?,
                    pg_mw: parse_float(row[4], "generator Pg")?,
                });
            }
            Some("BRANCH") if row.len() == 7 => {
                let source_row = parse_row_index(row[1], "branch source row")?;
                if source_row == 0 || !branch_rows.insert(source_row) {
                    return Err(format!("invalid or duplicate branch row at line {line_number}"));
                }
                branches.push(BranchCandidate {
                    source_row,
                    from_bus: parse_i64(row[2], "branch from bus")?,
                    to_bus: parse_i64(row[3], "branch to bus")?,
                    in_service: parse_status(row[4], "branch status")?,
                    pf_mw: parse_float(row[5], "branch PF")?,
                    pt_mw: parse_float(row[6], "branch PT")?,
                });
            }
            Some(kind) => {
                return Err(format!(
                    "malformed or unknown candidate TSV row '{kind}' at line {line_number}"
                ));
            }
            None => return Err(format!("empty candidate TSV row at line {line_number}")),
        }
    }

    let source_git_blob_sha1 = digest.ok_or("candidate receipt missing source Git blob SHA-1")?;
    if source_git_blob_sha1.len() != 40
        || !source_git_blob_sha1.chars().all(|character| character.is_ascii_hexdigit())
    {
        return Err("candidate source Git blob SHA-1 is malformed".to_owned());
    }

    let matpower_version = version.ok_or("candidate receipt missing MATPOWER version")?;
    let model_profile_id = profile.ok_or("candidate receipt missing model profile ID")?;
    let base_mva = base_mva.ok_or("candidate receipt missing base MVA")?;
    let objective = objective.ok_or("candidate receipt missing objective")?;
    if base_mva <= 0.0 {
        return Err("candidate base MVA must be positive".to_owned());
    }

    generators.sort_by_key(|generator| generator.source_row);
    branches.sort_by_key(|branch| branch.source_row);
    for (expected, generator) in generators.iter().enumerate() {
        if generator.source_row != expected + 1 {
            return Err(format!(
                "generator receipt coverage is incomplete: expected row {}, got {}",
                expected + 1,
                generator.source_row
            ));
        }
    }
    for (expected, branch) in branches.iter().enumerate() {
        if branch.source_row != expected + 1 {
            return Err(format!(
                "branch receipt coverage is incomplete: expected row {}, got {}",
                expected + 1,
                branch.source_row
            ));
        }
    }

    Ok(MatpowerDcOpfCandidate {
        source_git_blob_sha1,
        matpower_version,
        model_profile_id,
        base_mva,
        objective,
        buses,
        generators,
        branches,
    })
}

fn validate_projection_profile(projection: &MatpowerDcProjection) -> Result<(), String> {
    if projection.matpower_version != MATPOWER_VERSION {
        return Err(format!(
            "source MATPOWER version must be {MATPOWER_VERSION}, got {}",
            projection.matpower_version
        ));
    }
    if projection.generators.len() != projection.costs.len() {
        return Err(format!(
            "v0.1 requires one generator-cost row per generator ({} generators, {} cost rows)",
            projection.generators.len(),
            projection.costs.len()
        ));
    }
    for (generator, cost) in projection.generators.iter().zip(projection.costs.iter()) {
        if cost.source_row != generator.source_row {
            return Err(format!(
                "generator/cost row order mismatch at generator row {}",
                generator.source_row
            ));
        }
        if cost.model != 2 {
            return Err(format!(
                "unsupported cost model {} at generator row {}; v0.1 supports polynomial model 2 only",
                cost.model, cost.source_row
            ));
        }
        if cost.point_or_coefficient_count != cost.coefficients.len() {
            return Err(format!(
                "gencost NCOST/coefficients mismatch at row {}",
                cost.source_row
            ));
        }
        if !(1..=3).contains(&cost.point_or_coefficient_count) {
            return Err(format!(
                "unsupported polynomial degree/NCOST {} at row {}; v0.1 supports up to quadratic",
                cost.point_or_coefficient_count, cost.source_row
            ));
        }
        if cost.startup_cost != 0.0 || cost.shutdown_cost != 0.0 {
            return Err(format!(
                "nonzero startup/shutdown cost at generator row {}; static DC-OPF profile does not model commitment costs",
                cost.source_row
            ));
        }
        if cost.point_or_coefficient_count == 3 && cost.coefficients[0] < 0.0 {
            return Err(format!(
                "non-convex quadratic cost at generator row {}; convexity is required for later KKT certification",
                cost.source_row
            ));
        }
    }
    Ok(())
}

fn objective_from_dispatch(
    projection: &MatpowerDcProjection,
    candidate: &MatpowerDcOpfCandidate,
) -> Result<f64, String> {
    let mut objective = 0.0;
    for (source, cost) in projection
        .generators
        .iter()
        .zip(projection.costs.iter())
    {
        let result = candidate
            .generators
            .get(source.source_row - 1)
            .ok_or_else(|| format!("missing result for generator row {}", source.source_row))?;
        if !source.in_service {
            continue;
        }
        let mut value = 0.0;
        for coefficient in &cost.coefficients {
            value = value * result.pg_mw + coefficient;
        }
        if !value.is_finite() {
            return Err(format!(
                "non-finite recomputed cost at generator row {}",
                source.source_row
            ));
        }
        objective += value;
    }
    if !objective.is_finite() {
        return Err("recomputed objective is non-finite".to_owned());
    }
    Ok(objective)
}

fn add_violation(
    metrics: &mut Metrics,
    family: &str,
    identity: impl ToString,
    rule: &str,
    magnitude: f64,
    tolerance: f64,
) {
    metrics.record(family, identity, rule, magnitude, tolerance);
}

fn audit_projection_candidate(
    projection: &MatpowerDcProjection,
    candidate: &MatpowerDcOpfCandidate,
    options: AuditOptions,
) -> Result<AuditReport, String> {
    validate_projection_profile(projection)?;

    if !options.power_tolerance_mw.is_finite()
        || options.power_tolerance_mw < 0.0
        || !options.angle_tolerance_degrees.is_finite()
        || options.angle_tolerance_degrees < 0.0
        || !options.objective_absolute_tolerance.is_finite()
        || options.objective_absolute_tolerance < 0.0
        || !options.objective_relative_tolerance.is_finite()
        || options.objective_relative_tolerance < 0.0
    {
        return Err("audit tolerances must be finite and non-negative".to_owned());
    }

    let source_identity_matches = projection.source_git_blob_sha1 == candidate.source_git_blob_sha1;
    let model_profile_matches = candidate.model_profile_id == MODEL_PROFILE_ID
        && candidate.matpower_version == MATPOWER_VERSION;
    let base_mva_matches = (projection.base_mva - candidate.base_mva).abs()
        <= 1.0e-10 * projection.base_mva.abs().max(1.0);

    if !source_identity_matches {
        return Err(format!(
            "source identity mismatch: source {}, candidate {}",
            projection.source_git_blob_sha1, candidate.source_git_blob_sha1
        ));
    }
    if !model_profile_matches {
        return Err(format!(
            "model profile/version mismatch: expected {MODEL_PROFILE_ID}/{MATPOWER_VERSION}, got {}/{}",
            candidate.model_profile_id, candidate.matpower_version
        ));
    }
    if !base_mva_matches {
        return Err(format!(
            "base MVA mismatch: source {}, candidate {}",
            projection.base_mva, candidate.base_mva
        ));
    }
    if candidate.buses.len() != projection.buses.len()
        || candidate.generators.len() != projection.generators.len()
        || candidate.branches.len() != projection.branches.len()
    {
        return Err(format!(
            "candidate record counts do not match source (bus {}/{}, gen {}/{}, branch {}/{})",
            candidate.buses.len(),
            projection.buses.len(),
            candidate.generators.len(),
            projection.generators.len(),
            candidate.branches.len()
        ));
    }

    let mut metrics = Metrics::default();
    let mut generation_by_bus = BTreeMap::<i64, f64>::new();
    let mut bus_angle = BTreeMap::<i64, f64>::new();

    for source_bus in &projection.buses {
        let result = candidate
            .buses
            .get(&source_bus.id)
            .ok_or_else(|| format!("missing candidate bus {}", source_bus.id))?;
        bus_angle.insert(source_bus.id, result.voltage_angle_degrees);
        metrics.field_comparison_count += 1;
        // MATPOWER's DC power-flow approximation fixes voltage magnitude at 1 p.u.
        // Active-power shunt conductance is represented as a fixed MW demand at 1 p.u.
        generation_by_bus.entry(source_bus.id).or_insert(0.0);
    }

    for (source, result) in projection
        .generators
        .iter()
        .zip(candidate.generators.iter())
    {
        if source.source_row != result.source_row
            || source.bus_id != result.bus_id
            || source.in_service != result.in_service
        {
            return Err(format!(
                "generator identity/status mismatch at source row {}",
                source.source_row
            ));
        }
        if source.in_service {
            let bound_violation = (source.p_min_mw - result.pg_mw)
                .max(0.0)
                .max((result.pg_mw - source.p_max_mw).max(0.0));
            metrics.max_generator_bound_violation_mw = metrics
                .max_generator_bound_violation_mw
                .max(bound_violation);
            add_violation(
                &mut metrics,
                "generator",
                source.source_row,
                "pmin_pmax",
                bound_violation,
                options.power_tolerance_mw,
            );
            *generation_by_bus.entry(source.bus_id).or_insert(0.0) += result.pg_mw;
        } else {
            // Out-of-service generators are not part of the balance or objective.
            metrics.record(
                "generator",
                source.source_row,
                "offline_pg_is_zero",
                result.pg_mw.abs(),
                options.power_tolerance_mw,
            );
        }
    }

    let mut recomputed_branch_outflow = BTreeMap::<i64, f64>::new();

    for (source, result) in projection
        .branches
        .iter()
        .zip(candidate.branches.iter())
    {
        if source.source_row != result.source_row
            || source.from_bus != result.from_bus
            || source.to_bus != result.to_bus
            || source.in_service != result.in_service
        {
            return Err(format!(
                "branch identity/status mismatch at source row {}",
                source.source_row
            ));
        }
        let angle_from = *bus_angle
            .get(&source.from_bus)
            .ok_or_else(|| format!("unknown from-bus {}", source.from_bus))?;
        let angle_to = *bus_angle
            .get(&source.to_bus)
            .ok_or_else(|| format!("unknown to-bus {}", source.to_bus))?;

        let recomputed_pf = if source.in_service {
            let tap = if source.tap_ratio == 0.0 {
                1.0
            } else {
                source.tap_ratio
            };
            if tap <= 0.0 || source.reactance_pu <= 0.0 {
                return Err(format!(
                    "invalid tap/reactance at branch row {}",
                    source.source_row
                ));
            }
            let phase_shift_rad = source.phase_shift_degrees.to_radians();
            projection.base_mva / (source.reactance_pu * tap)
                * ((angle_from - angle_to).to_radians() - phase_shift_rad)
        } else {
            0.0
        };
        let recomputed_pt = -recomputed_pf;
        let from_error = (result.pf_mw - recomputed_pf).abs();
        let to_error = (result.pt_mw - recomputed_pt).abs();
        metrics.max_branch_from_flow_error_mw =
            metrics.max_branch_from_flow_error_mw.max(from_error);
        metrics.max_branch_to_flow_error_mw =
            metrics.max_branch_to_flow_error_mw.max(to_error);
        add_violation(
            &mut metrics,
            "branch",
            source.source_row,
            "from_end_flow_equation",
            from_error,
            options.power_tolerance_mw,
        );
        add_violation(
            &mut metrics,
            "branch",
            source.source_row,
            "to_end_flow_equation",
            to_error,
            options.power_tolerance_mw,
        );

        if source.in_service {
            *recomputed_branch_outflow.entry(source.from_bus).or_insert(0.0) += recomputed_pf;
            *recomputed_branch_outflow.entry(source.to_bus).or_insert(0.0) -= recomputed_pf;
        }

        if source.in_service && source.rate_a_mva > 0.0 {
            let overload = (recomputed_pf.abs() - source.rate_a_mva).max(0.0);
            metrics.max_branch_rate_violation_mw =
                metrics.max_branch_rate_violation_mw.max(overload);
            add_violation(
                &mut metrics,
                "branch",
                source.source_row,
                "rate_a",
                overload,
                options.power_tolerance_mw,
            );
        }

        if source.in_service {
            let angle_difference = angle_from - angle_to;
            let lower_violation = if source.angle_min_degrees > -360.0 {
                (source.angle_min_degrees - angle_difference).max(0.0)
            } else {
                0.0
            };
            let upper_violation = if source.angle_max_degrees < 360.0 {
                (angle_difference - source.angle_max_degrees).max(0.0)
            } else {
                0.0
            };
            let angle_violation = lower_violation.max(upper_violation);
            metrics.max_branch_angle_violation_degrees = metrics
                .max_branch_angle_violation_degrees
                .max(angle_violation);
            add_violation(
                &mut metrics,
                "branch",
                source.source_row,
                "angle_difference_limits",
                angle_violation,
                options.angle_tolerance_degrees,
            );
        }
    }

    for source_bus in &projection.buses {
        let generation = *generation_by_bus.get(&source_bus.id).unwrap_or(&0.0);
        let net_injection = generation
            - source_bus.demand_mw
            - source_bus.shunt_conductance_mw;
        let branch_outflow = *recomputed_branch_outflow.get(&source_bus.id).unwrap_or(&0.0);
        let residual = net_injection - branch_outflow;
        metrics.max_bus_balance_residual_mw = metrics
            .max_bus_balance_residual_mw
            .max(residual.abs());
        metrics.record_power(
            "bus",
            source_bus.id,
            "active_power_balance",
            residual,
            options.power_tolerance_mw,
        );
    }

    let recomputed_objective = objective_from_dispatch(projection, candidate)?;
    let objective_absolute_error = (recomputed_objective - candidate.objective).abs();
    let objective_tolerance = options.objective_absolute_tolerance
        + options.objective_relative_tolerance
            * recomputed_objective.abs().max(candidate.objective.abs());
    metrics.record(
        "objective",
        "system",
        "recomputed_cost_matches_reported_f",
        objective_absolute_error,
        objective_tolerance,
    );

    let primal_feasible = metrics.violations.iter().all(|violation| {
        violation.family != "objective"
    });
    let objective_consistent = objective_absolute_error <= objective_tolerance;
    let passed = primal_feasible && objective_consistent;

    Ok(AuditReport {
        source_git_blob_sha1: projection.source_git_blob_sha1.clone(),
        matpower_version: candidate.matpower_version.clone(),
        model_profile_id: candidate.model_profile_id.clone(),
        bus_count: candidate.buses.len(),
        generator_count: candidate.generators.len(),
        branch_count: candidate.branches.len(),
        field_comparison_count: metrics.field_comparison_count,
        recomputed_objective,
        reported_objective: candidate.objective,
        objective_absolute_error,
        max_bus_balance_residual_mw: metrics.max_bus_balance_residual_mw,
        max_branch_from_flow_error_mw: metrics.max_branch_from_flow_error_mw,
        max_branch_to_flow_error_mw: metrics.max_branch_to_flow_error_mw,
        max_generator_bound_violation_mw: metrics.max_generator_bound_violation_mw,
        max_branch_rate_violation_mw: metrics.max_branch_rate_violation_mw,
        max_branch_angle_violation_degrees: metrics.max_branch_angle_violation_degrees,
        source_identity_matches,
        model_profile_matches,
        primal_feasible,
        objective_consistent,
        passed,
        violations: metrics.violations,
        claim_ceiling: "independent_matpower_dc_opf_primal_feasibility_and_objective_audit_only",
    })
}

/// Independently audit a DC-OPF candidate against the exact MATPOWER source bytes.
pub fn audit_matpower_dc_opf_candidate(
    source: &str,
    candidate_tsv: &str,
    options: AuditOptions,
) -> Result<AuditReport, String> {
    let projection = project_matpower_dc_case(source)
        .map_err(|error| format!("MATPOWER source projection: {error}"))?;
    if projection.source_git_blob_sha1 != git_blob_sha1(source.as_bytes()) {
        return Err("projection digest does not bind the exact source bytes".to_owned());
    }
    let candidate = parse_candidate_tsv(candidate_tsv)?;
    audit_projection_candidate(&projection, &candidate, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_BUS: &str = include_str!("../fixtures/two_bus.m");

    fn valid_tsv(source: &str) -> String {
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
                "BRANCH\t1\t1\t2\t1\t100\t-100\n"
            ),
            git_blob_sha1(source.as_bytes())
        )
    }

    #[test]
    fn hand_calculated_two_bus_candidate_is_primal_feasible_and_objective_consistent() {
        let report = audit_matpower_dc_opf_candidate(
            TWO_BUS,
            &valid_tsv(TWO_BUS),
            AuditOptions::default(),
        )
        .expect("candidate receipt should parse and audit");
        assert!(report.passed, "{}", report.to_json());
        assert_eq!(report.bus_count, 2);
        assert_eq!(report.generator_count, 1);
        assert_eq!(report.branch_count, 1);
        assert!((report.recomputed_objective - 2100.0).abs() < 1.0e-8);
        assert!(report.max_bus_balance_residual_mw < 1.0e-6);
        assert!(report.max_branch_from_flow_error_mw < 1.0e-6);
    }

    #[test]
    fn branch_flow_mutation_is_detected() {
        let tsv = valid_tsv(TWO_BUS).replace(
            "BRANCH\t1\t1\t2\t1\t100\t-100",
            "BRANCH\t1\t1\t2\t1\t101\t-101",
        );
        let report = audit_matpower_dc_opf_candidate(TWO_BUS, &tsv, AuditOptions::default())
            .expect("numerical mismatch is an auditable result, not a parser error");
        assert!(!report.passed);
        assert!(report.max_branch_from_flow_error_mw > 0.9);
    }

    #[test]
    fn generator_bound_violation_is_reported_not_hidden() {
        let tsv = valid_tsv(TWO_BUS).replace(
            "GEN\t1\t1\t1\t100",
            "GEN\t1\t1\t1\t201",
        );
        let report = audit_matpower_dc_opf_candidate(TWO_BUS, &tsv, AuditOptions::default())
            .expect("candidate should produce a violation report");
        assert!(!report.primal_feasible);
        assert!(report.max_generator_bound_violation_mw > 0.9);
    }

    #[test]
    fn branch_overload_and_angle_violation_are_separate() {
        let source = TWO_BUS.replace("150 150 150 0 0 1 -30 30", "80 80 80 0 0 1 -3 3");
        let digest = git_blob_sha1(source.as_bytes());
        let tsv = valid_tsv(TWO_BUS).replace(
            &git_blob_sha1(TWO_BUS.as_bytes()),
            &digest,
        );
        let report = audit_matpower_dc_opf_candidate(&source, &tsv, AuditOptions::default())
            .expect("source and candidate identities match");
        assert!(!report.primal_feasible);
        assert!(report.max_branch_rate_violation_mw > 19.0);
        assert!(report.max_branch_angle_violation_degrees > 2.0);
    }

    #[test]
    fn objective_is_recomputed_and_reported_separately() {
        let tsv = valid_tsv(TWO_BUS).replace("META\tobjective\t2100", "META\tobjective\t2200");
        let report = audit_matpower_dc_opf_candidate(TWO_BUS, &tsv, AuditOptions::default())
            .expect("an objective mismatch is a reportable result");
        assert!(report.primal_feasible);
        assert!(!report.objective_consistent);
        assert!(!report.passed);
        assert!((report.objective_absolute_error - 100.0).abs() < 1.0e-8);
    }

    #[test]
    fn source_digest_mismatch_is_rejected_before_audit() {
        let tsv = valid_tsv(TWO_BUS).replace(
            &git_blob_sha1(TWO_BUS.as_bytes()),
            "0000000000000000000000000000000000000000",
        );
        assert!(
            audit_matpower_dc_opf_candidate(TWO_BUS, &tsv, AuditOptions::default())
                .unwrap_err()
                .contains("source identity mismatch")
        );
    }

    #[test]
    fn duplicate_bus_and_branch_rows_are_rejected() {
        let duplicate_bus = format!("{}\nBUS\t1\t0\n", valid_tsv(TWO_BUS));
        assert!(parse_candidate_tsv(&duplicate_bus).unwrap_err().contains("duplicate bus ID"));

        let duplicate_branch = format!(
            "{}\nBRANCH\t1\t1\t2\t1\t100\t-100\n",
            valid_tsv(TWO_BUS)
        );
        assert!(parse_candidate_tsv(&duplicate_branch)
            .unwrap_err()
            .contains("duplicate branch row"));
    }

    #[test]
    fn unsupported_cost_profile_fails_closed() {
        let source = TWO_BUS.replace(
            "2 0 0 3 0.01 20 0;",
            "1 0 0 2 0 0 200 200;",
        );
        let error = audit_matpower_dc_opf_candidate(
            &source,
            &valid_tsv(&source),
            AuditOptions::default(),
        )
        .unwrap_err();
        assert!(error.contains("unsupported cost model"));
    }

    #[test]
    fn rejects_wrong_dc_model_profile() {
        let tsv = valid_tsv(TWO_BUS).replace(
            "matpower_8_1_makeBdc_dcopf",
            "powermodels_dcp",
        );
        assert!(audit_matpower_dc_opf_candidate(TWO_BUS, &tsv, AuditOptions::default())
            .unwrap_err()
            .contains("model profile/version mismatch"));
    }
}
