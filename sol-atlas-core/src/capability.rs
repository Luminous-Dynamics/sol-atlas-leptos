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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceKind {
    Observed,
    Curated,
    Scenario,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    Required,
    Enabling,
    Maintenance,
    Knowledge,
    Energy,
    Material,
    Alternative,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDependency {
    pub capability: CapabilityId,
    pub relation: DependencyKind,
    pub required: bool,
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
                required: true,
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
    fn dependency_relation_and_substitute_remain_distinct() {
        let c = capability();
        let dep = &c.dependencies[0];
        assert_eq!(dep.relation, DependencyKind::Energy);
        assert!(dep.required);
        assert_eq!(dep.substitutes.len(), 1);
    }

    #[test]
    fn human_and_ai_contributions_remain_separate() {
        let c = capability();
        assert!(!c.contribution.human.is_empty());
        assert!(!c.contribution.ai.is_empty());
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGraphError {
    pub missing: Vec<CapabilityId>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CapabilityGraph {
    pub capabilities: Vec<Capability>,
}

impl CapabilityGraph {
    /// Compute the deterministic transitive dependency closure of a root.
    ///
    /// Only dependencies marked `required` participate. Substitutes are
    /// reported as metadata but are not silently selected by the closure.
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

            for dependency in capability.dependencies.iter().filter(|d| d.required) {
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
                    required: true,
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
            required: false,
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
