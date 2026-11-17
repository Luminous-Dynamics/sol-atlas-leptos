// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-freedom/HOME_FREEDOM_CLIMATE_EVIDENCE_SYNTHETIC_V0.1.json",
);
const SCHEMA: &str = include_str!(
    "../../docs/research/home-freedom/HOME_FREEDOM_CLIMATE_EVIDENCE_V0.1.schema.json",
);

fn object<'a>(
    value: &'a Value,
    context: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} must be an object"))
}

fn array<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.{key} must be an array"))
}

fn text<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| format!("{context}.{key} must be a non-empty string"))
}

fn only_keys(value: &Value, allowed: &[&str], context: &str) -> Result<(), String> {
    for key in object(value, context)?.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{context} contains unknown field {key}"));
        }
    }
    Ok(())
}

fn require_false(value: &Value, key: &str) -> Result<(), String> {
    if value.get(key).and_then(Value::as_bool) != Some(false) {
        return Err(format!("claim_ceiling.{key} must remain false"));
    }
    Ok(())
}

fn unique_insert(set: &mut BTreeSet<String>, id: &str, context: &str) -> Result<(), String> {
    if !set.insert(id.to_owned()) {
        return Err(format!("duplicate identity {id} in {context}"));
    }
    Ok(())
}

fn check_schema_contract() -> Result<(), String> {
    let schema: Value = serde_json::from_str(SCHEMA)
        .map_err(|error| format!("schema JSON parse failed: {error}"))?;
    if text(&schema, "$schema", "schema")? != "https://json-schema.org/draft/2020-12/schema" {
        return Err("schema must declare Draft 2020-12".to_owned());
    }
    let required = array(&schema, "required", "schema")?;
    for field in ["climate_members", "scenario_compositions", "adversarial_cases", "claim_ceiling"] {
        if !required.iter().any(|item| item.as_str() == Some(field)) {
            return Err(format!("schema root required list omits {field}"));
        }
    }
    for definition in ["climateMember", "scenarioComposition", "adversarialCase", "claimCeiling"] {
        if schema["$defs"].get(definition).is_none() {
            return Err(format!("schema definition {definition} is missing"));
        }
    }
    Ok(())
}

fn check_class_metadata(member: &Value, class: &str) -> Result<(), String> {
    let metadata = member
        .get("class_metadata")
        .ok_or_else(|| "member.class_metadata is required".to_owned())?;
    let required: &[&str] = match class {
        "TYPICAL_METEOROLOGICAL_YEAR" => &[
            "selection_period_start",
            "selection_period_end",
            "selection_method_ref",
            "supported_use",
        ],
        "STATION_OBSERVATION" => &[
            "station_id",
            "station_history_ref",
            "relocation_review_state",
            "instrument_change_review_state",
            "qc_profile_ref",
        ],
        "REANALYSIS" => &[
            "product_family",
            "grid_selection_method",
            "nearest_grid_point_used",
            "assimilation_relationship",
        ],
        "OBSERVED_EXTREME_EPISODE" => &[
            "parent_member_id",
            "episode_id",
            "event_class",
            "selection_rule_ref",
            "event_start_utc",
            "event_end_utc",
        ],
        "FUTURE_PROJECTION" => &[
            "climate_model",
            "ensemble_member_id",
            "scenario_or_pathway",
            "period_start_year",
            "period_end_year",
            "downscaling_method_ref",
            "bias_correction_method_ref",
        ],
        "ON_SITE_MICROCLIMATE" => &[
            "instrument_passport_ref",
            "sample_session_ref",
            "stream_timebase_ref",
            "installation_scope_ref",
            "location_owner_ref",
        ],
        "SYNTHETIC_TRACE" => &["fixture_id", "fixture_case_id"],
        _ => return Err(format!("unsupported evidence class {class}")),
    };
    for key in required {
        if metadata.get(*key).is_none() {
            return Err(format!("{class} metadata omits {key}"));
        }
    }
    only_keys(metadata, required, "class_metadata")
}

