use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::process::ExitCode;

const POLICY_SCHEMA: &str = "luminous.ieac-risk-assurance-policy.v0.1";
const PROFILE_SCHEMA: &str = "luminous.industry-assurance-profile.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-risk-control-selector-report.v0.1";
const DIMENSIONS: &[&str] = &[
    "consequence", "model_influence", "exposure", "reversibility", "detectability", "fallback",
];
const TIERS: &[&str] = &[
    "D0_descriptive", "D1_advisory", "D2_delegated_planning", "D3_autonomous_control",
];

#[derive(Debug)]
enum StrictJson {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<StrictJson>),
    Object(BTreeMap<String, StrictJson>),
}

impl StrictJson {
    fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(value),
            Self::Number(value) => Value::Number(value),
            Self::String(value) => Value::String(value),
            Self::Array(values) => Value::Array(values.into_iter().map(Self::into_value).collect()),
            Self::Object(values) => {
                let mut object = Map::new();
                for (key, value) in values {
                    object.insert(key, value.into_value());
                }
                Value::Object(object)
            }
        }
    }
}

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictJson;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("JSON without duplicate object member names")
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where E: de::Error {
                Ok(StrictJson::Null)
            }
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where E: de::Error {
                Ok(StrictJson::Null)
            }
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where E: de::Error {
                Ok(StrictJson::Bool(value))
            }
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where E: de::Error {
                Ok(StrictJson::Number(Number::from(value)))
            }
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where E: de::Error {
                Ok(StrictJson::Number(Number::from(value)))
            }
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where E: de::Error {
                Number::from_f64(value)
                    .map(StrictJson::Number)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: de::Error {
                Ok(StrictJson::String(value.to_owned()))
            }
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where E: de::Error {
                Ok(StrictJson::String(value))
            }
            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where A: SeqAccess<'de> {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element::<StrictJson>()? {
                    values.push(value);
                }
                Ok(StrictJson::Array(values))
            }
            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where A: MapAccess<'de> {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = access.next_entry::<String, StrictJson>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!(
                            "duplicate JSON object member: {key}"
                        )));
                    }
                    values.insert(key, value);
                }
                Ok(StrictJson::Object(values))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn required_string<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str)
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| format!("{context}: required non-empty string '{key}' is missing"))
}

fn array<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a [Value], String> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
        .ok_or_else(|| format!("{context}: '{key}' must be an array"))
}

fn add_controls(
    required: &mut BTreeMap<String, BTreeSet<String>>,
    control_ids: &[Value],
    reason_code: &str,
) -> Result<(), String> {
    for control in control_ids {
        let id = control.as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "policy control_ids must contain non-empty strings".to_owned())?;
        required.entry(id.to_owned()).or_default().insert(reason_code.to_owned());
    }
    Ok(())
}

fn add_controls_strings(
    required: &mut BTreeMap<String, BTreeSet<String>>,
    control_ids: &[String],
    reason_code: &str,
) {
    for id in control_ids {
        required.entry(id.clone()).or_default().insert(reason_code.to_owned());
    }
}

fn control_evidence_state(profile: &Value, evidence_dimension: &str) -> (String, usize) {
    let Some(dimension) = profile.pointer(&format!("/evidence_vector/{evidence_dimension}")) else {
        return ("unknown".into(), 0);
    };
    let state = dimension.get("state").and_then(Value::as_str).unwrap_or("unknown");
    let count = dimension.get("evidence").and_then(Value::as_array).map_or(0, |rows| {
        rows.iter().filter(|row| {
            row.get("artifact_id").and_then(Value::as_str).is_some_and(|id| !id.is_empty())
                && row.get("result").and_then(Value::as_str).is_some_and(|result| !result.is_empty())
        }).count()
    });
    match state {
        "established" if count > 0 => ("established".into(), count),
        "established" | "not_established" => ("missing".into(), count),
        "conflicted" => ("conflicted".into(), count),
        "not_applicable" => ("not_applicable".into(), count),
        "unknown" => ("unknown".into(), count),
        _ => ("unknown".into(), count),
    }
}

