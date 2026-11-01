use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::process::ExitCode;

const REQUEST_SCHEMA: &str = "luminous.ieac-proof-plan-request.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-proof-plan-report.v0.1";
const AUDIT_SCHEMA: &str = "luminous.ieac-proof-plan-audit-report.v0.1";
const PLANNER_VERSION: &str = "0.1.0";
const EXACT_LIMIT: usize = 20;
const RESOURCE_FIELDS: &[&str] = &[
    "money_microunits",
    "reviewer_minutes",
    "field_interruption_minutes",
    "compute_microunits",
    "duration_minutes",
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
            where E: de::Error { Ok(StrictJson::Null) }
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where E: de::Error { Ok(StrictJson::Null) }
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where E: de::Error { Ok(StrictJson::Bool(value)) }
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where E: de::Error { Ok(StrictJson::Number(Number::from(value))) }
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where E: de::Error { Ok(StrictJson::Number(Number::from(value))) }
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where E: de::Error {
                Number::from_f64(value).map(StrictJson::Number)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: de::Error { Ok(StrictJson::String(value.to_owned())) }
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where E: de::Error { Ok(StrictJson::String(value)) }
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Resources {
    money: u64,
    reviewer: u64,
    field: u64,
    compute: u64,
    duration: u64,
}

impl Resources {
    fn read(value: &Value, context: &str) -> Result<Self, String> {
        let mut result = Self::default();
        for key in RESOURCE_FIELDS {
            let amount = value.get(*key).and_then(Value::as_u64)
                .ok_or_else(|| format!("{context}.{key} must be a non-negative integer"))?;
            match *key {
                "money_microunits" => result.money = amount,
                "reviewer_minutes" => result.reviewer = amount,
                "field_interruption_minutes" => result.field = amount,
                "compute_microunits" => result.compute = amount,
                "duration_minutes" => result.duration = amount,
                _ => return Err(format!("unknown resource field '{key}'")),
            }
        }
        Ok(result)
    }

    fn plus(self, rhs: Self) -> Option<Self> {
        Some(Self {
            money: self.money.checked_add(rhs.money)?,
            reviewer: self.reviewer.checked_add(rhs.reviewer)?,
            field: self.field.checked_add(rhs.field)?,
            compute: self.compute.checked_add(rhs.compute)?,
            duration: self.duration.checked_add(rhs.duration)?,
        })
    }

    fn within(self, rhs: Self) -> bool {
        self.money <= rhs.money
            && self.reviewer <= rhs.reviewer
            && self.field <= rhs.field
            && self.compute <= rhs.compute
            && self.duration <= rhs.duration
    }

    fn as_value(self) -> Value {
        json!({
            "money_microunits": self.money,
            "reviewer_minutes": self.reviewer,
            "field_interruption_minutes": self.field,
            "compute_microunits": self.compute,
            "duration_minutes": self.duration
        })
    }
}

#[derive(Clone, Debug)]
struct Requirement {
    minimum_domains: usize,
    reasons: BTreeSet<String>,
    human_approval: bool,
}

#[derive(Clone, Debug)]
struct Action {
    id: String,
    digest: String,
    mapping_digest: String,
    registry_digest: String,
    rights_digest: String,
    domain: String,
    covers: BTreeSet<String>,
    prerequisites: BTreeSet<String>,
    conflicts: BTreeSet<String>,
    costs: Resources,
    human_approval: bool,
    rights_verified_at: String,
    rights_valid_until: Option<String>,
}

#[derive(Clone, Debug)]
struct ExistingReceipt {
    artifact_digest: String,
    verifier_digest: String,
    rights_digest: String,
    domain: String,
    registry_digest: String,
}

#[derive(Clone, Debug)]
struct Inputs {
    request_id: String,
    request_digest: String,
    report_digest: String,
    profile_digest: String,
    policy_digest: String,
    registry_digest: String,
    mapping_digest: String,
    currency: String,
    compute_unit: String,
    tier: String,
    evaluated_at: String,
    budget: Resources,
    requirements: BTreeMap<String, Requirement>,
    existing_rows: BTreeMap<String, Vec<ExistingReceipt>>,
    existing_domains: BTreeMap<String, BTreeSet<String>>,
    actions: BTreeMap<String, Action>,
    action_order: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Score {
    money: u64,
    field: u64,
    reviewer: u64,
    duration: u64,
    compute: u64,
    count: usize,
    ids: Vec<String>,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha(value: &Value, key: &str, context: &str) -> Result<String, String> {
    let value = value.get(key).and_then(Value::as_str)
        .ok_or_else(|| format!("{context}.{key} is missing"))?;
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(format!("{context}.{key} is not lowercase SHA-256"));
    }
    Ok(value.to_owned())
}

fn string<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str).filter(|text| !text.trim().is_empty())
        .ok_or_else(|| format!("{context}.{key} must be a non-empty string"))
}

fn strings(value: &Value, key: &str, context: &str) -> Result<BTreeSet<String>, String> {
    let rows = value.get(key).and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.{key} must be an array"))?;
    let mut result = BTreeSet::new();
    for row in rows {
        let text = row.as_str().filter(|text| !text.is_empty())
            .ok_or_else(|| format!("{context}.{key} contains a non-string or empty value"))?;
        if !result.insert(text.to_owned()) {
            return Err(format!("{context}.{key} contains duplicate '{text}'"));
        }
    }
    Ok(result)
}

fn array<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a [Value], String> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
        .ok_or_else(|| format!("{context}.{key} must be an array"))
}

