//! Semantic cross-owner validation for the South African Highveld resilience binding.
//!
//! This checks the tracked E0 adapter contract against reviewed owner identities and
//! non-promotion rules. It is not a general JSON Schema implementation, raw duplicate-key
//! detector, climate-file inspector, thermal solver, or professional approval authority.

use serde_json::Value;
use std::collections::HashSet;

pub const RESILIENCE_BINDING_JSON: &str =
    include_str!("../../../docs/research/highveld-keep/HIGHVELD_KEEP_RESILIENCE_BINDING_V0_1.json");
pub const RESILIENCE_BINDING_SCHEMA: &str = include_str!(
    "../../../docs/research/highveld-keep/HIGHVELD_KEEP_RESILIENCE_BINDING_V0_1.schema.json"
);

const BINDING_ID: &str = "HIGHVELD-KEEP-RESILIENCE-BINDING-001";
const SCHEMA_VERSION: &str = "luminous.highveld.resilience-binding/0.1.0";
const CLAIM_CEILING: &str = "adapter_contract_and_synthetic_binding_semantics_only_no_physical_building_performance_or_safety_claim";

const SCENARIOS: [(&str, &str); 10] = [
    ("HK-RES-001", "typical_year_screening"),
    ("HK-RES-002", "observed_historical_hot_spell_and_hot_night"),
    ("HK-RES-003", "observed_historical_cold_night"),
    ("HK-RES-004", "future_climate_sensitivity"),
    ("HK-RES-005", "grid_outage_during_hot_spell"),
    ("HK-RES-006", "water_supply_interruption"),
    ("HK-RES-007", "severe_rainfall_hail_wind_and_stormwater"),
    ("HK-RES-008", "air_quality_constrained_ventilation"),
    ("HK-RES-009", "equipment_control_fault_and_repair_delay"),
    ("HK-RES-010", "compound_heat_grid_water_and_repair_cascade"),
];

const WEATHER_CANDIDATES: [(&str, &str); 5] = [
    ("JOBURG_TAMBO_TMYX_2011_2025", "683680"),
    ("JOBURG_BOTANICAL_GARDENS_TMYX_2011_2025", "683610"),
    ("JOBURG_GRAND_CENTRAL_TMYX_2011_2025", "682650"),
    ("JOBURG_RAND_AIRPORT_TMYX_2011_2025", "683463"),
    ("JOBURG_LANSERIA_INTL_TMYX_2011_2025", "682635"),
];

const OWNER_CONTRACTS: [&str; 6] = [
    "SPATIAL-RESILIENCE-001",
    "ENERGYPLUS-QUAL-001",
    "BUILT-ENVIRONMENT-ADAPTER-BOUNDARIES",
    "SPATIAL-VOI-001",
    "CITY-HEAT-RETROFIT-ATLAS-001",
    "CITY-HEAT-WEATHER-QUAL-001",
];

const SERVICE_DOMAINS: [&str; 7] = [
    "resident_task_service",
    "thermal_environment",
    "electricity",
    "water",
    "ventilation_and_indoor_air_quality",
    "accessibility_and_egress",
    "repair_and_recovery",
];

const INVARIANTS: [&str; 16] = [
    "paired_trace_identical",
    "hard_floor_before_pareto",
    "unknown_not_pass",
    "scope_revisions_bind_evidence",
    "explicit_units_required",
    "stable_recovery_window_required",
    "missing_part_blocks_recovery",
    "no_future_information",
    "no_scalar_compensation",
    "uncertainty_tolerance_enforced",
    "resident_priority_preserved",
    "service_access_is_constraint",
    "duplicate_delivery_idempotent",
    "unrelated_change_preserves_current_result",
    "dissent_not_collapsed",
    "solver_claim_ceiling_preserved",
];

const VVUQ_STAGES: [&str; 6] = [
    "software_capability",
    "input_and_scope_validation",
    "numerical_verification",
    "independent_reference_comparison",
    "calibration_and_uncertainty",
    "professional_and_field_review",
];

const MEASUREMENT_QUESTIONS: [&str; 5] = ["VOI-01", "VOI-02", "VOI-03", "VOI-04", "VOI-05"];

