//! Strict offline checker for the E0 synthetic Highveld household-service package.
//! This verifies package semantics only; it is not a building, product or safety validator.

use serde::Deserialize;

pub const PACKAGE_JSON: &str = include_str!(
    "../../../docs/research/highveld-keep/HIGHVELD_HOUSEHOLD_SERVICE_PACKAGE_V0_1.json"
);
pub const PACKAGE_SCHEMA_JSON: &str = include_str!(
    "../../../docs/research/highveld-keep/HIGHVELD_HOUSEHOLD_SERVICE_PACKAGE_V0_1.schema.json"
);

const PACKAGE_ID: &str = "HIGHVELD-HOUSEHOLD-SERVICE-SYNTHETIC-001";
const SCENARIO_ID: &str = "HIGHVELD-KEEP-SYNTHETIC-001";
const SCHEMA_VERSION: &str = "luminous.highveld.household-service-package/0.1";
const SERVICE_IDS: [&str; 3] = ["WATERLOOP", "HEATBANK", "AIRCARE"];
const ARCHITECTURE_IDS: [&str; 3] = [
    "INDEPENDENT_SERVICES",
    "SHARED_UTILITY",
    "HYBRID_LOCAL_AUTONOMY",
];

fn deserialize_required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    schema_version: String,
    package_id: String,
    revision_status: String,
    claim_ceiling: String,
    design_reference: DesignReference,
    household_profile: HouseholdProfile,
    services: Vec<Service>,
    architectures: Vec<Architecture>,
    stress_cases: Vec<StressCase>,
    comparison_protocol: ComparisonProtocol,
    execution_authority: ExecutionAuthority,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DesignReference {
    scenario_id: String,
    design_branch: String,
    evidence_tier: String,
    site_state: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HouseholdProfile {
    occupants_min: u8,
    occupants_max: u8,
    profile_state: String,
    real_resident_requirements: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Quantity {
    #[serde(deserialize_with = "deserialize_required_nullable")]
    value: Option<f64>,
    unit: String,
    state: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    source_ref: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputStream {
    id: String,
    quantity: Quantity,
    quality_state: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceFlow {
    id: String,
    direction: String,
    quantity: Quantity,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpatialInterface {
    space_ref: String,
    purpose: String,
    geometry_state: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OperatingProfile {
    duty_cycle_state: String,
    concurrency_state: String,
    schedule_state: String,
    calibration_state: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fallback {
    manual_path_state: String,
    local_control_state: String,
    network_required_for_essential_service: bool,
    cloud_required_for_essential_service: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Maintenance {
    service_access_state: String,
    inspection_interval_state: String,
    replacement_parts_state: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Service {
    id: String,
    product_name: String,
    owner: String,
    owner_issue_url: String,
    product_revision: String,
    status: String,
    service_definition: String,
    functional_unit: String,
    performance_target: Quantity,
    hard_service_floor_state: String,
    input_streams: Vec<InputStream>,
    resource_flows: Vec<ResourceFlow>,
    spatial_interfaces: Vec<SpatialInterface>,
    operating_profile: OperatingProfile,
    fallback: Fallback,
    maintenance: Maintenance,
    evidence_state: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Architecture {
    id: String,
    description: String,
    status: String,
    coupling_assumptions: Vec<String>,
    common_mode_failure_considered: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StressCase {
    id: String,
    description: String,
    event_family: String,
    source_class: String,
    timing_state: String,
    trace_status: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComparisonProtocol {
    paired_trace_policy: String,
    trace_bundle_status: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    common_trace_digest: Option<String>,
    information_budget_state: String,
    human_time_budget_state: String,
    comparison_arms: Vec<String>,
    reported_dimensions: Vec<String>,
    hard_gate_policy: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionAuthority {
    authority_state: String,
    model_recommendations_can_dispatch: bool,
    device_commands_in_profile: bool,
    household_telemetry_policy: String,
    cloud_required_for_essential_service: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationReport {
    pub package_id: String,
    pub service_count: usize,
    pub architecture_count: usize,
    pub stress_case_count: usize,
    pub violations: Vec<&'static str>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }
}

fn quantity_is_unknown(quantity: &Quantity) -> bool {
    quantity.value.is_none()
        && quantity.state == "UNKNOWN"
        && quantity.source_ref.is_none()
        && !quantity.unit.trim().is_empty()
}

fn unique_ids(values: impl Iterator<Item = String>, expected: &[&str]) -> bool {
    let actual: Vec<String> = values.collect();
    actual.len() == expected.len()
        && expected.iter().all(|wanted| actual.iter().filter(|id| id.as_str() == *wanted).count() == 1)
}

pub fn validate_package(package: &Package) -> ValidationReport {
    let mut violations = Vec::new();

    if package.schema_version != SCHEMA_VERSION {
        violations.push("SCHEMA_VERSION_MISMATCH");
    }
    if package.package_id != PACKAGE_ID {
        violations.push("PACKAGE_ID_MISMATCH");
    }
    if package.revision_status != "E0_SYNTHETIC_REFERENCE" {
        violations.push("REVISION_STATUS_PROMOTED");
    }
    if package.claim_ceiling != "service_definition_and_fixture_integrity_only" {
        violations.push("CLAIM_CEILING_CHANGED");
    }
    if package.design_reference.scenario_id != SCENARIO_ID
        || package.design_reference.evidence_tier != "E0"
        || package.design_reference.site_state != "SYNTHETIC_REFERENCE_ONLY"
        || package.design_reference.design_branch.trim().is_empty()
    {
        violations.push("DESIGN_REFERENCE_INVALID");
    }
    if package.household_profile.occupants_min != 4
        || package.household_profile.occupants_max != 6
        || package.household_profile.profile_state != "SYNTHETIC_BENCHMARK_ASSUMPTION"
        || package.household_profile.real_resident_requirements != "NOT_COLLECTED"
    {
        violations.push("HOUSEHOLD_PROFILE_PROMOTED");
    }

    if !unique_ids(package.services.iter().map(|item| item.id.clone()), &SERVICE_IDS) {
        violations.push("SERVICE_ID_SET_INVALID");
    }
    for service in &package.services {
        let expected_issue_url = match service.id.as_str() {
            "WATERLOOP" => "https://github.com/Luminous-Dynamics/symthaea/issues/10323",
            "HEATBANK" => "https://github.com/Luminous-Dynamics/symthaea/issues/10329",
            "AIRCARE" => "https://github.com/Luminous-Dynamics/symthaea/issues/10330",
            _ => "",
        };
        let (expected_functional_unit, expected_inputs, expected_flows, expected_spaces):
            (&str, &[(&str, &str)], &[(&str, &str, &str)], &[&str]) = match service.id.as_str() {
                "WATERLOOP" => (
                    "usable non-potable water delivered (litres) to one named permitted end use over a declared observation or outage window",
                    &[("source_water", "L per declared observation window"), ("electrical_supply", "kWh per declared observation window")],
                    &[("source_withdrawal", "INPUT", "L"), ("usable_non_potable_delivery", "OUTPUT", "L"), ("rejected_or_residual_water", "OUTPUT", "L"), ("electrical_energy", "INPUT", "kWh")],
                    &["service_plant", "outdoor_living"],
                ),
                "HEATBANK" => (
                    "accepted hot-water delivery (litres meeting a declared temperature and timing profile) under a matched draw sequence",
                    &[("electrical_supply", "kWh per declared observation window"), ("water_supply", "L per declared observation window"), ("thermal_source", "kWh_th per declared observation window")],
                    &[("electrical_energy", "INPUT", "kWh"), ("useful_thermal_service", "OUTPUT", "kWh_th"), ("water_draw", "INPUT", "L"), ("storage_loss", "OUTPUT", "kWh_th")],
                    &["service_plant", "accessible_washroom", "kitchen_dining"],
                ),
                "AIRCARE" => (
                    "pollutant-specific clean-air delivery and outdoor-air delivery at defined occupied-room positions over a declared room-use window",
                    &[("electrical_supply", "kWh per declared observation window"), ("outdoor_air", "m3 per declared observation window"), ("filter_media", "UNKNOWN_NOT_SELECTED")],
                    &[("electrical_energy", "INPUT", "kWh"), ("outdoor_air_delivery", "INPUT", "m3"), ("clean_air_delivery", "OUTPUT", "m3"), ("condensate_or_captured_residual", "OUTPUT", "UNKNOWN_NOT_SELECTED")],
                    &["sleeping_rooms", "great_hall", "service_plant"],
                ),
                _ => ("", &[], &[], &[]),
            };
        if service.product_name.trim().is_empty()
            || service.owner != "Symthaea"
            || service.owner_issue_url != expected_issue_url
            || service.product_revision != "UNKNOWN_NOT_SELECTED"
            || service.status != "RESEARCH_PROFILE_ONLY"
            || service.service_definition.trim().is_empty()
            || service.functional_unit != expected_functional_unit
            || service.input_streams.len() != expected_inputs.len()
            || service.input_streams.iter().zip(expected_inputs.iter()).any(|(item, (id, unit))| {
                item.id != *id || item.quantity.unit != *unit
            })
            || service.resource_flows.len() != expected_flows.len()
            || service.resource_flows.iter().zip(expected_flows.iter()).any(|(item, (id, direction, unit))| {
                item.id != *id || item.direction != *direction || item.quantity.unit != *unit
            })
            || !unique_ids(service.spatial_interfaces.iter().map(|item| item.space_ref.clone()), expected_spaces)
            || service.hard_service_floor_state
                != "UNKNOWN_PENDING_RESIDENT_AND_APPLICABLE_REQUIREMENTS"
            || service.evidence_state != "RESEARCH_PROFILE_ONLY"
            || !service.owner_issue_url.starts_with("https://github.com/Luminous-Dynamics/symthaea/issues/")
        {
            violations.push("SERVICE_PROFILE_PROMOTED_OR_INCOMPLETE");
        }
        if !quantity_is_unknown(&service.performance_target) {
            violations.push("SERVICE_PERFORMANCE_VALUE_NOT_QUALIFIED");
        }
        if service.input_streams.is_empty() || service.resource_flows.is_empty()
            || service.spatial_interfaces.is_empty()
        {
            violations.push("SERVICE_INTERFACE_INCOMPLETE");
        }
        for input in &service.input_streams {
            if input.id.trim().is_empty()
                || input.quality_state != "UNKNOWN"
                || !quantity_is_unknown(&input.quantity)
            {
                violations.push("INPUT_STREAM_EVIDENCE_PROMOTED");
            }
        }
        for flow in &service.resource_flows {
            if flow.id.trim().is_empty()
                || !matches!(flow.direction.as_str(), "INPUT" | "OUTPUT")
                || !quantity_is_unknown(&flow.quantity)
            {
                violations.push("RESOURCE_FLOW_EVIDENCE_PROMOTED");
            }
        }
        for interface in &service.spatial_interfaces {
            if interface.space_ref.trim().is_empty()
                || interface.purpose.trim().is_empty()
                || interface.geometry_state != "NOT_MODELED"
            {
                violations.push("SPATIAL_INTERFACE_PROMOTED");
            }
        }
        if service.operating_profile.duty_cycle_state != "UNKNOWN"
            || service.operating_profile.concurrency_state != "UNKNOWN"
            || service.operating_profile.schedule_state != "UNKNOWN"
            || service.operating_profile.calibration_state != "NOT_ESTABLISHED"
        {
            violations.push("OPERATING_PROFILE_PROMOTED");
        }
        if service.fallback.manual_path_state != "REQUIRED_NOT_QUALIFIED"
            || service.fallback.local_control_state != "REQUIRED_NOT_QUALIFIED"
            || service.fallback.network_required_for_essential_service
            || service.fallback.cloud_required_for_essential_service
        {
            violations.push("FALLBACK_OR_NETWORK_CLAIM_PROMOTED");
        }
        if service.maintenance.service_access_state != "REQUIREMENT_NOT_GEOMETRY_VERIFIED"
            || service.maintenance.inspection_interval_state != "UNKNOWN"
            || service.maintenance.replacement_parts_state != "UNKNOWN"
        {
            violations.push("MAINTENANCE_PROFILE_PROMOTED");
        }
    }

    if !unique_ids(package.architectures.iter().map(|item| item.id.clone()), &ARCHITECTURE_IDS) {
        violations.push("ARCHITECTURE_ID_SET_INVALID");
    }
    for architecture in &package.architectures {
        if architecture.status != "NOT_EVALUATED"
            || architecture.description.trim().is_empty()
            || architecture.coupling_assumptions.is_empty()
            || !architecture.common_mode_failure_considered
        {
            violations.push("ARCHITECTURE_EVALUATION_OR_FAILURE_BOUNDARY_INVALID");
        }
    }

    if package.stress_cases.len() != 14 {
        violations.push("STRESS_CASE_COUNT_INVALID");
    }
    let mut stress_ids: Vec<&str> = package.stress_cases.iter().map(|item| item.id.as_str()).collect();
    stress_ids.sort_unstable();
    let expected_stress_ids: Vec<String> = (1..=14).map(|n| format!("HHS-{n:03}")).collect();
    if stress_ids.len() != expected_stress_ids.len()
        || stress_ids.iter().zip(expected_stress_ids.iter()).any(|(got, wanted)| *got != wanted.as_str())
    {
        violations.push("STRESS_CASE_ID_SET_INVALID");
    }
    for item in &package.stress_cases {
        let expected_event_family = match item.id.as_str() {
            "HHS-001" => "CONCURRENCY",
            "HHS-002" | "HHS-003" => "THERMAL_WEATHER",
            "HHS-004" | "HHS-005" => "ELECTRICAL_OUTAGE",
            "HHS-006" => "WATER_OUTAGE",
            "HHS-007" => "AIR_QUALITY",
            "HHS-008" => "COMPONENT_FAULT",
            "HHS-009" => "COMMON_MODE_FAILURE",
            "HHS-010" => "MAINTENANCE_DELAY",
            "HHS-011" => "OBSERVATION_FAILURE",
            "HHS-012" => "HOUSEHOLD_CHANGE",
            "HHS-013" => "RESOURCE_ALLOCATION",
            "HHS-014" => "REPEAT_DISTURBANCE",
            _ => "",
        };
        if item.description.trim().is_empty()
            || item.event_family != expected_event_family
            || item.source_class != "SYNTHETIC_CASE_DEFINITION"
            || item.timing_state != "UNSPECIFIED"
            || item.trace_status != "NOT_MATERIALIZED"
        {
            violations.push("STRESS_CASE_PROMOTED_OR_MISBOUND");
        }
    }

    let comparison = &package.comparison_protocol;
    if comparison.paired_trace_policy != "SAME_EXACT_TRACE_BYTES_FOR_ALL_COMPARISON_ARMS"
        || comparison.trace_bundle_status != "NOT_MATERIALIZED"
        || comparison.common_trace_digest.is_some()
        || comparison.information_budget_state != "UNSPECIFIED"
        || comparison.human_time_budget_state != "UNSPECIFIED"
        || comparison.hard_gate_policy != "NON_COMPENSATORY_NO_AGGREGATE_SCORE"
        || !unique_ids(comparison.comparison_arms.iter().cloned(), &ARCHITECTURE_IDS)
        || comparison.reported_dimensions.len() < 5
    {
        violations.push("COMPARISON_PROTOCOL_PROMOTED_OR_INCOMPLETE");
    }

    let authority = &package.execution_authority;
    if authority.authority_state != "NONE"
        || authority.model_recommendations_can_dispatch
        || authority.device_commands_in_profile
        || authority.household_telemetry_policy != "OPT_IN_PURPOSE_SPECIFIC_ONLY"
        || authority.cloud_required_for_essential_service
    {
        violations.push("EXECUTION_AUTHORITY_NONEMPTY");
    }

    ValidationReport {
        package_id: package.package_id.clone(),
        service_count: package.services.len(),
        architecture_count: package.architectures.len(),
        stress_case_count: package.stress_cases.len(),
        violations,
    }
}

pub fn validate_json(source: &str) -> Result<ValidationReport, String> {
    let package: Package = serde_json::from_str(source)
        .map_err(|error| format!("STRICT_JSON_REJECTED: {error}"))?;
    Ok(validate_package(&package))
}

pub fn validate_tracked_package() -> Result<ValidationReport, String> {
    validate_json(PACKAGE_JSON)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mutate_once(needle: &str, replacement: &str) -> String {
        let changed = PACKAGE_JSON.replacen(needle, replacement, 1);
        assert_ne!(changed, PACKAGE_JSON, "mutation fixture did not match the tracked package");
        changed
    }

    fn remove_once(needle: &str) -> String {
        let changed = PACKAGE_JSON.replacen(needle, "", 1);
        assert_ne!(changed, PACKAGE_JSON, "removal fixture did not match the tracked package");
        changed
    }

    #[test]
    fn schema_is_parseable_and_every_schema_object_is_closed() {
        let schema: serde_json::Value =
            serde_json::from_str(PACKAGE_SCHEMA_JSON).expect("schema JSON parses");
        fn visit(value: &serde_json::Value, path: &str) {
            if let Some(object) = value.as_object() {
                if object.get("type").and_then(serde_json::Value::as_str) == Some("object") {
                    assert_eq!(
                        object.get("additionalProperties"),
                        Some(&serde_json::Value::Bool(false)),
                        "open object schema at {path}"
                    );
                }
                for (key, child) in object {
                    visit(child, &format!("{path}.{key}"));
                }
            } else if let Some(array) = value.as_array() {
                for (index, child) in array.iter().enumerate() {
                    visit(child, &format!("{path}[{index}]"));
                }
            }
        }
        visit(&schema, "$");
    }

    #[test]
    fn tracked_package_is_valid_synthetic_only() {
        let report = validate_tracked_package().expect("tracked package parses strictly");
        assert!(report.is_valid(), "violations: {:?}", report.violations);
        assert_eq!(report.service_count, 3);
        assert_eq!(report.architecture_count, 3);
        assert_eq!(report.stress_case_count, 14);
    }

    #[test]
    fn unknown_root_field_is_rejected() {
        let changed = mutate_once(
            "\"package_id\":",
            "\"unreviewed_root_field\":true,\"package_id\":",
        );
        assert!(validate_json(&changed).is_err());
    }

    #[test]
    fn unknown_nested_field_is_rejected() {
        let changed = mutate_once(
            "\"profile_state\":",
            "\"unreviewed_nested_field\":true,\"profile_state\":",
        );
        assert!(validate_json(&changed).is_err());
    }

    #[test]
    fn duplicate_root_key_is_rejected() {
        let changed = mutate_once(
            "\"package_id\":",
            "\"package_id\":\"HIGHVELD-HOUSEHOLD-SERVICE-SYNTHETIC-001\",\"package_id\":",
        );
        assert!(validate_json(&changed).is_err());
    }

    #[test]
    fn duplicate_nested_key_is_rejected() {
        let changed = mutate_once(
            "\"status\": \"RESEARCH_PROFILE_ONLY\"",
            "\"status\": \"RESEARCH_PROFILE_ONLY\", \"status\": \"EVALUATED\"",
        );
        assert!(validate_json(&changed).is_err());
    }

    #[test]
    fn generic_functional_unit_is_rejected() {
        let changed = mutate_once(
            "pollutant-specific clean-air delivery and outdoor-air delivery at defined occupied-room positions over a declared room-use window",
            "one generic household service over a window",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"SERVICE_PROFILE_PROMOTED_OR_INCOMPLETE"));
    }

    #[test]
    fn nullable_quantity_value_must_be_present_even_when_null() {
        let changed = remove_once("\"value\": null,\n");
        assert!(validate_json(&changed).is_err());
    }

    #[test]
    fn nullable_quantity_source_ref_must_be_present_even_when_null() {
        let changed = remove_once("\"source_ref\": null,\n");
        assert!(validate_json(&changed).is_err());
    }

    #[test]
    fn nullable_trace_digest_must_be_present_even_when_null() {
        let changed = remove_once("\"common_trace_digest\": null,\n");
        assert!(validate_json(&changed).is_err());
    }

    #[test]
    fn resource_flow_unit_drift_is_rejected() {
        let changed = mutate_once(
            "\"unit\": \"kWh\"",
            "\"unit\": \"MJ\"",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"SERVICE_PROFILE_PROMOTED_OR_INCOMPLETE"));
    }

    #[test]
    fn unknown_quantity_cannot_carry_a_number() {
        let changed = mutate_once(
            "\"value\": null",
            "\"value\": 0.0",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"SERVICE_PERFORMANCE_VALUE_NOT_QUALIFIED"));
    }

    #[test]
    fn evaluated_architecture_is_rejected() {
        let changed = mutate_once(
            "\"status\": \"NOT_EVALUATED\"",
            "\"status\": \"QUALIFIED\"",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"ARCHITECTURE_EVALUATION_OR_FAILURE_BOUNDARY_INVALID"));
    }

    #[test]
    fn trace_digest_cannot_appear_before_trace_materialization() {
        let changed = mutate_once(
            "\"common_trace_digest\": null",
            "\"common_trace_digest\": \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"COMPARISON_PROTOCOL_PROMOTED_OR_INCOMPLETE"));
    }

    #[test]
    fn model_cannot_authorize_execution() {
        let changed = mutate_once(
            "\"model_recommendations_can_dispatch\": false",
            "\"model_recommendations_can_dispatch\": true",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"EXECUTION_AUTHORITY_NONEMPTY"));
    }

    #[test]
    fn unexpected_service_identity_is_rejected() {
        let changed = mutate_once(
            "\"id\": \"WATERLOOP\"",
            "\"id\": \"EXTRA_SERVICE\"",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"SERVICE_ID_SET_INVALID"));
    }

    #[test]
    fn service_stream_identity_drift_is_rejected() {
        let changed = mutate_once(
            "\"id\": \"source_water\"",
            "\"id\": \"generic_input\"",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"SERVICE_PROFILE_PROMOTED_OR_INCOMPLETE"));
    }

    #[test]
    fn stress_case_family_drift_is_rejected() {
        let changed = mutate_once(
            "\"event_family\": \"CONCURRENCY\"",
            "\"event_family\": \"GENERIC_EVENT\"",
        );
        let report = validate_json(&changed).expect("typed JSON remains parseable");
        assert!(report.violations.contains(&"STRESS_CASE_PROMOTED_OR_MISBOUND"));
    }

    #[test]
    fn unknown_spatial_interface_field_is_rejected() {
        let changed = mutate_once(
            "\"purpose\":",
            "\"unreviewed_interface_field\":true,\"purpose\":",
        );
        assert!(validate_json(&changed).is_err());
    }
}
