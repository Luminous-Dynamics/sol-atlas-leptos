//! Bounded synthetic reference oracle for effective urban service access.
//!
//! This tool is deliberately separate from the Leptos UI and production GIS.
//! It computes network paths and declared service usability for synthetic
//! fixtures only. Group-level reachable capacity is potential, not an
//! allocation. City/category supply is counted once per facility.
//!
//! No output is a real-world accessibility, safety, affordability, planning,
//! land-rights, community-consent, or city-quality determination.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Disposition {
    FeasibleForSyntheticCase,
    Unsupported,
    Unknown,
    Infeasible,
}
impl Disposition {
    fn as_str(self) -> &'static str {
        match self {
            Self::FeasibleForSyntheticCase => "FeasibleForSyntheticCase",
            Self::Unsupported => "Unsupported",
            Self::Unknown => "Unknown",
            Self::Infeasible => "Infeasible",
        }
    }
}

fn array<'a>(value: &'a Value, key: &str) -> Option<&'a [Value]> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
}
fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}
fn number(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|x| x.is_finite())
}
fn nonnegative(value: &Value, key: &str) -> Option<f64> {
    number(value, key).filter(|x| *x >= 0.0)
}
fn bool_value(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}
fn string_array<'a>(value: &'a Value, key: &str) -> Option<Vec<&'a str>> {
    value
        .get(key)?
        .as_array()?
        .iter()
        .map(Value::as_str)
        .collect()
}
/// Validate the Johannesburg source inventory without implying that any
/// layer has been acquired, licensed for redistribution, or qualified.
pub fn validate_source_register(register: &Value) -> Result<usize, Vec<String>> {
    let mut errors = BTreeSet::<String>::new();

    if string(register, "schema_id") != "org.luminousdynamics.citymorph.source-register"
        || string(register, "schema_version") != "0.1.0"
        || string(register, "register_id") != "JOHANNESBURG-SOURCE-REGISTER-V0.1"
        || string(register, "classification") != "research_inventory_not_ingested_data"
        || string(register, "as_of").is_empty()
    {
        errors.insert("SOURCE_REGISTER_IDENTITY_INVALID".to_owned());
    }

    let access_states = array(register, "access_status_values")
        .unwrap_or_default()
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let rights_states = array(register, "rights_status_values")
        .unwrap_or_default()
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();

    if access_states.is_empty() {
        errors.insert("SOURCE_REGISTER_ACCESS_STATES_MISSING".to_owned());
    }
    if rights_states.is_empty() {
        errors.insert("SOURCE_REGISTER_RIGHTS_STATES_MISSING".to_owned());
    }

    let Some(sources) = array(register, "sources") else {
        errors.insert("SOURCE_REGISTER_SOURCES_MISSING".to_owned());
        return Err(errors.into_iter().collect());
    };
    if sources.is_empty() {
        errors.insert("SOURCE_REGISTER_SOURCES_EMPTY".to_owned());
    }

    let mut source_ids = BTreeSet::<String>::new();
    for source in sources {
        let source_id = string(source, "source_id");
        if source_id.is_empty() || !source_ids.insert(source_id.to_owned()) {
            errors.insert("SOURCE_ID_MISSING_OR_DUPLICATE".to_owned());
        }
        if string(source, "name").is_empty() || string(source, "source_type").is_empty() {
            errors.insert(format!("SOURCE_METADATA_MISSING:{source_id}"));
        }

        let access_status = string(source, "access_status");
        let rights_status = string(source, "rights_status");
        if !access_states.contains(access_status) {
            errors.insert(format!("SOURCE_ACCESS_STATUS_UNSUPPORTED:{source_id}"));
        }
        if !rights_states.contains(rights_status) {
            errors.insert(format!("SOURCE_RIGHTS_STATUS_UNSUPPORTED:{source_id}"));
        }

        match source.get("url") {
            Some(Value::String(url))
                if url.starts_with("https://") || url.starts_with("http://") => {}
            Some(Value::Null) if access_status == "Unknown" => {}
            _ => {
                errors.insert(format!("SOURCE_URL_INVALID_OR_UNVERIFIED:{source_id}"));
            }
        }

        if rights_status == "VerifiedOpenLicense" && string(source, "license").is_empty() {
            errors.insert(format!("OPEN_LICENSE_WITHOUT_TERMS:{source_id}"));
        }

        let Some(required_checks) = array(source, "required_checks") else {
            errors.insert(format!("SOURCE_REQUIRED_CHECKS_MISSING:{source_id}"));
            continue;
        };
        if required_checks.is_empty()
            || required_checks
                .iter()
                .any(|value| value.as_str().is_none_or(str::is_empty))
        {
            errors.insert(format!("SOURCE_REQUIRED_CHECKS_INVALID:{source_id}"));
        }

        let Some(evidence_urls) = array(source, "evidence_urls") else {
            errors.insert(format!("SOURCE_EVIDENCE_URLS_MISSING:{source_id}"));
            continue;
        };
        if evidence_urls.is_empty()
            || evidence_urls.iter().any(|value| {
                !value
                    .as_str()
                    .is_some_and(|url| url.starts_with("https://") || url.starts_with("http://"))
            })
        {
            errors.insert(format!("SOURCE_EVIDENCE_URLS_INVALID:{source_id}"));
        }
    }

    if string(register, "claim_ceiling").is_empty() {
        errors.insert("SOURCE_REGISTER_CLAIM_CEILING_MISSING".to_owned());
    }

    if errors.is_empty() {
        Ok(sources.len())
    } else {
        Err(errors.into_iter().collect())
    }
}

