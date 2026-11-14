// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const CATALOGUE: &str = include_str!(
    "../../docs/research/home-material-intelligence/COMPONENT_CATALOGUE_SYNTHETIC_FIXTURE_V0.1.json"
);
const SUBSTITUTIONS: &str = include_str!(
    "../../docs/research/home-material-intelligence/SUBSTITUTION_MATRIX_SYNTHETIC_FIXTURE_V0.1.json"
);
const DEPENDENCIES: &str = include_str!(
    "../../docs/research/home-material-intelligence/DEPENDENCY_INVALIDATION_V0.1.json"
);
const CATALOGUE_SCHEMA: &str = include_str!(
    "../../docs/research/home-material-intelligence/COMPONENT_CATALOGUE_V0.1.schema.json"
);
const SUBSTITUTION_SCHEMA: &str = include_str!(
    "../../docs/research/home-material-intelligence/SUBSTITUTION_MATRIX_V0.1.schema.json"
);
const DEPENDENCY_SCHEMA: &str = include_str!(
    "../../docs/research/home-material-intelligence/DEPENDENCY_INVALIDATION_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn parse_json(source: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(source).map_err(|error| format!("{label} JSON parse failed: {error}"))
}

fn component<'a>(catalogue: &'a Value, component_id: &str) -> Option<&'a Value> {
    array_at(catalogue, "components")?
        .iter()
        .find(|entry| text_at(entry, "component_id") == Some(component_id))
}

fn property<'a>(component: &'a Value, property_id: &str) -> Option<&'a Value> {
    array_at(component, "properties")?
        .iter()
        .find(|entry| text_at(entry, "property_id") == Some(property_id))
}

fn interface_matches(component: &Value, required_interface_id: &str) -> bool {
    array_at(component, "interfaces").is_some_and(|interfaces| {
        interfaces.iter().any(|interface| {
            text_at(interface, "interface_id") == Some(required_interface_id)
                && text_at(interface, "revision") == Some("1")
        })
    })
}

fn find_assembly<'a>(catalogue: &'a Value, assembly_id: &str) -> Option<&'a Value> {
    array_at(catalogue, "assemblies")?
        .iter()
        .find(|entry| text_at(entry, "assembly_id") == Some(assembly_id))
}

fn numeric_property_value(value: &Value) -> Option<f64> {
    value.get("value").and_then(Value::as_f64)
}

fn threshold_met(value: f64, threshold: f64, comparison: &str) -> bool {
    match comparison {
        "MINIMUM" => value >= threshold,
        "MAXIMUM" => value <= threshold,
        _ => false,
    }
}

fn expected_property<'a>(assembly: &'a Value, property_id: &str) -> Option<&'a Value> {
    array_at(assembly, "required_properties")?
        .iter()
        .find(|entry| text_at(entry, "property_id") == Some(property_id))
}

