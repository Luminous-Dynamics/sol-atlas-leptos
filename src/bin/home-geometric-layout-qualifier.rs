// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::{env, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-geometric-layout/HOME_GEOMETRIC_LAYOUT_SYNTHETIC_FIXTURE_V0.1.json"
);
const GRAPH: &str = include_str!(
    "../../docs/research/home-spatial-intent/HOME_SPATIAL_INTENT_GRAPH_SYNTHETIC_FIXTURE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/home-geometric-layout/HOME_GEOMETRIC_LAYOUT_V0.1.schema.json"
);

#[derive(Clone, Debug)]
struct Room {
    id: String,
    space_type: String,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

impl Room {
    fn right(&self) -> Option<i64> {
        self.x.checked_add(self.width)
    }

    fn top(&self) -> Option<i64> {
        self.y.checked_add(self.height)
    }

    fn area_mm2(&self) -> Option<i64> {
        self.width.checked_mul(self.height)
    }
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn int_at(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn parse_json(source: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(source).map_err(|error| format!("{label} JSON parse failed: {error}"))
}

fn pair_key(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_owned(), b.to_owned())
    } else {
        (b.to_owned(), a.to_owned())
    }
}

fn shared_boundary_opening(
    opening: &Value,
    rooms: &BTreeMap<String, Room>,
) -> Result<(String, String), String> {
    let opening_id = text_at(opening, "opening_id").unwrap_or("<missing-opening-id>");
    let from = text_at(opening, "from_space_id")
        .ok_or_else(|| format!("{opening_id}: missing from_space_id"))?;
    let to = text_at(opening, "to_space_id")
        .ok_or_else(|| format!("{opening_id}: missing to_space_id"))?;
    let first = rooms
        .get(from)
        .ok_or_else(|| format!("{opening_id}: unknown room {from}"))?;
    let second = rooms
        .get(to)
        .ok_or_else(|| format!("{opening_id}: unknown room {to}"))?;
    let axis = text_at(opening, "axis").unwrap_or("");
    let coordinate = int_at(opening, "coordinate_mm")
        .ok_or_else(|| format!("{opening_id}: missing coordinate_mm"))?;
    let start = int_at(opening, "span_start_mm")
        .ok_or_else(|| format!("{opening_id}: missing span_start_mm"))?;
    let end = int_at(opening, "span_end_mm")
        .ok_or_else(|| format!("{opening_id}: missing span_end_mm"))?;
    if start < 0 || end <= start {
        return Err(format!("{opening_id}: opening span must have positive length"));
    }

    let on_shared_boundary = match axis {
        "HORIZONTAL" => {
            let first_top = first
                .top()
                .ok_or_else(|| format!("{opening_id}: room coordinate overflow"))?;
            let second_top = second
                .top()
                .ok_or_else(|| format!("{opening_id}: room coordinate overflow"))?;
            let shared_y = (first_top == coordinate && second.y == coordinate)
                || (second_top == coordinate && first.y == coordinate);
            if !shared_y {
                false
            } else {
                let first_right = first
                    .right()
                    .ok_or_else(|| format!("{opening_id}: room coordinate overflow"))?;
                let second_right = second
                    .right()
                    .ok_or_else(|| format!("{opening_id}: room coordinate overflow"))?;
                let shared_start = first.x.max(second.x);
                let shared_end = first_right.min(second_right);
                start >= shared_start && end <= shared_end && end > start
            }
        }
        "VERTICAL" => {
            let first_right = first
                .right()
                .ok_or_else(|| format!("{opening_id}: room coordinate overflow"))?;
            let second_right = second
                .right()
                .ok_or_else(|| format!("{opening_id}: room coordinate overflow"))?;
            let shared_x = (first_right == coordinate && second.x == coordinate)
                || (second_right == coordinate && first.x == coordinate);
            if !shared_x {
                false
            } else {
                let first_top = first
                    .top()
                    .ok_or_else(|| format!("{opening_id}: room coordinate overflow"))?;
                let second_top = second
                    .top()
                    .ok_or_else(|| format!("{opening_id}: room coordinate overflow"))?;
                let shared_start = first.y.max(second.y);
                let shared_end = first_top.min(second_top);
                start >= shared_start && end <= shared_end && end > start
            }
        }
        _ => false,
    };

    if !on_shared_boundary {
        return Err(format!(
            "{opening_id}: opening is not fully contained in the rooms' shared boundary"
        ));
    }
    Ok((from.to_owned(), to.to_owned()))
}

fn route_connected(
    from: &str,
    to: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    if from == to {
        return true;
    }
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    visited.insert(from.to_owned());
    queue.push_back(from.to_owned());
    while let Some(current) = queue.pop_front() {
        if let Some(neighbours) = adjacency.get(&current) {
            for next in neighbours {
                if next == to {
                    return true;
                }
                if visited.insert(next.clone()) {
                    queue.push_back(next.clone());
                }
            }
        }
    }
    false
}

fn validate_candidate(
    candidate: &Value,
    envelope: &Value,
    graph: &Value,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let candidate_id = text_at(candidate, "candidate_id").unwrap_or("<missing-candidate-id>");
    let graph_candidate_id = text_at(candidate, "source_graph_candidate_id").unwrap_or("");
    if text_at(candidate, "geometry_state") != Some("SYNTHETIC_GEOMETRY_NOT_BUILDABLE")
        || text_at(candidate, "physical_verification_state") != Some("UNKNOWN")
    {
        errors.push(format!("{candidate_id}: geometry or physical maturity is overstated"));
    }

    let graph_candidate = array_at(graph, "candidate_graphs")
        .and_then(|items| {
            items
                .iter()
                .find(|item| text_at(item, "candidate_id") == Some(graph_candidate_id))
        });
    let Some(graph_candidate) = graph_candidate else {
        errors.push(format!(
            "{candidate_id}: source graph candidate {graph_candidate_id} is not registered"
        ));
        return Err(errors);
    };
    let Some(source_spaces) = array_at(graph_candidate, "space_nodes") else {
        return Err(vec![format!("{candidate_id}: source graph has no space_nodes")]);
    };
    let mut source_by_id = BTreeMap::<String, &Value>::new();
    for source_space in source_spaces {
        let Some(id) = text_at(source_space, "space_id") else {
            errors.push(format!("{candidate_id}: source graph has a space without an ID"));
            continue;
        };
        if source_by_id.insert(id.to_owned(), source_space).is_some() {
            errors.push(format!("{candidate_id}: source graph repeats space {id}"));
        }
    }

    let envelope_width = int_at(envelope, "width_mm").unwrap_or(-1);
    let envelope_height = int_at(envelope, "height_mm").unwrap_or(-1);
    if envelope_width <= 0 || envelope_height <= 0 {
        errors.push(format!("{candidate_id}: envelope dimensions must be positive"));
    }

    let mut rooms = BTreeMap::<String, Room>::new();
    let mut seen_ids = BTreeSet::new();
    let room_rows = array_at(candidate, "rooms");
    if let Some(room_rows) = room_rows {
        for row in room_rows {
            let id = text_at(row, "space_id").unwrap_or("<missing-space-id>").to_owned();
            if !seen_ids.insert(id.clone()) {
                errors.push(format!("{candidate_id}: duplicate geometric room {id}"));
                continue;
            }
            if int_at(row, "floor_index") != Some(0) {
                errors.push(format!("{candidate_id}: {id} must be on floor index 0 in v0.1"));
                continue;
            }
            let room = Room {
                id: id.clone(),
                space_type: text_at(row, "space_type").unwrap_or("").to_owned(),
                x: int_at(row, "x_mm").unwrap_or(-1),
                y: int_at(row, "y_mm").unwrap_or(-1),
                width: int_at(row, "width_mm").unwrap_or(0),
                height: int_at(row, "height_mm").unwrap_or(0),
            };
            if !matches!(room.space_type.as_str(), "INDOOR" | "OUTDOOR") {
                errors.push(format!("{candidate_id}: {id} has an unknown space_type"));
                continue;
            }
            if room.x < 0 || room.y < 0 || room.width <= 0 || room.height <= 0 {
                errors.push(format!("{candidate_id}: {id} has invalid coordinates or dimensions"));
                continue;
            }
            let Some(right) = room.right() else {
                errors.push(format!("{candidate_id}: {id} horizontal coordinate overflow"));
                continue;
            };
            let Some(top) = room.top() else {
                errors.push(format!("{candidate_id}: {id} vertical coordinate overflow"));
                continue;
            };
            if right > envelope_width || top > envelope_height {
                errors.push(format!("{candidate_id}: {id} is outside the declared envelope"));
            }
            let Some(source) = source_by_id.get(&id) else {
                errors.push(format!("{candidate_id}: geometric room {id} is absent from source graph"));
                continue;
            };
            let source_type = text_at(source, "space_type").unwrap_or("");
            if room.space_type != source_type {
                errors.push(format!("{candidate_id}: {id} space_type differs from source graph"));
            }
            let Some(source_area_m2) = source.get("area_m2").and_then(Value::as_i64) else {
                errors.push(format!("{candidate_id}: {id} source area is not an exact integer m²"));
                continue;
            };
            let Some(expected_area_mm2) = source_area_m2.checked_mul(1_000_000) else {
                errors.push(format!("{candidate_id}: {id} source area overflows mm²"));
                continue;
            };
            if room.area_mm2() != Some(expected_area_mm2) {
                errors.push(format!("{candidate_id}: {id} geometry area does not match source area"));
            }
            rooms.insert(id, room);
        }
    } else {
        errors.push(format!("{candidate_id}: rooms array is missing"));
    }

    for id in source_by_id.keys() {
        if !seen_ids.contains(id) {
            errors.push(format!("{candidate_id}: source space {id} has no geometric room"));
        }
    }
    for id in &seen_ids {
        if !source_by_id.contains_key(id) {
            errors.push(format!("{candidate_id}: unregistered geometric room {id}"));
        }
    }

    let room_values = rooms.values().collect::<Vec<_>>();
    for index in 0..room_values.len() {
        for other in (index + 1)..room_values.len() {
            let first = room_values[index];
            let second = room_values[other];
            let overlap_x = first.x < second.right().unwrap_or(i64::MIN)
                && second.x < first.right().unwrap_or(i64::MIN);
            let overlap_y = first.y < second.top().unwrap_or(i64::MIN)
                && second.y < first.top().unwrap_or(i64::MIN);
            if overlap_x && overlap_y {
                errors.push(format!(
                    "{candidate_id}: rooms {} and {} overlap",
                    first.id, second.id
                ));
            }
        }
    }

    let mut opening_ids = BTreeSet::new();
    let mut opening_pairs = BTreeSet::new();
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    if let Some(openings) = array_at(candidate, "openings") {
        for opening in openings {
            let id = text_at(opening, "opening_id").unwrap_or("<missing-opening-id>");
            if !opening_ids.insert(id.to_owned()) {
                errors.push(format!("{candidate_id}: duplicate opening ID {id}"));
                continue;
            }
            match shared_boundary_opening(opening, &rooms) {
                Ok((from, to)) => {
                    opening_pairs.insert(pair_key(&from, &to));
                    adjacency.entry(from.clone()).or_default().insert(to.clone());
                    adjacency.entry(to).or_default().insert(from);
                }
                Err(error) => errors.push(format!("{candidate_id}: {error}")),
            }
        }
    } else {
        errors.push(format!("{candidate_id}: openings array is missing"));
    }

    if let Some(relationships) = array_at(graph_candidate, "relationships") {
        for relationship in relationships {
            if text_at(relationship, "relationship") == Some("DIRECT_ACCESS") {
                let from = text_at(relationship, "from_space_id").unwrap_or("");
                let to = text_at(relationship, "to_space_id").unwrap_or("");
                if !opening_pairs.contains(&pair_key(from, to)) {
                    errors.push(format!(
                        "{candidate_id}: required DIRECT_ACCESS {from} ↔ {to} has no qualified opening"
                    ));
                }
            }
        }
    }

    let mut route_ids = BTreeSet::new();
    if let Some(routes) = array_at(candidate, "access_routes") {
        for route in routes {
            let route_id = text_at(route, "route_id").unwrap_or("<missing-route-id>");
            if !route_ids.insert(route_id.to_owned()) {
                errors.push(format!("{candidate_id}: duplicate route ID {route_id}"));
                continue;
            }
            let from = text_at(route, "from_space_id").unwrap_or("");
            let to = text_at(route, "to_space_id").unwrap_or("");
            if !rooms.contains_key(from) || !rooms.contains_key(to) {
                errors.push(format!("{candidate_id}: route {route_id} has an unknown endpoint"));
            } else if !route_connected(from, to, &adjacency) {
                errors.push(format!("{candidate_id}: route {route_id} is disconnected"));
            }
        }
    } else {
        errors.push(format!("{candidate_id}: access_routes array is missing"));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate(fixture: &Value, graph: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let envelope = fixture.get("buildable_envelope").unwrap_or(&Value::Null);
    if text_at(fixture, "schema_version") != Some("0.1.0")
        || text_at(fixture, "study_id") != Some("HOME-GEOMETRIC-LAYOUT-SYNTHETIC-001")
        || text_at(fixture, "maturity") != Some("SYNTHETIC")
        || text_at(fixture, "execution_status") != Some("SYNTHETIC_FIXTURE_NOT_FIELD_EVALUATED")
    {
        errors.push("fixture identity or maturity is invalid".to_owned());
    }
    let coordinate = fixture.get("coordinate_system").unwrap_or(&Value::Null);
    if text_at(coordinate, "unit") != Some("MILLIMETRE")
        || text_at(coordinate, "origin") != Some("BUILDABLE_ENVELOPE_SOUTH_WEST")
        || text_at(coordinate, "x_axis") != Some("EAST")
        || text_at(coordinate, "y_axis") != Some("NORTH")
        || int_at(coordinate, "integer_grid_mm") != Some(1)
    {
        errors.push("coordinate system must use the frozen integer-millimetre convention".to_owned());
    }
    if text_at(envelope, "geometry") != Some("AXIS_ALIGNED_RECTANGLE") {
        errors.push("v0.1 supports only an axis-aligned rectangular envelope".to_owned());
    }
    if text_at(envelope, "state")
        != Some("SYNTHETIC_ENVELOPE_NOT_SURVEYED_OR_AUTHORITY_CONFIRMED")
    {
        errors.push("envelope is promoted beyond synthetic/unverified state".to_owned());
    }
    let policy = fixture.get("comparison_policy").unwrap_or(&Value::Null);
    if text_at(policy, "ranking_state") != Some("UNRANKED")
        || !policy.get("scalar_score").is_some_and(Value::is_null)
        || policy.get("resident_choice_required").and_then(Value::as_bool) != Some(true)
    {
        errors.push("candidate comparison must remain unranked and resident-directed".to_owned());
    }
    if text_at(fixture, "claim_ceiling")
        != Some("Synthetic 2D orthogonal geometry consistency only. No site, code, professional, physical performance, cost, buildability or approval claim.")
    {
        errors.push("claim ceiling is missing or altered".to_owned());
    }

    let candidates = array_at(fixture, "layout_candidates");
    if let Some(candidates) = candidates {
        if candidates.is_empty() {
            errors.push("at least one layout candidate is required".to_owned());
        }
        let mut ids = BTreeSet::new();
        for candidate in candidates {
            let id = text_at(candidate, "candidate_id").unwrap_or("<missing-candidate-id>");
            if !ids.insert(id.to_owned()) {
                errors.push(format!("duplicate candidate ID {id}"));
            }
            if let Err(candidate_errors) = validate_candidate(candidate, envelope, graph) {
                errors.extend(candidate_errors);
            }
        }
    } else {
        errors.push("layout_candidates array is missing".to_owned());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn run() -> Result<Value, String> {
    let fixture = parse_json(FIXTURE, "geometric layout fixture")?;
    let graph = parse_json(GRAPH, "spatial intent graph")?;
    parse_json(SCHEMA, "geometric layout schema")?;
    validate(&fixture, &graph).map_err(|errors| errors.join("\n"))?;
    Ok(json!({
        "study_id": "HOME-GEOMETRIC-LAYOUT-SYNTHETIC-001",
        "result": "PASS_GEOMETRIC_CONSISTENCY_ONLY",
        "candidate_count": array_at(&fixture, "layout_candidates").map_or(0, Vec::len),
        "claim_ceiling": "Synthetic 2D orthogonal geometry consistency only"
    }))
}

fn main() {
    let arguments = env::args().collect::<Vec<_>>();
    let result = run();
    if arguments.iter().any(|argument| argument == "--self-test") {
        match result {
            Ok(report) => {
                println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
                println!("Synthetic geometry consistency self-test passed; this is not a building approval.");
            }
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
    } else if let Err(error) = result {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> (Value, Value) {
        (
            parse_json(FIXTURE, "fixture").expect("fixture JSON parses"),
            parse_json(GRAPH, "graph").expect("graph JSON parses"),
        )
    }

    fn first_candidate_mut(fixture: &mut Value) -> &mut Value {
        &mut fixture["layout_candidates"][0]
    }

    #[test]
    fn positive_synthetic_layout_passes() {
        let (fixture, graph) = inputs();
        assert!(validate(&fixture, &graph).is_ok());
    }

    #[test]
    fn room_overlap_is_rejected() {
        let (mut fixture, graph) = inputs();
        let candidate = first_candidate_mut(&mut fixture);
        let rooms = candidate["rooms"].as_array_mut().expect("rooms");
        let storage = rooms
            .iter_mut()
            .find(|room| room["space_id"] == "B-STORAGE-01")
            .expect("storage");
        storage["x_mm"] = json!(12000);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn out_of_envelope_room_is_rejected() {
        let (mut fixture, graph) = inputs();
        let candidate = first_candidate_mut(&mut fixture);
        let storage = candidate["rooms"]
            .as_array_mut()
            .expect("rooms")
            .iter_mut()
            .find(|room| room["space_id"] == "B-STORAGE-01")
            .expect("storage");
        storage["x_mm"] = json!(14000);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn zero_width_room_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["rooms"][0]["width_mm"] = json!(0);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn area_mismatch_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["rooms"][0]["width_mm"] = json!(3999);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn missing_room_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["rooms"]
            .as_array_mut()
            .expect("rooms")
            .retain(|room| room["space_id"] != "B-STORAGE-01");
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn duplicate_room_is_rejected() {
        let (mut fixture, graph) = inputs();
        let candidate = first_candidate_mut(&mut fixture);
        let duplicate = candidate["rooms"][0].clone();
        candidate["rooms"].as_array_mut().expect("rooms").push(duplicate);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn opening_off_shared_boundary_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["openings"][0]["coordinate_mm"] = json!(2100);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn opening_outside_shared_span_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["openings"][0]["span_end_mm"] = json!(5000);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn required_direct_access_without_opening_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["openings"]
            .as_array_mut()
            .expect("openings")
            .retain(|opening| opening["opening_id"] != "D-KITCHEN-YARD");
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn disconnected_route_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["openings"]
            .as_array_mut()
            .expect("openings")
            .retain(|opening| {
                opening["opening_id"] != "D-CIRC-WORK"
                    && opening["opening_id"] != "D-UTILITY-CIRC"
            });
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn physical_maturity_promotion_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["physical_verification_state"] = json!("PASS");
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn scalar_score_is_rejected() {
        let (mut fixture, graph) = inputs();
        fixture["comparison_policy"]["scalar_score"] = json!(0.9);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn nonzero_floor_index_is_rejected() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["rooms"][0]["floor_index"] = json!(1);
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn coordinate_origin_drift_is_rejected() {
        let (mut fixture, graph) = inputs();
        fixture["coordinate_system"]["origin"] = json!("UNKNOWN");
        assert!(validate(&fixture, &graph).is_err());
    }

    #[test]
    fn source_graph_candidate_must_exist() {
        let (mut fixture, graph) = inputs();
        first_candidate_mut(&mut fixture)["source_graph_candidate_id"] = json!("UNKNOWN");
        assert!(validate(&fixture, &graph).is_err());
    }
}