fn canonical_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T'
        || bytes[13] != b':' || bytes[16] != b':' || bytes[19] != b'Z'
        || !bytes.iter().enumerate().all(|(index, byte)| {
            [4, 7, 10, 13, 16, 19].contains(&index) || byte.is_ascii_digit()
        })
    {
        return false;
    }
    let part = |range: std::ops::Range<usize>| {
        std::str::from_utf8(&bytes[range]).ok()?.parse::<u32>().ok()
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        part(0..4), part(5..7), part(8..10),
        part(11..13), part(14..16), part(17..19)
    ) else { return false; };
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=max_day).contains(&day)
}

fn timestamp(value: &Value, key: &str, context: &str) -> Result<String, String> {
    let value = string(value, key, context)?;
    if !canonical_utc(value) {
        return Err(format!("{context}.{key} is not a canonical UTC calendar timestamp"));
    }
    Ok(value.to_owned())
}

fn selected_set(report: &Value) -> Result<BTreeSet<String>, String> {
    let mut result = BTreeSet::new();
    for row in array(report, "selected_actions", "report")? {
        let id = string(row, "action_id", "selected action")?;
        if !result.insert(id.to_owned()) {
            return Err(format!("report duplicates selected action '{id}'"));
        }
    }
    Ok(result)
}

