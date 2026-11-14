// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Synthetic multi-family geometry and identity qualifier.
//!
//! This binary validates only the v0.1 floor-local orthogonal-rectangle contract:
//! floor-envelope containment/partition, same-floor overlap, exact unit/space
//! identity, declared everyday portal geometry, vertical-core references,
//! shared-service unit references, and the authored development-manifest shape.
//! It does not qualify accessibility, fire/egress, structure, daylight, privacy,
//! acoustics, thermal performance, utility capacity, planning, cost or buildability.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-building-synthetic-v0.1.json"
);
const CORPUS: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-adversarial-corpus-v0.1.json"
);

#[derive(Clone, Copy, Debug)]
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
            (Some(outer_right), Some(outer_bottom), Some(inner_right), Some(inner_bottom))
                if other.x >= self.x
                    && other.y >= self.y
                    && inner_right <= outer_right
                    && inner_bottom <= outer_bottom
        )
    }

    fn overlaps(self, other: Self) -> bool {
        matches!(
            (self.right(), self.bottom(), other.right(), other.bottom()),
            (Some(a_right), Some(a_bottom), Some(b_right), Some(b_bottom))
                if self.x < b_right
                    && other.x < a_right
                    && self.y < b_bottom
                    && other.y < a_bottom
        )
    }
}

#[derive(Clone, Debug)]
struct Space {
    floor_id: String,
    kind: String,
    unit_id: Option<String>,
    rect: Rect,
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
        errors.push(format!("{label}: coordinates must be non-negative and dimensions positive"));
        return None;
    }
    if x.checked_add(width).is_none() || y.checked_add(height).is_none() {
        errors.push(format!("{label}: coordinate extent overflows i64"));
        return None;
    }
    if width.checked_mul(height).is_none() {
        errors.push(format!("{label}: area overflows i64"));
        return None;
    }
    Some(Rect {
        x,
        y,
        width,
        height,
    })
}

fn unique_text_rows(
    rows: Option<&Vec<Value>>,
    id_key: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(rows) = rows else {
        errors.push(format!("{label}: array is missing"));
        return ids;
    };
    for (index, row) in rows.iter().enumerate() {
        let Some(id) = text_at(row, id_key) else {
            errors.push(format!("{label}[{index}]: {id_key} is missing"));
            continue;
        };
        if !ids.insert(id.to_owned()) {
            errors.push(format!("{label}: duplicate {id_key} {id}"));
        }
    }
    ids
}

fn portal_matches_shared_boundary(a: Rect, b: Rect, portal: &Value) -> bool {
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
            let common_boundary = (a.bottom() == Some(b.y) && coordinate == b.y)
                || (b.bottom() == Some(a.y) && coordinate == a.y);
            let low = a.x.max(b.x);
            let high = a.right().unwrap_or(i64::MIN).min(b.right().unwrap_or(i64::MIN));
            common_boundary && start >= low && end <= high && end > start
        }
        "vertical" => {
            let common_boundary = (a.right() == Some(b.x) && coordinate == b.x)
                || (b.right() == Some(a.x) && coordinate == a.x);
            let low = a.y.max(b.y);
            let high = a.bottom().unwrap_or(i64::MIN).min(b.bottom().unwrap_or(i64::MIN));
            common_boundary && start >= low && end <= high && end > start
        }
        _ => false,
    }
}

