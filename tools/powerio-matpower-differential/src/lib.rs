// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Field-by-field differential audit between Sol-Atlas's narrow MATPOWER
//! projection and the independently maintained PowerIO MATPOWER reader.
//!
//! This crate does not use PowerIO to generate the Sol-Atlas projection. Both
//! parsers consume the same immutable source bytes, then their typed results
//! are compared field-by-field. A mismatch is reported; it is never silently
//! normalized away.

use std::collections::BTreeMap;

use pglib_case_auditor::{MatpowerDcProjection, git_blob_sha1, project_matpower_dc_case};
use powerio::{PioValue, Source, parse};

#[derive(Clone, Debug, serde::Serialize, PartialEq)]
pub struct FieldMismatch {
    pub family: String,
    pub row: String,
    pub field: String,
    pub sol_atlas_value: String,
    pub powerio_value: String,
    pub absolute_error: Option<f64>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq)]
pub struct DifferentialReport {
    pub source_git_blob_sha1: String,
    pub powerio_version: String,
    pub matpower_function: String,
    pub bus_rows_compared: usize,
    pub generator_rows_compared: usize,
    pub branch_rows_compared: usize,
    pub load_rows_aggregated: usize,
    pub cost_rows_seen_by_powerio: usize,
    pub powerio_diagnostic_count: usize,
    pub field_comparison_count: usize,
    pub mismatch_count: usize,
    pub maximum_absolute_error: f64,
    pub mismatches: Vec<FieldMismatch>,
    pub claim_ceiling: &'static str,
}

impl DifferentialReport {
    #[must_use]
    pub fn is_field_parity(&self) -> bool {
        self.mismatch_count == 0
            && self.powerio_diagnostic_count == 0
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .expect("differential report fields are JSON serializable")
    }
}

#[derive(Clone, Default)]
struct BusTotals {
    load_p: f64,
    load_q: f64,
    generation_p: f64,
    generation_pmin: f64,
    generation_pmax: f64,
    shunt_g: f64,
    shunt_b: f64,
}

struct Comparator {
    mismatches: Vec<FieldMismatch>,
    comparisons: usize,
    maximum_absolute_error: f64,
}

impl Comparator {
    fn new() -> Self {
        Self {
            mismatches: Vec::new(),
            comparisons: 0,
            maximum_absolute_error: 0.0,
        }
    }

    fn number(
        &mut self,
        family: &str,
        row: impl ToString,
        field: &str,
        sol_atlas_value: f64,
        powerio_value: f64,
    ) {
        self.comparisons += 1;
        let equal = if sol_atlas_value == powerio_value {
            true
        } else if sol_atlas_value.is_finite() && powerio_value.is_finite() {
            let difference = (sol_atlas_value - powerio_value).abs();
            self.maximum_absolute_error = self.maximum_absolute_error.max(difference);
            difference <= 1.0e-8 + 1.0e-10 * sol_atlas_value.abs().max(powerio_value.abs())
        } else {
            false
        };

        if !equal {
            let absolute_error = if sol_atlas_value.is_finite() && powerio_value.is_finite() {
                Some((sol_atlas_value - powerio_value).abs())
            } else {
                None
            };
            self.mismatches.push(FieldMismatch {
                family: family.to_owned(),
                row: row.to_string(),
                field: field.to_owned(),
                sol_atlas_value: sol_atlas_value.to_string(),
                powerio_value: powerio_value.to_string(),
                absolute_error,
            });
        }
    }

    fn boolean(
        &mut self,
        family: &str,
        row: impl ToString,
        field: &str,
        sol_atlas_value: bool,
        powerio_value: bool,
    ) {
        self.number(
            family,
            row,
            field,
            u8::from(sol_atlas_value) as f64,
            u8::from(powerio_value) as f64,
        );
    }

    fn integer(
        &mut self,
        family: &str,
        row: impl ToString,
        field: &str,
        sol_atlas_value: i64,
        powerio_value: i64,
    ) {
        self.number(
            family,
            row,
            field,
            sol_atlas_value as f64,
            powerio_value as f64,
        );
    }
}

fn powerio_bus_type(kind: powerio::BusType) -> i64 {
    match kind {
        powerio::BusType::Pq => 1,
        powerio::BusType::Pv => 2,
        powerio::BusType::Ref => 3,
        powerio::BusType::Isolated => 4,
        _ => -1,
    }
}

