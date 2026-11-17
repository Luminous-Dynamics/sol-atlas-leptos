// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
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
const HOME_BRIEF_FIXTURE: &str =
    include_str!("../../docs/research/home-freedom/HOME_DESIGN_BRIEF_SYNTHETIC_EXAMPLE_V0.1.json");
const EXPECTED_BRIEF_ARTIFACT_REF: &str =
    "docs/research/home-freedom/HOME_DESIGN_BRIEF_SYNTHETIC_EXAMPLE_V0.1.json";
const EXPECTED_BRIEF_BLOB_SHA: &str = "0090e2f0c883c62817d1b6da51d1a2dbe92a477f";
const HOME_CANDIDATE_SET_FIXTURE: &str = include_str!(
    "../../docs/research/home-freedom/HOME_DESIGN_CANDIDATE_SET_SYNTHETIC_EXAMPLE_V0.1.json"
);
const EXPECTED_CANDIDATE_SET_ARTIFACT_REF: &str =
    "docs/research/home-freedom/HOME_DESIGN_CANDIDATE_SET_SYNTHETIC_EXAMPLE_V0.1.json";
const EXPECTED_CANDIDATE_SET_BLOB_SHA: &str = "030e6c0e71cef1a936738910409396a9a48dedcf";
const DECISION_RECEIPT_FIXTURE: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_DECISION_RECEIPT_SYNTHETIC_HIGHVELD_V0.1.json"
);
const DECISION_RECEIPT_SCHEMA: &str = include_str!(
    "../../docs/research/home-expression/HOME_EXPRESSION_DECISION_RECEIPT_V0.1.schema.json"
);
const EXPECTED_PROFILE_ARTIFACT_REF: &str =
    "docs/research/home-expression/HOME_EXPRESSION_PROFILE_SYNTHETIC_HIGHVELD_V0.1.json";
const EXPECTED_PROFILE_BLOB_SHA: &str = "a7f34f081c793e0794ad071b3694fe90206ca94a";