fn evaluate(root: &Value) -> Result<Value, String> {
    only_keys(
        root,
        &[
            "schema_version",
            "benchmark_id",
            "maturity",
            "execution_status",
            "as_of_utc",
            "climate_members",
            "scenario_compositions",
            "uncertainty_policy",
            "claim_ceiling",
            "adversarial_cases",
        ],
        "root",
    )?;
    if text(root, "schema_version", "root")? != "0.1.0"
        || text(root, "benchmark_id", "root")? != "HOME-FREEDOM-CLIMATE-EVIDENCE-001"
        || text(root, "maturity", "root")? != "SYNTHETIC_ONLY"
        || text(root, "execution_status", "root")? != "NOT_INGESTED_NOT_SIMULATED"
    {
        return Err("benchmark identity or synthetic claim ceiling changed".to_owned());
    }

    check_schema_contract()?;
    let claims = root
        .get("claim_ceiling")
        .ok_or_else(|| "root.claim_ceiling is required".to_owned())?;
    only_keys(
        claims,
        &[
            "weather_acquired",
            "real_station_admitted",
            "reanalysis_admitted_for_site",
            "physical_model_run",
            "sensor_measurement_admitted",
            "site_microclimate_claimed",
            "building_performance_claimed",
            "sota_superiority_claimed",
        ],
        "claim_ceiling",
    )?;
    for key in [
        "weather_acquired",
        "real_station_admitted",
        "reanalysis_admitted_for_site",
        "physical_model_run",
        "sensor_measurement_admitted",
        "site_microclimate_claimed",
        "building_performance_claimed",
        "sota_superiority_claimed",
    ] {
        require_false(claims, key)?;
    }

    let member_rows = array(root, "climate_members", "root")?;
    if member_rows.len() < 7 {
        return Err("expected at least seven typed climate evidence examples".to_owned());
    }
    let mut member_ids = BTreeSet::new();
    let mut member_classes = BTreeMap::<String, String>::new();
    for member in member_rows {
        only_keys(
            member,
            &[
                "member_id",
                "evidence_class",
                "admission_state",
                "source_identity",
                "spatial_support",
                "temporal_support",
                "quality",
                "uncertainty_components",
                "rights",
                "purpose_ceiling",
                "class_metadata",
            ],
            "climate_member",
        )?;
        let id = text(member, "member_id", "climate_member")?;
        unique_insert(&mut member_ids, id, "climate members")?;
        let class = text(member, "evidence_class", "climate_member")?;
        member_classes.insert(id.to_owned(), class.to_owned());
        if text(member, "admission_state", "climate_member")? != "SYNTHETIC_ONLY" {
            return Err(format!("synthetic fixture member {id} is not SYNTHETIC_ONLY"));
        }
        let source = member
            .get("source_identity")
            .ok_or_else(|| format!("member {id} has no source_identity"))?;
        if source["content_sha256"] != Value::Null
            || source["retrieved_at_utc"] != Value::Null
            || source["rights_record_ref"] != Value::Null
        {
            return Err(format!("synthetic member {id} contains a real-source receipt"));
        }
        let rights = member
            .get("rights")
            .ok_or_else(|| format!("member {id} has no rights record"))?;
        if text(rights, "state", "rights")? != "SYNTHETIC_ONLY" {
            return Err(format!("synthetic fixture member {id} claims non-synthetic rights"));
        }
        let purpose = array(member, "purpose_ceiling", "climate_member")?;
        if !purpose.iter().any(|item| item.as_str() == Some("SOFTWARE_CONTRACT_TEST_ONLY")) {
            return Err(format!("synthetic fixture member {id} lacks the software-only purpose ceiling"));
        }
        let quality = member
            .get("quality")
            .ok_or_else(|| format!("member {id} has no quality record"))?;
        if text(quality, "qc_state", "quality")? != "SYNTHETIC_ONLY" {
            return Err(format!("synthetic fixture member {id} has non-synthetic QC state"));
        }
        check_class_metadata(member, class)?;
        for uncertainty in array(member, "uncertainty_components", "climate_member")? {
            let state = text(uncertainty, "state", "uncertainty_component")?;
            if state == "QUANTIFIED" {
                let lower = uncertainty["lower"].as_f64();
                let upper = uncertainty["upper"].as_f64();
                if lower.is_none() || upper.is_none() || lower > upper {
                    return Err(format!("member {id} has invalid quantified uncertainty"));
                }
                text(uncertainty, "unit", "uncertainty_component")?;
                text(uncertainty, "method_ref", "uncertainty_component")?;
            }
        }
    }

    let scenarios = array(root, "scenario_compositions", "root")?;
    if scenarios.len() < 5 {
        return Err("expected at least five scenario compositions".to_owned());
    }
    let mut scenario_ids = BTreeSet::new();
    let mut trace_ids = BTreeSet::new();
    for scenario in scenarios {
        only_keys(
            scenario,
            &[
                "scenario_id",
                "mandatory",
                "purpose",
                "climate_member_refs",
                "service_event_refs",
                "composition_mode",
                "aggregation_method_ref",
                "decision_trace_id",
            ],
            "scenario_composition",
        )?;
        let scenario_id = text(scenario, "scenario_id", "scenario_composition")?;
        unique_insert(&mut scenario_ids, scenario_id, "scenario compositions")?;
        let trace_id = text(scenario, "decision_trace_id", "scenario_composition")?;
        unique_insert(&mut trace_ids, trace_id, "decision trace IDs")?;
        let refs = array(scenario, "climate_member_refs", "scenario_composition")?;
        if refs.is_empty() {
            return Err(format!("scenario {scenario_id} has no climate members"));
        }
        let mut scenario_classes = Vec::new();
        let mut seen_refs = BTreeSet::new();
        for member_ref in refs {
            let member_id = member_ref
                .as_str()
                .ok_or_else(|| format!("scenario {scenario_id} has a non-string member reference"))?;
            unique_insert(&mut seen_refs, member_id, "scenario member references")?;
            let class = member_classes
                .get(member_id)
                .ok_or_else(|| format!("scenario {scenario_id} refers to unknown member {member_id}"))?;
            scenario_classes.push(class.as_str());
        }
        let purpose = text(scenario, "purpose", "scenario_composition")?;
        if purpose == "TYPICAL_YEAR_SCREENING"
            && scenario_classes.iter().any(|class| *class != "TYPICAL_METEOROLOGICAL_YEAR")
        {
            return Err(format!("scenario {scenario_id} mixes non-TMY evidence into TMY screening"));
        }
        if purpose == "HISTORICAL_EPISODE_REPLAY"
            && scenario_classes.iter().any(|class| {
                !["STATION_OBSERVATION", "REANALYSIS", "OBSERVED_EXTREME_EPISODE"].contains(class)
            })
        {
            return Err(format!("scenario {scenario_id} uses an invalid class for historical replay"));
        }
        if purpose == "FUTURE_SCENARIO_EXPLORATION"
            && scenario_classes.iter().any(|class| *class != "FUTURE_PROJECTION")
        {
            return Err(format!("scenario {scenario_id} mixes non-projection evidence into future exploration"));
        }

        let mode = text(scenario, "composition_mode", "scenario_composition")?;
        let aggregation_method = &scenario["aggregation_method_ref"];
        match mode {
            "SINGLE_MEMBER" if refs.len() != 1 => {
                return Err(format!("scenario {scenario_id} is SINGLE_MEMBER but has multiple members"));
            }
            "PARALLEL_NOT_POOLED" if refs.len() < 2 => {
                return Err(format!("scenario {scenario_id} is PARALLEL_NOT_POOLED but has fewer than two members"));
            }
            "APPROVED_STATISTICAL_AGGREGATION"
                if aggregation_method.as_str().is_none_or(str::is_empty) =>
            {
                return Err(format!("scenario {scenario_id} aggregates without a method reference"));
            }
            "SINGLE_MEMBER" | "PARALLEL_NOT_POOLED" | "MEMBERWISE_STRESS_SET"
                if !aggregation_method.is_null() =>
            {
                return Err(format!("scenario {scenario_id} supplies an aggregation method for a non-aggregated mode"));
            }
            "SINGLE_MEMBER" | "PARALLEL_NOT_POOLED" | "MEMBERWISE_STRESS_SET"
            | "APPROVED_STATISTICAL_AGGREGATION" => {}
            _ => return Err(format!("scenario {scenario_id} has unknown composition mode {mode}")),
        }

        let events = array(scenario, "service_event_refs", "scenario_composition")?;
        if purpose == "COMPOUND_RESILIENCE" && events.is_empty() {
            return Err(format!("compound scenario {scenario_id} has no service event"));
        }
        for event in events {
            only_keys(
                event,
                &["owner", "event_ref", "event_class", "timebase_ref", "alignment_state"],
                "service_event_ref",
            )?;
            text(event, "owner", "service_event_ref")?;
            text(event, "event_ref", "service_event_ref")?;
            text(event, "event_class", "service_event_ref")?;
            let alignment = text(event, "alignment_state", "service_event_ref")?;
            if alignment == "QUALIFIED_ALIGNMENT" {
                return Err("synthetic fixture cannot claim a qualified real-world timebase alignment".to_owned());
            }
        }
    }

    let cases = array(root, "adversarial_cases", "root")?;
    if cases.len() != 25 {
        return Err(format!("expected exactly 25 adversarial cases, found {}", cases.len()));
    }
    let mut case_ids = BTreeSet::new();
    for case in cases {
        only_keys(case, &["case_id", "attack", "expected_result"], "adversarial_case")?;
        unique_insert(&mut case_ids, text(case, "case_id", "adversarial_case")?, "adversarial cases")?;
        text(case, "attack", "adversarial_case")?;
        let expected = text(case, "expected_result", "adversarial_case")?;
        if ![
            "REJECT_AS_CLASS_MISMATCH",
            "BLOCK_FROM_ADMISSION",
            "REQUIRE_OWNER_REVIEW",
            "KEEP_MEMBERS_SEPARATE",
            "UNALIGNED_NOT_ADMITTED",
            "INVALIDATE_DEPENDENT_RESULT",
        ]
        .contains(&expected)
        {
            return Err(format!("adversarial case has unsupported expected result {expected}"));
        }
    }

    Ok(json!({
        "status": "SYNTHETIC_CONTRACT_PASS",
        "climate_member_count": member_rows.len(),
        "scenario_count": scenarios.len(),
        "adversarial_case_count": cases.len(),
        "claim_ceiling_all_false": true,
        "real_evidence_admitted": false
    }))
}

