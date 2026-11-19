//! Fail-closed semantic checks for the frozen shared-amenity benchmark registry.
//!
//! This tool audits registry structure and claim-state invariants only. It does not
//! calculate capacity, allocate slots, evaluate costs, qualify buildings, or establish
//! legal ownership or physical performance.

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const EXPECTED_SCHEMA: &str = "luminous.shared-amenity-commons-benchmark";
const EXPECTED_VERSION: &str = "0.1.0";
const EXPECTED_STUDY: &str = "SHARED-AMENITY-COMMONS-001";
const EXPECTED_MATURITY: &str = "BENCHMARK_PROTOCOL_AND_CASE_REGISTRY_ONLY";
const EXPECTED_EXECUTION: &str = "NOT_EXECUTED";
const EXPECTED_GEO: &str = "JOHANNESBURG_HIGHVELD_REFERENCE_METHOD_PORTABLE";
const EXPECTED_CLAIM_CEILING: &str =
    "NO_EMPIRICAL_SAVINGS_SAFETY_ACCESSIBILITY_LEGAL_OWNERSHIP_OR_PREFERENCE_CLAIM";

const EXPECTED_ARMS: &[&str] = &[
    "PRIVATE_DUPLICATION",
    "CENTRAL_SHARED",
    "DISTRIBUTED_SHARED",
    "HYBRID_SHARED_PRIVATE",
    "EXTERNAL_COMMERCIAL",
];

const EXPECTED_SCENARIOS: &[&str] = &[
    "COMPACT_HIGH_DENSITY",
    "LOW_DENSITY_DISPERSED",
    "MIXED_USE_TRANSIT_ADJACENT",
    "SITE_EDGE_LONG_WALK",
    "ACCESSIBLE_ROUTE_DISCONTINUITY",
    "UNEQUAL_POPULATION_DISTRIBUTION",
    "PHASED_OCCUPANCY_GROWTH",
    "LAND_SCARCITY_COMPETING_USE",
    "EXTERNAL_PROVIDER_DISTANCE_OR_CLOSURE",
    "MULTI_NODE_CATCHMENT_REBALANCING",
];

const EXPECTED_FUNCTIONAL_UNITS: &[(&str, &str)] = &[
    ("COMMUNITY_HALL", "ACCESSIBLE_COMMUNITY_ROOM_HOUR"),
    ("COMMUNITY_KITCHEN", "ACCEPTED_MEAL_SERVICE"),
    ("COWORKING_LIBRARY_STUDY", "ACCESSIBLE_SEAT_HOUR"),
    ("WORKSHOP_TOOL_LIBRARY_REPAIR", "USABLE_WORKSHOP_HOUR"),
    ("GUEST_SUITES", "ACCESSIBLE_GUEST_NIGHT"),
    ("SHARED_LAUNDRY", "ACCEPTED_LAUNDRY_LOAD"),
    ("GREENHOUSE_NURSERY_FOOD_GROWING", "USABLE_HARVEST_KG"),
    ("POOL_WELLNESS", "OPERABLE_POOL_LANE_HOUR"),
    ("GYM_RECREATION", "USABLE_ACTIVITY_SPACE_HOUR"),
];

const EXPECTED_AMENITY_IDS: &[&str] = &[
    "COMMUNITY_HALL",
    "COMMUNITY_KITCHEN",
    "COWORKING_LIBRARY_STUDY",
    "WORKSHOP_TOOL_LIBRARY_REPAIR",
    "GUEST_SUITES",
    "SHARED_LAUNDRY",
    "GREENHOUSE_NURSERY_FOOD_GROWING",
    "POOL_WELLNESS",
    "GYM_RECREATION",
    "NON_ESSENTIAL_AMENITY_PLANT",
];

const EXPECTED_INCLUDED_SERVICE_CLASSES: &[&str] = &[
    "community_and_dining",
    "study_and_work",
    "routine_tools_and_making",
    "guest_accommodation",
    "laundry",
    "non_emergency_food_growing",
    "pool_and_wellness",
    "gym_and_recreation",
    "public_recreation",
];

const REQUIRED_EXCLUDED_REFERENCES: &[&str] = &[
    "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/694",
    "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/737",
    "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/840",
    "https://github.com/Luminous-Dynamics/symtropy/issues/3768",
    "https://github.com/Luminous-Dynamics/mycelix/issues/5901",
    "https://github.com/Luminous-Dynamics/mycelix/issues/987",
    "https://github.com/Luminous-Dynamics/mycelix/issues/4808",
];

