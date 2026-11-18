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
const CONFORMANCE_CORPUS: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_GRAMMAR_CONFORMANCE_CORPUS_V0.2.json"
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
    "SOUTH_AFRICAN_ART_DECO_HISTORIC",
];

const REQUIRED_AXES: &[&str] = &[
    "spatial_typology",
    "architectural_expression",
    "material_tectonic_intent",
    "environmental_strategy_intent",
    "experience_programme",
];

const REQUIRED_TYPOLOGY_IDS: &[&str] = &[
    "DETACHED",
    "SEMI_DETACHED",
    "ROW_TERRACE",
    "COURTYARD",
    "DUPLEX",
    "APARTMENT",
    "PAVILION",
    "MULTIGENERATIONAL_CLUSTER",
    "LIVE_WORK",
    "INCREMENTAL_CORE",
    "RETROFIT_FIRST",
    "CONE_ON_CYLINDER_DWELLING",
    "BEEHIVE_DOME_DWELLING",
];

const REQUIRED_MATERIAL_IDS: &[&str] = &[
    "STONE_MASONRY",
    "BRICK_MASONRY",
    "EARTH_MASONRY",
    "TIMBER",
    "STEEL",
    "HYBRID_MATERIALS",
    "RECLAIMED_REUSED",
    "REPAIRABLE_MODULAR",
];

