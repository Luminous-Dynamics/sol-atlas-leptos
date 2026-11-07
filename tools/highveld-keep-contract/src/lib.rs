//! Offline consistency checker for the synthetic Highveld Keep reference package.
//!
//! This is not a geometry checker, building simulator, code validator, or approval
//! engine. It checks only cross-file invariants in two versioned E0 JSON documents.

use serde_json::Value;

pub const SCENARIO_JSON: &str =
    include_str!("../../../docs/research/highveld-keep/HIGHVELD_KEEP_SCENARIO_V0_1.json");
pub const PARAMETER_JSON: &str =
    include_str!("../../../docs/research/highveld-keep/HIGHVELD_KEEP_PARAMETER_SPACE_V0_1.json");

pub const SCENARIO_ID: &str = "HIGHVELD-KEEP-SYNTHETIC-001";

const EXPECTED_FAMILIES: [&str; 4] = [
    "courtyard_cloister",
    "winged_great_hall",
    "compact_keep",
    "garden_pavilions",
];

const EXPECTED_ALTERNATIVE_IDS: [&str; 4] = [
    "A_COURTYARD_CLOISTER",
    "B_WINGED_GREAT_HALL",
    "C_COMPACT_KEEP",
    "D_GARDEN_PAVILIONS",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationReport {
    pub scenario_id: String,
    pub alternatives: usize,
    pub weather_candidates: usize,
    pub continuous_variables: usize,
    pub categorical_variables: usize,
    pub functional_relationships: usize,
    pub alternative_climate_methods: usize,
    pub station_metadata_crosschecks: usize,
    pub station_network_profiles: usize,
    pub violations: Vec<&'static str>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }
}

fn path<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter()
        .try_fold(value, |current, key| current.get(*key))
}

fn string_at<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    path(value, keys).and_then(Value::as_str)
}

fn array_at<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Vec<Value>> {
    path(value, keys).and_then(Value::as_array)
}

fn is_null_at(value: &Value, keys: &[&str]) -> bool {
    path(value, keys).is_some_and(Value::is_null)
}

fn array_contains(values: Option<&Vec<Value>>, expected: &str) -> bool {
    values.is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn find_by_id<'a>(items: &'a [Value], id: &str) -> Option<&'a Value> {
    items
        .iter()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
}

fn push_if(violations: &mut Vec<&'static str>, condition: bool, id: &'static str) {
    if condition {
        violations.push(id);
    }
}

