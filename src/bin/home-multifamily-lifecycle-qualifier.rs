// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Synthetic multi-family lifecycle identity and claim-lineage qualifier.
//!
//! This checks apartment/asset identity, revision ancestry, logical event order,
//! unit-to-event scope, shared-asset served-unit membership and stale-claim
//! rejection. It does not establish that construction, installation, occupation,
//! observation, maintenance, repair or recommissioning actually occurred.

use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, process,
};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-lifecycle-synthetic-v0.1.json"
);
const BUILDING: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-building-synthetic-v0.1.json"
);
const PROGRAMME: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-resident-programme-synthetic-v0.1.json"
);
const UNIT_LAYOUT: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-unit-layout-synthetic-v0.1.json"
);
const SERVICE_TOPOLOGY: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-service-distribution-synthetic-v0.1.json"
);
const MUTATION_CORPUS: &str = include_str!(
    "../../docs/research/home-delivery-systems/multifamily-lifecycle-adversarial-corpus-v0.1.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn is_null(value: Option<&Value>) -> bool {
    value.is_none_or(Value::is_null)
}

fn unique_ids(
    items: Option<&Vec<Value>>,
    key: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(items) = items else {
        errors.push(format!("{label}: array is missing"));
        return ids;
    };
    for (index, item) in items.iter().enumerate() {
        let Some(id) = text_at(item, key) else {
            errors.push(format!("{label}[{index}]: {key} is missing"));
            continue;
        };
        if !ids.insert(id.to_owned()) {
            errors.push(format!("{label}: duplicate {key} {id}"));
        }
    }
    ids
}

fn valid_reference(value: &Value) -> bool {
    value.is_object()
        && text_at(value, "kind").is_some()
        && text_at(value, "id").is_some()
        && value.get("revision").is_some()
        && value.get("digest").is_some()
}

fn valid_revision_transition(parent_kind: &str, child_kind: &str) -> bool {
    match child_kind {
        "design" => false,
        "adaptation" => matches!(
            parent_kind,
            "design" | "adaptation" | "commissioned" | "as_operated" | "repair_state"
        ),
        "as_built" => matches!(parent_kind, "design" | "adaptation"),
        "commissioned" => matches!(parent_kind, "as_built" | "repair_state"),
        "as_operated" => matches!(parent_kind, "commissioned" | "as_operated" | "repair_state"),
        "repair_state" => matches!(parent_kind, "commissioned" | "as_operated" | "repair_state"),
        _ => false,
    }
}

fn downstream_served_units(source_node_id: &str, service_topology: &Value) -> BTreeSet<String> {
    let nodes = array_at(service_topology, "nodes")
        .cloned()
        .unwrap_or_default();
    let Some(source) = nodes
        .iter()
        .find(|node| text_at(node, "node_id") == Some(source_node_id))
    else {
        return BTreeSet::new();
    };
    let Some(resource_id) = text_at(source, "resource_id") else {
        return BTreeSet::new();
    };

    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for edge in array_at(service_topology, "edges").into_iter().flatten() {
        if text_at(edge, "relation") != Some("resource_flow") {
            continue;
        }
        let from = text_at(edge, "from_node_id").unwrap_or("");
        let to = text_at(edge, "to_node_id").unwrap_or("");
        let from_resource = nodes
            .iter()
            .find(|node| text_at(node, "node_id") == Some(from))
            .and_then(|node| text_at(node, "resource_id"));
        let to_resource = nodes
            .iter()
            .find(|node| text_at(node, "node_id") == Some(to))
            .and_then(|node| text_at(node, "resource_id"));
        if from_resource == Some(resource_id) && to_resource == Some(resource_id) {
            adjacency
                .entry(from.to_owned())
                .or_default()
                .push(to.to_owned());
        }
    }

    let mut pending = vec![source_node_id.to_owned()];
    let mut visited = BTreeSet::new();
    while let Some(node_id) = pending.pop() {
        if !visited.insert(node_id.clone()) {
            continue;
        }
        if let Some(next) = adjacency.get(&node_id) {
            pending.extend(next.iter().cloned());
        }
    }

    nodes
        .iter()
        .filter(|node| {
            text_at(node, "node_kind") == Some("unit_boundary")
                && text_at(node, "resource_id") == Some(resource_id)
                && text_at(node, "node_id").is_some_and(|id| visited.contains(id))
        })
        .filter_map(|node| text_at(node, "unit_id"))
        .map(str::to_owned)
        .collect()
}

