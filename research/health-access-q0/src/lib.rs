use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

pub const EXPECTED_CORPUS_SHA256: &str =
    "59ee559cb4624f25103c153876d89bf64a91a3c7fc2027a10f238359e9826fd5";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corpus {
    pub schema_version: String,
    pub status: String,
    pub units: Units,
    pub cases: Vec<AccessCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Units {
    pub travel_time: String,
    pub capacity: String,
    pub demand: String,
    pub time_of_day: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessCase {
    pub id: String,
    pub description: String,
    pub nodes: Vec<String>,
    pub edges: Vec<Edge>,
    pub demand: Demand,
    pub facilities: Vec<Facility>,
    pub evaluation_minute: u32,
    pub max_evidence_age_days: u32,
    pub max_travel_time_minutes: u32,
    pub expected: Expected,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    #[serde(rename = "from")]
    pub from_node: String,
    #[serde(rename = "to")]
    pub to_node: String,
    pub travel_time_minutes: u32,
    pub state: EdgeState,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeState {
    Available,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Demand {
    pub node: String,
    pub service: String,
    pub units: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facility {
    pub id: String,
    pub node: String,
    pub service: String,
    pub availability: Availability,
    pub readiness: Readiness,
    pub capacity_units: u32,
    pub source_id: String,
    pub evidence_age_days: u32,
    pub opens_minute: u32,
    pub closes_minute: u32,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Readiness {
    Ready,
    NotReady,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    pub eligible_facility_ids: Vec<String>,
    pub reachable_capacity_units: u32,
    pub served_units: u32,
    pub unserved_units: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuditResult {
    pub case_id: String,
    pub eligible_facility_ids: Vec<String>,
    pub reachable_capacity_units: u32,
    pub served_units: u32,
    pub unserved_units: u32,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn parse_and_verify_corpus(bytes: &[u8]) -> Result<Corpus, String> {
    let actual_digest = sha256(bytes);
    if actual_digest != EXPECTED_CORPUS_SHA256 {
        return Err(format!(
            "fixture digest mismatch: expected {EXPECTED_CORPUS_SHA256}, got {actual_digest}"
        ));
    }
    let corpus: Corpus =
        serde_json::from_slice(bytes).map_err(|error| format!("invalid corpus JSON: {error}"))?;
    validate_corpus(&corpus)?;
    Ok(corpus)
}

pub fn validate_corpus(corpus: &Corpus) -> Result<(), String> {
    if corpus.schema_version != "health-access-reference-corpus-v0.1" {
        return Err("unsupported schema_version".to_string());
    }
    if corpus.status != "synthetic_contract_only" {
        return Err("corpus status must remain synthetic_contract_only".to_string());
    }
    if corpus.units.travel_time != "minutes"
        || corpus.units.capacity != "service_units_per_open_window"
        || corpus.units.demand != "service_units_per_open_window"
        || corpus.units.time_of_day != "minutes_after_midnight"
    {
        return Err("corpus units do not match the v0.1 contract".to_string());
    }
    if corpus.cases.is_empty() {
        return Err("corpus must include at least one case".to_string());
    }

    let mut seen_case_ids = HashSet::new();
    for case in &corpus.cases {
        if !seen_case_ids.insert(case.id.as_str()) {
            return Err(format!("duplicate case id: {}", case.id));
        }
        validate_case(case)?;
    }
    Ok(())
}

pub fn validate_case(case: &AccessCase) -> Result<(), String> {
    if case.id.trim().is_empty() {
        return Err("case id must not be empty".to_string());
    }
    if case.nodes.is_empty() {
        return Err(format!("{}: node list must not be empty", case.id));
    }
    if case.evaluation_minute >= 1440 {
        return Err(format!("{}: evaluation_minute must be within the day", case.id));
    }
    if case.demand.units == 0 {
        return Err(format!("{}: demand units must be positive", case.id));
    }
    let node_set: HashSet<&str> = case.nodes.iter().map(String::as_str).collect();
    if node_set.len() != case.nodes.len() {
        return Err(format!("{}: duplicate node id", case.id));
    }
    if !node_set.contains(case.demand.node.as_str()) {
        return Err(format!("{}: demand node is undeclared", case.id));
    }
    for edge in &case.edges {
        if !node_set.contains(edge.from_node.as_str()) || !node_set.contains(edge.to_node.as_str()) {
            return Err(format!("{}: edge endpoint is undeclared", case.id));
        }
    }
    let mut facility_ids = HashSet::new();
    for facility in &case.facilities {
        if facility.id.trim().is_empty() || !facility_ids.insert(facility.id.as_str()) {
            return Err(format!("{}: empty or duplicate facility id", case.id));
        }
        if !node_set.contains(facility.node.as_str()) {
            return Err(format!("{}: facility node is undeclared", case.id));
        }
        if facility.opens_minute >= facility.closes_minute || facility.closes_minute > 1440 {
            return Err(format!("{}: invalid service window for {}", case.id, facility.id));
        }
    }
    Ok(())
}

fn shortest_travel_minutes(case: &AccessCase, target: &str) -> Option<u32> {
    let mut distance: HashMap<&str, u32> = case
        .nodes
        .iter()
        .map(|node| (node.as_str(), u32::MAX))
        .collect();
    distance.insert(case.demand.node.as_str(), 0);

    // Bellman-Ford over undirected available edges. Fixture graphs are small;
    // this intentionally favors auditable simplicity over performance.
    for _ in 0..case.nodes.len().saturating_sub(1) {
        let mut changed = false;
        for edge in &case.edges {
            if edge.state != EdgeState::Available {
                continue;
            }
            let from_dist = *distance.get(edge.from_node.as_str())?;
            if let Some(candidate) = from_dist.checked_add(edge.travel_time_minutes) {
                if candidate < *distance.get(edge.to_node.as_str())? {
                    distance.insert(edge.to_node.as_str(), candidate);
                    changed = true;
                }
            }
            let to_dist = *distance.get(edge.to_node.as_str())?;
            if let Some(candidate) = to_dist.checked_add(edge.travel_time_minutes) {
                if candidate < *distance.get(edge.from_node.as_str())? {
                    distance.insert(edge.from_node.as_str(), candidate);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    match distance.get(target).copied() {
        Some(u32::MAX) | None => None,
        value => value,
    }
}

pub fn evaluate_case(case: &AccessCase) -> Result<AuditResult, String> {
    validate_case(case)?;
    let mut eligible: Vec<&Facility> = case
        .facilities
        .iter()
        .filter(|facility| {
            facility.service == case.demand.service
                && facility.availability == Availability::Available
                && facility.readiness == Readiness::Ready
                && !facility.source_id.trim().is_empty()
                && facility.evidence_age_days <= case.max_evidence_age_days
                && case.evaluation_minute >= facility.opens_minute
                && case.evaluation_minute < facility.closes_minute
                && shortest_travel_minutes(case, facility.node.as_str())
                    .is_some_and(|minutes| minutes <= case.max_travel_time_minutes)
        })
        .collect();
    eligible.sort_by(|left, right| left.id.cmp(&right.id));

    let mut reachable_capacity_units = 0u32;
    for facility in &eligible {
        reachable_capacity_units = reachable_capacity_units
            .checked_add(facility.capacity_units)
            .ok_or_else(|| format!("{}: reachable capacity overflow", case.id))?;
    }
    let served_units = reachable_capacity_units.min(case.demand.units);
    let unserved_units = case.demand.units - served_units;

    Ok(AuditResult {
        case_id: case.id.clone(),
        eligible_facility_ids: eligible.iter().map(|facility| facility.id.clone()).collect(),
        reachable_capacity_units,
        served_units,
        unserved_units,
    })
}

pub fn audit_corpus(corpus: &Corpus) -> Result<Vec<AuditResult>, String> {
    validate_corpus(corpus)?;
    corpus.cases.iter().map(evaluate_case).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] = include_bytes!("../fixtures/reference-cases.v0.json");

    fn corpus() -> Corpus {
        parse_and_verify_corpus(FIXTURE).expect("pinned synthetic corpus should parse")
    }

    #[test]
    fn fixture_digest_is_pinned() {
        assert_eq!(FIXTURE.len(), 12_127);
        assert_eq!(sha256(FIXTURE), EXPECTED_CORPUS_SHA256);
    }

    #[test]
    fn all_frozen_cases_match_independent_expected_results() {
        let corpus = corpus();
        let results = audit_corpus(&corpus).expect("audit should succeed");
        assert_eq!(results.len(), corpus.cases.len());
        for (case, result) in corpus.cases.iter().zip(results.iter()) {
            assert_eq!(
                result.eligible_facility_ids, case.expected.eligible_facility_ids,
                "{} eligible facilities",
                case.id
            );
            assert_eq!(result.reachable_capacity_units, case.expected.reachable_capacity_units, "{} capacity", case.id);
            assert_eq!(result.served_units, case.expected.served_units, "{} served", case.id);
            assert_eq!(result.unserved_units, case.expected.unserved_units, "{} unserved", case.id);
        }
    }

    #[test]
    fn unknown_availability_mutation_cannot_add_capacity() {
        let mut scenario = corpus().cases.remove(0);
        scenario.facilities[0].availability = Availability::Unknown;
        let result = evaluate_case(&scenario).unwrap();
        assert!(result.eligible_facility_ids.is_empty());
        assert_eq!(result.reachable_capacity_units, 0);
        assert_eq!(result.unserved_units, scenario.demand.units);
    }

    #[test]
    fn stale_evidence_mutation_cannot_add_capacity() {
        let mut scenario = corpus().cases.remove(0);
        scenario.facilities[0].evidence_age_days = scenario.max_evidence_age_days + 1;
        let result = evaluate_case(&scenario).unwrap();
        assert_eq!(result.reachable_capacity_units, 0);
    }

    #[test]
    fn unknown_readiness_mutation_cannot_add_capacity() {
        let mut scenario = corpus().cases.remove(0);
        scenario.facilities[0].readiness = Readiness::Unknown;
        let result = evaluate_case(&scenario).unwrap();
        assert_eq!(result.reachable_capacity_units, 0);
    }

    #[test]
    fn empty_source_identity_cannot_add_capacity() {
        let mut scenario = corpus().cases.remove(0);
        scenario.facilities[0].source_id.clear();
        let result = evaluate_case(&scenario).unwrap();
        assert_eq!(result.reachable_capacity_units, 0);
    }

    #[test]
    fn closing_service_window_at_evaluation_time_excludes_facility() {
        let mut scenario = corpus().cases.remove(0);
        scenario.facilities[0].closes_minute = scenario.evaluation_minute;
        let result = evaluate_case(&scenario).unwrap();
        assert_eq!(result.reachable_capacity_units, 0);
    }

    #[test]
    fn blocking_every_route_cannot_increase_capacity() {
        let mut scenario = corpus()
            .cases
            .into_iter()
            .find(|case| case.id == "blocked_direct_route_uses_verified_detour")
            .unwrap();
        for edge in &mut scenario.edges {
            edge.state = EdgeState::Blocked;
        }
        let result = evaluate_case(&scenario).unwrap();
        assert!(result.eligible_facility_ids.is_empty());
        assert_eq!(result.reachable_capacity_units, 0);
        assert_eq!(result.unserved_units, scenario.demand.units);
    }

    #[test]
    fn malformed_unknown_node_is_rejected() {
        let mut scenario = corpus().cases.remove(0);
        scenario.edges[0].to_node = "undeclared-node".to_string();
        assert!(validate_case(&scenario).is_err());
    }

    #[test]
    fn unknown_edge_state_does_not_make_a_route() {
        let mut scenario = corpus().cases.remove(0);
        scenario.edges[0].state = EdgeState::Unknown;
        let result = evaluate_case(&scenario).unwrap();
        assert_eq!(result.reachable_capacity_units, 0);
    }
}
