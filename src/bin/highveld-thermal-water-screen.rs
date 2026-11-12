use serde_json::{Value, json};
use std::{env, fs, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/THERMAL_WATER_SCREEN_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/THERMAL_WATER_SCREEN_CASE_V0.1.schema.json"
);
const ENERGYPLUS_INTERFACE_MAP: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/ENERGYPLUS_THERMAL_WATER_INTERFACE_MAP_V0.1.json"
);

fn number(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|n| n.is_finite())
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn calculate(input: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let schema: Value =
        serde_json::from_str(SCHEMA).map_err(|e| vec![format!("schema JSON parse failed: {e}")])?;
    if text(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema declaration is not Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("schema_version", "0.1.0"),
        ("study_id", "HIGHVELD-PASSIVE-COOLING-WATER-001"),
        ("evidence_class", "SyntheticScreeningExample"),
        ("maturity", "ANALYTICAL_SCREENING_ONLY"),
    ] {
        if text(input, key) != Some(expected) {
            errors.push(format!("{key} must be {expected}"));
        }
    }
    if input.get("useful_occupied_zone_cooling_kwh") != Some(&Value::Null) {
        errors.push("screening input must not prepopulate useful occupied-zone cooling".to_owned());
    }
    let keys = [
        "water_volume_m3",
        "water_temperature_swing_k",
        "evaporation_rate_l_per_hour",
        "exposure_duration_hours",
        "pump_input_power_kw",
        "pump_duty_fraction",
        "discretionary_water_budget_l",
        "water_density_kg_per_l",
        "specific_heat_kj_per_kg_k",
        "latent_heat_kj_per_kg",
    ];
    let mut v = std::collections::BTreeMap::new();
    for key in keys {
        match number(input, key) {
            Some(n) => {
                v.insert(key, n);
            }
            None => errors.push(format!("{key} must be a finite numeric value")),
        }
    }
    if let (Some(duty), Some(budget), Some(volume)) = (
        v.get("pump_duty_fraction"),
        v.get("discretionary_water_budget_l"),
        v.get("water_volume_m3"),
    ) {
        if !(0.0..=1.0).contains(duty) {
            errors.push("pump_duty_fraction must be between zero and one".to_owned());
        }
        if *budget < 0.0 || *volume <= 0.0 {
            errors.push("water budget must be nonnegative and pool volume positive".to_owned());
        }
    }
    for key in [
        "water_temperature_swing_k",
        "evaporation_rate_l_per_hour",
        "exposure_duration_hours",
        "pump_input_power_kw",
        "discretionary_water_budget_l",
    ] {
        if v.get(key).is_some_and(|n| *n < 0.0) {
            errors.push(format!("{key} must be nonnegative"));
        }
    }
    for key in [
        "water_density_kg_per_l",
        "specific_heat_kj_per_kg_k",
        "latent_heat_kj_per_kg",
    ] {
        if v.get(key).is_some_and(|n| *n <= 0.0) {
            errors.push(format!("{key} must be positive"));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let volume_m3 = v["water_volume_m3"];
    let delta_k = v["water_temperature_swing_k"];
    let evaporation_l = v["evaporation_rate_l_per_hour"] * v["exposure_duration_hours"];
    let storage_kwh =
        volume_m3 * 1000.0 * v["water_density_kg_per_l"] * v["specific_heat_kj_per_kg_k"] * delta_k
            / 3600.0;
    let latent_interface_kwh =
        evaporation_l * v["water_density_kg_per_l"] * v["latent_heat_kj_per_kg"] / 3600.0;
    let pump_runtime_h = v["exposure_duration_hours"] * v["pump_duty_fraction"];
    let pump_electricity_kwh = v["pump_input_power_kw"] * pump_runtime_h;
    let budget_l = v["discretionary_water_budget_l"];
    let water_remaining_l = budget_l - evaporation_l;

    let gate_fields = [
        ("source_rights_state", "QUALIFIED"),
        ("water_quality_state", "QUALIFIED"),
        ("weather_input_state", "QUALIFIED"),
        ("instrument_state", "QUALIFIED"),
        ("restriction_state", "CHECKED_PERMITTED"),
    ];
    let mut failed_gates = Vec::new();
    for (field, expected) in gate_fields {
        if text(input, field) != Some(expected) {
            failed_gates.push(json!({
                "field": field,
                "observed": input.get(field).cloned().unwrap_or(Value::Null),
                "required": expected
            }));
        }
    }
    if evaporation_l > budget_l {
        failed_gates.push(json!({
            "field": "discretionary_water_budget_l",
            "observed_l": budget_l,
            "required_at_least_l": evaporation_l
        }));
    }

    Ok(json!({
        "audit_state": "ANALYTICAL_SCREEN_PASS",
        "scenario_id": text(input, "scenario_id").unwrap_or_default(),
        "maturity": "ANALYTICAL_SCREENING_ONLY",
        "sensible_storage_capacity_kwh": storage_kwh,
        "evaporated_water_l": evaporation_l,
        "latent_heat_absorbed_at_water_interface_kwh": latent_interface_kwh,
        "pump_electricity_kwh": pump_electricity_kwh,
        "discretionary_water_budget_l": budget_l,
        "remaining_water_budget_l": water_remaining_l,
        "water_budget_exceeded": evaporation_l > budget_l,
        "useful_occupied_zone_cooling_kwh": Value::Null,
        "net_useful_cooling_kwh": Value::Null,
        "failed_operational_gates": failed_gates,
        "operational_eligibility": if failed_gates.is_empty() && input.get("useful_occupied_zone_cooling_kwh").and_then(Value::as_f64).is_some() { "REQUIRES_INDEPENDENT_VALIDATION" } else { "BLOCKED_OR_UNKNOWN" },
        "claim_ceiling": "Storage capacity and latent-interface heat are not useful occupied-zone cooling; no physical or economic benefit is claimed"
    }))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str).unwrap_or("--help") {
        "--self-test" | "--screen-default" => {
            let input: Value = serde_json::from_str(CASE).expect("committed screening case JSON");
            match calculate(&input) {
                Ok(report) => println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
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
                eprintln!("invalid input JSON: {error}");
                process::exit(2);
            });
            match calculate(&input) {
                Ok(report) => println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
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
            println!("highveld-thermal-water-screen");
            println!("  --self-test       screen the committed synthetic known-answer case");
            println!("  --screen-default  alias for --self-test");
            println!("  --input <path>    screen an external JSON case (analytical ceilings only)");
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
        serde_json::from_str(CASE).expect("sample case JSON")
    }

    #[test]
    fn energyplus_pool_and_water_storage_scopes_remain_distinct() {
        let map: Value =
            serde_json::from_str(ENERGYPLUS_INTERFACE_MAP).expect("EnergyPlus interface map JSON");
        assert_eq!(map["model_engine"].as_str(), Some("EnergyPlus"));
        assert_eq!(map["model_engine_version"].as_str(), Some("26.2.0"));
        assert_eq!(map["execution_status"].as_str(), Some("NOT_EXECUTED"));
        let bindings = map["object_bindings"].as_array().expect("object bindings");
        let indoor_pool = bindings
            .iter()
            .find(|item| item["object"] == "SwimmingPool:Indoor")
            .expect("indoor swimming pool model binding");
        assert_eq!(
            indoor_pool["role"].as_str(),
            Some("INDOOR_POOL_ZONE_HEAT_AND_MOISTURE_BALANCE")
        );
        let water_storage = bindings
            .iter()
            .find(|item| item["object"] == "WaterUse:Storage")
            .expect("water-use storage binding");
        assert!(
            water_storage["do_not_infer"]
                .as_array()
                .expect("water storage limits")
                .iter()
                .any(|item| item == "dynamic_pool_thermal_model")
        );
        assert_eq!(
            map["outdoor_feature_boundary"]
                ["dedicated_swimmingpool_outdoor_object_found_in_inspected_v26_2_index"]
                .as_bool(),
            Some(false)
        );
    }

    #[test]
    fn known_answer_water_storage_is_correct() {
        let result = calculate(&sample()).expect("valid synthetic case");
        let value = result["sensible_storage_capacity_kwh"].as_f64().unwrap();
        assert!((value - 11.627_777_777_8).abs() < 1.0e-8);
    }

    #[test]
    fn latent_heat_is_not_reported_as_useful_zone_cooling() {
        let result = calculate(&sample()).expect("valid synthetic case");
        let latent = result["latent_heat_absorbed_at_water_interface_kwh"]
            .as_f64()
            .unwrap();
        assert!((latent - 2.722_222_222_2).abs() < 1.0e-8);
        assert!(result["useful_occupied_zone_cooling_kwh"].is_null());
        assert!(result["net_useful_cooling_kwh"].is_null());
    }

    #[test]
    fn pump_energy_and_water_budget_are_separate() {
        let result = calculate(&sample()).expect("valid synthetic case");
        assert!((result["pump_electricity_kwh"].as_f64().unwrap() - 0.48).abs() < 1.0e-10);
        assert!((result["remaining_water_budget_l"].as_f64().unwrap() - 1.0).abs() < 1.0e-10);
        assert_eq!(result["water_budget_exceeded"].as_bool(), Some(false));
    }

    #[test]
    fn unqualified_source_weather_and_instruments_block_operation() {
        let result = calculate(&sample()).expect("analytical calculation still allowed");
        assert_eq!(
            result["operational_eligibility"].as_str(),
            Some("BLOCKED_OR_UNKNOWN")
        );
        assert!(result["failed_operational_gates"].as_array().unwrap().len() >= 4);
    }

    #[test]
    fn rejects_negative_or_non_finite_physical_inputs() {
        let mut input = sample();
        input["water_volume_m3"] = json!(-1.0);
        assert!(calculate(&input).is_err());
    }

    #[test]
    fn budget_exhaustion_is_reported_not_hidden() {
        let mut input = sample();
        input["discretionary_water_budget_l"] = json!(3.0);
        let result = calculate(&input).expect("screen should report budget failure");
        assert_eq!(result["water_budget_exceeded"].as_bool(), Some(true));
        assert_eq!(
            result["operational_eligibility"].as_str(),
            Some("BLOCKED_OR_UNKNOWN")
        );
    }
}
