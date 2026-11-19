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
    assert_eq!(summary.registered_case_count, 50);
    assert_eq!(summary.unique_case_pairs, 50);
    assert_eq!(summary.amenity_archetype_count, 10);
    assert_eq!(summary.functional_unit_count, 9);
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
