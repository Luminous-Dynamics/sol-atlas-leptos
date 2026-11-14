// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Synthetic multi-family shared-service topology qualifier.
//!
//! This binary validates resource classes, declared network topology, node and
//! apartment-boundary identity, event references, measurement completeness and
//! resource-accounting claim states. It does not calculate demand, pressure,
//! flow, utility capacity, water quality, electrical safety, thermal delivery,
//! service availability, allocation fairness or building-controls behaviour.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-service-distribution-synthetic-v0.1.json"
);
const BUILDING: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-building-synthetic-v0.1.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn non_null_ref(value: &Value, key: &str) -> bool {
    value.get(key).is_some_and(|reference| {
        !reference.is_null()
            && text_at(reference, "kind").is_some()
            && text_at(reference, "id").is_some()
    })
}

fn unique_ids(rows: Option<&Vec<Value>>, key: &str, label: &str, errors: &mut Vec<String>) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(rows) = rows else {
        errors.push(format!("{label}: array is missing"));
        return ids;
    };
    for (index, row) in rows.iter().enumerate() {
        let Some(id) = text_at(row, key) else {
            errors.push(format!("{label}[{index}]: {key} is missing"));
            continue;
        };
        if !ids.insert(id.to_owned()) {
            errors.push(format!("{label}: duplicate {key} {id}"));
        }
    }
    ids
}

fn validate_quantity(value: &Value, label: &str, errors: &mut Vec<String>) {
    let state = text_at(value, "state").unwrap_or("<missing>");
    if !matches!(state, "unknown" | "not_computed" | "measured" | "estimated" | "disputed" | "not_applicable") {
        errors.push(format!("{label}: invalid quantity state {state}"));
        return;
    }
    match state {
        "unknown" | "not_computed" | "disputed" | "not_applicable" => {
            if !value.get("lower").is_some_and(Value::is_null)
                || !value.get("upper").is_some_and(Value::is_null)
                || !value.get("unit").is_some_and(Value::is_null)
            {
                errors.push(format!("{label}: unresolved quantity must not contain numeric bounds or a unit"));
            }
        }
        "measured" | "estimated" => {
            let lower = number_at(value, "lower");
            let upper = number_at(value, "upper");
            if lower.is_none_or(|n| !n.is_finite() || n < 0.0)
                || upper.is_none_or(|n| !n.is_finite() || n < 0.0)
                || lower.zip(upper).is_none_or(|(lo, hi)| lo > hi)
            {
                errors.push(format!("{label}: measured/estimated quantity has invalid bounds"));
            }
            if text_at(value, "unit").is_none_or(str::is_empty) {
                errors.push(format!("{label}: measured/estimated quantity has no unit"));
            }
            if !non_null_ref(value, "boundary_ref") || !non_null_ref(value, "evidence_ref") {
                errors.push(format!("{label}: measured/estimated quantity lacks boundary or evidence reference"));
            }
        }
        _ => {}
    }
}