fn load_inputs(request: &Value, request_bytes: &[u8], report: &Value) -> Result<Inputs, String> {
    if request.get("schema_version").and_then(Value::as_str) != Some(REQUEST_SCHEMA) {
        return Err("unsupported proof-plan request schema".into());
    }
    if report.get("schema_version").and_then(Value::as_str) != Some(REPORT_SCHEMA) {
        return Err("unsupported proof-plan report schema".into());
    }
    if request.get("action_authority").and_then(Value::as_str) != Some("none")
        || report.get("action_authority").and_then(Value::as_str) != Some("none")
        || report.get("qualification_attested").and_then(Value::as_bool) != Some(false)
    {
        return Err("request/report crosses the no-authority or no-qualification boundary".into());
    }

    let request_id = string(request, "request_id", "request")?.to_owned();
    if string(report, "request_id", "report")? != request_id {
        return Err("report request_id does not match request".into());
    }
    let request_digest = digest(request_bytes);
    if string(report, "request_sha256", "report")? != request_digest {
        return Err("report does not bind the exact request file bytes".into());
    }

    if sha(report, "request_sha256", "report")? != request_digest {
        return Err("report does not bind the exact request file digest".into());
    }
    let report_profile = sha(report, "profile_sha256", "report")?;
    let report_policy = sha(report, "policy_sha256", "report")?;
    let request_profile = sha(request, "profile_sha256", "request")?;
    let request_policy = sha(request, "policy_sha256", "request")?;
    let request_report = sha(request, "risk_control_report_sha256", "request")?;
    let report_control = sha(report, "risk_control_report_sha256", "report")?;
    if report_profile != request_profile || report_policy != request_policy || report_control != request_report {
        return Err("report profile, policy or risk-control receipt binding does not match request".into());
    }
    let registry_digest = sha(request, "independence_registry_sha256", "request")?;
    let mapping_digest = sha(request, "control_mapping_policy_sha256", "request")?;
    if sha(report, "independence_registry_sha256", "report")? != registry_digest
        || sha(report, "control_mapping_policy_sha256", "report")? != mapping_digest
    {
        return Err("report registry/mapping-policy pins do not match request".into());
    }

    if request.get("exact_search_limit").and_then(Value::as_u64) != Some(EXACT_LIMIT as u64) {
        return Err("request exact_search_limit does not match auditor contract".into());
    }
    let tier = string(request, "decision_tier", "request")?.to_owned();
    if !["D0_descriptive", "D1_advisory", "D2_delegated_planning", "D3_autonomous_control"].contains(&tier.as_str()) {
        return Err("unsupported decision tier".into());
    }
    if string(report, "decision_tier", "report")? != tier {
        return Err("report decision tier does not match request".into());
    }
    let evaluated_at = timestamp(request, "evaluated_at", "request")?;
    let currency = string(request, "currency_code", "request")?.to_owned();
    if currency.len() != 3 || !currency.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err("invalid currency_code".into());
    }
    let compute_unit = string(request, "compute_unit_id", "request")?.to_owned();
    if compute_unit.len() > 64 || !compute_unit.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._:-".contains(&byte)) {
        return Err("invalid compute_unit_id".into());
    }
    if string(report, "currency_code", "report")? != currency
        || string(report, "compute_unit_id", "report")? != compute_unit
    {
        return Err("report resource unit identity does not match request".into());
    }

    let mut requirements = BTreeMap::new();
    for row in array(request, "required_controls", "request")? {
        let id = string(row, "control_id", "required control")?.to_owned();
        let reasons = strings(row, "reason_codes", "required control")?;
        let minimum = row.get("minimum_independent_domains").and_then(Value::as_u64)
            .filter(|value| *value > 0 && *value <= 32).ok_or("invalid minimum_independent_domains")? as usize;
        if row.get("mandatory").and_then(Value::as_bool) != Some(true) {
            return Err(format!("required control '{id}' is not mandatory"));
        }
        let human_approval = row.get("human_approval_required").and_then(Value::as_bool)
            .ok_or_else(|| format!("required control '{id}' has no approval flag"))?;
        if requirements.insert(id.clone(), Requirement { minimum_domains: minimum, reasons, human_approval }).is_some() {
            return Err(format!("duplicate required control '{id}'"));
        }
    }
    if requirements.is_empty() {
        return Err("empty mandatory control set".into());
    }

    let mut existing_rows = BTreeMap::<String, Vec<ExistingReceipt>>::new();
    let mut existing_domains = BTreeMap::<String, BTreeSet<String>>::new();
    for row in array(request, "verified_evidence", "request")? {
        let context = "verified evidence";
        let control = string(row, "control_id", context)?.to_owned();
        if !requirements.contains_key(&control) {
            return Err("existing evidence refers to an unrequired control".into());
        }
        let artifact_digest = sha(row, "artifact_sha256", context)?;
        let verifier_digest = sha(row, "verifier_build_sha256", context)?;
        let rights_digest = sha(row, "rights_evidence_sha256", context)?;
        let domain = string(row, "independence_domain_id", context)?.to_owned();
        let row_registry = sha(row, "independence_registry_sha256", context)?;
        if row_registry != registry_digest {
            return Err("existing evidence has a different independence registry pin".into());
        }
        let verified_at = timestamp(row, "verified_at", context)?;
        let valid_until = match row.get("valid_until") {
            Some(Value::Null) => None,
            Some(Value::String(value)) if canonical_utc(value) => Some(value.as_str()),
            _ => return Err("existing evidence has invalid valid_until".into()),
        };
        let verification_status = string(row, "verification_status", context)?;
        let rights_status = string(row, "rights_status", context)?;
        if !["verified", "failed", "unknown", "revoked"].contains(&verification_status)
            || !["verified", "restricted", "unknown", "revoked"].contains(&rights_status)
        {
            return Err("existing evidence contains an unsupported verification/rights state".into());
        }
        let statuses_ok = verification_status == "verified" && rights_status == "verified";
        let current = statuses_ok && verified_at <= evaluated_at
            && valid_until.is_none_or(|until| until >= evaluated_at.as_str());
        if current {
            let rows = existing_rows.entry(control.clone()).or_default();
            if rows.iter().any(|other| other.artifact_digest == artifact_digest && other.domain != domain) {
                return Err("one evidence artifact is claimed by multiple independence domains".into());
            }
            if !rows.iter().any(|other| other.domain == domain) {
                rows.push(ExistingReceipt {
                    artifact_digest,
                    verifier_digest,
                    rights_digest,
                    domain: domain.clone(),
                    registry_digest: row_registry,
                });
            }
            existing_domains.entry(control).or_default().insert(domain);
        }
    }

    let budget = Resources::read(request.get("resource_budgets").ok_or("request.resource_budgets missing")?, "resource_budgets")?;
    let mut actions = BTreeMap::new();
    let mut action_order = Vec::new();
    for row in array(request, "candidate_actions", "request")? {
        let id = string(row, "action_id", "candidate action")?.to_owned();
        let action_digest = sha(row, "action_sha256", &id)?;
        let mapping_policy_digest = sha(row, "mapping_policy_sha256", &id)?;
        let action_registry_digest = sha(row, "independence_registry_sha256", &id)?;
        let rights_digest = sha(row, "rights_evidence_sha256", &id)?;
        if mapping_policy_digest != mapping_digest || action_registry_digest != registry_digest {
            return Err(format!("candidate action '{id}' is outside the pinned registry/mapping policy"));
        }
        let covers = strings(row, "covers_control_ids", &id)?;
        if covers.iter().any(|control| !requirements.contains_key(control)) {
            return Err(format!("candidate action '{id}' covers an unrequired control"));
        }
        let prerequisites = strings(row, "prerequisites", &id)?;
        let conflicts = strings(row, "conflicts_with", &id)?;
        let domain = string(row, "independence_domain_id", &id)?.to_owned();
        let rights_status = string(row, "rights_status", &id)?;
        let rights_verified_at = timestamp(row, "rights_verified_at", &id)?;
        let rights_valid_until = match row.get("rights_valid_until") {
            Some(Value::Null) => None,
            Some(Value::String(value)) if canonical_utc(value) => Some(value.clone()),
            _ => return Err(format!("candidate action '{id}' has invalid rights_valid_until")),
        };
        if rights_status != "verified" || rights_verified_at > evaluated_at
            || rights_valid_until.as_ref().is_some_and(|until| until < &evaluated_at)
        {
            return Err(format!("candidate action '{id}' has no current verified rights"));
        }
        let costs = Resources::read(row.get("resource_costs").ok_or("candidate resource_costs missing")?, &id)?;
        let human_approval = row.get("human_approval_required").and_then(Value::as_bool)
            .ok_or_else(|| format!("candidate action '{id}' has no human approval flag"))?;
        if actions.contains_key(&id) {
            return Err(format!("duplicate candidate action '{id}'"));
        }
        action_order.push(id.clone());
        actions.insert(id.clone(), Action {
            id,
            digest: action_digest,
            mapping_digest: mapping_policy_digest,
            registry_digest: action_registry_digest,
            rights_digest,
            domain,
            covers,
            prerequisites,
            conflicts,
            costs,
            human_approval,
            rights_verified_at,
            rights_valid_until,
        });
    }

    for action in actions.values() {
        for prerequisite in &action.prerequisites {
            if !actions.contains_key(prerequisite) {
                return Err(format!("candidate action '{}' has unknown prerequisite", action.id));
            }
        }
        for conflict in &action.conflicts {
            let other = actions.get(conflict)
                .ok_or_else(|| format!("candidate action '{}' has unknown conflict", action.id))?;
            if !other.conflicts.contains(&action.id) || conflict == &action.id {
                return Err(format!("candidate action '{}' has an asymmetric/invalid conflict", action.id));
            }
        }
    }

    let mut indegree: BTreeMap<String, usize> = actions.keys().map(|id| (id.clone(), 0)).collect();
    let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for action in actions.values() {
        for prerequisite in &action.prerequisites {
            if prerequisite == &action.id || !actions.contains_key(prerequisite) {
                return Err(format!("candidate action '{}' has an invalid prerequisite", action.id));
            }
            *indegree.get_mut(&action.id).ok_or("missing action indegree")? += 1;
            dependents.entry(prerequisite.clone()).or_default().push(action.id.clone());
        }
        for conflict in &action.conflicts {
            let other = actions.get(conflict).ok_or("unknown conflict reference")?;
            if conflict == &action.id || !other.conflicts.contains(&action.id) {
                return Err(format!("candidate action '{}' has an asymmetric conflict", action.id));
            }
        }
    }
    let mut ready: BTreeSet<String> = indegree.iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(id.clone())).collect();
    let mut visited = 0usize;
    while let Some(id) = ready.pop_first() {
        visited += 1;
        if let Some(rows) = dependents.get(&id) {
            for dependent in rows {
                let degree = indegree.get_mut(dependent).ok_or("missing dependent indegree")?;
                *degree -= 1;
                if *degree == 0 { ready.insert(dependent.clone()); }
            }
        }
    }
    if visited != actions.len() {
        return Err("candidate action prerequisite graph contains a cycle".into());
    }

    Ok(Inputs {
        request_id,
        request_digest,
        report_digest: request_report,
        profile_digest: report_profile,
        policy_digest: report_policy,
        registry_digest,
        mapping_digest,
        currency,
        compute_unit,
        tier,
        evaluated_at,
        budget,
        requirements,
        existing_rows,
        existing_domains,
        actions,
        action_order,
    })
}

