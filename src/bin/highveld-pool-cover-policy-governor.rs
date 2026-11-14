use serde_json::{Value, json};
use std::{env, process};

const CASE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_COVER_POLICY_CASE_V0.1.json"
);
const SCHEMA: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/POOL_COVER_POLICY_CASE_V0.1.schema.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).filter(|text| !text.trim().is_empty())
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64).filter(|number| number.is_finite())
}

fn evaluate(case: &Value) -> Result<Value, Vec<String>> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| vec![format!("pool cover policy schema parse failed: {error}")])?;
    let mut errors = Vec::new();
    if text_at(&schema, "$schema") != Some("https://json-schema.org/draft/2020-12/schema") {
        errors.push("pool cover policy schema must declare Draft 2020-12".to_owned());
    }
    if text_at(case,"study_id") != Some("HIGHVELD-POOL-COVER-POLICY-001")
        || text_at(case,"maturity") != Some("SYNTHETIC_KNOWN_ANSWER")
        || text_at(case,"execution_status") != Some("NO_PHYSICAL_COVER_ACTUATION")
    {
        errors.push("pool cover policy identity or claim ceiling is invalid".to_owned());
    }
    let cases = case.get("cases").and_then(Value::as_array)
        .ok_or_else(||vec!["cases must be an array".to_owned()])?;
    if cases.len()!=8 {
        errors.push("exactly eight synthetic known-answer cases are required".to_owned());
    }
    if !errors.is_empty(){return Err(errors);}

    let mut results=Vec::new();
    for item in cases {
        let id=text_at(item,"case_id").unwrap_or("<missing-case-id>");
        let mut reasons=Vec::new();
        let mut mode="HOLD_FOR_HUMAN_REVIEW";
        let objective=text_at(item,"objective").unwrap_or("UNKNOWN");
        let time=text_at(item,"time_window").unwrap_or("UNKNOWN");

        for key in ["weather","pool_state","sky_view","rights","quality","cover","uncertainty","owner_limits"] {
            if text_at(&item["evidence_refs"],key).is_none() {
                reasons.push(format!("MISSING_EVIDENCE_REF_{key}"));
            }
        }
        for (key,expected) in [
            ("cover_mechanism_state","QUALIFIED"),
            ("manual_override_state","QUALIFIED"),
            ("water_quality_state","QUALIFIED"),
        ] {
            if text_at(item,key)!=Some(expected) {
                reasons.push(format!("{key}_NOT_QUALIFIED"));
            }
        }

        let pool_temp=number_at(item,"pool_temperature_c");
        let regen_target=number_at(item,"pool_regeneration_target_c");
        let max_temp=number_at(item,"pool_owner_max_temperature_c");
        let demand=number_at(item,"pool_heating_demand_kwh");
        let headroom=number_at(item,"pool_usable_headroom_kwh");
        let rejection=number_at(item,"net_heat_rejection_estimate_kw");
        let rejection_u95=number_at(item,"net_heat_rejection_u95_kw");
        let evaporation=number_at(item,"expected_evaporation_l");
        let evaporation_u95=number_at(item,"evaporation_u95_l");
        let budget=number_at(item,"discretionary_water_budget_l");
        let solar=number_at(item,"solar_gain_estimate_kw");
        let solar_u95=number_at(item,"solar_gain_u95_kw");
        let required_numbers=[
            ("pool_temperature_c",pool_temp),("pool_regeneration_target_c",regen_target),
            ("pool_owner_max_temperature_c",max_temp),("pool_heating_demand_kwh",demand),
            ("pool_usable_headroom_kwh",headroom),("net_heat_rejection_estimate_kw",rejection),
            ("net_heat_rejection_u95_kw",rejection_u95),("expected_evaporation_l",evaporation),
            ("evaporation_u95_l",evaporation_u95),("discretionary_water_budget_l",budget),
            ("solar_gain_estimate_kw",solar),("solar_gain_u95_kw",solar_u95)
        ];
        for (key,value) in required_numbers {
            if value.is_none(){reasons.push(format!("MISSING_OR_NONFINITE_{key}"));}
        }
        if !reasons.is_empty(){
            results.push(json!({"case_id":id,"recommended_mode":"HOLD_FOR_HUMAN_REVIEW","reasons":reasons,
                "actuation":"NONE","state":"INPUT_NOT_QUALIFIED"}));
            continue;
        }
        let pool_temp=pool_temp.unwrap();
        let regen_target=regen_target.unwrap();
        let max_temp=max_temp.unwrap();
        let demand=demand.unwrap();
        let headroom=headroom.unwrap();
        let rejection=rejection.unwrap();
        let rejection_u95=rejection_u95.unwrap();
        let evaporation=evaporation.unwrap();
        let evaporation_u95=evaporation_u95.unwrap();
        let budget=budget.unwrap();
        let solar=solar.unwrap();
        let solar_u95=solar_u95.unwrap();

        if pool_temp>max_temp {
            reasons.push("OWNER_TEMPERATURE_LIMIT_BREACH".to_owned());
        }
        if text_at(item,"water_rights_state")==Some("UNKNOWN")
            || text_at(item,"water_rights_state").is_none()
        {
            reasons.push("WATER_RIGHTS_UNKNOWN".to_owned());
        }
        if text_at(item,"water_quality_state")!=Some("QUALIFIED") {
            reasons.push("WATER_QUALITY_NOT_QUALIFIED".to_owned());
        }
        if !reasons.is_empty() {
            results.push(json!({"case_id":id,"recommended_mode":"HOLD_FOR_HUMAN_REVIEW","reasons":reasons,
                "actuation":"NONE","state":"HUMAN_REVIEW_REQUIRED"}));
            continue;
        }

        if objective=="POOL_HEAT_RECOVERY" {
            if time=="UNKNOWN" || demand<0.0 || headroom<0.0 {
                reasons.push("HEAT_RECOVERY_INPUT_INVALID".to_owned());
            } else if text_at(item,"heat_recovery_hx_state")!="QUALIFIED"
                || text_at(&item["evidence_refs"],"heat_exchanger").is_none()
            {
                reasons.push("ISOLATED_HEAT_EXCHANGER_NOT_QUALIFIED".to_owned());
            } else if demand>0.0 && headroom>0.0 {
                mode="COVERED_HEAT_RECOVERY_RETENTION";
            } else {
                mode="COVERED_HOLD";
                reasons.push("NO_ACTIVE_POOL_HEATING_DEMAND_OR_HEADROOM".to_owned());
            }
        } else if objective=="DAYTIME_SOLAR_PROTECTION" && time=="DAY" {
            if text_at(item,"shade_mechanism_state")!="QUALIFIED" {
                reasons.push("SHADE_MECHANISM_NOT_QUALIFIED".to_owned());
            } else if solar-solar_u95>0.0 {
                mode="SHADE_ONLY_SOLAR_PROTECTION";
            } else {
                mode="COVERED_HOLD";
                reasons.push("SOLAR_GAIN_LOWER_BOUND_NOT_POSITIVE".to_owned());
            }
        } else if objective=="POOL_BUFFER_REGENERATION" && time=="NIGHT" {
            if text_at(item,"sky_view_state")!="QUALIFIED" {
                reasons.push("SKY_VIEW_NOT_QUALIFIED".to_owned());
            } else if text_at(item,"water_rights_state")=="PROHIBITED" || budget<=0.0 {
                mode="COVERED_WATER_CONSERVATION";
                reasons.push("DISCRETIONARY_EVAPORATION_NOT_PERMITTED".to_owned());
            } else if pool_temp<=regen_target {
                mode="COVERED_HOLD";
                reasons.push("REGENERATION_NOT_NEEDED_AT_DECLARED_POOL_TEMPERATURE".to_owned());
            } else if rejection-rejection_u95<=0.0 {
                mode="COVERED_HOLD";
                reasons.push("NET_HEAT_REJECTION_LOWER_BOUND_NOT_POSITIVE".to_owned());
            } else if evaporation+evaporation_u95>budget {
                mode="COVERED_WATER_CONSERVATION";
                reasons.push("EVAPORATION_UPPER_BOUND_EXCEEDS_DISCRETIONARY_BUDGET".to_owned());
            } else {
                mode="OPEN_TO_SKY_REGENERATION";
            }
        } else {
            mode="COVERED_HOLD";
            reasons.push("NO_MATCHING_QUALIFIED_POLICY_WINDOW".to_owned());
        }

        if reasons.iter().any(|reason| reason=="OWNER_TEMPERATURE_LIMIT_BREACH") {
            mode="HOLD_FOR_HUMAN_REVIEW";
        }
        results.push(json!({
            "case_id":id,
            "recommended_mode":mode,
            "reasons":reasons,
            "net_heat_rejection_lower_bound_kw":rejection-rejection_u95,
            "evaporation_upper_bound_l":evaporation+evaporation_u95,
            "discretionary_water_budget_l":budget,
            "actuation":"NONE",
            "state":if mode=="HOLD_FOR_HUMAN_REVIEW" {"HUMAN_REVIEW_REQUIRED"} else {"POLICY_ELIGIBILITY_ONLY"}
        }));
    }

    Ok(json!({
        "audit_state":"POOL_COVER_POLICY_EVALUATED",
        "study_id":"HIGHVELD-POOL-COVER-POLICY-001",
        "case_count":results.len(),
        "results":results,
        "execution_status":"NO_PHYSICAL_COVER_ACTUATION",
        "claim_ceiling":"Synthetic policy eligibility only; no forecast accuracy, pool performance, water saving or pedestrian cooling is validated"
    }))
}

