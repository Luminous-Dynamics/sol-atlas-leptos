use serde_json::{Value, json};
use std::{collections::BTreeSet, env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_PASSIVE_REGENERATION_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_PASSIVE_REGENERATION_CASE_V0.1.schema.json"
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

fn audit(case: &Value) -> Result<Value, Vec<String>> {
    let schema: Value = serde_json::from_str(SCHEMA).map_err(|error| {
        vec![format!(
            "pool passive regeneration schema parse failed: {error}"
        )]
    })?;
    let mut errors = Vec::new();
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("pool passive regeneration schema must declare Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-POOL-PASSIVE-REGENERATION-LEDGER-001"),
        ("maturity", "SYNTHETIC_KNOWN_ANSWER"),
        ("execution_status", "NO_PHYSICAL_SIMULATION"),
    ] {
        if text_at(case, key) != Some(expected) {
            errors.push(format!("{key} must preserve the synthetic claim ceiling"));
        }
    }

    let evidence = &case["evidence_refs"];
    for key in [
        "source_bundle_ref",
        "layered_state_method_ref",
        "flux_measurement_method_ref",
        "water_meter_method_ref",
        "evaporation_method_ref",
        "latent_heat_method_ref",
        "uncertainty_method_ref",
        "independent_oracle_ref",
        "water_rights_evidence_ref",
    ] {
        if text_at(evidence, key).is_none() {
            errors.push(format!("missing top-level evidence reference {key}"));
        }
    }

    let pool = &case["pool"];
    if text_at(pool, "energy_state_basis")
        != Some("LAYER_RESOLVED_WATER_ENTHALPY_ABOVE_REFERENCE_OR_VALIDATED_EQUIVALENT")
    {
        errors.push("pool energy state must be layer-resolved or validated equivalent".to_owned());
    }
    let energy_min = number_at(pool, "usable_thermal_energy_min_kwh").unwrap_or(f64::NAN);
    let energy_capacity = number_at(pool, "usable_thermal_energy_capacity_kwh").unwrap_or(f64::NAN);
    let initial_energy = number_at(pool, "initial_thermal_energy_kwh").unwrap_or(f64::NAN);
    let water_capacity = number_at(pool, "water_volume_capacity_l").unwrap_or(f64::NAN);
    let initial_volume = number_at(pool, "initial_water_volume_l").unwrap_or(f64::NAN);
    let temperature_limit = number_at(pool, "maximum_permitted_temperature_c").unwrap_or(f64::NAN);
    if !energy_min.is_finite()
        || !energy_capacity.is_finite()
        || !initial_energy.is_finite()
        || energy_min < 0.0
        || energy_capacity <= energy_min
        || initial_energy < energy_min
        || initial_energy > energy_capacity
    {
        errors.push("pool thermal energy bounds are invalid".to_owned());
    }
    if !water_capacity.is_finite()
        || !initial_volume.is_finite()
        || water_capacity <= 0.0
        || initial_volume < 0.0
        || initial_volume > water_capacity
    {
        errors.push("pool water-volume bounds are invalid".to_owned());
    }
    if !temperature_limit.is_finite() || text_at(pool, "temperature_limit_authority_ref").is_none()
    {
        errors.push("owner/authority temperature ceiling and evidence are required".to_owned());
    }

    let water_governance = &case["water_governance"];
    let water_rights = text_at(water_governance, "water_rights_state").unwrap_or("UNKNOWN");
    let water_quality = text_at(water_governance, "water_quality_state").unwrap_or("UNKNOWN");
    let makeup_permitted = water_governance
        .get("discretionary_makeup_permitted")
        .and_then(Value::as_bool)
        == Some(true);
    let makeup_budget =
        number_at(water_governance, "discretionary_makeup_budget_l").unwrap_or(f64::NAN);
    if !makeup_budget.is_finite() || makeup_budget < 0.0 {
        errors.push("discretionary makeup water budget is invalid".to_owned());
    }
    let latent = &case["latent_heat_method"];
    let latent_heat_per_l = number_at(latent, "latent_heat_kwh_per_l").unwrap_or(f64::NAN);
    if !latent_heat_per_l.is_finite()
        || latent_heat_per_l <= 0.0
        || text_at(latent, "method_ref").is_none()
    {
        errors.push("temperature-specific latent heat method is missing or invalid".to_owned());
    }

    let intervals = case
        .get("intervals")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["intervals must be an array".to_owned()])?;
    if intervals.len() < 2 {
        errors.push("at least two intervals are required".to_owned());
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut seen_ids = BTreeSet::new();
    let mut prior_energy = initial_energy;
    let mut prior_volume = initial_volume;
    let mut total_heat_input = 0.0_f64;
    let mut total_longwave = 0.0_f64;
    let mut total_sensible = 0.0_f64;
    let mut total_evaporative = 0.0_f64;
    let mut total_ground = 0.0_f64;
    let mut total_heat_exchanger = 0.0_f64;
    let mut total_evaporation = 0.0_f64;
    let mut total_makeup = 0.0_f64;
    let mut total_rain = 0.0_f64;
    let mut total_leakage = 0.0_f64;
    let mut total_withdrawal = 0.0_f64;
    let mut total_backwash = 0.0_f64;
    let mut total_overflow = 0.0_f64;
    let mut maximum_layer_temperature = f64::NEG_INFINITY;
    let mut maximum_thermal_residual = 0.0_f64;
    let mut maximum_water_residual = 0.0_f64;
    let mut maximum_evaporation_latent_residual = 0.0_f64;
    let mut cover_summary = serde_json::Map::new();
    let mut interval_reports = Vec::new();

    for interval in intervals {
        let id = text_at(interval, "interval_id").unwrap_or("<missing-interval-id>");
        let mut interval_errors = Vec::new();
        if !seen_ids.insert(id.to_owned()) {
            interval_errors.push("duplicate interval identity".to_owned());
        }
        for key in [
            "thermal_state_start_evidence_ref",
            "thermal_state_end_evidence_ref",
            "heat_flux_evidence_ref",
            "water_meter_evidence_ref",
            "weather_evidence_ref",
            "cover_state_evidence_ref",
            "temperature_limit_evidence_ref",
            "uncertainty_evidence_ref",
        ] {
            if text_at(interval, key).is_none() {
                interval_errors.push(format!("missing interval evidence reference {key}"));
            }
        }

        let duration = number_at(interval, "duration_h").unwrap_or(f64::NAN);
        let water = &interval["weather"];
        let weather_valid = [
            "dry_bulb_c",
            "relative_humidity_percent",
            "wind_speed_m_s",
            "global_solar_w_m2",
            "downwelling_longwave_w_m2",
            "sky_view_factor",
        ]
        .iter()
        .all(|key| number_at(water, key).is_some());
        if !weather_valid {
            interval_errors.push("weather boundary contains missing/non-finite fields".to_owned());
        }
        if !duration.is_finite() || duration <= 0.0 {
            interval_errors.push("interval duration must be positive and finite".to_owned());
        }
        let cover_mode = text_at(interval, "cover_mode").unwrap_or("UNKNOWN");
        if ![
            "UNCOVERED",
            "COVERED",
            "SHADED_EXPOSED",
            "SHADED_COVERED",
            "ADAPTIVE",
        ]
        .contains(&cover_mode)
        {
            interval_errors.push("cover mode is unknown".to_owned());
        }

        let start_energy = number_at(interval, "thermal_energy_start_kwh").unwrap_or(f64::NAN);
        let end_energy = number_at(interval, "thermal_energy_end_kwh").unwrap_or(f64::NAN);
        let max_layer_temp = number_at(interval, "maximum_layer_temperature_c").unwrap_or(f64::NAN);
        let temperature_start = number_at(interval, "pool_temperature_start_c").unwrap_or(f64::NAN);
        let temperature_end = number_at(interval, "pool_temperature_end_c").unwrap_or(f64::NAN);
        let building_heat = number_at(interval, "building_heat_input_kwh").unwrap_or(f64::NAN);
        let solar = number_at(interval, "solar_absorbed_kwh").unwrap_or(f64::NAN);
        let other_input = number_at(interval, "other_heat_input_kwh").unwrap_or(f64::NAN);
        let liquid_in = number_at(interval, "liquid_enthalpy_in_kwh").unwrap_or(f64::NAN);
        let longwave = number_at(interval, "longwave_heat_rejected_kwh").unwrap_or(f64::NAN);
        let sensible = number_at(interval, "sensible_heat_rejected_kwh").unwrap_or(f64::NAN);
        let evaporative =
            number_at(interval, "evaporative_latent_heat_rejected_kwh").unwrap_or(f64::NAN);
        let ground = number_at(interval, "ground_heat_rejected_kwh").unwrap_or(f64::NAN);
        let hx = number_at(interval, "heat_exchanger_heat_rejected_kwh").unwrap_or(f64::NAN);
        let liquid_out = number_at(interval, "liquid_enthalpy_out_kwh").unwrap_or(f64::NAN);
        let thermal_u95 = number_at(interval, "thermal_balance_u95_kwh").unwrap_or(f64::NAN);
        let thermal_terms = [
            start_energy,
            end_energy,
            max_layer_temp,
            temperature_start,
            temperature_end,
            building_heat,
            solar,
            other_input,
            liquid_in,
            longwave,
            sensible,
            evaporative,
            ground,
            hx,
            liquid_out,
            thermal_u95,
        ];
        if thermal_terms.iter().any(|value| !value.is_finite())
            || [
                building_heat,
                solar,
                other_input,
                liquid_in,
                longwave,
                sensible,
                evaporative,
                ground,
                hx,
                liquid_out,
                thermal_u95,
            ]
            .iter()
            .any(|value| *value < 0.0)
        {
            interval_errors
                .push("thermal state/flux has missing, non-finite or negative terms".to_owned());
        }
        if (start_energy - prior_energy).abs() > 1e-9 {
            interval_errors
                .push("thermal energy state is discontinuous from prior interval".to_owned());
        }
        if start_energy < energy_min
            || start_energy > energy_capacity
            || end_energy < energy_min
            || end_energy > energy_capacity
        {
            interval_errors.push("pool thermal energy exceeds finite usable bounds".to_owned());
        }
        if max_layer_temp > temperature_limit {
            interval_errors
                .push("maximum layer temperature exceeds owner/authority ceiling".to_owned());
        }
        let thermal_input = building_heat + solar + other_input + liquid_in;
        let thermal_output = longwave + sensible + evaporative + ground + hx + liquid_out;
        let thermal_residual = start_energy + thermal_input - thermal_output - end_energy;
        if thermal_residual.abs() > thermal_u95 {
            interval_errors.push(format!(
                "thermal energy residual {thermal_residual:.6} kWh exceeds u95 {thermal_u95:.6} kWh"
            ));
        }

        let volume_start = number_at(interval, "water_volume_start_l").unwrap_or(f64::NAN);
        let makeup = number_at(interval, "makeup_l").unwrap_or(f64::NAN);
        let rain = number_at(interval, "rain_l").unwrap_or(f64::NAN);
        let evaporation = number_at(interval, "evaporation_l").unwrap_or(f64::NAN);
        let withdrawal = number_at(interval, "intentional_withdrawal_l").unwrap_or(f64::NAN);
        let leakage = number_at(interval, "leakage_l").unwrap_or(f64::NAN);
        let backwash = number_at(interval, "backwash_purge_l").unwrap_or(f64::NAN);
        let overflow = number_at(interval, "overflow_l").unwrap_or(f64::NAN);
        let volume_end = number_at(interval, "water_volume_end_l").unwrap_or(f64::NAN);
        let water_u95 = number_at(interval, "water_balance_u95_l").unwrap_or(f64::NAN);
        let water_terms = [
            volume_start,
            makeup,
            rain,
            evaporation,
            withdrawal,
            leakage,
            backwash,
            overflow,
            volume_end,
            water_u95,
        ];
        if water_terms.iter().any(|value| !value.is_finite())
            || [
                makeup,
                rain,
                evaporation,
                withdrawal,
                leakage,
                backwash,
                overflow,
                water_u95,
            ]
            .iter()
            .any(|value| *value < 0.0)
        {
            interval_errors.push(
                "water-volume ledger contains missing, non-finite or negative terms".to_owned(),
            );
        }
        if (volume_start - prior_volume).abs() > 1e-9 {
            interval_errors.push("water volume is discontinuous from prior interval".to_owned());
        }
        if volume_start < 0.0
            || volume_end < 0.0
            || volume_start > water_capacity
            || volume_end > water_capacity
        {
            interval_errors.push("pool water volume exceeds declared capacity".to_owned());
        }
        let water_residual = volume_start + makeup + rain
            - evaporation
            - withdrawal
            - leakage
            - backwash
            - overflow
            - volume_end;
        if water_residual.abs() > water_u95 {
            interval_errors.push(format!(
                "water-volume residual {water_residual:.6} L exceeds u95 {water_u95:.6} L"
            ));
        }

        if makeup > 0.0 {
            if water_rights != "QUALIFIED" || water_quality != "QUALIFIED" || !makeup_permitted {
                interval_errors.push(
                    "make-up water lacks qualified rights, quality or discretionary permission"
                        .to_owned(),
                );
            }
            if text_at(water_governance, "makeup_source_class") == Some("NONE") {
                interval_errors.push("make-up water source class is NONE".to_owned());
            }
        }
        if total_makeup + makeup > makeup_budget {
            interval_errors
                .push("cumulative pool make-up exceeds discretionary water budget".to_owned());
        }

        let expected_evaporative = evaporation * latent_heat_per_l;
        let latent_residual = expected_evaporative - evaporative;
        let latent_u95 = number_at(interval, "evaporation_latent_heat_u95_kwh").unwrap_or(f64::NAN);
        if !latent_u95.is_finite() || latent_u95 < 0.0 {
            interval_errors
                .push("evaporation latent-heat uncertainty is missing or invalid".to_owned());
        } else if latent_residual.abs() > latent_u95 {
            interval_errors.push(format!(
                "evaporation latent-heat cross-check residual {latent_residual:.6} kWh exceeds u95 {latent_u95:.6} kWh"
            ));
        }

        total_heat_input += thermal_input;
        total_longwave += longwave;
        total_sensible += sensible;
        total_evaporative += evaporative;
        total_ground += ground;
        total_heat_exchanger += hx;
        total_evaporation += evaporation;
        total_makeup += makeup;
        total_rain += rain;
        total_leakage += leakage;
        total_withdrawal += withdrawal;
        total_backwash += backwash;
        total_overflow += overflow;
        maximum_layer_temperature = maximum_layer_temperature.max(max_layer_temp);
        maximum_thermal_residual = maximum_thermal_residual.max(thermal_residual.abs());
        maximum_water_residual = maximum_water_residual.max(water_residual.abs());
        maximum_evaporation_latent_residual =
            maximum_evaporation_latent_residual.max(latent_residual.abs());

        let summary = cover_summary.entry(cover_mode.to_owned()).or_insert(json!({
            "interval_count":0,
            "duration_h":0.0,
            "evaporation_l":0.0,
            "evaporative_latent_heat_rejected_kwh":0.0,
            "total_heat_rejected_kwh":0.0
        }));
        summary["interval_count"] = json!(summary["interval_count"].as_u64().unwrap_or(0) + 1);
        summary["duration_h"] = json!(summary["duration_h"].as_f64().unwrap_or(0.0) + duration);
        summary["evaporation_l"] =
            json!(summary["evaporation_l"].as_f64().unwrap_or(0.0) + evaporation);
        summary["evaporative_latent_heat_rejected_kwh"] = json!(
            summary["evaporative_latent_heat_rejected_kwh"]
                .as_f64()
                .unwrap_or(0.0)
                + evaporative
        );
        summary["total_heat_rejected_kwh"] =
            json!(summary["total_heat_rejected_kwh"].as_f64().unwrap_or(0.0) + thermal_output);

        prior_energy = end_energy;
        prior_volume = volume_end;
        let interval_state = if interval_errors.is_empty() {
            "BALANCED_WITHIN_UNCERTAINTY"
        } else {
            "FAIL"
        };
        errors.extend(interval_errors.iter().map(|error| format!("{id}: {error}")));
        interval_reports.push(json!({
            "interval_id":id,
            "cover_mode":cover_mode,
            "thermal_energy_start_kwh":start_energy,
            "thermal_energy_end_kwh":end_energy,
            "thermal_balance_residual_kwh":thermal_residual,
            "thermal_balance_u95_kwh":thermal_u95,
            "water_volume_start_l":volume_start,
            "water_volume_end_l":volume_end,
            "water_balance_residual_l":water_residual,
            "water_balance_u95_l":water_u95,
            "evaporation_l":evaporation,
            "evaporative_latent_heat_rejected_kwh":evaporative,
            "expected_evaporative_latent_heat_kwh":expected_evaporative,
            "evaporation_latent_heat_residual_kwh":latent_residual,
            "pool_temperature_end_c":temperature_end,
            "maximum_layer_temperature_c":max_layer_temp,
            "interval_state":interval_state,
            "violations":interval_errors
        }));
    }

    let final_energy = number_at(
        intervals.last().unwrap_or(&Value::Null),
        "thermal_energy_end_kwh",
    );
    let cycle_error = final_energy.map(|value| (value - initial_energy).abs());
    if case.get("cycle_closure_required").and_then(Value::as_bool) == Some(true) {
        match cycle_error {
            Some(value)
                if value
                    <= number_at(case, "cycle_closure_tolerance_u95_kwh").unwrap_or(f64::NAN) => {}
            Some(value) => errors.push(format!(
                "pool thermal cycle closure error {value:.6} kWh exceeds tolerance"
            )),
            None => errors.push(
                "final pool thermal state is missing; cycle closure cannot be assessed".to_owned(),
            )
        }
    }

    let audit_state = if errors.is_empty() {
        "POOL_PASSIVE_REGENERATION_LEDGER_PASS"
    } else {
        "POOL_PASSIVE_REGENERATION_LEDGER_FAIL"
    };
    Ok(json!({
        "audit_state":audit_state,
        "study_id":"HIGHVELD-POOL-PASSIVE-REGENERATION-LEDGER-001",
        "case_id":text_at(case,"case_id").unwrap_or_default(),
        "interval_count":interval_reports.len(),
        "total_heat_input_kwh":total_heat_input,
        "total_longwave_heat_rejected_kwh":total_longwave,
        "total_sensible_heat_rejected_kwh":total_sensible,
        "total_evaporative_latent_heat_rejected_kwh":total_evaporative,
        "total_ground_heat_rejected_kwh":total_ground,
        "total_heat_exchanger_rejected_kwh":total_heat_exchanger,
        "total_evaporation_l":total_evaporation,
        "total_makeup_l":total_makeup,
        "total_rain_l":total_rain,
        "total_leakage_l":total_leakage,
        "total_withdrawal_l":total_withdrawal,
        "total_backwash_purge_l":total_backwash,
        "total_overflow_l":total_overflow,
        "initial_thermal_energy_kwh":initial_energy,
        "final_thermal_energy_kwh":final_energy,
        "thermal_cycle_closure_error_kwh":cycle_error,
        "thermal_cycle_closure_tolerance_u95_kwh":number_at(case,"cycle_closure_tolerance_u95_kwh"),
        "initial_water_volume_l":initial_volume,
        "final_water_volume_l":prior_volume,
        "maximum_layer_temperature_c":maximum_layer_temperature,
        "maximum_permitted_temperature_c":temperature_limit,
        "maximum_thermal_balance_residual_kwh":maximum_thermal_residual,
        "maximum_water_balance_residual_l":maximum_water_residual,
        "maximum_evaporation_latent_heat_residual_kwh":maximum_evaporation_latent_residual,
        "cover_mode_summary":cover_summary,
        "intervals":interval_reports,
        "violations":errors,
        "execution_status":"LEDGER_ONLY_NO_PHYSICAL_SIMULATION",
        "schema_engine":"NOT_RUN; schema declaration and semantic rules are checked by this Rust ledger",
        "claim_ceiling":"Synthetic conservation checks only; no actual pool cooling, water saving or physical model validation"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value =
                serde_json::from_str(CASE).expect("pool passive regeneration case JSON");
            let report = audit(&case).expect("pool passive regeneration case structure");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["audit_state"] != "POOL_PASSIVE_REGENERATION_LEDGER_PASS" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-pool-passive-regeneration-ledger");
            println!("  --self-test  audit synthetic pool thermal/water cycle");
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
    fn synthetic_pool_cycle_balances_heat_water_and_evaporation() {
        let report = audit(&sample()).expect("pool case evaluates");
        assert_eq!(
            report["audit_state"],
            "POOL_PASSIVE_REGENERATION_LEDGER_PASS"
        );
        assert_eq!(report["interval_count"], 3);
        assert_eq!(report["thermal_cycle_closure_error_kwh"], 0.0);
        assert_eq!(report["total_heat_input_kwh"], 13.0);
        assert_eq!(report["total_evaporation_l"], 4.1);
        assert_eq!(report["initial_water_volume_l"], 50000.0);
        assert_eq!(report["final_water_volume_l"], 49995.9);
    }

    #[test]
    fn energy_residual_outside_uncertainty_fails() {
        let mut case = sample();
        case["intervals"][0]["longwave_heat_rejected_kwh"] = json!(4.0);
        let report = audit(&case).unwrap();
        assert_eq!(
            report["audit_state"],
            "POOL_PASSIVE_REGENERATION_LEDGER_FAIL"
        );
        assert_eq!(report["intervals"][0]["interval_state"], "FAIL");
    }

    #[test]
    fn water_volume_residual_outside_uncertainty_fails() {
        let mut case = sample();
        case["intervals"][1]["water_volume_end_l"] = json!(49990.0);
        let report = audit(&case).unwrap();
        assert_eq!(report["audit_state"], "POOL_PASSIVE_REGENERATION_LEDGER_FAIL");
        assert_eq!(report["intervals"][1]["interval_state"], "FAIL");
    }

    #[test]
    fn evaporation_mass_and_latent_heat_must_agree() {
        let mut case = sample();
        case["intervals"][0]["evaporative_latent_heat_rejected_kwh"] = json!(3.0);
        let report = audit(&case).unwrap();
        assert_eq!(
            report["audit_state"],
            "POOL_PASSIVE_REGENERATION_LEDGER_FAIL"
        );
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| {
                    value
                        .as_str()
                        .unwrap_or("")
                        .contains("evaporation latent-heat cross-check residual")
                })
        );
    }

    #[test]
    fn pool_temperature_ceiling_is_owner_supplied_and_enforced() {
        let mut case = sample();
        case["intervals"][1]["maximum_layer_temperature_c"] = json!(29.0);
        let report = audit(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| {
                    value
                        .as_str()
                        .unwrap_or("")
                        .contains("exceeds owner/authority ceiling")
                })
        );
    }

    #[test]
    fn discretionary_makeup_without_permission_is_blocked() {
        let mut case = sample();
        case["intervals"][0]["makeup_l"] = json!(1.0);
        case["intervals"][0]["water_volume_end_l"] = json!(49998.0);
        case["intervals"][1]["water_volume_start_l"] = json!(49998.0);
        case["intervals"][1]["water_volume_end_l"] = json!(49997.9);
        case["intervals"][2]["water_volume_start_l"] = json!(49997.9);
        case["intervals"][2]["water_volume_end_l"] = json!(49996.9);
        let report = audit(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| {
                    value
                        .as_str()
                        .unwrap_or("")
                        .contains("lacks qualified rights, quality or discretionary permission")
                })
        );
    }

    #[test]
    fn missing_measurement_lineage_blocks_interval() {
        let mut case = sample();
        case["intervals"][2]["heat_flux_evidence_ref"] = Value::Null;
        let report = audit(&case).unwrap();
        assert_eq!(report["intervals"][2]["interval_state"], "FAIL");
    }

    #[test]
    fn thermal_cycle_drift_is_detected() {
        let mut case = sample();
        case["intervals"][2]["thermal_energy_end_kwh"] = json!(102.0);
        let report = audit(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| {
                    value
                        .as_str()
                        .unwrap_or("")
                        .contains("thermal cycle closure error")
                })
        );
    }
}
