// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::{env, fs, process};

const MANIFEST: &str = include_str!(
    "../../docs/research/home-assembly-demand/HOME_ASSEMBLY_SUBSTITUTION_CASES_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/home-assembly-demand/HOME_ASSEMBLY_SUBSTITUTION_CASES_V0.1.schema.json"
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

fn sorted_strings(values: &BTreeSet<String>) -> Vec<String> {
    values.iter().cloned().collect()
}

fn validate_case(case: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let case_id = text_at(case, "case_id").unwrap_or("<missing-case-id>");
    let original_product = text_at(case, "original_product_ref").unwrap_or_default();
    let replacement_product = text_at(case, "replacement_product_ref").unwrap_or_default();

    if original_product.is_empty() || replacement_product.is_empty() {
        errors.push(format!("{case_id}: original and replacement product references are required"));
    }

    let catalog_rows = array_at(case, "dependency_catalog").cloned().unwrap_or_default();
    let mut catalog = BTreeSet::new();
    for dependency in &catalog_rows {
        match dependency.as_str() {
            Some(id) if !id.is_empty() && catalog.insert(id.to_owned()) => {}
            Some(id) => errors.push(format!("{case_id}: empty or duplicate dependency ID {id}")),
            None => errors.push(format!("{case_id}: dependency catalog contains a non-string value")),
        }
    }
    if catalog.is_empty() {
        errors.push(format!("{case_id}: dependency catalog cannot be empty"));
    }

    let change_rows = array_at(case, "changes").cloned().unwrap_or_default();
    let mut changed = BTreeSet::new();
    let mut seen_changes = BTreeSet::new();
    let mut product_identity_change: Option<(String, String)> = None;

    for row in &change_rows {
        let dependency_id = text_at(row, "dependency_id").unwrap_or_default();
        let before = text_at(row, "before_state").unwrap_or_default();
        let after = text_at(row, "after_state").unwrap_or_default();
        let treatment = text_at(row, "treatment").unwrap_or_default();

        if dependency_id.is_empty() || !catalog.contains(dependency_id) {
            errors.push(format!(
                "{case_id}: changed dependency {dependency_id:?} is absent from the dependency catalog"
            ));
            continue;
        }
        if !seen_changes.insert(dependency_id.to_owned()) {
            errors.push(format!("{case_id}: dependency {dependency_id} is changed more than once"));
        }
        if before.is_empty() || after.is_empty() || before == after {
            errors.push(format!("{case_id}: dependency {dependency_id} has an empty or unchanged state pair"));
        }

        match treatment {
            "CHANGED" => {
                if after == "UNKNOWN" {
                    errors.push(format!(
                        "{case_id}: dependency {dependency_id} with UNKNOWN after-state must use UNKNOWN treatment"
                    ));
                }
                changed.insert(dependency_id.to_owned());
            }
            "UNKNOWN" => {
                if after != "UNKNOWN" {
                    errors.push(format!(
                        "{case_id}: dependency {dependency_id} UNKNOWN treatment must have an UNKNOWN after-state"
                    ));
                }
                changed.insert(dependency_id.to_owned());
            }
            _ => errors.push(format!("{case_id}: dependency {dependency_id} has an unsupported treatment")),
        }

        if dependency_id == "PRODUCT_IDENTITY" {
            product_identity_change = Some((before.to_owned(), after.to_owned()));
        }
    }

    if original_product != replacement_product {
        match product_identity_change {
            Some((before, after))
                if before == original_product && after == replacement_product => {}
            Some(_) => errors.push(format!(
                "{case_id}: PRODUCT_IDENTITY change does not bind the exact original/replacement references"
            )),
            None => errors.push(format!(
                "{case_id}: product reference changed without a PRODUCT_IDENTITY dependency change"
            )),
        }
    } else if product_identity_change.is_some() {
        errors.push(format!(
            "{case_id}: PRODUCT_IDENTITY was changed although product references are identical"
        ));
    }

    let claim_rows = array_at(case, "claims").cloned().unwrap_or_default();
    let mut claim_ids = BTreeSet::new();
    let mut derived_invalidated = BTreeSet::new();
    let mut derived_preserved = BTreeSet::new();

    for claim in &claim_rows {
        let claim_id = text_at(claim, "claim_id").unwrap_or_default();
        if claim_id.is_empty() || !claim_ids.insert(claim_id.to_owned()) {
            errors.push(format!("{case_id}: claim identity is missing or duplicated"));
            continue;
        }
        if !matches!(
            text_at(claim, "state_before"),
            Some("QUALIFIED_IN_ORIGINAL_SCOPE" | "UNKNOWN_IN_ORIGINAL_SCOPE")
        ) {
            errors.push(format!("{case_id}: claim {claim_id} has an unsupported prior state"));
        }

        let dependencies = array_at(claim, "dependency_ids").cloned().unwrap_or_default();
        if dependencies.is_empty() {
            errors.push(format!("{case_id}: claim {claim_id} has no declared dependencies"));
            continue;
        }
        let mut claim_dependencies = BTreeSet::new();
        let mut intersects = false;
        for dependency in &dependencies {
            match dependency.as_str() {
                Some(id) if catalog.contains(id) => {
                    if !claim_dependencies.insert(id.to_owned()) {
                        errors.push(format!(
                            "{case_id}: claim {claim_id} repeats dependency {id}"
                        ));
                    }
                    intersects |= changed.contains(id);
                }
                Some(id) => errors.push(format!(
                    "{case_id}: claim {claim_id} references undeclared dependency {id}"
                )),
                None => errors.push(format!(
                    "{case_id}: claim {claim_id} contains a non-string dependency"
                )),
            }
        }
        if intersects {
            derived_invalidated.insert(claim_id.to_owned());
        } else {
            derived_preserved.insert(claim_id.to_owned());
        }
    }

    let expected_invalidated = array_at(case, "expected_invalidated_claim_ids")
        .cloned()
        .unwrap_or_default();
    let expected_preserved = array_at(case, "expected_preserved_claim_ids")
        .cloned()
        .unwrap_or_default();
    let mut expected_invalidated_set = BTreeSet::new();
    let mut expected_preserved_set = BTreeSet::new();

    for row in &expected_invalidated {
        match row.as_str() {
            Some(id) if expected_invalidated_set.insert(id.to_owned()) => {}
            Some(id) => errors.push(format!("{case_id}: duplicate expected invalidated claim {id}")),
            None => errors.push(format!("{case_id}: expected invalidated claim ID is not a string")),
        }
    }
    for row in &expected_preserved {
        match row.as_str() {
            Some(id) if expected_preserved_set.insert(id.to_owned()) => {}
            Some(id) => errors.push(format!("{case_id}: duplicate expected preserved claim {id}")),
            None => errors.push(format!("{case_id}: expected preserved claim ID is not a string")),
        }
    }

    if !expected_invalidated_set.is_disjoint(&expected_preserved_set) {
        errors.push(format!("{case_id}: a claim is both invalidated and preserved"));
    }
    let expected_union: BTreeSet<String> = expected_invalidated_set
        .union(&expected_preserved_set)
        .cloned()
        .collect();
    if expected_union != claim_ids {
        errors.push(format!(
            "{case_id}: expected invalidated/preserved sets do not partition all declared claims"
        ));
    }
    if expected_invalidated_set != derived_invalidated {
        errors.push(format!(
            "{case_id}: invalidated known-answer mismatch; expected {:?}, derived {:?}",
            sorted_strings(&expected_invalidated_set),
            sorted_strings(&derived_invalidated)
        ));
    }
    if expected_preserved_set != derived_preserved {
        errors.push(format!(
            "{case_id}: preserved known-answer mismatch; expected {:?}, derived {:?}",
            sorted_strings(&expected_preserved_set),
            sorted_strings(&derived_preserved)
        ));
    }

    let disposition = if changed.is_empty() {
        "NO_DEPENDENCY_CHANGE"
    } else {
        "REQUALIFICATION_REQUIRED"
    };
    if text_at(case, "expected_disposition") != Some(disposition) {
        errors.push(format!("{case_id}: expected disposition does not match the changed dependency set"));
    }

    if errors.is_empty() {
        Ok(json!({
            "case_id":case_id,
            "disposition":disposition,
            "changed_dependency_ids":sorted_strings(&changed),
            "invalidated_claim_ids":sorted_strings(&derived_invalidated),
            "preserved_claim_ids":sorted_strings(&derived_preserved),
            "claims_checked":claim_ids.len(),
            "product_equivalence_decision":"NOT_MADE",
            "approval_decision":"NOT_MADE"
        }))
    } else {
        Err(errors)
    }
}

fn validate(manifest: &Value, schema: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("substitution case schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(schema, "type") != Some("object") {
        errors.push("substitution case schema root must be an object".to_owned());
    }
    if text_at(manifest, "schema_version") != Some("0.1.0")
        || text_at(manifest, "study_id") != Some("HOME-ASSEMBLY-SUBSTITUTION-001")
    {
        errors.push("substitution manifest identity or version is invalid".to_owned());
    }
    if text_at(manifest, "maturity") != Some("SYNTHETIC_KNOWN_ANSWER")
        || text_at(manifest, "execution_status") != Some("NO_PRODUCT_EQUIVALENCE_DECISION")
        || text_at(manifest, "algorithm")
            != Some("DEPENDENCY_INTERSECTION_INVALIDATION_V0.1")
    {
        errors.push("substitution manifest was promoted above dependency-propagation maturity".to_owned());
    }

    let cases = array_at(manifest, "cases").cloned().unwrap_or_default();
    if cases.len() != 18 {
        errors.push(format!("v0.1 frozen substitution corpus must contain exactly 18 cases; found {}", cases.len()));
    }

    let mut case_ids = BTreeSet::new();
    let mut reports = Vec::new();
    let mut total_claims = 0usize;
    let mut total_invalidated = 0usize;
    let mut total_preserved = 0usize;
    let mut no_change_cases = 0usize;
    let mut requalification_cases = 0usize;

    for case in &cases {
        let case_id = text_at(case, "case_id").unwrap_or("<missing-case-id>");
        if case_id.is_empty() || !case_ids.insert(case_id.to_owned()) {
            errors.push("substitution case ID is missing or duplicated".to_owned());
            continue;
        }
        match validate_case(case) {
            Ok(report) => {
                total_claims += report["claims_checked"].as_u64().unwrap_or(0) as usize;
                total_invalidated += report["invalidated_claim_ids"].as_array().map_or(0, Vec::len);
                total_preserved += report["preserved_claim_ids"].as_array().map_or(0, Vec::len);
                match text_at(&report, "disposition") {
                    Some("NO_DEPENDENCY_CHANGE") => no_change_cases += 1,
                    Some("REQUALIFICATION_REQUIRED") => requalification_cases += 1,
                    _ => errors.push(format!("{case_id}: unsupported result disposition")),
                }
                reports.push(report);
            }
            Err(case_errors) => errors.extend(case_errors),
        }
    }

    if errors.is_empty() {
        Ok(json!({
            "audit_state":"HOME_ASSEMBLY_SUBSTITUTION_PASS",
            "study_id":"HOME-ASSEMBLY-SUBSTITUTION-001",
            "case_count":reports.len(),
            "claim_count":total_claims,
            "invalidated_claim_count":total_invalidated,
            "preserved_claim_count":total_preserved,
            "no_dependency_change_cases":no_change_cases,
            "requalification_required_cases":requalification_cases,
            "missed_invalidation_count":0,
            "extra_invalidation_count":0,
            "product_equivalence_decisions":0,
            "approval_decisions":0,
            "claim_ceiling":"SYNTHETIC_DEPENDENCY_PROPAGATION_ONLY",
            "cases":reports
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
                    "usage":"home-assembly-substitution-qualifier [--input PATH] [--self-test]",
                    "default":"embedded 18-case synthetic substitution corpus",
                    "claim_ceiling":"dependency propagation only; no product equivalence or approval decision"
                }));
            }
            unknown => return Err(format!("unknown argument {unknown}")),
        }
        index += 1;
    }

    let manifest = match input_path {
        Some(path) => {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("could not read substitution input {path}: {error}"))?;
            parse_json(&content, "substitution input")?
        }
        None => parse_json(MANIFEST, "substitution manifest")?,
    };
    let schema = parse_json(SCHEMA, "substitution schema")?;
    validate(&manifest, &schema)
        .map_err(|errors| format!("HOME_ASSEMBLY_SUBSTITUTION_FAIL: {}", errors.join("; ")))
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            if text_at(&report, "audit_state") == Some("HOME_ASSEMBLY_SUBSTITUTION_PASS") {
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
            parse_json(MANIFEST, "manifest").expect("manifest JSON"),
            parse_json(SCHEMA, "schema").expect("schema JSON"),
        )
    }

    #[test]
    fn all_eighteen_known_answer_cases_match() {
        let (manifest, schema) = fixture();
        let report = validate(&manifest, &schema).expect("substitution corpus must qualify");
        assert_eq!(report["case_count"], 18);
        assert_eq!(report["claim_count"], 66);
        assert_eq!(report["invalidated_claim_count"], 37);
        assert_eq!(report["preserved_claim_count"], 29);
        assert_eq!(report["missed_invalidation_count"], 0);
        assert_eq!(report["extra_invalidation_count"], 0);
        assert_eq!(report["product_equivalence_decisions"], 0);
    }

    #[test]
    fn missed_dependent_claim_invalidation_is_rejected() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][1]["expected_invalidated_claim_ids"]
            .as_array_mut()
            .unwrap()
            .retain(|claim| claim != "CLAIM-FIRE-TEST");
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn unrelated_claim_invalidation_is_rejected() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][2]["expected_invalidated_claim_ids"]
            .as_array_mut()
            .unwrap()
            .push(json!("CLAIM-THERMAL"));
        manifest["cases"][2]["expected_preserved_claim_ids"]
            .as_array_mut()
            .unwrap()
            .retain(|claim| claim != "CLAIM-THERMAL");
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn product_reference_change_requires_identity_dependency() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][1]["changes"] = json!([]);
        manifest["cases"][1]["expected_invalidated_claim_ids"] = json!([]);
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn product_identity_change_must_bind_exact_references() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][1]["changes"][0]["before_state"] = json!("OTHER-SKU");
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn undeclared_claim_dependency_is_rejected() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][2]["claims"][0]["dependency_ids"] = json!(["UNDECLARED"]);
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn duplicate_changed_dependency_is_rejected() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][3]["changes"].as_array_mut().unwrap().push(
            manifest["cases"][3]["changes"][0].clone()
        );
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn unknown_treatment_requires_unknown_after_state() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][9]["changes"][0]["after_state"] = json!("AVAILABLE");
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn changed_treatment_cannot_hide_unknown_after_state() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][2]["changes"][0]["after_state"] = json!("UNKNOWN");
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn no_op_positive_control_must_preserve_every_claim() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][0]["expected_preserved_claim_ids"]
            .as_array_mut()
            .unwrap()
            .retain(|claim| claim != "CLAIM-LAYOUT");
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn duplicate_case_identity_is_rejected() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][1]["case_id"] = manifest["cases"][0]["case_id"].clone();
        assert!(validate(&manifest, &schema).is_err());
    }

    #[test]
    fn invalidated_and_preserved_sets_must_partition_claims() {
        let (mut manifest, schema) = fixture();
        manifest["cases"][2]["expected_preserved_claim_ids"] = json!([]);
        assert!(validate(&manifest, &schema).is_err());
    }
}
