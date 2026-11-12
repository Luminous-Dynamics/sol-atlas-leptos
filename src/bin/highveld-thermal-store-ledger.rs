use serde_json::{Value, json};
use std::{collections::BTreeSet, env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/THERMAL_STORE_LEDGER_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/THERMAL_STORE_LEDGER_CASE_V0.1.schema.json"
);

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|number| number.is_finite())
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let mut violations = Vec::new();
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("schema JSON parse failed: {error}")])?;

    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        violations.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(case, "study_id") != Some("HIGHVELD-THERMAL-STORE-LEDGER-001")
        || text_at(case, "schema_version") != Some("0.1")
        || text_at(case, "store_energy_basis")
            != Some("USABLE_THERMAL_ENERGY_ABOVE_DECLARED_REFERENCE_STATE")
    {
        violations.push("study/schema/store energy basis is invalid".to_owned());
    }
    if text_at(case, "maturity") != Some("SYNTHETIC_KNOWN_ANSWER")
        || text_at(case, "execution_status") != Some("NO_PHYSICAL_SIMULATION")
    {
        violations
            .push("case maturity and execution status must retain synthetic ceiling".to_owned());
    }

    let capacity = number_at(case, "store_capacity_kwh")
        .ok_or_else(|| vec!["store_capacity_kwh is missing or non-finite".to_owned()])?;
    let minimum = number_at(case, "minimum_usable_energy_kwh")
        .ok_or_else(|| vec!["minimum_usable_energy_kwh is missing or non-finite".to_owned()])?;
    let initial = number_at(case, "initial_store_energy_kwh")
        .ok_or_else(|| vec!["initial_store_energy_kwh is missing or non-finite".to_owned()])?;
    let cycle_tolerance = number_at(case, "cycle_closure_tolerance_u95_kwh")
        .ok_or_else(|| vec!["cycle closure tolerance is missing or non-finite".to_owned()])?;
    let cycle_required = case
        .get("cycle_closure_required")
        .and_then(Value::as_bool)
        .ok_or_else(|| vec!["cycle_closure_required must be boolean".to_owned()])?;

    if minimum < 0.0 || capacity <= minimum || cycle_tolerance < 0.0 {
        violations.push("usable store bounds or cycle tolerance are invalid".to_owned());
    }
    if initial < minimum || initial > capacity {
        violations.push("initial store energy is outside the declared usable range".to_owned());
    }

    let intervals = case
        .get("intervals")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["intervals must be an array".to_owned()])?;
    if intervals.is_empty() {
        violations.push("at least one interval is required".to_owned());
    }

    let allowed = [
        "RADIATIVE_SKY_PANEL",
        "DRY_AIR_COOLER",
        "GROUND_HEAT_EXCHANGER",
        "CONDITIONAL_EVAPORATIVE_REGENERATION",
    ];
    let mut prior_end = initial;
    let mut interval_reports = Vec::new();
    let mut seen_interval_ids = BTreeSet::new();
    let mut total_input = 0.0_f64;
    let mut total_rejected = 0.0_f64;
    let mut total_parasitic = 0.0_f64;
    let mut total_water = 0.0_f64;
    let mut total_duration = 0.0_f64;
    let mut peak_heat_input_kw = 0.0_f64;
    let mut peak_heat_rejection_kw = 0.0_f64;
    let mut maximum_state = initial;
    let mut maximum_interval_residual = 0.0_f64;
    let mut sink_totals = serde_json::Map::new();

    for interval in intervals {
        let id = text_at(interval, "interval_id").unwrap_or("<missing-interval-id>");
        let mut interval_violations = Vec::new();

        if !seen_interval_ids.insert(id.to_owned()) {
            interval_violations.push("duplicate interval identity".to_owned());
        }

        let duration = number_at(interval, "duration_h");
        let start = number_at(interval, "store_energy_start_kwh");
        let building_in = number_at(interval, "building_heat_input_kwh");
        let other_in = number_at(interval, "other_heat_input_kwh");
        let end = number_at(interval, "store_energy_end_kwh");
        let u95 = number_at(interval, "interval_balance_u95_kwh");
        let parasitic = number_at(interval, "parasitic_electricity_kwh");
        let water = number_at(interval, "water_makeup_l");

        let missing = [
            ("duration_h", duration),
            ("store_energy_start_kwh", start),
            ("building_heat_input_kwh", building_in),
            ("other_heat_input_kwh", other_in),
            ("store_energy_end_kwh", end),
            ("interval_balance_u95_kwh", u95),
            ("parasitic_electricity_kwh", parasitic),
            ("water_makeup_l", water),
        ]
        .into_iter()
        .filter(|(_, value)| value.is_none())
        .map(|(key, _)| key)
        .collect::<Vec<_>>();

        if !missing.is_empty() {
            let message = format!("{id}: missing/non-finite fields: {}", missing.join(", "));
            interval_violations.push(message.clone());
            violations.push(message);
            interval_reports.push(json!({
                "interval_id": id,
                "interval_state": "FAIL_MISSING_OR_NONFINITE_INPUT",
                "violations": interval_violations
            }));
            continue;
        }

        let duration = duration.unwrap();
        let start = start.unwrap();
        let building_in = building_in.unwrap();
        let other_in = other_in.unwrap();
        let end = end.unwrap();
        let u95 = u95.unwrap();
        let parasitic = parasitic.unwrap();
        let water = water.unwrap();

        if duration <= 0.0
            || building_in < 0.0
            || other_in < 0.0
            || u95 < 0.0
            || parasitic < 0.0
            || water < 0.0
        {
            interval_violations
                .push("negative energy/resource value or non-positive duration".to_owned());
        }
        if (start - prior_end).abs() > 1e-9 {
            interval_violations.push("interval start does not equal prior interval end".to_owned());
        }
        if start < minimum || start > capacity || end < minimum || end > capacity {
            interval_violations
                .push("store state exceeds usable minimum/maximum bounds".to_owned());
        }

        let rejection_rows = interval.get("rejection_by_sink").and_then(Value::as_array);
        let mut interval_rejected = 0.0_f64;
        let mut seen_sinks = BTreeSet::new();

        match rejection_rows {
            Some(rows) => {
                for row in rows {
                    let sink = text_at(row, "sink_path").unwrap_or("<missing-sink-id>");
                    if !allowed.contains(&sink) {
                        interval_violations.push(format!("unknown sink path {sink}"));
                    }
                    if !seen_sinks.insert(sink.to_owned()) {
                        interval_violations.push(format!("duplicate sink path row {sink}"));
                    }
                    match number_at(row, "energy_rejected_kwh") {
                        Some(value) if value >= 0.0 => {
                            interval_rejected += value;
                            let entry = sink_totals.entry(sink.to_owned()).or_insert(json!(0.0));
                            let old = entry.as_f64().unwrap_or(0.0);
                            *entry = json!(old + value);
                        }
                        _ => {
                            interval_violations.push(format!("invalid energy rejection for {sink}"))
                        }
                    }
                }
            }
            None => interval_violations.push("rejection_by_sink must be an array".to_owned()),
        }

        let residual = start + building_in + other_in - interval_rejected - end;
        maximum_interval_residual = maximum_interval_residual.max(residual.abs());
        if residual.abs() > u95 {
            interval_violations.push(format!(
                "energy balance residual {residual:.6} kWh exceeds u95 {u95:.6} kWh"
            ));
        }

        if duration > 0.0 {
            let heat_input_rate = (building_in + other_in) / duration;
            let rejection_rate = interval_rejected / duration;
            peak_heat_input_kw = peak_heat_input_kw.max(heat_input_rate);
            peak_heat_rejection_kw = peak_heat_rejection_kw.max(rejection_rate);
            total_duration += duration;
        }

        total_input += building_in + other_in;
        total_rejected += interval_rejected;
        total_parasitic += parasitic;
        total_water += water;
        maximum_state = maximum_state.max(start).max(end);
        prior_end = end;

        let interval_state = if interval_violations.is_empty() {
            "BALANCED_WITHIN_DECLARED_UNCERTAINTY"
        } else {
            "FAIL"
        };
        for message in &interval_violations {
            violations.push(format!("{id}: {message}"));
        }
        interval_reports.push(json!({
            "interval_id": id,
            "duration_h": duration,
            "store_energy_start_kwh": start,
            "heat_input_kwh": building_in + other_in,
            "heat_rejected_kwh": interval_rejected,
            "store_energy_end_kwh": end,
            "balance_residual_kwh": residual,
            "balance_u95_kwh": u95,
            "interval_state": interval_state,
            "violations": interval_violations
        }));
    }

    let final_state = intervals
        .last()
        .and_then(|last| number_at(last, "store_energy_end_kwh"));
    let cycle_error = final_state.map(|end| (end - initial).abs());
    if cycle_required {
        match cycle_error {
            Some(error) if error <= cycle_tolerance => {}
            Some(error) => violations.push(format!(
                "seasonal cycle closure error {error:.6} kWh exceeds tolerance {cycle_tolerance:.6} kWh"
            )),
            None => violations
                .push("final store energy is missing; cycle closure cannot be assessed".to_owned()),
        }
    }

    let audit_state = if violations.is_empty() {
        "STORE_LEDGER_PASS"
    } else {
        "STORE_LEDGER_FAIL"
    };
    Ok(json!({
        "audit_state": audit_state,
        "study_id": "HIGHVELD-THERMAL-STORE-LEDGER-001",
        "case_id": text_at(case, "case_id").unwrap_or_default(),
        "interval_count": interval_reports.len(),
        "total_duration_h": total_duration,
        "total_heat_input_kwh": total_input,
        "peak_heat_input_kw": peak_heat_input_kw,
        "total_heat_rejected_kwh": total_rejected,
        "peak_heat_rejection_kw": peak_heat_rejection_kw,
        "net_store_energy_change_kwh": final_state.unwrap_or(prior_end) - initial,
        "maximum_store_state_kwh": maximum_state,
        "minimum_store_headroom_kwh": capacity - maximum_state,
        "maximum_interval_balance_residual_kwh": maximum_interval_residual,
        "cycle_closure_error_kwh": cycle_error,
        "cycle_closure_tolerance_u95_kwh": cycle_tolerance,
        "heat_rejected_by_sink_kwh": sink_totals,
        "parasitic_electricity_kwh": total_parasitic,
        "water_makeup_l": total_water,
        "intervals": interval_reports,
        "violations": violations,
        "execution_status": "LEDGER_ONLY_NO_PHYSICAL_SIMULATION",
        "schema_engine": "NOT_RUN; schema declaration and semantic rules are checked by this Rust ledger",
        "claim_ceiling": "Conservation and state-bound checks on supplied values only; not physical validation"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value =
                serde_json::from_str(CASE).expect("committed thermal store case JSON");
            let report = evaluate(&case).expect("thermal store case structure");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["audit_state"] != "STORE_LEDGER_PASS" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-thermal-store-ledger");
            println!("  --self-test  check committed synthetic thermal-store trajectory");
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
    fn synthetic_cycle_closes_and_all_intervals_balance() {
        let report = evaluate(&sample()).expect("case evaluates");
        assert_eq!(report["audit_state"], "STORE_LEDGER_PASS");
        assert_eq!(report["interval_count"], 3);
        assert_eq!(report["cycle_closure_error_kwh"], 0.0);
        assert_eq!(report["total_heat_input_kwh"], 9.0);
        assert_eq!(report["total_heat_rejected_kwh"], 9.0);
        assert_eq!(report["peak_heat_input_kw"], 4.0);
        assert_eq!(
            report["intervals"][0]["interval_state"],
            "BALANCED_WITHIN_DECLARED_UNCERTAINTY"
        );
    }

    #[test]
    fn capacity_overrun_is_rejected_and_interval_marked_failed() {
        let mut case = sample();
        case["intervals"][0]["store_energy_end_kwh"] = json!(11.0);
        case["intervals"][1]["store_energy_start_kwh"] = json!(11.0);
        let report = evaluate(&case).unwrap();
        assert_eq!(report["audit_state"], "STORE_LEDGER_FAIL");
        assert_eq!(report["intervals"][0]["interval_state"], "FAIL");
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| {
                    value
                        .as_str()
                        .unwrap_or("")
                        .contains("store state exceeds usable minimum/maximum bounds")
                })
        );
    }

    #[test]
    fn energy_residual_outside_uncertainty_is_rejected() {
        let mut case = sample();
        case["intervals"][1]["store_energy_end_kwh"] = json!(4.0);
        case["intervals"][2]["store_energy_start_kwh"] = json!(4.0);
        let report = evaluate(&case).unwrap();
        assert_eq!(report["audit_state"], "STORE_LEDGER_FAIL");
        assert_eq!(report["intervals"][1]["interval_state"], "FAIL");
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| {
                    value
                        .as_str()
                        .unwrap_or("")
                        .contains("energy balance residual")
                })
        );
    }

    #[test]
    fn broken_interval_continuity_is_rejected() {
        let mut case = sample();
        case["intervals"][1]["store_energy_start_kwh"] = json!(3.5);
        let report = evaluate(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| { value.as_str().unwrap_or("").contains("prior interval end") })
        );
    }

    #[test]
    fn seasonal_cycle_drift_is_rejected() {
        let mut case = sample();
        case["intervals"][2]["store_energy_end_kwh"] = json!(3.0);
        let report = evaluate(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| { value.as_str().unwrap_or("").contains("cycle closure error") })
        );
    }

    #[test]
    fn duplicate_sink_paths_are_rejected() {
        let mut case = sample();
        case["intervals"][0]["rejection_by_sink"]
            .as_array_mut()
            .unwrap()
            .push(json!({"sink_path": "RADIATIVE_SKY_PANEL", "energy_rejected_kwh": 0.0}));
        let report = evaluate(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| {
                    value
                        .as_str()
                        .unwrap_or("")
                        .contains("duplicate sink path row")
                })
        );
    }
}