const REQUIRED_SOURCE_HARD_GATE_IDS: &[&str] = &[
    "C_SLEEP_02",
    "C_WORK_01",
    "C_ACCESS_01",
    "C_PHASE_01",
    "C_WATER_01",
    "SITE_RIGHTS",
    "AUTHORITY_CODE",
    "STRUCTURE_FIRE",
    "MOISTURE_DRAINAGE",
    "COST_LOCAL_SUPPLY",
];

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
    if string_at(manifest, "source_population_state") != Some("ONE_SYNTHETIC_BASELINE_ONLY")
        || manifest.get("source_case_count").and_then(Value::as_u64) != Some(1)
        || !nonempty_string(manifest, "source_population_note")
    {
        errors.push(
            "benchmark extension must disclose its one-case synthetic source population".to_owned(),
        );
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
        if string_at(case, "source_case_binding_status") != Some("BOUND")
            || string_at(case, "source_case_id") != Some("SYNTH-HV-CASE-001")
            || string_at(case, "derivation_kind") != Some("ONE_FACTOR_COUNTERFACTUAL_MUTATION")
        {
            errors.push(format!(
                "{id}: extension row is not bound to the exact synthetic Home Freedom baseline"
            ));
        }
        let expected_artifacts: BTreeSet<String> = [
            "docs/research/home-freedom/HOME_DESIGN_BRIEF_SYNTHETIC_EXAMPLE_V0.1.json|0090e2f0c883c62817d1b6da51d1a2dbe92a477f",
            "docs/research/home-freedom/HOME_DESIGN_CANDIDATE_SET_SYNTHETIC_EXAMPLE_V0.1.json|030e6c0e71cef1a936738910409396a9a48dedcf",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let actual_artifacts: BTreeSet<String> = array_at(case, "source_artifact_refs")
            .into_iter()
            .flatten()
            .filter_map(|artifact| {
                let path = string_at(artifact, "path")?;
                let sha = string_at(artifact, "blob_sha")?;
                Some(format!("{path}|{sha}"))
            })
            .collect();
        if actual_artifacts != expected_artifacts {
            errors.push(format!(
                "{id}: source artifacts do not bind the exact parent brief and candidate-set blobs"
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
        errors
            .push("revision impact identity, maturity or execution state was promoted".to_owned());
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
        errors
            .push("revision crosswalk is not bound to the exact v0.1 catalog revision".to_owned());
    }

    let style_ids = object_field_string_set(array_at(catalog, "style_catalog"), "style_id");
    let package_ids =
        object_field_string_set(array_at(catalog, "experience_catalog"), "package_id");

    let records = array_at(manifest, "records").cloned().unwrap_or_default();
    if records.len() != style_ids.len() + package_ids.len() {
        errors.push(
            "revision crosswalk must contain exactly one row per style and package ID".to_owned(),
        );
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
            errors.push(format!(
                "{id}: every record must point to the primary revision owner #832"
            ));
        }
        if record
            .get("resident_intent_review_required")
            .and_then(Value::as_bool)
            != Some(true)
        {
            errors.push(format!("{id}: resident intent review must remain required"));
        }
        if record
            .get("automatic_invalidation_permitted")
            .and_then(Value::as_bool)
            != Some(false)
            || record
                .get("automatic_approval_permitted")
                .and_then(Value::as_bool)
                != Some(false)
        {
            errors.push(format!(
                "{id}: a proposed crosswalk must not automatically invalidate or approve"
            ));
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
                    errors.push(format!(
                        "{id}: style record is missing, duplicated or orphaned"
                    ));
                }
                if change_class != "STYLE_INTENT_ONLY" || !domains.is_empty() {
                    errors.push(format!(
                        "{id}: style labels must not imply physical impacts by themselves"
                    ));
                }
                if record
                    .get("rights_or_context_review_required")
                    .and_then(Value::as_bool)
                    .is_none()
                {
                    errors.push(format!(
                        "{id}: rights/context review disposition is required"
                    ));
                }
            }
            "EXPERIENCE_PACKAGE" => {
                if !package_ids.contains(id) || !seen_packages.insert(id.to_owned()) {
                    errors.push(format!(
                        "{id}: experience-package record is missing, duplicated or orphaned"
                    ));
                }
                if change_class != "EXPERIENCE_PACKAGE_ADDED"
                    || record
                        .get("review_domains_are_nonbinding_hints")
                        .and_then(Value::as_bool)
                        != Some(true)
                    || record
                        .get("actual_artifact_change_required_before_evidence_invalidation")
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
        errors.push(
            "revision crosswalk does not cover the exact experience package catalog".to_owned(),
        );
    }

    let policy = manifest.get("global_policy").unwrap_or(&Value::Null);
    if policy
        .get("automatic_invalidation_permitted")
        .and_then(Value::as_bool)
        != Some(false)
        || policy
            .get("automatic_approval_permitted")
            .and_then(Value::as_bool)
            != Some(false)
        || policy
            .get("physical_effects_inferred_from_style_label")
            .and_then(Value::as_bool)
            != Some(false)
        || policy
            .get("impact_hints_are_qualification_evidence")
            .and_then(Value::as_bool)
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
        errors.push(
            "revision crosswalk global fail-closed or resident-authority policy was weakened"
                .to_owned(),
        );
    }
    if string_at(manifest, "claim_ceiling")
        != Some(
            "PROPOSED_OWNER_REVIEW_CROSSWALK_ONLY_NO_RUNTIME_INVALIDATION_OR_PHYSICAL_QUALIFICATION",
        )
    {
        errors.push("revision crosswalk claim ceiling was promoted".to_owned());
    }
    errors
}

fn validate_profile_source_bindings(
    profile: &Value,
    source_brief: &Value,
    source_candidate_set: &Value,
    catalog: &Value,
    errors: &mut Vec<String>,
) {
    let brief_id = string_at(source_brief, "brief_id").unwrap_or_default();
    if string_at(profile, "linked_brief_id") != Some(brief_id) {
        errors.push(
            "expression profile brief ID does not match the embedded source brief".to_owned(),
        );
    }
    if string_at(profile, "linked_brief_artifact_ref") != Some(EXPECTED_BRIEF_ARTIFACT_REF)
        || string_at(profile, "linked_brief_content_sha") != Some(EXPECTED_BRIEF_BLOB_SHA)
    {
        errors.push(
            "expression profile does not bind the exact source brief path and blob digest"
                .to_owned(),
        );
    }

    let expected_constraints =
        object_field_string_set(array_at(source_brief, "constraints"), "constraint_id");
    let actual_constraints = string_set(array_at(profile, "linked_brief_constraint_ids"));
    if expected_constraints != actual_constraints {
        errors.push(
            "expression profile linked constraint IDs differ from the exact source brief"
                .to_owned(),
        );
    }

    let candidate_set_id = string_at(source_candidate_set, "candidate_set_id").unwrap_or_default();
    let candidate_brief_id = string_at(source_candidate_set, "brief_id").unwrap_or_default();
    let candidate_revision = string_at(source_candidate_set, "design_revision").unwrap_or_default();
    if string_at(profile, "linked_candidate_set_id") != Some(candidate_set_id)
        || string_at(profile, "linked_candidate_set_id") != Some("SYNTH-HV-CASE-001")
        || string_at(profile, "linked_candidate_set_artifact_ref")
            != Some(EXPECTED_CANDIDATE_SET_ARTIFACT_REF)
        || string_at(profile, "linked_candidate_set_content_sha")
            != Some(EXPECTED_CANDIDATE_SET_BLOB_SHA)
    {
        errors.push(
            "expression profile does not bind the exact Home Freedom candidate-set path and digest"
                .to_owned(),
        );
    }
    if candidate_brief_id != brief_id
        || string_at(profile, "linked_design_revision_ref") != Some(candidate_revision)
        || candidate_revision != "HFC-SYNTH-REV-0"
    {
        errors.push("expression profile, source brief and candidate set do not share the exact brief/revision identity".to_owned());
    }

    let preferences = profile.get("style_preferences").unwrap_or(&Value::Null);
    if string_at(preferences, "catalog_revision") != string_at(catalog, "catalog_revision")
        || string_at(preferences, "catalog_revision") != Some(EXPECTED_CATALOG_REVISION)
        || string_at(preferences, "catalog_content_sha") != Some(EXPECTED_CATALOG_BLOB_SHA)
    {
        errors.push("expression profile does not bind the exact style/experience catalog revision and digest".to_owned());
    }
}

fn decision_receipt_fixture() -> Result<(Value, Value), String> {
    let receipt: Value = serde_json::from_str(DECISION_RECEIPT_FIXTURE)
        .map_err(|error| format!("decision receipt fixture is invalid JSON: {error}"))?;
    let schema: Value = serde_json::from_str(DECISION_RECEIPT_SCHEMA)
        .map_err(|error| format!("decision receipt schema is invalid JSON: {error}"))?;
    Ok((receipt, schema))
}

fn candidate_technical_disposition(gates: &[Value]) -> &'static str {
    if gates.is_empty()
        || gates
            .iter()
            .any(|gate| !matches!(string_at(gate, "state"), Some("PASS" | "FAIL" | "UNKNOWN")))
    {
        return "CONDITIONAL_CONCEPT_ONLY";
    }
    if gates
        .iter()
        .any(|gate| string_at(gate, "state") == Some("FAIL"))
    {
        "BLOCKED"
    } else if gates
        .iter()
        .any(|gate| string_at(gate, "state") == Some("UNKNOWN"))
    {
        "CONDITIONAL_CONCEPT_ONLY"
    } else {
        "ELIGIBLE_FOR_RESIDENT_PREFERENCE_COMPARISON"
    }
}