fn validate_case_manifest(corpus: &Value, errors: &mut Vec<String>) -> usize {
    if text_at(corpus, "schema_version") != Some("multifamily-adversarial-corpus/0.1") {
        errors.push("case manifest has an unsupported schema_version".to_owned());
    }
    if text_at(corpus, "maturity") != Some("authored_manifest_not_executed") {
        errors.push("case manifest must remain explicitly unexecuted".to_owned());
    }
    if text_at(corpus, "base_fixture") != Some("multifamily-building-synthetic-v0.1.json") {
        errors.push("case manifest is not bound to the exact base fixture".to_owned());
    }

    let rows = array_at(corpus, "cases");
    let case_ids = unique_text_rows(rows, "case_id", "cases", errors);
    let declared = integer_at(corpus, "case_count").unwrap_or(-1);
    if declared != case_ids.len() as i64 {
        errors.push(format!(
            "case manifest declares {declared} cases but contains {} unique IDs",
            case_ids.len()
        ));
    }

    let mut positive = 0usize;
    let mut adversarial = 0usize;
    if let Some(rows) = rows {
        for row in rows {
            if text_at(row, "base_fixture") != Some("multifamily-building-synthetic-v0.1.json") {
                errors.push("case row is bound to a different base fixture".to_owned());
            }
            if text_at(row, "scope") != Some("synthetic_contract_only") {
                errors.push("case row must retain synthetic_contract_only scope".to_owned());
            }
            if text_at(row, "expected_physical_claim") != Some("none") {
                errors.push("case row must not claim a physical result".to_owned());
            }
            match text_at(row, "case_kind") {
                Some("positive_control") => positive += 1,
                Some("adversarial_control") => adversarial += 1,
                _ => errors.push("case row has an unsupported case_kind".to_owned()),
            }
            if text_at(row, "expected_disposition").is_none() {
                errors.push("case row has no expected_disposition".to_owned());
            }
            if text_at(row, "oracle_invariant").is_none() {
                errors.push("case row has no oracle_invariant".to_owned());
            }
        }
    }
    if positive != 1 {
        errors.push(format!("case manifest must contain exactly one positive control; found {positive}"));
    }
    if positive + adversarial != case_ids.len() {
        errors.push("case manifest includes cases outside the development-control split".to_owned());
    }

    if let Some(split) = corpus.get("case_split") {
        let development = array_at(split, "development_cases");
        let split_ids = development
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if split_ids != case_ids {
            errors.push("development split does not exactly match authored case IDs".to_owned());
        }
        if !array_at(split, "sealed_confirmation_cases").is_some_and(Vec::is_empty) {
            errors.push("this v0.1 manifest must not claim sealed confirmation cases".to_owned());
        }
    } else {
        errors.push("case manifest has no case_split".to_owned());
    }
    case_ids.len()
}

