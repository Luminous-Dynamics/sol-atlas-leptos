//! Bounded synthetic reference oracle for effective urban service access.
//!
//! This tool is deliberately separate from the Leptos UI and production GIS.
//! It computes network paths and declared service usability for synthetic
//! fixtures only. Group-level reachable capacity is potential, not an
//! allocation. City/category supply is counted once per facility.
//!
//! No output is a real-world accessibility, safety, affordability, planning,
//! land-rights, community-consent, or city-quality determination.

use std::collections::{BTreeMap, BTreeSet};
use serde_json::{json, Value};

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
    value.get(key).and_then(Value::as_f64).filter(|x| x.is_finite())
}
fn nonnegative(value: &Value, key: &str) -> Option<f64> {
    number(value, key).filter(|x| *x >= 0.0)
}
fn bool_value(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}
fn string_array(value: &Value, key: &str) -> Option<Vec<&str>> {
    value.get(key)?.as_array()?.iter().map(Value::as_str).collect()
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
            || !(edge.get("step_free").is_some_and(|v| v.is_boolean() || v.is_null()))
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
        if !group.get("population").and_then(Value::as_u64).is_some_and(|x| x > 0)
            || !number(group, "demand").is_some_and(|x| x > 0.0)
            || !nonnegative(group, "max_travel_minutes").is_some()
            || !nonnegative(group, "max_service_price").is_some()
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
        if !matches!(string(service, "capacity_state"), "current" | "stale" | "missing" | "unknown") {
            unsupported.insert("SERVICE_CAPACITY_STATE_UNSUPPORTED".to_owned());
        }
        if string(service, "capacity_state") == "current"
            && !nonnegative(service, "capacity").is_some()
        {
            unsupported.insert("SERVICE_CAPACITY_VALUE_INVALID".to_owned());
        }
        if !matches!(string(service, "availability_state"), "open" | "closed" | "unknown") {
            unsupported.insert("SERVICE_AVAILABILITY_STATE_UNSUPPORTED".to_owned());
        }
        if array(service, "open_windows").is_none() {
            unsupported.insert("SERVICE_OPEN_WINDOWS_MISSING".to_owned());
        }
        if !matches!(string(service, "price_state"), "current" | "unknown" | "stale") {
            unsupported.insert("SERVICE_PRICE_STATE_UNSUPPORTED".to_owned());
        }
        if string(service, "price_state") == "current"
            && !nonnegative(service, "price").is_some()
        {
            unsupported.insert("SERVICE_PRICE_VALUE_INVALID".to_owned());
        }
        if !matches!(string(service, "eligibility_state"), "known" | "unknown") {
            unsupported.insert("SERVICE_ELIGIBILITY_STATE_UNSUPPORTED".to_owned());
        }
        if string_array(service, "eligible_group_ids").is_none() {
            unsupported.insert("SERVICE_ELIGIBILITY_GROUPS_INVALID".to_owned());
        }
        if !(service.get("step_free_status").is_some_and(|v| v.is_boolean() || v.is_null())) {
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
        let entry = category_demand.entry(category).or_insert((0.0, unit.clone()));
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

        for service in services.iter().filter(|service| string(service, "category") == category) {
            let service_id = string(service, "id");
            let known_path = if network_state == "current" {
                shortest_path(network, origin, string(service, "node"), mode, requires_step_free, false)
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
            let route_unknown = !route_known && possible_path.is_some_and(|minutes| minutes <= budget);

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
                    }.to_owned(),
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
                outcome.reason_codes.insert("SERVICE_AVAILABILITY_UNKNOWN".to_owned());
            }

            let eligibility_state = string(service, "eligibility_state");
            if eligibility_state == "known" {
                let eligible = string_array(service, "eligible_group_ids").unwrap();
                if !eligible.contains(&"*") && !eligible.contains(&string(group, "id")) {
                    excluded = true;
                }
            } else {
                uncertain = true;
                outcome.reason_codes.insert("SERVICE_ELIGIBILITY_UNKNOWN".to_owned());
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
                    outcome.reason_codes.insert("SERVICE_PRICE_UNKNOWN".to_owned());
                }
            }

            if requires_step_free {
                match service.get("step_free_status").and_then(Value::as_bool) {
                    Some(false) => excluded = true,
                    Some(true) => {}
                    None => {
                        uncertain = true;
                        outcome.reason_codes.insert("SERVICE_STEP_FREE_UNKNOWN".to_owned());
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
                outcome.reason_codes.insert("SERVICE_CAPACITY_EVIDENCE_UNKNOWN".to_owned());
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
            } else {
                outcome.reason_codes.insert("REACHABLE_CAPACITY_BELOW_GROUP_DEMAND".to_owned());
                "AccessibleButCapacityConstrained".to_owned()
            }
        } else if !outcome.unknown_services.is_empty() {
            "Unknown".to_owned()
        } else {
            outcome.reason_codes.insert("NO_USABLE_SERVICE_WITHIN_ACCESS_ENVELOPE".to_owned());
            "Inaccessible".to_owned()
        };

        if outcome.status == "Unknown" {
            group_unknown = true;
        }
        if outcome.status == "Inaccessible" || outcome.status == "AccessibleButCapacityConstrained" {
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
        for service in services.iter().filter(|service| string(service, "category") == category) {
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

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    expected
        .as_object()
        .is_some_and(|fields| fields.iter().all(|(key, value)| actual.get(key) == Some(value)))
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
        serde_json::from_slice(&bytes).expect("city-morph corpus must parse");
    }

    #[test]
    fn corpus_has_24_unique_synthetic_cases() {
        let corpus = corpus();
        assert_eq!(corpus["classification"], "synthetic_fixture_only");
        assert_eq!(corpus["corpus_id"], "CITY-MORPH-ACCESS-CORPUS-V0.1");
        let cases = corpus["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 24);
        let ids = cases.iter().map(|case| case["case_id"].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>();
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
        let case = corpus["cases"].as_array().unwrap().iter().find(|case| case["case_id"] == "MA-004").unwrap();
        let result = evaluate_case(case);
        assert_eq!(result["groups"][0]["status"], "Inaccessible");
        assert_eq!(result["groups"][0]["accessible_service_ids"], serde_json::json!([]));
    }

    #[test]
    fn shared_capacity_is_counted_once_at_city_category_level() {
        let corpus = corpus();
        let case = corpus["cases"].as_array().unwrap().iter().find(|case| case["case_id"] == "MA-019").unwrap();
        let result = evaluate_case(case);
        assert_eq!(result["groups"][0]["status"], "AccessibleAndAdequatePotential");
        assert_eq!(result["groups"][1]["status"], "AccessibleAndAdequatePotential");
        assert_eq!(result["category_totals"][0]["declared_open_capacity"], 120);
        assert_eq!(result["category_totals"][0]["demand"], 200);
        assert_eq!(result["category_totals"][0]["status"], "AggregateCapacityShortfall");
        assert_eq!(result["disposition"], "Infeasible");
    }

    #[test]
    fn input_permutations_do_not_change_normalized_access_results() {
        let corpus = corpus();
        let mut case = corpus["cases"].as_array().unwrap().iter().find(|case| case["case_id"] == "MA-020").unwrap().clone();
        let baseline = evaluate_case(&case);
        case["input"]["network"]["edges"].as_array_mut().unwrap().reverse();
        case["input"]["services"].as_array_mut().unwrap().reverse();
        let permuted = evaluate_case(&case);
        assert_eq!(baseline, permuted);
    }

    #[test]
    fn duplicate_service_and_group_ids_fail_closed() {
        let corpus = corpus();
        for id in ["MA-023", "MA-024"] {
            let case = corpus["cases"].as_array().unwrap().iter().find(|case| case["case_id"] == id).unwrap();
            let result = evaluate_case(case);
            assert_eq!(result["disposition"], "Unsupported");
            assert_eq!(result["groups"], serde_json::json!([]));
            assert_eq!(result["category_totals"], serde_json::json!([]));
        }
    }
}
