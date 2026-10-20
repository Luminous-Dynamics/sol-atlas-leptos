// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Topology-family holdout and cross-split leakage auditing for water-localization experiments.
//!
//! This audits explicit synthetic scenario descriptors. It does not run EPANET, score a
//! localizer, detect physical leaks, or qualify simulation-to-field transfer.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub const PLAN_SCHEMA_VERSION: &str = "water-localization-split-plan/v0.1";
pub const RECEIPT_SCHEMA_VERSION: &str = "water-localization-split-audit/v0.1";
pub const CLAIM_CEILING: &str = "SplitIntegrityOnly";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SplitPlan {
    pub schema_version: String,
    pub classification: String,
    pub benchmark_id: String,
    pub protocol: String,
    pub pressure_unit: String,
    pub train_topology_families: Vec<String>,
    pub calibration_topology_families: Vec<String>,
    pub test_topology_families: Vec<String>,
    pub cases: Vec<SplitCase>,
    pub assumptions: Vec<String>,
    pub nonclaims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SplitCase {
    pub case_id: String,
    pub split: String,
    /// Curator-reviewed grouping for exact duplicates and known relabeled or near-duplicate graphs.
    pub topology_family_id: String,
    pub topology: TopologyGraph,
    pub perturbation: PerturbationSpec,
    pub sensor_readings: Vec<SensorReading>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyGraph {
    pub nodes: Vec<String>,
    pub links: Vec<TopologyLink>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyLink {
    pub link_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PerturbationSpec {
    pub fault_class: String,
    pub target_node_id: Option<String>,
    pub withdrawal_gpm: f64,
    pub scenario_seed: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SensorReading {
    pub node_id: String,
    pub state: String,
    pub pressure_delta_psi: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SplitAuditReceipt {
    pub schema_version: String,
    pub run_state: String,
    pub benchmark_id: String,
    pub protocol: String,
    pub plan_sha256: String,
    pub implementation_revision: Option<String>,
    pub case_count: usize,
    pub family_count: usize,
    pub family_counts: Vec<FamilyCount>,
    pub cases: Vec<CaseFingerprint>,
    /// Label-invariant color-refinement collisions that need curator review; not proof of isomorphism.
    pub topology_review_candidates: Vec<TopologyReviewCandidate>,
    pub issues: Vec<AuditIssue>,
    pub claim_ceiling: String,
    pub nonclaims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FamilyCount {
    pub topology_family_id: String,
    pub split: String,
    pub case_count: usize,
    pub topology_variants: Vec<TopologyFingerprint>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TopologyFingerprint {
    pub topology_sha256: String,
    pub case_count: usize,
    pub node_count: usize,
    pub link_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CaseFingerprint {
    pub case_id: String,
    pub split: String,
    pub topology_family_id: String,
    pub topology_sha256: String,
    /// Label-invariant WL-style signature. Collisions are review signals, not proof of graph isomorphism.
    pub structural_topology_sha256: String,
    pub scenario_spec_sha256: String,
    pub observation_signature_sha256: String,
    pub observed_reading_count: usize,
    pub missing_reading_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TopologyReviewCandidate {
    pub structural_topology_sha256: String,
    pub family_a: String,
    pub split_a: String,
    pub case_ids_a: Vec<String>,
    pub family_b: String,
    pub split_b: String,
    pub case_ids_b: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuditIssue {
    pub code: String,
    pub message: String,
    pub case_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct CanonicalTopology {
    nodes: Vec<String>,
    links: Vec<CanonicalLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct CanonicalLink {
    link_id: String,
    endpoint_a: String,
    endpoint_b: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct CanonicalScenarioSpec<'a> {
    topology_sha256: &'a str,
    perturbation: &'a PerturbationSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct CanonicalObservationSignature {
    pressure_unit: String,
    sensor_readings: Vec<SensorReading>,
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn canonical_json_sha256<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("canonical JSON serialization failed: {error}"))?;
    Ok(sha256(&bytes))
}

fn nonblank(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Compute an order-independent graph digest. Node labels remain identity-bearing; graph-isomorphic
/// or relabeled variants must share a curator-reviewed topology_family_id to prevent split leakage.
pub fn topology_sha256(graph: &TopologyGraph) -> Result<String, String> {
    let mut nodes = graph.nodes.clone();
    nodes.sort();
    let mut links = Vec::with_capacity(graph.links.len());
    for link in &graph.links {
        let (endpoint_a, endpoint_b) = if link.from_node_id <= link.to_node_id {
            (link.from_node_id.clone(), link.to_node_id.clone())
        } else {
            (link.to_node_id.clone(), link.from_node_id.clone())
        };
        links.push(CanonicalLink {
            link_id: link.link_id.clone(),
            endpoint_a,
            endpoint_b,
        });
    }
    links.sort_by(|left, right| {
        (&left.link_id, &left.endpoint_a, &left.endpoint_b).cmp(&(
            &right.link_id,
            &right.endpoint_a,
            &right.endpoint_b,
        ))
    });
    canonical_json_sha256(&CanonicalTopology { nodes, links })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CanonicalStructuralTopology {
    algorithm: &'static str,
    node_count: usize,
    link_count: usize,
    node_colors: Vec<usize>,
    edge_colors: Vec<(usize, usize)>,
}

fn compress_structural_colors(signatures: &[String]) -> Vec<usize> {
    let labels: BTreeSet<String> = signatures.iter().cloned().collect();
    let ids: BTreeMap<String, usize> = labels.into_iter().enumerate()
        .map(|(index, label)| (label, index)).collect();
    signatures.iter().map(|signature| ids[signature]).collect()
}

/// Label-invariant 1-WL-style signature for an undirected topology.
///
/// This is only a candidate generator: non-isomorphic regular graphs can share
/// a 1-WL signature. A cross-split collision needs curator review and is not a
/// graph-isomorphism proof.
pub fn structural_topology_sha256(graph: &TopologyGraph) -> Result<String, String> {
    if !validate_graph(graph).is_empty() {
        return Err("structural signature requires a structurally valid topology".to_owned());
    }
    let node_index: BTreeMap<&str, usize> = graph.nodes.iter().enumerate()
        .map(|(index, node)| (node.as_str(), index)).collect();
    let mut adjacency = vec![Vec::<usize>::new(); graph.nodes.len()];
    let mut edges = Vec::with_capacity(graph.links.len());
    for link in &graph.links {
        let a = *node_index.get(link.from_node_id.as_str())
            .ok_or_else(|| "structural signature has an unknown link endpoint".to_owned())?;
        let b = *node_index.get(link.to_node_id.as_str())
            .ok_or_else(|| "structural signature has an unknown link endpoint".to_owned())?;
        adjacency[a].push(b);
        adjacency[b].push(a);
        edges.push((a, b));
    }
    let initial: Vec<String> = adjacency.iter()
        .map(|neighbors| format!("degree:{}", neighbors.len())).collect();
    let mut colors = compress_structural_colors(&initial);
    // Descriptors include the prior color, so refinement can split but never merge classes.
    for _ in 0..graph.nodes.len() {
        let signatures: Vec<String> = adjacency.iter().enumerate().map(|(node, neighbors)| {
            let mut neighbor_colors: Vec<usize> = neighbors.iter().map(|index| colors[*index]).collect();
            neighbor_colors.sort_unstable();
            format!("self:{}|neighbors:{:?}", colors[node], neighbor_colors)
        }).collect();
        let next = compress_structural_colors(&signatures);
        let old_count = colors.iter().copied().max().unwrap_or(0) + 1;
        let new_count = next.iter().copied().max().unwrap_or(0) + 1;
        colors = next;
        if new_count == old_count { break; }
    }
    let mut edge_colors: Vec<(usize, usize)> = edges.into_iter().map(|(a, b)| {
        let (ca, cb) = (colors[a], colors[b]);
        if ca <= cb { (ca, cb) } else { (cb, ca) }
    }).collect();
    edge_colors.sort_unstable();
    let mut node_colors = colors;
    node_colors.sort_unstable();
    canonical_json_sha256(&CanonicalStructuralTopology {
        algorithm: "water-topology-wl-1-v1",
        node_count: graph.nodes.len(),
        link_count: graph.links.len(),
        node_colors,
        edge_colors,
    })
}

fn validate_graph(graph: &TopologyGraph) -> Vec<String> {
    let mut issues = Vec::new();
    if graph.nodes.len() < 2 {
        issues.push("TOPOLOGY_TOO_SMALL".to_owned());
    }
    if graph.links.is_empty() {
        issues.push("TOPOLOGY_HAS_NO_LINKS".to_owned());
    }
    let mut node_ids = HashSet::new();
    for node in &graph.nodes {
        if !nonblank(node) {
            issues.push("TOPOLOGY_NODE_ID_EMPTY".to_owned());
        }
        if !node_ids.insert(node.as_str()) {
            issues.push("TOPOLOGY_NODE_ID_DUPLICATE".to_owned());
        }
    }
    let mut link_ids = HashSet::new();
    for link in &graph.links {
        if !nonblank(&link.link_id) {
            issues.push("TOPOLOGY_LINK_ID_EMPTY".to_owned());
        }
        if !link_ids.insert(link.link_id.as_str()) {
            issues.push("TOPOLOGY_LINK_ID_DUPLICATE".to_owned());
        }
        if !node_ids.contains(link.from_node_id.as_str())
            || !node_ids.contains(link.to_node_id.as_str())
        {
            issues.push("TOPOLOGY_LINK_ENDPOINT_UNKNOWN".to_owned());
        }
        if link.from_node_id == link.to_node_id {
            issues.push("TOPOLOGY_SELF_LOOP_UNSUPPORTED".to_owned());
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

fn validate_case(plan: &SplitPlan, case: &SplitCase) -> Vec<String> {
    let mut issues = validate_graph(&case.topology);
    let nodes: HashSet<&str> = case.topology.nodes.iter().map(String::as_str).collect();
    if !nonblank(&case.case_id) {
        issues.push("CASE_ID_EMPTY".to_owned());
    }
    if !nonblank(&case.topology_family_id) {
        issues.push("TOPOLOGY_FAMILY_ID_EMPTY".to_owned());
    }
    if !nonblank(&case.perturbation.scenario_seed) {
        issues.push("SCENARIO_SEED_EMPTY".to_owned());
    }
    if !["train", "calibration", "test"].contains(&case.split.as_str()) {
        issues.push("SPLIT_NAME_INVALID".to_owned());
    }
    if !["no_fault", "single_withdrawal", "incipient_withdrawal"]
        .contains(&case.perturbation.fault_class.as_str())
    {
        issues.push("FAULT_CLASS_INVALID".to_owned());
    }
    if !case.perturbation.withdrawal_gpm.is_finite() || case.perturbation.withdrawal_gpm < 0.0 {
        issues.push("WITHDRAWAL_MAGNITUDE_INVALID".to_owned());
    }
    match case.perturbation.fault_class.as_str() {
        "no_fault" => {
            if case.perturbation.target_node_id.is_some() || case.perturbation.withdrawal_gpm != 0.0
            {
                issues.push("NO_FAULT_LABEL_CONTRADICTS_PERTURBATION".to_owned());
            }
        }
        "single_withdrawal" | "incipient_withdrawal" => {
            if case
                .perturbation
                .target_node_id
                .as_deref()
                .is_none_or(|node| !nodes.contains(node))
                || case.perturbation.withdrawal_gpm <= 0.0
            {
                issues.push("WITHDRAWAL_TARGET_OR_MAGNITUDE_INVALID".to_owned());
            }
        }
        _ => {}
    }
    if case.sensor_readings.is_empty() {
        issues.push("SENSOR_READINGS_EMPTY".to_owned());
    }
    let mut reading_nodes = HashSet::new();
    let mut observed = 0usize;
    for reading in &case.sensor_readings {
        if !nodes.contains(reading.node_id.as_str()) {
            issues.push("SENSOR_NODE_UNKNOWN".to_owned());
        }
        if !reading_nodes.insert(reading.node_id.as_str()) {
            issues.push("SENSOR_NODE_DUPLICATE".to_owned());
        }
        match reading.state.as_str() {
            "observed" => {
                if reading
                    .pressure_delta_psi
                    .is_none_or(|value| !value.is_finite())
                {
                    issues.push("OBSERVED_PRESSURE_VALUE_INVALID".to_owned());
                } else {
                    observed += 1;
                }
            }
            "missing" => {
                if reading.pressure_delta_psi.is_some() {
                    issues.push("MISSING_PRESSURE_MUST_NOT_HAVE_VALUE".to_owned());
                }
            }
            _ => issues.push("SENSOR_READING_STATE_INVALID".to_owned()),
        }
    }
    if observed == 0 {
        issues.push("NO_OBSERVED_SENSOR_READINGS".to_owned());
    }
    if !nonblank(&plan.pressure_unit) {
        issues.push("PRESSURE_UNIT_EMPTY".to_owned());
    }
    issues.sort();
    issues.dedup();
    issues
}

fn family_lists(plan: &SplitPlan) -> Result<BTreeMap<String, String>, Vec<AuditIssue>> {
    let mut mapping = BTreeMap::new();
    let lists = [
        ("train", &plan.train_topology_families),
        ("calibration", &plan.calibration_topology_families),
        ("test", &plan.test_topology_families),
    ];
    let mut issues = Vec::new();
    for (split, families) in lists {
        if families.is_empty() {
            issues.push(AuditIssue {
                code: "SPLIT_FAMILY_LIST_EMPTY".to_owned(),
                message: format!("{split} topology family list must not be empty"),
                case_id: None,
            });
        }
        let mut seen = HashSet::new();
        for family in families {
            if !nonblank(family) {
                issues.push(AuditIssue {
                    code: "TOPOLOGY_FAMILY_ID_EMPTY".to_owned(),
                    message: format!("{split} family list contains an empty ID"),
                    case_id: None,
                });
                continue;
            }
            if !seen.insert(family.as_str()) {
                issues.push(AuditIssue {
                    code: "TOPOLOGY_FAMILY_LIST_DUPLICATE".to_owned(),
                    message: format!("{split} family list repeats {family}"),
                    case_id: None,
                });
            }
            if let Some(previous) = mapping.insert(family.clone(), split.to_owned()) {
                issues.push(AuditIssue {
                    code: "TOPOLOGY_FAMILY_SPLIT_OVERLAP".to_owned(),
                    message: format!(
                        "topology family {family} appears in both {previous} and {split}"
                    ),
                    case_id: None,
                });
            }
        }
    }
    if issues.is_empty() {
        Ok(mapping)
    } else {
        Err(issues)
    }
}

pub fn evaluate_split_plan(plan: &SplitPlan, raw_plan_sha256: &str) -> SplitAuditReceipt {
    let mut issues = Vec::new();
    if !valid_sha256(raw_plan_sha256) {
        issues.push(AuditIssue {
            code: "PLAN_SHA256_INVALID".to_owned(),
            message: "raw plan SHA-256 must be 64 lowercase hexadecimal characters".to_owned(),
            case_id: None,
        });
    }
    if plan.schema_version != PLAN_SCHEMA_VERSION {
        issues.push(AuditIssue {
            code: "PLAN_SCHEMA_VERSION_UNSUPPORTED".to_owned(),
            message: format!(
                "expected {PLAN_SCHEMA_VERSION}, got {}",
                plan.schema_version
            ),
            case_id: None,
        });
    }
    if plan.classification != "synthetic_public" {
        issues.push(AuditIssue {
            code: "PLAN_CLASSIFICATION_INVALID".to_owned(),
            message: "v0.1 split fixture must be marked synthetic_public".to_owned(),
            case_id: None,
        });
    }
    if !nonblank(&plan.benchmark_id) {
        issues.push(AuditIssue {
            code: "BENCHMARK_ID_EMPTY".to_owned(),
            message: "benchmark_id must be non-empty".to_owned(),
            case_id: None,
        });
    }
    if plan.protocol != "leave_topology_family_out" {
        issues.push(AuditIssue {
            code: "PROTOCOL_UNSUPPORTED".to_owned(),
            message: "v0.1 requires leave_topology_family_out".to_owned(),
            case_id: None,
        });
    }
    if plan.assumptions.is_empty() || plan.assumptions.iter().any(|item| !nonblank(item)) {
        issues.push(AuditIssue {
            code: "ASSUMPTIONS_INCOMPLETE".to_owned(),
            message: "assumptions must be a non-empty array of non-empty strings".to_owned(),
            case_id: None,
        });
    }
    if plan.nonclaims.is_empty() || plan.nonclaims.iter().any(|item| !nonblank(item)) {
        issues.push(AuditIssue {
            code: "NONCLAIMS_INCOMPLETE".to_owned(),
            message: "nonclaims must be a non-empty array of non-empty strings".to_owned(),
            case_id: None,
        });
    }

    let family_to_split = match family_lists(plan) {
        Ok(mapping) => mapping,
        Err(family_issues) => {
            issues.extend(family_issues);
            BTreeMap::new()
        }
    };
    if plan.cases.is_empty() {
        issues.push(AuditIssue {
            code: "CASE_SET_EMPTY".to_owned(),
            message: "split audit requires at least one case".to_owned(),
            case_id: None,
        });
    }

    let mut seen_case_ids = HashSet::new();
    let mut family_split: BTreeMap<String, String> = BTreeMap::new();
    let mut topology_family: BTreeMap<String, String> = BTreeMap::new();
    let mut seed_split: BTreeMap<String, String> = BTreeMap::new();
    let mut observation_split: BTreeMap<String, String> = BTreeMap::new();
    let mut scenario_split: BTreeMap<String, String> = BTreeMap::new();
    let mut family_members: BTreeMap<String, Vec<&SplitCase>> = BTreeMap::new();
    let mut case_fingerprints = Vec::new();
    let mut structural_family_groups: BTreeMap<(String, String, String), BTreeSet<String>> =
        BTreeMap::new();

    for case in &plan.cases {
        if !seen_case_ids.insert(case.case_id.as_str()) {
            issues.push(AuditIssue {
                code: "CASE_ID_DUPLICATE".to_owned(),
                message: format!("duplicate case ID {}", case.case_id),
                case_id: Some(case.case_id.clone()),
            });
        }
        for code in validate_case(plan, case) {
            issues.push(AuditIssue {
                code: code.clone(),
                message: format!("case {} failed {code}", case.case_id),
                case_id: Some(case.case_id.clone()),
            });
        }
        if let Some(expected_split) = family_to_split.get(&case.topology_family_id) {
            if expected_split != &case.split {
                issues.push(AuditIssue {
                    code: "CASE_SPLIT_DOES_NOT_MATCH_FAMILY_PLAN".to_owned(),
                    message: format!(
                        "case {} declares split {}, but family {} is assigned to {}",
                        case.case_id, case.split, case.topology_family_id, expected_split
                    ),
                    case_id: Some(case.case_id.clone()),
                });
            }
        } else {
            issues.push(AuditIssue {
                code: "CASE_TOPOLOGY_FAMILY_UNDECLARED".to_owned(),
                message: format!(
                    "case {} uses family {} not listed in the split plan",
                    case.case_id, case.topology_family_id
                ),
                case_id: Some(case.case_id.clone()),
            });
        }

        let topology_digest = match topology_sha256(&case.topology) {
            Ok(value) => value,
            Err(error) => {
                issues.push(AuditIssue {
                    code: "TOPOLOGY_FINGERPRINT_FAILED".to_owned(),
                    message: error,
                    case_id: Some(case.case_id.clone()),
                });
                continue;
            }
        };
        let structural_digest = match structural_topology_sha256(&case.topology) {
            Ok(value) => value,
            Err(error) => {
                issues.push(AuditIssue {
                    code: "STRUCTURAL_TOPOLOGY_FINGERPRINT_FAILED".to_owned(),
                    message: error,
                    case_id: Some(case.case_id.clone()),
                });
                continue;
            }
        };
        structural_family_groups.entry((
            structural_digest.clone(), case.split.clone(), case.topology_family_id.clone()
        )).or_default().insert(case.case_id.clone());
        if let Some(previous) =
            family_split.insert(case.topology_family_id.clone(), case.split.clone())
        {
            if previous != case.split {
                issues.push(AuditIssue {
                    code: "TOPOLOGY_FAMILY_CROSSES_SPLITS".to_owned(),
                    message: format!(
                        "topology family {} occurs in both {previous} and {}",
                        case.topology_family_id, case.split
                    ),
                    case_id: Some(case.case_id.clone()),
                });
            }
        }
        if let Some(previous) =
            topology_family.insert(topology_digest.clone(), case.topology_family_id.clone())
        {
            if previous != case.topology_family_id {
                issues.push(AuditIssue {
                    code: "EXACT_TOPOLOGY_HAS_MULTIPLE_FAMILY_IDS".to_owned(),
                    message: format!(
                        "exact topology appears under family IDs {previous} and {}",
                        case.topology_family_id
                    ),
                    case_id: Some(case.case_id.clone()),
                });
            }
        }
        insert_cross_split_identity(
            &mut seed_split,
            &case.perturbation.scenario_seed,
            &case.split,
            "SCENARIO_SEED_CROSSES_SPLITS",
            &case.case_id,
            &mut issues,
        );

        let observation_digest =
            match observation_signature_sha256(&plan.pressure_unit, &case.sensor_readings) {
                Ok(value) => value,
                Err(error) => {
                    issues.push(AuditIssue {
                        code: "OBSERVATION_FINGERPRINT_FAILED".to_owned(),
                        message: error,
                        case_id: Some(case.case_id.clone()),
                    });
                    continue;
                }
            };
        insert_cross_split_identity(
            &mut observation_split,
            &observation_digest,
            &case.split,
            "OBSERVATION_SIGNATURE_CROSSES_SPLITS",
            &case.case_id,
            &mut issues,
        );
        let scenario_digest = match scenario_spec_sha256(&topology_digest, &case.perturbation) {
            Ok(value) => value,
            Err(error) => {
                issues.push(AuditIssue {
                    code: "SCENARIO_FINGERPRINT_FAILED".to_owned(),
                    message: error,
                    case_id: Some(case.case_id.clone()),
                });
                continue;
            }
        };
        insert_cross_split_identity(
            &mut scenario_split,
            &scenario_digest,
            &case.split,
            "SCENARIO_SPEC_CROSSES_SPLITS",
            &case.case_id,
            &mut issues,
        );

        case_fingerprints.push(CaseFingerprint {
            case_id: case.case_id.clone(),
            split: case.split.clone(),
            topology_family_id: case.topology_family_id.clone(),
            topology_sha256: topology_digest,
            structural_topology_sha256: structural_digest,
            scenario_spec_sha256: scenario_digest,
            observation_signature_sha256: observation_digest,
            observed_reading_count: case
                .sensor_readings
                .iter()
                .filter(|r| r.state == "observed")
                .count(),
            missing_reading_count: case
                .sensor_readings
                .iter()
                .filter(|r| r.state == "missing")
                .count(),
        });
        family_members
            .entry(case.topology_family_id.clone())
            .or_default()
            .push(case);
    }

    let observed_families: BTreeSet<String> = family_members.keys().cloned().collect();
    let declared_families: BTreeSet<String> = family_to_split.keys().cloned().collect();
    for missing in declared_families.difference(&observed_families) {
        issues.push(AuditIssue {
            code: "DECLARED_TOPOLOGY_FAMILY_EMPTY".to_owned(),
            message: format!("declared topology family {missing} has no cases"),
            case_id: None,
        });
    }

    let mut family_counts = Vec::new();
    for (family_id, cases) in &family_members {
        if let Some(first) = cases.first() {
            let mut by_topology: BTreeMap<String, Vec<&SplitCase>> = BTreeMap::new();
            for case in cases {
                let digest = topology_sha256(&case.topology).unwrap_or_default();
                by_topology.entry(digest).or_default().push(case);
            }
            let topology_variants = by_topology
                .into_iter()
                .filter_map(|(digest, variants)| {
                    let first_variant = variants.first()?;
                    Some(TopologyFingerprint {
                        topology_sha256: digest,
                        case_count: variants.len(),
                        node_count: first_variant.topology.nodes.len(),
                        link_count: first_variant.topology.links.len(),
                    })
                })
                .collect();
            family_counts.push(FamilyCount {
                topology_family_id: family_id.clone(),
                split: first.split.clone(),
                case_count: cases.len(),
                topology_variants,
            });
        }
    }
    let mut groups_by_signature: BTreeMap<String, Vec<(String, String, Vec<String>)>> =
        BTreeMap::new();
    for ((signature, split, family), case_ids) in structural_family_groups {
        groups_by_signature.entry(signature).or_default()
            .push((split, family, case_ids.into_iter().collect()));
    }
    let mut topology_review_candidates = Vec::new();
    for (signature, groups) in groups_by_signature {
        for left_index in 0..groups.len() {
            for right_index in left_index + 1..groups.len() {
                let (split_a, family_a, case_ids_a) = &groups[left_index];
                let (split_b, family_b, case_ids_b) = &groups[right_index];
                if split_a == split_b || family_a == family_b { continue; }
                topology_review_candidates.push(TopologyReviewCandidate {
                    structural_topology_sha256: signature.clone(),
                    family_a: family_a.clone(),
                    split_a: split_a.clone(),
                    case_ids_a: case_ids_a.clone(),
                    family_b: family_b.clone(),
                    split_b: split_b.clone(),
                    case_ids_b: case_ids_b.clone(),
                });
            }
        }
    }
    topology_review_candidates.sort();
    case_fingerprints.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    family_counts.sort_by(|left, right| left.topology_family_id.cmp(&right.topology_family_id));
    issues.sort_by(|left, right| {
        (&left.code, &left.case_id, &left.message).cmp(&(
            &right.code,
            &right.case_id,
            &right.message,
        ))
    });
    issues.dedup();

    let mut passed = issues.is_empty()
        && ["train", "calibration", "test"]
            .iter()
            .all(|split| family_counts.iter().any(|family| family.split == *split));
    if !passed && issues.is_empty() {
        issues.push(AuditIssue {
            code: "SPLIT_NOT_COVERED".to_owned(),
            message: "train, calibration, and test must all contain at least one topology family"
                .to_owned(),
            case_id: None,
        });
        passed = false;
    }

    let run_state = if !passed {
        "Rejected"
    } else if !topology_review_candidates.is_empty() {
        "NeedsTopologyReview"
    } else {
        "Passed"
    };

    SplitAuditReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
        run_state: run_state.to_owned(),
        benchmark_id: plan.benchmark_id.clone(),
        protocol: plan.protocol.clone(),
        plan_sha256: raw_plan_sha256.to_owned(),
        implementation_revision: None,
        case_count: plan.cases.len(),
        family_count: family_counts.len(),
        family_counts,
        cases: case_fingerprints,
        topology_review_candidates,
        issues,
        claim_ceiling: CLAIM_CEILING.to_owned(),
        nonclaims: vec![
            "Split-integrity audit only; no hydraulic solver was executed and no localization score was measured.".to_owned(),
            "A graph digest is label-sensitive; curators must group isomorphic or near-duplicate topologies under the same reviewed topology_family_id.".to_owned(),
            "Synthetic pressure signatures are not EPANET outputs, calibrated predictions, experimental data, or field observations.".to_owned(),
            "Passing split integrity does not prove dataset representativeness, model quality, calibration, or field transfer.".to_owned(),
        ],
    }
}

fn insert_cross_split_identity(
    identity_to_split: &mut BTreeMap<String, String>,
    identity: &str,
    split: &str,
    code: &str,
    case_id: &str,
    issues: &mut Vec<AuditIssue>,
) {
    if let Some(previous) = identity_to_split.get(identity) {
        if previous != split {
            issues.push(AuditIssue {
                code: code.to_owned(),
                message: format!(
                    "identity {identity} is reused across {previous} and {split} splits"
                ),
                case_id: Some(case_id.to_owned()),
            });
        }
    } else {
        identity_to_split.insert(identity.to_owned(), split.to_owned());
    }
}

fn scenario_spec_sha256(
    topology_digest: &str,
    perturbation: &PerturbationSpec,
) -> Result<String, String> {
    canonical_json_sha256(&CanonicalScenarioSpec {
        topology_sha256: topology_digest,
        perturbation,
    })
}

fn observation_signature_sha256(
    pressure_unit: &str,
    readings: &[SensorReading],
) -> Result<String, String> {
    let mut canonical_readings = readings.to_vec();
    canonical_readings.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    canonical_json_sha256(&CanonicalObservationSignature {
        pressure_unit: pressure_unit.to_owned(),
        sensor_readings: canonical_readings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/water-engineering/water-localization-topology-split.synthetic.json")
    }

    fn load_plan() -> (SplitPlan, String) {
        let bytes = fs::read(fixture_path()).expect("frozen topology split fixture");
        let plan = serde_json::from_slice(&bytes).expect("parse split plan");
        (plan, sha256(&bytes))
    }

    #[test]
    fn frozen_topology_family_holdout_plan_passes() {
        let (plan, digest) = load_plan();
        let receipt = evaluate_split_plan(&plan, &digest);
        assert_eq!(receipt.run_state, "Passed", "{:?}", receipt.issues);
        assert_eq!(receipt.case_count, 9);
        assert_eq!(receipt.family_count, 3);
        assert_eq!(
            receipt
                .family_counts
                .iter()
                .map(|row| row.case_count)
                .sum::<usize>(),
            9
        );
        assert_eq!(
            receipt
                .family_counts
                .iter()
                .map(|row| row.split.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["train", "calibration", "test"])
        );
    }

    #[test]
    fn graph_and_reading_order_does_not_change_case_fingerprints() {
        let (mut plan, digest) = load_plan();
        let initial = evaluate_split_plan(&plan, &digest);
        for case in &mut plan.cases {
            case.topology.nodes.reverse();
            case.topology.links.reverse();
            for link in &mut case.topology.links {
                std::mem::swap(&mut link.from_node_id, &mut link.to_node_id);
            }
            case.sensor_readings.reverse();
        }
        let reordered = evaluate_split_plan(&plan, &digest);
        assert_eq!(initial.run_state, reordered.run_state);
        assert_eq!(initial.cases, reordered.cases);
        assert_eq!(initial.family_counts, reordered.family_counts);
    }

    #[test]
    fn held_out_topology_family_cannot_leak_into_training() {
        let (mut plan, digest) = load_plan();
        let case = plan
            .cases
            .iter_mut()
            .find(|case| case.split == "test")
            .unwrap();
        case.split = "train".to_owned();
        let receipt = evaluate_split_plan(&plan, &digest);
        assert_eq!(receipt.run_state, "Rejected");
        assert!(
            receipt
                .issues
                .iter()
                .any(|issue| issue.code == "CASE_SPLIT_DOES_NOT_MATCH_FAMILY_PLAN")
        );
    }

    #[test]
    fn scenario_seed_reuse_across_splits_is_rejected() {
        let (mut plan, digest) = load_plan();
        let train_seed = plan
            .cases
            .iter()
            .find(|case| case.split == "train")
            .unwrap()
            .perturbation
            .scenario_seed
            .clone();
        let test = plan
            .cases
            .iter_mut()
            .find(|case| case.split == "test")
            .unwrap();
        test.perturbation.scenario_seed = train_seed;
        let receipt = evaluate_split_plan(&plan, &digest);
        assert_eq!(receipt.run_state, "Rejected");
        assert!(
            receipt
                .issues
                .iter()
                .any(|issue| issue.code == "SCENARIO_SEED_CROSSES_SPLITS")
        );
    }

    #[test]
    fn identical_observation_signature_across_splits_is_rejected() {
        let (mut plan, digest) = load_plan();
        let train_readings = plan
            .cases
            .iter()
            .find(|case| case.split == "train")
            .unwrap()
            .sensor_readings
            .clone();
        let test = plan
            .cases
            .iter_mut()
            .find(|case| case.split == "test")
            .unwrap();
        test.sensor_readings = train_readings;
        let receipt = evaluate_split_plan(&plan, &digest);
        assert_eq!(receipt.run_state, "Rejected");
        assert!(
            receipt
                .issues
                .iter()
                .any(|issue| issue.code == "OBSERVATION_SIGNATURE_CROSSES_SPLITS")
        );
    }

    #[test]
    fn duplicate_case_and_unknown_sensor_node_fail_closed() {
        let (mut plan, digest) = load_plan();
        plan.cases[1].case_id = plan.cases[0].case_id.clone();
        plan.cases[2].sensor_readings[0].node_id = "MISSING-NODE".to_owned();
        let receipt = evaluate_split_plan(&plan, &digest);
        assert_eq!(receipt.run_state, "Rejected");
        assert!(
            receipt
                .issues
                .iter()
                .any(|issue| issue.code == "CASE_ID_DUPLICATE")
        );
        assert!(
            receipt
                .issues
                .iter()
                .any(|issue| issue.code == "SENSOR_NODE_UNKNOWN")
        );
    }

    #[test]
    fn reviewed_topology_family_may_contain_multiple_graph_variants() {
        let (mut plan, digest) = load_plan();
        let mut variant = plan
            .cases
            .iter()
            .find(|case| case.split == "train")
            .unwrap()
            .clone();
        variant.case_id = "SRC-TOPOLOGY-VARIANT".to_owned();
        variant.perturbation.scenario_seed = "seed-104".to_owned();
        variant.topology.nodes.push("N5".to_owned());
        variant.topology.links.push(TopologyLink {
            link_id: "L4".to_owned(),
            from_node_id: "N3".to_owned(),
            to_node_id: "N5".to_owned(),
        });
        variant.sensor_readings.push(SensorReading {
            node_id: "N5".to_owned(),
            state: "observed".to_owned(),
            pressure_delta_psi: Some(-0.02),
        });
        plan.cases.push(variant);

        let receipt = evaluate_split_plan(&plan, &digest);
        assert_eq!(receipt.run_state, "Passed", "{:?}", receipt.issues);
        assert_eq!(receipt.case_count, 10);
        let train_family = receipt
            .family_counts
            .iter()
            .find(|family| family.topology_family_id == "source-tree")
            .unwrap();
        assert_eq!(train_family.case_count, 4);
        assert_eq!(train_family.topology_variants.len(), 2);
        assert_eq!(
            train_family
                .topology_variants
                .iter()
                .map(|variant| variant.case_count)
                .sum::<usize>(),
            4
        );
    }

    #[test]
    fn structural_signature_ignores_node_and_link_labels_but_exact_digest_does_not() {
        let (plan, _) = load_plan();
        let original = plan.cases[0].topology.clone();
        let mut renamed = original.clone();
        let mapping = BTreeMap::from([
            ("N1", "X9"),
            ("N2", "X2"),
            ("N3", "X7"),
            ("N4", "X1"),
        ]);
        renamed.nodes = renamed.nodes.iter()
            .map(|node| mapping[node.as_str()].to_owned())
            .collect();
        for (index, link) in renamed.links.iter_mut().enumerate() {
            link.link_id = format!("pipe-renamed-{index}");
            link.from_node_id = mapping[link.from_node_id.as_str()].to_owned();
            link.to_node_id = mapping[link.to_node_id.as_str()].to_owned();
        }
        assert_ne!(
            topology_sha256(&original).unwrap(),
            topology_sha256(&renamed).unwrap(),
            "the source-identity digest should remain label-sensitive"
        );
        assert_eq!(
            structural_topology_sha256(&original).unwrap(),
            structural_topology_sha256(&renamed).unwrap(),
            "structural candidate signature must ignore names and link IDs"
        );
    }

    #[test]
    fn relabeled_isomorphic_graph_across_splits_requires_curator_review() {
        let (mut plan, digest) = load_plan();
        let mut variant = plan.cases[0].clone();
        let mapping = BTreeMap::from([
            ("N1", "X9"),
            ("N2", "X2"),
            ("N3", "X7"),
            ("N4", "X1"),
        ]);
        variant.case_id = "TEST-RELABELLED-SOURCE".to_owned();
        variant.split = "test".to_owned();
        variant.topology_family_id = "relabeled-source-tree".to_owned();
        variant.perturbation.scenario_seed = "seed-relabeled-source-unique".to_owned();
        variant.topology.nodes = variant.topology.nodes.iter()
            .map(|node| mapping[node.as_str()].to_owned())
            .collect();
        for (index, link) in variant.topology.links.iter_mut().enumerate() {
            link.link_id = format!("pipe-renamed-{index}");
            link.from_node_id = mapping[link.from_node_id.as_str()].to_owned();
            link.to_node_id = mapping[link.to_node_id.as_str()].to_owned();
        }
        if let Some(target) = variant.perturbation.target_node_id.as_mut() {
            *target = mapping[target.as_str()].to_owned();
        }
        for reading in &mut variant.sensor_readings {
            reading.node_id = mapping[reading.node_id.as_str()].to_owned();
        }
        plan.test_topology_families
            .push("relabeled-source-tree".to_owned());
        plan.cases.push(variant);

        let receipt = evaluate_split_plan(&plan, &digest);
        assert_eq!(receipt.issues.len(), 0, "{:?}", receipt.issues);
        assert_eq!(receipt.run_state, "NeedsTopologyReview");
        assert_eq!(receipt.topology_review_candidates.len(), 1);
        let candidate = &receipt.topology_review_candidates[0];
        assert!(candidate.family_a == "source-tree" || candidate.family_b == "source-tree");
        assert!(candidate.family_a == "relabeled-source-tree" || candidate.family_b == "relabeled-source-tree");
        assert!(candidate.split_a != candidate.split_b);
        assert!(candidate.case_ids_a.contains(&"TEST-RELABELLED-SOURCE".to_owned())
            || candidate.case_ids_b.contains(&"TEST-RELABELLED-SOURCE".to_owned()));
    }

    #[test]
    fn exact_duplicate_topology_cannot_have_two_family_ids() {
        let (mut plan, digest) = load_plan();
        let mut copy = plan.cases[0].clone();
        copy.case_id = "duplicate-exact-graph".to_owned();
        copy.topology_family_id = "separate-but-same-graph".to_owned();
        copy.split = "train".to_owned();
        copy.perturbation.scenario_seed = "unique-copy-seed".to_owned();
        plan.train_topology_families
            .push("separate-but-same-graph".to_owned());
        plan.cases.push(copy);
        let receipt = evaluate_split_plan(&plan, &digest);
        assert_eq!(receipt.run_state, "Rejected");
        assert!(
            receipt
                .issues
                .iter()
                .any(|issue| issue.code == "EXACT_TOPOLOGY_HAS_MULTIPLE_FAMILY_IDS")
        );
    }
}