fn empty_result(disposition: Disposition, reasons: BTreeSet<String>) -> Value {
    json!({
        "disposition": disposition.as_str(),
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "groups": [],
        "category_totals": []
    })
}

/// Dijkstra over the small, finite fixture graph. Unknown edges are admitted
/// only for the "possible" search. Closed edges are never traversed.
fn shortest_path(
    network: &Value,
    origin: &str,
    destination: &str,
    mode: &str,
    requires_step_free: bool,
    allow_unknown: bool,
) -> Option<f64> {
    if origin.is_empty() || destination.is_empty() {
        return None;
    }
    if !allow_unknown && string(network, "evidence_state") != "current" {
        return None;
    }
    let nodes = array(network, "nodes")?;
    if !nodes.iter().any(|node| node.as_str() == Some(origin))
        || !nodes.iter().any(|node| node.as_str() == Some(destination))
    {
        return None;
    }
    let edges = array(network, "edges")?;
    let mut distances = BTreeMap::<String, f64>::from([(origin.to_owned(), 0.0)]);
    let mut visited = BTreeSet::<String>::new();

    loop {
        let next = distances
            .iter()
            .filter(|(node, _)| !visited.contains(*node))
            .min_by(|left, right| left.1.total_cmp(right.1))
            .map(|(node, distance)| (node.clone(), *distance));
        let (current, distance) = next?;
        if current == destination {
            return Some(distance);
        }
        visited.insert(current.clone());

        for edge in edges {
            let from = string(edge, "from");
            let to = string(edge, "to");
            let one_way = bool_value(edge, "one_way").unwrap_or(false);
            let neighbor = if from == current {
                to
            } else if !one_way && to == current {
                from
            } else {
                continue;
            };
            if neighbor.is_empty() {
                continue;
            }

            match string(edge, "status") {
                "open" => {}
                "unknown" if allow_unknown => {}
                _ => continue,
            }

            if requires_step_free {
                match edge.get("step_free").and_then(Value::as_bool) {
                    Some(true) => {}
                    None if allow_unknown => {}
                    _ => continue,
                }
            }

            let field = match mode {
                "walk" => "walk_minutes",
                "cycle" => "cycle_minutes",
                _ => return None,
            };
            let cost = nonnegative(edge, field)?;
            let candidate = distance + cost;
            if candidate < distances.get(neighbor).copied().unwrap_or(f64::INFINITY) {
                distances.insert(neighbor.to_owned(), candidate);
            }
        }
    }
}

#[derive(Default)]
struct GroupResult {
    status: String,
    nearest_minutes: Option<f64>,
    accessible_services: BTreeSet<String>,
    unknown_services: BTreeSet<String>,
    capacity: f64,
    reason_codes: BTreeSet<String>,
}
impl GroupResult {
    fn finish(self, group: &Value) -> Value {
        json!({
            "group_id": string(group, "id"),
            "status": self.status,
            "nearest_known_service_minutes": self.nearest_minutes,
            "accessible_service_ids": self.accessible_services.into_iter().collect::<Vec<_>>(),
            "unknown_service_ids": self.unknown_services.into_iter().collect::<Vec<_>>(),
            "reachable_capacity_potential": self.capacity,
            "demand": number(group, "demand").unwrap_or(0.0),
            "demand_unit": string(group, "demand_unit"),
            "reason_codes": self.reason_codes.into_iter().collect::<Vec<_>>()
        })
    }
}

