// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, process};

const BRIEF_FIXTURE: &str = include_str!(
    "../../docs/research/home-freedom/HOME_DESIGN_BRIEF_SYNTHETIC_EXAMPLE_V0.1.json"
);
const CANDIDATE_FIXTURE: &str = include_str!(
    "../../docs/research/home-freedom/HOME_DESIGN_CANDIDATE_SET_SYNTHETIC_EXAMPLE_V0.1.json"
);
const MANIFEST_FIXTURE: &str = include_str!(
    "../../docs/research/home-freedom/HOME_FREEDOM_BENCHMARK_MANIFEST_V0.1.json"
);
const BRIEF_SCHEMA: &str = include_str!(
    "../../docs/research/home-freedom/HOME_DESIGN_BRIEF_PROFILE_V0.1.schema.json"
);
const CANDIDATE_SCHEMA: &str = include_str!(
    "../../docs/research/home-freedom/HOME_DESIGN_CANDIDATE_SET_V0.1.schema.json"
);
const MANIFEST_SCHEMA: &str = include_str!(
    "../../docs/research/home-freedom/HOME_FREEDOM_BENCHMARK_MANIFEST_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn parse_json(source: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(source).map_err(|error| format!("{label} is not valid JSON: {error}"))
}

fn validate_schema_headers(
    brief_schema: &Value,
    candidate_schema: &Value,
    manifest_schema: &Value,
) -> Vec<String> {
    let mut errors = Vec::new();
    for (label, schema) in [
        ("brief", brief_schema),
        ("candidate-set", candidate_schema),
        ("manifest", manifest_schema),
    ] {
        if text_at(schema, "$schema")
            != Some("https://json-schema.org/draft/2020-12/schema")
        {
            errors.push(format!("{label} schema does not declare JSON Schema Draft 2020-12"));
        }
        if text_at(schema, "type") != Some("object") {
            errors.push(format!("{label} schema root must be an object"));
        }
    }
    errors
}

