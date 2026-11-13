use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::{env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_FLOW_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_FLOW_CASE_V0.1.schema.json"
);

#[derive(Clone)]
struct Edge {
    to: usize,
    reverse: usize,
    capacity: u64,
    original: u64,
}

struct Dinic {
    graph: Vec<Vec<Edge>>,
    level: Vec<i32>,
    next_edge: Vec<usize>,
}

impl Dinic {
    fn new(node_count: usize) -> Self {
        Self {
            graph: vec![Vec::new(); node_count],
            level: vec![-1; node_count],
            next_edge: vec![0; node_count],
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, capacity: u64) -> usize {
        let forward_index = self.graph[from].len();
        let reverse_index = self.graph[to].len();
        self.graph[from].push(Edge {
            to,
            reverse: reverse_index,
            capacity,
            original: capacity,
        });
        self.graph[to].push(Edge {
            to: from,
            reverse: forward_index,
            capacity: 0,
            original: 0,
        });
        forward_index
    }

    fn build_levels(&mut self, source: usize, sink: usize) -> bool {
        self.level.fill(-1);
        let mut queue = VecDeque::new();
        self.level[source] = 0;
        queue.push_back(source);
        while let Some(node) = queue.pop_front() {
            for edge in &self.graph[node] {
                if edge.capacity > 0 && self.level[edge.to] < 0 {
                    self.level[edge.to] = self.level[node] + 1;
                    queue.push_back(edge.to);
                }
            }
        }
        self.level[sink] >= 0
    }

    fn send_flow(&mut self, node: usize, sink: usize, pushed: u64) -> u64 {
        if pushed == 0 {
            return 0;
        }
        if node == sink {
            return pushed;
        }
        while self.next_edge[node] < self.graph[node].len() {
            let edge_index = self.next_edge[node];
            let to = self.graph[node][edge_index].to;
            let reverse = self.graph[node][edge_index].reverse;
            let capacity = self.graph[node][edge_index].capacity;
            if capacity > 0 && self.level[to] == self.level[node] + 1 {
                let sent = self.send_flow(to, sink, pushed.min(capacity));
                if sent > 0 {
                    self.graph[node][edge_index].capacity -= sent;
                    self.graph[to][reverse].capacity += sent;
                    return sent;
                }
            }
            self.next_edge[node] += 1;
        }
        0
    }

    fn max_flow(&mut self, source: usize, sink: usize) -> u64 {
        let mut total = 0_u64;
        while self.build_levels(source, sink) {
            self.next_edge.fill(0);
            loop {
                let sent = self.send_flow(source, sink, u64::MAX);
                if sent == 0 {
                    break;
                }
                total = total.saturating_add(sent);
            }
        }
        total
    }

    fn used_capacity(&self, from: usize, edge_index: usize) -> u64 {
        let edge = &self.graph[from][edge_index];
        edge.original - edge.capacity
    }
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64).filter(|number| number.is_finite())
}

