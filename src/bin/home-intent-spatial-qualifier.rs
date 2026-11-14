// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::{env, fs, process};

const MANIFEST: &str = include_str!(
    "../../docs/research/home-freedom/HOME_INTENT_SPATIAL_CROSSWALK_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/home-freedom/HOME_INTENT_SPATIAL_CROSSWALK_V0.1.schema.json"
);
const BRIEF: &str = include_str!(
    "../../docs/research/home-freedom/HOME_DESIGN_BRIEF_SYNTHETIC_EXAMPLE_V0.1.json"
);
const BRIEF_SCHEMA: &str = include_str!(
    "../../docs/research/home-freedom/HOME_DESIGN_BRIEF_PROFILE_V0.1.schema.json"
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

fn path_exists(root: &Value, path: &str) -> bool {
    let mut current = root;
    for part in path.split('.') {
        let Some(next) = current.get(part) else {
            return false;
        };
        current = next;
    }
    true
}

fn validate(manifest: &Value, schema: &Value, brief: &Value, brief_schema: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();

    for (label, candidate_schema) in [("crosswalk", schema), ("brief", brief_schema)] {
        if text_at(candidate_schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
            errors.push(format!("{label} schema must declare JSON Schema Draft 2020-12"));
        }
        if text_at(candidate_schema, "type") != Some("object") {
            errors.push(format!("{label} schema root must be an object"));
        }
    }

    if text_at(manifest, "schema_version") != Some("0.1.0")
        || text_at(manifest, "study_id") != Some("HOME-INTENT-SPATIAL-BRIDGE-001")
    {
        errors.push("crosswalk identity or version is invalid".to_owned());
    }
    if text_at(manifest, "maturity") != Some("PROPOSED_OWNER_REVIEW_REQUIRED")
        || text_at(manifest, "execution_status") != Some("NO_RUNTIME_ADAPTER")
    {
        errors.push("crosswalk cannot be promoted to owner-accepted/runtime-connected state".to_owned());
    }
    if text_at(manifest, "source_schema") != Some("HOME_DESIGN_BRIEF_PROFILE_V0.1.schema.json")
        || text_at(manifest, "source_schema_version") != Some("0.1.0")
        || text_at(brief, "schema_version") != Some("0.1.0")
    {
        errors.push("crosswalk source schema binding is invalid".to_owned());
    }

    let mappings = array_at(manifest, "mappings").cloned().unwrap_or_default();
    if mappings.len() != 8 || manifest.get("mapping_count").and_then(Value::as_u64) != Some(8) {
        errors.push("P0 crosswalk must contain exactly eight mappings".to_owned());
    }

    let mut mapping_ids = BTreeSet::new();
    let mut source_paths = BTreeSet::new();
    for mapping in &mappings {
        let id = text_at(mapping, "mapping_id").unwrap_or("<missing-mapping-id>");
        if !mapping_ids.insert(id.to_owned()) {
            errors.push(format!("duplicate mapping ID {id}"));
        }
        let source_path = text_at(mapping, "source_path").unwrap_or_default();
        if source_path.is_empty() || !source_paths.insert(source_path.to_owned()) {
            errors.push(format!("{id}: source path is missing or duplicated"));
        } else if !path_exists(brief, source_path) {
            errors.push(format!("{id}: source path {source_path} does not exist in the exact synthetic brief"));
        }
        if text_at(mapping, "relation") != Some("PROPOSED_SEMANTIC_MAPPING")
            || text_at(mapping, "status") != Some("OWNER_CONFIRMATION_REQUIRED")
            || !mapping.get("target_field_path").is_some_and(Value::is_null)
        {
            errors.push(format!("{id}: unsupported or unconfirmed mapping was promoted"));
        }
        if array_at(mapping, "target_owners").is_none_or(Vec::is_empty)
            || text_at(mapping, "target_semantic").is_none_or(str::is_empty)
            || text_at(mapping, "transformation").is_none_or(str::is_empty)
            || text_at(mapping, "loss_policy").is_none_or(str::is_empty)
        {
            errors.push(format!("{id}: target owner, semantic or loss policy is missing"));
        }
    }

    let expected_paths = [
        "household_frame",
        "intent.activities",
        "intent.space_relationships",
        "constraints",
        "preferences",
        "site_reference",
        "delivery",
        "privacy",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if source_paths.iter().map(String::as_str).collect::<BTreeSet<_>>() != expected_paths {
        errors.push("crosswalk does not cover the eight required home-brief semantic slices".to_owned());
    }

    let geometry = manifest.get("geometry_gate").unwrap_or(&Value::Null);
    if text_at(geometry, "owner") != Some("Symtropy #683")
        || text_at(geometry, "state") != Some("BLOCKED_UNTIL_QUALIFIED")
        || geometry.get("geometry_claims_permitted").and_then(Value::as_bool) != Some(false)
        || geometry.get("production_takeoff_permitted").and_then(Value::as_bool) != Some(false)
    {
        errors.push("exact geometry / production takeoff must remain blocked until provider qualification".to_owned());
    }

    let voi = manifest.get("value_of_information").unwrap_or(&Value::Null);
    if text_at(voi, "state") != Some("PROPOSED_NOT_EXECUTED")
        || text_at(voi, "authority") != Some("ADVISORY_ONLY")
        || array_at(voi, "questions").is_none_or(Vec::is_empty)
    {
        errors.push("value-of-information must remain advisory and not executed".to_owned());
    }

    if errors.is_empty() {
        Ok(json!({
            "audit_state":"HOME_INTENT_SPATIAL_CROSSWALK_PASS",
            "study_id":"HOME-INTENT-SPATIAL-BRIDGE-001",
            "mapping_count":mappings.len(),
            "source_paths_checked":source_paths.len(),
            "owner_confirmed_mappings":0,
            "runtime_connected_mappings":0,
            "target_field_paths_invented":0,
            "geometry_claims_permitted":false,
            "production_takeoff_permitted":false,
            "value_of_information_authority":"ADVISORY_ONLY",
            "claim_ceiling":"OWNER_REVIEW_CROSSWALK_PROPOSAL_ONLY"
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
                    "usage":"home-intent-spatial-qualifier [--input PATH] [--self-test]",
                    "default":"embedded owner-review crosswalk and synthetic brief",
                    "claim_ceiling":"proposed semantic mappings only; no runtime adapter"
                }));
            }
            unknown => return Err(format!("unknown argument {unknown}")),
        }
        index += 1;
    }

    let manifest = match input_path {
        Some(path) => {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("could not read crosswalk input {path}: {error}"))?;
            parse_json(&content, "crosswalk input")?
        }
        None => parse_json(MANIFEST, "crosswalk manifest")?,
    };
    let schema = parse_json(SCHEMA, "crosswalk schema")?;
    let brief = parse_json(BRIEF, "synthetic brief")?;
    let brief_schema = parse_json(BRIEF_SCHEMA, "brief schema")?;
    validate(&manifest, &schema, &brief, &brief_schema)
        .map_err(|errors| format!("HOME_INTENT_SPATIAL_CROSSWALK_FAIL: {}", errors.join("; ")))
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            if text_at(&report, "audit_state") == Some("HOME_INTENT_SPATIAL_CROSSWALK_PASS") {
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

    fn fixture() -> (Value, Value, Value, Value) {
        (
            parse_json(MANIFEST, "manifest").expect("manifest JSON"),
            parse_json(SCHEMA, "schema").expect("schema JSON"),
            parse_json(BRIEF, "brief").expect("brief JSON"),
            parse_json(BRIEF_SCHEMA, "brief schema").expect("brief schema JSON"),
        )
    }

    #[test]
    fn owner_review_crosswalk_is_admissible_without_claiming_integration() {
        let (manifest, schema, brief, brief_schema) = fixture();
        let report = validate(&manifest, &schema, &brief, &brief_schema)
            .expect("owner-review crosswalk must qualify");
        assert_eq!(report["mapping_count"], 8);
        assert_eq!(report["source_paths_checked"], 8);
        assert_eq!(report["owner_confirmed_mappings"], 0);
        assert_eq!(report["runtime_connected_mappings"], 0);
        assert_eq!(report["geometry_claims_permitted"], false);
    }

    #[test]
    fn owner_mapping_cannot_be_promoted_without_owner_confirmation() {
        let (mut manifest, schema, brief, brief_schema) = fixture();
        manifest["mappings"][0]["status"] = json!("OWNER_CONFIRMED");
        assert!(validate(&manifest, &schema, &brief, &brief_schema).is_err());
    }

    #[test]
    fn target_field_path_cannot_be_invented() {
        let (mut manifest, schema, brief, brief_schema) = fixture();
        manifest["mappings"][0]["target_field_path"] = json!("target.unknown.path");
        assert!(validate(&manifest, &schema, &brief, &brief_schema).is_err());
    }

    #[test]
    fn exact_equivalence_cannot_be_claimed_by_proposal() {
        let (mut manifest, schema, brief, brief_schema) = fixture();
        manifest["mappings"][0]["relation"] = json!("EXACT");
        assert!(validate(&manifest, &schema, &brief, &brief_schema).is_err());
    }

    #[test]
    fn missing_source_semantic_is_rejected() {
        let (mut manifest, schema, brief, brief_schema) = fixture();
        manifest["mappings"][0]["source_path"] = json!("intent.made_up_field");
        assert!(validate(&manifest, &schema, &brief, &brief_schema).is_err());
    }

    #[test]
    fn hard_preference_and_accessibility_constraints_remain_distinct() {
        let (mut manifest, schema, brief, brief_schema) = fixture();
        manifest["mappings"].retain(|m| m["source_path"] != "constraints");
        assert!(validate(&manifest, &schema, &brief, &brief_schema).is_err());
    }

    #[test]
    fn geometry_cannot_be_claimed_before_provider_qualification() {
        let (mut manifest, schema, brief, brief_schema) = fixture();
        manifest["geometry_gate"]["geometry_claims_permitted"] = json!(true);
        assert!(validate(&manifest, &schema, &brief, &brief_schema).is_err());
    }

    #[test]
    fn value_of_information_remains_advisory() {
        let (mut manifest, schema, brief, brief_schema) = fixture();
        manifest["value_of_information"]["authority"] = json!("EXECUTION_AUTHORITY");
        assert!(validate(&manifest, &schema, &brief, &brief_schema).is_err());
    }

    #[test]
    fn runtime_integration_is_not_implied_by_schema_presence() {
        let (mut manifest, schema, brief, brief_schema) = fixture();
        manifest["execution_status"] = json!("RUNTIME_CONNECTED");
        assert!(validate(&manifest, &schema, &brief, &brief_schema).is_err());
    }
}
