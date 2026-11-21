// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Resident service to appliance/building interface contract qualifier.
//!
//! Validates that synthetic household-service profiles remain correctly typed,
//! revision-bound, and explicitly unknown until exact products, measurements,
//! standards and professional evidence are supplied. It does not assess product
//! safety, appliance performance, utility capacity, manufacturing or suitability.

use serde_json::{Value, json};
use std::{collections::BTreeSet, env, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-delivery-systems/home-service-interface-synthetic-v0.1.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn rows<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn is_null(value: Option<&Value>) -> bool {
    value.is_none_or(Value::is_null)
}

fn unique_ids(
    value: Option<&Vec<Value>>,
    key: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(items) = value else {
        errors.push(format!("{label} array is missing"));
        return ids;
    };
    for (i, item) in items.iter().enumerate() {
        let Some(id) = text_at(item, key) else {
            errors.push(format!("{label}[{i}] missing {key}"));
            continue;
        };
        if !ids.insert(id.to_owned()) {
            errors.push(format!("{label} duplicates {key}={id}"));
        }
    }
    ids
}

fn exact_reference_pair(left: &Value, right: &Value, key: &str) -> bool {
    let (Some(a), Some(b)) = (left.get(key), right.get(key)) else {
        return false;
    };
    text_at(a, "kind") == text_at(b, "kind")
        && text_at(a, "id") == text_at(b, "id")
        && a.get("revision") == b.get("revision")
        && a.get("digest") == b.get("digest")
}

fn expected_interfaces(service: &str) -> Option<BTreeSet<&'static str>> {
    let values: &[&str] = match service {
        "warewashing" => &[
            "spatial",
            "electrical",
            "water",
            "drainage",
            "thermal",
            "acoustic",
            "controls",
            "consumables",
            "service_access",
        ],
        "water_continuity" => &[
            "spatial",
            "electrical",
            "water",
            "water_quality",
            "drainage",
            "controls",
            "consumables",
            "service_access",
        ],
        "hot_water" => &[
            "spatial",
            "electrical",
            "water",
            "drainage",
            "thermal",
            "acoustic",
            "controls",
            "consumables",
            "service_access",
        ],
        "ventilation_filtration" => &[
            "spatial",
            "electrical",
            "thermal",
            "air",
            "acoustic",
            "controls",
            "consumables",
            "service_access",
        ],
        _ => return None,
    };
    Some(values.iter().copied().collect())
}

