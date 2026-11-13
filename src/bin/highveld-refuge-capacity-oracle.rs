use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, process,
};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_CAPACITY_ORACLE_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_CAPACITY_ORACLE_CASE_V0.1.schema.json"
);
const MAX_SITES: usize = 8;
const MAX_GROUPS: usize = 8;
const MAX_PLACES: usize = 32;
const DEFAULT_NODE_LIMIT: usize = 100_000;

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn u64_at(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn finite_at(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64).filter(|number| number.is_finite())
}

#[derive(Clone)]
struct SiteState {
    id: String,
    total_free: u64,
    accessible_free: u64,
    water_free_ml: u64,
    eligible: bool,
    reasons: Vec<String>,
}

#[derive(Clone)]
struct Unit {
    group_index: usize,
    accessible: bool,
    water_ml: u64,
    candidate_sites: Vec<usize>,
}

#[derive(Clone)]
struct SearchState {
    sites: Vec<SiteState>,
    total_assignments: Vec<Vec<u64>>,
    accessible_assignments: Vec<Vec<u64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchResult {
    Found,
    Exhausted,
    LimitReached,
}

fn search(
    index: usize,
    units: &[Unit],
    state: &mut SearchState,
    visits: &mut usize,
    limit: usize,
) -> SearchResult {
    if index == units.len() {
        return SearchResult::Found;
    }
    *visits += 1;
    if *visits > limit {
        return SearchResult::LimitReached;
    }

    let unit = &units[index];
    for &site_index in &unit.candidate_sites {
        let can_allocate = {
            let site = &state.sites[site_index];
            site.eligible
                && site.total_free > 0
                && (!unit.accessible || site.accessible_free > 0)
                && site.water_free_ml >= unit.water_ml
        };
        if !can_allocate {
            continue;
        }

        state.sites[site_index].total_free -= 1;
        if unit.accessible {
            state.sites[site_index].accessible_free -= 1;
            state.accessible_assignments[unit.group_index][site_index] += 1;
        }
        state.sites[site_index].water_free_ml -= unit.water_ml;
        state.total_assignments[unit.group_index][site_index] += 1;

        match search(index + 1, units, state, visits, limit) {
            SearchResult::Found => return SearchResult::Found,
            SearchResult::LimitReached => return SearchResult::LimitReached,
            SearchResult::Exhausted => {}
        }

        state.total_assignments[unit.group_index][site_index] -= 1;
        if unit.accessible {
            state.accessible_assignments[unit.group_index][site_index] -= 1;
            state.sites[site_index].accessible_free += 1;
        }
        state.sites[site_index].total_free += 1;
        state.sites[site_index].water_free_ml += unit.water_ml;
    }

    SearchResult::Exhausted
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("schema JSON parse failed: {error}")])?;
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        return Err(vec!["schema must declare JSON Schema Draft 2020-12".to_owned()]);
    }

    for (key, expected) in [
        ("study_id", "HIGHVELD-REFUGE-CAPACITY-ORACLE-001"),
        ("maturity", "SYNTHETIC_KNOWN_ANSWER"),
        ("execution_status", "NO_REAL_REFUGE_OR_DEMAND_DATA"),
    ] {
        if text_at(case, key) != Some(expected) {
            return Err(vec![format!("{key} must remain {expected}")]);
        }
    }

    let network_state = text_at(case, "network_state").unwrap_or("UNKNOWN");
    if !["CONNECTED", "PARTITIONED"].contains(&network_state) {
        return Ok(json!({
            "oracle_state": "UNQUALIFIED_INPUT",
            "reason": "NETWORK_STATE_UNKNOWN",
            "allocation_witness": [],
            "occupant_assignment": "NONE",
            "execution_status": "NO_COMMITMENT_NO_BOOKING"
        }));
    }
    if text_at(case, "local_refuge_directory_state") != Some("QUALIFIED") {
        return Ok(json!({
            "oracle_state": "UNQUALIFIED_INPUT",
            "reason": "LOCAL_REFUGE_DIRECTORY_NOT_QUALIFIED",
            "allocation_witness": [],
            "occupant_assignment": "NONE",
            "execution_status": "NO_COMMITMENT_NO_BOOKING"
        }));
    }

    let groups = case
        .get("demands")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["demands must be an array".to_owned()])?;
    let sites_input = case
        .get("sites")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["sites must be an array".to_owned()])?;

    if groups.is_empty()
        || sites_input.is_empty()
        || groups.len() > MAX_GROUPS
        || sites_input.len() > MAX_SITES
    {
        return Err(vec!["input exceeds the bounded oracle envelope".to_owned()]);
    }

    let mut group_ids = BTreeSet::new();
    let mut site_ids = BTreeSet::new();
    let mut total_places = 0_u64;
    for group in groups {
        let places = u64_at(group, "total_places")
            .ok_or_else(|| vec!["total_places must be an unsigned integer".to_owned()])?;
        total_places = total_places
            .checked_add(places)
            .ok_or_else(|| vec!["total place count overflowed".to_owned()])?;
    }
    if total_places == 0 || total_places > MAX_PLACES as u64 {
        return Err(vec![format!(
            "total anonymous place count must be 1..={MAX_PLACES}"
        )]);
    }

    let mut group_routes = Vec::new();
    let mut total_accessible = 0_u64;
    for group in groups {
        let id = text_at(group, "cohort_id")
            .ok_or_else(|| vec!["cohort_id is missing".to_owned()])?;
        if !group_ids.insert(id.to_owned()) {
            return Err(vec![format!("duplicate demand cohort ID {id}")]);
        }
        let total = u64_at(group, "total_places")
            .ok_or_else(|| vec![format!("{id}: total_places must be unsigned")])?;
        let accessible = u64_at(group, "accessible_places")
            .ok_or_else(|| vec![format!("{id}: accessible_places must be unsigned")])?;
        let water = u64_at(group, "safe_water_ml_per_place")
            .ok_or_else(|| vec![format!("{id}: safe_water_ml_per_place must be unsigned")])?;
        let max_time = finite_at(group, "maximum_route_travel_time_s")
            .ok_or_else(|| vec![format!("{id}: route time bound missing")])?;
        if total == 0 || accessible > total || max_time <= 0.0 {
            return Err(vec![format!("{id}: invalid demand bounds")]);
        }
        total_accessible += accessible;

        let routes = group
            .get("routes")
            .and_then(Value::as_array)
            .ok_or_else(|| vec![format!("{id}: routes must be an array")])?;
        let mut route_by_site = BTreeMap::new();
        for route in routes {
            let site_id = text_at(route, "site_id")
                .ok_or_else(|| vec![format!("{id}: route site ID missing")])?;
            if route_by_site.insert(site_id.to_owned(), route.clone()).is_some() {
                return Err(vec![format!("{id}: duplicate route for site {site_id}")]);
            }
        }
        group_routes.push((water, max_time, route_by_site));
    }

    let mut site_states = Vec::new();
    let mut site_index = BTreeMap::new();
    for site in sites_input {
        let id = text_at(site, "site_id").ok_or_else(|| vec!["site_id is missing".to_owned()])?;
        if !site_ids.insert(id.to_owned()) {
            return Err(vec![format!("duplicate refuge site ID {id}")]);
        }

        let mut reasons = Vec::new();
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
                reasons.push(format!("MISSING_EVIDENCE_REFERENCE_{key}"));
            }
        }

        let total = u64_at(site, "total_capacity");
        let occupied = u64_at(site, "current_occupancy");
        let committed = u64_at(site, "committed_reserve");
        let accessible = u64_at(site, "accessible_capacity");
        let accessible_occupied = u64_at(site, "accessible_occupancy");
        let accessible_committed = u64_at(site, "accessible_committed_reserve");
        let water = u64_at(site, "safe_water_available_ml");
        let water_committed = u64_at(site, "safe_water_committed_ml");
        let water_reserve = u64_at(site, "safe_water_protected_reserve_ml");

        let resources = match (
            total,
            occupied,
            committed,
            accessible,
            accessible_occupied,
            accessible_committed,
            water,
            water_committed,
            water_reserve,
        ) {
            (
                Some(total),
                Some(occupied),
                Some(committed),
                Some(accessible),
                Some(accessible_occupied),
                Some(accessible_committed),
                Some(water),
                Some(water_committed),
                Some(water_reserve),
            ) => {
                let invalid = occupied > total
                    || committed > total.saturating_sub(occupied)
                    || accessible > total
                    || accessible_occupied > accessible
                    || accessible_committed > accessible.saturating_sub(accessible_occupied)
                    || accessible_occupied > occupied
                    || accessible_committed > committed
                    || water_committed > water
                    || water_reserve > water.saturating_sub(water_committed);
                if invalid {
                    reasons.push("CAPACITY_OR_WATER_ACCOUNTING_INVALID".to_owned());
                    (0, 0, 0)
                } else {
                    (
                        total - occupied - committed,
                        accessible - accessible_occupied - accessible_committed,
                        water - water_committed - water_reserve,
                    )
                }
            }
            _ => {
                reasons.push("CAPACITY_OR_WATER_ACCOUNT_UNKNOWN".to_owned());
                (0, 0, 0)
            }
        };

        let eligible = reasons.is_empty();
        let (total_free, accessible_free, water_free_ml) = if eligible {
            resources
        } else {
            (0, 0, 0)
        };
        site_index.insert(id.to_owned(), site_states.len());
        site_states.push(SiteState {
            id: id.to_owned(),
            total_free,
            accessible_free,
            water_free_ml,
            eligible,
            reasons,
        });
    }

    let mut units = Vec::new();
    let mut candidate_edges = 0_usize;
    for (group_index, group) in groups.iter().enumerate() {
        let (water_ml, max_time, routes) = &group_routes[group_index];
        let accessible_places = u64_at(group, "accessible_places").unwrap_or(0);
        let total = u64_at(group, "total_places").unwrap_or(0);

        for accessible in [true, false] {
            let count = if accessible {
                accessible_places
            } else {
                total - accessible_places
            };
            for _ in 0..count {
                let mut candidates = Vec::new();
                for (site_id, route) in routes {
                    let Some(&index) = site_index.get(site_id) else {
                        continue;
                    };
                    if !site_states[index].eligible
                        || text_at(route, "route_state") != Some("REACHABLE")
                        || finite_at(route, "travel_time_s")
                            .is_none_or(|seconds| seconds > *max_time)
                        || text_at(route, "route_evidence_ref").is_none()
                    {
                        continue;
                    }
                    if accessible
                        && (text_at(route, "accessible_route_state") != Some("QUALIFIED")
                            || text_at(route, "accessibility_evidence_ref").is_none())
                    {
                        continue;
                    }
                    candidates.push(index);
                }
                candidates.sort_by(|left, right| {
                    site_states[*left].id.cmp(&site_states[*right].id)
                });
                candidates.dedup();
                candidate_edges += candidates.len();
                units.push(Unit {
                    group_index,
                    accessible,
                    water_ml: *water_ml,
                    candidate_sites: candidates,
                });
            }
        }
    }

    // Most constrained place-units first; this is search ordering, not a service priority.
    units.sort_by_key(|unit| {
        (
            unit.candidate_sites.len(),
            if unit.accessible { 0 } else { 1 },
            unit.group_index,
        )
    });

    let requested_limit = u64_at(case, "search_node_limit")
        .unwrap_or(DEFAULT_NODE_LIMIT as u64) as usize;
    let node_limit = requested_limit.clamp(1, DEFAULT_NODE_LIMIT);
    let mut state = SearchState {
        sites: site_states.clone(),
        total_assignments: vec![vec![0; site_states.len()]; groups.len()],
        accessible_assignments: vec![vec![0; site_states.len()]; groups.len()],
    };
    let mut visits = 0_usize;
    let result = search(0, &units, &mut state, &mut visits, node_limit);

    let report = match result {
        SearchResult::Found => {
            let mut witness = Vec::new();
            for (group_index, row) in state.total_assignments.iter().enumerate() {
                for (site_index, count) in row.iter().enumerate() {
                    if *count > 0 {
                        witness.push(json!({
                            "cohort_id": text_at(&groups[group_index], "cohort_id").unwrap_or_default(),
                            "site_id": state.sites[site_index].id,
                            "places": count,
                            "accessible_places": state.accessible_assignments[group_index][site_index],
                            "commitment": "NONE",
                            "human_review_required": true
                        }));
                    }
                }
            }
            json!({
                "oracle_state": "ALL_COHORTS_FEASIBLE",
                "search_exactness": "EXACT_WITHIN_DECLARED_BOUNDED_GRAPH",
                "demand_group_count": groups.len(),
                "total_places_requested": total_places,
                "accessible_places_requested": total_accessible,
                "candidate_route_edge_count": candidate_edges,
                "search_nodes_visited": visits,
                "allocation_witness": witness,
                "remaining_site_resources": state.sites.iter().map(|site| json!({
                    "site_id": site.id,
                    "remaining_total_places": site.total_free,
                    "remaining_accessible_places": site.accessible_free,
                    "remaining_safe_water_ml": site.water_free_ml
                })).collect::<Vec<_>>(),
                "occupant_assignment": "NONE",
                "booking_or_reservation": "NONE",
                "execution_status": "NO_COMMITMENT_NO_BOOKING"
            })
        }
        SearchResult::Exhausted => json!({
            "oracle_state": "NO_COMPLETE_FEASIBLE_ALLOCATION",
            "search_exactness": "EXACT_WITHIN_DECLARED_BOUNDED_GRAPH",
            "demand_group_count": groups.len(),
            "total_places_requested": total_places,
            "accessible_places_requested": total_accessible,
            "candidate_route_edge_count": candidate_edges,
            "search_nodes_visited": visits,
            "allocation_witness": [],
            "occupant_assignment": "NONE",
            "booking_or_reservation": "NONE",
            "execution_status": "NO_COMMITMENT_NO_BOOKING"
        }),
        SearchResult::LimitReached => json!({
            "oracle_state": "INDETERMINATE_LIMIT_REACHED",
            "search_exactness": "NOT_PROVEN",
            "demand_group_count": groups.len(),
            "total_places_requested": total_places,
            "accessible_places_requested": total_accessible,
            "candidate_route_edge_count": candidate_edges,
            "search_nodes_visited": visits,
            "allocation_witness": [],
            "occupant_assignment": "NONE",
            "booking_or_reservation": "NONE",
            "execution_status": "NO_COMMITMENT_NO_BOOKING"
        }),
    };

    let site_eligibility = site_states.iter().map(|site| json!({
        "site_id": site.id,
        "eligible": site.eligible,
        "reasons": site.reasons
    })).collect::<Vec<_>>();
    let mut report = report;
    report["site_eligibility"] = json!(site_eligibility);
    report["network_state"] = json!(network_state);
    report["schema_engine"] = json!("NOT_RUN; schema declaration and semantic invariants are checked by Rust");
    report["claim_ceiling"] = json!("Synthetic bounded feasibility only; no actual refuge, booking or occupant assignment");
    Ok(report)
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value =
                serde_json::from_str(CASE).expect("committed aggregate refuge case JSON");
            let report = evaluate(&case).expect("aggregate refuge case structure");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["oracle_state"] != "ALL_COHORTS_FEASIBLE" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-refuge-capacity-oracle");
            println!("  --self-test  solve the committed synthetic aggregate capacity case");
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
    fn complete_demand_is_feasible_across_shared_sites_during_partition() {
        let report = evaluate(&sample()).expect("case evaluates");
        assert_eq!(report["oracle_state"], "ALL_COHORTS_FEASIBLE");
        assert_eq!(
            report["search_exactness"],
            "EXACT_WITHIN_DECLARED_BOUNDED_GRAPH"
        );
        assert_eq!(report["total_places_requested"], 6);
        assert_eq!(report["allocation_witness"].as_array().unwrap().len(), 4);
        assert_eq!(report["occupant_assignment"], "NONE");
    }

    #[test]
    fn independent_site_checks_cannot_double_book_shared_capacity() {
        let mut case = sample();
        case["demands"][1]["routes"][0]["site_id"] = json!("POD_A");
        case["demands"][1]["routes"].as_array_mut().unwrap().truncate(1);
        case["sites"][0]["total_capacity"] = json!(4);
        case["sites"][0]["current_occupancy"] = json!(0);
        case["sites"][0]["accessible_capacity"] = json!(2);
        case["sites"][0]["safe_water_available_ml"] = json!(50000);
        case["sites"][0]["safe_water_committed_ml"] = json!(0);
        case["sites"][0]["safe_water_protected_reserve_ml"] = json!(0);
        case["sites"].as_array_mut().unwrap().truncate(1);
        let report = evaluate(&case).unwrap();
        assert_eq!(report["oracle_state"], "NO_COMPLETE_FEASIBLE_ALLOCATION");
        assert!(report["allocation_witness"].as_array().unwrap().is_empty());
    }

    #[test]
    fn accessible_demand_requires_qualified_accessible_route_and_capacity() {
        let mut case = sample();
        case["demands"][0]["routes"][0]["accessible_route_state"] = json!("UNKNOWN");
        case["demands"][0]["routes"][1]["accessible_route_state"] = json!("FAILED");
        let report = evaluate(&case).unwrap();
        assert_eq!(report["oracle_state"], "NO_COMPLETE_FEASIBLE_ALLOCATION");
    }

    #[test]
    fn protected_water_reserve_is_not_spendable_across_cohorts() {
        let mut case = sample();
        case["sites"][0]["safe_water_available_ml"] = json!(4000);
        case["sites"][0]["safe_water_committed_ml"] = json!(1000);
        case["sites"][0]["safe_water_protected_reserve_ml"] = json!(2000);
        case["sites"][1]["safe_water_available_ml"] = json!(4000);
        case["sites"][1]["safe_water_committed_ml"] = json!(1000);
        case["sites"][1]["safe_water_protected_reserve_ml"] = json!(2000);
        let report = evaluate(&case).unwrap();
        assert_eq!(report["oracle_state"], "NO_COMPLETE_FEASIBLE_ALLOCATION");
    }

    #[test]
    fn unknown_common_service_dependency_removes_site_from_candidate_graph() {
        let mut case = sample();
        case["sites"][0]["ventilation_state"] = json!("UNKNOWN");
        let report = evaluate(&case).unwrap();
        assert_eq!(report["oracle_state"], "NO_COMPLETE_FEASIBLE_ALLOCATION");
    }

    #[test]
    fn node_limit_exhaustion_is_indeterminate_not_a_false_decision() {
        let mut case = sample();
        case["search_node_limit"] = json!(1);
        let report = evaluate(&case).unwrap();
        assert_eq!(report["oracle_state"], "INDETERMINATE_LIMIT_REACHED");
        assert_eq!(report["search_exactness"], "NOT_PROVEN");
        assert!(report["allocation_witness"].as_array().unwrap().is_empty());
    }

    #[test]
    fn unknown_network_or_directory_fails_closed() {
        let mut case = sample();
        case["network_state"] = json!("UNKNOWN");
        let report = evaluate(&case).unwrap();
        assert_eq!(report["oracle_state"], "UNQUALIFIED_INPUT");
        case["network_state"] = json!("PARTITIONED");
        case["local_refuge_directory_state"] = json!("UNKNOWN");
        let report = evaluate(&case).unwrap();
        assert_eq!(report["oracle_state"], "UNQUALIFIED_INPUT");
    }
}
