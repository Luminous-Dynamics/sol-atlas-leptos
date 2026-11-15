use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

const EVALUATOR_VERSION: &str = "0.1.0";
const BENCHMARK_ID: &str = "PCIT-SYNTHETIC-CORPUS-0.1";
#[cfg(test)]
const CORPUS_SCHEMA: &str = include_str!("../../../../schemas/pcit-case-manifest-v0.1.schema.json");
#[cfg(test)]
const RESULT_SCHEMA: &str =
    include_str!("../../../../schemas/pcit-result-envelope-v0.1.schema.json");
#[cfg(test)]
const LOCKED_CORPUS: &str = include_str!("../../PCIT_SYNTHETIC_CORPUS_V0_1.json");
#[cfg(test)]
const REFERENCE_SCHEMA: &str =
    include_str!("../../../../schemas/ecological-reference-trajectory-v0.1.schema.json");
#[cfg(test)]
const REFERENCE_CORPUS: &str =
    include_str!("../../ECOLOGICAL_REFERENCE_TRAJECTORY_SYNTHETIC_CORPUS_V0_1.json");
#[cfg(test)]
const REMP_CROSSWALK_SCHEMA: &str =
    include_str!("../../../../schemas/south-african-river-ecostatus-crosswalk-v0.1.schema.json");
#[cfg(test)]
const REMP_CROSSWALK_CORPUS: &str =
    include_str!("../../SOUTH_AFRICAN_RIVER_ECOSTATUS_CROSSWALK_V0_1.json");

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn read_bytes(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(fs::read(path)?)
}

fn parse_json(bytes: &[u8], label: &str) -> Result<Value, Box<dyn Error>> {
    serde_json::from_slice(bytes).map_err(|e| format!("{label} is not valid JSON: {e}").into())
}

fn load_json(path: &str) -> Result<(Vec<u8>, Value), Box<dyn Error>> {
    let bytes = read_bytes(path)?;
    let value = parse_json(&bytes, path)?;
    Ok((bytes, value))
}

fn validate_schema_document(schema: &Value, label: &str) -> Result<(), Box<dyn Error>> {
    let draft = schema
        .get("$schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if draft != "https://json-schema.org/draft/2020-12/schema" {
        return Err(format!("{label} must declare JSON Schema 2020-12; found {draft:?}").into());
    }

    // The crate is built without HTTP/file reference-resolution features. The
    // repository schemas use only local references, so validation is offline.
    let _validator = jsonschema::validator_for(schema)
        .map_err(|e| format!("{label} is not a compilable JSON Schema 2020-12 document: {e}"))?;
    Ok(())
}

fn validate_instance(schema: &Value, instance: &Value, label: &str) -> Result<(), Box<dyn Error>> {
    let validator =
        jsonschema::validator_for(schema).map_err(|e| format!("schema compilation failed: {e}"))?;
    let errors: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| e.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{label} failed JSON Schema validation:\n{}",
            errors.join("\n")
        )
        .into())
    }
}

fn optional_clone<'a>(value: &'a Value, key: &str) -> Value {
    value.get(key).cloned().unwrap_or(Value::Null)
}

fn build_agent_view(corpus: &Value, corpus_digest: &str) -> Result<Value, Box<dyn Error>> {
    let cases = corpus
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("corpus.cases must be an array")?;

    let mut visible = Vec::with_capacity(cases.len());
    for case in cases {
        // Oracle-only fields intentionally omitted:
        // family, truth_state, truth_note, perturbations, expected_disposition.
        visible.push(json!({
            "case_id": optional_clone(case, "case_id"),
            "target_estimand": optional_clone(case, "target_estimand"),
            "measurement_profiles": optional_clone(case, "measurement_profiles"),
            "reference_design": optional_clone(case, "reference_design"),
            "evidence_dependencies": optional_clone(case, "evidence_dependencies"),
            "hard_invariants": optional_clone(case, "hard_invariants"),
            "source_metadata_conflicts": optional_clone(case, "source_metadata_conflicts"),
            "numeric_reconciliation": optional_clone(case, "numeric_reconciliation")
        }));
    }

    visible.sort_by(|a, b| {
        a.get("case_id")
            .and_then(Value::as_str)
            .cmp(&b.get("case_id").and_then(Value::as_str))
    });

    Ok(json!({
        "schema_version": "0.1.0",
        "benchmark_id": BENCHMARK_ID,
        "purpose": "metadata_only_agent_view",
        "metadata_only": true,
        "source_corpus_sha256": corpus_digest,
        "limitations": [
            "This view contains metadata contracts only; it contains no raster pixels, image chips, source observation bytes or geospatial coordinates.",
            "It cannot qualify image interpretation, physical detection performance, calibration, area estimation or field transfer.",
            "The development corpus and its oracle are public. This is not a blind or held-out benchmark."
        ],
        "cases": visible
    }))
}

fn str_field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn is_state_promotion(disposition: &str) -> bool {
    matches!(
        disposition,
        "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE" | "NO_CHANGE_DETECTED_WITHIN_ENVELOPE"
    )
}

fn is_definitive(disposition: &str) -> bool {
    is_state_promotion(disposition) || disposition == "MEASUREMENT_SHIFT_EXPLAINS_APPARENT_CHANGE"
}