fn validate_profile(profile: &Value, shared_binding: &Value, errors: &mut Vec<String>) {
    let id = text_at(profile, "profile_id").unwrap_or("<missing-profile>");
    if text_at(profile, "schema_version") != Some("home-service-interface/0.1") {
        errors.push(format!(
            "{id}: unsupported home-service-interface schema version"
        ));
    }
    if text_at(profile, "maturity") != Some("synthetic_contract") {
        errors.push(format!(
            "{id}: synthetic appliance profile cannot be promoted to production maturity"
        ));
    }

    let service = text_at(profile, "service").unwrap_or("<missing-service>");
    let expected_profile = expected_interfaces(service);
    if expected_profile.is_none() {
        errors.push(format!("{id}: unsupported service {service}"));
    }
    for key in [
        "resident_brief",
        "candidate_design",
        "design_revision",
        "jurisdiction_profile",
    ] {
        if !exact_reference_pair(
            shared_binding,
            profile.get("source_binding").unwrap_or(&Value::Null),
            key,
        ) {
            errors.push(format!(
                "{id}: source_binding.{key} does not match the enclosing stack"
            ));
        }
    }

    let functional = profile.get("functional_unit").unwrap_or(&Value::Null);
    if text_at(functional, "state") != Some("unknown")
        || !is_null(functional.get("quantity"))
        || !is_null(functional.get("unit"))
        || !is_null(functional.get("resident_definition_ref"))
    {
        errors.push(format!(
            "{id}: synthetic functional unit must remain UNKNOWN with no quantity/unit"
        ));
    }

    let implementation = profile.get("implementation").unwrap_or(&Value::Null);
    if text_at(implementation, "kind") != Some("service_only")
        || text_at(implementation, "identity_state") != Some("unknown")
        || !is_null(implementation.get("product_or_prototype_ref"))
        || !is_null(implementation.get("configuration_revision"))
        || !is_null(implementation.get("firmware_revision"))
        || !is_null(implementation.get("component_bom_revision"))
    {
        errors.push(format!(
            "{id}: synthetic product identity/configuration must remain unbound"
        ));
    }
    if text_at(implementation, "owner_reference").is_none() {
        errors.push(format!("{id}: service profile lacks its owner reference"));
    }

    let interface_rows = rows(profile, "interfaces");
    let interface_ids = unique_ids(interface_rows, "kind", &format!("{id}.interfaces"), errors);
    let expected = expected_profile
        .unwrap_or_default()
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<String>>();
    if interface_ids != expected {
        errors.push(format!(
            "{id}: interface set differs from the service contract"
        ));
    }
    for interface in interface_rows.into_iter().flatten() {
        let kind = text_at(interface, "kind").unwrap_or("<missing-interface>");
        if text_at(interface, "state") != Some("unknown") {
            errors.push(format!(
                "{id}.{kind}: synthetic interface state must remain unknown"
            ));
        }
        for key in ["requirements", "measurements", "evidence_refs"] {
            if rows(interface, key).is_none_or(|values| !values.is_empty()) {
                errors.push(format!(
                    "{id}.{kind}: synthetic interface {key} must remain empty"
                ));
            }
        }
    }

    let operation = profile.get("local_operation").unwrap_or(&Value::Null);
    for field in [
        "manual_operation_state",
        "network_loss_behavior",
        "user_override_state",
        "cloud_dependency_state",
    ] {
        if text_at(operation, field) != Some("unknown") {
            errors.push(format!("{id}: local_operation.{field} must remain unknown"));
        }
    }
    if !is_null(operation.get("safety_interlock_owner_ref")) {
        errors.push(format!(
            "{id}: synthetic safety interlock owner must remain unbound"
        ));
    }

    let maintenance = profile.get("maintenance").unwrap_or(&Value::Null);
    for field in [
        "repair_path_state",
        "spares_state",
        "tools_state",
        "firmware_support_state",
        "end_of_life_state",
    ] {
        if text_at(maintenance, field) != Some("unknown") {
            errors.push(format!("{id}: maintenance.{field} must remain unknown"));
        }
    }
    if !is_null(maintenance.get("instructions_ref")) {
        errors.push(format!(
            "{id}: synthetic maintenance instructions must remain unbound"
        ));
    }

    let qualification = profile.get("qualification").unwrap_or(&Value::Null);
    if text_at(qualification, "status") != Some("not_assessed") {
        errors.push(format!(
            "{id}: synthetic product cannot be professionally or physically qualified"
        ));
    }
    if rows(qualification, "authority_refs").is_none_or(|refs| !refs.is_empty()) {
        errors.push(format!(
            "{id}: synthetic qualification authority_refs must be empty"
        ));
    }
    let limitations = rows(qualification, "limitations");
    if limitations.is_none_or(|items| items.len() < 3) {
        errors.push(format!(
            "{id}: synthetic profile must retain explicit limitation statements"
        ));
    }
    if !text_at(profile, "claim_ceiling").is_some_and(|claim| {
        claim.contains("does not establish product safety") && claim.contains("does not establish")
    }) {
        errors.push(format!(
            "{id}: claim ceiling omits product-safety/performance boundary"
        ));
    }
}

