//! Synthetic-only crosswalk checker for Highveld household-service outcomes.
//! This does not evaluate observations or replace Sol-Atlas/Symtropy owner-native tools.

#![forbid(unsafe_code)]

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fmt;

pub const PROFILE_JSON: &str = include_str!("../../../docs/research/highveld-keep/HIGHVELD_SERVICE_OUTCOME_BINDINGS_V0_1.json");
pub const CORPUS_JSON: &str = include_str!("../../../docs/research/highveld-keep/HIGHVELD_SERVICE_OUTCOME_BINDING_CORPUS_V0_1.json");
pub const PROFILE_SCHEMA_JSON: &str = include_str!("../../../docs/research/highveld-keep/HIGHVELD_SERVICE_OUTCOME_BINDINGS_V0_1.schema.json");
pub const CORPUS_SCHEMA_JSON: &str = include_str!("../../../docs/research/highveld-keep/HIGHVELD_SERVICE_OUTCOME_BINDING_CORPUS_V0_1.schema.json");

const PROFILE_ID: &str = "HIGHVELD-SERVICE-OUTCOME-BINDINGS-V0.1";
const CORPUS_ID: &str = "HIGHVELD-SERVICE-OUTCOME-BINDING-CORPUS-V0.1";

const OBSERVATION_FIELDS: &[&str] = &[
    "metric_id","value","unit","evidence_state","evidence_kind","source_owner","source_contract",
    "source_revision","source_receipt_id","source_subject_digest","scope_digest","design_revision",
    "bom_revision","observation_window","context","instrument_id","calibration_state","coverage_fraction",
];
const REQUIREMENT_FIELDS: &[&str] = &[
    "metric_id","unit","direction","threshold","evidence_class","definition_revision",
    "expected_source_owner","expected_source_contract","expected_source_revision",
];
const EVENT_FIELDS: &[&str] = &["id","generation","tick","sequence","kind"];
const SIGNAL_FIELDS: &[&str] = &[
    "service_id","target","metric_id","unit","requirement_evidence_class","evidence_kind",
    "source_owner_bound","source_receipt_present","service_floor_defined","instrument_profile_admitted",
    "calibration_current","scope_matches","design_revision_matches","bom_revision_matches",
    "observation_window_matches","context_matches","coverage_meets_declared_scope","observation_current",
    "source_revision_current","consent_valid","purpose_authorized","duplicate_receipt",
    "device_command_requested","claim_from_model_or_vendor","physical_quality_claim",
    "event_wrapper_present","event_contract_valid","event_used_as_measurement",
];
const EXPECTED_METRICS: &[(&str,&str,&str,&str)] = &[
    ("water.source_withdrawal","WATERLOOP","L","resource_flows.source_withdrawal.quantity"),
    ("water.usable_non_potable_delivery","WATERLOOP","L","resource_flows.usable_non_potable_delivery.quantity"),
    ("water.rejected_or_residual_water","WATERLOOP","L","resource_flows.rejected_or_residual_water.quantity"),
    ("water.electrical_energy","WATERLOOP","kWh","resource_flows.electrical_energy.quantity"),
    ("heatbank.electrical_energy","HEATBANK","kWh","resource_flows.electrical_energy.quantity"),
    ("heatbank.useful_thermal_service","HEATBANK","kWh_th","resource_flows.useful_thermal_service.quantity"),
    ("heatbank.water_draw","HEATBANK","L","resource_flows.water_draw.quantity"),
    ("heatbank.storage_loss","HEATBANK","kWh_th","resource_flows.storage_loss.quantity"),
    ("aircare.electrical_energy","AIRCARE","kWh","resource_flows.electrical_energy.quantity"),
    ("aircare.outdoor_air_delivery","AIRCARE","m3","resource_flows.outdoor_air_delivery.quantity"),
];
const METRIC_FIELDS: &[&str] = &[
    "metric_id","service_id","package_path","unit","target_owner","target_contract","target_contract_version",
    "target_path","requirement_evidence_class","required_observation_evidence_kind","source_owner_state",
    "source_contract_state","source_revision_state","source_receipt_state","instrument_profile_state",
    "service_floor_state","admission_state",
];
const UNBOUND_FIELDS: &[&str] = &["id","service_id","package_path","reason"];
const BINDING_POLICY_FIELDS: &[&str] = &[
    "source_owner_must_be_exact","source_contract_revision_must_be_exact","source_receipt_and_subject_digest_required",
    "hard_service_floor_must_be_defined","instrument_profile_must_be_admitted_for_declared_metric",
    "calibration_and_coverage_must_be_current","scope_design_bom_window_and_context_must_match",
    "timeline_events_never_count_as_physical_measurements","post_occupancy_result_never_proves_instrument_calibration",
    "device_commands_not_authorized","eligible_state_is_review_only_not_pass",
];
const OUTCOME_CONTRACT_FIELDS: &[&str] = &[
    "owner","contract_id","schema_id","schema_version","corpus_id","requirement_fields","observation_fields",
    "evidence_classes","evidence_kinds","non_authority",
];
const TIMELINE_CONTRACT_FIELDS: &[&str] = &[
    "owner","contract_id","schema_version","benchmark_id","event_fields","event_kinds","grouping_rule","non_authority",
];
const EXPECTED_CASES: &[(&str,&str)] = &[
    ("HSOB-001","UNBOUND_SOURCE_OWNER"),
    ("HSOB-002","UNBOUND_SOURCE_RECEIPT"),
    ("HSOB-003","UNBOUND_SERVICE_FLOOR"),
    ("HSOB-004","UNBOUND_INSTRUMENT_PROFILE"),
    ("HSOB-005","UNKNOWN_CALIBRATION"),
    ("HSOB-006","UNKNOWN_COVERAGE"),
    ("HSOB-007","UNKNOWN_OBSERVATION"),
    ("HSOB-008","INVALIDATED_SOURCE"),
    ("HSOB-009","NOT_COMPARABLE"),
    ("HSOB-010","NOT_COMPARABLE"),
    ("HSOB-011","NOT_COMPARABLE"),
    ("HSOB-012","NOT_COMPARABLE"),
    ("HSOB-013","NOT_COMPARABLE"),
    ("HSOB-014","NOT_COMPARABLE"),
    ("HSOB-015","NOT_COMPARABLE"),
    ("HSOB-016","NOT_COMPARABLE"),
    ("HSOB-017","REJECTED_EVIDENCE_CLASS"),
    ("HSOB-018","REJECTED_EVIDENCE_CLASS"),
    ("HSOB-019","REJECTED_CLAIM_PROMOTION"),
    ("HSOB-020","REJECTED_PURPOSE"),
    ("HSOB-021","REJECTED_CONSENT"),
    ("HSOB-022","REJECTED_DUPLICATE_RECEIPT"),
    ("HSOB-023","REJECTED_NO_DEVICE_AUTHORITY"),
    ("HSOB-024","REJECTED_EVIDENCE_CLASS"),
    ("HSOB-025","REJECTED_EVENT_SCOPE"),
    ("HSOB-026","REJECTED_EVENT_CONTRACT"),
    ("HSOB-027","REJECTED_CLAIM_PROMOTION"),
    ("HSOB-028","EVENT_AVAILABILITY_ONLY"),
    ("HSOB-029","REJECTED_EVIDENCE_CLASS"),
    ("HSOB-030","REJECTED_EVENT_CONTRACT"),
    ("HSOB-031","REJECTED_CLAIM_PROMOTION"),
    ("HSOB-032","NOT_COMPARABLE"),
    ("HSOB-033","UNBOUND_INSTRUMENT_PROFILE"),
    ("HSOB-034","INVALIDATED_SOURCE"),
    ("HSOB-035","REJECTED_CLAIM_PROMOTION"),
    ("HSOB-036","ELIGIBLE_FOR_INDEPENDENT_OWNER_REVIEW_ONLY"),
];