fn validate(fixture: &Value, corpus: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let candidate_id = text_at(fixture, "candidate_id").unwrap_or("<missing-candidate-id>");
    if text_at(fixture, "schema_version") != Some("multi-family-building/0.1") {
        errors.push("fixture has an unsupported schema_version".to_owned());
    }
    if text_at(fixture, "maturity") != Some("synthetic_contract") {
        errors.push("synthetic fixture cannot be promoted to a professional or physical maturity".to_owned());
    }

    let case_count = validate_case_manifest(corpus, &mut errors);
    let Some(building) = fixture.get("building") else {
        return Err(vec!["fixture building object is missing".to_owned()]);
    };
    if text_at(building, "coordinate_unit") != Some("millimetre") {
        errors.push("building coordinate_unit must be millimetre".to_owned());
    }

    let Some(building_envelope) = building
        .get("envelope")
        .and_then(|rect| parse_rect(rect, "building.envelope", &mut errors))
    else {
        return Err(errors);
    };

    let floors_rows = array_at(building, "floors");
    let floor_ids = unique_text_rows(floors_rows, "floor_id", "building.floors", &mut errors);
    let mut floor_envelopes = BTreeMap::<String, Rect>::new();
    let mut floor_indices = BTreeSet::new();
    let mut elevations = BTreeSet::new();
    if let Some(floors) = floors_rows {
        for floor in floors {
            let id = text_at(floor, "floor_id").unwrap_or("<missing-floor-id>").to_owned();
            let Some(index) = integer_at(floor, "floor_index") else {
                errors.push(format!("{id}: floor_index must be an integer"));
                continue;
            };
            let Some(elevation) = integer_at(floor, "elevation_mm") else {
                errors.push(format!("{id}: elevation_mm must be an integer"));
                continue;
            };
            if !floor_indices.insert(index) {
                errors.push(format!("duplicate floor_index {index}"));
            }
            if !elevations.insert(elevation) {
                errors.push(format!("duplicate floor elevation {elevation}"));
            }
            let Some(rect) = floor
                .get("floor_envelope")
                .and_then(|r| parse_rect(r, &format!("floor {id} envelope"), &mut errors))
            else {
                continue;
            };
            if !building_envelope.contains(rect) {
                errors.push(format!("floor {id} envelope is outside the building envelope"));
            }
            floor_envelopes.insert(id, rect);
        }
    }
    if floor_ids.is_empty() {
        errors.push("building must declare at least one floor".to_owned());
    }

    let space_rows = array_at(building, "spaces");
    let space_ids = unique_text_rows(space_rows, "space_id", "building.spaces", &mut errors);
    let mut spaces = BTreeMap::<String, Space>::new();
    if let Some(rows) = space_rows {
        for row in rows {
            let id = text_at(row, "space_id").unwrap_or("<missing-space-id>").to_owned();
            let floor_id = text_at(row, "floor_id").unwrap_or("<missing-floor-id>").to_owned();
            let kind = text_at(row, "space_kind").unwrap_or("<missing-space-kind>").to_owned();
            let unit_id = text_at(row, "unit_id").map(str::to_owned);
            let Some(rect) = row
                .get("footprint")
                .and_then(|r| parse_rect(r, &format!("space {id} footprint"), &mut errors))
            else {
                continue;
            };
            let Some(floor_rect) = floor_envelopes.get(&floor_id).copied() else {
                errors.push(format!("space {id} references unknown floor {floor_id}"));
                continue;
            };
            if !floor_rect.contains(rect) {
                errors.push(format!("space {id} footprint is outside floor {floor_id} envelope"));
            }
            if !matches!(
                kind.as_str(),
                "dwelling_unit"
                    | "corridor"
                    | "lobby"
                    | "stair"
                    | "lift"
                    | "shared_amenity"
                    | "courtyard"
                    | "service_plant"
                    | "refuse"
                    | "storage"
                    | "service_access"
                    | "other"
            ) {
                errors.push(format!("space {id} has unknown space_kind {kind}"));
            }
            spaces.insert(
                id,
                Space {
                    floor_id,
                    kind,
                    unit_id,
                    rect,
                },
            );
        }
    }

    for floor_id in &floor_ids {
        let Some(envelope) = floor_envelopes.get(floor_id).copied() else {
            continue;
        };
        let same_floor = spaces
            .iter()
            .filter(|(_, space)| space.floor_id == *floor_id)
            .collect::<Vec<_>>();
        let mut area_sum = 0_i64;
        for (index, (id, space)) in same_floor.iter().enumerate() {
            match space.rect.area().and_then(|area| area_sum.checked_add(area)) {
                Some(next) => area_sum = next,
                None => errors.push(format!("floor {floor_id}: space area sum overflows i64")),
            }
            for (other_id, other) in same_floor.iter().skip(index + 1) {
                if space.rect.overlaps(other.rect) {
                    errors.push(format!(
                        "floor {floor_id}: spaces {id} and {other_id} overlap"
                    ));
                }
            }
        }
        let envelope_area = envelope.area().unwrap_or(-1);
        if area_sum != envelope_area {
            errors.push(format!(
                "floor {floor_id}: declared space area sum {area_sum} does not partition envelope area {envelope_area}"
            ));
        }
    }

    let unit_rows = array_at(fixture, "unit_programmes");
    let unit_ids = unique_text_rows(unit_rows, "unit_id", "unit_programmes", &mut errors);
    let mut unit_spaces = BTreeSet::<String>::new();
    if let Some(rows) = unit_rows {
        for row in rows {
            let unit_id = text_at(row, "unit_id").unwrap_or("<missing-unit-id>");
            let space_id = text_at(row, "space_id").unwrap_or("<missing-space-id>");
            if !unit_spaces.insert(space_id.to_owned()) {
                errors.push(format!("multiple unit programmes reference space {space_id}"));
            }
            match spaces.get(space_id) {
                Some(space)
                    if space.kind == "dwelling_unit"
                        && space.unit_id.as_deref() == Some(unit_id) => {}
                Some(_) => errors.push(format!(
                    "unit {unit_id} does not match dwelling-space identity {space_id}"
                )),
                None => errors.push(format!("unit {unit_id} references missing space {space_id}")),
            }
        }
    }
    for (space_id, space) in &spaces {
        if space.kind == "dwelling_unit" {
            match space.unit_id.as_deref() {
                Some(unit_id) if unit_ids.contains(unit_id) && unit_spaces.contains(space_id) => {}
                _ => errors.push(format!("dwelling space {space_id} has no exact unit programme")),
            }
        }
    }

    let connection_rows = array_at(building, "everyday_connections");
    let connection_ids = unique_text_rows(
        connection_rows,
        "connection_id",
        "building.everyday_connections",
        &mut errors,
    );
    if let Some(rows) = connection_rows {
        for row in rows {
            let id = text_at(row, "connection_id").unwrap_or("<missing-connection-id>");
            let floor_id = text_at(row, "floor_id").unwrap_or("<missing-floor-id>");
            let from_id = text_at(row, "from_space_id").unwrap_or("<missing-from-space>");
            let to_id = text_at(row, "to_space_id").unwrap_or("<missing-to-space>");
            let (Some(from), Some(to)) = (spaces.get(from_id), spaces.get(to_id)) else {
                errors.push(format!("connection {id} references a missing space"));
                continue;
            };
            if from.floor_id != floor_id || to.floor_id != floor_id {
                errors.push(format!("connection {id} binds spaces to the wrong floor"));
            }
            if !matches!(
                text_at(row, "connection_kind"),
                Some("everyday_access" | "shared_amenity_access" | "service_access")
            ) {
                errors.push(format!("connection {id} has an invalid connection_kind"));
            }
            if !portal_matches_shared_boundary(from.rect, to.rect, row.get("portal").unwrap_or(&Value::Null)) {
                errors.push(format!("connection {id} portal is not on a shared boundary span"));
            }
        }
    }

    let core_rows = array_at(building, "vertical_cores");
    let core_ids = unique_text_rows(core_rows, "core_id", "building.vertical_cores", &mut errors);
    let mut cores = BTreeMap::<String, (String, BTreeSet<String>, BTreeSet<String>)>::new();
    if let Some(rows) = core_rows {
        for row in rows {
            let id = text_at(row, "core_id").unwrap_or("<missing-core-id>").to_owned();
            let kind = text_at(row, "core_kind").unwrap_or("<missing-core-kind>").to_owned();
            let floor_refs = array_at(row, "floor_ids")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            let space_refs = array_at(row, "space_ids")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            if floor_refs.len() < 2 || space_refs.len() < 2 {
                errors.push(format!("vertical core {id} must reference at least two floors and spaces"));
            }
            if floor_refs.iter().any(|floor| !floor_ids.contains(floor)) {
                errors.push(format!("vertical core {id} references an unknown floor"));
            }
            let expected_kind = match kind.as_str() {
                "stair" => "stair",
                "lift" => "lift",
                "ramp" => "other",
                _ => "<invalid>",
            };
            for space_id in &space_refs {
                match spaces.get(space_id) {
                    Some(space)
                        if space.floor_id.len() > 0
                            && floor_refs.contains(&space.floor_id)
                            && (space.kind == expected_kind
                                || (kind == "ramp" && space.kind == "service_access")) => {}
                    Some(_) => errors.push(format!("vertical core {id} space {space_id} has mismatched kind/floor")),
                    None => errors.push(format!("vertical core {id} references missing space {space_id}")),
                }
            }
            cores.insert(id, (kind, floor_refs, space_refs));
        }
    }
    for connection in array_at(fixture, "vertical_connections").into_iter().flatten() {
        let id = text_at(connection, "connection_id").unwrap_or("<missing-vertical-link>");
        let core_id = text_at(connection, "core_id").unwrap_or("<missing-core-id>");
        let from_floor = text_at(connection, "from_floor_id").unwrap_or("<missing-from-floor>");
        let to_floor = text_at(connection, "to_floor_id").unwrap_or("<missing-to-floor>");
        match cores.get(core_id) {
            Some((_, floors, _))
                if floors.contains(from_floor)
                    && floors.contains(to_floor)
                    && from_floor != to_floor
                    && floor_ids.contains(from_floor)
                    && floor_ids.contains(to_floor) => {}
            Some(_) => errors.push(format!("vertical connection {id} disagrees with core floor identity")),
            None => errors.push(format!("vertical connection {id} references missing core {core_id}")),
        }
    }

    let shared_services = array_at(fixture, "shared_services");
    let shared_service_ids = unique_text_rows(
        shared_services,
        "service_id",
        "shared_services",
        &mut errors,
    );
    if let Some(rows) = shared_services {
        for row in rows {
            let service_id = text_at(row, "service_id").unwrap_or("<missing-service-id>");
            let kind = text_at(row, "service_kind").unwrap_or("<missing-service-kind>");
            if !matches!(
                kind,
                "water" | "potable_water" | "non_potable_water" | "wastewater"
                    | "electricity" | "thermal" | "ventilation" | "communications"
                    | "waste" | "maintenance" | "delivery" | "other"
            ) {
                errors.push(format!("shared service {service_id} has invalid service_kind"));
            }
            let served = array_at(row, "served_unit_ids");
            if let Some(served) = served {
                let mut seen = BTreeSet::new();
                for unit in served.iter().filter_map(Value::as_str) {
                    if !seen.insert(unit.to_owned()) {
                        errors.push(format!("shared service {service_id} duplicates served unit {unit}"));
                    }
                    if !unit_ids.contains(unit) {
                        errors.push(format!("shared service {service_id} references unknown unit {unit}"));
                    }
                }
            } else {
                errors.push(format!("shared service {service_id} has no served_unit_ids array"));
            }
            let capacity_state = text_at(row, "capacity_state").unwrap_or("unknown");
            if capacity_state != "unknown"
                && row.get("capacity_evidence_ref").is_none_or(Value::is_null)
            {
                errors.push(format!("shared service {service_id} promotes capacity without evidence"));
            }
        }
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "MULTIFAMILY_SYNTHETIC_GEOMETRY_PASS" } else { "MULTIFAMILY_SYNTHETIC_GEOMETRY_FAIL" },
        "candidate_id": candidate_id,
        "floor_count": floor_ids.len(),
        "space_count": space_ids.len(),
        "unit_count": unit_ids.len(),
        "everyday_connection_count": connection_ids.len(),
        "vertical_core_count": core_ids.len(),
        "shared_service_count": shared_service_ids.len(),
        "authored_development_case_count": case_count,
        "physical_verification": "UNKNOWN",
        "claim_ceiling": "SYNTHETIC_FLOOR_LOCAL_RECTANGLE_PARTITION_AND_IDENTITY_ONLY"
    });
    if errors.is_empty() {
        Ok(report)
    } else {
        Err(errors)
    }
}

