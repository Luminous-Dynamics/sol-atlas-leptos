// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Synthetic apartment premise-plumbing evidence qualifier.
//!
//! Validates apartment/use-point/asset identity, source/resource scope,
//! observation provenance, privacy and potable-quality claim boundaries.
//! It does not calculate hydraulics, water age, microbial risk, treatment
//! efficacy, potability, temperature safety or any public-health outcome.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-premise-plumbing-synthetic-v0.1.json"
);
const BUILDING: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-building-synthetic-v0.1.json"
);
const SERVICE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-service-distribution-synthetic-v0.1.json"
);
const HYDRAULIC: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-hydraulic-run-evidence-synthetic-v0.1.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn rows<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
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
    values: Option<&Vec<Value>>,
    key: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(values) = values else {
        errors.push(format!("{label} array is missing"));
        return ids;
    };
    for (index, value) in values.iter().enumerate() {
        let Some(id) = text_at(value, key) else {
            errors.push(format!("{label}[{index}] missing {key}"));
            continue;
        };
        if !ids.insert(id.to_owned()) {
            errors.push(format!("{label} duplicates {key}={id}"));
        }
    }
    ids
}

fn validate(
    fixture: &Value,
    building: &Value,
    service: &Value,
    hydraulic: &Value,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(fixture, "schema_version") != Some("multifamily-premise-plumbing/0.1") {
        errors.push("unsupported premise-plumbing schema_version".to_owned());
    }
    if text_at(fixture, "maturity") != Some("synthetic_contract") {
        errors.push("premise-plumbing fixture must remain synthetic_contract".to_owned());
    }

    let bindings = fixture.get("source_bindings").unwrap_or(&Value::Null);
    let expected = [
        ("building_ref", text_at(building, "candidate_id")),
        ("service_topology_ref", text_at(service, "profile_id")),
        ("hydraulic_evidence_ref", text_at(hydraulic, "run_id")),
    ];
    for (field, expected_id) in expected {
        let reference = bindings.get(field).unwrap_or(&Value::Null);
        if !non_null_ref(Some(reference)) {
            errors.push(format!("source binding {field} is missing or malformed"));
        } else if text_at(reference, "id") != expected_id {
            errors.push(format!("source binding {field} does not match exact upstream fixture"));
        }
    }
    let service_binding = service.get("source_binding").unwrap_or(&Value::Null);
    for field in ["design_revision_ref", "jurisdiction_profile_ref"] {
        let local = bindings.get(field).unwrap_or(&Value::Null);
        let upstream = service_binding.get(field).unwrap_or(&Value::Null);
        if !non_null_ref(Some(local)) || text_at(local, "id") != text_at(upstream, "id") {
            errors.push(format!("premise-plumbing {field} disagrees with service topology"));
        }
    }

    let building_units = rows(building, "unit_programmes")
        .into_iter()
        .flatten()
        .filter_map(|unit| {
            let unit_id = text_at(unit, "unit_id")?.to_owned();
            let space_id = text_at(unit, "space_id")?;
            let space = rows(building.get("building")?, "spaces")?
                .iter()
                .find(|space| text_at(space, "space_id") == Some(space_id))?;
            Some((unit_id, text_at(space, "floor_id")?.to_owned()))
        })
        .collect::<BTreeMap<_, _>>();
    let building_unit_ids = building_units.keys().cloned().collect::<BTreeSet<_>>();
    if building_unit_ids.len() != 4 {
        errors.push(format!("P0 fixture expects four building unit identities, found {}", building_unit_ids.len()));
    }

    let assets = rows(fixture, "assets");
    let asset_ids = unique_ids(assets, "asset_id", "assets", &mut errors);
    let mut asset_records = BTreeMap::<String, (String, Option<String>, Option<String>, BTreeSet<String>)>::new();
    for asset in assets.into_iter().flatten() {
        let id = text_at(asset, "asset_id").unwrap_or("<missing-asset>");
        let kind = text_at(asset, "asset_kind").unwrap_or("<missing-kind>");
        let resource = text_at(asset, "resource_class").unwrap_or("<missing-resource>");
        let unit_id = text_at(asset, "unit_id").map(str::to_owned);
        let floor_id = text_at(asset, "floor_id").map(str::to_owned);
        let served = rows(asset, "served_unit_ids")
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if !matches!(resource, "potable_water" | "non_potable_water" | "wastewater") {
            errors.push(format!("asset {id} has unsupported resource_class {resource}"));
        }
        if !matches!(
            kind,
            "incoming_service" | "storage" | "cold_pipe" | "hot_pipe"
                | "recirculation_loop" | "pump" | "treatment" | "filter"
                | "mixing_valve" | "meter" | "isolation_valve" | "fixture"
                | "sampling_port" | "other"
        ) {
            errors.push(format!("asset {id} has unsupported asset_kind {kind}"));
        }
        if let Some(unit) = &unit_id {
            if !building_unit_ids.contains(unit) {
                errors.push(format!("asset {id} references unknown apartment {unit}"));
            }
            if let (Some(expected_floor), Some(actual_floor)) = (building_units.get(unit), floor_id.as_ref()) {
                if expected_floor != actual_floor {
                    errors.push(format!("asset {id} floor does not match apartment {unit}"));
                }
            }
        }
        if served.iter().any(|unit| !building_unit_ids.contains(unit)) {
            errors.push(format!("asset {id} serves an unknown apartment"));
        }
        if unit_id.is_some() && served.len() != 1 {
            errors.push(format!("unit-owned asset {id} must serve exactly its owning apartment"));
        }
        let volume_state = text_at(asset, "volume_state").unwrap_or("<missing>");
        if !matches!(volume_state, "unknown" | "not_computed" | "measured" | "modelled" | "disputed") {
            errors.push(format!("asset {id} has invalid volume_state"));
        }
        if volume_state == "unknown" || volume_state == "not_computed" || volume_state == "disputed" {
            if !asset.get("volume_l").is_some_and(Value::is_null) {
                errors.push(format!("asset {id} has volume_l despite unresolved volume_state"));
            }
        }
        if volume_state == "measured" || volume_state == "modelled" {
            if asset.get("volume_l").and_then(Value::as_f64).is_none_or(|value| !value.is_finite() || value < 0.0) {
                errors.push(format!("asset {id} has invalid numeric volume"));
            }
            if asset.get("geometry_ref").is_none_or(Value::is_null) {
                errors.push(format!("asset {id} promotes volume without exact geometry reference"));
            }
            if array_is_empty(asset, "evidence_refs") {
                errors.push(format!("asset {id} promotes volume without evidence reference"));
            }
            if text_at(fixture, "maturity") == Some("synthetic_contract") {
                errors.push(format!("synthetic fixture cannot claim measured/modelled volume for asset {id}"));
            }
        }
        asset_records.insert(id.to_owned(), (kind.to_owned(), resource.to_owned(), unit_id, served));
    }
    for asset in assets.into_iter().flatten() {
        let id = text_at(asset, "asset_id").unwrap_or("<missing-asset>");
        for parent in rows(asset, "parent_asset_ids").into_iter().flatten().filter_map(Value::as_str) {
            if !asset_ids.contains(parent) {
                errors.push(format!("asset {id} references unknown parent asset {parent}"));
            }
        }
    }

    let use_points = rows(fixture, "use_points");
    let use_point_ids = unique_ids(use_points, "use_point_id", "use_points", &mut errors);
    let mut use_point_records = BTreeMap::<String, (String, String, String)>::new();
    for point in use_points.into_iter().flatten() {
        let id = text_at(point, "use_point_id").unwrap_or("<missing-use-point>");
        let unit = text_at(point, "unit_id").unwrap_or("<missing-unit>");
        let floor = text_at(point, "floor_id").unwrap_or("<missing-floor>");
        let resource = text_at(point, "resource_class").unwrap_or("<missing-resource>");
        if building_units.get(unit).map(String::as_str) != Some(floor) {
            errors.push(format!("use point {id} has unknown unit or mismatched floor"));
        }
        if text_at(point, "resident_scope") != Some("synthetic_identity_only") {
            errors.push(format!("use point {id} exceeds synthetic resident-scope ceiling"));
        }
        let asset_refs = rows(point, "asset_ids");
        if asset_refs.is_none_or(Vec::is_empty) {
            errors.push(format!("use point {id} has no plumbing asset identity"));
        }
        for asset_id in asset_refs.into_iter().flatten().filter_map(Value::as_str) {
            match asset_records.get(asset_id) {
                Some((kind, asset_resource, unit_id, served))
                    if kind == "fixture"
                        && asset_resource == resource
                        && (unit_id.as_deref() == Some(unit) || (unit_id.is_none() && served.contains(unit))) => {}
                Some(_) => errors.push(format!("use point {id} asset {asset_id} has mismatched resource/unit identity")),
                None => errors.push(format!("use point {id} references unknown asset {asset_id}")),
            }
        }
        if !matches!(resource, "potable_water" | "non_potable_water") {
            errors.push(format!("use point {id} has invalid resource class"));
        }
        use_point_records.insert(id.to_owned(), (unit.to_owned(), floor.to_owned(), resource.to_owned()));
    }

    let plans = rows(fixture, "observation_plan");
    let plan_ids = unique_ids(plans, "plan_id", "observation_plan", &mut errors);
    for plan in plans.into_iter().flatten() {
        let id = text_at(plan, "plan_id").unwrap_or("<missing-plan>");
        let use_point = text_at(plan, "use_point_id").unwrap_or("<missing-use-point>");
        if !use_point_records.contains_key(use_point) {
            errors.push(format!("observation plan {id} references unknown use point {use_point}"));
        }
        let state = text_at(plan, "schedule_state").unwrap_or("<missing>");
        if !matches!(state, "not_scheduled" | "draft" | "approved" | "withdrawn" | "disputed") {
            errors.push(format!("observation plan {id} has invalid schedule_state"));
        }
        if state == "approved"
            && (!non_null_ref(plan.get("instrument_ref"))
                || !non_null_ref(plan.get("method_ref"))
                || !non_null_ref(plan.get("collection_authority_ref")))
        {
            errors.push(format!("approved observation plan {id} lacks instrument/method/authority evidence"));
        }
        if state == "not_scheduled" && text_at(plan, "privacy_scope") == Some("resident_opt_in")
            && !non_null_ref(plan.get("collection_authority_ref"))
        {
            // No collection is scheduled, so no resident consent is presumed; the plan remains non-operative.
        }
    }

    let observations = rows(fixture, "observations");
    let observation_ids = unique_ids(observations, "observation_id", "observations", &mut errors);
    for observation in observations.into_iter().flatten() {
        let id = text_at(observation, "observation_id").unwrap_or("<missing-observation>");
        let point_id = text_at(observation, "use_point_id").unwrap_or("<missing-use-point>");
        let state = text_at(observation, "evidence_state").unwrap_or("<missing-state>");
        if !use_point_records.contains_key(point_id) {
            errors.push(format!("observation {id} references unknown use point {point_id}"));
        }
        match state {
            "modelled" => {
                if !non_null_ref(observation.get("evidence_ref")) {
                    errors.push(format!("modelled observation {id} lacks model evidence"));
                }
            }
            "measured" => {
                for field in ["instrument_ref","calibration_ref","evidence_ref"] {
                    if !non_null_ref(observation.get(field)) {
                        errors.push(format!("measured observation {id} lacks {field}"));
                    }
                }
                if text_at(observation, "observation_kind") == Some("water_temperature")
                    && (observation.get("value").and_then(Value::as_f64).is_none()
                        || text_at(observation, "unit").is_none_or(str::is_empty)
                        || observation.get("uncertainty").and_then(Value::as_f64).is_none())
                {
                    errors.push(format!("measured temperature observation {id} lacks value/unit/uncertainty"));
                }
            }
            "lab_reported" => {
                for field in ["lab_ref","method_ref","chain_of_custody_ref","evidence_ref"] {
                    if !non_null_ref(observation.get(field)) {
                        errors.push(format!("lab-reported observation {id} lacks {field}"));
                    }
                }
            }
            "self_reported" => {
                if !non_null_ref(observation.get("consent_ref")) {
                    errors.push(format!("self-reported observation {id} lacks resident consent reference"));
                }
            }
            "professionally_assessed" => {
                if !non_null_ref(observation.get("evidence_ref")) {
                    errors.push(format!("professional observation {id} lacks exact evidence reference"));
                }
            }
            "disputed" | "invalidated" => {}
            _ => errors.push(format!("observation {id} has invalid evidence_state {state}")),
        }
        if state == "measured" && observation.get("value").is_some_and(|v| v.is_string()) {
            errors.push(format!("measured numeric observation {id} cannot use a string value"));
        }
    }

    let claims = rows(fixture, "water_quality_claims");
    let claim_ids = unique_ids(claims, "claim_id", "water_quality_claims", &mut errors);
    for claim in claims.into_iter().flatten() {
        let id = text_at(claim, "claim_id").unwrap_or("<missing-claim>");
        let point = text_at(claim, "use_point_id").unwrap_or("<missing-use-point>");
        if !use_point_records.contains_key(point) {
            errors.push(format!("water quality claim {id} references unknown use point"));
        }
        if text_at(claim, "potable_credit") != Some("none") {
            errors.push(format!("synthetic premise-plumbing claim {id} cannot grant potable credit"));
        }
        if !matches!(text_at(claim, "claim_state"), Some("unknown" | "not_assessed" | "disputed" | "invalidated")) {
            errors.push(format!("water quality claim {id} has invalid claim_state"));
        }
        if text_at(claim, "basis") != Some("no_water_quality_adjudication") {
            errors.push(format!("water quality claim {id} exceeds no-adjudication claim ceiling"));
        }
    }

    let policy = fixture.get("collection_policy").unwrap_or(&Value::Null);
    if text_at(policy, "default_collection") != Some("none")
        || policy.get("audio_video_allowed").and_then(Value::as_bool) != Some(false)
        || policy.get("continuous_occupancy_allowed").and_then(Value::as_bool) != Some(false)
        || policy.get("resident_withdrawal_supported").and_then(Value::as_bool) != Some(true)
    {
        errors.push("collection policy violates default privacy/withdrawal guard".to_owned());
    }
    if observations.is_some_and(|items| !items.is_empty()) && text_at(policy, "retention_state") != Some("approved") {
        errors.push("observations cannot be retained without an approved retention policy".to_owned());
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "PREMISE_PLUMBING_IDENTITY_PASS" } else { "PREMISE_PLUMBING_IDENTITY_FAIL" },
        "profile_id": text_at(fixture, "profile_id").unwrap_or("<missing>"),
        "asset_count": asset_ids.len(),
        "use_point_count": use_point_ids.len(),
        "observation_plan_count": plan_ids.len(),
        "observation_count": observation_ids.len(),
        "water_quality_claim_count": claim_ids.len(),
        "water_quality_credit": "NONE",
        "potable_water_state": "NOT_ASSESSED",
        "microbial_risk": "NOT_ASSESSED",
        "physical_plumbing_result": "NOT_ESTABLISHED",
        "claim_ceiling": "SYNTHETIC_ASSET_USE_POINT_AND_EVIDENCE_IDENTITY_ONLY"
    });
    if errors.is_empty() { Ok(report) } else { Err(errors) }
}