#[derive(Debug)]
struct UniqueJson(Value);

impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D>(deserializer:D)->Result<Self,D::Error>
    where D:Deserializer<'de> {
        deserializer.deserialize_any(UniqueJsonVisitor)
    }
}
struct UniqueJsonVisitor;
impl<'de> Visitor<'de> for UniqueJsonVisitor {
    type Value=UniqueJson;
    fn expecting(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
        f.write_str("a JSON value with no duplicate member names")
    }
    fn visit_bool<E>(self,v:bool)->Result<Self::Value,E> where E:serde::de::Error { Ok(UniqueJson(Value::Bool(v))) }
    fn visit_i64<E>(self,v:i64)->Result<Self::Value,E> where E:serde::de::Error { Ok(UniqueJson(Value::Number(v.into()))) }
    fn visit_u64<E>(self,v:u64)->Result<Self::Value,E> where E:serde::de::Error { Ok(UniqueJson(Value::Number(v.into()))) }
    fn visit_f64<E>(self,v:f64)->Result<Self::Value,E> where E:serde::de::Error {
        let n=serde_json::Number::from_f64(v).ok_or_else(||E::custom("non-finite JSON number"))?;
        Ok(UniqueJson(Value::Number(n)))
    }
    fn visit_str<E>(self,v:&str)->Result<Self::Value,E> where E:serde::de::Error { Ok(UniqueJson(Value::String(v.to_owned()))) }
    fn visit_string<E>(self,v:String)->Result<Self::Value,E> where E:serde::de::Error { Ok(UniqueJson(Value::String(v))) }
    fn visit_unit<E>(self)->Result<Self::Value,E> where E:serde::de::Error { Ok(UniqueJson(Value::Null)) }
    fn visit_none<E>(self)->Result<Self::Value,E> where E:serde::de::Error { Ok(UniqueJson(Value::Null)) }
    fn visit_some<D>(self,d:D)->Result<Self::Value,D::Error> where D:Deserializer<'de> { UniqueJson::deserialize(d) }
    fn visit_seq<A>(self,mut seq:A)->Result<Self::Value,A::Error> where A:SeqAccess<'de> {
        let mut v=Vec::new();
        while let Some(item)=seq.next_element::<UniqueJson>()? {v.push(item.0);}
        Ok(UniqueJson(Value::Array(v)))
    }
    fn visit_map<A>(self,mut map:A)->Result<Self::Value,A::Error> where A:MapAccess<'de> {
        let mut v=Map::new();let mut seen=BTreeSet::new();
        while let Some(k)=map.next_key::<String>()?{
            if !seen.insert(k.clone()){return Err(serde::de::Error::custom(format!("duplicate JSON member {k:?}")));}
            let item=map.next_value::<UniqueJson>()?;
            v.insert(k,item.0);
        }
        Ok(UniqueJson(Value::Object(v)))
    }
}
fn parse_unique(s:&str)->Result<Value,String>{
    let mut d=serde_json::Deserializer::from_str(s);
    let v=UniqueJson::deserialize(&mut d).map_err(|e|format!("STRICT_JSON_REJECTED:{e}"))?;
    d.end().map_err(|e|format!("JSON_TRAILING_DATA:{e}"))?;
    Ok(v.0)
}
fn exact_keys(v:&Value,keys:&[&str])->bool{
    let Some(o)=v.as_object() else{return false;};
    o.len()==keys.len()&&keys.iter().all(|k|o.contains_key(*k))
}
fn arr_eq(v:&Value,want:&[&str])->bool{
    v.as_array().is_some_and(|a|a.len()==want.len()&&a.iter().zip(want).all(|(v,w)|v.as_str()==Some(*w)))
}
fn text<'a>(v:&'a Value,k:&str)->&'a str{v.get(k).and_then(Value::as_str).unwrap_or("")}
fn flag(v:&Value,k:&str)->bool{v.get(k).and_then(Value::as_bool)==Some(true)}

