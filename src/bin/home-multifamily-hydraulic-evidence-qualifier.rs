// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! External hydraulic-solver evidence bridge qualifier.
//!
//! Validates solver provenance, exact building/service-topology bindings,
//! solver-entity crosswalks, time-series integrity, calibration/validation
//! separation, and water-quality claim boundaries. It does not run EPANET,
//! OpenFlows, a hydraulic model, a water-quality model or any physical test.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const EVIDENCE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-hydraulic-run-evidence-synthetic-v0.1.json"
);
const SERVICE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-service-distribution-synthetic-v0.1.json"
);
const BUILDING: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-building-synthetic-v0.1.json"
);
const STRESS: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-service-stress-corpus-v0.1.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn rows<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn integer_at(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn is_null(value: Option<&Value>) -> bool {
    value.is_none_or(Value::is_null)
}

fn non_null_ref(value: Option<&Value>) -> bool {
    value.is_some_and(|reference| {
        !reference.is_null()
            && text_at(reference, "kind").is_some()
            && text_at(reference, "id").is_some()
            && reference.get("revision").is_some()
            && reference.get("digest").is_some()
    })
}

fn unique_ids(
    items: Option<&Vec<Value>>,
    key: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(items) = items else {
        errors.push(format!("{label} array is missing"));
        return ids;
    };
    for (index, item) in items.iter().enumerate() {
        let Some(id) = text_at(item, key) else {
            errors.push(format!("{label}[{index}] is missing {key}"));
            continue;
        };
        if !ids.insert(id.to_owned()) {
            errors.push(format!("{label} duplicates {key}={id}"));
        }
    }
    ids
}

fn valid_digest(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|digest| digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn valid_unit_for_quantity(kind: &str, unit: &str) -> bool {
    match kind {
        "pressure" => matches!(unit, "Pa" | "kPa" | "bar" | "mH2O" | "psi"),
        "head" | "headloss" | "tank_level" => matches!(unit, "m" | "mm" | "ft"),
        "flow" => matches!(unit, "L/s" | "m3/s" | "m3/h" | "L/min" | "gpm"),
        "velocity" => matches!(unit, "m/s" | "ft/s"),
        "pump_power" => matches!(unit, "W" | "kW" | "MW"),
        "pump_energy" => matches!(unit, "Wh" | "kWh" | "MWh"),
        "water_age" => matches!(unit, "s" | "min" | "h" | "d"),
        "constituent_concentration" => matches!(unit, "mg/L" | "ug/L" | "ppm"),
        "source_fraction" => matches!(unit, "1" | "%" ),
        _ => false,
    }
}

fn validate(
    evidence: &Value,
    service: &Value,
    building: &Value,
    stress: &Value,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(evidence, "schema_version") != Some("multifamily-hydraulic-run-evidence/0.1") {
        errors.push("unsupported hydraulic evidence schema_version".to_owned());
    }
    if text_at(evidence, "maturity") != Some("synthetic_contract") {
        errors.push("hydraulic evidence fixture must remain synthetic_contract".to_owned());
    }

    let execution_state = text_at(evidence, "execution_state").unwrap_or("<missing>");
    if !matches!(execution_state, "not_executed" | "completed" | "failed" | "invalidated") {
        errors.push(format!("invalid execution_state {execution_state}"));
    }

    let solver = evidence.get("solver").unwrap_or(&Value::Null);
    if text_at(solver, "family") != Some("hydraulic") {
        errors.push("solver family must be hydraulic".to_owned());
    }
    if text_at(solver, "product").is_none_or(str::is_empty)
        || text_at(solver, "vendor").is_none_or(str::is_empty)
        || text_at(solver, "version").is_none_or(str::is_empty)
        || text_at(solver, "adapter_version").is_none_or(str::is_empty)
    {
        errors.push("solver product/vendor/version/adapter identity is incomplete".to_owned());
    }
    if !matches!(text_at(solver, "product"), Some("EPANET" | "OpenFlows_Water" | "consultant_model" | "manufacturer_model" | "other")) {
        errors.push("unsupported hydraulic solver product".to_owned());
    }
    if solver.get("build_digest").is_some_and(|v| !v.is_null()) && !valid_digest(solver.get("build_digest")) {
        errors.push("solver build_digest must be 64 hexadecimal characters or null".to_owned());
    }

    let binding = evidence.get("model_binding").unwrap_or(&Value::Null);
    for field in ["building_ref", "service_topology_ref", "design_revision_ref", "jurisdiction_profile_ref"] {
        if !non_null_ref(binding.get(field)) {
            errors.push(format!("model_binding.{field} is missing or malformed"));
        }
    }
    let expected_building = text_at(building, "candidate_id").unwrap_or("<missing-building>");
    let expected_topology = text_at(service, "profile_id").unwrap_or("<missing-service-profile>");
    if text_at(binding.get("building_ref").unwrap_or(&Value::Null), "id") != Some(expected_building) {
        errors.push("hydraulic model is not bound to the exact building candidate".to_owned());
    }
    if text_at(binding.get("service_topology_ref").unwrap_or(&Value::Null), "id") != Some(expected_topology) {
        errors.push("hydraulic model is not bound to the exact service-topology profile".to_owned());
    }
    let source_binding = service.get("source_binding").unwrap_or(&Value::Null);
    if text_at(binding.get("design_revision_ref").unwrap_or(&Value::Null), "id")
        != text_at(source_binding.get("design_revision_ref").unwrap_or(&Value::Null), "id")
    {
        errors.push("hydraulic model design revision differs from the service topology binding".to_owned());
    }
    if text_at(binding.get("jurisdiction_profile_ref").unwrap_or(&Value::Null), "id")
        != text_at(source_binding.get("jurisdiction_profile_ref").unwrap_or(&Value::Null), "id")
    {
        errors.push("hydraulic model jurisdiction binding differs from the service topology".to_owned());
    }
    let scenario_id = text_at(binding, "scenario_id").unwrap_or("<missing-scenario>");
    let stress_case_ids = rows(stress, "cases")
        .into_iter()
        .flatten()
        .filter_map(|case| text_at(case, "case_id"))
        .collect::<BTreeSet<_>>();
    if !stress_case_ids.contains(scenario_id) {
        errors.push(format!("hydraulic model references unknown stress scenario {scenario_id}"));
    }
    for key in ["input_model_digest", "result_digest"] {
        let digest = binding.get(key);
        if !is_null(digest) && !valid_digest(digest) {
            errors.push(format!("model_binding.{key} must be 64 hexadecimal characters or null"));
        }
    }

    let configuration = evidence.get("run_configuration").unwrap_or(&Value::Null);
    let solver_status = text_at(configuration, "solver_status").unwrap_or("<missing>");
    match execution_state {
        "not_executed" => {
            if solver_status != "not_run" {
                errors.push("not_executed evidence must have solver_status=not_run".to_owned());
            }
            if !is_null(binding.get("input_model_digest")) || !is_null(binding.get("result_digest")) {
                errors.push("not_executed evidence cannot claim input or result digests".to_owned());
            }
            if rows(evidence, "result_series").is_some_and(|series| !series.is_empty()) {
                errors.push("not_executed evidence cannot contain solver result series".to_owned());
            }
        }
        "completed" => {
            if solver_status != "converged" {
                errors.push("completed execution requires solver_status=converged".to_owned());
            }
            if !valid_digest(binding.get("input_model_digest")) || !valid_digest(binding.get("result_digest")) {
                errors.push("completed execution requires exact input and result digests".to_owned());
            }
            if integer_at(configuration, "time_step_s").is_none_or(|step| step <= 0)
                || integer_at(configuration, "duration_s").is_none_or(|duration| duration <= 0)
                || integer_at(configuration, "end_elapsed_s").is_none_or(|end| end <= 0)
            {
                errors.push("completed execution requires positive time step, duration and end time".to_owned());
            }
        }
        "failed" if !matches!(solver_status, "failed" | "non_converged") => {
            errors.push("failed execution requires failed or non_converged solver_status".to_owned());
        }
        "invalidated" if solver_status != "invalidated" => {
            errors.push("invalidated execution requires solver_status=invalidated".to_owned());
        }
        _ => {}
    }

    let node_rows = rows(service, "nodes");
    let mut node_records = BTreeMap::<String, (String, Option<String>)>::new();
    for node in node_rows.into_iter().flatten() {
        if let (Some(id), Some(resource)) = (text_at(node, "node_id"), text_at(node, "resource_id")) {
            node_records.insert(id.to_owned(), (resource.to_owned(), text_at(node, "unit_id").map(str::to_owned)));
        }
    }
    let edge_rows = rows(service, "edges");
    let mut edge_records = BTreeMap::<String, String>::new();
    for edge in edge_rows.into_iter().flatten() {
        let edge_id = text_at(edge, "edge_id").unwrap_or("");
        let from = text_at(edge, "from_node_id").unwrap_or("");
        if let Some((resource, _)) = node_records.get(from) {
            edge_records.insert(edge_id.to_owned(), resource.clone());
        }
    }

    let mappings = rows(evidence, "mappings");
    let mapping_ids = unique_ids(mappings, "mapping_id", "mappings", &mut errors);
    let mut mapping_records = BTreeMap::<String, (Option<String>, String, String, String, Option<String>)>::new();
    let mut mapped_solver_entities = BTreeSet::new();
    let mut mapped_targets = BTreeSet::new();
    let mut bound_node_ids = BTreeSet::new();
    if let Some(mappings) = mappings {
        for mapping in mappings {
            let mapping_id = text_at(mapping, "mapping_id").unwrap_or("<missing-mapping>");
            let state = text_at(mapping, "mapping_state").unwrap_or("<missing-state>");
            let solver_entity_id = text_at(mapping, "solver_entity_id").map(str::to_owned);
            let node_id = text_at(mapping, "service_node_id");
            let edge_id = text_at(mapping, "service_edge_id");
            let resource_id = text_at(mapping, "resource_id").unwrap_or("<missing-resource>");
            let unit_id = text_at(mapping, "unit_id").map(str::to_owned);
            if node_id.is_some() == edge_id.is_some() {
                errors.push(format!("mapping {mapping_id} must target exactly one service node or edge"));
                continue;
            }
            if node_id.is_some() {
                let target = node_id.unwrap_or("");
                match node_records.get(target) {
                    Some((resource, unit)) => {
                        if resource != resource_id {
                            errors.push(format!("mapping {mapping_id} resource class does not match service node"));
                        }
                        if unit != &unit_id {
                            errors.push(format!("mapping {mapping_id} unit identity does not match service node"));
                        }
                        if !mapped_targets.insert(format!("node:{target}")) {
                            errors.push(format!("multiple solver mappings target service node {target}"));
                        }
                        if state == "bound" {
                            bound_node_ids.insert(target.to_owned());
                        }
                    }
                    None => errors.push(format!("mapping {mapping_id} references unknown service node {target}")),
                }
            } else if let Some(target) = edge_id {
                match edge_records.get(target) {
                    Some(resource) if resource == resource_id => {
                        if unit_id.is_some() {
                            errors.push(format!("mapping {mapping_id} edge-level mapping must not claim a unit"));
                        }
                        if !mapped_targets.insert(format!("edge:{target}")) {
                            errors.push(format!("multiple solver mappings target service edge {target}"));
                        }
                    }
                    Some(_) => errors.push(format!("mapping {mapping_id} resource class does not match service edge")),
                    None => errors.push(format!("mapping {mapping_id} references unknown service edge {target}")),
                }
            }
            match state {
                "planned" => {
                    if solver_entity_id.is_some() || !is_null(mapping.get("mapping_evidence_ref")) {
                        errors.push(format!("planned mapping {mapping_id} cannot claim a solver entity or binding evidence"));
                    }
                }
                "bound" => {
                    if solver_entity_id.as_deref().is_none_or(str::is_empty)
                        || !non_null_ref(mapping.get("mapping_evidence_ref"))
                    {
                        errors.push(format!("bound mapping {mapping_id} requires solver entity and mapping evidence"));
                    }
                    if let Some(entity) = &solver_entity_id {
                        if !mapped_solver_entities.insert(entity.clone()) {
                            errors.push(format!("solver entity {entity} is mapped more than once"));
                        }
                    }
                }
                "ambiguous" | "excluded" | "unknown" => {
                    if solver_entity_id.is_some() {
                        errors.push(format!("{state} mapping {mapping_id} cannot have a selected solver entity"));
                    }
                }
                _ => errors.push(format!("mapping {mapping_id} has invalid mapping_state {state}")),
            }
            mapping_records.insert(
                mapping_id.to_owned(),
                (solver_entity_id, state.to_owned(), resource_id.to_owned(), node_id.unwrap_or(edge_id.unwrap_or("")).to_owned(), unit_id),
            );
        }
    }

    if execution_state == "completed" {
        let required_water_units = rows(building, "unit_programmes")
            .into_iter()
            .flatten()
            .filter_map(|unit| text_at(unit, "unit_id"))
            .collect::<BTreeSet<_>>();
        for (node_id, (resource, unit)) in &node_records {
            if resource == "R-POTABLE-WATER" && unit.as_deref().is_some_and(|id| required_water_units.contains(id)) {
                if !bound_node_ids.contains(node_id) {
                    errors.push(format!("completed hydraulic run lacks a bound mapping for required water boundary {node_id}"));
                }
            }
        }
    }

    let series_rows = rows(evidence, "result_series");
    let series_ids = unique_ids(series_rows, "series_id", "result_series", &mut errors);
    let mut pressure_covered_units = BTreeSet::new();
    if let Some(series_rows) = series_rows {
        for series in series_rows {
            let series_id = text_at(series, "series_id").unwrap_or("<missing-series>");
            let mapping_id = text_at(series, "mapping_id").unwrap_or("<missing-mapping>");
            let solver_entity_id = text_at(series, "solver_entity_id").unwrap_or("<missing-entity>");
            let quantity_kind = text_at(series, "quantity_kind").unwrap_or("<missing-quantity>");
            let unit = text_at(series, "unit").unwrap_or("");
            let state = text_at(series, "value_state").unwrap_or("<missing-state>");
            let Some((mapped_solver, mapping_state, resource, target, mapped_unit)) = mapping_records.get(mapping_id) else {
                errors.push(format!("result series {series_id} references unknown mapping {mapping_id}"));
                continue;
            };
            if mapping_state != "bound" || mapped_solver.as_deref() != Some(solver_entity_id) {
                errors.push(format!("result series {series_id} does not match an exact bound solver mapping"));
            }
            if !valid_unit_for_quantity(quantity_kind, unit) {
                errors.push(format!("result series {series_id} has unsupported quantity/unit pair {quantity_kind}/{unit}"));
            }
            match state {
                "modelled" => {
                    if execution_state != "completed" {
                        errors.push(format!("modelled series {series_id} requires completed solver execution"));
                    }
                }
                "calibrated_modelled" => {
                    if execution_state != "completed"
                        || text_at(evidence.get("calibration").unwrap_or(&Value::Null), "state") != Some("calibrated_model")
                    {
                        errors.push(format!("calibrated series {series_id} lacks completed calibrated-model evidence"));
                    }
                }
                "measured" => errors.push(format!("solver result series {series_id} cannot be promoted to measured")),
                "disputed" | "invalidated" => {}
                _ => errors.push(format!("result series {series_id} has invalid value_state")),
            }
            if matches!(state, "modelled" | "calibrated_modelled") && !non_null_ref(series.get("evidence_ref")) {
                errors.push(format!("solver result series {series_id} lacks exact output evidence reference"));
            }
            let points = rows(series, "points");
            if points.is_none_or(Vec::is_empty) {
                errors.push(format!("result series {series_id} has no time points"));
                continue;
            }
            let mut prior_time = None;
            for point in points.into_iter().flatten() {
                let elapsed = integer_at(point, "elapsed_s");
                let lower = point.get("lower").and_then(Value::as_f64);
                let value = point.get("value").and_then(Value::as_f64);
                let upper = point.get("upper").and_then(Value::as_f64);
                if let Some(time) = elapsed {
                    if prior_time.is_some_and(|prior| time <= prior) {
                        errors.push(format!("result series {series_id} has duplicate or unordered timestamps"));
                    }
                    prior_time = Some(time);
                    if execution_state == "completed"
                        && integer_at(configuration, "duration_s").is_some_and(|duration| time > duration)
                    {
                        errors.push(format!("result series {series_id} exceeds run duration"));
                    }
                } else {
                    errors.push(format!("result series {series_id} has invalid elapsed_s"));
                }
                match (lower, value, upper) {
                    (Some(lo), Some(v), Some(hi))
                        if lo.is_finite() && v.is_finite() && hi.is_finite() && lo <= v && v <= hi => {}
                    _ => errors.push(format!("result series {series_id} has non-finite or unordered value bounds")),
                }
            }
            if quantity_kind == "pressure" {
                if let Some(unit_id) = mapped_unit {
                    if resource == "R-POTABLE-WATER" && target.starts_with("BOUNDARY-") {
                        pressure_covered_units.insert(unit_id.clone());
                    }
                }
            }
        }
    }
    if execution_state == "completed" {
        for service in rows(service, "unit_services").into_iter().flatten() {
            if text_at(service, "service_id") == Some("POTABLE-WATER-DELIVERY") {
                if let Some(unit) = text_at(service, "unit_id") {
                    if !pressure_covered_units.contains(unit) {
                        errors.push(format!("completed hydraulic run has no pressure result series for potable boundary of {unit}"));
                    }
                }
            }
        }
    }

    let calibration = evidence.get("calibration").unwrap_or(&Value::Null);
    let calibration_state = text_at(calibration, "state").unwrap_or("<missing>");
    if !matches!(calibration_state, "not_assessed" | "uncalibrated" | "calibrated_model" | "disputed") {
        errors.push("invalid calibration state".to_owned());
    }
    if calibration_state == "calibrated_model" {
        if execution_state != "completed"
            || !non_null_ref(calibration.get("calibration_dataset_ref"))
            || !non_null_ref(calibration.get("validation_dataset_ref"))
            || text_at(calibration, "calibration_window_id").is_none_or(str::is_empty)
            || text_at(calibration, "validation_window_id").is_none_or(str::is_empty)
            || text_at(calibration, "calibration_window_id") == text_at(calibration, "validation_window_id")
            || text_at(calibration, "residual_state") != Some("reported")
            || !non_null_ref(calibration.get("residual_evidence_ref"))
        {
            errors.push("calibrated_model requires distinct calibration/validation datasets and windows plus residual evidence".to_owned());
        }
    }

    let quality = evidence.get("water_quality").unwrap_or(&Value::Null);
    if text_at(quality, "potable_quality_credit") != Some("none_from_solver_output") {
        errors.push("hydraulic solver outputs cannot grant potable-quality credit".to_owned());
    }
    let mut sample_ids = BTreeSet::new();
    for sample in rows(quality, "laboratory_samples").into_iter().flatten() {
        let id = text_at(sample, "sample_id").unwrap_or("<missing-sample>");
        if !sample_ids.insert(id.to_owned()) {
            errors.push(format!("duplicate laboratory sample identity {id}"));
        }
        for field in ["sample_boundary_ref", "lab_ref", "method_ref", "chain_of_custody_ref"] {
            if !non_null_ref(sample.get(field)) {
                errors.push(format!("laboratory sample {id} lacks {field}"));
            }
        }
    }
    let has_quality_series = series_rows.into_iter().flatten().any(|series| {
        matches!(text_at(series, "quantity_kind"), Some("water_age" | "constituent_concentration" | "source_fraction"))
    });
    if has_quality_series && text_at(quality, "model_output_scope") != Some("modelled_constituent_or_age_only") {
        errors.push("water-quality solver series must remain labelled modelled_constituent_or_age_only".to_owned());
    }

    let warning_rows = rows(evidence, "warnings");
    let _warning_ids = unique_ids(warning_rows, "warning_id", "warnings", &mut errors);
    for warning in warning_rows.into_iter().flatten() {
        if text_at(warning, "severity") == Some("blocking") && execution_state == "completed" {
            errors.push(format!(
                "completed run cannot retain blocking warning {}",
                text_at(warning, "warning_id").unwrap_or("<missing>")
            ));
        }
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "HYDRAULIC_EVIDENCE_CONTRACT_PASS" } else { "HYDRAULIC_EVIDENCE_CONTRACT_FAIL" },
        "run_id": text_at(evidence, "run_id").unwrap_or("<missing>"),
        "execution_state": execution_state,
        "solver": text_at(solver, "product").unwrap_or("<missing>"),
        "service_mapping_count": mapping_ids.len(),
        "bound_mapping_count": mapping_records.values().filter(|record| record.1 == "bound").count(),
        "result_series_count": series_ids.len(),
        "calibration_state": calibration_state,
        "potable_quality_credit": "NONE_FROM_SOLVER_OUTPUT",
        "physical_hydraulic_result": "NOT_ESTABLISHED_BY_QUALIFIER",
        "claim_ceiling": "SOLVER_PROVENANCE_AND_TOPOLOGY_MAPPING_INTEGRITY_ONLY"
    });
    if errors.is_empty() { Ok(report) } else { Err(errors) }
}

