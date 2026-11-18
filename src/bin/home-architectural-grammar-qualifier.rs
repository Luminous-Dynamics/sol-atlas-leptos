// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::Value;
use std::collections::BTreeSet;
use std::{env, process};

const BASE_CATALOG: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_CATALOG_V0.1.json"
);
const CATALOG: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_CATALOG_V0.2.json"
);
const CATALOG_SCHEMA: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_CATALOG_V0.2.schema.json"
);
const GRAMMAR_SCHEMA: &str = include_str!(
    "../../docs/research/home-expression/HOME_ARCHITECTURAL_GRAMMAR_COMPOSITION_V0.2.schema.json"
);
const GRAMMAR_FIXTURE: &str = include_str!(
    "../../docs/research/home-expression/HOME_ARCHITECTURAL_GRAMMAR_SYNTHETIC_HIGHVELD_V0.2.json"
);

const REQUIRED_BASE_STYLE_IDS: &[&str] = &[
    "HIGHVELD_STONE_TIMBER_CONTEMPORARY",
    "CAPE_COURTYARD_VERNACULAR",
    "REGIONALLY_AUTHORED_MODERNISM",
    "MODERNIST",
    "MID_CENTURY_MODERN",
    "BIOPHILIC_ORGANIC",
    "TROPICAL_MODERN",
    "MEDITERRANEAN_COURTYARD",
    "JAPANESE_INSPIRED_MINIMAL",
    "INDUSTRIAL",
    "FARMHOUSE_CRAFT",
    "ART_DECO",
    "BRUTALIST_MONOLITHIC",
    "GOTHIC_REVIVAL",
    "HERITAGE_RESTORATION",
    "SOLARPUNK",
    "MINIMALIST",
    "MAXIMALIST",
    "CONTEMPORARY_CLASSIC",
    "OTHER_CUSTOM",
];

const REQUIRED_ADDITIONAL_STYLE_IDS: &[&str] = &[
    "CAPE_DUTCH_HISTORIC",
    "CAPE_GEORGIAN_INFLUENCE",
    "VICTORIAN",
    "EDWARDIAN",
    "GEORGIAN_NEOCLASSICAL",
    "ARTS_AND_CRAFTS_CRAFTSMAN",
    "ART_NOUVEAU",
    "PRAIRIE_SCHOOL",
    "BAUHAUS",
    "INTERNATIONAL_STYLE",
    "NORDIC_SCANDINAVIAN",
    "SPANISH_COLONIAL_MISSION",
    "ALPINE_CHALET",
    "POSTMODERN",
];

const REQUIRED_AXES: &[&str] = &[
    "spatial_typology",
    "architectural_expression",
    "material_tectonic_intent",
    "environmental_strategy_intent",
    "experience_programme",
];