fn validate_policy_catalog(policy: &Value) -> Result<BTreeMap<String, Value>, String> {
    if policy.get("schema_version").and_then(Value::as_str) != Some(POLICY_SCHEMA) {
        return Err("unsupported risk assurance policy schema".into());
    }
    if policy.get("action_authority").and_then(Value::as_str) != Some("none")
        || policy.get("independent_qualification_attested").and_then(Value::as_bool) != Some(false)
    {
        return Err("risk selector policy may not authorize actions or attest qualification".into());
    }
    let action_policy = policy.get("action_policy").ok_or("policy.action_policy is missing")?;
    if action_policy.get("selector_may_authorize_action").and_then(Value::as_bool) != Some(false)
        || action_policy.get("selector_may_upgrade_tier").and_then(Value::as_bool) != Some(false)
        || action_policy.get("unknown_risk_blocks_clearance").and_then(Value::as_bool) != Some(true)
        || action_policy.get("direct_actuation_requires_tier").and_then(Value::as_str) != Some("D3_autonomous_control")
    {
        return Err("risk selector action policy violates non-authority and D3 boundaries".into());
    }
    let unknown_policy = policy.get("unknown_policy").ok_or("policy.unknown_policy is missing")?;
    if unknown_policy.get("never_reduce_controls").and_then(Value::as_bool) != Some(true)
        || unknown_policy.get("mode").and_then(Value::as_str) != Some("worst_case_union_plus_review")
    {
        return Err("risk selector unknown policy must be monotone and worst-case conservative".into());
    }
    let mut catalog = BTreeMap::new();
    for (index, item) in array(policy, "control_catalog", "policy")?.iter().enumerate() {
        let context = format!("policy.control_catalog[{index}]");
        let id = required_string(item, "control_id", &context)?;
        let dimension = required_string(item, "evidence_dimension", &context)?;
        if !DIMENSION_EVIDENCE_FIELDS.contains(&dimension) {
            return Err(format!("{context}: unknown evidence dimension '{dimension}'"));
        }
        if catalog.insert(id.to_owned(), item.clone()).is_some() {
            return Err(format!("{context}: duplicate control_id '{id}'"));
        }
    }

    let mut referenced = BTreeSet::new();
    let mut tier_keys = BTreeSet::new();
    for item in array(policy, "tier_minimums", "policy")? {
        let tier = required_string(item, "decision_tier", "tier rule")?;
        if !TIERS.contains(&tier) || !tier_keys.insert(tier.to_owned()) {
            return Err(format!("invalid or duplicate tier minimum '{tier}'"));
        }
        for control in array(item, "control_ids", "tier rule")? {
            referenced.insert(control.as_str().ok_or("tier control ID must be a string")?.to_owned());
        }
    }
    if tier_keys.len() != TIERS.len() {
        return Err("policy must define exactly one minimum for each D0-D3 tier".into());
    }

    let mut dimension_keys = BTreeSet::new();
    for item in array(policy, "dimension_rules", "policy")? {
        let dimension = required_string(item, "dimension", "dimension rule")?;
        let value = required_string(item, "value", "dimension rule")?;
        if !DIMENSIONS.contains(&dimension)
            || !dimension_keys.insert((dimension.to_owned(), value.to_owned()))
        {
            return Err(format!("invalid or duplicate risk rule '{dimension}={value}'"));
        }
        for control in array(item, "control_ids", "dimension rule")? {
            referenced.insert(control.as_str().ok_or("dimension control ID must be a string")?.to_owned());
        }
    }

    let mut combination_ids = BTreeSet::new();
    for item in array(policy, "combination_rules", "policy")? {
        let id = required_string(item, "rule_id", "combination rule")?;
        if !combination_ids.insert(id.to_owned()) {
            return Err(format!("duplicate combination rule '{id}'"));
        }
        for control in array(item, "control_ids", "combination rule")? {
            referenced.insert(control.as_str().ok_or("combination control ID must be a string")?.to_owned());
        }
    }
    let mut reviewed_classes = BTreeSet::new();
    for item in array(policy, "reviewed_class_rules", "policy")? {
        let class = required_string(item, "reviewed_class", "reviewed class rule")?;
        if !reviewed_classes.insert(class.to_owned()) {
            return Err(format!("duplicate reviewed-class rule '{class}'"));
        }
        for control in array(item, "additional_control_ids", "reviewed class rule")? {
            referenced.insert(control.as_str().ok_or("reviewed class control ID must be a string")?.to_owned());
        }
    }
    for control in array(unknown_policy, "additional_control_ids", "unknown policy")? {
        referenced.insert(control.as_str().ok_or("unknown-policy control ID must be a string")?.to_owned());
    }
    let worst_values = unknown_policy.get("worst_case_values").ok_or("unknown_policy.worst_case_values is missing")?;
    for dimension in DIMENSIONS {
        required_string(worst_values, dimension, "unknown_policy.worst_case_values")?;
    }
    for id in referenced {
        if !catalog.contains_key(&id) {
            return Err(format!("policy references unknown control_id '{id}'"));
        }
    }
    Ok(catalog)
}

