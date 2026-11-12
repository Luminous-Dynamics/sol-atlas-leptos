use serde_json::{Value, json};
use std::{collections::BTreeSet, env, process};

const MANIFEST: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_WATER_FEATURE_THERMAL_MANIFEST_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_WATER_FEATURE_THERMAL_MANIFEST_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn unique_ids(items: &[Value]) -> bool {
    let mut seen = BTreeSet::new();
    items.iter().all(|item| {
        text_at(item, "id").is_some_and(|id| seen.insert(id.to_owned()))
    })
}

fn contains_id(items: &[Value], id: &str) -> bool {
    items.iter().any(|item| text_at(item, "id") == Some(id))
}

fn audit() -> Result<Value, Vec<String>> {
    let manifest: Value = serde_json::from_str(MANIFEST)
        .map_err(|e| vec![format!("pool/water manifest JSON parse failed: {e}")])?;
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|e| vec![format!("pool/water schema JSON parse failed: {e}")])?;
    let mut errors = Vec::new();

    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("pool/water schema must declare JSON Schema Draft 2020-12".to_owned());
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

    let archetypes = array_at(&manifest, "archetypes").cloned().unwrap_or_default();
    let scenarios = array_at(&manifest, "scenarios").cloned().unwrap_or_default();
    let cover_modes = array_at(&manifest, "cover_modes").cloned().unwrap_or_default();
    if archetypes.len() != 9 || !unique_ids(&archetypes) {
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
    if pool_buffer.is_none_or(|item| {
        text_at(item, "thermal_role") != Some("FINITE_THERMAL_BUFFER_ONLY")
    }) {
        errors.push("pool must remain a finite buffer, not an infinite heat sink".to_owned());
    }
    let aerosol = archetypes
        .iter()
        .find(|item| text_at(item, "id") == Some("AEROSOL_GENERATING_FOUNTAIN"));
    if aerosol.is_none_or(|item| text_at(item, "aerosol_generation") != Some("YES")) {
        errors.push("decorative fountain aerosol generation must be explicit".to_owned());
    }

    let expansion = &manifest["case_expansion"];
    if expansion["archetypes"].as_u64() != Some(9)
        || expansion["scenarios"].as_u64() != Some(8)
        || expansion["cases"].as_u64() != Some(72)
        || text_at(expansion, "state") != Some("REGISTERED_NOT_EXECUTED")
    {
        errors.push("pool/water matrix must remain nine by eight, 72 registered cases".to_owned());
    }

    let balances = array_at(&manifest, "required_balance_terms").cloned().unwrap_or_default();
    for required in [
        "pool_or_cistern_usable_thermal_state_change",
        "heat_rejected_by_each_external_sink",
        "evaporation_latent_heat",
        "makeup_water_enthalpy",
        "zone_moisture_balance",
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
    ] {
        if manifest["hard_gates"].get(key).and_then(Value::as_u64) != Some(0) {
            errors.push(format!("pool/water hard gate {key} must equal zero"));
        }
    }
    if text_at(&manifest["solver_roles"], "indoor_pool_heat_balance")
        != Some("ENERGYPLUS_26_2_SWIMMINGPOOL_INDOOR")
        || text_at(&manifest["solver_roles"], "trajectory_accounting")
            != Some("HIGHVELD_THERMAL_STORE_LEDGER")
    {
        errors.push("specialist indoor pool physics and independent ledger roles must remain distinct".to_owned());
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
        "registered_case_count":72,
        "pool_role":"FINITE_THERMAL_BUFFER_NOT_INFINITE_SINK",
        "aerosol_water_management_gate":"REQUIRED_WHEN_AEROSOL_GENERATING",
        "execution_status":"NOT_EXECUTED",
        "schema_engine":"NOT_RUN; schema declaration and manifest invariants checked by this Rust qualifier",
        "claim_ceiling":"Design and benchmark registration only; no water feature or cooling performance admitted"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" | "--audit" => match audit() {
            Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default()),
            Err(errors) => {
                eprintln!("{}", serde_json::to_string_pretty(&json!({"errors":errors})).unwrap_or_default());
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
    fn all_nine_archetypes_and_72_cases_are_registered() {
        let report = audit().expect("pool/water programme contract");
        assert_eq!(report["archetype_count"], 9);
        assert_eq!(report["scenario_count"], 8);
        assert_eq!(report["registered_case_count"], 72);
    }

    #[test]
    fn pool_is_a_finite_buffer_not_an_infinite_sink() {
        let report = audit().expect("pool/water programme contract");
        assert_eq!(report["pool_role"], "FINITE_THERMAL_BUFFER_NOT_INFINITE_SINK");
    }

    #[test]
    fn aerosol_features_keep_health_management_gate() {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        let required = array_at(&manifest, "required_health_gates").unwrap();
        assert!(required.iter().any(|item| item == "water_management_program_when_aerosol_generating"));
        assert_eq!(
            report_aerosol_kind(&manifest, "AEROSOL_GENERATING_FOUNTAIN"),
            Some("YES")
        );
    }

    #[test]
    fn cover_exposure_schedules_are_explicit() {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        let modes = array_at(&manifest, "cover_modes").unwrap();
        assert!(modes.iter().any(|item| item == "DAY_SHADED_NIGHT_EXPOSED"));
        assert!(modes.iter().any(|item| item == "ADAPTIVE_COVER_BY_WATER_BUDGET_AND_SKY_CONDITIONS"));
    }

    fn report_aerosol_kind<'a>(manifest: &'a Value, id: &str) -> Option<&'a str> {
        array_at(manifest, "archetypes")?
            .iter()
            .find(|item| text_at(item, "id") == Some(id))
            .and_then(|item| text_at(item, "aerosol_generation"))
    }
}
