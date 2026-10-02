// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Renderer-neutral capability graph primitives.
//!
//! This module is deliberately semantic rather than geographic: a capability
//! can be rendered by Sol Atlas, qualified by Mycelix CIV-BOOT, simulated by
//! Symtropy, or implemented by another system without making any renderer
//! the source of truth.
//!
//! Important semantic boundaries:
//! - capability != authority
//! - visualization != evidence
//! - evidence != qualification
//! - a deployed instance != universal availability

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceKind {
    Observed,
    Curated,
    Scenario,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    /// A hard dependency that must be present for the modeled capability.
    Required,
    /// A context/enabler that may support a capability without entering its hard closure.
    Enabling,
    /// A maintenance capability required to sustain the modeled capability.
    Maintenance,
    /// Required knowledge needed by the modeled capability.
    Knowledge,
    /// Required energy capability needed by the modeled capability.
    Energy,
    /// Required material capability needed by the modeled capability.
    Material,
    /// A documented alternative path; never selected implicitly.
    Alternative,
}

impl DependencyKind {
    /// Whether this relation participates in the required capability closure.
    ///
    /// Requirement semantics live in the relation kind itself. There is
    /// intentionally no second boolean source of truth.
    pub fn is_required(self) -> bool {
        matches!(
            self,
            Self::Required | Self::Maintenance | Self::Knowledge | Self::Energy | Self::Material
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityState {
    Conceptual,
    Demonstrated,
    Deployed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEvidence {
    pub kind: EvidenceKind,
    pub reference: String,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProvenance {
    pub source: String,
    pub snapshot: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityLocation {
    pub id: String,
    pub label: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityQualification {
    /// External qualification authority, e.g. a CIV-BOOT adapter.
    pub authority: String,
    /// Stable reference to the qualification artifact.
    pub reference: String,
    /// Exact scope for which the qualification applies.
    pub scope: String,
    /// Qualification-specific claim ceiling.
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceState {
    Planned,
    Installed,
    Operational,
    Maintenance,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityInstance {
    /// Stable identity for one concrete instantiation of a capability.
    pub id: String,
    /// Abstract capability this instance realizes.
    pub capability: CapabilityId,
    /// Geographic placement; presence alone is not operational evidence.
    pub location: CapabilityLocation,
    /// Current declared lifecycle state of this instance.
    pub state: InstanceState,
    /// Evidence references scoped to this concrete instance.
    pub evidence: Vec<String>,
    /// Optional external qualification scoped to this instance.
    pub qualification: Option<CapabilityQualification>,
}

impl CapabilityInstance {
    /// Whether this concrete instance has any attached evidence references.
    ///
    /// Evidence presence is intentionally independent from lifecycle state:
    /// an installed, maintained, or retired instance may retain historical
    /// evidence without being operational now.
    pub fn has_evidence(&self) -> bool {
        !self.evidence.is_empty()
    }

    /// Whether the instance is currently declared operational and has evidence.
    ///
    /// This is still not a qualification claim; qualification remains explicit.
    pub fn has_current_operational_evidence(&self) -> bool {
        self.state == InstanceState::Operational && self.has_evidence()
    }

    /// Qualification remains explicit even when an instance is operational.
    pub fn is_qualified(&self) -> bool {
        self.qualification.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDependency {
    pub capability: CapabilityId,
    pub relation: DependencyKind,
    /// Explicit alternative candidates; never silently selected.
    pub substitutes: Vec<CapabilityId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanAiContribution {
    pub human: String,
    pub ai: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub id: CapabilityId,
    pub name: String,
    pub description: String,
    pub state: CapabilityState,
    pub dependencies: Vec<CapabilityDependency>,
    pub evidence: Vec<CapabilityEvidence>,
    pub provenance: Vec<CapabilityProvenance>,
    pub locations: Vec<CapabilityLocation>,
    pub qualification: Option<CapabilityQualification>,
    pub contribution: HumanAiContribution,
}

impl Capability {
    /// Qualification is explicit and scoped; evidence alone never upgrades it.
    pub fn is_qualified(&self) -> bool {
        self.qualification.is_some()
    }

    /// A scenario record is never treated as observational evidence.
    pub fn has_observed_evidence(&self) -> bool {
        self.evidence
            .iter()
            .any(|e| e.kind == EvidenceKind::Observed)
    }

    /// True when the capability has a concrete geographic instance.
    ///
    /// This intentionally says nothing about whether the instance is
    /// operational, independently reproducible, or qualified.
    pub fn has_location(&self) -> bool {
        !self.locations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capability() -> Capability {
        Capability {
            id: CapabilityId("water.purification".into()),
            name: "Water purification".into(),
            description: "Produces potable water from an identified source.".into(),
            state: CapabilityState::Demonstrated,
            dependencies: vec![CapabilityDependency {
                capability: CapabilityId("energy.electricity".into()),
                relation: DependencyKind::Energy,
                substitutes: vec![CapabilityId("energy.mechanical".into())],
            }],
            evidence: vec![CapabilityEvidence {
                kind: EvidenceKind::Observed,
                reference: "fixture-001".into(),
                claim_ceiling: "Exact fixture only.".into(),
            }],
            provenance: vec![CapabilityProvenance {
                source: "fixture".into(),
                snapshot: Some("2026-10-01".into()),
                notes: "Synthetic test fixture; not field validation.".into(),
            }],
            locations: vec![CapabilityLocation {
                id: "node-001".into(),
                label: "Demo node".into(),
                lat: 0.0,
                lon: 0.0,
            }],
            qualification: None,
            contribution: HumanAiContribution {
                human: "Operation, maintenance, judgment".into(),
                ai: "Optional analysis and planning assistance".into(),
            },
        }
    }

    #[test]
    fn evidence_does_not_imply_qualification() {
        assert!(!capability().is_qualified());
        assert!(capability().has_observed_evidence());
    }

    #[test]
    fn qualification_is_explicit() {
        let mut c = capability();
        c.qualification = Some(CapabilityQualification {
            authority: "CIV-BOOT adapter".into(),
            reference: "qual-001".into(),
            scope: "Exact demo node/profile".into(),
            claim_ceiling: "Exact qualified scope only.".into(),
        });
        assert!(c.is_qualified());
    }

    #[test]
    fn location_does_not_imply_availability() {
        let c = capability();
        assert!(c.has_location());
        assert_eq!(c.state, CapabilityState::Demonstrated);
    }

    #[test]
    fn instance_location_does_not_imply_operation_or_qualification() {
        let instance = CapabilityInstance {
            id: "instance-001".into(),
            capability: CapabilityId("water.purification".into()),
            location: CapabilityLocation {
                id: "node-001".into(),
                label: "Demo node".into(),
                lat: 0.0,
                lon: 0.0,
            },
            state: InstanceState::Installed,
            evidence: vec![],
            qualification: None,
        };

        assert!(!instance.has_evidence());
        assert!(!instance.has_current_operational_evidence());
        assert!(!instance.is_qualified());
    }

    #[test]
    fn instance_qualification_is_scoped_and_explicit() {
        let mut instance = CapabilityInstance {
            id: "instance-002".into(),
            capability: CapabilityId("water.purification".into()),
            location: CapabilityLocation {
                id: "node-002".into(),
                label: "Qualified demo node".into(),
                lat: 1.0,
                lon: 1.0,
            },
            state: InstanceState::Operational,
            evidence: vec!["run-002".into()],
            qualification: None,
        };

        assert!(instance.has_evidence());
        assert!(instance.has_current_operational_evidence());
        assert!(!instance.is_qualified());

        instance.qualification = Some(CapabilityQualification {
            authority: "CIV-BOOT adapter".into(),
            reference: "qual-002".into(),
            scope: "instance-002 only".into(),
            claim_ceiling: "Exact instance/profile only.".into(),
        });

        assert!(instance.is_qualified());
    }

    #[test]
    fn dependency_relation_and_substitute_remain_distinct() {
        let c = capability();
        let dep = &c.dependencies[0];
        assert_eq!(dep.relation, DependencyKind::Energy);
        assert!(dep.relation.is_required());
        assert_eq!(dep.substitutes.len(), 1);
    }

    #[test]
    fn human_and_ai_contributions_remain_separate() {
        let c = capability();
        assert!(!c.contribution.human.is_empty());
        assert!(!c.contribution.ai.is_empty());
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternativePath {
    /// The dependency this candidate could replace.
    pub for_dependency: CapabilityId,
    /// An explicitly declared candidate capability.
    pub candidate: CapabilityId,
    /// Evidence/reference supporting the alternative relationship.
    pub evidence: Vec<String>,
    /// What this declaration is allowed to claim.
    pub claim_ceiling: String,
}

impl Capability {
    /// Enumerate explicitly declared alternatives for this capability.
    ///
    /// This is discovery only: no candidate is treated as selected,
    /// equivalent, or operationally interchangeable.
    pub fn alternative_paths(&self) -> Vec<AlternativePath> {
        self.dependencies
            .iter()
            .flat_map(|dependency| {
                dependency.substitutes.iter().cloned().map(|candidate| AlternativePath {
                    for_dependency: dependency.capability.clone(),
                    candidate,
                    evidence: Vec::new(),
                    claim_ceiling: "Declared alternative candidate only; equivalence and operational interchangeability are not established.".into(),
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGraphError {
    pub missing: Vec<CapabilityId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityImpact {
    /// The capability explicitly declared unavailable by the analysis input.
    pub unavailable: CapabilityId,
    /// Capabilities that directly require the unavailable capability.
    pub direct_affected: Vec<CapabilityId>,
    /// Capabilities that depend on the unavailable capability through one or more
    /// intermediate required dependencies.
    pub transitive_affected: Vec<CapabilityId>,
    /// Capabilities whose required closure could not be fully evaluated.
    ///
    /// Unresolved is deliberately separate from affected: incomplete graph data
    /// is not evidence of safety, equivalence, or absence of impact.
    pub unresolved: Vec<CapabilityId>,
    /// Compatibility view of the complete affected set, excluding the unavailable
    /// capability itself. Prefer direct_affected and transitive_affected when the
    /// causal distance matters.
    pub affected: Vec<CapabilityId>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CapabilityGraph {
    pub capabilities: Vec<Capability>,
}

impl CapabilityGraph {
    /// Return capabilities whose required closure depends on an unavailable
    /// capability.
    ///
    /// This is a declared dependency blast radius, not a prediction of real
    /// world impact. Explicit substitutes are intentionally not selected here;
    /// resilience policy remains a separate, auditable decision.
    pub fn affected_by(&self, unavailable: &CapabilityId) -> CapabilityImpact {
        use std::collections::{BTreeMap, BTreeSet, VecDeque};

        let index = self
            .capabilities
            .iter()
            .map(|c| (c.id.clone(), c))
            .collect::<BTreeMap<_, _>>();

        let mut direct_affected = Vec::new();
        let mut transitive_affected = Vec::new();
        let mut unresolved = Vec::new();

        for capability in &self.capabilities {
            if capability.id == *unavailable {
                continue;
            }

            // Analyze reachability specifically toward the unavailable capability.
            // This avoids treating unrelated missing graph data as evidence of
            // unresolved impact.
            let mut queue = VecDeque::from([capability.id.clone()]);
            let mut seen = BTreeSet::new();
            let mut reaches_unavailable = false;
            let mut has_relevant_gap = false;

            while let Some(id) = queue.pop_front() {
                if !seen.insert(id.clone()) {
                    continue;
                }

                if id == *unavailable {
                    reaches_unavailable = true;
                    continue;
                }

                let Some(current) = index.get(&id) else {
                    has_relevant_gap = true;
                    continue;
                };

                for dependency in current.dependencies.iter().filter(|d| d.relation.is_required()) {
                    queue.push_back(dependency.capability.clone());
                }
            }

            if reaches_unavailable {
                if capability.dependencies.iter().any(|dependency| {
                    dependency.relation.is_required() && dependency.capability == *unavailable
                }) {
                    direct_affected.push(capability.id.clone());
                } else {
                    transitive_affected.push(capability.id.clone());
                }
            } else if has_relevant_gap {
                unresolved.push(capability.id.clone());
            }
        }

        direct_affected.sort();
        transitive_affected.sort();
        unresolved.sort();

        let mut affected = direct_affected.clone();
        affected.extend(transitive_affected.iter().cloned());
        affected.sort();

        CapabilityImpact {
            unavailable: unavailable.clone(),
            direct_affected,
            transitive_affected,
            unresolved,
            affected,
        }
    }

    /// Compute the deterministic transitive dependency closure of a root.
    ///
    /// Only dependency relations whose kind is required participate.
    /// Substitutes are metadata and are never silently selected by the closure.
    /// Missing required capabilities are returned as an error instead of
    /// being interpreted as satisfied.
    pub fn required_closure(
        &self,
        root: &CapabilityId,
    ) -> Result<Vec<CapabilityId>, CapabilityGraphError> {
        use std::collections::{BTreeSet, VecDeque};

        let index = self
            .capabilities
            .iter()
            .map(|c| (c.id.clone(), c))
            .collect::<std::collections::BTreeMap<_, _>>();

        let mut queue = VecDeque::from([root.clone()]);
        let mut seen = BTreeSet::new();
        let mut missing = BTreeSet::new();

        while let Some(id) = queue.pop_front() {
            if !seen.insert(id.clone()) {
                continue;
            }

            let Some(capability) = index.get(&id) else {
                missing.insert(id);
                continue;
            };

            for dependency in capability.dependencies.iter().filter(|d| d.relation.is_required()) {
                queue.push_back(dependency.capability.clone());
            }
        }

        if !missing.is_empty() {
            return Err(CapabilityGraphError {
                missing: missing.into_iter().collect(),
            });
        }

        Ok(seen.into_iter().collect())
    }
}

#[cfg(test)]
mod graph_tests {
    use super::*;

    fn cap(id: &str, dependencies: &[&str]) -> Capability {
        Capability {
            id: CapabilityId(id.into()),
            name: id.into(),
            description: String::new(),
            state: CapabilityState::Demonstrated,
            dependencies: dependencies
                .iter()
                .map(|dependency| CapabilityDependency {
                    capability: CapabilityId((*dependency).into()),
                    relation: DependencyKind::Required,
                    substitutes: vec![],
                })
                .collect(),
            evidence: vec![],
            provenance: vec![],
            locations: vec![],
            qualification: None,
            contribution: HumanAiContribution {
                human: String::new(),
                ai: String::new(),
            },
        }
    }

    #[test]
    fn affected_by_reports_declared_dependency_blast_radius() {
        let graph = CapabilityGraph {
            capabilities: vec![
                cap("a", &["b"]),
                cap("b", &["c"]),
                cap("c", &[]),
                cap("independent", &[]),
            ],
        };

        let impact = graph.affected_by(&CapabilityId("c".into()));
        assert_eq!(impact.unavailable, CapabilityId("c".into()));
        assert_eq!(impact.direct_affected, vec![CapabilityId("b".into())]);
        assert_eq!(impact.transitive_affected, vec![CapabilityId("a".into())]);
        assert_eq!(impact.affected, vec![CapabilityId("a".into()), CapabilityId("b".into())]);
        assert!(impact.unresolved.is_empty());

        let independent = graph.affected_by(&CapabilityId("independent".into()));
        assert!(independent.affected.is_empty());
        assert!(independent.direct_affected.is_empty());
        assert!(independent.transitive_affected.is_empty());
    }

    #[test]
    fn alternatives_are_explicit_candidates_not_selections() {
        let mut root = cap("a", &["b"]);
        root.dependencies[0].substitutes = vec![
            CapabilityId("alternative-1".into()),
            CapabilityId("alternative-2".into()),
        ];

        let paths = root.alternative_paths();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].for_dependency, CapabilityId("b".into()));
        assert_eq!(paths[0].candidate, CapabilityId("alternative-1".into()));
        assert!(paths[0].evidence.is_empty());
        assert!(paths[0].claim_ceiling.contains("not established"));
    }

    #[test]
    fn alternatives_are_not_implicitly_selected_by_blast_radius() {
        let mut root = cap("a", &["b"]);
        root.dependencies[0].substitutes = vec![CapabilityId("alternative".into())];

        let graph = CapabilityGraph {
            capabilities: vec![root, cap("b", &[]), cap("alternative", &[])],
        };

        assert_eq!(
            graph.affected_by(&CapabilityId("b".into())).affected,
            vec![CapabilityId("a".into()), CapabilityId("b".into())]
        );
    }

    #[test]
    fn required_closure_is_transitive_and_deterministic() {
        let graph = CapabilityGraph {
            capabilities: vec![
                cap("c", &["b"]),
                cap("a", &["c"]),
                cap("b", &["d"]),
                cap("d", &[]),
            ],
        };

        assert_eq!(
            graph.required_closure(&CapabilityId("a".into())).unwrap(),
            vec![
                CapabilityId("a".into()),
                CapabilityId("b".into()),
                CapabilityId("c".into()),
                CapabilityId("d".into()),
            ]
        );
    }

    #[test]
    fn missing_dependency_is_not_silently_satisfied() {
        let graph = CapabilityGraph {
            capabilities: vec![cap("a", &["missing"])],
        };

        let error = graph.required_closure(&CapabilityId("a".into())).unwrap_err();
        assert_eq!(
            error.missing,
            vec![CapabilityId("missing".into())]
        );
    }

    #[test]
    fn optional_dependencies_do_not_expand_required_closure() {
        let mut root = cap("a", &[]);
        root.dependencies.push(CapabilityDependency {
            capability: CapabilityId("optional".into()),
            relation: DependencyKind::Enabling,
            substitutes: vec![],
        });

        let graph = CapabilityGraph {
            capabilities: vec![root],
        };

        assert_eq!(
            graph.required_closure(&CapabilityId("a".into())).unwrap(),
            vec![CapabilityId("a".into())]
        );
    }
}