/// Validate two already-parsed JSON documents. The function is pure and offline.
pub fn validate_values(scenario: &Value, parameters: &Value) -> ValidationReport {
    let mut violations = Vec::new();

    let scenario_id = string_at(scenario, &["scenario_id"])
        .unwrap_or("UNKNOWN_SCENARIO_ID")
        .to_owned();
    let alternatives = array_at(scenario, &["architectural_alternatives"]);
    let weather = array_at(scenario, &["climate_inputs", "candidate_weather_files"]);
    let topology = array_at(parameters, &["topology_variables"]);
    let mut continuous_count = 0usize;
    let mut categorical_count = 0usize;
    let functional_relationships = array_at(scenario, &["functional_relationships"]);
    let station_metadata = array_at(scenario, &["station_metadata_crosschecks"]);
    let mut station_metadata_count = 0usize;
    let mut functional_relationship_count = 0usize;

    push_if(
        &mut violations,
        string_at(scenario, &["schema_version"]) != Some("luminous.habitat.reference-scenario/0.1"),
        "SCENARIO_SCHEMA_VERSION_MISMATCH",
    );
    push_if(
        &mut violations,
        string_at(parameters, &["schema_version"]) != Some("luminous.habitat.parameter-space/0.1"),
        "PARAMETER_SCHEMA_VERSION_MISMATCH",
    );
    push_if(
        &mut violations,
        scenario_id != SCENARIO_ID,
        "SCENARIO_ID_MISMATCH",
    );
    push_if(
        &mut violations,
        string_at(parameters, &["scenario_id"]) != Some(SCENARIO_ID),
        "PARAMETER_SCENARIO_ID_MISMATCH",
    );
    push_if(
        &mut violations,
        string_at(scenario, &["revision_status"]) != Some("E0_CONCEPT_SYNTHETIC"),
        "SCENARIO_NOT_E0_SYNTHETIC",
    );
    push_if(
        &mut violations,
        string_at(scenario, &["evidence_tier"]) != Some("E0"),
        "SCENARIO_EVIDENCE_TIER_PROMOTED",
    );
    push_if(
        &mut violations,
        string_at(scenario, &["claim_ceiling"])
            != Some("synthetic_requirements_and_design_space_only"),
        "SCENARIO_CLAIM_CEILING_CHANGED",
    );

    // Real-site fields must remain absent until a surveyed, legally reviewed parcel exists.
    push_if(
        &mut violations,
        path(scenario, &["geography", "reference_only"]).and_then(Value::as_bool) != Some(true)
            || !is_null_at(scenario, &["geography", "parcel_id"])
            || !is_null_at(scenario, &["geography", "coordinates"])
            || !is_null_at(scenario, &["geography", "survey"])
            || !is_null_at(scenario, &["geography", "zoning"])
            || !is_null_at(scenario, &["geography", "geotechnical_report"])
            || !is_null_at(scenario, &["geography", "dolomite_assessment"])
            || string_at(scenario, &["geography", "status"])
                != Some("UNKNOWN_NO_REAL_PARCEL_SELECTED"),
        "REAL_SITE_EVIDENCE_PROMOTED",
    );
    push_if(
        &mut violations,
        path(
            scenario,
            &["synthetic_site_envelope", "enabled_for_fixture_search_only"],
        )
        .and_then(Value::as_bool)
            != Some(true)
            || string_at(scenario, &["synthetic_site_envelope", "label"])
                != Some("SYNTHETIC_NOT_A_SURVEY_OR_ZONING_ENVELOPE")
            || string_at(scenario, &["synthetic_site_envelope", "status"])
                != Some("ASSUMPTION_NOT_SITE_EVIDENCE"),
        "SYNTHETIC_SITE_LABEL_MISSING_OR_CHANGED",
    );

    // Require the exact four declared design families, exactly once, and bind
    // each architecture candidate to the matching parameter-space family.
    let family_map = [
        ("A_COURTYARD_CLOISTER", "courtyard_cloister"),
        ("B_WINGED_GREAT_HALL", "winged_great_hall"),
        ("C_COMPACT_KEEP", "compact_keep"),
        ("D_GARDEN_PAVILIONS", "garden_pavilions"),
    ];
    let mut alternative_ids = Vec::new();
    let mut family_count = 0usize;
    let mut family_bindings_valid = true;
    if let Some(items) = alternatives {
        family_count = items.len();
        for item in items {
            if let Some(id) = item.get("id").and_then(Value::as_str) {
                alternative_ids.push(id.to_owned());
                let expected_family = family_map
                    .iter()
                    .find(|(expected_id, _)| *expected_id == id)
                    .map(|(_, family)| *family);
                let actual_family = item.get("family").and_then(Value::as_str);
                if expected_family != actual_family {
                    family_bindings_valid = false;
                }
            } else {
                family_bindings_valid = false;
            }
        }
    }
    let unique_alternative_ids: std::collections::HashSet<&str> =
        alternative_ids.iter().map(String::as_str).collect();
    push_if(
        &mut violations,
        family_count != EXPECTED_ALTERNATIVE_IDS.len()
            || unique_alternative_ids.len() != EXPECTED_ALTERNATIVE_IDS.len()
            || EXPECTED_ALTERNATIVE_IDS
                .iter()
                .any(|expected| !unique_alternative_ids.contains(expected)),
        "ALTERNATIVE_FAMILY_SET_INVALID",
    );
    push_if(
        &mut violations,
        !family_bindings_valid,
        "ALTERNATIVE_TO_PARAMETER_FAMILY_BINDING_INVALID",
    );

    let family_parameter = topology.and_then(|items| find_by_id(items, "spatial_family"));
    let family_values = family_parameter
        .and_then(|item| item.get("values"))
        .and_then(Value::as_array);
    let family_strings: Vec<&str> = family_values
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let unique_families: std::collections::HashSet<&str> = family_strings.iter().copied().collect();
    push_if(
        &mut violations,
        family_strings.len() != EXPECTED_FAMILIES.len()
            || unique_families.len() != EXPECTED_FAMILIES.len()
            || EXPECTED_FAMILIES
                .iter()
                .any(|expected| !unique_families.contains(expected)),
        "SPATIAL_FAMILY_PARAMETER_SET_INVALID",
    );

    // Weather candidates must remain typical-year, unqualified source candidates.
    let mut weather_count = 0usize;
    if let Some(items) = weather {
        weather_count = items.len();
        let expected_weather = [
            (
                "JOBURG_TAMBO_TMYX_2011_2025",
                "ZAF_GT_Johannesburg-Tambo.Intl.AP.683680_TMYx.2011-2025.zip",
            ),
            (
                "JOBURG_BOTANICAL_GARDENS_TMYX_2011_2025",
                "ZAF_GT_Johannesburg.Botanical.Gardens.683610_TMYx.2011-2025.zip",
            ),
            (
                "JOBURG_GRAND_CENTRAL_TMYX_2011_2025",
                "ZAF_GT_Grand.Central.AP.682650_TMYx.2011-2025.zip",
            ),
            (
                "JOBURG_RAND_AIRPORT_TMYX_2011_2025",
                "ZAF_GT_Rand.AP.683463_TMYx.2011-2025.zip",
            ),
            (
                "JOBURG_LANSERIA_INTL_TMYX_2011_2025",
                "ZAF_GT_Lanseria.Intl.AP.682635_TMYx.2011-2025.zip",
            ),
        ];
        for item in items {
            let identity_is_expected = item
                .get("id")
                .and_then(Value::as_str)
                .zip(item.get("filename").and_then(Value::as_str))
                .is_some_and(|identity| expected_weather.contains(&identity));
            let valid_candidate = identity_is_expected
                && item.get("dataset_type").and_then(Value::as_str)
                    == Some("TYPICAL_METEOROLOGICAL_YEAR")
                && item.get("status").and_then(Value::as_str)
                    == Some("CATALOGUED_NOT_HEADER_INSPECTED_OR_QUALIFIED")
                && item.get("sha256").is_some_and(Value::is_null)
                && item.get("reuse_rights").and_then(Value::as_str) == Some("UNKNOWN")
                && item.get("site_representativeness").and_then(Value::as_str) == Some("UNKNOWN")
                && item
                    .get("source_page_rights_note")
                    .and_then(Value::as_str)
                    .is_some_and(|note| {
                        note.contains("all_rights_reserved")
                            && note.contains("no_file_specific_license")
                    })
                && item
                    .get("source")
                    .and_then(Value::as_str)
                    .is_some_and(|source| {
                        source.contains(
                            "climate.onebuilding.org/WMO_Region_1_Africa/ZAF_South_Africa",
                        )
                    });
            if !valid_candidate {
                violations.push("WEATHER_CANDIDATE_PROMOTED_OR_INCOMPLETE");
                break;
            }
        }
    }
    push_if(
        &mut violations,
        weather_count != 5,
        "WEATHER_CANDIDATE_COUNT_CHANGED",
    );
    let station_ensemble = path(
        scenario,
        &["climate_inputs", "station_ensemble_policy"],
    );
    let ensemble_policy_valid = station_ensemble.is_some_and(|policy| {
        policy.get("status").and_then(Value::as_str)
            == Some("REGIONAL_CANDIDATE_ENSEMBLE_NOT_LOCAL_SITE_WEATHER")
            && policy.get("selected_site_station").is_some_and(Value::is_null)
            && policy.get("rule").and_then(Value::as_str)
                == Some("preserve_each_file_as_a_distinct_source_and_model_scenario")
            && [
                "blind_average_of_station_files",
                "choose_nearest_station_without_exposure_review",
                "infer_parcel_representativeness_from_city_name",
                "use_tmy_as_extreme_or_future_climate",
            ]
            .iter()
            .all(|prohibited| {
                array_contains(
                    policy.get("prohibited").and_then(Value::as_array),
                    prohibited,
                )
            })
    });
    push_if(
        &mut violations,
        !ensemble_policy_valid,
        "STATION_ENSEMBLE_POLICY_INVALID",
    );

    let required_profiles = [
        "historical_hot_spell_and_hot_night",
        "historical_cold_night",
        "hail_and_intense_rainfall",
        "site_specific_wind_and_storm",
        "future_climate_scenario_with_model_period_and_quantile",
        "grid_outage_during_hot_spell",
        "water_supply_disruption",
        "air_quality_constrained_ventilation",
    ];
    let profiles = array_at(scenario, &["climate_inputs", "separate_required_profiles"]);
    push_if(
        &mut violations,
        required_profiles
            .iter()
            .any(|required| !array_contains(profiles, required)),
        "WEATHER_PROFILE_SEPARATION_INCOMPLETE",
    );

    // Preserve the second climate-provider methodology as research only.
    // Provider-published global validation and a coarse regional grid trend are
    // not local Highveld file qualification.
    let alternative_methods = array_at(scenario, &["alternative_climate_provider_methodologies"]);
    let mut alternative_method_count = 0usize;
    if let Some(items) = alternative_methods {
        alternative_method_count = items.len();
        let fusion = find_by_id(items, "EPW_TOOLS_STATION_REANALYSIS_FUSION");
        let fusion_is_research_only = fusion.is_some_and(|item| {
            item.get("provider").and_then(Value::as_str) == Some("EPW.tools")
                && item.get("status").and_then(Value::as_str)
                    == Some("RESEARCH_ONLY_NOT_INGESTED_OR_QUALIFIED")
                && item.get("source_file_downloaded").and_then(Value::as_bool) == Some(false)
                && item.get("file_digest").is_some_and(Value::is_null)
                && item.get("reuse_rights").and_then(Value::as_str) == Some("UNKNOWN")
                && item
                    .get("licence_and_terms_checked")
                    .and_then(Value::as_bool)
                    == Some(false)
                && item.get("local_validation_status").and_then(Value::as_str)
                    == Some("NOT_INDEPENDENTLY_REVIEWED_FOR_JOHANNESBURG_OR_A_HIGHVELD_PARCEL")
        });
        push_if(
            &mut violations,
            !fusion_is_research_only,
            "ALTERNATIVE_CLIMATE_PROVIDER_PROMOTED_OR_INCOMPLETE",
        );

        let regional_trend = find_by_id(items, "JOHANNESBURG_GRID_TREND_EPWT");
        let regional_scope_is_bounded = regional_trend.is_some_and(|item| {
            item.get("spatial_resolution").and_then(Value::as_str) == Some("0.5_degree_grid_cell")
                && item.get("status").and_then(Value::as_str)
                    == Some("REGIONAL_RESEARCH_CONTEXT_ONLY_NOT_DESIGN_WEATHER")
                && item
                    .get("scope_limit")
                    .and_then(Value::as_str)
                    .is_some_and(|value| {
                        value.contains("not an airport station")
                            && value.contains("central-city thermometer")
                            && value.contains("parcel measurement")
                    })
        });
        push_if(
            &mut violations,
            !regional_scope_is_bounded,
            "REGIONAL_CLIMATE_TREND_SCOPE_PROMOTED",
        );
    } else {
        violations.push("ALTERNATIVE_CLIMATE_METHODS_MISSING");
    }
    push_if(
        &mut violations,
        alternative_method_count != 2,
        "ALTERNATIVE_CLIMATE_METHOD_COUNT_CHANGED",
    );

    // Station identity/coordinate records are metadata cross-checks only.
    // They are not weather observations and cannot qualify a weather file or site.
    const STATION_METADATA_STATUS: [(&str, &str); 7] = [
        (
            "JOHANNESBURG_BOTANICAL_ASHRAE_2021",
            "METADATA_CROSSCHECK_ONLY_NOT_SERIES_OR_SITE_QUALIFICATION",
        ),
        (
            "JOHANNESBURG_TAMBO_ASHRAE_2021",
            "METADATA_CROSSCHECK_ONLY_NOT_SERIES_OR_SITE_QUALIFICATION",
        ),
        (
            "JOHANNESBURG_TAMBO_NOAA_GHCND",
            "METADATA_CROSSCHECK_ONLY_IDENTITY_AND_HISTORY_NOT_RECONCILED",
        ),
        (
            "JOHANNESBURG_TAMBO_SAWS_WIGOS_OSCAR",
            "SECONDARY_REFERENCE_TO_WMO_OSCAR_NOT_DIRECTLY_VERIFIED",
        ),
        (
            "JOBURG_GRAND_CENTRAL_TMYX_CATALOGUE_ONLY",
            "CATALOGUE_ID_ONLY_STATION_METADATA_NOT_RETRIEVED",
        ),
        (
            "JOBURG_RAND_AIRPORT_TMYX_CATALOGUE_ONLY",
            "CATALOGUE_ID_ONLY_STATION_METADATA_NOT_RETRIEVED",
        ),
        (
            "JOBURG_LANSERIA_TMYX_CATALOGUE_ONLY",
            "CATALOGUE_ID_ONLY_STATION_METADATA_NOT_RETRIEVED",
        ),
    ];
    if let Some(items) = station_metadata {
        station_metadata_count = items.len();
        for (id, required_status) in STATION_METADATA_STATUS {
            let valid = find_by_id(items, id).is_some_and(|item| {
                item.get("status").and_then(Value::as_str) == Some(required_status)
                    && item
                        .get("measurement_series_downloaded")
                        .and_then(Value::as_bool)
                        == Some(false)
                    && item.get("file_digest").is_some_and(Value::is_null)
            });
            if !valid {
                violations.push("STATION_METADATA_CROSSCHECK_PROMOTED_OR_INCOMPLETE");
            }
        }

        for id in [
            "JOBURG_GRAND_CENTRAL_TMYX_CATALOGUE_ONLY",
            "JOBURG_RAND_AIRPORT_TMYX_CATALOGUE_ONLY",
            "JOBURG_LANSERIA_TMYX_CATALOGUE_ONLY",
        ] {
            let record_is_catalogue_only = find_by_id(items, id).is_some_and(|item| {
                item.get("authority_scope").and_then(Value::as_str)
                    == Some("filename_station_code_only")
                    && item.get("history_and_instrument_metadata").and_then(Value::as_str)
                        == Some("not_retrieved")
                    && item.get("latitude_deg").is_none()
                    && item.get("longitude_deg").is_none()
            });
            if !record_is_catalogue_only {
                violations.push("GAUTENG_STATION_METADATA_INVENTED_OR_PROMOTED");
            }
        }

        let noaa_record = find_by_id(items, "JOHANNESBURG_TAMBO_NOAA_GHCND");
        if !noaa_record.is_some_and(|item| {
            item.get("daily_summary_period_end").and_then(Value::as_str) == Some("2020-05-27")
                && item
                    .get("hourly_series_downloaded")
                    .and_then(Value::as_bool)
                    == Some(false)
        }) {
            violations.push("NOAA_GHCND_DAILY_RECORD_SCOPE_CHANGED");
        }

        let wigos_record = find_by_id(items, "JOHANNESBURG_TAMBO_SAWS_WIGOS_OSCAR");
        if !wigos_record.is_some_and(|item| {
            item.get("history_and_instrument_metadata")
                .and_then(Value::as_str)
                == Some("must_retrieve_directly_from_wmo_oscar_and_saws")
                && item
                    .get("reported_previous_to_current_distance_km")
                    .is_some()
        }) {
            violations.push("WIGOS_STATION_HISTORY_RECONCILIATION_GATE_REMOVED");
        }
    } else {
        violations.push("STATION_METADATA_CROSSCHECKS_MISSING");
    }
    push_if(
        &mut violations,
        station_metadata_count != 7,
        "STATION_METADATA_CROSSCHECK_COUNT_CHANGED",
    );

    // Network/archive summaries are separate from station observations.
    // In particular, ARC's operational network (~500) and wider archive (~1900)
    // must not be conflated, nor may any network total imply local station fitness.
    let station_network_profiles = array_at(scenario, &["station_network_source_profiles"]);
    let mut station_network_profile_count = 0usize;
    if let Some(items) = station_network_profiles {
        station_network_profile_count = items.len();

        let operational_arc = find_by_id(items, "ARC_OPERATIONAL_AGROCLIMATE_NETWORK");
        if !operational_arc.is_some_and(|item| {
            item.get("inventory_scope").and_then(Value::as_str)
                == Some("operational_national_agroclimate_network")
                && item
                    .get("reported_station_count_approx")
                    .and_then(Value::as_u64)
                    == Some(500)
                && item.get("status").and_then(Value::as_str)
                    == Some("NETWORK_SCOPE_METADATA_ONLY_NOT_STATION_QUALIFICATION")
        }) {
            violations.push("ARC_OPERATIONAL_NETWORK_SCOPE_CHANGED");
        }

        let arc_archive = find_by_id(items, "ARC_AGROCLIMATE_WEB_PORTAL_ARCHIVE");
        if !arc_archive.is_some_and(|item| {
            item.get("reported_total_stations_approx")
                .and_then(Value::as_u64)
                == Some(1900)
                && item.get("reported_manual_stations").and_then(Value::as_u64) == Some(1100)
                && item
                    .get("reported_automatic_weather_stations")
                    .and_then(Value::as_u64)
                    == Some(763)
                && item
                    .get("reported_active_automatic_stations")
                    .and_then(Value::as_u64)
                    == Some(460)
                && item.get("status").and_then(Value::as_str)
                    == Some("ARCHIVE_INVENTORY_METADATA_ONLY_NOT_STATION_QUALIFICATION")
                && item.get("data_access_status").and_then(Value::as_str)
                    == Some("paid_ad_hoc_or_subscription")
        }) {
            violations.push("ARC_ARCHIVE_INVENTORY_SCOPE_OR_ACCESS_CHANGED");
        }

        let saws = find_by_id(items, "SAWS_WIS2_FIXED_LAND_SYNOP");
        if !saws.is_some_and(|item| {
            item.get("product").and_then(Value::as_str)
                == Some("hourly_synoptic_observations_fixed_land_stations")
                && item
                    .get("access_and_license_status")
                    .and_then(Value::as_str)
                    == Some("must_check_product_topic_specific_license_and_reuse_terms")
                && item
                    .get("measurement_messages_downloaded")
                    .and_then(Value::as_bool)
                    == Some(false)
        }) {
            violations.push("SAWS_WIS_PRODUCT_OR_LICENSE_SCOPE_CHANGED");
        }

        let noaa = find_by_id(items, "NOAA_NCEI_GLOBAL_HOURLY_ISD");
        if !noaa.is_some_and(|item| {
            item.get("local_johannesburg_series_downloaded")
                .and_then(Value::as_bool)
                == Some(false)
                && item
                    .get("local_station_history_reconciled")
                    .and_then(Value::as_bool)
                    == Some(false)
                && item.get("status").and_then(Value::as_str)
                    == Some("GLOBAL_ARCHIVE_PRODUCT_METADATA_ONLY_NOT_LOCAL_SERIES_QUALIFICATION")
        }) {
            violations.push("NOAA_GLOBAL_ARCHIVE_PROMOTED_TO_LOCAL_SERIES");
        }
    } else {
        violations.push("STATION_NETWORK_PROFILES_MISSING");
    }
    push_if(
        &mut violations,
        station_network_profile_count != 4,
        "STATION_NETWORK_PROFILE_COUNT_CHANGED",
    );

    // Preserve the existing geometry and solver authority gates.
    push_if(
        &mut violations,
        string_at(
            scenario,
            &["solver_contracts", "geometry_authority", "status"],
        ) != Some("EXACT_GEOMETRY_PROVIDER_REQUIRED"),
        "GEOMETRY_PROVIDER_GATE_REMOVED",
    );
    push_if(
        &mut violations,
        string_at(scenario, &["solver_contracts", "thermal_energy", "status"])
            != Some("USE_VERSION_PINNED_QUALIFIED_ADAPTER")
            || string_at(
                scenario,
                &["solver_contracts", "thermal_energy", "engine_candidate"],
            ) != Some("EnergyPlus 26.2.0"),
        "THERMAL_ADAPTER_REQUIREMENT_CHANGED",
    );
    push_if(
        &mut violations,
        string_at(scenario, &["solver_contracts", "structural", "status"])
            != Some("NOT_SELECTED_REQUIRES_REFERENCE_CASE_AND_PROFESSIONAL_OWNER")
            || !is_null_at(scenario, &["solver_contracts", "structural", "engine"]),
        "STRUCTURAL_SOLVER_PROMOTED",
    );
    push_if(
        &mut violations,
        string_at(scenario, &["solver_contracts", "fire_egress", "status"])
            != Some("QUALIFIED_FIRE_PROFESSIONAL_REVIEW_REQUIRED")
            || !is_null_at(scenario, &["solver_contracts", "fire_egress", "engine"]),
        "FIRE_REVIEW_GATE_REMOVED",
    );

    // Parameter bounds are search envelopes, never legal or engineering limits.
    push_if(
        &mut violations,
        string_at(parameters, &["revision_status"]) != Some("E0_SYNTHETIC_SEARCH_BOUNDS")
            || string_at(parameters, &["claim_ceiling"])
                != Some("design_space_exploration_only_not_building_dimensions_or_code_limits")
            || string_at(parameters, &["bounds_policy", "all_numeric_ranges_are"])
                != Some("SYNTHETIC_SEARCH_BOUNDS"),
        "PARAMETER_BOUNDS_CLAIM_CEILING_CHANGED",
    );
    let prohibited_bounds = [
        "structural_limits",
        "statutory_minimums_or_maximums",
        "site_setbacks_or_zoning_envelope",
        "fire_or_accessibility_dimensions",
        "verified_optimal_values",
    ];
    let prohibited = array_at(parameters, &["bounds_policy", "not_interpretable_as"]);
    push_if(
        &mut violations,
        prohibited_bounds
            .iter()
            .any(|required| !array_contains(prohibited, required)),
        "PARAMETER_BOUNDS_PROHIBITION_INCOMPLETE",
    );

    // Check every typed variable collection, not just the topology block.
    // This prevents invalid dimensions in envelope, materials or building systems
    // from escaping validation.
    let variable_collections = [
        "topology_variables",
        "envelope_variables",
        "materials_and_production_variables",
        "building_system_variables",
    ];
    let mut variable_ids = std::collections::HashSet::new();
    for collection in variable_collections {
        if let Some(items) = array_at(parameters, &[collection]) {
            for item in items {
                let id = item.get("id").and_then(Value::as_str);
                if id.is_none_or(str::is_empty) || !variable_ids.insert(id.unwrap_or_default()) {
                    violations.push("PARAMETER_ID_MISSING_OR_DUPLICATED");
                }
                match item.get("kind").and_then(Value::as_str) {
                    Some("continuous_design_variable") => {
                        continuous_count += 1;
                        let min = item.get("minimum").and_then(Value::as_f64);
                        let max = item.get("maximum").and_then(Value::as_f64);
                        let unit = item.get("unit").and_then(Value::as_str);
                        if !matches!((min, max), (Some(a), Some(b)) if a.is_finite() && b.is_finite() && a <= b)
                            || unit.is_none_or(str::is_empty)
                        {
                            violations.push("CONTINUOUS_PARAMETER_BOUNDS_INVALID");
                        }
                    }
                    Some("categorical_design_variable")
                    | Some("categorical_preference")
                    | Some("categorical_hard_requirement") => {
                        categorical_count += 1;
                        if item
                            .get("values")
                            .and_then(Value::as_array)
                            .is_none_or(Vec::is_empty)
                        {
                            violations.push("CATEGORICAL_PARAMETER_VALUES_EMPTY");
                        }
                    }
                    Some("qualified_product_reference") => {
                        let values_empty = item
                            .get("values")
                            .and_then(Value::as_array)
                            .is_some_and(Vec::is_empty);
                        if !values_empty
                            || item.get("status").and_then(Value::as_str)
                                != Some("UNKNOWN_UNTIL_PRODUCT_AND_ASSEMBLY_EVIDENCE")
                        {
                            violations.push("UNQUALIFIED_PRODUCT_REFERENCE_PROMOTED");
                        }
                    }
                    Some("qualified_evidence_reference") => {
                        let values_empty = item
                            .get("values")
                            .and_then(Value::as_array)
                            .is_some_and(Vec::is_empty);
                        if !values_empty
                            || item.get("status").and_then(Value::as_str)
                                != Some("UNKNOWN_UNTIL_SUPPLIER_LOT_AND_QUALIFICATION")
                        {
                            violations.push("UNQUALIFIED_MATERIAL_EVIDENCE_PROMOTED");
                        }
                    }
                    _ => violations.push("UNKNOWN_PARAMETER_KIND"),
                }
            }
        } else {
            violations.push("PARAMETER_COLLECTION_MISSING");
        }
    }
    push_if(
        &mut violations,
        continuous_count == 0 || categorical_count == 0,
        "PARAMETER_SPACE_INCOMPLETE",
    );

    // Validate the task-level relationship graph without interpreting it as geometry.
    // The graph may state a route or adjacency requirement, but this checker does
    // not calculate distances, paths, gradients, acoustic isolation or clearances.
    let functional_programme = array_at(scenario, &["functional_programme"]);
    let mut known_functions = std::collections::HashSet::new();
    let mut functional_programme_ids_valid = true;
    if let Some(items) = functional_programme {
        for item in items {
            let id = item.get("id").and_then(Value::as_str);
            if id.is_none_or(str::is_empty) || !known_functions.insert(id.unwrap_or_default()) {
                functional_programme_ids_valid = false;
            }
        }
    } else {
        functional_programme_ids_valid = false;
    }
    push_if(
        &mut violations,
        !functional_programme_ids_valid || known_functions.is_empty(),
        "FUNCTIONAL_PROGRAMME_IDS_MISSING_OR_DUPLICATED",
    );
    let mut relationship_ids = std::collections::HashSet::new();
    const EXPECTED_RELATIONSHIPS: [(&str, &str, &str, &str, &str, Option<&str>); 11] = [
        (
            "REL-001",
            "arrival",
            "great_hall",
            "must_have_step_free_route",
            "hard",
            None,
        ),
        (
            "REL-002",
            "arrival",
            "kitchen_dining",
            "must_have_step_free_route",
            "hard",
            None,
        ),
        (
            "REL-003",
            "arrival",
            "accessible_washroom",
            "must_have_arrival_level_access",
            "hard",
            None,
        ),
        (
            "REL-004",
            "arrival",
            "work_making",
            "must_have_step_free_route",
            "hard",
            None,
        ),
        (
            "REL-005",
            "arrival",
            "outdoor_living",
            "must_have_step_free_route",
            "hard",
            None,
        ),
        (
            "REL-006",
            "arrival",
            "great_hall",
            "visitor_route_must_avoid_private_zone",
            "hard",
            Some("sleeping_rooms"),
        ),
        (
            "REL-007",
            "service_plant",
            "arrival",
            "maintenance_access_must_avoid_private_zone",
            "hard",
            Some("sleeping_rooms"),
        ),
        (
            "REL-008",
            "kitchen_dining",
            "outdoor_living",
            "should_be_adjacent",
            "preference",
            None,
        ),
        (
            "REL-009",
            "library_quiet",
            "great_hall",
            "should_be_acoustically_separated",
            "preference",
            None,
        ),
        (
            "REL-010",
            "sleeping_rooms",
            "great_hall",
            "should_be_acoustically_separated",
            "preference",
            None,
        ),
        (
            "REL-011",
            "work_making",
            "service_plant",
            "should_be_adjacent",
            "preference",
            None,
        ),
    ];

    const HARD_RELATIONS: [&str; 4] = [
        "must_have_step_free_route",
        "must_have_arrival_level_access",
        "visitor_route_must_avoid_private_zone",
        "maintenance_access_must_avoid_private_zone",
    ];
    const PREFERENCE_RELATIONS: [&str; 2] =
        ["should_be_adjacent", "should_be_acoustically_separated"];
    const ALLOWED_RELATIONSHIP_KEYS: [&str; 7] = [
        "id",
        "source",
        "target",
        "relation",
        "via_excluded",
        "requirement_class",
        "reason",
    ];

    if let Some(items) = functional_relationships {
        functional_relationship_count = items.len();
        for item in items {
            let id = item.get("id").and_then(Value::as_str);
            let source = item.get("source").and_then(Value::as_str);
            let target = item.get("target").and_then(Value::as_str);
            let relation = item.get("relation").and_then(Value::as_str);
            let requirement_class = item.get("requirement_class").and_then(Value::as_str);
            let reason = item.get("reason").and_then(Value::as_str);

            if id.is_none_or(str::is_empty) || !relationship_ids.insert(id.unwrap_or_default()) {
                violations.push("FUNCTIONAL_RELATIONSHIP_ID_MISSING_OR_DUPLICATED");
            }
            if source.is_none_or(|value| !known_functions.contains(value))
                || target.is_none_or(|value| !known_functions.contains(value))
            {
                violations.push("FUNCTIONAL_RELATIONSHIP_ENDPOINT_UNKNOWN");
            }
            let expected = id.and_then(|relationship_id| {
                EXPECTED_RELATIONSHIPS
                    .iter()
                    .find(|expected| expected.0 == relationship_id)
            });
            let identity_matches = expected.is_some_and(|expected| {
                source == Some(expected.1)
                    && target == Some(expected.2)
                    && relation == Some(expected.3)
                    && requirement_class == Some(expected.4)
            });
            if !identity_matches {
                violations.push("FUNCTIONAL_RELATIONSHIP_STABLE_ID_CONTRACT_CHANGED");
            }
            if let Some(expected) = expected {
                let actual_excluded = item
                    .get("via_excluded")
                    .and_then(Value::as_array)
                    .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>())
                    .unwrap_or_default();
                let exclusions_match = match expected.5 {
                    Some(function_id) => actual_excluded.as_slice() == [function_id],
                    None => actual_excluded.is_empty(),
                };
                if !exclusions_match {
                    violations.push("FUNCTIONAL_RELATIONSHIP_EXCLUSION_CONTRACT_CHANGED");
                }
            }
            if reason.is_none_or(str::is_empty) {
                violations.push("FUNCTIONAL_RELATIONSHIP_REASON_MISSING");
            }

            let class_matches = match relation {
                Some(value) if HARD_RELATIONS.contains(&value) => requirement_class == Some("hard"),
                Some(value) if PREFERENCE_RELATIONS.contains(&value) => {
                    requirement_class == Some("preference")
                }
                _ => false,
            };
            if !class_matches {
                violations.push("FUNCTIONAL_RELATIONSHIP_CLASS_OR_TYPE_INVALID");
            }

            let requires_excluded = matches!(
                relation,
                Some("visitor_route_must_avoid_private_zone")
                    | Some("maintenance_access_must_avoid_private_zone")
            );
            if requires_excluded
                && item
                    .get("via_excluded")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty)
            {
                violations.push("FUNCTIONAL_RELATIONSHIP_REQUIRED_EXCLUSIONS_MISSING");
            }

            if let Some(excluded) = item.get("via_excluded") {
                if let Some(excluded_items) = excluded.as_array() {
                    if excluded_items.iter().any(|entry| {
                        entry
                            .as_str()
                            .is_none_or(|value| !known_functions.contains(value))
                    }) {
                        violations.push("FUNCTIONAL_RELATIONSHIP_EXCLUDED_FUNCTION_UNKNOWN");
                    }
                } else {
                    violations.push("FUNCTIONAL_RELATIONSHIP_EXCLUDED_FUNCTIONS_INVALID");
                }
            }

            if let Some(object) = item.as_object() {
                if object
                    .keys()
                    .any(|key| !ALLOWED_RELATIONSHIP_KEYS.contains(&key.as_str()))
                {
                    violations.push("FUNCTIONAL_RELATIONSHIP_GEOMETRY_OR_UNVERSIONED_FIELD");
                }
            } else {
                violations.push("FUNCTIONAL_RELATIONSHIP_RECORD_NOT_OBJECT");
            }
        }
    } else {
        violations.push("FUNCTIONAL_RELATIONSHIP_GRAPH_MISSING");
    }

    push_if(
        &mut violations,
        functional_relationship_count != 11
            || relationship_ids.len() != functional_relationship_count,
        "FUNCTIONAL_RELATIONSHIP_GRAPH_COUNT_OR_IDENTITY_INVALID",
    );

    // Hard requirements remain a separate authority class.
    let hard_requirements = array_at(parameters, &["hard_requirements"]);
    push_if(
        &mut violations,
        hard_requirements.is_none_or(Vec::is_empty)
            || hard_requirements.is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("owner")
                        .and_then(Value::as_str)
                        .is_none_or(str::is_empty)
                        || item
                            .get("evidence_state")
                            .and_then(Value::as_str)
                            .is_none_or(|state| state == "verified" || state == "approved")
                })
            }),
        "HARD_REQUIREMENT_AUTHORITY_OR_STATE_INVALID",
    );

    let uncertainty = array_at(parameters, &["uncertainties_to_sweep"]);
    push_if(
        &mut violations,
        uncertainty.is_none_or(Vec::is_empty),
        "UNCERTAINTY_REGISTER_MISSING",
    );
    let invalidators = array_at(parameters, &["invalidators"]);
    push_if(
        &mut violations,
        invalidators.is_none_or(Vec::is_empty),
        "INVALIDATION_REGISTER_MISSING",
    );

    // Optimization must keep a Pareto set and non-compensatory constraints.
    push_if(
        &mut violations,
        string_at(parameters, &["optimization_policy", "hard_constraints"])
            != Some("non_compensatory")
            || !string_at(parameters, &["optimization_policy", "aggregation"])
                .is_some_and(|v| v.contains("pareto") && v.contains("no_universal_weighted_best")),
        "OPTIMIZATION_POLICY_COMPENSATES_HARD_CONSTRAINTS",
    );
    if let Some(objectives) = array_at(parameters, &["optimization_policy", "soft_objectives"]) {
        let forbidden = [
            "fire_safety",
            "fire_egress",
            "structural_approval",
            "code_compliance",
        ];
        push_if(
            &mut violations,
            objectives.iter().any(|item| {
                item.as_str()
                    .is_some_and(|value| forbidden.contains(&value))
            }),
            "HARD_AUTHORITY_MOVED_TO_SOFT_OBJECTIVES",
        );
    } else {
        violations.push("SOFT_OBJECTIVE_VECTOR_MISSING");
    }

    let unresolved = array_at(scenario, &["unresolved_inputs"]);
    push_if(
        &mut violations,
        unresolved.is_none_or(Vec::is_empty),
        "UNRESOLVED_INPUTS_NOT_PRESERVED",
    );

    // Stable order and no duplicate violation IDs make receipts reviewable.
    let mut unique = std::collections::HashSet::new();
    violations.retain(|violation| unique.insert(*violation));

    ValidationReport {
        scenario_id,
        alternatives: family_count,
        weather_candidates: weather_count,
        continuous_variables: continuous_count,
        categorical_variables: categorical_count,
        functional_relationships: functional_relationship_count,
        alternative_climate_methods: alternative_method_count,
        station_metadata_crosschecks: station_metadata_count,
        station_network_profiles: station_network_profile_count,
        violations,
    }
}

