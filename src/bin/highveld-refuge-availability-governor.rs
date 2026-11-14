use serde_json::{Value, json};
use std::{collections::BTreeMap, env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_AVAILABILITY_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/REFUGE_AVAILABILITY_CASE_V0.1.schema.json"
);

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

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("schema JSON parse failed: {error}")])?;
    let mut input_errors = Vec::new();
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        input_errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(case, "study_id") != Some("HIGHVELD-REFUGE-AVAILABILITY-001")
        || text_at(case, "maturity") != Some("SYNTHETIC_KNOWN_ANSWER")
        || text_at(case, "execution_status") != Some("NO_SITE_OR_ROUTE_VALIDATION")
    {
        input_errors.push("study identity or synthetic claim ceiling is invalid".to_owned());
    }
    let demand = &case["demand"];
    let total_demand = demand
        .get("total_places")
        .and_then(Value::as_u64)
        .ok_or_else(|| vec!["demand.total_places must be an unsigned integer".to_owned()])?;
    let accessible_demand = demand
        .get("accessible_places")
        .and_then(Value::as_u64)
        .ok_or_else(|| vec!["demand.accessible_places must be an unsigned integer".to_owned()])?;
    let water_demand = number_at(demand, "safe_water_l")
        .ok_or_else(|| vec!["demand.safe_water_l must be finite".to_owned()])?;
    let duration = number_at(demand, "required_duration_h")
        .ok_or_else(|| vec!["demand.required_duration_h must be finite".to_owned()])?;
    let travel_limit = number_at(demand, "maximum_route_travel_time_s")
        .ok_or_else(|| vec!["demand.maximum_route_travel_time_s must be finite".to_owned()])?;
    if accessible_demand > total_demand
        || total_demand == 0
        || water_demand < 0.0
        || duration <= 0.0
        || travel_limit <= 0.0
    {
        input_errors.push("group demand envelope is invalid".to_owned());
    }
    if !input_errors.is_empty() {
        return Err(input_errors);
    }

    let sites = case
        .get("sites")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["sites must be an array".to_owned()])?;
    let mut counts = BTreeMap::<String, usize>::new();
    for site in sites {
        if let Some(id) = text_at(site, "site_id") {
            *counts.entry(id.to_owned()).or_default() += 1;
        }
    }

    let mut eligible = Vec::new();
    let mut unavailable = Vec::new();
    for site in sites {
        let id = text_at(site, "site_id").unwrap_or("<missing-site-id>");
        let mut reasons = Vec::new();
        if counts.get(id).copied().unwrap_or(0) > 1 {
            reasons.push("DUPLICATE_SITE_ID".to_owned());
        }
        if text_at(case, "local_refuge_directory_state") != Some("QUALIFIED") {
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
            ("route_state", "REACHABLE", "ROUTE_NOT_REACHABLE"),
            (
                "route_accessibility_state",
                "QUALIFIED",
                "ROUTE_ACCESSIBILITY_NOT_QUALIFIED",
            ),
        ] {
            if text_at(site, key) != Some(expected) {
                reasons.push(reason.to_owned());
            }
        }

        match number_at(site, "route_travel_time_s") {
            Some(seconds) if seconds <= travel_limit => {}
            Some(_) => reasons.push("ROUTE_TRAVEL_TIME_EXCEEDS_DECLARED_LIMIT".to_owned()),
            None => reasons.push("ROUTE_TRAVEL_TIME_UNKNOWN".to_owned()),
        }

        for key in [
            "route_evidence_ref",
            "accessibility_evidence_ref",
            "capacity_evidence_ref",
            "service_evidence_ref",
            "water_evidence_ref",
        ] {
            if text_at(site, key).is_none() {
                reasons.push(format!("MISSING_EVIDENCE_REFERENCE_{key}"));
            }
        }

        let total_capacity = site.get("total_capacity").and_then(Value::as_u64);
        let occupancy = site.get("current_occupancy").and_then(Value::as_u64);
        let reserve = site.get("committed_reserve").and_then(Value::as_u64);
        let accessible_capacity = site.get("accessible_capacity").and_then(Value::as_u64);
        let accessible_occupancy = site.get("accessible_occupancy").and_then(Value::as_u64);
        let accessible_reserve = site
            .get("accessible_committed_reserve")
            .and_then(Value::as_u64);
        let (free_total, free_accessible) = match (
            total_capacity,
            occupancy,
            reserve,
            accessible_capacity,
            accessible_occupancy,
            accessible_reserve,
        ) {
            (
                Some(cap),
                Some(occupied),
                Some(committed),
                Some(access_cap),
                Some(access_occupied),
                Some(access_committed),
            ) => {
                let valid_total = occupied <= cap
                    && committed <= cap.saturating_sub(occupied);
                let valid_accessible = access_cap <= cap
                    && access_occupied <= access_cap
                    && access_committed <= access_cap.saturating_sub(access_occupied)
                    && access_occupied <= occupied
                    && access_committed <= committed;
                let valid_non_accessible = if valid_total && valid_accessible {
                    let non_accessible_capacity = cap - access_cap;
                    let non_accessible_occupied = occupied - access_occupied;
                    let non_accessible_committed = committed - access_committed;
                    non_accessible_occupied <= non_accessible_capacity
                        && non_accessible_committed
                            <= non_accessible_capacity.saturating_sub(non_accessible_occupied)
                } else {
                    false
                };
                let invalid = !valid_total || !valid_accessible || !valid_non_accessible;
                if invalid {
                    reasons.push("CAPACITY_ACCOUNTING_INVALID".to_owned());
                    (0_u64, 0_u64)
                } else {
                    (
                        cap - occupied - committed,
                        access_cap - access_occupied - access_committed,
                    )
                }
            }
            _ => {
                reasons.push("CAPACITY_EVIDENCE_UNKNOWN".to_owned());
                (0_u64, 0_u64)
            }
        };
        if free_total < total_demand {
            reasons.push("INSUFFICIENT_FREE_TOTAL_CAPACITY".to_owned());
        }
        if free_accessible < accessible_demand {
            reasons.push("INSUFFICIENT_FREE_ACCESSIBLE_CAPACITY".to_owned());
        }

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
                available - committed - reserve
            }
            (Some(available), Some(committed), Some(reserve))
                if available >= 0.0 && committed >= 0.0 && reserve >= 0.0 =>
            {
                reasons.push("SAFE_WATER_BELOW_COMMITMENTS_OR_PROTECTED_RESERVE".to_owned());
                (available - committed - reserve).max(0.0)
            }
            _ => {
                reasons.push("SAFE_WATER_ACCOUNT_UNKNOWN_OR_INVALID".to_owned());
                0.0
            }
        };
        if free_water < water_demand {
            reasons.push("INSUFFICIENT_FREE_SAFE_WATER".to_owned());
        }

        if reasons.is_empty() {
            eligible.push(json!({
                "site_id":id,
                "availability":"ELIGIBLE_FOR_HUMAN_CAPACITY_PLANNING",
                "free_total_capacity":free_total,
                "free_accessible_capacity":free_accessible,
                "free_safe_water_l":free_water,
                "route_travel_time_s":number_at(site,"route_travel_time_s").unwrap_or(0.0),
                "allocation":"NONE",
                "actuation":"NONE"
            }));
        } else {
            unavailable.push(json!({
                "site_id":id,
                "availability":"UNAVAILABLE_OR_UNVERIFIED",
                "reasons":reasons
            }));
        }
    }

    let eligible_site_count = eligible.len();
    Ok(json!({
        "audit_state":"REFUGE_AVAILABILITY_FILTER_COMPLETE",
        "case_id":text_at(case,"case_id").unwrap_or_default(),
        "network_state":text_at(case,"network_state").unwrap_or("UNKNOWN"),
        "local_refuge_directory_state":text_at(case,"local_refuge_directory_state").unwrap_or("UNKNOWN"),
        "eligible_sites":eligible,
        "unavailable_sites":unavailable,
        "total_sites":sites.len(),
        "eligible_site_count":eligible_site_count,
        "human_authority":"REQUIRED",
        "occupant_assignment":"NONE",
        "route_computation":"NOT_PERFORMED; route evidence is accepted from Sol-Atlas",
        "execution_status":"NO_REFUGE_ACTUATION_OR_ASSIGNMENT",
        "claim_ceiling":"Synthetic eligibility filter only; no actual refuge availability or safety certification"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value = serde_json::from_str(CASE).expect("committed refuge case JSON");
            let report = evaluate(&case).expect("refuge case structure");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["audit_state"] != "REFUGE_AVAILABILITY_FILTER_COMPLETE" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-refuge-availability-governor");
            println!("  --self-test  filter the committed synthetic refuge availability case");
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
    fn local_pod_remains_available_during_partition_with_qualified_local_evidence() {
        let report = evaluate(&sample()).expect("case evaluates");
        assert_eq!(report["network_state"], "PARTITIONED");
        assert_eq!(report["eligible_site_count"], 1);
        assert_eq!(
            report["eligible_sites"][0]["site_id"],
            "SYNTHETIC_LOCAL_POD_A"
        );
        assert_eq!(report["eligible_sites"][0]["allocation"], "NONE");
    }

    #[test]
    fn insufficient_total_capacity_blocks_hub() {
        let report = evaluate(&sample()).unwrap();
        let hub = report["unavailable_sites"]
            .as_array()
            .unwrap()
            .iter()
            .find(|site| site["site_id"] == "SYNTHETIC_CENTRAL_HUB_B")
            .unwrap();
        assert!(
            hub["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "INSUFFICIENT_FREE_TOTAL_CAPACITY")
        );
    }

    #[test]
    fn blocked_route_never_counts_as_available() {
        let report = evaluate(&sample()).unwrap();
        assert!(
            report["unavailable_sites"]
                .as_array()
                .unwrap()
                .iter()
                .any(|site| site["site_id"] == "SYNTHETIC_ROUTE_BLOCKED_POD_C")
        );
    }

    #[test]
    fn unknown_ventilation_or_water_rights_blocks_refuge() {
        let report = evaluate(&sample()).unwrap();
        let sites = report["unavailable_sites"].as_array().unwrap();
        let ventilation = sites
            .iter()
            .find(|site| site["site_id"] == "SYNTHETIC_UNKNOWN_VENTILATION_POD_D")
            .unwrap();
        assert!(
            ventilation["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "VENTILATION_NOT_QUALIFIED")
        );
        let rights = sites
            .iter()
            .find(|site| site["site_id"] == "SYNTHETIC_UNKNOWN_WATER_RIGHTS_POD_E")
            .unwrap();
        assert!(
            rights["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "SAFE_WATER_RIGHTS_NOT_QUALIFIED")
        );
    }

    #[test]
    fn protected_water_reserve_is_never_counted_as_available() {
        let mut case = sample();
        case["sites"][0]["safe_water_available_l"] = json!(18.0);
        let report = evaluate(&case).unwrap();
        let site = report["unavailable_sites"]
            .as_array()
            .unwrap()
            .iter()
            .find(|site| site["site_id"] == "SYNTHETIC_LOCAL_POD_A")
            .unwrap();
        assert!(
            site["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "SAFE_WATER_BELOW_COMMITMENTS_OR_PROTECTED_RESERVE")
        );
    }

    #[test]
    fn general_seats_cannot_substitute_for_accessible_capacity() {
        let mut case = sample();
        case["sites"][0]["accessible_capacity"] = json!(5);
        let report = evaluate(&case).unwrap();
        let site = report["unavailable_sites"]
            .as_array()
            .unwrap()
            .iter()
            .find(|site| site["site_id"] == "SYNTHETIC_LOCAL_POD_A")
            .unwrap();
        assert!(
            site["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "INSUFFICIENT_FREE_ACCESSIBLE_CAPACITY")
        );
    }

    #[test]
    fn non_accessible_occupancy_and_commitments_cannot_overbook_non_accessible_seats() {
        let mut case = sample();
        case["sites"][0]["committed_reserve"] = json!(8);
        let report = evaluate(&case).unwrap();
        let site = report["unavailable_sites"]
            .as_array()
            .unwrap()
            .iter()
            .find(|site| site["site_id"] == "SYNTHETIC_LOCAL_POD_A")
            .unwrap();
        assert!(
            site["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "CAPACITY_ACCOUNTING_INVALID")
        );
    }

    #[test]
    fn unknown_local_directory_blocks_all_sites_even_if_network_is_partitioned() {
        let mut case = sample();
        case["local_refuge_directory_state"] = json!("UNKNOWN");
        let report = evaluate(&case).unwrap();
        assert_eq!(report["eligible_site_count"], 0);
        assert_eq!(report["unavailable_sites"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn duplicate_site_ids_block_ambiguous_entries() {
        let mut case = sample();
        let dup = case["sites"][0].clone();
        case["sites"].as_array_mut().unwrap().push(dup);
        let report = evaluate(&case).unwrap();
        assert_eq!(report["eligible_site_count"], 0);
        assert!(
            report["unavailable_sites"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|site| site["site_id"] == "SYNTHETIC_LOCAL_POD_A")
                .all(|site| site["reasons"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|reason| reason == "DUPLICATE_SITE_ID"))
        );
    }
}
