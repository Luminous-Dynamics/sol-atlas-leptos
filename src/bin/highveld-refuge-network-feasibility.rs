use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::{env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_FEASIBILITY_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_FEASIBILITY_CASE_V0.1.schema.json"
);

#[derive(Clone)]
struct Edge {
    to: usize,
    reverse_index: usize,
    residual: u64,
    initial: u64,
}

#[derive(Clone, Copy)]
struct EdgeHandle {
    from: usize,
    index: usize,
}

struct FlowNetwork {
    graph: Vec<Vec<Edge>>,
}

struct FlowResult {
    total: u64,
    served_by_cohort: Vec<u64>,
    used_by_site: Vec<u64>,
}

impl FlowNetwork {
    fn new(node_count: usize) -> Self {
        Self {
            graph: vec![Vec::new(); node_count],
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, capacity: u64) -> EdgeHandle {
        let forward_index = self.graph[from].len();
        let reverse_index = self.graph[to].len();
        self.graph[from].push(Edge {
            to,
            reverse_index,
            residual: capacity,
            initial: capacity,
        });
        self.graph[to].push(Edge {
            to: from,
            reverse_index: forward_index,
            residual: 0,
            initial: 0,
        });
        EdgeHandle {
            from,
            index: forward_index,
        }
    }

    fn edge_flow(&self, handle: EdgeHandle) -> u64 {
        let edge = &self.graph[handle.from][handle.index];
        edge.initial - edge.residual
    }

    fn max_flow(&mut self, source: usize, sink: usize) -> u64 {
        let mut total = 0_u64;
        loop {
            let mut parent: Vec<Option<(usize, usize)>> = vec![None; self.graph.len()];
            let mut queue = VecDeque::new();
            parent[source] = Some((source, usize::MAX));
            queue.push_back(source);

            while let Some(node) = queue.pop_front() {
                for (index, edge) in self.graph[node].iter().enumerate() {
                    if edge.residual > 0 && parent[edge.to].is_none() {
                        parent[edge.to] = Some((node, index));
                        queue.push_back(edge.to);
                    }
                }
            }

            if parent[sink].is_none() {
                break;
            }

            let mut path_capacity = u64::MAX;
            let mut node = sink;
            while node != source {
                let (previous, edge_index) = parent[node].expect("reachable node has parent");
                path_capacity = path_capacity.min(self.graph[previous][edge_index].residual);
                node = previous;
            }
            if path_capacity == 0 || path_capacity == u64::MAX {
                break;
            }

            node = sink;
            while node != source {
                let (previous, edge_index) = parent[node].expect("reachable node has parent");
                let reverse_index = self.graph[previous][edge_index].reverse_index;
                self.graph[previous][edge_index].residual -= path_capacity;
                self.graph[node][reverse_index].residual += path_capacity;
                node = previous;
            }
            total += path_capacity;
        }
        total
    }
}

fn run_flow(
    demands: &[(String, u64)],
    sites: &[(String, u64)],
    links: &BTreeSet<(String, String)>,
) -> FlowResult {
    let source = 0;
    let demand_start = 1;
    let site_start = demand_start + demands.len();
    let sink = site_start + sites.len();
    let mut network = FlowNetwork::new(sink + 1);

    let demand_nodes: BTreeMap<String, usize> = demands
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id.clone(), demand_start + index))
        .collect();
    let site_nodes: BTreeMap<String, usize> = sites
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id.clone(), site_start + index))
        .collect();

    let demand_handles: Vec<EdgeHandle> = demands
        .iter()
        .map(|(id, quantity)| {
            network.add_edge(source, demand_nodes[id], *quantity)
        })
        .collect();
    let site_handles: Vec<EdgeHandle> = sites
        .iter()
        .map(|(id, capacity)| {
            network.add_edge(site_nodes[id], sink, *capacity)
        })
        .collect();

    for (cohort_id, site_id) in links {
        if let (Some(demand_node), Some(site_node)) =
            (demand_nodes.get(cohort_id), site_nodes.get(site_id))
        {
            let demand_capacity = demands
                .iter()
                .find(|(id, _)| id == cohort_id)
                .map(|(_, count)| *count)
                .unwrap_or(0);
            network.add_edge(*demand_node, *site_node, demand_capacity);
        }
    }

    let total = network.max_flow(source, sink);
    FlowResult {
        total,
        served_by_cohort: demand_handles
            .iter()
            .map(|handle| network.edge_flow(*handle))
            .collect(),
        used_by_site: site_handles
            .iter()
            .map(|handle| network.edge_flow(*handle))
            .collect(),
    }
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn uint_at(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64).filter(|number| number.is_finite())
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("schema JSON parse failed: {error}")])?;
    let mut errors = Vec::new();
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-REFUGE-NETWORK-FEASIBILITY-001"),
        ("maturity", "SYNTHETIC_KNOWN_ANSWER"),
        ("execution_status", "NO_SITE_OR_ROUTE_VALIDATION"),
    ] {
        if text_at(case, key) != Some(expected) {
            errors.push(format!("{key} must remain {expected}"));
        }
    }
    let local_directory_qualified =
        text_at(case, "local_refuge_directory_state") == Some("QUALIFIED");
    if case.get("network_state").and_then(Value::as_str).is_none() {
        errors.push("network_state is missing".to_owned());
    }
    let water_basis = number_at(case, "safe_water_l_per_place")
        .ok_or_else(|| vec!["safe_water_l_per_place must be finite".to_owned()])?;
    if water_basis <= 0.0 {
        errors.push("safe water planning basis must be positive".to_owned());
    }

    let cohorts = case.get("cohorts").and_then(Value::as_array)
        .ok_or_else(|| vec!["cohorts must be an array".to_owned()])?;
    let sites = case.get("sites").and_then(Value::as_array)
        .ok_or_else(|| vec!["sites must be an array".to_owned()])?;
    let routes = case.get("routes").and_then(Value::as_array)
        .ok_or_else(|| vec!["routes must be an array".to_owned()])?;
    if cohorts.is_empty() || sites.is_empty() {
        errors.push("at least one cohort and one site are required".to_owned());
    }

    let mut cohort_index = BTreeMap::<String, usize>::new();
    let mut total_requested = 0_u64;
    let mut accessible_requested = 0_u64;
    let mut cohort_data = Vec::<(String, u64, u64, f64)>::new();
    for (index, cohort) in cohorts.iter().enumerate() {
        let id = text_at(cohort, "cohort_id").unwrap_or("<missing-cohort-id>").to_owned();
        if cohort_index.insert(id.clone(), index).is_some() {
            errors.push(format!("duplicate cohort identity {id}"));
        }
        let total = uint_at(cohort, "total_places_requested").unwrap_or(0);
        let accessible = uint_at(cohort, "accessible_places_requested").unwrap_or(0);
        let max_time = number_at(cohort, "maximum_route_travel_time_s").unwrap_or(0.0);
        if id.starts_with("<missing") || total == 0 || accessible > total || max_time <= 0.0 {
            errors.push(format!("{id}: invalid cohort demand or travel-time limit"));
        }
        total_requested += total;
        accessible_requested += accessible;
        cohort_data.push((id, total, accessible, max_time));
    }

    let mut site_index = BTreeMap::<String, usize>::new();
    let mut site_ids = Vec::new();
    let mut site_effective_total = Vec::new();
    let mut site_effective_accessible = Vec::new();
    let mut site_reports = Vec::new();
    for (index, site) in sites.iter().enumerate() {
        let id = text_at(site, "site_id").unwrap_or("<missing-site-id>").to_owned();
        if site_index.insert(id.clone(), index).is_some() {
            errors.push(format!("duplicate refuge site identity {id}"));
        }
        site_ids.push(id.clone());

        let mut reasons = Vec::new();
        if !local_directory_qualified {
            reasons.push("LOCAL_REFUGE_DIRECTORY_NOT_QUALIFIED".to_owned());
        }
        for (key, expected, reason) in [
            ("operating_window_state", "OPEN_FOR_REQUIRED_WINDOW", "NOT_OPEN_FOR_REQUIRED_WINDOW"),
            ("thermal_service_state", "QUALIFIED", "THERMAL_SERVICE_NOT_QUALIFIED"),
            ("ventilation_state", "QUALIFIED", "VENTILATION_NOT_QUALIFIED"),
            ("safe_water_service_state", "QUALIFIED", "SAFE_WATER_SERVICE_NOT_QUALIFIED"),
            ("safe_water_rights_state", "QUALIFIED", "SAFE_WATER_RIGHTS_NOT_QUALIFIED"),
            ("safe_water_quality_state", "QUALIFIED", "SAFE_WATER_QUALITY_NOT_QUALIFIED"),
            ("staffing_maintenance_state", "QUALIFIED", "STAFFING_OR_MAINTENANCE_NOT_QUALIFIED"),
        ] {
            if text_at(site, key) != Some(expected) {
                reasons.push(reason.to_owned());
            }
        }

        for key in ["capacity_evidence_ref", "service_evidence_ref", "water_evidence_ref"] {
            if text_at(site, key).is_none() {
                reasons.push(format!("MISSING_SITE_EVIDENCE_{key}"));
            }
        }

        let total_capacity = uint_at(site, "total_capacity");
        let occupancy = uint_at(site, "current_occupancy");
        let committed = uint_at(site, "committed_reserve");
        let accessible_capacity = uint_at(site, "accessible_capacity");
        let accessible_occupancy = uint_at(site, "accessible_occupancy");
        let accessible_committed = uint_at(site, "accessible_committed_reserve");
        let free_water = match (
            number_at(site, "safe_water_available_l"),
            number_at(site, "safe_water_committed_l"),
            number_at(site, "safe_water_protected_reserve_l"),
        ) {
            (Some(available), Some(reserved), Some(protected))
                if available >= 0.0 && reserved >= 0.0 && protected >= 0.0
                    && available >= reserved + protected =>
            {
                Some(available - reserved - protected)
            }
            (Some(_), Some(_), Some(_)) => {
                reasons.push("SAFE_WATER_BELOW_COMMITMENTS_OR_PROTECTED_RESERVE".to_owned());
                None
            }
            _ => {
                reasons.push("SAFE_WATER_ACCOUNT_UNKNOWN_OR_INVALID".to_owned());
                None
            }
        };

        let (free_total, free_accessible) = match (
            total_capacity, occupancy, committed,
            accessible_capacity, accessible_occupancy, accessible_committed
        ) {
            (Some(total), Some(occupied), Some(reserved), Some(access), Some(access_occupied), Some(access_reserved))
                if occupied <= total
                    && reserved <= total - occupied
                    && access_occupied <= access
                    && access_reserved <= access - access_occupied
                    && access <= total
                    && access_occupied <= occupied
                    && access_reserved <= reserved =>
            {
                (Some(total - occupied - reserved), Some(access - access_occupied - access_reserved))
            }
            (Some(_), Some(_), Some(_), Some(_), Some(_), Some(_)) => {
                reasons.push("CAPACITY_ACCOUNTING_INVALID".to_owned());
                (None, None)
            }
            _ => {
                reasons.push("CAPACITY_EVIDENCE_UNKNOWN".to_owned());
                (None, None)
            }
        };

        let effective_total = match (free_total, free_water) {
            (Some(free), Some(water)) => {
                let water_slots = (water / water_basis).floor().max(0.0) as u64;
                Some(free.min(water_slots))
            }
            _ => None
        };
        let effective_accessible = match (free_accessible, effective_total) {
            (Some(access), Some(total)) => Some(access.min(total)),
            _ => None
        };

        let eligible = reasons.is_empty() && effective_total.is_some() && effective_accessible.is_some();
        site_effective_total.push(if eligible { effective_total.unwrap_or(0) } else { 0 });
        site_effective_accessible.push(if eligible { effective_accessible.unwrap_or(0) } else { 0 });
        site_reports.push(json!({
            "site_id":id,
            "site_eligibility":if eligible {"ELIGIBLE"} else {"EXCLUDED"},
            "raw_free_total_capacity":free_total,
            "raw_free_accessible_capacity":free_accessible,
            "free_safe_water_l":free_water,
            "water_limited_effective_total_capacity":effective_total,
            "effective_accessible_capacity":effective_accessible,
            "reasons":reasons
        }));
    }

    let mut general_links = BTreeSet::<(String, String)>::new();
    let mut accessible_links = BTreeSet::<(String, String)>::new();
    let mut seen_routes = BTreeSet::<(String, String)>::new();
    let mut route_reports = Vec::new();
    for route in routes {
        let cohort_id = text_at(route, "cohort_id").unwrap_or("<missing-cohort>").to_owned();
        let site_id = text_at(route, "site_id").unwrap_or("<missing-site>").to_owned();
        let mut reasons = Vec::new();
        if !cohort_index.contains_key(&cohort_id) {
            errors.push(format!("route references unknown cohort {cohort_id}"));
        }
        if !site_index.contains_key(&site_id) {
            errors.push(format!("route references unknown site {site_id}"));
        }
        if !seen_routes.insert((cohort_id.clone(), site_id.clone())) {
            errors.push(format!("duplicate route edge {cohort_id} -> {site_id}"));
        }
        if text_at(route, "route_evidence_ref").is_none()
            || text_at(route, "accessibility_evidence_ref").is_none()
        {
            reasons.push("ROUTE_OR_ACCESSIBILITY_EVIDENCE_REFERENCE_MISSING".to_owned());
        }
        let cohort = cohort_index.get(&cohort_id).map(|index| &cohort_data[*index]);
        let site = site_index.get(&site_id).copied();
        let time = number_at(route, "travel_time_s");
        let time_ok = match (time, cohort) {
            (Some(seconds), Some((_, _, _, maximum))) => seconds <= *maximum,
            _ => false,
        };
        if !time_ok {
            reasons.push("ROUTE_TRAVEL_TIME_UNKNOWN_OR_OVER_LIMIT".to_owned());
        }
        let route_reachable = text_at(route, "route_state") == Some("REACHABLE");
        if !route_reachable {
            reasons.push("ROUTE_NOT_REACHABLE".to_owned());
        }
        let site_ok = site.map(|i| site_reports[i]["site_eligibility"] == "ELIGIBLE").unwrap_or(false);
        if !site_ok {
            reasons.push("REFUGE_SITE_NOT_ELIGIBLE".to_owned());
        }

        let accessibility = text_at(route, "accessibility_state");
        let general_access_ok = matches!(accessibility, Some("QUALIFIED" | "NOT_ACCESSIBLE"));
        let accessible_access_ok = accessibility == Some("QUALIFIED");
        if !general_access_ok {
            reasons.push("ACCESSIBILITY_STATE_UNKNOWN".to_owned());
        }

        let base_ok = time_ok && route_reachable && site_ok
            && text_at(route, "route_evidence_ref").is_some()
            && text_at(route, "accessibility_evidence_ref").is_some();
        if base_ok && general_access_ok {
            general_links.insert((cohort_id.clone(), site_id.clone()));
        }
        if base_ok && accessible_access_ok {
            accessible_links.insert((cohort_id.clone(), site_id.clone()));
        }

        route_reports.push(json!({
            "cohort_id":cohort_id,
            "site_id":site_id,
            "general_route_usable":base_ok && general_access_ok,
            "accessible_route_usable":base_ok && accessible_access_ok,
            "reasons":reasons
        }));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let accessible_demands: Vec<(String, u64)> = cohort_data
        .iter()
        .map(|(id, _, accessible, _)| (id.clone(), *accessible))
        .collect();
    let accessible_sites: Vec<(String, u64)> = site_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (id.clone(), site_effective_accessible[index]))
        .collect();
    let accessible_flow = run_flow(&accessible_demands, &accessible_sites, &accessible_links);

    let general_demands: Vec<(String, u64)> = cohort_data
        .iter()
        .map(|(id, total, accessible, _)| (id.clone(), total - accessible))
        .collect();
    let residual_sites: Vec<(String, u64)> = site_ids
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let remaining = site_effective_total[index].saturating_sub(accessible_flow.used_by_site[index]);
            (id.clone(), remaining)
        })
        .collect();
    let general_flow = run_flow(&general_demands, &residual_sites, &general_links);

    let mut cohort_reports = Vec::new();
    let mut accessible_unmet = 0_u64;
    let mut general_unmet = 0_u64;
    let mut minimum_ratio = 1.0_f64;
    for (index, (id, total, accessible, _)) in cohort_data.iter().enumerate() {
        let accessible_served = accessible_flow.served_by_cohort[index];
        let general_served = general_flow.served_by_cohort[index];
        let general_requested = total - accessible;
        let total_served = accessible_served + general_served;
        accessible_unmet += accessible.saturating_sub(accessible_served);
        general_unmet += general_requested.saturating_sub(general_served);
        let ratio = if *total == 0 { 1.0 } else { total_served as f64 / *total as f64 };
        minimum_ratio = minimum_ratio.min(ratio);
        cohort_reports.push(json!({
            "cohort_id":id,
            "total_places_requested":total,
            "accessible_places_requested":accessible,
            "accessible_places_feasible":accessible_served,
            "general_places_requested":general_requested,
            "general_places_feasible":general_served,
            "total_places_feasible":total_served,
            "cohort_service_ratio":ratio
        }));
    }

    let total_accessible_feasible = accessible_flow.total;
    let total_general_feasible = general_flow.total;
    let total_feasible = total_accessible_feasible + total_general_feasible;
    let total_unmet = total_requested.saturating_sub(total_feasible);

    Ok(json!({
        "audit_state":"REFUGE_NETWORK_FEASIBILITY_COMPLETE",
        "study_id":"HIGHVELD-REFUGE-NETWORK-FEASIBILITY-001",
        "case_id":text_at(case,"case_id").unwrap_or_default(),
        "network_state":text_at(case,"network_state").unwrap_or("UNKNOWN"),
        "local_refuge_directory_state":text_at(case,"local_refuge_directory_state").unwrap_or("UNKNOWN"),
        "allocation_policy":"ACCESSIBLE_DEMAND_FIRST_THEN_GENERAL_DEMAND; BENCHMARK_ONLY",
        "total_places_requested":total_requested,
        "accessible_places_requested":accessible_requested,
        "accessible_places_feasible":total_accessible_feasible,
        "accessible_places_unmet":accessible_unmet,
        "general_places_requested":total_requested - accessible_requested,
        "general_places_feasible":total_general_feasible,
        "general_places_unmet":general_unmet,
        "total_places_feasible":total_feasible,
        "total_places_unmet":total_unmet,
        "minimum_cohort_service_ratio":minimum_ratio,
        "cohort_service":cohort_reports,
        "site_capacity_evidence":site_reports,
        "route_edge_audit":route_reports,
        "qualified_site_count":site_reports.iter().filter(|item|item["site_eligibility"]=="ELIGIBLE").count(),
        "qualified_general_route_count":general_links.len(),
        "qualified_accessible_route_count":accessible_links.len(),
        "assignment":"NONE",
        "reservation":"NONE",
        "execution_status":"AGGREGATE_FEASIBILITY_ONLY_NO_ASSIGNMENT",
        "claim_ceiling":"Maximum aggregate feasibility under synthetic frozen inputs; not real allocation, emergency dispatch or physical service truth"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value = serde_json::from_str(CASE).expect("committed network case JSON");
            let report = evaluate(&case).expect("network case structure");
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            if report["audit_state"] != "REFUGE_NETWORK_FEASIBILITY_COMPLETE" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-refuge-network-feasibility");
            println!("  --self-test  evaluate aggregate refuge capacity feasibility");
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
    fn multi_cohort_flow_accounts_for_shared_capacity_and_water_limits() {
        let report = evaluate(&sample()).expect("case must evaluate");
        assert_eq!(report["total_places_requested"], 20);
        assert_eq!(report["accessible_places_requested"], 6);
        assert_eq!(report["accessible_places_feasible"], 6);
        assert_eq!(report["general_places_feasible"], 6);
        assert_eq!(report["total_places_feasible"], 12);
        assert_eq!(report["total_places_unmet"], 8);
        assert_eq!(report["qualified_site_count"], 2);
    }

    #[test]
    fn accessible_demand_never_uses_non_accessible_route() {
        let mut case = sample();
        for route in case["routes"].as_array_mut().unwrap() {
            if route["cohort_id"] == "ANON_ZONE_A" && route["site_id"] == "SYNTHETIC_POD_1" {
                route["accessibility_state"] = json!("NOT_ACCESSIBLE");
            }
        }
        let report = evaluate(&case).unwrap();
        assert_eq!(report["accessible_places_feasible"], 3);
        assert_eq!(report["accessible_places_unmet"], 3);
    }

    #[test]
    fn unqualified_site_service_removes_its_capacity_from_network_flow() {
        let mut case = sample();
        case["sites"][0]["thermal_service_state"] = json!("UNKNOWN");
        let report = evaluate(&case).unwrap();
        assert_eq!(report["qualified_site_count"], 1);
        assert!(report["total_places_feasible"].as_u64().unwrap() < 12);
    }

    #[test]
    fn missing_local_directory_blocks_network_feasibility() {
        let mut case = sample();
        case["local_refuge_directory_state"] = json!("UNKNOWN");
        let report = evaluate(&case).unwrap();
        assert_eq!(report["total_places_feasible"], 0);
    }

    #[test]
    fn route_over_travel_time_is_not_a_feasible_edge() {
        let mut case = sample();
        for route in case["routes"].as_array_mut().unwrap() {
            if route["cohort_id"] == "ANON_ZONE_C" && route["site_id"] == "SYNTHETIC_POD_2" {
                route["travel_time_s"] = json!(5000);
            }
        }
        let report = evaluate(&case).unwrap();
        assert_eq!(report["accessible_places_feasible"], 5);
    }

    #[test]
    fn oracle_never_assigns_or_reserves_people() {
        let report = evaluate(&sample()).unwrap();
        assert_eq!(report["assignment"], "NONE");
        assert_eq!(report["reservation"], "NONE");
        assert_eq!(report["execution_status"], "AGGREGATE_FEASIBILITY_ONLY_NO_ASSIGNMENT");
    }

    #[test]
    fn duplicate_route_pair_is_rejected() {
        let mut case = sample();
        let route = case["routes"][0].clone();
        case["routes"].as_array_mut().unwrap().push(route);
        assert!(evaluate(&case).is_err());
    }
}
