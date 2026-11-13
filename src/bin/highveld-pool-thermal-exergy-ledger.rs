use serde_json::{Value, json};
use std::{collections::BTreeSet, env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_THERMAL_EXERGY_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_THERMAL_EXERGY_CASE_V0.1.schema.json"
);
const PARENT_CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_PASSIVE_REGENERATION_CASE_V0.1.json"
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

/// Specific exergy of a sensible-water layer relative to an environmental
/// reference state. Returns kWh. Temperatures are absolute Kelvin; the
/// ln_1p form improves numerical stability near thermal equilibrium.
fn close_state(left: f64, right: f64) -> bool {
    left.is_finite() && right.is_finite() && (left - right).abs() <= 1e-9
}

/// The exergy sidecar must describe the exact same water/thermal trajectory
/// that passed the first-law passive-regeneration ledger. This deliberately
/// does not compare environmental reference temperatures: those are the
/// additional boundary state used only for temperature-grade accounting.
fn validate_parent_ledger_link(case: &Value, parent: &Value) -> Vec<String> {
    let mut errors = Vec::new();

    if text_at(case, "parent_case_ref") != Some("POOL_PASSIVE_REGENERATION_CASE_V0.1.json") {
        errors.push(
            "parent_case_ref must identify the passive-regeneration ledger fixture".to_owned(),
        );
    }
    if text_at(parent, "study_id") != Some("HIGHVELD-POOL-PASSIVE-REGENERATION-LEDGER-001")
        || text_at(parent, "maturity") != Some("SYNTHETIC_KNOWN_ANSWER")
        || text_at(parent, "execution_status") != Some("NO_PHYSICAL_SIMULATION")
    {
        errors.push(
            "referenced parent must retain its expected first-law synthetic claim ceiling"
                .to_owned(),
        );
    }
    if text_at(case, "parent_case_id") != text_at(parent, "case_id") {
        errors.push("parent_case_id does not match the referenced first-law case".to_owned());
    }

    let parent_pool = &parent["pool"];
    for (case_key, parent_key, label) in [
        (
            "usable_thermal_energy_min_kwh",
            "usable_thermal_energy_min_kwh",
            "minimum usable store energy",
        ),
        (
            "usable_thermal_energy_capacity_kwh",
            "usable_thermal_energy_capacity_kwh",
            "usable store capacity",
        ),
        (
            "initial_thermal_energy_kwh",
            "initial_thermal_energy_kwh",
            "initial thermal energy",
        ),
        (
            "maximum_permitted_temperature_c",
            "maximum_permitted_temperature_c",
            "maximum permitted temperature",
        ),
    ] {
        match (
            number_at(case, case_key),
            number_at(parent_pool, parent_key),
        ) {
            (Some(left), Some(right)) if close_state(left, right) => {}
            _ => errors.push(format!(
                "exergy sidecar {label} does not match the first-law parent"
            )),
        }
    }
    if case.get("cycle_closure_required").and_then(Value::as_bool)
        != parent
            .get("cycle_closure_required")
            .and_then(Value::as_bool)
    {
        errors.push("cycle_closure_required does not match the first-law parent".to_owned());
    }
    match (
        number_at(case, "cycle_closure_tolerance_u95_kwh"),
        number_at(parent, "cycle_closure_tolerance_u95_kwh"),
    ) {
        (Some(left), Some(right)) if close_state(left, right) => {}
        _ => errors.push("cycle closure tolerance does not match the first-law parent".to_owned()),
    }

    let sidecar_intervals = case.get("intervals").and_then(Value::as_array);
    let parent_intervals = parent.get("intervals").and_then(Value::as_array);
    let (Some(sidecar_intervals), Some(parent_intervals)) = (sidecar_intervals, parent_intervals)
    else {
        errors.push("exergy sidecar or first-law parent intervals are missing".to_owned());
        return errors;
    };
    if sidecar_intervals.len() != parent_intervals.len() {
        errors.push("exergy interval count does not match the first-law parent".to_owned());
        return errors;
    }

    for (index, (sidecar, source)) in sidecar_intervals.iter().zip(parent_intervals).enumerate() {
        let sidecar_id = text_at(sidecar, "interval_id");
        let parent_id = text_at(source, "interval_id");
        let label = sidecar_id.unwrap_or("<missing-interval-id>");
        if sidecar_id.is_none() || sidecar_id != parent_id {
            errors.push(format!(
                "exergy interval {index} identity/order does not match first-law parent ({label})"
            ));
            continue;
        }
        for (sidecar_key, parent_key, quantity) in [
            (
                "thermal_energy_state_start_kwh",
                "thermal_energy_start_kwh",
                "opening thermal energy",
            ),
            (
                "thermal_energy_state_end_kwh",
                "thermal_energy_end_kwh",
                "closing thermal energy",
            ),
            (
                "water_volume_start_l",
                "water_volume_start_l",
                "opening water volume",
            ),
            (
                "water_volume_end_l",
                "water_volume_end_l",
                "closing water volume",
            ),
        ] {
            match (
                number_at(sidecar, sidecar_key),
                number_at(source, parent_key),
            ) {
                (Some(left), Some(right)) if close_state(left, right) => {}
                _ => errors.push(format!(
                    "{label}: exergy {quantity} does not match the first-law parent"
                )),
            }
        }
    }
    errors
}

