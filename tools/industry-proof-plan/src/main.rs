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
const PLANNER_VERSION: &str = "0.1.0";
const EXACT_LIMIT: usize = 20;
const OBJECTIVE_ORDER: &[&str] = &[
    "money_microunits",
    "field_interruption_minutes",
    "reviewer_minutes",
    "duration_minutes",
    "compute_microunits",
    "action_count",
    "lexicographic_action_ids",
];
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
            where
                E: de::Error,
            {
                Ok(StrictJson::Null)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Null)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Number(Number::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Number(Number::from(value)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Number::from_f64(value)
                    .map(StrictJson::Number)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::String(value))
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element::<StrictJson>()? {
                    values.push(value);
                }
                Ok(StrictJson::Array(values))
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Resources {
    money_microunits: u64,
    reviewer_minutes: u64,
    field_interruption_minutes: u64,
    compute_microunits: u64,
    duration_minutes: u64,
}

impl Resources {
    fn from_value(value: &Value, context: &str) -> Result<Self, String> {
        let mut result = Self::default();
        for field in RESOURCE_FIELDS {
            let amount = value.get(*field).and_then(Value::as_u64)
                .ok_or_else(|| format!("{context}.{field} must be a non-negative integer"))?;
            match *field {
                "money_microunits" => result.money_microunits = amount,
                "reviewer_minutes" => result.reviewer_minutes = amount,
                "field_interruption_minutes" => result.field_interruption_minutes = amount,
                "compute_microunits" => result.compute_microunits = amount,
                "duration_minutes" => result.duration_minutes = amount,
                _ => return Err(format!("unsupported resource field '{field}'")),
            }
        }
        Ok(result)
    }

    fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            money_microunits: self.money_microunits.checked_add(other.money_microunits)?,
            reviewer_minutes: self.reviewer_minutes.checked_add(other.reviewer_minutes)?,
            field_interruption_minutes: self.field_interruption_minutes.checked_add(other.field_interruption_minutes)?,
            compute_microunits: self.compute_microunits.checked_add(other.compute_microunits)?,
            duration_minutes: self.duration_minutes.checked_add(other.duration_minutes)?,
        })
    }

    fn within(self, budget: Self) -> bool {
        self.money_microunits <= budget.money_microunits
            && self.reviewer_minutes <= budget.reviewer_minutes
            && self.field_interruption_minutes <= budget.field_interruption_minutes
            && self.compute_microunits <= budget.compute_microunits
            && self.duration_minutes <= budget.duration_minutes
    }

    fn as_value(self) -> Value {
        json!({
            "money_microunits": self.money_microunits,
            "reviewer_minutes": self.reviewer_minutes,
            "field_interruption_minutes": self.field_interruption_minutes,
            "compute_microunits": self.compute_microunits,
            "duration_minutes": self.duration_minutes
        })
    }

    fn score(self, action_ids: &BTreeSet<String>) -> PlanScore {
        PlanScore {
            money_microunits: self.money_microunits,
            field_interruption_minutes: self.field_interruption_minutes,
            reviewer_minutes: self.reviewer_minutes,
            duration_minutes: self.duration_minutes,
            compute_microunits: self.compute_microunits,
            action_count: action_ids.len(),
            action_ids: action_ids.iter().cloned().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PlanScore {
    money_microunits: u64,
    field_interruption_minutes: u64,
    reviewer_minutes: u64,
    duration_minutes: u64,
    compute_microunits: u64,
    action_count: usize,
    action_ids: Vec<String>,
}

#[derive(Clone, Debug)]
struct Action {
    id: String,
    artifact_sha256: String,
    independence_domain_id: String,
    covers: BTreeSet<String>,
    prerequisites: BTreeSet<String>,
    conflicts: BTreeSet<String>,
    resources: Resources,
    human_approval_required: bool,
    mapping_policy_sha256: String,
    independence_registry_sha256: String,
    rights_status: String,
    rights_verified_at: String,
    rights_valid_until: Option<String>,
}

#[derive(Clone, Debug)]
struct Requirement {
    minimum_domains: usize,
    reason_codes: Vec<String>,
}

#[derive(Clone, Debug)]
struct ExistingEvidence {
    domain_id: String,
    artifact_sha256: String,
    verifier_build_sha256: String,
    independence_registry_sha256: String,
}

#[derive(Clone, Debug)]
struct ValidatedRequest {
    request_id: String,
    risk_control_report_sha256: String,
    profile_sha256: String,
    policy_sha256: String,
    decision_tier: String,
    evaluated_at: String,
    requirements: BTreeMap<String, Requirement>,
    existing: BTreeMap<String, BTreeMap<String, ExistingEvidence>>,
    actions: BTreeMap<String, Action>,
    action_order: Vec<String>,
    budget: Resources,
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn required_string<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str).filter(|item| !item.trim().is_empty())
        .ok_or_else(|| format!("{context}: required non-empty string '{key}' is missing"))
}

fn array<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a [Value], String> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
        .ok_or_else(|| format!("{context}.{key} must be an array"))
}