pub fn classify_case(case:&Value)->&'static str{
    let s=&case["signals"];
    let service=text(s,"service_id");
    if !["WATERLOOP","HEATBANK","AIRCARE"].contains(&service){return "NOT_COMPARABLE";}
    if flag(s,"device_command_requested"){return "REJECTED_NO_DEVICE_AUTHORITY";}
    match text(s,"target"){
        "timeline_event"=>{
            if !flag(s,"event_wrapper_present"){return "REJECTED_EVENT_SCOPE";}
            if !flag(s,"event_contract_valid"){return "REJECTED_EVENT_CONTRACT";}
            if flag(s,"event_used_as_measurement")||flag(s,"physical_quality_claim"){return "REJECTED_CLAIM_PROMOTION";}
            return "EVENT_AVAILABILITY_ONLY";
        }
        "observation"=>{}
        _=>return "NOT_COMPARABLE",
    }
    if flag(s,"claim_from_model_or_vendor")||flag(s,"physical_quality_claim"){return "REJECTED_CLAIM_PROMOTION";}
    let metric=text(s,"metric_id");
    let unit=text(s,"unit");
    let known=EXPECTED_METRICS.iter().find(|(id,svc,_)|*id==metric&&*svc==service);
    let Some((_,_,expected_unit))=known else{return "NOT_COMPARABLE";};
    if unit!=*expected_unit{return "NOT_COMPARABLE";}
    if text(s,"requirement_evidence_class")!="meter"||text(s,"evidence_kind")!="meter_measurement"{return "REJECTED_EVIDENCE_CLASS";}
    if !flag(s,"purpose_authorized"){return "REJECTED_PURPOSE";}
    if !flag(s,"consent_valid"){return "REJECTED_CONSENT";}
    if !flag(s,"scope_matches")||!flag(s,"design_revision_matches")||!flag(s,"bom_revision_matches")
        ||!flag(s,"observation_window_matches")||!flag(s,"context_matches"){return "NOT_COMPARABLE";}
    if !flag(s,"source_revision_current"){return "INVALIDATED_SOURCE";}
    if !flag(s,"observation_current"){return "UNKNOWN_OBSERVATION";}
    if !flag(s,"calibration_current"){return "UNKNOWN_CALIBRATION";}
    if !flag(s,"instrument_profile_admitted"){return "UNBOUND_INSTRUMENT_PROFILE";}
    if !flag(s,"coverage_meets_declared_scope"){return "UNKNOWN_COVERAGE";}
    if !flag(s,"source_owner_bound"){return "UNBOUND_SOURCE_OWNER";}
    if !flag(s,"source_receipt_present"){return "UNBOUND_SOURCE_RECEIPT";}
    if !flag(s,"service_floor_defined"){return "UNBOUND_SERVICE_FLOOR";}
    if flag(s,"duplicate_receipt"){return "REJECTED_DUPLICATE_RECEIPT";}
    "ELIGIBLE_FOR_INDEPENDENT_OWNER_REVIEW_ONLY"
}