const INVALIDATORS: [&str; 6] = [
    "weather_file_or_source_revision",
    "geometry_or_site_orientation_revision",
    "occupancy_or_resident_requirement_revision",
    "material_or_product_revision",
    "solver_or_model_revision",
    "outage_water_or_fault_trace_revision",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResilienceBindingReport {
    pub binding_id: String,
    pub scenario_families: usize,
    pub weather_candidates: usize,
    pub service_domains: usize,
    pub owner_bindings: usize,
    pub invariant_mappings: usize,
    pub vvuq_stages: usize,
    pub measurement_questions: usize,
    pub violations: Vec<&'static str>,
}

impl ResilienceBindingReport {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }
}

fn string_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_at(value: &Value, key: &str) -> Option<&Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn has_string(items: Option<&Vec<Value>>, expected: &str) -> bool {
    items.is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn push(violations: &mut Vec<&'static str>, invalid: bool, code: &'static str) {
    if invalid {
        violations.push(code);
    }
}

/// Validate the tracked adapter manifest and the contract-critical properties of its
/// companion schema. This is a narrow semantic verifier, not a general JSON Schema engine.
pub fn validate_resilience_binding_values(
    binding: &Value,
    schema: &Value,
) -> ResilienceBindingReport {
    let mut violations = Vec::new();
    let binding_id = string_at(binding, "binding_id")
        .unwrap_or("UNKNOWN_BINDING_ID")
        .to_owned();

    let required_root_fields = [
        "$schema",
        "schema_version",
        "binding_id",
        "created_date",
        "revision_status",
        "claim_ceiling",
        "reference_scenario",
        "owner_bindings",
        "weather_source_candidates",
        "scenario_families",
        "service_domains",
        "paired_comparison_contract",
        "model_credibility",
        "decision_relevant_measurements",
        "symtropy_invariant_crosswalk",
        "invalidation_dependencies",
        "benchmark_targets",
    ];
    let schema_required = array_at(schema, "required");
    push(
        &mut violations,
        required_root_fields
            .iter()
            .any(|field| !has_string(schema_required, field)),
        "RESILIENCE_SCHEMA_REQUIRED_ROOT_FIELD_MISSING",
    );
    push(
        &mut violations,
        string_at(schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema")
            || string_at(binding, "$schema")
                != Some("https://json-schema.org/draft/2020-12/schema")
            || string_at(schema, "title")
                != Some("Highveld Keep resilience owner-binding manifest v0.1"),
        "RESILIENCE_SCHEMA_DRAFT_OR_IDENTITY_INVALID",
    );
    push(
        &mut violations,
        schema
            .get("properties")
            .and_then(|properties| properties.get("schema_version"))
            .and_then(|property| property.get("const"))
            .and_then(Value::as_str)
            != Some(SCHEMA_VERSION),
        "RESILIENCE_SCHEMA_VERSION_CONTRACT_CHANGED",
    );
    push(
        &mut violations,
        schema
            .get("properties")
            .and_then(|properties| properties.get("scenario_families"))
            .and_then(|property| property.get("minItems"))
            .and_then(Value::as_u64)
            != Some(10)
            || schema
                .get("properties")
                .and_then(|properties| properties.get("scenario_families"))
                .and_then(|property| property.get("maxItems"))
                .and_then(Value::as_u64)
                != Some(10),
        "RESILIENCE_SCHEMA_SCENARIO_COUNT_NOT_FROZEN",
    );
    push(
        &mut violations,
        string_at(binding, "schema_version") != Some(SCHEMA_VERSION)
            || binding_id != BINDING_ID
            || string_at(binding, "revision_status")
                != Some("E0_SOUTH_AFRICAN_ADAPTER_CONTRACT_ONLY")
            || string_at(binding, "claim_ceiling") != Some(CLAIM_CEILING),
        "RESILIENCE_BINDING_ID_VERSION_OR_CLAIM_CEILING_CHANGED",
    );

    let reference = binding.get("reference_scenario");
    push(
        &mut violations,
        reference.is_none_or(|reference| {
            string_at(reference, "scenario_id") != Some("HIGHVELD-KEEP-SYNTHETIC-001")
                || string_at(reference, "source_path")
                    != Some("docs/research/highveld-keep/HIGHVELD_KEEP_SCENARIO_V0_1.json")
                || !reference.get("parcel_id").is_some_and(Value::is_null)
                || !reference.get("coordinates").is_some_and(Value::is_null)
                || !reference
                    .get("selected_weather_station")
                    .is_some_and(Value::is_null)
                || string_at(reference, "site_weather_status")
                    != Some("UNKNOWN_NO_REAL_PARCEL_OR_SITE_SURVEY")
        }),
        "RESILIENCE_REAL_SITE_OR_WEATHER_SELECTION_PROMOTED",
    );

    let owners = array_at(binding, "owner_bindings");
    let owner_contract_ids: Vec<&str> = owners
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("contract").and_then(Value::as_str))
        .collect();
    let owner_contract_set: HashSet<&str> = owner_contract_ids.iter().copied().collect();
    push(
        &mut violations,
        owners.is_none()
            || owners.is_some_and(|items| items.len() != OWNER_CONTRACTS.len())
            || owner_contract_ids.len() != OWNER_CONTRACTS.len()
            || owner_contract_set.len() != OWNER_CONTRACTS.len()
            || OWNER_CONTRACTS
                .iter()
                .any(|expected| !owner_contract_set.contains(expected)),
        "RESILIENCE_OWNER_BINDING_SET_INVALID",
    );
    if let Some(owners) = owners {
        for owner in owners {
            push(
                &mut violations,
                string_at(owner, "issue").is_none_or(|url| !url.starts_with("https://github.com/"))
                    || string_at(owner, "authority").is_none_or(str::is_empty)
                    || string_at(owner, "status").is_none_or(str::is_empty),
                "RESILIENCE_OWNER_BINDING_METADATA_MISSING",
            );
        }
        let energy_plus = owners
            .iter()
            .find(|owner| string_at(owner, "contract") == Some("ENERGYPLUS-QUAL-001"));
        push(
            &mut violations,
            energy_plus.is_none_or(|owner| {
                string_at(owner, "release_candidate") != Some("EnergyPlus 26.2.0")
                    || string_at(owner, "release_date") != Some("2026-09-30")
                    || string_at(owner, "status")
                        != Some("RELEASE_IDENTIFIED_ADAPTER_NOT_QUALIFIED")
            }),
            "ENERGYPLUS_RELEASE_OR_QUALIFICATION_STATUS_CHANGED",
        );
    }

    let weather = array_at(binding, "weather_source_candidates");
    push(
        &mut violations,
        weather.is_none_or(|items| items.len() != WEATHER_CANDIDATES.len()),
        "RESILIENCE_WEATHER_CANDIDATE_COUNT_CHANGED",
    );
    if let Some(items) = weather {
        for (expected_id, expected_station) in WEATHER_CANDIDATES {
            let candidate = items
                .iter()
                .find(|item| string_at(item, "source_id") == Some(expected_id));
            push(
                &mut violations,
                candidate.is_none_or(|candidate| {
                    string_at(candidate, "station_code") != Some(expected_station)
                        || string_at(candidate, "dataset_class") != Some("TMYx")
                        || string_at(candidate, "period") != Some("2011-2025")
                        || !candidate.get("file_sha256").is_some_and(Value::is_null)
                        || string_at(candidate, "rights")
                            != Some("UNKNOWN_FILE_SPECIFIC_REUSE_AND_REDISTRIBUTION")
                        || string_at(candidate, "site_representativeness") != Some("UNKNOWN")
                        || string_at(candidate, "admission")
                            != Some("CATALOGUED_NOT_HEADER_INSPECTED_OR_QUALIFIED")
                }),
                "RESILIENCE_WEATHER_CANDIDATE_PROMOTED_OR_IDENTITY_CHANGED",
            );
        }
        let mut source_ids = HashSet::new();
        push(
            &mut violations,
            items.iter().any(|item| {
                string_at(item, "source_id").is_none_or(|id| !source_ids.insert(id.to_owned()))
            }),
            "RESILIENCE_WEATHER_SOURCE_ID_DUPLICATED",
        );
    }

    let scenarios = array_at(binding, "scenario_families");
    let scenario_ids: Vec<&str> = scenarios
        .into_iter()
        .flatten()
        .filter_map(|item| string_at(item, "scenario_id"))
        .collect();
    let scenario_names: Vec<&str> = scenarios
        .into_iter()
        .flatten()
        .filter_map(|item| string_at(item, "name"))
        .collect();
    push(
        &mut violations,
        scenarios.is_none_or(|items| items.len() != SCENARIOS.len())
            || scenario_ids.len() != SCENARIOS.len()
            || scenario_names.len() != SCENARIOS.len()
            || SCENARIOS
                .iter()
                .any(|(id, name)| !scenario_ids.contains(id) || !scenario_names.contains(name))
            || scenario_ids.iter().collect::<HashSet<_>>().len() != scenario_ids.len(),
        "RESILIENCE_SCENARIO_FAMILY_SET_INVALID",
    );
    if let Some(scenarios) = scenarios {
        for scenario in scenarios {
            push(
                &mut violations,
                string_at(scenario, "admission").is_none_or(|state| {
                    !(state.starts_with("BLOCKED_") || state.starts_with("UNKNOWN_"))
                }) || array_at(scenario, "required_inputs").is_none_or(Vec::is_empty),
                "RESILIENCE_SCENARIO_ADMITTED_OR_INPUTS_UNDECLARED",
            );
        }
        let observed_hot = scenarios
            .iter()
            .find(|item| string_at(item, "scenario_id") == Some("HK-RES-002"));
        let observed_cold = scenarios
            .iter()
            .find(|item| string_at(item, "scenario_id") == Some("HK-RES-003"));
        let future = scenarios
            .iter()
            .find(|item| string_at(item, "scenario_id") == Some("HK-RES-004"));
        let compound = scenarios
            .iter()
            .find(|item| string_at(item, "scenario_id") == Some("HK-RES-010"));
        push(
            &mut violations,
            observed_hot.is_none_or(|item| {
                string_at(item, "weather_class")
                    != Some("ACTUAL_METEOROLOGICAL_YEAR_OR_OBSERVED_EVENT")
                    || !has_string(array_at(item, "prohibited_substitutions"), "TMY_OR_TMYX")
            }) || observed_cold.is_none_or(|item| {
                string_at(item, "weather_class")
                    != Some("ACTUAL_METEOROLOGICAL_YEAR_OR_OBSERVED_EVENT")
                    || !has_string(array_at(item, "prohibited_substitutions"), "TMY_OR_TMYX")
            }) || future.is_none_or(|item| {
                string_at(item, "weather_class") != Some("MODELLED_FUTURE_CLIMATE_SCENARIO")
                    || !has_string(
                        array_at(item, "prohibited_substitutions"),
                        "TMY_AS_FUTURE_CLIMATE",
                    )
            }),
            "RESILIENCE_TMY_EXTREME_OR_FUTURE_SUBSTITUTION_ALLOWED",
        );
        push(
            &mut violations,
            compound.is_none_or(|item| {
                string_at(item, "weather_class")
                    != Some("ONE_JOINT_MULTIHAZARD_TRACE_WITH_DEPENDENCY_SEMANTICS")
                    || !has_string(
                        array_at(item, "prohibited_substitutions"),
                        "independent_marginal_probabilities_as_joint_probability_without_justified_model",
                    )
                    || !has_string(
                        array_at(item, "required_inputs"),
                        "joint_event_dependency_or_causal_ordering_basis",
                    )
            }),
            "RESILIENCE_COMPOUND_HAZARD_DEPENDENCY_RULE_MISSING",
        );
    }

    let services = array_at(binding, "service_domains");
    let service_ids: Vec<&str> = services
        .into_iter()
        .flatten()
        .filter_map(|item| string_at(item, "domain_id"))
        .collect();
    push(
        &mut violations,
        services.is_none_or(|items| items.len() != SERVICE_DOMAINS.len())
            || service_ids.len() != SERVICE_DOMAINS.len()
            || SERVICE_DOMAINS
                .iter()
                .any(|expected| !service_ids.contains(expected))
            || services.is_some_and(|items| {
                items.iter().any(|item| {
                    !string_at(item, "threshold_state")
                        .is_some_and(|state| state.starts_with("UNKNOWN_"))
                        || !item.get("threshold_source").is_some_and(Value::is_null)
                        || string_at(item, "unit").is_none_or(str::is_empty)
                })
            }),
        "RESILIENCE_SERVICE_DOMAIN_THRESHOLD_OR_UNIT_PROMOTED",
    );

    let paired = binding.get("paired_comparison_contract");
    push(
        &mut violations,
        paired.is_none_or(|paired| {
            paired
                .get("same_exogenous_trace_required")
                .and_then(Value::as_bool)
                != Some(true)
                || paired
                    .get("candidate_must_not_receive_future_events")
                    .and_then(Value::as_bool)
                    != Some(true)
                || string_at(paired, "service_floor_semantics")
                    != Some("PER_RESIDENT_PER_TASK_NON_COMPENSATORY")
                || string_at(paired, "unknown_or_missing_evidence")
                    != Some("BLOCKED_OR_NOT_COMPARABLE_NEVER_ZERO_OR_PASS")
                || !string_at(paired, "stable_recovery")
                    .is_some_and(|value| value.contains("SYMTROPY"))
                || !string_at(paired, "pareto").is_some_and(|value| value.contains("SYMTROPY"))
                || !string_at(paired, "joint_event_dependency_semantics")
                    .is_some_and(|value| value.contains("do_not_assume_independence"))
        }),
        "RESILIENCE_PAIRED_COMPARISON_RULES_WEAKENED",
    );
    if let Some(paired) = paired {
        for field in ["trace_identity_fields", "candidate_identity_fields"] {
            let values = array_at(paired, field);
            push(
                &mut violations,
                values.is_none_or(Vec::is_empty)
                    || values.is_some_and(|items| {
                        items
                            .iter()
                            .any(|item| item.as_str().is_none_or(str::is_empty))
                    }),
                "RESILIENCE_PAIRED_IDENTITY_FIELDS_INCOMPLETE",
            );
        }
    }

    let credibility = binding.get("model_credibility");
    let stages = credibility.and_then(|item| array_at(item, "stages"));
    let stage_names: Vec<&str> = stages
        .into_iter()
        .flatten()
        .filter_map(|stage| string_at(stage, "stage"))
        .collect();
    push(
        &mut violations,
        credibility.is_none_or(|item| {
            string_at(item, "framework") != Some("RISK_PROPORTIONATE_VVUQ")
                || array_at(item, "prohibited_shortcuts").is_none_or(|values| values.len() < 5)
        }) || stages.is_none_or(|items| items.len() != VVUQ_STAGES.len())
            || VVUQ_STAGES
                .iter()
                .any(|expected| !stage_names.contains(expected)),
        "RESILIENCE_VVUQ_STAGES_OR_SHORTCUTS_INVALID",
    );

    let measurements = array_at(binding, "decision_relevant_measurements");
    let measurement_ids: Vec<&str> = measurements
        .into_iter()
        .flatten()
        .filter_map(|item| string_at(item, "question_id"))
        .collect();
    push(
        &mut violations,
        measurements.is_none_or(|items| items.len() != MEASUREMENT_QUESTIONS.len())
            || MEASUREMENT_QUESTIONS
                .iter()
                .any(|expected| !measurement_ids.contains(expected))
            || measurements.is_some_and(|items| {
                items.iter().any(|item| {
                    string_at(item, "current_state").is_none_or(str::is_empty)
                        || string_at(item, "stop_rule").is_none_or(str::is_empty)
                })
            }),
        "RESILIENCE_MEASUREMENT_STOP_RULES_INCOMPLETE",
    );

    let invariants = array_at(binding, "symtropy_invariant_crosswalk");
    let invariant_ids: Vec<&str> = invariants
        .into_iter()
        .flatten()
        .filter_map(|item| string_at(item, "invariant_id"))
        .collect();
    push(
        &mut violations,
        invariants.is_none_or(|items| items.len() != INVARIANTS.len())
            || invariant_ids.len() != INVARIANTS.len()
            || INVARIANTS
                .iter()
                .any(|expected| !invariant_ids.contains(expected))
            || invariant_ids.iter().collect::<HashSet<_>>().len() != invariant_ids.len(),
        "RESILIENCE_SYMTROPY_INVARIANT_CROSSWALK_INVALID",
    );

    let invalidators = array_at(binding, "invalidation_dependencies");
    let invalidator_changes: Vec<&str> = invalidators
        .into_iter()
        .flatten()
        .filter_map(|item| string_at(item, "change"))
        .collect();
    push(
        &mut violations,
        invalidators.is_none_or(|items| items.len() != INVALIDATORS.len())
            || INVALIDATORS
                .iter()
                .any(|expected| !invalidator_changes.contains(expected))
            || invalidators.is_some_and(|items| {
                items
                    .iter()
                    .any(|item| array_at(item, "invalidates").is_none_or(Vec::is_empty))
            }),
        "RESILIENCE_INVALIDATION_DEPENDENCIES_INCOMPLETE",
    );

    let targets = binding.get("benchmark_targets");
    push(
        &mut violations,
        targets.is_none_or(|targets| {
            string_at(targets, "paired_trace_identity_match") != Some("100_percent")
                || string_at(
                    targets,
                    "unknown_missing_stale_nonconverged_out_of_domain_result_false_pass",
                ) != Some("0")
                || string_at(targets, "hard_floor_violations_hidden_by_aggregation") != Some("0")
                || string_at(targets, "admitted_result_exact_input_revision_binding")
                    != Some("100_percent")
                || string_at(
                    targets,
                    "unqualified_tmy_promoted_to_extreme_or_future_scenario",
                ) != Some("0")
                || string_at(
                    targets,
                    "site_weather_claim_without_parcel_survey_and_representativeness_review",
                ) != Some("0")
                || string_at(targets, "compound_event_trace_dependency_loss") != Some("0")
        }),
        "RESILIENCE_BENCHMARK_INTEGRITY_TARGETS_WEAKENED",
    );

    let mut seen_violations = HashSet::new();
    violations.retain(|violation| seen_violations.insert(*violation));
    let report = ResilienceBindingReport {
        binding_id,
        scenario_families: scenarios.map_or(0, Vec::len),
        weather_candidates: weather.map_or(0, Vec::len),
        service_domains: services.map_or(0, Vec::len),
        owner_bindings: owners.map_or(0, Vec::len),
        invariant_mappings: invariants.map_or(0, Vec::len),
        vvuq_stages: stages.map_or(0, Vec::len),
        measurement_questions: measurements.map_or(0, Vec::len),
        violations,
    };
    report
}

/// Parse and validate the tracked resilience binding and its companion schema.
pub fn validate_tracked_resilience_binding() -> Result<ResilienceBindingReport, String> {
    let binding: Value = serde_json::from_str(RESILIENCE_BINDING_JSON)
        .map_err(|error| format!("RESILIENCE_BINDING_JSON_INVALID: {error}"))?;
    let schema: Value = serde_json::from_str(RESILIENCE_BINDING_SCHEMA)
        .map_err(|error| format!("RESILIENCE_BINDING_SCHEMA_JSON_INVALID: {error}"))?;
    Ok(validate_resilience_binding_values(&binding, &schema))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn documents() -> (Value, Value) {
        (
            serde_json::from_str(RESILIENCE_BINDING_JSON).expect("binding JSON parses"),
            serde_json::from_str(RESILIENCE_BINDING_SCHEMA).expect("schema JSON parses"),
        )
    }

    fn contains(report: &ResilienceBindingReport, code: &str) -> bool {
        report.violations.contains(&code)
    }

    #[test]
    fn tracked_highveld_resilience_binding_is_consistent() {
        let report =
            validate_tracked_resilience_binding().expect("tracked resilience binding parses");
        assert!(report.is_valid(), "{:?}", report.violations);
        assert_eq!(report.scenario_families, 10);
        assert_eq!(report.weather_candidates, 5);
        assert_eq!(report.service_domains, 7);
        assert_eq!(report.owner_bindings, 6);
        assert_eq!(report.invariant_mappings, 16);
        assert_eq!(report.vvuq_stages, 6);
        assert_eq!(report.measurement_questions, 5);
    }

    #[test]
    fn tmy_candidates_cannot_be_promoted_to_extreme_weather() {
        let (mut binding, schema) = documents();
        let observed = binding["scenario_families"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|item| item["scenario_id"] == "HK-RES-002")
            .unwrap();
        observed["weather_class"] = json!("TMYx");
        let report = validate_resilience_binding_values(&binding, &schema);
        assert!(
            contains(
                &report,
                "RESILIENCE_TMY_EXTREME_OR_FUTURE_SUBSTITUTION_ALLOWED"
            )
        );
    }

    #[test]
    fn weather_file_hash_and_rights_cannot_be_silently_promoted() {
        let (mut binding, schema) = documents();
        binding["weather_source_candidates"][0]["file_sha256"] =
            json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        binding["weather_source_candidates"][0]["rights"] = json!("OPEN");
        let report = validate_resilience_binding_values(&binding, &schema);
        assert!(contains(
            &report,
            "RESILIENCE_WEATHER_CANDIDATE_PROMOTED_OR_IDENTITY_CHANGED"
        ));
    }

    #[test]
    fn unknown_service_thresholds_cannot_be_filled_with_unsourced_values() {
        let (mut binding, schema) = documents();
        binding["service_domains"][0]["threshold_source"] = json!("unreviewed-blog");
        binding["service_domains"][0]["threshold_state"] = json!("VERIFIED");
        let report = validate_resilience_binding_values(&binding, &schema);
        assert!(contains(
            &report,
            "RESILIENCE_SERVICE_DOMAIN_THRESHOLD_OR_UNIT_PROMOTED"
        ));
    }

    #[test]
    fn compound_hazards_require_a_joint_dependency_basis() {
        let (mut binding, schema) = documents();
        binding["scenario_families"][9]["prohibited_substitutions"]
            .as_array_mut()
            .unwrap()
            .retain(|item| {
                item.as_str()
                    != Some("independent_marginal_probabilities_as_joint_probability_without_justified_model")
            });
        let report = validate_resilience_binding_values(&binding, &schema);
        assert!(contains(
            &report,
            "RESILIENCE_COMPOUND_HAZARD_DEPENDENCY_RULE_MISSING"
        ));
    }

    #[test]
    fn paired_candidates_must_share_trace_and_not_receive_future_events() {
        let (mut binding, schema) = documents();
        binding["paired_comparison_contract"]["same_exogenous_trace_required"] = json!(false);
        binding["paired_comparison_contract"]["candidate_must_not_receive_future_events"] =
            json!(false);
        let report = validate_resilience_binding_values(&binding, &schema);
        assert!(contains(
            &report,
            "RESILIENCE_PAIRED_COMPARISON_RULES_WEAKENED"
        ));
    }

    #[test]
    fn all_symtropy_invariants_must_remain_mapped() {
        let (mut binding, schema) = documents();
        binding["symtropy_invariant_crosswalk"]
            .as_array_mut()
            .unwrap()
            .pop();
        let report = validate_resilience_binding_values(&binding, &schema);
        assert!(contains(
            &report,
            "RESILIENCE_SYMTROPY_INVARIANT_CROSSWALK_INVALID"
        ));
    }

    #[test]
    fn compound_trace_loss_is_a_benchmark_failure() {
        let (mut binding, schema) = documents();
        binding["benchmark_targets"]["compound_event_trace_dependency_loss"] = json!("1");
        let report = validate_resilience_binding_values(&binding, &schema);
        assert!(contains(
            &report,
            "RESILIENCE_BENCHMARK_INTEGRITY_TARGETS_WEAKENED"
        ));
    }
}