const REQUIRED_ENVIRONMENTAL_IDS: &[&str] = &[
    "ORIENTATION_SOLAR_RESPONSE",
    "EXTERNAL_SHADING",
    "CROSS_VENTILATION",
    "THERMAL_MASS",
    "DAYLIGHT_CONTROL",
    "RAINWATER_HARVESTING",
    "GREYWATER_REUSE",
    "ON_SITE_SOLAR",
    "PRODUCTIVE_LANDSCAPE",
    "PASSIVE_FIRST",
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

fn validate_intent_option_catalog(
    catalog: &Value,
    key: &str,
    expected_ids: &[&str],
    expected_claim_ceiling: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let options = catalog
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if options.len() != expected_ids.len() {
        errors.push(format!(
            "{key} must contain exactly {} entries, found {}",
            expected_ids.len(),
            options.len()
        ));
    }

    let mut ids = BTreeSet::new();
    for option in &options {
        let id = text(option, "option_id").unwrap_or_default();
        if id.is_empty() || !ids.insert(id.to_owned()) {
            errors.push(format!("{key} contains a missing or duplicate option ID: {id}"));
        }
        for field in ["display_name", "description"] {
            if text(option, field).is_none_or(str::is_empty) {
                errors.push(format!("{key} option {id} is missing {field}"));
            }
        }
        if text(option, "claim_ceiling") != Some(expected_claim_ceiling) {
            errors.push(format!("{key} option {id} has an invalid claim ceiling"));
        }
        // Regional vernacular forms are source-bound, context-specific
        // spatial references. They are never inferred from personal identity.
        if option.get("reference_sources").is_some() {
            let sources = option.get("reference_sources").and_then(Value::as_array);
            if sources.is_none_or(Vec::is_empty)
                || sources.is_some_and(|items| items.iter().any(|item| {
                    item.as_str().is_none_or(|value| !value.starts_with("https://"))
                }))
            {
                errors.push(format!("{key} option {id} has missing or invalid source references"));
            }
            for field in ["reference_scope", "context_note"] {
                if text(option, field).is_none_or(str::is_empty) {
                    errors.push(format!("{key} option {id} is missing {field}"));
                }
            }
            if option.get("identity_must_not_be_inferred").and_then(Value::as_bool) != Some(true) {
                errors.push(format!("{key} option {id} must prohibit identity inference"));
            }
            if text(option, "selection_status") != Some("ILLUSTRATIVE_NOT_EVALUATED") {
                errors.push(format!("{key} option {id} has an unsupported status"));
            }
        }
    }

    let expected: BTreeSet<String> = expected_ids.iter().map(|id| (*id).to_owned()).collect();
    if ids != expected {
        errors.push(format!("{key} option IDs differ from the v0.2 grammar contract"));
    }
    ids
}

fn validate_intent_entries(
    composition: &Value,
    axis: &str,
    known_ids: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let null = Value::Null;
    let axis_value = composition.get(axis).unwrap_or(&null);
    let entries = axis_value
        .get("entries")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let state = text(axis_value, "state").unwrap_or_default();
    if !["UNDECIDED", "INTENT_ONLY_NOT_EVALUATED", "CONFLICT_UNRESOLVED"].contains(&state) {
        errors.push(format!("{axis} has an unsupported intent state: {state}"));
    }
    if entries.is_empty() && state != "UNDECIDED" {
        errors.push(format!("{axis} with no entries must remain UNDECIDED"));
    }
    if !entries.is_empty() && state == "UNDECIDED" {
        errors.push(format!("{axis} with explicit entries cannot be marked UNDECIDED"));
    }

    let mut seen = BTreeSet::new();
    let mut preferred_count = 0usize;
    for entry in &entries {
        let id = text(entry, "item_id").unwrap_or_default();
        if !known_ids.contains(id) {
            errors.push(format!("{axis} references unknown option ID: {id}"));
        }
        if !seen.insert(id.to_owned()) {
            errors.push(format!("{axis} contains duplicate option entry: {id}"));
        }
        let preference = text(entry, "preference").unwrap_or_default();
        if !["PREFERRED", "CONSIDERED", "AVOIDED", "UNDECIDED"].contains(&preference) {
            errors.push(format!("{axis} option {id} has unsupported preference: {preference}"));
        }
        if preference == "PREFERRED" {
            preferred_count += 1;
        }
    }
    if axis == "spatial_typology" && preferred_count > 1 && state != "CONFLICT_UNRESOLVED" {
        errors.push("multiple preferred spatial typologies must remain CONFLICT_UNRESOLVED".to_owned());
    }
    if axis == "spatial_typology" && state == "CONFLICT_UNRESOLVED" && preferred_count < 2 {
        errors.push("spatial typology conflict state requires multiple preferred options".to_owned());
    }
}

fn validate_catalog(catalog: &Value, schema: &Value, errors: &mut Vec<String>) {
    if text(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("catalogue schema must use JSON Schema Draft 2020-12".to_owned());
    }

    let schema_required = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), None))
        .unwrap_or_default();
    for (key, count, claim) in [
        (
            "spatial_typology_catalog",
            11u64,
            "INTENT_ONLY_NO_GEOMETRY_CODE_OR_ACCESSIBILITY_CLAIM",
        ),
        (
            "material_tectonic_intent_catalog",
            8u64,
            "INTENT_ONLY_NO_ASSEMBLY_STRUCTURAL_OR_SOURCING_CLAIM",
        ),
        (
            "environmental_strategy_intent_catalog",
            10u64,
            "INTENT_ONLY_NO_PHYSICAL_PERFORMANCE_OR_RESOURCE_CLAIM",
        ),
    ] {
        let option_schema = schema
            .get("properties")
            .and_then(|value| value.get(key));
        let option_item_schema = option_schema.and_then(|value| value.get("items"));
        if !schema_required.contains(key)
            || option_schema.and_then(|value| value.get("minItems")).and_then(Value::as_u64)
                != Some(count)
            || option_schema.and_then(|value| value.get("maxItems")).and_then(Value::as_u64)
                != Some(count)
            || option_item_schema
                .and_then(|value| value.get("additionalProperties"))
                .and_then(Value::as_bool)
                != Some(false)
            || option_item_schema
                .and_then(|value| value.get("properties"))
                .and_then(|value| value.get("claim_ceiling"))
                .and_then(|value| value.get("const"))
                .and_then(Value::as_str)
                != Some(claim)
        {
            errors.push(format!("{key} schema is missing or weakens its shared vocabulary contract"));
        }
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
    if styles.len() != 35 {
        errors.push(format!("expected 35 styles, found {}", styles.len()));
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

    validate_intent_option_catalog(
        catalog,
        "spatial_typology_catalog",
        REQUIRED_TYPOLOGY_IDS,
        "INTENT_ONLY_NO_GEOMETRY_CODE_OR_ACCESSIBILITY_CLAIM",
        errors,
    );
    validate_intent_option_catalog(
        catalog,
        "material_tectonic_intent_catalog",
        REQUIRED_MATERIAL_IDS,
        "INTENT_ONLY_NO_ASSEMBLY_STRUCTURAL_OR_SOURCING_CLAIM",
        errors,
    );
    validate_intent_option_catalog(
        catalog,
        "environmental_strategy_intent_catalog",
        REQUIRED_ENVIRONMENTAL_IDS,
        "INTENT_ONLY_NO_PHYSICAL_PERFORMANCE_OR_RESOURCE_CLAIM",
        errors,
    );
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

    if ![
        "NO_RESIDENT_CHOICE_OR_DESIGN_APPROVAL_RECORDED",
        "PARTIAL_RESIDENT_INTENT_RECORDED",
    ]
    .contains(&text(grammar, "decision_state").unwrap_or_default())
    {
        errors.push("grammar decision state is unsupported or implies an unauthorized approval".to_owned());
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

    let typology_ids = catalog
        .get("spatial_typology_catalog")
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), Some("option_id")))
        .unwrap_or_default();
    let material_ids = catalog
        .get("material_tectonic_intent_catalog")
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), Some("option_id")))
        .unwrap_or_default();
    let environmental_ids = catalog
        .get("environmental_strategy_intent_catalog")
        .and_then(Value::as_array)
        .map(|values| strings(Some(values), Some("option_id")))
        .unwrap_or_default();

    for (axis, expected_ids) in [
        ("spatial_typology", &typology_ids),
        ("material_tectonic_intent", &material_ids),
        ("environmental_strategy_intent", &environmental_ids),
    ] {
        let schema_ids = schema
            .get("properties")
            .and_then(|value| value.get("composition"))
            .and_then(|value| value.get("properties"))
            .and_then(|value| value.get(axis))
            .and_then(|value| value.get("properties"))
            .and_then(|value| value.get("entries"))
            .and_then(|value| value.get("items"))
            .and_then(|value| value.get("properties"))
            .and_then(|value| value.get("item_id"))
            .and_then(|value| value.get("enum"))
            .and_then(Value::as_array)
            .map(|values| strings(Some(values), None))
            .unwrap_or_default();
        if &schema_ids != expected_ids {
            errors.push(format!("{axis} schema enum differs from its versioned catalogue"));
        }
    }

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
        if entry.get("resident_selected").and_then(Value::as_bool) != Some(true) {
            errors.push(format!("grammar style {id} must retain an explicit resident selection flag"));
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
        let preference = text(entry, "preference").unwrap_or_default();
        if !["PREFERRED", "CONSIDERED", "AVOIDED", "UNDECIDED"].contains(&preference) {
            errors.push(format!("experience {id} has unsupported preference: {preference}"));
        }
    }

    let experience_state = composition
        .get("experience_programme")
        .and_then(|value| value.get("state"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !["UNDECIDED", "OPTIONAL_IDEAS_ONLY", "CONFLICT_UNRESOLVED"].contains(&experience_state) {
        errors.push("experience programme state is unsupported".to_owned());
    }

    validate_intent_entries(composition, "spatial_typology", &typology_ids, errors);
    validate_intent_entries(composition, "material_tectonic_intent", &material_ids, errors);
    validate_intent_entries(composition, "environmental_strategy_intent", &environmental_ids, errors);

    let context = grammar.get("context").unwrap_or(&null);
    let context_state = text(context, "context_state").unwrap_or_default();
    if !["UNKNOWN", "PARTIAL", "SOURCE_BOUND_NOT_QUALIFIED"].contains(&context_state) {
        errors.push("context state is unsupported or implies unqualified authority".to_owned());
    }
    if context_state == "UNKNOWN"
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

fn pointer_segment_mut<'a>(value: &'a mut Value, segment: &str) -> Option<&'a mut Value> {
    match value {
        Value::Object(map) => map.get_mut(segment),
        Value::Array(items) => segment.parse::<usize>().ok().and_then(|index| items.get_mut(index)),
        _ => None,
    }
}