fn parse(source: &str, label: &str, errors: &mut Vec<String>) -> Value {
    match serde_json::from_str(source) {
        Ok(value) => value,
        Err(error) => {
            errors.push(format!("{label} is invalid JSON: {error}"));
            Value::Null
        }
    }
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn strings(value: Option<&Vec<Value>>, field: Option<&str>) -> BTreeSet<String> {
    value
        .into_iter()
        .flatten()
        .filter_map(|item| match field {
            Some(key) => item.get(key).and_then(Value::as_str),
            None => item.as_str(),
        })
        .map(str::to_owned)
        .collect()
}

fn validate_catalog(catalog: &Value, schema: &Value, errors: &mut Vec<String>) {
    if text(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("catalogue schema must use JSON Schema Draft 2020-12".to_owned());
    }
    if text(catalog, "schema_version") != Some("0.2.0")
        || text(catalog, "catalog_revision") != Some("HOME-EXPRESSION-CATALOG-0.2.0")
        || text(catalog, "supersedes_catalog_revision") != Some("HOME-EXPRESSION-CATALOG-0.1.0")
        || text(catalog, "maturity") != Some("ILLUSTRATIVE_CATALOG_NOT_EVALUATED")
    {
        errors.push("catalogue identity, lineage or maturity was changed".to_owned());
    }

    let base_catalog: Value = match serde_json::from_str(BASE_CATALOG) {
        Ok(value) => value,
        Err(error) => {
            errors.push(format!("v0.1 baseline catalogue is invalid JSON: {error}"));
            Value::Null
        }
    };

    let styles = catalog
        .get("style_catalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if styles.len() != 34 {
        errors.push(format!("expected 34 styles, found {}", styles.len()));
    }
    let mut style_ids = BTreeSet::new();
    for style in &styles {
        let id = text(style, "style_id").unwrap_or_default();
        if id.is_empty() || !style_ids.insert(id.to_owned()) {
            errors.push(format!("missing or duplicate style ID: {id}"));
        }
        for field in ["display_name", "category", "description", "authenticity_note"] {
            if text(style, field).is_none_or(str::is_empty) {
                errors.push(format!("style {id} is missing {field}"));
            }
        }
        if style.get("not_a_typology").and_then(Value::as_bool) != Some(true) {
            errors.push(format!("style {id} must remain distinct from spatial typology"));
        }
        if text(style, "selection_status") != Some("ILLUSTRATIVE_NOT_EVALUATED") {
            errors.push(format!("style {id} has an unsupported evaluation claim"));
        }
        if text(style, "reference_policy").is_none() {
            errors.push(format!("style {id} has no reference policy"));
        }
        if style
            .get("reference_sources")
            .and_then(Value::as_array)
            .is_none()
        {
            errors.push(format!("style {id} must declare a reference_sources array"));
        }
    }

    for id in REQUIRED_BASE_STYLE_IDS.iter().chain(REQUIRED_ADDITIONAL_STYLE_IDS) {
        if !style_ids.contains(*id) {
            errors.push(format!("required style ID is missing: {id}"));
        }
    }
    let base_style_ids = base_catalog
        .get("style_catalog")
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), Some("style_id")))
        .unwrap_or_default();
    if base_style_ids.len() != 20 || !base_style_ids.is_subset(&style_ids) {
        errors.push("v0.2 must preserve every v0.1 style ID".to_owned());
    }

    let composition_axes = catalog
        .get("composition_model")
        .and_then(|value| value.get("independent_axes"))
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), None))
        .unwrap_or_default();
    if composition_axes.len() != 5 {
        errors.push("catalogue must define five independent grammar axes".to_owned());
    }
    let forbidden_terms = ["african style", "ethnic style", "pan-african"];
    for style in &styles {
        let name = text(style, "display_name").unwrap_or_default().to_ascii_lowercase();
        if forbidden_terms.iter().any(|term| name.contains(term)) {
            errors.push(format!("generic pan-cultural preset is forbidden: {}", text(style, "style_id").unwrap_or_default()));
        }
    }

    let packages = catalog
        .get("experience_catalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if packages.len() != 26 {
        errors.push(format!("expected 26 experience packages, found {}", packages.len()));
    }
    let package_ids = strings(Some(&packages), Some("package_id"));
    if package_ids.len() != packages.len() {
        errors.push("experience package IDs are missing or duplicated".to_owned());
    }
    let base_package_ids = base_catalog
        .get("experience_catalog")
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), Some("package_id")))
        .unwrap_or_default();
    if base_package_ids.len() != 26 || package_ids != base_package_ids {
        errors.push("v0.2 must preserve the exact v0.1 experience package ID set".to_owned());
    }
    for package in &packages {
        let id = text(package, "package_id").unwrap_or_default();
        if package.get("optional_by_default").and_then(Value::as_bool) != Some(true)
            || text(package, "selection_default") != Some("NOT_SELECTED")
            || text(package, "claim_ceiling")
                != Some("CATALOG_ONLY_NO_COST_PERFORMANCE_OR_FEASIBILITY_CLAIM")
        {
            errors.push(format!("experience package {id} is no longer optional/non-qualifying by default"));
        }
    }

    let heritage = catalog
        .get("regional_reference_modules")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if heritage.len() != 1
        || text(&heritage[0], "reference_id") != Some("KAROO_CORBELLED_STONE_HERITAGE")
        || text(&heritage[0], "module_type") != Some("CONSERVATION_REFERENCE_ONLY")
        || text(&heritage[0], "selection_status") != Some("REFERENCE_ONLY_NOT_A_STYLE_PRESET")
    {
        errors.push("Karoo corbelled heritage must remain a separate conservation-only reference module".to_owned());
    }
    if heritage
        .first()
        .and_then(|item| item.get("reference_sources"))
        .and_then(Value::as_array)
        .is_none_or(|items| items.len() < 2)
    {
        errors.push("Karoo heritage reference requires at least two source records".to_owned());
    }

    let null = Value::Null;
    let community = catalog.get("community_authored_reference_policy").unwrap_or(&null);
    if text(community, "maturity") != Some("POLICY_ONLY_NO_COMMUNITY_MODULES_PUBLISHED")
        || text(community, "prohibition") != Some("DO_NOT_CREATE_GENERIC_AFRICAN_OR_ETHNIC_STYLE_PRESETS")
    {
        errors.push("community-authored reference safeguards are missing or were weakened".to_owned());
    }
}