fn run_self_tests() -> Result<(), String> {
    let fixture: Value = serde_json::from_str(FIXTURE)
        .map_err(|error| format!("fixture JSON parse failed: {error}"))?;
    evaluate(&fixture)?;

    let mut tmy_as_extreme = fixture.clone();
    tmy_as_extreme["scenario_compositions"][0]["purpose"] = json!("HISTORICAL_EPISODE_REPLAY");
    if evaluate(&tmy_as_extreme).is_ok() {
        return Err("TMY-as-extreme negative control was accepted".to_owned());
    }

    let mut promoted = fixture.clone();
    promoted["climate_members"][0]["admission_state"] = json!("ADMITTED_FOR_MODEL_INPUT");
    if evaluate(&promoted).is_ok() {
        return Err("synthetic member promotion negative control was accepted".to_owned());
    }

    let mut unknown_ref = fixture.clone();
    unknown_ref["scenario_compositions"][0]["climate_member_refs"][0] = json!("MISSING_MEMBER");
    if evaluate(&unknown_ref).is_ok() {
        return Err("unresolved member-reference negative control was accepted".to_owned());
    }

    let mut pooled_without_method = fixture.clone();
    pooled_without_method["scenario_compositions"][1]["composition_mode"] =
        json!("APPROVED_STATISTICAL_AGGREGATION");
    if evaluate(&pooled_without_method).is_ok() {
        return Err("aggregation without method negative control was accepted".to_owned());
    }

    let mut aligned_without_receipt = fixture.clone();
    aligned_without_receipt["scenario_compositions"][3]["service_event_refs"][0]["alignment_state"] =
        json!("QUALIFIED_ALIGNMENT");
    if evaluate(&aligned_without_receipt).is_ok() {
        return Err("synthetic timebase promotion negative control was accepted".to_owned());
    }

    let mut duplicate_case = fixture.clone();
    duplicate_case["adversarial_cases"][1]["case_id"] =
        duplicate_case["adversarial_cases"][0]["case_id"].clone();
    if evaluate(&duplicate_case).is_ok() {
        return Err("duplicate adversarial identity negative control was accepted".to_owned());
    }

    let mut single_with_multiple = fixture.clone();
    single_with_multiple["scenario_compositions"][0]["climate_member_refs"] =
        json!(["SYN-TMY-001", "SYN-STATION-001"]);
    if evaluate(&single_with_multiple).is_ok() {
        return Err("single-member composition negative control was accepted".to_owned());
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = if args.as_slice() == ["--self-test"] {
        run_self_tests().map(|()| {
            println!(
                "HOME_FREEDOM_CLIMATE_EVIDENCE_PASS: typed members, source rights, scenario references, composition and 25 adversarial cases"
            );
        })
    } else if args.is_empty() {
        serde_json::from_str::<Value>(FIXTURE)
            .map_err(|error| format!("fixture JSON parse failed: {error}"))
            .and_then(|fixture| evaluate(&fixture))
            .map(|report| println!("{}", report))
    } else {
        Err("usage: home-freedom-climate-evidence-qualifier [--self-test]".to_owned())
    };
    if let Err(error) = result {
        eprintln!("HOME_FREEDOM_CLIMATE_EVIDENCE_FAIL: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_fixture_passes() {
        let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
        let report = evaluate(&fixture).unwrap();
        assert_eq!(report["climate_member_count"], 7);
        assert_eq!(report["scenario_count"], 5);
        assert_eq!(report["adversarial_case_count"], 25);
    }

    #[test]
    fn full_negative_control_self_test_passes() {
        run_self_tests().unwrap();
    }

    #[test]
    fn schema_declares_draft_2020_12_and_required_contract_sections() {
        check_schema_contract().unwrap();
    }
}
