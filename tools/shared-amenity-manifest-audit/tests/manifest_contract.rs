use serde_json::{Value, json};
use shared_amenity_manifest_audit::audit_manifest_bytes;

const FIXTURE: &str =
    include_str!("../../../docs/research/highveld-neighborhood-commons/SHARED_AMENITY_COMMONS_MANIFEST_V0.1.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("checked-in manifest must parse")
}

fn audit(value: &Value) -> Result<shared_amenity_manifest_audit::AuditSummary, shared_amenity_manifest_audit::AuditError> {
    let bytes = serde_json::to_vec(value).expect("test JSON serialization");
    audit_manifest_bytes(&bytes)
}

#[test]
fn frozen_manifest_has_complete_case_coverage() {
    let summary = audit_manifest_bytes(FIXTURE.as_bytes()).expect("manifest should pass");
    assert_eq!(summary.comparison_arm_count, 5);
    assert_eq!(summary.scenario_count, 10);
    assert_eq!(summary.scenario_definition_count, 10);
    assert_eq!(summary.registered_case_count, 50);
    assert_eq!(summary.unique_case_pairs, 50);
    assert_eq!(summary.amenity_archetype_count, 10);
    assert_eq!(summary.functional_unit_count, 9);
    assert_eq!(summary.reference_source_count, 43);
    assert_eq!(summary.owner_native_dependency_count, 16);
    assert_eq!(summary.metric_vector_count, 8);
    assert_eq!(summary.execution_status, "NOT_EXECUTED");
    assert_eq!(summary.manifest_sha256.len(), 64);
}

#[test]
fn missing_case_is_rejected() {
    let mut value = fixture();
    value["cases"].as_array_mut().expect("cases array").pop();
    assert!(audit(&value).is_err());
}

#[test]
fn duplicate_arm_scenario_pair_is_rejected() {
    let mut value = fixture();
    let first = value["cases"][0].clone();
    value["cases"][1]["comparison_arm"] = first["comparison_arm"].clone();
    value["cases"][1]["scenario"] = first["scenario"].clone();
    assert!(audit(&value).is_err());
}

#[test]
fn optimistic_execution_state_is_rejected() {
    let mut value = fixture();
    value["cases"][0]["execution_status"] = json!("PASS");
    value["cases"][0]["result_state"] = json!("PASS");
    assert!(audit(&value).is_err());
}

#[test]
fn functional_unit_cannot_be_reassigned_to_another_amenity() {
    let mut value = fixture();
    value["functional_units"][0]["amenity_id"] = json!("SHARED_LAUNDRY");
    assert!(audit(&value).is_err());
}

#[test]
fn enabling_plant_cannot_claim_an_independent_service_unit() {
    let mut value = fixture();
    let plant = value["amenity_archetypes"]
        .as_array_mut()
        .expect("amenities array")
        .iter_mut()
        .find(|item| item["id"] == "NON_ESSENTIAL_AMENITY_PLANT")
        .expect("enabling plant");
    plant["functional_unit_id"] = json!("SHARED_UTILITY_SERVICE_HOUR");
    assert!(audit(&value).is_err());
}

#[test]
fn scope_exclusions_cannot_be_removed_silently() {
    let mut value = fixture();
    value["scope_boundary"]["excluded_scopes"]
        .as_array_mut()
        .expect("excluded scopes")
        .retain(|item| item["reference"] != "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/737");
    assert!(audit(&value).is_err());
}

#[test]
fn unknown_root_field_is_rejected() {
    let mut value = fixture();
    value["unreviewed_extension"] = json!(true);
    assert!(audit(&value).is_err());
}

#[test]
fn greenhouse_and_home_owner_dependencies_cannot_be_removed() {
    let mut value = fixture();
    value["scope_boundary"]["owner_native_dependencies"]
        .as_array_mut()
        .expect("owner dependencies")
        .retain(|item| item["reference"] != "https://github.com/Luminous-Dynamics/symtropy/issues/3957");
    assert!(audit(&value).is_err());
}