const REQUIRED_OWNER_NATIVE_DEPENDENCIES: &[(&str, &str, &str, &str)] = &[
    ("HIGHVELD_KEEP_REFERENCE_HOME", "Sol-Atlas", "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/607", "PRIMARY_HOME_REFERENCE_DESIGN"),
    ("HOME_EXPRESSION_LIBRARY", "Sol-Atlas", "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/904", "RESIDENT_EXPRESSION_AND_OPTIONAL_EXPERIENCE_PACKAGES"),
    ("AGRO_CLIMATE_HIGHVELD", "Sol-Atlas", "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/107", "HIGHVELD_FOOD_CLIMATE_SITE_READINESS"),
    ("ATLAS_FOOD_NETWORK", "Sol-Atlas", "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/105", "PRIVACY_PRESERVING_FOOD_NETWORK_PROJECTION"),
    ("CEA_FULL_BOUNDARY", "Symtropy", "https://github.com/Luminous-Dynamics/symtropy/issues/3957", "CEA_FULL_BOUNDARY_RESOURCE_AND_NUTRITIONAL_SERVICE"),
    ("CEA_CLIMATE", "Symtropy", "https://github.com/Luminous-Dynamics/symtropy/issues/4065", "CEA_SPATIAL_CLIMATE_HEAT_MOISTURE_AIRFLOW"),
    ("CEA_HARVEST_CONTROL", "Symtropy", "https://github.com/Luminous-Dynamics/symtropy/issues/4074", "CEA_HARVEST_CONSTRAINED_LIGHTING_AND_CLIMATE_CONTROL"),
    ("MYCELIX_FOOD_COOPERATIVE", "Mycelix", "https://github.com/Luminous-Dynamics/mycelix/issues/5001", "FOOD_COOPERATIVE_PRODUCT_LOT_AND_FULFILLMENT"),
    ("HABITAT_SERVICE_TEST_CELL", "Symtropy", "https://github.com/Luminous-Dynamics/symtropy/issues/4531", "ROOM_SCALE_HOUSEHOLD_SERVICE_TEST_CELL"),
    ("WATERLOOP_PHYSICAL_DEMO", "Symtropy", "https://github.com/Luminous-Dynamics/symtropy/issues/5335", "WATERLOOP_PHYSICAL_DEMONSTRATOR"),
    ("ABOVEGROUND_SERVICE_CASSETTE", "Symtropy", "https://github.com/Luminous-Dynamics/symtropy/issues/4181", "MAINTAINABLE_BUILDING_SERVICE_CASSETTES"),
    ("MYCELIX_HOUSING_AMENITY_COMMONS", "Mycelix", "https://github.com/Luminous-Dynamics/mycelix/issues/6328", "CANONICAL_AMENITY_ASSET_SERVICE_ACCESS_FINANCE_MAINTENANCE_GOVERNANCE_CONTRACT"),
    ("WHOLE_HOUSEHOLD_COST_OF_PLACE", "Sol-Atlas", "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/816", "HOUSEHOLD_COST_OF_PLACE_AFFORDABILITY"),
    ("CAPITAL_TO_COMMONS_FINANCE", "Mycelix", "https://github.com/Luminous-Dynamics/mycelix/issues/912", "CAPITAL_TO_COMMONS_STEWARD_FINANCE_TRANSITION"),
    ("CITY_MORPH_EFFECTIVE_ACCESS", "Sol-Atlas", "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/555", "EVIDENCE_QUALIFIED_MULTIMODAL_EFFECTIVE_ACCESS"),
    ("CITY_SCOPE_BINDING", "Sol-Atlas", "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/623", "COMMON_SCOPE_DIGEST_AND_OWNER_ENVELOPE"),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditError {
    pub path: String,
    pub message: String,
}

impl AuditError {
    fn new(path: impl AsRef<str>, message: impl Into<String>) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditSummary {
    pub schema: String,
    pub schema_version: String,
    pub study_id: String,
    pub manifest_sha256: String,
    pub comparison_arm_count: usize,
    pub scenario_count: usize,
    pub registered_case_count: usize,
    pub unique_case_pairs: usize,
    pub amenity_archetype_count: usize,
    pub functional_unit_count: usize,
    pub reference_source_count: usize,
    pub owner_native_dependency_count: usize,
    pub execution_status: String,
    pub claim_ceiling: String,
}

/// Validate the semantic invariants of the v0.1 registry.
///
/// The returned digest binds the exact input bytes; it is not an authenticity,
/// source-truth, engineering, governance, or legal-approval proof.
pub fn audit_manifest_bytes(bytes: &[u8]) -> Result<AuditSummary, AuditError> {
    let root: Value = serde_json::from_slice(bytes).map_err(|error| {
        AuditError::new("$", format!("invalid JSON: {error}"))
    })?;

    let root_object = root
        .as_object()
        .ok_or_else(|| AuditError::new("$", "expected a JSON object"))?;

    let allowed_root = [
        "schema",
        "schema_version",
        "study_id",
        "issue_url",
        "parent_issues",
        "maturity",
        "execution_status",
        "geographic_profile",
        "claim_ceiling",
        "comparison_arms",
        "scenarios",
        "amenity_archetypes",
        "functional_units",
        "scope_boundary",
        "case_matrix",
        "cases",
        "metric_vectors",
        "non_compensatory_gates",
        "evidence_states",
        "owner_boundaries",
        "reference_sources",
        "future_empirical_inputs_required",
        "targets",
    ];
    check_keys(root_object, &allowed_root, "$")?;

    let schema = required_string(&root, "schema", "$")?;
    let version = required_string(&root, "schema_version", "$")?;
    let study_id = required_string(&root, "study_id", "$")?;
    let maturity = required_string(&root, "maturity", "$")?;
    let execution_status = required_string(&root, "execution_status", "$")?;
    let geographic_profile = required_string(&root, "geographic_profile", "$")?;
    let claim_ceiling = required_string(&root, "claim_ceiling", "$")?;

    expect_equal(&schema, EXPECTED_SCHEMA, "$.schema")?;
    expect_equal(&version, EXPECTED_VERSION, "$.schema_version")?;
    expect_equal(&study_id, EXPECTED_STUDY, "$.study_id")?;
    expect_equal(&maturity, EXPECTED_MATURITY, "$.maturity")?;
    expect_equal(&execution_status, EXPECTED_EXECUTION, "$.execution_status")?;
    expect_equal(&geographic_profile, EXPECTED_GEO, "$.geographic_profile")?;
    expect_equal(&claim_ceiling, EXPECTED_CLAIM_CEILING, "$.claim_ceiling")?;

    let arms = required_array(&root, "comparison_arms", "$")?;
    if arms.len() != EXPECTED_ARMS.len() {
        return Err(AuditError::new(
            "$.comparison_arms",
            format!("expected {} arms, found {}", EXPECTED_ARMS.len(), arms.len()),
        ));
    }
    let arm_ids = collect_object_ids(arms, "id", "$.comparison_arms")?;
    expect_same_set(&arm_ids, EXPECTED_ARMS, "$.comparison_arms")?;

    let scenarios = required_array(&root, "scenarios", "$")?;
    if scenarios.len() != EXPECTED_SCENARIOS.len() {
        return Err(AuditError::new(
            "$.scenarios",
            format!(
                "expected {} scenarios, found {}",
                EXPECTED_SCENARIOS.len(),
                scenarios.len()
            ),
        ));
    }
    let scenario_ids = collect_string_set(scenarios, "$.scenarios")?;
    expect_same_set(&scenario_ids, EXPECTED_SCENARIOS, "$.scenarios")?;

    validate_amenities(&root)?;
    validate_scope_boundary(&root)?;
    validate_case_matrix(&root, &arm_ids, &scenario_ids)?;
    validate_reference_sources(&root)?;
    validate_targets(&root)?;

    let cases = required_array(&root, "cases", "$")?;
    let digest = Sha256::digest(bytes);
    let manifest_sha256 = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    Ok(AuditSummary {
        schema,
        schema_version: version,
        study_id,
        manifest_sha256,
        comparison_arm_count: arms.len(),
        scenario_count: scenarios.len(),
        registered_case_count: cases.len(),
        unique_case_pairs: cases.len(),
        amenity_archetype_count: required_array(&root, "amenity_archetypes", "$")?.len(),
        functional_unit_count: required_array(&root, "functional_units", "$")?.len(),
        reference_source_count: required_array(&root, "reference_sources", "$")?.len(),
        owner_native_dependency_count: required_array(root.get("scope_boundary").ok_or_else(|| AuditError::new("$.scope_boundary", "missing field"))?, "owner_native_dependencies", "$.scope_boundary")?.len(),
        execution_status,
        claim_ceiling,
    })
}

fn validate_amenities(root: &Value) -> Result<(), AuditError> {
    let amenities = required_array(root, "amenity_archetypes", "$")?;
    if amenities.len() != EXPECTED_AMENITY_IDS.len() {
        return Err(AuditError::new(
            "$.amenity_archetypes",
            format!(
                "expected {} archetypes, found {}",
                EXPECTED_AMENITY_IDS.len(),
                amenities.len()
            ),
        ));
    }

    let mut amenity_map = BTreeMap::new();
    for (index, amenity) in amenities.iter().enumerate() {
        let path = format!("$.amenity_archetypes[{index}]");
        let object = amenity
            .as_object()
            .ok_or_else(|| AuditError::new(&path, "expected an object"))?;
        check_keys(
            object,
            &["id", "service", "critical_dependencies", "role", "functional_unit_id"],
            &path,
        )?;

        let id = required_string(amenity, "id", &path)?;
        let service = required_string(amenity, "service", &path)?;
        if service.trim().is_empty() {
            return Err(AuditError::new(format!("{path}.service"), "must not be empty"));
        }
        let dependencies = amenity
            .get("critical_dependencies")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AuditError::new(format!("{path}.critical_dependencies"), "expected an array")
            })?;
        if dependencies.is_empty() {
            return Err(AuditError::new(
                format!("{path}.critical_dependencies"),
                "must contain at least one dependency",
            ));
        }
        collect_string_set(dependencies, &format!("{path}.critical_dependencies"))?;

        let role = required_string(amenity, "role", &path)?;
        let functional_unit = amenity.get("functional_unit_id").ok_or_else(|| {
            AuditError::new(format!("{path}.functional_unit_id"), "missing field")
        })?;

        if id == "NON_ESSENTIAL_AMENITY_PLANT" {
            expect_equal(&role, "ENABLING_ASSET_ONLY", &format!("{path}.role"))?;
            if !functional_unit.is_null() {
                return Err(AuditError::new(
                    format!("{path}.functional_unit_id"),
                    "an enabling asset must not claim a standalone resident-facing functional unit",
                ));
            }
            if amenity_map.insert(id.clone(), None).is_some() {
                return Err(AuditError::new(format!("{path}.id"), "duplicate amenity id"));
            }
        } else {
            expect_equal(
                &role,
                "RESIDENT_FACING_SERVICE_ASSET",
                &format!("{path}.role"),
            )?;
            let unit_id = functional_unit.as_str().ok_or_else(|| {
                AuditError::new(
                    format!("{path}.functional_unit_id"),
                    "resident-facing service asset requires a functional-unit id",
                )
            })?;
            if amenity_map.insert(id.clone(), Some(unit_id.to_owned())).is_some() {
                return Err(AuditError::new(format!("{path}.id"), "duplicate amenity id"));
            }
        }
    }

    let actual_amenity_ids = amenity_map.keys().cloned().collect::<BTreeSet<_>>();
    let expected_amenity_ids = EXPECTED_AMENITY_IDS
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    if actual_amenity_ids != expected_amenity_ids {
        return Err(AuditError::new(
            "$.amenity_archetypes",
            "amenity identifiers do not match the frozen v0.1 catalogue",
        ));
    }

    let units = required_array(root, "functional_units", "$")?;
    if units.len() != EXPECTED_FUNCTIONAL_UNITS.len() {
        return Err(AuditError::new(
            "$.functional_units",
            format!(
                "expected {} resident-facing functional units, found {}",
                EXPECTED_FUNCTIONAL_UNITS.len(),
                units.len()
            ),
        ));
    }

    let expected_units = EXPECTED_FUNCTIONAL_UNITS
        .iter()
        .map(|(amenity, unit)| ((*amenity).to_owned(), (*unit).to_owned()))
        .collect::<BTreeMap<_, _>>();
    let mut seen_unit_ids = BTreeSet::new();

    for (index, unit) in units.iter().enumerate() {
        let path = format!("$.functional_units[{index}]");
        let object = unit
            .as_object()
            .ok_or_else(|| AuditError::new(&path, "expected an object"))?;
        check_keys(
            object,
            &[
                "id",
                "amenity_id",
                "output_unit",
                "acceptance_definition",
                "comparison_policy",
                "owner_contract_reference",
                "owner_alignment_state",
            ],
            &path,
        )?;
        let id = required_string(unit, "id", &path)?;
        let amenity_id = required_string(unit, "amenity_id", &path)?;
        let output_unit = required_string(unit, "output_unit", &path)?;
        let acceptance_definition = required_string(unit, "acceptance_definition", &path)?;
        let comparison_policy = required_string(unit, "comparison_policy", &path)?;
        let owner_contract_reference = required_string(unit, "owner_contract_reference", &path)?;
        let owner_alignment_state = required_string(unit, "owner_alignment_state", &path)?;

        expect_equal(
            &comparison_policy,
            "WITHIN_FUNCTIONAL_UNIT_ONLY",
            &format!("{path}.comparison_policy"),
        )?;
        expect_equal(
            &owner_contract_reference,
            "https://github.com/Luminous-Dynamics/mycelix/issues/6328",
            &format!("{path}.owner_contract_reference"),
        )?;
        expect_equal(
            &owner_alignment_state,
            "CANDIDATE_NOT_OWNER_APPROVED",
            &format!("{path}.owner_alignment_state"),
        )?;
        if output_unit.trim().is_empty() || acceptance_definition.trim().is_empty() {
            return Err(AuditError::new(
                &path,
                "output unit and acceptance definition must be non-empty",
            ));
        }
        if !seen_unit_ids.insert(id.clone()) {
            return Err(AuditError::new(format!("{path}.id"), "duplicate functional-unit id"));
        }

        match expected_units.get(&amenity_id) {
            Some(expected_id) if expected_id == &id => {}
            Some(expected_id) => {
                return Err(AuditError::new(
                    &path,
                    format!("amenity {amenity_id} requires functional unit {expected_id}"),
                ));
            }
            None => {
                return Err(AuditError::new(
                    format!("{path}.amenity_id"),
                    "functional unit refers to an unknown or enabling-only amenity",
                ));
            }
        }

        match amenity_map.get(&amenity_id) {
            Some(Some(mapped_id)) if mapped_id == &id => {}
            _ => {
                return Err(AuditError::new(
                    &path,
                    "functional-unit mapping does not agree with the amenity catalogue",
                ));
            }
        }
    }

    Ok(())
}