const DIMENSION_EVIDENCE_FIELDS: &[&str] = &[
    "software_conformance", "numerical_verification", "physical_validation",
    "data_integrity_rights", "uncertainty_credibility", "generalization", "robustness",
    "operational_assurance", "safety_security_authority", "prospective_outcomes",
];

fn select_requirements(profile: &Value, policy: &Value, input_digest: &str, policy_digest: &str) -> Result<Value, String> {
    if profile.get("schema_version").and_then(Value::as_str) != Some(PROFILE_SCHEMA) {
        return Err("unsupported IEAC qualification profile schema".into());
    }
    let tier = required_string(profile, "decision_tier", "profile")?;
    if !TIERS.contains(&tier) {
        return Err(format!("unsupported decision tier '{tier}'"));
    }
    let risk = profile.get("risk_assessment").ok_or("profile.risk_assessment is missing")?;
    let reviewed_class = required_string(risk, "reviewed_class", "risk_assessment")?;
    let dimensions = risk.get("dimensions").ok_or("risk_assessment.dimensions is missing")?;
    let catalog = validate_policy_catalog(policy)?;
    let mut required: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut risk_has_unknown = false;
    let mut risk_unassessed = reviewed_class == "unassessed";
    let mut risk_values = Map::new();

    let tier_rule = array(policy, "tier_minimums", "policy")?
        .iter()
        .find(|item| item.get("decision_tier").and_then(Value::as_str) == Some(tier))
        .ok_or_else(|| format!("policy has no minimum controls for tier '{tier}'"))?;
    add_controls(
        &mut required,
        array(tier_rule, "control_ids", "tier rule")?,
        required_string(tier_rule, "reason_code", "tier rule")?,
    )?;

    let dimension_rules = array(policy, "dimension_rules", "policy")?;
    let unknown_policy = policy.get("unknown_policy").ok_or("policy.unknown_policy is missing")?;
    let worst_values = unknown_policy.get("worst_case_values").ok_or("unknown_policy.worst_case_values is missing")?;
    for dimension in DIMENSIONS {
        let actual = required_string(dimensions, dimension, "risk dimensions")?;
        risk_values.insert((*dimension).to_owned(), json!(actual));
        if actual == "unknown" {
            risk_has_unknown = true;
        }
        let lookup_value = if actual == "unknown" || risk_unassessed {
            required_string(worst_values, dimension, "unknown_policy.worst_case_values")?
        } else {
            actual
        };
        let rule = dimension_rules.iter()
            .find(|item| item.get("dimension").and_then(Value::as_str) == Some(dimension)
                && item.get("value").and_then(Value::as_str) == Some(lookup_value))
            .ok_or_else(|| format!("policy has no rule for risk dimension '{dimension}' value '{lookup_value}'"))?;
        let reason = if risk_unassessed {
            "RISK_UNASSESSED_WORST_CASE_DIMENSION"
        } else if actual == "unknown" {
            "RISK_UNKNOWN_WORST_CASE_DIMENSION"
        } else {
            required_string(rule, "reason_code", "dimension rule")?
        };
        add_controls(&mut required, array(rule, "control_ids", "dimension rule")?, reason)?;
    }

    let class_rule = array(policy, "reviewed_class_rules", "policy")?
        .iter()
        .find(|item| item.get("reviewed_class").and_then(Value::as_str) == Some(reviewed_class))
        .ok_or_else(|| format!("policy has no reviewed-class rule for '{reviewed_class}'"))?;
    let class_reason = format!("REVIEWED_CLASS_{}", reviewed_class.to_ascii_uppercase());
    add_controls(
        &mut required,
        array(class_rule, "additional_control_ids", "reviewed class rule")?,
        &class_reason,
    )?;

    for combination in array(policy, "combination_rules", "policy")? {
        let mut matches = true;
        for condition in array(combination, "conditions", "combination rule")? {
            let axis = required_string(condition, "dimension", "combination condition")?;
            let actual = dimensions.get(axis).and_then(Value::as_str)
                .ok_or_else(|| format!("profile risk dimension '{axis}' is missing"))?;
            let current = if actual == "unknown" || risk_unassessed {
                required_string(worst_values, axis, "unknown_policy.worst_case_values")?
            } else {
                actual
            };
            let values = array(condition, "values", "combination condition")?;
            if !values.iter().any(|value| value.as_str() == Some(current)) {
                matches = false;
                break;
            }
        }
        if matches {
            add_controls(
                &mut required,
                array(combination, "control_ids", "combination rule")?,
                required_string(combination, "reason_code", "combination rule")?,
            )?;
        }
    }

    if risk_has_unknown || risk_unassessed {
        add_controls(
            &mut required,
            array(unknown_policy, "additional_control_ids", "unknown policy")?,
            "UNKNOWN_OR_UNASSESSED_RISK",
        )?;
    }

    let influence = dimensions.get("model_influence").and_then(Value::as_str).unwrap_or("unknown");
    let required_actuation_tier = required_string(
        policy.get("action_policy").ok_or("policy.action_policy is missing")?,
        "direct_actuation_requires_tier",
        "policy.action_policy",
    )?;
    let tier_compatibility = if influence == "direct_actuation" && tier != required_actuation_tier {
        "blocked"
    } else {
        "compatible"
    };

    let mut missing = Vec::new();
    let mut unknown = Vec::new();
    let mut conflicted = Vec::new();
    let mut not_applicable = Vec::new();
    let mut output_controls = Vec::new();
    for (id, reasons) in &required {
        let definition = catalog.get(id)
            .ok_or_else(|| format!("policy requirement references unknown control_id '{id}'"))?;
        let evidence_dimension = required_string(definition, "evidence_dimension", "control catalog")?;
        let description = required_string(definition, "description", "control catalog")?;
        let (state, evidence_count) = control_evidence_state(profile, evidence_dimension);
        match state.as_str() {
            "missing" => missing.push(id.clone()),
            "unknown" => unknown.push(id.clone()),
            "conflicted" => conflicted.push(id.clone()),
            "not_applicable" => not_applicable.push(id.clone()),
            _ => {}
        }
        output_controls.push(json!({
            "control_id": id,
            "evidence_dimension": evidence_dimension,
            "description": description,
            "evidence_state": state,
            "evidence_count": evidence_count,
            "reason_codes": reasons.iter().cloned().collect::<Vec<_>>()
        }));
    }

    let disposition = if tier_compatibility == "blocked"
        || risk_unassessed
        || risk_has_unknown
        || !unknown.is_empty()
        || !conflicted.is_empty()
        || !not_applicable.is_empty()
    {
        "indeterminate"
    } else if !missing.is_empty() {
        "requirements_missing"
    } else {
        "requirements_satisfied"
    };

    let approvals: Vec<String> = required.keys()
        .filter(|id| ["HUMAN","DUAL","SAFETY","HAZARD","GOVERN","PRECHECK"].contains(&id.as_str()))
        .cloned().collect();
    let monitoring: Vec<String> = required.keys()
        .filter(|id| ["MONITOR","DRIFT","CANARY","STAGED","REDUNDANT","INSPECT","COMMON","FAILMON","FLOOR"].contains(&id.as_str()))
        .cloned().collect();
    let fallback: Vec<String> = required.keys()
        .filter(|id| ["FALLBACK","STOP","SAFESTATE","FALLBACK_BUILD","RECOVERY","ROLLBACK","FAILSAFE","DEGRADED","DRILL"].contains(&id.as_str()))
        .cloned().collect();

    Ok(json!({
        "schema_version": REPORT_SCHEMA,
        "profile_id": required_string(profile, "profile_id", "profile")?,
        "decision_tier": tier,
        "risk_assessment_state": if risk_unassessed { "unassessed" } else if risk_has_unknown { "contains_unknown" } else { "reviewed" },
        "risk_assessment_class": reviewed_class,
        "risk_dimensions": risk_values,
        "policy_id": required_string(policy, "policy_id", "policy")?,
        "policy_version": required_string(policy, "policy_version", "policy")?,
        "policy_sha256": policy_digest,
        "selector_version": required_string(policy, "selector_version", "policy")?,
        "input_profile_sha256": input_digest,
        "required_controls": output_controls,
        "missing_controls": missing,
        "unknown_controls": unknown,
        "conflicted_controls": conflicted,
        "not_applicable_controls": not_applicable,
        "human_approvals": approvals,
        "monitoring_obligations": monitoring,
        "fallback_obligations": fallback,
        "tier_compatibility": tier_compatibility,
        "requirements_disposition": disposition,
        "independent_qualification_attested": false,
        "action_authority": "none"
    }))
}