fn validate(
    brief: &Value,
    candidate_set: &Value,
    manifest: &Value,
    brief_schema: &Value,
    candidate_schema: &Value,
    manifest_schema: &Value,
) -> Result<Value, Vec<String>> {
    let mut errors = validate_schema_headers(brief_schema, candidate_schema, manifest_schema);

    if text_at(brief, "schema_version") != Some("0.1.0")
        || text_at(candidate_set, "schema_version") != Some("0.1.0")
        || text_at(manifest, "schema_version") != Some("0.1.0")
    {
        errors.push("all artifacts must use schema version 0.1.0".to_owned());
    }
    if text_at(brief, "maturity") != Some("SYNTHETIC") {
        errors.push("the embedded P0 fixture must be marked SYNTHETIC".to_owned());
    }
    if text_at(candidate_set, "maturity") != Some("SYNTHETIC_SCREENING_ONLY")
        || text_at(candidate_set, "execution_status") != Some("NOT_SIMULATED")
    {
        errors.push("candidate set must remain SYNTHETIC_SCREENING_ONLY / NOT_SIMULATED".to_owned());
    }
    if text_at(manifest, "maturity") != Some("PROPOSED_NOT_EVALUATED")
        || text_at(manifest, "execution_status") != Some("NOT_EXECUTED")
    {
        errors.push("benchmark manifest must remain PROPOSED_NOT_EVALUATED / NOT_EXECUTED".to_owned());
    }

    let brief_id = text_at(brief, "brief_id").unwrap_or_default();
    if brief_id.is_empty() || text_at(candidate_set, "brief_id") != Some(brief_id) {
        errors.push("candidate set is not bound to the exact resident brief ID".to_owned());
    }

    let privacy = brief.get("privacy").unwrap_or(&Value::Null);
    if privacy.get("contains_direct_identifiers").and_then(Value::as_bool) != Some(false)
        || text_at(privacy, "redaction_status") != Some("REVIEWED_NO_DIRECT_IDENTIFIERS")
        || text_at(privacy, "data_sharing_mode") != Some("SYNTHETIC_PUBLIC")
    {
        errors.push("public synthetic fixture fails its privacy/redaction contract".to_owned());
    }

    let mut activity_ids = BTreeSet::new();
    let activities = array_at(brief.get("intent").unwrap_or(&Value::Null), "activities")
        .cloned()
        .unwrap_or_default();
    if activities.is_empty() {
        errors.push("resident brief must contain at least one activity".to_owned());
    }
    for activity in &activities {
        match text_at(activity, "activity_id") {
            Some(id) if activity_ids.insert(id.to_owned()) => {}
            Some(id) => errors.push(format!("duplicate resident activity ID {id}")),
            None => errors.push("resident activity is missing activity_id".to_owned()),
        }
    }
    if let Some(relationships) = array_at(
        brief.get("intent").unwrap_or(&Value::Null),
        "space_relationships",
    ) {
        for relationship in relationships {
            for key in ["from_activity_id", "to_activity_id"] {
                match text_at(relationship, key) {
                    Some(id) if activity_ids.contains(id) => {}
                    Some(id) => errors.push(format!(
                        "space relationship references unknown activity ID {id}"
                    )),
                    None => errors.push(format!("space relationship is missing {key}")),
                }
            }
        }
    }

    let constraints = array_at(brief, "constraints").cloned().unwrap_or_default();
    let mut constraint_ids = BTreeSet::new();
    for constraint in &constraints {
        match text_at(constraint, "constraint_id") {
            Some(id) if constraint_ids.insert(id.to_owned()) => {}
            Some(id) => errors.push(format!("duplicate resident constraint ID {id}")),
            None => errors.push("resident constraint is missing constraint_id".to_owned()),
        }
        if text_at(constraint, "severity") != Some("NON_NEGOTIABLE") {
            errors.push("P0 fixture constraints must remain non-negotiable".to_owned());
        }
    }
    if constraints.is_empty() {
        errors.push("resident brief must include non-negotiable constraints".to_owned());
    }

    let mut manifest_archetypes = BTreeSet::new();
    if let Some(archetypes) = array_at(manifest, "archetypes") {
        for archetype in archetypes {
            if let Some(id) = text_at(archetype, "id") {
                manifest_archetypes.insert(id.to_owned());
            }
        }
    } else {
        errors.push("benchmark manifest is missing archetypes".to_owned());
    }

    let candidates = array_at(candidate_set, "candidates").cloned().unwrap_or_default();
    if candidates.len() < 2 {
        errors.push("candidate set must preserve at least two alternatives".to_owned());
    }

    let mut candidate_ids = BTreeSet::new();
    let mut gate_rows_checked = 0usize;
    let mut total_floor_area = 0.0_f64;
    for candidate in &candidates {
        let id = text_at(candidate, "candidate_id").unwrap_or("<missing-candidate-id>");
        if !candidate_ids.insert(id.to_owned()) {
            errors.push(format!("duplicate candidate ID {id}"));
        }
        let archetype = text_at(candidate, "archetype_id").unwrap_or_default();
        if !manifest_archetypes.contains(archetype) {
            errors.push(format!("{id}: archetype {archetype} is absent from the benchmark manifest"));
        }
        if text_at(candidate, "disposition") != Some("CONDITIONAL_CONCEPT_ONLY")
            || text_at(candidate, "space_program_state")
                != Some("SYMBOLIC_ROOM_PROGRAM_NOT_GEOMETRY")
        {
            errors.push(format!("{id}: candidate was promoted above symbolic concept maturity"));
        }

        let floor_area = number_at(candidate, "floor_area_m2").unwrap_or(f64::NAN);
        let room_program = array_at(candidate, "room_program").cloned().unwrap_or_default();
        let room_sum: f64 = room_program
            .iter()
            .map(|space| number_at(space, "target_area_m2").unwrap_or(f64::NAN))
            .sum();
        if !floor_area.is_finite()
            || floor_area <= 0.0
            || !room_sum.is_finite()
            || (room_sum - floor_area).abs() > 1e-6
        {
            errors.push(format!(
                "{id}: room-program area sum {room_sum} does not equal floor area {floor_area}"
            ));
        } else {
            total_floor_area += floor_area;
        }

        let mut present_gates = BTreeMap::<String, String>::new();
        if let Some(gates) = array_at(candidate, "hard_gate_results") {
            for gate in gates {
                let gate_id = text_at(gate, "gate_id").unwrap_or_default().to_owned();
                let state = text_at(gate, "state").unwrap_or_default().to_owned();
                if gate_id.is_empty() {
                    errors.push(format!("{id}: hard-gate row has no gate_id"));
                    continue;
                }
                if present_gates.insert(gate_id.clone(), state.clone()).is_some() {
                    errors.push(format!("{id}: duplicate gate row {gate_id}"));
                }
                if state != "UNKNOWN" {
                    errors.push(format!(
                        "{id}: synthetic symbolic case cannot promote gate {gate_id} to {state}"
                    ));
                }
                if gate.get("evidence_ref").is_some_and(|v| !v.is_null()) {
                    errors.push(format!("{id}: UNKNOWN gate {gate_id} must not attach invented evidence"));
                }
                gate_rows_checked += 1;
            }
        } else {
            errors.push(format!("{id}: missing hard_gate_results"));
        }
        for constraint_id in &constraint_ids {
            if present_gates.get(constraint_id).map(String::as_str) != Some("UNKNOWN") {
                errors.push(format!(
                    "{id}: resident hard constraint {constraint_id} has no UNKNOWN result row"
                ));
            }
        }

        let cost = candidate.get("cost_state").unwrap_or(&Value::Null);
        if text_at(cost, "state") != Some("NOT_ESTIMATED")
            || !cost.get("low_estimate").is_some_and(Value::is_null)
            || !cost.get("high_estimate").is_some_and(Value::is_null)
            || !cost.get("rate_evidence_ref").is_some_and(Value::is_null)
        {
            errors.push(format!("{id}: unpriced synthetic case must preserve unknown cost as null"));
        }
        if text_at(cost, "currency") != Some("ZAR") {
            errors.push(format!("{id}: currency must match the synthetic Johannesburg profile (ZAR)"));
        }

        let thermal = candidate.get("thermal_state").unwrap_or(&Value::Null);
        if text_at(thermal, "state") != Some("NOT_SIMULATED")
            || text_at(thermal, "baseline") != Some("NO_WATER_PASSIVE_BASELINE_REQUIRED")
        {
            errors.push(format!("{id}: thermal outcome cannot be inferred or simulated in this fixture"));
        }

        let future = candidate.get("future_change_state").unwrap_or(&Value::Null);
        if text_at(future, "state") != Some("RESERVED_SPACE_ONLY") {
            errors.push(format!("{id}: future extension is not a qualified engineered interface"));
        }
    }

    let policy = candidate_set.get("comparison_policy").unwrap_or(&Value::Null);
    if text_at(policy, "ranking_state") != Some("UNRANKED_PARETO_CANDIDATES")
        || !policy.get("scalar_score").is_some_and(Value::is_null)
        || policy.get("resident_choice_required").and_then(Value::as_bool) != Some(true)
    {
        errors.push("candidate set must remain unranked with resident choice and no scalar score".to_owned());
    }
    if array_at(candidate_set, "common_unknowns").is_none_or(Vec::is_empty) {
        errors.push("candidate set must disclose common unknowns".to_owned());
    }

    if errors.is_empty() {
        Ok(json!({
            "audit_state":"HOME_FREEDOM_SCREEN_PASS",
            "study_id":"HOME-FREEDOM-COMPILER-001",
            "brief_id":brief_id,
            "candidate_count":candidates.len(),
            "hard_constraint_count":constraint_ids.len(),
            "gate_rows_checked":gate_rows_checked,
            "symbolic_floor_area_total_m2":total_floor_area,
            "candidate_ids":candidate_ids.into_iter().collect::<Vec<_>>(),
            "all_hard_constraint_rows_unknown":true,
            "cost_state":"NOT_ESTIMATED",
            "thermal_state":"NOT_SIMULATED",
            "candidate_ranking":"UNRANKED_PARETO_CANDIDATES",
            "professional_and_authority_approval":"NOT_GRANTED",
            "claim_ceiling":"SYNTHETIC_SYMBOLIC_ROOM_PROGRAM_ONLY"
        }))
    } else {
        Err(errors)
    }
}