fn unique_string_set(values: &[Value], context: &str) -> Result<BTreeSet<String>, String> {
    let mut output = BTreeSet::new();
    for item in values {
        let value = item.as_str().filter(|value| !value.is_empty())
            .ok_or_else(|| format!("{context} contains a non-string or empty value"))?;
        if !output.insert(value.to_owned()) {
            return Err(format!("{context} contains duplicate value '{value}'"));
        }
    }
    Ok(output)
}

fn parse_digest(value: &Value, key: &str, context: &str) -> Result<String, String> {
    let digest = required_string(value, key, context)?;
    if !valid_sha256(digest) {
        return Err(format!("{context}.{key} must be lowercase SHA-256"));
    }
    Ok(digest.to_owned())
}

fn parse_timestamp(value: &Value, key: &str, context: &str) -> Result<String, String> {
    let timestamp = required_string(value, key, context)?;
    if timestamp.len() != 20 || !timestamp.ends_with('Z') {
        return Err(format!("{context}.{key} must be normalized UTC YYYY-MM-DDTHH:MM:SSZ"));
    }
    Ok(timestamp.to_owned())
}

fn validate_prerequisite_dag(actions: &BTreeMap<String, Action>) -> Result<(), String> {
    let mut indegree: BTreeMap<String, usize> = actions.keys().map(|id| (id.clone(), 0)).collect();
    let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (id, action) in actions {
        for prerequisite in &action.prerequisites {
            if prerequisite == id {
                return Err(format!("action '{id}' cannot require itself"));
            }
            if !actions.contains_key(prerequisite) {
                return Err(format!("action '{id}' has unknown prerequisite '{prerequisite}'"));
            }
            *indegree.get_mut(id).expect("action indegree exists") += 1;
            dependents.entry(prerequisite.clone()).or_default().push(id.clone());
        }
        for conflict in &action.conflicts {
            if conflict == id || !actions.contains_key(conflict) {
                return Err(format!("action '{id}' has invalid conflict reference '{conflict}'"));
            }
            if !actions.get(conflict).is_some_and(|other| other.conflicts.contains(id)) {
                return Err(format!("action conflict '{id}' <-> '{conflict}' must be symmetric"));
            }
        }
    }

    let mut ready: BTreeSet<String> = indegree.iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(id.clone()))
        .collect();
    let mut visited = 0usize;
    while let Some(id) = ready.pop_first() {
        visited += 1;
        if let Some(rows) = dependents.get(&id) {
            for dependent in rows {
                let degree = indegree.get_mut(dependent).expect("dependent indegree exists");
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(dependent.clone());
                }
            }
        }
    }
    if visited != actions.len() {
        return Err("candidate action prerequisite graph contains a cycle".into());
    }
    Ok(())
}