fn add_to_bus_totals(
    totals: &mut BTreeMap<i64, BusTotals>,
    bus_id: i64,
) -> &mut BusTotals {
    totals.entry(bus_id).or_default()
}

fn compare_projection(
    source: &str,
    projection: &MatpowerDcProjection,
    module: &powerio::PioModule<PioValue>,
) -> Result<DifferentialReport, String> {
    let network = match module.value() {
        PioValue::BalancedNetwork(network) => network,
        other => {
            return Err(format!(
                "PowerIO returned {}, expected powerio.BalancedNetwork",
                other.type_name()
            ));
        }
    };

    let mut cmp = Comparator::new();
    cmp.number(
        "network",
        "case",
        "base_mva",
        projection.base_mva,
        network.base_mva(),
    );

    let powerio_buses = network.buses();
    let powerio_generators = network.generators();
    let powerio_branches = network.branches();
    let powerio_loads = network.loads();
    let powerio_shunts = network.shunts();

    cmp.integer(
        "network",
        "case",
        "bus_count",
        projection.buses.len() as i64,
        powerio_buses.len() as i64,
    );
    cmp.integer(
        "network",
        "case",
        "generator_count",
        projection.generators.len() as i64,
        powerio_generators.len() as i64,
    );
    cmp.integer(
        "network",
        "case",
        "branch_count",
        projection.branches.len() as i64,
        powerio_branches.len() as i64,
    );

    let mut bus_totals = BTreeMap::<i64, BusTotals>::new();
    for load in powerio_loads.iter().filter(|load| load.in_service) {
        let total = add_to_bus_totals(&mut bus_totals, load.bus.0 as i64);
        total.load_p += load.p;
        total.load_q += load.q;
    }
    for generator in powerio_generators.iter().filter(|generator| generator.in_service) {
        let total = add_to_bus_totals(&mut bus_totals, generator.bus.0 as i64);
        total.generation_p += generator.pg;
        total.generation_pmin += generator.pmin;
        total.generation_pmax += generator.pmax;
    }
    for shunt in powerio_shunts.iter().filter(|shunt| shunt.in_service) {
        let total = add_to_bus_totals(&mut bus_totals, shunt.bus.0 as i64);
        total.shunt_g += shunt.g;
        total.shunt_b += shunt.b;
    }

    let row_bus_count = projection.buses.len().min(powerio_buses.len());
    for (row, (expected, observed)) in projection
        .buses
        .iter()
        .zip(powerio_buses.iter())
        .enumerate()
        .take(row_bus_count)
    {
        let label = format!("row-{}", row + 1);
        cmp.integer("bus", &label, "id", expected.id, observed.id.0 as i64);
        cmp.integer(
            "bus",
            &label,
            "type",
            expected.bus_type,
            powerio_bus_type(observed.kind),
        );
        cmp.number("bus", &label, "base_kv", expected.base_kv, observed.base_kv);
        cmp.number("bus", &label, "vm", expected.voltage_magnitude_initial_pu, observed.vm);
        cmp.number("bus", &label, "va_degrees", expected.voltage_angle_initial_degrees, observed.va);
        cmp.number("bus", &label, "vmax", expected.voltage_max_pu, observed.vmax);
        cmp.number("bus", &label, "vmin", expected.voltage_min_pu, observed.vmin);
        cmp.integer("bus", &label, "area", expected.area, observed.area as i64);
        cmp.integer("bus", &label, "zone", expected.zone, observed.zone as i64);

        let totals = bus_totals.get(&expected.id).cloned().unwrap_or_default();
        cmp.number("bus", &label, "pd_aggregated", expected.demand_mw, totals.load_p);
        cmp.number(
            "bus",
            &label,
            "qd_aggregated",
            expected.reactive_demand_mvar,
            totals.load_q,
        );
        cmp.number(
            "bus",
            &label,
            "pg_aggregated_online",
            expected.online_generation_mw,
            totals.generation_p,
        );
        cmp.number(
            "bus",
            &label,
            "pmin_aggregated_online",
            expected.online_generation_p_min_mw,
            totals.generation_pmin,
        );
        cmp.number(
            "bus",
            &label,
            "pmax_aggregated_online",
            expected.online_generation_p_max_mw,
            totals.generation_pmax,
        );
        cmp.number(
            "bus",
            &label,
            "gs_aggregated",
            expected.shunt_conductance_mw,
            totals.shunt_g,
        );
        cmp.number(
            "bus",
            &label,
            "bs_aggregated",
            expected.shunt_susceptance_mvar,
            totals.shunt_b,
        );
    }

    let row_generator_count = projection.generators.len().min(powerio_generators.len());
    for (row, (expected, observed)) in projection
        .generators
        .iter()
        .zip(powerio_generators.iter())
        .enumerate()
        .take(row_generator_count)
    {
        let label = format!("row-{}", row + 1);
        cmp.integer("generator", &label, "bus", expected.bus_id, observed.bus.0 as i64);
        cmp.number("generator", &label, "pg", expected.pg_mw, observed.pg);
        cmp.number("generator", &label, "qg", expected.qg_mvar, observed.qg);
        cmp.number("generator", &label, "pmax", expected.p_max_mw, observed.pmax);
        cmp.number("generator", &label, "pmin", expected.p_min_mw, observed.pmin);
        cmp.number("generator", &label, "qmax", expected.q_max_mvar, observed.qmax);
        cmp.number("generator", &label, "qmin", expected.q_min_mvar, observed.qmin);
        cmp.number(
            "generator",
            &label,
            "voltage_setpoint",
            expected.voltage_setpoint_pu,
            observed.vg,
        );
        cmp.number("generator", &label, "mbase", expected.m_base_mva, observed.mbase);
        cmp.boolean("generator", &label, "in_service", expected.in_service, observed.in_service);
    }

    let row_branch_count = projection.branches.len().min(powerio_branches.len());
    for (row, (expected, observed)) in projection
        .branches
        .iter()
        .zip(powerio_branches.iter())
        .enumerate()
        .take(row_branch_count)
    {
        let label = format!("row-{}", row + 1);
        cmp.integer("branch", &label, "from_bus", expected.from_bus, observed.from.0 as i64);
        cmp.integer("branch", &label, "to_bus", expected.to_bus, observed.to.0 as i64);
        cmp.number("branch", &label, "resistance_pu", expected.resistance_pu, observed.r);
        cmp.number("branch", &label, "reactance_pu", expected.reactance_pu, observed.x);
        cmp.number(
            "branch",
            &label,
            "charging_b_pu",
            expected.line_charging_susceptance_pu,
            observed.b,
        );
        // MATPOWER uses zero to mean an unlimited thermal rating; PowerIO
        // canonicalizes this semantic value as positive infinity.
        let unlimited_to_infinity = |rating: f64| {
            if rating == 0.0 { f64::INFINITY } else { rating }
        };
        cmp.number(
            "branch",
            &label,
            "rate_a_mva_semantic",
            unlimited_to_infinity(expected.rate_a_mva),
            observed.rate_a,
        );
        cmp.number(
            "branch",
            &label,
            "rate_b_mva_semantic",
            unlimited_to_infinity(expected.rate_b_mva),
            observed.rate_b,
        );
        cmp.number(
            "branch",
            &label,
            "rate_c_mva_semantic",
            unlimited_to_infinity(expected.rate_c_mva),
            observed.rate_c,
        );
        cmp.number("branch", &label, "tap_ratio", expected.tap_ratio, observed.tap);
        cmp.number("branch", &label, "phase_shift_degrees", expected.phase_shift_degrees, observed.shift);
        cmp.boolean("branch", &label, "in_service", expected.in_service, observed.in_service);
        cmp.number("branch", &label, "angle_min_degrees", expected.angle_min_degrees, observed.angmin);
        cmp.number("branch", &label, "angle_max_degrees", expected.angle_max_degrees, observed.angmax);
    }

    for (row, (expected, observed)) in projection
        .costs
        .iter()
        .zip(powerio_generators.iter())
        .enumerate()
    {
        let label = format!("row-{}", row + 1);
        match &observed.cost {
            Some(cost) => {
                cmp.integer("gencost", &label, "model", expected.model as i64, cost.model as i64);
                cmp.number("gencost", &label, "startup", expected.startup_cost, cost.startup);
                cmp.number("gencost", &label, "shutdown", expected.shutdown_cost, cost.shutdown);
                cmp.integer(
                    "gencost",
                    &label,
                    "ncost",
                    expected.point_or_coefficient_count as i64,
                    cost.ncost as i64,
                );
                cmp.integer(
                    "gencost",
                    &label,
                    "coefficient_count",
                    expected.coefficients.len() as i64,
                    cost.coeffs.len() as i64,
                );
                for (coefficient, (sol_value, powerio_value)) in expected
                    .coefficients
                    .iter()
                    .zip(cost.coeffs.iter())
                    .enumerate()
                {
                    cmp.number(
                        "gencost",
                        &label,
                        &format!("coefficient-{}", coefficient + 1),
                        *sol_value,
                        *powerio_value,
                    );
                }
            }
            None => {
                cmp.integer("gencost", &label, "present", 1, 0);
            }
        }
    }
    let powerio_cost_rows = powerio_generators
        .iter()
        .filter(|generator| generator.cost.is_some())
        .count();
    cmp.integer(
        "generator_cost",
        "case",
        "cost_row_count",
        projection.cost_row_count as i64,
        powerio_cost_rows as i64,
    );

    let diagnostics = module.diagnostics().len();
    Ok(DifferentialReport {
        source_git_blob_sha1: git_blob_sha1(source.as_bytes()),
        powerio_version: powerio::VERSION.to_owned(),
        matpower_function: projection.function_name.clone(),
        bus_rows_compared: row_bus_count,
        generator_rows_compared: row_generator_count,
        branch_rows_compared: row_branch_count,
        load_rows_aggregated: powerio_loads.len(),
        cost_rows_seen_by_powerio: powerio_cost_rows,
        powerio_diagnostic_count: diagnostics,
        field_comparison_count: cmp.comparisons,
        mismatch_count: cmp.mismatches.len(),
        maximum_absolute_error: cmp.maximum_absolute_error,
        mismatches: cmp.mismatches,
        claim_ceiling: "independent_matpower_field_differential_only",
    })
}

