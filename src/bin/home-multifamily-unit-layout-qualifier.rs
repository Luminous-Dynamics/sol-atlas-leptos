// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Synthetic multi-family unit-interior geometry qualifier.
//!
//! Validates unit-local integer-mm room rectangles, exact unit/building transform,
//! room-envelope partition, non-overlap, resident activity assignment, explicit
//! shared-space references, resident relationship mapping, shared-boundary portal
//! geometry and everyday room-graph reachability.
//!
//! It does not qualify wall thickness, door leaf/swing, clear width, accessibility,
//! fire or emergency egress, structure, daylight, view/privacy performance,
//! acoustics, thermal/ventilation, service capacity, cost, planning or buildability.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::{env, process};

const LAYOUT: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-unit-layout-synthetic-v0.1.json"
);
const BUILDING: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-building-synthetic-v0.1.json"
);
const PROGRAMME: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-resident-programme-synthetic-v0.1.json"
);
const CORPUS: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-unit-layout-corpus-v0.1.json"
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

impl Rect {
    fn right(self) -> Option<i64> {
        self.x.checked_add(self.width)
    }

    fn bottom(self) -> Option<i64> {
        self.y.checked_add(self.height)
    }

    fn area(self) -> Option<i64> {
        self.width.checked_mul(self.height)
    }

    fn contains(self, other: Self) -> bool {
        matches!(
            (self.right(), self.bottom(), other.right(), other.bottom()),
            (Some(right), Some(bottom), Some(other_right), Some(other_bottom))
                if other.x >= self.x
                    && other.y >= self.y
                    && other_right <= right
                    && other_bottom <= bottom
        )
    }