fn evaluate_substitution(
    catalogue: &Value,
    assembly: &Value,
    substitution: &Value,
) -> Result<&'static str, String> {
    let from_id = text_at(substitution, "from_component_id")
        .ok_or_else(|| "substitution is missing from_component_id".to_owned())?;
    let to_id = text_at(substitution, "to_component_id")
        .ok_or_else(|| "substitution is missing to_component_id".to_owned())?;
    let interface_id = text_at(substitution, "required_interface_id")
        .ok_or_else(|| "substitution is missing required_interface_id".to_owned())?;
    let target = component(catalogue, to_id)
        .ok_or_else(|| format!("substitute component {to_id} is not registered"))?;
    if component(catalogue, from_id).is_none() {
        return Err(format!("baseline component {from_id} is not registered"));
    }
    if !interface_matches(target, interface_id) {
        return Ok("NOT_COMPARABLE");
    }

    let availability = array_at(target, "supplier_coverage")
        .and_then(|rows| rows.first())
        .and_then(|row| text_at(row, "availability_state"));
    if !matches!(availability, Some("SYNTHETIC_AVAILABLE" | "IN_STOCK_VERIFIED" | "LEAD_TIME_CONFIRMED")) {
        return Ok("BLOCKED_UNKNOWN");
    }

    let required = array_at(substitution, "required_properties")
        .ok_or_else(|| "substitution is missing required_properties".to_owned())?;
    for property_id in required {
        let Some(property_id) = property_id.as_str() else {
            return Err("required property ID must be text".to_owned());
        };
        let Some(requirement) = expected_property(assembly, property_id) else {
            return Err(format!("assembly does not declare required property {property_id}"));
        };
        let Some(candidate_property) = property(target, property_id) else {
            return Ok("BLOCKED_UNKNOWN");
        };
        let status = text_at(candidate_property, "status").unwrap_or("UNKNOWN");
        if status == "UNKNOWN" {
            return Ok("BLOCKED_UNKNOWN");
        }
        if status == "FAILED" {
            return Ok("REJECTED");
        }
        if !matches!(status, "SYNTHETIC_PASS" | "QUALIFIED_IN_SCOPE") {
            return Ok("BLOCKED_UNKNOWN");
        }

        let candidate_unit = text_at(candidate_property, "unit").unwrap_or_default();
        let required_unit = text_at(requirement, "unit").unwrap_or_default();
        if candidate_unit != required_unit {
            return Ok("NOT_COMPARABLE");
        }
        let Some(value) = numeric_property_value(candidate_property) else {
            return Ok("BLOCKED_UNKNOWN");
        };
        let Some(threshold) = requirement.get("threshold").and_then(Value::as_f64) else {
            return Err(format!("required property {property_id} has no numeric threshold"));
        };
        let comparison = text_at(requirement, "comparison").unwrap_or_default();
        if !threshold_met(value, threshold, comparison) {
            return Ok("REJECTED");
        }
    }

    // This is a synthetic known-answer disposition, not real product qualification.
    Ok("QUALIFIED_IN_SCOPE")
}

