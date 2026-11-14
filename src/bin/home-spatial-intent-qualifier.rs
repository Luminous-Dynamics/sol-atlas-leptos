// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const BRIEF: &str = include_str!(
    "../../docs/research/home-freedom/HOME_DESIGN_BRIEF_SYNTHETIC_EXAMPLE_V0.1.json"
);
const GRAPH: &str = include_str!(
    "../../docs/research/home-spatial-intent/HOME_SPATIAL_INTENT_GRAPH_SYNTHETIC_FIXTURE_V0.1.json"
);
const GRAPH_SCHEMA: &str = include_str!(
    "../../docs/research/home-spatial-intent/HOME_SPATIAL_INTENT_GRAPH_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn parse_json(source: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(source)
        .map_err(|error| format!("{label} JSON parse failed: {error}"))
}

fn activity_ids(brief: &Value) -> Result<BTreeSet<String>, Vec<String>> {
    let mut errors = Vec::new();
    let mut ids = BTreeSet::new();
    let activities = array_at(brief.get("intent").unwrap_or(&Value::Null), "activities");
    if let Some(activities) = activities {
        for activity in activities {
            let id = text_at(activity, "activity_id")
                .unwrap_or("<missing-activity-id>")
                .to_owned();
            if !ids.insert(id.clone()) {
                errors.push(format!("duplicate activity ID {id}"));
            }
        }
    } else {
        errors.push("resident brief has no intent.activities array".to_owned());
    }
    if errors.is_empty() {
        Ok(ids)
    } else {
        Err(errors)
    }
}

fn relationship_key(value: &Value) -> Option<(String, String, String)> {
    Some((
        text_at(value, "from_activity_id")?.to_owned(),
        text_at(value, "to_activity_id")?.to_owned(),
        text_at(value, "relationship")?.to_owned(),
    ))
}