fn total_cost(selected: &BTreeSet<String>, inputs: &Inputs) -> Result<Resources, String> {
    let mut total = Resources::default();
    for id in selected {
        let action = inputs.actions.get(id).ok_or_else(|| format!("unknown selected action '{id}'"))?;
        total = total.plus(action.costs).ok_or("resource overflow")?;
    }
    Ok(total)
}

fn domains_for(selected: &BTreeSet<String>, inputs: &Inputs) -> BTreeMap<String, BTreeSet<String>> {
    let mut domains = inputs.existing_domains.clone();
    for id in selected {
        if let Some(action) = inputs.actions.get(id) {
            for control in &action.covers {
                domains.entry(control.clone()).or_default().insert(action.domain.clone());
            }
        }
    }
    domains
}

fn gaps(selected: &BTreeSet<String>, inputs: &Inputs) -> Vec<String> {
    let domains = domains_for(selected, inputs);
    inputs.requirements.iter().filter_map(|(control, requirement)| {
        (domains.get(control).map_or(0, BTreeSet::len) < requirement.minimum_domains)
            .then_some(control.clone())
    }).collect()
}

fn conflicts_clear(selected: &BTreeSet<String>, inputs: &Inputs) -> bool {
    selected.iter().all(|id| {
        inputs.actions.get(id).is_some_and(|action| action.conflicts.is_disjoint(selected))
    })
}