fn validate_request(value: &Value) -> Result<ValidatedRequest, String> {
    if value.get("schema_version").and_then(Value::as_str) != Some(REQUEST_SCHEMA) {
        return Err("unsupported IEAC proof-plan request schema".into());
    }
    if value.get("action_authority").and_then(Value::as_str) != Some("none") {
        return Err("proof planner request cannot grant action authority".into());
    }
    if value.get("exact_search_limit").and_then(Value::as_u64) != Some(EXACT_LIMIT as u64) {
        return Err(format!("exact_search_limit must be {EXACT_LIMIT}"));
    }

    let request_id = required_string(value, "request_id", "request")?.to_owned();
    let risk_control_report_sha256 = parse_digest(value, "risk_control_report_sha256", "request")?;
    let profile_sha256 = parse_digest(value, "profile_sha256", "request")?;
    let policy_sha256 = parse_digest(value, "policy_sha256", "request")?;
    let decision_tier = required_string(value, "decision_tier", "request")?.to_owned();
    let evaluated_at = parse_timestamp(value, "evaluated_at", "request")?;
    let budget = Resources::from_value(value.get("resource_budgets").ok_or("resource_budgets missing")?, "resource_budgets")?;

    let mut requirements = BTreeMap::new();
    for (index, row) in array(value, "required_controls", "request")?.iter().enumerate() {
        let context = format!("required_controls[{index}]");
        let id = required_string(row, "control_id", &context)?.to_owned();
        let reason_codes: Vec<String> = array(row, "reason_codes", &context)?.iter()
            .map(|item| item.as_str().filter(|v| !v.is_empty()).map(str::to_owned)
                .ok_or_else(|| format!("{context}.reason_codes contains an invalid value")))
            .collect::<Result<_, _>>()?;
        if reason_codes.is_empty() {
            return Err(format!("{context} must include at least one trigger reason"));
        }
        let minimum_domains = row.get("minimum_independent_domains").and_then(Value::as_u64)
            .filter(|value| *value > 0 && *value <= 32)
            .ok_or_else(|| format!("{context}.minimum_independent_domains must be 1..=32"))? as usize;
        if row.get("mandatory").and_then(Value::as_bool) != Some(true) {
            return Err(format!("{context}: every selected control must remain mandatory"));
        }
        if requirements.insert(id.clone(), Requirement { minimum_domains, reason_codes }).is_some() {
            return Err(format!("duplicate required control '{id}'"));
        }
    }
    if requirements.is_empty() {
        return Err("proof plan requires at least one mandatory control".into());
    }

    let mut existing = BTreeMap::<String, BTreeMap<String, ExistingEvidence>>::new();
    for (index, row) in array(value, "verified_evidence", "request")?.iter().enumerate() {
        let context = format!("verified_evidence[{index}]");
        let control = required_string(row, "control_id", &context)?.to_owned();
        if !requirements.contains_key(&control) {
            return Err(format!("{context} refers to a control not in the fixed requirement set"));
        }
        let artifact = parse_digest(row, "artifact_sha256", &context)?;
        let verifier_build_sha256 = parse_digest(row, "verifier_build_sha256", &context)?;
        let domain = required_string(row, "independence_domain_id", &context)?.to_owned();
        let independence_registry_sha256 = parse_digest(row, "independence_registry_sha256", &context)?;
        let verified_at = parse_timestamp(row, "verified_at", &context)?;
        let valid_until = match row.get("valid_until") {
            Some(Value::Null) => None,
            Some(Value::String(value)) => Some(value.as_str()),
            _ => return Err(format!("{context}.valid_until must be a UTC timestamp or null")),
        };
        if row.get("verification_status").and_then(Value::as_str)
            .is_none_or(|value| !["verified", "failed", "unknown", "revoked"].contains(&value))
            || row.get("rights_status").and_then(Value::as_str)
                .is_none_or(|value| !["verified", "restricted", "unknown", "revoked"].contains(&value))
        {
            return Err(format!("{context} has an unsupported verification or rights state"));
        }
        let valid_time = valid_until.is_none_or(|until| until >= evaluated_at.as_str());
        let current = row.get("verification_status").and_then(Value::as_str) == Some("verified")
            && row.get("rights_status").and_then(Value::as_str) == Some("verified")
            && verified_at <= evaluated_at
            && valid_time;

        if current {
            existing.entry(control.clone()).or_default().entry(domain.clone()).or_insert(
                ExistingEvidence {
                    domain_id: domain,
                    artifact_sha256: artifact,
                    verifier_build_sha256,
                    independence_registry_sha256,
                }
            );
        }
    }

    let mut actions = BTreeMap::new();
    let mut action_order = Vec::new();
    for (index, row) in array(value, "candidate_actions", "request")?.iter().enumerate() {
        let context = format!("candidate_actions[{index}]");
        let id = required_string(row, "action_id", &context)?.to_owned();
        let artifact_sha256 = parse_digest(row, "action_sha256", &context)?;
        let mapping_policy_sha256 = parse_digest(row, "mapping_policy_sha256", &context)?;
        let covers = unique_string_set(array(row, "covers_control_ids", &context)?, &format!("{context}.covers_control_ids"))?;
        for control in &covers {
            if !requirements.contains_key(control) {
                return Err(format!("action '{id}' claims unmapped or unrequired control '{control}'"));
            }
        }
        let prerequisites = unique_string_set(array(row, "prerequisites", &context)?, &format!("{context}.prerequisites"))?;
        let conflicts = unique_string_set(array(row, "conflicts_with", &context)?, &format!("{context}.conflicts_with"))?;
        let independence_domain_id = required_string(row, "independence_domain_id", &context)?.to_owned();
        let independence_registry_sha256 = parse_digest(row, "independence_registry_sha256", &context)?;
        let rights_status = required_string(row, "rights_status", &context)?;
        if rights_status != "verified" {
            return Err(format!("{context}: candidate action rights must be verified before planning"));
        }
        let rights_verified_at = parse_timestamp(row, "rights_verified_at", &context)?;
        let rights_valid_until = match row.get("rights_valid_until") {
            Some(Value::Null) => None,
            Some(Value::String(value)) => Some(value.clone()),
            _ => return Err(format!("{context}.rights_valid_until must be a UTC timestamp or null")),
        };
        if rights_verified_at > evaluated_at
            || rights_valid_until.as_ref().is_some_and(|until| until < &evaluated_at)
        {
            return Err(format!("{context}: candidate action rights are not current at the planning time"));
        }
        let resources = Resources::from_value(row.get("resource_costs").ok_or("resource_costs missing")?, &format!("{context}.resource_costs"))?;
        let human_approval_required = row.get("human_approval_required").and_then(Value::as_bool)
            .ok_or_else(|| format!("{context}.human_approval_required must be boolean"))?;
        if actions.contains_key(&id) {
            return Err(format!("duplicate candidate action '{id}'"));
        }
        action_order.push(id.clone());
        actions.insert(id.clone(), Action {
            id, artifact_sha256, independence_domain_id, covers,
            prerequisites, conflicts, resources, human_approval_required,
            mapping_policy_sha256, independence_registry_sha256,
            rights_status: rights_status.to_owned(), rights_verified_at, rights_valid_until,
        });
    }
    if actions.is_empty() {
        return Err("proof plan has no candidate evidence actions".into());
    }
    validate_prerequisite_dag(&actions)?;

    Ok(ValidatedRequest {
        request_id, risk_control_report_sha256, profile_sha256, policy_sha256,
        decision_tier, evaluated_at, requirements, existing, actions,
        action_order, budget,
    })
}

