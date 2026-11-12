use serde_json::{Value, json};
use std::{env, process};

const MANIFEST: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/THERMAL_SINK_CASCADE_MANIFEST_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/THERMAL_SINK_CASCADE_MANIFEST_V0.1.schema.json"
);
const RADIATIVE_BENCHMARK: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/RADIATIVE_SKY_SINK_BENCHMARK_MANIFEST_V0.1.json"
);
const RADIATIVE_SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/RADIATIVE_SKY_SINK_BENCHMARK_MANIFEST_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn contains_id(items: &[Value], expected: &str) -> bool {
    items
        .iter()
        .any(|item| item.as_str() == Some(expected) || text_at(item, "id") == Some(expected))
}

fn build_inventory() -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let manifest: Value = match serde_json::from_str(MANIFEST) {
        Ok(value) => value,
        Err(error) => return Err(vec![format!("manifest JSON parse failed: {error}")]),
    };
    let schema: Value = match serde_json::from_str(SCHEMA) {
        Ok(value) => value,
        Err(error) => return Err(vec![format!("schema JSON parse failed: {error}")]),
    };
    let radiative: Value = match serde_json::from_str(RADIATIVE_BENCHMARK) {
        Ok(value) => value,
        Err(error) => return Err(vec![format!("radiative benchmark JSON parse failed: {error}")]),
    };
    let radiative_schema: Value = match serde_json::from_str(RADIATIVE_SCHEMA) {
        Ok(value) => value,
        Err(error) => return Err(vec![format!("radiative schema JSON parse failed: {error}")]),
    };

    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(&radiative_schema, "$schema")
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        errors.push("radiative benchmark schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-RADIATIVE-SKY-SINK-001"),
        ("parent_study", "HIGHVELD-THERMAL-SINK-CASCADE-001"),
        ("maturity", "BENCHMARK_PROTOCOL_NOT_EXECUTED"),
        ("execution_status", "NOT_EXECUTED"),
    ] {
        if text_at(&radiative, key) != Some(expected) {
            errors.push(format!("radiative benchmark {key} must remain {expected}"));
        }
    }
    let radiative_arms = array_at(&radiative, "comparison_arms")
        .cloned()
        .unwrap_or_default();
    let radiative_scenarios = array_at(&radiative, "scenarios")
        .cloned()
        .unwrap_or_default();
    if radiative_arms.len() != 7 || radiative_scenarios.len() != 7 {
        errors.push("radiative benchmark must preserve its 7-arm by 7-scenario matrix".to_owned());
    }
    for required in [
        "R0_NO_ADDED_RADIATIVE_PANEL",
        "R1_REFERENCE_PANEL",
        "R2_CANDIDATE_PANEL",
        "R3_PANEL_PLUS_THERMAL_STORE",
        "R4_LAYERED_SINK_CASCADE",
        "R5_SOLAR_ABSORBING_NEGATIVE_CONTROL",
        "R6_INFINITE_SKY_NEGATIVE_CONTROL",
    ] {
        if !contains_id(&radiative_arms, required) {
            errors.push(format!("radiative benchmark arm {required} is missing"));
        }
    }
    for required in [
        "spectral_solar_reflectance",
        "spectral_longwave_emittance",
        "downwelling_longwave_or_qualified_sky_boundary",
        "sky_view_factor_and_horizon",
        "loop_flow_and_inlet_outlet_temperature",
        "occupied_zone_comfort_when_claimed",
    ] {
        let measurements = array_at(&radiative, "required_measurements")
            .cloned()
            .unwrap_or_default();
        if !contains_id(&measurements, required) {
            errors.push(format!("radiative measurement {required} is missing"));
        }
    }
    let radiative_gates = &radiative["hard_gates"];
    for key in [
        "material_to_building_claim_leakage",
        "useful_cooling_without_zone_path_evidence",
        "missing_radiative_boundary_claimed_as_valid",
        "ignored_soiling_or_wetness_state",
        "accepted_infinite_sky_assumption",
        "unclosed_component_balance_outside_uncertainty",
        "water_floor_violations",
        "unknown_out_of_domain_as_pass",
    ] {
        if radiative_gates.get(key).and_then(Value::as_u64) != Some(0) {
            errors.push(format!("radiative hard gate {key} must be exactly zero"));
        }
    }
    for (key, expected) in [
        ("format_version", "0.1"),
        ("study_id", "HIGHVELD-THERMAL-SINK-CASCADE-001"),
        ("parent_study", "HIGHVELD-PASSIVE-COOLING-WATER-001"),
        ("parent_candidate", "NEIGHBORHOOD_RESILIENCE_COMMONS"),
        ("maturity", "PROTOCOL_ONLY_NOT_EXECUTED"),
        ("climate_context", "JOHANNESBURG_HIGHVELD"),
        ("execution_status", "NOT_EXECUTED"),
    ] {
        if text_at(&manifest, key) != Some(expected) {
            errors.push(format!("{key} must be {expected}"));
        }
    }

    let arms = array_at(&manifest, "architecture_arms")
        .cloned()
        .unwrap_or_default();
    let expected_arms = [
        "NO_WATER_PASSIVE_BASELINE",
        "WATER_STORE_NO_REGENERATION_NEGATIVE_CONTROL",
        "WATER_STORE_RADIATIVE_SKY_REGENERATION",
        "WATER_STORE_DRY_AIR_REGENERATION",
        "WATER_STORE_GROUND_LOOP_REGENERATION",
        "WATER_STORE_CONDITIONAL_EVAPORATIVE_REGENERATION",
        "WATER_STORE_LAYERED_SINK_CASCADE",
        "INFINITE_SINK_ASSUMPTION_NEGATIVE_CONTROL",
    ];
    if arms.len() != expected_arms.len() {
        errors.push("exactly eight architecture arms are required".to_owned());
    }
    for expected in expected_arms {
        if !contains_id(&arms, expected) {
            errors.push(format!("missing architecture arm {expected}"));
        }
    }

    let primary = arms.iter().any(|arm| {
        text_at(arm, "id") == Some("NO_WATER_PASSIVE_BASELINE")
            && text_at(arm, "role") == Some("PRIMARY_CONTROL")
    });
    if !primary {
        errors.push("no-water passive baseline must be the primary control".to_owned());
    }
    let infinite_sink = arms.iter().any(|arm| {
        text_at(arm, "id") == Some("INFINITE_SINK_ASSUMPTION_NEGATIVE_CONTROL")
            && text_at(arm, "role") == Some("NEGATIVE_CONTROL")
    });
    if !infinite_sink {
        errors.push(
            "infinite-sink assumption must be registered as a negative control".to_owned(),
        );
    }

    let sink_paths = array_at(&manifest, "sink_path_registry")
        .cloned()
        .unwrap_or_default();
    for required in [
        "RADIATIVE_SKY_PANEL",
        "DRY_AIR_COOLER",
        "GROUND_HEAT_EXCHANGER",
        "CONDITIONAL_EVAPORATIVE_REGENERATION",
        "PASSIVE_LOAD_REDUCTION",
        "INFINITE_SINK_INVALID",
    ] {
        if !contains_id(&sink_paths, required) {
            errors.push(format!("missing sink path {required}"));
        }
    }

    let scenarios = array_at(&manifest, "scenarios")
        .cloned()
        .unwrap_or_default();
    if scenarios.len() != 9 {
        errors.push("exactly nine stress scenarios are required".to_owned());
    }
    let frozen = array_at(&manifest, "frozen_inputs_required")
        .cloned()
        .unwrap_or_default();
    if frozen.len() < 10 {
        errors.push("at least ten frozen input digests/identifiers are required".to_owned());
    }
    for required in [
        "weather_package_digest",
        "thermal_load_profile_digest",
        "store_initial_state_and_stratification_digest",
        "water_source_rights_quality_and_reserve_digest",
        "independent_energy_water_balance_oracle_id",
    ] {
        if !contains_id(&frozen, required) {
            errors.push(format!("missing frozen input {required}"));
        }
    }

    let gates = &manifest["hard_gates"];
    for required in [
        "essential_water_floor_violations",
        "unqualified_water_operation",
        "unmodelled_heat_or_water_terms",
        "useful_cooling_without_occupied_zone_evidence",
        "accepted_infinite_sink_assumptions",
        "missed_seeded_sensor_or_equipment_faults",
        "unknown_state_as_pass",
        "positive_control_false_blocks",
    ] {
        if gates.get(required).and_then(Value::as_u64) != Some(0) {
            errors.push(format!("hard gate {required} must be exactly zero"));
        }
    }
    if gates
        .get("energy_and_water_balance_within_propagated_uncertainty")
        .and_then(Value::as_bool)
        != Some(true)
    {
        errors.push("energy and water balance closure is mandatory".to_owned());
    }
    if gates
        .get("seasonal_store_cycle_closure_required")
        .and_then(Value::as_bool)
        != Some(true)
    {
        errors.push("seasonal store-cycle closure is mandatory".to_owned());
    }
    if gates
        .get("exact_input_model_controller_instrument_traceability_percent")
        .and_then(Value::as_u64)
        != Some(100)
    {
        errors
            .push("input/model/controller/instrument traceability target must be 100%".to_owned());
    }

    let control = &manifest["control_layers"];
    if text_at(control, "independent_feasibility_governor") != Some("DETERMINISTIC_FAIL_CLOSED") {
        errors.push("independent feasibility governor must fail closed".to_owned());
    }
    if text_at(control, "optimizer") != Some("BOUNDED_MULTI_OBJECTIVE_PARETO_DISPATCH") {
        errors.push("optimizer must remain bounded multi-objective Pareto dispatch".to_owned());
    }
    if text_at(control, "future_information") != Some("PROHIBITED") {
        errors.push("future information must be prohibited".to_owned());
    }
    if text_at(control, "human_authority_for_discretionary_water") != Some("REQUIRED") {
        errors.push("human authority is required for discretionary water".to_owned());
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut cases = Vec::new();
    for arm in &arms {
        let arm_id = text_at(arm, "id").unwrap_or_default();
        for scenario in &scenarios {
            let scenario_id = scenario.as_str().unwrap_or_default();
            cases.push(json!({
                "case_id": format!("{arm_id}__{scenario_id}"),
                "architecture_arm": arm_id,
                "scenario": scenario_id,
                "case_state": "REGISTERED_NOT_EXECUTED",
                "execution_status": "NOT_EXECUTED"
            }));
        }
    }

    let expected = arms.len() * scenarios.len();
    if expected != 72 || cases.len() != expected {
        return Err(vec![format!(
            "case expansion mismatch: expected 72, expanded {}",
            cases.len()
        )]);
    }

    Ok(json!({
        "audit_state": "STRUCTURAL_CONTRACT_PASS",
        "study_id": "HIGHVELD-THERMAL-SINK-CASCADE-001",
        "maturity": "PROTOCOL_ONLY_NOT_EXECUTED",
        "architecture_arm_count": arms.len(),
        "scenario_count": scenarios.len(),
        "case_count": cases.len(),
        "radiative_sky_benchmark_study_id": "HIGHVELD-RADIATIVE-SKY-SINK-001",
        "radiative_sky_arm_count": radiative_arms.len(),
        "radiative_sky_scenario_count": radiative_scenarios.len(),
        "radiative_sky_case_count": radiative_arms.len() * radiative_scenarios.len(),
        "execution_status": "NOT_EXECUTED",
        "schema_engine": "NOT_RUN; schema declaration and manifest semantics are checked by this Rust qualifier",
        "claim_ceiling": "No physical heat rejection, cooling, water saving, cost or resilience superiority is claimed",
        "cases": cases
    }))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str).unwrap_or("--help") {
        "--self-test" | "--plan" => match build_inventory() {
            Ok(report) => println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            ),
            Err(errors) => {
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "audit_state": "STRUCTURAL_CONTRACT_FAIL",
                        "errors": errors,
                        "execution_status": "NOT_EXECUTED"
                    }))
                    .unwrap_or_default()
                );
                process::exit(1);
            }
        },
        "--help" | "-h" => {
            println!("highveld-thermal-sink-cascade-qualifier");
            println!("  --self-test  validate the manifest and emit its 72-case inventory");
            println!("  --plan       alias for --self-test; does not execute simulations");
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

    #[test]
    fn expands_eight_arms_and_nine_scenarios() {
        let report = build_inventory().expect("thermal sink cascade manifest must qualify");
        assert_eq!(report["architecture_arm_count"].as_u64(), Some(8));
        assert_eq!(report["scenario_count"].as_u64(), Some(9));
        assert_eq!(report["case_count"].as_u64(), Some(72));
    }

    #[test]
    fn radiative_sky_benchmark_is_linked_and_not_executed() {
        let report = build_inventory().expect("radiative benchmark must qualify");
        assert_eq!(
            report["radiative_sky_benchmark_study_id"].as_str(),
            Some("HIGHVELD-RADIATIVE-SKY-SINK-001")
        );
        assert_eq!(report["radiative_sky_arm_count"].as_u64(), Some(7));
        assert_eq!(report["radiative_sky_scenario_count"].as_u64(), Some(7));
        assert_eq!(report["radiative_sky_case_count"].as_u64(), Some(49));
        assert_eq!(report["execution_status"].as_str(), Some("NOT_EXECUTED"));
    }

    #[test]
    fn pool_is_treated_as_buffer_and_not_an_infinite_sink() {
        let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest JSON");
        let arms = manifest["architecture_arms"].as_array().expect("arms");
        let pool = arms
            .iter()
            .find(|arm| arm["id"] == "WATER_STORE_NO_REGENERATION_NEGATIVE_CONTROL")
            .expect("no-regeneration negative control");
        assert_eq!(pool["store_role"].as_str(), Some("WATER_BUFFER_ONLY"));
        let invalid = arms
            .iter()
            .find(|arm| arm["id"] == "INFINITE_SINK_ASSUMPTION_NEGATIVE_CONTROL")
            .expect("infinite sink negative control");
        assert_eq!(invalid["role"].as_str(), Some("NEGATIVE_CONTROL"));
    }

    #[test]
    fn no_future_information_and_water_rights_governor_are_mandatory() {
        let report = build_inventory().expect("manifest must qualify");
        assert_eq!(
            report["maturity"].as_str(),
            Some("PROTOCOL_ONLY_NOT_EXECUTED")
        );
        assert_eq!(report["execution_status"].as_str(), Some("NOT_EXECUTED"));
        assert!(
            report["cases"]
                .as_array()
                .expect("case inventory")
                .iter()
                .all(|case| case["execution_status"] == "NOT_EXECUTED")
        );
    }

    #[test]
    fn radiative_sink_requires_real_boundary_evidence() {
        let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest JSON");
        let paths = manifest["sink_path_registry"]
            .as_array()
            .expect("sink registry");
        let path = paths
            .iter()
            .find(|item| item["id"] == "RADIATIVE_SKY_PANEL")
            .expect("radiative sink");
        for required in [
            "SKY_VIEW_FACTOR",
            "LONGWAVE_RADIATION",
            "CLOUD_STATE",
            "LOOP_INLET_OUTLET_TEMPERATURE",
        ] {
            assert!(
                path["evidence_required"]
                    .as_array()
                    .expect("radiative evidence list")
                    .iter()
                    .any(|item| item == required)
            );
        }
    }
}
