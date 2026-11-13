use serde_json::{Value, json};
use std::{env, fs, process};

const SAMPLE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/THERMAL_SINK_GOVERNOR_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/THERMAL_SINK_GOVERNOR_INPUT_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn push_reason(reasons: &mut Vec<String>, reason: &str) {
    reasons.push(reason.to_owned());
}

fn evaluate_store_charge_path(input: &Value, global_ok: bool) -> Value {
    let path = &input["store_charge_path"];
    let mut reasons = Vec::new();
    if !global_ok {
        push_reason(
            &mut reasons,
            "GLOBAL_MODEL_INSTRUMENT_OR_LOCAL_CONTROL_GATE_FAILED",
        );
    }
    if text_at(path, "id") != Some("BUILDING_LOAD_TO_STORE") {
        push_reason(&mut reasons, "UNKNOWN_OR_UNSUPPORTED_STORE_CHARGE_PATH");
    }
    if text_at(path, "qualification_state") != Some("QUALIFIED") {
        push_reason(&mut reasons, "LOAD_TO_STORE_PATH_NOT_QUALIFIED");
    }
    if text_at(path, "sensor_state") != Some("QUALIFIED") {
        push_reason(&mut reasons, "LOAD_TO_STORE_SENSOR_NOT_QUALIFIED");
    }
    for key in [
        "load_boundary_evidence_ref",
        "store_state_evidence_ref",
        "heat_exchanger_evidence_ref",
    ] {
        if text_at(path, key).is_none() {
            reasons.push(format!("MISSING_LOAD_TO_STORE_EVIDENCE_{key}"));
        }
    }

    let source_temperature = number_at(path, "source_return_temperature_c");
    let store_temperature = number_at(path, "store_temperature_c");
    let minimum_approach = number_at(path, "minimum_approach_k");
    let temperature_approach = match (
        source_temperature,
        store_temperature,
        minimum_approach,
    ) {
        (Some(source), Some(store), Some(minimum)) => {
            let approach = source - store;
            if approach <= minimum {
                push_reason(
                    &mut reasons,
                    "INSUFFICIENT_LOAD_TO_STORE_TEMPERATURE_APPROACH",
                );
            }
            Some(approach)
        }
        _ => {
            push_reason(
                &mut reasons,
                "LOAD_TO_STORE_TEMPERATURE_OR_APPROACH_UNKNOWN",
            );
            None
        }
    };

    let available_power = number_at(path, "available_heat_transfer_kw");
    let requested_power = number_at(path, "requested_heat_transfer_kw");
    match (available_power, requested_power) {
        (Some(available), Some(requested))
            if available >= requested && requested > 0.0 => {}
        (Some(_), Some(_)) => {
            push_reason(
                &mut reasons,
                "INSUFFICIENT_LOAD_TO_STORE_HEAT_EXCHANGER_CAPACITY",
            );
        }
        _ => push_reason(
            &mut reasons,
            "LOAD_TO_STORE_HEAT_EXCHANGER_CAPACITY_UNKNOWN",
        ),
    }

    let duration = number_at(path, "requested_duration_h");
    let capacity = number_at(path, "store_capacity_kwh");
    let state = number_at(path, "store_energy_kwh");
    let (headroom, requested_energy) = match (duration, capacity, state, requested_power) {
        (Some(hours), Some(maximum), Some(current), Some(power))
            if hours > 0.0 && maximum > 0.0 && current >= 0.0 && current <= maximum =>
        {
            let free = maximum - current;
            let energy = power * hours;
            if energy > free {
                push_reason(
                    &mut reasons,
                    "INSUFFICIENT_FINITE_STORE_HEADROOM_NO_CONCURRENT_REGENERATION_CREDIT",
                );
            }
            (Some(free), Some(energy))
        }
        (Some(_), Some(_), Some(_), Some(_)) => {
            push_reason(&mut reasons, "LOAD_TO_STORE_STATE_OR_DURATION_INVALID");
            (None, None)
        }
        _ => {
            push_reason(&mut reasons, "LOAD_TO_STORE_HEADROOM_OR_DURATION_UNKNOWN");
            (None, None)
        }
    };

    let state = if reasons.is_empty() {
        "ELIGIBLE_FOR_OPTIMIZER_REVIEW"
    } else {
        "BLOCKED"
    };
    json!({
        "path_id": text_at(path, "id").unwrap_or("<missing-id>"),
        "state": state,
        "temperature_approach_k": temperature_approach,
        "store_headroom_kwh": headroom,
        "requested_transfer_energy_kwh": requested_energy,
        "simultaneous_regeneration_credit": "NOT_ASSUMED",
        "actuation": "NOT_ISSUED",
        "reasons": reasons
    })
}