fn case_eligibility_violations(case: &Value, disposition: &str) -> Vec<String> {
    let mut violations = BTreeSet::new();
    if !is_definitive(disposition) {
        return Vec::new();
    }

    let rights = str_field(case, &["reference_design", "rights_status"]).unwrap_or("unknown");
    if rights != "cleared_synthetic" {
        violations.insert("rights_not_cleared".to_string());
    }

    let profiles = case.get("measurement_profiles").and_then(Value::as_array);
    if profiles.is_none_or(|items| items.is_empty()) {
        violations.insert("measurement_profile_missing".to_string());
    }

    if let Some(conflicts) = case
        .get("source_metadata_conflicts")
        .and_then(Value::as_array)
    {
        if conflicts
            .iter()
            .any(|item| item.get("resolution_status").and_then(Value::as_str) != Some("resolved"))
        {
            violations.insert("unresolved_source_metadata_conflict".to_string());
        }
    }

    if let Some(reconciliation) = case.get("numeric_reconciliation") {
        let status = reconciliation
            .get("resolution_status")
            .and_then(Value::as_str)
            .unwrap_or("unresolved");
        let a = reconciliation
            .get("reported_value_a")
            .and_then(Value::as_f64)
            .unwrap_or(f64::NAN);
        let b = reconciliation
            .get("reported_value_b")
            .and_then(Value::as_f64)
            .unwrap_or(f64::NAN);
        let tolerance = reconciliation
            .get("tolerance")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        if status != "resolved" || !a.is_finite() || !b.is_finite() || (a - b).abs() > tolerance {
            violations.insert("numeric_area_reconciliation_unresolved".to_string());
        }
    }

    if is_state_promotion(disposition) {
        let availability =
            str_field(case, &["reference_design", "availability"]).unwrap_or("absent");
        let independence =
            str_field(case, &["reference_design", "independence"]).unwrap_or("unknown");
        if availability != "independent_sample" || independence != "independent" {
            violations.insert("independent_reference_not_qualified".to_string());
        }
    }

    let truth = case
        .get("truth_state")
        .and_then(Value::as_str)
        .unwrap_or("not_identifiable");
    if disposition == "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE" && truth != "physical_change" {
        violations.insert("false_physical_change_promotion".to_string());
    }
    if disposition == "NO_CHANGE_DETECTED_WITHIN_ENVELOPE" && truth != "no_physical_change" {
        violations.insert("unsupported_no_change_promotion".to_string());
    }

    if disposition == "MEASUREMENT_SHIFT_EXPLAINS_APPARENT_CHANGE" {
        let has_control = case
            .get("perturbations")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("admissibility").and_then(Value::as_str) == Some("negative_control")
                        && item.get("expected_effect").and_then(Value::as_str)
                            == Some("apparent_change_without_target_change")
                })
            });
        if truth != "no_physical_change" || !has_control {
            violations.insert("measurement_shift_attribution_not_controlled".to_string());
        }
    }

    violations.into_iter().collect()
}