/// Parse the exact same source bytes with both independently maintained readers.
pub fn compare_matpower_source(source: &str) -> Result<DifferentialReport, String> {
    let projection =
        project_matpower_dc_case(source).map_err(|error| format!("Sol-Atlas parser: {error}"))?;
    let memory = Source::from_memory("pglib-case.m", source.as_bytes().to_vec())
        .map_err(|error| format!("PowerIO source construction: {error}"))?
        .with_format(
            powerio::FormatId::new("matpower")
                .map_err(|error| format!("PowerIO format selection: {error}"))?,
        );
    let module = parse(memory).map_err(|error| format!("PowerIO MATPOWER parser: {error}"))?;
    compare_projection(source, &projection, &module)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASE14: &str =
        include_str!("../../pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m");
    const CASE24: &str =
        include_str!("../../pglib-case-auditor/fixtures/pglib_opf_case24_ieee_rts.m");
    const CASE57: &str =
        include_str!("../../pglib-case-auditor/fixtures/pglib_opf_case57_ieee.m");

    #[test]
    fn powerio_matches_all_declared_fields_for_ieee_14() {
        let report = compare_matpower_source(CASE14).expect("both parsers admit IEEE 14");
        assert_eq!(report.source_git_blob_sha1, "2c8260fff0311c902c3d36dcabcbe861576fc97c");
        assert!(report.field_comparison_count > 100);
        assert!(report.is_field_parity(), "{}", report.to_json());
    }

    #[test]
    fn powerio_matches_all_declared_fields_for_ieee_rts_24() {
        let report = compare_matpower_source(CASE24).expect("both parsers admit IEEE RTS 24");
        assert_eq!(report.source_git_blob_sha1, "00b024c2e583b82452392514d2c22881c708ba7c");
        assert!(report.field_comparison_count > 250);
        assert!(report.is_field_parity(), "{}", report.to_json());
    }

    #[test]
    fn powerio_matches_all_declared_fields_for_ieee_57() {
        let report = compare_matpower_source(CASE57).expect("both parsers admit IEEE 57");
        assert_eq!(report.source_git_blob_sha1, "d8b0c9e8b45265e57f52b7ab96880ad3a874cde5");
        assert!(report.field_comparison_count > 500);
        assert!(report.is_field_parity(), "{}", report.to_json());
    }

    #[test]
    fn source_byte_change_is_visible_in_cross_parser_receipt() {
        let changed = format!("{CASE14}\n% independent parser digest negative control\n");
        let report = compare_matpower_source(&changed).expect("modified comment still parses");
        assert_ne!(report.source_git_blob_sha1, "2c8260fff0311c902c3d36dcabcbe861576fc97c");
        assert!(report.is_field_parity());
    }

    #[test]
    fn strict_sol_atlas_profile_rejects_unsupported_matpower_extensions() {
        let source = format!("{CASE14}\nmpc.unsupported = 7;\n");
        let error = compare_matpower_source(&source)
            .expect_err("the Sol-Atlas strict importer must reject unknown statements");
        assert!(error.contains("Sol-Atlas parser"));
    }
}