fn set_json_pointer(root: &mut Value, pointer: &str, replacement: Value) -> Result<(), String> {
    let raw_segments = pointer
        .strip_prefix('/')
        .ok_or_else(|| format!("JSON Pointer must start with '/': {pointer}"))?;
    let segments: Vec<String> = raw_segments
        .split('/')
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect();
    if segments.is_empty() || segments.iter().any(String::is_empty) {
        return Err(format!("JSON Pointer has an empty segment: {pointer}"));
    }

    let mut parent = root;
    for segment in &segments[..segments.len() - 1] {
        parent = pointer_segment_mut(parent, segment)
            .ok_or_else(|| format!("JSON Pointer parent does not exist: {pointer}"))?;
    }
    let final_segment = segments.last().expect("non-empty segments");
    let target = pointer_segment_mut(parent, final_segment)
        .ok_or_else(|| format!("JSON Pointer target does not exist: {pointer}"))?;
    *target = replacement;
    Ok(())
}

fn run_conformance_corpus() -> Result<(usize, usize), String> {
    let corpus: Value = serde_json::from_str(CONFORMANCE_CORPUS)
        .map_err(|error| format!("conformance corpus is invalid JSON: {error}"))?;
    let catalog: Value = serde_json::from_str(CATALOG).map_err(|e| e.to_string())?;
    let catalog_schema: Value = serde_json::from_str(CATALOG_SCHEMA).map_err(|e| e.to_string())?;
    let grammar_schema: Value = serde_json::from_str(GRAMMAR_SCHEMA).map_err(|e| e.to_string())?;
    let base: Value = serde_json::from_str(GRAMMAR_FIXTURE).map_err(|e| e.to_string())?;

    if text(&corpus, "corpus_id") != Some("HOME-EXPRESSION-GRAMMAR-CONFORMANCE-001")
        || text(&corpus, "schema_version") != Some("0.2.0")
        || text(&corpus, "catalogue_revision") != Some("HOME-EXPRESSION-CATALOG-0.2.0")
        || corpus.get("case_count").and_then(Value::as_u64) != Some(30)
        || corpus.get("positive_case_count").and_then(Value::as_u64) != Some(5)
        || corpus.get("negative_case_count").and_then(Value::as_u64) != Some(25)
    {
        return Err("conformance corpus identity, lineage or expected case counts changed".to_owned());
    }

    let cases = corpus
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| "conformance corpus cases must be an array".to_owned())?;
    if cases.len() != 30 {
        return Err(format!("expected 30 conformance cases, found {}", cases.len()));
    }

    let mut case_ids = BTreeSet::new();
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    for case in cases {
        let case_id = text(case, "case_id").unwrap_or_default();
        if case_id.is_empty() || !case_ids.insert(case_id.to_owned()) {
            return Err(format!("conformance corpus has a missing or duplicate case ID: {case_id}"));
        }
        let expected = text(case, "expected_disposition").unwrap_or_default();
        let category = text(case, "category").unwrap_or_default();
        if !["ACCEPT", "REJECT"].contains(&expected)
            || (category == "POSITIVE" && expected != "ACCEPT")
            || (category == "NEGATIVE" && expected != "REJECT")
            || !["POSITIVE", "NEGATIVE"].contains(&category)
        {
            return Err(format!("conformance case {case_id} has inconsistent category/disposition"));
        }

        let mut candidate = base.clone();
        let mutations = case
            .get("mutations")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("conformance case {case_id} has no mutation array"))?;
        for mutation in mutations {
            if text(mutation, "op") != Some("SET") {
                return Err(format!("conformance case {case_id} contains an unsupported mutation op"));
            }
            let pointer = text(mutation, "path")
                .ok_or_else(|| format!("conformance case {case_id} mutation has no path"))?;
            let replacement = mutation
                .get("value")
                .cloned()
                .ok_or_else(|| format!("conformance case {case_id} mutation has no value"))?;
            set_json_pointer(&mut candidate, pointer, replacement)
                .map_err(|error| format!("{case_id}: {error}"))?;
        }

        let errors = validate_all(&catalog, &catalog_schema, &candidate, &grammar_schema);
        let accepted_by_qualifier = errors.is_empty();
        if (expected == "ACCEPT") != accepted_by_qualifier {
            return Err(format!(
                "conformance case {case_id} expected {expected}, got {}: {}",
                if accepted_by_qualifier { "ACCEPT" } else { "REJECT" },
                errors.join("; ")
            ));
        }
        if accepted_by_qualifier {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }

    if accepted != 5 || rejected != 25 {
        return Err(format!("conformance distribution drifted: {accepted} accepted / {rejected} rejected"));
    }
    Ok((accepted, rejected))
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

    let mut unknown_material = grammar.clone();
    unknown_material["composition"]["material_tectonic_intent"]["entries"][0]["item_id"] =
        Value::String("UNKNOWN_ASSEMBLY".to_owned());
    if validate_all(&catalog, &catalog_schema, &unknown_material, &grammar_schema).is_empty() {
        return Err("negative control failed: unknown material vocabulary ID was accepted".to_owned());
    }

    let mut wrong_typology_count = catalog.clone();
    wrong_typology_count["spatial_typology_catalog"].as_array_mut().unwrap().pop();
    if validate_all(&wrong_typology_count, &catalog_schema, &grammar, &grammar_schema).is_empty() {
        return Err("negative control failed: incomplete spatial typology vocabulary was accepted".to_owned());
    }

    let mut mismatched_axis_schema = grammar_schema.clone();
    mismatched_axis_schema["properties"]["composition"]["properties"]["material_tectonic_intent"]["properties"]["entries"]["items"]["properties"]["item_id"]["enum"] =
        Value::Array(vec![Value::String("UNKNOWN_MATERIAL".to_owned())]);
    if validate_all(&catalog, &catalog_schema, &grammar, &mismatched_axis_schema).is_empty() {
        return Err("negative control failed: schema vocabulary drift was accepted".to_owned());
    }

    let (accepted, rejected) = run_conformance_corpus()?;
    if accepted != 5 || rejected != 25 {
        return Err("conformance corpus expected 5 positive and 25 negative cases".to_owned());
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
        let (accepted, rejected) = run_conformance_corpus().expect("conformance corpus must pass");
        println!("PASS: v0.2 catalogue and five-axis grammar integrity; 11 focused negative controls plus 30 corpus cases ({accepted} accepted / {rejected} rejected)");
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
        assert_eq!(styles.len(), 35);
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
    fn frozen_grammar_conformance_corpus_has_thirty_cases() {
        let corpus: Value = serde_json::from_str(CONFORMANCE_CORPUS).unwrap();
        assert_eq!(corpus["cases"].as_array().unwrap().len(), 30);
        let (accepted, rejected) = run_conformance_corpus().unwrap();
        assert_eq!((accepted, rejected), (5, 25));
    }

    #[test]
    fn regional_spatial_references_keep_sources_context_and_identity_guard() {
        let catalog: Value = serde_json::from_str(CATALOG).unwrap();
        let options = catalog["spatial_typology_catalog"].as_array().unwrap();
        for id in ["CONE_ON_CYLINDER_DWELLING", "BEEHIVE_DOME_DWELLING"] {
            let option = options
                .iter()
                .find(|item| item["option_id"] == id)
                .expect("regional spatial reference must exist");
            assert_eq!(option["identity_must_not_be_inferred"], true);
            assert_eq!(option["selection_status"], "ILLUSTRATIVE_NOT_EVALUATED");
            assert!(!option["reference_sources"].as_array().unwrap().is_empty());
            assert!(!option["reference_scope"].as_str().unwrap().is_empty());
            assert!(!option["context_note"].as_str().unwrap().is_empty());
        }

        let schema: Value = serde_json::from_str(CATALOG_SCHEMA).unwrap();
        let mut altered = catalog.clone();
        let regional = altered["spatial_typology_catalog"]
            .as_array_mut().unwrap()
            .iter_mut()
            .find(|item| item["option_id"] == "CONE_ON_CYLINDER_DWELLING")
            .unwrap();
        regional["identity_must_not_be_inferred"] = Value::Bool(false);
        let mut errors = Vec::new();
        validate_catalog(&altered, &schema, &mut errors);
        assert!(errors.iter().any(|error| error.contains("identity inference")));
    }

    #[test]
    fn regional_typology_without_source_is_rejected() {
        let mut catalog: Value = serde_json::from_str(CATALOG).unwrap();
        let schema: Value = serde_json::from_str(CATALOG_SCHEMA).unwrap();
        let regional = catalog["spatial_typology_catalog"]
            .as_array_mut().unwrap()
            .iter_mut()
            .find(|item| item["option_id"] == "BEEHIVE_DOME_DWELLING")
            .unwrap();
        regional["reference_sources"] = serde_json::json!([]);
        let mut errors = Vec::new();
        validate_catalog(&catalog, &schema, &mut errors);
        assert!(errors.iter().any(|error| error.contains("source references")));
    }

    #[test]
    fn karoo_heritage_is_not_a_general_style_preset() {
        let catalog: Value = serde_json::from_str(CATALOG).unwrap();
        let module = &catalog["regional_reference_modules"][0];
        assert_eq!(module["module_type"], "CONSERVATION_REFERENCE_ONLY");
        assert_eq!(module["selection_status"], "REFERENCE_ONLY_NOT_A_STYLE_PRESET");
    }
}
