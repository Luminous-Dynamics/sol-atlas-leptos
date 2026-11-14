// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const CORPUS: &str = include_str!(
    "../../docs/research/home-freedom/HOME_FREEDOM_100_CASE_CORPUS_V0.1.json"
);
const CORPUS_SCHEMA: &str = include_str!(
    "../../docs/research/home-freedom/HOME_FREEDOM_100_CASE_CORPUS_V0.1.schema.json"
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

fn required_ids(profile: &Value) -> Result<Vec<String>, String> {
    let values = array_at(profile, "required_gate_ids")
        .ok_or_else(|| "household profile has no required_gate_ids".to_owned())?;
    let mut seen = BTreeSet::new();
    let mut ids = Vec::new();
    for value in values {
        let id = value
            .as_str()
            .ok_or_else(|| "required gate ID must be text".to_owned())?;
        if !seen.insert(id.to_owned()) {
            return Err(format!("duplicate required gate ID {id}"));
        }
        ids.push(id.to_owned());
    }
    Ok(ids)
}

fn evaluate_case(
    case: &Value,
    profiles: &BTreeMap<String, Value>,
    scenarios: &BTreeMap<String, Value>,
) -> Result<(String, Vec<String>), String> {
    let profile_id = text_at(case, "household_profile_id")
        .ok_or_else(|| "case is missing household_profile_id".to_owned())?;
    let scenario_id = text_at(case, "site_scenario_id")
        .ok_or_else(|| "case is missing site_scenario_id".to_owned())?;
    let profile = profiles
        .get(profile_id)
        .ok_or_else(|| format!("unknown household profile {profile_id}"))?;
    let scenario = scenarios
        .get(scenario_id)
        .ok_or_else(|| format!("unknown site scenario {scenario_id}"))?;

    let required = required_ids(profile)?;
    let case_required = required_ids(case)?;
    if required != case_required {
        return Ok((
            "NOT_COMPARABLE".to_owned(),
            vec!["HOUSEHOLD_REQUIREMENT_SCOPE_MISMATCH".to_owned()],
        ));
    }

    let mut observed = BTreeMap::<String, String>::new();
    if let Some(rows) = array_at(case, "gate_observations") {
        for row in rows {
            let Some(gate_id) = text_at(row, "gate_id") else {
                return Ok((
                    "NOT_COMPARABLE".to_owned(),
                    vec!["GATE_ID_MISSING".to_owned()],
                ));
            };
            let Some(state) = text_at(row, "state") else {
                return Ok((
                    "NOT_COMPARABLE".to_owned(),
                    vec![format!("{gate_id}_STATE_MISSING")],
                ));
            };
            if observed.insert(gate_id.to_owned(), state.to_owned()).is_some() {
                return Ok((
                    "NOT_COMPARABLE".to_owned(),
                    vec![format!("{gate_id}_DUPLICATE_OBSERVATION")],
                ));
            }
        }
    } else {
        return Ok((
            "NOT_COMPARABLE".to_owned(),
            vec!["GATE_OBSERVATIONS_MISSING".to_owned()],
        ));
    }

    let scenario_observations = scenario
        .get("observations")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("scenario {scenario_id} has no observations object"))?;

    let mut reason_codes = Vec::new();
    let mut has_unknown = false;
    let mut has_failure = false;
    for gate_id in &required {
        let Some(state) = observed.get(gate_id) else {
            return Ok((
                "NOT_COMPARABLE".to_owned(),
                vec![format!("{gate_id}_OBSERVATION_MISSING")],
            ));
        };
        let Some(scenario_state) = scenario_observations.get(gate_id).and_then(Value::as_str) else {
            return Ok((
                "NOT_COMPARABLE".to_owned(),
                vec![format!("{gate_id}_SCENARIO_SCOPE_MISSING")],
            ));
        };
        if state != scenario_state {
            return Ok((
                "NOT_COMPARABLE".to_owned(),
                vec![format!("{gate_id}_CASE_SCENARIO_MISMATCH")],
            ));
        }
        match state.as_str() {
            "PASS" => {}
            "UNKNOWN" => {
                has_unknown = true;
                reason_codes.push(format!("{gate_id}_UNKNOWN"));
            }
            "FAIL" => {
                has_failure = true;
                reason_codes.push(format!("{gate_id}_FAIL"));
            }
            "NOT_APPLICABLE" => {
                return Ok((
                    "NOT_COMPARABLE".to_owned(),
                    vec![format!("{gate_id}_REQUIRED_BUT_NOT_APPLICABLE")],
                ));
            }
            _ => {
                return Ok((
                    "NOT_COMPARABLE".to_owned(),
                    vec![format!("{gate_id}_INVALID_STATE")],
                ));
            }
        }
    }

    reason_codes.sort();
    let disposition = if has_failure {
        "BLOCKED"
    } else if has_unknown {
        "CONDITIONAL_CONCEPT_ONLY"
    } else {
        "SCREENING_ELIGIBLE"
    };
    Ok((disposition.to_owned(), reason_codes))
}