fn array_is_empty(value: &Value, key: &str) -> bool {
    rows(value, key).is_none_or(Vec::is_empty)
}

fn fixtures() -> (Value, Value, Value, Value) {
    (
        serde_json::from_str(FIXTURE).expect("premise plumbing fixture JSON"),
        serde_json::from_str(BUILDING).expect("building fixture JSON"),
        serde_json::from_str(SERVICE).expect("service topology fixture JSON"),
        serde_json::from_str(HYDRAULIC).expect("hydraulic evidence fixture JSON"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-multifamily-premise-plumbing-qualifier [--self-test]");
        return;
    }
    let (fixture, building, service, hydraulic) = fixtures();
    match validate(&fixture, &building, &service, &hydraulic) {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default()),
        Err(errors) => {
            eprintln!("HOME_MULTIFAMILY_PREMISE_PLUMBING_QUALIFIER_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_apartment_plumbing_fixture_qualifies_without_water_quality_claims() {
        let (fixture, building, service, hydraulic) = fixtures();
        let report = validate(&fixture, &building, &service, &hydraulic).expect("synthetic premise-plumbing contract");
        assert_eq!(report["asset_count"], 8);
        assert_eq!(report["use_point_count"], 8);
        assert_eq!(report["observation_plan_count"], 8);
        assert_eq!(report["observation_count"], 0);
        assert_eq!(report["water_quality_claim_count"], 8);
        assert_eq!(report["water_quality_credit"], "NONE");
        assert_eq!(report["microbial_risk"], "NOT_ASSESSED");
    }

    #[test]
    fn unknown_asset_volume_cannot_be_promoted_to_a_number() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["assets"][0]["volume_state"] = json!("measured");
        fixture["assets"][0]["volume_l"] = json!(200.0);
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }

    #[test]
    fn orphan_parent_asset_is_rejected() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["assets"][1]["parent_asset_ids"] = json!(["NO-SUCH-ASSET"]);
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }

    #[test]
    fn mismatched_use_point_apartment_floor_is_rejected() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["use_points"][0]["floor_id"] = json!("F1");
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }

    #[test]
    fn use_point_cannot_reference_another_apartments_fixture() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["use_points"][0]["asset_ids"] = json!(["SYN-U-002-FIXTURE-SET"]);
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }

    #[test]
    fn potable_quality_credit_cannot_be_granted_by_a_synthetic_claim() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["water_quality_claims"][0]["potable_credit"] = json!("qualified");
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }

    #[test]
    fn modelled_water_quality_observation_does_not_create_potable_credit() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["collection_policy"]["retention_state"] = json!("approved");
        fixture["observations"] = json!([{
            "observation_id":"MODEL-WATER-AGE-U001",
            "use_point_id":"U-001-KITCHEN-SINK",
            "observation_kind":"water_quality_sample",
            "evidence_state":"modelled",
            "observed_at":"2026-11-15T00:00:00Z",
            "value":10.0,
            "unit":"min",
            "uncertainty":null,
            "instrument_ref":null,
            "calibration_ref":null,
            "lab_ref":null,
            "method_ref":null,
            "chain_of_custody_ref":null,
            "consent_ref":null,
            "evidence_ref":{"kind":"model_result","id":"MODEL-1","revision":"0.1","digest":null}
        }]);
        assert!(validate(&fixture, &building, &service, &hydraulic).is_ok());
    }

    #[test]
    fn lab_reported_observation_requires_chain_of_custody() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["observations"] = json!([{
            "observation_id":"LAB-U001",
            "use_point_id":"U-001-KITCHEN-SINK",
            "observation_kind":"water_quality_sample",
            "evidence_state":"lab_reported",
            "observed_at":"2026-11-15T00:00:00Z",
            "value":"reported",
            "unit":null,
            "uncertainty":null,
            "instrument_ref":null,
            "calibration_ref":null,
            "lab_ref":null,
            "method_ref":null,
            "chain_of_custody_ref":null,
            "consent_ref":null,
            "evidence_ref":null
        }]);
        fixture["collection_policy"]["retention_state"] = json!("approved");
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }

    #[test]
    fn approved_observation_plan_requires_instrument_method_and_authority() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["observation_plan"][0]["schedule_state"] = json!("approved");
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }

    #[test]
    fn continuous_occupancy_collection_is_rejected_by_default_policy() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["collection_policy"]["continuous_occupancy_allowed"] = json!(true);
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }

    #[test]
    fn resident_report_requires_explicit_consent_reference() {
        let (mut fixture, building, service, hydraulic) = fixtures();
        fixture["observations"] = json!([{
            "observation_id":"RESIDENT-REPORT-U001",
            "use_point_id":"U-001-KITCHEN-SINK",
            "observation_kind":"resident_report",
            "evidence_state":"self_reported",
            "observed_at":"2026-11-15T00:00:00Z",
            "value":"taste/odour report",
            "unit":null,
            "uncertainty":null,
            "instrument_ref":null,
            "calibration_ref":null,
            "lab_ref":null,
            "method_ref":null,
            "chain_of_custody_ref":null,
            "consent_ref":null,
            "evidence_ref":null
        }]);
        fixture["collection_policy"]["retention_state"] = json!("approved");
        assert!(validate(&fixture, &building, &service, &hydraulic).is_err());
    }
}