pub fn evaluate_case(case: &Value) -> Value {
    let input = case.get("input").unwrap_or(&Value::Null);
    let network = input.get("network").unwrap_or(&Value::Null);
    let mut unsupported = BTreeSet::<String>::new();

    let requested_window = string(input, "requested_window");
    if requested_window.is_empty() {
        unsupported.insert("REQUESTED_WINDOW_MISSING".to_owned());
    }
    let network_state = string(network, "evidence_state");
    if !matches!(network_state, "current" | "stale") {
        unsupported.insert("NETWORK_EVIDENCE_STATE_UNSUPPORTED".to_owned());
    }

    let Some(nodes) = array(network, "nodes") else {
        unsupported.insert("NETWORK_NODES_MISSING".to_owned());
        return empty_result(Disposition::Unsupported, unsupported);
    };
    let mut node_ids = BTreeSet::new();
    for node in nodes {
        let Some(id) = node.as_str() else {
            unsupported.insert("NETWORK_NODE_ID_INVALID".to_owned());
            continue;
        };
        if id.is_empty() || !node_ids.insert(id) {
            unsupported.insert("NETWORK_NODE_ID_INVALID_OR_DUPLICATE".to_owned());
        }
    }
    let Some(edges) = array(network, "edges") else {
        unsupported.insert("NETWORK_EDGES_MISSING".to_owned());
        return empty_result(Disposition::Unsupported, unsupported);
    };
    let mut edge_ids = BTreeSet::new();
    for edge in edges {
        let id = string(edge, "id");
        if id.is_empty() || !edge_ids.insert(id) {
            unsupported.insert("NETWORK_EDGE_ID_MISSING_OR_DUPLICATE".to_owned());
        }
        let from = string(edge, "from");
        let to = string(edge, "to");
        if !node_ids.contains(from) || !node_ids.contains(to) {
            unsupported.insert("NETWORK_EDGE_ENDPOINT_UNKNOWN".to_owned());
        }
        if !matches!(string(edge, "status"), "open" | "closed" | "unknown") {
            unsupported.insert("NETWORK_EDGE_STATUS_UNSUPPORTED".to_owned());
        }
        if !number(edge, "walk_minutes").is_some_and(|x| x >= 0.0)
            || !number(edge, "cycle_minutes").is_some_and(|x| x >= 0.0)
        {
            unsupported.insert("NETWORK_EDGE_COST_INVALID".to_owned());
        }
        if edge.get("step_free").is_none()
            || !(edge
                .get("step_free")
                .is_some_and(|v| v.is_boolean() || v.is_null()))
        {
            unsupported.insert("NETWORK_EDGE_STEP_FREE_STATE_INVALID".to_owned());
        }
        if edge.get("one_way").and_then(Value::as_bool).is_none() {
            unsupported.insert("NETWORK_EDGE_DIRECTION_INVALID".to_owned());
        }
    }

    let Some(groups) = array(input, "population_groups") else {
        unsupported.insert("POPULATION_GROUPS_MISSING".to_owned());
        return empty_result(Disposition::Unsupported, unsupported);
    };
    let Some(services) = array(input, "services") else {
        unsupported.insert("SERVICES_MISSING".to_owned());
        return empty_result(Disposition::Unsupported, unsupported);
    };

    let mut group_ids = BTreeSet::new();
    for group in groups {
        let id = string(group, "id");
        if id.is_empty() || !group_ids.insert(id) {
            unsupported.insert("DUPLICATE_POPULATION_GROUP_ID".to_owned());
        }
        if !node_ids.contains(string(group, "origin_node")) {
            unsupported.insert("POPULATION_GROUP_ORIGIN_UNKNOWN".to_owned());
        }
        if !matches!(string(group, "mode"), "walk" | "cycle") {
            unsupported.insert("POPULATION_GROUP_MODE_UNSUPPORTED".to_owned());
        }
        if string(group, "service_category").is_empty()
            || string(group, "demand_unit").is_empty()
            || string(group, "price_unit").is_empty()
        {
            unsupported.insert("POPULATION_GROUP_SEMANTICS_MISSING".to_owned());
        }
        if group
            .get("population")
            .and_then(Value::as_u64)
            .is_none_or(|x| x == 0)
            || !number(group, "demand").is_some_and(|x| x > 0.0)
            || nonnegative(group, "max_travel_minutes").is_none()
            || nonnegative(group, "max_service_price").is_none()
            || bool_value(group, "requires_step_free").is_none()
        {
            unsupported.insert("POPULATION_GROUP_QUANTITY_OR_ACCESS_PROFILE_INVALID".to_owned());
        }
    }

    let mut service_ids = BTreeSet::new();
    for service in services {
        let id = string(service, "id");
        if id.is_empty() || !service_ids.insert(id) {
            unsupported.insert("DUPLICATE_SERVICE_ID".to_owned());
        }
        if !node_ids.contains(string(service, "node")) {
            unsupported.insert("SERVICE_NODE_UNKNOWN".to_owned());
        }
        if string(service, "category").is_empty()
            || string(service, "capacity_unit").is_empty()
            || string(service, "price_unit").is_empty()
        {
            unsupported.insert("SERVICE_SEMANTICS_MISSING".to_owned());
        }
        if !matches!(
            string(service, "capacity_state"),
            "current" | "stale" | "missing" | "unknown"
        ) {
            unsupported.insert("SERVICE_CAPACITY_STATE_UNSUPPORTED".to_owned());
        }
        if string(service, "capacity_state") == "current"
            && nonnegative(service, "capacity").is_none()
        {
            unsupported.insert("SERVICE_CAPACITY_VALUE_INVALID".to_owned());
        }
        if !matches!(
            string(service, "availability_state"),
            "open" | "closed" | "unknown"
        ) {
            unsupported.insert("SERVICE_AVAILABILITY_STATE_UNSUPPORTED".to_owned());
        }
        if array(service, "open_windows").is_none() {
            unsupported.insert("SERVICE_OPEN_WINDOWS_MISSING".to_owned());
        }
        if !matches!(
            string(service, "price_state"),
            "current" | "unknown" | "stale"
        ) {
            unsupported.insert("SERVICE_PRICE_STATE_UNSUPPORTED".to_owned());
        }
        if string(service, "price_state") == "current" && nonnegative(service, "price").is_none() {
            unsupported.insert("SERVICE_PRICE_VALUE_INVALID".to_owned());
        }
        if !matches!(string(service, "eligibility_state"), "known" | "unknown") {
            unsupported.insert("SERVICE_ELIGIBILITY_STATE_UNSUPPORTED".to_owned());
        }
        if string_array(service, "eligible_group_ids").is_none() {
            unsupported.insert("SERVICE_ELIGIBILITY_GROUPS_INVALID".to_owned());
        }
        if !(service
            .get("step_free_status")
            .is_some_and(|v| v.is_boolean() || v.is_null()))
        {
            unsupported.insert("SERVICE_STEP_FREE_STATE_INVALID".to_owned());
        }
    }

    if !unsupported.is_empty() {
        return empty_result(Disposition::Unsupported, unsupported);
    }

    let mut category_demand: BTreeMap<String, (f64, String)> = BTreeMap::new();
    for group in groups {
        let category = string(group, "service_category").to_owned();
        let demand = number(group, "demand").unwrap();
        let unit = string(group, "demand_unit").to_owned();
        let entry = category_demand
            .entry(category)
            .or_insert((0.0, unit.clone()));
        if entry.1 != unit {
            unsupported.insert("CATEGORY_DEMAND_UNIT_MISMATCH".to_owned());
        }
        entry.0 += demand;
    }
    if !unsupported.is_empty() {
        return empty_result(Disposition::Unsupported, unsupported);
    }

    let mut group_outputs = Vec::new();
    let mut group_failed = false;
    let mut group_unknown = false;
    let mut all_reasons = BTreeSet::<String>::new();

    for group in groups {
        let mut outcome = GroupResult::default();
        let category = string(group, "service_category");
        let origin = string(group, "origin_node");
        let mode = string(group, "mode");
        let budget = number(group, "max_travel_minutes").unwrap();
        let requires_step_free = bool_value(group, "requires_step_free").unwrap();
        let max_price = number(group, "max_service_price").unwrap();
        let price_unit = string(group, "price_unit");
        let demand_unit = string(group, "demand_unit");
        let mut nearest = f64::INFINITY;

        for service in services
            .iter()
            .filter(|service| string(service, "category") == category)
        {
            let service_id = string(service, "id");
            let known_path = if network_state == "current" {
                shortest_path(
                    network,
                    origin,
                    string(service, "node"),
                    mode,
                    requires_step_free,
                    false,
                )
            } else {
                None
            };
            let possible_path = shortest_path(
                network,
                origin,
                string(service, "node"),
                mode,
                requires_step_free,
                true,
            );
            let route_known = known_path.is_some_and(|minutes| minutes <= budget);
            let route_unknown =
                !route_known && possible_path.is_some_and(|minutes| minutes <= budget);

            if !route_known && !route_unknown {
                continue;
            }
            if route_unknown {
                outcome.unknown_services.insert(service_id.to_owned());
                outcome.reason_codes.insert(
                    if network_state == "current" {
                        "NETWORK_EDGE_UNKNOWN"
                    } else {
                        "NETWORK_EVIDENCE_STALE"
                    }
                    .to_owned(),
                );
                continue;
            }

            let minutes = known_path.unwrap();
            nearest = nearest.min(minutes);
            let mut excluded = false;
            let mut uncertain = false;

            let availability = string(service, "availability_state");
            let open_windows = string_array(service, "open_windows").unwrap();
            if availability == "closed"
                || (availability == "open" && !open_windows.contains(&requested_window))
            {
                excluded = true;
            } else if availability != "open" {
                uncertain = true;
                outcome
                    .reason_codes
                    .insert("SERVICE_AVAILABILITY_UNKNOWN".to_owned());
            }

            let eligibility_state = string(service, "eligibility_state");
            if eligibility_state == "known" {
                let eligible = string_array(service, "eligible_group_ids").unwrap();
                if !eligible.contains(&"*") && !eligible.contains(&string(group, "id")) {
                    excluded = true;
                }
            } else {
                uncertain = true;
                outcome
                    .reason_codes
                    .insert("SERVICE_ELIGIBILITY_UNKNOWN".to_owned());
            }

            match string(service, "price_state") {
                "current" => {
                    if string(service, "price_unit") != price_unit {
                        unsupported.insert("SERVICE_PRICE_UNIT_MISMATCH".to_owned());
                    } else if number(service, "price").unwrap() > max_price {
                        excluded = true;
                    }
                }
                _ => {
                    uncertain = true;
                    outcome
                        .reason_codes
                        .insert("SERVICE_PRICE_UNKNOWN".to_owned());
                }
            }

            if requires_step_free {
                match service.get("step_free_status").and_then(Value::as_bool) {
                    Some(false) => excluded = true,
                    Some(true) => {}
                    None => {
                        uncertain = true;
                        outcome
                            .reason_codes
                            .insert("SERVICE_STEP_FREE_UNKNOWN".to_owned());
                    }
                }
            }

            if string(service, "capacity_state") == "current" {
                if string(service, "capacity_unit") != demand_unit {
                    unsupported.insert("SERVICE_CAPACITY_UNIT_MISMATCH".to_owned());
                } else if !excluded && !uncertain {
                    outcome.accessible_services.insert(service_id.to_owned());
                    outcome.capacity += number(service, "capacity").unwrap();
                } else if !excluded && uncertain {
                    outcome.unknown_services.insert(service_id.to_owned());
                }
            } else {
                uncertain = true;
                outcome
                    .reason_codes
                    .insert("SERVICE_CAPACITY_EVIDENCE_UNKNOWN".to_owned());
                if !excluded {
                    outcome.unknown_services.insert(service_id.to_owned());
                }
            }

            if excluded {
                outcome.accessible_services.remove(service_id);
            } else if uncertain {
                outcome.accessible_services.remove(service_id);
                outcome.unknown_services.insert(service_id.to_owned());
            }
        }

        outcome.nearest_minutes = nearest.is_finite().then_some(nearest);
        let demand = number(group, "demand").unwrap();
        outcome.status = if !outcome.accessible_services.is_empty() {
            if outcome.capacity >= demand {
                "AccessibleAndAdequatePotential".to_owned()
            } else if !outcome.unknown_services.is_empty() {
                outcome
                    .reason_codes
                    .insert("ACCESSIBLE_CAPACITY_ADEQUACY_UNKNOWN".to_owned());
                "Unknown".to_owned()
            } else {
                outcome
                    .reason_codes
                    .insert("REACHABLE_CAPACITY_BELOW_GROUP_DEMAND".to_owned());
                "AccessibleButCapacityConstrained".to_owned()
            }
        } else if !outcome.unknown_services.is_empty() {
            "Unknown".to_owned()
        } else {
            outcome
                .reason_codes
                .insert("NO_USABLE_SERVICE_WITHIN_ACCESS_ENVELOPE".to_owned());
            "Inaccessible".to_owned()
        };

        if outcome.status == "Unknown" {
            group_unknown = true;
        }
        if outcome.status == "Inaccessible" || outcome.status == "AccessibleButCapacityConstrained"
        {
            group_failed = true;
        }
        for reason in &outcome.reason_codes {
            all_reasons.insert(reason.clone());
        }
        group_outputs.push(outcome.finish(group));
    }

    if !unsupported.is_empty() {
        return empty_result(Disposition::Unsupported, unsupported);
    }

    let mut category_totals = Vec::new();
    let mut aggregate_failed = false;
    let mut aggregate_unknown = false;
    for (category, (demand, unit)) in &category_demand {
        let mut known_capacity = 0.0;
        let mut unknown_capacity_count = 0usize;
        for service in services
            .iter()
            .filter(|service| string(service, "category") == category)
        {
            let availability = string(service, "availability_state");
            let open_windows = string_array(service, "open_windows").unwrap();
            if availability == "closed"
                || (availability == "open" && !open_windows.contains(&requested_window))
            {
                continue;
            }
            if availability != "open" || string(service, "capacity_state") != "current" {
                unknown_capacity_count += 1;
            } else if string(service, "capacity_unit") == unit {
                known_capacity += number(service, "capacity").unwrap();
            } else {
                unsupported.insert("SERVICE_CAPACITY_UNIT_MISMATCH".to_owned());
            }
        }

        let status = if known_capacity >= *demand {
            "KnownAggregateCapacityAdequate"
        } else if unknown_capacity_count > 0 {
            aggregate_unknown = true;
            "AggregateCapacityUnknown"
        } else {
            aggregate_failed = true;
            all_reasons.insert("AGGREGATE_CAPACITY_SHORTFALL".to_owned());
            "AggregateCapacityShortfall"
        };
        category_totals.push(json!({
            "category": category,
            "demand": demand,
            "demand_unit": unit,
            "declared_open_capacity": known_capacity,
            "unknown_capacity_service_count": unknown_capacity_count,
            "status": status
        }));
    }

    if !unsupported.is_empty() {
        return empty_result(Disposition::Unsupported, unsupported);
    }

    let disposition = if group_failed || aggregate_failed {
        Disposition::Infeasible
    } else if group_unknown || aggregate_unknown {
        Disposition::Unknown
    } else {
        Disposition::FeasibleForSyntheticCase
    };

    json!({
        "disposition": disposition.as_str(),
        "reason_codes": all_reasons.into_iter().collect::<Vec<_>>(),
        "groups": group_outputs,
        "category_totals": category_totals
    })
}

