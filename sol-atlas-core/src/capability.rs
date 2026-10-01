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
