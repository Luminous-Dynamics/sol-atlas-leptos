use serde_json::{Value, json};
use std::{env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_HEAT_RECOVERY_LEDGER_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_HEAT_RECOVERY_LEDGER_CASE_V0.1.schema.json"
);

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|number| number.is_finite())
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn audit(case: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("pool heat-recovery schema parse failed: {error}")])?;
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("pool heat-recovery schema must declare Draft 2020-12".to_owned());
    }
    for (key, expected) in [
        ("study_id", "HIGHVELD-POOL-HEAT-RECOVERY-LEDGER-001"),
        ("maturity", "SYNTHETIC_KNOWN_ANSWER"),
        ("execution_status", "NO_PHYSICAL_SIMULATION"),
    ] {
        if text_at(case, key) != Some(expected) {
            errors.push(format!("{key} must remain {expected}"));
        }
    }

    let refs = &case["evidence_refs"];
    for key in [
        "source_bundle_ref",
        "building_heat_meter_ref",
        "compressor_electric_meter_ref",
        "compressor_loss_method_ref",
        "condenser_heat_meter_ref",
        "pool_demand_headroom_ref",
        "isolated_heat_exchanger_ref",
        "pool_heat_transfer_meter_ref",
        "fallback_sink_meter_ref",
        "distribution_loss_method_ref",
        "uncertainty_method_ref",
        "independent_oracle_ref",
        "displaced_heater_meter_ref",
        "displaced_heater_counterfactual_ref",
    ] {
        if text_at(refs, key).is_none() {
            errors.push(format!("missing evidence reference {key}"));
        }
    }

    let circuit = &case["refrigeration_circuit"];
    let q_building = number_at(circuit, "building_side_heat_removed_kwh");
    let w_compressor = number_at(circuit, "compressor_electrical_work_kwh");
    let q_ambient_loss = number_at(circuit, "compressor_ambient_loss_kwh");
    let q_condenser = number_at(circuit, "measured_condenser_heat_output_kwh");
    let u95_circuit = number_at(circuit, "balance_u95_kwh");
    let (circuit_residual, circuit_expected) = match (
        q_building,
        w_compressor,
        q_ambient_loss,
        q_condenser,
        u95_circuit,
    ) {
        (Some(building), Some(work), Some(loss), Some(condenser), Some(u95))
            if building >= 0.0 && work >= 0.0 && loss >= 0.0 && condenser >= 0.0 && u95 >= 0.0 =>
        {
            let expected = building + work - loss;
            let residual = expected - condenser;
            if expected < 0.0 || !expected.is_finite() || !residual.is_finite() {
                errors.push("refrigeration circuit arithmetic is invalid/non-finite".to_owned());
                (None, None)
            } else {
                if residual.abs() > u95 {
                    errors.push(format!(
                        "condenser first-law residual {residual:.6} kWh exceeds u95 {u95:.6} kWh"
                    ));
                }
                (Some(residual), Some(expected))
            }
        }
        _ => {
            errors.push(
                "refrigeration circuit contains missing, negative or non-finite terms".to_owned(),
            );
            (None, None)
        }
    };

    let disposition = &case["heat_disposition"];
    let pool_demand = number_at(disposition, "pool_heating_demand_kwh");
    let pool_headroom = number_at(disposition, "pool_thermal_headroom_kwh");
    let pool_temp = number_at(disposition, "pool_temperature_c");
    let pool_max = number_at(disposition, "pool_maximum_temperature_c");
    let q_pool = number_at(disposition, "pool_heat_recovered_kwh");
    let fallback_capacity = number_at(disposition, "fallback_sink_capacity_kwh");
    let q_fallback = number_at(disposition, "fallback_heat_rejected_kwh");
    let q_distribution_loss = number_at(disposition, "distribution_losses_kwh");
    let u95_disposition = number_at(disposition, "disposition_balance_u95_kwh");
    let q_cond = q_condenser;

    if text_at(disposition, "isolated_heat_exchanger_state") != Some("QUALIFIED") {
        errors.push("pool heat recovery requires a qualified isolated heat exchanger".to_owned());
    }
    if let (Some(demand), Some(headroom), Some(temperature), Some(maximum), Some(recovered)) =
        (pool_demand, pool_headroom, pool_temp, pool_max, q_pool)
    {
        if demand < 0.0 || headroom < 0.0 || recovered < 0.0 {
            errors.push("pool demand/headroom/recovery must be non-negative".to_owned());
        }
        if recovered > demand {
            errors.push("pool recovery exceeds real pool heating demand".to_owned());
        }
        if recovered > headroom {
            errors.push("pool recovery exceeds remaining thermal headroom".to_owned());
        }
        if temperature >= maximum && recovered > 0.0 {
            errors.push("pool recovery violates the admitted maximum pool temperature".to_owned());
        }
        if demand == 0.0 && recovered > 0.0 {
            errors.push(
                "pool recovery cannot be enabled when no pool heating demand exists".to_owned(),
            );
        }
    } else {
        errors.push(
            "pool demand/headroom/temperature/recovery input is missing or non-finite".to_owned(),
        );
    }

    if let (Some(capacity), Some(rejected)) = (fallback_capacity, q_fallback) {
        if capacity < 0.0 || rejected < 0.0 {
            errors.push("fallback sink capacity/rejection must be non-negative".to_owned());
        }
        if rejected > capacity {
            errors.push("fallback heat rejection exceeds qualified sink capacity".to_owned());
        }
        if rejected > 0.0 && text_at(disposition, "fallback_sink_state") != Some("QUALIFIED") {
            errors
                .push("excess condenser heat is routed to an unqualified fallback sink".to_owned());
        }
    } else {
        errors.push("fallback sink capacity/rejection is missing or non-finite".to_owned());
    }

    let (disposition_residual, disposition_expected) = match (
        q_cond,
        q_pool,
        q_fallback,
        q_distribution_loss,
        u95_disposition,
    ) {
        (Some(condenser), Some(pool), Some(fallback), Some(loss), Some(u95))
            if condenser >= 0.0 && pool >= 0.0 && fallback >= 0.0 && loss >= 0.0 && u95 >= 0.0 =>
        {
            let accounted = pool + fallback + loss;
            let residual = condenser - accounted;
            if !accounted.is_finite() || !residual.is_finite() {
                errors.push("condenser disposition arithmetic is non-finite".to_owned());
                (None, None)
            } else {
                if residual.abs() > u95 {
                    errors.push(format!(
                        "condenser disposition residual {residual:.6} kWh exceeds u95 {u95:.6} kWh"
                    ));
                }
                (Some(residual), Some(accounted))
            }
        }
        _ => {
            errors.push("condenser disposition has missing, negative or non-finite terms".to_owned());
            (None, None)
        }
    };

    if text_at(&case["displaced_pool_heater"], "evidence_state") != Some("QUALIFIED")
        || number_at(
            &case["displaced_pool_heater"],
            "measured_displaced_heater_input_kwh",
        )
        .is_none_or(|value| value < 0.0)
        || text_at(refs, "displaced_heater_meter_ref").is_none()
        || text_at(refs, "displaced_heater_counterfactual_ref").is_none()
    {
        errors.push(
            "displaced pool-heater energy requires its own qualified meter and counterfactual"
                .to_owned(),
        );
    }

    let audit_state = if errors.is_empty() {
        "POOL_HEAT_RECOVERY_LEDGER_PASS"
    } else {
        "POOL_HEAT_RECOVERY_LEDGER_FAIL"
    };
    Ok(json!({
        "audit_state":audit_state,
        "study_id":"HIGHVELD-POOL-HEAT-RECOVERY-LEDGER-001",
        "case_id":text_at(case,"case_id").unwrap_or_default(),
        "building_side_heat_removed_kwh":q_building,
        "compressor_electrical_work_kwh":w_compressor,
        "compressor_ambient_loss_kwh":q_ambient_loss,
        "expected_condenser_heat_kwh":circuit_expected,
        "measured_condenser_heat_kwh":q_condenser,
        "condenser_first_law_residual_kwh":circuit_residual,
        "pool_heat_recovered_kwh":q_pool,
        "fallback_heat_rejected_kwh":q_fallback,
        "distribution_losses_kwh":q_distribution_loss,
        "accounted_condenser_disposition_kwh":disposition_expected,
        "condenser_disposition_residual_kwh":disposition_residual,
        "measured_displaced_pool_heater_input_kwh":case["displaced_pool_heater"]["measured_displaced_heater_input_kwh"],
        "cooling_service_credit_kwh":q_building,
        "pool_heat_credit_kwh":q_pool,
        "double_count_policy":"BUILDING_COOLING_POOL_HEAT_AND_DISPLACED_HEATER_INPUT_REPORTED_SEPARATELY",
        "violations":errors,
        "execution_status":"SYNTHETIC_CONSERVATION_AUDIT_ONLY",
        "claim_ceiling":"Supplied-value conservation arithmetic only; no heat-pump, pool or fallback-sink physical validation"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case: Value =
                serde_json::from_str(CASE).expect("committed pool heat-recovery case JSON");
            let report = audit(&case).expect("pool heat-recovery case is parseable");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
            if report["audit_state"] != "POOL_HEAT_RECOVERY_LEDGER_PASS" {
                process::exit(1);
            }
        }
        "--help" | "-h" => {
            println!("highveld-pool-heat-recovery-ledger");
            println!(
                "  --self-test  audit the synthetic refrigeration and condenser disposition balances"
            );
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

    fn sample() -> Value {
        serde_json::from_str(CASE).expect("case JSON")
    }

    #[test]
    fn synthetic_first_law_and_disposition_balances_close() {
        let report = audit(&sample()).expect("case is parseable");
        assert_eq!(report["audit_state"], "POOL_HEAT_RECOVERY_LEDGER_PASS");
        assert_eq!(report["expected_condenser_heat_kwh"], 12.8);
        assert_eq!(report["measured_condenser_heat_kwh"], 12.8);
        assert_eq!(report["condenser_first_law_residual_kwh"], 0.0);
        assert_eq!(report["condenser_disposition_residual_kwh"], 0.0);
        assert_eq!(report["cooling_service_credit_kwh"], 10.0);
        assert_eq!(report["pool_heat_credit_kwh"], 5.0);
    }

    #[test]
    fn condenser_first_law_residual_outside_uncertainty_fails() {
        let mut case = sample();
        case["refrigeration_circuit"]["measured_condenser_heat_output_kwh"] = json!(12.0);
        let report = audit(&case).unwrap();
        assert_eq!(report["audit_state"], "POOL_HEAT_RECOVERY_LEDGER_FAIL");
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .unwrap_or("")
                    .contains("condenser first-law residual"))
        );
    }

    #[test]
    fn disposition_residual_outside_uncertainty_fails() {
        let mut case = sample();
        case["heat_disposition"]["fallback_heat_rejected_kwh"] = json!(6.0);
        let report = audit(&case).unwrap();
        assert_eq!(report["audit_state"], "POOL_HEAT_RECOVERY_LEDGER_FAIL");
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .unwrap_or("")
                    .contains("condenser disposition residual"))
        );
    }

    #[test]
    fn pool_recovery_cannot_exceed_headroom_or_demand() {
        let mut case = sample();
        case["heat_disposition"]["pool_heat_recovered_kwh"] = json!(5.5);
        let report = audit(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .unwrap_or("")
                    .contains("exceeds remaining thermal headroom"))
        );
    }

    #[test]
    fn excess_heat_requires_qualified_fallback() {
        let mut case = sample();
        case["heat_disposition"]["fallback_sink_state"] = json!("UNKNOWN");
        let report = audit(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .unwrap_or("")
                    .contains("unqualified fallback sink"))
        );
    }

    #[test]
    fn pool_temperature_ceiling_blocks_recovery() {
        let mut case = sample();
        case["heat_disposition"]["pool_temperature_c"] = json!(29.0);
        let report = audit(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .unwrap_or("")
                    .contains("maximum pool temperature"))
        );
    }

    #[test]
    fn pool_recovery_is_disabled_when_there_is_no_heating_demand() {
        let mut case = sample();
        case["heat_disposition"]["pool_heating_demand_kwh"] = json!(0.0);
        case["heat_disposition"]["pool_heat_recovered_kwh"] = json!(0.0);
        case["heat_disposition"]["fallback_heat_rejected_kwh"] = json!(12.5);
        let report = audit(&case).unwrap();
        assert_eq!(report["audit_state"], "POOL_HEAT_RECOVERY_LEDGER_PASS");
        assert_eq!(report["pool_heat_recovered_kwh"], 0.0);
    }

    #[test]
    fn undersized_fallback_sink_rejects_the_recovery_case() {
        let mut case = sample();
        case["heat_disposition"]["fallback_sink_capacity_kwh"] = json!(7.0);
        let report = audit(&case).unwrap();
        assert_eq!(report["audit_state"], "POOL_HEAT_RECOVERY_LEDGER_FAIL");
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == "fallback heat rejection exceeds qualified sink capacity")
        );
    }

    #[test]
    fn missing_heater_meter_reference_blocks_displacement_claim() {
        let mut case = sample();
        case["evidence_refs"]["displaced_heater_meter_ref"] = Value::Null;
        let report = audit(&case).unwrap();
        assert!(
            report["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .unwrap_or("")
                    .contains("displaced pool-heater energy"))
        );
    }
}