fn validate(stack: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(stack, "schema_version") != Some("home-service-stack/0.1") {
        errors.push("unsupported home-service-stack schema_version".to_owned());
    }
    if text_at(stack, "maturity") != Some("synthetic_contract") {
        errors
            .push("synthetic appliance stack cannot be promoted to production maturity".to_owned());
    }
    let profiles = rows(stack, "service_profiles");
    let profile_ids = unique_ids(profiles, "profile_id", "service_profiles", &mut errors);
    let services = profiles
        .into_iter()
        .flatten()
        .filter_map(|p| text_at(p, "service").map(str::to_owned))
        .collect::<BTreeSet<String>>();
    let expected_services = [
        "warewashing",
        "water_continuity",
        "hot_water",
        "ventilation_filtration",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<String>>();
    if services != expected_services || profile_ids.len() != 4 {
        errors.push(
            "synthetic stack must contain exactly the four declared service profiles".to_owned(),
        );
    }

    let binding = stack.get("source_binding").unwrap_or(&Value::Null);
    for profile in profiles.into_iter().flatten() {
        validate_profile(profile, binding, &mut errors);
    }

    if !text_at(stack, "claim_ceiling").is_some_and(|claim| {
        claim.contains("no real resident requirement")
            && claim.contains("no product identity")
            && claim.contains("no performance")
    }) {
        errors.push(
            "stack claim ceiling does not preserve synthetic product/requirement boundary"
                .to_owned(),
        );
    }
    let notes = rows(stack, "fixture_notes");
    if notes.is_none_or(|items| items.len() < 3) {
        errors.push("stack must include explicit synthetic fixture notes".to_owned());
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "HOME_SERVICE_INTERFACE_CONTRACT_PASS" } else { "HOME_SERVICE_INTERFACE_CONTRACT_FAIL" },
        "profile_id": text_at(stack, "profile_id").unwrap_or("<missing>"),
        "service_profile_count": profile_ids.len(),
        "service_count": services.len(),
        "product_identity": "UNKNOWN",
        "functional_units": "UNKNOWN",
        "interface_measurements": "NOT_SUPPLIED",
        "qualification_state": "NOT_ASSESSED",
        "physical_product_safety": "NOT_EVALUATED",
        "claim_ceiling": "INTERFACE_SET_AND_UNKNOWN_STATE_INTEGRITY_ONLY"
    });
    if errors.is_empty() {
        Ok(report)
    } else {
        Err(errors)
    }
}

fn load_fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("home-service stack fixture JSON must parse")
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-service-stack-qualifier [--self-test]");
        return;
    }
    match validate(&load_fixture()) {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        ),
        Err(errors) => {
            eprintln!("HOME_SERVICE_STACK_QUALIFIER_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_unknown_service_profiles_qualify_as_interface_contracts_only() {
        let report = validate(&load_fixture()).expect("synthetic household service stack");
        assert_eq!(report["service_profile_count"], 4);
        assert_eq!(report["product_identity"], "UNKNOWN");
        assert_eq!(report["qualification_state"], "NOT_ASSESSED");
        assert_eq!(report["physical_product_safety"], "NOT_EVALUATED");
    }

    #[test]
    fn missing_service_profile_is_rejected() {
        let mut stack = load_fixture();
        stack["service_profiles"].as_array_mut().unwrap().pop();
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn duplicate_service_profile_identity_is_rejected() {
        let mut stack = load_fixture();
        stack["service_profiles"][1]["profile_id"] =
            stack["service_profiles"][0]["profile_id"].clone();
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn service_kind_cannot_be_rebound_to_another_product_family() {
        let mut stack = load_fixture();
        stack["service_profiles"][0]["service"] = json!("hot_water");
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn mandatory_interface_class_cannot_be_omitted() {
        let mut stack = load_fixture();
        stack["service_profiles"][1]["interfaces"]
            .as_array_mut()
            .unwrap()
            .retain(|interface| text_at(interface, "kind") != Some("water_quality"));
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn product_identity_cannot_be_promoted_without_exact_product_binding() {
        let mut stack = load_fixture();
        stack["service_profiles"][0]["implementation"]["identity_state"] = json!("known");
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn measured_interface_cannot_appear_without_measurement_and_evidence() {
        let mut stack = load_fixture();
        stack["service_profiles"][0]["interfaces"][0]["state"] = json!("known");
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn synthetic_functional_unit_cannot_be_fabricated() {
        let mut stack = load_fixture();
        stack["service_profiles"][0]["functional_unit"]["quantity"] = json!(12.0);
        stack["service_profiles"][0]["functional_unit"]["unit"] = json!("cycles/week");
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn qualification_cannot_be_promoted_without_professional_evidence() {
        let mut stack = load_fixture();
        stack["service_profiles"][2]["qualification"]["status"] = json!("independently_qualified");
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn source_binding_cannot_drift_from_stack_revision() {
        let mut stack = load_fixture();
        stack["service_profiles"][0]["source_binding"]["design_revision"]["id"] =
            json!("DIFFERENT-REVISION");
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn network_loss_behavior_cannot_be_assumed() {
        let mut stack = load_fixture();
        stack["service_profiles"][3]["local_operation"]["network_loss_behavior"] =
            json!("continues_locally");
        assert!(validate(&stack).is_err());
    }

    #[test]
    fn maintenance_repair_path_cannot_be_claimed_without_instructions() {
        let mut stack = load_fixture();
        stack["service_profiles"][1]["maintenance"]["repair_path_state"] = json!("documented");
        assert!(validate(&stack).is_err());
    }
}