fn validate_profile_schema(schema:&Value)->Result<(),String>{
    if text(schema,"title")!="Highveld Household Service Outcome Binding Profile v0.1"
        ||schema["type"]!="object"||schema["additionalProperties"]!=false
        ||!arr_eq(&schema["required"],&["schema_version","profile_id","classification","claim_ceiling","parent_issue","base_package_id","owner_contracts","service_ids","outcome_metrics","unbound_metrics","binding_policy"])
    {return Err("PROFILE_SCHEMA_ROOT_INVALID".into());}
    let observation=&schema["properties"]["owner_contracts"]["properties"]["in_use_outcomes"];
    if observation["properties"]["observation_fields"]["const"]!=serde_json::json!(OBSERVATION_FIELDS)
        ||observation["properties"]["requirement_fields"]["const"]!=serde_json::json!(REQUIREMENT_FIELDS)
    {return Err("PROFILE_SCHEMA_OBSERVATION_FIELDS_DRIFT".into());}
    let timeline=&schema["properties"]["owner_contracts"]["properties"]["service_timeline"];
    if timeline["properties"]["event_fields"]["const"]!=serde_json::json!(EVENT_FIELDS)
        ||timeline["properties"]["event_kinds"]["const"]!=serde_json::json!(["restored","lost"])
    {return Err("PROFILE_SCHEMA_TIMELINE_FIELDS_DRIFT".into());}
    Ok(())
}
fn validate_profile(p:&Value)->Result<(),String>{
    if !exact_keys(p,&["schema_version","profile_id","classification","claim_ceiling","parent_issue","base_package_id","owner_contracts","service_ids","outcome_metrics","unbound_metrics","binding_policy"])
        ||text(p,"schema_version")!="luminous.highveld.service-outcome-binding/0.1"
        ||text(p,"profile_id")!=PROFILE_ID
        ||text(p,"classification")!="synthetic_owner_contract_crosswalk_only"
        ||text(p,"claim_ceiling")!="owner_contract_field_mapping_and_unbound_state_conformance_only"
        ||text(p,"parent_issue")!="https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/700"
        ||text(p,"base_package_id")!="HIGHVELD-HOUSEHOLD-SERVICE-SYNTHETIC-001"
    {return Err("PROFILE_IDENTITY_OR_CLAIM_CEILING_INVALID".into());}
    if !arr_eq(&p["service_ids"],&["WATERLOOP","HEATBANK","AIRCARE"])||!exact_keys(&p["owner_contracts"],&["in_use_outcomes","service_timeline"]){return Err("SERVICE_OR_OWNER_CONTRACT_SET_INVALID".into());}
    let outcome=&p["owner_contracts"]["in_use_outcomes"];
    if !exact_keys(outcome,OUTCOME_CONTRACT_FIELDS){return Err("OUTCOME_OWNER_CONTRACT_FIELDS_INVALID".into());}
    if text(outcome,"owner")!="sol-atlas"||text(outcome,"contract_id")!="CITY-IN-USE-OUTCOMES-001"
        ||text(outcome,"schema_id")!="org.luminousdynamics.city-in-use-outcomes.synthetic-corpus"
        ||text(outcome,"schema_version")!="0.1.0"||text(outcome,"corpus_id")!="CITY-IN-USE-OUTCOMES-CORPUS-V0.1"
        ||!arr_eq(&outcome["requirement_fields"],REQUIREMENT_FIELDS)
        ||!arr_eq(&outcome["observation_fields"],OBSERVATION_FIELDS)
        ||!arr_eq(&outcome["evidence_classes"],&["meter","experience","service"])
        ||!arr_eq(&outcome["evidence_kinds"],&["meter_measurement","consented_survey","service_receipt"])
        ||text(outcome,"non_authority")!="read_only_comparability_classifier_not_meter_owner_or_service_executor"
    {return Err("OUTCOME_OWNER_CONTRACT_DRIFT".into());}
    let timeline=&p["owner_contracts"]["service_timeline"];
    if !exact_keys(timeline,TIMELINE_CONTRACT_FIELDS){return Err("TIMELINE_OWNER_CONTRACT_FIELDS_INVALID".into());}
    if text(timeline,"owner")!="symtropy"||text(timeline,"contract_id")!="SPATIAL-RESILIENCE-TIMELINE-ORACLE"
        ||text(timeline,"schema_version")!="0.1.0"||text(timeline,"benchmark_id")!="SPATIAL-RESILIENCE-TIMELINE-ORACLE"
        ||!arr_eq(&timeline["event_fields"],EVENT_FIELDS)||!arr_eq(&timeline["event_kinds"],&["restored","lost"])
        ||text(timeline,"non_authority")!="availability_and_stable_recovery_only_not_physical_measurement_or_quality"
        ||text(timeline,"grouping_rule").len()<40
    {return Err("TIMELINE_OWNER_CONTRACT_DRIFT".into());}
    let metrics=p["outcome_metrics"].as_array().ok_or("METRICS_MISSING")?;
    if metrics.len()!=EXPECTED_METRICS.len(){return Err("METRIC_COUNT_INVALID".into());}
    for (m,(id,service,unit,package_path)) in metrics.iter().zip(EXPECTED_METRICS){
        if !exact_keys(m,METRIC_FIELDS)||text(m,"metric_id")!=*id||text(m,"service_id")!=*service||text(m,"unit")!=*unit
            ||text(m,"package_path")!=*package_path
            ||text(m,"target_owner")!="sol-atlas"||text(m,"target_contract")!="CITY-IN-USE-OUTCOMES-001"||text(m,"target_contract_version")!="0.1.0"
            ||text(m,"target_path")!="input.requirements[] + input.observations[]"
            ||text(m,"requirement_evidence_class")!="meter"||text(m,"required_observation_evidence_kind")!="meter_measurement"
            ||text(m,"source_owner_state")!="UNBOUND_NOT_SELECTED"||text(m,"source_contract_state")!="UNBOUND_NOT_SELECTED"
            ||text(m,"source_revision_state")!="UNBOUND_NOT_SELECTED"||text(m,"source_receipt_state")!="NOT_PRESENT"
            ||text(m,"instrument_profile_state")!="NOT_ADMITTED_OR_UNBOUND"||text(m,"service_floor_state")!="UNKNOWN"
            ||text(m,"admission_state")!="UNBOUND"
        {return Err(format!("METRIC_MAPPING_DRIFT:{id}"));}
    }
    let unbound=p["unbound_metrics"].as_array().ok_or("UNBOUND_METRICS_MISSING")?;
    let expected_unbound=[
        ("aircare.clean_air_delivery","AIRCARE","resource_flows.clean_air_delivery.quantity"),
        ("aircare.condensate_or_captured_residual","AIRCARE","resource_flows.condensate_or_captured_residual.quantity"),
    ];
    if unbound.len()!=expected_unbound.len(){return Err("UNBOUND_METRIC_COUNT_INVALID".into());}
    for (item,(id,service,path)) in unbound.iter().zip(expected_unbound) {
        if !exact_keys(item,UNBOUND_FIELDS)||text(item,"id")!=id||text(item,"service_id")!=service
            ||text(item,"package_path")!=path||text(item,"reason").len()<20
        {return Err(format!("UNBOUND_METRIC_DRIFT:{id}"));}
    }
    let policy=p.get("binding_policy").ok_or("BINDING_POLICY_MISSING")?;
    if !exact_keys(policy,BINDING_POLICY_FIELDS)||policy.as_object().is_none_or(|o|o.values().any(|v|v.as_bool()!=Some(true))){
        return Err("BINDING_POLICY_WEAKENED".into());
    }
    Ok(())
}
pub fn validate_corpus(p:&Value,c:&Value)->Result<usize,String>{
    if !exact_keys(c,&["schema_version","corpus_id","classification","claim_ceiling","profile_id","cases"])
        ||text(c,"schema_version")!="luminous.highveld.service-outcome-binding-corpus/0.1"
        ||text(c,"corpus_id")!=CORPUS_ID||text(c,"classification")!="synthetic_owner_contract_crosswalk_cases"
        ||text(c,"claim_ceiling")!="synthetic_mapping_and_unbound_state_checks_only"||text(c,"profile_id")!=PROFILE_ID
    {return Err("CORPUS_IDENTITY_INVALID".into());}
    let cases=c["cases"].as_array().ok_or("CASES_MISSING")?;
    if cases.len()!=EXPECTED_CASES.len(){return Err("CASE_COUNT_INVALID".into());}
    for(case,(id,expected))in cases.iter().zip(EXPECTED_CASES){
        if !exact_keys(case,&["case_id","description","signals","expected_disposition"])
            ||text(case,"case_id")!=*id||text(case,"description").len()<20
            ||!exact_keys(&case["signals"],SIGNAL_FIELDS)
            ||text(case,"expected_disposition")!=*expected
        {return Err(format!("CASE_ID_OR_EXPECTED_DRIFT:{id}"));}
        let actual=classify_case(case);
        if actual!=*expected{return Err(format!("{id}:EXPECTED_{expected},GOT_{actual}"));}
    }
    Ok(cases.len())
}
pub fn validate_tracked()->Result<usize,String>{
    let ps=parse_unique(PROFILE_SCHEMA_JSON)?;validate_profile_schema(&ps)?;
    let cs=parse_unique(CORPUS_SCHEMA_JSON)?;
    if cs["type"]!="object"||cs["additionalProperties"]!=false
        ||!arr_eq(&cs["required"],&["schema_version","corpus_id","classification","claim_ceiling","profile_id","cases"])
        ||cs["properties"]["cases"]["type"]!="array"
        ||cs["properties"]["cases"]["minItems"]!=EXPECTED_CASES.len()
        ||cs["properties"]["cases"]["maxItems"]!=EXPECTED_CASES.len()
        ||cs["properties"]["cases"]["items"]["$ref"]!="#/$defs/case"
    {return Err("CORPUS_SCHEMA_CONTRACT_INVALID".into());}
    let case_schema=&cs["$defs"]["case"];
    if case_schema["type"]!="object"||case_schema["additionalProperties"]!=false
        ||!arr_eq(&case_schema["required"],&["case_id","description","signals","expected_disposition"])
        ||!exact_keys(&case_schema["properties"],&["case_id","description","signals","expected_disposition"])
    {return Err("CORPUS_CASE_SCHEMA_INVALID".into());}
    let signals_schema=&case_schema["properties"]["signals"];
    if signals_schema["type"]!="object"||signals_schema["additionalProperties"]!=false
        ||!arr_eq(&signals_schema["required"],SIGNAL_FIELDS)
        ||!exact_keys(&signals_schema["properties"],SIGNAL_FIELDS)
    {return Err("CORPUS_SIGNAL_SCHEMA_INVALID".into());}
    let p=parse_unique(PROFILE_JSON)?;validate_profile(&p)?;
    let c=parse_unique(CORPUS_JSON)?;validate_corpus(&p,&c)
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test] fn tracked_owner_bindings_and_36_known_answers_validate(){assert_eq!(validate_tracked().unwrap(),36);}
 #[test] fn duplicate_json_member_is_rejected(){assert!(parse_unique(r#"{"a":1,"a":2}"#).is_err());}
 #[test] fn observation_fields_are_exactly_owner_native(){let p=parse_unique(PROFILE_JSON).unwrap();assert!(arr_eq(&p["owner_contracts"]["in_use_outcomes"]["observation_fields"],OBSERVATION_FIELDS));}
 #[test] fn timeline_is_not_a_measurement_contract(){let c=parse_unique(CORPUS_JSON).unwrap();let case=c["cases"][27].clone();assert_eq!(classify_case(&case),"EVENT_AVAILABILITY_ONLY");}
 #[test] fn event_cannot_prove_physical_quality(){let c=parse_unique(CORPUS_JSON).unwrap();let case=c["cases"][26].clone();assert_eq!(classify_case(&case),"REJECTED_CLAIM_PROMOTION");}
 #[test] fn owner_unknown_is_unbound_not_pass(){let c=parse_unique(CORPUS_JSON).unwrap();let case=c["cases"][0].clone();assert_eq!(classify_case(&case),"UNBOUND_SOURCE_OWNER");}
 #[test] fn device_command_is_never_authorized(){let mut c=parse_unique(CORPUS_JSON).unwrap();c["cases"][35]["signals"]["device_command_requested"]=Value::Bool(true);assert_eq!(classify_case(&c["cases"][35]),"REJECTED_NO_DEVICE_AUTHORITY");}
 #[test] fn unknown_aircare_pollutant_metric_is_not_comparable(){let c=parse_unique(CORPUS_JSON).unwrap();assert_eq!(classify_case(&c["cases"][15]),"NOT_COMPARABLE");}
 #[test] fn unknown_service_floor_never_passes(){let c=parse_unique(CORPUS_JSON).unwrap();assert_eq!(classify_case(&c["cases"][2]),"UNBOUND_SERVICE_FLOOR");}
 #[test] fn stale_source_invalidates_observation(){let c=parse_unique(CORPUS_JSON).unwrap();assert_eq!(classify_case(&c["cases"][7]),"INVALIDATED_SOURCE");}
 #[test] fn unknown_or_stale_calibration_is_not_admitted(){let c=parse_unique(CORPUS_JSON).unwrap();assert_eq!(classify_case(&c["cases"][4]),"UNKNOWN_CALIBRATION");}
 #[test] fn all_synthetic_gates_yield_review_only_not_pass(){let c=parse_unique(CORPUS_JSON).unwrap();assert_eq!(classify_case(&c["cases"][35]),"ELIGIBLE_FOR_INDEPENDENT_OWNER_REVIEW_ONLY");}
 #[test] fn changing_timeline_event_kind_contract_is_rejected(){let mut p=parse_unique(PROFILE_JSON).unwrap();p["owner_contracts"]["service_timeline"]["event_kinds"][0]=Value::String("commanded".into());assert!(validate_profile(&p).is_err());}
 #[test] fn changing_meter_mapping_unit_is_rejected(){let mut p=parse_unique(PROFILE_JSON).unwrap();p["outcome_metrics"][0]["unit"]=Value::String("m3".into());assert!(validate_profile(&p).is_err());}
}
