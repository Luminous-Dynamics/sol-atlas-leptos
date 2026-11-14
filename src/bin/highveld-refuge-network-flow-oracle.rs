use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::{env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_FLOW_CASE_V0.1.json"
);
const PRIMARY_CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_FEASIBILITY_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_FLOW_CASE_V0.1.schema.json"
);
const MAX_DEMAND_AREAS: usize = 64;
const MAX_REFUGE_SITES: usize = 64;
const MAX_AGGREGATE_PLACES: u64 = 1_000_000;

#[derive(Clone)]
struct FlowEdge {
    to: usize,
    reverse: usize,
    capacity: u64,
}

struct FlowGraph {
    edges: Vec<Vec<FlowEdge>>,
}

impl FlowGraph {
    fn new(nodes: usize) -> Self {
        Self {
            edges: vec![Vec::new(); nodes],
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, capacity: u64) {
        let forward_reverse = self.edges[to].len();
        let reverse_reverse = self.edges[from].len();
        self.edges[from].push(FlowEdge {
            to,
            reverse: forward_reverse,
            capacity,
        });
        self.edges[to].push(FlowEdge {
            to: from,
            reverse: reverse_reverse,
            capacity: 0,
        });
    }

    fn max_flow(&mut self, source: usize, sink: usize) -> u64 {
        let mut total = 0_u64;
        loop {
            let mut parent: Vec<Option<(usize, usize)>> = vec![None; self.edges.len()];
            let mut queue = VecDeque::new();
            queue.push_back(source);
            parent[source] = Some((source, usize::MAX));
            while let Some(node) = queue.pop_front() {
                for (index, edge) in self.edges[node].iter().enumerate() {
                    if edge.capacity > 0 && parent[edge.to].is_none() {
                        parent[edge.to] = Some((node, index));
                        queue.push_back(edge.to);
                        if edge.to == sink {
                            break;
                        }
                    }
                }
                if parent[sink].is_some() {
                    break;
                }
            }
            if parent[sink].is_none() {
                break;
            }
            let mut amount = u64::MAX;
            let mut node = sink;
            while node != source {
                let (previous, edge_index) = parent[node].expect("path parent");
                amount = amount.min(self.edges[previous][edge_index].capacity);
                node = previous;
            }
            node = sink;
            while node != source {
                let (previous, edge_index) = parent[node].expect("path parent");
                let reverse = self.edges[previous][edge_index].reverse;
                self.edges[previous][edge_index].capacity -= amount;
                self.edges[node][reverse].capacity += amount;
                node = previous;
            }
            total += amount;
        }
        total
    }
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn int_at(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|number| number.is_finite())
}

fn checked_unique_ids(items: &[Value], key: &str) -> Result<(), Vec<String>> {
    let mut seen = BTreeSet::new();
    for item in items {
        let id = text_at(item, key).ok_or_else(|| vec![format!("missing {key}")])?;
        if !seen.insert(id.to_owned()) {
            return Err(vec![format!("duplicate {key}: {id}")]);
        }
    }
    Ok(())
}

fn run_flow(
    demands: &[(String, u64)],
    site_caps: &BTreeMap<String, u64>,
    routes: &BTreeMap<String, BTreeSet<String>>,
) -> (u64, BTreeMap<(String, String), u64>) {
    let demand_count = demands.len();
    let site_ids: Vec<String> = site_caps.keys().cloned().collect();
    let site_count = site_ids.len();
    let source = 0;
    let first_demand = 1;
    let first_site = first_demand + demand_count;
    let sink = first_site + site_count;
    let mut graph = FlowGraph::new(sink + 1);
    let mut edge_refs = Vec::new();

    for (index, (area_id, demand)) in demands.iter().enumerate() {
        let demand_node = first_demand + index;
        graph.add_edge(source, demand_node, *demand);
        if let Some(allowed_sites) = routes.get(area_id) {
            for (site_index, site_id) in site_ids.iter().enumerate() {
                if allowed_sites.contains(site_id) {
                    let edge_index = graph.edges[demand_node].len();
                    graph.add_edge(demand_node, first_site + site_index, *demand);
                    edge_refs.push((
                        area_id.clone(),
                        site_id.clone(),
                        demand_node,
                        edge_index,
                        *demand,
                    ));
                }
            }
        }
    }

    for (site_index, site_id) in site_ids.iter().enumerate() {
        graph.add_edge(
            first_site + site_index,
            sink,
            *site_caps.get(site_id).unwrap_or(&0),
        );
    }

    let flow = graph.max_flow(source, sink);
    let mut allocations = BTreeMap::new();
    for (area, site, node, edge_index, original_capacity) in edge_refs {
        let remaining = graph.edges[node][edge_index].capacity;
        let used = original_capacity - remaining;
        if used > 0 {
            allocations.insert((area, site), used);
        }
    }
    (flow, allocations)
}

fn freshness_is_qualified(owner: &Value, key: &str) -> bool {
    let Some(freshness) = owner.get(key) else {
        return false;
    };
    let age = number_at(freshness, "age_elapsed_s");
    let maximum = number_at(freshness, "max_valid_age_s");
    age.is_some_and(|value| value >= 0.0)
        && maximum.is_some_and(|value| value >= 0.0)
        && age
            .zip(maximum)
            .is_some_and(|(age, maximum)| age <= maximum)
        && text_at(freshness, "source_event_id").is_some()
        && text_at(freshness, "max_age_policy_ref").is_some()
}

fn adapt_primary_network_case(primary: &Value) -> Result<Value, Vec<String>> {
    if text_at(primary, "study_id") != Some("HIGHVELD-REFUGE-NETWORK-FEASIBILITY-001")
        || text_at(primary, "maturity") != Some("SYNTHETIC_KNOWN_ANSWER")
    {
        return Err(vec![
            "primary fixture identity or maturity is invalid".to_owned(),
        ]);
    }

    let water_per_person = number_at(primary, "safe_water_l_per_place")
        .filter(|value| *value > 0.0)
        .ok_or_else(|| vec!["primary fixture water basis is invalid".to_owned()])?;
    let cohorts = primary["cohorts"]
        .as_array()
        .ok_or_else(|| vec!["primary fixture cohorts must be an array".to_owned()])?;
    let source_sites = primary["sites"]
        .as_array()
        .ok_or_else(|| vec!["primary fixture sites must be an array".to_owned()])?;
    let source_routes = primary["routes"]
        .as_array()
        .ok_or_else(|| vec!["primary fixture routes must be an array".to_owned()])?;

    let mut sites = Vec::new();
    let mut eligible_site_ids = BTreeSet::new();
    let mut excluded_sites = Vec::new();

    for site in source_sites {
        let site_id = text_at(site, "site_id").unwrap_or("<missing-site-id>");
        let mut reasons = Vec::new();
        for (key, expected, reason) in [
            (
                "operating_window_state",
                "OPEN_FOR_REQUIRED_WINDOW",
                "OPERATING_WINDOW_NOT_QUALIFIED",
            ),
            (
                "thermal_service_state",
                "QUALIFIED",
                "THERMAL_SERVICE_NOT_QUALIFIED",
            ),
            (
                "ventilation_state",
                "QUALIFIED",
                "VENTILATION_NOT_QUALIFIED",
            ),
            (
                "safe_water_service_state",
                "QUALIFIED",
                "SAFE_WATER_SERVICE_NOT_QUALIFIED",
            ),
            (
                "safe_water_rights_state",
                "QUALIFIED",
                "SAFE_WATER_RIGHTS_NOT_QUALIFIED",
            ),
            (
                "safe_water_quality_state",
                "QUALIFIED",
                "SAFE_WATER_QUALITY_NOT_QUALIFIED",
            ),
            (
                "staffing_maintenance_state",
                "QUALIFIED",
                "STAFFING_MAINTENANCE_NOT_QUALIFIED",
            ),
        ] {
            if text_at(site, key) != Some(expected) {
                reasons.push(reason.to_owned());
            }
        }
        for key in [
            "site_status_freshness",
            "capacity_freshness",
            "service_freshness",
            "water_freshness",
        ] {
            if !freshness_is_qualified(site, key) {
                reasons.push(format!("FRESHNESS_NOT_QUALIFIED_{key}"));
            }
        }
        for key in [
            "capacity_evidence_ref",
            "service_evidence_ref",
            "water_evidence_ref",
        ] {
            if text_at(site, key).is_none() {
                reasons.push(format!("MISSING_EVIDENCE_{key}"));
            }
        }

        let total = int_at(site, "total_capacity");
        let occupied = int_at(site, "current_occupancy");
        let committed = int_at(site, "committed_reserve");
        let accessible = int_at(site, "accessible_capacity");
        let accessible_occupied = int_at(site, "accessible_occupancy");
        let accessible_committed = int_at(site, "accessible_committed_reserve");
        let free_water = match (
            number_at(site, "safe_water_available_l"),
            number_at(site, "safe_water_committed_l"),
            number_at(site, "safe_water_protected_reserve_l"),
        ) {
            (Some(available), Some(committed), Some(reserve))
                if available >= 0.0
                    && committed >= 0.0
                    && reserve >= 0.0
                    && available >= committed + reserve =>
            {
                Some(available - committed - reserve)
            }
            _ => {
                reasons.push("WATER_ACCOUNT_NOT_QUALIFIED".to_owned());
                None
            }
        };

        let (free_total, free_accessible) = match (
            total,
            occupied,
            committed,
            accessible,
            accessible_occupied,
            accessible_committed,
        ) {
            (
                Some(total),
                Some(occupied),
                Some(committed),
                Some(accessible),
                Some(accessible_occupied),
                Some(accessible_committed),
            ) if occupied <= total
                && committed <= total.saturating_sub(occupied)
                && accessible <= total
                && accessible_occupied <= accessible
                && accessible_committed <= accessible.saturating_sub(accessible_occupied)
                && accessible_occupied <= occupied
                && accessible_committed <= committed =>
            {
                (
                    Some(total - occupied - committed),
                    Some(accessible - accessible_occupied - accessible_committed),
                )
            }
            _ => {
                reasons.push("CAPACITY_ACCOUNTING_NOT_QUALIFIED".to_owned());
                (None, None)
            }
        };

        match (free_total, free_accessible, free_water) {
            (Some(free_total), Some(free_accessible), Some(free_water))
                if free_accessible <= free_total && reasons.is_empty() =>
            {
                eligible_site_ids.insert(site_id.to_owned());
                sites.push(json!({
                    "site_id":site_id,
                    "availability_state":"ELIGIBLE",
                    "free_total_places":free_total,
                    "free_accessible_places":free_accessible,
                    "free_safe_water_l":free_water,
                    "availability_evidence_ref":text_at(&site["site_status_freshness"],"source_event_id").unwrap_or_default(),
                    "capacity_evidence_ref":text_at(site,"capacity_evidence_ref").unwrap_or_default(),
                    "water_evidence_ref":text_at(site,"water_evidence_ref").unwrap_or_default()
                }));
            }
            _ => excluded_sites.push(json!({
                "site_id":site_id,
                "reasons":reasons
            })),
        }
    }

    let mut demand_areas = Vec::new();
    for cohort in cohorts {
        let area_id = text_at(cohort, "cohort_id").unwrap_or("<missing-cohort-id>");
        let total = int_at(cohort, "total_places_requested")
            .ok_or_else(|| vec![format!("{area_id}: total demand is missing")])?;
        let accessible = int_at(cohort, "accessible_places_requested")
            .ok_or_else(|| vec![format!("{area_id}: accessible demand is missing")])?;
        let maximum_route = number_at(cohort, "maximum_route_travel_time_s")
            .ok_or_else(|| vec![format!("{area_id}: maximum route travel time is missing")])?;
        let mut route_options = Vec::new();

        for route in source_routes
            .iter()
            .filter(|route| text_at(route, "cohort_id") == Some(area_id))
        {
            let site_id = text_at(route, "site_id").unwrap_or("<missing-site-id>");
            if !eligible_site_ids.contains(site_id) {
                continue;
            }
            let route_fresh = freshness_is_qualified(route, "route_freshness");
            let accessibility_fresh = freshness_is_qualified(route, "accessibility_freshness");
            let travel_time = number_at(route, "travel_time_s");
            let route_state = if route_fresh && travel_time.is_some() {
                text_at(route, "route_state").unwrap_or("UNKNOWN")
            } else {
                "UNKNOWN"
            };
            let accessibility_state = if accessibility_fresh {
                text_at(route, "accessibility_state").unwrap_or("UNKNOWN")
            } else {
                "UNKNOWN"
            };
            route_options.push(json!({
                "site_id":site_id,
                "route_state":route_state,
                "accessibility_state":accessibility_state,
                "travel_time_s":travel_time.unwrap_or(0.0),
                "route_evidence_ref":text_at(route,"route_evidence_ref").unwrap_or_default(),
                "accessibility_evidence_ref":text_at(route,"accessibility_evidence_ref").unwrap_or_default()
            }));
        }

        demand_areas.push(json!({
            "area_id":area_id,
            "total_places":total,
            "accessible_places":accessible,
            "maximum_route_travel_time_s":maximum_route,
            "route_options":route_options
        }));
    }

    let max_route = cohorts
        .iter()
        .filter_map(|cohort| number_at(cohort, "maximum_route_travel_time_s"))
        .fold(0.0_f64, f64::max);
    Ok(json!({
        "schema_version":"0.1",
        "study_id":"HIGHVELD-REFUGE-NETWORK-FLOW-001",
        "case_id":"CROSS_ORACLE_ADAPTED_PRIMARY_REFUGE_CASE_001",
        "maturity":"SYNTHETIC_KNOWN_ANSWER",
        "execution_status":"NO_REAL_DEMAND_OR_ROUTE_DATA",
        "allocation_policy":"ACCESSIBLE_FIRST_THEN_GENERAL_REMAINDER_OF_SHARED_SITE_CAPACITY",
        "water_l_per_person":water_per_person,
        "maximum_route_travel_time_s":max_route,
        "oracle_revision":"EDMONDS_KARP_BIPARTITE_MAX_FLOW_V0_1",
        "demand_areas":demand_areas,
        "sites":sites,
        "demand_evidence_ref":text_at(primary,"case_id").unwrap_or("PRIMARY_FIXTURE"),
        "independent_oracle_ref":"CROSS_ORACLE_PRIMARY_FIXTURE_ADAPTER_V0_1",
        "source_adapter":{
            "source_study_id":text_at(primary,"study_id").unwrap_or_default(),
            "source_case_id":text_at(primary,"case_id").unwrap_or_default(),
            "eligible_site_ids":eligible_site_ids,
            "excluded_sites":excluded_sites,
            "route_freshness_owner":text_at(primary,"freshness_assessment_owner").unwrap_or("SYMTROPY"),
            "adapter_revision":"PRIMARY_REFUGE_CASE_TO_REFERENCE_FLOW_V0_1"
        }
    }))
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("schema JSON parse failed: {error}")])?;
    let mut errors = Vec::new();
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(case, "study_id") != Some("HIGHVELD-REFUGE-NETWORK-FLOW-001")
        || text_at(case, "maturity") != Some("SYNTHETIC_KNOWN_ANSWER")
        || text_at(case, "execution_status") != Some("NO_REAL_DEMAND_OR_ROUTE_DATA")
        || text_at(case, "allocation_policy")
            != Some("ACCESSIBLE_FIRST_THEN_GENERAL_REMAINDER_OF_SHARED_SITE_CAPACITY")
        || text_at(case, "oracle_revision") != Some("EDMONDS_KARP_BIPARTITE_MAX_FLOW_V0_1")
    {
        errors.push("study identity, maturity or allocation policy is invalid".to_owned());
    }
    for key in ["demand_evidence_ref", "independent_oracle_ref"] {
        if text_at(case, key).is_none() {
            errors.push(format!("top-level evidence reference {key} is missing"));
        }
    }

    let water_per_person = number_at(case, "water_l_per_person")
        .filter(|value| *value > 0.0)
        .ok_or_else(|| vec!["water_l_per_person must be finite and positive".to_owned()])?;
    let max_travel = number_at(case, "maximum_route_travel_time_s")
        .filter(|value| *value > 0.0)
        .ok_or_else(|| {
            vec!["maximum_route_travel_time_s must be finite and positive".to_owned()]
        })?;

    let mut areas = case
        .get("demand_areas")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| vec!["demand_areas must be an array".to_owned()])?;
    let mut sites = case
        .get("sites")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| vec!["sites must be an array".to_owned()])?;
    if areas.is_empty() || sites.is_empty() {
        errors.push("at least one demand area and site are required".to_owned());
    }
    if areas.len() > MAX_DEMAND_AREAS || sites.len() > MAX_REFUGE_SITES {
        errors.push("input exceeds the bounded area/site envelope".to_owned());
    }
    checked_unique_ids(&areas, "area_id")?;
    checked_unique_ids(&sites, "site_id")?;
    areas.sort_by_key(|area| text_at(area, "area_id").unwrap_or_default().to_owned());
    sites.sort_by_key(|site| text_at(site, "site_id").unwrap_or_default().to_owned());

    let site_ids: BTreeSet<String> = sites
        .iter()
        .filter_map(|site| text_at(site, "site_id").map(str::to_owned))
        .collect();
    let mut routes_general = BTreeMap::<String, BTreeSet<String>>::new();
    let mut routes_accessible = BTreeMap::<String, BTreeSet<String>>::new();
    let mut demand_accessible = Vec::new();
    let mut demand_general = Vec::new();
    let mut route_rejections = Vec::new();
    let mut accessibility_rejections = Vec::new();
    let mut aggregate_places = 0_u64;

    for area in &areas {
        let area_id = text_at(area, "area_id").unwrap_or_default().to_owned();
        let total = int_at(area, "total_places").unwrap_or(0);
        let accessible = int_at(area, "accessible_places").unwrap_or(u64::MAX);
        let area_max_travel = number_at(area, "maximum_route_travel_time_s").unwrap_or(max_travel);
        if !area_max_travel.is_finite() || area_max_travel <= 0.0 {
            errors.push(format!("{area_id}: maximum route travel time is invalid"));
        }
        aggregate_places = aggregate_places
            .checked_add(total)
            .ok_or_else(|| vec!["aggregate place demand overflowed".to_owned()])?;
        if aggregate_places > MAX_AGGREGATE_PLACES {
            errors.push("aggregate place demand exceeds the bounded oracle envelope".to_owned());
        }
        if total == 0 || accessible > total {
            errors.push(format!("{area_id}: invalid total/accessible demand"));
            continue;
        }
        demand_accessible.push((area_id.clone(), accessible));
        demand_general.push((area_id.clone(), total - accessible));

        let route_options = area
            .get("route_options")
            .and_then(Value::as_array)
            .ok_or_else(|| vec![format!("{area_id}: route_options must be an array")])?;
        let mut seen_edges = BTreeSet::new();
        for route in route_options {
            let site_id = text_at(route, "site_id").unwrap_or("<missing-site-id>");
            if !seen_edges.insert(site_id.to_owned()) {
                errors.push(format!("{area_id}: duplicate route edge to {site_id}"));
                continue;
            }
            if !site_ids.contains(site_id) {
                errors.push(format!(
                    "{area_id}: route references unknown site {site_id}"
                ));
                continue;
            }
            let travel = number_at(route, "travel_time_s");
            let route_ok = text_at(route, "route_state") == Some("REACHABLE")
                && travel.is_some_and(|seconds| seconds <= area_max_travel)
                && text_at(route, "route_evidence_ref").is_some();
            if route_ok {
                routes_general
                    .entry(area_id.clone())
                    .or_default()
                    .insert(site_id.to_owned());
            } else {
                route_rejections.push(json!({
                    "area_id":area_id,
                    "site_id":site_id,
                    "reason":"ROUTE_BLOCKED_UNKNOWN_OVER_TIME_OR_UNEVIDENCED"
                }));
            }
            let access_ok = route_ok
                && text_at(route, "accessibility_state") == Some("QUALIFIED")
                && text_at(route, "accessibility_evidence_ref").is_some();
            if access_ok {
                routes_accessible
                    .entry(area_id.clone())
                    .or_default()
                    .insert(site_id.to_owned());
            } else {
                accessibility_rejections.push(json!({
                    "area_id": area_id,
                    "site_id": site_id,
                    "reason": "ACCESSIBILITY_NOT_QUALIFIED_OR_UNEVIDENCED"
                }));
            }
        }
    }

    let mut accessible_caps = BTreeMap::new();
    let mut general_caps = BTreeMap::new();
    let mut site_water = BTreeMap::new();
    for site in &sites {
        let site_id = text_at(site, "site_id").unwrap_or_default().to_owned();
        let total = int_at(site, "free_total_places").unwrap_or(0);
        let accessible = int_at(site, "free_accessible_places").unwrap_or(u64::MAX);
        let water = number_at(site, "free_safe_water_l").unwrap_or(-1.0);
        let refs_ok = [
            "availability_evidence_ref",
            "capacity_evidence_ref",
            "water_evidence_ref",
        ]
        .iter()
        .all(|key| text_at(site, key).is_some());
        if text_at(site, "availability_state") != Some("ELIGIBLE")
            || accessible > total
            || water < 0.0
            || !refs_ok
        {
            errors.push(format!(
                "{site_id}: site availability/capacity/water evidence is invalid"
            ));
            continue;
        }
        let water_places = (water / water_per_person).floor() as u64;
        let effective_total = total.min(water_places);
        let accessible_cap = accessible.min(effective_total);
        accessible_caps.insert(site_id.clone(), accessible_cap);
        general_caps.insert(site_id.clone(), effective_total);
        site_water.insert(site_id, water);
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let (accessible_served, accessible_allocations) =
        run_flow(&demand_accessible, &accessible_caps, &routes_accessible);
    let mut water_after_accessible = site_water.clone();
    for ((_, site_id), people) in &accessible_allocations {
        let remaining = water_after_accessible.get_mut(site_id).expect("known site");
        *remaining = (*remaining - (*people as f64 * water_per_person)).max(0.0);
    }
    let mut remaining_general_caps = BTreeMap::new();
    for (site_id, capacity) in &general_caps {
        let accessible_used = accessible_allocations
            .iter()
            .filter(|((_, allocated_site), _)| allocated_site == site_id)
            .map(|(_, places)| *places)
            .sum::<u64>();
        let remaining_total_capacity = capacity.saturating_sub(accessible_used);
        let water_places = (water_after_accessible.get(site_id).copied().unwrap_or(0.0)
            / water_per_person)
            .floor() as u64;
        remaining_general_caps.insert(site_id.clone(), remaining_total_capacity.min(water_places));
    }
    let (general_served, general_allocations) =
        run_flow(&demand_general, &remaining_general_caps, &routes_general);

    let mut area_report = Vec::new();
    let mut total_requested = 0_u64;
    let mut total_served = 0_u64;
    let mut accessible_requested = 0_u64;
    let mut accessible_total_served = 0_u64;
    let mut general_requested = 0_u64;
    let mut general_total_served = 0_u64;
    for area in &areas {
        let area_id = text_at(area, "area_id").unwrap_or_default();
        let total = int_at(area, "total_places").unwrap_or(0);
        let accessible = int_at(area, "accessible_places").unwrap_or(0);
        let accessible_count = accessible_allocations
            .iter()
            .filter(|((id, _), _)| id == area_id)
            .map(|(_, count)| *count)
            .sum::<u64>();
        let general_count = general_allocations
            .iter()
            .filter(|((id, _), _)| id == area_id)
            .map(|(_, count)| *count)
            .sum::<u64>();
        let general = total - accessible;
        let unserved_accessible = accessible - accessible_count;
        let unserved_general = general - general_count;
        total_requested += total;
        total_served += accessible_count + general_count;
        accessible_requested += accessible;
        accessible_total_served += accessible_count;
        general_requested += general;
        general_total_served += general_count;
        area_report.push(json!({
            "area_id":area_id,
            "total_requested":total,
            "accessible_requested":accessible,
            "accessible_served":accessible_count,
            "accessible_unmet":unserved_accessible,
            "general_requested":general,
            "general_served":general_count,
            "general_unmet":unserved_general,
            "total_served":accessible_count+general_count,
            "total_unmet":unserved_accessible+unserved_general
        }));
    }

    let mut site_report = Vec::new();
    for site in &sites {
        let site_id = text_at(site, "site_id").unwrap_or_default();
        let accessible_used = accessible_allocations
            .iter()
            .filter(|((_, id), _)| id == site_id)
            .map(|(_, count)| *count)
            .sum::<u64>();
        let general_used = general_allocations
            .iter()
            .filter(|((_, id), _)| id == site_id)
            .map(|(_, count)| *count)
            .sum::<u64>();
        let water_remaining = (site_water[site_id]
            - ((accessible_used + general_used) as f64 * water_per_person))
            .max(0.0);
        site_report.push(json!({
            "site_id":site_id,
            "accessible_places_used":accessible_used,
            "general_places_used":general_used,
            "total_places_used":accessible_used+general_used,
            "free_safe_water_remaining_l":water_remaining,
            "availability_evidence_ref":text_at(site,"availability_evidence_ref").unwrap_or_default(),
            "capacity_evidence_ref":text_at(site,"capacity_evidence_ref").unwrap_or_default(),
            "water_evidence_ref":text_at(site,"water_evidence_ref").unwrap_or_default()
        }));
    }

    let total_unmet = total_requested - total_served;
    let coverage = if total_requested > 0 {
        total_served as f64 / total_requested as f64
    } else {
        0.0
    };
    let accessible_coverage = if accessible_requested > 0 {
        accessible_total_served as f64 / accessible_requested as f64
    } else {
        1.0
    };
    let general_coverage = if general_requested > 0 {
        general_total_served as f64 / general_requested as f64
    } else {
        1.0
    };

    Ok(json!({
        "audit_state":"NETWORK_FLOW_COMPLETE",
        "study_id":"HIGHVELD-REFUGE-NETWORK-FLOW-001",
        "case_id":text_at(case,"case_id").unwrap_or_default(),
        "allocation_policy":"ACCESSIBLE_FIRST_THEN_GENERAL_REMAINDER_OF_SHARED_SITE_CAPACITY",
        "demand_evidence_ref":text_at(case,"demand_evidence_ref").unwrap_or_default(),
        "independent_oracle_ref":text_at(case,"independent_oracle_ref").unwrap_or_default(),
        "source_adapter":case.get("source_adapter").cloned().unwrap_or(Value::Null),
        "total_requested":total_requested,
        "total_served":total_served,
        "total_unmet":total_unmet,
        "total_coverage":coverage,
        "accessible_requested":accessible_requested,
        "accessible_served":accessible_total_served,
        "accessible_unmet":accessible_requested-accessible_total_served,
        "accessible_coverage":accessible_coverage,
        "general_requested":general_requested,
        "general_served":general_total_served,
        "general_unmet":general_requested-general_total_served,
        "general_coverage":general_coverage,
        "area_results":area_report,
        "site_results":site_report,
        "route_rejections":route_rejections,
        "accessibility_rejections":accessibility_rejections,
        "accessible_flow_total":accessible_served,
        "general_flow_total":general_served,
        "execution_status":"SYNTHETIC_AGGREGATE_ORACLE_ONLY",
        "individual_assignment":"NONE",
        "reservation_or_actuation":"NONE",
        "claim_ceiling":"Aggregate synthetic coverage only; not real refuge availability, routing, booking or emergency dispatch"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value = serde_json::from_str(CASE).expect("network flow case JSON");
            let report = evaluate(&case).expect("network flow case structure");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["audit_state"] != "NETWORK_FLOW_COMPLETE" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-refuge-network-flow-oracle");
            println!("  --self-test  evaluate the synthetic aggregate refuge capacity flow");
        }
        _ => {
            eprintln!("invalid argument; use --help");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Value {
        serde_json::from_str(CASE).expect("case JSON")
    }

    #[test]
    fn synthetic_flow_serves_all_accessible_and_ten_of_eleven_general_requests() {
        let report = evaluate(&sample()).expect("flow case evaluates");
        assert_eq!(report["total_requested"], 15);
        assert_eq!(report["total_served"], 14);
        assert_eq!(report["total_unmet"], 1);
        assert_eq!(report["accessible_requested"], 4);
        assert_eq!(report["accessible_served"], 4);
        assert_eq!(report["accessible_unmet"], 0);
        assert_eq!(report["general_requested"], 11);
        assert_eq!(report["general_served"], 10);
        assert_eq!(report["general_unmet"], 1);
        assert_eq!(report["individual_assignment"], "NONE");
    }

    #[test]
    fn shared_capacity_is_not_double_counted() {
        let report = evaluate(&sample()).unwrap();
        let hub = report["site_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|site| site["site_id"] == "HUB_1")
            .unwrap();
        let pod = report["site_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|site| site["site_id"] == "POD_2")
            .unwrap();
        assert_eq!(hub["total_places_used"], 8);
        assert_eq!(pod["total_places_used"], 6);
        assert_eq!(report["total_served"], 14);
    }

    #[test]
    fn route_time_and_accessibility_gate_edges_separately() {
        let report = evaluate(&sample()).unwrap();
        assert!(
            report["route_rejections"]
                .as_array()
                .unwrap()
                .iter()
                .any(|edge| edge["area_id"] == "AREA_B" && edge["site_id"] == "POD_2")
        );
        let area_b = report["area_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|area| area["area_id"] == "AREA_B")
            .unwrap();
        assert_eq!(area_b["accessible_served"], 2);
    }

    #[test]
    fn water_limited_site_capacity_is_not_exceeded() {
        let mut case = sample();
        case["sites"][0]["free_safe_water_l"] = json!(4.0);
        let report = evaluate(&case).unwrap();
        let hub = report["site_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|site| site["site_id"] == "HUB_1")
            .unwrap();
        assert!(hub["total_places_used"].as_u64().unwrap() <= 2);
        assert!(hub["free_safe_water_remaining_l"].as_f64().unwrap() >= 0.0);
    }

    #[test]
    fn primary_refuge_fixture_replays_same_max_throughput_through_independent_oracle() {
        let primary: Value =
            serde_json::from_str(PRIMARY_CASE).expect("primary refuge network fixture JSON");
        let adapted = adapt_primary_network_case(&primary)
            .expect("primary fixture adapter must produce the reference case");
        let report = evaluate(&adapted).expect("adapted primary network case must qualify");
        assert_eq!(report["total_requested"], 20);
        assert_eq!(report["accessible_requested"], 6);
        assert_eq!(report["accessible_served"], 6);
        assert_eq!(report["general_requested"], 14);
        assert_eq!(
            report["source_adapter"]["source_case_id"],
            "SYNTHETIC_MULTI_ZONE_PARTITIONED_REFUGE_NETWORK_001"
        );
        assert_eq!(
            report["source_adapter"]["eligible_site_ids"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        // Exact-head parity reference from highveld-refuge-network-feasibility
        // self-test: 6 accessible + 6 general = 12 max-throughput places.
        assert_eq!(report["accessible_served"], 6);
        assert_eq!(report["general_served"], 6);
        assert_eq!(report["total_served"], 12);
        assert_eq!(report["total_unmet"], 8);
    }

    #[test]
    fn unused_accessible_capacity_is_released_after_accessible_demand() {
        let mut case = sample();
        case["demand_areas"] = json!([{
            "area_id":"ONLY_AREA",
            "total_places":10,
            "accessible_places":1,
            "maximum_route_travel_time_s":600,
            "route_options":[{
                "site_id":"HUB_1",
                "route_state":"REACHABLE",
                "accessibility_state":"QUALIFIED",
                "travel_time_s":100,
                "route_evidence_ref":"SYNTH_ROUTE_ONLY",
                "accessibility_evidence_ref":"SYNTH_ACCESS_ONLY"
            }]
        }]);
        case["sites"] = json!([{
            "site_id":"HUB_1",
            "availability_state":"ELIGIBLE",
            "free_total_places":10,
            "free_accessible_places":4,
            "free_safe_water_l":20.0,
            "availability_evidence_ref":"SYNTH_AVAIL_ONLY",
            "capacity_evidence_ref":"SYNTH_CAP_ONLY",
            "water_evidence_ref":"SYNTH_WATER_ONLY"
        }]);
        let report = evaluate(&case).unwrap();
        assert_eq!(report["accessible_served"], 1);
        assert_eq!(report["general_served"], 9);
        assert_eq!(report["total_served"], 10);
    }

    #[test]
    fn duplicate_route_edges_are_rejected() {
        let mut case = sample();
        let duplicate = case["demand_areas"][0]["route_options"][0].clone();
        case["demand_areas"][0]["route_options"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(evaluate(&case).is_err());
    }

    #[test]
    fn unknown_site_route_reference_is_rejected() {
        let mut case = sample();
        case["demand_areas"][0]["route_options"][0]["site_id"] = json!("MISSING_SITE");
        assert!(evaluate(&case).is_err());
    }
}
