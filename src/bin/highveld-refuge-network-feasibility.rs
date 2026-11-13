use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::{env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_NETWORK_FEASIBILITY_CASE_V0.1.json"
);
const TIME_SERIES_CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_SUPPLY_DEMAND_DEGRADATION_CASE_V0.1.json"
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

#[derive(Default)]
struct CohortServiceDebt {
    requested_place_hours: f64,
    served_place_hours: f64,
    unmet_place_hours: f64,
    accessible_requested_place_hours: f64,
    accessible_served_place_hours: f64,
    accessible_unmet_place_hours: f64,
    general_requested_place_hours: f64,
    general_served_place_hours: f64,
    general_unmet_place_hours: f64,
    minimum_snapshot_service_ratio: Option<f64>,
}

#[derive(Default)]
struct CohortServiceContinuity {
    baseline_service_ratio: Option<f64>,
    first_degradation_elapsed_h: Option<f64>,
    first_baseline_recovery_elapsed_h: Option<f64>,
    current_degraded_episode_start_elapsed_h: Option<f64>,
    current_degraded_episode_duration_h: f64,
    longest_degraded_episode_duration_h: f64,
    total_degraded_duration_h: f64,
    degraded_episode_count: u64,
    recovered_episode_count: u64,
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
        .map(|(id, quantity)| network.add_edge(source, demand_nodes[id], *quantity))
        .collect();
    let site_handles: Vec<EdgeHandle> = sites
        .iter()
        .map(|(id, capacity)| network.add_edge(site_nodes[id], sink, *capacity))
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

fn progressive_max_min_flow(
    demands: &[(String, u64)],
    sites: &[(String, u64)],
    links: &BTreeSet<(String, String)>,
    baseline_served: &[u64],
    fairness_totals: &[u64],
    tie_break_ranks: &[u64],
) -> FlowResult {
    let count = demands.len();
    assert_eq!(baseline_served.len(), count);
    assert_eq!(fairness_totals.len(), count);
    assert_eq!(tie_break_ranks.len(), count);

    let mut targets = vec![0_u64; count];
    let mut blocked = vec![false; count];

    loop {
        let mut order: Vec<usize> = (0..count)
            .filter(|index| demands[*index].1 > 0 && !blocked[*index])
            .collect();
        order.sort_by(|left, right| {
            let left_denominator = fairness_totals[*left].max(1);
            let right_denominator = fairness_totals[*right].max(1);
            let left_numerator = baseline_served[*left].saturating_add(targets[*left]);
            let right_numerator = baseline_served[*right].saturating_add(targets[*right]);

            (u128::from(left_numerator) * u128::from(right_denominator))
                .cmp(&(u128::from(right_numerator) * u128::from(left_denominator)))
                .then_with(|| tie_break_ranks[*left].cmp(&tie_break_ranks[*right]))
                .then_with(|| demands[*left].0.cmp(&demands[*right].0))
        });

        let mut advanced = false;
        for index in order {
            if targets[index] >= demands[index].1 {
                blocked[index] = true;
                continue;
            }

            targets[index] += 1;
            let required_flow: u64 = targets.iter().sum();
            let candidate_demands: Vec<(String, u64)> = demands
                .iter()
                .enumerate()
                .map(|(candidate_index, (id, _))| (id.clone(), targets[candidate_index]))
                .collect();

            if run_flow(&candidate_demands, sites, links).total == required_flow {
                advanced = true;
                break;
            }

            targets[index] -= 1;
            blocked[index] = true;
        }

        if !advanced {
            break;
        }
    }

    let fair_demands: Vec<(String, u64)> = demands
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id.clone(), targets[index]))
        .collect();
    run_flow(&fair_demands, sites, links)
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
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|number| number.is_finite())
}

