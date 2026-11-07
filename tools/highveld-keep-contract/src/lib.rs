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
        weather_count != 2,
        "WEATHER_CANDIDATE_COUNT_CHANGED",
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