fn sensible_exergy_kwh(
    mass_kg: f64,
    specific_heat_kj_kg_k: f64,
    temperature_c: f64,
    reference_temperature_c: f64,
) -> Option<f64> {
    if !mass_kg.is_finite()
        || !specific_heat_kj_kg_k.is_finite()
        || mass_kg < 0.0
        || specific_heat_kj_kg_k <= 0.0
    {
        return None;
    }
    let temperature_k = temperature_c + 273.15;
    let reference_k = reference_temperature_c + 273.15;
    if !temperature_k.is_finite()
        || !reference_k.is_finite()
        || temperature_k <= 0.0
        || reference_k <= 0.0
    {
        return None;
    }
    let relative_delta = (temperature_k - reference_k) / reference_k;
    if relative_delta <= -1.0 {
        return None;
    }
    let dimensionless = relative_delta - relative_delta.ln_1p();
    if !dimensionless.is_finite() || dimensionless < -1e-12 {
        return None;
    }
    Some((mass_kg * specific_heat_kj_kg_k * reference_k * dimensionless.max(0.0)) / 3600.0)
}

fn exergy_by_grade(
    mass_kg: f64,
    specific_heat_kj_kg_k: f64,
    temperature_c: f64,
    reference_temperature_c: f64,
) -> Option<(f64, f64)> {
    let exergy = sensible_exergy_kwh(
        mass_kg,
        specific_heat_kj_kg_k,
        temperature_c,
        reference_temperature_c,
    )?;
    if temperature_c < reference_temperature_c {
        Some((exergy, 0.0))
    } else if temperature_c > reference_temperature_c {
        Some((0.0, exergy))
    } else {
        Some((0.0, 0.0))
    }
}