fn evaluate_snapshot(case: &Value) -> Result<Value, Vec<String>> {
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

    let cohorts = case
        .get("cohorts")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["cohorts must be an array".to_owned()])?;
    let sites = case
        .get("sites")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["sites must be an array".to_owned()])?;
    let routes = case
        .get("routes")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["routes must be an array".to_owned()])?;
    if cohorts.is_empty() || sites.is_empty() {
        errors.push("at least one cohort and one site are required".to_owned());
    }

    let mut cohort_index = BTreeMap::<String, usize>::new();
    let mut total_requested = 0_u64;
    let mut accessible_requested = 0_u64;
    let mut cohort_data = Vec::<(String, u64, u64, f64)>::new();
    let mut cohort_tie_break_ranks = Vec::<u64>::new();
    let mut seen_tie_break_ranks = BTreeSet::new();
    for (index, cohort) in cohorts.iter().enumerate() {
        let id = text_at(cohort, "cohort_id")
            .unwrap_or("<missing-cohort-id>")
            .to_owned();
        if cohort_index.insert(id.clone(), index).is_some() {
            errors.push(format!("duplicate cohort identity {id}"));
        }
        let total = uint_at(cohort, "total_places_requested").unwrap_or(0);
        let accessible = uint_at(cohort, "accessible_places_requested").unwrap_or(0);
        let max_time = number_at(cohort, "maximum_route_travel_time_s").unwrap_or(0.0);
        let tie_break_rank = uint_at(cohort, "fairness_tie_break_rank");
        if id.starts_with("<missing") || total == 0 || accessible > total || max_time <= 0.0 {
            errors.push(format!("{id}: invalid cohort demand or travel-time limit"));
        }
        match tie_break_rank {
            Some(rank) if seen_tie_break_ranks.insert(rank) => {
                cohort_tie_break_ranks.push(rank);
            }
            Some(_) => errors.push(format!("{id}: duplicate fairness tie-break rank")),
            None => errors.push(format!(
                "{id}: fairness_tie_break_rank is missing or invalid"
            )),
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
        let id = text_at(site, "site_id")
            .unwrap_or("<missing-site-id>")
            .to_owned();
        if site_index.insert(id.clone(), index).is_some() {
            errors.push(format!("duplicate refuge site identity {id}"));
        }
        site_ids.push(id.clone());

        let mut reasons = Vec::new();
        if !local_directory_qualified {
            reasons.push("LOCAL_REFUGE_DIRECTORY_NOT_QUALIFIED".to_owned());
        }
        for (key, expected, reason) in [
            (
                "operating_window_state",
                "OPEN_FOR_REQUIRED_WINDOW",
                "NOT_OPEN_FOR_REQUIRED_WINDOW",
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
                "STAFFING_OR_MAINTENANCE_NOT_QUALIFIED",
            ),
        ] {
            if text_at(site, key) != Some(expected) {
                reasons.push(reason.to_owned());
            }
        }

        for key in [
            "capacity_evidence_ref",
            "service_evidence_ref",
            "water_evidence_ref",
        ] {
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
                if available >= 0.0
                    && reserved >= 0.0
                    && protected >= 0.0
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
            total_capacity,
            occupancy,
            committed,
            accessible_capacity,
            accessible_occupancy,
            accessible_committed,
        ) {
            (
                Some(total),
                Some(occupied),
                Some(reserved),
                Some(access),
                Some(access_occupied),
                Some(access_reserved),
            ) if occupied <= total
                && reserved <= total - occupied
                && access_occupied <= access
                && access_reserved <= access - access_occupied
                && access <= total
                && access_occupied <= occupied
                && access_reserved <= reserved =>
            {
                (
                    Some(total - occupied - reserved),
                    Some(access - access_occupied - access_reserved),
                )
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
            _ => None,
        };
        let effective_accessible = match (free_accessible, effective_total) {
            (Some(access), Some(total)) => Some(access.min(total)),
            _ => None,
        };

        let eligible =
            reasons.is_empty() && effective_total.is_some() && effective_accessible.is_some();
        site_effective_total.push(if eligible {
            effective_total.unwrap_or(0)
        } else {
            0
        });
        site_effective_accessible.push(if eligible {
            effective_accessible.unwrap_or(0)
        } else {
            0
        });
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
        let cohort_id = text_at(route, "cohort_id")
            .unwrap_or("<missing-cohort>")
            .to_owned();
        let site_id = text_at(route, "site_id")
            .unwrap_or("<missing-site>")
            .to_owned();
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
        let cohort = cohort_index
            .get(&cohort_id)
            .map(|index| &cohort_data[*index]);
        let site = site_index.get(&site_id).copied();
        let time = number_at(route, "travel_time_s");
        let time_ok = match (time, cohort) {
            (Some(seconds), Some((_, _, _, maximum))) => seconds >= 0.0 && seconds <= *maximum,
            _ => false,
        };
        if !time_ok {
            reasons.push("ROUTE_TRAVEL_TIME_UNKNOWN_OR_OVER_LIMIT".to_owned());
        }
        let route_reachable = text_at(route, "route_state") == Some("REACHABLE");
        if !route_reachable {
            reasons.push("ROUTE_NOT_REACHABLE".to_owned());
        }
        let site_ok = site
            .map(|i| site_reports[i]["site_eligibility"] == "ELIGIBLE")
            .unwrap_or(false);
        if !site_ok {
            reasons.push("REFUGE_SITE_NOT_ELIGIBLE".to_owned());
        }

        let accessibility = text_at(route, "accessibility_state");
        let general_access_ok = matches!(accessibility, Some("QUALIFIED" | "NOT_ACCESSIBLE"));
        let accessible_access_ok = accessibility == Some("QUALIFIED");
        if !general_access_ok {
            reasons.push("ACCESSIBILITY_STATE_UNKNOWN".to_owned());
        }

        let base_ok = time_ok
            && route_reachable
            && site_ok
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
        elapsed_h += duration;
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
            let remaining =
                site_effective_total[index].saturating_sub(accessible_flow.used_by_site[index]);
            (id.clone(), remaining)
        })
        .collect();
    let general_flow = run_flow(&general_demands, &residual_sites, &general_links);

    let accessible_fair_flow = progressive_max_min_flow(
        &accessible_demands,
        &accessible_sites,
        &accessible_links,
        &vec![0; cohort_data.len()],
        &cohort_data
            .iter()
            .map(|(_, _, accessible, _)| *accessible)
            .collect::<Vec<_>>(),
        &cohort_tie_break_ranks,
    );
    let fair_residual_sites: Vec<(String, u64)> = site_ids
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let remaining = site_effective_total[index]
                .saturating_sub(accessible_fair_flow.used_by_site[index]);
            (id.clone(), remaining)
        })
        .collect();
    let general_fair_baseline = accessible_fair_flow.served_by_cohort.clone();
    let general_fair_totals: Vec<u64> = cohort_data.iter().map(|(_, total, _, _)| *total).collect();
    let general_fair_flow = progressive_max_min_flow(
        &general_demands,
        &fair_residual_sites,
        &general_links,
        &general_fair_baseline,
        &general_fair_totals,
        &cohort_tie_break_ranks,
    );

    let mut fairness_cohort_reports = Vec::new();
    let mut minimum_fair_ratio = 1.0_f64;
    let mut maximum_fair_ratio = 0.0_f64;
    for (index, (id, total, accessible, _)) in cohort_data.iter().enumerate() {
        let accessible_served = accessible_fair_flow.served_by_cohort[index];
        let general_served = general_fair_flow.served_by_cohort[index];
        let total_served = accessible_served + general_served;
        let ratio = total_served as f64 / *total as f64;
        minimum_fair_ratio = minimum_fair_ratio.min(ratio);
        maximum_fair_ratio = maximum_fair_ratio.max(ratio);
        fairness_cohort_reports.push(json!({
            "cohort_id":id,
            "total_places_requested":total,
            "accessible_places_requested":accessible,
            "accessible_places_feasible":accessible_served,
            "general_places_requested":total - accessible,
            "general_places_feasible":general_served,
            "total_places_feasible":total_served,
            "cohort_service_ratio":ratio,
            "fairness_tie_break_rank":cohort_tie_break_ranks[index]
        }));
    }
    let fair_total_feasible = accessible_fair_flow.total + general_fair_flow.total;

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
        let ratio = if *total == 0 {
            1.0
        } else {
            total_served as f64 / *total as f64
        };
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
        "max_min_fairness_comparison":{
            "algorithm":"DISCRETE_PROGRESSIVE_FILLING_AFTER_ACCESSIBLE_DEMAND_PROTECTION",
            "accessible_places_feasible":accessible_fair_flow.total,
            "general_places_feasible":general_fair_flow.total,
            "total_places_feasible":fair_total_feasible,
            "total_places_delta_vs_max_throughput":i128::from(fair_total_feasible) - i128::from(total_feasible),
            "minimum_cohort_service_ratio":minimum_fair_ratio,
            "maximum_cohort_service_ratio":maximum_fair_ratio,
            "cohort_service_ratio_spread":maximum_fair_ratio - minimum_fair_ratio,
            "cohort_service":fairness_cohort_reports,
            "tie_break_policy":"EXPLICIT_OWNER_SUPPLIED_RANK_ONLY_FOR_EQUAL_DISCRETE_SERVICE_RATIOS",
            "assignment":"NONE",
            "reservation":"NONE"
        },
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

