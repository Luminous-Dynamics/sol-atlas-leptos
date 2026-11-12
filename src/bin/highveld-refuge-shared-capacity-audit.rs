use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_SHARED_CAPACITY_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_SHARED_CAPACITY_CASE_V0.1.schema.json"
);

#[derive(Clone, Default)]
struct Usage {
    total: u64,
    accessible: u64,
    water_l: f64,
}

#[derive(Clone)]
struct Capacity {
    eligible: bool,
    total: u64,
    accessible: u64,
    water_l: f64,
    evidence_ok: bool,
    used: Usage,
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
}

fn uint_at(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64).filter(|n| n.is_finite())
}

fn ratio(served: u64, requested: u64) -> Option<f64> {
    (requested > 0).then_some(served as f64 / requested as f64)
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|e| vec![format!("schema JSON parse failed: {e}")])?;
    let mut violations = Vec::new();

    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        violations.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(case, "study_id") != Some("HIGHVELD-REFUGE-ALLOCATION-AUDIT-001")
        || text_at(case, "maturity") != Some("SYNTHETIC_KNOWN_ANSWER")
        || text_at(case, "execution_status") != Some("NO_ALLOCATION_OR_BOOKING")
    {
        violations.push("study identity or execution claim ceiling is invalid".to_owned());
    }

    let site_rows = case.get("sites").and_then(Value::as_array)
        .ok_or_else(|| vec!["sites must be an array".to_owned()])?;
    let mut sites = BTreeMap::<String, Capacity>::new();
    let mut site_ids = BTreeSet::new();

    for site in site_rows {
        let id = text_at(site, "site_id").unwrap_or("<missing-site-id>").to_owned();
        if !site_ids.insert(id.clone()) {
            violations.push(format!("{id}: duplicate site identity"));
            continue;
        }
        let total = uint_at(site, "free_total_places");
        let accessible = uint_at(site, "free_accessible_places");
        let water = number_at(site, "free_safe_water_l");
        if total.is_none() || accessible.is_none() || water.is_none() {
            violations.push(format!("{id}: missing or invalid finite site capacity"));
            continue;
        }
        let total = total.unwrap();
        let accessible = accessible.unwrap();
        let water = water.unwrap();
        if accessible > total || water < 0.0 {
            violations.push(format!("{id}: invalid site capacity dimensions"));
        }
        let evidence_ok = [
            "site_evidence_ref",
            "capacity_evidence_ref",
            "water_evidence_ref",
        ]
        .iter()
        .all(|key| text_at(site, key).is_some());
        if !evidence_ok {
            violations.push(format!("{id}: required site/capacity/water evidence reference missing"));
        }
        let eligible = text_at(site, "availability_state")
            == Some("ELIGIBLE_FOR_HUMAN_CAPACITY_PLANNING");
        if !eligible {
            violations.push(format!("{id}: site is unavailable or unverified"));
        }
        sites.insert(id, Capacity {
            eligible,
            total,
            accessible,
            water_l,
            evidence_ok,
            used: Usage::default(),
        });
    }

    let groups = case.get("demand_areas").and_then(Value::as_array)
        .ok_or_else(|| vec!["demand_areas must be an array".to_owned()])?;
    let mut group_ids = BTreeSet::new();
    let mut group_reports = Vec::new();
    let mut minimum_total_ratio = 1.0_f64;
    let mut minimum_accessible_ratio = 1.0_f64;
    let mut minimum_water_ratio = 1.0_f64;
    let mut max_total_gap = 0_u64;
    let mut max_accessible_gap = 0_u64;
    let mut max_water_gap = 0.0_f64;
    let mut requested_total = 0_u64;
    let mut requested_accessible = 0_u64;
    let mut requested_water = 0.0_f64;
    let mut served_total = 0_u64;
    let mut served_accessible = 0_u64;
    let mut served_water = 0.0_f64;

    for group in groups {
        let group_id = text_at(group, "group_id").unwrap_or("<missing-group-id>").to_owned();
        if !group_ids.insert(group_id.clone()) {
            violations.push(format!("{group_id}: duplicate demand-group identity"));
        }
        let total = uint_at(group, "requested_total_places");
        let accessible = uint_at(group, "requested_accessible_places");
        let water = number_at(group, "requested_safe_water_l");
        if total.is_none() || accessible.is_none() || water.is_none() {
            violations.push(format!("{group_id}: missing or invalid demand dimensions"));
            continue;
        }
        let total = total.unwrap();
        let accessible = accessible.unwrap();
        let water = water.unwrap();
        if total == 0 || accessible > total || water < 0.0 {
            violations.push(format!("{group_id}: invalid demand envelope"));
        }
        requested_total = requested_total.saturating_add(total);
        requested_accessible = requested_accessible.saturating_add(accessible);
        requested_water += water;

        let allocations = group.get("allocations").and_then(Value::as_array)
            .ok_or_else(|| vec![format!("{group_id}: allocations must be an array")])?;
        let mut per_group_sites = BTreeSet::new();
        let mut group_total = 0_u64;
        let mut group_accessible = 0_u64;
        let mut group_water = 0.0_f64;

        for allocation in allocations {
            let site_id = text_at(allocation, "site_id").unwrap_or("<missing-site-id>").to_owned();
            if !per_group_sites.insert(site_id.clone()) {
                violations.push(format!("{group_id}/{site_id}: duplicate group/site allocation row"));
                continue;
            }
            let alloc_total = uint_at(allocation, "total_places");
            let alloc_accessible = uint_at(allocation, "accessible_places");
            let alloc_water = number_at(allocation, "safe_water_l");
            if alloc_total.is_none() || alloc_accessible.is_none() || alloc_water.is_none() {
                violations.push(format!("{group_id}/{site_id}: invalid allocation dimensions"));
                continue;
            }
            let alloc_total = alloc_total.unwrap();
            let alloc_accessible = alloc_accessible.unwrap();
            let alloc_water = alloc_water.unwrap();
            if alloc_accessible > alloc_total || alloc_water < 0.0 {
                violations.push(format!("{group_id}/{site_id}: accessible places exceed total or water is negative"));
            }
            if text_at(allocation, "route_state") != Some("REACHABLE")
                || text_at(allocation, "route_accessibility_state") != Some("QUALIFIED")
                || text_at(allocation, "route_evidence_ref").is_none()
            {
                violations.push(format!("{group_id}/{site_id}: route/accessibility evidence is not qualified"));
            }

            match sites.get_mut(&site_id) {
                Some(site) => {
                    if !site.eligible || !site.evidence_ok {
                        violations.push(format!("{group_id}/{site_id}: allocation references unavailable/unqualified site"));
                    }
                    site.used.total = site.used.total.saturating_add(alloc_total);
                    site.used.accessible = site.used.accessible.saturating_add(alloc_accessible);
                    site.used.water_l += alloc_water;
                }
                None => violations.push(format!("{group_id}/{site_id}: unknown site ID")),
            }
            group_total = group_total.saturating_add(alloc_total);
            group_accessible = group_accessible.saturating_add(alloc_accessible);
            group_water += alloc_water;
        }

        if group_total > total {
            violations.push(format!("{group_id}: allocated total places exceed group demand"));
        }
        if group_accessible > accessible {
            violations.push(format!("{group_id}: allocated accessible places exceed group demand"));
        }
        if group_water > water + 1e-9 {
            violations.push(format!("{group_id}: allocated safe water exceeds group demand"));
        }

        let total_gap = total.saturating_sub(group_total.min(total));
        let accessible_gap = accessible.saturating_sub(group_accessible.min(accessible));
        let water_gap = (water - group_water).max(0.0);
        let total_ratio = ratio(group_total.min(total), total).unwrap_or(0.0);
        let accessible_ratio = ratio(group_accessible.min(accessible), accessible);
        let water_ratio = if water > 0.0 {
            Some((group_water.min(water) / water).clamp(0.0, 1.0))
        } else {
            None
        };
        minimum_total_ratio = minimum_total_ratio.min(total_ratio);
        if let Some(value) = accessible_ratio {
            minimum_accessible_ratio = minimum_accessible_ratio.min(value);
        }
        if let Some(value) = water_ratio {
            minimum_water_ratio = minimum_water_ratio.min(value);
        }
        max_total_gap = max_total_gap.max(total_gap);
        max_accessible_gap = max_accessible_gap.max(accessible_gap);
        max_water_gap = max_water_gap.max(water_gap);
        served_total = served_total.saturating_add(group_total.min(total));
        served_accessible = served_accessible.saturating_add(group_accessible.min(accessible));
        served_water += group_water.min(water);

        group_reports.push(json!({
            "group_id": group_id,
            "requested_total_places": total,
            "allocated_total_places": group_total,
            "total_place_gap": total_gap,
            "total_fulfilment_ratio": total_ratio,
            "requested_accessible_places": accessible,
            "allocated_accessible_places": group_accessible,
            "accessible_place_gap": accessible_gap,
            "accessible_fulfilment_ratio": accessible_ratio,
            "requested_safe_water_l": water,
            "allocated_safe_water_l": group_water,
            "safe_water_gap_l": water_gap,
            "safe_water_fulfilment_ratio": water_ratio
        }));
    }

    let mut site_reports = Vec::new();
    for (site_id, site) in &sites {
        if site.used.total > site.total {
            violations.push(format!("{site_id}: shared total-seat capacity double-spent"));
        }
        if site.used.accessible > site.accessible {
            violations.push(format!("{site_id}: shared accessible capacity double-spent"));
        }
        if site.used.water_l > site.water_l + 1e-9 {
            violations.push(format!("{site_id}: shared safe-water capacity/reserve double-spent"));
        }
        site_reports.push(json!({
            "site_id": site_id,
            "free_total_places": site.total,
            "allocated_total_places": site.used.total,
            "free_accessible_places": site.accessible,
            "allocated_accessible_places": site.used.accessible,
            "free_safe_water_l": site.water_l,
            "allocated_safe_water_l": site.used.water_l
        }));
    }

    let state = if violations.is_empty() {
        "PLAN_FEASIBLE_WITH_OR_WITHOUT_UNSERVED_DEMAND"
    } else {
        "PLAN_INVALID"
    };
    Ok(json!({
        "audit_state": state,
        "study_id": "HIGHVELD-REFUGE-ALLOCATION-AUDIT-001",
        "case_id": text_at(case, "case_id").unwrap_or_default(),
        "requested": {
            "total_places": requested_total,
            "accessible_places": requested_accessible,
            "safe_water_l": requested_water
        },
        "served": {
            "total_places": served_total,
            "accessible_places": served_accessible,
            "safe_water_l": served_water
        },
        "service_gaps": {
            "total_places": requested_total.saturating_sub(served_total),
            "accessible_places": requested_accessible.saturating_sub(served_accessible),
            "safe_water_l": (requested_water - served_water).max(0.0)
        },
        "minimum_group_fulfilment": {
            "total_places": minimum_total_ratio,
            "accessible_places": minimum_accessible_ratio,
            "safe_water": minimum_water_ratio
        },
        "maximum_group_gap": {
            "total_places": max_total_gap,
            "accessible_places": max_accessible_gap,
            "safe_water_l": max_water_gap
        },
        "groups": group_reports,
        "sites": site_reports,
        "violations": violations,
        "allocation_solver": "NOT_PRESENT",
        "reservation_or_booking": "NONE",
        "execution_status": "AGGREGATE_PLAN_AUDIT_ONLY"
    }))
}