fn validate_scope_boundary(root: &Value) -> Result<(), AuditError> {
    let scope = root
        .get("scope_boundary")
        .ok_or_else(|| AuditError::new("$.scope_boundary", "missing field"))?;
    let object = scope
        .as_object()
        .ok_or_else(|| AuditError::new("$.scope_boundary", "expected an object"))?;
    check_keys(
        object,
        &[
            "included_scope",
            "included_service_classes",
            "excluded_scopes",
            "composition_rule",
            "owner_native_dependencies",
        ],
        "$.scope_boundary",
    )?;

    expect_equal(
        &required_string(scope, "included_scope", "$.scope_boundary")?,
        "SPATIAL_PROVISIONING_COMPARISON_CONSUMES_CANONICAL_AMENITY_SERVICE_AND_HOUSEHOLD_AFFORDABILITY",
        "$.scope_boundary.included_scope",
    )?;
    expect_equal(
        &required_string(scope, "composition_rule", "$.scope_boundary")?,
        "SOL_ATLAS_OWNS_SPATIAL_PROVISIONING_COMPARISON_MYCELIX_OWNS_CANONICAL_AMENITY_ASSET_SERVICE_AND_FINANCE_OWNER_OUTPUTS_READ_ONLY",
        "$.scope_boundary.composition_rule",
    )?;

    let included = required_array(scope, "included_service_classes", "$.scope_boundary")?;
    let included_set = collect_string_set(included, "$.scope_boundary.included_service_classes")?;
    expect_same_set(
        &included_set,
        EXPECTED_INCLUDED_SERVICE_CLASSES,
        "$.scope_boundary.included_service_classes",
    )?;

    let exclusions = required_array(scope, "excluded_scopes", "$.scope_boundary")?;
    let mut references = BTreeSet::new();
    for (index, exclusion) in exclusions.iter().enumerate() {
        let path = format!("$.scope_boundary.excluded_scopes[{index}]");
        let object = exclusion
            .as_object()
            .ok_or_else(|| AuditError::new(&path, "expected an object"))?;
        check_keys(object, &["owner", "reference", "responsibility"], &path)?;
        let owner = required_string(exclusion, "owner", &path)?;
        let reference = required_string(exclusion, "reference", &path)?;
        let responsibility = required_string(exclusion, "responsibility", &path)?;
        if owner.trim().is_empty() || responsibility.trim().is_empty() {
            return Err(AuditError::new(&path, "owner and responsibility must be non-empty"));
        }
        if !references.insert(reference) {
            return Err(AuditError::new(
                format!("{path}.reference"),
                "duplicate excluded-owner reference",
            ));
        }
    }

    for required in REQUIRED_EXCLUDED_REFERENCES {
        if !references.contains(*required) {
            return Err(AuditError::new(
                "$.scope_boundary.excluded_scopes",
                format!("missing required owner boundary reference: {required}"),
            ));
        }
    }

    let dependencies = required_array(scope, "owner_native_dependencies", "$.scope_boundary")?;
    if dependencies.len() != REQUIRED_OWNER_NATIVE_DEPENDENCIES.len() {
        return Err(AuditError::new(
            "$.scope_boundary.owner_native_dependencies",
            format!(
                "expected {} owner-native dependencies, found {}",
                REQUIRED_OWNER_NATIVE_DEPENDENCIES.len(),
                dependencies.len()
            ),
        ));
    }

    let mut dependency_ids = BTreeSet::new();
    let mut dependency_refs = BTreeSet::new();
    let mut dependency_roles = BTreeSet::new();

    for (index, dependency) in dependencies.iter().enumerate() {
        let path = format!("$.scope_boundary.owner_native_dependencies[{index}]");
        let object = dependency
            .as_object()
            .ok_or_else(|| AuditError::new(&path, "expected an object"))?;
        check_keys(
            object,
            &["id", "owner", "reference", "role", "responsibility"],
            &path,
        )?;

        let id = required_string(dependency, "id", &path)?;
        let owner = required_string(dependency, "owner", &path)?;
        let reference = required_string(dependency, "reference", &path)?;
        let role = required_string(dependency, "role", &path)?;
        let responsibility = required_string(dependency, "responsibility", &path)?;

        if id.trim().is_empty()
            || owner.trim().is_empty()
            || reference.trim().is_empty()
            || role.trim().is_empty()
            || responsibility.trim().is_empty()
        {
            return Err(AuditError::new(
                &path,
                "id, owner, reference, role and responsibility must be non-empty",
            ));
        }

        if !dependency_ids.insert(id.clone()) {
            return Err(AuditError::new(format!("{path}.id"), "duplicate dependency id"));
        }
        if !dependency_refs.insert(reference.clone()) {
            return Err(AuditError::new(
                format!("{path}.reference"),
                "duplicate owner-native dependency reference",
            ));
        }
        if !dependency_roles.insert(role.clone()) {
            return Err(AuditError::new(
                format!("{path}.role"),
                "duplicate owner-native dependency role",
            ));
        }
    }

    for (expected_id, expected_owner, expected_ref, expected_role) in
        REQUIRED_OWNER_NATIVE_DEPENDENCIES
    {
        let found = dependencies.iter().any(|item| {
            item.get("id").and_then(Value::as_str) == Some(*expected_id)
                && item.get("owner").and_then(Value::as_str) == Some(*expected_owner)
                && item.get("reference").and_then(Value::as_str) == Some(*expected_ref)
                && item.get("role").and_then(Value::as_str) == Some(*expected_role)
        });
        if !found {
            return Err(AuditError::new(
                "$.scope_boundary.owner_native_dependencies",
                format!("missing or mismatched owner-native dependency: {expected_id}"),
            ));
        }
    }

    Ok(())
}

