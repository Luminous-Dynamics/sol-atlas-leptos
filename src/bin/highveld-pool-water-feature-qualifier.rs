use serde_json::{Value, json};
use std::{collections::BTreeSet, env, process};

const MANIFEST: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_WATER_FEATURE_THERMAL_MANIFEST_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_WATER_FEATURE_THERMAL_MANIFEST_V0.1.schema.json"
);
const MICROCLIMATE_BENCHMARK: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_MICROCLIMATE_ATTRIBUTION_MANIFEST_V0.1.json"
);
const MICROCLIMATE_SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_MICROCLIMATE_ATTRIBUTION_MANIFEST_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn unique_ids(items: &[Value]) -> bool {
    let mut seen = BTreeSet::new();
    items
        .iter()
        .all(|item| text_at(item, "id").is_some_and(|id| seen.insert(id.to_owned())))
}

fn contains_id(items: &[Value], id: &str) -> bool {
    items.iter().any(|item| text_at(item, "id") == Some(id))
}

fn audit() -> Result<Value, Vec<String>> {
    let manifest: Value = serde_json::from_str(MANIFEST)
        .map_err(|e| vec![format!("pool/water manifest JSON parse failed: {e}")])?;
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|e| vec![format!("pool/water schema JSON parse failed: {e}")])?;
    let microclimate: Value = serde_json::from_str(MICROCLIMATE_BENCHMARK)
        .map_err(|e| vec![format!("pool microclimate manifest JSON parse failed: {e}")])?;
    let microclimate_schema: Value = serde_json::from_str(MICROCLIMATE_SCHEMA)
        .map_err(|e| vec![format!("pool microclimate schema JSON parse failed: {e}")])?;
    let mut errors = Vec::new();

    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("pool/water schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(&microclimate_schema, "$schema")
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        errors.push("pool microclimate schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-POOL-MICROCLIMATE-ATTRIBUTION-001"),
        ("maturity", "BENCHMARK_PROTOCOL_NOT_EXECUTED"),
        ("execution_status", "NOT_EXECUTED"),
        ("climate_context", "JOHANNESBURG_HIGHVELD"),
        ("control_arm", "DRY_SHADE_CONTROL"),
    ] {
        if text_at(&microclimate, key) != Some(expected) {
            errors.push(format!("pool microclimate {key} must remain {expected}"));
        }
    }
    let micro_arms = array_at(&microclimate, "treatment_arms")
        .cloned()
        .unwrap_or_default();
    let micro_scenarios = array_at(&microclimate, "weather_scenarios")
        .cloned()
        .unwrap_or_default();
    let micro_stations = array_at(&microclimate, "spatial_stations")
        .cloned()
        .unwrap_or_default();
    let micro_heights = array_at(&microclimate, "measurement_heights_m")
        .cloned()
        .unwrap_or_default();
    if micro_arms.len() != 6 || !unique_ids(&micro_arms) {
        errors.push("pool microclimate benchmark must preserve six unique comparison arms".to_owned());
    }
    if !contains_id(&micro_arms, "DRY_SHADE_CONTROL") {
        errors.push("pool microclimate benchmark must include a dry-shade primary control".to_owned());
    }
    if !micro_arms.iter().any(|arm| {
        text_at(arm, "id") == Some("AEROSOL_GENERATING_FOUNTAIN_NEGATIVE_CONTROL")
            && text_at(arm, "role") == Some("NEGATIVE_CONTROL")
    }) {
        errors.push("aerosol-generating fountain must remain a negative control".to_owned());
    }
    if micro_scenarios.len() != 6 || micro_stations.len() != 6 || micro_heights.len() != 4 {
        errors.push("pool microclimate benchmark must preserve six weather scenarios, six stations and four measurement heights".to_owned());
    }
    for required in [
        "UPWIND_BACKGROUND",
        "FEATURE_EDGE",
        "NEAR_FIELD_1M",
        "DOWNWIND_3M",
        "OCCUPIED_ZONE",
        "MATCHED_DRY_CONTROL",
    ] {
        if !contains_id(&micro_stations, required) {
            errors.push(format!("pool microclimate spatial station {required} is missing"));
        }
    }
    if microclimate["design"]["minute_samples_independent"].as_bool() != Some(false)
        || microclimate["design"]["matched_dry_shade_control_required"].as_bool() != Some(true)
        || microclimate["design"]["upwind_background_required"].as_bool() != Some(true)
    {
        errors.push("pool microclimate design must retain paired controls and day-block independence".to_owned());
    }
    for key in [
        "water_effect_without_dry_shade_control",
        "comfort_claim_from_dry_bulb_only",
        "indoor_cooling_claim_without_building_side_measurement",
        "unqualified_water_or_aerosol_operation",
        "untraceable_treatment_control_inputs",
        "post_hoc_holdout_or_block_changes",
        "essential_water_reserve_violations",
        "forced_water_feature_win_when_no_water_control_is_better",
    ] {
        if microclimate["hard_gates"].get(key).and_then(Value::as_u64) != Some(0) {
            errors.push(format!("pool microclimate hard gate {key} must equal zero"));
        }
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-POOL-WATER-THERMAL-001"),
        ("maturity", "DESIGN_AND_BENCHMARK_CONTRACT_NOT_EXECUTED"),
        ("execution_status", "NOT_EXECUTED"),
        ("climate_context", "JOHANNESBURG_HIGHVELD"),
    ] {
        if text_at(&manifest, key) != Some(expected) {
            errors.push(format!("{key} must be {expected}"));
        }
    }

    let archetypes = array_at(&manifest, "archetypes")
        .cloned()
        .unwrap_or_default();
    let scenarios = array_at(&manifest, "scenarios")
        .cloned()
        .unwrap_or_default();
    let cover_modes = array_at(&manifest, "cover_modes")
        .cloned()
        .unwrap_or_default();
    if archetypes.len() != 10 || !unique_ids(&archetypes) {
        errors.push("nine unique pool/water archetypes are required".to_owned());
    }
    if scenarios.len() != 8 {
        errors.push("eight pool/water stress scenarios are required".to_owned());
    }
    for required in [
        "NO_WATER_PASSIVE_BASELINE",
        "COVERED_POOL_THERMAL_BUFFER",
        "SHADED_POOL_WITH_CLOSED_LOOP_HX",
        "OPEN_POOL_EVAPORATIVE_EXCHANGE",
        "SHADED_NO_SPRAY_RILL",
        "AEROSOL_GENERATING_FOUNTAIN",
        "CLOSED_LOOP_CISTERN_THERMAL_BATTERY",
        "INDIRECT_EVAPORATIVE_HEAT_REJECTION",
        "DIRECT_MISTING_SYSTEM",
        "POOL_HEAT_RECOVERY_FROM_COOLING_LOAD",
    ] {
        if !contains_id(&archetypes, required) {
            errors.push(format!("required water archetype {required} is missing"));
        }
    }
    if ![
        "ALWAYS_UNCOVERED",
        "ALWAYS_COVERED",
        "DAY_SHADED_NIGHT_EXPOSED",
        "DAY_SHADED_NIGHT_COVERED",
        "ADAPTIVE_COVER_BY_WATER_BUDGET_AND_SKY_CONDITIONS",
    ]
    .iter()
    .all(|required| cover_modes.iter().any(|item| item == required))
    {
        errors.push("matched pool cover/shade schedules are incomplete".to_owned());
    }

    let pool_buffer = archetypes
        .iter()
        .find(|item| text_at(item, "id") == Some("COVERED_POOL_THERMAL_BUFFER"));
    if pool_buffer
        .is_none_or(|item| text_at(item, "thermal_role") != Some("FINITE_THERMAL_BUFFER_ONLY"))
    {
        errors.push("pool must remain a finite buffer, not an infinite heat sink".to_owned());
    }
    let aerosol = archetypes
        .iter()
        .find(|item| text_at(item, "id") == Some("AEROSOL_GENERATING_FOUNTAIN"));
    if aerosol.is_none_or(|item| text_at(item, "aerosol_generation") != Some("YES")) {
        errors.push("decorative fountain aerosol generation must be explicit".to_owned());
    }

    let expansion = &manifest["case_expansion"];
    if expansion["archetypes"].as_u64() != Some(10)
        || expansion["scenarios"].as_u64() != Some(8)
        || expansion["cases"].as_u64() != Some(80)
        || text_at(expansion, "state") != Some("REGISTERED_NOT_EXECUTED")
    {
        errors.push("pool/water matrix must remain ten by eight, 80 registered cases".to_owned());
    }

    let balances = array_at(&manifest, "required_balance_terms")
        .cloned()
        .unwrap_or_default();
    for required in [
        "pool_or_cistern_usable_thermal_state_change",
        "heat_rejected_by_each_external_sink",
        "evaporation_latent_heat",
        "makeup_water_enthalpy",
        "zone_moisture_balance",
        "recovered_condenser_heat",
        "displaced_pool_heater_energy",
        "excess_heat_rejected_by_fallback_sink",
        "building_side_heat_removed_from_cooling_load",
        "compressor_electrical_work_input",
        "compressor_ambient_heat_loss",
        "condenser_heat_output",
        "pool_heat_recovery",
        "fallback_sink_rejection",
        "distribution_losses",
    ] {
        if !balances.iter().any(|item| item == required) {
            errors.push(format!("thermal balance term {required} is missing"));
        }
    }
    for key in [
        "sustained_cooling_from_store_without_regeneration",
        "protected_water_floor_breaches",
        "aerosol_operation_with_unknown_or_failed_water_management",
        "untreated_greywater_in_aerosol_path",
        "indoor_cooling_claim_from_outdoor_or_pool_temp_only",
        "energy_water_balance_outside_uncertainty",
        "untraceable_inputs_or_controls",
        "pool_heat_recovery_above_temperature_limit",
        "pool_heat_recovery_without_isolated_heat_exchanger",
        "unqualified_fallback_sink_for_excess_heat",
        "heat_recovery_claim_without_measured_displaced_heater_energy",
        "condenser_first_law_balance_outside_uncertainty",
        "condenser_disposition_balance_outside_uncertainty",
        "condenser_heat_double_counting",
    ] {
        if manifest["hard_gates"].get(key).and_then(Value::as_u64) != Some(0) {
            errors.push(format!("pool/water hard gate {key} must equal zero"));
        }
    }
    let industry_refs = array_at(&manifest, "industry_reference_evidence")
        .cloned()
        .unwrap_or_default();
    let field_reference = industry_refs.iter().any(|item| {
        text_at(item, "id") == Some("UC_DAVIS_WCEC_SAN_DIEGO_HOTEL_2018")
            && text_at(item, "evidence_class") == Some("FIELD_DEMONSTRATION")
            && text_at(item, "transferability") == Some("REFERENCE_ONLY_NOT_HIGHVELD_PREDICTION")
    });
    let modeled_reference = industry_refs.iter().any(|item| {
        text_at(item, "id") == Some("CALIFORNIA_POOL_HEAT_SINK_FEASIBILITY_2013")
            && text_at(item, "evidence_class") == Some("MODELED_FEASIBILITY_ANALYSIS")
            && text_at(item, "transferability")
                == Some("MODELLED_SELECTED_CONDITIONS_NOT_FIELD_RESULT_OR_HIGHVELD_PREDICTION")
    });
    if !field_reference || !modeled_reference {
        errors.push(
            "pool heat recovery must preserve distinct field and modeled industry evidence classes"
                .to_owned(),
        );
    }
    let heat_recovery = &manifest["heat_recovery_contract"];
    for key in [
        "enabled_only_when_pool_heating_demand_exists",
        "requires_isolated_heat_exchanger",
        "maximum_pool_temperature_is_owner_and_authority_defined",
        "fallback_sink_required_when_pool_has_no_headroom",
        "excess_condenser_heat_must_be_routed_or_recovery_disabled",
        "no_pool_as_infinite_sink",
    ] {
        if heat_recovery.get(key).and_then(Value::as_bool) != Some(true) {
            errors.push(format!("pool heat-recovery contract {key} must be true"));
        }
    }
    if text_at(&manifest["solver_roles"], "indoor_pool_heat_balance")
        != Some("ENERGYPLUS_26_2_SWIMMINGPOOL_INDOOR")
        || text_at(&manifest["solver_roles"], "trajectory_accounting")
            != Some("HIGHVELD_THERMAL_STORE_LEDGER")
    {
        errors.push(
            "specialist indoor pool physics and independent ledger roles must remain distinct"
                .to_owned(),
        );
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(json!({
        "audit_state":"STRUCTURAL_CONTRACT_PASS",
        "study_id":"HIGHVELD-POOL-WATER-THERMAL-001",
        "archetype_count":archetypes.len(),
        "scenario_count":scenarios.len(),
        "cover_mode_count":cover_modes.len(),
        "registered_case_count":80,
        "pool_heat_recovery_archetype":"POOL_HEAT_RECOVERY_FROM_COOLING_LOAD",
        "pool_heat_recovery_requires_demand_hx_and_fallback":true,
        "industry_field_reference_count":industry_refs.iter().filter(|item| text_at(item,"evidence_class")==Some("FIELD_DEMONSTRATION")).count(),
        "industry_modeled_reference_count":industry_refs.iter().filter(|item| text_at(item,"evidence_class")==Some("MODELED_FEASIBILITY_ANALYSIS")).count(),
        "pool_role":"FINITE_THERMAL_BUFFER_NOT_AN_INFINITE_SINK",
        "aerosol_water_management_gate":"REQUIRED_WHEN_AEROSOL_GENERATING",
        "microclimate_attribution_study_id":"HIGHVELD-POOL-MICROCLIMATE-ATTRIBUTION-001",
        "microclimate_comparison_arm_count":micro_arms.len(),
        "microclimate_weather_scenario_count":micro_scenarios.len(),
        "microclimate_spatial_station_count":micro_stations.len(),
        "microclimate_measurement_height_count":micro_heights.len(),
        "microclimate_execution_status":"NOT_EXECUTED",
        "execution_status":"NOT_EXECUTED",
        "schema_engine":"NOT_RUN; schema declaration and manifest invariants checked by this Rust qualifier",
        "claim_ceiling":"Design and benchmark registration only; no water feature or cooling performance admitted"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" | "--audit" => match audit() {
            Ok(report) => println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            ),
            Err(errors) => {
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&json!({"errors":errors})).unwrap_or_default()
                );
                process::exit(1);
            }
        },
        "--help" | "-h" => {
            println!("highveld-pool-water-feature-qualifier");
            println!("  --self-test  audit pool/water archetype and water-safety contract");
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

    #[test]
    fn all_ten_archetypes_and_80_cases_are_registered() {
        let report = audit().expect("pool/water programme contract");
        assert_eq!(report["archetype_count"], 10);
        assert_eq!(report["scenario_count"], 8);
        assert_eq!(report["registered_case_count"], 80);
    }

    #[test]
    fn microclimate_benchmark_requires_dry_shade_control_and_spatial_transect() {
        let report = audit().expect("pool/water and microclimate contracts");
        assert_eq!(report["microclimate_comparison_arm_count"], 6);
        assert_eq!(report["microclimate_weather_scenario_count"], 6);
        assert_eq!(report["microclimate_spatial_station_count"], 6);
        assert_eq!(report["microclimate_measurement_height_count"], 4);
        assert_eq!(report["microclimate_execution_status"], "NOT_EXECUTED");
    }

    #[test]
    fn microclimate_claim_cannot_be_based_on_dry_bulb_or_aerosol_only() {
        let manifest: Value =
            serde_json::from_str(MICROCLIMATE_BENCHMARK).expect("microclimate manifest");
        assert_eq!(manifest["control_arm"], "DRY_SHADE_CONTROL");
        assert_eq!(
            manifest["design"]["matched_dry_shade_control_required"],
            true
        );
        let gates = &manifest["hard_gates"];
        assert_eq!(gates["comfort_claim_from_dry_bulb_only"], 0);
        assert_eq!(gates["unqualified_water_or_aerosol_operation"], 0);
    }

    #[test]
    fn pool_is_a_finite_buffer_not_an_infinite_sink() {
        let report = audit().expect("pool/water programme contract");
        assert_eq!(
            report["pool_role"],
            "FINITE_THERMAL_BUFFER_NOT_AN_INFINITE_SINK"
        );
    }

    #[test]
    fn aerosol_features_keep_health_management_gate() {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        let required = array_at(&manifest, "required_health_gates").unwrap();
        assert!(
            required
                .iter()
                .any(|item| item == "water_management_program_when_aerosol_generating")
        );
        assert_eq!(
            report_aerosol_kind(&manifest, "AEROSOL_GENERATING_FOUNTAIN"),
            Some("YES")
        );
    }

    #[test]
    fn pool_heat_recovery_requires_headroom_hx_and_overflow_sink() {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        let contract = &manifest["heat_recovery_contract"];
        assert_eq!(
            contract["enabled_only_when_pool_heating_demand_exists"],
            true
        );
        assert_eq!(contract["requires_isolated_heat_exchanger"], true);
        assert_eq!(
            contract["fallback_sink_required_when_pool_has_no_headroom"],
            true
        );
        assert_eq!(
            contract["excess_condenser_heat_must_be_routed_or_recovery_disabled"],
            true
        );
        assert_eq!(
            manifest["hard_gates"]["pool_heat_recovery_above_temperature_limit"],
            0
        );
        assert_eq!(
            manifest["hard_gates"]["unqualified_fallback_sink_for_excess_heat"],
            0
        );
    }

    #[test]
    fn field_trial_and_modeled_heat_recovery_evidence_are_not_conflated() {
        let report = audit().expect("pool/water programme contract");
        assert_eq!(report["industry_field_reference_count"], 1);
        assert_eq!(report["industry_modeled_reference_count"], 1);
        assert_eq!(
            report["pool_heat_recovery_archetype"],
            "POOL_HEAT_RECOVERY_FROM_COOLING_LOAD"
        );
    }

    #[test]
    fn cover_exposure_schedules_are_explicit() {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        let modes = array_at(&manifest, "cover_modes").unwrap();
        assert!(modes.iter().any(|item| item == "DAY_SHADED_NIGHT_EXPOSED"));
        assert!(
            modes
                .iter()
                .any(|item| item == "ADAPTIVE_COVER_BY_WATER_BUDGET_AND_SKY_CONDITIONS")
        );
    }

    fn report_aerosol_kind<'a>(manifest: &'a Value, id: &str) -> Option<&'a str> {
        array_at(manifest, "archetypes")?
            .iter()
            .find(|item| text_at(item, "id") == Some(id))
            .and_then(|item| text_at(item, "aerosol_generation"))
    }
}
