use serde_json::{Value, json};
use std::{env, process};

const REGISTER: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/JOHANNESBURG_WEATHER_SOURCE_REGISTER_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/JOHANNESBURG_WEATHER_SOURCE_REGISTER_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn has_id(items: &[Value], id: &str) -> bool {
    items.iter().any(|item| text_at(item, "id") == Some(id))
}

fn audit() -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let register: Value = serde_json::from_str(REGISTER)
        .map_err(|e| vec![format!("weather register JSON parse failed: {e}")])?;
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|e| vec![format!("weather schema JSON parse failed: {e}")])?;

    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("schema must declare JSON Schema Draft 2020-12".to_owned());
    }

    for (field, expected) in [
        ("format_version", "0.1"),
        ("study_id", "HIGHVELD-JOHANNESBURG-WEATHER-PACKAGE-001"),
        ("parent_study", "HIGHVELD-PASSIVE-COOLING-WATER-001"),
        ("climate_context", "JOHANNESBURG_HIGHVELD"),
        ("maturity", "WEATHER_SOURCE_REGISTER_ONLY_NOT_ADMITTED"),
        ("acquisition_status", "NOT_ACQUIRED"),
        ("site_definition_status", "OWNER_SITE_NOT_SELECTED"),
    ] {
        if text_at(&register, field) != Some(expected) {
            errors.push(format!("{field} must remain {expected}"));
        }
    }

    let sources = register["candidate_sources"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if sources.len() < 10 {
        errors.push("at least ten candidate source records are required".to_owned());
    }
    for id in [
        "SAWS_STATION_ARCHIVE",
        "ARC_AGROMET_STATIONS",
        "DWS_HYDROLOGICAL_STATIONS",
        "WRO_SAAACA_1950_1999",
        "WRO_DATASET_CATALOGUE",
        "NOAA_GHCN_DAILY_ISD",
        "CHIRPS_PRECIPITATION",
        "NASA_POWER_MERRA2_REANALYSIS",
        "ERA5_REANALYSIS",
        "EPW_TMY_DERIVATIVE",
    ] {
        if !has_id(&sources, id) {
            errors.push(format!("candidate source {id} is missing"));
        }
    }

    let required = register["required_temporal_packages"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for id in [
        "PINNED_TMY_EPW_WITH_ORIGIN_METADATA",
        "RECENT_ACTUAL_METEOROLOGICAL_YEAR",
        "PREREGISTERED_HISTORIC_HOT_TAIL",
        "MULTIDAY_HEATWAVE_AND_HOT_NIGHT_EVENTS",
        "FUTURE_CLIMATE_SERIES_WITH_SCENARIO_AND_HORIZON",
        "LOCAL_COURTYARD_MEASUREMENT_SERIES",
    ] {
        if !required.iter().any(|v| v.as_str() == Some(id)) {
            errors.push(format!("temporal package {id} is missing"));
        }
    }

    let variables = register["required_variables"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for id in [
        "DRY_BULB_TEMPERATURE",
        "RELATIVE_HUMIDITY_OR_DEW_POINT",
        "WIND_SPEED_AND_DIRECTION_WITH_EXPOSURE",
        "GLOBAL_SOLAR_IRRADIANCE",
        "CLOUD_OR_LONGWAVE_DOWNWELLING_FOR_RADIATIVE_SINK",
        "SITE_SKY_VIEW_FACTOR_AND_HORIZON",
    ] {
        if !variables.iter().any(|v| v.as_str() == Some(id)) {
            errors.push(format!("required variable {id} is missing"));
        }
    }

    let gates = &register["hard_gates"];
    for key in [
        "unlicensed_use",
        "grid_data_relabelled_as_site_measurements",
        "hidden_critical_variable_imputation",
        "future_weather_in_causal_controller",
        "untraceable_source_transforms",
        "required_claims_without_representative_weather",
    ] {
        if gates.get(key).and_then(Value::as_u64) != Some(0) {
            errors.push(format!("hard gate {key} must equal zero"));
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(json!({
        "audit_state": "STRUCTURAL_CONTRACT_PASS",
        "study_id": "HIGHVELD-JOHANNESBURG-WEATHER-PACKAGE-001",
        "candidate_source_count": sources.len(),
        "required_temporal_package_count": required.len(),
        "required_variable_count": variables.len(),
        "acquisition_status": "NOT_ACQUIRED",
        "site_definition_status": "OWNER_SITE_NOT_SELECTED",
        "instrument_or_station_admission": "NOT_ADMITTED",
        "execution_status": "NOT_EXECUTED",
        "schema_engine": "NOT_RUN; schema declaration and manifest semantics are checked by this Rust qualifier",
        "claim_ceiling": "No weather observations or station representativeness have been admitted"
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
                    serde_json::to_string_pretty(&json!({"errors": errors})).unwrap_or_default()
                );
                process::exit(1);
            }
        },
        "--help" | "-h" => {
            println!("highveld-weather-source-qualifier");
            println!("  --self-test  audit Johannesburg weather source register invariants");
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
    fn measured_paid_and_free_context_sources_are_kept_distinct() {
        let report = audit().expect("weather source register must qualify");
        assert_eq!(report["candidate_source_count"].as_u64(), Some(10));
        assert_eq!(report["acquisition_status"].as_str(), Some("NOT_ACQUIRED"));
        let register: Value = serde_json::from_str(REGISTER).unwrap();
        let sources = register["candidate_sources"].as_array().unwrap();
        let saw_saws = sources
            .iter()
            .find(|s| s["id"] == "SAWS_STATION_ARCHIVE")
            .unwrap();
        assert_eq!(saw_saws["not_for_resale"].as_bool(), Some(true));
        let dws = sources
            .iter()
            .find(|s| s["id"] == "DWS_HYDROLOGICAL_STATIONS")
            .unwrap();
        assert_eq!(dws["role"].as_str(), Some("HYDROLOGICAL_CONTEXT_ONLY"));
    }

    #[test]
    fn no_site_station_or_weather_data_is_fabricated() {
        let report = audit().expect("register must qualify");
        assert_eq!(
            report["site_definition_status"].as_str(),
            Some("OWNER_SITE_NOT_SELECTED")
        );
        assert_eq!(
            report["instrument_or_station_admission"].as_str(),
            Some("NOT_ADMITTED")
        );
        assert_eq!(report["execution_status"].as_str(), Some("NOT_EXECUTED"));
    }

    #[test]
    fn radiative_and_evaporative_variables_are_required() {
        let register: Value = serde_json::from_str(REGISTER).unwrap();
        let vars = register["required_variables"].as_array().unwrap();
        for needed in [
            "CLOUD_OR_LONGWAVE_DOWNWELLING_FOR_RADIATIVE_SINK",
            "WIND_SPEED_AND_DIRECTION_WITH_EXPOSURE",
            "RELATIVE_HUMIDITY_OR_DEW_POINT",
        ] {
            assert!(vars.iter().any(|v| v == needed));
        }
    }
}
