// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic, hash-bound scenario corpus generation and EPANET execution.
//!
//! The v0.1 runner intentionally targets small declared synthetic INP files.
//! It is not a general EPANET parser, leak model, or engineering decision service.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_SCENARIOS: usize = 1_000;
const MAX_ADDED_DEMAND_LPS: f64 = 20.0;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioCorpusPlan {
    pub schema_version: String,
    pub classification: String,
    pub plan_id: String,
    pub base_input: ScenarioBaseInput,
    pub observation_node_ids: Vec<String>,
    pub scenarios: Vec<ScenarioDefinition>,
    pub assumptions: Vec<String>,
    pub nonclaims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioBaseInput {
    pub path: String,
    pub sha256: String,
    pub byte_length: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioDefinition {
    pub scenario_id: String,
    pub ground_truth_class: String,
    pub target_junction_id: Option<String>,
    pub additional_demand_lps: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioArtifact {
    pub path: String,
    pub sha256: String,
    pub byte_length: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PressureReading {
    pub node_id: String,
    pub state: String,
    pub pressure_m: Option<f64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioRunRecord {
    pub scenario_id: String,
    pub ground_truth_class: String,
    pub target_junction_id: Option<String>,
    pub additional_demand_lps: f64,
    pub generated_input: ScenarioArtifact,
    pub run_state: String,
    pub solver_exit_code: Option<i32>,
    pub solver_output_sha256: Option<String>,
    pub report: Option<ScenarioArtifact>,
    pub binary_output: Option<ScenarioArtifact>,
    pub pressure_readings: Vec<PressureReading>,
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioCorpusRunManifest {
    pub schema_version: String,
    pub plan_id: String,
    pub plan_sha256: String,
    pub base_input_sha256: String,
    pub solver: ScenarioSolverIdentity,
    pub observation_node_ids: Vec<String>,
    pub scenario_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub run_state: String,
    pub scenarios: Vec<ScenarioRunRecord>,
    pub claim_ceiling: String,
    pub nonclaims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioSolverIdentity {
    pub executable_sha256: String,
    pub source_commit: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_identifier(value: &str) -> bool {
    value.len() >= 3
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte))
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn token_spans(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index == bytes.len() {
            break;
        }
        let start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        spans.push((start, index));
    }
    spans
}

fn section_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    (trimmed.starts_with('[') && trimmed.ends_with(']'))
        .then(|| trimmed[1..trimmed.len() - 1].trim().to_ascii_uppercase())
}

fn parse_junction_ids(input: &str) -> Result<HashSet<String>, String> {
    let mut section = String::new();
    let mut found_junction_section = false;
    let mut ids = HashSet::new();

    for line in input.lines() {
        if let Some(name) = section_name(line) {
            section = name;
            if section == "JUNCTIONS" {
                found_junction_section = true;
            }
            continue;
        }
        if section != "JUNCTIONS" {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let spans = token_spans(line);
        if spans.len() < 3 {
            return Err(format!("malformed JUNCTIONS row: {line:?}"));
        }
        let id = &line[spans[0].0..spans[0].1];
        if !valid_identifier(id) {
            return Err(format!("invalid junction identifier {id:?}"));
        }
        let elevation = line[spans[1].0..spans[1].1]
            .parse::<f64>()
            .map_err(|_| format!("invalid elevation in JUNCTIONS row: {line:?}"))?;
        let demand = line[spans[2].0..spans[2].1]
            .parse::<f64>()
            .map_err(|_| format!("invalid demand in JUNCTIONS row: {line:?}"))?;
        if !elevation.is_finite() || !demand.is_finite() || demand < 0.0 {
            return Err(format!("non-finite or negative junction data in {line:?}"));
        }
        if !ids.insert(id.to_owned()) {
            return Err(format!("duplicate junction identifier {id:?}"));
        }
    }

    if !found_junction_section || ids.is_empty() {
        return Err("base input must declare a non-empty [JUNCTIONS] section".to_owned());
    }
    Ok(ids)
}

fn validate_base_input(input: &str) -> Result<HashSet<String>, String> {
    if !input.is_ascii() {
        return Err("v0.1 runner accepts ASCII EPANET INP inputs only".to_owned());
    }
    let upper = input.to_ascii_uppercase();
    for required in ["[JUNCTIONS]", "[PIPES]", "[RESERVOIRS]", "[OPTIONS]", "[REPORT]", "[END]"] {
        if !upper.contains(required) {
            return Err(format!("base input is missing required section {required}"));
        }
    }
    if !upper.lines().any(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        fields.len() >= 2 && fields[0].eq_ignore_ascii_case("UNITS") && fields[1].eq_ignore_ascii_case("LPS")
    }) {
        return Err("base input must declare UNITS LPS".to_owned());
    }
    if !upper.lines().any(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        fields.len() >= 2 && fields[0].eq_ignore_ascii_case("NODES") && fields[1].eq_ignore_ascii_case("ALL")
    }) || !upper.lines().any(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        fields.len() >= 2 && fields[0].eq_ignore_ascii_case("LINKS") && fields[1].eq_ignore_ascii_case("ALL")
    }) {
        return Err("base input must request REPORT NODES ALL and LINKS ALL".to_owned());
    }
    parse_junction_ids(input)
}

fn format_number(value: f64) -> String {
    let mut formatted = format!("{value:.6}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    formatted
}

fn mutate_junction_demand(
    input: &str,
    target: Option<&str>,
    additional_demand_lps: f64,
) -> Result<String, String> {
    if !additional_demand_lps.is_finite()
        || !(0.0..=MAX_ADDED_DEMAND_LPS).contains(&additional_demand_lps)
    {
        return Err(format!(
            "additional_demand_lps must be finite and within [0, {MAX_ADDED_DEMAND_LPS}]"
        ));
    }
    if target.is_none() && additional_demand_lps != 0.0 {
        return Err("a positive withdrawal requires target_junction_id".to_owned());
    }
    if additional_demand_lps == 0.0 {
        return Ok(input.to_owned());
    }
    let target = target.ok_or_else(|| "missing target junction ID".to_owned())?;
    if !valid_identifier(target) {
        return Err(format!("invalid target_junction_id {target:?}"));
    }

    let mut section = String::new();
    let mut matches = 0usize;
    let mut output = String::with_capacity(input.len() + 32);

    for segment in input.split_inclusive('\n') {
        let (line, newline) = segment
            .strip_suffix('\n')
            .map(|line| (line, "\n"))
            .unwrap_or((segment, ""));
        if let Some(name) = section_name(line) {
            section = name;
            output.push_str(segment);
            continue;
        }
        if section == "JUNCTIONS" {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with(';') {
                let spans = token_spans(line);
                if spans.len() < 3 {
                    return Err(format!("malformed JUNCTIONS row: {line:?}"));
                }
                let id = &line[spans[0].0..spans[0].1];
                if id == target {
                    let demand_text = &line[spans[2].0..spans[2].1];
                    let baseline = demand_text
                        .parse::<f64>()
                        .map_err(|_| format!("invalid demand for target {target:?}"))?;
                    let candidate = baseline + additional_demand_lps;
                    if !candidate.is_finite() {
                        return Err(format!("non-finite candidate demand for {target:?}"));
                    }
                    output.push_str(&line[..spans[2].0]);
                    output.push_str(&format_number(candidate));
                    output.push_str(&line[spans[2].1..]);
                    output.push_str(newline);
                    matches += 1;
                    continue;
                }
            }
        }
        output.push_str(segment);
    }

    if matches != 1 {
        return Err(format!(
            "expected exactly one target junction row for {target:?}, found {matches}"
        ));
    }
    Ok(output)
}

fn parse_first_node_pressure(report: &str, node_id: &str) -> Result<f64, String> {
    let mut in_nodes = false;
    for line in report.lines() {
        if line.contains("Node Results at ") {
            in_nodes = true;
            continue;
        }
        if line.contains("Link Results at ") {
            in_nodes = false;
        }
        if !in_nodes {
            continue;
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.first().copied() == Some(node_id) {
            if fields.len() < 4 {
                return Err(format!("node {node_id} result row has fewer than four fields"));
            }
            let pressure = fields[3]
                .parse::<f64>()
                .map_err(|_| format!("node {node_id} pressure is not numeric: {:?}", fields[3]))?;
            if !pressure.is_finite() {
                return Err(format!("node {node_id} pressure is not finite"));
            }
            return Ok(pressure);
        }
    }
    Err(format!("node {node_id} has no pressure row in the first reported period"))
}

fn artifact(path: &Path, bytes: &[u8], output_root: &Path) -> Result<ScenarioArtifact, String> {
    let relative = path
        .strip_prefix(output_root)
        .map_err(|_| format!("artifact path is outside output root: {}", path.display()))?;
    Ok(ScenarioArtifact {
        path: relative.to_string_lossy().replace('\\', "/"),
        sha256: sha256(bytes),
        byte_length: bytes.len() as u64,
    })
}

fn validate_plan(
    plan: &ScenarioCorpusPlan,
    plan_path_arg: &str,
    base_path_arg: &str,
    input_bytes: &[u8],
    base_input: &str,
    solver_source_commit: &str,
) -> Result<(), String> {
    if plan.schema_version != "water-scenario-corpus-plan/v0.1" {
        return Err("schema_version must be water-scenario-corpus-plan/v0.1".to_owned());
    }
    if plan.classification != "synthetic_public" {
        return Err("classification must be synthetic_public".to_owned());
    }
    if !valid_identifier(&plan.plan_id) {
        return Err("plan_id is missing or invalid".to_owned());
    }
    if !valid_identifier(&plan.plan_id) {
        return Err("plan_id is missing or invalid".to_owned());
    }
    if plan.base_input.path != base_path_arg {
        return Err(format!(
            "CLI base input path {:?} differs from plan path {:?}",
            base_path_arg, plan.base_input.path
        ));
    }
    if Path::new(&plan.base_input.path).is_absolute()
        || plan.base_input.path.split('/').any(|part| part == "..")
        || plan.base_input.path.contains('\\')
    {
        return Err("base_input.path must be a safe relative POSIX path".to_owned());
    }
    if !valid_sha256(&plan.base_input.sha256) {
        return Err("base_input.sha256 must be 64 lowercase hexadecimal characters".to_owned());
    }
    if plan.base_input.byte_length != input_bytes.len() as u64 {
        return Err(format!(
            "base input length mismatch: plan={} actual={}",
            plan.base_input.byte_length,
            input_bytes.len()
        ));
    }
    if sha256(input_bytes) != plan.base_input.sha256 {
        return Err("base input SHA-256 does not match the frozen plan".to_owned());
    }
    if !valid_commit(solver_source_commit) {
        return Err("solver source commit must be a full lowercase 40-character Git SHA".to_owned());
    }
    if !valid_sha256(plan_path_arg) && false {
        return Err("unreachable".to_owned());
    }
    let junction_ids = validate_base_input(base_input)?;
    if plan.observation_node_ids.is_empty() {
        return Err("observation_node_ids must not be empty".to_owned());
    }
    let mut observation_ids = HashSet::new();
    for node_id in &plan.observation_node_ids {
        if !valid_epanet_identifier(node_id) || !observation_ids.insert(node_id) {
            return Err(format!("invalid or duplicate observation node ID {node_id:?}"));
        }
        if !junction_ids.contains(node_id) {
            return Err(format!("observation node {node_id:?} is not a junction in the base input"));
        }
    }
    if plan.scenarios.is_empty() || plan.scenarios.len() > MAX_SCENARIOS {
        return Err(format!("scenario count must be within [1, {MAX_SCENARIOS}]"));
    }
    if plan.assumptions.is_empty() || plan.assumptions.iter().any(|s| s.trim().is_empty()) {
        return Err("plan must include non-empty assumptions".to_owned());
    }
    if plan.nonclaims.is_empty() || plan.nonclaims.iter().any(|s| s.trim().is_empty()) {
        return Err("plan must include non-empty nonclaims".to_owned());
    }
    let mut scenario_ids = HashSet::new();
    let mut has_no_fault = false;
    let mut perturbed_targets = HashSet::new();
    for scenario in &plan.scenarios {
        if !valid_identifier(&scenario.scenario_id) || !scenario_ids.insert(&scenario.scenario_id) {
            return Err(format!("invalid or duplicate scenario ID {:?}", scenario.scenario_id));
        }
        if !scenario.additional_demand_lps.is_finite()
            || !(0.0..=MAX_ADDED_DEMAND_LPS).contains(&scenario.additional_demand_lps)
        {
            return Err(format!("scenario {} has invalid additional demand", scenario.scenario_id));
        }
        match (scenario.ground_truth_class.as_str(), scenario.target_junction_id.as_deref()) {
            ("no_fault", None) if scenario.additional_demand_lps == 0.0 => has_no_fault = true,
            ("synthetic_withdrawal", Some(target)) if scenario.additional_demand_lps > 0.0 => {
                if !junction_ids.contains(target) {
                    return Err(format!(
                        "scenario {} target {:?} is not a base junction",
                        scenario.scenario_id, target
                    ));
                }
                perturbed_targets.insert(target.to_owned());
            }
            _ => {
                return Err(format!(
                    "scenario {} must be no_fault with null target/zero demand or synthetic_withdrawal with target/positive demand",
                    scenario.scenario_id
                ));
            }
        }
    }
    if !has_no_fault {
        return Err("scenario plan must include a no_fault negative control".to_owned());
    }
    for node_id in &plan.observation_node_ids {
        if !perturbed_targets.contains(node_id) {
            return Err(format!(
                "scenario plan must perturb each observation junction at least once; missing {node_id}"
            ));
        }
    }
    Ok(())
}

fn safe_output_id(value: &str) -> bool {
    valid_identifier(value) && !value.contains("..")
}

/// Generate and execute the declared synthetic scenario corpus. Failures are
/// retained in the output manifest; the CLI returns nonzero if any case fails.
pub fn run_scenario_corpus(
    base_input_arg: &str,
    plan_path_arg: &str,
    solver_arg: &str,
    output_dir_arg: &str,
    solver_source_commit: &str,
) -> Result<ScenarioCorpusRunManifest, String> {
    let input_bytes = fs::read(base_input_arg)
        .map_err(|error| format!("cannot read base input {base_input_arg}: {error}"))?;
    let base_input = std::str::from_utf8(&input_bytes)
        .map_err(|_| "base input is not UTF-8".to_owned())?
        .to_owned();
    let plan_bytes = fs::read(plan_path_arg)
        .map_err(|error| format!("cannot read scenario plan {plan_path_arg}: {error}"))?;
    let plan: ScenarioCorpusPlan = serde_json::from_slice(&plan_bytes)
        .map_err(|error| format!("invalid scenario plan JSON: {error}"))?;
    validate_plan(
        &plan,
        plan_path_arg,
        base_input_arg,
        &input_bytes,
        &base_input,
        solver_source_commit,
    )?;

    let solver_path = PathBuf::from(solver_arg)
        .canonicalize()
        .map_err(|error| format!("cannot resolve solver {solver_arg}: {error}"))?;
    let solver_bytes = fs::read(&solver_path)
        .map_err(|error| format!("cannot hash solver {}: {error}", solver_path.display()))?;
    let solver_sha256 = sha256(&solver_bytes);
    let current_dir = std::env::current_dir().map_err(|error| format!("cannot determine working directory: {error}"))?;
    let output_root = {
        let path = PathBuf::from(output_dir_arg);
        if path.is_absolute() { path } else { current_dir.join(path) }
    };
    fs::create_dir_all(&output_root)
        .map_err(|error| format!("cannot create output directory {}: {error}", output_root.display()))?;
    let base_sha = sha256(&input_bytes);
    let plan_sha = sha256(&plan_bytes);
    let absolute_base = PathBuf::from(base_input_arg)
        .canonicalize()
        .map_err(|error| format!("cannot resolve base input: {error}"))?;
    let _ = absolute_base;

    let mut records = Vec::with_capacity(plan.scenarios.len());
    let mut completed_count = 0usize;
    for scenario in &plan.scenarios {
        if !safe_output_id(&scenario.scenario_id) {
            return Err(format!("unsafe scenario output ID {:?}", scenario.scenario_id));
        }
        let generated = mutate_junction_demand(
            &base_input,
            scenario.target_junction_id.as_deref(),
            scenario.additional_demand_lps,
        )?;
        let scenario_dir = output_root.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)
            .map_err(|error| format!("cannot create scenario output directory: {error}"))?;
        let input_path = scenario_dir.join("scenario.inp");
        fs::write(&input_path, generated.as_bytes())
            .map_err(|error| format!("cannot write generated INP: {error}"))?;
        let generated_input = artifact(&input_path, generated.as_bytes(), &output_root)?;

        let report_path = scenario_dir.join("scenario.rpt");
        let binary_path = scenario_dir.join("scenario.bin");
        let stdout_path = scenario_dir.join("solver-output.txt");
        let output = Command::new(&solver_path)
            .arg(&input_path)
            .arg(&report_path)
            .arg(&binary_path)
            .output();

        let mut failure_reasons = Vec::new();
        let mut solver_exit_code = None;
        let mut solver_output_sha256 = None;
        let mut report_artifact = None;
        let mut binary_artifact = None;
        let mut pressure_readings = Vec::with_capacity(plan.observation_node_ids.len());

        match output {
            Err(error) => failure_reasons.push(format!("solver spawn failed: {error}")),
            Ok(output) => {
                solver_exit_code = output.status.code();
                let mut captured = output.stdout;
                captured.extend_from_slice(b"\n--- STDERR ---\n");
                captured.extend_from_slice(&output.stderr);
                if let Err(error) = fs::write(&stdout_path, &captured) {
                    failure_reasons.push(format!("cannot write solver output: {error}"));
                } else {
                    solver_output_sha256 = Some(sha256(&captured));
                }
                if !output.status.success() {
                    failure_reasons.push(format!("solver exited with status {}", output.status));
                }

                match fs::read(&report_path) {
                    Ok(bytes) if !bytes.is_empty() => {
                        report_artifact = Some(artifact(&report_path, &bytes, &output_root)?);
                        match String::from_utf8(bytes) {
                            Ok(report_text) => {
                                for node_id in &plan.observation_node_ids {
                                    match parse_first_node_pressure(&report_text, node_id) {
                                        Ok(pressure) => pressure_readings.push(PressureReading {
                                            node_id: node_id.clone(),
                                            state: "measured".to_owned(),
                                            pressure_m: Some(pressure),
                                            reason: None,
                                        }),
                                        Err(error) => {
                                            failure_reasons.push(error.clone());
                                            pressure_readings.push(PressureReading {
                                                node_id: node_id.clone(),
                                                state: "unavailable".to_owned(),
                                                pressure_m: None,
                                                reason: Some(error),
                                            });
                                        }
                                    }
                                }
                            }
                            Err(_) => failure_reasons.push("report is not UTF-8".to_owned()),
                        }
                    }
                    _ => failure_reasons.push("solver report is missing or empty".to_owned()),
                }
                match fs::read(&binary_path) {
                    Ok(bytes) if !bytes.is_empty() => {
                        binary_artifact = Some(artifact(&binary_path, &bytes, &output_root)?);
                    }
                    _ => failure_reasons.push("solver binary output is missing or empty".to_owned()),
                }
            }
        }

        let run_state = if failure_reasons.is_empty()
            && pressure_readings.len() == plan.observation_node_ids.len()
            && pressure_readings.iter().all(|reading| reading.state == "measured")
        {
            completed_count += 1;
            "Completed"
        } else {
            "Failed"
        };

        records.push(ScenarioRunRecord {
            scenario_id: scenario.scenario_id.clone(),
            ground_truth_class: scenario.ground_truth_class.clone(),
            target_junction_id: scenario.target_junction_id.clone(),
            additional_demand_lps: scenario.additional_demand_lps,
            generated_input,
            run_state: run_state.to_owned(),
            solver_exit_code,
            solver_output_sha256,
            report: report_artifact,
            binary_output: binary_artifact,
            pressure_readings,
            failure_reasons,
        });
    }

    let failed_count = records.len() - completed_count;
    let manifest = ScenarioCorpusRunManifest {
        schema_version: "water-scenario-corpus-run/v0.1".to_owned(),
        plan_id: plan.plan_id,
        plan_sha256: plan_sha,
        base_input_sha256: base_sha,
        solver: ScenarioSolverIdentity {
            executable_sha256: solver_sha256,
            source_commit: solver_source_commit.to_owned(),
        },
        observation_node_ids: plan.observation_node_ids,
        scenario_count: records.len(),
        completed_count,
        failed_count,
        run_state: if failed_count == 0 { "Completed" } else { "CompletedWithFailures" }.to_owned(),
        scenarios: records,
        claim_ceiling: "synthetic_single_topology_hydraulic_pilot".to_owned(),
        nonclaims: plan.nonclaims,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("cannot serialize scenario corpus manifest: {error}"))?;
    fs::write(output_root.join("scenario-corpus-run.v0.1.json"), &manifest_bytes)
        .map_err(|error| format!("cannot write scenario corpus manifest: {error}"))?;
    fs::write(
        output_root.join("scenario-corpus-run.v0.1.sha256"),
        format!("{}  scenario-corpus-run.v0.1.json\n", sha256(&manifest_bytes)),
    )
    .map_err(|error| format!("cannot write manifest digest: {error}"))?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINI_INP: &str = "[TITLE]\nMini synthetic network\n\n[JUNCTIONS]\nJ1 50 15\nJ2 45 10\n\n[RESERVOIRS]\nR1 100\n\n[PIPES]\nP1 R1 J1 1000 200 100 0 OPEN\nP2 J1 J2 800 150 100 0 OPEN\n\n[OPTIONS]\nUNITS LPS\nHEADLOSS H-W\n\n[REPORT]\nNODES ALL\nLINKS ALL\n\n[END]\n";

    #[test]
    fn no_fault_scenario_preserves_exact_base_bytes() {
        assert_eq!(mutate_junction_demand(MINI_INP, None, 0.0).unwrap(), MINI_INP);
    }

    #[test]
    fn withdrawal_mutates_only_the_declared_junction_demand() {
        let changed = mutate_junction_demand(MINI_INP, Some("J2"), 2.5).unwrap();
        assert!(changed.contains("J1 50 15"));
        assert!(changed.contains("J2 45 12.5"));
        assert!(changed.contains("P2 J1 J2 800 150 100 0 OPEN"));
        assert_eq!(changed.lines().count(), MINI_INP.lines().count());
    }

    #[test]
    fn withdrawal_rejects_missing_or_ambiguous_targets() {
        assert!(mutate_junction_demand(MINI_INP, Some("J3"), 2.0).unwrap_err().contains("found 0"));
        let duplicate = MINI_INP.replace("J2 45 10", "J1 45 10");
        assert!(mutate_junction_demand(&duplicate, Some("J1"), 2.0).unwrap_err().contains("found 2"));
    }

    #[test]
    fn withdrawal_rejects_non_finite_and_out_of_range_values() {
        assert!(mutate_junction_demand(MINI_INP, Some("J1"), f64::NAN).is_err());
        assert!(mutate_junction_demand(MINI_INP, Some("J1"), 20.1).is_err());
        assert!(mutate_junction_demand(MINI_INP, None, 2.0).is_err());
    }

    #[test]
    fn parser_reads_pressure_only_from_node_result_section() {
        let report = "Node Results at 0:00:00\nNode Demand Head Pressure\nJ1 15.00 90.00 40.00\nLink Results at 0:00:00\nP1 25.00 1.25 2.50\n";
        assert!((parse_first_node_pressure(report, "J1").unwrap() - 40.0).abs() < 1e-9);
        assert!(parse_first_node_pressure(report, "J2").is_err());
    }

    #[test]
    fn parser_rejects_non_finite_pressure() {
        let report = "Node Results at 0:00:00\nJ1 15.00 90.00 NaN\nLink Results at 0:00:00\n";
        assert!(parse_first_node_pressure(report, "J1").is_err());
    }

    #[test]
    fn sha256_and_commit_contracts_fail_closed() {
        assert!(valid_sha256(&"a".repeat(64)));
        assert!(!valid_sha256(&"G".repeat(64)));
        assert!(valid_commit(&"a".repeat(40)));
        assert!(!valid_commit(&"z".repeat(40)));
    }
}