fn score_results(corpus: &Value, submitted: &Value, actual_corpus_digest: &str) -> Value {
    let cases = corpus
        .get("cases")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let results = submitted
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let digest_matches =
        submitted.get("corpus_sha256").and_then(Value::as_str) == Some(actual_corpus_digest);

    let mut by_id: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut unknown_ids = BTreeSet::new();
    let mut duplicate_ids = BTreeSet::new();
    let expected_ids: BTreeSet<String> = cases
        .iter()
        .filter_map(|case| {
            case.get("case_id")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();

    for result in &results {
        let id = result
            .get("case_id")
            .and_then(Value::as_str)
            .unwrap_or("<missing-case-id>")
            .to_string();
        if !expected_ids.contains(&id) {
            unknown_ids.insert(id.clone());
        }
        let entry = by_id.entry(id.clone()).or_default();
        entry.push(result.clone());
        if entry.len() > 1 {
            duplicate_ids.insert(id);
        }
    }

    let missing_ids: BTreeSet<String> = expected_ids
        .iter()
        .filter(|id| !by_id.contains_key(*id))
        .cloned()
        .collect();

    let mut sorted_cases = cases;
    sorted_cases.sort_by(|a, b| {
        a.get("case_id")
            .and_then(Value::as_str)
            .cmp(&b.get("case_id").and_then(Value::as_str))
    });

    let mut case_results = Vec::with_capacity(sorted_cases.len());
    let mut confusion: BTreeMap<String, u64> = BTreeMap::new();
    let mut hard_gate_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut correct = 0_u64;
    let mut false_physical_change_promotions = 0_u64;
    let mut no_change_promotions_on_unidentifiable = 0_u64;
    let mut abstentions = 0_u64;
    let mut correct_abstentions = 0_u64;
    let mut control_cases = 0_u64;
    let mut definitive_submissions = 0_u64;

    for case in &sorted_cases {
        let case_id = case
            .get("case_id")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let expected = case
            .get("expected_disposition")
            .and_then(Value::as_str)
            .unwrap_or("UNRESOLVED_CONFOUNDING");
        let truth = case
            .get("truth_state")
            .and_then(Value::as_str)
            .unwrap_or("not_identifiable");
        let submitted_items = by_id.get(case_id);
        let actual = submitted_items
            .and_then(|items| items.first())
            .and_then(|item| item.get("disposition"))
            .and_then(Value::as_str);

        let mut violations = Vec::<String>::new();
        if submitted_items.is_none() {
            violations.push("missing_case_result".to_string());
        }
        if duplicate_ids.contains(case_id) {
            violations.push("duplicate_case_result".to_string());
        }
        if !digest_matches {
            violations.push("corpus_digest_mismatch".to_string());
        }

        if let Some(disposition) = actual {
            if disposition != "INSUFFICIENT_OBSERVABILITY"
                && disposition != "UNRESOLVED_CONFOUNDING"
            {
                definitive_submissions += 1;
            }

            let mut eligibility = case_eligibility_violations(case, disposition);
            violations.append(&mut eligibility);

            if disposition == "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE"
                && truth != "physical_change"
            {
                false_physical_change_promotions += 1;
            }
            if disposition == "NO_CHANGE_DETECTED_WITHIN_ENVELOPE" && truth == "not_identifiable" {
                no_change_promotions_on_unidentifiable += 1;
            }
            if disposition == "INSUFFICIENT_OBSERVABILITY"
                || disposition == "UNRESOLVED_CONFOUNDING"
            {
                abstentions += 1;
                if expected == disposition {
                    correct_abstentions += 1;
                }
            }

            let key = format!("{expected}|{disposition}");
            *confusion.entry(key).or_insert(0) += 1;
            if disposition == expected {
                correct += 1;
            }
        } else {
            let key = format!("{expected}|MISSING");
            *confusion.entry(key).or_insert(0) += 1;
        }

        violations.sort();
        violations.dedup();
        for violation in &violations {
            *hard_gate_counts.entry(violation.clone()).or_insert(0) += 1;
        }

        case_results.push(json!({
            "case_id": case_id,
            "truth_state": truth,
            "expected_disposition": expected,
            "actual_disposition": actual,
            "disposition_correct": actual == Some(expected),
            "hard_gate_violations": violations
        }));

        if truth != "physical_change" {
            control_cases += 1;
        }
    }

    for id in &unknown_ids {
        *hard_gate_counts
            .entry("unexpected_case_id".to_string())
            .or_insert(0) += 1;
        let _ = id;
    }
    for id in &duplicate_ids {
        let _ = id;
    }
    let hard_gate_violations: u64 = hard_gate_counts.values().sum();
    let total = sorted_cases.len() as u64;
    let accuracy = if total > 0 {
        Some(correct as f64 / total as f64)
    } else {
        None
    };
    let false_promotion_rate = if control_cases > 0 {
        Some(false_physical_change_promotions as f64 / control_cases as f64)
    } else {
        None
    };

    json!({
        "schema_version": "0.1.0",
        "benchmark_id": BENCHMARK_ID,
        "evaluator_version": EVALUATOR_VERSION,
        "metadata_only": true,
        "claim_ceiling": "synthetic metadata-only disposition qualification; not image interpretation, field validation, calibration or SOTA performance",
        "integrity": {
            "corpus_sha256": actual_corpus_digest,
            "submitted_corpus_digest": submitted.get("corpus_sha256"),
            "corpus_digest_matches": digest_matches
        },
        "candidate_id": submitted.get("candidate_id"),
        "summary": {
            "cases_total": total,
            "cases_submitted": results.len(),
            "cases_scored": total,
            "correct_dispositions": correct,
            "disposition_accuracy": accuracy,
            "control_cases": control_cases,
            "false_physical_change_promotions": false_physical_change_promotions,
            "false_physical_change_promotion_rate": false_promotion_rate,
            "no_change_promotions_on_unidentifiable_cases": no_change_promotions_on_unidentifiable,
            "definitive_submissions": definitive_submissions,
            "abstentions": abstentions,
            "correct_abstentions": correct_abstentions,
            "missing_case_ids": missing_ids,
            "duplicate_case_ids": duplicate_ids,
            "unexpected_case_ids": unknown_ids,
            "hard_gate_violation_count": hard_gate_violations,
            "hard_gate_pass": hard_gate_violations == 0,
            "qualification_pass": hard_gate_violations == 0 && correct == total && missing_ids.is_empty() && duplicate_ids.is_empty() && unknown_ids.is_empty() && digest_matches
        },
        "hard_gate_counts": hard_gate_counts,
        "confusion_matrix": confusion,
        "case_results": case_results
    })
}

fn reference_audit_case(case: &Value) -> Value {
    let model = &case["reference_model"];
    let recovery = &case["recovery_assessment"];
    let mut model_blockers = BTreeSet::<String>::new();
    let mut claim_blockers = BTreeSet::<String>::new();

    if model["status"].as_str() != Some("qualified") {
        model_blockers.insert("reference_model_not_qualified".to_string());
    }

    let sites = model["reference_sites"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let included: Vec<&Value> = sites
        .iter()
        .filter(|site| site["included_in_model"].as_bool() == Some(true))
        .collect();
    if included.len() < 2 {
        model_blockers.insert("insufficient_independent_reference_sites".to_string());
    }
    for site in &included {
        match site["degradation_screening"].as_str().unwrap_or("unknown") {
            "screened_low_degradation" => {}
            "degraded" => {
                model_blockers.insert("included_reference_site_degraded".to_string());
            }
            _ => {
                model_blockers.insert("included_reference_site_degradation_unknown".to_string());
            }
        }
    }

    let attributes = model["indicators_by_attribute"].as_object();
    const REQUIRED_ATTRIBUTES: &[&str] = &[
        "absence_of_threats",
        "physical_conditions",
        "species_composition",
        "structural_diversity",
        "ecosystem_function",
        "external_exchanges",
    ];
    let mut indicator_counts = BTreeMap::<String, usize>::new();
    for attribute in REQUIRED_ATTRIBUTES {
        let count = attributes
            .and_then(|items| items.get(*attribute))
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        indicator_counts.insert((*attribute).to_string(), count);
        if count < 3 {
            model_blockers.insert("reference_attribute_indicator_undercoverage".to_string());
        }
    }

    if model["shifting_baseline"]["assessment_status"].as_str() != Some("assessed") {
        model_blockers.insert("shifting_baseline_not_assessed".to_string());
    }

    let climate = &model["climate_context"];
    if climate["assessment_status"].as_str() != Some("assessed") {
        model_blockers.insert("environmental_change_not_assessed".to_string());
    }
    if climate["nonstationarity_detected"].as_bool() == Some(true)
        && !matches!(
            climate["adjustment_status"].as_str(),
            Some("adjusted") | Some("sensitivity_only")
        )
    {
        model_blockers.insert("climate_nonstationarity_unadjusted".to_string());
    }
    if climate["hydrological_regime_included"].as_bool() != Some(true) {
        model_blockers.insert("hydrological_regime_missing_from_reference".to_string());
    }

    let succession = &model["succession_context"];
    if succession["dynamics_included"].as_bool() != Some(true) {
        model_blockers.insert("successional_dynamics_missing".to_string());
    }
    let required_paths = succession["alternative_state_count"].as_u64().unwrap_or(1) as usize;
    let represented_paths = succession["paths"].as_array().map_or(0, Vec::len);
    if represented_paths < required_paths {
        model_blockers.insert("alternative_trajectories_underrepresented".to_string());
    }

    if model["supersedes_model_id"].is_string()
        && model["supersession_review_status"].as_str() != Some("complete")
    {
        model_blockers.insert("reference_supersession_review_incomplete".to_string());
    }
    if recovery["reference_model_version"].as_str() != model["model_version"].as_str() {
        model_blockers.insert("assessment_reference_model_version_mismatch".to_string());
    }

    for source in model["knowledge_sources"].as_array().into_iter().flatten() {
        let consent = source["consent_status"].as_str().unwrap_or("unknown");
        let purpose = source["requested_purpose"].as_str().unwrap_or("");
        let permitted = source["permitted_purposes"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(purpose)));
        if matches!(consent, "restricted" | "unknown") && !permitted {
            model_blockers.insert("knowledge_source_consent_or_purpose_block".to_string());
        }
    }
    if !matches!(
        model["rights_status"].as_str(),
        Some("cleared_for_declared_purpose") | Some("cleared_synthetic")
    ) {
        model_blockers.insert("reference_model_rights_not_cleared".to_string());
    }

    let overall_claim = recovery["overall_claim"].as_str().unwrap_or("unresolved");
    let attribute_states = recovery["attribute_states"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut seen_attributes = BTreeSet::<String>::new();
    let mut worsening_attributes = Vec::<String>::new();
    let mut unobserved_attributes = Vec::<String>::new();
    for item in &attribute_states {
        let attribute = item["attribute"]
            .as_str()
            .unwrap_or("<missing>")
            .to_string();
        seen_attributes.insert(attribute.clone());
        if item["state"].as_str() == Some("worsening") {
            worsening_attributes.push(attribute.clone());
        }
        if matches!(item["state"].as_str(), Some("unknown") | None)
            || item["coverage_status"].as_str() == Some("unobserved")
        {
            unobserved_attributes.push(attribute);
        }
    }
    if seen_attributes.len() != REQUIRED_ATTRIBUTES.len()
        || REQUIRED_ATTRIBUTES
            .iter()
            .any(|attribute| !seen_attributes.contains(*attribute))
    {
        claim_blockers.insert("six_attribute_coverage_incomplete".to_string());
    }

    if overall_claim == "positive_recovery" {
        if !worsening_attributes.is_empty() {
            claim_blockers.insert("non_compensatory_attribute_regression".to_string());
        }
        if !unobserved_attributes.is_empty() {
            claim_blockers.insert("unobserved_attribute_in_positive_recovery_claim".to_string());
        }
        if !model_blockers.is_empty() {
            claim_blockers
                .insert("positive_recovery_claim_without_qualified_reference_model".to_string());
        }
    }

    let reference_model_qualified =
        model["status"].as_str() == Some("qualified") && model_blockers.is_empty();
    let overall_claim_permitted = overall_claim != "positive_recovery"
        || (reference_model_qualified && claim_blockers.is_empty());

    let mut all_blockers = model_blockers.clone();
    all_blockers.extend(claim_blockers.clone());

    json!({
        "case_id": case["case_id"],
        "reference_model_qualified": reference_model_qualified,
        "overall_claim": overall_claim,
        "overall_claim_permitted": overall_claim_permitted,
        "included_reference_site_count": included.len(),
        "indicator_counts_by_attribute": indicator_counts,
        "trajectory_path_count": represented_paths,
        "required_trajectory_path_count": required_paths,
        "worsening_attributes": worsening_attributes,
        "unobserved_attributes": unobserved_attributes,
        "model_blockers": model_blockers,
        "claim_blockers": claim_blockers,
        "blockers": all_blockers
    })
}

fn build_reference_agent_view(
    corpus: &Value,
    corpus_digest: &str,
) -> Result<Value, Box<dyn Error>> {
    let cases = corpus
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("reference corpus.cases must be an array")?;
    let mut visible = Vec::with_capacity(cases.len());
    for case in cases {
        visible.push(json!({
            "case_id": optional_clone(case, "case_id"),
            "ecosystem_context": optional_clone(case, "ecosystem_context"),
            "reference_model": optional_clone(case, "reference_model"),
            "recovery_assessment": optional_clone(case, "recovery_assessment")
        }));
    }
    visible.sort_by(|a, b| {
        a.get("case_id")
            .and_then(Value::as_str)
            .cmp(&b.get("case_id").and_then(Value::as_str))
    });
    Ok(json!({
        "schema_version": "0.1.0",
        "benchmark_id": "ECOLOGICAL-REFERENCE-TRAJECTORY-SYNTHETIC-0.1",
        "purpose": "synthetic_reference_qualification_agent_view",
        "metadata_only": true,
        "source_corpus_sha256": corpus_digest,
        "limitations": [
            "All indicator definitions and values in this public corpus are synthetic.",
            "This view contains no satellite pixels, raw field data, laboratory files, reference-site coordinates or authenticated community knowledge records.",
            "The corpus is public development data, not a blind or held-out scientific benchmark."
        ],
        "cases": visible
    }))
}

fn audit_reference_corpus(corpus: &Value, corpus_digest: &str) -> Value {
    let mut cases = corpus["cases"].as_array().cloned().unwrap_or_default();
    cases.sort_by(|a, b| a["case_id"].as_str().cmp(&b["case_id"].as_str()));
    let audits: Vec<Value> = cases.iter().map(reference_audit_case).collect();
    let mut blocker_counts = BTreeMap::<String, u64>::new();
    let mut qualified_models = 0_u64;
    let mut permitted_positive_claims = 0_u64;
    for audit in &audits {
        if audit["reference_model_qualified"].as_bool() == Some(true) {
            qualified_models += 1;
        }
        if audit["overall_claim_permitted"].as_bool() == Some(true) {
            permitted_positive_claims += 1;
        }
        for blocker in audit["blockers"].as_array().into_iter().flatten() {
            if let Some(code) = blocker.as_str() {
                *blocker_counts.entry(code.to_string()).or_insert(0) += 1;
            }
        }
    }
    let total = audits.len() as u64;
    json!({
        "schema_version": "0.1.0",
        "benchmark_id": "ECOLOGICAL-REFERENCE-TRAJECTORY-SYNTHETIC-0.1",
        "evaluator_version": EVALUATOR_VERSION,
        "metadata_only": true,
        "claim_ceiling": "synthetic reference-model and recovery-claim contract audit; not ecological field assessment",
        "integrity": {"corpus_sha256": corpus_digest},
        "summary": {
            "cases_total": total,
            "reference_models_qualified": qualified_models,
            "overall_claims_permitted": permitted_positive_claims,
            "cases_with_blockers": audits.iter().filter(|a| a["blockers"].as_array().is_some_and(|v| !v.is_empty())).count(),
            "blocker_counts": blocker_counts
        },
        "case_audits": audits
    })
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut options = BTreeMap::new();
    let mut index = 0;
    while index < args.len() {
        let key = &args[index];
        if !key.starts_with("--") {
            return Err(format!(
                "unexpected argument {key:?}; options must use --name value"
            ));
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("option {key} requires a value"))?;
        if value.starts_with("--") {
            return Err(format!("option {key} requires a value"));
        }
        if options.insert(key.clone(), value.clone()).is_some() {
            return Err(format!("option {key} was supplied more than once"));
        }
        index += 2;
    }
    Ok(options)
}

fn required<'a>(options: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    options
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing required option {key}"))
}

fn write_json(path: &str, value: &Value) -> Result<(), Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    if path == "-" {
        let mut stdout = io::stdout().lock();
        stdout.write_all(&bytes)?;
        stdout.flush()?;
    } else {
        fs::write(path, bytes)?;
    }
    Ok(())
}

fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let command = args
        .first()
        .ok_or("usage: pcit-evaluator <validate|agent-view|score|reference-validate|reference-view|reference-audit> [options]")?;
    let options = parse_options(&args[1..])?;

    match command.as_str() {
        "validate" => {
            let schema_path = required(&options, "--schema")?;
            let corpus_path = required(&options, "--corpus")?;
            let (schema_bytes, schema) = load_json(schema_path)?;
            let (corpus_bytes, corpus) = load_json(corpus_path)?;
            validate_schema_document(&schema, schema_path)?;
            validate_instance(&schema, &corpus, corpus_path)?;
            let count = corpus
                .get("cases")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let report = json!({
                "valid": true,
                "schema_sha256": sha256_hex(&schema_bytes),
                "corpus_sha256": sha256_hex(&corpus_bytes),
                "case_count": count,
                "schema_draft": "2020-12"
            });
            write_json("-", &report)?;
        }
        "agent-view" => {
            let corpus_path = required(&options, "--corpus")?;
            let output_path = required(&options, "--out")?;
            let (corpus_bytes, corpus) = load_json(corpus_path)?;
            let digest = sha256_hex(&corpus_bytes);
            let view = build_agent_view(&corpus, &digest)?;
            write_json(output_path, &view)?;
        }
        "reference-validate" => {
            let schema_path = required(&options, "--schema")?;
            let corpus_path = required(&options, "--corpus")?;
            let (schema_bytes, schema) = load_json(schema_path)?;
            let (corpus_bytes, corpus) = load_json(corpus_path)?;
            validate_schema_document(&schema, schema_path)?;
            validate_instance(&schema, &corpus, corpus_path)?;
            let collection_counts: BTreeMap<String, usize> = [
                "cases",
                "indices",
                "ser_attribute_crosswalk",
                "sources",
                "adversarial_controls",
            ]
            .into_iter()
            .filter_map(|key| {
                corpus[key]
                    .as_array()
                    .map(|items| (key.to_string(), items.len()))
            })
            .collect();
            let report = json!({
                "valid": true,
                "schema_sha256": sha256_hex(&schema_bytes),
                "corpus_sha256": sha256_hex(&corpus_bytes),
                "collection_counts": collection_counts,
                "schema_draft": "2020-12"
            });
            write_json("-", &report)?;
        }
        "reference-view" => {
            let corpus_path = required(&options, "--corpus")?;
            let output_path = required(&options, "--out")?;
            let (corpus_bytes, corpus) = load_json(corpus_path)?;
            let view = build_reference_agent_view(&corpus, &sha256_hex(&corpus_bytes))?;
            write_json(output_path, &view)?;
        }
        "reference-audit" => {
            let schema_path = required(&options, "--schema")?;
            let corpus_path = required(&options, "--corpus")?;
            let output_path = required(&options, "--out")?;
            let (corpus_bytes, corpus) = load_json(corpus_path)?;
            let (_schema_bytes, schema) = load_json(schema_path)?;
            validate_schema_document(&schema, schema_path)?;
            validate_instance(&schema, &corpus, corpus_path)?;
            let report = audit_reference_corpus(&corpus, &sha256_hex(&corpus_bytes));
            write_json(output_path, &report)?;
        }
        "score" => {
            let corpus_schema_path = required(&options, "--corpus-schema")?;
            let corpus_path = required(&options, "--corpus")?;
            let result_schema_path = required(&options, "--result-schema")?;
            let results_path = required(&options, "--results")?;
            let output_path = required(&options, "--out")?;

            let (_corpus_schema_bytes, corpus_schema) = load_json(corpus_schema_path)?;
            let (corpus_bytes, corpus) = load_json(corpus_path)?;
            let (_result_schema_bytes, result_schema) = load_json(result_schema_path)?;
            let (_results_bytes, results) = load_json(results_path)?;

            validate_schema_document(&corpus_schema, corpus_schema_path)?;
            validate_schema_document(&result_schema, result_schema_path)?;
            validate_instance(&corpus_schema, &corpus, corpus_path)?;
            validate_instance(&result_schema, &results, results_path)?;

            let digest = sha256_hex(&corpus_bytes);
            let report = score_results(&corpus, &results, &digest);
            write_json(output_path, &report)?;
        }
        _ => {
            return Err(
                format!("unknown command {command:?}; use validate, agent-view, score, reference-validate, reference-view or reference-audit").into(),
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pcit-evaluator: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_case(case_id: &str, truth: &str, expected: &str) -> Value {
        json!({
            "case_id": case_id,
            "truth_state": truth,
            "expected_disposition": expected,
            "measurement_profiles": [{"profile_id":"SENSOR-A"}],
            "reference_design": {
                "availability":"independent_sample",
                "independence":"independent",
                "rights_status":"cleared_synthetic"
            },
            "perturbations": [{
                "admissibility":"negative_control",
                "expected_effect":"apparent_change_without_target_change"
            }],
            "source_metadata_conflicts": [],
            "hard_invariants": []
        })
    }

    fn sample_envelope(digest: &str, results: Vec<Value>) -> Value {
        json!({
            "schema_version":"0.1.0",
            "benchmark_id":BENCHMARK_ID,
            "corpus_sha256":digest,
            "candidate_id":"test-agent",
            "results":results
        })
    }

    fn sample_corpus(cases: Vec<Value>) -> Value {
        json!({
            "schema_version":"0.1.0",
            "benchmark_id":BENCHMARK_ID,
            "cases":cases
        })
    }

    fn sample_result(case_id: &str, disposition: &str) -> Value {
        json!({"case_id":case_id,"disposition":disposition})
    }

    #[test]
    fn locked_corpus_passes_full_json_schema_2020_12() {
        let schema: Value = serde_json::from_str(CORPUS_SCHEMA).expect("schema parses");
        let corpus: Value = serde_json::from_str(LOCKED_CORPUS).expect("corpus parses");
        validate_schema_document(&schema, "embedded PCIT corpus schema").expect("schema compiles");
        validate_instance(&schema, &corpus, "embedded PCIT corpus").expect("corpus validates");
        assert_eq!(corpus["cases"].as_array().expect("cases").len(), 15);
    }

    #[test]
    fn result_schema_is_valid_and_known_answer_envelope_validates() {
        let schema: Value = serde_json::from_str(RESULT_SCHEMA).expect("result schema parses");
        validate_schema_document(&schema, "embedded result schema")
            .expect("result schema compiles");
        let envelope = sample_envelope(
            &"a".repeat(64),
            vec![sample_result(
                "PCIT-SYN-001",
                "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE",
            )],
        );
        validate_instance(&schema, &envelope, "known-answer result").expect("result validates");
    }

    fn contains_forbidden_key(value: &Value) -> bool {
        const FORBIDDEN: &[&str] = &[
            "family",
            "truth_state",
            "truth_note",
            "perturbations",
            "expected_disposition",
        ];
        match value {
            Value::Object(map) => map.iter().any(|(key, value)| {
                FORBIDDEN.contains(&key.as_str()) || contains_forbidden_key(value)
            }),
            Value::Array(items) => items.iter().any(contains_forbidden_key),
            _ => false,
        }
    }

    #[test]
    fn agent_view_omits_oracle_fields_recursively() {
        let corpus: Value = serde_json::from_str(LOCKED_CORPUS).expect("corpus parses");
        let view = build_agent_view(&corpus, &"b".repeat(64)).expect("agent view builds");
        assert_eq!(view["metadata_only"], true);
        assert_eq!(view["cases"].as_array().expect("cases").len(), 15);
        assert!(!contains_forbidden_key(&view));
    }

    #[test]
    fn full_locked_corpus_oracle_answers_pass_without_gate_violations() {
        let corpus: Value = serde_json::from_str(LOCKED_CORPUS).expect("corpus parses");
        let results: Vec<Value> = corpus["cases"]
            .as_array()
            .expect("cases")
            .iter()
            .map(|case| {
                sample_result(
                    case["case_id"].as_str().expect("case id"),
                    case["expected_disposition"]
                        .as_str()
                        .expect("expected disposition"),
                )
            })
            .collect();
        let digest = sha256_hex(LOCKED_CORPUS.as_bytes());
        let submitted = sample_envelope(&digest, results);
        let report = score_results(&corpus, &submitted, &digest);
        assert_eq!(report["summary"]["cases_total"], 15);
        assert_eq!(report["summary"]["correct_dispositions"], 15);
        assert_eq!(report["summary"]["hard_gate_violation_count"], 0);
        assert_eq!(report["summary"]["qualification_pass"], true);
    }

    #[test]
    fn dependent_reference_blocks_state_promotion() {
        let mut case = sample_case(
            "PCIT-SYN-009",
            "no_physical_change",
            "MEASUREMENT_SHIFT_EXPLAINS_APPARENT_CHANGE",
        );
        case["reference_design"]["availability"] = json!("derivative_product");
        case["reference_design"]["independence"] = json!("dependent");
        let corpus = sample_corpus(vec![case]);
        let results = sample_envelope(
            &"9".repeat(64),
            vec![sample_result(
                "PCIT-SYN-009",
                "NO_CHANGE_DETECTED_WITHIN_ENVELOPE",
            )],
        );
        let report = score_results(&corpus, &results, &"9".repeat(64));
        assert_eq!(report["summary"]["hard_gate_pass"], false);
        assert!(
            report["hard_gate_counts"]
                .get("independent_reference_not_qualified")
                .is_some()
        );
    }

    #[test]
    fn result_schema_rejects_oracle_fields_in_agent_submission() {
        let schema: Value = serde_json::from_str(RESULT_SCHEMA).expect("result schema parses");
        let mut envelope = sample_envelope(
            &"a".repeat(64),
            vec![sample_result(
                "PCIT-SYN-001",
                "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE",
            )],
        );
        envelope["results"][0]["expected_disposition"] =
            json!("PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE");
        assert!(validate_instance(&schema, &envelope, "leaky result envelope").is_err());
    }

    #[test]
    fn known_answer_scores_and_replays_deterministically() {
        let corpus = sample_corpus(vec![sample_case(
            "PCIT-SYN-001",
            "physical_change",
            "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE",
        )]);
        let results = sample_envelope(
            &"c".repeat(64),
            vec![sample_result(
                "PCIT-SYN-001",
                "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE",
            )],
        );
        let report_a = score_results(&corpus, &results, &"c".repeat(64));
        let report_b = score_results(&corpus, &results, &"c".repeat(64));
        assert_eq!(report_a["summary"]["qualification_pass"], true);
        assert_eq!(report_a["summary"]["disposition_accuracy"], 1.0);
        assert_eq!(
            serde_json::to_vec(&report_a).expect("serialize"),
            serde_json::to_vec(&report_b).expect("serialize")
        );
    }

    #[test]
    fn false_physical_change_on_null_control_is_a_hard_gate() {
        let corpus = sample_corpus(vec![sample_case(
            "PCIT-SYN-002",
            "no_physical_change",
            "MEASUREMENT_SHIFT_EXPLAINS_APPARENT_CHANGE",
        )]);
        let results = sample_envelope(
            &"d".repeat(64),
            vec![sample_result(
                "PCIT-SYN-002",
                "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE",
            )],
        );
        let report = score_results(&corpus, &results, &"d".repeat(64));
        assert_eq!(report["summary"]["hard_gate_pass"], false);
        assert_eq!(report["summary"]["false_physical_change_promotions"], 1);
        assert!(
            report["hard_gate_counts"]
                .get("false_physical_change_promotion")
                .is_some()
        );
    }

    #[test]
    fn unresolved_source_metadata_blocks_definitive_claims() {
        let mut case = sample_case(
            "PCIT-SYN-014",
            "not_identifiable",
            "INSUFFICIENT_OBSERVABILITY",
        );
        case["source_metadata_conflicts"] = json!([{
            "resolution_status":"unresolved"
        }]);
        let corpus = sample_corpus(vec![case]);
        let results = sample_envelope(
            &"e".repeat(64),
            vec![sample_result(
                "PCIT-SYN-014",
                "MEASUREMENT_SHIFT_EXPLAINS_APPARENT_CHANGE",
            )],
        );
        let report = score_results(&corpus, &results, &"e".repeat(64));
        assert_eq!(report["summary"]["hard_gate_pass"], false);
        assert!(
            report["hard_gate_counts"]
                .get("unresolved_source_metadata_conflict")
                .is_some()
        );
    }

    #[test]
    fn unreconciled_area_totals_block_definitive_claims() {
        let mut case = sample_case("PCIT-SYN-015", "not_identifiable", "UNRESOLVED_CONFOUNDING");
        case["numeric_reconciliation"] = json!({
            "reported_value_a":76052.8,
            "reported_value_b":41209.0,
            "tolerance":0.5,
            "resolution_status":"unresolved"
        });
        let corpus = sample_corpus(vec![case]);
        let results = sample_envelope(
            &"f".repeat(64),
            vec![sample_result(
                "PCIT-SYN-015",
                "NO_CHANGE_DETECTED_WITHIN_ENVELOPE",
            )],
        );
        let report = score_results(&corpus, &results, &"f".repeat(64));
        assert_eq!(report["summary"]["hard_gate_pass"], false);
        assert!(
            report["hard_gate_counts"]
                .get("numeric_area_reconciliation_unresolved")
                .is_some()
        );
    }

    #[test]
    fn missing_duplicate_unexpected_and_digest_mismatch_are_reported() {
        let corpus = sample_corpus(vec![
            sample_case(
                "PCIT-SYN-001",
                "physical_change",
                "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE",
            ),
            sample_case(
                "PCIT-SYN-002",
                "no_physical_change",
                "MEASUREMENT_SHIFT_EXPLAINS_APPARENT_CHANGE",
            ),
        ]);
        let results = sample_envelope(
            &"0".repeat(64),
            vec![
                sample_result("PCIT-SYN-001", "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE"),
                sample_result("PCIT-SYN-001", "PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE"),
                sample_result("PCIT-SYN-999", "INSUFFICIENT_OBSERVABILITY"),
            ],
        );
        let report = score_results(&corpus, &results, &"1".repeat(64));
        assert_eq!(report["summary"]["hard_gate_pass"], false);
        assert_eq!(
            report["summary"]["missing_case_ids"]
                .as_array()
                .expect("missing")
                .len(),
            1
        );
        assert_eq!(
            report["summary"]["duplicate_case_ids"]
                .as_array()
                .expect("duplicates")
                .len(),
            1
        );
        assert_eq!(
            report["summary"]["unexpected_case_ids"]
                .as_array()
                .expect("unknown")
                .len(),
            1
        );
        assert_eq!(report["integrity"]["corpus_digest_matches"], false);
    }

    #[test]
    fn south_african_remp_crosswalk_validates_and_preserves_ser_attribute_gaps() {
        let schema: Value =
            serde_json::from_str(REMP_CROSSWALK_SCHEMA).expect("REMP crosswalk schema parses");
        let crosswalk: Value =
            serde_json::from_str(REMP_CROSSWALK_CORPUS).expect("REMP crosswalk corpus parses");
        validate_schema_document(&schema, "embedded REMP crosswalk schema")
            .expect("REMP crosswalk schema compiles");
        validate_instance(&schema, &crosswalk, "embedded REMP crosswalk")
            .expect("REMP crosswalk validates");

        assert_eq!(crosswalk["indices"].as_array().expect("indices").len(), 9);
        assert_eq!(
            crosswalk["ser_attribute_crosswalk"]
                .as_array()
                .expect("SER attributes")
                .len(),
            6
        );
        assert!(
            crosswalk["ser_attribute_crosswalk"]
                .as_array()
                .expect("SER attributes")
                .iter()
                .any(|item| item["ser_attribute"] == "ecosystem_function"
                    && item["coverage_status"] == "partial_proxy")
        );
        assert!(
            crosswalk["non_equivalence_rules"]
                .as_array()
                .expect("non-equivalence rules")
                .iter()
                .any(|rule| rule
                    .as_str()
                    .is_some_and(|text| text.contains("SER Five-star")))
        );

        let legacy = &crosswalk["legacy_jukskei_assessment"];
        assert_eq!(legacy["programme"], "legacy_RHP");
        assert_eq!(legacy["overall_condition"]["reported_category"], "D");
        assert_eq!(
            legacy["sub_quaternary_reaches"]
                .as_array()
                .expect("legacy reaches")
                .len(),
            5
        );
        assert_eq!(legacy["monitoring_site"]["site_id"], "A2JUKS-DIENR");
        assert_eq!(legacy["monitoring_site"]["reach_id"], "A21C-01167");
        assert_eq!(
            legacy["admission_status"],
            "published_report_summary_raw_records_pending"
        );
        assert!(
            legacy["sampling_events"]
                .as_array()
                .expect("sampling events")
                .iter()
                .any(|event| event["event_type"] == "not_sampled" && event["date"] == "2015")
        );
        assert!(
            legacy["indicator_summaries"]
                .as_array()
                .expect("indicator summaries")
                .iter()
                .any(|item| item["indicator"] == "macroinvertebrate_MIRAI_EC"
                    && item["measurement_status"] == "derived_from_prior_data")
        );
    }

    #[test]
    fn reference_schema_and_locked_corpus_validate_under_json_schema_2020_12() {
        let schema: Value =
            serde_json::from_str(REFERENCE_SCHEMA).expect("reference schema parses");
        let corpus: Value =
            serde_json::from_str(REFERENCE_CORPUS).expect("reference corpus parses");
        validate_schema_document(&schema, "embedded reference schema")
            .expect("reference schema compiles");
        validate_instance(&schema, &corpus, "embedded reference corpus")
            .expect("reference corpus validates");
        assert_eq!(corpus["cases"].as_array().expect("cases").len(), 10);
    }

    #[test]
    fn reference_audit_matches_all_locked_expected_blockers() {
        let corpus: Value =
            serde_json::from_str(REFERENCE_CORPUS).expect("reference corpus parses");
        for case in corpus["cases"].as_array().expect("cases") {
            let audit = reference_audit_case(case);
            let expected: BTreeSet<String> = case["expected_audit"]["expected_blockers"]
                .as_array()
                .expect("expected blockers")
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
            let actual: BTreeSet<String> = audit["blockers"]
                .as_array()
                .expect("audit blockers")
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
            assert_eq!(
                actual,
                expected,
                "blocker mismatch for {}",
                case["case_id"].as_str().unwrap_or("<unknown>")
            );
            assert_eq!(
                audit["reference_model_qualified"],
                case["expected_audit"]["reference_model_qualified"],
                "reference qualification mismatch for {}",
                case["case_id"].as_str().unwrap_or("<unknown>")
            );
            assert_eq!(
                audit["overall_claim_permitted"],
                case["expected_audit"]["overall_claim_permitted"],
                "claim permission mismatch for {}",
                case["case_id"].as_str().unwrap_or("<unknown>")
            );
        }
    }

    fn contains_reference_oracle_key(value: &Value) -> bool {
        const FORBIDDEN: &[&str] = &["family", "expected_audit", "hard_invariants"];
        match value {
            Value::Object(map) => map.iter().any(|(key, value)| {
                FORBIDDEN.contains(&key.as_str()) || contains_reference_oracle_key(value)
            }),
            Value::Array(items) => items.iter().any(contains_reference_oracle_key),
            _ => false,
        }
    }

    #[test]
    fn reference_agent_view_excludes_oracle_fields_recursively() {
        let corpus: Value =
            serde_json::from_str(REFERENCE_CORPUS).expect("reference corpus parses");
        let view = build_reference_agent_view(&corpus, &"a".repeat(64))
            .expect("reference agent view builds");
        assert_eq!(view["metadata_only"], true);
        assert_eq!(view["cases"].as_array().expect("cases").len(), 10);
        assert!(!contains_reference_oracle_key(&view));
    }

    #[test]
    fn worsening_attribute_cannot_be_compensated_by_other_improvements() {
        let corpus: Value =
            serde_json::from_str(REFERENCE_CORPUS).expect("reference corpus parses");
        let case = corpus["cases"]
            .as_array()
            .expect("cases")
            .iter()
            .find(|case| case["case_id"] == "REF-SYN-005")
            .expect("tradeoff fixture");
        let audit = reference_audit_case(case);
        assert_eq!(audit["reference_model_qualified"], true);
        assert_eq!(audit["overall_claim_permitted"], false);
        assert_eq!(
            audit["claim_blockers"][0],
            "non_compensatory_attribute_regression"
        );
        assert_eq!(
            audit["worsening_attributes"]
                .as_array()
                .expect("worsening")
                .len(),
            1
        );
    }

    #[test]
    fn unobserved_attribute_blocks_positive_overall_recovery_claim() {
        let corpus: Value =
            serde_json::from_str(REFERENCE_CORPUS).expect("reference corpus parses");
        let case = corpus["cases"]
            .as_array()
            .expect("cases")
            .iter()
            .find(|case| case["case_id"] == "REF-SYN-010")
            .expect("unobserved attribute fixture");
        let audit = reference_audit_case(case);
        assert_eq!(audit["reference_model_qualified"], true);
        assert_eq!(audit["overall_claim_permitted"], false);
        assert!(
            audit["blockers"]
                .as_array()
                .expect("blockers")
                .iter()
                .any(|item| item == "unobserved_attribute_in_positive_recovery_claim")
        );
    }

    #[test]
    fn dynamic_reference_audit_is_deterministic() {
        let corpus: Value =
            serde_json::from_str(REFERENCE_CORPUS).expect("reference corpus parses");
        let digest = sha256_hex(REFERENCE_CORPUS.as_bytes());
        let first = audit_reference_corpus(&corpus, &digest);
        let second = audit_reference_corpus(&corpus, &digest);
        assert_eq!(
            serde_json::to_vec(&first).expect("serialize first"),
            serde_json::to_vec(&second).expect("serialize second")
        );
    }
}