fn action_closure(
    id: &str,
    actions: &BTreeMap<String, Action>,
    selected: &mut BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
) -> Result<(), String> {
    if selected.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id.to_owned()) {
        return Err(format!("dependency cycle reached while expanding action '{id}'"));
    }
    let action = actions.get(id).ok_or_else(|| format!("unknown action '{id}'"))?;
    for prerequisite in &action.prerequisites {
        action_closure(prerequisite, actions, selected, visiting)?;
    }
    visiting.remove(id);
    selected.insert(id.to_owned());
    Ok(())
}

fn selected_resources(selected: &BTreeSet<String>, actions: &BTreeMap<String, Action>) -> Result<Resources, String> {
    let mut total = Resources::default();
    for id in selected {
        let action = actions.get(id).ok_or_else(|| format!("selected action '{id}' is not in candidate set"))?;
        total = total.checked_add(action.resources)
            .ok_or_else(|| "resource total overflow".to_owned())?;
    }
    Ok(total)
}

fn selected_conflict_free(selected: &BTreeSet<String>, actions: &BTreeMap<String, Action>) -> bool {
    selected.iter().all(|id| {
        actions.get(id).is_some_and(|action| action.conflicts.is_disjoint(selected))
    })
}

fn control_domains(
    selected: &BTreeSet<String>,
    request: &ValidatedRequest,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut domains: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (control, rows) in &request.existing {
        domains.entry(control.clone()).or_default().extend(rows.keys().cloned());
    }
    for id in selected {
        if let Some(action) = request.actions.get(id) {
            for control in &action.covers {
                domains.entry(control.clone()).or_default().insert(action.independence_domain_id.clone());
            }
        }
    }
    domains
}

fn total_domain_deficit(selected: &BTreeSet<String>, request: &ValidatedRequest) -> usize {
    let domains = control_domains(selected, request);
    request.requirements.iter().map(|(control, requirement)| {
        let count = domains.get(control).map_or(0, BTreeSet::len);
        requirement.minimum_domains.saturating_sub(count)
    }).sum()
}

fn unsatisfied_controls(selected: &BTreeSet<String>, request: &ValidatedRequest) -> Vec<String> {
    let domains = control_domains(selected, request);
    request.requirements.iter().filter_map(|(control, requirement)| {
        let count = domains.get(control).map_or(0, BTreeSet::len);
        (count < requirement.minimum_domains).then_some(control.clone())
    }).collect()
}

