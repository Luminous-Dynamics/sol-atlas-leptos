use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::process::ExitCode;

const REQUEST_SCHEMA: &str = "luminous.ieac-common-cause-request.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-common-cause-report.v0.1";
const REGISTRY_SCHEMA: &str = "luminous.ieac-independence-registry.v0.1";
const EXACT_LIMIT: usize = 20;

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

#[derive(Clone, Debug)]
struct Domain {
    organization_id: String,
    status: String,
    dependency_completeness: String,
    dependency_ids: BTreeSet<String>,
    common_cause_group_ids: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct DependencyNode {
    status: String,
    common_cause_group_ids: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct Source {
    id: String,
    control_id: String,
    source_kind: String,
    identity_sha256: String,
    domain_id: String,
    evidence_state: String,
    rights_status: String,
    rights_evidence_sha256: String,
    verified_at: String,
    valid_until: Option<String>,
}

#[derive(Clone, Debug)]
struct ControlRequirement {
    minimum_independent_sources: usize,
}

#[derive(Clone, Debug)]
struct SourceResolution {
    source_id: String,
    domain_id: String,
    organization_id: String,
    source_kind: String,
    source_identity_sha256: String,
    evidence_state: String,
    rights_status: String,
    rights_evidence_sha256: String,
    verified_at: String,
    valid_until: Option<String>,
    dependency_completeness: String,
    dependency_closure: BTreeSet<String>,
    common_cause_group_ids: BTreeSet<String>,
    registry_model_status: String,
}

#[derive(Clone, Debug, Default)]
struct Conflict {
    reason_codes: BTreeSet<String>,
    shared_dependency_ids: BTreeSet<String>,
    shared_common_cause_group_ids: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct RegistryResolution {
    registry_id: String,
    registry_version: String,
    authority_id: String,
    registry_sha256: String,
    domains: BTreeMap<String, Domain>,
    resolved_domains: BTreeMap<String, SourceResolution>,
}

#[derive(Clone, Debug)]
struct Inputs {
    request_id: String,
    request_sha256: String,
    evaluated_at: String,
    registry: RegistryResolution,
    requirements: BTreeMap<String, ControlRequirement>,
    sources: BTreeMap<String, Source>,
    source_order: Vec<String>,
    conflicts: BTreeMap<(String, String), Conflict>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn parse_digest(value: &Value, key: &str, context: &str) -> Result<String, String> {
    let digest = value.get(key).and_then(Value::as_str)
        .ok_or_else(|| format!("{context}.{key} must be a lowercase SHA-256 string"))?;
    if !valid_digest(digest) {
        return Err(format!("{context}.{key} must be a lowercase SHA-256 string"));
    }
    Ok(digest.to_owned())
}

fn required_string<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str).filter(|item| !item.trim().is_empty())
        .ok_or_else(|| format!("{context}.{key} must be a non-empty string"))
}

fn parse_string_set(value: &Value, key: &str, context: &str) -> Result<BTreeSet<String>, String> {
    let rows = value.get(key).and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.{key} must be an array"))?;
    let mut result = BTreeSet::new();
    for row in rows {
        let item = row.as_str().filter(|item| !item.is_empty())
            .ok_or_else(|| format!("{context}.{key} contains an invalid value"))?;
        if !result.insert(item.to_owned()) {
            return Err(format!("{context}.{key} contains duplicate '{item}'"));
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

fn parse_timestamp(value: &Value, key: &str, context: &str) -> Result<String, String> {
    let text = required_string(value, key, context)?;
    if !canonical_utc(text) {
        return Err(format!("{context}.{key} must be a real UTC calendar value"));
    }
    Ok(text.to_owned())
}

fn parse_optional_timestamp(value: &Value, key: &str, context: &str) -> Result<Option<String>, String> {
    match value.get(key) {
        Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if canonical_utc(text) => Ok(Some(text.clone())),
        _ => Err(format!("{context}.{key} must be a UTC calendar value or null")),
    }
}

fn checked_current(
    valid_from: &str,
    valid_until: Option<&str>,
    evaluated_at: &str,
    context: &str,
) -> Result<(), String> {
    if valid_from > evaluated_at || valid_until.is_some_and(|until| until < evaluated_at) {
        return Err(format!("{context} is not current at the request evaluation time"));
    }
    Ok(())
}

fn expand_dependency(
    id: &str,
    nodes: &BTreeMap<String, DependencyNode>,
    edges: &BTreeMap<String, BTreeSet<String>>,
    visiting: &mut BTreeSet<String>,
    memo: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeSet<String>, String> {
    if let Some(cached) = memo.get(id) {
        return Ok(cached.clone());
    }
    if !visiting.insert(id.to_owned()) {
        return Err(format!("dependency graph contains a cycle through '{id}'"));
    }
    let node = nodes.get(id).ok_or_else(|| format!("unknown dependency node '{id}'"))?;
    if node.status != "active" {
        return Err(format!("dependency node '{id}' is revoked"));
    }
    let mut closure = BTreeSet::from([id.to_owned()]);
    if let Some(parents) = edges.get(id) {
        for parent in parents {
            closure.extend(expand_dependency(parent, nodes, edges, visiting, memo)?);
        }
    }
    visiting.remove(id);
    memo.insert(id.to_owned(), closure.clone());
    Ok(closure)
}

fn build_registry(
    registry: &Value,
    expected_digest: &str,
    evaluated_at: &str,
) -> Result<RegistryResolution, String> {
    if registry.get("schema_version").and_then(Value::as_str) != Some(REGISTRY_SCHEMA) {
        return Err("unsupported independence registry schema".into());
    }
    let bytes = serde_json::to_vec(registry)
        .map_err(|error| format!("cannot serialize registry: {error}"))?;
    let actual_digest = sha256_hex(&bytes);
    if actual_digest != expected_digest {
        return Err("independence registry content does not match pinned digest".into());
    }

    let registry_id = required_string(registry, "registry_id", "registry")?.to_owned();
    let registry_version = required_string(registry, "registry_version", "registry")?.to_owned();
    let authority_id = required_string(registry, "authority_id", "registry")?.to_owned();
    let issued_at = parse_timestamp(registry, "issued_at", "registry")?;
    let valid_from = parse_timestamp(registry, "valid_from", "registry")?;
    let valid_until = parse_optional_timestamp(registry, "valid_until", "registry")?;
    if issued_at > evaluated_at {
        return Err("registry was issued after request evaluation time".into());
    }
    checked_current(&valid_from, valid_until.as_deref(), evaluated_at, "registry")?;

    let mut groups = BTreeMap::new();
    for (index, row) in array(registry, "common_cause_groups", "registry")?.iter().enumerate() {
        let context = format!("common_cause_groups[{index}]");
        let id = required_string(row, "group_id", &context)?.to_owned();
        let cause_type = required_string(row, "cause_type", &context)?.to_owned();
        if !["upstream_data_feed", "calibration_lineage", "reference_standard", "software_build",
            "test_harness", "operator_process", "facility", "power_supply", "communications",
            "shared_environment", "maintenance_process", "other"].contains(&cause_type.as_str())
        {
            return Err(format!("{context} has an unsupported cause_type"));
        }
        if groups.insert(id.clone(), cause_type).is_some() {
            return Err(format!("duplicate common-cause group '{id}'"));
        }
    }

    let mut nodes = BTreeMap::new();
    for (index, row) in array(registry, "dependency_nodes", "registry")?.iter().enumerate() {
        let context = format!("dependency_nodes[{index}]");
        let id = required_string(row, "dependency_id", &context)?.to_owned();
        let dependency_type = required_string(row, "dependency_type", &context)?;
        if !["upstream_data_feed", "calibration_chain", "reference_standard", "software_build",
            "test_harness", "operator_process", "facility", "power_supply", "communications",
            "shared_environment", "maintenance_process", "other"].contains(&dependency_type)
        {
            return Err(format!("{context} has an unsupported dependency_type"));
        }
        let status = required_string(row, "status", &context)?.to_owned();
        if !["active", "revoked"].contains(&status.as_str()) {
            return Err(format!("{context} has unsupported status"));
        }
        let group_ids = parse_string_set(row, "common_cause_group_ids", &context)?;
        if group_ids.iter().any(|group| !groups.contains_key(group)) {
            return Err(format!("{context} refers to an unknown common-cause group"));
        }
        if nodes.insert(id.clone(), DependencyNode {
            status, common_cause_group_ids: group_ids,
        }).is_some() {
            return Err(format!("duplicate dependency node '{id}'"));
        }
    }

    let mut edges = BTreeMap::<String, BTreeSet<String>>::new();
    let mut edge_pairs = BTreeSet::new();
    for (index, row) in array(registry, "dependency_edges", "registry")?.iter().enumerate() {
        let context = format!("dependency_edges[{index}]");
        let from = required_string(row, "dependency_id", &context)?.to_owned();
        let to = required_string(row, "depends_on_dependency_id", &context)?.to_owned();
        if from == to || !nodes.contains_key(&from) || !nodes.contains_key(&to) {
            return Err(format!("{context} has a self-edge or unknown endpoint"));
        }
        if !edge_pairs.insert((from.clone(), to.clone())) {
            return Err(format!("{context} duplicates an edge"));
        }
        edges.entry(from).or_default().insert(to);
    }

    let mut domains = BTreeMap::new();
    for (index, row) in array(registry, "domains", "registry")?.iter().enumerate() {
        let context = format!("domains[{index}]");
        let id = required_string(row, "domain_id", &context)?.to_owned();
        let organization_id = required_string(row, "organization_id", &context)?.to_owned();
        let status = required_string(row, "status", &context)?.to_owned();
        let dependency_completeness = required_string(row, "dependency_completeness", &context)?.to_owned();
        if !["active", "revoked"].contains(&status.as_str())
            || !["complete", "unknown", "disputed"].contains(&dependency_completeness.as_str())
        {
            return Err(format!("{context} has unsupported status/completeness"));
        }
        let dependency_ids = parse_string_set(row, "dependency_ids", &context)?;
        if dependency_ids.iter().any(|dependency| !nodes.contains_key(dependency)) {
            return Err(format!("{context} references an unknown dependency node"));
        }
        let common_cause_group_ids = parse_string_set(row, "common_cause_group_ids", &context)?;
        if common_cause_group_ids.iter().any(|group| !groups.contains_key(group)) {
            return Err(format!("{context} references an unknown common-cause group"));
        }
        let valid_from = parse_timestamp(row, "valid_from", &context)?;
        let valid_until = parse_optional_timestamp(row, "valid_until", &context)?;
        checked_current(&valid_from, valid_until.as_deref(), evaluated_at, &context)?;
        if domains.insert(id.clone(), Domain {
            organization_id, status, dependency_completeness,
            dependency_ids, common_cause_group_ids,
        }).is_some() {
            return Err(format!("duplicate source domain '{id}'"));
        }
    }

    let mut memo = BTreeMap::new();
    // Validate every active dependency node, including unreferenced nodes, for cycles/revocation.
    for id in nodes.keys() {
        if nodes[id].status == "active" {
            let _ = expand_dependency(id, &nodes, &edges, &mut BTreeSet::new(), &mut memo)?;
        }
    }
    let mut resolved_domains = BTreeMap::new();
    for (id, domain) in &domains {
        if domain.status != "active" {
            continue;
        }
        let mut closure = BTreeSet::new();
        let mut groups_for_domain = domain.common_cause_group_ids.clone();
        for dependency in &domain.dependency_ids {
            closure.extend(expand_dependency(
                dependency, &nodes, &edges, &mut BTreeSet::new(), &mut memo
            )?);
        }
        for dependency in &closure {
            let node = nodes.get(dependency).ok_or_else(|| format!("missing dependency node '{dependency}'"))?;
            groups_for_domain.extend(node.common_cause_group_ids.iter().cloned());
        }
        let registry_model_status = match domain.dependency_completeness.as_str() {
            "complete" => "resolved",
            "unknown" => "unknown_dependency",
            "disputed" => "disputed_dependency",
            _ => return Err("unsupported dependency completeness".into()),
        };
        resolved_domains.insert(id.clone(), SourceResolution {
            source_id: String::new(),
            domain_id: id.clone(),
            organization_id: domain.organization_id.clone(),
            source_kind: String::new(),
            source_identity_sha256: String::new(),
            evidence_state: String::new(),
            rights_status: String::new(),
            rights_evidence_sha256: String::new(),
            verified_at: String::new(),
            valid_until: None,
            dependency_completeness: domain.dependency_completeness.clone(),
            dependency_closure: closure,
            common_cause_group_ids: groups_for_domain,
            registry_model_status: registry_model_status.into(),
        });
    }

    Ok(RegistryResolution {
        registry_id, registry_version, authority_id, registry_sha256: actual_digest,
        domains, resolved_domains,
    })
}

fn make_conflict(left: &SourceResolution, right: &SourceResolution, same_identity: bool, same_domain: bool) -> Option<Conflict> {
    let shared_dependencies: BTreeSet<String> =
        left.dependency_closure.intersection(&right.dependency_closure).cloned().collect();
    let shared_groups: BTreeSet<String> =
        left.common_cause_group_ids.intersection(&right.common_cause_group_ids).cloned().collect();
    let unknown = left.dependency_completeness != "complete"
        || right.dependency_completeness != "complete";
    let mut reasons = BTreeSet::new();
    if same_identity {
        reasons.insert("DUPLICATE_SOURCE_IDENTITY".to_owned());
    }
    if same_domain {
        reasons.insert("SAME_SOURCE_DOMAIN".to_owned());
    }
    if !shared_dependencies.is_empty() {
        reasons.insert("SHARED_TRANSITIVE_DEPENDENCY".to_owned());
    }
    if !shared_groups.is_empty() {
        reasons.insert("SHARED_COMMON_CAUSE_GROUP".to_owned());
    }
    if unknown {
        reasons.insert("UNKNOWN_OR_DISPUTED_DEPENDENCY_CLOSURE".to_owned());
    }
    if reasons.is_empty() {
        None
    } else {
        Some(Conflict {
            reason_codes: reasons,
            shared_dependency_ids: shared_dependencies,
            shared_common_cause_group_ids: shared_groups,
        })
    }
}

fn validate_request(request: &Value, request_bytes: &[u8]) -> Result<Inputs, String> {
    if request.get("schema_version").and_then(Value::as_str) != Some(REQUEST_SCHEMA) {
        return Err("unsupported common-cause request schema".into());
    }
    if request.get("action_authority").and_then(Value::as_str) != Some("none") {
        return Err("common-cause auditor cannot grant action authority".into());
    }
    if request.get("exact_source_limit").and_then(Value::as_u64) != Some(EXACT_LIMIT as u64) {
        return Err(format!("exact_source_limit must equal {EXACT_LIMIT}"));
    }
    let request_id = required_string(request, "request_id", "request")?.to_owned();
    let evaluated_at = parse_timestamp(request, "evaluated_at", "request")?;
    let registry_sha256 = parse_digest(request, "registry_sha256", "request")?;
    let registry = build_registry(
        request.get("registry").ok_or("request.registry is missing")?,
        &registry_sha256,
        &evaluated_at,
    )?;

    let mut requirements = BTreeMap::new();
    for (index, row) in array(request, "control_requirements", "request")?.iter().enumerate() {
        let context = format!("control_requirements[{index}]");
        let control = required_string(row, "control_id", &context)?.to_owned();
        let minimum = row.get("minimum_independent_sources").and_then(Value::as_u64)
            .filter(|value| *value > 0 && *value <= 32)
            .ok_or_else(|| format!("{context}.minimum_independent_sources must be 1..=32"))? as usize;
        if requirements.insert(control.clone(), ControlRequirement {
            minimum_independent_sources: minimum,
        }).is_some() {
            return Err(format!("duplicate control requirement '{control}'"));
        }
    }
    if requirements.is_empty() {
        return Err("common-cause evaluation requires at least one control requirement".into());
    }

    let mut sources = BTreeMap::new();
    let mut source_order = Vec::new();
    for (index, row) in array(request, "sources", "request")?.iter().enumerate() {
        let context = format!("sources[{index}]");
        let id = required_string(row, "source_id", &context)?.to_owned();
        let control_id = required_string(row, "control_id", &context)?.to_owned();
        if !requirements.contains_key(&control_id) {
            return Err(format!("{context} refers to unrequired control '{control_id}'"));
        }
        let source_kind = required_string(row, "source_kind", &context)?.to_owned();
        let source_identity_sha256 = parse_digest(row, "source_identity_sha256", &context)?;
        let domain_id = required_string(row, "domain_id", &context)?.to_owned();
        let evidence_state = required_string(row, "evidence_state", &context)?.to_owned();
        let rights_status = required_string(row, "rights_status", &context)?.to_owned();
        let rights_evidence_sha256 = parse_digest(row, "rights_evidence_sha256", &context)?;
        let verified_at = parse_timestamp(row, "verified_at", &context)?;
        let valid_until = parse_optional_timestamp(row, "valid_until", &context)?;
        let expected_state = match source_kind.as_str() {
            "existing_evidence" => "verified",
            "planned_action" => "planned",
            _ => return Err(format!("{context} has unsupported source_kind")),
        };
        if evidence_state != expected_state || rights_status != "verified"
            || verified_at > evaluated_at
            || valid_until.as_ref().is_some_and(|until| until < &evaluated_at)
        {
            return Err(format!("{context} is not a current, rights-verified source candidate"));
        }
        let domain = registry.domains.get(&domain_id)
            .ok_or_else(|| format!("{context} references an unknown source domain '{domain_id}'"))?;
        if domain.status != "active" {
            return Err(format!("{context} references a revoked source domain '{domain_id}'"));
        }
        if sources.contains_key(&id) {
            return Err(format!("duplicate source_id '{id}'"));
        }
        source_order.push(id.clone());
        sources.insert(id.clone(), Source {
            id, control_id, source_kind, identity_sha256: source_identity_sha256,
            domain_id, evidence_state, rights_status, rights_evidence_sha256, verified_at, valid_until,
        });
    }

    let mut source_resolutions = BTreeMap::new();
    for (id, source) in &sources {
        let mut resolved = registry.resolved_domains.get(&source.domain_id)
            .ok_or_else(|| format!("source '{}' has no resolved registry domain", source.id))?
            .clone();
        resolved.source_id = id.clone();
        resolved.source_kind = source.source_kind.clone();
        resolved.source_identity_sha256 = source.identity_sha256.clone();
        resolved.evidence_state = source.evidence_state.clone();
        resolved.rights_status = source.rights_status.clone();
        resolved.rights_evidence_sha256 = source.rights_evidence_sha256.clone();
        resolved.verified_at = source.verified_at.clone();
        resolved.valid_until = source.valid_until.clone();
        source_resolutions.insert(id.clone(), resolved);
    }

    let mut conflicts = BTreeMap::new();
    for i in 0..source_order.len() {
        for j in (i + 1)..source_order.len() {
            let left_id = &source_order[i];
            let right_id = &source_order[j];
            let left_source = &sources[left_id];
            let right_source = &sources[right_id];
            let conflict = make_conflict(
                &source_resolutions[left_id],
                &source_resolutions[right_id],
                left_source.identity_sha256 == right_source.identity_sha256,
                left_source.domain_id == right_source.domain_id,
            );
            if let Some(conflict) = conflict {
                let key = if left_id < right_id {
                    (left_id.clone(), right_id.clone())
                } else {
                    (right_id.clone(), left_id.clone())
                };
                conflicts.insert(key, conflict);
            }
        }
    }

    Ok(Inputs {
        request_id,
        request_sha256: sha256_hex(request_bytes),
        evaluated_at,
        registry,
        requirements,
        sources,
        source_order,
        source_resolutions,
        conflicts,
    })
}

fn independent(selected: &BTreeSet<String>, conflicts: &BTreeMap<(String, String), Conflict>) -> bool {
    for left in selected {
        for right in selected {
            if left >= right {
                continue;
            }
            let key = (left.clone(), right.clone());
            if conflicts.contains_key(&key) {
                return false;
            }
        }
    }
    true
}

fn exact_maximum_independent_set(
    candidates: &[String],
    conflicts: &BTreeMap<(String, String), Conflict>,
) -> BTreeSet<String> {
    let mut ordered = candidates.to_vec();
    ordered.sort();
    let mut best = BTreeSet::new();
    let total = 1u64 << ordered.len();
    for mask in 0..total {
        let mut selected = BTreeSet::new();
        if mask.count_ones() < best.len() as u32 {
            continue;
        }
        for (index, id) in ordered.iter().enumerate() {
            if (mask & (1u64 << index)) != 0 {
                selected.insert(id.clone());
            }
        }
        if selected.len() > best.len() && independent(&selected, conflicts) {
            best = selected;
        } else if selected.len() == best.len() && independent(&selected, conflicts) {
            let current: Vec<String> = selected.iter().cloned().collect();
            let best_vec: Vec<String> = best.iter().cloned().collect();
            if current < best_vec {
                best = selected;
            }
        }
    }
    best
}

fn greedy_independent_lower_bound(
    candidates: &[String],
    conflicts: &BTreeMap<(String, String), Conflict>,
) -> BTreeSet<String> {
    let candidate_set: BTreeSet<&str> = candidates.iter().map(String::as_str).collect();
    let mut ranked: Vec<(usize, String)> = candidates.iter().map(|id| {
        let degree = conflicts.keys()
            .filter(|(left, right)| {
                (left == id || right == id)
                    && candidate_set.contains(left.as_str())
                    && candidate_set.contains(right.as_str())
            })
            .count();
        (degree, id.clone())
    }).collect();
    ranked.sort();
    let mut selected = BTreeSet::new();
    for (_, id) in ranked {
        let mut next = selected.clone();
        next.insert(id.clone());
        if independent(&next, conflicts) {
            selected.insert(id);
        }
    }
    selected
}

fn conflict_report(left: &str, right: &str, conflict: &Conflict) -> Value {
    json!({
        "source_id_a": left,
        "source_id_b": right,
        "reason_codes": conflict.reason_codes,
        "shared_dependency_ids": conflict.shared_dependency_ids,
        "shared_common_cause_group_ids": conflict.shared_common_cause_group_ids
    })
}

fn evaluate(request_value: &Value, request_bytes: &[u8]) -> Result<Value, String> {
    let inputs = validate_request(request_value, request_bytes)?;
    let mut source_resolutions: Vec<Value> = inputs.source_order.iter().map(|id| {
        let row = &inputs.source_resolutions[id];
        json!({
            "source_id": row.source_id,
            "domain_id": row.domain_id,
            "organization_id": row.organization_id,
            "source_kind": row.source_kind,
            "source_identity_sha256": row.source_identity_sha256,
            "evidence_state": row.evidence_state,
            "rights_status": row.rights_status,
            "rights_evidence_sha256": row.rights_evidence_sha256,
            "verified_at": row.verified_at,
            "valid_until": row.valid_until,
            "dependency_completeness": row.dependency_completeness,
            "dependency_closure": row.dependency_closure,
            "common_cause_group_ids": row.common_cause_group_ids,
            "registry_model_status": row.registry_model_status
        })
    }).collect();
    source_resolutions.sort_by(|a, b| a["source_id"].as_str().cmp(&b["source_id"].as_str()));

    let mut conflict_edges: Vec<Value> = inputs.conflicts.iter().map(|((left, right), conflict)| {
        conflict_report(left, right, conflict)
    }).collect();

    let mut control_results = Vec::new();
    for (control_id, requirement) in &inputs.requirements {
        let candidates: Vec<String> = inputs.sources.values()
            .filter(|source| source.control_id == *control_id)
            .map(|source| source.id.clone())
            .collect();
        let mut control_conflicts = Vec::new();
        for ((left, right), conflict) in &inputs.conflicts {
            if candidates.contains(left) && candidates.contains(right) {
                control_conflicts.push(json!({
                    "source_id_a": left,
                    "source_id_b": right,
                    "reason_codes": conflict.reason_codes
                }));
            }
        }
        let (independent_set, method, optimality_status) = if candidates.len() <= EXACT_LIMIT {
            (
                exact_maximum_independent_set(&candidates, &inputs.conflicts),
                "exact_maximum_independent_set",
                "proven_optimal",
            )
        } else {
            (
                greedy_independent_lower_bound(&candidates, &inputs.conflicts),
                "deterministic_greedy_lower_bound",
                "lower_bound_not_proven",
            )
        };
        let threshold_met = independent_set.len() >= requirement.minimum_independent_sources;
        let unresolved_present = candidates.iter().any(|id| {
            inputs.source_resolutions[id].dependency_completeness != "complete"
        });
        let registry_model_disposition = if threshold_met {
            "qualified_by_registry_model"
        } else if unresolved_present || optimality_status == "lower_bound_not_proven" {
            "indeterminate_under_registry_model"
        } else {
            "not_qualified_by_registry_model"
        };
        control_results.push(json!({
            "control_id": control_id,
            "minimum_independent_sources": requirement.minimum_independent_sources,
            "candidate_source_ids": candidates,
            "conflicting_source_pairs": control_conflicts,
            "independent_source_ids": independent_set,
            "independent_source_count": independent_set.len(),
            "threshold_met": threshold_met,
            "method": method,
            "optimality_status": optimality_status,
            "registry_model_disposition": registry_model_disposition
        }));
    }

    Ok(json!({
        "schema_version": REPORT_SCHEMA,
        "request_id": inputs.request_id,
        "request_sha256": inputs.request_sha256,
        "registry_sha256": inputs.registry.registry_sha256,
        "registry_id": inputs.registry.registry_id,
        "registry_version": inputs.registry.registry_version,
        "registry_authority_id": inputs.registry.authority_id,
        "evaluated_at": inputs.evaluated_at,
        "control_results": control_results,
        "source_resolutions": source_resolutions,
        "conflict_edges": conflict_edges,
        "action_authority": "none",
        "profile_qualification_attested": false
    }))
}

fn read_strict(path: &str) -> Result<(Vec<u8>, Value), String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let parsed: StrictJson = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{path}: invalid or duplicate-member JSON: {error}"))?;
    Ok((bytes, parsed.into_value()))
}

fn run(request_path: &str, output_path: &str) -> Result<(), String> {
    let (bytes, request) = read_strict(request_path)?;
    let report = evaluate(&request, &bytes)?;
    let encoded = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot encode independence report: {error}"))?;
    fs::write(output_path, encoded).map_err(|error| format!("cannot write independence report: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: industry-independence-audit <request.json> <report.json>");
        return ExitCode::from(4);
    }
    match run(&args[1], &args[2]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("common-cause independence check rejected the request: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../schemas/examples/ieac-common-cause-synthetic-v0.1.json"
        )).unwrap()
    }

    fn request() -> Value {
        fixture()["request"].clone()
    }

    fn evaluate_fixture(request: &Value) -> Value {
        let bytes = serde_json::to_vec(request).unwrap();
        evaluate(request, &bytes).unwrap()
    }

    #[test]
    fn common_cause_fixture_matches_frozen_independent_source_sets() {
        let fixture = fixture();
        let report = evaluate_fixture(&fixture["request"]);
        assert_eq!(
            report["schema_version"],
            "luminous.ieac-common-cause-report.v0.1"
        );
        for expected in ["numerical_known_answers", "physical_reference_validation"] {
            let actual = report["control_results"].as_array().unwrap().iter()
                .find(|row| row["control_id"] == expected).unwrap();
            assert_eq!(
                actual["independent_source_count"],
                fixture["expected"][expected]["independent_source_count"]
            );
            assert_eq!(
                actual["independent_source_ids"],
                fixture["expected"][expected]["independent_source_ids"]
            );
            assert_eq!(actual["threshold_met"], true);
            assert_eq!(actual["optimality_status"], "proven_optimal");
        }
        assert_eq!(report["action_authority"], "none");
        assert_eq!(report["profile_qualification_attested"], false);
    }

    #[test]
    fn transitive_calibration_reference_and_upstream_feed_create_conflicts() {
        let report = evaluate_fixture(&request());
        let pair = report["conflict_edges"].as_array().unwrap().iter()
            .find(|row| row["source_id_a"] == "num-alpha" && row["source_id_b"] == "num-beta")
            .unwrap();
        assert!(pair["reason_codes"].as_array().unwrap()
            .iter().any(|reason| reason == "SHARED_TRANSITIVE_DEPENDENCY"));
        assert!(pair["shared_dependency_ids"].as_array().unwrap()
            .iter().any(|id| id == "reference-standard-r1"));
        assert!(pair["reason_codes"].as_array().unwrap()
            .iter().any(|reason| reason == "SHARED_COMMON_CAUSE_GROUP"));
    }

    #[test]
    fn shared_test_harness_and_power_supply_conflict_distinct_laboratories() {
        let report = evaluate_fixture(&request());
        let pair = report["conflict_edges"].as_array().unwrap().iter()
            .find(|row| row["source_id_a"] == "physical-delta" && row["source_id_b"] == "physical-epsilon")
            .unwrap();
        assert!(pair["shared_dependency_ids"].as_array().unwrap()
            .iter().any(|id| id == "test-harness-prod-7"));
        assert!(pair["shared_dependency_ids"].as_array().unwrap()
            .iter().any(|id| id == "power-grid-zone-west"));
    }

    #[test]
    fn unknown_dependency_completeness_conflicts_with_every_other_source() {
        let report = evaluate_fixture(&request());
        let numerical = report["control_results"].as_array().unwrap().iter()
            .find(|row| row["control_id"] == "numerical_known_answers").unwrap();
        let unknown_edges = numerical["conflicting_source_pairs"].as_array().unwrap().iter()
            .filter(|row| row["source_id_a"] == "num-unknown" || row["source_id_b"] == "num-unknown")
            .count();
        assert_eq!(unknown_edges, 3);
        let resolution = report["source_resolutions"].as_array().unwrap().iter()
            .find(|row| row["source_id"] == "num-unknown").unwrap();
        assert_eq!(resolution["registry_model_status"], "unknown_dependency");
    }

    #[test]
    fn registry_digest_mismatch_is_rejected() {
        let mut request = request();
        request["registry_sha256"] = json!("0".repeat(64));
        let bytes = serde_json::to_vec(&request).unwrap();
        assert!(evaluate(&request, &bytes).is_err());
    }

    #[test]
    fn dependency_graph_cycles_are_rejected() {
        let mut request = request();
        request["registry"]["dependency_edges"].as_array_mut().unwrap()
            .push(json!({"dependency_id":"reference-standard-r1","depends_on_dependency_id":"calibration-chain-alpha"}));
        let bytes = serde_json::to_vec(&request).unwrap();
        assert!(evaluate(&request, &bytes).is_err());
    }

    #[test]
    fn revoked_domain_is_rejected_for_source_credit() {
        let mut request = request();
        request["registry"]["domains"][0]["status"] = json!("revoked");
        let registry = serde_json::to_vec(&request["registry"]).unwrap();
        request["registry_sha256"] = json!(sha256_hex(&registry));
        let bytes = serde_json::to_vec(&request).unwrap();
        assert!(evaluate(&request, &bytes).is_err());
    }

    #[test]
    fn duplicate_source_identity_cannot_gain_multiple_domains() {
        let mut request = request();
        request["sources"][1]["source_identity_sha256"] = request["sources"][0]["source_identity_sha256"].clone();
        let bytes = serde_json::to_vec(&request).unwrap();
        let report = evaluate(&request, &bytes).unwrap();
        let pair = report["conflict_edges"].as_array().unwrap().iter()
            .find(|row| row["source_id_a"] == "num-alpha" && row["source_id_b"] == "num-beta")
            .unwrap();
        assert!(pair["reason_codes"].as_array().unwrap()
            .iter().any(|reason| reason == "DUPLICATE_SOURCE_IDENTITY"));
    }

    #[test]
    fn exact_oracle_for_20_sources_and_greedy_lower_bound_above_limit() {
        let base = request();
        for target in [5usize, 10, 15, 20] {
            let mut request = base.clone();
            let original = request["sources"].as_array().unwrap().clone();
            let mut rows = original.clone();
            while rows.len() < target {
                let index = rows.len();
                let mut row = original[2].clone();
                row["source_id"] = json!(format!("scale-source-{target}-{index:03}"));
                row["source_identity_sha256"] = json!(format!("{:064x}", 1000 + index));
                rows.push(row);
            }
            request["sources"] = json!(rows);
            let bytes = serde_json::to_vec(&request).unwrap();
            let report = evaluate(&request, &bytes).unwrap();
            assert!(report["control_results"].as_array().unwrap().iter()
                .all(|row| row["method"] == "exact_maximum_independent_set"));
        }
        let mut request = base.clone();
        let original = request["sources"].as_array().unwrap().clone();
        let mut rows = original.clone();
        while rows.len() < 24 {
            let index = rows.len();
            let mut row = original[2].clone();
            row["source_id"] = json!(format!("large-source-{index:03}"));
            row["source_identity_sha256"] = json!(format!("{:064x}", 2000 + index));
            rows.push(row);
        }
        request["sources"] = json!(rows);
        let bytes = serde_json::to_vec(&request).unwrap();
        let report = evaluate(&request, &bytes).unwrap();
        let numerical = report["control_results"].as_array().unwrap().iter()
            .find(|row| row["control_id"] == "numerical_known_answers").unwrap();
        let physical = report["control_results"].as_array().unwrap().iter()
            .find(|row| row["control_id"] == "physical_reference_validation").unwrap();
        assert_eq!(numerical["method"], "deterministic_greedy_lower_bound");
        assert_eq!(numerical["optimality_status"], "lower_bound_not_proven");
        assert_eq!(physical["method"], "exact_maximum_independent_set");
        assert_eq!(physical["optimality_status"], "proven_optimal");
    }

    #[test]
    fn unknown_dependency_set_cannot_claim_two_independent_sources() {
        let mut request = request();
        request["registry"]["domains"][0]["dependency_completeness"] = json!("unknown");
        request["sources"].as_array_mut().unwrap().remove(2);
        let registry_bytes = serde_json::to_vec(&request["registry"]).unwrap();
        request["registry_sha256"] = json!(sha256_hex(&registry_bytes));
        let report = evaluate_fixture(&request);
        let row = report["control_results"].as_array().unwrap().iter()
            .find(|item| item["control_id"] == "numerical_known_answers").unwrap();
        assert_eq!(row["independent_source_count"], 1);
        assert_eq!(row["threshold_met"], false);
        assert_eq!(row["registry_model_disposition"], "indeterminate_under_registry_model");
        let unknown_edges = report["conflict_edges"].as_array().unwrap().iter()
            .filter(|edge| edge["source_id_a"] == "num-alpha" || edge["source_id_b"] == "num-alpha")
            .count();
        assert_eq!(unknown_edges, 2);
    }

    #[test]
    fn registry_validity_expiry_is_fail_closed() {
        let mut request = request();
        request["registry"]["valid_until"] = json!("2026-10-31T23:59:59Z");
        let registry_bytes = serde_json::to_vec(&request["registry"]).unwrap();
        request["registry_sha256"] = json!(sha256_hex(&registry_bytes));
        let bytes = serde_json::to_vec(&request).unwrap();
        assert!(evaluate(&request, &bytes).is_err());
    }

    #[test]
    fn duplicate_json_members_are_rejected() {
        assert!(serde_json::from_str::<StrictJson>(r#"{"key":1,"key":2}"#).is_err());
    }

    #[test]
    fn sha256_known_answer() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