fn evaluate_time_series(case: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    for (key, expected) in [
        ("study_id", "HIGHVELD-REFUGE-SUPPLY-DEMAND-DEGRADATION-001"),
        ("maturity", "SYNTHETIC_KNOWN_ANSWER"),
        (
            "execution_status",
            "NO_NATIVE_EVENT_TIME_OR_SITE_VALIDATION",
        ),
    ] {
        if text_at(case, key) != Some(expected) {
            errors.push(format!(
                "{key} must preserve the temporal benchmark claim ceiling"
            ));
        }
    }
    let time_order_source = text_at(case, "time_order_source")
        .ok_or_else(|| vec!["time_order_source is required".to_owned()])?;
    let slices = case
        .get("time_slices")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["time_slices must be an array".to_owned()])?;
    if slices.len() < 2 {
        errors.push("at least two timepoints are required for a degradation sequence".to_owned());
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut seen_timepoints = BTreeSet::new();
    let mut timepoint_reports = Vec::new();
    let mut cohort_minimum_ratio = BTreeMap::<String, f64>::new();
    let mut cohort_service_debt = BTreeMap::<String, CohortServiceDebt>::new();
    let mut cohort_service_continuity = BTreeMap::<String, CohortServiceContinuity>::new();
    let mut cohort_fairness_ranks = BTreeMap::<String, u64>::new();
    let mut expected_cohort_ids: Option<BTreeSet<String>> = None;
    let mut elapsed_h = 0.0_f64;
    let mut total_unmet_place_hours = 0.0_f64;
    let mut accessible_unmet_place_hours = 0.0_f64;
    let mut unmet_place_snapshot_sum = 0_u64;
    let mut maximum_unmet = 0_u64;
    let mut maximum_unmet_timepoint = String::new();
    let mut minimum_ratio = 1.0_f64;

    for (sequence_index, time_slice) in slices.iter().enumerate() {
        let timepoint_id = text_at(time_slice, "timepoint_id").ok_or_else(|| {
            vec![format!(
                "time_slices[{sequence_index}].timepoint_id is missing"
            )]
        })?;
        if !seen_timepoints.insert(timepoint_id.to_owned()) {
            errors.push(format!("duplicate timepoint identity {timepoint_id}"));
            continue;
        }
        let duration = number_at(time_slice, "duration_h").ok_or_else(|| {
            vec![format!(
                "{timepoint_id}: duration_h is missing or non-finite"
            )]
        })?;
        if duration <= 0.0 {
            errors.push(format!("{timepoint_id}: duration_h must be positive"));
            continue;
        }
        let scenario_id = text_at(time_slice, "scenario_id").unwrap_or("UNSPECIFIED_SCENARIO");
        let snapshot_value = time_slice
            .get("snapshot")
            .and_then(Value::as_object)
            .ok_or_else(|| vec![format!("{timepoint_id}: snapshot must be an object")])?;
        let mut snapshot_object = snapshot_value.clone();
        snapshot_object.insert(
            "study_id".to_owned(),
            json!("HIGHVELD-REFUGE-NETWORK-FEASIBILITY-001"),
        );
        snapshot_object.insert(
            "case_id".to_owned(),
            json!(format!(
                "{}__{}",
                text_at(case, "case_id").unwrap_or("TEMPORAL_CASE"),
                timepoint_id
            )),
        );
        snapshot_object.insert("maturity".to_owned(), json!("SYNTHETIC_KNOWN_ANSWER"));
        snapshot_object.insert(
            "execution_status".to_owned(),
            json!("NO_SITE_OR_ROUTE_VALIDATION"),
        );
        let snapshot = Value::Object(snapshot_object);
        let snapshot_cohorts = snapshot
            .get("cohorts")
            .and_then(Value::as_array)
            .ok_or_else(|| vec![format!("{timepoint_id}: cohorts must be an array")])?;
        let mut current_cohort_ids = BTreeSet::new();
        let mut current_tie_break_ranks = BTreeSet::new();
        for cohort in snapshot_cohorts {
            let cohort_id = text_at(cohort, "cohort_id").ok_or_else(|| {
                vec![format!("{timepoint_id}: cohort_id is missing")]
            })?;
            if !current_cohort_ids.insert(cohort_id.to_owned()) {
                errors.push(format!("{timepoint_id}: duplicate cohort identity {cohort_id}"));
            }
            let rank = uint_at(cohort, "fairness_tie_break_rank").ok_or_else(|| {
                vec![format!("{timepoint_id}/{cohort_id}: fairness_tie_break_rank is missing")]
            })?;
            if !current_tie_break_ranks.insert(rank) {
                errors.push(format!(
                    "{timepoint_id}: duplicate fairness_tie_break_rank {rank}"
                ));
            }
            if let Some(prior_rank) = cohort_fairness_ranks.get(cohort_id) {
                if *prior_rank != rank {
                    errors.push(format!(
                        "{timepoint_id}/{cohort_id}: fairness_tie_break_rank changed within sequence"
                    ));
                }
            } else {
                cohort_fairness_ranks.insert(cohort_id.to_owned(), rank);
            }
        }
        match &expected_cohort_ids {
            Some(expected) if expected != &current_cohort_ids => {
                errors.push(format!(
                    "{timepoint_id}: cohort set changed within the service sequence"
                ));
            }
            None => expected_cohort_ids = Some(current_cohort_ids),
            _ => {}
        }
        let sequence_elapsed_h = elapsed_h;
        let report = evaluate_snapshot(&snapshot)?;

        let total_requested = report["total_places_requested"].as_u64().unwrap_or(0);
        let total_feasible = report["total_places_feasible"].as_u64().unwrap_or(0);
        let total_unmet = report["total_places_unmet"].as_u64().unwrap_or(0);
        let accessible_unmet = report["accessible_places_unmet"].as_u64().unwrap_or(0);
        let minimum_cohort_ratio =
            number_at(&report, "minimum_cohort_service_ratio").unwrap_or(0.0);

        let mut timepoint_cohort_debt = Vec::new();
        if let Some(cohort_rows) = report["cohort_service"].as_array() {
            for cohort in cohort_rows {
                let Some(cohort_id) = text_at(cohort, "cohort_id") else {
                    errors.push(format!("{timepoint_id}: cohort service row has no ID"));
                    continue;
                };
                let requested = uint_at(cohort, "total_places_requested").unwrap_or(0);
                let served = uint_at(cohort, "total_places_feasible").unwrap_or(0);
                let accessible_requested =
                    uint_at(cohort, "accessible_places_requested").unwrap_or(0);
                let accessible_served = uint_at(cohort, "accessible_places_feasible").unwrap_or(0);
                let general_requested = uint_at(cohort, "general_places_requested").unwrap_or(0);
                let general_served = uint_at(cohort, "general_places_feasible").unwrap_or(0);
                let unmet = requested.saturating_sub(served);
                let accessible_unmet = accessible_requested.saturating_sub(accessible_served);
                let general_unmet = general_requested.saturating_sub(general_served);
                let snapshot_ratio = number_at(cohort, "cohort_service_ratio").unwrap_or(0.0);

                let continuity = cohort_service_continuity
                    .entry(cohort_id.to_owned())
                    .or_default();
                let baseline_ratio = *continuity
                    .baseline_service_ratio
                    .get_or_insert(snapshot_ratio);
                if baseline_ratio <= 0.0 {
                    errors.push(format!(
                        "{timepoint_id}/{cohort_id}: baseline service ratio must be positive for recovery analysis"
                    ));
                }
                let degraded_vs_baseline = snapshot_ratio + 1e-12 < baseline_ratio;
                if degraded_vs_baseline {
                    if continuity.current_degraded_episode_start_elapsed_h.is_none() {
                        continuity.current_degraded_episode_start_elapsed_h =
                            Some(sequence_elapsed_h);
                        continuity.degraded_episode_count += 1;
                        if continuity.first_degradation_elapsed_h.is_none() {
                            continuity.first_degradation_elapsed_h = Some(sequence_elapsed_h);
                        }
                    }
                    continuity.current_degraded_episode_duration_h += duration;
                    continuity.total_degraded_duration_h += duration;
                    continuity.longest_degraded_episode_duration_h = continuity
                        .longest_degraded_episode_duration_h
                        .max(continuity.current_degraded_episode_duration_h);
                } else if let Some(episode_start) =
                    continuity.current_degraded_episode_start_elapsed_h
                {
                    if continuity.first_baseline_recovery_elapsed_h.is_none() {
                        continuity.first_baseline_recovery_elapsed_h = Some(sequence_elapsed_h);
                    }
                    let _episode_duration = sequence_elapsed_h - episode_start;
                    continuity.recovered_episode_count += 1;
                    continuity.current_degraded_episode_start_elapsed_h = None;
                    continuity.current_degraded_episode_duration_h = 0.0;
                }

                let debt = cohort_service_debt.entry(cohort_id.to_owned()).or_default();
                debt.requested_place_hours += requested as f64 * duration;
                debt.served_place_hours += served as f64 * duration;
                debt.unmet_place_hours += unmet as f64 * duration;
                debt.accessible_requested_place_hours += accessible_requested as f64 * duration;
                debt.accessible_served_place_hours += accessible_served as f64 * duration;
                debt.accessible_unmet_place_hours += accessible_unmet as f64 * duration;
                debt.general_requested_place_hours += general_requested as f64 * duration;
                debt.general_served_place_hours += general_served as f64 * duration;
                debt.general_unmet_place_hours += general_unmet as f64 * duration;
                debt.minimum_snapshot_service_ratio = Some(
                    debt.minimum_snapshot_service_ratio
                        .map_or(snapshot_ratio, |current| current.min(snapshot_ratio)),
                );

                timepoint_cohort_debt.push(json!({
                    "cohort_id":cohort_id,
                    "requested_places":requested,
                    "served_places":served,
                    "unmet_places":unmet,
                    "unmet_place_hours":unmet as f64 * duration,
                    "accessible_unmet_places":accessible_unmet,
                    "accessible_unmet_place_hours":accessible_unmet as f64 * duration,
                    "general_unmet_places":general_unmet,
                    "general_unmet_place_hours":general_unmet as f64 * duration,
                    "snapshot_service_ratio":snapshot_ratio,
                    "baseline_service_ratio":baseline_ratio,
                    "below_baseline_service":degraded_vs_baseline,
                    "degraded_episode_open":continuity.current_degraded_episode_start_elapsed_h.is_some(),
                    "current_degraded_episode_duration_h":continuity.current_degraded_episode_duration_h
                }));
            }
        }

        total_unmet_place_hours += total_unmet as f64 * duration;
        accessible_unmet_place_hours += accessible_unmet as f64 * duration;
        unmet_place_snapshot_sum += total_unmet;
        minimum_ratio = minimum_ratio.min(minimum_cohort_ratio);
        if total_unmet > maximum_unmet {
            maximum_unmet = total_unmet;
            maximum_unmet_timepoint = timepoint_id.to_owned();
        }

        if let Some(cohort_rows) = report["cohort_service"].as_array() {
            for cohort in cohort_rows {
                if let (Some(cohort_id), Some(ratio)) = (
                    text_at(cohort, "cohort_id"),
                    number_at(cohort, "cohort_service_ratio"),
                ) {
                    cohort_minimum_ratio
                        .entry(cohort_id.to_owned())
                        .and_modify(|current| *current = current.min(ratio))
                        .or_insert(ratio);
                }
            }
        }

        timepoint_reports.push(json!({
            "sequence_index": sequence_index,
            "timepoint_id": timepoint_id,
            "scenario_id": scenario_id,
            "elapsed_start_h": sequence_elapsed_h,
            "elapsed_end_h": sequence_elapsed_h + duration,
            "duration_h": duration,
            "total_places_requested": total_requested,
            "total_places_feasible": total_feasible,
            "total_places_unmet": total_unmet,
            "accessible_places_unmet": accessible_unmet,
            "minimum_cohort_service_ratio": minimum_cohort_ratio,
            "qualified_site_count": report["qualified_site_count"],
            "qualified_general_route_count": report["qualified_general_route_count"],
            "qualified_accessible_route_count": report["qualified_accessible_route_count"],
            "cohort_service_debt_delta": timepoint_cohort_debt
        }));
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    let cohort_minimum_rows: Vec<Value> = cohort_minimum_ratio
        .iter()
        .map(|(cohort_id, ratio)| {
            json!({
                "cohort_id": cohort_id,
                "minimum_service_ratio_across_sequence": ratio
            })
        })
        .collect();
    let cohort_service_continuity_rows: Vec<Value> = cohort_service_continuity
        .iter()
        .map(|(cohort_id, continuity)| {
            let time_to_first_baseline_recovery_h = match (
                continuity.first_degradation_elapsed_h,
                continuity.first_baseline_recovery_elapsed_h,
            ) {
                (Some(first_degradation), Some(first_recovery)) => {
                    Some(first_recovery - first_degradation)
                }
                _ => None,
            };
            json!({
                "cohort_id": cohort_id,
                "baseline_service_ratio": continuity.baseline_service_ratio,
                "first_degradation_elapsed_h": continuity.first_degradation_elapsed_h,
                "first_baseline_recovery_elapsed_h": continuity.first_baseline_recovery_elapsed_h,
                "time_to_first_baseline_recovery_h": time_to_first_baseline_recovery_h,
                "degraded_episode_count": continuity.degraded_episode_count,
                "recovered_episode_count": continuity.recovered_episode_count,
                "total_degraded_duration_h": continuity.total_degraded_duration_h,
                "longest_degraded_episode_duration_h": continuity.longest_degraded_episode_duration_h,
                "unrecovered_at_sequence_end": continuity.current_degraded_episode_start_elapsed_h.is_some(),
                "current_unrecovered_episode_duration_h": continuity.current_degraded_episode_duration_h
            })
        })
        .collect();
    let cohort_service_debt_rows: Vec<Value> = cohort_service_debt
        .iter()
        .map(|(cohort_id, debt)| {
            let cumulative_service_ratio = if debt.requested_place_hours > 0.0 {
                debt.served_place_hours / debt.requested_place_hours
            } else {
                1.0
            };
            let cumulative_unmet_fraction = if debt.requested_place_hours > 0.0 {
                debt.unmet_place_hours / debt.requested_place_hours
            } else {
                0.0
            };
            json!({
                "cohort_id":cohort_id,
                "requested_place_hours":debt.requested_place_hours,
                "served_place_hours":debt.served_place_hours,
                "unmet_place_hours":debt.unmet_place_hours,
                "accessible_requested_place_hours":debt.accessible_requested_place_hours,
                "accessible_served_place_hours":debt.accessible_served_place_hours,
                "accessible_unmet_place_hours":debt.accessible_unmet_place_hours,
                "general_requested_place_hours":debt.general_requested_place_hours,
                "general_served_place_hours":debt.general_served_place_hours,
                "general_unmet_place_hours":debt.general_unmet_place_hours,
                "cumulative_service_ratio":cumulative_service_ratio,
                "cumulative_unmet_fraction":cumulative_unmet_fraction,
                "minimum_snapshot_service_ratio":debt.minimum_snapshot_service_ratio
            })
        })
        .collect();
    let worst_service_debt = cohort_service_debt
        .iter()
        .max_by(|left, right| {
            left.1
                .unmet_place_hours
                .total_cmp(&right.1.unmet_place_hours)
        })
        .map(|(cohort_id, debt)| {
            json!({
                "cohort_id":cohort_id,
                "unmet_place_hours":debt.unmet_place_hours
            })
        })
        .unwrap_or(Value::Null);
    let summed_cohort_debt: f64 = cohort_service_debt
        .values()
        .map(|debt| debt.unmet_place_hours)
        .sum();
    let summed_accessible_cohort_debt: f64 = cohort_service_debt
        .values()
        .map(|debt| debt.accessible_unmet_place_hours)
        .sum();

    Ok(json!({
        "audit_state": "REFUGE_SUPPLY_DEMAND_DEGRADATION_COMPLETE",
        "study_id": "HIGHVELD-REFUGE-SUPPLY-DEMAND-DEGRADATION-001",
        "case_id": text_at(case, "case_id").unwrap_or_default(),
        "timepoint_count": timepoint_reports.len(),
        "time_order_source": time_order_source,
        "native_event_time_owner": "SYMTROPY",
        "native_timestamps_or_event_ids_generated": false,
        "total_unmet_place_hours": total_unmet_place_hours,
        "accessible_unmet_place_hours": accessible_unmet_place_hours,
        "unmet_place_snapshot_sum_not_unique_people": unmet_place_snapshot_sum,
        "maximum_unmet_places_at_any_timepoint": maximum_unmet,
        "maximum_unmet_timepoint_id": maximum_unmet_timepoint,
        "minimum_cohort_service_ratio_any_timepoint": minimum_ratio,
        "cohort_minimum_service_ratios": cohort_minimum_rows,
        "cohort_cumulative_service_debt": cohort_service_debt_rows,
        "cohort_service_continuity": cohort_service_continuity_rows,
        "total_sequence_duration_h": elapsed_h,
        "worst_cumulative_service_debt_cohort": worst_service_debt,
        "cohort_debt_matches_total_unmet_place_hours": (summed_cohort_debt - total_unmet_place_hours).abs() <= 1e-9,
        "accessible_cohort_debt_matches_total": (summed_accessible_cohort_debt - accessible_unmet_place_hours).abs() <= 1e-9,
        "timepoints": timepoint_reports,
        "assignment": "NONE",
        "reservation": "NONE",
        "execution_status": "TEMPORAL_AGGREGATE_FEASIBILITY_ONLY_NO_ASSIGNMENT",
        "claim_ceiling": "Synthetic time-sequence replay only; not live event chronology, real refuge service or occupant allocation"
    }))
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    if case.get("time_slices").is_some() {
        evaluate_time_series(case)
    } else {
        evaluate_snapshot(case)
    }
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value = serde_json::from_str(CASE).expect("committed network case JSON");
            let report = evaluate(&case).expect("network case structure");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["audit_state"] != "REFUGE_NETWORK_FEASIBILITY_COMPLETE" {
                process::exit(1);
            }
        }
        "--time-series-self-test" => {
            let case: Value =
                serde_json::from_str(TIME_SERIES_CASE).expect("committed time-series case JSON");
            let report = evaluate_time_series(&case).expect("temporal network case structure");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["audit_state"] != "REFUGE_SUPPLY_DEMAND_DEGRADATION_COMPLETE" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-refuge-network-feasibility");
            println!("  --self-test  evaluate aggregate refuge capacity feasibility");
            println!("  --time-series-self-test  evaluate ordered aggregate service degradation");
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
    fn temporal_service_sequence_reports_unmet_place_hours() {
        let case: Value = serde_json::from_str(TIME_SERIES_CASE).expect("temporal case JSON");
        let report = evaluate_time_series(&case).expect("temporal case must evaluate");
        assert_eq!(report["timepoint_count"], 3);
        assert_eq!(report["total_unmet_place_hours"], 56.0);
        assert_eq!(report["accessible_unmet_place_hours"], 12.0);
        assert_eq!(report["unmet_place_snapshot_sum_not_unique_people"], 36);
        assert_eq!(report["maximum_unmet_places_at_any_timepoint"], 20);
        assert_eq!(report["maximum_unmet_timepoint_id"], "TP-002");
        assert_eq!(report["minimum_cohort_service_ratio_any_timepoint"], 0.0);
        assert_eq!(report["native_timestamps_or_event_ids_generated"], false);
        assert_eq!(report["assignment"], "NONE");
    }

    #[test]
    fn temporal_service_debt_conserves_place_hours_by_cohort() {
        let case: Value = serde_json::from_str(TIME_SERIES_CASE).expect("temporal case JSON");
        let report = evaluate_time_series(&case).expect("temporal case must evaluate");
        assert_eq!(report["cohort_debt_matches_total_unmet_place_hours"], true);
        assert_eq!(report["accessible_cohort_debt_matches_total"], true);
        let rows = report["cohort_cumulative_service_debt"]
            .as_array()
            .expect("cohort debt rows");
        assert_eq!(rows.len(), 3);
        let unmet_sum: f64 = rows
            .iter()
            .map(|row| row["unmet_place_hours"].as_f64().unwrap())
            .sum();
        assert!((unmet_sum - 56.0).abs() <= 1e-9);
        let worst = &report["worst_cumulative_service_debt_cohort"];
        assert!(worst["cohort_id"].as_str().is_some());
    }

    #[test]
    fn temporal_sequence_reports_baseline_recovery_time_and_longest_gap() {
        let case: Value = serde_json::from_str(TIME_SERIES_CASE).expect("temporal case JSON");
        let report = evaluate_time_series(&case).expect("temporal case must evaluate");
        assert_eq!(report["total_sequence_duration_h"], 4.0);
        let continuity = report["cohort_service_continuity"]
            .as_array()
            .expect("cohort continuity rows");
        assert_eq!(continuity.len(), 3);
        assert!(continuity.iter().any(|row| {
            row["degraded_episode_count"].as_u64().unwrap_or(0) == 1
                && row["recovered_episode_count"].as_u64().unwrap_or(0) == 1
                && (row["longest_degraded_episode_duration_h"].as_f64().unwrap_or(0.0) - 2.0).abs() < 1e-9
                && (row["time_to_first_baseline_recovery_h"].as_f64().unwrap_or(-1.0) - 2.0).abs() < 1e-9
                && row["unrecovered_at_sequence_end"] == false
        }));
    }

    #[test]
    fn temporal_sequence_rejects_changed_tie_break_rank() {
        let mut case: Value = serde_json::from_str(TIME_SERIES_CASE).expect("temporal case JSON");
        case["time_slices"][1]["snapshot"]["cohorts"][0]["fairness_tie_break_rank"] = json!(4);
        assert!(evaluate_time_series(&case).is_err());
    }

    #[test]
    fn temporal_sequence_rejects_changed_cohort_set() {
        let mut case: Value = serde_json::from_str(TIME_SERIES_CASE).expect("temporal case JSON");
        case["time_slices"][1]["snapshot"]["cohorts"].as_array_mut().unwrap().pop();
        assert!(evaluate_time_series(&case).is_err());
    }

    #[test]
    fn temporal_sequence_rejects_duplicate_timepoint_ids() {
        let mut case: Value = serde_json::from_str(TIME_SERIES_CASE).expect("temporal case JSON");
        let duplicate = case["time_slices"][0].clone();
        case["time_slices"].as_array_mut().unwrap().push(duplicate);
        assert!(evaluate_time_series(&case).is_err());
    }

    #[test]
    fn temporal_sequence_rejects_nonpositive_duration() {
        let mut case: Value = serde_json::from_str(TIME_SERIES_CASE).expect("temporal case JSON");
        case["time_slices"][1]["duration_h"] = json!(0.0);
        assert!(evaluate_time_series(&case).is_err());
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
    fn max_min_fairness_improves_worst_cohort_ratio_without_losing_throughput() {
        let report = evaluate(&sample()).expect("case must evaluate");
        let fairness = &report["max_min_fairness_comparison"];
        let baseline_minimum = report["minimum_cohort_service_ratio"]
            .as_f64()
            .expect("baseline minimum ratio");
        let fair_minimum = fairness["minimum_cohort_service_ratio"]
            .as_f64()
            .expect("fair minimum ratio");

        assert!(fair_minimum > baseline_minimum);
        assert_eq!(
            fairness["total_places_feasible"],
            report["total_places_feasible"]
        );
        assert_eq!(
            fairness["tie_break_policy"],
            "EXPLICIT_OWNER_SUPPLIED_RANK_ONLY_FOR_EQUAL_DISCRETE_SERVICE_RATIOS"
        );
        assert_eq!(fairness["assignment"], "NONE");
        assert_eq!(fairness["reservation"], "NONE");
    }

    #[test]
    fn missing_or_duplicate_fairness_rank_is_rejected() {
        let mut case = sample();
        case["cohorts"][1]["fairness_tie_break_rank"] = json!(1);
        assert!(evaluate(&case).is_err());

        let mut missing = sample();
        missing["cohorts"][0]
            .as_object_mut()
            .unwrap()
            .remove("fairness_tie_break_rank");
        assert!(evaluate(&missing).is_err());
    }

    #[test]
    fn oracle_never_assigns_or_reserves_people() {
        let report = evaluate(&sample()).unwrap();
        assert_eq!(report["assignment"], "NONE");
        assert_eq!(report["reservation"], "NONE");
        assert_eq!(
            report["execution_status"],
            "AGGREGATE_FEASIBILITY_ONLY_NO_ASSIGNMENT"
        );
    }

    #[test]
    fn duplicate_route_pair_is_rejected() {
        let mut case = sample();
        let route = case["routes"][0].clone();
        case["routes"].as_array_mut().unwrap().push(route);
        assert!(evaluate(&case).is_err());
    }
}
