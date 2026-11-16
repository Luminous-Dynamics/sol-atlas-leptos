// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::Value;
use std::collections::BTreeSet;
use std::{env, fs, process};

const PROFILE_FIXTURE: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_PROFILE_SYNTHETIC_HIGHVELD_V0.1.json"
);
const PROFILE_SCHEMA: &str =
    include_str!("../../docs/research/home-expression/HOME_EXPRESSION_PROFILE_V0.1.schema.json");
const CATALOG_FIXTURE: &str =
    include_str!("../../docs/research/home-expression/HOME_EXPRESSION_CATALOG_V0.1.json");
const BENCHMARK_FIXTURE: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_BENCHMARK_EXTENSION_V0.1.json"
);
const BENCHMARK_SCHEMA: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_BENCHMARK_EXTENSION_V0.1.schema.json"
);
const REVISION_IMPACT_FIXTURE: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_REVISION_IMPACT_CROSSWALK_V0.1.json"
);
const REVISION_IMPACT_SCHEMA: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_REVISION_IMPACT_CROSSWALK_V0.1.schema.json"
);
const EXPECTED_CATALOG_REVISION: &str = "HOME-EXPRESSION-CATALOG-0.1.0";
const EXPECTED_CATALOG_BLOB_SHA: &str = "f247bfc326e2e5fe5b625452e84de53faae38aa9";

const IMPACT_DOMAINS: &[&str] = &[
    "AREA_AND_CIRCULATION",
    "CAPITAL_COST",
    "LIFECYCLE_OPERATING_COST",
    "ENERGY_AND_PEAK_DEMAND",
    "WATER_AND_DRAINAGE",
    "MATERIALS_AND_CARBON",
    "MAINTENANCE_AND_REPAIR",
    "ACCESSIBILITY",
    "STRUCTURE_FIRE_MOISTURE_ACOUSTICS",
    "SOURCING_AND_DELIVERY",
];

fn string_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn nonempty_string(value: &Value, key: &str) -> bool {
    string_at(value, key).is_some_and(|text| !text.trim().is_empty())
}

fn is_null(value: &Value, key: &str) -> bool {
    value.get(key).is_none_or(Value::is_null)
}

fn forbidden_score_path(value: &Value, path: &str, errors: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}.{key}");
                let normalized = key.to_ascii_lowercase();
                if [
                    "beauty_score",
                    "luxury_score",
                    "cultural_fit_score",
                    "meaning_score",
                    "poetry_score",
                ]
                .contains(&normalized.as_str())
                {
                    errors.push(format!(
                        "{child_path}: universal aesthetic/symbolic score is forbidden"
                    ));
                }
                forbidden_score_path(child, &child_path, errors);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                forbidden_score_path(child, &format!("{path}[{index}]"), errors);
            }
        }
        _ => {}
    }
}

fn validate_schema_header(schema: &Value, errors: &mut Vec<String>) {
    if string_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("profile schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if string_at(schema, "type") != Some("object") {
        errors.push("profile schema root must be an object".to_owned());
    }
}