fn u64_at(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

#[derive(Clone)]
struct Group {
    id: String,
    total: u64,
    accessible: u64,
    routes: BTreeMap<String, (bool, bool)>,
}

#[derive(Clone)]
struct Site {
    id: String,
    total: u64,
    accessible: u64,
}

fn solve_accessible(
    groups: &[Group],
    sites: &[Site],
) -> (u64, Vec<Vec<u64>>, Vec<u64>) {
    let source = 0;
    let group_start = 1;
    let site_start = group_start + groups.len();
    let sink = site_start + sites.len();
    let mut network = Dinic::new(sink + 1);
    let mut allocation_edges = Vec::new();

    for (group_index, group) in groups.iter().enumerate() {
        let node = group_start + group_index;
        network.add_edge(source, node, group.accessible);
        for (site_index, site) in sites.iter().enumerate() {
            let route_ok = group.routes.get(&site.id).is_some_and(|(reachable, accessible)| {
                *reachable && *accessible
            });
            if route_ok {
                let edge = network.add_edge(node, site_start + site_index, group.accessible);
                allocation_edges.push((group_index, site_index, node, edge));
            }
        }
    }
    for (site_index, site) in sites.iter().enumerate() {
        network.add_edge(site_start + site_index, sink, site.accessible);
    }

    let flow = network.max_flow(source, sink);
    let mut allocations = vec![vec![0_u64; sites.len()]; groups.len()];
    let mut site_used = vec![0_u64; sites.len()];
    for (group_index, site_index, node, edge) in allocation_edges {
        let used = network.used_capacity(node, edge);
        allocations[group_index][site_index] += used;
        site_used[site_index] += used;
    }
    (flow, allocations, site_used)
}

fn solve_general(
    groups: &[Group],
    sites: &[Site],
    accessible_remaining: &[u64],
    general_capacity: &[u64],
) -> (u64, Vec<Vec<u64>>) {
    let source = 0;
    let group_start = 1;
    let access_start = group_start + groups.len();
    let general_start = access_start + sites.len();
    let sink = general_start + sites.len();
    let mut network = Dinic::new(sink + 1);
    let mut allocation_edges = Vec::new();

    for (group_index, group) in groups.iter().enumerate() {
        let node = group_start + group_index;
        network.add_edge(source, node, group.total - group.accessible);
        for (site_index, site) in sites.iter().enumerate() {
            let reachable = group.routes.get(&site.id).is_some_and(|(reachable, _)| *reachable);
            if reachable {
                let access_edge = network.add_edge(node, access_start + site_index, group.total);
                allocation_edges.push((group_index, site_index, node, access_edge));
                let general_edge = network.add_edge(node, general_start + site_index, group.total);
                allocation_edges.push((group_index, site_index, node, general_edge));
            }
        }
    }
    for (site_index, capacity) in accessible_remaining.iter().enumerate() {
        network.add_edge(access_start + site_index, sink, *capacity);
    }
    for (site_index, capacity) in general_capacity.iter().enumerate() {
        network.add_edge(general_start + site_index, sink, *capacity);
    }

    let flow = network.max_flow(source, sink);
    let mut allocations = vec![vec![0_u64; sites.len()]; groups.len()];
    for (group_index, site_index, node, edge) in allocation_edges {
        allocations[group_index][site_index] += network.used_capacity(node, edge);
    }
    (flow, allocations)
}

fn solve(case: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("schema JSON parse failed: {error}")])?;
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-REFUGE-NETWORK-FLOW-001"),
        ("maturity", "SYNTHETIC_KNOWN_ANSWER"),
        ("execution_status", "NO_SITE_OR_ROUTE_VALIDATION"),
        ("policy", "ACCESSIBLE_DEMAND_FIRST_THEN_GENERAL_MAX_FLOW"),
    ] {
        if text_at(case, key) != Some(expected) {
            errors.push(format!("{key} must remain {expected}"));
        }
    }
    for key in ["evidence_bundle_ref", "independent_oracle_ref"] {
        if text_at(case, key).is_none() {
            errors.push(format!("top-level evidence reference {key} is missing"));
        }
    }
    let safe_water_lpp = number_at(case, "safe_water_l_per_place")
        .ok_or_else(|| vec!["safe_water_l_per_place is missing/non-finite".to_owned()])?;
    let duration = number_at(case, "required_duration_h")
        .ok_or_else(|| vec!["required_duration_h is missing/non-finite".to_owned()])?;
    let max_travel = number_at(case, "maximum_route_travel_time_s")
        .ok_or_else(|| vec!["maximum_route_travel_time_s is missing/non-finite".to_owned()])?;
    if safe_water_lpp <= 0.0 || duration <= 0.0 || max_travel <= 0.0 {
        errors.push("shared water quota, duration and travel bound must be positive".to_owned());
    }

    let mut groups = Vec::new();
    let mut group_ids = BTreeSet::new();
    let group_values = case.get("demand_groups").and_then(Value::as_array)
        .ok_or_else(|| vec!["demand_groups must be an array".to_owned()])?;
    for group in group_values {
        let id = text_at(group, "group_id").unwrap_or("").to_owned();
        let total = u64_at(group, "total_places");
        let accessible = u64_at(group, "accessible_places");
        if id.is_empty() || !group_ids.insert(id.clone()) {
            errors.push(format!("empty or duplicate demand group ID: {id}"));
            continue;
        }
        let (Some(total), Some(accessible)) = (total, accessible) else {
            errors.push(format!("{id}: total/accessibility demand is missing or invalid"));
            continue;
        };
        if total == 0 || accessible > total {
            errors.push(format!("{id}: demand counts are inconsistent"));
        }

        let mut routes = BTreeMap::new();
        let route_values = group.get("routes").and_then(Value::as_array)
            .ok_or_else(|| vec![format!("{id}: routes must be an array")])?;
        for route in route_values {
            let site_id = text_at(route, "site_id").unwrap_or("").to_owned();
            if site_id.is_empty() || routes.contains_key(&site_id) {
                errors.push(format!("{id}: empty or duplicate route edge for {site_id}"));
                continue;
            }
            if text_at(route, "route_evidence_ref").is_none()
                || text_at(route, "accessibility_evidence_ref").is_none()
            {
                errors.push(format!("{id}->{site_id}: route/accessibility evidence reference missing"));
            }
            let seconds = number_at(route, "travel_time_s");
            let reachable = text_at(route, "route_state") == Some("REACHABLE")
                && seconds.is_some_and(|value| value <= max_travel);
            let accessible_route = reachable
                && text_at(route, "accessibility_state") == Some("ACCESSIBLE");
            routes.insert(site_id, (reachable, accessible_route));
        }
        groups.push(Group { id, total, accessible, routes });
    }

    let mut sites = Vec::new();
    let mut site_ids = BTreeSet::new();
    let site_values = case.get("sites").and_then(Value::as_array)
        .ok_or_else(|| vec!["sites must be an array".to_owned()])?;
    for site in site_values {
        let id = text_at(site, "site_id").unwrap_or("").to_owned();
        if id.is_empty() || !site_ids.insert(id.clone()) {
            errors.push(format!("empty or duplicate site ID: {id}"));
            continue;
        }
        for key in ["eligibility_receipt_ref", "capacity_evidence_ref", "water_evidence_ref"] {
            if text_at(site, key).is_none() {
                errors.push(format!("{id}: evidence reference {key} is missing"));
            }
        }
        if text_at(site, "eligibility_state") != Some("ELIGIBLE_FOR_HUMAN_CAPACITY_PLANNING") {
            sites.push(Site { id, total: 0, accessible: 0 });
            continue;
        }
        let total = u64_at(site, "free_total_capacity");
        let accessible = u64_at(site, "free_accessible_capacity");
        let water = number_at(site, "free_safe_water_l");
        let (Some(total), Some(accessible), Some(water)) = (total, accessible, water) else {
            errors.push(format!("{id}: eligible site lacks finite capacity/water inputs"));
            continue;
        };
        if accessible > total || water < 0.0 {
            errors.push(format!("{id}: free capacity/water values are inconsistent"));
            continue;
        }
        let water_cap_float = (water / safe_water_lpp).floor();
        let water_cap = if water_cap_float >= u64::MAX as f64 {
            u64::MAX
        } else {
            water_cap_float as u64
        };
        let effective_total = total.min(water_cap);
        let effective_accessible = accessible.min(effective_total);
        sites.push(Site {
            id,
            total: effective_total,
            accessible: effective_accessible,
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    if groups.is_empty() || sites.is_empty() {
        return Err(vec!["at least one valid demand group and site are required".to_owned()]);
    }

    groups.sort_by(|a, b| a.id.cmp(&b.id));
    sites.sort_by(|a, b| a.id.cmp(&b.id));

    let (accessible_served, accessible_alloc, access_used) = solve_accessible(&groups, &sites);
    let accessible_remaining = sites.iter().enumerate()
        .map(|(index, site)| site.accessible.saturating_sub(access_used[index]))
        .collect::<Vec<_>>();
    let general_capacity = sites.iter().map(|site| site.total - site.accessible).collect::<Vec<_>>();
    let (general_served, general_alloc) =
        solve_general(&groups, &sites, &accessible_remaining, &general_capacity);

    let mut group_reports = Vec::new();
    let mut site_reports = Vec::new();
    let total_demand: u64 = groups.iter().map(|group| group.total).sum();
    let accessible_demand: u64 = groups.iter().map(|group| group.accessible).sum();
    let mut group_served_total = 0_u64;
    let mut group_accessible_served = 0_u64;
    let mut min_group_fulfilment = 1.0_f64;

    for (index, group) in groups.iter().enumerate() {
        let access = accessible_alloc[index].iter().sum::<u64>();
        let general = general_alloc[index].iter().sum::<u64>();
        let served = access + general;
        let unmet = group.total.saturating_sub(served);
        let ratio = served as f64 / group.total as f64;
        min_group_fulfilment = min_group_fulfilment.min(ratio);
        group_served_total += served;
        group_accessible_served += access;
        let mut by_site = Vec::new();
        for (site_index, site) in sites.iter().enumerate() {
            let a = accessible_alloc[index][site_index];
            let g = general_alloc[index][site_index];
            if a + g > 0 {
                by_site.push(json!({
                    "site_id": site.id,
                    "accessible_places": a,
                    "general_places": g,
                    "total_places": a + g
                }));
            }
        }
        group_reports.push(json!({
            "group_id": group.id,
            "total_demand": group.total,
            "accessible_demand": group.accessible,
            "accessible_served": access,
            "general_served": general,
            "total_served": served,
            "unmet_total": unmet,
            "unmet_accessible": group.accessible.saturating_sub(access),
            "fulfilment_ratio": ratio,
            "aggregate_site_feasibility": by_site
        }));
    }

    for (site_index, site) in sites.iter().enumerate() {
        let assigned_accessible: u64 = groups.iter().enumerate()
            .map(|(group_index, _)| accessible_alloc[group_index][site_index]).sum();
        let assigned_general: u64 = groups.iter().enumerate()
            .map(|(group_index, _)| general_alloc[group_index][site_index]).sum();
        let assigned_total = assigned_accessible + assigned_general;
        let water_used = assigned_total as f64 * safe_water_lpp;
        site_reports.push(json!({
            "site_id": site.id,
            "effective_total_capacity_after_water": site.total,
            "effective_accessible_capacity": site.accessible,
            "allocated_accessible_places": assigned_accessible,
            "allocated_general_places": assigned_general,
            "allocated_total_places": assigned_total,
            "remaining_total_capacity": site.total.saturating_sub(assigned_total),
            "remaining_accessible_capacity": site.accessible.saturating_sub(assigned_accessible),
            "safe_water_l_used_by_synthetic_quota": water_used,
            "allocation_commitment": "NONE"
        }));
    }

    let total_unmet = total_demand.saturating_sub(group_served_total);
    let accessible_unmet = accessible_demand.saturating_sub(group_accessible_served);
    Ok(json!({
        "audit_state":"NETWORK_FLOW_FEASIBILITY_COMPLETE",
        "study_id":"HIGHVELD-REFUGE-NETWORK-FLOW-001",
        "case_id":text_at(case,"case_id").unwrap_or_default(),
        "policy":"ACCESSIBLE_DEMAND_FIRST_THEN_GENERAL_MAX_FLOW",
        "demand_group_count":groups.len(),
        "eligible_site_count":sites.iter().filter(|site|site.total>0).count(),
        "total_demand":total_demand,
        "accessible_demand":accessible_demand,
        "total_served":group_served_total,
        "accessible_served":group_accessible_served,
        "general_served":general_served,
        "unmet_total_demand":total_unmet,
        "unmet_accessible_demand":accessible_unmet,
        "network_fulfilment_ratio":if total_demand==0 {0.0} else {group_served_total as f64/total_demand as f64},
        "minimum_group_fulfilment_ratio":min_group_fulfilment,
        "group_results":group_reports,
        "site_results":site_reports,
        "tie_break":"STABLE_LEXICOGRAPHIC_GROUP_AND_SITE_IDS",
        "water_quota_l_per_place":safe_water_lpp,
        "required_duration_h":duration,
        "evidence_bundle_ref":text_at(case,"evidence_bundle_ref").unwrap_or_default(),
        "independent_oracle_ref":text_at(case,"independent_oracle_ref").unwrap_or_default(),
        "allocation_authority":"HUMAN_AND_ASSET_OWNER_REQUIRED",
        "booking_or_reservation":"NONE",
        "occupant_assignment":"NONE",
        "route_computation":"NOT_PERFORMED; route edges supplied by Sol-Atlas",
        "execution_status":"SYNTHETIC_NETWORK_FLOW_ONLY",
        "claim_ceiling":"Capacity feasibility on supplied synthetic route and resource graph only; no site/service/route truth claim"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value = serde_json::from_str(CASE).expect("committed network flow case JSON");
            let report = solve(&case).expect("network flow case structure");
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            if report["audit_state"] != "NETWORK_FLOW_FEASIBILITY_COMPLETE" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-refuge-network-flow");
            println!("  --self-test  run the synthetic accessible-first refuge network-flow oracle");
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
    fn two_group_network_is_fully_covered_without_double_counting() {
        let report = solve(&sample()).unwrap();
        assert_eq!(report["total_demand"], 8);
        assert_eq!(report["total_served"], 8);
        assert_eq!(report["accessible_demand"], 3);
        assert_eq!(report["accessible_served"], 3);
        for site in report["site_results"].as_array().unwrap() {
            assert!(site["allocated_total_places"].as_u64().unwrap()
                <= site["effective_total_capacity_after_water"].as_u64().unwrap());
            assert!(site["allocated_accessible_places"].as_u64().unwrap()
                <= site["effective_accessible_capacity"].as_u64().unwrap());
        }
    }

    #[test]
    fn demand_surge_reports_unmet_without_overbooking() {
        let mut case = sample();
        case["demand_groups"][0]["total_places"] = json!(6);
        case["demand_groups"][0]["accessible_places"] = json!(3);
        let report = solve(&case).unwrap();
        assert!(report["unmet_total_demand"].as_u64().unwrap() > 0);
        for site in report["site_results"].as_array().unwrap() {
            assert!(site["allocated_total_places"].as_u64().unwrap()
                <= site["effective_total_capacity_after_water"].as_u64().unwrap());
        }
    }

    #[test]
    fn inaccessible_route_does_not_serve_accessible_demand() {
        let mut case = sample();
        case["demand_groups"][0]["routes"][0]["accessibility_state"] = json!("NOT_ACCESSIBLE");
        case["demand_groups"][1]["routes"][1]["accessibility_state"] = json!("NOT_ACCESSIBLE");
        let report = solve(&case).unwrap();
        assert!(report["unmet_accessible_demand"].as_u64().unwrap() > 0);
    }

    #[test]
    fn safe_water_limits_effective_site_capacity() {
        let mut case = sample();
        case["sites"][0]["free_safe_water_l"] = json!(4.0);
        let report = solve(&case).unwrap();
        let site = report["site_results"].as_array().unwrap()
            .iter().find(|site| site["site_id"] == "SITE_DELTA").unwrap();
        assert_eq!(site["effective_total_capacity_after_water"], 2);
        assert!(site["allocated_total_places"].as_u64().unwrap() <= 2);
    }

    #[test]
    fn duplicate_route_edges_fail_closed() {
        let mut case = sample();
        let duplicate = case["demand_groups"][0]["routes"][0].clone();
        case["demand_groups"][0]["routes"].as_array_mut().unwrap().push(duplicate);
        assert!(solve(&case).is_err());
    }

    #[test]
    fn unavailable_site_is_excluded_from_capacity() {
        let mut case = sample();
        case["sites"][1]["eligibility_state"] = json!("UNAVAILABLE_OR_UNVERIFIED");
        let report = solve(&case).unwrap();
        let site = report["site_results"].as_array().unwrap()
            .iter().find(|site| site["site_id"] == "SITE_ECHO").unwrap();
        assert_eq!(site["effective_total_capacity_after_water"], 0);
        assert_eq!(report["total_served"], 4);
    }

    #[test]
    fn result_is_stable_under_input_array_permutation() {
        let case = sample();
        let first = solve(&case).unwrap();
        let mut reversed = case;
        reversed["demand_groups"].as_array_mut().unwrap().reverse();
        reversed["sites"].as_array_mut().unwrap().reverse();
        let second = solve(&reversed).unwrap();
        assert_eq!(first["group_results"], second["group_results"]);
        assert_eq!(first["site_results"], second["site_results"]);
    }
}
