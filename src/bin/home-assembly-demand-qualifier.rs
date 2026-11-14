// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::{env, fs, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-assembly-demand/HOME_ASSEMBLY_DEMAND_SYNTHETIC_EXAMPLE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/home-assembly-demand/HOME_ASSEMBLY_DEMAND_PROFILE_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn parse_json(source: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(source).map_err(|error| format!("{label} is not valid JSON: {error}"))
}

fn validate(demand: &Value, schema: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();

    if text_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("assembly-demand schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(schema, "type") != Some("object") {
        errors.push("assembly-demand schema root must be an object".to_owned());
    }
    if text_at(demand, "schema_version") != Some("0.1.0")
        || text_at(demand, "study_id") != Some("HOME-ASSEMBLY-DEMAND-001")
    {
        errors.push("assembly-demand identity or schema version is invalid".to_owned());
    }
    if text_at(demand, "maturity") != Some("SYNTHETIC_REQUIREMENTS_ONLY")
        || text_at(demand, "execution_status") != Some("NOT_GEOMETRY_BOUND")
    {
        errors.push("synthetic demand packet was promoted beyond requirements-only maturity".to_owned());
    }

    let source = demand.get("source_design").unwrap_or(&Value::Null);
    if text_at(source, "brief_id") != Some("SYNTH-HV-FAMILY-001")
        || text_at(source, "candidate_set_id") != Some("SYNTH-HV-CASE-001")
        || text_at(source, "design_revision") != Some("HFC-SYNTH-REV-0")
        || source.get("source_pr").and_then(Value::as_u64) != Some(801)
    {
        errors.push("assembly demand is not bound to the exact synthetic home design source".to_owned());
    }

    let declared_candidates = array_at(demand, "candidate_ids").cloned().unwrap_or_default();
    let mut candidate_ids = BTreeSet::new();
    for value in &declared_candidates {
        match value.as_str() {
            Some(id) if candidate_ids.insert(id.to_owned()) => {}
            Some(id) => errors.push(format!("duplicate candidate binding {id}")),
            None => errors.push("candidate binding is not a string".to_owned()),
        }
    }
    let expected_candidates = ["SYNTH-HV-A", "SYNTH-HV-B"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    if candidate_ids.iter().map(String::as_str).collect::<BTreeSet<_>>() != expected_candidates {
        errors.push("P0 paired fixture must bind exactly SYNTH-HV-A and SYNTH-HV-B".to_owned());
    }

    let assemblies = array_at(demand, "assemblies").cloned().unwrap_or_default();
    if assemblies.len() != 8 {
        errors.push(format!(
            "paired P0 fixture must contain exactly eight assembly demands; found {}",
            assemblies.len()
        ));
    }

    let mut assembly_ids = BTreeSet::new();
    let mut component_role_ids = BTreeSet::new();
    let mut property_ids = BTreeSet::new();
    let mut property_rows = 0usize;
    let mut component_roles = 0usize;
    let mut dependency_rows = 0usize;

    for assembly in &assemblies {
        let id = text_at(assembly, "assembly_id").unwrap_or("<missing-assembly-id>");
        if !assembly_ids.insert(id.to_owned()) {
            errors.push(format!("duplicate assembly ID {id}"));
        }
        let candidate_id = text_at(assembly, "candidate_id").unwrap_or_default();
        if !candidate_ids.contains(candidate_id) {
            errors.push(format!("{id}: assembly references an undeclared candidate"));
        }

        if text_at(assembly, "assembly_role").is_none_or(str::is_empty) {
            errors.push(format!("{id}: assembly role is missing"));
        }

        let roles = array_at(assembly, "component_roles").cloned().unwrap_or_default();
        if roles.is_empty() {
            errors.push(format!("{id}: assembly has no component-role requirements"));
        }
        for role in &roles {
            let role_id = text_at(role, "role_id").unwrap_or_default();
            if role_id.is_empty() || !component_role_ids.insert(role_id.to_owned()) {
                errors.push(format!("{id}: component-role identity is missing or duplicated"));
            }
            if text_at(role, "product_selection_state") != Some("UNSELECTED")
                || !role.get("product_ref").is_some_and(Value::is_null)
            {
                errors.push(format!("{id}: component role implies an unsupported product selection"));
            }
            component_roles += 1;
        }

        let properties = array_at(assembly, "required_properties")
            .cloned()
            .unwrap_or_default();
        if properties.is_empty() {
            errors.push(format!("{id}: assembly has no property/approval requirements"));
        }
        for property in &properties {
            let property_id = text_at(property, "property_id").unwrap_or_default();
            if property_id.is_empty() || !property_ids.insert(property_id.to_owned()) {
                errors.push(format!("{id}: requirement property identity is missing or duplicated"));
            }
            if text_at(property, "requirement_ref").is_none_or(str::is_empty)
                || text_at(property, "evidence_owner").is_none_or(str::is_empty)
            {
                errors.push(format!("{id}: property requirement has no source owner/reference"));
            }
            if text_at(property, "state") != Some("UNKNOWN")
                || !property.get("target_value").is_some_and(Value::is_null)
                || !property.get("unit").is_some_and(Value::is_null)
                || !array_at(property, "evidence_refs").is_some_and(Vec::is_empty)
            {
                errors.push(format!("{id}: synthetic property was promoted or assigned invented evidence"));
            }
            property_rows += 1;
        }

        let quantity = assembly.get("quantity").unwrap_or(&Value::Null);
        if text_at(quantity, "state") != Some("NOT_COMPUTED")
            || !quantity.get("measured_quantity").is_some_and(Value::is_null)
            || !quantity.get("unit").is_some_and(Value::is_null)
            || !quantity.get("geometry_ref").is_some_and(Value::is_null)
            || !quantity.get("takeoff_method_ref").is_some_and(Value::is_null)
        {
            errors.push(format!("{id}: takeoff must remain NOT_COMPUTED until exact geometry and method exist"));
        }

        let product_selection = assembly.get("product_selection").unwrap_or(&Value::Null);
        if text_at(product_selection, "state") != Some("UNSELECTED")
            || !array_at(product_selection, "product_refs").is_some_and(Vec::is_empty)
        {
            errors.push(format!("{id}: no product is selected in the synthetic requirements fixture"));
        }

        let approval = assembly.get("approval").unwrap_or(&Value::Null);
        if text_at(approval, "state") != Some("NOT_CHECKED")
            || text_at(approval, "owner_ref").is_none_or(str::is_empty)
            || !array_at(approval, "evidence_refs").is_some_and(Vec::is_empty)
        {
            errors.push(format!("{id}: approval cannot be implied by a product-family or synthetic record"));
        }

        let manufacturing = assembly.get("manufacturing").unwrap_or(&Value::Null);
        if text_at(manufacturing, "state") != Some("NOT_ASSESSED")
            || !manufacturing.get("capability_ref").is_some_and(Value::is_null)
            || !manufacturing.get("process_ref").is_some_and(Value::is_null)
            || !array_at(manufacturing, "evidence_refs").is_some_and(Vec::is_empty)
        {
            errors.push(format!("{id}: manufacturability is not established by this demand contract"));
        }

        let supply = assembly.get("local_supply").unwrap_or(&Value::Null);
        if text_at(supply, "state") != Some("UNKNOWN")
            || !array_at(supply, "supplier_refs").is_some_and(Vec::is_empty)
            || !supply.get("lead_time_days").is_some_and(Value::is_null)
        {
            errors.push(format!("{id}: local supply must remain UNKNOWN without dated supplier evidence"));
        }

        let cost = assembly.get("cost").unwrap_or(&Value::Null);
        if text_at(cost, "state") != Some("NOT_ESTIMATED")
            || text_at(cost, "currency") != Some("ZAR")
            || !cost.get("amount").is_some_and(Value::is_null)
            || !cost.get("quote_ref").is_some_and(Value::is_null)
        {
            errors.push(format!("{id}: cost must remain NOT_ESTIMATED with null amount and quote"));
        }

        let repair = assembly.get("repair").unwrap_or(&Value::Null);
        if text_at(repair, "state") != Some("UNKNOWN")
            || array_at(repair, "requirements").is_none_or(Vec::is_empty)
            || !array_at(repair, "evidence_refs").is_some_and(Vec::is_empty)
        {
            errors.push(format!("{id}: repair claims need evidence or must remain UNKNOWN"));
        }

        let substitution = assembly.get("substitution").unwrap_or(&Value::Null);
        let dependencies = array_at(substitution, "dependency_claim_ids")
            .cloned()
            .unwrap_or_default();
        if text_at(substitution, "state") != Some("NOT_ASSESSED")
            || text_at(substitution, "equivalence_rule")
                != Some("REQUALIFY_ALL_DEPENDENT_CLAIMS")
            || dependencies.is_empty()
        {
            errors.push(format!("{id}: substitution policy is missing or prematurely qualified"));
        }
        for dependency in &dependencies {
            if dependency.as_str().is_none_or(str::is_empty) {
                errors.push(format!("{id}: substitution dependency claim ID is invalid"));
            }
        }
        dependency_rows += dependencies.len();

        if text_at(assembly, "claim_ceiling").is_none_or(str::is_empty) {
            errors.push(format!("{id}: assembly claim ceiling is missing"));
        }
    }

    if errors.is_empty() {
        Ok(json!({
            "audit_state":"HOME_ASSEMBLY_DEMAND_SCREEN_PASS",
            "study_id":"HOME-ASSEMBLY-DEMAND-001",
            "source_brief_id":text_at(source,"brief_id").unwrap_or_default(),
            "candidate_count":candidate_ids.len(),
            "assembly_count":assembly_ids.len(),
            "component_role_rows":component_roles,
            "property_requirement_rows":property_rows,
            "substitution_dependency_rows":dependency_rows,
            "products_selected":0,
            "quantities_computed":0,
            "approvals_checked":0,
            "manufacturing_capabilities_admitted":0,
            "local_supply_states":"UNKNOWN",
            "cost_state":"NOT_ESTIMATED",
            "claim_ceiling":"SYNTHETIC_ASSEMBLY_REQUIREMENTS_ONLY"
        }))
    } else {
        Err(errors)
    }
}

fn run(args: &[String]) -> Result<Value, String> {
    let mut input_path = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                index += 1;
                input_path = args.get(index).cloned();
            }
            "--self-test" => {}
            "--help" | "-h" => {
                return Ok(json!({
                    "usage":"home-assembly-demand-qualifier [--input PATH] [--self-test]",
                    "default":"embedded synthetic assembly-demand fixture",
                    "claim_ceiling":"requirements-only; no product, quantity, approval or physical evidence"
                }));
            }
            unknown => return Err(format!("unknown argument {unknown}")),
        }
        index += 1;
    }

    let demand = match input_path {
        Some(path) => {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("could not read assembly-demand input {path}: {error}"))?;
            parse_json(&content, "assembly-demand input")?
        }
        None => parse_json(FIXTURE, "assembly-demand fixture")?,
    };
    let schema = parse_json(SCHEMA, "assembly-demand schema")?;
    validate(&demand, &schema)
        .map_err(|errors| format!("HOME_ASSEMBLY_DEMAND_SCREEN_FAIL: {}", errors.join("; ")))
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            if text_at(&report, "audit_state") == Some("HOME_ASSEMBLY_DEMAND_SCREEN_PASS") {
                return;
            }
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
    process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Value, Value) {
        (
            parse_json(FIXTURE, "fixture").expect("fixture JSON"),
            parse_json(SCHEMA, "schema").expect("schema JSON"),
        )
    }

    #[test]
    fn paired_requirements_only_packet_is_admissible() {
        let (demand, schema) = fixture();
        let report = validate(&demand, &schema).expect("synthetic demand must qualify");
        assert_eq!(report["candidate_count"], 2);
        assert_eq!(report["assembly_count"], 8);
        assert_eq!(report["component_role_rows"], 40);
        assert_eq!(report["property_requirement_rows"], 42);
        assert_eq!(report["products_selected"], 0);
        assert_eq!(report["quantities_computed"], 0);
        assert_eq!(report["approvals_checked"], 0);
    }

    #[test]
    fn product_selection_is_not_implied_by_a_component_role() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["component_roles"][0]["product_selection_state"] =
            json!("SELECTED");
        demand["assemblies"][0]["component_roles"][0]["product_ref"] =
            json!("SUPPLIER-SKU-123");
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn product_catalogue_entry_cannot_be_injected() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["product_selection"]["product_refs"] =
            json!(["UNQUALIFIED-PRODUCT-REF"]);
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn quantity_without_exact_geometry_is_rejected() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["quantity"]["measured_quantity"] = json!(18.0);
        demand["assemblies"][0]["quantity"]["unit"] = json!("m2");
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn approval_cannot_be_promoted_from_synthetic_data() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["approval"]["state"] = json!("APPROVED");
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn local_supplier_presence_must_remain_unknown() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["local_supply"]["state"] = json!("AVAILABLE");
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn zero_filled_cost_is_rejected() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["cost"]["amount"] = json!(0.0);
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn substitution_cannot_be_promoted_without_requalification() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["substitution"]["state"] = json!("QUALIFIED");
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn missing_claim_dependency_is_rejected() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["substitution"]["dependency_claim_ids"] = json!([]);
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn repairability_claim_requires_evidence_or_unknown_state() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["repair"]["state"] = json!("REPAIRABLE");
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn duplicate_assembly_ids_are_rejected() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][1]["assembly_id"] = demand["assemblies"][0]["assembly_id"].clone();
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn stale_candidate_binding_is_rejected() {
        let (mut demand, schema) = fixture();
        demand["assemblies"][0]["candidate_id"] = json!("UNRELATED-CANDIDATE");
        assert!(validate(&demand, &schema).is_err());
    }

    #[test]
    fn scope_cannot_be_promoted_to_geometry_bound() {
        let (mut demand, schema) = fixture();
        demand["execution_status"] = json!("GEOMETRY_BOUND");
        assert!(validate(&demand, &schema).is_err());
    }
}