fn validate(
    catalogue: &Value,
    substitutions: &Value,
    dependencies: &Value,
    catalogue_schema: &Value,
    substitution_schema: &Value,
    dependency_schema: &Value,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    for (name, schema) in [
        ("catalogue", catalogue_schema),
        ("substitution", substitution_schema),
        ("dependency", dependency_schema),
    ] {
        if text_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
            errors.push(format!("{name} schema must declare Draft 2020-12"));
        }
        if text_at(schema, "type") != Some("object") {
            errors.push(format!("{name} schema root must be an object"));
        }
    }

    if text_at(catalogue, "maturity") != Some("SYNTHETIC_FIXTURE")
        || text_at(substitutions, "maturity") != Some("SYNTHETIC_FIXTURE")
        || text_at(dependencies, "maturity") != Some("SYNTHETIC_FIXTURE")
    {
        errors.push("all fixture artifacts must remain SYNTHETIC_FIXTURE".to_owned());
    }
    if text_at(substitutions, "study_id") != Some("HOME-MATERIAL-INTELLIGENCE-001")
        || text_at(dependencies, "study_id") != Some("HOME-MATERIAL-INTELLIGENCE-001")
    {
        errors.push("study ID mismatch".to_owned());
    }

    let mut component_ids = BTreeSet::new();
    for entry in array_at(catalogue, "components").into_iter().flatten() {
        let id = text_at(entry, "component_id").unwrap_or("<missing-component-id>");
        if !component_ids.insert(id.to_owned()) {
            errors.push(format!("duplicate component ID {id}"));
        }
        if text_at(entry, "product_revision").is_none() || text_at(entry, "manufacturer_ref").is_none() {
            errors.push(format!("{id}: exact product revision and source owner are required"));
        }
        if text_at(entry, "evidence_class") != Some("SYNTHETIC_FIXTURE") {
            errors.push(format!("{id}: fixture evidence class mismatch"));
        }
    }

    for assembly in array_at(catalogue, "assemblies").into_iter().flatten() {
        for row in array_at(assembly, "components").into_iter().flatten() {
            let id = text_at(row, "component_id").unwrap_or("<missing>");
            if !component_ids.contains(id) {
                errors.push(format!("assembly references unregistered component {id}"));
            }
            if row.get("quantity").and_then(Value::as_f64).is_none_or(|v| !v.is_finite() || v <= 0.0) {
                errors.push(format!("assembly component {id} has invalid quantity"));
            }
        }
    }

    for quote in array_at(catalogue, "supplier_quotes").into_iter().flatten() {
        let id = text_at(quote, "component_id").unwrap_or("<missing>");
        if !component_ids.contains(id) {
            errors.push(format!("quote references unregistered component {id}"));
        }
        if text_at(quote, "quote_date").is_none()
            || text_at(quote, "valid_through").is_none()
            || text_at(quote, "currency").is_none()
            || text_at(quote, "unit").is_none()
        {
            errors.push(format!("quote for {id} is missing date, currency or unit"));
        }
        if text_at(quote, "evidence_class") != Some("SYNTHETIC_FIXTURE") {
            errors.push(format!("quote for {id} is not marked synthetic"));
        }
    }

    for index in array_at(catalogue, "public_price_indices").into_iter().flatten() {
        if text_at(index, "use_restriction")
            != Some("ESCALATION_SENSITIVITY_ONLY_NOT_A_UNIT_PRICE")
        {
            errors.push("public price index has invalid use restriction".to_owned());
        }
        if index.get("unit_rate").is_some() || index.get("supplier_quote_id").is_some() {
            errors.push("public index must never be represented as a unit-price quote".to_owned());
        }
    }

    let dependency_invalidates: BTreeSet<String> = array_at(dependencies, "invalidates")
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    if dependency_invalidates.len() < 10 {
        errors.push("dependency manifest has fewer than ten invalidation targets".to_owned());
    }

    let mut dispositions = Vec::new();
    if let Some(items) = array_at(substitutions, "substitutions") {
        for item in items {
            let id = text_at(item, "substitution_id").unwrap_or("<missing-substitution-id>");
            let declared: BTreeSet<String> = array_at(item, "invalidation_set")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
            if !dependency_invalidates.is_subset(&declared) {
                errors.push(format!("{id}: substitution omits one or more required invalidations"));
            }
            let Some(assembly_id) = text_at(substitutions, "assembly_id") else {
                errors.push("substitution matrix has no assembly_id".to_owned());
                continue;
            };
            let Some(assembly) = find_assembly(catalogue, assembly_id) else {
                errors.push(format!("assembly {assembly_id} is not registered"));
                continue;
            };
            match evaluate_substitution(catalogue, assembly, item) {
                Ok(actual) => {
                    let expected = text_at(item, "expected_disposition").unwrap_or("UNKNOWN");
                    if actual != expected {
                        errors.push(format!("{id}: expected {expected}, evaluated {actual}"));
                    }
                    dispositions.push(json!({
                        "substitution_id":id,
                        "disposition":actual,
                        "basis":"SYNTHETIC_FIXTURE_ONLY"
                    }));
                }
                Err(error) => errors.push(format!("{id}: {error}")),
            }
        }
    } else {
        errors.push("substitution matrix is missing substitutions".to_owned());
    }

    if errors.is_empty() {
        Ok(json!({
            "audit_state":"HOME_MATERIAL_INTELLIGENCE_PASS",
            "study_id":"HOME-MATERIAL-INTELLIGENCE-001",
            "component_count":component_ids.len(),
            "assembly_count":array_at(catalogue,"assemblies").map_or(0,Vec::len),
            "supplier_quote_count":array_at(catalogue,"supplier_quotes").map_or(0,Vec::len),
            "public_price_indices_are_not_quotes":true,
            "invalidation_target_count":dependency_invalidates.len(),
            "substitutions":dispositions,
            "professional_or_authority_acceptance":"NOT_REQUESTED",
            "claim_ceiling":"SYNTHETIC_SUBSTITUTION_RULE_FIXTURE_ONLY"
        }))
    } else {
        Err(errors)
    }
}