#[test]
fn current_sota_and_owner_source_register_cannot_be_thinned() {
    let mut value = fixture();
    value["reference_sources"]
        .as_array_mut()
        .expect("reference sources")
        .pop();
    assert!(audit(&value).is_err());
}

#[test]
fn canonical_mycelix_amenity_service_contract_cannot_be_removed() {
    let mut value = fixture();
    value["scope_boundary"]["owner_native_dependencies"]
        .as_array_mut()
        .expect("owner dependencies")
        .retain(|item| item["reference"] != "https://github.com/Luminous-Dynamics/mycelix/issues/6328");
    assert!(audit(&value).is_err());
}

#[test]
fn city_morph_effective_access_owner_cannot_be_removed() {
    let mut value = fixture();
    value["scope_boundary"]["owner_native_dependencies"]
        .as_array_mut()
        .expect("owner dependencies")
        .retain(|item| item["reference"] != "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/555");
    assert!(audit(&value).is_err());
}

#[test]
fn obsolete_operational_scenario_is_rejected() {
    let mut value = fixture();
    value["scenarios"][0] = json!("ORDINARY_DEMAND");
    assert!(audit(&value).is_err());
}

#[test]
fn case_identity_must_match_its_arm_and_scenario() {
    let mut value = fixture();
    value["cases"][0]["id"] = json!("ARBITRARY_CASE_ID");
    assert!(audit(&value).is_err());
}

#[test]
fn included_service_classes_are_a_frozen_exact_set() {
    let mut value = fixture();
    value["scope_boundary"]["included_service_classes"]
        .as_array_mut()
        .expect("included service classes")
        .push(json!("unreviewed_service_class"));
    assert!(audit(&value).is_err());
}

#[test]
fn owner_aware_metric_vector_cannot_be_silently_removed() {
    let mut value = fixture();
    value["metric_vectors"]
        .as_array_mut()
        .expect("metric vectors")
        .retain(|item| item != "ROBUSTNESS_AND_RECONFIGURATION");
    assert!(audit(&value).is_err());
}

#[test]
fn incomplete_scenario_definition_registry_is_rejected() {
    let mut value = fixture();
    value["scenario_definitions"]
        .as_array_mut()
        .expect("scenario definitions")
        .pop();
    assert!(audit(&value).is_err());
}

#[test]
fn invented_scenario_probabilities_are_rejected() {
    let mut value = fixture();
    value["scenario_definitions"][0]["probability_model_state"] = json!("ASSIGNED_0_25");
    assert!(audit(&value).is_err());
}

#[test]
fn unregistered_owner_output_in_scenario_definition_is_rejected() {
    let mut value = fixture();
    value["scenario_definitions"][0]["required_owner_outputs"] = json!(["PARALLEL_UNKNOWN_SERVICE_ORACLE"]);
    assert!(audit(&value).is_err());
}

#[test]
fn duplicate_json_object_keys_are_rejected_before_map_deserialization() {
    let duplicate = FIXTURE.replacen(
        "\"schema\": \"luminous.shared-amenity-commons-benchmark\",",
        "\"schema\": \"luminous.shared-amenity-commons-benchmark\", \"schema\": \"luminous.shared-amenity-commons-benchmark\",",
        1,
    );
    assert_ne!(duplicate, FIXTURE);
    let error = audit_manifest_bytes(duplicate.as_bytes())
        .expect_err("duplicate JSON object keys must fail closed");
    assert!(error.message.contains("duplicate object key"));
}

#[test]
fn duplicate_nested_scenario_keys_are_rejected_recursively() {
    let duplicate = FIXTURE.replacen(
        "\"scenario_id\": \"COMPACT_HIGH_DENSITY\",",
        "\"scenario_id\": \"COMPACT_HIGH_DENSITY\", \"scenario_id\": \"COMPACT_HIGH_DENSITY\",",
        1,
    );
    assert_ne!(duplicate, FIXTURE);
    let error = audit_manifest_bytes(duplicate.as_bytes())
        .expect_err("duplicate nested scenario keys must fail closed");
    assert!(error.message.contains("duplicate object key"));
}