fn sensible_headroom_kwh(
    mass_kg: f64,
    specific_heat_kj_kg_k: f64,
    temperature_c: f64,
    maximum_temperature_c: f64,
) -> Option<f64> {
    if !mass_kg.is_finite()
        || !specific_heat_kj_kg_k.is_finite()
        || mass_kg < 0.0
        || specific_heat_kj_kg_k <= 0.0
        || !temperature_c.is_finite()
        || !maximum_temperature_c.is_finite()
    {
        return None;
    }
    // A layer over the authority ceiling has zero positive headroom, but remains
    // reportable so the interval can fail on the explicit temperature-limit gate.
    Some(
        mass_kg * specific_heat_kj_kg_k * (maximum_temperature_c - temperature_c).max(0.0) / 3600.0,
    )
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let parent: Value = serde_json::from_str(PARENT_CASE)
        .map_err(|error| vec![format!("parent first-law case JSON parse failed: {error}")])?;
    errors.extend(validate_parent_ledger_link(case, &parent));
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("exergy schema JSON parse failed: {error}")])?;
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("exergy schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-POOL-THERMAL-EXERGY-001"),
        (
            "parent_study",
            "HIGHVELD-POOL-PASSIVE-REGENERATION-LEDGER-001",
        ),
        ("maturity", "SYNTHETIC_KNOWN_ANSWER"),
        ("execution_status", "NO_PHYSICAL_SIMULATION"),
    ] {
        if text_at(case, key) != Some(expected) {
            errors.push(format!("{key} must remain {expected}"));
        }
    }

    let property = &case["property_model"];
    let cp = number_at(property, "specific_heat_kj_kg_k").unwrap_or(f64::NAN);
    let density = number_at(property, "water_density_kg_l").unwrap_or(f64::NAN);
    let property_u95 = number_at(property, "property_uncertainty_u95_kwh").unwrap_or(f64::NAN);
    if !cp.is_finite()
        || cp <= 0.0
        || !density.is_finite()
        || density <= 0.0
        || !property_u95.is_finite()
        || property_u95 < 0.0
        || text_at(property, "specific_heat_method_ref").is_none()
        || text_at(property, "density_method_ref").is_none()
        || text_at(property, "property_evidence_ref").is_none()
    {
        errors.push("water property inputs/method/evidence are invalid".to_owned());
    }

    let maximum_temperature =
        number_at(case, "maximum_permitted_temperature_c").ok_or_else(|| {
            vec!["maximum_permitted_temperature_c is missing or non-finite".to_owned()]
        })?;
    if maximum_temperature <= -273.15 || text_at(case, "temperature_limit_authority_ref").is_none()
    {
        errors.push(
            "owner/authority maximum temperature limit is invalid or unreferenced".to_owned(),
        );
    }

    let intervals = case
        .get("intervals")
        .and_then(Value::as_array)
        .ok_or_else(|| vec!["intervals must be an array".to_owned()])?;
    if intervals.is_empty() {
        errors.push("at least one exergy interval is required".to_owned());
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let required_layers = ["TOP", "MID", "BOTTOM"];
    let mut seen_interval_ids = BTreeSet::new();
    let mut reports = Vec::new();
    let mut total_start_cold_exergy = 0.0_f64;
    let mut total_end_cold_exergy = 0.0_f64;
    let mut total_start_hot_exergy = 0.0_f64;
    let mut total_end_hot_exergy = 0.0_f64;
    let mut total_start_headroom = 0.0_f64;
    let mut total_end_headroom = 0.0_f64;
    let mut maximum_uncertainty = 0.0_f64;

    for interval in intervals {
        let id = text_at(interval, "interval_id").unwrap_or("<missing-interval-id>");
        let mut interval_errors = Vec::new();
        if !seen_interval_ids.insert(id.to_owned()) {
            interval_errors.push("duplicate interval ID".to_owned());
        }

        let reference_temperature = number_at(interval, "environmental_reference_temperature_c");
        let volume_start = number_at(interval, "water_volume_start_l");
        let volume_end = number_at(interval, "water_volume_end_l");
        let uncertainty = number_at(interval, "exergy_uncertainty_u95_kwh");
        for (name, reference) in [
            (
                "environmental_reference_method_ref",
                "environmental reference method",
            ),
            ("layer_state_evidence_ref", "layer state evidence"),
            ("uncertainty_evidence_ref", "exergy uncertainty evidence"),
        ] {
            if text_at(interval, name).is_none() {
                interval_errors.push(format!("missing {reference}"));
            }
        }
        if reference_temperature.is_none()
            || volume_start.is_none()
            || volume_end.is_none()
            || uncertainty.is_none()
        {
            interval_errors
                .push("non-finite or missing interval reference/volume/uncertainty".to_owned());
        }
        if !interval_errors.is_empty() {
            errors.extend(interval_errors.iter().map(|error| format!("{id}: {error}")));
            reports.push(json!({
                "interval_id":id,
                "interval_state":"FAIL_MISSING_OR_INVALID_INPUT",
                "errors":interval_errors
            }));
            continue;
        }

        let reference_temperature = reference_temperature.unwrap();
        let volume_start = volume_start.unwrap();
        let volume_end = volume_end.unwrap();
        let uncertainty = uncertainty.unwrap();
        maximum_uncertainty = maximum_uncertainty.max(uncertainty + property_u95);
        if reference_temperature <= -273.15
            || volume_start <= 0.0
            || volume_end <= 0.0
            || uncertainty < 0.0
        {
            interval_errors.push(
                "environment reference, water volume or uncertainty is outside valid bounds"
                    .to_owned(),
            );
        }

        let layers = interval.get("layers").and_then(Value::as_array);
        let Some(layers) = layers else {
            interval_errors.push("layers must be an array".to_owned());
            errors.extend(interval_errors.iter().map(|error| format!("{id}: {error}")));
            reports.push(json!({
                "interval_id":id,
                "interval_state":"FAIL_INVALID_LAYERS",
                "errors":interval_errors
            }));
            continue;
        };

        let mut layer_ids = BTreeSet::new();
        let mut fraction_start = 0.0_f64;
        let mut fraction_end = 0.0_f64;
        let mut start_cold = 0.0_f64;
        let mut end_cold = 0.0_f64;
        let mut start_hot = 0.0_f64;
        let mut end_hot = 0.0_f64;
        let mut start_headroom = 0.0_f64;
        let mut end_headroom = 0.0_f64;
        let mut layer_reports = Vec::new();

        for layer in layers {
            let layer_id = text_at(layer, "layer_id").unwrap_or("<missing-layer-id>");
            if !layer_ids.insert(layer_id.to_owned()) {
                interval_errors.push(format!("duplicate layer ID {layer_id}"));
            }
            if !required_layers.contains(&layer_id) {
                interval_errors.push(format!("unsupported water layer {layer_id}"));
            }
            let start_fraction = number_at(layer, "mass_fraction_start");
            let end_fraction = number_at(layer, "mass_fraction_end");
            let start_temperature = number_at(layer, "temperature_start_c");
            let end_temperature = number_at(layer, "temperature_end_c");
            if start_fraction.is_none()
                || end_fraction.is_none()
                || start_temperature.is_none()
                || end_temperature.is_none()
            {
                interval_errors.push(format!(
                    "{layer_id}: layer fraction/temperature is missing or non-finite"
                ));
                continue;
            }
            let start_fraction = start_fraction.unwrap();
            let end_fraction = end_fraction.unwrap();
            let start_temperature = start_temperature.unwrap();
            let end_temperature = end_temperature.unwrap();
            if start_fraction <= 0.0
                || end_fraction <= 0.0
                || start_fraction > 1.0
                || end_fraction > 1.0
                || start_temperature <= -273.15
                || end_temperature <= -273.15
            {
                interval_errors.push(format!(
                    "{layer_id}: layer fraction or absolute temperature is invalid"
                ));
                continue;
            }
            if start_temperature > maximum_temperature || end_temperature > maximum_temperature {
                interval_errors.push(format!(
                    "{layer_id}: layer exceeds owner/authority temperature ceiling"
                ));
            }
            fraction_start += start_fraction;
            fraction_end += end_fraction;

            let start_mass = volume_start * density * start_fraction;
            let end_mass = volume_end * density * end_fraction;
            let (start_cold_layer, start_hot_layer) =
                exergy_by_grade(start_mass, cp, start_temperature, reference_temperature)
                    .ok_or_else(|| {
                        vec![format!(
                            "{id}/{layer_id}: start exergy cannot be calculated"
                        )]
                    })?;
            let (end_cold_layer, end_hot_layer) =
                exergy_by_grade(end_mass, cp, end_temperature, reference_temperature).ok_or_else(
                    || vec![format!("{id}/{layer_id}: end exergy cannot be calculated")],
                )?;
            let start_layer_headroom =
                sensible_headroom_kwh(start_mass, cp, start_temperature, maximum_temperature)
                    .ok_or_else(|| {
                        vec![format!(
                            "{id}/{layer_id}: start headroom cannot be calculated"
                        )]
                    })?;
            let end_layer_headroom =
                sensible_headroom_kwh(end_mass, cp, end_temperature, maximum_temperature)
                    .ok_or_else(|| {
                        vec![format!(
                            "{id}/{layer_id}: end headroom cannot be calculated"
                        )]
                    })?;

            start_cold += start_cold_layer;
            end_cold += end_cold_layer;
            start_hot += start_hot_layer;
            end_hot += end_hot_layer;
            start_headroom += start_layer_headroom;
            end_headroom += end_layer_headroom;
            layer_reports.push(json!({
                "layer_id":layer_id,
                "start_water_mass_kg":start_mass,
                "end_water_mass_kg":end_mass,
                "start_temperature_c":start_temperature,
                "end_temperature_c":end_temperature,
                "start_temperature_limit_exceedance_c":(start_temperature - maximum_temperature).max(0.0),
                "end_temperature_limit_exceedance_c":(end_temperature - maximum_temperature).max(0.0),
                "start_cold_exergy_kwh":start_cold_layer,
                "end_cold_exergy_kwh":end_cold_layer,
                "start_hot_exergy_kwh":start_hot_layer,
                "end_hot_exergy_kwh":end_hot_layer,
                "start_idealized_heat_headroom_kwh":start_layer_headroom,
                "end_idealized_heat_headroom_kwh":end_layer_headroom
            }));
        }

        if layer_ids.len() != 3
            || !required_layers
                .iter()
                .all(|required| layer_ids.contains(*required))
        {
            interval_errors.push("exact TOP/MID/BOTTOM layer records are required".to_owned());
        }
        if (fraction_start - 1.0).abs() > 1e-6 || (fraction_end - 1.0).abs() > 1e-6 {
            interval_errors.push(format!(
                "layer mass fractions must close to one (start={fraction_start:.8}, end={fraction_end:.8})"
            ));
        }
        if !interval_errors.is_empty() {
            errors.extend(interval_errors.iter().map(|error| format!("{id}: {error}")));
        }

        total_start_cold_exergy += start_cold;
        total_end_cold_exergy += end_cold;
        total_start_hot_exergy += start_hot;
        total_end_hot_exergy += end_hot;
        total_start_headroom += start_headroom;
        total_end_headroom += end_headroom;

        reports.push(json!({
            "interval_id":id,
            "environmental_reference_temperature_c":reference_temperature,
            "start_cold_exergy_kwh":start_cold,
            "end_cold_exergy_kwh":end_cold,
            "cold_exergy_change_kwh":end_cold-start_cold,
            "start_hot_exergy_kwh":start_hot,
            "end_hot_exergy_kwh":end_hot,
            "hot_exergy_change_kwh":end_hot-start_hot,
            "start_idealized_sensible_heat_headroom_kwh":start_headroom,
            "end_idealized_sensible_heat_headroom_kwh":end_headroom,
            "exergy_u95_kwh":uncertainty+property_u95,
            "layers":layer_reports,
            "interval_state":if interval_errors.is_empty() {"EXERGY_ACCOUNT_COMPLETE"} else {"FAIL"},
            "errors":interval_errors
        }));
    }

    let state = if errors.is_empty() {
        "POOL_EXERGY_ACCOUNT_PASS"
    } else {
        "POOL_EXERGY_ACCOUNT_FAIL"
    };
    Ok(json!({
        "audit_state":state,
        "study_id":"HIGHVELD-POOL-THERMAL-EXERGY-001",
        "case_id":text_at(case,"case_id").unwrap_or_default(),
        "parent_case_id":text_at(case,"parent_case_id").unwrap_or_default(),
        "parent_ledger_cross_check_state":"PASS",
        "parent_ledger_cross_check_scope":"INTERVAL_THERMAL_ENERGY_AND_WATER_VOLUME_PLUS_STORE_AND_CYCLE_POLICY",
        "interval_count":reports.len(),
        "total_start_cold_exergy_kwh":total_start_cold_exergy,
        "total_end_cold_exergy_kwh":total_end_cold_exergy,
        "total_start_hot_exergy_kwh":total_start_hot_exergy,
        "total_end_hot_exergy_kwh":total_end_hot_exergy,
        "total_start_idealized_sensible_heat_headroom_kwh":total_start_headroom,
        "total_end_idealized_sensible_heat_headroom_kwh":total_end_headroom,
        "maximum_exergy_u95_kwh":maximum_uncertainty,
        "intervals":reports,
        "errors":errors,
        "execution_status":"EXERGY_ACCOUNTING_ONLY_NO_PHYSICAL_VALIDATION",
        "schema_engine":"NOT_RUN; schema declaration and semantic invariants are checked by this Rust ledger",
        "claim_ceiling":"Temperature-grade accounting from supplied layer state and property method only; not deliverable cooling or physical validation"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value = serde_json::from_str(CASE).expect("pool exergy case JSON");
            let report = evaluate(&case).expect("pool exergy case structure");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["audit_state"] != "POOL_EXERGY_ACCOUNT_PASS" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-pool-thermal-exergy-ledger");
            println!("  --self-test  audit synthetic layered-water exergy and heat headroom");
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
        serde_json::from_str(CASE).expect("pool exergy case JSON")
    }

    #[test]
    fn parent_first_law_trajectory_matches_exergy_sidecar() {
        let case = sample();
        let parent: Value = serde_json::from_str(PARENT_CASE).expect("parent first-law JSON");
        let errors = validate_parent_ledger_link(&case, &parent);
        assert!(
            errors.is_empty(),
            "unexpected cross-ledger mismatches: {errors:?}"
        );
        let report = evaluate(&case).expect("linked exergy case qualifies");
        assert_eq!(report["parent_ledger_cross_check_state"], "PASS");
        assert_eq!(
            report["parent_case_id"],
            "SYNTHETIC_THREE_INTERVAL_POOL_BUFFER_CYCLE_001"
        );
    }

    #[test]
    fn parent_thermal_energy_state_mismatch_is_rejected() {
        let mut case = sample();
        case["intervals"][1]["thermal_energy_state_end_kwh"] = json!(100.5);
        let parent: Value = serde_json::from_str(PARENT_CASE).expect("parent first-law JSON");
        let errors = validate_parent_ledger_link(&case, &parent);
        assert!(errors.iter().any(|error| {
            error.contains("closing thermal energy does not match the first-law parent")
        }));
    }

    #[test]
    fn parent_water_volume_mismatch_is_rejected() {
        let mut case = sample();
        case["intervals"][1]["water_volume_end_l"] = json!(49996.8);
        let parent: Value = serde_json::from_str(PARENT_CASE).expect("parent first-law JSON");
        let errors = validate_parent_ledger_link(&case, &parent);
        assert!(errors.iter().any(|error| {
            error.contains("closing water volume does not match the first-law parent")
        }));
    }

    #[test]
    fn parent_case_identity_and_cycle_policy_must_match() {
        let mut case = sample();
        let parent: Value = serde_json::from_str(PARENT_CASE).expect("parent first-law JSON");
        case["parent_case_id"] = json!("UNRELATED_POOL_CASE");
        case["cycle_closure_tolerance_u95_kwh"] = json!(0.1);
        let errors = validate_parent_ledger_link(&case, &parent);
        assert!(errors.iter().any(|error| error.contains("parent_case_id")));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cycle closure tolerance"))
        );
    }

    #[test]
    fn hot_weather_layers_report_cold_exergy_and_cool_night_reports_hot_exergy() {
        let report = evaluate(&sample()).expect("synthetic exergy case qualifies");
        assert_eq!(report["audit_state"], "POOL_EXERGY_ACCOUNT_PASS");
        assert!(
            report["intervals"][0]["start_cold_exergy_kwh"]
                .as_f64()
                .unwrap()
                > 0.0
        );
        assert_eq!(report["intervals"][0]["start_hot_exergy_kwh"], 0.0);
        assert!(
            report["intervals"][2]["start_hot_exergy_kwh"]
                .as_f64()
                .unwrap()
                > 0.0
        );
        assert_eq!(report["intervals"][2]["start_cold_exergy_kwh"], 0.0);
    }

    #[test]
    fn warmer_water_closer_to_ambient_has_less_cold_exergy() {
        let cold = sensible_exergy_kwh(1000.0, 4.18, 25.0, 32.0).unwrap();
        let warmer = sensible_exergy_kwh(1000.0, 4.18, 27.0, 32.0).unwrap();
        assert!(cold > warmer);
    }

    #[test]
    fn equal_water_mass_and_heat_capacity_do_not_imply_equal_exergy() {
        let ambient = 32.0;
        let warm_pool = sensible_exergy_kwh(5000.0, 4.18, 25.0, ambient).unwrap();
        let near_ambient_pool = sensible_exergy_kwh(5000.0, 4.18, 30.0, ambient).unwrap();
        assert!(warm_pool > near_ambient_pool);
    }

    #[test]
    fn mass_fractions_must_close_for_each_layered_state() {
        let mut case = sample();
        case["intervals"][0]["layers"][0]["mass_fraction_start"] = json!(0.1);
        let report = evaluate(&case).expect("mass fraction gap is a reportable validation failure");
        assert_eq!(report["audit_state"], "POOL_EXERGY_ACCOUNT_FAIL");
        assert!(report["errors"].as_array().unwrap().iter().any(|error| {
            error
                .as_str()
                .unwrap_or("")
                .contains("layer mass fractions must close to one")
        }));
    }

    #[test]
    fn layer_above_temperature_limit_is_rejected() {
        let mut case = sample();
        case["intervals"][1]["layers"][0]["temperature_end_c"] = json!(29.0);
        let report = evaluate(&case).expect("limit breach is a reportable validation failure");
        assert_eq!(report["audit_state"], "POOL_EXERGY_ACCOUNT_FAIL");
        assert_eq!(report["intervals"][1]["interval_state"], "FAIL");
        assert_eq!(
            report["intervals"][1]["layers"][0]["end_temperature_limit_exceedance_c"],
            4.0
        );
        assert_eq!(
            report["intervals"][1]["layers"][0]["end_idealized_heat_headroom_kwh"],
            0.0
        );
        assert!(report["errors"].as_array().unwrap().iter().any(|error| {
            error
                .as_str()
                .unwrap_or("")
                .contains("exceeds owner/authority temperature ceiling")
        }));
    }

    #[test]
    fn numerical_exergy_is_zero_at_the_reference_temperature() {
        let value = sensible_exergy_kwh(5000.0, 4.18, 32.0, 32.0).unwrap();
        assert_eq!(value, 0.0);
    }
}