fn dependencies_closed(selected: &BTreeSet<String>, inputs: &Inputs) -> bool {
    selected.iter().all(|id| {
        inputs.actions.get(id).is_some_and(|action| action.prerequisites.is_subset(selected))
    })
}

fn feasibility(selected: &BTreeSet<String>, inputs: &Inputs) -> bool {
    conflicts_clear(selected, inputs)
        && dependencies_closed(selected, inputs)
        && total_cost(selected, inputs).is_ok_and(|cost| cost.within(inputs.budget))
        && gaps(selected, inputs).is_empty()
}

fn score(selected: &BTreeSet<String>, inputs: &Inputs) -> Result<Score, String> {
    let cost = total_cost(selected, inputs)?;
    Ok(Score {
        money: cost.money,
        field: cost.field,
        reviewer: cost.reviewer,
        duration: cost.duration,
        compute: cost.compute,
        count: selected.len(),
        ids: selected.iter().cloned().collect(),
    })
}

fn exact_oracle(inputs: &Inputs) -> Result<Option<BTreeSet<String>>, String> {
    if inputs.action_order.len() > EXACT_LIMIT {
        return Err("exact oracle cannot process more than 20 actions".into());
    }
    let mut best: Option<(Score, BTreeSet<String>)> = None;
    let combinations = 1u64 << inputs.action_order.len();
    for mask in 0..combinations {
        let mut selected = BTreeSet::new();
        for (index, id) in inputs.action_order.iter().enumerate() {
            if mask & (1u64 << index) != 0 {
                selected.insert(id.clone());
            }
        }
        if feasibility(&selected, inputs) {
            let candidate_score = score(&selected, inputs)?;
            if best.as_ref().is_none_or(|(best_score, _)| candidate_score < *best_score) {
                best = Some((candidate_score, selected));
            }
        }
    }
    Ok(best.map(|(_, selected)| selected))
}

fn same_string_array(actual: &Value, expected: &BTreeSet<String>, context: &str) -> Result<(), String> {
    let actual_values = strings_value(actual, context)?;
    if &actual_values != expected {
        return Err(format!("{context} does not match independently recomputed values"));
    }
    Ok(())
}

fn strings_value(value: &Value, context: &str) -> Result<BTreeSet<String>, String> {
    let rows = value.as_array().ok_or_else(|| format!("{context} must be an array"))?;
    let mut result = BTreeSet::new();
    for row in rows {
        let text = row.as_str().filter(|text| !text.is_empty())
            .ok_or_else(|| format!("{context} contains an invalid string"))?;
        if !result.insert(text.to_owned()) {
            return Err(format!("{context} contains a duplicate value"));
        }
    }
    Ok(result)
}