    fn overlaps(self, other: Self) -> bool {
        matches!(
            (self.right(), self.bottom(), other.right(), other.bottom()),
            (Some(right), Some(bottom), Some(other_right), Some(other_bottom))
                if self.x < other_right
                    && other.x < right
                    && self.y < other_bottom
                    && other.y < bottom
        )
    }
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn integer_at(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn parse_rect(value: &Value, label: &str, errors: &mut Vec<String>) -> Option<Rect> {
    let Some(x) = integer_at(value, "x_mm") else {
        errors.push(format!("{label}: x_mm must be an integer"));
        return None;
    };
    let Some(y) = integer_at(value, "y_mm") else {
        errors.push(format!("{label}: y_mm must be an integer"));
        return None;
    };
    let Some(width) = integer_at(value, "width_mm") else {
        errors.push(format!("{label}: width_mm must be an integer"));
        return None;
    };
    let Some(height) = integer_at(value, "height_mm") else {
        errors.push(format!("{label}: height_mm must be an integer"));
        return None;
    };
    if x < 0 || y < 0 || width <= 0 || height <= 0 {
        errors.push(format!(
            "{label}: coordinates must be non-negative and dimensions positive"
        ));
        return None;
    }
    if x.checked_add(width).is_none()
        || y.checked_add(height).is_none()
        || width.checked_mul(height).is_none()
    {
        errors.push(format!("{label}: coordinate extent or area overflows i64"));
        return None;
    }
    Some(Rect {
        x,
        y,
        width,
        height,
    })
}

fn unique_ids(
    rows: Option<&Vec<Value>>,
    key: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(rows) = rows else {
        errors.push(format!("{label}: array is missing"));
        return ids;
    };
    for (index, row) in rows.iter().enumerate() {
        let Some(id) = text_at(row, key) else {
            errors.push(format!("{label}[{index}]: {key} is missing"));
            continue;
        };
        if !ids.insert(id.to_owned()) {
            errors.push(format!("{label}: duplicate {key} {id}"));
        }
    }
    ids
}

fn portal_matches_boundary(a: Rect, b: Rect, portal: &Value) -> bool {
    let Some(orientation) = text_at(portal, "orientation") else {
        return false;
    };
    let Some(coordinate) = integer_at(portal, "coordinate_mm") else {
        return false;
    };
    let Some(start) = integer_at(portal, "start_mm") else {
        return false;
    };
    let Some(end) = integer_at(portal, "end_mm") else {
        return false;
    };
    if start < 0 || end <= start {
        return false;
    }
    match orientation {
        "horizontal" => {
            let boundary = (a.bottom() == Some(b.y) && coordinate == b.y)
                || (b.bottom() == Some(a.y) && coordinate == a.y);
            let lower = a.x.max(b.x);
            let upper = a
                .right()
                .unwrap_or(i64::MIN)
                .min(b.right().unwrap_or(i64::MIN));
            boundary && start >= lower && end <= upper
        }
        "vertical" => {
            let boundary = (a.right() == Some(b.x) && coordinate == b.x)
                || (b.right() == Some(a.x) && coordinate == a.x);
            let lower = a.y.max(b.y);
            let upper = a
                .bottom()
                .unwrap_or(i64::MIN)
                .min(b.bottom().unwrap_or(i64::MIN));
            boundary && start >= lower && end <= upper
        }
        _ => false,
    }
}

fn privacy_compatible(requirement: &str, target_kind: &str, target_zone: Option<&str>) -> bool {
    if target_kind == "building_shared_space" {
        return requirement == "shared" && target_zone == Some("shared_amenity");
    }
    match requirement {
        "private" => target_zone == Some("private"),
        "semi_private" => matches!(target_zone, Some("semi_private" | "private")),
        "service" => target_zone == Some("service"),
        "shared" => target_zone == Some("shared"),
        "public" => target_zone == Some("public"),
        "not_stated" => true,
        _ => false,
    }
}

fn validate_corpus(corpus: &Value, errors: &mut Vec<String>) -> usize {
    if text_at(corpus, "schema_version") != Some("multifamily-unit-layout-corpus/0.1") {
        errors.push("unit-layout corpus has unsupported schema_version".to_owned());
    }
    if text_at(corpus, "maturity") != Some("authored_manifest_not_executed")
        || integer_at(corpus, "executed_cases") != Some(0)
    {
        errors.push("unit-layout corpus must remain explicitly unexecuted".to_owned());
    }
    if text_at(corpus, "base_fixture") != Some("multifamily-unit-layout-synthetic-v0.1.json") {
        errors.push("unit-layout corpus is not bound to the exact base fixture".to_owned());
    }
    let rows = array_at(corpus, "cases");
    let case_ids = unique_ids(rows, "case_id", "unit-layout cases", errors);
    let declared = integer_at(corpus, "case_count").unwrap_or(-1);
    if declared != 24 || case_ids.len() != 24 || declared != case_ids.len() as i64 {
        errors.push(format!(
            "unit-layout corpus requires 24 cases; declared {declared}, found {}",
            case_ids.len()
        ));
    }
    let mut positive = 0usize;
    let mut adversarial = 0usize;
    if let Some(rows) = rows {
        for case in rows {
            let id = text_at(case, "case_id").unwrap_or("<missing-case>");
            if text_at(case, "base_fixture") != Some("multifamily-unit-layout-synthetic-v0.1.json")
            {
                errors.push(format!("unit-layout case {id} has wrong base fixture"));
            }
            if text_at(case, "scope") != Some("synthetic_geometry_only")
                || text_at(case, "execution_state") != Some("not_executed")
                || !case.get("probability").is_some_and(Value::is_null)
            {
                errors.push(format!(
                    "unit-layout case {id} has invalid scope/execution/probability"
                ));
            }
            if text_at(case, "mutation").is_none_or(str::is_empty)
                || text_at(case, "oracle_invariant").is_none_or(str::is_empty)
            {
                errors.push(format!(
                    "unit-layout case {id} lacks mutation or oracle invariant"
                ));
            }
            match text_at(case, "case_kind") {
                Some("positive_control") => {
                    positive += 1;
                    if text_at(case, "expected_disposition") != Some("PASS_SYNTHETIC_TOPOLOGY_ONLY")
                    {
                        errors.push(format!(
                            "positive unit-layout case {id} has incorrect disposition"
                        ));
                    }
                }
                Some("adversarial_control") => {
                    adversarial += 1;
                    if !matches!(
                        text_at(case, "expected_disposition"),
                        Some("REJECT" | "REJECT_PROMOTION")
                    ) {
                        errors.push(format!(
                            "adversarial unit-layout case {id} is not reject/block"
                        ));
                    }
                }
                _ => errors.push(format!("unit-layout case {id} has invalid case_kind")),
            }
        }
    }
    if positive != 1 || adversarial != 23 {
        errors.push(format!("unit-layout corpus must contain one positive and 23 adversarial cases; found {positive} and {adversarial}"));
    }
    if let Some(split) = corpus.get("case_split") {
        let development = array_at(split, "development_cases")
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if development != case_ids {
            errors.push("unit-layout development split does not equal case IDs".to_owned());
        }
        if !array_at(split, "calibration_cases").is_some_and(Vec::is_empty)
            || !array_at(split, "sealed_confirmation_cases").is_some_and(Vec::is_empty)
        {
            errors.push("unit-layout corpus must not claim calibration or sealed cases".to_owned());
        }
    } else {
        errors.push("unit-layout corpus has no case_split".to_owned());
    }
    case_ids.len()
}

fn validate(
    layout: &Value,
    building: &Value,
    programme: &Value,
    corpus: &Value,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let corpus_case_count = validate_corpus(corpus, &mut errors);
    if text_at(layout, "schema_version") != Some("multifamily-unit-layout/0.1") {
        errors.push("unit layout has unsupported schema_version".to_owned());
    }
    if text_at(layout, "maturity") != Some("synthetic_room_rectangles_only") {
        errors.push("unit layout cannot be promoted beyond synthetic room rectangles".to_owned());
    }
    if text_at(layout, "claim_ceiling").is_none() {
        errors.push("unit layout claim ceiling is missing".to_owned());
    }

    let source = layout.get("source_binding").unwrap_or(&Value::Null);
    if text_at(
        source.get("building_candidate_ref").unwrap_or(&Value::Null),
        "id",
    ) != text_at(building, "candidate_id")
    {
        errors.push("unit layout is not bound to the exact building candidate".to_owned());
    }
    if text_at(
        source.get("resident_programme_ref").unwrap_or(&Value::Null),
        "id",
    ) != text_at(programme, "programme_id")
    {
        errors.push("unit layout is not bound to the exact resident programme".to_owned());
    }

    let building_units = array_at(building, "unit_programmes");
    let building_unit_ids = unique_ids(
        building_units,
        "unit_id",
        "building.unit_programmes",
        &mut errors,
    );
    let programme_units = array_at(programme, "unit_programmes");
    let programme_unit_ids = unique_ids(
        programme_units,
        "unit_id",
        "programme.unit_programmes",
        &mut errors,
    );
    if building_unit_ids != programme_unit_ids {
        errors.push("building and resident programme unit sets do not match".to_owned());
    }

    let building_spaces = array_at(building.get("building").unwrap_or(&Value::Null), "spaces");
    let mut building_space_records = BTreeMap::<String, (String, String, Rect)>::new();
    if let Some(rows) = building_spaces {
        for space in rows {
            let id = text_at(space, "space_id").unwrap_or("<missing-space>");
            let floor_id = text_at(space, "floor_id").unwrap_or("<missing-floor>");
            let kind = text_at(space, "space_kind").unwrap_or("<missing-kind>");
            if let Some(rect) = space
                .get("footprint")
                .and_then(|v| parse_rect(v, id, &mut errors))
            {
                building_space_records
                    .insert(id.to_owned(), (floor_id.to_owned(), kind.to_owned(), rect));
            }
        }
    }

    let floor_rows = array_at(building.get("building").unwrap_or(&Value::Null), "floors");
    let mut floor_envelopes = BTreeMap::<String, Rect>::new();
    for floor in floor_rows.into_iter().flatten() {
        let floor_id = text_at(floor, "floor_id").unwrap_or("<missing-floor>");
        if let Some(rect) = floor
            .get("floor_envelope")
            .and_then(|v| parse_rect(v, floor_id, &mut errors))
        {
            floor_envelopes.insert(floor_id.to_owned(), rect);
        }
    }

    let mut report_units = 0usize;
    let mut total_rooms = 0usize;
    let mut total_activities = 0usize;
    let mut total_portals = 0usize;
    let mut total_relationships = 0usize;
    let mut all_unit_layout_ids = BTreeSet::new();

    let unit_layouts = array_at(layout, "unit_layouts");
    let layout_unit_ids = unique_ids(unit_layouts, "unit_id", "unit_layouts", &mut errors);
    if layout_unit_ids != building_unit_ids || layout_unit_ids != programme_unit_ids {
        errors.push("layout/building/programme unit sets do not match exactly".to_owned());
    }

    if let Some(unit_rows) = unit_layouts {
        for unit_layout in unit_rows {
            report_units += 1;
            let unit_id = text_at(unit_layout, "unit_id").unwrap_or("<missing-unit>");
            let unit_space_id =
                text_at(unit_layout, "unit_space_id").unwrap_or("<missing-unit-space>");
            let floor_id = text_at(unit_layout, "floor_id").unwrap_or("<missing-floor>");
            let programme_unit = programme_units
                .into_iter()
                .flatten()
                .find(|unit| text_at(unit, "unit_id") == Some(unit_id));
            if programme_unit.is_none() {
                errors.push(format!("layout unit {unit_id} has no resident programme"));
                continue;
            }
            let programme_unit = programme_unit.unwrap_or(&Value::Null);

            let Some((building_floor, building_kind, building_rect)) =
                building_space_records.get(unit_space_id)
            else {
                errors.push(format!(
                    "unit {unit_id} references unknown building space {unit_space_id}"
                ));
                continue;
            };
            if building_kind != "dwelling_unit"
                || text_at(
                    unit_layout
                        .get("unit_footprint_ref")
                        .unwrap_or(&Value::Null),
                    "id",
                ) != Some(unit_space_id)
                || unit_id
                    != text_at(
                        building_units
                            .into_iter()
                            .flatten()
                            .find(|unit| text_at(unit, "unit_id") == Some(unit_id))
                            .unwrap_or(&Value::Null),
                        "space_id",
                    )
            {
                errors.push(format!(
                    "unit {unit_id} footprint identity does not match building dwelling space"
                ));
            }
            if building_floor != floor_id {
                errors.push(format!(
                    "unit {unit_id} floor identity does not match building footprint"
                ));
            }
            if text_at(
                unit_layout.get("unit_transform").unwrap_or(&Value::Null),
                "coordinate_basis",
            ) != Some(
                "unit-local lower-left origin translated into building floor-plan coordinates",
            ) {
                errors.push(format!("unit {unit_id} has unsupported coordinate basis"));
            }
            let transform = unit_layout.get("unit_transform").unwrap_or(&Value::Null);
            if integer_at(transform, "translation_x_mm") != Some(building_rect.x)
                || integer_at(transform, "translation_y_mm") != Some(building_rect.y)
            {
                errors.push(format!(
                    "unit {unit_id} transform does not match building footprint origin"
                ));
            }
            let Some(unit_envelope) = unit_layout
                .get("unit_envelope_local")
                .and_then(|v| parse_rect(v, &format!("{unit_id} local envelope"), &mut errors))
            else {
                continue;
            };
            if unit_envelope.x != 0
                || unit_envelope.y != 0
                || unit_envelope.width != building_rect.width
                || unit_envelope.height != building_rect.height
            {
                errors.push(format!(
                    "unit {unit_id} local envelope does not match building unit dimensions"
                ));
            }
            if let Some(floor_envelope) = floor_envelopes.get(floor_id) {
                let world = Rect {
                    x: building_rect.x,
                    y: building_rect.y,
                    width: building_rect.width,
                    height: building_rect.height,
                };
                if !floor_envelope.contains(world) {
                    errors.push(format!(
                        "unit {unit_id} building footprint is outside floor envelope"
                    ));
                }
            }

            let rooms = array_at(unit_layout, "rooms");
            let room_ids = unique_ids(rooms, "room_id", &format!("{unit_id}.rooms"), &mut errors);
            let mut room_records = BTreeMap::<String, (Rect, String, BTreeSet<String>)>::new();
            let mut area_sum = 0i64;
            if let Some(rooms) = rooms {
                for room in rooms {
                    let room_id = text_at(room, "room_id").unwrap_or("<missing-room>");
                    let room_kind = text_at(room, "room_kind").unwrap_or("<missing-kind>");
                    let privacy = text_at(room, "privacy_zone").unwrap_or("<missing-privacy>");
                    if !matches!(
                        room_kind,
                        "sleeping"
                            | "food_preparation"
                            | "work_study"
                            | "making"
                            | "living_guest"
                            | "hygiene_laundry"
                            | "storage"
                            | "circulation_entry"
                            | "other"
                    ) {
                        errors.push(format!("room {room_id} has invalid room_kind"));
                    }
                    if !matches!(
                        privacy,
                        "private" | "semi_private" | "service" | "shared" | "public" | "unknown"
                    ) {
                        errors.push(format!("room {room_id} has invalid privacy_zone"));
                    }
                    let Some(rect) = room
                        .get("footprint_local_mm")
                        .and_then(|v| parse_rect(v, room_id, &mut errors))
                    else {
                        continue;
                    };
                    if !unit_envelope.contains(rect) {
                        errors.push(format!("room {room_id} is outside unit-local envelope"));
                    }
                    match rect.area().and_then(|area| area_sum.checked_add(area)) {
                        Some(next) => area_sum = next,
                        None => errors.push(format!("unit {unit_id} room area sum overflows i64")),
                    }
                    let activity_refs = array_at(room, "activity_ids")
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect::<BTreeSet<_>>();
                    if activity_refs.len() != array_at(room, "activity_ids").map_or(0, Vec::len) {
                        errors.push(format!("room {room_id} has duplicate activity IDs"));
                    }
                    room_records.insert(
                        room_id.to_owned(),
                        (rect, privacy.to_owned(), activity_refs),
                    );
                }
            }
            let room_rows = rooms.into_iter().flatten().collect::<Vec<_>>();
            for (index, (id, _)) in room_records.iter().enumerate() {
                let _ = (index, id);
            }
            for i in 0..room_rows.len() {
                for j in (i + 1)..room_rows.len() {
                    let id_a = text_at(room_rows[i], "room_id").unwrap_or("<missing-a>");
                    let id_b = text_at(room_rows[j], "room_id").unwrap_or("<missing-b>");
                    if let (Some((a, _, _)), Some((b, _, _))) =
                        (room_records.get(id_a), room_records.get(id_b))
                    {
                        if a.overlaps(*b) {
                            errors.push(format!("unit {unit_id} rooms {id_a} and {id_b} overlap"));
                        }
                    }
                }
            }
            if area_sum != unit_envelope.area().unwrap_or(-1) {
                errors.push(format!(
                    "unit {unit_id} room rectangles total {area_sum}, not exact envelope area {}",
                    unit_envelope.area().unwrap_or(-1)
                ));
            }

            let activities = array_at(programme_unit, "activities");
            let activity_ids = unique_ids(
                activities,
                "activity_id",
                &format!("{unit_id}.programme.activities"),
                &mut errors,
            );
            let assignments = array_at(unit_layout, "activity_assignments");
            let assignment_activity_ids = unique_ids(
                assignments,
                "activity_id",
                &format!("{unit_id}.activity_assignments"),
                &mut errors,
            );
            if activity_ids != assignment_activity_ids {
                errors.push(format!(
                    "unit {unit_id} activities are not assigned exactly once"
                ));
            }
            let mut activity_targets = BTreeMap::<String, (String, String)>::new();
            if let Some(assignments) = assignments {
                for assignment in assignments {
                    let activity_id =
                        text_at(assignment, "activity_id").unwrap_or("<missing-activity>");
                    let target_kind =
                        text_at(assignment, "target_kind").unwrap_or("<missing-target-kind>");
                    let target_id = text_at(assignment, "target_id").unwrap_or("<missing-target>");
                    if !activity_ids.contains(activity_id) {
                        errors.push(format!(
                            "unit {unit_id} assignment references unknown activity {activity_id}"
                        ));
                    }
                    match target_kind {
                        "unit_room" => {
                            if !room_ids.contains(target_id) {
                                errors.push(format!(
                                    "activity {activity_id} targets unknown room {target_id}"
                                ));
                            }
                        }
                        "building_shared_space" => match building_space_records.get(target_id) {
                            Some((target_floor, kind, _))
                                if kind == "shared_amenity" && target_floor == floor_id => {}
                            Some(_) => errors.push(format!(
                                "activity {activity_id} has invalid shared-space target {target_id}"
                            )),
                            None => errors.push(format!(
                                "activity {activity_id} targets unknown shared space {target_id}"
                            )),
                        },
                        _ => errors.push(format!(
                            "activity {activity_id} has invalid target_kind {target_kind}"
                        )),
                    }
                    activity_targets.insert(
                        activity_id.to_owned(),
                        (target_kind.to_owned(), target_id.to_owned()),
                    );
                }
            }

            let mut expected_room_activities = BTreeMap::<String, BTreeSet<String>>::new();
            for (activity_id, (kind, target_id)) in &activity_targets {
                if kind == "unit_room" {
                    expected_room_activities
                        .entry(target_id.clone())
                        .or_default()
                        .insert(activity_id.clone());
                }
            }
            for (room_id, (_, _, actual)) in &room_records {
                let expected = expected_room_activities.remove(room_id).unwrap_or_default();
                if *actual != expected {
                    errors.push(format!(
                        "room {room_id} activity_ids do not match assignments"
                    ));
                }
            }
            if !expected_room_activities.is_empty() {
                errors.push(format!(
                    "unit {unit_id} has activity assignments to undeclared rooms"
                ));
            }

            if let Some(activities) = activities {
                for activity in activities {
                    let id = text_at(activity, "activity_id").unwrap_or("<missing-activity>");
                    let requirement =
                        text_at(activity, "privacy_requirement").unwrap_or("not_stated");
                    if let Some((target_kind, target_id)) = activity_targets.get(id) {
                        let target_zone = if target_kind == "unit_room" {
                            room_records
                                .get(target_id)
                                .map(|(_, privacy, _)| privacy.as_str())
                        } else {
                            building_space_records
                                .get(target_id)
                                .map(|(_, kind, _)| kind.as_str())
                        };
                        if !privacy_compatible(requirement, target_kind, target_zone) {
                            errors.push(format!("activity {id} privacy requirement is incompatible with target {target_id}"));
                        }
                    }
                }
            }

            let programme_relationships = array_at(programme_unit, "relationships");
            let layout_relationships = array_at(unit_layout, "relationship_mappings");
            let programme_rel_ids = unique_ids(
                programme_relationships,
                "relationship_id",
                &format!("{unit_id}.programme.relationships"),
                &mut errors,
            );
            let layout_rel_ids = unique_ids(
                layout_relationships,
                "relationship_id",
                &format!("{unit_id}.layout.relationships"),
                &mut errors,
            );
            if programme_rel_ids != layout_rel_ids {
                errors.push(format!(
                    "unit {unit_id} relationship mapping IDs do not match programme"
                ));
            }
            if let Some(rows) = programme_relationships {
                for relationship in rows {
                    let id = text_at(relationship, "relationship_id")
                        .unwrap_or("<missing-relationship>");
                    let mapping = layout_relationships
                        .into_iter()
                        .flatten()
                        .find(|row| text_at(row, "relationship_id") == Some(id));
                    let Some(mapping) = mapping else {
                        continue;
                    };
                    for key in ["from_activity_id", "to_activity_id", "relationship"] {
                        if text_at(mapping, key) != text_at(relationship, key) {
                            errors.push(format!("unit {unit_id} relationship {id} changes {key}"));
                        }
                    }
                    let from_id =
                        text_at(relationship, "from_activity_id").unwrap_or("<missing-from>");
                    let to_id = text_at(relationship, "to_activity_id").unwrap_or("<missing-to>");
                    let from_target = activity_targets.get(from_id).map(|v| v.1.as_str());
                    let to_target = activity_targets.get(to_id).map(|v| v.1.as_str());
                    if from_target != text_at(mapping, "from_target_id")
                        || to_target != text_at(mapping, "to_target_id")
                    {
                        errors.push(format!(
                            "unit {unit_id} relationship {id} has incorrect target mapping"
                        ));
                    }
                    if matches!(
                        text_at(relationship, "relationship"),
                        Some("separate" | "acoustically_separate" | "visually_private")
                    ) && from_target == to_target
                    {
                        errors.push(format!(
                            "unit {unit_id} separation relationship {id} maps to one target"
                        ));
                    }
                }
            }

            let portals = array_at(unit_layout, "portals");
            let portal_ids = unique_ids(
                portals,
                "portal_id",
                &format!("{unit_id}.portals"),
                &mut errors,
            );
            let mut graph = BTreeMap::<String, Vec<String>>::new();
            for room_id in &room_ids {
                graph.insert(room_id.clone(), Vec::new());
            }
            if let Some(portals) = portals {
                for portal in portals {
                    let portal_id = text_at(portal, "portal_id").unwrap_or("<missing-portal>");
                    let from = text_at(portal, "from_room_id").unwrap_or("<missing-from-room>");
                    let to = text_at(portal, "to_room_id").unwrap_or("<missing-to-room>");
                    if from == to || !room_ids.contains(from) || !room_ids.contains(to) {
                        errors.push(format!("portal {portal_id} has invalid room endpoints"));
                        continue;
                    }
                    let (Some((from_rect, _, _)), Some((to_rect, _, _))) =
                        (room_records.get(from), room_records.get(to))
                    else {
                        errors.push(format!("portal {portal_id} has unresolved room geometry"));
                        continue;
                    };
                    if !portal_matches_boundary(*from_rect, *to_rect, portal) {
                        errors.push(format!(
                            "portal {portal_id} is not on a shared boundary span"
                        ));
                    } else {
                        graph
                            .entry(from.to_owned())
                            .or_default()
                            .push(to.to_owned());
                        graph
                            .entry(to.to_owned())
                            .or_default()
                            .push(from.to_owned());
                    }
                    if text_at(portal, "clear_width_state") != Some("not_computed")
                        || !portal.get("clear_width_mm").is_some_and(Value::is_null)
                        || text_at(portal, "door_swing_state") != Some("not_modelled")
                    {
                        errors.push(format!(
                            "portal {portal_id} fabricates clear width or door swing"
                        ));
                    }
                }
            }
            if portal_ids.is_empty() {
                errors.push(format!("unit {unit_id} has no everyday portals"));
            }
            let root =
                text_at(unit_layout, "everyday_root_room_id").unwrap_or("<missing-root-room>");
            if !room_ids.contains(root) {
                errors.push(format!(
                    "unit {unit_id} everyday root room {root} is missing"
                ));
            } else {
                let mut reached = BTreeSet::from([root.to_owned()]);
                let mut queue = VecDeque::from([root.to_owned()]);
                while let Some(current) = queue.pop_front() {
                    for next in graph.get(&current).into_iter().flatten() {
                        if reached.insert(next.clone()) {
                            queue.push_back(next.clone());
                        }
                    }
                }
                if reached != room_ids {
                    errors.push(format!(
                        "unit {unit_id} everyday room graph is disconnected"
                    ));
                }
            }

            total_rooms += room_ids.len();
            total_activities += activity_ids.len();
            total_portals += portal_ids.len();
            total_relationships += layout_rel_ids.len();
            let _ = &all_unit_layout_ids;
        }
    }

    if report_units != building_unit_ids.len() {
        errors.push("unit layout count does not equal building unit count".to_owned());
    }
    let report = json!({
        "audit_state": if errors.is_empty() { "MULTIFAMILY_SYNTHETIC_UNIT_LAYOUT_PASS" } else { "MULTIFAMILY_SYNTHETIC_UNIT_LAYOUT_FAIL" },
        "unit_count": report_units,
        "authored_development_case_count": corpus_case_count,
        "room_count": total_rooms,
        "activity_assignment_count": total_activities,
        "relationship_count": total_relationships,
        "portal_count": total_portals,
        "clear_widths": "UNKNOWN",
        "door_swings": "NOT_MODELLED",
        "accessibility": "UNKNOWN",
        "fire_egress": "UNKNOWN",
        "physical_verification": "UNKNOWN",
        "claim_ceiling": "SYNTHETIC_UNIT_LOCAL_ROOM_RECTANGLES_AND_EVERYDAY_TOPOLOGY_ONLY"
    });
    if errors.is_empty() {
        Ok(report)
    } else {
        Err(errors)
    }
}

fn load_fixtures() -> (Value, Value, Value, Value) {
    (
        serde_json::from_str(LAYOUT).expect("unit layout fixture JSON"),
        serde_json::from_str(BUILDING).expect("building fixture JSON"),
        serde_json::from_str(PROGRAMME).expect("resident programme fixture JSON"),
        serde_json::from_str(CORPUS).expect("unit layout corpus JSON"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-multifamily-unit-layout-qualifier [--self-test]");
        return;
    }
    let (layout, building, programme, corpus) = load_fixtures();
    match validate(&layout, &building, &programme, &corpus) {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        ),
        Err(errors) => {
            eprintln!("HOME_MULTIFAMILY_UNIT_LAYOUT_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_four_unit_room_layout_qualifies() {
        let (layout, building, programme, corpus) = load_fixtures();
        let report =
            validate(&layout, &building, &programme, &corpus).expect("synthetic room layout");
        assert_eq!(report["unit_count"], 4);
        assert_eq!(report["room_count"], 28);
        assert_eq!(report["activity_assignment_count"], 24);
        assert_eq!(report["relationship_count"], 7);
        assert_eq!(report["portal_count"], 32);
        assert_eq!(report["authored_development_case_count"], 24);
        assert_eq!(report["clear_widths"], "UNKNOWN");
        assert_eq!(report["accessibility"], "UNKNOWN");
    }

    #[test]
    fn duplicate_corpus_case_identity_is_rejected() {
        let (layout, building, programme, mut corpus) = load_fixtures();
        let duplicate_id = corpus["cases"][0]["case_id"].clone();
        corpus["cases"][1]["case_id"] = duplicate_id;
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn unit_layout_corpus_cannot_claim_execution_or_sealed_cases() {
        let (layout, building, programme, mut corpus) = load_fixtures();
        corpus["executed_cases"] = json!(24);
        corpus["case_split"]["sealed_confirmation_cases"] = json!(["UL-SEALED-001"]);
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn unit_layout_corpus_split_must_match_case_ids() {
        let (layout, building, programme, mut corpus) = load_fixtures();
        corpus["case_split"]["development_cases"][0] = json!("ORPHAN-CASE");
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn room_overlap_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["rooms"][1]["footprint_local_mm"]["x_mm"] = json!(2900);
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn room_outside_unit_envelope_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["rooms"][0]["footprint_local_mm"]["x_mm"] = json!(5900);
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn unallocated_room_region_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["rooms"][6]["footprint_local_mm"]["width_mm"] = json!(2000);
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn room_activity_binding_mismatch_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["rooms"][0]["activity_ids"][0] = json!("UNKNOWN-ACTIVITY");
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn duplicate_activity_assignment_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["activity_assignments"][1]["activity_id"] =
            layout["unit_layouts"][0]["activity_assignments"][0]["activity_id"].clone();
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn shared_space_target_must_exist_on_same_floor() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][2]["activity_assignments"][4]["target_id"] =
            json!("NOT-A-SHARED-SPACE");
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn portal_off_shared_boundary_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["portals"][0]["coordinate_mm"] = json!(2999);
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn disconnected_room_graph_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["portals"].pop();
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn door_clear_width_promotion_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["portals"][0]["clear_width_mm"] = json!(900);
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn door_swing_promotion_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["portals"][0]["door_swing_state"] = json!("modelled");
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn stale_unit_transform_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["unit_transform"]["translation_x_mm"] = json!(1);
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn privacy_zone_downgrade_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["rooms"][0]["privacy_zone"] = json!("public");
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn relationship_activity_binding_drift_is_rejected() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["relationship_mappings"][0]["from_activity_id"] =
            json!("UNKNOWN-ACTIVITY");
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }

    #[test]
    fn separation_relationship_cannot_map_to_one_room() {
        let (mut layout, building, programme, corpus) = load_fixtures();
        layout["unit_layouts"][0]["relationship_mappings"][0]["to_target_id"] =
            layout["unit_layouts"][0]["relationship_mappings"][0]["from_target_id"].clone();
        assert!(validate(&layout, &building, &programme, &corpus).is_err());
    }
}
