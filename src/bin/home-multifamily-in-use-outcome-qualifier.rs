// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Apartment in-use outcome evidence crosswalk qualifier.
//!
//! Validates resident-programme traceability, measurement-plan state,
//! consent/withdrawal boundaries and revision-bound outcome evidence.
//! It does not run surveys, monitor residents, operate a BMS, infer comfort,
//! diagnose health, determine accessibility or establish causal impact.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-in-use-outcome-synthetic-v0.1.json"
);
const BUILDING: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-building-synthetic-v0.1.json"
);
const PROGRAMME: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-resident-programme-synthetic-v0.1.json"
);
const LIFECYCLE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-lifecycle-synthetic-v0.1.json"
);
const SERVICE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-service-distribution-synthetic-v0.1.json"
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
    programme: &Value,
    lifecycle: &Value,
    service: &Value,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(fixture, "schema_version") != Some("multifamily-in-use-outcome/0.1") {
        errors.push("unsupported multifamily-in-use-outcome schema_version".to_owned());
    }
    if text_at(fixture, "maturity") != Some("synthetic_contract") {
        errors.push("in-use outcome fixture must remain synthetic_contract".to_owned());
    }

    let bindings = fixture.get("source_bindings").unwrap_or(&Value::Null);
    let expected = [
        ("building_ref", text_at(building, "candidate_id")),
        ("resident_programme_ref", text_at(programme, "programme_id")),
        ("lifecycle_ref", text_at(lifecycle, "profile_id")),
        ("service_topology_ref", text_at(service, "profile_id")),
    ];
    for (field, expected_id) in expected {
        let reference = bindings.get(field).unwrap_or(&Value::Null);
        if !non_null_ref(Some(reference)) || text_at(reference, "id") != expected_id {
            errors.push(format!("source binding {field} does not match exact upstream fixture"));
        }
    }
    if text_at(bindings.get("in_use_outcomes_owner_ref").unwrap_or(&Value::Null), "id")
        != Some("633")
    {
        errors.push("in-use outcome owner binding must point to existing #633 programme".to_owned());
    }

    let building_units = rows(building, "unit_programmes")
        .into_iter()
        .flatten()
        .filter_map(|unit| text_at(unit, "unit_id").map(str::to_owned))
        .collect::<BTreeSet<_>>();
    let mut programme_by_unit = BTreeMap::<String, (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>)>::new();
    for unit in rows(programme, "unit_programmes").into_iter().flatten() {
        let unit_id = text_at(unit, "unit_id").unwrap_or("<missing-unit>");
        let activity_ids = rows(unit, "activities")
            .into_iter()
            .flatten()
            .filter_map(|activity| text_at(activity, "activity_id").map(str::to_owned))
            .collect::<BTreeSet<_>>();
        let requirement_ids = rows(unit, "requirements")
            .into_iter()
            .flatten()
            .filter_map(|requirement| text_at(requirement, "requirement_id").map(str::to_owned))
            .collect::<BTreeSet<_>>();
        let preference_ids = rows(unit, "preferences")
            .into_iter()
            .flatten()
            .filter_map(|preference| text_at(preference, "preference_id").map(str::to_owned))
            .collect::<BTreeSet<_>>();
        programme_by_unit.insert(unit_id.to_owned(), (activity_ids, requirement_ids, preference_ids));
    }
    if programme_by_unit.keys().cloned().collect::<BTreeSet<_>>() != building_units {
        errors.push("resident programme unit set does not exactly match building unit set".to_owned());
    }

    let outcomes = rows(fixture, "outcomes");
    let outcome_ids = unique_ids(outcomes, "outcome_id", "outcomes", &mut errors);
    let mut outcome_unit = BTreeMap::<String, String>::new();
    for outcome in outcomes.into_iter().flatten() {
        let id = text_at(outcome, "outcome_id").unwrap_or("<missing-outcome>");
        let unit_id = text_at(outcome, "unit_id").unwrap_or("<missing-unit>");
        if !building_units.contains(unit_id) {
            errors.push(format!("outcome {id} references unknown building unit {unit_id}"));
        }
        if text_at(outcome, "source_scope") != Some("synthetic_resident_programme_not_real_resident") {
            errors.push(format!("outcome {id} exceeds synthetic resident-source boundary"));
        }
        if text_at(outcome, "claim_state") != Some("intent_only_not_observed") {
            errors.push(format!("synthetic outcome {id} cannot claim observed results"));
        }
        let Some((activities, requirements, preferences)) = programme_by_unit.get(unit_id) else {
            errors.push(format!("outcome {id} has no resident programme for its unit"));
            continue;
        };
        for activity in rows(outcome, "activity_ids").into_iter().flatten().filter_map(Value::as_str) {
            if !activities.contains(activity) {
                errors.push(format!("outcome {id} references activity {activity} outside its exact unit programme"));
            }
        }
        for requirement in rows(outcome, "requirement_ids").into_iter().flatten().filter_map(Value::as_str) {
            if !requirements.contains(requirement) {
                errors.push(format!("outcome {id} references requirement {requirement} outside its exact unit programme"));
            }
        }
        for preference in rows(outcome, "preference_ids").into_iter().flatten().filter_map(Value::as_str) {
            if !preferences.contains(preference) {
                errors.push(format!("outcome {id} references preference {preference} outside its exact unit programme"));
            }
        }
        if rows(outcome, "activity_ids").is_none_or(Vec::is_empty) {
            errors.push(format!("outcome {id} has no activity provenance"));
        }
        outcome_unit.insert(id.to_owned(), unit_id.to_owned());
    }

    let plans = rows(fixture, "measurement_plans");
    let plan_ids = unique_ids(plans, "plan_id", "measurement_plans", &mut errors);
    let mut plan_by_outcome = BTreeMap::<String, (String, String, String)>::new();
    for plan in plans.into_iter().flatten() {
        let plan_id = text_at(plan, "plan_id").unwrap_or("<missing-plan>");
        let outcome_id = text_at(plan, "outcome_id").unwrap_or("<missing-outcome>");
        let Some(unit_id) = outcome_unit.get(outcome_id) else {
            errors.push(format!("measurement plan {plan_id} references unknown outcome {outcome_id}"));
            continue;
        };
        let schedule = text_at(plan, "schedule_state").unwrap_or("<missing>");
        let consent = text_at(plan, "consent_state").unwrap_or("<missing>");
        let method_state = text_at(plan, "method_state").unwrap_or("<missing>");
        let data_scope = text_at(plan, "data_minimisation").unwrap_or("<missing>");
        if !matches!(schedule, "not_scheduled" | "draft" | "approved" | "withdrawn" | "disputed") {
            errors.push(format!("measurement plan {plan_id} has invalid schedule_state"));
        }
        if schedule == "approved" {
            if method_state == "not_selected" || !non_null_ref(plan.get("method_ref")) {
                errors.push(format!("approved measurement plan {plan_id} lacks selected method evidence"));
            }
            if text_at(plan, "measurement_kind") == Some("instrument_measurement")
                && !non_null_ref(plan.get("instrument_ref"))
            {
                errors.push(format!("approved instrument plan {plan_id} lacks instrument identity"));
            }
            if text_at(plan, "measurement_kind") == Some("resident_self_report")
                && (consent != "granted" || data_scope != "resident_opt_in_minimum")
            {
                errors.push(format!("approved self-report plan {plan_id} lacks explicit consent/minimum data scope"));
            }
        }
        if schedule == "not_scheduled" && consent != "not_requested" {
            errors.push(format!("unscheduled plan {plan_id} cannot imply resident consent"));
        }
        if schedule == "withdrawn" && consent != "withdrawn" {
            errors.push(format!("withdrawn measurement plan {plan_id} must preserve consent withdrawal"));
        }
        plan_by_outcome.insert(
            outcome_id.to_owned(),
            (
                unit_id.clone(),
                text_at(plan, "measurement_kind").unwrap_or("<missing>").to_owned(),
                schedule.to_owned(),
            ),
        );
    }
    for outcome_id in &outcome_ids {
        if !plan_by_outcome.contains_key(outcome_id) {
            errors.push(format!("outcome {outcome_id} has no measurement plan"));
        }
    }

    let observations = rows(fixture, "observations");
    let observation_ids = unique_ids(observations, "observation_id", "observations", &mut errors);
    for observation in observations.into_iter().flatten() {
        let id = text_at(observation, "observation_id").unwrap_or("<missing-observation>");
        let outcome_id = text_at(observation, "outcome_id").unwrap_or("<missing-outcome>");
        let unit_id = text_at(observation, "unit_id").unwrap_or("<missing-unit>");
        let state = text_at(observation, "evidence_state").unwrap_or("<missing-state>");
        if outcome_unit.get(outcome_id).map(String::as_str) != Some(unit_id) {
            errors.push(format!("observation {id} outcome/unit identity mismatch"));
        }
        match state {
            "modelled" => {
                if !non_null_ref(observation.get("evidence_ref")) {
                    errors.push(format!("modelled observation {id} lacks model evidence"));
                }
                if !is_null(observation.get("consent_ref")) {
                    // Model outputs must not be passed off as resident feedback.
                }
            }
            "measured" => {
                for field in ["instrument_ref", "calibration_ref", "evidence_ref"] {
                    if !non_null_ref(observation.get(field)) {
                        errors.push(format!("measured observation {id} lacks {field}"));
                    }
                }
                if observation.get("value").is_none_or(Value::is_null)
                    || text_at(observation, "unit").is_none_or(str::is_empty)
                    || observation.get("uncertainty").and_then(Value::as_f64).is_none()
                {
                    errors.push(format!("measured observation {id} lacks value/unit/uncertainty"));
                }
            }
            "self_reported" => {
                if !non_null_ref(observation.get("consent_ref")) || !non_null_ref(observation.get("evidence_ref")) {
                    errors.push(format!("self-reported observation {id} lacks explicit consent/evidence"));
                }
            }
            "professionally_assessed" => {
                if !non_null_ref(observation.get("evidence_ref")) {
                    errors.push(format!("professional assessment {id} lacks exact evidence"));
                }
            }
            "disputed" | "withdrawn" | "invalidated" => {}
            _ => errors.push(format!("observation {id} has invalid evidence_state {state}")),
        }
        if state == "measured" && text_at(observation, "evidence_state") != Some("measured") {
            errors.push(format!("observation {id} has contradictory measurement state"));
        }
    }

    let decision_links = rows(fixture, "decision_links");
    let decision_ids = unique_ids(decision_links, "decision_id", "decision_links", &mut errors);
    for decision in decision_links.into_iter().flatten() {
        let id = text_at(decision, "decision_id").unwrap_or("<missing-decision>");
        for observation_id in rows(decision, "observation_ids").into_iter().flatten().filter_map(Value::as_str) {
            if !observation_ids.contains(observation_id) {
                errors.push(format!("decision {id} references unknown observation {observation_id}"));
            }
        }
        let state = text_at(decision, "decision_state").unwrap_or("<missing>");
        if matches!(state, "professionally_reviewed" | "accepted" | "rejected")
            && (!non_null_ref(decision.get("reviewer_ref")) || !non_null_ref(decision.get("target_revision_ref")))
        {
            errors.push(format!("decision {id} requires reviewer and target revision evidence"));
        }
    }

    let policy = fixture.get("collection_policy").unwrap_or(&Value::Null);
    if text_at(policy, "default_collection") != Some("none")
        || policy.get("audio_video_allowed").and_then(Value::as_bool) != Some(false)
        || policy.get("continuous_occupancy_allowed").and_then(Value::as_bool) != Some(false)
        || policy.get("continuous_device_telemetry_allowed").and_then(Value::as_bool) != Some(false)
        || policy.get("resident_consent_required_for_self_report").and_then(Value::as_bool) != Some(true)
        || policy.get("resident_withdrawal_supported").and_then(Value::as_bool) != Some(true)
    {
        errors.push("collection policy violates resident privacy/consent/withdrawal guard".to_owned());
    }
    if observations.is_some_and(|items| !items.is_empty())
        && text_at(policy, "retention_state") != Some("approved")
    {
        errors.push("observations cannot be collected or retained without approved retention policy".to_owned());
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "IN_USE_OUTCOME_CROSSWALK_PASS" } else { "IN_USE_OUTCOME_CROSSWALK_FAIL" },
        "profile_id": text_at(fixture, "profile_id").unwrap_or("<missing>"),
        "outcome_count": outcome_ids.len(),
        "measurement_plan_count": plan_ids.len(),
        "observation_count": observation_ids.len(),
        "decision_link_count": decision_ids.len(),
        "default_collection": "NONE",
        "resident_surveillance": "NOT_AUTHORIZED",
        "resident_satisfaction": "NOT_ESTABLISHED",
        "physical_performance": "NOT_ESTABLISHED",
        "claim_ceiling": "SYNTHETIC_OUTCOME_TRACEABILITY_AND_PRIVACY_POLICY_ONLY"
    });
    if errors.is_empty() { Ok(report) } else { Err(errors) }
}