fn validate_candidate(
    candidate: &Value,
    brief_activity_ids: &BTreeSet<String>,
    brief_relationships: &BTreeSet<(String, String, String)>,
    brief_constraint_ids: &BTreeSet<String>,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let candidate_id = text_at(candidate, "candidate_id")
        .unwrap_or("<missing-candidate-id>")
        .to_owned();

    if text_at(candidate, "topology_state") != Some("SYMBOLIC_TOPOLOGY_NOT_GEOMETRY")
        || text_at(candidate, "verification_state") != Some("SYMBOLIC_STRUCTURE_ONLY")
    {
        errors.push(format!(
            "{candidate_id}: candidate is promoted above symbolic topology maturity"
        ));
    }

    let mut space_ids = BTreeSet::new();
    let mut internal_area = 0.0_f64;
    let mut outdoor_area = 0.0_f64;
    if let Some(spaces) = array_at(candidate, "space_nodes") {
        for space in spaces {
            let id = text_at(space, "space_id")
                .unwrap_or("<missing-space-id>")
                .to_owned();
            if !space_ids.insert(id.clone()) {
                errors.push(format!("{candidate_id}: duplicate space ID {id}"));
            }
            let Some(area) = space.get("area_m2").and_then(Value::as_f64) else {
                errors.push(format!("{candidate_id}: {id} has no numeric area"));
                continue;
            };
            if !area.is_finite() || area < 0.0 {
                errors.push(format!("{candidate_id}: {id} has invalid area"));
            }
            match text_at(space, "space_type") {
                Some("INDOOR") => internal_area += area,
                Some("OUTDOOR") => outdoor_area += area,
                _ => errors.push(format!("{candidate_id}: {id} has invalid space_type")),
            }
        }
    } else {
        errors.push(format!("{candidate_id}: space_nodes array is missing"));
    }

    let declared_internal = candidate
        .get("internal_floor_area_m2")
        .and_then(Value::as_f64)
        .unwrap_or(f64::NAN);
    let declared_outdoor = candidate
        .get("outdoor_area_m2")
        .and_then(Value::as_f64)
        .unwrap_or(f64::NAN);
    if !declared_internal.is_finite()
        || (internal_area - declared_internal).abs() > 1e-9
    {
        errors.push(format!(
            "{candidate_id}: indoor area sum {internal_area} does not match {declared_internal}"
        ));
    }
    if !declared_outdoor.is_finite()
        || (outdoor_area - declared_outdoor).abs() > 1e-9
    {
        errors.push(format!(
            "{candidate_id}: outdoor area sum {outdoor_area} does not match {declared_outdoor}"
        ));
    }

    let mut assignments = BTreeMap::<String, String>::new();
    if let Some(rows) = array_at(candidate, "activity_assignments") {
        for row in rows {
            let activity_id = text_at(row, "activity_id").unwrap_or("<missing>").to_owned();
            let space_id = text_at(row, "space_id").unwrap_or("<missing>").to_owned();
            if !brief_activity_ids.contains(&activity_id) {
                errors.push(format!(
                    "{candidate_id}: assignment references unknown activity {activity_id}"
                ));
            }
            if !space_ids.contains(&space_id) {
                errors.push(format!(
                    "{candidate_id}: assignment references unknown space {space_id}"
                ));
            }
            if assignments.insert(activity_id.clone(), space_id).is_some() {
                errors.push(format!(
                    "{candidate_id}: activity {activity_id} has multiple assignments"
                ));
            }
        }
    } else {
        errors.push(format!("{candidate_id}: activity_assignments array is missing"));
    }
    for activity_id in brief_activity_ids {
        if !assignments.contains_key(activity_id) {
            errors.push(format!(
                "{candidate_id}: resident activity {activity_id} is unassigned"
            ));
        }
    }

    let mut relationship_rows = BTreeSet::new();
    if let Some(rows) = array_at(candidate, "relationships") {
        for row in rows {
            let Some(key) = relationship_key(row) else {
                errors.push(format!(
                    "{candidate_id}: relationship row is missing endpoint or type"
                ));
                continue;
            };
            if !brief_relationships.contains(&key) {
                errors.push(format!(
                    "{candidate_id}: relationship {:?} is not declared in the resident brief",
                    key
                ));
            }
            if !relationship_rows.insert(key.clone()) {
                errors.push(format!(
                    "{candidate_id}: duplicate relationship {:?}",
                    key
                ));
            }
            if assignments.get(&key.0).map(String::as_str)
                != text_at(row, "from_space_id")
            {
                errors.push(format!(
                    "{candidate_id}: relationship {:?} has wrong source-space binding",
                    key
                ));
            }
            if assignments.get(&key.1).map(String::as_str)
                != text_at(row, "to_space_id")
            {
                errors.push(format!(
                    "{candidate_id}: relationship {:?} has wrong target-space binding",
                    key
                ));
            }
            if text_at(row, "verification_state") != Some("UNKNOWN")
                || !row.get("evidence_ref").is_some_and(Value::is_null)
            {
                errors.push(format!(
                    "{candidate_id}: symbolic relationship {:?} cannot be physically verified",
                    key
                ));
            }
        }
    } else {
        errors.push(format!("{candidate_id}: relationships array is missing"));
    }
    if relationship_rows != *brief_relationships {
        errors.push(format!(
            "{candidate_id}: candidate does not preserve every brief relationship"
        ));
    }

    if let Some(routes) = array_at(candidate, "access_routes") {
        let mut route_ids = BTreeSet::new();
        for route in routes {
            let id = text_at(route, "route_id").unwrap_or("<missing-route-id>");
            if !route_ids.insert(id.to_owned()) {
                errors.push(format!("{candidate_id}: duplicate route ID {id}"));
            }
            for key in ["from_space_id", "to_space_id"] {
                let space_id = text_at(route, key).unwrap_or("<missing-space-id>");
                if !space_ids.contains(space_id) {
                    errors.push(format!(
                        "{candidate_id}: route {id} references unknown space {space_id}"
                    ));
                }
            }
            if text_at(route, "verification_state") != Some("UNKNOWN")
                || !route.get("evidence_ref").is_some_and(Value::is_null)
            {
                errors.push(format!(
                    "{candidate_id}: symbolic access route {id} cannot be physically verified"
                ));
            }
        }
    } else {
        errors.push(format!("{candidate_id}: access_routes array is missing"));
    }

    let mut bound_constraints = BTreeSet::new();
    if let Some(bindings) = array_at(candidate, "hard_constraint_bindings") {
        for binding in bindings {
            let id = text_at(binding, "constraint_id").unwrap_or("<missing>").to_owned();
            if !brief_constraint_ids.contains(&id) {
                errors.push(format!(
                    "{candidate_id}: unknown hard-constraint binding {id}"
                ));
            }
            if !bound_constraints.insert(id.clone()) {
                errors.push(format!(
                    "{candidate_id}: duplicate hard-constraint binding {id}"
                ));
            }
            if binding.get("represented").and_then(Value::as_bool) != Some(true) {
                errors.push(format!(
                    "{candidate_id}: hard constraint {id} is not represented"
                ));
            }
            if text_at(binding, "verification_state") != Some("UNKNOWN")
                || !binding.get("evidence_ref").is_some_and(Value::is_null)
            {
                errors.push(format!(
                    "{candidate_id}: hard constraint {id} cannot be physically verified here"
                ));
            }
        }
    } else {
        errors.push(format!(
            "{candidate_id}: hard_constraint_bindings array is missing"
        ));
    }
    if bound_constraints != *brief_constraint_ids {
        errors.push(format!(
            "{candidate_id}: candidate does not preserve every hard constraint"
        ));
    }

    if errors.is_empty() {
        Ok(json!({
            "candidate_id":candidate_id,
            "space_count":space_ids.len(),
            "assignment_count":assignments.len(),
            "relationship_count":relationship_rows.len(),
            "access_route_count":array_at(candidate,"access_routes").map_or(0,Vec::len),
            "hard_constraint_count":bound_constraints.len(),
            "internal_floor_area_m2":internal_area,
            "outdoor_area_m2":outdoor_area,
            "physical_verification":"UNKNOWN"
        }))
    } else {
        Err(errors)
    }
}