fn string_set(values: Option<&Vec<Value>>) -> BTreeSet<String> {
    values
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn object_field_string_set(values: Option<&Vec<Value>>, field: &str) -> BTreeSet<String> {
    values
        .into_iter()
        .flatten()
        .filter_map(|value| value.get(field).and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

fn validate_catalog(
    schema: &Value,
    errors: &mut Vec<String>,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let catalog: Value = match serde_json::from_str(CATALOG_FIXTURE) {
        Ok(value) => value,
        Err(error) => {
            errors.push(format!("expression catalog is invalid JSON: {error}"));
            return (BTreeSet::new(), BTreeSet::new());
        }
    };

    if string_at(&catalog, "schema_version") != Some("0.1.0")
        || string_at(&catalog, "maturity") != Some("ILLUSTRATIVE_CATALOG_NOT_EVALUATED")
    {
        errors.push(
            "expression catalog must remain version 0.1.0 / ILLUSTRATIVE_CATALOG_NOT_EVALUATED"
                .to_owned(),
        );
    }

    let catalog_domains = string_set(array_at(&catalog, "impact_domains"));
    let expected_domains: BTreeSet<String> = IMPACT_DOMAINS
        .iter()
        .map(|domain| (*domain).to_owned())
        .collect();
    if catalog_domains != expected_domains {
        errors.push("catalog impact domains differ from the profile qualifier contract".to_owned());
    }

    let styles = array_at(&catalog, "style_catalog")
        .cloned()
        .unwrap_or_default();
    let mut style_ids = BTreeSet::new();
    for style in &styles {
        let style_id = string_at(style, "style_id").unwrap_or_default();
        if style_id.is_empty() || !style_ids.insert(style_id.to_owned()) {
            errors.push(format!(
                "catalog contains a missing or duplicate style_id: {style_id}"
            ));
        }
        if !nonempty_string(style, "display_name") || !nonempty_string(style, "description") {
            errors.push(format!(
                "catalog style {style_id} needs a display name and description"
            ));
        }
        if string_at(style, "authenticity_note").is_none() {
            errors.push(format!(
                "catalog style {style_id} is missing its non-authenticity claim boundary"
            ));
        }
    }

    let schema_style_ids = schema
        .get("properties")
        .and_then(|value| value.get("style_preferences"))
        .and_then(|value| value.get("properties"))
        .and_then(|value| value.get("entries"))
        .and_then(|value| value.get("items"))
        .and_then(|value| value.get("properties"))
        .and_then(|value| value.get("style_id"))
        .and_then(|value| value.get("enum"))
        .and_then(Value::as_array);
    if string_set(schema_style_ids) != style_ids {
        errors.push("style catalog IDs do not exactly match the profile schema enum".to_owned());
    }

    let packages = array_at(&catalog, "experience_catalog")
        .cloned()
        .unwrap_or_default();
    let mut package_ids = BTreeSet::new();
    for package in &packages {
        let package_id = string_at(package, "package_id").unwrap_or_default();
        if package_id.is_empty() || !package_ids.insert(package_id.to_owned()) {
            errors.push(format!(
                "catalog contains a missing or duplicate package_id: {package_id}"
            ));
        }
        if !nonempty_string(package, "display_name") || !nonempty_string(package, "description") {
            errors.push(format!(
                "catalog package {package_id} needs a display name and description"
            ));
        }
        if package.get("optional_by_default").and_then(Value::as_bool) != Some(true)
            || string_at(package, "selection_default") != Some("NOT_SELECTED")
            || string_at(package, "claim_ceiling")
                != Some("CATALOG_ONLY_NO_COST_PERFORMANCE_OR_FEASIBILITY_CLAIM")
        {
            errors.push(format!(
                "catalog package {package_id} must remain optional, unselected and non-qualifying by default"
            ));
        }
        if string_set(array_at(package, "impact_domains")) != expected_domains {
            errors.push(format!(
                "catalog package {package_id} must declare all ten impact domains"
            ));
        }
        let priority_domains = string_set(array_at(package, "priority_review_domains"));
        if !priority_domains.is_subset(&expected_domains) {
            errors.push(format!(
                "catalog package {package_id} contains an unknown priority review domain"
            ));
        }
    }

    let schema_package_ids = schema
        .get("properties")
        .and_then(|value| value.get("experience_options"))
        .and_then(|value| value.get("items"))
        .and_then(|value| value.get("properties"))
        .and_then(|value| value.get("package_id"))
        .and_then(|value| value.get("enum"))
        .and_then(Value::as_array);
    if string_set(schema_package_ids) != package_ids {
        errors
            .push("experience catalog IDs do not exactly match the profile schema enum".to_owned());
    }

    (style_ids, package_ids)
}

fn benchmark_fixture() -> Result<(Value, Value), String> {
    let manifest: Value = serde_json::from_str(BENCHMARK_FIXTURE)
        .map_err(|error| format!("benchmark extension is invalid JSON: {error}"))?;
    let schema: Value = serde_json::from_str(BENCHMARK_SCHEMA)
        .map_err(|error| format!("benchmark extension schema is invalid JSON: {error}"))?;
    Ok((manifest, schema))
}

fn validate_benchmark_manifest(manifest: &Value, schema: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    if string_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema")
        || string_at(schema, "type") != Some("object")
    {
        errors.push("benchmark extension schema must declare a Draft 2020-12 object".to_owned());
    }
    if string_at(manifest, "schema_version") != Some("0.1.0")
        || string_at(manifest, "study_id") != Some("HOME-EXPRESSION-BENCH-EXT-001")
        || string_at(manifest, "maturity") != Some("PROPOSED_NOT_EVALUATED")
        || string_at(manifest, "execution_status") != Some("NOT_EXECUTED")
    {
        errors.push(
            "benchmark extension identity, maturity or execution state was promoted".to_owned(),
        );
    }
    if !nonempty_string(manifest, "parent_home_freedom_corpus_ref")
        || !nonempty_string(manifest, "purpose")
        || !nonempty_string(manifest, "functional_unit")
    {
        errors
            .push("benchmark extension must declare its Home Freedom parent and scope".to_owned());
    }

    let allowed_statuses = string_set(array_at(manifest, "status_semantics"));
    let cases = array_at(manifest, "cases").cloned().unwrap_or_default();
    if cases.len() < 16 {
        errors.push("benchmark extension requires at least 16 contract mutation cases".to_owned());
    }
    let mut case_ids = BTreeSet::new();
    for case in &cases {
        let id = string_at(case, "case_id").unwrap_or_default();
        if id.is_empty() || !case_ids.insert(id.to_owned()) {
            errors.push(format!(
                "benchmark extension contains a missing or duplicate case ID: {id}"
            ));
        }
        let disposition = string_at(case, "expected_disposition").unwrap_or_default();
        if !allowed_statuses.contains(disposition) {
            errors.push(format!(
                "{id}: expected disposition is absent from status_semantics"
            ));
        }
        if string_at(case, "case_kind") != Some("SYNTHETIC_CONTRACT_MUTATION")
            || string_at(case, "physical_evaluation") != Some("NOT_PERFORMED")
            || string_at(case, "claim_ceiling")
                != Some("CONTRACT_ORACLE_ONLY_NOT_BUILDING_FEASIBILITY")
        {
            errors.push(format!("{id}: case maturity or claim ceiling was promoted"));
        }
        if string_at(case, "source_case_binding_status") != Some("OWNER_CASE_BINDING_PENDING")
            || !case.get("source_case_id").is_some_and(Value::is_null)
        {
            errors.push(format!(
                "{id}: unverified Home Freedom source-case binding was invented"
            ));
        }
        if !nonempty_string(case, "source_population_ref") || !nonempty_string(case, "mutation") {
            errors.push(format!(
                "{id}: source population or mutation description is missing"
            ));
        }
    }

    let expected_dimensions: BTreeSet<String> = ["STYLE", "AMENITY", "BUDGET", "HARD_CONSTRAINT"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let pairs = array_at(manifest, "counterfactual_pairs")
        .cloned()
        .unwrap_or_default();
    if pairs.len() != expected_dimensions.len() {
        errors.push(
            "benchmark extension requires exactly four one-factor counterfactual pairs".to_owned(),
        );
    }
    let mut pair_ids = BTreeSet::new();
    let mut pair_dimensions = BTreeSet::new();
    for pair in &pairs {
        let pair_id = string_at(pair, "pair_id").unwrap_or_default();
        if pair_id.is_empty() || !pair_ids.insert(pair_id.to_owned()) {
            errors.push(format!(
                "counterfactual pair has a missing or duplicate ID: {pair_id}"
            ));
        }
        let dimension = string_at(pair, "changed_dimension").unwrap_or_default();
        if !pair_dimensions.insert(dimension.to_owned()) {
            errors.push(format!("{pair_id}: counterfactual dimension is duplicated"));
        }
        if !["STYLE", "AMENITY", "BUDGET", "HARD_CONSTRAINT"].contains(&dimension) {
            errors.push(format!("{pair_id}: unsupported counterfactual dimension"));
        }
        for key in ["left_case", "right_case"] {
            let case_id = string_at(pair, key).unwrap_or_default();
            if !case_ids.contains(case_id) {
                errors.push(format!("{pair_id}: {key} does not reference a known case"));
            }
        }
        if array_at(pair, "changed_fields").is_none_or(Vec::is_empty)
            || array_at(pair, "held_constant_fields").is_none_or(Vec::is_empty)
            || !nonempty_string(pair, "expected_invariant")
        {
            errors.push(format!(
                "{pair_id}: counterfactual pair lacks changed/held-constant fields or invariant"
            ));
        }
    }
    if pair_dimensions != expected_dimensions {
        errors.push("counterfactual pairs must cover STYLE, AMENITY, BUDGET and HARD_CONSTRAINT exactly once".to_owned());
    }

    errors
}

fn revision_impact_fixture() -> Result<(Value, Value), String> {
    let manifest: Value = serde_json::from_str(REVISION_IMPACT_FIXTURE)
        .map_err(|error| format!("revision impact crosswalk is invalid JSON: {error}"))?;
    let schema: Value = serde_json::from_str(REVISION_IMPACT_SCHEMA)
        .map_err(|error| format!("revision impact schema is invalid JSON: {error}"))?;
    Ok((manifest, schema))
}

fn validate_revision_impact_crosswalk(
    manifest: &Value,
    schema: &Value,
    catalog: &Value,
) -> Vec<String> {
    let mut errors = Vec::new();

    if string_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema")
        || string_at(schema, "type") != Some("object")
    {
        errors.push("revision impact schema must declare a Draft 2020-12 object".to_owned());
    }
    if string_at(manifest, "schema_version") != Some("0.1.0")
        || string_at(manifest, "study_id") != Some("HOME-EXPRESSION-REVISION-IMPACT-001")
        || string_at(manifest, "maturity") != Some("PROPOSED_OWNER_REVIEW_REQUIRED")
        || string_at(manifest, "execution_status") != Some("NO_RUNTIME_INTEGRATION")
    {
        errors.push("revision impact identity, maturity or execution state was promoted".to_owned());
    }
    if string_at(manifest, "primary_revision_owner") != Some("Sol-Atlas #832")
        || string_at(manifest, "geometry_qualification_owner") != Some("Symtropy #683")
        || string_at(manifest, "universal_invalidation_authority") != Some("SOL-ATLAS #832 ONLY")
    {
        errors.push("revision/geometry ownership boundary was changed".to_owned());
    }
    if string_at(catalog, "catalog_revision") != Some(EXPECTED_CATALOG_REVISION)
        || string_at(manifest, "parent_catalog_revision") != Some(EXPECTED_CATALOG_REVISION)
        || string_at(manifest, "parent_catalog_content_sha") != Some(EXPECTED_CATALOG_BLOB_SHA)
    {
        errors.push("revision crosswalk is not bound to the exact v0.1 catalog revision".to_owned());
    }

    let style_ids = object_field_string_set(array_at(catalog, "style_catalog"), "style_id");
    let package_ids =
        object_field_string_set(array_at(catalog, "experience_catalog"), "package_id");

    let records = array_at(manifest, "records").cloned().unwrap_or_default();
    if records.len() != style_ids.len() + package_ids.len() {
        errors.push("revision crosswalk must contain exactly one row per style and package ID".to_owned());
    }

    let allowed_domains = string_set(array_at(manifest, "impact_domains"));
    let expected_domains = string_set(array_at(catalog, "impact_domains"));
    if allowed_domains != expected_domains {
        errors.push("revision crosswalk impact domains differ from the parent catalog".to_owned());
    }

    let mut seen_styles = BTreeSet::new();
    let mut seen_packages = BTreeSet::new();
    for record in &records {
        let id = string_at(record, "id").unwrap_or_default();
        let kind = string_at(record, "kind").unwrap_or_default();
        let change_class = string_at(record, "change_class").unwrap_or_default();
        let domains = string_set(array_at(record, "potential_review_domains"));
        let owner_refs = string_set(array_at(record, "owner_refs"));

        if owner_refs.is_empty() || !owner_refs.contains("Sol-Atlas #832") {
            errors.push(format!("{id}: every record must point to the primary revision owner #832"));
        }
        if record.get("resident_intent_review_required").and_then(Value::as_bool) != Some(true) {
            errors.push(format!("{id}: resident intent review must remain required"));
        }
        if record.get("automatic_invalidation_permitted").and_then(Value::as_bool) != Some(false)
            || record.get("automatic_approval_permitted").and_then(Value::as_bool) != Some(false)
        {
            errors.push(format!("{id}: a proposed crosswalk must not automatically invalidate or approve"));
        }
        if !nonempty_string(record, "evidence_retention_rule") {
            errors.push(format!("{id}: exact evidence-retention rule is required"));
        }
        if !domains.is_subset(&allowed_domains) {
            errors.push(format!("{id}: record contains an undeclared impact domain"));
        }

        match kind {
            "STYLE" => {
                if !style_ids.contains(id) || !seen_styles.insert(id.to_owned()) {
                    errors.push(format!("{id}: style record is missing, duplicated or orphaned"));
                }
                if change_class != "STYLE_INTENT_ONLY" || !domains.is_empty() {
                    errors.push(format!("{id}: style labels must not imply physical impacts by themselves"));
                }
                if record.get("rights_or_context_review_required").and_then(Value::as_bool).is_none() {
                    errors.push(format!("{id}: rights/context review disposition is required"));
                }
            }
            "EXPERIENCE_PACKAGE" => {
                if !package_ids.contains(id) || !seen_packages.insert(id.to_owned()) {
                    errors.push(format!("{id}: experience-package record is missing, duplicated or orphaned"));
                }
                if change_class != "EXPERIENCE_PACKAGE_ADDED"
                    || record.get("review_domains_are_nonbinding_hints").and_then(Value::as_bool)
                        != Some(true)
                    || record.get("actual_artifact_change_required_before_evidence_invalidation")
                        .and_then(Value::as_bool)
                        != Some(true)
                {
                    errors.push(format!(
                        "{id}: package review hints must not trigger invalidation before an actual artifact change"
                    ));
                }
            }
            _ => errors.push(format!("{id}: unsupported crosswalk record kind")),
        }
    }
    if seen_styles != style_ids {
        errors.push("revision crosswalk does not cover the exact style catalog".to_owned());
    }
    if seen_packages != package_ids {
        errors.push("revision crosswalk does not cover the exact experience package catalog".to_owned());
    }

    let policy = manifest.get("global_policy").unwrap_or(&Value::Null);
    if policy.get("automatic_invalidation_permitted").and_then(Value::as_bool) != Some(false)
        || policy.get("automatic_approval_permitted").and_then(Value::as_bool) != Some(false)
        || policy.get("physical_effects_inferred_from_style_label").and_then(Value::as_bool)
            != Some(false)
        || policy.get("impact_hints_are_qualification_evidence").and_then(Value::as_bool)
            != Some(false)
        || policy
            .get("stale_pass_retention_requires_exact_unchanged_dependency_proof")
            .and_then(Value::as_bool)
            != Some(true)
        || string_at(policy, "unknown_dependency_disposition")
            != Some("QUARANTINE_FOR_OWNER_REVIEW")
        || string_at(policy, "resident_choice_authority") != Some("RESIDENT")
        || string_at(policy, "multi_resident_authority_ref") != Some("Sol-Atlas #907")
    {
        errors.push("revision crosswalk global fail-closed or resident-authority policy was weakened".to_owned());
    }
    if string_at(manifest, "claim_ceiling")
        != Some("PROPOSED_OWNER_REVIEW_CROSSWALK_ONLY_NO_RUNTIME_INVALIDATION_OR_PHYSICAL_QUALIFICATION")
    {
        errors.push("revision crosswalk claim ceiling was promoted".to_owned());
    }
    errors
}

fn validate_profile(profile: &Value, schema: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    validate_schema_header(schema, &mut errors);
    let (catalog_style_ids, catalog_package_ids) = validate_catalog(schema, &mut errors);
    match benchmark_fixture() {
        Ok((manifest, benchmark_schema)) => {
            errors.extend(validate_benchmark_manifest(&manifest, &benchmark_schema));
        }
        Err(error) => errors.push(error),
    }
    match revision_impact_fixture() {
        Ok((manifest, crosswalk_schema)) => match serde_json::from_str::<Value>(CATALOG_FIXTURE) {
            Ok(catalog) => errors.extend(validate_revision_impact_crosswalk(
                &manifest,
                &crosswalk_schema,
                &catalog,
            )),
            Err(error) => errors.push(format!("catalog fixture is invalid JSON: {error}")),
        },
        Err(error) => errors.push(error),
    }
    forbidden_score_path(profile, "$", &mut errors);

    if string_at(profile, "schema_version") != Some("0.1.0") {
        errors.push("schema_version must be 0.1.0".to_owned());
    }
    if !nonempty_string(profile, "profile_id")
        || !nonempty_string(profile, "linked_brief_id")
        || !nonempty_string(profile, "linked_design_revision_ref")
    {
        errors.push(
            "profile, exact source brief and design revision references are required".to_owned(),
        );
    }

    let maturity = string_at(profile, "maturity").unwrap_or_default();
    if !["SYNTHETIC", "RESIDENT_AUTHORIZED", "RESTRICTED_PRIVATE"].contains(&maturity) {
        errors.push("unsupported maturity state".to_owned());
    }

    let constraints: BTreeSet<&str> = array_at(profile, "linked_brief_constraint_ids")
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();

    let expression = profile.get("expression_intent").unwrap_or(&Value::Null);
    if !nonempty_string(expression, "statement") {
        errors
            .push("expression_intent.statement must be resident-authored and nonempty".to_owned());
    }
    let context = expression.get("cultural_context").unwrap_or(&Value::Null);
    if context.get("resident_provided").and_then(Value::as_bool) == Some(false)
        && !is_null(context, "context_statement")
    {
        errors.push(
            "non-resident-provided cultural context must not be inserted by the system".to_owned(),
        );
    }

    let style_preferences = profile.get("style_preferences").unwrap_or(&Value::Null);
    let mode = string_at(style_preferences, "mode").unwrap_or_default();
    let entries = array_at(style_preferences, "entries")
        .cloned()
        .unwrap_or_default();

    if !["SINGLE", "BLENDED", "CUSTOM", "NONE_UNDECIDED"].contains(&mode) {
        errors.push("style preference mode is not supported".to_owned());
    }
    if !nonempty_string(style_preferences, "catalog_revision") {
        errors.push("style catalog revision is required".to_owned());
    }

    let mut style_ids = BTreeSet::new();
    let mut contributing_weights = 0_u64;
    let mut primary_count = 0usize;
    let mut secondary_count = 0usize;
    let mut custom_count = 0usize;

    for entry in &entries {
        let style_id = string_at(entry, "style_id").unwrap_or_default();
        let role = string_at(entry, "role").unwrap_or_default();
        if !catalog_style_ids.contains(style_id) {
            errors.push(format!("unknown style_id {style_id}"));
        }
        if !style_ids.insert(style_id) {
            errors.push(format!("duplicate style_id {style_id}"));
        }

        if !nonempty_string(entry, "display_name") {
            errors.push(format!("{style_id}: display_name is required"));
        }

        match role {
            "PRIMARY" | "SECONDARY" => {
                let weight = entry.get("blend_weight_pct").and_then(Value::as_u64);
                match weight {
                    Some(weight) if (1..=100).contains(&weight) => {
                        contributing_weights += weight;
                    }
                    _ => errors.push(format!(
                        "{style_id}: PRIMARY/SECONDARY needs a resident-authored integer weight in 1..=100"
                    )),
                }
                if role == "PRIMARY" {
                    primary_count += 1;
                } else {
                    secondary_count += 1;
                }
            }
            "ACCEPTABLE" | "AVOID" => {
                if !is_null(entry, "blend_weight_pct") {
                    errors.push(format!(
                        "{style_id}: ACCEPTABLE/AVOID entries must not contribute blend weight"
                    ));
                }
            }
            _ => errors.push(format!("{style_id}: unsupported style role {role}")),
        }

        if style_id == "OTHER_CUSTOM" {
            custom_count += 1;
            if !nonempty_string(entry, "custom_definition") {
                errors
                    .push("OTHER_CUSTOM requires a resident-authored custom_definition".to_owned());
            }
        } else if !is_null(entry, "custom_definition") {
            errors.push(format!(
                "{style_id}: custom_definition is only valid for OTHER_CUSTOM"
            ));
        }
    }

    match mode {
        "SINGLE" => {
            if primary_count != 1 || contributing_weights != 100 || secondary_count != 0 {
                errors.push(
                    "SINGLE mode requires exactly one PRIMARY at 100% and no SECONDARY".to_owned(),
                );
            }
        }
        "BLENDED" => {
            if primary_count == 0 || contributing_weights != 100 {
                errors.push(
                    "BLENDED mode requires at least one PRIMARY and weights summing exactly to 100"
                        .to_owned(),
                );
            }
        }
        "CUSTOM" => {
            if custom_count == 0 {
                errors.push("CUSTOM mode requires at least one OTHER_CUSTOM entry".to_owned());
            }
        }
        "NONE_UNDECIDED" => {
            if !entries.is_empty() {
                errors
                    .push("NONE_UNDECIDED mode must not silently assign style entries".to_owned());
            }
        }
        _ => {}
    }

    let options = array_at(profile, "experience_options")
        .cloned()
        .unwrap_or_default();
    let mut option_ids = BTreeSet::new();
    let mut package_ids = BTreeSet::new();

    for option in &options {
        let option_id = string_at(option, "option_id").unwrap_or_default();
        if option_id.is_empty() || !option_ids.insert(option_id) {
            errors.push(format!(
                "missing or duplicate experience option ID: {option_id}"
            ));
        }
        let package_id = string_at(option, "package_id").unwrap_or_default();
        if package_id.is_empty() || !package_ids.insert(package_id) {
            errors.push(format!("{option_id}: missing or duplicate package_id"));
        }
        if !catalog_package_ids.contains(package_id) {
            errors.push(format!(
                "{option_id}: package_id {package_id} is absent from the catalog"
            ));
        }

        let optionality = string_at(option, "optionality").unwrap_or_default();
        if optionality == "RESIDENT_REQUIRED" {
            let requirement = string_at(option, "resident_constraint_ref").unwrap_or_default();
            if requirement.is_empty() || !constraints.contains(requirement) {
                errors.push(format!(
                    "{option_id}: RESIDENT_REQUIRED must reference an exact constraint in the linked resident brief"
                ));
            }
        } else if optionality == "OPTIONAL" && !is_null(option, "resident_constraint_ref") {
            errors.push(format!(
                "{option_id}: optional package must not invent or attach a hard-constraint reference"
            ));
        } else if !["OPTIONAL", "RESIDENT_REQUIRED"].contains(&optionality) {
            errors.push(format!("{option_id}: unsupported optionality state"));
        }

        let impacts = array_at(option, "impact_profile")
            .cloned()
            .unwrap_or_default();
        if impacts.len() != IMPACT_DOMAINS.len() {
            errors.push(format!(
                "{option_id}: expected exactly {} impact domains, got {}",
                IMPACT_DOMAINS.len(),
                impacts.len()
            ));
        }
        let mut seen_domains = BTreeSet::new();

        for impact in &impacts {
            let domain = string_at(impact, "domain").unwrap_or_default();
            if !IMPACT_DOMAINS.contains(&domain) {
                errors.push(format!("{option_id}: unknown impact domain {domain}"));
                continue;
            }
            if !seen_domains.insert(domain) {
                errors.push(format!("{option_id}: duplicate impact domain {domain}"));
            }

            let state = string_at(impact, "state").unwrap_or_default();
            let has_value = !is_null(impact, "value");
            let has_bounds = !is_null(impact, "lower_bound") || !is_null(impact, "upper_bound");
            let has_basis = nonempty_string(impact, "basis_note");
            let has_unit = nonempty_string(impact, "unit");
            let has_evidence = nonempty_string(impact, "evidence_reference");
            let has_method = nonempty_string(impact, "method_reference");

            match state {
                "UNKNOWN" | "NOT_APPLICABLE" => {
                    if has_value || has_bounds || has_unit || has_evidence || has_method {
                        errors.push(format!(
                            "{option_id}/{domain}: {state} must not carry a fabricated numeric value, unit or qualification reference"
                        ));
                    }
                    if !has_basis {
                        errors.push(format!(
                            "{option_id}/{domain}: {state} needs an explicit explanation"
                        ));
                    }
                }
                "ESTIMATED" => {
                    if (!has_value && !has_bounds) || !has_unit || !has_basis {
                        errors.push(format!(
                            "{option_id}/{domain}: ESTIMATED needs a value or interval, unit and basis"
                        ));
                    }
                    if let (Some(low), Some(high)) = (
                        impact.get("lower_bound").and_then(Value::as_f64),
                        impact.get("upper_bound").and_then(Value::as_f64),
                    ) {
                        if low > high {
                            errors.push(format!(
                                "{option_id}/{domain}: lower_bound exceeds upper_bound"
                            ));
                        }
                    }
                }
                "MEASURED" | "QUALIFIED" => {
                    if (!has_value && !has_bounds) || !has_unit || !has_evidence || !has_method {
                        errors.push(format!(
                            "{option_id}/{domain}: {state} needs value/interval, unit, evidence and method references"
                        ));
                    }
                }
                _ => errors.push(format!(
                    "{option_id}/{domain}: unsupported impact state {state}"
                )),
            }
        }

        for required_domain in IMPACT_DOMAINS {
            if !seen_domains.contains(required_domain) {
                errors.push(format!(
                    "{option_id}: missing impact domain {required_domain}"
                ));
            }
        }
    }

    let protocol = profile.get("evaluation_protocol").unwrap_or(&Value::Null);
    if string_at(protocol, "selection_authority") != Some("RESIDENT")
        || protocol.get("automatic_approval").and_then(Value::as_bool) != Some(false)
        || protocol
            .get("universal_aesthetic_score")
            .and_then(Value::as_bool)
            != Some(false)
        || !nonempty_string(protocol, "review_plan")
    {
        errors.push("evaluation protocol must preserve resident authority and prohibit automatic/universal aesthetic approval".to_owned());
    }

    let privacy = profile.get("privacy").unwrap_or(&Value::Null);
    if maturity == "SYNTHETIC"
        && (privacy
            .get("contains_direct_identifiers")
            .and_then(Value::as_bool)
            != Some(false)
            || string_at(privacy, "sharing_mode") != Some("SYNTHETIC_PUBLIC")
            || string_at(privacy, "redaction_status") != Some("REVIEWED_NO_DIRECT_IDENTIFIERS"))
    {
        errors.push("public synthetic profile fails the privacy/redaction contract".to_owned());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn embedded_profile() -> Result<Value, String> {
    serde_json::from_str(PROFILE_FIXTURE)
        .map_err(|error| format!("synthetic expression profile is invalid JSON: {error}"))
}

fn embedded_schema() -> Result<Value, String> {
    serde_json::from_str(PROFILE_SCHEMA)
        .map_err(|error| format!("expression schema is invalid JSON: {error}"))
}

fn self_test() -> Result<(), String> {
    let profile = embedded_profile()?;
    let schema = embedded_schema()?;
    validate_profile(&profile, &schema).map_err(|errors| errors.join("\n"))?;
    println!("PASS: synthetic home expression profile satisfies v0.1 owner contract");
    println!(
        "PASS: resident style blend, optional packages and explicit UNKNOWN impacts preserved"
    );
    println!("PASS: no universal aesthetic score or automatic approval path");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = if args.is_empty() || (args.len() == 1 && args[0] == "--self-test") {
        self_test()
    } else if args.len() == 2 && args[0] == "--profile" {
        let profile = fs::read_to_string(&args[1])
            .map_err(|error| format!("cannot read {}: {error}", args[1]));
        profile.and_then(|source| {
            let value: Value = serde_json::from_str(&source)
                .map_err(|error| format!("profile is invalid JSON: {error}"))?;
            let schema = embedded_schema()?;
            validate_profile(&value, &schema).map_err(|errors| errors.join("\n"))?;
            println!(
                "PASS: {} satisfies the v0.1 home expression contract",
                args[1]
            );
            Ok(())
        })
    } else {
        Err("usage: home-expression-library-qualifier [--self-test | --profile PATH]".to_owned())
    };

    if let Err(error) = result {
        eprintln!("FAIL: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> (Value, Value) {
        (
            embedded_profile().expect("valid embedded fixture"),
            embedded_schema().expect("valid embedded schema"),
        )
    }

    #[test]
    fn accepts_synthetic_highveld_profile() {
        let (profile, schema) = fixture();
        assert!(validate_profile(&profile, &schema).is_ok());
    }

    #[test]
    fn supports_no_style_selected() {
        let (mut profile, schema) = fixture();
        profile["style_preferences"]["mode"] = json!("NONE_UNDECIDED");
        profile["style_preferences"]["entries"] = json!([]);
        assert!(validate_profile(&profile, &schema).is_ok());
    }

    #[test]
    fn accepts_catalog_bound_revision_impact_crosswalk() {
        let (manifest, schema) =
            revision_impact_fixture().expect("valid revision impact fixture");
        let catalog: Value =
            serde_json::from_str(CATALOG_FIXTURE).expect("valid expression catalog");
        assert!(validate_revision_impact_crosswalk(&manifest, &schema, &catalog).is_empty());
    }

    #[test]
    fn rejects_automatic_evidence_invalidation_from_review_hints() {
        let (mut manifest, schema) =
            revision_impact_fixture().expect("valid revision impact fixture");
        let catalog: Value =
            serde_json::from_str(CATALOG_FIXTURE).expect("valid expression catalog");
        manifest["records"][20]["automatic_invalidation_permitted"] = json!(true);
        assert!(!validate_revision_impact_crosswalk(&manifest, &schema, &catalog).is_empty());
    }

    #[test]
    fn rejects_style_label_physical_impact_inference() {
        let (mut manifest, schema) =
            revision_impact_fixture().expect("valid revision impact fixture");
        let catalog: Value =
            serde_json::from_str(CATALOG_FIXTURE).expect("valid expression catalog");
        manifest["records"][0]["potential_review_domains"] = json!(["STRUCTURE_FIRE_MOISTURE_ACOUSTICS"]);
        assert!(!validate_revision_impact_crosswalk(&manifest, &schema, &catalog).is_empty());
    }

    #[test]
    fn rejects_stale_parent_catalog_binding() {
        let (mut manifest, schema) =
            revision_impact_fixture().expect("valid revision impact fixture");
        let catalog: Value =
            serde_json::from_str(CATALOG_FIXTURE).expect("valid expression catalog");
        manifest["parent_catalog_content_sha"] = json!("0000000000000000000000000000000000000000");
        assert!(!validate_revision_impact_crosswalk(&manifest, &schema, &catalog).is_empty());
    }

    #[test]
    fn rejects_missing_counterfactual_dimension() {
        let (mut manifest, schema) = benchmark_fixture().expect("valid benchmark fixture");
        manifest["counterfactual_pairs"][0]["changed_dimension"] = json!("UNDECLARED");
        assert!(!validate_benchmark_manifest(&manifest, &schema).is_empty());
    }

    #[test]
    fn rejects_schema_catalog_style_drift() {
        let (profile, mut schema) = fixture();
        schema["properties"]["style_preferences"]["properties"]["entries"]["items"]["properties"]
            ["style_id"]["enum"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_blend_weight_drift() {
        let (mut profile, schema) = fixture();
        profile["style_preferences"]["entries"][0]["blend_weight_pct"] = json!(60);
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_unknown_impact_coerced_to_zero() {
        let (mut profile, schema) = fixture();
        profile["experience_options"][0]["impact_profile"][1]["value"] = json!(0);
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_missing_impact_domain() {
        let (mut profile, schema) = fixture();
        profile["experience_options"][0]["impact_profile"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_required_package_without_brief_constraint() {
        let (mut profile, schema) = fixture();
        profile["experience_options"][0]["optionality"] = json!("RESIDENT_REQUIRED");
        profile["experience_options"][0]["resident_constraint_ref"] = json!("C_NOT_IN_BRIEF");
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_measured_claim_without_evidence_and_method() {
        let (mut profile, schema) = fixture();
        let impact = &mut profile["experience_options"][0]["impact_profile"][0];
        impact["state"] = json!("MEASURED");
        impact["value"] = json!(100.0);
        impact["unit"] = json!("m2");
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_universal_beauty_score() {
        let (mut profile, schema) = fixture();
        profile["beauty_score"] = json!(0.9);
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_custom_style_without_resident_definition() {
        let (mut profile, schema) = fixture();
        profile["style_preferences"]["mode"] = json!("CUSTOM");
        profile["style_preferences"]["entries"][3]["resident_selected"] = json!(true);
        profile["style_preferences"]["entries"][3]["custom_definition"] = Value::Null;
        assert!(validate_profile(&profile, &schema).is_err());
    }
}
