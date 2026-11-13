use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::{env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_AGGREGATE_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_AGGREGATE_CASE_V0.1.schema.json"
);

#[derive(Clone, Debug)]
struct FlowResult {
    total: u64,
    by_site: Vec<u64>,
    by_zone_site: Vec<Vec<u64>>,
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|number| number.is_finite())
}

fn uint_at(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn max_flow(demands: &[u64], edges: &[Vec<bool>], site_caps: &[u64]) -> FlowResult {
    let zone_count = demands.len();
    let site_count = site_caps.len();
    let source = 0;
    let first_zone = 1;
    let first_site = first_zone + zone_count;
    let sink = first_site + site_count;
    let node_count = sink + 1;

    let mut residual = vec![vec![0_u64; node_count]; node_count];
    let mut original = vec![vec![0_u64; node_count]; node_count];
    let mut adjacency = vec![Vec::<usize>::new(); node_count];

    let mut add_edge = |from: usize, to: usize, capacity: u64| {
        if residual[from][to] == 0 && residual[to][from] == 0 {
            adjacency[from].push(to);
            adjacency[to].push(from);
        }
        residual[from][to] = residual[from][to].saturating_add(capacity);
        original[from][to] = original[from][to].saturating_add(capacity);
    };

    for (zone, demand) in demands.iter().enumerate() {
        add_edge(source, first_zone + zone, *demand);
        for site in 0..site_count {
            if edges.get(zone).and_then(|row| row.get(site)).copied().unwrap_or(false) {
                add_edge(first_zone + zone, first_site + site, *demand);
            }
        }
    }
    for (site, capacity) in site_caps.iter().enumerate() {
        add_edge(first_site + site, sink, *capacity);
    }

    let mut total = 0_u64;
    loop {
        let mut parent = vec![usize::MAX; node_count];
        let mut queue = VecDeque::new();
        parent[source] = source;
        queue.push_back(source);
        while let Some(node) = queue.pop_front() {
            for &next in &adjacency[node] {
                if parent[next] == usize::MAX && residual[node][next] > 0 {
                    parent[next] = node;
                    if next == sink {
                        break;
                    }
                    queue.push_back(next);
                }
            }
            if parent[sink] != usize::MAX {
                break;
            }
        }
        if parent[sink] == usize::MAX {
            break;
        }
        let mut path_capacity = u64::MAX;
        let mut cursor = sink;
        while cursor != source {
            let prev = parent[cursor];
            path_capacity = path_capacity.min(residual[prev][cursor]);
            cursor = prev;
        }
        cursor = sink;
        while cursor != source {
            let prev = parent[cursor];
            residual[prev][cursor] -= path_capacity;
            residual[cursor][prev] = residual[cursor][prev].saturating_add(path_capacity);
            cursor = prev;
        }
        total = total.saturating_add(path_capacity);
    }

    let mut by_zone_site = vec![vec![0_u64; site_count]; zone_count];
    let mut by_site = vec![0_u64; site_count];
    for zone in 0..zone_count {
        for site in 0..site_count {
            let from = first_zone + zone;
            let to = first_site + site;
            let flow = original[from][to].saturating_sub(residual[from][to]);
            by_zone_site[zone][site] = flow;
            by_site[site] = by_site[site].saturating_add(flow);
        }
    }
    FlowResult { total, by_site, by_zone_site }
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("schema JSON parse failed: {error}")])?;
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-REFUGE-NETWORK-001"),
        ("maturity", "SYNTHETIC_KNOWN_ANSWER"),
        ("execution_status", "NO_SITE_OR_ROUTE_VALIDATION"),
    ] {
        if text_at(case, key) != Some(expected) {
            errors.push(format!("{key} must remain {expected}"));
        }
    }
    let water_per_place = number_at(case, "safe_water_l_per_place")
        .ok_or_else(|| vec!["safe_water_l_per_place must be finite".to_owned()])?;
    let travel_limit = number_at(case, "maximum_route_travel_time_s")
        .ok_or_else(|| vec!["maximum_route_travel_time_s must be finite".to_owned()])?;
    if water_per_place <= 0.0 || travel_limit <= 0.0 {
        errors.push("water-per-place and travel-time limit must be positive".to_owned());
    }
    if text_at(case, "local_directory_state") != Some("QUALIFIED") {
        errors.push("local refuge directory is not qualified".to_owned());
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let zones = case.get("demand_zones").and_then(Value::as_array)
        .ok_or_else(|| vec!["demand_zones must be an array".to_owned()])?;
    let sites = case.get("sites").and_then(Value::as_array)
        .ok_or_else(|| vec!["sites must be an array".to_owned()])?;
    if zones.is_empty() || sites.is_empty() {
        return Err(vec!["at least one demand zone and one refuge site are required".to_owned()]);
    }

    let mut zone_ids = BTreeSet::new();
    let mut total_demands = Vec::new();
    let mut accessible_demands = Vec::new();
    for zone in zones {
        let id = text_at(zone, "zone_id").ok_or_else(|| vec!["demand zone ID is missing".to_owned()])?;
        if !zone_ids.insert(id.to_owned()) {
            errors.push(format!("duplicate demand zone ID {id}"));
        }
        let total = uint_at(zone, "total_places")
            .ok_or_else(|| vec![format!("{id}: total_places must be an unsigned integer")])?;
        let accessible = uint_at(zone, "accessible_places")
            .ok_or_else(|| vec![format!("{id}: accessible_places must be an unsigned integer")])?;
        if total == 0 || accessible > total {
            errors.push(format!("{id}: invalid total/accessibility demand"));
        }
        total_demands.push(total);
        accessible_demands.push(accessible);
        let routes = zone.get("route_options").and_then(Value::as_array)
            .ok_or_else(|| vec![format!("{id}: route_options must be an array")])?;
        let mut route_ids = BTreeSet::new();
        for route in routes {
            let site_id = text_at(route, "site_id")
                .ok_or_else(|| vec![format!("{id}: route option has no site_id")])?;
            if !route_ids.insert(site_id.to_owned()) {
                errors.push(format!("{id}: duplicate route option for {site_id}"));
            }
            if text_at(route, "evidence_ref").is_none() {
                errors.push(format!("{id}: route evidence missing for {site_id}"));
            }
        }
    }

    let active_failures = case.get("active_failure_domains").and_then(Value::as_array)
        .ok_or_else(|| vec!["active_failure_domains must be an array".to_owned()])?
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let mut site_ids = BTreeSet::new();
    let mut serviceable_total = Vec::new();
    let mut serviceable_accessible = Vec::new();
    let mut site_status = Vec::new();
    let mut site_index = BTreeMap::new();

    for (index, site) in sites.iter().enumerate() {
        let id = text_at(site, "site_id").ok_or_else(|| vec!["site ID is missing".to_owned()])?;
        if !site_ids.insert(id.to_owned()) {
            errors.push(format!("duplicate site ID {id}"));
        }
        site_index.insert(id.to_owned(), index);
        let failure_domains = site.get("failure_domains").and_then(Value::as_array)
            .ok_or_else(|| vec![format!("{id}: failure_domains must be an array")])?;
        let cut_domains = failure_domains.iter().filter_map(Value::as_str)
            .filter(|domain| active_failures.contains(domain))
            .map(str::to_owned).collect::<Vec<_>>();
        let mut reasons = Vec::new();
        if !cut_domains.is_empty() {
            reasons.push(format!("ACTIVE_COMMON_CAUSE_FAILURE:{}", cut_domains.join(",")));
        }
        if text_at(site, "operating_state") != Some("QUALIFIED") {
            reasons.push("OPERATING_STATE_NOT_QUALIFIED".to_owned());
        }
        for (key, reason) in [
            ("thermal_service_state", "THERMAL_SERVICE_NOT_QUALIFIED"),
            ("ventilation_state", "VENTILATION_NOT_QUALIFIED"),
            ("safe_water_rights_state", "SAFE_WATER_RIGHTS_NOT_QUALIFIED"),
            ("safe_water_quality_state", "SAFE_WATER_QUALITY_NOT_QUALIFIED"),
            ("staffing_state", "STAFFING_NOT_QUALIFIED"),
        ] {
            if text_at(site, key) != Some("QUALIFIED") {
                reasons.push(reason.to_owned());
            }
        }
        for key in ["site_evidence_ref", "capacity_evidence_ref", "water_evidence_ref", "service_evidence_ref"] {
            if text_at(site, key).is_none() {
                reasons.push(format!("MISSING_SITE_EVIDENCE_{key}"));
            }
        }

        let total = uint_at(site, "total_capacity");
        let occupied = uint_at(site, "current_occupancy");
        let committed = uint_at(site, "committed_reserve");
        let accessible = uint_at(site, "accessible_capacity");
        let accessible_occupied = uint_at(site, "accessible_occupancy");
        let accessible_committed = uint_at(site, "accessible_committed_reserve");
        let (free_total, free_accessible) = match (
            total, occupied, committed, accessible, accessible_occupied, accessible_committed
        ) {
            (Some(cap), Some(used), Some(reserved), Some(access_cap), Some(access_used), Some(access_reserved)) => {
                let invalid = used > cap
                    || reserved > cap - used
                    || access_cap > cap
                    || access_used > access_cap
                    || access_reserved > access_cap - access_used
                    || access_used > used
                    || access_reserved > reserved;
                if invalid {
                    reasons.push("CAPACITY_ACCOUNTING_INVALID".to_owned());
                    (0_u64, 0_u64)
                } else {
                    (cap - used - reserved, access_cap - access_used - access_reserved)
                }
            }
            _ => {
                reasons.push("CAPACITY_ACCOUNTING_UNKNOWN".to_owned());
                (0_u64, 0_u64)
            }
        };
        let water_free = match (
            number_at(site, "safe_water_available_l"),
            number_at(site, "safe_water_committed_l"),
            number_at(site, "safe_water_protected_reserve_l")
        ) {
            (Some(available), Some(used), Some(reserve))
                if available.is_finite() && used >= 0.0 && reserve >= 0.0 && available >= used + reserve =>
            {
                available - used - reserve
            }
            _ => {
                reasons.push("SAFE_WATER_ACCOUNT_UNKNOWN_OR_BELOW_COMMITMENTS".to_owned());
                0.0
            }
        };
        let water_capacity = (water_free / water_per_place).floor().max(0.0) as u64;
        let total_capacity = free_total.min(water_capacity);
        let accessible_capacity = free_accessible.min(total_capacity);
        let available = reasons.is_empty();
        serviceable_total.push(if available { total_capacity } else { 0 });
        serviceable_accessible.push(if available { accessible_capacity } else { 0 });
        site_status.push(json!({
            "site_id":id,
            "site_state":if available {"QUALIFIED"} else {"UNAVAILABLE_OR_UNVERIFIED"},
            "reasons":reasons,
            "free_total_places":free_total,
            "free_accessible_places":free_accessible,
            "free_safe_water_l":water_free,
            "water_supported_places":water_capacity,
            "serviceable_total_places":if available {total_capacity} else {0},
            "serviceable_accessible_places":if available {accessible_capacity} else {0},
            "active_failure_domains":cut_domains
        }));
    }
    for zone in zones {
        for route in zone["route_options"].as_array().unwrap() {
            let site_id = text_at(route, "site_id").unwrap();
            if !site_index.contains_key(site_id) {
                errors.push(format!("route from {} references unknown site {site_id}", text_at(zone, "zone_id").unwrap()));
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let zone_count = zones.len();
    let site_count = sites.len();
    let mut accessible_edges = vec![vec![false; site_count]; zone_count];
    let mut general_edges = vec![vec![false; site_count]; zone_count];
    for (zone_index, zone) in zones.iter().enumerate() {
        for route in zone["route_options"].as_array().unwrap() {
            let site_id = text_at(route, "site_id").unwrap();
            let site = site_index[site_id];
            let time_ok = number_at(route, "travel_time_s")
                .is_some_and(|seconds| seconds <= travel_limit);
            let reachable = text_at(route, "route_state") == Some("REACHABLE") && time_ok;
            let ref_ok = text_at(route, "evidence_ref").is_some();
            if reachable && ref_ok && serviceable_total[site] > 0 {
                general_edges[zone_index][site] = true;
                if text_at(route, "accessibility_state") == Some("QUALIFIED")
                    && serviceable_accessible[site] > 0
                {
                    accessible_edges[zone_index][site] = true;
                }
            }
        }
    }

    let accessible_flow = max_flow(&accessible_demands, &accessible_edges, &serviceable_accessible);
    let general_demands = total_demands.iter().zip(&accessible_demands)
        .map(|(total, accessible)| total - accessible).collect::<Vec<_>>();
    let residual_total = serviceable_total.iter().zip(&accessible_flow.by_site)
        .map(|(capacity, used)| capacity.saturating_sub(*used)).collect::<Vec<_>>();
    let general_flow = max_flow(&general_demands, &general_edges, &residual_total);

    let mut zone_reports = Vec::new();
    let mut allocation_witness = Vec::new();
    let mut total_demand = 0_u64;
    let mut total_accessible = 0_u64;
    let mut total_served = 0_u64;
    let mut accessible_served = 0_u64;
    let mut general_served = 0_u64;
    for (zone_index, zone) in zones.iter().enumerate() {
        let accessible_demand = accessible_demands[zone_index];
        let general_demand = general_demands[zone_index];
        let served_accessible = accessible_flow.by_zone_site[zone_index].iter().sum::<u64>();
        let served_general = general_flow.by_zone_site[zone_index].iter().sum::<u64>();
        total_demand = total_demand.saturating_add(total_demands[zone_index]);
        total_accessible = total_accessible.saturating_add(accessible_demand);
        total_served = total_served.saturating_add(served_accessible + served_general);
        accessible_served = accessible_served.saturating_add(served_accessible);
        general_served = general_served.saturating_add(served_general);
        zone_reports.push(json!({
            "zone_id":text_at(zone,"zone_id").unwrap_or_default(),
            "total_demand":total_demands[zone_index],
            "accessible_demand":accessible_demand,
            "general_demand":general_demand,
            "accessible_served":served_accessible,
            "general_served":served_general,
            "total_served":served_accessible + served_general,
            "unmet_accessible":accessible_demand.saturating_sub(served_accessible),
            "unmet_general":general_demand.saturating_sub(served_general)
        }));
        for (site_index, site) in sites.iter().enumerate() {
            let a = accessible_flow.by_zone_site[zone_index][site_index];
            let g = general_flow.by_zone_site[zone_index][site_index];
            if a > 0 || g > 0 {
                allocation_witness.push(json!({
                    "zone_id":text_at(zone,"zone_id").unwrap_or_default(),
                    "site_id":text_at(site,"site_id").unwrap_or_default(),
                    "accessible_places":a,
                    "general_places":g
                }));
            }
        }
    }

    let unmet_total = total_demand.saturating_sub(total_served);
    let unmet_accessible = total_accessible.saturating_sub(accessible_served);
    let coverage = if total_demand == 0 { 0.0 } else { total_served as f64 / total_demand as f64 };
    Ok(json!({
        "audit_state":if unmet_total == 0 && unmet_accessible == 0 {"AGGREGATE_DEMAND_FEASIBLE"} else {"AGGREGATE_DEMAND_SHORTFALL"},
        "study_id":"HIGHVELD-REFUGE-NETWORK-001",
        "case_id":text_at(case,"case_id").unwrap_or_default(),
        "network_state":text_at(case,"network_state").unwrap_or("UNKNOWN"),
        "active_failure_domains":active_failures,
        "total_demand_places":total_demand,
        "accessible_demand_places":total_accessible,
        "served_total_places":total_served,
        "served_accessible_places":accessible_served,
        "served_general_places":general_served,
        "unmet_total_places":unmet_total,
        "unmet_accessible_places":unmet_accessible,
        "coverage_fraction":coverage,
        "demand_zones":zone_reports,
        "site_capacity_ledger":site_status,
        "allocation_witness":allocation_witness,
        "allocation_semantics":"ANONYMOUS_ZONE_LEVEL_COUNTS_ONLY_NO_PERSON_ASSIGNMENT",
        "infeasibility_semantics":"A feasible witness proves sufficiency for this input; a shortfall is a deterministic witness shortfall, not a proof of global infeasibility",
        "execution_status":"NO_BOOKING_NO_ROUTE_COMPUTATION_NO_OCCUPANT_ASSIGNMENT",
        "claim_ceiling":"Synthetic aggregate feasibility oracle only; no actual refuge network, route, resident or capacity has been validated"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value = serde_json::from_str(CASE).expect("aggregate refuge case JSON");
            let report = evaluate(&case).expect("aggregate refuge case structure");
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            if report["audit_state"] != "AGGREGATE_DEMAND_FEASIBLE" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-refuge-network-aggregate-oracle");
            println!("  --self-test  evaluate synthetic multi-zone capacity and common-cause case");
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
        serde_json::from_str(CASE).expect("aggregate case JSON")
    }

    #[test]
    fn baseline_flow_serves_all_zones_without_double_counting() {
        let report = evaluate(&sample()).expect("synthetic baseline evaluates");
        assert_eq!(report["audit_state"], "AGGREGATE_DEMAND_FEASIBLE");
        assert_eq!(report["total_demand_places"], 9);
        assert_eq!(report["served_total_places"], 9);
        assert_eq!(report["accessible_demand_places"], 3);
        assert_eq!(report["served_accessible_places"], 3);
        for site in report["site_capacity_ledger"].as_array().unwrap() {
            let id = site["site_id"].as_str().unwrap();
            let used = report["allocation_witness"]
                .as_array().unwrap().iter()
                .filter(|entry| entry["site_id"] == id)
                .map(|entry| entry["accessible_places"].as_u64().unwrap()
                    + entry["general_places"].as_u64().unwrap())
                .sum::<u64>();
            assert!(used <= site["serviceable_total_places"].as_u64().unwrap());
        }
    }

    #[test]
    fn shared_utility_failure_exposes_zone_service_gap() {
        let mut case = sample();
        case["active_failure_domains"] = json!(["SHARED_UTILITY"]);
        case["network_state"] = json!("PARTITIONED");
        let report = evaluate(&case).expect("common-cause case evaluates");
        assert_eq!(report["audit_state"], "AGGREGATE_DEMAND_SHORTFALL");
        assert_eq!(report["served_total_places"], 3);
        assert_eq!(report["unmet_total_places"], 6);
        assert!(report["site_capacity_ledger"].as_array().unwrap().iter()
            .filter(|site| site["site_id"] == "SYNTHETIC_POD_A" || site["site_id"] == "SYNTHETIC_HUB_B")
            .all(|site| site["serviceable_total_places"] == 0));
    }

    #[test]
    fn accessible_demand_requires_accessible_route_and_capacity() {
        let mut case = sample();
        case["demand_zones"][1]["route_options"][0]["accessibility_state"] = json!("FAILED");
        case["demand_zones"][1]["route_options"][1]["accessibility_state"] = json!("FAILED");
        let report = evaluate(&case).expect("accessibility case evaluates");
        assert!(report["unmet_accessible_places"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn water_reserve_reduces_serviceable_places() {
        let mut case = sample();
        case["sites"][0]["safe_water_available_l"] = json!(8.0);
        let report = evaluate(&case).expect("water-limited case evaluates");
        let site = report["site_capacity_ledger"].as_array().unwrap()
            .iter().find(|site| site["site_id"] == "SYNTHETIC_POD_A").unwrap();
        assert_eq!(site["free_safe_water_l"], 1.0);
        assert_eq!(site["water_supported_places"], 0);
        assert_eq!(site["serviceable_total_places"], 0);
    }

    #[test]
    fn duplicate_site_or_zone_ids_fail_closed() {
        let mut case = sample();
        let duplicate = case["sites"][0].clone();
        case["sites"].as_array_mut().unwrap().push(duplicate);
        assert!(evaluate(&case).is_err());

        let mut case = sample();
        let duplicate = case["demand_zones"][0].clone();
        case["demand_zones"].as_array_mut().unwrap().push(duplicate);
        assert!(evaluate(&case).is_err());
    }

    #[test]
    fn no_zone_place_can_be_counted_twice() {
        let mut case = sample();
        case["sites"][0]["total_capacity"] = json!(4);
        case["sites"][0]["current_occupancy"] = json!(0);
        case["sites"][0]["committed_reserve"] = json!(0);
        case["sites"][0]["accessible_capacity"] = json!(2);
        case["sites"][0]["accessible_occupancy"] = json!(0);
        case["sites"][0]["accessible_committed_reserve"] = json!(0);
        case["sites"][0]["safe_water_available_l"] = json!(50.0);
        case["active_failure_domains"] = json!(["SHARED_UTILITY", "LOCAL_AUTONOMY"]);
        case["demand_zones"][0]["route_options"] = json!([
            {"site_id":"SYNTHETIC_POD_A","route_state":"REACHABLE","accessibility_state":"QUALIFIED","travel_time_s":500,"evidence_ref":"SYNTHETIC_WEST_A_ROUTE"}
        ]);
        case["demand_zones"][1]["route_options"] = json!([
            {"site_id":"SYNTHETIC_POD_A","route_state":"REACHABLE","accessibility_state":"QUALIFIED","travel_time_s":500,"evidence_ref":"SYNTHETIC_EAST_A_ROUTE"}
        ]);
        case["sites"][0]["failure_domains"] = json!([]);
        let report = evaluate(&case).expect("shared capacity case evaluates");
        assert_eq!(report["served_total_places"], 4);
        assert_eq!(report["unmet_total_places"], 5);
        let used = report["allocation_witness"].as_array().unwrap().iter()
            .map(|entry| entry["accessible_places"].as_u64().unwrap()
                + entry["general_places"].as_u64().unwrap()).sum::<u64>();
        assert_eq!(used, 4);
    }
}