fn validate_case_matrix(
    root: &Value,
    arm_ids: &BTreeSet<String>,
    scenario_ids: &BTreeSet<String>,
) -> Result<(), AuditError> {
    let matrix = root
        .get("case_matrix")
        .ok_or_else(|| AuditError::new("$.case_matrix", "missing field"))?;
    let matrix_object = matrix
        .as_object()
        .ok_or_else(|| AuditError::new("$.case_matrix", "expected an object"))?;
    check_keys(
        matrix_object,
        &[
            "comparison_arm_count",
            "scenario_count",
            "registered_case_count",
            "state",
        ],
        "$.case_matrix",
    )?;

    expect_usize(matrix, "comparison_arm_count", 5, "$.case_matrix")?;
    expect_usize(matrix, "scenario_count", 10, "$.case_matrix")?;
    expect_usize(matrix, "registered_case_count", 50, "$.case_matrix")?;
    expect_equal(
        &required_string(matrix, "state", "$.case_matrix")?,
        "REGISTERED_DESIGN_ONLY",
        "$.case_matrix.state",
    )?;

    let cases = required_array(root, "cases", "$")?;
    if cases.len() != 50 {
        return Err(AuditError::new(
            "$.cases",
            format!("expected 50 cases, found {}", cases.len()),
        ));
    }

    let mut case_ids = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    for (index, case) in cases.iter().enumerate() {
        let path = format!("$.cases[{index}]");
        let object = case
            .as_object()
            .ok_or_else(|| AuditError::new(&path, "expected an object"))?;
        check_keys(
            object,
            &[
                "id",
                "comparison_arm",
                "scenario",
                "definition_state",
                "execution_status",
                "numerical_inputs_state",
                "result_state",
            ],
            &path,
        )?;

        let id = required_string(case, "id", &path)?;
        let arm = required_string(case, "comparison_arm", &path)?;
        let scenario = required_string(case, "scenario", &path)?;
        let expected_case_id = format!("{arm}__{scenario}");
        expect_equal(&id, &expected_case_id, &format!("{path}.id"))?;
        expect_equal(
            &required_string(case, "definition_state", &path)?,
            "REGISTERED_DESIGN_ONLY",
            &format!("{path}.definition_state"),
        )?;
        expect_equal(
            &required_string(case, "execution_status", &path)?,
            "NOT_EXECUTED",
            &format!("{path}.execution_status"),
        )?;
        expect_equal(
            &required_string(case, "numerical_inputs_state", &path)?,
            "NOT_SUPPLIED",
            &format!("{path}.numerical_inputs_state"),
        )?;
        expect_equal(
            &required_string(case, "result_state", &path)?,
            "NO_RESULT",
            &format!("{path}.result_state"),
        )?;

        if !arm_ids.contains(&arm) {
            return Err(AuditError::new(
                format!("{path}.comparison_arm"),
                "unknown comparison arm",
            ));
        }
        if !scenario_ids.contains(&scenario) {
            return Err(AuditError::new(
                format!("{path}.scenario"),
                "unknown scenario",
            ));
        }
        if !case_ids.insert(id) {
            return Err(AuditError::new(format!("{path}.id"), "duplicate case id"));
        }
        let pair = format!("{arm}::{scenario}");
        if !pairs.insert(pair.clone()) {
            return Err(AuditError::new(
                &path,
                format!("duplicate arm/scenario pair: {pair}"),
            ));
        }
    }

    let expected_pairs = arm_ids.len() * scenario_ids.len();
    if pairs.len() != expected_pairs {
        return Err(AuditError::new(
            "$.cases",
            format!(
                "expected complete Cartesian coverage of {expected_pairs} arm/scenario pairs, found {}",
                pairs.len()
            ),
        ));
    }

    Ok(())
}

