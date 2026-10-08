// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Renderer-neutral capability semantics for the Humanity/AI Bootstrap Atlas.
//!
//! The crucial boundaries are structural:
//! - a capability definition is not a deployed instance;
//! - a location is not evidence of operation;
//! - operation is not independent reproducibility;
//! - reproducibility is not sustainability;
//! - evidence is not qualification;
//! - an alternative is not an automatically selected substitute.
//!
//! Sol Atlas consumes these types for exploration/rendering. It does not
//! become the authority for CIV-BOOT qualification or real-world truth.

use crate::types::DataKind;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct CapabilityId(String);

impl CapabilityId {
    pub fn new(id: impl Into<String>) -> Result<Self, CapabilityIdError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(CapabilityIdError::Empty);
        }
        if id.chars().any(char::is_whitespace) {
            return Err(CapabilityIdError::Whitespace);
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CapabilityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        Self::new(id).map_err(<D::Error as serde::de::Error>::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityIdError {
    Empty,
    Whitespace,
}

impl fmt::Display for CapabilityIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("capability id must not be empty"),
            Self::Whitespace => f.write_str("capability id must not contain whitespace"),
        }
    }
}

impl std::error::Error for CapabilityIdError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProvenance {
    pub source: String,
    #[serde(default)]
    pub snapshot_date: Option<String>,
    pub kind: DataKind,
    pub scope: String,
    pub claim_ceiling: String,
}

