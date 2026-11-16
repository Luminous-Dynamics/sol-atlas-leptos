// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Differential comparison of Sol-Atlas DC power flow against MATPOWER 8.1 rundcpf.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::process::ExitCode;

use dc_power_flow_reference::SolveOptions;
use matpower_dc_network_evaluation::evaluate_matpower_dc_case;

const MATPOWER_VERSION: &str = "8.1";
const ANGLE_TOLERANCE_DEGREES: f64 = 1.0e-7;
const POWER_TOLERANCE_MW: f64 = 1.0e-6;
const BASE_MVA_TOLERANCE: f64 = 1.0e-10;

#[derive(Clone, Debug)]
struct ReferenceBus {
    angle_degrees: f64,
    net_injection_mw: f64,
}

#[derive(Clone, Debug)]
struct ReferenceBranch {
    row: usize,
    from_bus: i64,
    to_bus: i64,
    in_service: bool,
    pf_mw: f64,
    pt_mw: f64,
}

#[derive(Clone, Debug)]
struct ReferenceResult {
    source_git_blob_sha1: String,
    matpower_version: String,
    base_mva: f64,
    reference_bus: i64,
    buses: BTreeMap<i64, ReferenceBus>,
    branches: Vec<ReferenceBranch>,
}

#[derive(Clone, Debug)]
struct Mismatch {
    element: String,
    identity: String,
    field: String,
    sol_atlas_value: f64,
    matpower_value: f64,
    absolute_error: f64,
    tolerance: f64,
}

#[derive(Default)]
struct Checker {
    mismatches: Vec<Mismatch>,
    field_comparison_count: usize,
}

impl Checker {
    fn compare(
        &mut self,
        element: &str,
        identity: impl ToString,
        field: &str,
        sol_atlas_value: f64,
        matpower_value: f64,
        tolerance: f64,
    ) -> f64 {
        self.field_comparison_count += 1;
        let absolute_error = (sol_atlas_value - matpower_value).abs();
        if !sol_atlas_value.is_finite() || !matpower_value.is_finite() || absolute_error > tolerance
        {
            self.mismatches.push(Mismatch {
                element: element.to_owned(),
                identity: identity.to_string(),
                field: field.to_owned(),
                sol_atlas_value,
                matpower_value,
                absolute_error,
                tolerance,
            });
        }
        absolute_error
    }
}

#[derive(Clone, Debug)]
struct ComparisonReport {
    source_git_blob_sha1: String,
    matpower_version: String,
    reference_bus: i64,
    bus_count: usize,
    branch_count: usize,
    field_comparison_count: usize,
    max_abs_angle_error_degrees: f64,
    max_abs_injection_error_mw: f64,
    max_abs_branch_flow_error_mw: f64,
    mismatch_count: usize,
    mismatches: Vec<Mismatch>,
}

impl ComparisonReport {
    fn passed(&self) -> bool {
        self.mismatch_count == 0
    }