fn main() {
    match std::env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value = serde_json::from_str(CASE).expect("committed allocation case JSON");
            let report = evaluate(&case).expect("allocation case structure");
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            if report["audit_state"] != "PLAN_FEASIBLE_WITH_OR_WITHOUT_UNSERVED_DEMAND" {
                std::process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-refuge-shared-capacity-audit");
            println!("  --self-test  audit the committed synthetic multi-area capacity plan");
        }
        _ => {
            eprintln!("invalid argument; use --help");
            std::process::exit(2);
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
    fn multi_group_positive_control_preserves_partial_service_gap() {
        let report = evaluate(&sample()).expect("case evaluates");
        assert_eq!(report["audit_state"], "PLAN_FEASIBLE_WITH_OR_WITHOUT_UNSERVED_DEMAND");
        assert_eq!(report["service_gaps"]["total_places"], 1);
        assert_eq!(report["service_gaps"]["accessible_places"], 0);
        assert_eq!(report["minimum_group_fulfilment"]["total_places"], 0.8);
    }

    #[test]
    fn shared_total_capacity_double_spending_is_rejected() {
        let mut case = sample();
        case["demand_areas"][2]["allocations"][0]["total_places"] = json!(3);
        let report = evaluate(&case).unwrap();
        assert_eq!(report["audit_state"], "PLAN_INVALID");
        assert!(report["violations"].as_array().unwrap().iter().any(|v| {
            v.as_str().unwrap_or("").contains("shared total-seat capacity double-spent")
        }));
    }

    #[test]
    fn shared_accessible_capacity_double_spending_is_rejected() {
        let mut case = sample();
        case["demand_areas"][2]["allocations"][0]["accessible_places"] = json!(2);
        let report = evaluate(&case).unwrap();
        assert!(report["violations"].as_array().unwrap().iter().any(|v| {
            v.as_str().unwrap_or("").contains("shared accessible capacity double-spent")
        }));
    }

    #[test]
    fn shared_water_double_spending_is_rejected() {
        let mut case = sample();
        case["sites"][0]["free_safe_water_l"] = json!(37.0);
        let report = evaluate(&case).unwrap();
        assert!(report["violations"].as_array().unwrap().iter().any(|v| {
            v.as_str().unwrap_or("").contains("shared safe-water capacity/reserve double-spent")
        }));
    }

    #[test]
    fn blocked_route_cannot_receive_group_allocation() {
        let mut case = sample();
        case["demand_areas"][0]["allocations"][0]["route_state"] = json!("BLOCKED");
        let report = evaluate(&case).unwrap();
        assert!(report["violations"].as_array().unwrap().iter().any(|v| {
            v.as_str().unwrap_or("").contains("route/accessibility evidence is not qualified")
        }));
    }

    #[test]
    fn duplicate_group_site_rows_are_rejected_not_silently_aggregated() {
        let mut case = sample();
        let dup = case["demand_areas"][0]["allocations"][0].clone();
        case["demand_areas"][0]["allocations"].as_array_mut().unwrap().push(dup);
        let report = evaluate(&case).unwrap();
        assert!(report["violations"].as_array().unwrap().iter().any(|v| {
            v.as_str().unwrap_or("").contains("duplicate group/site allocation row")
        }));
    }

    #[test]
    fn per_group_over_allocation_is_rejected() {
        let mut case = sample();
        case["demand_areas"][0]["allocations"][0]["total_places"] = json!(7);
        let report = evaluate(&case).unwrap();
        assert!(report["violations"].as_array().unwrap().iter().any(|v| {
            v.as_str().unwrap_or("").contains("allocated total places exceed group demand")
        }));
    }

    #[test]
    fn unknown_site_is_rejected() {
        let mut case = sample();
        case["demand_areas"][0]["allocations"][0]["site_id"] = json!("UNKNOWN_SITE");
        let report = evaluate(&case).unwrap();
        assert!(report["violations"].as_array().unwrap().iter().any(|v| {
            v.as_str().unwrap_or("").contains("unknown site ID")
        }));
    }
}