fn validate(brief: &Value, graph: &Value, schema: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("spatial graph schema must declare Draft 2020-12".to_owned());
    }
    if text_at(brief, "brief_id") != text_at(graph, "brief_id") {
        errors.push("spatial graph is not bound to the exact resident brief ID".to_owned());
    }
    if text_at(graph, "study_id") != Some("HOME-SPATIAL-INTENT-GRAPH-001")
        || text_at(graph, "maturity") != Some("SYNTHETIC_SYMBOLIC_GRAPH_ONLY")
        || text_at(graph, "execution_status") != Some("NOT_GEOMETRICALLY_VERIFIED")
    {
        errors.push("spatial graph identity or maturity is invalid".to_owned());
    }

    let brief_activities = match activity_ids(brief) {
        Ok(ids) => ids,
        Err(activity_errors) => {
            errors.extend(activity_errors);
            BTreeSet::new()
        }
    };
    let graph_activities: BTreeSet<String> = array_at(graph, "activity_nodes")
        .into_iter()
        .flatten()
        .filter_map(|activity| text_at(activity, "activity_id"))
        .map(str::to_owned)
        .collect();
    if brief_activities != graph_activities {
        errors.push("activity nodes do not exactly match the resident brief".to_owned());
    }

    let mut brief_relationships = BTreeSet::new();
    if let Some(rows) = array_at(
        brief.get("intent").unwrap_or(&Value::Null),
        "space_relationships",
    ) {
        for row in rows {
            match relationship_key(row) {
                Some(key) => {
                    brief_relationships.insert(key);
                }
                None => errors.push("resident brief relationship is incomplete".to_owned()),
            }
        }
    }
    let mut brief_constraint_ids = BTreeSet::new();
    if let Some(constraints) = array_at(brief, "constraints") {
        for constraint in constraints {
            if let Some(id) = text_at(constraint, "constraint_id") {
                brief_constraint_ids.insert(id.to_owned());
            }
        }
    }

    let policy = graph.get("comparison_policy").unwrap_or(&Value::Null);
    if text_at(policy, "ranking_state") != Some("UNRANKED")
        || !policy.get("scalar_score").is_some_and(Value::is_null)
        || policy.get("resident_choice_required").and_then(Value::as_bool) != Some(true)
    {
        errors.push("spatial candidates must remain unranked with resident choice".to_owned());
    }

    let candidates = array_at(graph, "candidate_graphs").cloned().unwrap_or_default();
    if candidates.len() < 2 {
        errors.push("at least two candidate topologies are required".to_owned());
    }
    let mut candidate_ids = BTreeSet::new();
    let mut reports = Vec::new();
    for candidate in &candidates {
        let id = text_at(candidate, "candidate_id").unwrap_or("<missing>").to_owned();
        if !candidate_ids.insert(id.clone()) {
            errors.push(format!("duplicate candidate graph ID {id}"));
            continue;
        }
        match validate_candidate(
            candidate,
            &brief_activities,
            &brief_relationships,
            &brief_constraint_ids,
        ) {
            Ok(report) => reports.push(report),
            Err(candidate_errors) => {
                errors.extend(candidate_errors.into_iter().map(|e| format!("{id}: {e}")));
            }
        }
    }

    if errors.is_empty() {
        Ok(json!({
            "audit_state":"HOME_SPATIAL_INTENT_GRAPH_PASS",
            "study_id":"HOME-SPATIAL-INTENT-GRAPH-001",
            "activity_count":brief_activities.len(),
            "relationship_requirement_count":brief_relationships.len(),
            "hard_constraint_count":brief_constraint_ids.len(),
            "candidate_count":candidate_ids.len(),
            "candidates":reports,
            "resident_choice_required":true,
            "physical_verification":"UNKNOWN",
            "claim_ceiling":"SYMBOLIC_SPATIAL_INTENT_GRAPH_ONLY"
        }))
    } else {
        Err(errors)
    }
}