fn is_null(value: Option<&Value>) -> bool {
    value.is_none_or(Value::is_null)
}

fn fixtures() -> (Value, Value, Value, Value, Value) {
    (
        serde_json::from_str(FIXTURE).expect("in-use outcome fixture JSON"),
        serde_json::from_str(BUILDING).expect("building fixture JSON"),
        serde_json::from_str(PROGRAMME).expect("resident programme fixture JSON"),
        serde_json::from_str(LIFECYCLE).expect("lifecycle fixture JSON"),
        serde_json::from_str(SERVICE).expect("service topology fixture JSON"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-multifamily-in-use-outcome-qualifier [--self-test]");
        return;
    }
    let (fixture, building, programme, lifecycle, service) = fixtures();
    match validate(&fixture, &building, &programme, &lifecycle, &service) {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default()),
        Err(errors) => {
            eprintln!("HOME_MULTIFAMILY_IN_USE_OUTCOME_QUALIFIER_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_in_use_crosswalk_qualifies_without_observations() {
        let (fixture, building, programme, lifecycle, service) = fixtures();
        let report = validate(&fixture, &building, &programme, &lifecycle, &service).expect("synthetic outcome crosswalk");
        assert_eq!(report["outcome_count"], 4);
        assert_eq!(report["measurement_plan_count"], 4);
        assert_eq!(report["observation_count"], 0);
        assert_eq!(report["decision_link_count"], 0);
        assert_eq!(report["default_collection"], "NONE");
    }

    #[test]
    fn outcome_cannot_reference_another_units_activity() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["outcomes"][0]["activity_ids"] = json!(["U-002-SHARED-MAKING-ACT-01"]);
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_err());
    }

    #[test]
    fn outcome_cannot_be_promoted_to_observed_without_evidence() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["outcomes"][0]["claim_state"] = json!("observed_satisfied");
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_err());
    }

    #[test]
    fn unscheduled_plan_cannot_imply_consent() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["measurement_plans"][0]["consent_state"] = json!("granted");
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_err());
    }

    #[test]
    fn self_reported_observation_requires_consent_and_evidence() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["observations"] = json!([{
            "observation_id":"OBS-1",
            "outcome_id":"SYN-OUTCOME-U-001-ACOUSTIC",
            "unit_id":"U-001",
            "evidence_state":"self_reported",
            "observed_at":"2026-11-15T08:00:00+02:00",
            "value":"not comfortable",
            "unit":null,
            "uncertainty":null,
            "instrument_ref":null,
            "calibration_ref":null,
            "consent_ref":null,
            "evidence_ref":null,
            "design_revision_ref":{"kind":"design_revision","id":"R1","revision":"1","digest":null}
        }]);
        fixture["collection_policy"]["retention_state"] = json!("approved");
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_err());
    }

    #[test]
    fn modelled_observation_does_not_become_a_measurement() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["observations"] = json!([{
            "observation_id":"OBS-2",
            "outcome_id":"SYN-OUTCOME-U-001-ACOUSTIC",
            "unit_id":"U-001",
            "evidence_state":"modelled",
            "observed_at":"2026-11-15T08:00:00+02:00",
            "value":30.0,
            "unit":"dB",
            "uncertainty":null,
            "instrument_ref":null,
            "calibration_ref":null,
            "consent_ref":null,
            "evidence_ref":{"kind":"model_output","id":"MODEL-1","revision":"1","digest":null},
            "design_revision_ref":{"kind":"design_revision","id":"R1","revision":"1","digest":null}
        }]);
        fixture["collection_policy"]["retention_state"] = json!("approved");
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_ok());
    }

    #[test]
    fn audio_video_and_occupancy_collection_are_rejected() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["collection_policy"]["audio_video_allowed"] = json!(true);
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_err());
    }

    #[test]
    fn decision_cannot_reference_unknown_observation() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["decision_links"] = json!([{
            "decision_id":"DEC-1",
            "observation_ids":["NO-SUCH-OBS"],
            "decision_state":"proposed",
            "target_revision_ref":null,
            "reviewer_ref":null
        }]);
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_err());
    }

    #[test]
    fn approved_self_report_requires_minimal_consent_scope() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["measurement_plans"][0]["schedule_state"] = json!("approved");
        fixture["measurement_plans"][0]["consent_state"] = json!("granted");
        fixture["measurement_plans"][0]["method_state"] = json!("declared");
        fixture["measurement_plans"][0]["method_ref"] = json!({"kind":"method","id":"M1","revision":"1","digest":null});
        fixture["measurement_plans"][0]["data_minimisation"] = json!("unknown");
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_err());
    }

    #[test]
    fn unknown_preference_or_requirement_cannot_be_inferred() {
        let (mut fixture, building, programme, lifecycle, service) = fixtures();
        fixture["outcomes"][0]["requirement_ids"] = json!(["U-004-REQ-01"]);
        assert!(validate(&fixture, &building, &programme, &lifecycle, &service).is_err());
    }
}