fn json_equivalent(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .zip(right.as_f64())
            .is_some_and(|(a, b)| a == b),
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(a, b)| json_equivalent(a, b))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && right.iter().all(|(key, expected_value)| {
                    left.get(key)
                        .is_some_and(|actual_value| json_equivalent(actual_value, expected_value))
                })
        }
        _ => actual == expected,
    }
}

/// Compare the declared oracle fields structurally. JSON integer and floating
/// representations compare by exact numeric value (e.g. 120 and 120.0);
/// strings, booleans, arrays, object keys and all other values remain exact.
pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    expected.as_object().is_some_and(|fields| {
        fields.iter().all(|(key, expected_value)| {
            actual
                .get(key)
                .is_some_and(|actual_value| json_equivalent(actual_value, expected_value))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case, matches_expected};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn corpus() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/city-morph-access/CITY_MORPH_ACCESS_CORPUS_V0_1.json");
        let bytes = std::fs::read(path).expect("city-morph corpus must exist");
        serde_json::from_slice(&bytes).expect("city-morph corpus must parse")
    }

    fn source_register() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/city-morph-access/JOHANNESBURG_SOURCE_REGISTER_V0_1.json");
        let bytes = std::fs::read(path).expect("Johannesburg source register must exist");
        serde_json::from_slice(&bytes).expect("Johannesburg source register must parse")
    }

    #[test]
    fn source_register_is_rights_aware_and_keeps_access_separate() {
        let register = source_register();
        let count = super::validate_source_register(&register)
            .expect("source register must satisfy its evidence contract");
        assert_eq!(count, 22);

        let sources = register["sources"].as_array().unwrap();
        let online_maps = sources
            .iter()
            .find(|source| source["source_id"] == "COJ-EXTERNAL-ONLINE-MAPS")
            .unwrap();
        assert_eq!(online_maps["access_status"], "VerifiedPublic");
        assert_eq!(online_maps["rights_status"], "PublicWithConditions");

        let municipal_gis = sources
            .iter()
            .find(|source| source["source_id"] == "COJ-ISDP-SPMS")
            .unwrap();
        assert_eq!(municipal_gis["access_status"], "PublicWithConditions");
        assert_eq!(municipal_gis["rights_status"], "PublicWithConditions");
        assert!(
            municipal_gis["known_facts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|fact| fact.as_str().unwrap().contains("August 2019"))
        );

        let bus_trial = sources
            .iter()
            .find(|source| source["source_id"] == "COJ-GREEN-BUS-TRIAL-2026")
            .unwrap();
        assert_eq!(bus_trial["access_status"], "RequestRequired");
        assert_eq!(bus_trial["rights_status"], "Unknown");

        let metrobus_reports = sources
            .iter()
            .find(|source| source["source_id"] == "METROBUS-QUARTERLY-PERFORMANCE-REPORTS")
            .unwrap();
        assert_eq!(metrobus_reports["access_status"], "PublicWithConditions");
        assert_eq!(metrobus_reports["rights_status"], "Unknown");

        let transit = sources
            .iter()
            .find(|source| source["source_id"] == "JOHANNESBURG-TRANSIT-GTFS")
            .unwrap();
        assert_eq!(transit["access_status"], "Unknown");
        assert_eq!(transit["rights_status"], "Unknown");
        assert!(transit["url"].is_null());

        let udap = sources
            .iter()
            .find(|source| source["source_id"] == "JDA-JOHANNESBURG-UDAP-2026")
            .unwrap();
        assert_eq!(udap["access_status"], "VerifiedPublic");
        assert_eq!(udap["rights_status"], "Unknown");
        assert!(
            udap["unknowns"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item.as_str().unwrap().contains("facility-level findings"))
        );
        assert!(
            udap["known_facts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|fact| fact.as_str().unwrap().contains("2026-06-09"))
        );

        let survey = sources
            .iter()
            .find(|source| source["source_id"] == "GCRO-QOL7-2023-24")
            .unwrap();
        assert_eq!(survey["rights_status"], "VerifiedOpenLicense");
        assert!(survey["license"].as_str().unwrap().contains("CC BY 4.0"));
    }

    #[test]
    fn verified_open_license_requires_explicit_license_terms() {
        let mut register = source_register();
        let osm = register["sources"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|source| source["source_id"] == "OPENSTREETMAP-SOUTH-AFRICA")
            .unwrap();
        osm.as_object_mut().unwrap().remove("license");
        let errors = super::validate_source_register(&register)
            .expect_err("open-license status without terms must fail closed");
        assert!(
            errors
                .iter()
                .any(|error| error == "OPEN_LICENSE_WITHOUT_TERMS:OPENSTREETMAP-SOUTH-AFRICA")
        );
    }

    #[test]
    fn public_access_without_a_verified_url_or_rights_is_not_promoted() {
        let mut register = source_register();
        let transit = register["sources"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|source| source["source_id"] == "JOHANNESBURG-TRANSIT-GTFS")
            .unwrap();
        transit["access_status"] = serde_json::Value::String("VerifiedPublic".to_owned());
        let errors = super::validate_source_register(&register)
            .expect_err("a feed with no URL cannot be marked publicly verified");
        assert!(
            errors
                .iter()
                .any(|error| error == "SOURCE_URL_INVALID_OR_UNVERIFIED:JOHANNESBURG-TRANSIT-GTFS")
        );
    }

    #[test]
    fn corpus_has_24_unique_synthetic_cases() {
        let corpus = corpus();
        assert_eq!(corpus["classification"], "synthetic_fixture_only");
        assert_eq!(corpus["corpus_id"], "CITY-MORPH-ACCESS-CORPUS-V0.1");
        let cases = corpus["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 24);
        let ids = cases
            .iter()
            .map(|case| case["case_id"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), cases.len());
    }

    #[test]
    fn independent_reference_oracle_matches_all_known_answers() {
        let corpus = corpus();
        for case in corpus["cases"].as_array().unwrap() {
            let actual = evaluate_case(case);
            assert!(
                matches_expected(&actual, &case["expected"]),
                "{} mismatch
actual: {}
expected: {}",
                case["case_id"],
                serde_json::to_string_pretty(&actual).unwrap(),
                serde_json::to_string_pretty(&case["expected"]).unwrap()
            );
        }
    }

    #[test]
    fn euclidean_nearness_without_network_route_is_inaccessible() {
        let corpus = corpus();
        let case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "MA-004")
            .unwrap();
        let result = evaluate_case(case);
        assert_eq!(result["groups"][0]["status"], "Inaccessible");
        assert_eq!(
            result["groups"][0]["accessible_service_ids"],
            serde_json::json!([])
        );
    }

    #[test]
    fn shared_capacity_is_counted_once_at_city_category_level() {
        let corpus = corpus();
        let case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "MA-019")
            .unwrap();
        let result = evaluate_case(case);
        assert_eq!(
            result["groups"][0]["status"],
            "AccessibleAndAdequatePotential"
        );
        assert_eq!(
            result["groups"][1]["status"],
            "AccessibleAndAdequatePotential"
        );
        assert_eq!(
            result["category_totals"][0]["declared_open_capacity"].as_f64(),
            Some(120.0)
        );
        assert_eq!(result["category_totals"][0]["demand"].as_f64(), Some(200.0));
        assert_eq!(
            result["category_totals"][0]["status"],
            "AggregateCapacityShortfall"
        );
        assert_eq!(result["disposition"], "Infeasible");
    }

    #[test]
    fn cycle_mode_uses_cycle_impedance() {
        let corpus = corpus();
        let mut case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "MA-006")
            .unwrap()
            .clone();
        case["input"]["population_groups"][0]["mode"] =
            serde_json::Value::String("cycle".to_owned());
        let result = evaluate_case(&case);
        assert_eq!(
            result["groups"][0]["nearest_known_service_minutes"].as_f64(),
            Some(10.0)
        );
        assert_eq!(
            result["groups"][0]["status"],
            "AccessibleAndAdequatePotential"
        );
    }

    #[test]
    fn unknown_usable_service_prevents_false_capacity_shortfall() {
        let mut case = corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "MA-002")
            .unwrap()
            .clone();
        case["input"]["services"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "id": "unknown-price-clinic",
                "node": "s",
                "category": "health",
                "capacity": 200,
                "capacity_unit": "visits/day",
                "capacity_state": "current",
                "availability_state": "open",
                "open_windows": ["weekday_day"],
                "price": 0,
                "price_unit": "credits/visit",
                "price_state": "unknown",
                "eligibility_state": "known",
                "eligible_group_ids": ["*"],
                "step_free_status": true
            }));
        let result = evaluate_case(&case);
        assert_eq!(result["groups"][0]["status"], "Unknown");
        assert_eq!(
            result["groups"][0]["accessible_service_ids"],
            serde_json::json!(["clinic"])
        );
        assert_eq!(
            result["groups"][0]["unknown_service_ids"],
            serde_json::json!(["unknown-price-clinic"])
        );
        assert_eq!(result["disposition"], "Unknown");
        assert!(
            result["groups"][0]["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "ACCESSIBLE_CAPACITY_ADEQUACY_UNKNOWN")
        );
    }

    #[test]
    fn stale_network_evidence_cannot_produce_known_access() {
        let corpus = corpus();
        let mut case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "MA-001")
            .unwrap()
            .clone();
        case["input"]["network"]["evidence_state"] = serde_json::Value::String("stale".to_owned());
        let result = evaluate_case(&case);
        assert_eq!(result["disposition"], "Unknown");
        assert_eq!(result["groups"][0]["status"], "Unknown");
        assert_eq!(
            result["groups"][0]["accessible_service_ids"],
            serde_json::json!([])
        );
        assert_eq!(
            result["groups"][0]["unknown_service_ids"],
            serde_json::json!(["clinic"])
        );
    }

    #[test]
    fn incompatible_affordability_units_are_unsupported() {
        let corpus = corpus();
        let mut case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "MA-001")
            .unwrap()
            .clone();
        case["input"]["population_groups"][0]["price_unit"] =
            serde_json::Value::String("USD/visit".to_owned());
        let result = evaluate_case(&case);
        assert_eq!(result["disposition"], "Unsupported");
        assert_eq!(result["groups"], serde_json::json!([]));
        assert!(
            result["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "SERVICE_PRICE_UNIT_MISMATCH")
        );
    }

    #[test]
    fn group_specific_travel_time_budgets_are_applied_independently() {
        let corpus = corpus();
        let case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "MA-018")
            .unwrap();
        let result = evaluate_case(case);
        assert_eq!(
            result["groups"][0]["nearest_known_service_minutes"].as_f64(),
            Some(6.0)
        );
        assert_eq!(
            result["groups"][1]["nearest_known_service_minutes"].as_f64(),
            Some(17.0)
        );
        assert_eq!(
            result["groups"][0]["status"],
            "AccessibleAndAdequatePotential"
        );
        assert_eq!(
            result["groups"][1]["status"],
            "AccessibleAndAdequatePotential"
        );
    }

    #[test]
    fn input_permutations_do_not_change_normalized_access_results() {
        let corpus = corpus();
        let mut case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == "MA-020")
            .unwrap()
            .clone();
        let baseline = evaluate_case(&case);
        case["input"]["network"]["edges"]
            .as_array_mut()
            .unwrap()
            .reverse();
        case["input"]["services"].as_array_mut().unwrap().reverse();
        let permuted = evaluate_case(&case);
        assert_eq!(baseline, permuted);
    }

    #[test]
    fn duplicate_service_and_group_ids_fail_closed() {
        let corpus = corpus();
        for id in ["MA-023", "MA-024"] {
            let case = corpus["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|case| case["case_id"] == id)
                .unwrap();
            let result = evaluate_case(case);
            assert_eq!(result["disposition"], "Unsupported");
            assert_eq!(result["groups"], serde_json::json!([]));
            assert_eq!(result["category_totals"], serde_json::json!([]));
        }
    }
}