fn verify_report(request: &Value, request_bytes: &[u8], report: &Value, report_bytes: &[u8], build_id: &str) -> Result<Value, String> {
    let inputs = load_inputs(request, request_bytes, report)?;
    if report.get("planner_version").and_then(Value::as_str) != Some(PLANNER_VERSION) {
        return Err("unsupported planner version in report".into());
    }
    if string(report, "decision_tier", "report")? != inputs.tier {
        return Err("report decision tier does not match the original request".into());
    }
    let report_build_id = string(report, "planner_build_id", "report")?;
    if report_build_id.is_empty() {
        return Err("missing planner build identity".into());
    }
    let report_actions = selected_set(report)?;
    let report_order: Vec<String> = array(report, "execution_order", "report")?.iter()
        .map(|value| value.as_str().map(str::to_owned).ok_or("execution_order contains non-string"))
        .collect::<Result<_, _>>()?;
    let order_set: BTreeSet<String> = report_order.iter().cloned().collect();
    if report_order.len() != order_set.len() || order_set != report_actions {
        return Err("execution_order is not an exact permutation of selected actions".into());
    }

    let mut position = BTreeMap::new();
    for (index, id) in report_order.iter().enumerate() {
        position.insert(id.clone(), index);
    }
    let mut selected_human_actions = BTreeSet::new();
    for row in array(report, "selected_actions", "report")? {
        let id = string(row, "action_id", "selected action")?;
        let action = inputs.actions.get(id).ok_or_else(|| format!("report selects unknown action '{id}'"))?;
        let rights_valid_until_matches = match (&action.rights_valid_until, row.get("rights_valid_until")) {
            (None, Some(Value::Null)) => true,
            (Some(expected), Some(Value::String(actual))) => expected == actual,
            _ => false,
        };
        if sha(row, "action_sha256", id)? != action.digest
            || sha(row, "mapping_policy_sha256", id)? != action.mapping_digest
            || sha(row, "independence_registry_sha256", id)? != action.registry_digest
            || sha(row, "rights_evidence_sha256", id)? != action.rights_digest
            || string(row, "independence_domain_id", id)? != action.domain
            || string(row, "rights_status", id)? != "verified"
            || timestamp(row, "rights_verified_at", id)? != action.rights_verified_at
            || !rights_valid_until_matches
        {
            return Err(format!("selected action '{id}' differs from the request-bound action record"));
        }
        if row.get("human_approval_required").and_then(Value::as_bool) != Some(action.human_approval) {
            return Err(format!("selected action '{id}' changed its human-approval requirement"));
        }
        if strings(row, "covers_control_ids", id)? != action.covers
            || Resources::read(row.get("resource_costs").ok_or("selected action resource costs missing")?, id)? != action.costs
        {
            return Err(format!("selected action '{id}' changed its coverage or resource costs"));
        }
        let expected_prerequisite = report_actions.iter().any(|other_id| {
            inputs.actions.get(other_id).is_some_and(|other| other.prerequisites.contains(id))
        });
        if row.get("prerequisite").and_then(Value::as_bool) != Some(expected_prerequisite) {
            return Err(format!("selected action '{id}' has incorrect prerequisite marker"));
        }
        if action.human_approval {
            selected_human_actions.insert(id.to_owned());
        }
        for prerequisite in &action.prerequisites {
            if !report_actions.contains(prerequisite) {
                return Err(format!("selected action '{id}' omits prerequisite '{prerequisite}'"));
            }
            if position.get(prerequisite).is_none_or(|pre| *pre >= *position.get(id).unwrap_or(&usize::MAX)) {
                return Err(format!("prerequisite '{prerequisite}' is not before '{id}'"));
            }
        }
    }
    if !conflicts_clear(&report_actions, &inputs) || !dependencies_closed(&report_actions, &inputs) {
        return Err("selected actions violate conflict/dependency constraints".into());
    }

    let calculated_cost = total_cost(&report_actions, &inputs)?;
    if !calculated_cost.within(inputs.budget)
        || Resources::read(report.get("resource_totals").ok_or("report.resource_totals missing")?, "resource_totals")? != calculated_cost
        || Resources::read(report.get("resource_budgets").ok_or("report.resource_budgets missing")?, "resource_budgets")? != inputs.budget
    {
        return Err("report resource totals/budgets do not match independently recomputed values".into());
    }

    let domains = domains_for(&report_actions, &inputs);
    let mut covered = BTreeSet::new();
    let mut unsatisfied = BTreeSet::new();
    let mut control_approvals = BTreeSet::new();
    let mut all_approvals = BTreeSet::new();
    for (control, requirement) in &inputs.requirements {
        let found = domains.get(control).cloned().unwrap_or_default();
        let is_covered = found.len() >= requirement.minimum_domains;
        if is_covered {
            covered.insert(control.clone());
        } else {
            unsatisfied.insert(control.clone());
        }
        let row = array(report, "control_coverage", "report")?.iter()
            .find(|row| row.get("control_id").and_then(Value::as_str) == Some(control.as_str()))
            .ok_or_else(|| format!("report lacks control-coverage row for '{control}'"))?;
        if row.get("minimum_independent_domains").and_then(Value::as_u64) != Some(requirement.minimum_domains as u64)
            || row.get("domain_count").and_then(Value::as_u64) != Some(found.len() as u64)
            || row.get("covered").and_then(Value::as_bool) != Some(is_covered)
            || strings(row, "planned_or_existing_domain_ids", control)? != found
            || strings(row, "reason_codes", control)? != requirement.reasons
        {
            return Err(format!("control coverage row for '{control}' is inconsistent"));
        }
        let expected_existing = inputs.existing_rows.get(control).cloned().unwrap_or_default();
        let existing_report = array(row, "existing_verified_evidence", control)?;
        if existing_report.len() != expected_existing.len() {
            return Err(format!("existing evidence count for '{control}' changed"));
        }
        for receipt in &expected_existing {
            if !existing_report.iter().any(|record| {
                record.get("artifact_sha256").and_then(Value::as_str) == Some(receipt.artifact_digest.as_str())
                    && record.get("verifier_build_sha256").and_then(Value::as_str) == Some(receipt.verifier_digest.as_str())
                    && record.get("rights_evidence_sha256").and_then(Value::as_str) == Some(receipt.rights_digest.as_str())
                    && record.get("independence_domain_id").and_then(Value::as_str) == Some(receipt.domain.as_str())
                    && record.get("independence_registry_sha256").and_then(Value::as_str) == Some(receipt.registry_digest.as_str())
            }) {
                return Err(format!("existing evidence provenance for '{control}' changed"));
            }
        }
        if requirement.human_approval {
            control_approvals.insert(control.clone());
            all_approvals.insert(format!("control:{control}"));
        }
    }
    if array(report, "control_coverage", "report")?.len() != inputs.requirements.len() {
        return Err("report control coverage has duplicate or extra controls".into());
    }
    same_string_array(report.get("covered_controls").ok_or("covered_controls missing")?, &covered, "covered_controls")?;
    same_string_array(report.get("unsatisfied_controls").ok_or("unsatisfied_controls missing")?, &unsatisfied, "unsatisfied_controls")?;
    same_string_array(report.get("human_approval_controls").ok_or("human_approval_controls missing")?, &control_approvals, "human_approval_controls")?;
    same_string_array(report.get("human_approval_actions").ok_or("human_approval_actions missing")?, &selected_human_actions, "human_approval_actions")?;
    for id in &selected_human_actions {
        all_approvals.insert(format!("action:{id}"));
    }
    same_string_array(report.get("human_approvals_required").ok_or("human_approvals_required missing")?, &all_approvals, "human_approvals_required")?;

    let method = string(report, "optimization_method", "report")?;
    let optimality = string(report, "optimality_status", "report")?;
    let status = string(report, "plan_status", "report")?;
    let count = inputs.actions.len();
    let mut optimality_checked = false;
    let verdict = match status {
        "optimal_plan" => {
            if count > EXACT_LIMIT || method != "exact_subset_enumeration" || optimality != "proven_optimal" || !unsatisfied.is_empty() {
                return Err("optimal_plan status is inconsistent with method, limit or coverage".into());
            }
            let best = exact_oracle(&inputs)?;
            if best.as_ref() != Some(&report_actions) {
                return Err("independent exact oracle does not agree with the planner's claimed optimum".into());
            }
            optimality_checked = true;
            "accepted_optimal"
        }
        "infeasible" => {
            if count > EXACT_LIMIT || method != "exact_subset_enumeration" || optimality != "proven_infeasible"
                || !report_actions.is_empty() || exact_oracle(&inputs)?.is_some()
            {
                return Err("infeasible claim is not confirmed by the independent exact oracle".into());
            }
            optimality_checked = true;
            "accepted_infeasible"
        }
        "feasible_heuristic" => {
            if count <= EXACT_LIMIT || method != "deterministic_greedy" || optimality != "not_proven"
                || !unsatisfied.is_empty() || !feasibility(&report_actions, &inputs)
            {
                return Err("heuristic plan is infeasible or mislabeled".into());
            }
            "accepted_heuristic"
        }
        "no_feasible_plan_found" => {
            if count <= EXACT_LIMIT || method != "deterministic_greedy" || optimality != "not_proven"
                || !report_actions.is_empty() || unsatisfied.is_empty()
            {
                return Err("no-feasible-plan-found status is mislabeled".into());
            }
            "accepted_no_plan_found"
        }
        _ => return Err(format!("unsupported plan_status '{status}'")),
    };

    let report_file_digest = digest(report_bytes);
    let summary = json!({
        "schema_version": AUDIT_SCHEMA,
        "request_id": inputs.request_id,
        "request_sha256": inputs.request_digest,
        "plan_report_sha256": report_file_digest,
        "risk_control_report_sha256": inputs.report_digest,
        "profile_sha256": inputs.profile_digest,
        "policy_sha256": inputs.policy_digest,
        "independence_registry_sha256": inputs.registry_digest,
        "control_mapping_policy_sha256": inputs.mapping_digest,
        "planner_version": PLANNER_VERSION,
        "planner_build_id": report_build_id,
        "auditor_version": "0.1.0",
        "auditor_build_id": build_id,
        "plan_status": status,
        "verdict": verdict,
        "selected_action_count": report_actions.len(),
        "candidate_action_count": count,
        "covered_control_count": covered.len(),
        "unsatisfied_control_count": unsatisfied.len(),
        "resources_recomputed": calculated_cost.as_value(),
        "hard_constraints_verified": true,
        "optimality_independently_checked": optimality_checked,
        "action_authority": "none",
        "qualification_attested": false
    });
    Ok(summary)
}