    fn to_json(&self) -> String {
        let mismatches = self
            .mismatches
            .iter()
            .map(|mismatch| {
                format!(
                    concat!(
                        "{{\"element\":{},\"identity\":{},\"field\":{},",
                        "\"sol_atlas_value\":{},\"matpower_value\":{},",
                        "\"absolute_error\":{},\"tolerance\":{}}}"
                    ),
                    quote(&mismatch.element),
                    quote(&mismatch.identity),
                    quote(&mismatch.field),
                    number(mismatch.sol_atlas_value),
                    number(mismatch.matpower_value),
                    number(mismatch.absolute_error),
                    number(mismatch.tolerance)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{",
                "\"schema_version\":\"matpower-dc-reference-differential.v0.1.0\",",
                "\"source_git_blob_sha1\":{},",
                "\"matpower_version\":{},",
                "\"reference_bus\":{},",
                "\"bus_count\":{},\"branch_count\":{},",
                "\"field_comparison_count\":{},",
                "\"max_abs_angle_error_degrees\":{},",
                "\"max_abs_injection_error_mw\":{},",
                "\"max_abs_branch_flow_error_mw\":{},",
                "\"mismatch_count\":{},\"passed\":{},",
                "\"mismatches\":[{}],",
                "\"claim_ceiling\":\"cross_implementation_fixed_injection_dc_differential_only\"",
                "}}"
            ),
            quote(&self.source_git_blob_sha1),
            quote(&self.matpower_version),
            self.reference_bus,
            self.bus_count,
            self.branch_count,
            self.field_comparison_count,
            number(self.max_abs_angle_error_degrees),
            number(self.max_abs_injection_error_mw),
            number(self.max_abs_branch_flow_error_mw),
            self.mismatch_count,
            self.passed(),
            mismatches
        )
    }
}

fn quote(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
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
    if !value.is_finite() {
        "null".to_owned()
    } else if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
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

fn parse_reference_tsv(source: &str) -> Result<ReferenceResult, String> {
    let mut source_digest = None;
    let mut matpower_version = None;
    let mut base_mva = None;
    let mut reference_bus = None;
    let mut buses = BTreeMap::new();
    let mut branches = Vec::new();
    let mut branch_rows = BTreeSet::new();

    for (line_index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        let line_number = line_index + 1;
        match fields.first().copied() {
            Some("META") if fields.len() == 3 => match fields[1] {
                "source_git_blob_sha1" => {
                    if source_digest.replace(fields[2].to_owned()).is_some() {
                        return Err(format!("duplicate source digest at TSV line {line_number}"));
                    }
                }
                "matpower_version" => {
                    if matpower_version.replace(fields[2].to_owned()).is_some() {
                        return Err(format!(
                            "duplicate MATPOWER version at TSV line {line_number}"
                        ));
                    }
                }
                "base_mva" => {
                    if base_mva
                        .replace(parse_float(fields[2], "base MVA")?)
                        .is_some()
                    {
                        return Err(format!("duplicate base MVA at TSV line {line_number}"));
                    }
                }
                "reference_bus" => {
                    if reference_bus
                        .replace(parse_i64(fields[2], "reference bus")?)
                        .is_some()
                    {
                        return Err(format!("duplicate reference bus at TSV line {line_number}"));
                    }
                }
                unknown => {
                    return Err(format!(
                        "unknown META key '{unknown}' at TSV line {line_number}"
                    ));
                }
            },
            Some("BUS") if fields.len() == 4 => {
                let id = parse_i64(fields[1], "bus ID")?;
                let bus = ReferenceBus {
                    angle_degrees: parse_float(fields[2], "bus angle")?,
                    net_injection_mw: parse_float(fields[3], "bus injection")?,
                };
                if buses.insert(id, bus).is_some() {
                    return Err(format!("duplicate bus ID {id} at TSV line {line_number}"));
                }
            }
            Some("BRANCH") if fields.len() == 7 => {
                let row = usize::try_from(parse_i64(fields[1], "branch row")?)
                    .map_err(|_| format!("negative branch row at TSV line {line_number}"))?;
                if row == 0 || !branch_rows.insert(row) {
                    return Err(format!(
                        "invalid or duplicate branch row {row} at TSV line {line_number}"
                    ));
                }
                let status = parse_i64(fields[4], "branch status")?;
                if !matches!(status, 0 | 1) {
                    return Err(format!(
                        "branch status must be 0 or 1 at TSV line {line_number}"
                    ));
                }
                branches.push(ReferenceBranch {
                    row,
                    from_bus: parse_i64(fields[2], "branch from bus")?,
                    to_bus: parse_i64(fields[3], "branch to bus")?,
                    in_service: status == 1,
                    pf_mw: parse_float(fields[5], "branch from flow")?,
                    pt_mw: parse_float(fields[6], "branch to flow")?,
                });
            }
            Some(kind) => {
                return Err(format!(
                    "malformed or unknown TSV row '{kind}' at line {line_number}"
                ));
            }
            None => return Err(format!("empty TSV record at line {line_number}")),
        }
    }

    let source_git_blob_sha1 = source_digest.ok_or("MATPOWER TSV is missing source digest")?;
    if source_git_blob_sha1.len() != 40
        || !source_git_blob_sha1.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err("MATPOWER TSV has malformed source digest".to_owned());
    }
    let matpower_version = matpower_version.ok_or("MATPOWER TSV is missing version")?;
    if matpower_version != MATPOWER_VERSION {
        return Err(format!(
            "MATPOWER reference version drift: expected {MATPOWER_VERSION}, got {matpower_version}"
        ));
    }
    for (index, branch) in branches.iter().enumerate() {
        if branch.row != index + 1 {
            return Err(format!(
                "MATPOWER branch rows are not complete and ordered: expected {}, got {}",
                index + 1,
                branch.row
            ));
        }
    }

    Ok(ReferenceResult {
        source_git_blob_sha1,
        matpower_version,
        base_mva: base_mva.ok_or("MATPOWER TSV is missing base MVA")?,
        reference_bus: reference_bus.ok_or("MATPOWER TSV is missing reference bus")?,
        buses,
        branches,
    })
}

fn compare(source: &str, reference_tsv: &str) -> Result<ComparisonReport, String> {
    let evaluation = evaluate_matpower_dc_case(source, SolveOptions::default())
        .map_err(|error| format!("Sol-Atlas evaluation failed: {error}"))?;
    let reference = parse_reference_tsv(reference_tsv)?;

    if evaluation.source_git_blob_sha1 != reference.source_git_blob_sha1 {
        return Err(format!(
            "source identity mismatch: Sol-Atlas {}, MATPOWER {}",
            evaluation.source_git_blob_sha1, reference.source_git_blob_sha1
        ));
    }

    let mut checker = Checker::default();
    let mut max_abs_angle_error_degrees: f64 = 0.0;
    let mut max_abs_injection_error_mw: f64 = 0.0;
    let mut max_abs_branch_flow_error_mw: f64 = 0.0;

    checker.compare(
        "network",
        "case",
        "base_mva",
        evaluation.network.base_mva,
        reference.base_mva,
        BASE_MVA_TOLERANCE,
    );

    let our_buses = evaluation
        .solution
        .bus_states
        .iter()
        .map(|bus| (bus.bus_id, bus))
        .collect::<BTreeMap<_, _>>();
    if our_buses.len() != evaluation.solution.bus_states.len()
        || our_buses.len() != reference.buses.len()
    {
        return Err(format!(
            "bus count or identity mismatch: Sol-Atlas {}, MATPOWER {}",
            our_buses.len(),
            reference.buses.len()
        ));
    }
    let our_reference_buses = evaluation
        .solution
        .bus_states
        .iter()
        .filter(|bus| bus.is_reference_bus)
        .collect::<Vec<_>>();
    if our_reference_buses.len() != 1 || our_reference_buses[0].bus_id != reference.reference_bus {
        return Err(format!(
            "reference-bus mismatch: Sol-Atlas {:?}, MATPOWER {}",
            our_reference_buses
                .iter()
                .map(|bus| bus.bus_id)
                .collect::<Vec<_>>(),
            reference.reference_bus
        ));
    }

    for (bus_id, expected) in &reference.buses {
        let observed = our_buses
            .get(bus_id)
            .ok_or_else(|| format!("MATPOWER bus {bus_id} is missing from Sol-Atlas"))?;
        let angle = observed.voltage_angle_rad.to_degrees();
        let angle_error = (angle - expected.angle_degrees).abs();
        max_abs_angle_error_degrees = max_abs_angle_error_degrees.max(angle_error);
        checker.compare(
            "bus",
            bus_id,
            "angle_relative_to_reference_degrees",
            angle,
            expected.angle_degrees,
            ANGLE_TOLERANCE_DEGREES,
        );
        let injection_error = (observed.net_injection_mw - expected.net_injection_mw).abs();
        max_abs_injection_error_mw = max_abs_injection_error_mw.max(injection_error);
        checker.compare(
            "bus",
            bus_id,
            "net_injection_mw",
            observed.net_injection_mw,
            expected.net_injection_mw,
            POWER_TOLERANCE_MW,
        );
    }

    if evaluation.solution.branch_flows.len() != reference.branches.len() {
        return Err(format!(
            "branch count mismatch: Sol-Atlas {}, MATPOWER {}",
            evaluation.solution.branch_flows.len(),
            reference.branches.len()
        ));
    }
    for branch in &evaluation.solution.branch_flows {
        let source_row = branch.branch_index + 1;
        let expected = reference
            .branches
            .iter()
            .find(|candidate| candidate.row == source_row)
            .ok_or_else(|| format!("MATPOWER branch row {source_row} is missing"))?;
        if branch.from_bus != expected.from_bus || branch.to_bus != expected.to_bus {
            return Err(format!(
                "branch row {source_row} endpoint identity mismatch"
            ));
        }
        if branch.in_service != expected.in_service {
            return Err(format!("branch row {source_row} status mismatch"));
        }
        let forward_error = (branch.flow_from_to_mw - expected.pf_mw).abs();
        let reverse_error = (-branch.flow_from_to_mw - expected.pt_mw).abs();
        max_abs_branch_flow_error_mw = max_abs_branch_flow_error_mw
            .max(forward_error)
            .max(reverse_error);
        checker.compare(
            "branch",
            source_row,
            "from_to_flow_mw",
            branch.flow_from_to_mw,
            expected.pf_mw,
            POWER_TOLERANCE_MW,
        );
        checker.compare(
            "branch",
            source_row,
            "to_from_flow_mw",
            -branch.flow_from_to_mw,
            expected.pt_mw,
            POWER_TOLERANCE_MW,
        );
    }

    Ok(ComparisonReport {
        source_git_blob_sha1: evaluation.source_git_blob_sha1,
        matpower_version: reference.matpower_version,
        reference_bus: reference.reference_bus,
        bus_count: our_buses.len(),
        branch_count: reference.branches.len(),
        field_comparison_count: checker.field_comparison_count,
        max_abs_angle_error_degrees,
        max_abs_injection_error_mw,
        max_abs_branch_flow_error_mw,
        mismatch_count: checker.mismatches.len(),
        mismatches: checker.mismatches,
    })
}

fn run() -> Result<ComparisonReport, String> {
    let mut args = env::args().skip(1);
    let case_path = args
        .next()
        .ok_or_else(|| "usage: compare-matpower-dc <case.m> <matpower-reference.tsv>".to_owned())?;
    let reference_path = args
        .next()
        .ok_or_else(|| "missing MATPOWER reference TSV path".to_owned())?;
    if args.next().is_some() {
        return Err("too many command-line arguments".to_owned());
    }
    let source_bytes =
        fs::read(&case_path).map_err(|error| format!("read {case_path}: {error}"))?;
    let source =
        std::str::from_utf8(&source_bytes).map_err(|_| format!("{case_path} is not UTF-8"))?;
    let reference = fs::read_to_string(&reference_path)
        .map_err(|error| format!("read {reference_path}: {error}"))?;
    compare(source, &reference)
}

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            println!("{}", report.to_json());
            if report.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("compare-matpower-dc: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_matpower_reference_receipt() {
        let tsv = concat!(
            "META\tsource_git_blob_sha1\t2c8260fff0311c902c3d36dcabcbe861576fc97c\n",
            "META\tmatpower_version\t8.1\n",
            "META\tbase_mva\t100\n",
            "META\treference_bus\t1\n",
            "BUS\t1\t0\t10\n",
            "BUS\t2\t-1.5\t-10\n",
            "BRANCH\t1\t1\t2\t1\t10\t-10\n"
        );
        let report = parse_reference_tsv(tsv).expect("valid reference TSV");
        assert_eq!(report.reference_bus, 1);
        assert_eq!(report.buses.len(), 2);
        assert_eq!(report.branches.len(), 1);
    }

    #[test]
    fn rejects_duplicate_bus_and_branch_ids_and_unknown_metadata() {
        let duplicate_bus = concat!(
            "META\tsource_git_blob_sha1\t2c8260fff0311c902c3d36dcabcbe861576fc97c\n",
            "META\tmatpower_version\t8.1\n",
            "META\tbase_mva\t100\n",
            "META\treference_bus\t1\n",
            "BUS\t1\t0\t0\n",
            "BUS\t1\t0\t0\n"
        );
        assert!(
            parse_reference_tsv(duplicate_bus)
                .unwrap_err()
                .contains("duplicate bus ID")
        );

        let duplicate_branch = concat!(
            "META\tsource_git_blob_sha1\t2c8260fff0311c902c3d36dcabcbe861576fc97c\n",
            "META\tmatpower_version\t8.1\n",
            "META\tbase_mva\t100\n",
            "META\treference_bus\t1\n",
            "BUS\t1\t0\t0\n",
            "BRANCH\t1\t1\t1\t1\t0\t0\n",
            "BRANCH\t1\t1\t1\t1\t0\t0\n"
        );
        assert!(
            parse_reference_tsv(duplicate_branch)
                .unwrap_err()
                .contains("invalid or duplicate branch row")
        );

        let unknown = concat!(
            "META\tsource_git_blob_sha1\t2c8260fff0311c902c3d36dcabcbe861576fc97c\n",
            "META\tmatpower_version\t8.1\n",
            "META\tbase_mva\t100\n",
            "META\treference_bus\t1\n",
            "BOGUS\t1\n"
        );
        assert!(
            parse_reference_tsv(unknown)
                .unwrap_err()
                .contains("unknown TSV row")
        );
    }

    const CASE14: &str =
        include_str!("../../../pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m");

    fn sol_atlas_reference_tsv(source: &str) -> String {
        let evaluation =
            evaluate_matpower_dc_case(source, SolveOptions::default()).expect("valid DC case");
        let reference = evaluation
            .solution
            .bus_states
            .iter()
            .find(|bus| bus.is_reference_bus)
            .expect("one DC reference bus");
        let mut tsv = format!(
            concat!(
                "META\tsource_git_blob_sha1\t{}\n",
                "META\tmatpower_version\t8.1\n",
                "META\tbase_mva\t{}\n",
                "META\treference_bus\t{}\n"
            ),
            evaluation.source_git_blob_sha1,
            number(evaluation.network.base_mva),
            reference.bus_id
        );
        for bus in &evaluation.solution.bus_states {
            let relative_angle_degrees =
                bus.voltage_angle_rad.to_degrees() - reference.voltage_angle_rad.to_degrees();
            tsv.push_str(&format!(
                "BUS\t{}\t{}\t{}\n",
                bus.bus_id,
                number(relative_angle_degrees),
                number(bus.net_injection_mw)
            ));
        }
        for branch in &evaluation.solution.branch_flows {
            tsv.push_str(&format!(
                "BRANCH\t{}\t{}\t{}\t{}\t{}\t{}\n",
                branch.branch_index + 1,
                branch.from_bus,
                branch.to_bus,
                if branch.in_service { 1 } else { 0 },
                number(branch.flow_from_to_mw),
                number(-branch.flow_from_to_mw)
            ));
        }
        tsv
    }

    #[test]
    fn exact_source_identity_and_unmodified_branch_receipt_pass() {
        let reference = sol_atlas_reference_tsv(CASE14);
        let report = compare(CASE14, &reference).expect("matching source and receipt");
        assert!(report.passed(), "{}", report.to_json());
        assert_eq!(report.bus_count, 14);
        assert_eq!(report.branch_count, 20);
        assert_eq!(report.field_comparison_count, 14 * 2 + 20 * 2 + 1);
    }

    #[test]
    fn a_single_mutated_branch_flow_is_a_reported_failure_not_a_pass() {
        let reference = sol_atlas_reference_tsv(CASE14);
        let mut lines = reference.lines().map(str::to_owned).collect::<Vec<_>>();
        let branch_index = lines
            .iter()
            .position(|line| line.starts_with("BRANCH\t1\t"))
            .expect("first branch exists");
        let mut fields = lines[branch_index].split('\t').map(str::to_owned).collect::<Vec<_>>();
        let original = fields[5].parse::<f64>().expect("numeric branch flow");
        fields[5] = number(original + 0.01);
        lines[branch_index] = fields.join("\t");
        let mutated = lines.join("\n") + "\n";

        let report = compare(CASE14, &mutated).expect("well-formed but wrong flow is reportable");
        assert!(!report.passed());
        assert!(report.mismatches.iter().any(|mismatch| {
            mismatch.element == "branch" && mismatch.field == "from_to_flow_mw"
        }));
    }

    #[test]
    fn source_identity_and_reference_version_drift_fail_closed() {
        let reference = sol_atlas_reference_tsv(CASE14);
        let wrong_identity = reference.replace(
            "META\tsource_git_blob_sha1\t2c8260fff0311c902c3d36dcabcbe861576fc97c",
            "META\tsource_git_blob_sha1\t0000000000000000000000000000000000000000",
        );
        assert!(
            compare(CASE14, &wrong_identity)
                .unwrap_err()
                .contains("source identity mismatch")
        );

        let wrong_version = reference.replace(
            "META\tmatpower_version\t8.1",
            "META\tmatpower_version\t8.0",
        );
        assert!(
            compare(CASE14, &wrong_version)
                .unwrap_err()
                .contains("version drift")
        );
    }

    #[test]
    fn missing_branch_reference_row_is_rejected_instead_of_truncated() {
        let reference = sol_atlas_reference_tsv(CASE14);
        let mut lines = reference.lines().map(str::to_owned).collect::<Vec<_>>();
        let branch_index = lines
            .iter()
            .position(|line| line.starts_with("BRANCH\t1\t"))
            .expect("first branch exists");
        lines.remove(branch_index);
        let mutated = lines.join("\n") + "\n";

        assert!(
            parse_reference_tsv(&mutated)
                .unwrap_err()
                .contains("branch rows are not complete and ordered")
        );
    }

}