/// Parse and validate the two tracked source documents.
pub fn validate_tracked_documents() -> Result<ValidationReport, String> {
    let scenario: Value = serde_json::from_str(SCENARIO_JSON)
        .map_err(|error| format!("SCENARIO_JSON_INVALID: {error}"))?;
    let parameters: Value = serde_json::from_str(PARAMETER_JSON)
        .map_err(|error| format!("PARAMETER_JSON_INVALID: {error}"))?;
    Ok(validate_values(&scenario, &parameters))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn documents() -> (Value, Value) {
        (
            serde_json::from_str(SCENARIO_JSON).expect("scenario JSON must parse"),
            serde_json::from_str(PARAMETER_JSON).expect("parameter JSON must parse"),
        )
    }

    fn contains(report: &ValidationReport, id: &str) -> bool {
        report.violations.contains(&id)
    }

    #[test]
    fn tracked_synthetic_documents_are_internally_consistent() {
        let report = validate_tracked_documents().expect("tracked JSON should parse");
        assert!(report.is_valid(), "{:?}", report.violations);
        assert_eq!(report.scenario_id, SCENARIO_ID);
        assert_eq!(report.alternatives, 4);
        assert_eq!(report.weather_candidates, 2);
        assert!(report.continuous_variables > 0);
        assert!(report.categorical_variables > 0);
    }

    #[test]
    fn tracked_functional_relationship_graph_has_eleven_relationships() {
        let report = validate_tracked_documents().expect("tracked JSON should parse");
        assert!(report.is_valid(), "{:?}", report.violations);
        assert_eq!(report.functional_relationships, 11);
    }

    #[test]
    fn functional_relationship_unknown_endpoint_is_rejected() {
        let (mut scenario, parameters) = documents();
        scenario["functional_relationships"][0]["target"] = json!("nonexistent_room");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_RELATIONSHIP_ENDPOINT_UNKNOWN"
        ));
    }

    #[test]
    fn functional_programme_ids_must_be_unique() {
        let (mut scenario, parameters) = documents();
        scenario["functional_programme"][1]["id"] = json!("arrival");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_PROGRAMME_IDS_MISSING_OR_DUPLICATED"
        ));
    }

    #[test]
    fn stable_relationship_id_cannot_be_repurposed_for_another_target() {
        let (mut scenario, parameters) = documents();
        scenario["functional_relationships"][0]["target"] = json!("kitchen_dining");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_RELATIONSHIP_STABLE_ID_CONTRACT_CHANGED"
        ));
    }

    #[test]
    fn private_route_exclusion_set_must_match_the_declared_contract() {
        let (mut scenario, parameters) = documents();
        scenario["functional_relationships"][5]["via_excluded"] =
            json!(["sleeping_rooms", "library_quiet"]);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_RELATIONSHIP_EXCLUSION_CONTRACT_CHANGED"
        ));
    }

    #[test]
    fn functional_relationship_ids_must_be_unique() {
        let (mut scenario, parameters) = documents();
        scenario["functional_relationships"][1]["id"] = json!("REL-001");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_RELATIONSHIP_ID_MISSING_OR_DUPLICATED"
        ));
    }

    #[test]
    fn hard_and_preference_relationships_cannot_be_reclassified() {
        let (mut scenario, parameters) = documents();
        scenario["functional_relationships"][0]["requirement_class"] = json!("preference");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_RELATIONSHIP_CLASS_OR_TYPE_INVALID"
        ));
    }

    #[test]
    fn excluded_private_functions_must_exist_in_the_programme() {
        let (mut scenario, parameters) = documents();
        scenario["functional_relationships"][5]["via_excluded"][0] = json!("unknown_private_room");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_RELATIONSHIP_EXCLUDED_FUNCTION_UNKNOWN"
        ));
    }

    #[test]
    fn private_route_relationship_requires_explicit_excluded_functions() {
        let (mut scenario, parameters) = documents();
        scenario["functional_relationships"][5]
            .as_object_mut()
            .expect("relationship object")
            .remove("via_excluded");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_RELATIONSHIP_REQUIRED_EXCLUSIONS_MISSING"
        ));
    }

    #[test]
    fn relationship_graph_does_not_accept_unversioned_geometry_fields() {
        let (mut scenario, parameters) = documents();
        scenario["functional_relationships"][0]["distance_m"] = json!(4.5);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "FUNCTIONAL_RELATIONSHIP_GEOMETRY_OR_UNVERSIONED_FIELD"
        ));
    }

    #[test]
    fn populated_parcel_or_coordinates_are_rejected() {
        let (mut scenario, parameters) = documents();
        scenario["geography"]["parcel_id"] = json!("REAL-PARCEL-123");
        scenario["geography"]["coordinates"] = json!([-26.2, 28.0]);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "REAL_SITE_EVIDENCE_PROMOTED"));
    }

    #[test]
    fn missing_synthetic_site_label_is_rejected() {
        let (mut scenario, parameters) = documents();
        scenario["synthetic_site_envelope"]["label"] = json!("SITE-PLAN");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "SYNTHETIC_SITE_LABEL_MISSING_OR_CHANGED"));
    }

    #[test]
    fn missing_or_duplicate_alternative_family_is_rejected() {
        let (mut scenario, parameters) = documents();
        scenario["architectural_alternatives"]
            .as_array_mut()
            .expect("alternatives array")
            .pop();
        let missing = validate_values(&scenario, &parameters);
        assert!(contains(&missing, "ALTERNATIVE_FAMILY_SET_INVALID"));

        let (mut scenario, parameters) = documents();
        scenario["architectural_alternatives"][1]["id"] = json!("A_COURTYARD_CLOISTER");
        let duplicate = validate_values(&scenario, &parameters);
        assert!(contains(&duplicate, "ALTERNATIVE_FAMILY_SET_INVALID"));
    }

    #[test]
    fn family_parameter_mismatch_is_rejected() {
        let (scenario, mut parameters) = documents();
        parameters["topology_variables"][0]["values"][3] = json!("castle");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "SPATIAL_FAMILY_PARAMETER_SET_INVALID"));
    }

    #[test]
    fn alternative_id_must_bind_to_its_declared_spatial_family() {
        let (mut scenario, parameters) = documents();
        scenario["architectural_alternatives"][0]["family"] = json!("garden_pavilions");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "ALTERNATIVE_TO_PARAMETER_FAMILY_BINDING_INVALID"
        ));
    }

    #[test]
    fn envelope_continuous_variables_are_checked_too() {
        let (scenario, mut parameters) = documents();
        let items = parameters["envelope_variables"]
            .as_array_mut()
            .expect("envelope variable array");
        let idx = items
            .iter()
            .position(|item| item["id"] == "north_facade_glazing_ratio")
            .expect("north glazing variable");
        items[idx]["minimum"] = json!(0.9);
        items[idx]["maximum"] = json!(0.1);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "CONTINUOUS_PARAMETER_BOUNDS_INVALID"));
    }

    #[test]
    fn reversed_continuous_bounds_are_rejected() {
        let (scenario, mut parameters) = documents();
        let items = parameters["topology_variables"]
            .as_array_mut()
            .expect("topology variable array");
        let idx = items
            .iter()
            .position(|item| item["id"] == "building_orientation")
            .expect("orientation variable");
        items[idx]["minimum"] = json!(360.0);
        items[idx]["maximum"] = json!(0.0);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "CONTINUOUS_PARAMETER_BOUNDS_INVALID"));
    }

    #[test]
    fn empty_categorical_choices_are_rejected() {
        let (scenario, mut parameters) = documents();
        parameters["topology_variables"][0]["values"] = json!([]);
        let report = validate_values(&scenario, &parameters);
        assert!(
            contains(&report, "CATEGORICAL_PARAMETER_VALUES_EMPTY")
                || contains(&report, "SPATIAL_FAMILY_PARAMETER_SET_INVALID")
        );
    }

    #[test]
    fn tmy_candidate_cannot_be_promoted_without_qualification_evidence() {
        let (mut scenario, parameters) = documents();
        scenario["climate_inputs"]["candidate_weather_files"][0]["status"] = json!("QUALIFIED");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "WEATHER_CANDIDATE_PROMOTED_OR_INCOMPLETE"
        ));
    }

    #[test]
    fn all_five_joburg_tmyx_candidates_are_required() {
        let (mut scenario, parameters) = documents();
        scenario["climate_inputs"]["candidate_weather_files"]
            .as_array_mut()
            .expect("weather candidates")
            .pop();
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "WEATHER_CANDIDATE_COUNT_CHANGED"));
    }

    #[test]
    fn weather_reuse_rights_cannot_be_silently_promoted() {
        let (mut scenario, parameters) = documents();
        scenario["climate_inputs"]["candidate_weather_files"][0]["reuse_rights"] =
            json!("CC-BY");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "WEATHER_CANDIDATE_PROMOTED_OR_INCOMPLETE"
        ));
    }

    #[test]
    fn candidate_weather_ensemble_cannot_be_collapsed_to_a_parcel_station() {
        let (mut scenario, parameters) = documents();
        scenario["climate_inputs"]["station_ensemble_policy"]["selected_site_station"] =
            json!("JOBURG_TAMBO_TMYX_2011_2025");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "STATION_ENSEMBLE_POLICY_INVALID"));
    }

    #[test]
    fn arbitrary_tmy_file_cannot_replace_the_two_catalogued_candidates() {
        let (mut scenario, parameters) = documents();
        scenario["climate_inputs"]["candidate_weather_files"][0]["filename"] =
            json!("OTHER_CITY_TMYx.zip");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "WEATHER_CANDIDATE_PROMOTED_OR_INCOMPLETE"
        ));
    }

    #[test]
    fn qualified_material_reference_cannot_be_assumed_without_lot_evidence() {
        let (scenario, mut parameters) = documents();
        let items = parameters["materials_and_production_variables"]
            .as_array_mut()
            .expect("materials and production variables");
        let idx = items
            .iter()
            .position(|item| item["id"] == "material_source_and_grade")
            .expect("material source and grade reference");
        items[idx]["status"] = json!("APPROVED");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "UNQUALIFIED_MATERIAL_EVIDENCE_PROMOTED"));
    }

    #[test]
    fn non_finite_or_unitless_continuous_values_are_rejected() {
        let (scenario, mut parameters) = documents();
        let items = parameters["envelope_variables"]
            .as_array_mut()
            .expect("envelope variable array");
        let idx = items
            .iter()
            .position(|item| item["id"] == "north_facade_glazing_ratio")
            .expect("north glazing variable");
        items[idx]["maximum"] = Value::Null;
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "CONTINUOUS_PARAMETER_BOUNDS_INVALID"));

        let (scenario, mut parameters) = documents();
        let items = parameters["envelope_variables"]
            .as_array_mut()
            .expect("envelope variable array");
        let idx = items
            .iter()
            .position(|item| item["id"] == "north_facade_glazing_ratio")
            .expect("north glazing variable");
        items[idx]
            .as_object_mut()
            .expect("parameter object")
            .remove("unit");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "CONTINUOUS_PARAMETER_BOUNDS_INVALID"));
    }

    #[test]
    fn alternate_provider_remains_unqualified_until_local_source_review() {
        let (mut scenario, parameters) = documents();
        scenario["alternative_climate_provider_methodologies"][0]["status"] = json!("QUALIFIED");
        scenario["alternative_climate_provider_methodologies"][0]["local_validation_status"] =
            json!("VERIFIED");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "ALTERNATIVE_CLIMATE_PROVIDER_PROMOTED_OR_INCOMPLETE"
        ));
    }

    #[test]
    fn catalogue_only_station_codes_cannot_gain_unverified_coordinates() {
        let (mut scenario, parameters) = documents();
        scenario["station_metadata_crosschecks"][4]["latitude_deg"] = json!(-26.0);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "GAUTENG_STATION_METADATA_INVENTED_OR_PROMOTED"
        ));
    }

    #[test]
    fn arc_operational_network_and_archive_inventory_are_not_conflated() {
        let (mut scenario, parameters) = documents();
        scenario["station_network_source_profiles"][1]["reported_active_automatic_stations"] =
            json!(763);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "ARC_ARCHIVE_INVENTORY_SCOPE_OR_ACCESS_CHANGED"
        ));
    }

    #[test]
    fn station_network_summaries_cannot_be_promoted_to_local_measurements() {
        let (mut scenario, parameters) = documents();
        scenario["station_network_source_profiles"][2]["measurement_messages_downloaded"] =
            json!(true);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "SAWS_WIS_PRODUCT_OR_LICENSE_SCOPE_CHANGED"
        ));

        let (mut scenario, parameters) = documents();
        scenario["station_network_source_profiles"][3]["local_johannesburg_series_downloaded"] =
            json!(true);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "NOAA_GLOBAL_ARCHIVE_PROMOTED_TO_LOCAL_SERIES"
        ));
    }

    #[test]
    fn station_metadata_remains_unqualified_until_identity_history_is_reconciled() {
        let (mut scenario, parameters) = documents();
        scenario["station_metadata_crosschecks"][0]["status"] = json!("QUALIFIED");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "STATION_METADATA_CROSSCHECK_PROMOTED_OR_INCOMPLETE"
        ));
    }

    #[test]
    fn ncei_daily_record_cannot_be_promoted_to_hourly_series() {
        let (mut scenario, parameters) = documents();
        scenario["station_metadata_crosschecks"][2]["hourly_series_downloaded"] = json!(true);
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "NOAA_GHCND_DAILY_RECORD_SCOPE_CHANGED"));
    }

    #[test]
    fn grid_climate_trend_cannot_be_promoted_to_site_design_weather() {
        let (mut scenario, parameters) = documents();
        scenario["alternative_climate_provider_methodologies"][1]["status"] =
            json!("SITE_DESIGN_WEATHER");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "REGIONAL_CLIMATE_TREND_SCOPE_PROMOTED"));
    }

    #[test]
    fn tmy_cannot_be_relabelled_as_a_heatwave_profile() {
        let (mut scenario, parameters) = documents();
        scenario["climate_inputs"]["candidate_weather_files"][0]["dataset_type"] =
            json!("HEATWAVE_DESIGN_FILE");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(
            &report,
            "WEATHER_CANDIDATE_PROMOTED_OR_INCOMPLETE"
        ));
    }

    #[test]
    fn geometry_or_structural_solver_cannot_be_promoted_to_approval() {
        let (mut scenario, parameters) = documents();
        scenario["solver_contracts"]["geometry_authority"]["status"] = json!("APPROVED");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "GEOMETRY_PROVIDER_GATE_REMOVED"));
    }

    #[test]
    fn evidence_tier_promotion_is_rejected() {
        let (mut scenario, parameters) = documents();
        scenario["evidence_tier"] = json!("E3");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "SCENARIO_EVIDENCE_TIER_PROMOTED"));
    }

    #[test]
    fn hard_life_safety_gate_cannot_become_a_soft_objective() {
        let (scenario, mut parameters) = documents();
        parameters["optimization_policy"]["soft_objectives"]
            .as_array_mut()
            .expect("soft objectives")
            .push(json!("fire_safety"));
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "HARD_AUTHORITY_MOVED_TO_SOFT_OBJECTIVES"));
    }

    #[test]
    fn unqualified_product_assembly_cannot_be_assumed() {
        let (scenario, mut parameters) = documents();
        let items = parameters["envelope_variables"]
            .as_array_mut()
            .expect("envelope variable array");
        let idx = items
            .iter()
            .position(|item| item["id"] == "insulation_assembly")
            .expect("insulation field");
        items[idx]["status"] = json!("APPROVED");
        let report = validate_values(&scenario, &parameters);
        assert!(contains(&report, "UNQUALIFIED_PRODUCT_REFERENCE_PROMOTED"));
    }
}
