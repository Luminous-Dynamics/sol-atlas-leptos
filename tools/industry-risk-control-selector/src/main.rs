use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::process::ExitCode;

const POLICY_SCHEMA: &str = "luminous.ieac-risk-control-policy.v0.2";
const PROFILE_SCHEMA: &str = "luminous.industry-assurance-profile.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-risk-control-selection-report.v0.1";
const DIMENSIONS: &[&str] = &[
    "consequence", "model_influence", "exposure", "reversibility", "detectability", "fallback",
];
const EVIDENCE_DIMENSIONS: &[&str] = &[
    "software_conformance", "numerical_verification", "physical_validation",
    "data_integrity_rights", "uncertainty_credibility", "generalization", "robustness",
    "operational_assurance", "safety_security_authority", "prospective_outcomes",
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
                Number::from_f64(value).map(StrictJson::Number)
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
                        return Err(de::Error::custom(format!("duplicate JSON object member: {key}")));
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

fn json_digest(value: &Value) -> Result<String, String> {
    // serde_json uses sorted object keys by default; policy and profile are
    // structurally validated before hashing and contain no floating-point policy values.
    let bytes = serde_json::to_vec(value).map_err(|error| format!("cannot serialize digest input: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn required_string<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str).filter(|item| !item.trim().is_empty())
        .ok_or_else(|| format!("{context}: required non-empty string '{key}' is missing"))
}

fn array<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a [Value], String> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
        .ok_or_else(|| format!("{context}: '{key}' must be an array"))
}

fn add_controls(
    required: &mut BTreeMap<String, BTreeSet<String>>,
    controls: &[Value],
    reason_code: &str,
) -> Result<(), String> {
    for control in controls {
        let id = control.as_str().filter(|value| !value.is_empty())
            .ok_or_else(|| "policy control IDs must be non-empty strings".to_owned())?;
        required.entry(id.to_owned()).or_default().insert(reason_code.to_owned());
    }
    Ok(())
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
        return Err("unsupported IEAC risk-control policy schema".into());
    }
    if policy.get("action_authority").and_then(Value::as_str) != Some("none") {
        return Err("risk-control policy cannot grant action authority".into());
    }

    let mut catalog = BTreeMap::new();
    let controls = policy.get("controls").and_then(Value::as_object)
        .ok_or("policy.controls must be an object")?;
    for (id, definition) in controls {
        let dimension = required_string(definition, "evidence_dimension", id)?;
        if !EVIDENCE_DIMENSIONS.contains(&dimension) {
            return Err(format!("control '{id}' maps to unknown evidence dimension '{dimension}'"));
        }
        required_string(definition, "description", id)?;
        if definition.get("human_approval").and_then(Value::as_bool).is_none() {
            return Err(format!("control '{id}' must declare human_approval"));
        }
        for obligation in array(definition, "obligations", id)? {
            if obligation.as_str().is_none_or(str::is_empty) {
                return Err(format!("control '{id}' contains an invalid obligation"));
            }
        }
        catalog.insert(id.clone(), definition.clone());
    }

    let mut referenced = BTreeSet::new();
    let mut tiers = BTreeSet::new();
    for (tier, rows) in policy.get("tier_controls").and_then(Value::as_object)
        .ok_or("policy.tier_controls must be an object")? {
        if !TIERS.contains(&tier.as_str()) || !tiers.insert(tier.clone()) {
            return Err(format!("invalid or duplicate decision tier '{tier}'"));
        }
        for control in rows.as_array().ok_or("tier control list must be an array")? {
            referenced.insert(control.as_str().ok_or("tier control ID must be a string")?.to_owned());
        }
    }
    if tiers.len() != TIERS.len() {
        return Err("policy must define one tier control set for each D0-D3 tier".into());
    }

    let risk_classes = policy.get("risk_class_controls").and_then(Value::as_object)
        .ok_or("policy.risk_class_controls must be an object")?;
    for class in ["low", "moderate", "high", "critical", "unassessed"] {
        let controls = risk_classes.get(class).and_then(Value::as_array)
            .ok_or_else(|| format!("policy has no risk class control set for '{class}'"))?;
        for control in controls {
            referenced.insert(control.as_str().ok_or("risk class control ID must be a string")?.to_owned());
        }
    }

    let dimensions = policy.get("dimensions").and_then(Value::as_object)
        .ok_or("policy.dimensions must be an object")?;
    for axis in DIMENSIONS {
        let rules = dimensions.get(*axis).and_then(Value::as_object)
            .ok_or_else(|| format!("policy is missing dimension rules for '{axis}'"))?;
        if !rules.contains_key("unknown") {
            return Err(format!("policy must define a conservative unknown value for '{axis}'"));
        }
        for (value, controls) in rules {
            if value.is_empty() {
                return Err(format!("dimension '{axis}' has an empty value rule"));
            }
            for control in controls.as_array()
                .ok_or_else(|| format!("'{axis}={value}' controls must be an array"))? {
                referenced.insert(control.as_str().ok_or("dimension control ID must be a string")?.to_owned());
            }
        }
    }

    for control in array(policy, "always_required_controls", "policy")? {
        referenced.insert(control.as_str().ok_or("baseline control ID must be a string")?.to_owned());
    }
    let unknown = policy.get("unknown_policy").ok_or("policy.unknown_policy is missing")?;
    if unknown.get("dimension_value").and_then(Value::as_str) != Some("unknown")
        || unknown.get("behavior").and_then(Value::as_str)
            != Some("apply_the_maximum_control_set_for_that_dimension_and_mark_indeterminate")
        || unknown.get("conflicted_dimension_behavior").and_then(Value::as_str)
            != Some("same_as_unknown_and_mark_indeterminate")
        || unknown.get("reviewed_class_unassessed_behavior").and_then(Value::as_str)
            != Some("apply_unassessed_controls_and_mark_indeterminate") {
        return Err("unknown/conflicted policy must be worst-case conservative and indeterminate".into());
    }
    for control in array(unknown, "reviewed_class_unassessed_controls", "unknown policy")? {
        referenced.insert(control.as_str().ok_or("unassessed control ID must be a string")?.to_owned());
    }

    let mut combination_ids = BTreeSet::new();
    for rule in array(policy, "combination_rules", "policy")? {
        let id = required_string(rule, "rule_id", "combination rule")?;
        if !combination_ids.insert(id.to_owned()) {
            return Err(format!("duplicate combination rule '{id}'"));
        }
        required_string(rule, "reason_code", "combination rule")?;
        for condition in array(rule, "conditions", "combination rule")? {
            let axis = required_string(condition, "dimension", "combination condition")?;
            if !DIMENSIONS.contains(&axis) {
                return Err(format!("combination rule '{id}' uses unknown dimension '{axis}'"));
            }
            let valid_values = dimensions.get(axis).and_then(Value::as_object)
                .ok_or_else(|| format!("combination rule '{id}' references unknown dimension '{axis}'"))?;
            let mut seen_values = BTreeSet::new();
            for value in array(condition, "values", "combination condition")? {
                let value = value.as_str().filter(|value| !value.is_empty())
                    .ok_or_else(|| format!("combination rule '{id}' contains an empty value"))?;
                if !valid_values.contains_key(value) {
                    return Err(format!("combination rule '{id}' references unmapped value '{axis}={value}'"));
                }
                if !seen_values.insert(value.to_owned()) {
                    return Err(format!("combination rule '{id}' duplicates value '{axis}={value}'"));
                }
            }
        }
        for control in array(rule, "control_ids", "combination rule")? {
            referenced.insert(control.as_str().ok_or("combination control ID must be a string")?.to_owned());
        }
    }

    for id in referenced {
        if !catalog.contains_key(&id) {
            return Err(format!("policy references unknown control ID '{id}'"));
        }
    }
    validate_policy_monotonicity(policy)?;
    Ok(catalog)
}

fn string_set(value: &Value, context: &str) -> Result<BTreeSet<String>, String> {
    let mut values = BTreeSet::new();
    for item in value.as_array().ok_or_else(|| format!("{context} must be an array"))? {
        let text = item.as_str().filter(|value| !value.is_empty())
            .ok_or_else(|| format!("{context} contains a non-string or empty value"))?;
        values.insert(text.to_owned());
    }
    Ok(values)
}

fn require_monotone_superset(
    previous: &BTreeSet<String>,
    current: &BTreeSet<String>,
    context: &str,
) -> Result<(), String> {
    if !previous.is_subset(current) {
        let missing: Vec<_> = previous.difference(current).cloned().collect();
        return Err(format!("{context} is not monotone; later state removed controls: {}", missing.join(", ")));
    }
    Ok(())
}

fn validate_policy_monotonicity(policy: &Value) -> Result<(), String> {
    let dimensions = policy.get("dimensions").and_then(Value::as_object)
        .ok_or("policy.dimensions is missing")?;
    let orders = policy.get("monotonicity_order").and_then(Value::as_object)
        .ok_or("policy.monotonicity_order is missing")?;
    for axis in DIMENSIONS {
        let order = orders.get(*axis).and_then(Value::as_array)
            .ok_or_else(|| format!("policy.monotonicity_order is missing '{axis}'"))?;
        let rules = dimensions.get(*axis).and_then(Value::as_object)
            .ok_or_else(|| format!("policy.dimensions is missing '{axis}'"))?;
        let mut previous: Option<(String, BTreeSet<String>)> = None;
        for value in order {
            let value = value.as_str().ok_or_else(|| format!("monotonicity order for '{axis}' contains a non-string"))?;
            let controls = rules.get(value)
                .ok_or_else(|| format!("monotonicity order for '{axis}' references unmapped value '{value}'"))?;
            let current = string_set(controls, &format!("dimensions.{axis}.{value}"))?;
            if let Some((previous_value, previous_controls)) = &previous {
                require_monotone_superset(previous_controls, &current, &format!("{axis}: {previous_value} -> {value}"))?;
            }
            previous = Some((value.to_owned(), current));
        }
    }

    let class_order = ["low", "moderate", "high", "critical", "unassessed"];
    let class_rules = policy.get("risk_class_controls").and_then(Value::as_object)
        .ok_or("policy.risk_class_controls is missing")?;
    let mut previous_class: Option<(String, BTreeSet<String>)> = None;
    for class in class_order {
        let controls = class_rules.get(class)
            .ok_or_else(|| format!("policy.risk_class_controls is missing '{class}'"))?;
        let current = string_set(controls, &format!("risk_class_controls.{class}"))?;
        if let Some((previous_name, previous_controls)) = &previous_class {
            require_monotone_superset(previous_controls, &current, &format!("risk class: {previous_name} -> {class}"))?;
        }
        previous_class = Some((class.to_owned(), current));
    }

    let tier_order = ["D0_descriptive", "D1_advisory", "D2_delegated_planning", "D3_autonomous_control"];
    let tier_rules = policy.get("tier_controls").and_then(Value::as_object)
        .ok_or("policy.tier_controls is missing")?;
    let mut previous_tier: Option<(String, BTreeSet<String>)> = None;
    for tier in tier_order {
        let controls = tier_rules.get(tier)
            .ok_or_else(|| format!("policy.tier_controls is missing '{tier}'"))?;
        let current = string_set(controls, &format!("tier_controls.{tier}"))?;
        if let Some((previous_name, previous_controls)) = &previous_tier {
            require_monotone_superset(previous_controls, &current, &format!("decision tier: {previous_name} -> {tier}"))?;
        }
        previous_tier = Some((tier.to_owned(), current));
    }
    Ok(())
}

fn evaluate(profile: &Value, policy: &Value, build_id: &str) -> Result<Value, String> {
    if profile.get("schema_version").and_then(Value::as_str) != Some(PROFILE_SCHEMA) {
        return Err("unsupported IEAC qualification profile schema".into());
    }
    let tier = required_string(profile, "decision_tier", "profile")?;
    if !TIERS.contains(&tier) {
        return Err(format!("unsupported decision tier '{tier}'"));
    }
    let risk = profile.get("risk_assessment").ok_or("profile.risk_assessment is missing")?;
    let reviewed_class = required_string(risk, "reviewed_class", "risk assessment")?;
    let risk_classes = ["low", "moderate", "high", "critical", "unassessed"];
    if !risk_classes.contains(&reviewed_class) {
        return Err(format!("unsupported risk class '{reviewed_class}'"));
    }
    let dimensions = risk.get("dimensions").ok_or("risk_assessment.dimensions is missing")?;
    let catalog = validate_policy_catalog(policy)?;

    let mut required = BTreeMap::<String, BTreeSet<String>>::new();
    let mut risk_has_unknown = false;
    let mut risk_has_conflicted = false;
    let risk_unassessed = reviewed_class == "unassessed";
    let mut risk_values = Map::new();
    let mut lookup_values = BTreeMap::<String, String>::new();

    add_controls(&mut required, array(policy, "always_required_controls", "policy")?, "BASELINE_REQUIRED")?;
    let tier_controls = policy.get("tier_controls").and_then(Value::as_object)
        .and_then(|object| object.get(tier)).and_then(Value::as_array)
        .ok_or_else(|| format!("policy has no control set for tier '{tier}'"))?;
    add_controls(&mut required, tier_controls, &format!("TIER_{}", tier.to_ascii_uppercase()))?;

    let risk_class_controls = policy.get("risk_class_controls").and_then(Value::as_object)
        .and_then(|object| object.get(reviewed_class)).and_then(Value::as_array)
        .ok_or_else(|| format!("policy has no control set for risk class '{reviewed_class}'"))?;
    add_controls(&mut required, risk_class_controls, &format!("RISK_CLASS_{}", reviewed_class.to_ascii_uppercase()))?;

    let dimension_rules = policy.get("dimensions").and_then(Value::as_object)
        .ok_or("policy.dimensions is missing")?;
    for axis in DIMENSIONS {
        let actual = required_string(dimensions, axis, "risk dimensions")?;
        let rules = dimension_rules.get(*axis).and_then(Value::as_object)
            .ok_or_else(|| format!("policy is missing risk axis '{axis}'"))?;
        if !rules.contains_key(actual) && !["unknown", "conflicted"].contains(&actual) {
            return Err(format!("unsupported risk value '{actual}' for '{axis}'"));
        }
        let lookup = if risk_unassessed || actual == "unknown" || actual == "conflicted" {
            "unknown"
        } else {
            actual
        };
        risk_has_unknown |= actual == "unknown";
        risk_has_conflicted |= actual == "conflicted";
        lookup_values.insert((*axis).to_owned(), lookup.to_owned());
        risk_values.insert((*axis).to_owned(), json!(actual));
        let controls = rules.get(lookup).and_then(Value::as_array)
            .ok_or_else(|| format!("policy is missing conservative mapping '{axis}={lookup}'"))?;
        let reason = if risk_unassessed {
            "RISK_CLASS_UNASSESSED_WORST_CASE".to_owned()
        } else if actual == "conflicted" {
            "RISK_DIMENSION_CONFLICTED_WORST_CASE".to_owned()
        } else if actual == "unknown" {
            "RISK_DIMENSION_UNKNOWN_WORST_CASE".to_owned()
        } else {
            format!("RISK_{}_{}", axis, actual).to_ascii_uppercase()
        };
        add_controls(&mut required, controls, &reason)?;
    }

    for rule in array(policy, "combination_rules", "policy")? {
        let mut matches = true;
        for condition in array(rule, "conditions", "combination rule")? {
            let axis = required_string(condition, "dimension", "combination condition")?;
            let current = lookup_values.get(axis)
                .ok_or_else(|| format!("combination rule references unavailable axis '{axis}'"))?;
            let allowed = array(condition, "values", "combination condition")?;
            if !allowed.iter().any(|value| value.as_str() == Some(current.as_str())) {
                matches = false;
                break;
            }
        }
        if matches {
            add_controls(&mut required, array(rule, "control_ids", "combination rule")?,
                required_string(rule, "reason_code", "combination rule")?)?;
        }
    }

    if risk_unassessed {
        add_controls(&mut required, array(policy.get("unknown_policy").ok_or("unknown policy missing")?,
            "reviewed_class_unassessed_controls", "unknown policy")?, "RISK_CLASS_UNASSESSED_CONSERVATIVE")?;
    }

    let model_influence = dimensions.get("model_influence").and_then(Value::as_str).unwrap_or("unknown");
    let tier_compatibility = if ["direct_actuation", "unknown", "conflicted"].contains(&model_influence)
        && tier != "D3_autonomous_control"
    {
        "blocked"
    } else {
        "compatible"
    };

    let mut missing = Vec::new();
    let mut unknown = Vec::new();
    let mut conflicted = Vec::new();
    let mut not_applicable = Vec::new();
    let mut controls_with_evidence = Vec::new();
    let mut output_controls = Vec::new();
    let mut human_approvals = BTreeSet::new();
    let mut operational_obligations = BTreeSet::new();

    for (id, reasons) in &required {
        let definition = catalog.get(id)
            .ok_or_else(|| format!("selected control '{id}' is absent from policy catalog"))?;
        let evidence_dimension = required_string(definition, "evidence_dimension", id)?;
        let description = required_string(definition, "description", id)?;
        let human_approval = definition.get("human_approval").and_then(Value::as_bool)
            .ok_or_else(|| format!("control '{id}' has invalid human_approval"))?;
        let obligations = array(definition, "obligations", id)?;
        if human_approval {
            human_approvals.insert(id.clone());
        }
        let mut obligation_values = Vec::new();
        for obligation in obligations {
            let value = obligation.as_str().ok_or_else(|| format!("control '{id}' has non-string obligation"))?;
            operational_obligations.insert(value.to_owned());
            obligation_values.push(value.to_owned());
        }
        let (state, evidence_count) = control_evidence_state(profile, evidence_dimension);
        match state.as_str() {
            "established" => controls_with_evidence.push(id.clone()),
            "missing" => missing.push(id.clone()),
            "unknown" => unknown.push(id.clone()),
            "conflicted" => conflicted.push(id.clone()),
            "not_applicable" => not_applicable.push(id.clone()),
            _ => unknown.push(id.clone()),
        }
        output_controls.push(json!({
            "control_id": id,
            "evidence_dimension": evidence_dimension,
            "description": description,
            "evidence_state": state,
            "evidence_count": evidence_count,
            "human_approval": human_approval,
            "obligations": obligation_values,
            "reason_codes": reasons.iter().cloned().collect::<Vec<_>>()
        }));
    }

    let indeterminate = tier_compatibility == "blocked" || risk_unassessed || risk_has_unknown
        || risk_has_conflicted || !unknown.is_empty() || !conflicted.is_empty() || !not_applicable.is_empty();
    let disposition = if indeterminate { "indeterminate" }
        else if !missing.is_empty() { "requirements_missing" }
        else { "requirements_satisfied" };

    Ok(json!({
        "schema_version": REPORT_SCHEMA,
        "profile_id": required_string(profile, "profile_id", "profile")?,
        "decision_tier": tier,
        "risk_assessment_state": if risk_unassessed { "unassessed" } else if risk_has_conflicted { "contains_conflicted" } else if risk_has_unknown { "contains_unknown" } else { "reviewed" },
        "risk_assessment_class": reviewed_class,
        "risk_dimensions": risk_values,
        "policy_id": required_string(policy, "policy_id", "policy")?,
        "policy_version": required_string(policy, "policy_version", "policy")?,
        "policy_sha256": json_digest(policy)?,
        "selector_version": "0.2.0",
        "selector_build_id": build_id,
        "input_profile_sha256": json_digest(profile)?,
        "required_controls": output_controls,
        "controls_with_evidence": controls_with_evidence,
        "missing_controls": missing,
        "unknown_controls": unknown,
        "conflicted_controls": conflicted,
        "not_applicable_controls": not_applicable,
        "human_approvals": human_approvals.into_iter().collect::<Vec<_>>(),
        "operational_obligations": operational_obligations.into_iter().collect::<Vec<_>>(),
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

fn run(profile_path: &str, policy_path: &str, output_path: &str, build_id: &str) -> Result<(), String> {
    let (profile_bytes, profile) = read_strict(profile_path)?;
    let (policy_bytes, policy) = read_strict(policy_path)?;
    let mut report = evaluate(&profile, &policy, build_id)?;
    // Hash the exact acquired file bytes, not a parsed/re-serialized semantic value.
    report["input_profile_sha256"] = json!(sha256_hex(&profile_bytes));
    report["policy_sha256"] = json!(sha256_hex(&policy_bytes));
    let encoded = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot encode selector report: {error}"))?;
    fs::write(output_path, encoded).map_err(|error| format!("cannot write selector report: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: industry-risk-control-selector <ieac-profile.json> <risk-policy.json> <report.json>");
        return ExitCode::from(4);
    }
    let build_id = env::var("IEAC_SELECTOR_BUILD_ID").unwrap_or_else(|_| "local-unattested".to_owned());
    match run(&args[1], &args[2], &args[3], &build_id) {
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
        serde_json::from_str(include_str!("../../../schemas/examples/industry-assurance-qualification-profile-synthetic-v0.1.json")).unwrap()
    }

    fn policy() -> Value {
        serde_json::from_str(include_str!("../../../schemas/ieac-risk-control-policy-v0.2.json")).unwrap()
    }

    fn select(profile: &Value) -> Value {
        evaluate(profile, &policy(), "test-build").unwrap()
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
    fn conflicted_risk_uses_worst_case_controls_and_cannot_clear() {
        let mut profile = fixture();
        profile["risk_assessment"]["dimensions"]["consequence"] = json!("conflicted");
        let conflicted = select(&profile);
        profile["risk_assessment"]["dimensions"]["consequence"] = json!("unknown");
        let unknown = select(&profile);
        assert_eq!(control_set(&conflicted), control_set(&unknown));
        assert_eq!(conflicted["requirements_disposition"], "indeterminate");
    }

    #[test]
    fn risk_class_controls_are_monotone() {
        let classes = ["low", "moderate", "high", "critical", "unassessed"];
        let mut previous = BTreeSet::new();
        for class in classes {
            let mut profile = fixture();
            profile["risk_assessment"]["reviewed_class"] = json!(class);
            let current = control_set(&select(&profile));
            assert!(previous.is_subset(&current), "requirements decreased for reviewed_class={class}");
            previous = current;
        }
    }

    #[test]
    fn tier_minimum_controls_are_monotone_and_do_not_authorize() {
        let tiers = ["D0_descriptive", "D1_advisory", "D2_delegated_planning", "D3_autonomous_control"];
        let mut previous = BTreeSet::new();
        for tier in tiers {
            let mut profile = fixture();
            profile["decision_tier"] = json!(tier);
            let report = select(&profile);
            let current = control_set(&report);
            assert!(previous.is_subset(&current), "requirements decreased for tier={tier}");
            assert_eq!(report["decision_tier"], tier);
            assert_eq!(report["action_authority"], "none");
            previous = current;
        }
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
    fn unknown_or_conflicted_model_influence_blocks_below_d3() {
        for value in ["unknown", "conflicted"] {
            let mut profile = fixture();
            profile["risk_assessment"]["dimensions"]["model_influence"] = json!(value);
            profile["decision_tier"] = json!("D2_delegated_planning");
            let report = select(&profile);
            assert_eq!(report["tier_compatibility"], "blocked", "value={value}");
            assert_eq!(report["requirements_disposition"], "indeterminate", "value={value}");
            assert_eq!(report["action_authority"], "none");
        }
    }

    #[test]
    fn compound_risk_triggers_non_compensating_controls() {
        let mut profile = fixture();
        profile["risk_assessment"]["dimensions"]["consequence"] = json!("catastrophic");
        profile["risk_assessment"]["dimensions"]["model_influence"] = json!("direct_actuation");
        profile["risk_assessment"]["dimensions"]["detectability"] = json!("poor");
        profile["risk_assessment"]["dimensions"]["fallback"] = json!("no_verified_fallback");
        let ids = control_set(&select(&profile));
        for required in [
            "independent_safety_assessment", "fail_safe_emergency_stop", "dual_control_authorization",
            "independent_monitoring", "redundant_measurement", "incident_response_playbook", "tested_safe_stop",
        ] {
            assert!(ids.contains(required), "missing compound control {required}");
        }
    }

    #[test]
    fn established_dimension_without_evidence_does_not_satisfy_a_control() {
        let mut profile = fixture();
        profile["evidence_vector"]["software_conformance"]["state"] = json!("established");
        profile["evidence_vector"]["software_conformance"]["evidence"] = json!([]);
        let report = select(&profile);
        assert!(report["missing_controls"].as_array().unwrap().iter()
            .any(|item| item.as_str() == Some("artifact_identity_integrity")));
    }

    #[test]
    fn conflicted_evidence_cannot_become_established() {
        let mut profile = fixture();
        profile["evidence_vector"]["safety_security_authority"]["state"] = json!("conflicted");
        let report = select(&profile);
        assert_eq!(report["requirements_disposition"], "indeterminate");
        assert!(!report["conflicted_controls"].as_array().unwrap().is_empty());
    }

    #[test]
    fn field_order_does_not_change_required_control_ids() {
        let profile = fixture();
        let report_a = select(&profile);
        let mut reordered = profile.clone();
        let object = reordered.as_object_mut().unwrap();
        let keys: Vec<_> = object.keys().cloned().collect();
        for key in keys {
            let value = object.remove(&key).unwrap();
            object.insert(key, value);
        }
        let report_b = select(&reordered);
        assert_eq!(report_a["required_controls"], report_b["required_controls"]);
        assert_eq!(report_a["input_profile_sha256"], report_b["input_profile_sha256"]);
    }

    #[test]
    fn frozen_50_case_truth_table_matches_exact_control_sets() {
        let truth: Value = serde_json::from_str(include_str!(
            "../../../schemas/examples/ieac-risk-control-truth-table-synthetic-v0.2.json"
        )).unwrap();
        assert_eq!(truth["fixture_schema_version"], "luminous.ieac-risk-control-truth-table.v0.2");
        assert_eq!(truth["fixture_status"], "synthetic_only");
        let cases = truth["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 50);
        assert_eq!(truth["case_count"], 50);

        let mut case_ids = BTreeSet::new();
        for case in cases {
            let case_id = case["case_id"].as_str().expect("case ID must be a string");
            assert!(case_ids.insert(case_id.to_owned()), "duplicate truth-table case {case_id}");

            let mut profile = fixture();
            profile["decision_tier"] = case["decision_tier"].clone();
            profile["risk_assessment"]["reviewed_class"] = case["reviewed_class"].clone();
            profile["risk_assessment"]["dimensions"] = case["risk_dimensions"].clone();

            let report = select(&profile);
            let expected: BTreeSet<String> = case["expected_control_ids"]
                .as_array().unwrap().iter()
                .map(|value| value.as_str().expect("expected control ID").to_owned())
                .collect();
            assert_eq!(
                control_set(&report),
                expected,
                "control-set mismatch for frozen truth-table case {case_id}"
            );
            assert_eq!(
                report["risk_assessment_state"],
                case["expected_risk_assessment_state"],
                "risk state mismatch for frozen truth-table case {case_id}"
            );
            assert_eq!(report["action_authority"], "none");
            assert_eq!(report["independent_qualification_attested"], false);
        }
    }

    #[test]
    fn cli_report_binds_exact_input_and_policy_file_bytes() {
        let suffix = std::process::id();
        let root = std::env::temp_dir();
        let profile_path = root.join(format!("ieac-risk-profile-{suffix}.json"));
        let policy_path = root.join(format!("ieac-risk-policy-{suffix}.json"));
        let output_path = root.join(format!("ieac-risk-report-{suffix}.json"));

        let profile_bytes = serde_json::to_vec(&fixture()).unwrap();
        let policy_bytes = serde_json::to_vec(&policy()).unwrap();
        fs::write(&profile_path, &profile_bytes).unwrap();
        fs::write(&policy_path, &policy_bytes).unwrap();

        run(
            profile_path.to_str().unwrap(),
            policy_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            "test-build",
        ).unwrap();

        let report: Value = serde_json::from_slice(&fs::read(&output_path).unwrap()).unwrap();
        assert_eq!(report["input_profile_sha256"], sha256_hex(&profile_bytes));
        assert_eq!(report["policy_sha256"], sha256_hex(&policy_bytes));

        let mut whitespace_variant = b" \n".to_vec();
        whitespace_variant.extend_from_slice(&profile_bytes);
        fs::write(&profile_path, &whitespace_variant).unwrap();
        run(
            profile_path.to_str().unwrap(),
            policy_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            "test-build",
        ).unwrap();
        let second: Value = serde_json::from_slice(&fs::read(&output_path).unwrap()).unwrap();
        assert_eq!(second["input_profile_sha256"], sha256_hex(&whitespace_variant));
        assert_ne!(second["input_profile_sha256"], report["input_profile_sha256"]);

        let _ = fs::remove_file(profile_path);
        let _ = fs::remove_file(policy_path);
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn policy_rejects_unmapped_combination_values() {
        let mut policy = policy();
        policy["combination_rules"][0]["conditions"][0]["values"] = json!(["catastrophe"]);
        assert!(validate_policy_catalog(&policy).is_err());
    }

    #[test]
    fn policy_rejects_non_monotone_dimension_risk_class_or_tier_rules() {
        let mut bad_fallback = policy();
        bad_fallback["dimensions"]["fallback"]["human_fallback"] = json!([]);
        assert!(validate_policy_catalog(&bad_fallback).is_err());

        let mut bad_class = policy();
        bad_class["risk_class_controls"]["unassessed"] = json!([]);
        assert!(validate_policy_catalog(&bad_class).is_err());

        let mut bad_tier = policy();
        bad_tier["tier_controls"]["D3_autonomous_control"] = json!([]);
        assert!(validate_policy_catalog(&bad_tier).is_err());
    }

    #[test]
    fn policy_rejects_unknown_control_references() {
        let mut policy = policy();
        policy["always_required_controls"][0] = json!("not-a-real-control");
        assert!(validate_policy_catalog(&policy).is_err());
    }

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn duplicate_json_members_are_rejected() {
        assert!(serde_json::from_str::<StrictJson>(r#"{"x":1,"x":2}"#).is_err());
    }
}