fn fixture() -> (Value, Value, Value) {
    (
        parse_json(BRIEF, "resident brief").expect("brief JSON"),
        parse_json(GRAPH, "spatial graph").expect("graph JSON"),
        parse_json(GRAPH_SCHEMA, "graph schema").expect("schema JSON"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-spatial-intent-qualifier [--self-test]");
        return;
    }
    let (brief, graph, schema) = fixture();
    match validate(&brief, &graph, &schema) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
        }
        Err(errors) => {
            eprintln!("HOME_SPATIAL_INTENT_GRAPH_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paired_symbolic_graphs_preserve_resident_intent() {
        let (brief, graph, schema) = fixture();
        let report = validate(&brief, &graph, &schema)
            .expect("paired symbolic graphs must qualify");
        assert_eq!(report["activity_count"], 6);
        assert_eq!(report["relationship_requirement_count"], 4);
        assert_eq!(report["hard_constraint_count"], 5);
        assert_eq!(report["candidate_count"], 2);
        assert_eq!(report["physical_verification"], "UNKNOWN");
        assert_eq!(report["candidates"][0]["internal_floor_area_m2"], 82.0);
        assert_eq!(report["candidates"][1]["internal_floor_area_m2"], 78.0);
    }

    #[test]
    fn missing_activity_assignment_is_rejected() {
        let (brief, mut graph, schema) = fixture();
        graph["candidate_graphs"][0]["activity_assignments"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["activity_id"] != "ACT_WORK_01");
        assert!(validate(&brief, &graph, &schema).is_err());
    }

    #[test]
    fn orphan_space_reference_is_rejected() {
        let (brief, mut graph, schema) = fixture();
        graph["candidate_graphs"][0]["activity_assignments"][0]["space_id"] =
            json!("SPACE_DOES_NOT_EXIST");
        assert!(validate(&brief, &graph, &schema).is_err());
    }

    #[test]
    fn area_mismatch_is_rejected() {
        let (brief, mut graph, schema) = fixture();
        graph["candidate_graphs"][0]["space_nodes"][0]["area_m2"] = json!(99.0);
        assert!(validate(&brief, &graph, &schema).is_err());
    }

    #[test]
    fn missing_relationship_is_rejected() {
        let (brief, mut graph, schema) = fixture();
        graph["candidate_graphs"][0]["relationships"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(validate(&brief, &graph, &schema).is_err());
    }

    #[test]
    fn relationship_cannot_be_promoted_to_physically_verified() {
        let (brief, mut graph, schema) = fixture();
        graph["candidate_graphs"][0]["relationships"][0]["verification_state"] =
            json!("VERIFIED");
        assert!(validate(&brief, &graph, &schema).is_err());
    }

    #[test]
    fn hard_constraint_cannot_be_removed() {
        let (brief, mut graph, schema) = fixture();
        graph["candidate_graphs"][0]["hard_constraint_bindings"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["constraint_id"] != "C_ACCESS_01");
        assert!(validate(&brief, &graph, &schema).is_err());
    }

    #[test]
    fn access_route_cannot_reference_unknown_space() {
        let (brief, mut graph, schema) = fixture();
        graph["candidate_graphs"][0]["access_routes"][0]["to_space_id"] =
            json!("UNKNOWN_SPACE");
        assert!(validate(&brief, &graph, &schema).is_err());
    }

    #[test]
    fn scalar_winner_cannot_replace_resident_choice() {
        let (brief, mut graph, schema) = fixture();
        graph["comparison_policy"]["scalar_score"] = json!(0.99);
        assert!(validate(&brief, &graph, &schema).is_err());
    }

    #[test]
    fn physical_relationship_states_remain_unknown() {
        let (brief, mut graph, schema) = fixture();
        graph["candidate_graphs"][1]["access_routes"][0]["verification_state"] =
            json!("PASS");
        assert!(validate(&brief, &graph, &schema).is_err());
    }
}