fn main() {
    match env::args().nth(1).as_deref().unwrap_or("--help") {
        "--self-test" => {
            let case:Value=serde_json::from_str(CASE).expect("committed pool cover policy case JSON");
            let report=evaluate(&case).expect("pool cover policy input structure");
            println!("{}",serde_json::to_string_pretty(&report).unwrap_or_default());
            if report["audit_state"]!="POOL_COVER_POLICY_EVALUATED"{process::exit(1);}
        }
        "--help" | "-h" => {
            println!("highveld-pool-cover-policy-governor");
            println!("  --self-test  evaluate the synthetic non-actuating cover policy cases");
        }
        _ => {eprintln!("invalid argument; use --help");process::exit(2);}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample()->Value{serde_json::from_str(CASE).expect("case JSON")}

    fn result<'a>(report:&'a Value,id:&str)->&'a Value{
        report["results"].as_array().unwrap().iter()
            .find(|item|item["case_id"]==id).unwrap()
    }

    #[test]
    fn clear_night_exposes_only_when_rejection_and_water_bounds_qualify(){
        let report=evaluate(&sample()).unwrap();
        assert_eq!(result(&report,"NIGHT_REGENERATION_QUALIFIED")["recommended_mode"],"OPEN_TO_SKY_REGENERATION");
        assert_eq!(result(&report,"NIGHT_REGENERATION_QUALIFIED")["actuation"],"NONE");
    }

    #[test]
    fn nonpositive_rejection_or_excess_evaporation_keeps_cover_closed(){
        let report=evaluate(&sample()).unwrap();
        assert_eq!(result(&report,"CLOUDY_NIGHT_NO_ROBUST_REGENERATION")["recommended_mode"],"COVERED_HOLD");
        assert_eq!(result(&report,"EVAPORATION_EXCEEDS_DISCRETIONARY_BUDGET")["recommended_mode"],"COVERED_WATER_CONSERVATION");
    }

    #[test]
    fn heat_recovery_retains_cover_and_day_solar_uses_shade_only(){
        let report=evaluate(&sample()).unwrap();
        assert_eq!(result(&report,"HEAT_RECOVERY_RETAINS_COVER")["recommended_mode"],"COVERED_HEAT_RECOVERY_RETENTION");
        assert_eq!(result(&report,"DAYTIME_SOLAR_PROTECTION")["recommended_mode"],"SHADE_ONLY_SOLAR_PROTECTION");
    }

    #[test]
    fn unknown_rights_and_temperature_limit_breach_require_human_review(){
        let report=evaluate(&sample()).unwrap();
        assert_eq!(result(&report,"UNKNOWN_WATER_RIGHTS_REQUIRES_REVIEW")["recommended_mode"],"HOLD_FOR_HUMAN_REVIEW");
        assert_eq!(result(&report,"POOL_OVER_OWNER_TEMPERATURE_LIMIT")["recommended_mode"],"HOLD_FOR_HUMAN_REVIEW");
    }

    #[test]
    fn cold_pool_does_not_regenerate_below_owner_target(){
        let report=evaluate(&sample()).unwrap();
        assert_eq!(result(&report,"REGENERATION_NOT_NEEDED_AT_TARGET")["recommended_mode"],"COVERED_HOLD");
    }
}