fn validate_grammar(
    grammar: &Value,
    schema: &Value,
    catalog: &Value,
    errors: &mut Vec<String>,
) {
    if text(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("grammar schema must use JSON Schema Draft 2020-12".to_owned());
    }
    let required_axes = schema
        .get("properties")
        .and_then(|value| value.get("composition"))
        .and_then(|value| value.get("required"))
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), None))
        .unwrap_or_default();
    for axis in REQUIRED_AXES {
        if !required_axes.contains(*axis) {
            errors.push(format!("grammar schema is missing independent axis {axis}"));
        }
    }

    if !["SYNTHETIC_NOT_EVALUATED", "RESIDENT_WORKING_DRAFT_NOT_VALIDATED", "OWNER_REVIEW_REQUIRED"]
        .contains(&text(grammar, "maturity").unwrap_or_default())
    {
        errors.push("grammar maturity is unsupported".to_owned());
    }
    if text(grammar, "schema_version") != Some("0.2.0")
        || text(grammar, "catalog_revision") != Some("HOME-EXPRESSION-CATALOG-0.2.0")
        || text(grammar, "evidence_state") != Some("UNKNOWN_NOT_EVALUATED")
        || text(grammar, "claim_ceiling")
            != Some("INTENT_ONLY_NO_GEOMETRY_COST_PERFORMANCE_OR_APPROVAL")
        || text(grammar, "resident_statement").is_none()
    {
        errors.push("grammar identity, resident statement, evidence or claim ceiling is invalid".to_owned());
    }

    let styles = catalog
        .get("style_catalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let style_ids = strings(Some(&styles), Some("style_id"));
    let package_ids = catalog
        .get("experience_catalog")
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), Some("package_id")))
        .unwrap_or_default();

    let null = Value::Null;
    let composition = grammar.get("composition").unwrap_or(&null);
    for axis in REQUIRED_AXES {
        if composition.get(axis).is_none() {
            errors.push(format!("grammar composition is missing axis {axis}"));
        }
    }

    let expression = composition
        .get("architectural_expression")
        .and_then(|value| value.get("entries"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut primary_count = 0usize;
    let mut expression_ids = BTreeSet::new();
    for entry in &expression {
        let id = text(entry, "style_id").unwrap_or_default();
        if !style_ids.contains(id) {
            errors.push(format!("grammar references unknown style ID: {id}"));
        }
        if !expression_ids.insert(id.to_owned()) {
            errors.push(format!("grammar contains duplicate expression entry: {id}"));
        }
        let role = text(entry, "role").unwrap_or_default();
        if !["PRIMARY", "SECONDARY", "ACCENT", "REFERENCE_ONLY", "AVOID", "ACCEPTABLE", "UNDECIDED"].contains(&role) {
            errors.push(format!("grammar style {id} has unsupported resident role: {role}"));
        }
        if role == "PRIMARY" {
            primary_count += 1;
        }
        if role == "REFERENCE_ONLY" {
            let source_refs = entry.get("source_refs").and_then(Value::as_array);
            if source_refs.is_none_or(|refs| refs.is_empty()) {
                errors.push(format!("reference-only style {id} must carry source references"));
            } else if let Some(catalog_style) = styles
                .iter()
                .find(|style| text(style, "style_id") == Some(id))
            {
                let allowed_sources = strings(
                    catalog_style.get("reference_sources").and_then(Value::as_array),
                    None,
                );
                for source in strings(source_refs, None) {
                    if !allowed_sources.contains(&source) {
                        errors.push(format!("reference-only style {id} cites a source not declared by the catalogue"));
                    }
                }
            }
        }
        if let Some(weight) = entry.get("blend_weight_pct") {
            if weight.as_u64().is_none_or(|value| value > 100) {
                errors.push(format!("grammar style {id} has an invalid blend weight"));
            }
        }
    }
    if primary_count > 1 {
        errors.push("grammar must not silently resolve multiple primary expression roles".to_owned());
    }

    let experiences = composition
        .get("experience_programme")
        .and_then(|value| value.get("entries"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut experience_ids = BTreeSet::new();
    for entry in &experiences {
        let id = text(entry, "package_id").unwrap_or_default();
        if !package_ids.contains(id) {
            errors.push(format!("grammar references unknown experience package: {id}"));
        }
        if !experience_ids.insert(id.to_owned()) {
            errors.push(format!("grammar contains duplicate experience entry: {id}"));
        }
    }

    let context = grammar.get("context").unwrap_or(&null);
    if text(context, "context_state") == Some("UNKNOWN")
        && text(grammar, "evidence_state") != Some("UNKNOWN_NOT_EVALUATED")
    {
        errors.push("unknown site/climate/jurisdiction context cannot be promoted to evaluated".to_owned());
    }
}

fn validate_all(
    catalog: &Value,
    catalog_schema: &Value,
    grammar: &Value,
    grammar_schema: &Value,
) -> Vec<String> {
    let mut errors = Vec::new();
    validate_catalog(catalog, catalog_schema, &mut errors);
    validate_grammar(grammar, grammar_schema, catalog, &mut errors);
    errors
}

fn run_self_test() -> Result<(), String> {
    let catalog: Value = serde_json::from_str(CATALOG).map_err(|e| e.to_string())?;
    let catalog_schema: Value = serde_json::from_str(CATALOG_SCHEMA).map_err(|e| e.to_string())?;
    let grammar: Value = serde_json::from_str(GRAMMAR_FIXTURE).map_err(|e| e.to_string())?;
    let grammar_schema: Value = serde_json::from_str(GRAMMAR_SCHEMA).map_err(|e| e.to_string())?;

    let baseline = validate_all(&catalog, &catalog_schema, &grammar, &grammar_schema);
    if !baseline.is_empty() {
        return Err(format!("baseline contract failed: {}", baseline.join("; ")));
    }

    let mut duplicate = catalog.clone();
    let first = duplicate["style_catalog"][0].clone();
    duplicate["style_catalog"][1]["style_id"] = first["style_id"].clone();
    if validate_all(&duplicate, &catalog_schema, &grammar, &grammar_schema).is_empty() {
        return Err("negative control failed: duplicate style ID was accepted".to_owned());
    }

    let mut wrong_style = grammar.clone();
    wrong_style["composition"]["architectural_expression"]["entries"][0]["style_id"] =
        Value::String("UNKNOWN_STYLE".to_owned());
    if validate_all(&catalog, &catalog_schema, &wrong_style, &grammar_schema).is_empty() {
        return Err("negative control failed: unknown style reference was accepted".to_owned());
    }

    let mut promoted = grammar.clone();
    promoted["evidence_state"] = Value::String("QUALIFIED".to_owned());
    if validate_all(&catalog, &catalog_schema, &promoted, &grammar_schema).is_empty() {
        return Err("negative control failed: evidence promotion was accepted".to_owned());
    }

    let mut geometry_claim = grammar.clone();
    geometry_claim["claim_ceiling"] = Value::String("BUILDABLE_PLAN".to_owned());
    if validate_all(&catalog, &catalog_schema, &geometry_claim, &grammar_schema).is_empty() {
        return Err("negative control failed: geometry/approval claim was accepted".to_owned());
    }

    let mut lost_heritage_source = catalog.clone();
    lost_heritage_source["regional_reference_modules"][0]["reference_sources"] =
        Value::Array(Vec::new());
    if validate_all(&lost_heritage_source, &catalog_schema, &grammar, &grammar_schema).is_empty() {
        return Err("negative control failed: unreferenced heritage module was accepted".to_owned());
    }

    let mut collapsed = grammar_schema.clone();
    collapsed["properties"]["composition"]["required"] =
        Value::Array(vec![Value::String("architectural_expression".to_owned())]);
    if validate_all(&catalog, &catalog_schema, &grammar, &collapsed).is_empty() {
        return Err("negative control failed: collapsed grammar axes were accepted".to_owned());
    }

    let mut duplicate_expression = grammar.clone();
    let expression_entry = duplicate_expression["composition"]["architectural_expression"]["entries"][0].clone();
    duplicate_expression["composition"]["architectural_expression"]["entries"].as_array_mut().unwrap().push(expression_entry);
    if validate_all(&catalog, &catalog_schema, &duplicate_expression, &grammar_schema).is_empty() {
        return Err("negative control failed: duplicate expression entry was accepted".to_owned());
    }

    let mut unreferenced_reference = grammar.clone();
    unreferenced_reference["composition"]["architectural_expression"]["entries"][1]["source_refs"] = Value::Array(Vec::new());
    if validate_all(&catalog, &catalog_schema, &unreferenced_reference, &grammar_schema).is_empty() {
        return Err("negative control failed: unreferenced heritage style entry was accepted".to_owned());
    }

    Ok(())
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let catalog = parse(CATALOG, "catalogue", &mut Vec::new());
    let catalog_schema = parse(CATALOG_SCHEMA, "catalogue schema", &mut Vec::new());
    let grammar = parse(GRAMMAR_FIXTURE, "grammar fixture", &mut Vec::new());
    let grammar_schema = parse(GRAMMAR_SCHEMA, "grammar schema", &mut Vec::new());

    let errors = validate_all(&catalog, &catalog_schema, &grammar, &grammar_schema);
    if !errors.is_empty() {
        for error in errors {
            eprintln!("FAIL: {error}");
        }
        process::exit(1);
    }

    if arguments.get(1).is_some_and(|arg| arg == "--self-test") {
        if let Err(error) = run_self_test() {
            eprintln!("SELF-TEST FAIL: {error}");
            process::exit(1);
        }
        println!("PASS: v0.2 catalogue and five-axis grammar integrity; 8 adversarial negative controls rejected");
    } else {
        println!("PASS: v0.2 catalogue and five-axis grammar integrity");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_preserves_twenty_original_and_adds_fourteen_families() {
        let catalog: Value = serde_json::from_str(CATALOG).unwrap();
        let schema: Value = serde_json::from_str(CATALOG_SCHEMA).unwrap();
        let styles = catalog["style_catalog"].as_array().unwrap();
        assert_eq!(styles.len(), 34);
        let ids = strings(Some(styles), Some("style_id"));
        for id in REQUIRED_BASE_STYLE_IDS.iter().chain(REQUIRED_ADDITIONAL_STYLE_IDS) {
            assert!(ids.contains(*id), "missing {id}");
        }
        let mut errors = Vec::new();
        validate_catalog(&catalog, &schema, &mut errors);
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn grammar_keeps_five_axes_and_unknown_physical_context() {
        let catalog: Value = serde_json::from_str(CATALOG).unwrap();
        let catalog_schema: Value = serde_json::from_str(CATALOG_SCHEMA).unwrap();
        let grammar: Value = serde_json::from_str(GRAMMAR_FIXTURE).unwrap();
        let grammar_schema: Value = serde_json::from_str(GRAMMAR_SCHEMA).unwrap();
        let errors = validate_all(&catalog, &catalog_schema, &grammar, &grammar_schema);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(grammar["context"]["context_state"], "UNKNOWN");
        assert_eq!(grammar["evidence_state"], "UNKNOWN_NOT_EVALUATED");
    }

    #[test]
    fn karoo_heritage_is_not_a_general_style_preset() {
        let catalog: Value = serde_json::from_str(CATALOG).unwrap();
        let module = &catalog["regional_reference_modules"][0];
        assert_eq!(module["module_type"], "CONSERVATION_REFERENCE_ONLY");
        assert_eq!(module["selection_status"], "REFERENCE_ONLY_NOT_A_STYLE_PRESET");
    }
}