fn validate_reference_sources(root: &Value) -> Result<(), AuditError> {
    let sources = required_array(root, "reference_sources", "$")?;
    if sources.len() < 43 {
        return Err(AuditError::new(
            "$.reference_sources",
            format!("expected at least 43 sources, found {}", sources.len()),
        ));
    }

    let mut ids = BTreeSet::new();
    let mut urls = BTreeSet::new();
    for (index, source) in sources.iter().enumerate() {
        let path = format!("$.reference_sources[{index}]");
        let object = source
            .as_object()
            .ok_or_else(|| AuditError::new(&path, "expected an object"))?;
        check_keys(object, &["id", "url", "role"], &path)?;
        let id = required_string(source, "id", &path)?;
        let url = required_string(source, "url", &path)?;
        let role = required_string(source, "role", &path)?;
        if id.trim().is_empty() || url.trim().is_empty() || role.trim().is_empty() {
            return Err(AuditError::new(&path, "id, url and role must be non-empty"));
        }
        if !ids.insert(id.clone()) {
            return Err(AuditError::new(format!("{path}.id"), "duplicate source id"));
        }
        if !urls.insert(url.clone()) {
            return Err(AuditError::new(format!("{path}.url"), "duplicate source URL"));
        }
    }
    Ok(())
}