impl CapabilityProvenance {
    pub fn synthetic_fixture() -> Self {
        Self {
            source: "Sol Atlas deterministic bootstrap fixture".into(),
            snapshot_date: None,
            kind: DataKind::Scenario,
            scope: "local fixture graph only".into(),
            claim_ceiling:
                "synthetic structure; not evidence of real-world availability, viability, or qualification"
                    .into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEvidence {
    pub evidence_id: String,
    pub provenance: CapabilityProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanAiContribution {
    pub human_role: String,
    pub ai_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityAlternative {
    pub candidate: CapabilityId,
    pub replaces: CapabilityId,
    pub rationale: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub id: CapabilityId,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required_dependencies: Vec<CapabilityId>,
    #[serde(default)]
    pub alternatives: Vec<CapabilityAlternative>,
    #[serde(default)]
    pub evidence: Vec<CapabilityEvidence>,
    pub human_ai: HumanAiContribution,
    pub provenance: CapabilityProvenance,
    pub claim_ceiling: String,
}

impl Capability {
    pub fn sorted_required_dependencies(&self) -> Vec<CapabilityId> {
        let mut deps = self.required_dependencies.clone();
        deps.sort();
        deps.dedup();
        deps
    }

    pub fn sorted_alternatives(&self) -> Vec<CapabilityAlternative> {
        let mut alternatives = self.alternatives.clone();
        alternatives.sort_by(|a, b| {
            a.candidate
                .cmp(&b.candidate)
                .then_with(|| a.replaces.cmp(&b.replaces))
                .then_with(|| a.rationale.cmp(&b.rationale))
        });
        alternatives
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapabilityLocation {
    label: String,
    lat: f64,
    lon: f64,
    elevation_m: Option<f64>,
}

impl CapabilityLocation {
    pub fn new(
        label: impl Into<String>,
        lat: f64,
        lon: f64,
    ) -> Result<Self, CapabilityLocationError> {
        if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) {
            return Err(CapabilityLocationError::Latitude(lat));
        }
        if !lon.is_finite() || !(-180.0..=180.0).contains(&lon) {
            return Err(CapabilityLocationError::Longitude(lon));
        }
        Ok(Self {
            label: label.into(),
            lat,
            lon,
            elevation_m: None,
        })
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lon(&self) -> f64 {
        self.lon
    }

    pub fn elevation_m(&self) -> Option<f64> {
        self.elevation_m
    }

    pub fn with_elevation_m(
        mut self,
        elevation_m: f64,
    ) -> Result<Self, CapabilityLocationError> {
        if !elevation_m.is_finite() {
            return Err(CapabilityLocationError::Elevation(elevation_m));
        }
        self.elevation_m = Some(elevation_m);
        Ok(self)
    }
}

#[derive(Deserialize)]
struct CapabilityLocationUnchecked {
    label: String,
    lat: f64,
    lon: f64,
    #[serde(default)]
    elevation_m: Option<f64>,
}

impl<'de> Deserialize<'de> for CapabilityLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = CapabilityLocationUnchecked::deserialize(deserializer)?;
        let mut location = Self::new(raw.label, raw.lat, raw.lon)
            .map_err(<D::Error as serde::de::Error>::custom)?;
        if let Some(elevation_m) = raw.elevation_m {
            location = location
                .with_elevation_m(elevation_m)
                .map_err(<D::Error as serde::de::Error>::custom)?;
        }
        Ok(location)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CapabilityLocationError {
    Latitude(f64),
    Longitude(f64),
    Elevation(f64),
}

impl fmt::Display for CapabilityLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Latitude(v) => write!(f, "invalid latitude: {v}"),
            Self::Longitude(v) => write!(f, "invalid longitude: {v}"),
            Self::Elevation(v) => write!(f, "invalid elevation: {v}"),
        }
    }
}

impl std::error::Error for CapabilityLocationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityLifecycle {
    Planned,
    UnderConstruction,
    Operational,
    Suspended,
    Retired,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityAvailability {
    Available,
    Degraded,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityInstanceState {
    pub lifecycle: CapabilityLifecycle,
    pub availability: CapabilityAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalProfile {
    #[serde(default)]
    pub service_description: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceProfile {
    #[serde(default)]
    pub required_capabilities: Vec<CapabilityId>,
    #[serde(default)]
    pub recovery_class: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproducibilityProfile {
    pub demonstrated: bool,
    #[serde(default)]
    pub independent_demonstrations: u32,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualificationRef {
    pub authority: String,
    pub reference: String,
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityInstance {
    pub instance_id: String,
    pub capability_id: CapabilityId,
    pub location: CapabilityLocation,
    pub state: CapabilityInstanceState,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub operational_profile: OperationalProfile,
    pub maintenance_profile: MaintenanceProfile,
    pub reproducibility_profile: ReproducibilityProfile,
    #[serde(default)]
    pub qualification: Option<QualificationRef>,
    pub provenance: CapabilityProvenance,
    pub claim_ceiling: String,
}

impl CapabilityInstance {
    pub fn is_qualified(&self) -> bool {
        self.qualification.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGraph {
    capabilities: BTreeMap<CapabilityId, Capability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityGraphError {
    DuplicateId(CapabilityId),
    MissingRoot(CapabilityId),
    MissingDependency {
        capability: CapabilityId,
        dependency: CapabilityId,
    },
    MissingAlternativeTarget {
        capability: CapabilityId,
        target: CapabilityId,
    },
    AlternativeReplacementMismatch {
        capability: CapabilityId,
        replaces: CapabilityId,
    },
    SelfAlternative {
        capability: CapabilityId,
    },
    Cycle {
        capability: CapabilityId,
    },
}

impl fmt::Display for CapabilityGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate capability id: {id}"),
            Self::MissingRoot(id) => write!(f, "root capability not found: {id}"),
            Self::MissingDependency { capability, dependency } => {
                write!(f, "capability {capability} depends on missing {dependency}")
            }
            Self::MissingAlternativeTarget { capability, target } => {
                write!(f, "capability {capability} references missing alternative target {target}")
            }
            Self::AlternativeReplacementMismatch { capability, replaces } => {
                write!(f, "alternative declared on {capability} cannot replace unrelated capability {replaces}")
            }
            Self::SelfAlternative { capability } => {
                write!(f, "capability {capability} cannot be its own alternative")
            }
            Self::Cycle { capability } => {
                write!(f, "dependency cycle detected through {capability}")
            }
        }
    }
}

impl CapabilityGraph {
    pub fn new(
        capabilities: impl IntoIterator<Item = Capability>,
    ) -> Result<Self, CapabilityGraphError> {
        let mut map = BTreeMap::new();
        for capability in capabilities {
            let id = capability.id.clone();
            if map.insert(id.clone(), capability).is_some() {
                return Err(CapabilityGraphError::DuplicateId(id));
            }
        }

        for capability in map.values() {
            for dependency in capability.sorted_required_dependencies() {
                if !map.contains_key(&dependency) {
                    return Err(CapabilityGraphError::MissingDependency {
                        capability: capability.id.clone(),
                        dependency,
                    });
                }
            }
            for alternative in capability.sorted_alternatives() {
                if alternative.replaces != capability.id {
                    return Err(CapabilityGraphError::AlternativeReplacementMismatch {
                        capability: capability.id.clone(),
                        replaces: alternative.replaces,
                    });
                }
                if alternative.candidate == capability.id {
                    return Err(CapabilityGraphError::SelfAlternative {
                        capability: capability.id.clone(),
                    });
                }
                for target in [&alternative.candidate, &alternative.replaces] {
                    if !map.contains_key(target) {
                        return Err(CapabilityGraphError::MissingAlternativeTarget {
                            capability: capability.id.clone(),
                            target: target.clone(),
                        });
                    }
                }
            }
        }

        Ok(Self { capabilities: map })
    }

    pub fn get(&self, id: &CapabilityId) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    pub fn capabilities(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.values()
    }

    pub fn required_closure(
        &self,
        root: &CapabilityId,
    ) -> Result<Vec<CapabilityId>, CapabilityGraphError> {
        if !self.capabilities.contains_key(root) {
            return Err(CapabilityGraphError::MissingRoot(root.clone()));
        }

        // Iterative DFS avoids stack overflow on long, valid bootstrap chains.
        // 0/None = unseen, 1 = active on the current DFS path, 2 = complete.
        let mut colors: BTreeMap<CapabilityId, u8> = BTreeMap::new();
        let mut visited = BTreeSet::new();
        let mut stack = vec![(root.clone(), false)];

        while let Some((id, exiting)) = stack.pop() {
            if exiting {
                colors.insert(id.clone(), 2);
                visited.insert(id);
                continue;
            }

            match colors.get(&id).copied() {
                Some(2) => continue,
                Some(1) => {
                    return Err(CapabilityGraphError::Cycle { capability: id });
                }
                _ => {}
            }

            colors.insert(id.clone(), 1);
            stack.push((id.clone(), true));

            let capability = self
                .capabilities
                .get(&id)
                .expect("validated by CapabilityGraph::new");

            // Reverse push preserves ascending DFS traversal even though the
            // final closure is sorted independently for a stable public result.
            for dependency in capability
                .sorted_required_dependencies()
                .into_iter()
                .rev()
            {
                match colors.get(&dependency).copied() {
                    Some(1) => {
                        return Err(CapabilityGraphError::Cycle {
                            capability: dependency,
                        });
                    }
                    Some(2) => {}
                    _ => stack.push((dependency, false)),
                }
            }
        }

        Ok(visited.into_iter().collect())
    }

    pub fn deterministic_json(&self) -> Result<String, serde_json::Error> {
        let ordered: Vec<&Capability> = self.capabilities.values().collect();
        serde_json::to_string(&ordered)
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapFixture {
    pub graph: CapabilityGraph,
    pub root: CapabilityId,
    pub instance: CapabilityInstance,
}

pub fn bootstrap_fixture() -> BootstrapFixture {
    let provenance = CapabilityProvenance::synthetic_fixture();
    let claim_ceiling = provenance.claim_ceiling.clone();
    let evidence = CapabilityEvidence {
        evidence_id: "fixture.synthetic.bootstrap".into(),
        provenance: provenance.clone(),
    };
    let ids = |s: &str| CapabilityId::new(s).expect("fixture ids are valid");
    let human_ai = |human: &str, ai: &str| HumanAiContribution {
        human_role: human.into(),
        ai_role: ai.into(),
    };

    let capabilities = vec![
        Capability {
            id: ids("water-purification"),
            name: "Water purification".into(),
            description: "Synthetic bootstrap capability target.".into(),
            required_dependencies: vec![ids("energy")],
            alternatives: vec![],
            evidence: vec![evidence.clone()],
            human_ai: human_ai("fixture-defined operator", "fixture-defined analysis assistance"),
            provenance: provenance.clone(),
            claim_ceiling: claim_ceiling.clone(),
        },
        Capability {
            id: ids("energy"),
            name: "Energy".into(),
            description: "Synthetic upstream energy capability.".into(),
            required_dependencies: vec![ids("materials")],
            alternatives: vec![],
            evidence: vec![evidence.clone()],
            human_ai: human_ai("fixture-defined maintainer", "fixture-defined planning assistance"),
            provenance: provenance.clone(),
            claim_ceiling: claim_ceiling.clone(),
        },
        Capability {
            id: ids("materials"),
            name: "Materials".into(),
            description: "Synthetic material supply capability.".into(),
            required_dependencies: vec![ids("manufacturing")],
            alternatives: vec![],
            evidence: vec![evidence.clone()],
            human_ai: human_ai("fixture-defined fabricator", "fixture-defined design assistance"),
            provenance: provenance.clone(),
            claim_ceiling: claim_ceiling.clone(),
        },
        Capability {
            id: ids("manufacturing"),
            name: "Manufacturing".into(),
            description: "Synthetic manufacturing capability.".into(),
            required_dependencies: vec![ids("maintenance")],
            alternatives: vec![],
            evidence: vec![evidence.clone()],
            human_ai: human_ai("fixture-defined craft", "fixture-defined process assistance"),
            provenance: provenance.clone(),
            claim_ceiling: claim_ceiling.clone(),
        },
        Capability {
            id: ids("maintenance"),
            name: "Maintenance".into(),
            description: "Synthetic repair and upkeep capability.".into(),
            required_dependencies: vec![ids("knowledge")],
            alternatives: vec![],
            evidence: vec![evidence.clone()],
            human_ai: human_ai("fixture-defined repair", "fixture-defined diagnostic assistance"),
            provenance: provenance.clone(),
            claim_ceiling: claim_ceiling.clone(),
        },
        Capability {
            id: ids("knowledge"),
            name: "Knowledge".into(),
            description: "Synthetic knowledge capability.".into(),
            required_dependencies: vec![ids("evidence")],
            alternatives: vec![],
            evidence: vec![evidence.clone()],
            human_ai: human_ai("fixture-defined learning", "fixture-defined retrieval assistance"),
            provenance: provenance.clone(),
            claim_ceiling: claim_ceiling.clone(),
        },
        Capability {
            id: ids("evidence"),
            name: "Evidence".into(),
            description: "Synthetic evidence-handling capability.".into(),
            required_dependencies: vec![],
            alternatives: vec![],
            evidence: vec![evidence],
            human_ai: human_ai("fixture-defined observation", "fixture-defined organization"),
            provenance: provenance.clone(),
            claim_ceiling: claim_ceiling.clone(),
        },
    ];

    let graph = CapabilityGraph::new(capabilities).expect("synthetic fixture is valid");
    let root = ids("water-purification");
    let instance = CapabilityInstance {
        instance_id: "fixture.bootstrap.node".into(),
        capability_id: root.clone(),
        location: CapabilityLocation::new("Synthetic bootstrap node", 0.0, 0.0)
            .expect("fixture coordinates are valid"),
        state: CapabilityInstanceState {
            lifecycle: CapabilityLifecycle::Planned,
            availability: CapabilityAvailability::Unknown,
        },
        evidence_refs: vec!["fixture.synthetic.bootstrap".into()],
        operational_profile: OperationalProfile {
            service_description: None,
            evidence_refs: vec![],
        },
        maintenance_profile: MaintenanceProfile {
            required_capabilities: vec![ids("maintenance")],
            recovery_class: None,
            evidence_refs: vec![],
        },
        reproducibility_profile: ReproducibilityProfile {
            demonstrated: false,
            independent_demonstrations: 0,
            evidence_refs: vec![],
            claim_ceiling: claim_ceiling.clone(),
        },
        qualification: None,
        provenance,
        claim_ceiling: "synthetic fixture only; qualification is intentionally absent".into(),
    };

    BootstrapFixture { graph, root, instance }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_id_rejects_empty_and_whitespace() {
        assert_eq!(CapabilityId::new("").unwrap_err(), CapabilityIdError::Empty);
        assert_eq!(
            CapabilityId::new("water purification").unwrap_err(),
            CapabilityIdError::Whitespace
        );
    }

    #[test]
    fn location_is_validated() {
        assert!(CapabilityLocation::new("ok", 90.0, 180.0).is_ok());
        assert!(CapabilityLocation::new("bad", 90.1, 0.0).is_err());
        assert!(CapabilityLocation::new("bad", 0.0, 180.1).is_err());
        assert!(CapabilityLocation::new("bad", f64::NAN, 0.0).is_err());
        assert!(CapabilityLocation::new("bad", 0.0, 0.0)
            .unwrap()
            .with_elevation_m(f64::INFINITY)
            .is_err());
    }

    #[test]
    fn deserialization_cannot_bypass_identity_or_location_validation() {
        assert!(serde_json::from_str::<CapabilityId>(r#""""#).is_err());
        assert!(serde_json::from_str::<CapabilityId>(r#""water purification""#).is_err());
        assert!(serde_json::from_str::<CapabilityLocation>(
            r#"{"label":"bad","lat":90.1,"lon":0.0}"#
        )
        .is_err());
        assert!(serde_json::from_str::<CapabilityLocation>(
            r#"{"label":"bad","lat":0.0,"lon":0.0,"elevation_m":1e999}"#
        )
        .is_err());
    }

    fn fixture_definition(id: &str, dependency: Option<&str>) -> Capability {
        Capability {
            id: CapabilityId::new(id).unwrap(),
            name: id.into(),
            description: "test".into(),
            required_dependencies: dependency
                .map(|v| vec![CapabilityId::new(v).unwrap()])
                .unwrap_or_default(),
            alternatives: vec![],
            evidence: vec![],
            human_ai: HumanAiContribution {
                human_role: "h".into(),
                ai_role: "a".into(),
            },
            provenance: CapabilityProvenance::synthetic_fixture(),
            claim_ceiling: "test-only".into(),
        }
    }

    #[test]
    fn required_closure_is_insertion_order_independent() {
        let a = fixture_definition("a", Some("b"));
        let b = fixture_definition("b", Some("c"));
        let c = fixture_definition("c", None);
        let g1 = CapabilityGraph::new(vec![a.clone(), b.clone(), c.clone()]).unwrap();
        let g2 = CapabilityGraph::new(vec![c, a, b]).unwrap();
        let root = CapabilityId::new("a").unwrap();
        assert_eq!(
            g1.required_closure(&root).unwrap(),
            vec![
                CapabilityId::new("a").unwrap(),
                CapabilityId::new("b").unwrap(),
                CapabilityId::new("c").unwrap()
            ]
        );
        assert_eq!(g1.required_closure(&root), g2.required_closure(&root));
    }

    #[test]
    fn required_closure_handles_deep_chains_without_recursion() {
        const DEPTH: usize = 10_000;
        let capabilities = (0..DEPTH)
            .map(|index| {
                let dependency = (index + 1 < DEPTH)
                    .then(|| CapabilityId::new(format!("cap-{index_plus_one:05}", index_plus_one = index + 1)).unwrap());
                Capability {
                    id: CapabilityId::new(format!("cap-{index:05}")).unwrap(),
                    name: format!("Capability {index}"),
                    description: "deep-chain regression fixture".into(),
                    required_dependencies: dependency.into_iter().collect(),
                    alternatives: vec![],
                    evidence: vec![],
                    human_ai: HumanAiContribution {
                        human_role: "test".into(),
                        ai_role: "test".into(),
                    },
                    provenance: CapabilityProvenance::synthetic_fixture(),
                    claim_ceiling: "test-only".into(),
                }
            })
            .collect::<Vec<_>>();
        let graph = CapabilityGraph::new(capabilities).unwrap();
        let root = CapabilityId::new("cap-00000").unwrap();
        assert_eq!(graph.required_closure(&root).unwrap().len(), DEPTH);
    }

    #[test]
    fn missing_dependency_fails_closed() {
        let a = fixture_definition("a", Some("missing"));
        let err = CapabilityGraph::new(vec![a]).unwrap_err();
        assert!(matches!(err, CapabilityGraphError::MissingDependency { .. }));
    }

    #[test]
    fn cycles_fail_closed() {
        let a = fixture_definition("a", Some("b"));
        let b = fixture_definition("b", Some("a"));
        let graph = CapabilityGraph::new(vec![a, b]).unwrap();
        let root = CapabilityId::new("a").unwrap();
        assert!(matches!(
            graph.required_closure(&root),
            Err(CapabilityGraphError::Cycle { .. })
        ));
    }

    #[test]
    fn dangling_alternatives_fail_closed() {
        let mut base = fixture_definition("base", None);
        base.alternatives.push(CapabilityAlternative {
            candidate: CapabilityId::new("missing").unwrap(),
            replaces: base.id.clone(),
            rationale: "test".into(),
            evidence_refs: vec![],
        });
        assert!(matches!(
            CapabilityGraph::new(vec![base]),
            Err(CapabilityGraphError::MissingAlternativeTarget { .. })
        ));
    }

    #[test]
    fn self_alternatives_fail_closed() {
        let mut base = fixture_definition("base", None);
        base.alternatives.push(CapabilityAlternative {
            candidate: base.id.clone(),
            replaces: base.id.clone(),
            rationale: "test".into(),
            evidence_refs: vec![],
        });
        assert!(matches!(
            CapabilityGraph::new(vec![base]),
            Err(CapabilityGraphError::SelfAlternative { .. })
        ));
    }

    #[test]
    fn alternative_cannot_replace_an_unrelated_capability() {
        let mut base = fixture_definition("base", None);
        let other = fixture_definition("other", None);
        let candidate = fixture_definition("candidate", None);
        base.alternatives.push(CapabilityAlternative {
            candidate: candidate.id.clone(),
            replaces: other.id.clone(),
            rationale: "invalid cross-target alternative".into(),
            evidence_refs: vec![],
        });
        let err = CapabilityGraph::new(vec![base, other, candidate]).unwrap_err();
        assert!(matches!(
            err,
            CapabilityGraphError::AlternativeReplacementMismatch { .. }
        ));
    }

    #[test]
    fn qualification_is_independent_from_instance_existence() {
        let fixture = bootstrap_fixture();
        assert!(!fixture.instance.is_qualified());
        assert_eq!(fixture.instance.capability_id, fixture.root);
    }

    #[test]
    fn bootstrap_fixture_has_declared_chain_only() {
        let fixture = bootstrap_fixture();
        let closure = fixture.graph.required_closure(&fixture.root).unwrap();
        let names = closure.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "evidence",
                "energy",
                "knowledge",
                "maintenance",
                "manufacturing",
                "materials",
                "water-purification"
            ]
        );
    }

    #[test]
    fn alternative_candidates_are_not_auto_selected() {
        let base = fixture_definition("base", None);
        let candidate = fixture_definition("candidate", None);
        let mut base_with_alt = base;
        base_with_alt.alternatives.push(CapabilityAlternative {
            candidate: candidate.id.clone(),
            replaces: base_with_alt.id.clone(),
            rationale: "test alternative".into(),
            evidence_refs: vec![],
        });
        let graph = CapabilityGraph::new(vec![candidate, base_with_alt]).unwrap();
        let definition = graph.get(&CapabilityId::new("base").unwrap()).unwrap();
        assert_eq!(definition.sorted_alternatives().len(), 1);
        assert_eq!(definition.sorted_alternatives()[0].candidate.as_str(), "candidate");
    }

    #[test]
    fn synthetic_fixture_cannot_be_mistaken_for_observed_evidence() {
        let fixture = bootstrap_fixture();
        for capability in fixture.graph.capabilities() {
            assert_eq!(capability.provenance.kind, DataKind::Scenario);
            assert!(capability.claim_ceiling.contains("not evidence of real-world"));
        }
        assert_eq!(fixture.instance.provenance.kind, DataKind::Scenario);
        assert!(!fixture.instance.is_qualified());
    }

    #[test]
    fn deterministic_json_is_stable_across_insertion_order() {
        let a = fixture_definition("a", Some("b"));
        let b = fixture_definition("b", None);
        let g1 = CapabilityGraph::new(vec![a.clone(), b.clone()]).unwrap();
        let g2 = CapabilityGraph::new(vec![b, a]).unwrap();
        assert_eq!(
            g1.deterministic_json().unwrap(),
            g2.deterministic_json().unwrap()
        );

        let fixture = bootstrap_fixture();
        let serialized = fixture.graph.deterministic_json().unwrap();
        assert!(serialized.starts_with("[{"));
    }
}