fn validate(fixture: &Value, building: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(fixture, "schema_version") != Some("multifamily-service-distribution/0.1") {
        errors.push("service-distribution fixture has unsupported schema_version".to_owned());
    }
    if text_at(fixture, "maturity") != Some("synthetic_contract") {
        errors.push("service-distribution fixture cannot be promoted beyond synthetic_contract".to_owned());
    }
    if text_at(fixture, "claim_ceiling").is_none() {
        errors.push("service-distribution claim_ceiling is missing".to_owned());
    }

    let building_units = array_at(building, "unit_programmes")
        .into_iter()
        .flatten()
        .filter_map(|unit| {
            let unit_id = text_at(unit, "unit_id")?.to_owned();
            let space_id = text_at(unit, "space_id")?;
            let floor_id = array_at(building.get("building").unwrap_or(&Value::Null), "spaces")?
                .iter()
                .find(|space| text_at(space, "space_id") == Some(space_id))?;
            Some((
                unit_id,
                text_at(floor_id, "floor_id").unwrap_or("<missing-floor>").to_owned(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let building_unit_ids = building_units.keys().cloned().collect::<BTreeSet<_>>();
    if building_unit_ids.len() != 4 {
        errors.push(format!("P0 service fixture expects exactly four building unit identities; found {}", building_unit_ids.len()));
    }

    let resources = array_at(fixture, "resources");
    let resource_ids = unique_ids(resources, "resource_id", "resources", &mut errors);
    let mut resource_classes = BTreeMap::<String, String>::new();
    if let Some(resources) = resources {
        for resource in resources {
            let id = text_at(resource, "resource_id").unwrap_or("<missing-resource>");
            let class = text_at(resource, "resource_class").unwrap_or("<missing-class>");
            if !matches!(class, "potable_water" | "non_potable_water" | "electricity" | "useful_thermal" | "wastewater" | "communications") {
                errors.push(format!("resource {id} has invalid resource_class {class}"));
            }
            if !matches!(text_at(resource, "state"), Some("unknown" | "declared" | "measured" | "disputed" | "not_available")) {
                errors.push(format!("resource {id} has invalid state"));
            }
            if !matches!(text_at(resource, "quality_state"), Some("unknown" | "qualified" | "failed" | "not_applicable" | "disputed")) {
                errors.push(format!("resource {id} has invalid quality_state"));
            }
            if text_at(resource, "quality_state") == Some("qualified")
                && array_at(resource, "evidence_refs").is_none_or(Vec::is_empty)
            {
                errors.push(format!("resource {id} promotes quality without evidence"));
            }
            resource_classes.insert(id.to_owned(), class.to_owned());
        }
    }

    let nodes = array_at(fixture, "nodes");
    let node_ids = unique_ids(nodes, "node_id", "nodes", &mut errors);
    let mut node_records = BTreeMap::<String, (String, String, Option<String>, Option<String>)>::new();
    if let Some(nodes) = nodes {
        for node in nodes {
            let id = text_at(node, "node_id").unwrap_or("<missing-node>");
            let kind = text_at(node, "node_kind").unwrap_or("<missing-kind>");
            let resource_id = text_at(node, "resource_id").unwrap_or("<missing-resource>");
            let floor_id = text_at(node, "floor_id").map(str::to_owned);
            let unit_id = text_at(node, "unit_id").map(str::to_owned);
            if !matches!(kind, "source" | "storage" | "plant" | "distribution_zone" | "riser" | "branch" | "meter" | "unit_boundary" | "equipment" | "other") {
                errors.push(format!("node {id} has invalid node_kind"));
            }
            if !resource_ids.contains(resource_id) {
                errors.push(format!("node {id} references unknown resource {resource_id}"));
            }
            if !matches!(text_at(node, "state"), Some("unknown" | "declared" | "measured" | "disputed" | "isolated" | "out_of_service")) {
                errors.push(format!("node {id} has invalid state"));
            }
            if let Some(unit) = &unit_id {
                if !building_unit_ids.contains(unit) {
                    errors.push(format!("node {id} references unknown unit {unit}"));
                }
            }
            if kind == "unit_boundary" {
                let Some(unit) = unit_id.as_ref() else {
                    errors.push(format!("unit-boundary node {id} has no unit_id"));
                    continue;
                };
                if building_units.get(unit) != floor_id.as_ref() {
                    errors.push(format!("unit-boundary node {id} disagrees with building unit/floor identity"));
                }
            } else if unit_id.is_some() {
                errors.push(format!("non-unit-boundary node {id} must not claim unit ownership"));
            }
            let capacity = node.get("capacity").unwrap_or(&Value::Null);
            validate_quantity(capacity, &format!("node {id} capacity"), &mut errors);
            if text_at(capacity, "state") == Some("measured")
                && node.get("measurement_ref").is_none_or(Value::is_null)
            {
                errors.push(format!("node {id} measured capacity has no measurement_ref"));
            }
            node_records.insert(
                id.to_owned(),
                (kind.to_owned(), resource_id.to_owned(), floor_id, unit_id),
            );
        }
    }

    let edges = array_at(fixture, "edges");
    let edge_ids = unique_ids(edges, "edge_id", "edges", &mut errors);
    if let Some(edges) = edges {
        for edge in edges {
            let id = text_at(edge, "edge_id").unwrap_or("<missing-edge>");
            let from = text_at(edge, "from_node_id").unwrap_or("<missing-from>");
            let to = text_at(edge, "to_node_id").unwrap_or("<missing-to>");
            let relation = text_at(edge, "relation").unwrap_or("<missing-relation>");
            let state = text_at(edge, "state").unwrap_or("<missing-state>");
            let (Some(from_node), Some(to_node)) = (node_records.get(from), node_records.get(to)) else {
                errors.push(format!("edge {id} references an unknown endpoint"));
                continue;
            };
            if !matches!(relation, "resource_flow" | "power_dependency" | "control_dependency" | "measurement_of" | "isolation_of") {
                errors.push(format!("edge {id} has invalid relation"));
            }
            if !matches!(state, "unknown" | "declared" | "measured" | "disputed") {
                errors.push(format!("edge {id} has invalid state"));
            }
            if relation == "resource_flow" && from_node.1 != to_node.1 {
                errors.push(format!("edge {id} crosses resource classes on a resource_flow edge"));
            }
            if relation == "power_dependency"
                && resource_classes.get(&from_node.1).is_none_or(|class| class != "electricity")
            {
                errors.push(format!("edge {id} power_dependency must originate at an electricity resource"));
            }
            let loss = edge.get("loss").unwrap_or(&Value::Null);
            validate_quantity(loss, &format!("edge {id} loss"), &mut errors);
        }
    }

    let unit_services = array_at(fixture, "unit_services");
    let mut service_keys = BTreeSet::new();
    if unit_services.is_none() {
        errors.push("unit_services array is missing".to_owned());
    }
    for service in unit_services.into_iter().flatten() {
        let unit_id = text_at(service, "unit_id").unwrap_or("<missing-unit>");
        let service_id = text_at(service, "service_id").unwrap_or("<missing-service>");
        let resource_id = text_at(service, "resource_id").unwrap_or("<missing-resource>");
        let boundary_id = text_at(service, "boundary_node_id").unwrap_or("<missing-boundary>");
        let key = format!("{unit_id}:{service_id}");
        if !service_keys.insert(key.clone()) {
            errors.push(format!("duplicate unit service {key}"));
        }
        if !building_unit_ids.contains(unit_id) {
            errors.push(format!("unit service {key} references unknown building unit"));
        }
        if !resource_ids.contains(resource_id) {
            errors.push(format!("unit service {key} references unknown resource"));
        }
        match node_records.get(boundary_id) {
            Some((kind, node_resource, _, node_unit))
                if kind == "unit_boundary"
                    && node_resource == resource_id
                    && node_unit.as_deref() == Some(unit_id) => {}
            Some(_) => errors.push(format!("unit service {key} boundary identity/resource mismatch")),
            None => errors.push(format!("unit service {key} references unknown boundary node")),
        }
        validate_quantity(service.get("demand").unwrap_or(&Value::Null), &format!("{key} demand"), &mut errors);
        validate_quantity(service.get("delivered").unwrap_or(&Value::Null), &format!("{key} delivered"), &mut errors);
        if !matches!(text_at(service, "service_floor"), Some("unknown" | "resident_defined" | "professional_requirement" | "not_applicable" | "disputed")) {
            errors.push(format!("unit service {key} has invalid service_floor"));
        }
        let quality_state = text_at(service, "quality_state").unwrap_or("<missing>");
        if !matches!(quality_state, "unknown" | "qualified" | "failed" | "not_applicable" | "disputed") {
            errors.push(format!("unit service {key} has invalid quality_state"));
        }
        if quality_state == "qualified" && array_at(service, "evidence_refs").is_none_or(Vec::is_empty) {
            errors.push(format!("unit service {key} promotes quality without evidence"));
        }
    }
    if service_keys.len() != building_unit_ids.len() * 3 {
        errors.push(format!("P0 synthetic fixture expected three service classes per unit; found {} unit-service rows", service_keys.len()));
    }

    let events = array_at(fixture, "events");
    let event_ids = unique_ids(events, "event_id", "events", &mut errors);
    if let Some(events) = events {
        for event in events {
            let id = text_at(event, "event_id").unwrap_or("<missing-event>");
            if text_at(event, "state") != Some("declared_synthetic") {
                errors.push(format!("event {id} is not explicitly marked synthetic"));
            }
            if !event.get("probability").is_some_and(Value::is_null) {
                errors.push(format!("event {id} must not claim a failure probability"));
            }
            let start = event.get("start_s").and_then(Value::as_i64).unwrap_or(-1);
            if start < 0 {
                errors.push(format!("event {id} has invalid start_s"));
            }
            if let Some(duration) = event.get("duration_s").and_then(Value::as_i64) {
                if duration < 0 {
                    errors.push(format!("event {id} has negative duration_s"));
                }
            }
            for node_id in array_at(event, "affected_node_ids").into_iter().flatten().filter_map(Value::as_str) {
                if !node_ids.contains(node_id) {
                    errors.push(format!("event {id} affects unknown node {node_id}"));
                }
            }
        }
    }

    let policy = fixture.get("allocation_policy").unwrap_or(&Value::Null);
    let policy_state = text_at(policy, "state").unwrap_or("<missing>");
    if !matches!(policy_state, "unknown" | "draft" | "authorized" | "disputed" | "not_applicable") {
        errors.push("allocation policy has invalid state".to_owned());
    }
    if policy_state == "authorized"
        && (!non_null_ref(policy, "authority_ref") || !non_null_ref(policy, "rules_ref"))
    {
        errors.push("authorized allocation policy lacks authority/rules evidence".to_owned());
    }

    let accounting = fixture.get("accounting").unwrap_or(&Value::Null);
    let accounting_resource = text_at(accounting, "resource_id").unwrap_or("<missing>");
    if !resource_ids.contains(accounting_resource) {
        errors.push("accounting references unknown resource".to_owned());
    }
    for key in ["source_input", "storage_change", "unit_delivered", "declared_loss", "unresolved_residual"] {
        validate_quantity(accounting.get(key).unwrap_or(&Value::Null), &format!("accounting {key}"), &mut errors);
    }
    let conservation_state = text_at(accounting, "conservation_state").unwrap_or("<missing>");
    if !matches!(conservation_state, "unknown" | "not_computed" | "reconciled" | "disputed") {
        errors.push("accounting has invalid conservation_state".to_owned());
    }
    if conservation_state == "reconciled" {
        let all_measured = ["source_input", "storage_change", "unit_delivered", "declared_loss"]
            .iter()
            .all(|key| text_at(accounting.get(key).unwrap_or(&Value::Null), "state") == Some("measured"));
        if !all_measured || !non_null_ref(accounting, "observation_window") {
            errors.push("resource conservation cannot be reconciled without measured terms and observation window".to_owned());
        }
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "MULTIFAMILY_SERVICE_TOPOLOGY_PASS" } else { "MULTIFAMILY_SERVICE_TOPOLOGY_FAIL" },
        "resource_count": resource_ids.len(),
        "node_count": node_ids.len(),
        "edge_count": edge_ids.len(),
        "unit_count": building_unit_ids.len(),
        "unit_service_count": service_keys.len(),
        "event_count": event_ids.len(),
        "physical_delivery": "UNKNOWN",
        "resource_conservation": conservation_state,
        "allocation_policy": policy_state,
        "claim_ceiling": "SYNTHETIC_SERVICE_TOPOLOGY_AND_BOUNDARY_IDENTITY_ONLY"
    });
    if errors.is_empty() {
        Ok(report)
    } else {
        Err(errors)
    }
}

fn load_fixtures() -> (Value, Value) {
    (
        serde_json::from_str(FIXTURE).expect("service distribution fixture JSON"),
        serde_json::from_str(BUILDING).expect("multi-family building fixture JSON"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-multifamily-service-qualifier [--self-test]");
        return;
    }
    let (fixture, building) = load_fixtures();
    match validate(&fixture, &building) {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default()),
        Err(errors) => {
            eprintln!("HOME_MULTIFAMILY_SERVICE_QUALIFIER_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_service_topology_qualifies_without_performance_claims() {
        let (fixture, building) = load_fixtures();
        let report = validate(&fixture, &building).expect("service distribution fixture");
        assert_eq!(report["resource_count"], 5);
        assert_eq!(report["node_count"], 22);
        assert_eq!(report["edge_count"], 21);
        assert_eq!(report["unit_count"], 4);
        assert_eq!(report["unit_service_count"], 12);
        assert_eq!(report["event_count"], 6);
        assert_eq!(report["physical_delivery"], "UNKNOWN");
    }

    #[test]
    fn duplicate_node_identity_is_rejected() {
        let (mut fixture, building) = load_fixtures();
        fixture["nodes"][1]["node_id"] = fixture["nodes"][0]["node_id"].clone();
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn orphan_edge_endpoint_is_rejected() {
        let (mut fixture, building) = load_fixtures();
        fixture["edges"][0]["to_node_id"] = json!("NO-SUCH-NODE");
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn cross_carrier_resource_flow_is_rejected() {
        let (mut fixture, building) = load_fixtures();
        fixture["edges"][5]["to_node_id"] = json!("BOUNDARY-U-001-ELECTRICITY");
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn wrong_unit_boundary_is_rejected() {
        let (mut fixture, building) = load_fixtures();
        fixture["unit_services"][0]["boundary_node_id"] = json!("BOUNDARY-U-002-WATER");
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn unqualified_water_cannot_be_promoted_to_quality_pass() {
        let (mut fixture, building) = load_fixtures();
        fixture["resources"][0]["quality_state"] = json!("qualified");
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn unsupported_unit_quality_cannot_be_promoted() {
        let (mut fixture, building) = load_fixtures();
        fixture["unit_services"][0]["quality_state"] = json!("qualified");
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn measured_capacity_without_boundary_evidence_is_rejected() {
        let (mut fixture, building) = load_fixtures();
        fixture["nodes"][0]["capacity"]["state"] = json!("measured");
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn unknown_event_node_is_rejected() {
        let (mut fixture, building) = load_fixtures();
        fixture["events"][0]["affected_node_ids"] = json!(["NO-SUCH-NODE"]);
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn authorized_allocation_requires_authority_and_rules() {
        let (mut fixture, building) = load_fixtures();
        fixture["allocation_policy"]["state"] = json!("authorized");
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn conservation_cannot_be_reconciled_from_unknown_quantities() {
        let (mut fixture, building) = load_fixtures();
        fixture["accounting"]["conservation_state"] = json!("reconciled");
        assert!(validate(&fixture, &building).is_err());
    }

    #[test]
    fn duplicate_service_per_unit_is_rejected() {
        let (mut fixture, building) = load_fixtures();
        fixture["unit_services"][1]["service_id"] = fixture["unit_services"][0]["service_id"].clone();
        fixture["unit_services"][1]["unit_id"] = fixture["unit_services"][0]["unit_id"].clone();
        assert!(validate(&fixture, &building).is_err());
    }
}