fn load_fixtures() -> (Value, Value, Value, Value) {
    (
        serde_json::from_str(EVIDENCE).expect("hydraulic evidence fixture JSON"),
        serde_json::from_str(SERVICE).expect("service topology fixture JSON"),
        serde_json::from_str(BUILDING).expect("building fixture JSON"),
        serde_json::from_str(STRESS).expect("stress corpus fixture JSON"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-multifamily-hydraulic-evidence-qualifier [--self-test]");
        return;
    }
    let (evidence, service, building, stress) = load_fixtures();
    match validate(&evidence, &service, &building, &stress) {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default()),
        Err(errors) => {
            eprintln!("HOME_MULTIFAMILY_HYDRAULIC_EVIDENCE_QUALIFIER_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_run() -> (Value, Value, Value, Value) {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["execution_state"] = json!("completed");
        evidence["model_binding"]["input_model_digest"] = json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        evidence["model_binding"]["result_digest"] = json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        evidence["run_configuration"]["solver_status"] = json!("converged");
        evidence["run_configuration"]["time_step_s"] = json!(60);
        evidence["run_configuration"]["duration_s"] = json!(3600);
        evidence["run_configuration"]["end_elapsed_s"] = json!(3600);
        evidence["run_configuration"]["flow_units"] = json!("LPS");
        evidence["run_configuration"]["pressure_units"] = json!("kPa");
        evidence["run_configuration"]["time_zone"] = json!("Africa/Johannesburg");
        evidence["run_configuration"]["model_coordinate_basis"] = json!("synthetic local coordinate frame");
        evidence["run_configuration"]["vertical_datum"] = json!("synthetic datum");
        let pump_mapping = &mut evidence["mappings"][1];
        pump_mapping["solver_entity_id"] = json!("P-PUMP");
        pump_mapping["mapping_state"] = json!("bound");
        pump_mapping["mapping_evidence_ref"] = json!({
            "kind":"synthetic_mapping_record","id":"MAP-PUMP-EVIDENCE","revision":"0.1","digest":null
        });
        let mut series = Vec::new();
        for (index, unit_id) in ["U-001", "U-002", "U-003", "U-004"].iter().enumerate() {
            let mapping_index = 6 + index;
            let mapping_id = format!("MAP-{unit_id}-BOUNDARY");
            let solver_id = format!("J-{unit_id}");
            let mapping = &mut evidence["mappings"][mapping_index];
            mapping["solver_entity_id"] = json!(solver_id);
            mapping["mapping_state"] = json!("bound");
            mapping["mapping_evidence_ref"] = json!({
                "kind":"synthetic_mapping_record",
                "id":format!("MAP-{unit_id}-EVIDENCE"),
                "revision":"0.1",
                "digest":null
            });
            series.push(json!({
                "series_id":format!("SERIES-{unit_id}-PRESSURE"),
                "mapping_id":mapping_id,
                "solver_entity_id":solver_id,
                "quantity_kind":"pressure",
                "unit":"kPa",
                "value_state":"modelled",
                "time_basis":"elapsed_seconds",
                "points":[
                    {"elapsed_s":0,"lower":99.0,"value":100.0,"upper":101.0},
                    {"elapsed_s":60,"lower":98.0,"value":99.0,"upper":100.0}
                ],
                "evidence_ref":{
                    "kind":"synthetic_solver_output",
                    "id":format!("RESULT-{unit_id}"),
                    "revision":"0.1",
                    "digest":null
                }
            }));
        }
        evidence["result_series"] = Value::Array(series);
        (evidence, service, building, stress)
    }

    #[test]
    fn unexecuted_epanet_contract_fixture_qualifies_without_claiming_a_run() {
        let (evidence, service, building, stress) = load_fixtures();
        let report = validate(&evidence, &service, &building, &stress).expect("not-executed fixture");
        assert_eq!(report["execution_state"], "not_executed");
        assert_eq!(report["service_mapping_count"], 10);
        assert_eq!(report["bound_mapping_count"], 0);
        assert_eq!(report["result_series_count"], 0);
        assert_eq!(report["physical_hydraulic_result"], "NOT_ESTABLISHED_BY_QUALIFIER");
    }

    #[test]
    fn exact_completed_model_result_mapping_contract_is_valid() {
        let (evidence, service, building, stress) = complete_run();
        let report = validate(&evidence, &service, &building, &stress).expect("completed modelled result contract");
        assert_eq!(report["execution_state"], "completed");
        assert_eq!(report["bound_mapping_count"], 2);
        assert_eq!(report["result_series_count"], 1);
        assert_eq!(report["potable_quality_credit"], "NONE_FROM_SOLVER_OUTPUT");
    }

    #[test]
    fn wrong_building_binding_is_rejected() {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["model_binding"]["building_ref"]["id"] = json!("OTHER-BUILDING");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn wrong_service_topology_binding_is_rejected() {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["model_binding"]["service_topology_ref"]["id"] = json!("OTHER-TOPOLOGY");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn wrong_design_revision_binding_is_rejected() {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["model_binding"]["design_revision_ref"]["id"] = json!("STALE-DESIGN-REVISION");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn unknown_stress_scenario_is_rejected() {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["model_binding"]["scenario_id"] = json!("UNKNOWN-SCENARIO");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn duplicate_service_mapping_target_is_rejected() {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["mappings"][1]["service_node_id"] = json!("BOUNDARY-U-001-WATER");
        evidence["mappings"][1]["resource_id"] = json!("R-POTABLE-WATER");
        evidence["mappings"][1]["unit_id"] = json!("U-001");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn cross_resource_mapping_is_rejected() {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["mappings"][0]["resource_id"] = json!("R-ELECTRICITY");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn planned_mapping_cannot_claim_solver_entity() {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["mappings"][0]["solver_entity_id"] = json!("J-01");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn unexecuted_run_cannot_contain_result_series() {
        let (mut evidence, service, building, stress) = load_fixtures();
        evidence["result_series"] = json!([{"series_id":"FAKE"}]);
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn solver_result_cannot_be_promoted_to_measured() {
        let (mut evidence, service, building, stress) = complete_run();
        evidence["result_series"][0]["value_state"] = json!("measured");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn completed_run_requires_input_and_result_digests() {
        let (mut evidence, service, building, stress) = complete_run();
        evidence["model_binding"]["result_digest"] = Value::Null;
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn calibration_and_validation_windows_must_be_distinct() {
        let (mut evidence, service, building, stress) = complete_run();
        evidence["calibration"]["state"] = json!("calibrated_model");
        evidence["calibration"]["calibration_dataset_ref"] = json!({"kind":"dataset","id":"D1","revision":"1","digest":null});
        evidence["calibration"]["validation_dataset_ref"] = json!({"kind":"dataset","id":"D2","revision":"1","digest":null});
        evidence["calibration"]["calibration_window_id"] = json!("WINDOW-A");
        evidence["calibration"]["validation_window_id"] = json!("WINDOW-A");
        evidence["calibration"]["residual_state"] = json!("reported");
        evidence["calibration"]["residual_evidence_ref"] = json!({"kind":"residual-report","id":"R1","revision":"1","digest":null});
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn potable_quality_credit_is_rejected_even_with_quality_model_output() {
        let (mut evidence, service, building, stress) = complete_run();
        evidence["water_quality"]["potable_quality_credit"] = json!("qualified");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn non_monotonic_result_time_series_is_rejected() {
        let (mut evidence, service, building, stress) = complete_run();
        evidence["result_series"][0]["points"][1]["elapsed_s"] = json!(0);
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn invalid_result_unit_is_rejected() {
        let (mut evidence, service, building, stress) = complete_run();
        evidence["result_series"][0]["unit"] = json!("buckets");
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn blocking_solver_warning_prevents_completed_run() {
        let (mut evidence, service, building, stress) = complete_run();
        evidence["warnings"] = json!([{
            "warning_id":"W-BLOCK","severity":"blocking","code":"NON_CONVERGENCE",
            "statement":"solver did not converge","solver_entity_id":null
        }]);
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }

    #[test]
    fn missing_unit_boundary_pressure_coverage_blocks_completed_run() {
        let (mut evidence, service, building, stress) = complete_run();
        evidence["mappings"][6]["mapping_state"] = json!("planned");
        evidence["mappings"][6]["solver_entity_id"] = Value::Null;
        evidence["mappings"][6]["mapping_evidence_ref"] = Value::Null;
        evidence["result_series"].as_array_mut().unwrap().remove(0);
        assert!(validate(&evidence, &service, &building, &stress).is_err());
    }
}