fn read_strict(path: &str) -> Result<(Vec<u8>, Value), String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let parsed: StrictJson = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{path}: invalid or duplicate-member JSON: {error}"))?;
    Ok((bytes, parsed.into_value()))
}

fn run(request_path: &str, report_path: &str, output_path: &str, build_id: &str) -> Result<(), String> {
    let (request_bytes, request) = read_strict(request_path)?;
    let (report_bytes, report) = read_strict(report_path)?;
    let audit = verify_report(&request, &request_bytes, &report, &report_bytes, build_id)?;
    let encoded = serde_json::to_vec_pretty(&audit)
        .map_err(|error| format!("cannot encode audit report: {error}"))?;
    fs::write(output_path, encoded).map_err(|error| format!("cannot write audit report: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: industry-proof-plan-audit <request.json> <plan-report.json> <audit-report.json>");
        return ExitCode::from(4);
    }
    let build_id = env::var("IEAC_PROOF_AUDITOR_BUILD_ID")
        .unwrap_or_else(|_| "local-unattested".to_owned());
    match run(&args[1], &args[2], &args[3], &build_id) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("independent proof-plan audit rejected the report: {error}");
            ExitCode::from(1)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../schemas/examples/ieac-proof-plan-synthetic-v0.1.json"
        )).unwrap()
    }

    fn request() -> Value {
        fixture()["request"].clone()
    }

    fn stub_report(request: &Value, request_bytes: &[u8]) -> Value {
        json!({
            "schema_version": REPORT_SCHEMA,
            "request_id": request["request_id"],
            "request_sha256": digest(request_bytes),
            "risk_control_report_sha256": request["risk_control_report_sha256"],
            "profile_sha256": request["profile_sha256"],
            "policy_sha256": request["policy_sha256"],
            "independence_registry_sha256": request["independence_registry_sha256"],
            "control_mapping_policy_sha256": request["control_mapping_policy_sha256"],
            "decision_tier": request["decision_tier"],
            "currency_code": request["currency_code"],
            "compute_unit_id": request["compute_unit_id"]
        })
    }

    fn inputs_for(request: &Value) -> Inputs {
        let bytes = serde_json::to_vec(request).unwrap();
        let report = stub_report(request, &bytes);
        load_inputs(request, &bytes, &report).unwrap()
    }

    #[test]
    fn independent_exact_oracle_matches_frozen_optimal_action_set() {
        let fixture = fixture();
        let request = &fixture["request"];
        let inputs = inputs_for(request);
        let selected = exact_oracle(&inputs).unwrap().unwrap();
        let expected: BTreeSet<String> = fixture["expected"]["selected_action_ids"]
            .as_array().unwrap().iter()
            .map(|item| item.as_str().unwrap().to_owned()).collect();
        assert_eq!(selected, expected);
    }

    #[test]
    fn independent_exact_oracle_proves_collapsed_domains_infeasible() {
        let mut request = request();
        for index in [2usize, 3, 4] {
            request["candidate_actions"][index]["independence_domain_id"] = json!("one-source-domain");
        }
        let inputs = inputs_for(&request);
        assert_eq!(exact_oracle(&inputs).unwrap(), None);
    }

    #[test]
    fn auditor_rejects_report_not_bound_to_exact_request_bytes() {
        let request = request();
        let bytes = serde_json::to_vec(&request).unwrap();
        let mut report = stub_report(&request, &bytes);
        report["request_sha256"] = json!("0".repeat(64));
        assert!(load_inputs(&request, &bytes, &report).is_err());
    }

    #[test]
    fn auditor_rejects_cyclic_candidate_action_graph() {
        let mut request = request();
        request["candidate_actions"][0]["prerequisites"] = json!(["numeric-reference-run-a"]);
        request["candidate_actions"][2]["prerequisites"] = json!(["artifact-provenance-capture"]);
        let bytes = serde_json::to_vec(&request).unwrap();
        let report = stub_report(&request, &bytes);
        assert!(load_inputs(&request, &bytes, &report).is_err());
    }

    #[test]
    fn duplicate_json_members_are_rejected() {
        assert!(serde_json::from_str::<StrictJson>(r#"{"key":1,"key":2}"#).is_err());
    }

    #[test]
    fn sha256_known_answer() {
        assert_eq!(
            digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