fn selection_valid(selected: &BTreeSet<String>, request: &ValidatedRequest) -> bool {
    if !selected_conflict_free(selected, &request.actions) {
        return false;
    }
    if selected.iter().any(|id| {
        request.actions.get(id).is_none_or(|action| !action.prerequisites.is_subset(selected))
    }) {
        return false;
    }
    selected_resources(selected, &request.actions).is_ok_and(|resources| resources.within(request.budget))
        && unsatisfied_controls(selected, request).is_empty()
}

fn plan_score(selected: &BTreeSet<String>, request: &ValidatedRequest) -> Result<PlanScore, String> {
    Ok(selected_resources(selected, &request.actions)?.score(selected))
}

fn exact_plan(request: &ValidatedRequest) -> Result<Option<BTreeSet<String>>, String> {
    let n = request.action_order.len();
    if n > EXACT_LIMIT {
        return Err("exact planner received more actions than the configured limit".into());
    }
    let combinations = 1u64 << n;
    let mut best: Option<(PlanScore, BTreeSet<String>)> = None;
    for mask in 0..combinations {
        let mut selected = BTreeSet::new();
        for (index, id) in request.action_order.iter().enumerate() {
            if (mask & (1u64 << index)) != 0 {
                selected.insert(id.clone());
            }
        }
        if !selection_valid(&selected, request) {
            continue;
        }
        let score = plan_score(&selected, request)?;
        let replace = best.as_ref().is_none_or(|(best_score, _)| score < *best_score);
        if replace {
            best = Some((score, selected));
        }
    }
    Ok(best.map(|(_, selected)| selected))
}

fn greedy_plan(request: &ValidatedRequest) -> Option<BTreeSet<String>> {
    let mut selected = BTreeSet::new();
    while !unsatisfied_controls(&selected, request).is_empty() {
        let before = total_domain_deficit(&selected, request);
        let mut best: Option<(usize, PlanScore, BTreeSet<String>)> = None;

        for id in &request.action_order {
            if selected.contains(id) {
                continue;
            }
            let mut candidate = selected.clone();
            if action_closure(id, &request.actions, &mut candidate, &mut BTreeSet::new()).is_err() {
                continue;
            }
            if !selected_conflict_free(&candidate, &request.actions) {
                continue;
            }
            let Some(resources) = selected_resources(&candidate, &request.actions).ok() else {
                continue;
            };
            if !resources.within(request.budget) {
                continue;
            }
            let after = total_domain_deficit(&candidate, request);
            let gain = before.saturating_sub(after);
            if gain == 0 {
                continue;
            }
            let score = resources.score(&candidate);
            let replace = match &best {
                None => true,
                Some((best_gain, best_score, _)) => gain > *best_gain
                    || (gain == *best_gain && score < *best_score),
            };
            if replace {
                best = Some((gain, score, candidate));
            }
        }

        let (_, _, next) = best?;
        selected = next;
    }
    selection_valid(&selected, request).then_some(selected)
}

fn topological_selected_order(
    selected: &BTreeSet<String>,
    actions: &BTreeMap<String, Action>,
) -> Result<Vec<String>, String> {
    let mut indegree: BTreeMap<String, usize> =
        selected.iter().map(|id| (id.clone(), 0usize)).collect();
    let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for id in selected {
        let action = actions.get(id).ok_or_else(|| format!("selected action '{id}' is missing"))?;
        for prerequisite in &action.prerequisites {
            if selected.contains(prerequisite) {
                *indegree.get_mut(id).ok_or_else(|| "missing selected action indegree".to_owned())? += 1;
                dependents.entry(prerequisite.clone()).or_default().push(id.clone());
            }
        }
    }
    let mut ready: BTreeSet<String> = indegree.iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(id.clone()))
        .collect();
    let mut order = Vec::with_capacity(selected.len());
    while let Some(id) = ready.pop_first() {
        order.push(id.clone());
        if let Some(rows) = dependents.get(&id) {
            for dependent in rows {
                let degree = indegree.get_mut(dependent)
                    .ok_or_else(|| "missing dependent action indegree".to_owned())?;
                *degree = degree.saturating_sub(1);
                if *degree == 0 {
                    ready.insert(dependent.clone());
                }
            }
        }
    }
    if order.len() != selected.len() {
        return Err("selected action subgraph contains a dependency cycle".into());
    }
    Ok(order)
}