fn validate(corpus: &Value, schema: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("corpus schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(corpus, "schema_version") != Some("0.1.0")
        || text_at(corpus, "study_id") != Some("HOME-FREEDOM-100-CASE-CORPUS-001")
        || text_at(corpus, "maturity") != Some("SYNTHETIC_FROZEN_CORPUS")
        || text_at(corpus, "execution_status") != Some("NOT_EXECUTED")
    {
        errors.push("corpus identity or maturity header is invalid".to_owned());
    }
    if corpus.get("expected_case_count").and_then(Value::as_u64) != Some(100) {
        errors.push("expected_case_count must be exactly 100".to_owned());
    }

    let mut profiles = BTreeMap::<String, Value>::new();
    if let Some(items) = array_at(corpus, "household_profiles") {
        if items.len() != 10 {
            errors.push(format!("expected 10 household profiles, found {}", items.len()));
        }
        for item in items {
            let id = text_at(item, "id").unwrap_or("<missing-profile-id>").to_owned();
            if profiles.insert(id.clone(), item.clone()).is_some() {
                errors.push(format!("duplicate household profile ID {id}"));
            }
            if let Err(error) = required_ids(item) {
                errors.push(format!("profile {id}: {error}"));
            }
            if item.get("step_free").and_then(Value::as_bool).is_none() {
                errors.push(format!("profile {id}: step_free must be explicitly declared"));
            }
        }
    } else {
        errors.push("household_profiles is missing".to_owned());
    }

    let mut scenarios = BTreeMap::<String, Value>::new();
    if let Some(items) = array_at(corpus, "site_scenarios") {
        if items.len() != 10 {
            errors.push(format!("expected 10 site scenarios, found {}", items.len()));
        }
        for item in items {
            let id = text_at(item, "id").unwrap_or("<missing-scenario-id>").to_owned();
            if scenarios.insert(id.clone(), item.clone()).is_some() {
                errors.push(format!("duplicate site scenario ID {id}"));
            }
            let Some(observations) = item.get("observations").and_then(Value::as_object) else {
                errors.push(format!("scenario {id}: observations object is missing"));
                continue;
            };
            for gate in observations {
                if !matches!(
                    gate.1.as_str(),
                    Some("PASS" | "UNKNOWN" | "FAIL" | "NOT_APPLICABLE")
                ) {
                    errors.push(format!("scenario {id}: invalid state for {}", gate.0));
                }
            }
        }
    } else {
        errors.push("site_scenarios is missing".to_owned());
    }

    let mut case_ids = BTreeSet::new();
    let mut counts = BTreeMap::<String, u64>::new();
    let mut expected_reason_mismatches = 0usize;
    let mut case_count = 0usize;
    if let Some(items) = array_at(corpus, "cases") {
        if items.len() != 100 {
            errors.push(format!("expected exactly 100 cases, found {}", items.len()));
        }
        for case in items {
            case_count += 1;
            let id = text_at(case, "case_id").unwrap_or("<missing-case-id>").to_owned();
            if !case_ids.insert(id.clone()) {
                errors.push(format!("duplicate case ID {id}"));
                continue;
            }

            match evaluate_case(case, &profiles, &scenarios) {
                Ok((actual, mut reasons)) => {
                    let expected = text_at(case, "expected_disposition").unwrap_or("MISSING");
                    if actual != expected {
                        errors.push(format!("{id}: expected {expected}, evaluated {actual}"));
                    }
                    reasons.sort();
                    let mut expected_reasons = array_at(case, "expected_reason_codes")
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    expected_reasons.sort();
                    if reasons != expected_reasons {
                        expected_reason_mismatches += 1;
                        errors.push(format!("{id}: expected reason codes do not match observed gate states"));
                    }
                    *counts.entry(actual).or_insert(0) += 1;
                }
                Err(error) => errors.push(format!("{id}: {error}")),
            }
        }
    } else {
        errors.push("cases array is missing".to_owned());
    }

    let eligible = counts.get("SCREENING_ELIGIBLE").copied().unwrap_or(0);
    let conditional = counts.get("CONDITIONAL_CONCEPT_ONLY").copied().unwrap_or(0);
    let blocked = counts.get("BLOCKED").copied().unwrap_or(0);
    let not_comparable = counts.get("NOT_COMPARABLE").copied().unwrap_or(0);
    if eligible == 0 || conditional == 0 || blocked == 0 {
        errors.push("corpus must preserve eligible, conditional and blocked positive/negative controls".to_owned());
    }
    if case_count != case_ids.len() {
        errors.push("case identity coverage is incomplete".to_owned());
    }

    if errors.is_empty() {
        Ok(json!({
            "audit_state":"HOME_FREEDOM_100_CASE_CORPUS_PASS",
            "study_id":"HOME-FREEDOM-100-CASE-CORPUS-001",
            "household_profile_count":profiles.len(),
            "site_scenario_count":scenarios.len(),
            "case_count":case_count,
            "disposition_counts":{
                "SCREENING_ELIGIBLE":eligible,
                "CONDITIONAL_CONCEPT_ONLY":conditional,
                "BLOCKED":blocked,
                "NOT_COMPARABLE":not_comparable
            },
            "expected_reason_mismatches":expected_reason_mismatches,
            "false_screening_eligible_count":0,
            "claim_ceiling":"SYNTHETIC_P0_DISPOSITION_ORACLE_ONLY"
        }))
    } else {
        Err(errors)
    }
}

fn fixture() -> (Value, Value) {
    (
        parse_json(CORPUS, "100-case corpus").expect("corpus JSON"),
        parse_json(CORPUS_SCHEMA, "corpus schema").expect("corpus schema JSON"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-freedom-corpus-qualifier [--self-test]");
        return;
    }
    let (corpus, schema) = fixture();
    match validate(&corpus, &schema) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
        }
        Err(errors) => {
            eprintln!("HOME_FREEDOM_100_CASE_CORPUS_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_corpus_matches_all_one_hundred_known_answers() {
        let (corpus, schema) = fixture();
        let report = validate(&corpus, &schema).expect("100-case oracle must qualify");
        assert_eq!(report["household_profile_count"], 10);
        assert_eq!(report["site_scenario_count"], 10);
        assert_eq!(report["case_count"], 100);
        assert_eq!(report["disposition_counts"]["SCREENING_ELIGIBLE"], 17);
        assert_eq!(report["disposition_counts"]["CONDITIONAL_CONCEPT_ONLY"], 60);
        assert_eq!(report["disposition_counts"]["BLOCKED"], 23);
        assert_eq!(report["disposition_counts"]["NOT_COMPARABLE"], 0);
    }

    #[test]
    fn unknown_required_gate_never_promotes_to_screening_eligible() {
        let (mut corpus, schema) = fixture();
        let case = corpus["cases"].as_array_mut().unwrap().iter_mut()
            .find(|case| case["case_id"] == "H01_COMPACT_SINGLE__S01_COMPLETE_REFERENCE")
            .unwrap();
        let observation = case["gate_observations"].as_array_mut().unwrap().iter_mut()
            .find(|gate| gate["gate_id"] == "SITE_SURVEY").unwrap();
        observation["state"] = json!("UNKNOWN");
        assert_eq!(
            evaluate_case(case, &profile_map(&corpus), &scenario_map(&corpus)).unwrap().0,
            "CONDITIONAL_CONCEPT_ONLY"
        );
    }

    #[test]
    fn failed_required_gate_blocks_only_the_affected_case() {
        let (corpus, _) = fixture();
        let profiles = profile_map(&corpus);
        let scenarios = scenario_map(&corpus);
        let standard = corpus["cases"].as_array().unwrap().iter()
            .find(|case| case["case_id"] == "H01_COMPACT_SINGLE__S07_STEP_FREE_FAILED")
            .unwrap();
        let step_free = corpus["cases"].as_array().unwrap().iter()
            .find(|case| case["case_id"] == "H05_STEP_FREE_FIRST__S07_STEP_FREE_FAILED")
            .unwrap();
        assert_eq!(evaluate_case(standard, &profiles, &scenarios).unwrap().0, "SCREENING_ELIGIBLE");
        assert_eq!(evaluate_case(step_free, &profiles, &scenarios).unwrap().0, "BLOCKED");
    }

    #[test]
    fn missing_required_gate_is_not_comparable() {
        let (corpus, _) = fixture();
        let mut case = corpus["cases"][0].clone();
        case["gate_observations"].as_array_mut().unwrap().pop();
        let result = evaluate_case(&case, &profile_map(&corpus), &scenario_map(&corpus)).unwrap();
        assert_eq!(result.0, "NOT_COMPARABLE");
    }

    #[test]
    fn duplicate_gate_observation_is_not_comparable() {
        let (corpus, _) = fixture();
        let mut case = corpus["cases"][0].clone();
        let duplicate = case["gate_observations"][0].clone();
        case["gate_observations"].as_array_mut().unwrap().push(duplicate);
        let result = evaluate_case(&case, &profile_map(&corpus), &scenario_map(&corpus)).unwrap();
        assert_eq!(result.0, "NOT_COMPARABLE");
    }

    #[test]
    fn required_not_applicable_gate_is_not_comparable() {
        let (corpus, _) = fixture();
        let mut case = corpus["cases"].as_array().unwrap().iter()
            .find(|case| case["case_id"] == "H05_STEP_FREE_FIRST__S01_COMPLETE_REFERENCE")
            .unwrap().clone();
        let observation = case["gate_observations"].as_array_mut().unwrap().iter_mut()
            .find(|gate| gate["gate_id"] == "STEP_FREE_INTERNAL_ROUTE").unwrap();
        observation["state"] = json!("NOT_APPLICABLE");
        let result = evaluate_case(&case, &profile_map(&corpus), &scenario_map(&corpus)).unwrap();
        assert_eq!(result.0, "NOT_COMPARABLE");
    }

    #[test]
    fn case_order_does_not_change_disposition_counts() {
        let (mut corpus, schema) = fixture();
        let before = validate(&corpus, &schema).unwrap();
        corpus["cases"].as_array_mut().unwrap().reverse();
        let after = validate(&corpus, &schema).unwrap();
        assert_eq!(before["disposition_counts"], after["disposition_counts"]);
    }
    
    fn profile_map(corpus: &Value) -> BTreeMap<String, Value> {
        array_at(corpus, "household_profiles").unwrap().iter()
            .map(|item| (text_at(item, "id").unwrap().to_owned(), item.clone()))
            .collect()
    }
    
    fn scenario_map(corpus: &Value) -> BTreeMap<String, Value> {
        array_at(corpus, "site_scenarios").unwrap().iter()
            .map(|item| (text_at(item, "id").unwrap().to_owned(), item.clone()))
            .collect()
    }
}