fn fixture() -> (Value, Value, Value, Value, Value, Value) {
    (
        parse_json(CATALOGUE,"catalogue").expect("catalogue JSON"),
        parse_json(SUBSTITUTIONS,"substitutions").expect("substitution JSON"),
        parse_json(DEPENDENCIES,"dependencies").expect("dependency JSON"),
        parse_json(CATALOGUE_SCHEMA,"catalogue schema").expect("catalogue schema JSON"),
        parse_json(SUBSTITUTION_SCHEMA,"substitution schema").expect("substitution schema JSON"),
        parse_json(DEPENDENCY_SCHEMA,"dependency schema").expect("dependency schema JSON"),
    )
}

fn run() -> Result<Value,String> {
    let (catalogue,substitutions,dependencies,cs,ss,ds)=fixture();
    validate(&catalogue,&substitutions,&dependencies,&cs,&ss,&ds)
        .map_err(|errors|format!("HOME_MATERIAL_INTELLIGENCE_FAIL: {}",errors.join("; ")))
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg=="--help" || arg=="-h") {
        println!("home-material-qualifier [--self-test]");
        return;
    }
    match run() {
        Ok(report) => {
            println!("{}",serde_json::to_string_pretty(&report).unwrap_or_default());
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_and_negative_substitution_controls_match_expected_dispositions() {
        let (c,s,d,cs,ss,ds)=fixture();
        let report=validate(&c,&s,&d,&cs,&ss,&ds).expect("synthetic controls qualify");
        assert_eq!(report["component_count"],6);
        assert_eq!(report["assembly_count"],1);
        assert_eq!(report["supplier_quote_count"],2);
        let rows=report["substitutions"].as_array().expect("substitution rows");
        assert_eq!(rows.len(),3);
        assert_eq!(rows[0]["disposition"],"QUALIFIED_IN_SCOPE");
        assert_eq!(rows[1]["disposition"],"BLOCKED_UNKNOWN");
        assert_eq!(rows[2]["disposition"],"REJECTED");
    }

    #[test]
    fn stats_sa_index_cannot_be_used_as_a_unit_quote() {
        let (c, s, d, cs, ss, ds)=fixture();
        let mut c=c;
        c["public_price_indices"][0]["unit_rate"]=json!(113.6);
        assert!(validate(&c,&s,&d,&cs,&ss,&ds).is_err());
    }

    #[test]
    fn omitted_dependency_invalidation_is_rejected() {
        let (c, mut s, d, cs, ss, ds)=fixture();
        s["substitutions"][0]["invalidation_set"].as_array_mut().unwrap().pop();
        assert!(validate(&c,&s,&d,&cs,&ss,&ds).is_err());
    }

    #[test]
    fn missing_critical_property_cannot_be_promoted() {
        let (c, mut s, d, cs, ss, ds)=fixture();
        s["substitutions"][0]["expected_disposition"]=json!("BLOCKED_UNKNOWN");
        s["substitutions"][0]["to_component_id"]=json!("SYNTH-PANEL-B");
        let report=validate(&c,&s,&d,&cs,&ss,&ds);
        assert!(report.is_ok());
    }

    #[test]
    fn missing_component_reference_is_rejected() {
        let (mut c,s,d,cs,ss,ds)=fixture();
        c["assemblies"][0]["components"][0]["component_id"]=json!("UNREGISTERED-PART");
        assert!(validate(&c,&s,&d,&cs,&ss,&ds).is_err());
    }

    #[test]
    fn duplicate_component_identity_is_rejected() {
        let (mut c,s,d,cs,ss,ds)=fixture();
        let duplicate=c["components"][0].clone();
        c["components"].as_array_mut().unwrap().push(duplicate);
        assert!(validate(&c,&s,&d,&cs,&ss,&ds).is_err());
    }

    #[test]
    fn unit_mismatch_blocks_equivalence() {
        let (mut c,mut s,d,cs,ss,ds)=fixture();
        c["components"][1]["properties"][0]["unit"]=json!("hours");
        s["substitutions"][0]["expected_disposition"]=json!("NOT_COMPARABLE");
        let report=validate(&c,&s,&d,&cs,&ss,&ds);
        assert!(report.is_ok());
    }
}
