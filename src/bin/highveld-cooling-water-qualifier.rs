use serde_json::{Value, json};
use std::{env, process};

const MANIFEST: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/PASSIVE_COOLING_STUDY_MANIFEST_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/PASSIVE_COOLING_STUDY_MANIFEST_V0.1.schema.json"
);

fn string_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn has_string(values: &[Value], expected: &str) -> bool {
    values.iter().any(|value| value.as_str() == Some(expected))
}

fn unique_nonempty_ids(items: &[Value]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    items.iter().all(|item| {
        item.get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| !id.trim().is_empty() && seen.insert(id))
    })
}

fn build_report() -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let manifest: Value = match serde_json::from_str(MANIFEST) {
        Ok(value) => value,
        Err(error) => return Err(vec![format!("cooling manifest JSON parse failed: {error}")]),
    };
    let schema: Value = match serde_json::from_str(SCHEMA) {
        Ok(value) => value,
        Err(error) => return Err(vec![format!("cooling schema JSON parse failed: {error}")]),
    };

    if string_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("cooling schema must declare JSON Schema Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("format_version", "0.1"),
        ("study_id", "HIGHVELD-PASSIVE-COOLING-WATER-001"),
        ("parent_candidate", "NEIGHBORHOOD_RESILIENCE_COMMONS"),
        ("maturity", "DESIGN_HYPOTHESES_NOT_PHYSICALLY_EVALUATED"),
        ("climate_context", "JOHANNESBURG_HIGHVELD"),
        ("baseline_required", "NO_WATER_PASSIVE_BASELINE"),
        (
            "physics_owner",
            "SPECIALIST_BUILDING_AIRFLOW_HYDRAULIC_SOLVERS",
        ),
        ("event_and_trajectory_owner", "SYMTROPY"),
        ("geometry_and_configuration_owner", "SOL_ATLAS"),
        ("rights_and_water_account_owner", "MYCELIX"),
        ("execution_status", "NOT_EXECUTED"),
    ] {
        if string_at(&manifest, key) != Some(expected) {
            errors.push(format!("{key} must be {expected}"));
        }
    }

    let archetypes = array_at(&manifest, "archetypes")
        .cloned()
        .unwrap_or_default();
    if archetypes.len() < 7 || !unique_nonempty_ids(&archetypes) {
        errors.push("archetypes must contain at least seven unique non-empty IDs".to_owned());
    }
    let baseline = archetypes.iter().any(|item| {
        string_at(item, "id") == Some("NO_WATER_PASSIVE_BASELINE")
            && string_at(item, "role") == Some("PRIMARY_CONTROL")
    });
    if !baseline {
        errors.push("a no-water passive primary control is mandatory".to_owned());
    }
    for required in [
        "COVERED_POOL_THERMAL_MASS",
        "SHADED_RECIRCULATING_RILL",
        "EVAPORATIVE_DOWNSHAFT",
        "CLOSED_LOOP_THERMAL_WATER_STORE",
        "POOL_COURTYARD_HYBRID",
    ] {
        if !archetypes
            .iter()
            .any(|item| string_at(item, "id") == Some(required))
        {
            errors.push(format!("missing cooling archetype {required}"));
        }
    }
    let spray_control = archetypes.iter().any(|item| {
        string_at(item, "id") == Some("UNBOUNDED_SPRAY_NEGATIVE_CONTROL")
            && string_at(item, "role") == Some("NEGATIVE_CONTROL")
            && string_at(item, "expected_claim") == Some("MUST_NOT_QUALIFY")
    });
    if !spray_control {
        errors.push("unbounded spray must remain an explicit negative control".to_owned());
    }

    let comparisons = array_at(&manifest, "comparison_modes")
        .cloned()
        .unwrap_or_default();
    for required in [
        "WATER_LIMITED_PRIMARY",
        "THERMAL_CAPACITY_MATCHED",
        "ENERGY_BUDGET_MATCHED_SENSITIVITY",
        "DROUGHT_AND_CURTAILMENT",
        "HUMID_HOT",
        "COLD_NIGHT",
    ] {
        if !has_string(&comparisons, required) {
            errors.push(format!("missing comparison mode {required}"));
        }
    }

    for (field, required) in [
        ("matched_inputs_required", "weather_file_digest"),
        ("matched_inputs_required", "building_geometry_digest"),
        (
            "matched_inputs_required",
            "occupancy_and_internal_gains_digest",
        ),
        ("matched_inputs_required", "water_budget_and_reserve_digest"),
        ("metered_water_terms", "makeup_by_source_class"),
        ("metered_water_terms", "evaporation_estimate_with_method"),
        ("metered_water_terms", "leakage"),
        ("metered_water_terms", "closing_volume"),
        ("thermal_terms", "useful_heat_removed_from_occupied_zone"),
        ("thermal_terms", "latent_heat_at_evaporation_interface"),
        ("thermal_terms", "heat_rejection_path"),
        ("required_outcomes", "occupied_hour_operative_temperature"),
        ("required_outcomes", "water_litres_per_useful_kwh"),
        ("required_outcomes", "pump_filtration_treatment_kwh"),
        (
            "negative_controls",
            "POOL_VOLUME_COUNTED_AS_COOLING_WITHOUT_HEAT_PATH",
        ),
        (
            "negative_controls",
            "LATENT_HEAT_REPORTED_AS_USEFUL_ROOM_COOLING",
        ),
        (
            "negative_controls",
            "MISSING_WATER_METER_INTERPRETED_AS_ZERO_LOSS",
        ),
        (
            "negative_controls",
            "UNTREATED_GREYWATER_USED_IN_POOL_OR_AEROSOL",
        ),
        (
            "negative_controls",
            "WATER_RESERVE_FLOOR_OR_MUNICIPAL_RESTRICTION_IGNORED",
        ),
        ("safety_and_governance_gates", "water_quality_qualified"),
        (
            "safety_and_governance_gates",
            "drowning_and_child_safety_qualified",
        ),
        (
            "safety_and_governance_gates",
            "household_essential_water_reserve_protected",
        ),
        (
            "safety_and_governance_gates",
            "resident_authority_and_appeal_preserved",
        ),
    ] {
        let values = array_at(&manifest, field).cloned().unwrap_or_default();
        if !has_string(&values, required) {
            errors.push(format!("{field} is missing required entry {required}"));
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let negative_controls = array_at(&manifest, "negative_controls").map_or(0, Vec::len);
    let safety_gates = array_at(&manifest, "safety_and_governance_gates").map_or(0, Vec::len);
    Ok(json!({
        "audit_state": "STRUCTURAL_CONTRACT_PASS",
        "study_id": "HIGHVELD-PASSIVE-COOLING-WATER-001",
        "maturity": "DESIGN_HYPOTHESES_NOT_PHYSICALLY_EVALUATED",
        "archetype_count": archetypes.len(),
        "primary_no_water_control": "PRESENT",
        "comparison_mode_count": comparisons.len(),
        "negative_control_count": negative_controls,
        "safety_and_governance_gate_count": safety_gates,
        "execution_status": "NOT_EXECUTED",
        "schema_engine": "NOT_RUN; schema declaration and manifest semantics are checked by this Rust qualifier",
        "claim_ceiling": "No physical cooling, water saving, payback or resilience outcome is claimed"
    }))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("--help");
    match mode {
        "--self-test" | "--audit" => match build_report() {
            Ok(report) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
                );
            }
            Err(errors) => {
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "audit_state": "STRUCTURAL_CONTRACT_FAIL",
                        "errors": errors,
                        "execution_status": "NOT_EXECUTED"
                    }))
                    .unwrap_or_default()
                );
                process::exit(1);
            }
        },
        "--help" | "-h" => {
            println!("highveld-cooling-water-qualifier");
            println!(
                "  --self-test  validate the cooling study manifest and emit a structural audit"
            );
            println!("  --audit      emit the same audit");
        }
        _ => {
            eprintln!("unknown argument; use --help");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_study_has_no_water_control_and_all_primary_archetypes() {
        let report = build_report().expect("the committed cooling design contract must qualify");
        assert_eq!(report["archetype_count"].as_u64(), Some(7));
        assert_eq!(report["primary_no_water_control"].as_str(), Some("PRESENT"));
    }

    #[test]
    fn cooling_claims_remain_unexecuted_and_unvalidated() {
        let report = build_report().expect("the cooling manifest must qualify");
        assert_eq!(
            report["maturity"].as_str(),
            Some("DESIGN_HYPOTHESES_NOT_PHYSICALLY_EVALUATED")
        );
        assert_eq!(report["execution_status"].as_str(), Some("NOT_EXECUTED"));
        assert!(
            report["claim_ceiling"]
                .as_str()
                .is_some_and(|text| text.contains("No physical cooling"))
        );
    }

    #[test]
    fn water_negative_controls_and_safety_gates_are_preserved() {
        let report = build_report().expect("the cooling manifest must qualify");
        assert_eq!(report["negative_control_count"].as_u64(), Some(12));
        assert_eq!(report["safety_and_governance_gate_count"].as_u64(), Some(9));
    }
}
