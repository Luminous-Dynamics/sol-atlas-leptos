#![recursion_limit = "256"]

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
const THERMAL_EXERGY_CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_THERMAL_EXERGY_CASE_V0.1.json"
);
const THERMAL_EXERGY_SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_THERMAL_EXERGY_CASE_V0.1.schema.json"
);
const SCALE_TRANSFERABILITY: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_SCALE_TRANSFERABILITY_MANIFEST_V0.1.json"
);
const SCALE_TRANSFERABILITY_SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_SCALE_TRANSFERABILITY_MANIFEST_V0.1.schema.json"
);
const WATER_BACKED_REFUGE_BENCHMARK: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/WATER_BACKED_REFUGE_BENCHMARK_MANIFEST_V0.1.json"
);
const WATER_BACKED_REFUGE_SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/WATER_BACKED_REFUGE_BENCHMARK_MANIFEST_V0.1.schema.json"
);
const POOL_COVER_SCHEDULE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_COVER_SCHEDULE_BENCHMARK_MANIFEST_V0.1.json"
);
const POOL_COVER_SCHEDULE_SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_COVER_SCHEDULE_BENCHMARK_MANIFEST_V0.1.schema.json"
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
    let exergy_case: Value = serde_json::from_str(THERMAL_EXERGY_CASE)
        .map_err(|e| vec![format!("pool thermal exergy case JSON parse failed: {e}")])?;
    let exergy_schema: Value = serde_json::from_str(THERMAL_EXERGY_SCHEMA)
        .map_err(|e| vec![format!("pool thermal exergy schema JSON parse failed: {e}")])?;
    let scale_transferability: Value =
        serde_json::from_str(SCALE_TRANSFERABILITY).map_err(|e| {
            vec![format!(
                "pool scale-transferability manifest JSON parse failed: {e}"
            )]
        })?;
    let scale_transferability_schema: Value = serde_json::from_str(SCALE_TRANSFERABILITY_SCHEMA)
        .map_err(|e| {
            vec![format!(
                "pool scale-transferability schema JSON parse failed: {e}"
            )]
        })?;
    let water_backed_refuge: Value =
        serde_json::from_str(WATER_BACKED_REFUGE_BENCHMARK).map_err(|e| {
            vec![format!(
                "water-backed refuge benchmark JSON parse failed: {e}"
            )]
        })?;
    let water_backed_refuge_schema: Value = serde_json::from_str(WATER_BACKED_REFUGE_SCHEMA)
        .map_err(|e| vec![format!("water-backed refuge schema JSON parse failed: {e}")])?;
    let pool_cover_schedule: Value = serde_json::from_str(POOL_COVER_SCHEDULE)
        .map_err(|e| vec![format!("pool cover schedule manifest JSON parse failed: {e}")])?;
    let pool_cover_schedule_schema: Value = serde_json::from_str(POOL_COVER_SCHEDULE_SCHEMA)
        .map_err(|e| vec![format!("pool cover schedule schema JSON parse failed: {e}")])?;
    let mut errors = Vec::new();

    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("pool/water schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(&microclimate_schema, "$schema")
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        errors.push("pool microclimate schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(&exergy_schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("pool thermal exergy schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(&scale_transferability_schema, "$schema")
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        errors.push(
            "pool scale-transferability schema must declare JSON Schema Draft 2020-12".to_owned(),
        );
    }
    if text_at(&water_backed_refuge_schema, "$schema")
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        errors.push("water-backed refuge schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    if text_at(&pool_cover_schedule_schema, "$schema")
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        errors.push("pool cover schedule schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-POOL-COVER-SCHEDULE-001"),
        ("maturity", "BENCHMARK_PROTOCOL_NOT_EXECUTED"),
        ("execution_status", "NOT_EXECUTED"),
    ] {
        if text_at(&pool_cover_schedule, key) != Some(expected) {
            errors.push(format!("pool cover schedule {key} must remain {expected}"));
        }
    }
    let pool_cover_arms = array_at(&pool_cover_schedule, "arms")
        .cloned()
        .unwrap_or_default();
    let pool_cover_scenarios = array_at(&pool_cover_schedule, "scenarios")
        .cloned()
        .unwrap_or_default();
    if pool_cover_arms.len() != 4 || !unique_ids(&pool_cover_arms) {
        errors.push("pool cover benchmark must preserve four unique schedule arms".to_owned());
    }
    for required in [
        "ALWAYS_COVERED",
        "ALWAYS_OPEN",
        "DAY_SHADE_NIGHT_EXPOSE",
        "ADAPTIVE_COVER_BY_BOUNDARY_AND_WATER",
    ] {
        if !contains_id(&pool_cover_arms, required) {
            errors.push(format!("pool cover schedule arm {required} is missing"));
        }
    }
    let unique_pool_cover_scenarios = pool_cover_scenarios
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    if pool_cover_scenarios.len() != 6
        || unique_pool_cover_scenarios.len() != pool_cover_scenarios.len()
    {
        errors.push("pool cover benchmark must preserve six unique scenarios".to_owned());
    }
    let pool_cover_expansion = &pool_cover_schedule["case_expansion"];
    if pool_cover_expansion["arms"].as_u64() != Some(4)
        || pool_cover_expansion["scenarios"].as_u64() != Some(6)
        || pool_cover_expansion["cases"].as_u64() != Some(24)
        || text_at(pool_cover_expansion, "state") != Some("REGISTERED_NOT_EXECUTED")
    {
        errors.push("pool cover benchmark must remain four by six, 24 registered cases".to_owned());
    }
    for required in [
        "cover_evaporation_factor",
        "cover_convection_factor",
        "cover_shortwave_radiation_factor",
        "cover_longwave_radiation_factor",
    ] {
        let factors = array_at(&pool_cover_schedule, "cover_factors_required")
            .cloned()
            .unwrap_or_default();
        if !factors.iter().any(|factor| factor.as_str() == Some(required)) {
            errors.push(format!("pool cover factor {required} is missing"));
        }
    }
    for key in [
        "net_heat_rejection_lower_bound_kw_positive",
        "evaporation_upper_bound_within_discretionary_budget",
        "qualified_water_rights_and_quality",
        "pool_temperature_inside_owner_limits",
        "cover_and_manual_override_qualified",
        "no_conflict_with_active_pool_heat_recovery",
        "unknown_critical_input_means_hold_review",
    ] {
        if pool_cover_schedule["robust_adaptive_gates"].get(key).and_then(Value::as_bool)
            != Some(true)
        {
            errors.push(format!("pool cover robust gate {key} must remain true"));
        }
    }
    for key in [
        "water_rights_or_quality_violations",
        "protected_essential_water_reserve_violations",
        "pool_temperature_limit_violations",
        "regeneration_with_nonpositive_lower_heat_rejection_bound",
        "exposure_with_evaporation_upper_bound_above_budget",
        "missing_required_cover_pathway_factors",
        "indoor_pool_model_claimed_as_outdoor_validation",
        "untraceable_schedule_cases",
        "hard_floor_failures_hidden_by_weighted_score",
    ] {
        if pool_cover_schedule["hard_gates"].get(key).and_then(Value::as_u64) != Some(0) {
            errors.push(format!("pool cover hard gate {key} must equal zero"));
        }
    }
    if text_at(&pool_cover_schedule["model_scope"], "indoor_pool_baseline")
        != Some("ENERGYPLUS_26_2_SWIMMINGPOOL_INDOOR_ONLY")
        || text_at(&pool_cover_schedule["model_scope"], "outdoor_pool")
            != Some("SPECIALIST_OUTDOOR_BOUNDARY_MODEL_AND_FIELD_VALIDATION_REQUIRED")
        || pool_cover_schedule["model_scope"]["no_cross_scope_promotion"].as_bool() != Some(true)
    {
        errors.push("pool cover benchmark must preserve indoor/outdoor solver boundary".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-WATER-BACKED-REFUGE-001"),
        ("maturity", "BENCHMARK_PROTOCOL_NOT_EXECUTED"),
        ("execution_status", "NOT_EXECUTED"),
        ("climate_context", "JOHANNESBURG_HIGHVELD"),
    ] {
        if text_at(&water_backed_refuge, key) != Some(expected) {
            errors.push(format!("water-backed refuge {key} must remain {expected}"));
        }
    }
    let water_refuge_arms = array_at(&water_backed_refuge, "comparison_arms")
        .cloned()
        .unwrap_or_default();
    let water_refuge_scenarios = array_at(&water_backed_refuge, "scenarios")
        .cloned()
        .unwrap_or_default();
    if water_refuge_arms.len() != 6 || !unique_ids(&water_refuge_arms) {
        errors.push("water-backed refuge must preserve six unique comparison arms".to_owned());
    }
    if water_refuge_scenarios.len() != 6
        || water_refuge_scenarios
            .iter()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>()
            .len()
            != water_refuge_scenarios.len()
    {
        errors.push("water-backed refuge must preserve six unique operating scenarios".to_owned());
    }
    for required in [
        "DRY_SHADE_VENTILATION_CONTROL",
        "MECHANICAL_REFUGE_REFERENCE",
        "RADIANT_WATER_LOOP_NO_STORE",
        "CLOSED_LOOP_STORE_GROUND_REGENERATION",
        "CLOSED_LOOP_STORE_RADIATIVE_REGENERATION",
        "LAYERED_WATER_BACKED_REFUGE",
    ] {
        if !contains_id(&water_refuge_arms, required) {
            errors.push(format!(
                "water-backed refuge comparison arm {required} is missing"
            ));
        }
    }
    let water_expansion = &water_backed_refuge["case_expansion"];
    if water_expansion["arms"].as_u64() != Some(6)
        || water_expansion["scenarios"].as_u64() != Some(6)
        || water_expansion["cases"].as_u64() != Some(36)
        || text_at(water_expansion, "state") != Some("REGISTERED_NOT_EXECUTED")
    {
        errors.push(
            "water-backed refuge matrix must remain six by six, 36 registered cases".to_owned(),
        );
    }
    let qanat = &water_backed_refuge["field_reference"];
    if text_at(qanat, "reference_id") != Some("CARTUJA_QANAT_AGORA_SEVILLE_2026")
        || text_at(qanat, "evidence_class") != Some("LARGE_SCALE_FIELD_ASSESSMENT")
        || text_at(qanat, "transferability") != Some("REFERENCE_ONLY_NOT_HIGHVELD_PREDICTION")
        || qanat["reported_values"]["night_regeneration_capacity_kw"].as_f64() != Some(152.0)
        || qanat["reported_values"]["reported_cop"].as_f64() != Some(32.4)
    {
        errors.push(
            "Qanat field reference must retain its reported values and non-transferable boundary"
                .to_owned(),
        );
    }
    for key in [
        "protected_essential_water_reserve_breaches",
        "unqualified_water_or_aerosol_operation",
        "indoor_cooling_claim_without_building_side_heat_path",
        "thermal_store_as_sink_without_measured_regeneration",
        "balance_residual_outside_propagated_uncertainty",
        "owner_temperature_limit_breach_reported_as_positive_headroom",
        "untraceable_model_weather_property_rights_or_instrument_inputs",
        "missed_seeded_negative_controls",
        "false_blocks_of_positive_controls",
    ] {
        if water_backed_refuge["hard_gates"]
            .get(key)
            .and_then(Value::as_u64)
            != Some(0)
        {
            errors.push(format!(
                "water-backed refuge hard gate {key} must equal zero"
            ));
        }
    }
    if water_backed_refuge["governance"]["aerosol_generating_path_default_enabled"].as_bool()
        != Some(false)
        || water_backed_refuge["governance"]["comfort_is_health_safety_limit"].as_bool()
            != Some(false)
        || water_backed_refuge["governance"]["unknown_water_state_means_disallowed"].as_bool()
            != Some(true)
        || water_backed_refuge["governance"]["pool_is_finite_thermal_buffer"].as_bool()
            != Some(true)
    {
        errors.push(
            "water-backed refuge must preserve aerosol, comfort, water and finite-store governance"
                .to_owned(),
        );
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-POOL-SCALE-TRANSFER-001"),
        ("maturity", "RESEARCH_PROTOCOL_NOT_EXECUTED"),
        ("execution_status", "NOT_EXECUTED"),
        ("climate_context", "JOHANNESBURG_HIGHVELD"),
    ] {
        if text_at(&scale_transferability, key) != Some(expected) {
            errors.push(format!(
                "pool scale-transferability {key} must remain {expected}"
            ));
        }
    }
    let scale_tiers = array_at(&scale_transferability, "scale_tiers")
        .cloned()
        .unwrap_or_default();
    if scale_tiers.len() != 5 || !unique_ids(&scale_tiers) {
        errors
            .push("pool scale-transferability must preserve five unique evidence tiers".to_owned());
    }
    for required in [
        "COMPONENT_POOL",
        "COURTYARD_PEDESTRIAN",
        "BUILDING_OCCUPIED_ZONE",
        "NEIGHBORHOOD_BLUE_SPACE",
        "THERMAL_SERVICE_REFUGE",
    ] {
        if !contains_id(&scale_tiers, required) {
            errors.push(format!(
                "pool scale-transferability tier {required} is missing"
            ));
        }
    }
    let promotion_rules = array_at(&scale_transferability, "promotion_rules")
        .cloned()
        .unwrap_or_default();
    if promotion_rules.len() != 4 || !unique_ids(&promotion_rules) {
        errors.push(
            "exactly four unique non-automatic scale-promotion rules are required".to_owned(),
        );
    }
    if promotion_rules
        .iter()
        .any(|rule| rule["automatic"].as_bool() != Some(false))
    {
        errors.push("scale promotion must never be automatic".to_owned());
    }
    if promotion_rules.iter().any(|rule| {
        text_at(rule, "from") == Some("COMPONENT_POOL")
            && text_at(rule, "to") == Some("THERMAL_SERVICE_REFUGE")
    }) {
        errors.push("direct component-to-refuge claim promotion is prohibited".to_owned());
    }

    let literature = array_at(
        &scale_transferability,
        "literature_transferability_controls",
    )
    .cloned()
    .unwrap_or_default();
    let urban_threshold = literature
        .iter()
        .find(|item| text_at(item, "id") == Some("DENSE_URBAN_WATER_BODY_2026_3_41_HA"));
    if urban_threshold.is_none_or(|item| {
        item["reported_value"].as_f64() != Some(3.41)
            || text_at(item, "reported_unit") != Some("ha")
            || text_at(item, "transferability")
                != Some("NOT_TRANSFERABLE_AS_RESIDENTIAL_POOL_OR_COURTYARD_THRESHOLD")
    }) {
        errors.push("the 3.41 ha study-specific threshold must remain non-transferable".to_owned());
    }
    let park_context = literature
        .iter()
        .find(|item| text_at(item, "id") == Some("MOCHOU_LAKE_PARK_2026"));
    if park_context.is_none_or(|item| {
        item["reported_park_mean_air_temperature_difference_c"].as_f64() != Some(1.30)
            || item["reported_activity_space_air_temperature_range_c"].as_f64() != Some(2.22)
            || text_at(item, "reported_utci_state")
                != Some("AT_LEAST_STRONG_HEAT_STRESS_AT_ALL_SAMPLED_INTERNAL_POINTS")
    }) {
        errors.push(
            "park-average cooling must remain distinct from activity-space heat stress".to_owned(),
        );
    }
    for key in [
        "urban_water_body_area_threshold_transferred_to_residential_pool",
        "land_surface_temperature_only_promoted_to_pedestrian_comfort",
        "dry_bulb_only_promoted_to_thermal_comfort_or_safety",
        "spatial_mean_hiding_activity_space_heat_stress",
        "courtyard_evidence_promoted_to_building_cooling_without_zone_path",
        "neighborhood_map_promoted_to_refuge_without_capacity_route_operations",
        "treatment_without_matched_dry_shade_and_upwind_control",
        "minute_samples_treated_as_independent_replicates",
        "untraceable_scale_claims",
    ] {
        if scale_transferability["hard_gates"]
            .get(key)
            .and_then(Value::as_u64)
            != Some(0)
        {
            errors.push(format!(
                "pool scale-transferability hard gate {key} must equal zero"
            ));
        }
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
        if text_at(&exergy_case, key) != Some(expected) {
            errors.push(format!("pool thermal exergy {key} must be {expected}"));
        }
    }
    let exergy_intervals = array_at(&exergy_case, "intervals")
        .cloned()
        .unwrap_or_default();
    if exergy_intervals.len() != 3 {
        errors.push("pool thermal exergy sidecar must preserve three interval controls".to_owned());
    }
    for interval in &exergy_intervals {
        let layers = array_at(interval, "layers").cloned().unwrap_or_default();
        let mut layer_ids = BTreeSet::new();
        let unique_layer_ids = layers.iter().all(|layer| {
            text_at(layer, "layer_id").is_some_and(|id| layer_ids.insert(id.to_owned()))
        });
        if layers.len() != 3 || !unique_layer_ids {
            errors.push(format!(
                "{}: three unique water layers are required for exergy",
                text_at(interval, "interval_id").unwrap_or("<missing-interval-id>")
            ));
        }
        if text_at(interval, "environmental_reference_method_ref").is_none()
            || text_at(interval, "layer_state_evidence_ref").is_none()
            || text_at(interval, "uncertainty_evidence_ref").is_none()
        {
            errors.push(format!(
                "{}: exergy reference, layer state and uncertainty evidence are required",
                text_at(interval, "interval_id").unwrap_or("<missing-interval-id>")
            ));
        }
    }
    if text_at(&exergy_case["property_model"], "specific_heat_method_ref").is_none()
        || text_at(&exergy_case["property_model"], "density_method_ref").is_none()
    {
        errors.push(
            "temperature-grade calculation requires explicit water property methods".to_owned(),
        );
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
        errors.push(
            "pool microclimate benchmark must preserve six unique comparison arms".to_owned(),
        );
    }
    if !contains_id(&micro_arms, "DRY_SHADE_CONTROL") {
        errors.push(
            "pool microclimate benchmark must include a dry-shade primary control".to_owned(),
        );
    }
    if !micro_arms.iter().any(|arm| {
        text_at(arm, "id") == Some("AEROSOL_GENERATING_FOUNTAIN_NEGATIVE_CONTROL")
            && text_at(arm, "role") == Some("NEGATIVE_CONTROL")
    }) {
        errors.push("aerosol-generating fountain must remain a negative control".to_owned());
    }
    let unique_micro_scenarios = micro_scenarios
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    if micro_scenarios.len() != 6
        || unique_micro_scenarios.len() != micro_scenarios.len()
        || micro_stations.len() != 6
        || !unique_ids(&micro_stations)
        || micro_heights.len() != 4
    {
        errors.push(
            "pool microclimate benchmark must preserve six unique weather scenarios, six unique stations and four measurement heights".to_owned(),
        );
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
            errors.push(format!(
                "pool microclimate spatial station {required} is missing"
            ));
        }
    }
    if micro_stations
        .iter()
        .any(|station| station["control_pair_required"].as_bool() != Some(true))
    {
        errors.push("every microclimate spatial station must bind to a matched control".to_owned());
    }
    let micro_measurements = array_at(&microclimate, "required_measurements")
        .cloned()
        .unwrap_or_default();
    for required in [
        "DRY_BULB_AIR_TEMPERATURE_BY_STATION_AND_HEIGHT",
        "RELATIVE_HUMIDITY_OR_DEW_POINT_BY_STATION",
        "WIND_SPEED_AND_DIRECTION_WITH_EXPOSURE",
        "GLOBE_OR_MEAN_RADIANT_TEMPERATURE",
        "WATER_EVAPORATION_AND_VOLUME_CHANGE",
        "STATION_LOCATION_HEIGHT_AND_SHADE_GEOMETRY",
    ] {
        if !micro_measurements
            .iter()
            .any(|item| item.as_str() == Some(required))
        {
            errors.push(format!(
                "pool microclimate measurement {required} is missing"
            ));
        }
    }
    if !micro_arms.iter().any(|arm| {
        text_at(arm, "id") == Some("DRY_SHADE_CONTROL")
            && text_at(arm, "role") == Some("PRIMARY_CONTROL")
    }) {
        errors.push("DRY_SHADE_CONTROL must be the primary control arm".to_owned());
    }
    if microclimate["design"]["minute_samples_independent"].as_bool() != Some(false)
        || microclimate["design"]["matched_dry_shade_control_required"].as_bool() != Some(true)
        || microclimate["design"]["upwind_background_required"].as_bool() != Some(true)
    {
        errors.push(
            "pool microclimate design must retain paired controls and day-block independence"
                .to_owned(),
        );
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
        errors.push("ten unique pool/water archetypes are required".to_owned());
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
        "thermal_exergy_study_id":"HIGHVELD-POOL-THERMAL-EXERGY-001",
        "thermal_exergy_interval_count":exergy_intervals.len(),
        "thermal_exergy_claim_ceiling":"LAYER_RESOLVED_TEMPERATURE_GRADE_AND_IDEALIZED_HEADROOM_ONLY",
        "scale_transferability_study_id":"HIGHVELD-POOL-SCALE-TRANSFER-001",
        "scale_transferability_tier_count":scale_tiers.len(),
        "scale_promotion_rule_count":promotion_rules.len(),
        "urban_water_area_threshold_transferability":"NOT_TRANSFERABLE_TO_RESIDENTIAL_POOL_OR_COURTYARD",
        "park_average_vs_activity_space_stress_boundary":"ENFORCED",
        "scale_transferability_execution_status":"NOT_EXECUTED",
        "water_backed_refuge_study_id":"HIGHVELD-WATER-BACKED-REFUGE-001",
        "water_backed_refuge_arm_count":water_refuge_arms.len(),
        "water_backed_refuge_scenario_count":water_refuge_scenarios.len(),
        "water_backed_refuge_case_count":water_expansion["cases"].as_u64().unwrap_or(0),
        "water_backed_refuge_execution_status":"NOT_EXECUTED",
        "pool_cover_schedule_study_id":"HIGHVELD-POOL-COVER-SCHEDULE-001",
        "pool_cover_schedule_arm_count":pool_cover_arms.len(),
        "pool_cover_schedule_scenario_count":pool_cover_scenarios.len(),
        "pool_cover_schedule_case_count":pool_cover_expansion["cases"].as_u64().unwrap_or(0),
        "pool_cover_schedule_execution_status":"NOT_EXECUTED",
        "pool_cover_solver_boundary":"ENERGYPLUS_INDOOR_POOL_ONLY_OUTDOOR_PHYSICS_REQUIRES_SPECIALIST_FIELD_VALIDATION",
        "qanat_reference_transferability":"REFERENCE_ONLY_NOT_HIGHVELD_PREDICTION",
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
    fn scale_transferability_manifest_preserves_five_tiers_and_no_automatic_promotion() {
        let report = audit().expect("pool scale-transferability contract");
        assert_eq!(report["scale_transferability_tier_count"], 5);
        assert_eq!(report["scale_promotion_rule_count"], 4);
        assert_eq!(
            report["urban_water_area_threshold_transferability"],
            "NOT_TRANSFERABLE_TO_RESIDENTIAL_POOL_OR_COURTYARD"
        );
        assert_eq!(
            report["park_average_vs_activity_space_stress_boundary"],
            "ENFORCED"
        );
        assert_eq!(
            report["scale_transferability_execution_status"],
            "NOT_EXECUTED"
        );
    }

    #[test]
    fn thermal_exergy_sidecar_requires_layer_resolved_property_and_reference_evidence() {
        let report = audit().expect("pool thermal exergy structural contract");
        assert_eq!(report["thermal_exergy_interval_count"], 3);
        assert_eq!(
            report["thermal_exergy_claim_ceiling"],
            "LAYER_RESOLVED_TEMPERATURE_GRADE_AND_IDEALIZED_HEADROOM_ONLY"
        );
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
    fn integrated_water_backed_refuge_is_36_cases_and_qanat_is_nontransferable() {
        let report = audit().expect("integrated water-backed refuge benchmark");
        assert_eq!(report["water_backed_refuge_arm_count"], 6);
        assert_eq!(report["water_backed_refuge_scenario_count"], 6);
        assert_eq!(report["water_backed_refuge_case_count"], 36);
        assert_eq!(
            report["water_backed_refuge_execution_status"],
            "NOT_EXECUTED"
        );
        assert_eq!(
            report["qanat_reference_transferability"],
            "REFERENCE_ONLY_NOT_HIGHVELD_PREDICTION"
        );
    }

    #[test]
    fn adaptive_pool_cover_schedule_is_registered_with_robust_uncertainty_gates() {
        let report = audit().expect("pool cover schedule benchmark contract");
        assert_eq!(report["pool_cover_schedule_arm_count"], 4);
        assert_eq!(report["pool_cover_schedule_scenario_count"], 6);
        assert_eq!(report["pool_cover_schedule_case_count"], 24);
        assert_eq!(report["pool_cover_schedule_execution_status"], "NOT_EXECUTED");
        assert_eq!(
            report["pool_cover_solver_boundary"],
            "ENERGYPLUS_INDOOR_POOL_ONLY_OUTDOOR_PHYSICS_REQUIRES_SPECIALIST_FIELD_VALIDATION"
        );
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