fn evaluate(input: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("schema JSON parse failed: {error}")])?;
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(input, "schema_version") != Some("0.1") {
        errors.push("schema_version must be 0.1".to_owned());
    }
    if text_at(input, "input_id").is_none() {
        errors.push("input_id must be a non-empty string".to_owned());
    }
    let paths = input
        .get("sink_paths")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["sink_paths must be an array".to_owned()])?;
    if paths.is_empty() {
        errors.push("at least one sink path is required".to_owned());
    }

    let global_ok = text_at(input, "model_applicability") == Some("IN_DOMAIN")
        && text_at(input, "instrument_state") == Some("QUALIFIED")
        && text_at(input, "local_control_state") == Some("AVAILABLE")
        && input
            .get("manual_safe_mode_available")
            .and_then(Value::as_bool)
            == Some(true);
    let store_charge_path = evaluate_store_charge_path(input, global_ok);

    let mut id_counts = std::collections::BTreeMap::<String, usize>::new();
    for path in paths {
        if let Some(id) = text_at(path, "id") {
            *id_counts.entry(id.to_owned()).or_default() += 1;
        }
    }

    let mut eligible = Vec::new();
    let mut blocked = Vec::new();
    for path in paths {
        let id = text_at(path, "id").unwrap_or("<missing-id>");
        let mut reasons = Vec::new();
        if id_counts.get(id).copied().unwrap_or(0) > 1 {
            push_reason(&mut reasons, "DUPLICATE_SINK_PATH_ID");
        }
        if ![
            "RADIATIVE_SKY_PANEL",
            "DRY_AIR_COOLER",
            "GROUND_HEAT_EXCHANGER",
            "CONDITIONAL_EVAPORATIVE_REGENERATION",
        ]
        .contains(&id)
        {
            push_reason(&mut reasons, "UNKNOWN_OR_UNSUPPORTED_SINK_PATH");
        }
        if !global_ok {
            push_reason(
                &mut reasons,
                "GLOBAL_MODEL_INSTRUMENT_OR_LOCAL_CONTROL_GATE_FAILED",
            );
        }
        if text_at(path, "qualification_state") != Some("QUALIFIED") {
            push_reason(&mut reasons, "PATH_NOT_QUALIFIED");
        }
        if text_at(path, "sensor_state") != Some("QUALIFIED") {
            push_reason(&mut reasons, "PATH_SENSOR_NOT_QUALIFIED");
        }

        let sink_temperature = number_at(path, "effective_sink_temperature_c");
        let store_temperature = number_at(path, "store_temperature_c");
        let approach = number_at(path, "minimum_approach_k");
        match (sink_temperature, store_temperature, approach) {
            (Some(sink), Some(store), Some(minimum)) => {
                if store - sink - minimum <= 0.0 {
                    push_reason(&mut reasons, "INSUFFICIENT_TEMPERATURE_HEADROOM");
                }
            }
            _ => push_reason(&mut reasons, "TEMPERATURE_OR_APPROACH_UNKNOWN"),
        }

        match (
            number_at(path, "available_heat_rejection_kw"),
            number_at(path, "requested_heat_rejection_kw"),
        ) {
            (Some(available), Some(requested)) if available >= requested => {}
            (Some(_), Some(_)) => push_reason(&mut reasons, "INSUFFICIENT_HEAT_REJECTION_CAPACITY"),
            _ => push_reason(&mut reasons, "HEAT_REJECTION_CAPACITY_UNKNOWN"),
        }

        match (
            number_at(path, "auxiliary_power_available_kw"),
            number_at(path, "auxiliary_power_required_kw"),
        ) {
            (Some(available), Some(required)) if available >= required => {}
            (Some(_), Some(_)) => push_reason(&mut reasons, "INSUFFICIENT_AUXILIARY_POWER"),
            _ => push_reason(&mut reasons, "AUXILIARY_POWER_UNKNOWN"),
        }

        if id == "CONDITIONAL_EVAPORATIVE_REGENERATION" {
            let water = &input["water_account"];
            if text_at(water, "rights_state") != Some("QUALIFIED") {
                push_reason(&mut reasons, "WATER_RIGHTS_NOT_QUALIFIED");
            }
            if text_at(water, "quality_state") != Some("QUALIFIED") {
                push_reason(&mut reasons, "WATER_QUALITY_NOT_QUALIFIED");
            }
            if text_at(water, "restriction_state") != Some("PERMITS") {
                push_reason(&mut reasons, "WATER_RESTRICTION_DOES_NOT_PERMIT_OPERATION");
            }
            if text_at(input, "aerosol_controls_state") != Some("QUALIFIED") {
                push_reason(&mut reasons, "AEROSOL_CONTROLS_NOT_QUALIFIED");
            }
            if text_at(input, "humidity_state") != Some("WITHIN_LIMIT") {
                push_reason(&mut reasons, "HUMIDITY_OR_CONDENSATION_LIMIT_NOT_QUALIFIED");
            }
            match (
                number_at(water, "account_available_l"),
                number_at(water, "committed_l"),
                number_at(water, "essential_reserve_l"),
                number_at(path, "water_draw_l"),
            ) {
                (Some(account), Some(committed), Some(reserve), Some(draw))
                    if account - committed - reserve >= draw => {}
                (Some(_), Some(_), Some(_), Some(_)) => {
                    push_reason(
                        &mut reasons,
                        "DISCRETIONARY_WATER_WOULD_BREACH_COMMITMENTS_OR_ESSENTIAL_RESERVE",
                    );
                }
                _ => push_reason(&mut reasons, "WATER_ACCOUNT_OR_DRAW_UNKNOWN"),
            }
        } else {
            match number_at(path, "water_draw_l") {
                Some(draw) if draw == 0.0 => {}
                Some(_) => {
                    push_reason(
                        &mut reasons,
                        "UNDECLARED_WATER_DRAW_ON_NON_EVAPORATIVE_PATH",
                    );
                }
                None => push_reason(&mut reasons, "NON_EVAPORATIVE_WATER_DRAW_UNKNOWN"),
            }
        }

        if reasons.is_empty() {
            eligible.push(json!({
                "path_id": id,
                "state": "ELIGIBLE_FOR_OPTIMIZER_REVIEW",
                "actuation": "NOT_ISSUED"
            }));
        } else {
            blocked.push(json!({
                "path_id": id,
                "state": "BLOCKED",
                "reasons": reasons
            }));
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(json!({
        "audit_state": "FEASIBILITY_FILTER_COMPLETE",
        "input_id": text_at(input, "input_id").unwrap_or_default(),
        "network_state": text_at(input, "network_state").unwrap_or("UNKNOWN"),
        "local_control_state": text_at(input, "local_control_state").unwrap_or("UNKNOWN"),
        "store_charge_path": store_charge_path,
        "eligible_paths": eligible,
        "blocked_paths": blocked,
        "optimizer_authority": "MAY_COMPARE_ELIGIBLE_PATHS_ONLY",
        "actuation": "NONE",
        "execution_status": "NO_HARDWARE_ACTUATED",
        "claim_ceiling": "Eligibility only; no thermal performance or operational safety certification is claimed"
    }))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str).unwrap_or("--help") {
        "--self-test" => {
            let input: Value = serde_json::from_str(SAMPLE).expect("committed governor case JSON");
            match evaluate(&input) {
                Ok(output) => println!(
                    "{}",
                    serde_json::to_string_pretty(&output).unwrap_or_default()
                ),
                Err(errors) => {
                    eprintln!(
                        "{}",
                        serde_json::to_string_pretty(&json!({"errors": errors}))
                            .unwrap_or_default()
                    );
                    process::exit(1);
                }
            }
        }
        "--input" if args.len() == 2 => {
            let raw = fs::read_to_string(&args[1]).unwrap_or_else(|error| {
                eprintln!("cannot read input: {error}");
                process::exit(2);
            });
            let input: Value = serde_json::from_str(&raw).unwrap_or_else(|error| {
                eprintln!("invalid JSON: {error}");
                process::exit(2);
            });
            match evaluate(&input) {
                Ok(output) => println!(
                    "{}",
                    serde_json::to_string_pretty(&output).unwrap_or_default()
                ),
                Err(errors) => {
                    eprintln!(
                        "{}",
                        serde_json::to_string_pretty(&json!({"errors": errors}))
                            .unwrap_or_default()
                    );
                    process::exit(1);
                }
            }
        }
        "--help" | "-h" => {
            println!("highveld-thermal-sink-dispatch-governor");
            println!("  --self-test    evaluate the committed synthetic control-gate case");
            println!("  --input <path> evaluate a JSON case; this never actuates equipment");
        }
        _ => {
            eprintln!("invalid arguments; use --help");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        serde_json::from_str(SAMPLE).expect("sample JSON")
    }

    #[test]
    fn qualified_radiative_path_survives_network_partition_with_local_control() {
        let report = evaluate(&sample()).expect("sample case must evaluate");
        assert_eq!(report["network_state"].as_str(), Some("PARTITIONED"));
        assert!(
            report["eligible_paths"]
                .as_array()
                .unwrap()
                .iter()
                .any(|path| path["path_id"] == "RADIATIVE_SKY_PANEL")
        );
    }

    #[test]
    fn temperature_approach_blocks_dry_cooler() {
        let report = evaluate(&sample()).expect("sample case must evaluate");
        let blocked = report["blocked_paths"].as_array().unwrap();
        let dry = blocked
            .iter()
            .find(|path| path["path_id"] == "DRY_AIR_COOLER")
            .unwrap();
        assert!(
            dry["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "INSUFFICIENT_TEMPERATURE_HEADROOM")
        );
    }

    #[test]
    fn restrictions_and_reserve_block_evaporative_path() {
        let report = evaluate(&sample()).expect("sample case must evaluate");
        let evap = report["blocked_paths"]
            .as_array()
            .unwrap()
            .iter()
            .find(|path| path["path_id"] == "CONDITIONAL_EVAPORATIVE_REGENERATION")
            .unwrap();
        let reasons = evap["reasons"].as_array().unwrap();
        assert!(
            reasons
                .iter()
                .any(|reason| reason == "WATER_RESTRICTION_DOES_NOT_PERMIT_OPERATION")
        );
        assert!(
            reasons.iter().any(|reason| reason
                == "DISCRETIONARY_WATER_WOULD_BREACH_COMMITMENTS_OR_ESSENTIAL_RESERVE")
        );
    }

    #[test]
    fn valid_evaporative_positive_control_is_preserved() {
        let mut input = sample();
        input["water_account"]["restriction_state"] = json!("PERMITS");
        input["water_account"]["account_available_l"] = json!(200.0);
        input["water_account"]["committed_l"] = json!(10.0);
        input["water_account"]["essential_reserve_l"] = json!(100.0);
        input["sink_paths"][2]["water_draw_l"] = json!(20.0);
        let report = evaluate(&input).expect("positive control must be evaluable");
        assert!(
            report["eligible_paths"]
                .as_array()
                .unwrap()
                .iter()
                .any(|path| path["path_id"] == "CONDITIONAL_EVAPORATIVE_REGENERATION")
        );
    }

    #[test]
    fn unknown_global_instrument_state_blocks_active_paths() {
        let mut input = sample();
        input["instrument_state"] = json!("UNKNOWN");
        let report = evaluate(&input).expect("unknown state should block, not crash");
        assert!(report["eligible_paths"].as_array().unwrap().is_empty());
        assert_eq!(report["blocked_paths"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn unknown_sink_path_fails_closed() {
        let mut input = sample();
        input["sink_paths"][0]["id"] = json!("UNKNOWN_SINK");
        let report = evaluate(&input).expect("unknown path should be blocked, not crash");
        assert!(
            report["eligible_paths"]
                .as_array()
                .unwrap()
                .iter()
                .all(|path| path["path_id"] != "UNKNOWN_SINK")
        );
        let blocked = report["blocked_paths"].as_array().unwrap();
        let unknown = blocked
            .iter()
            .find(|path| path["path_id"] == "UNKNOWN_SINK")
            .unwrap();
        assert!(
            unknown["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "UNKNOWN_OR_UNSUPPORTED_SINK_PATH")
        );
    }

    #[test]
    fn duplicate_sink_path_ids_block_all_ambiguous_entries() {
        let mut input = sample();
        input["sink_paths"][1]["id"] = json!("RADIATIVE_SKY_PANEL");
        let report = evaluate(&input).expect("duplicate path IDs should be blocked");
        assert!(
            report["eligible_paths"]
                .as_array()
                .unwrap()
                .iter()
                .all(|path| path["path_id"] != "RADIATIVE_SKY_PANEL")
        );
        let matching = report["blocked_paths"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|path| path["path_id"] == "RADIATIVE_SKY_PANEL")
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 2);
        assert!(matching.iter().all(|path| {
            path["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "DUPLICATE_SINK_PATH_ID")
        }));
    }

    #[test]
    fn non_evaporative_sink_cannot_hide_water_draw() {
        let mut input = sample();
        input["sink_paths"][0]["water_draw_l"] = json!(0.1);
        let report = evaluate(&input).expect("undeclared water draw should be blocked");
        let blocked = report["blocked_paths"].as_array().unwrap();
        let radiative = blocked
            .iter()
            .find(|path| path["path_id"] == "RADIATIVE_SKY_PANEL")
            .unwrap();
        assert!(
            radiative["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "UNDECLARED_WATER_DRAW_ON_NON_EVAPORATIVE_PATH")
        );
    }

    #[test]
    fn load_to_store_path_passes_when_grade_capacity_and_headroom_are_sufficient() {
        let report = evaluate(&sample()).expect("sample case must evaluate");
        assert_eq!(
            report["store_charge_path"]["state"],
            "ELIGIBLE_FOR_OPTIMIZER_REVIEW"
        );
        assert_eq!(report["store_charge_path"]["temperature_approach_k"], 9.0);
        assert_eq!(report["store_charge_path"]["store_headroom_kwh"], 4.0);
        assert_eq!(
            report["store_charge_path"]["requested_transfer_energy_kwh"],
            3.0
        );
        assert_eq!(
            report["store_charge_path"]["simultaneous_regeneration_credit"],
            "NOT_ASSUMED"
        );
    }

    #[test]
    fn inadequate_load_to_store_temperature_grade_blocks_charge() {
        let mut input = sample();
        input["store_charge_path"]["source_return_temperature_c"] = json!(24.0);
        let report = evaluate(&input).expect("case evaluates");
        assert_eq!(report["store_charge_path"]["state"], "BLOCKED");
        assert!(report["store_charge_path"]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "INSUFFICIENT_LOAD_TO_STORE_TEMPERATURE_APPROACH"));
    }

    #[test]
    fn insufficient_store_headroom_blocks_charge_without_regeneration_credit() {
        let mut input = sample();
        input["store_charge_path"]["store_energy_kwh"] = json!(8.0);
        let report = evaluate(&input).expect("case evaluates");
        assert_eq!(report["store_charge_path"]["state"], "BLOCKED");
        assert!(report["store_charge_path"]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason
                == "INSUFFICIENT_FINITE_STORE_HEADROOM_NO_CONCURRENT_REGENERATION_CREDIT"));
    }

    #[test]
    fn insufficient_load_to_store_exchanger_power_blocks_charge() {
        let mut input = sample();
        input["store_charge_path"]["available_heat_transfer_kw"] = json!(1.0);
        let report = evaluate(&input).expect("case evaluates");
        assert_eq!(report["store_charge_path"]["state"], "BLOCKED");
        assert!(report["store_charge_path"]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "INSUFFICIENT_LOAD_TO_STORE_HEAT_EXCHANGER_CAPACITY"));
    }

    #[test]
    fn unknown_load_to_store_sensor_blocks_charge() {
        let mut input = sample();
        input["store_charge_path"]["sensor_state"] = json!("UNKNOWN");
        let report = evaluate(&input).expect("case evaluates");
        assert_eq!(report["store_charge_path"]["state"], "BLOCKED");
        assert!(report["store_charge_path"]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "LOAD_TO_STORE_SENSOR_NOT_QUALIFIED"));
    }

    #[test]
    fn optimizer_cannot_actuate() {
        let report = evaluate(&sample()).expect("sample case must evaluate");
        assert_eq!(report["actuation"].as_str(), Some("NONE"));
        assert_eq!(
            report["execution_status"].as_str(),
            Some("NO_HARDWARE_ACTUATED")
        );
    }
}