fn preference_event_is_well_formed(value: &Value, design_revision: &str) -> bool {
    string_at(value, "participant_ref").is_some_and(|participant| {
        participant.starts_with("participant:") && participant.len() > "participant:".len()
    }) && string_at(value, "event_ref")
        .is_some_and(|event_ref| event_ref.starts_with("urn:mycelix:"))
        && string_at(value, "design_revision") == Some(design_revision)
        && string_at(value, "event_content_sha256").is_some_and(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        && string_at(value, "decision_scope") == Some("CONCEPT_DIRECTION_ONLY")
        && string_at(value, "provenance_status") == Some("PENDING_MYCELIX_5149_VERIFICATION")
}

fn preference_graph_has_cycle(pairs: &[Value]) -> bool {
    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut indegree: BTreeMap<String, usize> = BTreeMap::new();

    for pair in pairs {
        let left = string_at(pair, "left_candidate_id")
            .unwrap_or_default()
            .to_owned();
        let right = string_at(pair, "right_candidate_id")
            .unwrap_or_default()
            .to_owned();
        let direction = match string_at(pair, "decision") {
            Some("PREFER_LEFT") => Some((left, right)),
            Some("PREFER_RIGHT") => Some((right, left)),
            _ => None,
        };
        if let Some((preferred, less_preferred)) = direction {
            indegree.entry(preferred.clone()).or_insert(0);
            indegree.entry(less_preferred.clone()).or_insert(0);
            let inserted = edges
                .entry(preferred)
                .or_default()
                .insert(less_preferred.clone());
            if inserted {
                *indegree.entry(less_preferred).or_insert(0) += 1;
            }
        }
    }

    let mut ready: VecDeque<String> = indegree
        .iter()
        .filter_map(|(node, degree)| (*degree == 0).then_some(node.clone()))
        .collect();
    let mut visited = 0usize;

    while let Some(node) = ready.pop_front() {
        visited += 1;
        if let Some(neighbors) = edges.get(&node) {
            for neighbor in neighbors {
                if let Some(degree) = indegree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    visited != indegree.len()
}

fn validate_resident_choice(
    resident_choice: &Value,
    source_candidate_ids: &BTreeSet<String>,
    design_revision: &str,
    errors: &mut Vec<String>,
) {
    let state = string_at(resident_choice, "state").unwrap_or_default();
    let selected = resident_choice.get("selected_candidate_id");
    let event = resident_choice
        .get("decision_event")
        .unwrap_or(&Value::Null);
    let pairwise = array_at(resident_choice, "pairwise_preferences")
        .cloned()
        .unwrap_or_default();

    if string_at(resident_choice, "authority") != Some("RESIDENT") {
        errors.push("resident choice authority must remain with the resident".to_owned());
    }

    match state {
        "NOT_RECORDED" => {
            if !selected.is_some_and(Value::is_null)
                || !event.is_null()
                || pairwise
                    .iter()
                    .any(|pair| string_at(pair, "decision") != Some("UNDECIDED"))
            {
                errors.push(
                    "NOT_RECORDED must not contain an inferred selection or preference".to_owned(),
                );
            }
        }
        "UNDECIDED" => {
            if !selected.is_some_and(Value::is_null)
                || !preference_event_is_well_formed(event, design_revision)
                || pairwise
                    .iter()
                    .any(|pair| string_at(pair, "decision") != Some("UNDECIDED"))
            {
                errors.push(
                    "explicit UNDECIDED requires a pending provenance event and no preference"
                        .to_owned(),
                );
            }
        }
        "PREFERENCES_RECORDED" => {
            if !selected.is_some_and(Value::is_null)
                || !preference_event_is_well_formed(event, design_revision)
                || !pairwise
                    .iter()
                    .any(|pair| !matches!(string_at(pair, "decision"), Some("UNDECIDED")))
            {
                errors.push("recorded preferences require an event reference and at least one explicit comparison".to_owned());
            }
        }
        "CONCEPT_DIRECTION_SELECTED" => {
            let selected_id = selected.and_then(Value::as_str).unwrap_or_default();
            if !source_candidate_ids.contains(selected_id)
                || !preference_event_is_well_formed(event, design_revision)
            {
                errors.push("concept direction selection must reference a source candidate and provenance event".to_owned());
            }
        }
        "INCOMPARABLE" => {
            if !selected.is_some_and(Value::is_null)
                || !preference_event_is_well_formed(event, design_revision)
                || !pairwise
                    .iter()
                    .any(|pair| string_at(pair, "decision") == Some("INCOMPARABLE"))
            {
                errors.push(
                    "INCOMPARABLE requires an explicit incomparable pair and provenance event"
                        .to_owned(),
                );
            }
        }
        _ => errors.push("unsupported resident choice state".to_owned()),
    }

    let mut seen_pairs = BTreeSet::new();
    for pair in &pairwise {
        let left = string_at(pair, "left_candidate_id").unwrap_or_default();
        let right = string_at(pair, "right_candidate_id").unwrap_or_default();
        if left == right
            || !source_candidate_ids.contains(left)
            || !source_candidate_ids.contains(right)
        {
            errors.push(
                "resident pairwise decision must reference two distinct source candidates"
                    .to_owned(),
            );
        }
        let normalized = if left <= right {
            format!("{left}|{right}")
        } else {
            format!("{right}|{left}")
        };
        if !seen_pairs.insert(normalized) {
            errors.push(
                "resident pairwise decision contains a duplicate or reversed duplicate pair"
                    .to_owned(),
            );
        }
        if !matches!(
            string_at(pair, "decision"),
            Some("UNDECIDED" | "PREFER_LEFT" | "PREFER_RIGHT" | "EQUIVALENT" | "INCOMPARABLE")
        ) || !nonempty_string(pair, "basis")
        {
            errors.push(
                "resident pairwise decision has an unsupported state or missing basis".to_owned(),
            );
        }
    }
    if preference_graph_has_cycle(&pairwise) {
        errors.push(
            "resident pairwise preferences contain a contradictory directed cycle".to_owned(),
        );
    }
}

fn validate_decision_receipt(
    receipt: &Value,
    schema: &Value,
    profile: &Value,
    brief: &Value,
    source_candidate_set: &Value,
) -> Vec<String> {
    let mut errors = Vec::new();

    if string_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema")
        || string_at(schema, "type") != Some("object")
    {
        errors.push("decision receipt schema must declare a Draft 2020-12 object".to_owned());
    }
    if string_at(receipt, "schema_version") != Some("0.1.0")
        || string_at(receipt, "study_id") != Some("HOME-EXPRESSION-DECISION-RECEIPT-001")
        || string_at(receipt, "maturity") != Some("SYNTHETIC_CONTRACT_FIXTURE")
        || string_at(receipt, "execution_status") != Some("NOT_EXECUTED")
    {
        errors.push(
            "decision receipt identity, maturity or execution status was promoted".to_owned(),
        );
    }

    let binding = receipt.get("source_binding").unwrap_or(&Value::Null);
    if string_at(binding, "profile_id") != string_at(profile, "profile_id")
        || string_at(binding, "profile_artifact_ref") != Some(EXPECTED_PROFILE_ARTIFACT_REF)
        || string_at(binding, "profile_content_sha") != Some(EXPECTED_PROFILE_BLOB_SHA)
    {
        errors.push(
            "decision receipt does not bind the exact resident expression profile path and digest"
                .to_owned(),
        );
    }
    if string_at(binding, "brief_id") != string_at(brief, "brief_id")
        || string_at(binding, "brief_id") != string_at(profile, "linked_brief_id")
        || string_at(binding, "brief_artifact_ref") != Some(EXPECTED_BRIEF_ARTIFACT_REF)
        || string_at(binding, "brief_content_sha") != Some(EXPECTED_BRIEF_BLOB_SHA)
    {
        errors.push(
            "decision receipt does not bind the exact Home Freedom brief path and digest"
                .to_owned(),
        );
    }
    if string_at(binding, "candidate_set_id") != string_at(source_candidate_set, "candidate_set_id")
        || string_at(binding, "candidate_set_id") != string_at(profile, "linked_candidate_set_id")
        || string_at(binding, "candidate_set_artifact_ref")
            != Some(EXPECTED_CANDIDATE_SET_ARTIFACT_REF)
        || string_at(binding, "candidate_set_content_sha") != Some(EXPECTED_CANDIDATE_SET_BLOB_SHA)
        || string_at(binding, "design_revision")
            != string_at(source_candidate_set, "design_revision")
        || string_at(binding, "design_revision") != string_at(profile, "linked_design_revision_ref")
    {
        errors.push("decision receipt does not bind the exact Home Freedom candidate set and design revision".to_owned());
    }

    let source_candidates = array_at(source_candidate_set, "candidates")
        .cloned()
        .unwrap_or_default();
    let candidate_receipts = array_at(receipt, "candidate_receipts")
        .cloned()
        .unwrap_or_default();
    if source_candidates.len() != candidate_receipts.len() || source_candidates.is_empty() {
        errors.push("decision receipt must cover every and only source candidate".to_owned());
    }

    let mut seen_candidates = BTreeSet::new();
    let mut all_candidates_blocked = true;
    let mut any_candidate_preference_eligible = false;
    for candidate_receipt in &candidate_receipts {
        let candidate_id = string_at(candidate_receipt, "candidate_id").unwrap_or_default();
        if candidate_id.is_empty() || !seen_candidates.insert(candidate_id.to_owned()) {
            errors.push(format!(
                "decision receipt has a missing or duplicate candidate ID: {candidate_id}"
            ));
        }
        let source_candidate = source_candidates
            .iter()
            .find(|candidate| string_at(candidate, "candidate_id") == Some(candidate_id));
        let Some(source_candidate) = source_candidate else {
            errors.push(format!(
                "{candidate_id}: candidate is not present in the exact source candidate set"
            ));
            continue;
        };

        if string_at(candidate_receipt, "source_disposition")
            != string_at(source_candidate, "disposition")
            || string_at(candidate_receipt, "space_program_state")
                != string_at(source_candidate, "space_program_state")
        {
            errors.push(format!(
                "{candidate_id}: source disposition or symbolic-program state drifted"
            ));
        }

        let source_gates = array_at(source_candidate, "hard_gate_results")
            .cloned()
            .unwrap_or_default();
        let receipt_gates = array_at(candidate_receipt, "hard_gate_results")
            .cloned()
            .unwrap_or_default();
        if source_gates.len() != receipt_gates.len() {
            errors.push(format!(
                "{candidate_id}: decision receipt gate count differs from source"
            ));
        }
        let source_gate_ids = object_field_string_set(Some(&source_gates), "gate_id");
        let receipt_gate_ids = object_field_string_set(Some(&receipt_gates), "gate_id");
        let required_source_gate_ids: BTreeSet<String> = REQUIRED_SOURCE_HARD_GATE_IDS
            .iter()
            .map(|gate_id| (*gate_id).to_owned())
            .collect();
        if source_gates.len() != REQUIRED_SOURCE_HARD_GATE_IDS.len()
            || source_gate_ids != required_source_gate_ids
        {
            errors.push(format!(
                "{candidate_id}: source must preserve the exact ten required hard-gate IDs"
            ));
        }
        if receipt_gates
            .iter()
            .any(|gate| !matches!(string_at(gate, "state"), Some("PASS" | "FAIL" | "UNKNOWN")))
        {
            errors.push(format!(
                "{candidate_id}: receipt contains an unsupported hard-gate state"
            ));
        }
        if source_gate_ids != receipt_gate_ids || source_gate_ids.len() != source_gates.len() {
            errors.push(format!(
                "{candidate_id}: hard-gate IDs are missing, duplicated or invented"
            ));
        }

        for gate in &receipt_gates {
            let gate_id = string_at(gate, "gate_id").unwrap_or_default();
            let Some(source_gate) = source_gates
                .iter()
                .find(|source| string_at(source, "gate_id") == Some(gate_id))
            else {
                continue;
            };
            if string_at(gate, "state") != string_at(source_gate, "state")
                || string_at(gate, "reason") != string_at(source_gate, "reason")
                || gate.get("evidence_ref") != source_gate.get("evidence_ref")
            {
                errors.push(format!(
                    "{candidate_id}/{gate_id}: gate state or evidence differs from exact source"
                ));
            }
        }

        let expected_disposition = candidate_technical_disposition(&receipt_gates);
        if string_at(candidate_receipt, "technical_disposition") != Some(expected_disposition) {
            errors.push(format!(
                "{candidate_id}: technical disposition does not follow fail-closed hard-gate semantics"
            ));
        }
        all_candidates_blocked &= expected_disposition == "BLOCKED";
        any_candidate_preference_eligible |=
            expected_disposition == "ELIGIBLE_FOR_RESIDENT_PREFERENCE_COMPARISON";
    }
    let source_candidate_ids = object_field_string_set(Some(&source_candidates), "candidate_id");
    if seen_candidates != source_candidate_ids {
        errors.push("decision receipt does not cover the exact source candidate ID set".to_owned());
    }

    let policy = receipt.get("comparison_policy").unwrap_or(&Value::Null);
    let expected_pareto_state = if all_candidates_blocked || !any_candidate_preference_eligible {
        "NOT_COMPUTABLE_UNTIL_REQUIRED_GATES_PASS_AND_OBJECTIVES_ARE_ADMITTED"
    } else {
        "READY_FOR_OWNER_NATIVE_PARETO_COMPARISON"
    };
    if string_at(policy, "comparison_layer") != Some("EXPRESSION_PREVIEW_ONLY")
        || string_at(policy, "technical_pareto_status") != Some(expected_pareto_state)
        || policy.get("candidates_ranked").and_then(Value::as_bool) != Some(false)
        || !policy.get("scalar_score").is_some_and(Value::is_null)
        || policy.get("automatic_selection").and_then(Value::as_bool) != Some(false)
        || policy
            .get("feasibility_claim_permitted")
            .and_then(Value::as_bool)
            != Some(false)
        || policy
            .get("pareto_comparison_requires_all_hard_gates_pass")
            .and_then(Value::as_bool)
            != Some(true)
        || string_at(policy, "preference_source") != Some("RESIDENT_AUTHORED_PROFILE_ONLY")
        || string_at(policy, "unknown_gate_semantics") != Some("CONDITIONAL_NOT_PASS")
    {
        errors.push("decision comparison policy weakens resident authority, unknown handling or non-ranking boundary".to_owned());
    }

    let resident_choice = receipt.get("resident_choice").unwrap_or(&Value::Null);
    let decision_revision = string_at(binding, "design_revision").unwrap_or_default();
    validate_resident_choice(
        resident_choice,
        &source_candidate_ids,
        decision_revision,
        &mut errors,
    );

    if string_at(receipt, "claim_ceiling")
        != Some("CONTRACT_ONLY_NO_TECHNICAL_FEASIBILITY_OR_PREFERENCE_WINNER")
    {
        errors.push("decision receipt claim ceiling was promoted".to_owned());
    }
    errors
}

fn validate_profile(profile: &Value, schema: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    validate_schema_header(schema, &mut errors);
    let (catalog_style_ids, catalog_package_ids) = validate_catalog(schema, &mut errors);
    match (
        serde_json::from_str::<Value>(HOME_BRIEF_FIXTURE),
        serde_json::from_str::<Value>(HOME_CANDIDATE_SET_FIXTURE),
        serde_json::from_str::<Value>(CATALOG_FIXTURE),
    ) {
        (Ok(source_brief), Ok(source_candidate_set), Ok(catalog)) => {
            validate_profile_source_bindings(
                profile,
                &source_brief,
                &source_candidate_set,
                &catalog,
                &mut errors,
            );
        }
        (Err(error), _, _) => {
            errors.push(format!("embedded source brief is invalid JSON: {error}"))
        }
        (_, Err(error), _) => errors.push(format!(
            "embedded source candidate set is invalid JSON: {error}"
        )),
        (_, _, Err(error)) => errors.push(format!("embedded catalog is invalid JSON: {error}")),
    }
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
    let (receipt, receipt_schema) = decision_receipt_fixture()?;
    let source_brief: Value = serde_json::from_str(HOME_BRIEF_FIXTURE)
        .map_err(|error| format!("source brief is invalid JSON: {error}"))?;
    let source_candidate_set: Value = serde_json::from_str(HOME_CANDIDATE_SET_FIXTURE)
        .map_err(|error| format!("source candidate set is invalid JSON: {error}"))?;
    let receipt_errors = validate_decision_receipt(
        &receipt,
        &receipt_schema,
        &profile,
        &source_brief,
        &source_candidate_set,
    );
    if !receipt_errors.is_empty() {
        return Err(receipt_errors.join("\n"));
    }
    println!("PASS: synthetic home expression profile satisfies v0.1 owner contract");
    println!(
        "PASS: resident decision receipt preserves both source candidates and all UNKNOWN gates"
    );
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

    fn decision_fixture() -> (Value, Value, Value, Value, Value) {
        let (receipt, schema) = decision_receipt_fixture().expect("valid decision receipt fixture");
        let profile = embedded_profile().expect("valid expression profile");
        let brief: Value = serde_json::from_str(HOME_BRIEF_FIXTURE).expect("valid source brief");
        let candidate_set: Value =
            serde_json::from_str(HOME_CANDIDATE_SET_FIXTURE).expect("valid source candidate set");
        (receipt, schema, profile, brief, candidate_set)
    }

    #[test]
    fn accepts_exact_source_bound_decision_receipt() {
        let (receipt, schema, profile, brief, candidate_set) = decision_fixture();
        assert!(
            validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn an_empty_hard_gate_set_never_becomes_preference_eligible() {
        assert_eq!(
            candidate_technical_disposition(&[]),
            "CONDITIONAL_CONCEPT_ONLY"
        );
    }

    #[test]
    fn unsupported_hard_gate_state_never_becomes_preference_eligible() {
        let gates = vec![json!({"gate_id":"C_ACCESS_01","state":"MAYBE"})];
        assert_eq!(
            candidate_technical_disposition(&gates),
            "CONDITIONAL_CONCEPT_ONLY"
        );
    }

    #[test]
    fn rejects_missing_required_source_hard_gate() {
        let (receipt, schema, profile, brief, mut candidate_set) = decision_fixture();
        candidate_set["candidates"][0]["hard_gate_results"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_empty_source_hard_gate_set() {
        let (receipt, schema, profile, brief, mut candidate_set) = decision_fixture();
        candidate_set["candidates"][0]["hard_gate_results"] = json!([]);
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn unknown_hard_gate_never_becomes_preference_eligible() {
        let gates = vec![json!({"gate_id":"C_ACCESS_01","state":"UNKNOWN"})];
        assert_eq!(
            candidate_technical_disposition(&gates),
            "CONDITIONAL_CONCEPT_ONLY"
        );
    }

    #[test]
    fn a_known_hard_gate_failure_blocks_candidate() {
        let gates = vec![
            json!({"gate_id":"C_ACCESS_01","state":"PASS"}),
            json!({"gate_id":"STRUCTURE_FIRE","state":"FAIL"}),
        ];
        assert_eq!(candidate_technical_disposition(&gates), "BLOCKED");
    }

    #[test]
    fn all_pass_gates_only_allow_resident_preference_comparison() {
        let gates = vec![
            json!({"gate_id":"C_ACCESS_01","state":"PASS"}),
            json!({"gate_id":"STRUCTURE_FIRE","state":"PASS"}),
        ];
        assert_eq!(
            candidate_technical_disposition(&gates),
            "ELIGIBLE_FOR_RESIDENT_PREFERENCE_COMPARISON"
        );
    }

    #[test]
    fn rejects_unknown_gate_promoted_to_pass() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["candidate_receipts"][0]["hard_gate_results"][0]["state"] = json!("PASS");
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_automatic_candidate_selection() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["comparison_policy"]["automatic_selection"] = json!(true);
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn preference_graph_detects_directed_cycles() {
        let pairs = vec![
            json!({"left_candidate_id":"A","right_candidate_id":"B","decision":"PREFER_LEFT"}),
            json!({"left_candidate_id":"B","right_candidate_id":"C","decision":"PREFER_LEFT"}),
            json!({"left_candidate_id":"C","right_candidate_id":"A","decision":"PREFER_LEFT"}),
        ];
        assert!(preference_graph_has_cycle(&pairs));
    }

    #[test]
    fn preference_graph_allows_incomparable_and_equivalent_pairs() {
        let pairs = vec![
            json!({"left_candidate_id":"A","right_candidate_id":"B","decision":"INCOMPARABLE"}),
            json!({"left_candidate_id":"B","right_candidate_id":"C","decision":"EQUIVALENT"}),
        ];
        assert!(!preference_graph_has_cycle(&pairs));
    }

    #[test]
    fn accepts_explicit_resident_preferences_with_pending_provenance() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["resident_choice"]["state"] = json!("PREFERENCES_RECORDED");
        receipt["resident_choice"]["decision_event"] = json!({
            "participant_ref":"participant:synthetic-01",
            "event_ref":"urn:mycelix:synthetic:decision-001",
            "event_content_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "design_revision":"HFC-SYNTH-REV-0",
            "decision_scope":"CONCEPT_DIRECTION_ONLY",
            "provenance_status":"PENDING_MYCELIX_5149_VERIFICATION"
        });
        receipt["resident_choice"]["pairwise_preferences"][0]["decision"] = json!("PREFER_LEFT");
        receipt["resident_choice"]["pairwise_preferences"][0]["basis"] =
            json!("Synthetic positive control only; not a real resident choice.");
        assert!(
            validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_recorded_preference_without_provenance_event() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["resident_choice"]["state"] = json!("PREFERENCES_RECORDED");
        receipt["resident_choice"]["pairwise_preferences"][0]["decision"] = json!("PREFER_LEFT");
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_preference_event_from_a_stale_design_revision() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["resident_choice"]["state"] = json!("PREFERENCES_RECORDED");
        receipt["resident_choice"]["decision_event"] = json!({
            "participant_ref":"participant:synthetic-01",
            "event_ref":"urn:mycelix:synthetic:decision-003",
            "event_content_sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "design_revision":"STALE-REVISION",
            "decision_scope":"CONCEPT_DIRECTION_ONLY",
            "provenance_status":"PENDING_MYCELIX_5149_VERIFICATION"
        });
        receipt["resident_choice"]["pairwise_preferences"][0]["decision"] = json!("PREFER_LEFT");
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_direct_identity_in_participant_reference() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["resident_choice"]["state"] = json!("PREFERENCES_RECORDED");
        receipt["resident_choice"]["decision_event"] = json!({
            "participant_ref":"Tristan",
            "event_ref":"urn:mycelix:synthetic:decision-004",
            "event_content_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "design_revision":"HFC-SYNTH-REV-0",
            "decision_scope":"CONCEPT_DIRECTION_ONLY",
            "provenance_status":"PENDING_MYCELIX_5149_VERIFICATION"
        });
        receipt["resident_choice"]["pairwise_preferences"][0]["decision"] = json!("PREFER_LEFT");
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_concept_selection_for_unknown_candidate() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["resident_choice"]["state"] = json!("CONCEPT_DIRECTION_SELECTED");
        receipt["resident_choice"]["selected_candidate_id"] = json!("NOT-IN-SOURCE");
        receipt["resident_choice"]["decision_event"] = json!({
            "participant_ref":"participant:synthetic-01",
            "event_ref":"urn:mycelix:synthetic:decision-002",
            "event_content_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "design_revision":"HFC-SYNTH-REV-0",
            "decision_scope":"CONCEPT_DIRECTION_ONLY",
            "provenance_status":"PENDING_MYCELIX_5149_VERIFICATION"
        });
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_invented_resident_choice() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["resident_choice"]["selected_candidate_id"] = json!("SYNTH-HV-A");
        receipt["resident_choice"]["state"] = json!("CHOICE_RECORDED");
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_candidate_set_digest_drift() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["source_binding"]["candidate_set_content_sha"] =
            json!("0000000000000000000000000000000000000000");
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn rejects_missing_source_gate() {
        let (mut receipt, schema, profile, brief, candidate_set) = decision_fixture();
        receipt["candidate_receipts"][0]["hard_gate_results"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(
            !validate_decision_receipt(&receipt, &schema, &profile, &brief, &candidate_set)
                .is_empty()
        );
    }

    #[test]
    fn accepts_synthetic_highveld_profile() {
        let (profile, schema) = fixture();
        assert!(validate_profile(&profile, &schema).is_ok());
    }

    #[test]
    fn rejects_source_candidate_set_digest_drift() {
        let (mut profile, schema) = fixture();
        profile["linked_candidate_set_content_sha"] =
            json!("0000000000000000000000000000000000000000");
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_source_candidate_revision_drift() {
        let (mut profile, schema) = fixture();
        profile["linked_design_revision_ref"] = json!("STALE-REVISION");
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_source_brief_digest_drift() {
        let (mut profile, schema) = fixture();
        profile["linked_brief_content_sha"] = json!("0000000000000000000000000000000000000000");
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_linked_constraint_loss() {
        let (mut profile, schema) = fixture();
        profile["linked_brief_constraint_ids"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(validate_profile(&profile, &schema).is_err());
    }

    #[test]
    fn rejects_catalog_digest_drift() {
        let (mut profile, schema) = fixture();
        profile["style_preferences"]["catalog_content_sha"] =
            json!("0000000000000000000000000000000000000000");
        assert!(validate_profile(&profile, &schema).is_err());
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
        let (manifest, schema) = revision_impact_fixture().expect("valid revision impact fixture");
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
        manifest["records"][0]["potential_review_domains"] =
            json!(["STRUCTURE_FIRE_MOISTURE_ACOUSTICS"]);
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
    fn rejects_unknown_source_case_artifact_binding() {
        let (mut manifest, schema) = benchmark_fixture().expect("valid benchmark fixture");
        manifest["cases"][0]["source_artifact_refs"][0]["blob_sha"] =
            json!("0000000000000000000000000000000000000000");
        assert!(!validate_benchmark_manifest(&manifest, &schema).is_empty());
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