fn load_input(path: Option<String>, embedded: &str, label: &str) -> Result<Value, String> {
    match path {
        Some(path) => {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("could not read {label} at {path}: {error}"))?;
            parse_json(&content, label)
        }
        None => parse_json(embedded, label),
    }
}

fn run(args: &[String]) -> Result<Value, String> {
    let mut brief_path = None;
    let mut candidate_path = None;
    let mut manifest_path = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--brief" => {
                index += 1;
                brief_path = args.get(index).cloned();
            }
            "--candidates" => {
                index += 1;
                candidate_path = args.get(index).cloned();
            }
            "--manifest" => {
                index += 1;
                manifest_path = args.get(index).cloned();
            }
            "--self-test" => {}
            "--help" | "-h" => {
                return Ok(json!({
                    "usage":"home-freedom-qualifier [--brief PATH] [--candidates PATH] [--manifest PATH] [--self-test]",
                    "default":"embedded synthetic fixture",
                    "claim_ceiling":"synthetic screening contract only"
                }));
            }
            unknown => return Err(format!("unknown argument {unknown}")),
        }
        index += 1;
    }

    let brief = load_input(brief_path, BRIEF_FIXTURE, "resident brief")?;
    let candidates = load_input(candidate_path, CANDIDATE_FIXTURE, "candidate set")?;
    let manifest = load_input(manifest_path, MANIFEST_FIXTURE, "benchmark manifest")?;
    let brief_schema = parse_json(BRIEF_SCHEMA, "brief schema")?;
    let candidate_schema = parse_json(CANDIDATE_SCHEMA, "candidate schema")?;
    let manifest_schema = parse_json(MANIFEST_SCHEMA, "manifest schema")?;

    validate(
        &brief,
        &candidates,
        &manifest,
        &brief_schema,
        &candidate_schema,
        &manifest_schema,
    )
    .map_err(|errors| format!("HOME_FREEDOM_SCREEN_FAIL: {}", errors.join("; ")))
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            if text_at(&report, "audit_state") == Some("HOME_FREEDOM_SCREEN_PASS") {
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

    fn fixture() -> (Value, Value, Value, Value, Value, Value) {
        (
            parse_json(BRIEF_FIXTURE, "brief").expect("brief JSON"),
            parse_json(CANDIDATE_FIXTURE, "candidates").expect("candidate JSON"),
            parse_json(MANIFEST_FIXTURE, "manifest").expect("manifest JSON"),
            parse_json(BRIEF_SCHEMA, "brief schema").expect("brief schema JSON"),
            parse_json(CANDIDATE_SCHEMA, "candidate schema").expect("candidate schema JSON"),
            parse_json(MANIFEST_SCHEMA, "manifest schema").expect("manifest schema JSON"),
        )
    }

    #[test]
    fn paired_synthetic_case_is_admissible_at_screening_only_maturity() {
        let (brief, candidates, manifest, bs, cs, ms) = fixture();
        let report = validate(&brief, &candidates, &manifest, &bs, &cs, &ms)
            .expect("synthetic candidate set must qualify");
        assert_eq!(report["candidate_count"], 2);
        assert_eq!(report["hard_constraint_count"], 5);
        assert_eq!(report["gate_rows_checked"], 20);
        assert_eq!(report["symbolic_floor_area_total_m2"], 160.0);
        assert_eq!(report["candidate_ranking"], "UNRANKED_PARETO_CANDIDATES");
    }

    #[test]
    fn brief_id_mismatch_is_rejected() {
        let (brief, mut candidates, manifest, bs, cs, ms) = fixture();
        candidates["brief_id"] = json!("SOME-OTHER-BRIEF");
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }

    #[test]
    fn omitted_non_negotiable_constraint_gate_is_rejected() {
        let (brief, mut candidates, manifest, bs, cs, ms) = fixture();
        candidates["candidates"][0]["hard_gate_results"]
            .as_array_mut()
            .unwrap()
            .retain(|gate| gate["gate_id"] != "C_ACCESS_01");
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }

    #[test]
    fn unsupported_pass_on_symbolic_candidate_is_rejected() {
        let (brief, mut candidates, manifest, bs, cs, ms) = fixture();
        candidates["candidates"][0]["hard_gate_results"][0]["state"] = json!("PASS");
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }

    #[test]
    fn unknown_cost_cannot_be_silently_zero_filled() {
        let (brief, mut candidates, manifest, bs, cs, ms) = fixture();
        candidates["candidates"][0]["cost_state"]["low_estimate"] = json!(0.0);
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }

    #[test]
    fn room_program_area_disagreement_is_rejected() {
        let (brief, mut candidates, manifest, bs, cs, ms) = fixture();
        candidates["candidates"][1]["room_program"][0]["target_area_m2"] = json!(99.0);
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }

    #[test]
    fn unknown_archetype_not_in_manifest_is_rejected() {
        let (brief, mut candidates, manifest, bs, cs, ms) = fixture();
        candidates["candidates"][0]["archetype_id"] = json!("MAGIC_UNREGISTERED_HOME");
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }

    #[test]
    fn scalar_winner_cannot_replace_resident_choice() {
        let (brief, mut candidates, manifest, bs, cs, ms) = fixture();
        candidates["comparison_policy"]["scalar_score"] = json!(0.99);
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }

    #[test]
    fn direct_identifiers_are_not_allowed_in_public_fixture() {
        let (mut brief, candidates, manifest, bs, cs, ms) = fixture();
        brief["privacy"]["contains_direct_identifiers"] = json!(true);
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }

    #[test]
    fn a_future_extension_is_not_promoted_without_requalification() {
        let (brief, mut candidates, manifest, bs, cs, ms) = fixture();
        candidates["candidates"][0]["future_change_state"]["state"] = json!("PROFESSIONALLY_REVIEWED");
        assert!(validate(&brief, &candidates, &manifest, &bs, &cs, &ms).is_err());
    }
}