fn validate_targets(root: &Value) -> Result<(), AuditError> {
    let targets = root
        .get("targets")
        .and_then(Value::as_object)
        .ok_or_else(|| AuditError::new("$.targets", "expected an object"))?;
    let expected = [
        "same_scope_or_not_comparable",
        "critical_missing_evidence_fails_closed",
        "unserved_demand_retained",
        "all_lifecycle_exclusions_reported",
        "no_weighted_score_overrides_hard_gates",
        "exact_oracle_agreement_required",
        "targets_are_proposed_not_measured",
    ];
    check_keys(targets, &expected, "$.targets")?;
    for key in expected {
        if targets.get(key).and_then(Value::as_bool) != Some(true) {
            return Err(AuditError::new(
                format!("$.targets.{key}"),
                "must remain true in the v0.1 protocol",
            ));
        }
    }
    Ok(())
}

fn check_keys(
    object: &serde_json::Map<String, Value>,
    allowed: &[&str],
    path: &str,
) -> Result<(), AuditError> {
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(AuditError::new(
                format!("{path}.{key}"),
                "unknown field; update the versioned contract before adding fields",
            ));
        }
    }
    for key in allowed {
        if !object.contains_key(*key) {
            return Err(AuditError::new(
                format!("{path}.{key}"),
                "required field is missing",
            ));
        }
    }
    Ok(())
}