fn report_for(
    raw_request: &[u8],
    request: &ValidatedRequest,
    selected: &BTreeSet<String>,
    method: &str,
    optimality: &str,
    status: &str,
    build_id: &str,
) -> Result<Value, String> {
    let resources = selected_resources(selected, &request.actions)?;
    let domains = control_domains(selected, request);
    let execution_order = topological_selected_order(selected, &request.actions)?;
    let mut selected_actions = Vec::new();
    let mut human_approvals = BTreeSet::new();

    for id in &execution_order {
        let action = request.actions.get(id).ok_or_else(|| format!("selected action '{id}' disappeared"))?;
        let prerequisite = selected.iter().any(|other| {
            request.actions.get(other).is_some_and(|candidate| candidate.prerequisites.contains(id))
        });
        if action.human_approval_required {
            human_approvals.insert(id.clone());
        }
        selected_actions.push(json!({
            "action_id": action.id,
            "action_sha256": action.artifact_sha256,
            "prerequisite": prerequisite,
            "human_approval_required": action.human_approval_required,
            "independence_domain_id": action.independence_domain_id,
            "independence_registry_sha256": action.independence_registry_sha256,
            "mapping_policy_sha256": action.mapping_policy_sha256,
            "rights_status": action.rights_status,
            "rights_verified_at": action.rights_verified_at,
            "rights_valid_until": action.rights_valid_until,
            "covers_control_ids": action.covers,
            "resource_costs": action.resources.as_value()
        }));
    }

    let mut covered = Vec::new();
    let mut unsatisfied = Vec::new();
    let mut coverage = Vec::new();
    for (control, requirement) in &request.requirements {
        let present = domains.get(control).cloned().unwrap_or_default();
        let satisfied = present.len() >= requirement.minimum_domains;
        if satisfied {
            covered.push(control.clone());
        } else {
            unsatisfied.push(control.clone());
        }
        let existing_evidence = request.existing.get(control).map(|rows| rows.values().map(|row| json!({
            "artifact_sha256": row.artifact_sha256,
            "verifier_build_sha256": row.verifier_build_sha256,
            "independence_domain_id": row.domain_id,
            "independence_registry_sha256": row.independence_registry_sha256
        })).collect::<Vec<_>>()).unwrap_or_default();
        coverage.push(json!({
            "control_id": control,
            "existing_verified_evidence": existing_evidence,
            "minimum_independent_domains": requirement.minimum_domains,
            "planned_or_existing_domain_ids": present,
            "domain_count": present.len(),
            "covered": satisfied,
            "reason_codes": requirement.reason_codes
        }));
    }

    Ok(json!({
        "schema_version": REPORT_SCHEMA,
        "request_id": request.request_id,
        "request_sha256": sha256_hex(raw_request),
        "risk_control_report_sha256": request.risk_control_report_sha256,
        "profile_sha256": request.profile_sha256,
        "policy_sha256": request.policy_sha256,
        "planner_version": PLANNER_VERSION,
        "planner_build_id": build_id,
        "optimization_method": method,
        "optimality_status": optimality,
        "plan_status": status,
        "selected_actions": selected_actions,
        "execution_order": execution_order,
        "control_coverage": coverage,
        "covered_controls": covered,
        "unsatisfied_controls": unsatisfied,
        "human_approvals_required": human_approvals,
        "resource_totals": resources.as_value(),
        "resource_budgets": request.budget.as_value(),
        "objective_order": OBJECTIVE_ORDER,
        "action_authority": "none",
        "qualification_attested": false
    }))
}

fn evaluate(request_value: &Value, request_bytes: &[u8], build_id: &str) -> Result<Value, String> {
    let request = validate_request(request_value)?;
    let action_count = request.actions.len();

    let (selected, method, optimality, status) = if action_count <= EXACT_LIMIT {
        match exact_plan(&request)? {
            Some(selected) => (selected, "exact_subset_enumeration", "proven_optimal", "optimal_plan"),
            None => (BTreeSet::new(), "exact_subset_enumeration", "proven_infeasible", "infeasible"),
        }
    } else {
        match greedy_plan(&request) {
            Some(selected) => (selected, "deterministic_greedy", "not_proven", "feasible_heuristic"),
            None => (BTreeSet::new(), "deterministic_greedy", "not_proven", "no_feasible_plan_found"),
        }
    };

    let report = report_for(
        request_bytes, &request, &selected, method, optimality, status, build_id
    )?;
    Ok(report)
}