fn read_strict(path: &str) -> Result<(Vec<u8>, Value), String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let parsed: StrictJson = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{path}: invalid or duplicate-member JSON: {error}"))?;
    Ok((bytes, parsed.into_value()))
}

fn run(profile_path: &str, policy_path: &str, output_path: &str) -> Result<(), String> {
    let (profile_bytes, profile) = read_strict(profile_path)?;
    let (policy_bytes, policy) = read_strict(policy_path)?;
    let report = select_requirements(
        &profile,
        &policy,
        &sha256_hex(&profile_bytes),
        &sha256_hex(&policy_bytes),
    )?;
    let encoded = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot encode selector report: {error}"))?;
    fs::write(output_path, encoded)
        .map_err(|error| format!("cannot write selector report: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: industry-risk-control-selector <ieac-profile.json> <risk-policy.json> <report.json>");
        return ExitCode::from(4);
    }
    match run(&args[1], &args[2], &args[3]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("risk-control selection failed closed: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../schemas/examples/industry-assurance-qualification-profile-synthetic-v0.1.json"
        )).unwrap()
    }

    fn policy() -> Value {
        serde_json::from_str(include_str!(
            "../../../schemas/industry-risk-assurance-policy-v0.1.json"
        )).unwrap()
    }

    fn select(profile: &Value) -> Value {
        let policy = policy();
        let profile_bytes = serde_json::to_vec(profile).unwrap();
        let policy_bytes = serde_json::to_vec(&policy).unwrap();
        select_requirements(profile, &policy, &sha256_hex(&profile_bytes), &sha256_hex(&policy_bytes)).unwrap()
    }

    fn control_set(report: &Value) -> BTreeSet<String> {
        report["required_controls"].as_array().unwrap().iter()
            .map(|item| item["control_id"].as_str().unwrap().to_owned()).collect()
    }

    #[test]
    fn baseline_is_requirements_only_and_never_qualifies() {
        let report = select(&fixture());
        assert_eq!(report["independent_qualification_attested"], false);
        assert_eq!(report["action_authority"], "none");
        assert_eq!(report["decision_tier"], "D1_advisory");
        assert!(!report["required_controls"].as_array().unwrap().is_empty());
    }

    #[test]
    fn all_six_risk_axes_are_monotone() {
        let sequences: [(&str, &[&str]); 6] = [
            ("consequence", &["negligible","minor","major","severe","catastrophic","unknown"]),
            ("model_influence", &["informational","advisory","constraint_gating","selection","direct_actuation","unknown"]),
            ("exposure", &["single_asset","site_or_facility","multi_site","regional","national_or_larger","unknown"]),
            ("reversibility", &["readily_reversible","costly_or_delayed","difficult_or_irreversible","unknown"]),
            ("detectability", &["immediate_and_reliable","delayed_or_partial","poor","unknown"]),
            ("fallback", &["independent_tested_fallback","human_fallback","degraded_fallback","no_verified_fallback","unknown"]),
        ];
        for (axis, values) in sequences {
            let mut previous = BTreeSet::new();
            for value in values {
                let mut profile = fixture();
                profile["risk_assessment"]["dimensions"][axis] = json!(value);
                let current = control_set(&select(&profile));
                assert!(previous.is_subset(&current), "requirements decreased for {axis}={value}");
                previous = current;
            }
        }
    }

    #[test]
    fn unknown_and_unassessed_risk_cannot_clear() {
        let mut profile = fixture();
        profile["risk_assessment"]["dimensions"]["consequence"] = json!("unknown");
        let report = select(&profile);
        assert_eq!(report["risk_assessment_state"], "contains_unknown");
        assert_eq!(report["requirements_disposition"], "indeterminate");

        profile["risk_assessment"]["reviewed_class"] = json!("unassessed");
        let report = select(&profile);
        assert_eq!(report["risk_assessment_state"], "unassessed");
        assert_eq!(report["requirements_disposition"], "indeterminate");
    }

    #[test]
    fn direct_actuation_cannot_be_used_below_d3() {
        let mut profile = fixture();
        profile["risk_assessment"]["dimensions"]["model_influence"] = json!("direct_actuation");
        profile["decision_tier"] = json!("D2_delegated_planning");
        let report = select(&profile);
        assert_eq!(report["tier_compatibility"], "blocked");
        assert_eq!(report["requirements_disposition"], "indeterminate");
        assert_eq!(report["action_authority"], "none");
    }

    #[test]
    fn compound_risk_triggers_non_compensating_controls() {
        let mut profile = fixture();
        profile["risk_assessment"]["dimensions"]["consequence"] = json!("catastrophic");
        profile["risk_assessment"]["dimensions"]["model_influence"] = json!("direct_actuation");
        profile["risk_assessment"]["dimensions"]["detectability"] = json!("poor");
        profile["risk_assessment"]["dimensions"]["fallback"] = json!("no_verified_fallback");
        let ids = control_set(&select(&profile));
        for required in ["SAFETY","HIL","CASE","DUAL","STOP","SAFESTATE","FAILMON"] {
            assert!(ids.contains(required), "missing compound control {required}");
        }
    }

    #[test]
    fn an_established_dimension_without_evidence_does_not_satisfy_a_control() {
        let mut profile = fixture();
        profile["evidence_vector"]["software_conformance"]["state"] = json!("established");
        profile["evidence_vector"]["software_conformance"]["evidence"] = json!([]);
        let report = select(&profile);
        assert!(report["missing_controls"].as_array().unwrap().iter()
            .any(|item| item.as_str() == Some("SOFT")));
    }

    #[test]
    fn field_order_does_not_change_required_control_ids() {
        let profile = fixture();
        let report_a = select(&profile);
        let mut reordered = profile.clone();
        let reordered_obj = reordered.as_object_mut().unwrap();
        let mut keys: Vec<_> = reordered_obj.keys().cloned().collect();
        keys.sort_by(|a,b| b.cmp(a));
        let values: BTreeMap<_,_> = keys.into_iter().map(|key| (key.clone(), reordered_obj.remove(&key).unwrap())).collect();
        for (key,value) in values { reordered_obj.insert(key,value); }
        let report_b = select(&reordered);
        assert_eq!(report_a["required_controls"], report_b["required_controls"]);
    }

    #[test]
    fn duplicate_json_members_are_rejected() {
        assert!(serde_json::from_str::<StrictJson>(r#"{"x":1,"x":2}"#).is_err());
    }
}