fn required_string(value: &Value, key: &str, path: &str) -> Result<String, AuditError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| AuditError::new(format!("{path}.{key}"), "expected a string"))
}

fn required_array<'a>(
    value: &'a Value,
    key: &str,
    path: &str,
) -> Result<&'a Vec<Value>, AuditError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| AuditError::new(format!("{path}.{key}"), "expected an array"))
}

fn collect_object_ids(
    items: &[Value],
    key: &str,
    path: &str,
) -> Result<BTreeSet<String>, AuditError> {
    let mut ids = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let id = required_string(item, key, &format!("{path}[{index}]"))?;
        if !ids.insert(id.clone()) {
            return Err(AuditError::new(
                format!("{path}[{index}].{key}"),
                format!("duplicate id: {id}"),
            ));
        }
    }
    Ok(ids)
}

fn collect_string_set(items: &[Value], path: &str) -> Result<BTreeSet<String>, AuditError> {
    let mut values = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let value = item
            .as_str()
            .ok_or_else(|| AuditError::new(format!("{path}[{index}]"), "expected a string"))?;
        if value.trim().is_empty() {
            return Err(AuditError::new(
                format!("{path}[{index}]"),
                "must not be empty",
            ));
        }
        if !values.insert(value.to_owned()) {
            return Err(AuditError::new(
                format!("{path}[{index}]"),
                format!("duplicate value: {value}"),
            ));
        }
    }
    Ok(values)
}

fn expect_same_set(
    actual: &BTreeSet<String>,
    expected: &[&str],
    path: &str,
) -> Result<(), AuditError> {
    let expected_set = expected
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    if actual != &expected_set {
        return Err(AuditError::new(path, "identifier set differs from frozen v0.1 contract"));
    }
    Ok(())
}

fn expect_equal(actual: &str, expected: &str, path: &str) -> Result<(), AuditError> {
    if actual != expected {
        return Err(AuditError::new(
            path,
            format!("expected {expected:?}, found {actual:?}"),
        ));
    }
    Ok(())
}

fn expect_usize(value: &Value, key: &str, expected: usize, path: &str) -> Result<(), AuditError> {
    let actual = value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|number| usize::try_from(number).ok())
        .ok_or_else(|| AuditError::new(format!("{path}.{key}"), "expected a non-negative integer"))?;
    if actual != expected {
        return Err(AuditError::new(
            format!("{path}.{key}"),
            format!("expected {expected}, found {actual}"),
        ));
    }
    Ok(())
}