fn read_strict(path: &str) -> Result<(Vec<u8>, Value), String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let parsed: StrictJson = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{path}: invalid or duplicate-member JSON: {error}"))?;
    Ok((bytes, parsed.into_value()))
}

fn run(request_path: &str, output_path: &str, build_id: &str) -> Result<(), String> {
    let (bytes, request) = read_strict(request_path)?;
    let report = evaluate(&request, &bytes, build_id)?;
    let encoded = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot encode plan report: {error}"))?;
    fs::write(output_path, encoded).map_err(|error| format!("cannot write plan report: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: industry-proof-plan <request.json> <plan-report.json>");
        return ExitCode::from(4);
    }
    let build_id = env::var("IEAC_PROOF_PLANNER_BUILD_ID")
        .unwrap_or_else(|_| "local-unattested".to_owned());
    match run(&args[1], &args[2], &build_id) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("proof planning failed closed: {error}");
            ExitCode::from(2)
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

    fn validate() -> ValidatedRequest {
        validate_request(&request()).unwrap()
    }

    fn test_report(value: &Value) -> Value {
        let bytes = serde_json::to_vec(value).unwrap();
        evaluate(value, &bytes, "test-build").unwrap()
    }

    #[test]
    fn bounded_exact_fixture_replays_frozen_optimal_plan() {
        let fixture = fixture();
        let request = &fixture["request"];
        let bytes = serde_json::to_vec(request).unwrap();
        let report = evaluate(request, &bytes, "test-build").unwrap();
        assert_eq!(report["plan_status"], "optimal_plan");
        assert_eq!(report["optimality_status"], "proven_optimal");
        let expected = &fixture["expected"];
        let selected: Vec<String> = report["selected_actions"].as_array().unwrap().iter()
            .map(|row| row["action_id"].as_str().unwrap().to_owned()).collect();
        assert_eq!(selected, expected["selected_action_ids"].as_array().unwrap().iter()
            .map(|value| value.as_str().unwrap().to_owned()).collect::<Vec<_>>());
        assert_eq!(report["resource_totals"], expected["resource_totals"]);
        assert_eq!(report["covered_controls"], expected["covered_controls"]);
        assert_eq!(report["unsatisfied_controls"], expected["unsatisfied_controls"]);
        assert_eq!(report["action_authority"], "none");
        assert_eq!(report["qualification_attested"], false);
    }

    #[test]
    fn prerequisite_action_is_included_and_marked() {
        let report = test_report(&request());
        let reservation = report["selected_actions"].as_array().unwrap().iter()
            .find(|row| row["action_id"] == "physical-lab-reservation").unwrap();
        assert_eq!(reservation["prerequisite"], true);
        let order = report["execution_order"].as_array().unwrap();
        let reservation_index = order.iter().position(|id| id == "physical-lab-reservation").unwrap();
        let lab_index = order.iter().position(|id| id == "physical-calibration-lab").unwrap();
        assert!(reservation_index < lab_index);
    }

    #[test]
    fn minimum_independent_domains_are_not_replaced_by_action_count() {
        let mut request = request();
        for index in [2usize, 3, 4] {
            request["candidate_actions"][index]["independence_domain_id"] = json!("solver-team-a");
        }
        let report = test_report(&request);
        assert_eq!(report["plan_status"], "infeasible");
        assert_eq!(report["optimality_status"], "proven_infeasible");
        assert!(report["unsatisfied_controls"].as_array().unwrap().iter()
            .any(|control| control == "numerical_known_answers"));
    }

    #[test]
    fn large_greedy_plan_counts_domain_deficit_not_control_count() {
        let mut request = request();
        let base = request["candidate_actions"].as_array().unwrap().clone();
        for index in 0..10 {
            let mut row = base[0].clone();
            row["action_id"] = json!(format!("unused-scale-action-{index:02}"));
            row["action_sha256"] = json!(format!("{:064x}", index + 700));
            row["covers_control_ids"] = json!([]);
            row["prerequisites"] = json!([]);
            row["conflicts_with"] = json!([]);
            request["candidate_actions"].as_array_mut().unwrap().push(row);
        }
        let report = test_report(&request);
        assert_eq!(report["plan_status"], "feasible_heuristic");
        assert_eq!(report["optimality_status"], "not_proven");
        assert!(report["covered_controls"].as_array().unwrap().iter()
            .any(|control| control == "numerical_known_answers"));
        assert_eq!(report["action_authority"], "none");
    }

    #[test]
    fn expired_or_rights_revoked_existing_evidence_does_not_satisfy_controls() {
        let mut request = request();
        request["verified_evidence"][0]["rights_status"] = json!("revoked");
        let report = test_report(&request);
        assert!(report["unsatisfied_controls"].as_array().unwrap().iter()
            .any(|control| control == "source_rights_lineage"));
    }

    #[test]
    fn candidate_action_rights_must_be_current_at_planning_time() {
        let mut request = request();
        request["candidate_actions"][0]["rights_valid_until"] = json!("2026-10-31T23:59:59Z");
        assert!(validate_request(&request).is_err());

        request = request();
        request["candidate_actions"][0]["rights_verified_at"] = json!("2026-11-02T00:00:00Z");
        assert!(validate_request(&request).is_err());
    }

    #[test]
    fn report_preserves_existing_evidence_and_action_mapping_identity() {
        let report = test_report(&request());
        let row = report["control_coverage"].as_array().unwrap().iter()
            .find(|item| item["control_id"] == "source_rights_lineage").unwrap();
        assert_eq!(row["existing_verified_evidence"][0]["artifact_sha256"], "4444444444444444444444444444444444444444444444444444444444444444");
        let action = report["selected_actions"].as_array().unwrap().iter()
            .find(|item| item["action_id"] == "artifact-provenance-capture").unwrap();
        assert_eq!(action["mapping_policy_sha256"], "8888888888888888888888888888888888888888888888888888888888888888");
        assert_eq!(action["rights_status"], "verified");
    }

    #[test]
    fn prerequisite_cycles_are_rejected() {
        let mut request = request();
        request["candidate_actions"][0]["prerequisites"] = json!(["numeric-reference-run-a"]);
        request["candidate_actions"][2]["prerequisites"] = json!(["artifact-provenance-capture"]);
        assert!(validate_request(&request).is_err());
    }

    #[test]
    fn asymmetric_conflict_declarations_are_rejected() {
        let mut request = request();
        request["candidate_actions"][0]["conflicts_with"] = json!(["artifact-provenance-full-audit"]);
        request["candidate_actions"][1]["conflicts_with"] = json!([]);
        assert!(validate_request(&request).is_err());
    }

    #[test]
    fn unknown_control_mapping_is_rejected() {
        let mut request = request();
        request["candidate_actions"][0]["covers_control_ids"] = json!(["not-required"]);
        assert!(validate_request(&request).is_err());
    }

    #[test]
    fn resource_overrun_is_not_accepted() {
        let mut request = request();
        request["resource_budgets"]["money_microunits"] = json!(189);
        let report = test_report(&request);
        assert_ne!(report["plan_status"], "optimal_plan");
    }

    #[test]
    fn exact_solver_proves_infeasibility_when_independent_domain_is_unavailable() {
        let mut request = request();
        request["candidate_actions"][3]["independence_domain_id"] = json!("solver-team-a");
        request["candidate_actions"][4]["independence_domain_id"] = json!("solver-team-a");
        request["candidate_actions"][4]["conflicts_with"] = json!(["numeric-reference-run-b"]);
        request["candidate_actions"][3]["conflicts_with"] = json!(["numeric-reference-run-c"]);
        let report = test_report(&request);
        assert_eq!(report["plan_status"], "infeasible");
        assert_eq!(report["optimality_status"], "proven_infeasible");
    }

    #[test]
    fn large_action_set_uses_heuristic_and_never_claims_optimality() {
        let mut request = request();
        let base = request["candidate_actions"].as_array().unwrap().clone();
        for index in 0..10 {
            let mut row = base[index % base.len()].clone();
            row["action_id"] = json!(format!("scaled-action-{index:02}"));
            row["action_sha256"] = json!(format!("{:064x}", index + 100));
            row["covers_control_ids"] = json!([]);
            row["prerequisites"] = json!([]);
            row["conflicts_with"] = json!([]);
            request["candidate_actions"].as_array_mut().unwrap().push(row);
        }
        let report = test_report(&request);
        assert_eq!(report["optimization_method"], "deterministic_greedy");
        assert_eq!(report["optimality_status"], "not_proven");
    }

    #[test]
    fn duplicate_json_members_are_rejected() {
        assert!(serde_json::from_str::<StrictJson>(r#"{"a":1,"a":2}"#).is_err());
    }

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