fn event_revision_transition_is_valid(
    event_kind: &str,
    from_revision: &str,
    to_revision: &str,
    from_kind: &str,
    to_kind: &str,
) -> bool {
    match event_kind {
        "design_issue" => from_revision == to_revision && to_kind == "design",
        "design_change" => {
            from_revision != to_revision && matches!(to_kind, "design" | "adaptation")
        }
        "adaptation" => from_revision != to_revision && to_kind == "adaptation",
        "as_built_capture" => from_revision != to_revision && to_kind == "as_built",
        "commissioning" => from_revision != to_revision && to_kind == "commissioned",
        "occupancy_start" | "post_occupancy_observation" | "maintenance" => {
            from_revision == to_revision
                && matches!(to_kind, "commissioned" | "as_operated" | "repair_state")
        }
        "repair" | "reinspection" | "recommissioning" => {
            from_revision == to_revision
                || (from_revision != to_revision
                    && matches!(to_kind, "repair_state" | "commissioned" | "as_operated"))
        }
        _ => from_kind == to_kind || from_revision != to_revision,
    }
}

fn validate(
    fixture: &Value,
    building: &Value,
    programme: &Value,
    unit_layout: &Value,
    service_topology: &Value,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();

    if text_at(fixture, "schema_version") != Some("multifamily-lifecycle/0.1") {
        errors.push("unsupported multifamily lifecycle schema_version".to_owned());
    }
    if text_at(fixture, "maturity") != Some("synthetic_scenario_only") {
        errors
            .push("lifecycle fixture cannot be promoted beyond synthetic_scenario_only".to_owned());
    }
    if text_at(fixture, "building_id") != text_at(building, "candidate_id")
        || text_at(
            fixture
                .get("source_bindings")
                .unwrap_or(&Value::Null)
                .get("building_ref")
                .unwrap_or(&Value::Null),
            "id",
        ) != text_at(building, "candidate_id")
    {
        errors.push("lifecycle fixture is not bound to the exact building identity".to_owned());
    }
    if text_at(
        fixture
            .get("source_bindings")
            .unwrap_or(&Value::Null)
            .get("resident_programme_ref")
            .unwrap_or(&Value::Null),
        "id",
    ) != text_at(programme, "programme_id")
    {
        errors.push("lifecycle fixture is not bound to the exact resident programme".to_owned());
    }
    if text_at(
        fixture
            .get("source_bindings")
            .unwrap_or(&Value::Null)
            .get("unit_layout_ref")
            .unwrap_or(&Value::Null),
        "id",
    ) != text_at(unit_layout, "layout_id")
    {
        errors.push("lifecycle fixture is not bound to the exact unit-layout identity".to_owned());
    }
    if text_at(
        fixture
            .get("source_bindings")
            .unwrap_or(&Value::Null)
            .get("service_topology_ref")
            .unwrap_or(&Value::Null),
        "id",
    ) != text_at(service_topology, "profile_id")
    {
        errors.push(
            "lifecycle fixture is not bound to the exact service-topology identity".to_owned(),
        );
    }
    if text_at(fixture, "claim_ceiling").is_none() {
        errors.push("lifecycle claim_ceiling is missing".to_owned());
    }

    let building_units = array_at(building, "unit_programmes")
        .into_iter()
        .flatten()
        .filter_map(|row| text_at(row, "unit_id"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let programme_units = array_at(programme, "unit_programmes")
        .into_iter()
        .flatten()
        .filter_map(|row| text_at(row, "unit_id"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let layout_units = array_at(unit_layout, "unit_layouts")
        .into_iter()
        .flatten()
        .filter_map(|row| text_at(row, "unit_id"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let fixture_unit_ids = array_at(fixture, "unit_ids")
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if building_units.is_empty()
        || building_units != programme_units
        || building_units != layout_units
        || building_units != fixture_unit_ids
    {
        errors.push("building, resident-programme, unit-layout and lifecycle unit sets do not match exactly".to_owned());
    }

    let revision_rows = array_at(fixture, "revisions");
    let revision_ids = unique_ids(revision_rows, "revision_id", "revisions", &mut errors);
    let mut revisions = BTreeMap::<String, (String, Option<String>, String)>::new();
    if let Some(rows) = revision_rows {
        for row in rows {
            let id = text_at(row, "revision_id").unwrap_or("<missing-revision>");
            let unit_id = text_at(row, "unit_id").unwrap_or("<missing-unit>");
            let parent = text_at(row, "parent_revision_id").map(str::to_owned);
            let kind = text_at(row, "revision_kind").unwrap_or("<missing-kind>");
            if !building_units.contains(unit_id) {
                errors.push(format!("revision {id} references unknown unit {unit_id}"));
            }
            if !matches!(
                kind,
                "design"
                    | "adaptation"
                    | "as_built"
                    | "commissioned"
                    | "as_operated"
                    | "repair_state"
            ) {
                errors.push(format!("revision {id} has invalid revision_kind {kind}"));
            }
            if text_at(row, "revision_state") != Some("synthetic_identity_only") {
                errors.push(format!(
                    "revision {id} is promoted beyond synthetic identity"
                ));
            }
            if !is_null(row.get("artifact_ref")) {
                errors.push(format!(
                    "synthetic revision {id} cannot claim a real artifact reference"
                ));
            }
            if let Some(parent_id) = &parent {
                if parent_id == id {
                    errors.push(format!("revision {id} is its own parent"));
                }
            }
            revisions.insert(id.to_owned(), (unit_id.to_owned(), parent, kind.to_owned()));
        }
    }

    for (revision_id, (unit_id, parent, child_kind)) in &revisions {
        if let Some(parent_id) = parent {
            match revisions.get(parent_id) {
                Some((parent_unit, _, parent_kind)) if parent_unit == unit_id => {
                    if !valid_revision_transition(parent_kind, child_kind) {
                        errors.push(format!(
                            "revision {revision_id} has invalid transition {parent_kind} -> {child_kind}"
                        ));
                    }
                }
                Some(_) => errors.push(format!(
                    "revision {revision_id} parent belongs to a different unit"
                )),
                None => errors.push(format!(
                    "revision {revision_id} references missing parent {parent_id}"
                )),
            }
        } else if child_kind != "design" {
            errors.push(format!(
                "root revision {revision_id} must have revision_kind design"
            ));
        }
        let mut visited = BTreeSet::new();
        let mut cursor = Some(revision_id.clone());
        while let Some(id) = cursor {
            if !visited.insert(id.clone()) {
                errors.push(format!("revision ancestry cycle detected at {id}"));
                break;
            }
            cursor = revisions.get(&id).and_then(|record| record.1.clone());
        }
    }

    let service_node_records = array_at(service_topology, "nodes")
        .cloned()
        .unwrap_or_default();
    let service_nodes = service_node_records
        .iter()
        .filter_map(|node| text_at(node, "node_id"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();

    let asset_rows = array_at(fixture, "asset_bindings");
    let asset_ids = unique_ids(asset_rows, "asset_id", "asset_bindings", &mut errors);
    let mut assets = BTreeMap::<String, BTreeSet<String>>::new();
    if let Some(rows) = asset_rows {
        for asset in rows {
            let asset_id = text_at(asset, "asset_id").unwrap_or("<missing-asset>");
            let source_ref = asset.get("source_ref").unwrap_or(&Value::Null);
            let source_node_id = text_at(source_ref, "id").unwrap_or("");
            let source_node = service_node_records
                .iter()
                .find(|node| text_at(node, "node_id") == Some(source_node_id));
            if !valid_reference(source_ref)
                || text_at(source_ref, "kind") != Some("service_node")
                || !service_nodes.contains(source_node_id)
            {
                errors.push(format!(
                    "asset {asset_id} does not bind to an exact service-topology node"
                ));
            }
            let expected_node_kind = match text_at(asset, "asset_kind") {
                Some("water_riser" | "service_riser") => Some("riser"),
                Some("water_pump" | "hot_water_plant") => Some("plant"),
                Some("lift") => Some("lift"),
                Some("facade_module") => Some("other"),
                _ => None,
            };
            if expected_node_kind.is_none_or(|expected| {
                source_node.and_then(|node| text_at(node, "node_kind")) != Some(expected)
            }) {
                errors.push(format!(
                    "asset {asset_id} asset_kind does not match its source node kind"
                ));
            }
            if text_at(asset, "identity_state") != Some("identity_only_synthetic")
                || text_at(asset, "evidence_state") != Some("unknown")
                || !is_null(asset.get("current_revision_id"))
            {
                errors.push(format!(
                    "asset {asset_id} overstates synthetic asset or evidence maturity"
                ));
            }
            let served = array_at(asset, "served_unit_ids")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            if served.is_empty() || served.iter().any(|unit| !building_units.contains(unit)) {
                errors.push(format!(
                    "asset {asset_id} has an empty or unknown served-unit set"
                ));
            }
            let topology_served = downstream_served_units(source_node_id, service_topology);
            if served != topology_served {
                errors.push(format!(
                    "asset {asset_id} served-unit set disagrees with downstream service-topology reachability"
                ));
            }
            assets.insert(asset_id.to_owned(), served);
        }
    }

    let event_rows = array_at(fixture, "events");
    let event_ids = unique_ids(event_rows, "event_id", "events", &mut errors);
    let mut event_order_ids = BTreeSet::new();
    let mut event_records =
        BTreeMap::<String, (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>, i64)>::new();
    let mut prior_order = 0_i64;
    if let Some(rows) = event_rows {
        for event in rows {
            let id = text_at(event, "event_id").unwrap_or("<missing-event>");
            let order = event
                .get("event_order")
                .and_then(Value::as_i64)
                .unwrap_or(-1);
            if order <= 0 || order <= prior_order || !event_order_ids.insert(order) {
                errors.push(format!(
                    "event {id} has duplicate, invalid or unordered event_order"
                ));
            }
            prior_order = order;

            let event_kind = text_at(event, "event_kind").unwrap_or("<missing-kind>");
            if !matches!(
                event_kind,
                "design_issue"
                    | "design_change"
                    | "fabrication_release"
                    | "installation"
                    | "as_built_capture"
                    | "commissioning"
                    | "occupancy_start"
                    | "post_occupancy_observation"
                    | "maintenance"
                    | "repair"
                    | "adaptation"
                    | "reinspection"
                    | "recommissioning"
                    | "shared_asset_maintenance"
                    | "shared_asset_repair"
                    | "retirement"
            ) {
                errors.push(format!("event {id} has invalid event_kind {event_kind}"));
            }
            if text_at(event, "execution_state") != Some("authored_not_executed")
                || text_at(event, "event_time_state") != Some("not_supplied")
                || !is_null(event.get("occurred_at"))
                || text_at(event, "record_time_state") != Some("not_supplied")
                || !is_null(event.get("recorded_at"))
            {
                errors.push(format!(
                    "synthetic event {id} claims execution or an actual event/record time"
                ));
            }
            if text_at(event, "data_scope") != Some("synthetic_structure_only")
                || text_at(event, "resident_data_state") != Some("no_personal_data")
                || text_at(event, "consent_state") != Some("not_applicable")
            {
                errors.push(format!(
                    "synthetic event {id} promotes personal/resident data or consent"
                ));
            }
            if array_at(event, "evidence_refs").is_none_or(|refs| !refs.is_empty()) {
                errors.push(format!(
                    "synthetic event {id} cannot claim real evidence references"
                ));
            }

            let units = array_at(event, "unit_ids")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            let event_assets = array_at(event, "asset_ids")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            if units.is_empty() || units.iter().any(|unit| !building_units.contains(unit)) {
                errors.push(format!("event {id} has an empty or unknown unit scope"));
            }
            for asset_id in &event_assets {
                match assets.get(asset_id) {
                    Some(served) if units.is_subset(served) => {}
                    Some(_) => errors.push(format!(
                        "event {id} affects a unit outside asset {asset_id}'s served-unit set"
                    )),
                    None => errors.push(format!(
                        "event {id} references unknown shared asset {asset_id}"
                    )),
                }
            }

            let from_id = text_at(event, "from_revision_id");
            let to_id = text_at(event, "to_revision_id");
            match (from_id, to_id) {
                (Some(from), Some(to)) => {
                    let (Some(from_record), Some(to_record)) =
                        (revisions.get(from), revisions.get(to))
                    else {
                        errors.push(format!("event {id} references an unknown revision"));
                        continue;
                    };
                    if from_record.0 != to_record.0
                        || !units.contains(&from_record.0)
                        || units.len() != 1
                    {
                        errors.push(format!("event {id} revision transition is not bound to exactly one matching unit"));
                    }
                    if from != to && to_record.1.as_deref() != Some(from) {
                        errors.push(format!("event {id} transition does not follow the declared parent-revision edge"));
                    }
                    if !event_revision_transition_is_valid(
                        event_kind,
                        from,
                        to,
                        &from_record.2,
                        &to_record.2,
                    ) {
                        errors.push(format!(
                            "event {id} kind does not match its revision transition"
                        ));
                    }
                    if from == to && event_kind == "adaptation" {
                        errors.push(format!(
                            "adaptation event {id} does not create a new configuration revision"
                        ));
                    }
                }
                (None, None) => {
                    if !matches!(
                        text_at(event, "event_kind"),
                        Some("shared_asset_maintenance" | "shared_asset_repair")
                    ) {
                        errors.push(format!(
                            "event {id} is missing its unit-scoped revision transition"
                        ));
                    }
                }
                _ => errors.push(format!(
                    "event {id} has only one side of a revision transition"
                )),
            }

            let kind = text_at(event, "event_kind").unwrap_or("<missing-kind>");
            let impact_state =
                text_at(event, "change_impact_state").unwrap_or("<missing-impact-state>");
            if matches!(kind, "adaptation" | "design_change") {
                if !matches!(impact_state, "not_computed" | "owner_receipt_attached") {
                    errors.push(format!("change event {id} has invalid change-impact state"));
                }
                if text_at(event, "change_kind").is_none_or(str::is_empty) {
                    errors.push(format!("change event {id} has no typed change_kind"));
                }
            } else if impact_state != "not_applicable" {
                errors.push(format!(
                    "non-change event {id} incorrectly claims change-impact evaluation"
                ));
            }
            if impact_state == "owner_receipt_attached"
                && !valid_reference(event.get("change_impact_ref").unwrap_or(&Value::Null))
            {
                errors.push(format!(
                    "event {id} claims an owner change-impact receipt without a valid reference"
                ));
            }
            if impact_state != "owner_receipt_attached" && !is_null(event.get("change_impact_ref"))
            {
                errors.push(format!(
                    "event {id} attaches a change-impact receipt in an incompatible state"
                ));
            }

            let invalidated = array_at(event, "invalidated_claim_ids")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            event_records.insert(id.to_owned(), (units, event_assets, invalidated, order));
        }
    }

    let timeline_rows = array_at(fixture, "unit_timelines");
    let timeline_ids = unique_ids(timeline_rows, "unit_id", "unit_timelines", &mut errors);
    if timeline_ids != building_units {
        errors.push("unit_timelines do not exactly cover the building unit set".to_owned());
    }
    let mut current_revisions = BTreeMap::<String, String>::new();
    if let Some(rows) = timeline_rows {
        for timeline in rows {
            let unit_id = text_at(timeline, "unit_id").unwrap_or("<missing-unit>");
            let baseline_id =
                text_at(timeline, "baseline_revision_id").unwrap_or("<missing-baseline>");
            let current_id =
                text_at(timeline, "current_revision_id").unwrap_or("<missing-current>");
            match revisions.get(baseline_id) {
                Some((revision_unit, None, _)) if revision_unit == unit_id => {}
                _ => errors.push(format!(
                    "unit {unit_id} baseline revision is not an exact root revision"
                )),
            }
            match revisions.get(current_id) {
                Some((revision_unit, _, _)) if revision_unit == unit_id => {}
                _ => errors.push(format!(
                    "unit {unit_id} current revision does not belong to the unit"
                )),
            }
            let authored_timeline_event_ids = array_at(timeline, "event_ids")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let timeline_event_ids = authored_timeline_event_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if timeline_event_ids.len() != authored_timeline_event_ids.len() {
                errors.push(format!("unit {unit_id} timeline duplicates an event ID"));
            }
            let expected_event_ids = event_records
                .iter()
                .filter(|(_, (units, _, _, _))| units.contains(unit_id))
                .map(|(event_id, _)| event_id.clone())
                .collect::<BTreeSet<_>>();
            if timeline_event_ids != expected_event_ids {
                errors.push(format!(
                    "unit {unit_id} timeline does not contain its exact event scope"
                ));
            }
            let mut prior_event_order = 0_i64;
            let mut last_revision = None::<String>;
            for event_id in array_at(timeline, "event_ids")
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                match event_records.get(event_id) {
                    Some((units, _, _, event_order)) if units.contains(unit_id) => {
                        if *event_order <= prior_event_order {
                            errors.push(format!(
                                "unit {unit_id} timeline event order is not strictly increasing"
                            ));
                        }
                        prior_event_order = *event_order;
                        if let Some(event) = event_rows
                            .into_iter()
                            .flatten()
                            .find(|event| text_at(event, "event_id") == Some(event_id))
                        {
                            if let Some(to) = text_at(event, "to_revision_id") {
                                last_revision = Some(to.to_owned());
                            }
                        }
                    }
                    Some(_) => errors.push(format!(
                        "unit {unit_id} references an event outside its scope"
                    )),
                    None => errors.push(format!(
                        "unit {unit_id} references missing event {event_id}"
                    )),
                }
            }
            if last_revision.as_deref() != Some(current_id) {
                errors.push(format!("unit {unit_id} current_revision_id does not match its latest timeline transition"));
            }
            current_revisions.insert(unit_id.to_owned(), current_id.to_owned());
        }
    }

    let claim_rows = array_at(fixture, "claims");
    let claim_ids = unique_ids(claim_rows, "claim_id", "claims", &mut errors);
    let mut claim_records = BTreeMap::<String, (String, String, String, Option<String>)>::new();
    if let Some(rows) = claim_rows {
        for claim in rows {
            let id = text_at(claim, "claim_id").unwrap_or("<missing-claim>");
            let unit_id = text_at(claim, "unit_id").unwrap_or("<missing-unit>");
            let revision_id = text_at(claim, "revision_id").unwrap_or("<missing-revision>");
            let state = text_at(claim, "claim_state").unwrap_or("<missing-state>");
            let invalidated_by = text_at(claim, "invalidated_by_event_id").map(str::to_owned);
            if !building_units.contains(unit_id) {
                errors.push(format!("claim {id} references unknown unit {unit_id}"));
            }
            match revisions.get(revision_id) {
                Some((revision_unit, _, _)) if revision_unit == unit_id => {}
                _ => errors.push(format!(
                    "claim {id} revision is missing or belongs to another unit"
                )),
            }
            if text_at(claim, "claim_scope") != Some("synthetic_claim_identity_only") {
                errors.push(format!(
                    "claim {id} is promoted beyond synthetic identity scope"
                ));
            }
            if array_at(claim, "evidence_refs").is_none_or(Vec::is_empty)
                && !matches!(
                    state,
                    "not_assessed" | "unknown" | "invalidated" | "superseded" | "revoked"
                )
            {
                errors.push(format!("claim {id} promotes a result without evidence"));
            }
            if matches!(state, "pass" | "fail") {
                errors.push(format!(
                    "synthetic lifecycle fixture cannot claim PASS or FAIL for {id}"
                ));
            }
            if state == "invalidated" {
                if invalidated_by.is_none() {
                    errors.push(format!("invalidated claim {id} has no invalidating event"));
                }
            } else if invalidated_by.is_some() {
                errors.push(format!(
                    "non-invalidated claim {id} carries invalidated_by_event_id"
                ));
            }
            if let Some(current) = current_revisions.get(unit_id) {
                if current != revision_id
                    && !matches!(state, "invalidated" | "superseded" | "revoked")
                {
                    errors.push(format!(
                        "stale claim {id} is not invalidated/superseded/revoked"
                    ));
                }
            }
            claim_records.insert(
                id.to_owned(),
                (
                    unit_id.to_owned(),
                    revision_id.to_owned(),
                    state.to_owned(),
                    invalidated_by,
                ),
            );
        }
    }

    for (event_id, (units, _, invalidated_claim_ids, _)) in &event_records {
        for claim_id in invalidated_claim_ids {
            match claim_records.get(claim_id) {
                Some((unit_id, revision_id, state, invalidated_by))
                    if units.contains(unit_id)
                        && state == "invalidated"
                        && invalidated_by.as_deref() == Some(event_id.as_str())
                        && current_revisions
                            .get(unit_id)
                            .is_some_and(|current| current != revision_id) => {}
                Some(_) => errors.push(format!(
                    "event {event_id} invalidation list disagrees with claim {claim_id}"
                )),
                None => errors.push(format!(
                    "event {event_id} invalidates unknown claim {claim_id}"
                )),
            }
        }
    }
    for (claim_id, (unit_id, _, state, invalidated_by)) in &claim_records {
        if state == "invalidated" {
            match invalidated_by
                .as_ref()
                .and_then(|event_id| event_records.get(event_id).map(|record| (event_id, record)))
            {
                Some((event_id, (units, _, invalidated, _)))
                    if units.contains(unit_id) && invalidated.contains(claim_id) => {}
                _ => errors.push(format!(
                    "invalidated claim {claim_id} has no reciprocal event record"
                )),
            }
        }
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "MULTIFAMILY_LIFECYCLE_IDENTITY_PASS" } else { "MULTIFAMILY_LIFECYCLE_IDENTITY_FAIL" },
        "building_id": text_at(fixture, "building_id").unwrap_or("<missing>"),
        "unit_count": building_units.len(),
        "revision_count": revision_ids.len(),
        "asset_binding_count": asset_ids.len(),
        "event_count": event_ids.len(),
        "unit_timeline_count": timeline_ids.len(),
        "claim_count": claim_ids.len(),
        "stale_pass_count": 0,
        "physical_lifecycle": "NOT_ESTABLISHED",
        "as_built_state": "UNKNOWN",
        "commissioning_state": "UNKNOWN",
        "post_occupancy_state": "UNKNOWN",
        "repair_state": "UNKNOWN",
        "claim_ceiling": "SYNTHETIC_IDENTITY_REVISION_EVENT_AND_CLAIM_STATE_ONLY"
    });
    if errors.is_empty() {
        Ok(report)
    } else {
        Err(errors)
    }
}

fn replay_mutation_corpus(
    corpus: &Value,
    fixture: &Value,
    building: &Value,
    programme: &Value,
    unit_layout: &Value,
    service_topology: &Value,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(corpus, "schema_version") != Some("multifamily-lifecycle-adversarial-corpus/0.1") {
        errors.push("lifecycle mutation corpus has unsupported schema_version".to_owned());
    }
    if text_at(corpus, "maturity") != Some("authored_manifest_not_executed") {
        errors.push(
            "lifecycle mutation corpus must remain an authored development manifest".to_owned(),
        );
    }
    if text_at(corpus, "base_fixture") != Some("multifamily-lifecycle-synthetic-v0.1.json") {
        errors.push("lifecycle mutation corpus is not bound to the exact base fixture".to_owned());
    }
    if corpus.get("executed_cases").and_then(Value::as_u64) != Some(0) {
        errors.push(
            "authored lifecycle mutation corpus must not claim persisted execution".to_owned(),
        );
    }

    let cases = array_at(corpus, "cases");
    let case_ids = unique_ids(cases, "case_id", "lifecycle mutation cases", &mut errors);
    let declared_count = corpus
        .get("case_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if declared_count != 30 || case_ids.len() != 30 {
        errors.push(format!(
            "lifecycle mutation corpus requires 30 cases; declared {declared_count}, found {}",
            case_ids.len()
        ));
    }

    if let Some(split) = corpus.get("case_split") {
        let development_ids = array_at(split, "development_cases")
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if development_ids != case_ids {
            errors.push(
                "lifecycle mutation development split does not exactly match case IDs".to_owned(),
            );
        }
        if !array_at(split, "calibration_cases").is_some_and(Vec::is_empty)
            || !array_at(split, "sealed_confirmation_cases").is_some_and(Vec::is_empty)
        {
            errors.push(
                "lifecycle mutation corpus must not claim calibration or sealed cases".to_owned(),
            );
        }
    } else {
        errors.push("lifecycle mutation corpus has no case_split".to_owned());
    }

    let mut positive_count = 0usize;
    let mut adversarial_count = 0usize;
    let mut replayed_count = 0usize;
    let mut passed_negative_count = 0usize;

    if let Some(cases) = cases {
        for case in cases {
            let case_id = text_at(case, "case_id").unwrap_or("<missing-case>");
            if text_at(case, "execution_state") != Some("not_executed")
                || !case.get("probability").is_some_and(Value::is_null)
            {
                errors.push(format!(
                    "lifecycle mutation case {case_id} must remain unexecuted with null probability"
                ));
            }
            let mutations = array_at(case, "mutations");
            let mutation_rows = mutations.cloned().unwrap_or_default();
            let disposition =
                text_at(case, "expected_disposition").unwrap_or("<missing-disposition>");
            match text_at(case, "case_kind") {
                Some("positive_control") => {
                    positive_count += 1;
                    if disposition != "ACCEPT_SYNTHETIC_BASELINE" || !mutation_rows.is_empty() {
                        errors.push(format!(
                            "positive lifecycle case {case_id} must be an unmutated accept control"
                        ));
                    }
                }
                Some("adversarial_mutation") => {
                    adversarial_count += 1;
                    if disposition != "REJECT_SYNTHETIC_MUTATION" || mutation_rows.is_empty() {
                        errors.push(format!("adversarial lifecycle case {case_id} must contain mutations and expect rejection"));
                    }
                }
                _ => errors.push(format!(
                    "lifecycle mutation case {case_id} has invalid case_kind"
                )),
            }

            let mut candidate = fixture.clone();
            let mut seen_pointers = BTreeSet::new();
            for mutation in &mutation_rows {
                let pointer = text_at(mutation, "pointer").unwrap_or("");
                if !pointer.starts_with("/") || !seen_pointers.insert(pointer.to_owned()) {
                    errors.push(format!(
                        "lifecycle mutation case {case_id} has invalid or duplicate JSON pointer"
                    ));
                    continue;
                }
                let Some(value) = mutation.get("value") else {
                    errors.push(format!(
                        "lifecycle mutation case {case_id} mutation has no value"
                    ));
                    continue;
                };
                match candidate.pointer_mut(pointer) {
                    Some(target) => *target = value.clone(),
                    None => errors.push(format!(
                        "lifecycle mutation case {case_id} pointer {pointer} does not resolve"
                    )),
                }
            }

            let actual_accepts = validate(
                &candidate,
                building,
                programme,
                unit_layout,
                service_topology,
            )
            .is_ok();
            replayed_count += 1;
            match disposition {
                "ACCEPT_SYNTHETIC_BASELINE" if actual_accepts => {}
                "REJECT_SYNTHETIC_MUTATION" if !actual_accepts => {
                    passed_negative_count += 1;
                }
                "ACCEPT_SYNTHETIC_BASELINE" => errors.push(format!(
                    "lifecycle mutation case {case_id} unexpectedly rejected its positive control"
                )),
                "REJECT_SYNTHETIC_MUTATION" => errors.push(format!(
                    "lifecycle mutation case {case_id} was not rejected by the independent qualifier"
                )),
                _ => errors.push(format!(
                    "lifecycle mutation case {case_id} has unsupported expected_disposition {disposition}"
                )),
            }
        }
    }

    if positive_count != 1 || adversarial_count != 29 {
        errors.push(format!(
            "lifecycle mutation corpus requires one positive and 29 adversarial cases; found {positive_count} and {adversarial_count}"
        ));
    }
    if replayed_count != 30 || passed_negative_count != 29 {
        errors.push(format!(
            "lifecycle mutation replay incomplete: replayed {replayed_count}, negative controls rejected {passed_negative_count}"
        ));
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "LIFECYCLE_MUTATION_CORPUS_PASS" } else { "LIFECYCLE_MUTATION_CORPUS_FAIL" },
        "case_count": case_ids.len(),
        "replayed_case_count": replayed_count,
        "positive_control_count": positive_count,
        "adversarial_control_count": adversarial_count,
        "adversarial_controls_rejected": passed_negative_count,
        "sealed_confirmation_cases": 0,
        "physical_lifecycle": "NOT_ESTABLISHED",
        "claim_ceiling": "SYNTHETIC_IDENTITY_REVISION_EVENT_MUTATION_REPLAY_ONLY"
    });
    if errors.is_empty() {
        Ok(report)
    } else {
        Err(errors)
    }
}

fn load_mutation_corpus() -> Value {
    serde_json::from_str(MUTATION_CORPUS).expect("lifecycle mutation corpus JSON must parse")
}

fn load_fixtures() -> (Value, Value, Value, Value, Value) {
    (
        serde_json::from_str(FIXTURE).expect("lifecycle fixture JSON must parse"),
        serde_json::from_str(BUILDING).expect("building fixture JSON must parse"),
        serde_json::from_str(PROGRAMME).expect("resident programme JSON must parse"),
        serde_json::from_str(UNIT_LAYOUT).expect("unit layout JSON must parse"),
        serde_json::from_str(SERVICE_TOPOLOGY).expect("service topology JSON must parse"),
    )
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-multifamily-lifecycle-qualifier [--self-test]");
        return;
    }
    let (fixture, building, programme, unit_layout, service_topology) = load_fixtures();
    let corpus = load_mutation_corpus();
    let base_report = validate(
        &fixture,
        &building,
        &programme,
        &unit_layout,
        &service_topology,
    );
    let mutation_report = replay_mutation_corpus(
        &corpus,
        &fixture,
        &building,
        &programme,
        &unit_layout,
        &service_topology,
    );
    match (base_report, mutation_report) {
        (Ok(base), Ok(mutations)) => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "base_fixture": base,
                "mutation_corpus": mutations
            }))
            .unwrap_or_default()
        ),
        (Err(errors), _) => {
            eprintln!(
                "HOME_MULTIFAMILY_LIFECYCLE_QUALIFIER_FAIL: {}",
                errors.join("; ")
            );
            process::exit(1);
        }
        (_, Err(errors)) => {
            eprintln!(
                "HOME_MULTIFAMILY_LIFECYCLE_MUTATION_REPLAY_FAIL: {}",
                errors.join("; ")
            );
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_lifecycle_lineage_qualifies_without_physical_claims() {
        let (fixture, building, programme, layout, services) = load_fixtures();
        let report = validate(&fixture, &building, &programme, &layout, &services)
            .expect("synthetic lifecycle identity fixture");
        assert_eq!(report["unit_count"], 4);
        assert_eq!(report["revision_count"], 13);
        assert_eq!(report["asset_binding_count"], 2);
        assert_eq!(report["event_count"], 24);
        assert_eq!(report["unit_timeline_count"], 4);
        assert_eq!(report["claim_count"], 7);
        assert_eq!(report["physical_lifecycle"], "NOT_ESTABLISHED");
    }

    #[test]
    fn frozen_30_case_mutation_corpus_matches_known_answers() {
        let (fixture, building, programme, layout, services) = load_fixtures();
        let corpus = load_mutation_corpus();
        let report =
            replay_mutation_corpus(&corpus, &fixture, &building, &programme, &layout, &services)
                .expect("30-case lifecycle mutation corpus");
        assert_eq!(report["case_count"], 30);
        assert_eq!(report["positive_control_count"], 1);
        assert_eq!(report["adversarial_control_count"], 29);
        assert_eq!(report["adversarial_controls_rejected"], 29);
        assert_eq!(report["sealed_confirmation_cases"], 0);
    }

    #[test]
    fn duplicate_revision_identity_is_rejected() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["revisions"][1]["revision_id"] = fixture["revisions"][0]["revision_id"].clone();
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn broken_parent_revision_chain_is_rejected() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["revisions"][1]["parent_revision_id"] = json!("NO-SUCH-REVISION");
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn revision_ancestry_cycle_is_rejected() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["revisions"][0]["parent_revision_id"] = json!("U-001-R1");
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn wrong_unit_event_scope_is_rejected() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][0]["unit_ids"] = json!(["U-999"]);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn unknown_shared_asset_is_rejected() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][22]["asset_ids"] = json!(["ASSET-UNKNOWN"]);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn shared_asset_cannot_be_applied_to_unserved_unit() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][22]["unit_ids"] = json!(["U-001", "U-003"]);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn stale_pass_after_adaptation_is_rejected() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["claims"][0]["claim_state"] = json!("pass");
        fixture["claims"][0]["invalidated_by_event_id"] = json!(null);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn missing_invalidation_reciprocity_is_rejected() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][1]["invalidated_claim_ids"] = json!([]);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn event_time_or_record_time_cannot_be_inferred() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][0]["occurred_at"] = json!("2026-01-01T00:00:00Z");
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn synthetic_event_cannot_be_promoted_to_executed() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][0]["execution_state"] = json!("executed");
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn event_order_must_be_monotonic_and_unique() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][1]["event_order"] = json!(1);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn unit_timeline_cannot_omit_an_authored_event() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["unit_timelines"][0]["event_ids"]
            .as_array_mut()
            .unwrap()
            .remove(1);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn unit_timeline_must_cover_every_scoped_event() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["unit_timelines"][0]["event_ids"]
            .as_array_mut()
            .unwrap()
            .remove(1);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn unknown_event_kind_is_rejected() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][0]["event_kind"] = json!("some_untyped_event");
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn current_revision_must_match_last_timeline_transition() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["unit_timelines"][0]["current_revision_id"] = json!("U-001-R1");
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn stale_unknown_claim_must_be_invalidated_or_superseded() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["claims"][0]["claim_state"] = json!("unknown");
        fixture["claims"][0]["invalidated_by_event_id"] = json!(null);
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn invalidated_claim_requires_a_matching_change_event() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["claims"][0]["invalidated_by_event_id"] = json!("EV-U-001-DESIGN");
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }

    #[test]
    fn resident_data_cannot_be_smuggled_into_synthetic_lifecycle_fixture() {
        let (mut fixture, building, programme, layout, services) = load_fixtures();
        fixture["events"][5]["resident_data_state"] = json!("personal_data_present");
        fixture["events"][5]["consent_state"] = json!("obtained");
        assert!(validate(&fixture, &building, &programme, &layout, &services).is_err());
    }
}