fn load_fixture() -> (Value, Value) {
    (
        serde_json::from_str(FIXTURE).expect("multi-family fixture JSON must parse"),
        serde_json::from_str(CORPUS).expect("multi-family corpus JSON must parse"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-multifamily-qualifier [--self-test]");
        return;
    }
    let (fixture, corpus) = load_fixture();
    match validate(&fixture, &corpus) {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default()),
        Err(errors) => {
            eprintln!("HOME_MULTIFAMILY_QUALIFIER_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_synthetic_multi_floor_fixture_qualifies() {
        let (fixture, corpus) = load_fixture();
        let report = validate(&fixture, &corpus).expect("synthetic multi-family fixture");
        assert_eq!(report["floor_count"], 2);
        assert_eq!(report["space_count"], 14);
        assert_eq!(report["unit_count"], 4);
        assert_eq!(report["everyday_connection_count"], 12);
        assert_eq!(report["vertical_core_count"], 2);
        assert_eq!(report["authored_development_case_count"], 47);
        assert_eq!(report["physical_verification"], "UNKNOWN");
    }

    #[test]
    fn duplicate_space_identity_is_rejected() {
        let (mut fixture, corpus) = load_fixture();
        fixture["building"]["spaces"][1]["space_id"] = json!("U-001");
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn positive_area_overlap_is_rejected() {
        let (mut fixture, corpus) = load_fixture();
        fixture["building"]["spaces"][1]["footprint"]["x_mm"] = json!(5900);
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn out_of_envelope_space_is_rejected() {
        let (mut fixture, corpus) = load_fixture();
        fixture["building"]["spaces"][0]["footprint"]["x_mm"] = json!(11900);
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn unallocated_floor_region_is_rejected() {
        let (mut fixture, corpus) = load_fixture();
        fixture["building"]["spaces"][6]["footprint"]["width_mm"] = json!(4000);
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn opening_not_on_shared_boundary_is_rejected() {
        let (mut fixture, corpus) = load_fixture();
        fixture["building"]["everyday_connections"][0]["portal"]["coordinate_mm"] = json!(7999);
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn unit_to_space_identity_mismatch_is_rejected() {
        let (mut fixture, corpus) = load_fixture();
        fixture["building"]["spaces"][0]["unit_id"] = json!("U-OTHER");
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn unknown_shared_service_unit_is_rejected() {
        let (fixture, mut corpus) = load_fixture();
        corpus["cases"][0]["case_id"] = json!("CORRUPT");
        let mut fixture = fixture;
        fixture["shared_services"][0]["served_unit_ids"][0] = json!("U-999");
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn vertical_core_floor_mismatch_is_rejected() {
        let (mut fixture, corpus) = load_fixture();
        fixture["building"]["vertical_cores"][0]["floor_ids"][1] = json!("F9");
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn professional_promotion_of_synthetic_fixture_is_rejected() {
        let (mut fixture, corpus) = load_fixture();
        fixture["maturity"] = json!("professionally_reviewed");
        assert!(validate(&fixture, &corpus).is_err());
    }

    #[test]
    fn duplicate_manifest_case_identity_is_rejected() {
        let (fixture, mut corpus) = load_fixture();
        corpus["cases"][1]["case_id"] = corpus["cases"][0]["case_id"].clone();
        assert!(validate(&fixture, &corpus).is_err());
    }
}
