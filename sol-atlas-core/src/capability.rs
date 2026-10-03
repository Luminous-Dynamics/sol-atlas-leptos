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

use crate::evidence_reference::{
    DigestContextV1, EvidenceReferenceCanonicalV1, EvidenceReferenceProfileRegistryV1,
    EvidenceReferenceV1, EvidenceReferenceVerificationV1,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceKind {
    Observed,
    Curated,
    Scenario,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

/// Canonical snapshot of the evidence records consumed for a verification scope.
///
/// The snapshot records structured evidence identity and claim ceilings rather
/// than hashing a caller-provided label. It identifies what was consumed; it
/// does not establish that the referenced evidence is true or sufficient.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSnapshotV1 {
    pub schema: String,
    pub subject: CapabilityId,
    pub evidence: Vec<EvidenceSnapshotEntryV1>,
    pub coverage: RecoveryEvidenceCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSnapshotEntryV1 {
    pub kind: EvidenceKind,
    pub reference: EvidenceReferenceV1,
    pub claim_ceiling: String,
    /// Legacy locators remain visible as unresolved metadata rather than being
    /// silently upgraded into content identity.
    pub unresolved_locator: Option<String>,
}

impl EvidenceSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:evidence-snapshot:v1";

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.subject.0.is_empty()
            && self.evidence.iter().all(|entry| {
                entry.reference.is_well_formed()
                    && entry.claim_ceiling == entry.reference.claim_ceiling
            })
    }

    /// Stable project-specific canonical bytes.
    ///
    /// The digest binds typed content identity, context, and claim ceiling while
    /// deliberately excluding display labels. Unresolved legacy locators are
    /// included separately so their presence cannot collapse into another
    /// unresolved record. This is project-specific canonicalization; it does
    /// not claim RFC 8785/JCS interoperability.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize, PartialEq)]
        struct CanonicalEntry {
            kind: EvidenceKind,
            reference: EvidenceReferenceCanonicalV1,
            claim_ceiling: String,
            unresolved_locator: Option<String>,
        }

        #[derive(Serialize)]
        struct CanonicalSnapshot {
            schema: String,
            subject: CapabilityId,
            evidence: Vec<CanonicalEntry>,
            coverage: RecoveryEvidenceCoverage,
        }

        let mut evidence = self
            .evidence
            .iter()
            .map(|entry| CanonicalEntry {
                kind: entry.kind,
                reference: (&entry.reference).into(),
                claim_ceiling: entry.claim_ceiling.clone(),
                unresolved_locator: entry.unresolved_locator.clone(),
            })
            .collect::<Vec<_>>();
        evidence.sort_by(|left, right| {
            (
                &left.kind,
                &left.reference.artifact_type,
                &left.reference.context,
                &left.reference.digest,
                &left.claim_ceiling,
                &left.unresolved_locator,
            )
                .cmp(&(
                    &right.kind,
                    &right.reference.artifact_type,
                    &right.reference.context,
                    &right.reference.digest,
                    &right.claim_ceiling,
                    &right.unresolved_locator,
                ))
        });
        evidence.dedup();

        let canonical = CanonicalSnapshot {
            schema: self.schema.clone(),
            subject: self.subject.clone(),
            evidence,
            coverage: self.coverage,
        };

        serde_json::to_vec(&canonical)
            .expect("evidence snapshot contains only serializable evidence primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }

    /// Preserve a legacy capability evidence record as an explicitly unresolved
    /// typed reference. No mutable label is promoted into content identity.
    pub fn from_capability(capability: &Capability, coverage: RecoveryEvidenceCoverage) -> Self {
        let mut evidence = capability
            .evidence
            .iter()
            .map(|entry| EvidenceSnapshotEntryV1 {
                kind: entry.kind,
                reference: EvidenceReferenceV1::unresolved_legacy(
                    entry.reference.clone(),
                    entry.claim_ceiling.clone(),
                ),
                claim_ceiling: entry.claim_ceiling.clone(),
                unresolved_locator: Some(entry.reference.clone()),
            })
            .collect::<Vec<_>>();

        evidence.sort_by(|left, right| {
            (&left.kind, &left.claim_ceiling, &left.unresolved_locator).cmp(&(
                &right.kind,
                &right.claim_ceiling,
                &right.unresolved_locator,
            ))
        });
        evidence.dedup();

        Self {
            schema: Self::SCHEMA.into(),
            subject: capability.id.clone(),
            evidence,
            coverage,
        }
    }

    /// Construct a snapshot from already-typed evidence references.
    pub fn from_entries(
        subject: CapabilityId,
        coverage: RecoveryEvidenceCoverage,
        mut evidence: Vec<EvidenceSnapshotEntryV1>,
    ) -> Self {
        evidence.sort_by(|left, right| {
            (
                &left.kind,
                &left.reference.artifact_type,
                &left.reference.context,
                &left.reference.digest,
                &left.claim_ceiling,
                &left.unresolved_locator,
            )
                .cmp(&(
                    &right.kind,
                    &right.reference.artifact_type,
                    &right.reference.context,
                    &right.reference.digest,
                    &right.claim_ceiling,
                    &right.unresolved_locator,
                ))
        });
        evidence.dedup();

        Self {
            schema: Self::SCHEMA.into(),
            subject,
            evidence,
            coverage,
        }
    }

    /// A snapshot can be used as closed-world evidence only when its entries
    /// form an exact set of well-formed typed references with independently
    /// verified digest bindings.
    pub fn all_references_verified(
        &self,
        verifications: &[EvidenceReferenceVerificationV1],
    ) -> bool {
        if self.evidence.len() != verifications.len() {
            return false;
        }

        if !self
            .evidence
            .iter()
            .all(|entry| entry.reference.is_well_formed() && entry.unresolved_locator.is_none())
        {
            return false;
        }

        if !verifications
            .iter()
            .all(EvidenceReferenceVerificationV1::is_verified)
        {
            return false;
        }

        fn key(entry: &EvidenceSnapshotEntryV1) -> String {
            serde_json::to_string(&(
                EvidenceReferenceCanonicalV1::from(&entry.reference),
                entry.claim_ceiling.clone(),
            ))
            .expect("evidence snapshot comparison key is serializable")
        }

        fn verification_key(verification: &EvidenceReferenceVerificationV1) -> String {
            serde_json::to_string(&(
                EvidenceReferenceCanonicalV1::from(&verification.reference),
                verification.reference.claim_ceiling.clone(),
            ))
            .expect("evidence verification comparison key is serializable")
        }

        let mut evidence_keys = self.evidence.iter().map(key).collect::<Vec<_>>();
        let mut verification_keys = verifications
            .iter()
            .map(verification_key)
            .collect::<Vec<_>>();
        evidence_keys.sort();
        verification_keys.sort();

        evidence_keys.dedup();
        verification_keys.dedup();

        evidence_keys.len() == self.evidence.len()
            && verification_keys.len() == verifications.len()
            && evidence_keys == verification_keys
    }
}

/// Canonical snapshot of explicit environment facts relevant to a verification scope.
///
/// Environment facts are modeled as structured observations with an explicit
/// scope and source reference. The snapshot identifies the exact inputs used;
/// it does not assert that the measurements are accurate outside their declared
/// scope or time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentSnapshotV1 {
    pub schema: String,
    pub subject: CapabilityId,
    pub scope: String,
    pub facts: Vec<EnvironmentFactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentFactV1 {
    pub key: String,
    pub value: String,
    pub unit: Option<String>,
    pub source: String,
}

impl EnvironmentSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:environment-snapshot:v1";

    pub fn is_well_formed(&self) -> bool {
        if self.schema != Self::SCHEMA || self.subject.0.is_empty() || self.scope.is_empty() {
            return false;
        }

        let mut facts = self.facts.clone();
        if facts.iter().any(|fact| {
            fact.key.is_empty()
                || fact.value.is_empty()
                || fact.source.is_empty()
                || fact.unit.as_ref().is_some_and(|unit| unit.is_empty())
        }) {
            return false;
        }
        facts.sort_by(|left, right| {
            (&left.key, &left.value, &left.unit, &left.source).cmp(&(
                &right.key,
                &right.value,
                &right.unit,
                &right.source,
            ))
        });
        facts.len() == self.facts.len() && facts.windows(2).all(|pair| pair[0] != pair[1])
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalSnapshot {
            schema: String,
            subject: CapabilityId,
            scope: String,
            facts: Vec<EnvironmentFactV1>,
        }

        let mut facts = self.facts.clone();
        facts.sort_by(|left, right| {
            (&left.key, &left.value, &left.unit, &left.source).cmp(&(
                &right.key,
                &right.value,
                &right.unit,
                &right.source,
            ))
        });
        facts.dedup();

        let canonical = CanonicalSnapshot {
            schema: self.schema.clone(),
            subject: self.subject.clone(),
            scope: self.scope.clone(),
            facts,
        };

        serde_json::to_vec(&canonical)
            .expect("environment snapshot contains only serializable environment primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }

    pub fn from_facts(
        subject: CapabilityId,
        scope: impl Into<String>,
        mut facts: Vec<EnvironmentFactV1>,
    ) -> Self {
        facts.sort_by(|left, right| {
            (&left.key, &left.value, &left.unit, &left.source).cmp(&(
                &right.key,
                &right.value,
                &right.unit,
                &right.source,
            ))
        });
        facts.dedup();

        Self {
            schema: Self::SCHEMA.into(),
            subject,
            scope: scope.into(),
            facts,
        }
    }
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
        let mut paths = self
            .dependencies
            .iter()
            .flat_map(|dependency| {
                dependency.substitutes.iter().cloned().map(|candidate| AlternativePath {
                    for_dependency: dependency.capability.clone(),
                    candidate,
                    evidence: Vec::new(),
                    claim_ceiling: "Declared alternative candidate only; equivalence and operational interchangeability are not established.".into(),
                })
            })
            .collect::<Vec<_>>();

        paths.sort_by(|left, right| {
            (&left.for_dependency, &left.candidate).cmp(&(&right.for_dependency, &right.candidate))
        });
        paths.dedup_by(|left, right| {
            left.for_dependency == right.for_dependency && left.candidate == right.candidate
        });
        paths
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryPlanState {
    Draft,
    Ready,
    Executing,
    Succeeded,
    Failed,
    Verified,
}

/// An explicit, auditable recovery plan. Planning does not imply execution or
/// successful restoration; those claims require later execution evidence.
fn unique_nonempty_strings(values: &[String]) -> bool {
    if values.iter().any(|value| value.is_empty()) {
        return false;
    }

    let mut unique = values.to_vec();
    unique.sort();
    unique.dedup();
    unique.len() == values.len()
}

fn unique_nonempty_ids(values: &[CapabilityId]) -> bool {
    if values.iter().any(|value| value.0.is_empty()) {
        return false;
    }

    let mut unique = values.to_vec();
    unique.sort();
    unique.dedup();
    unique.len() == values.len()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub id: String,
    pub unavailable: CapabilityId,
    pub candidate: CapabilityId,
    /// Exact digest of the candidate record bound to this plan, when present.
    #[serde(default)]
    pub candidate_snapshot: String,
    pub prerequisites: Vec<CapabilityId>,
    pub steps: Vec<String>,
    pub preconditions: Vec<String>,
    pub expected_evidence: Vec<String>,
    pub human_contribution: String,
    pub ai_contribution: String,
    pub state: RecoveryPlanState,
    pub claim_ceiling: String,
}

impl RecoveryPlan {
    fn steps_are_well_formed(&self) -> bool {
        if self.steps.is_empty() || self.steps.iter().any(|step| step.is_empty()) {
            return false;
        }

        let mut steps = self.steps.clone();
        steps.sort();
        steps.dedup();
        steps.len() == self.steps.len()
    }

    fn prerequisites_are_well_formed(&self) -> bool {
        unique_nonempty_ids(&self.prerequisites)
            && unique_nonempty_strings(&self.preconditions)
            && unique_nonempty_strings(&self.expected_evidence)
    }

    pub fn is_ready(&self) -> bool {
        self.state == RecoveryPlanState::Ready
            && !self.id.is_empty()
            && !self.unavailable.0.is_empty()
            && !self.candidate.0.is_empty()
            && self.steps_are_well_formed()
            && self.prerequisites_are_well_formed()
            && !self.expected_evidence.is_empty()
            && !self.claim_ceiling.is_empty()
    }

    /// Bind the plan to the canonical candidate snapshot persisted with the plan.
    pub fn is_ready_against_candidate_snapshot(
        &self,
        candidate: &RecoveryCandidate,
        snapshot: &RecoveryCandidateSnapshotV1,
    ) -> bool {
        self.is_ready_against_candidate(candidate)
            && snapshot.is_well_formed()
            && snapshot == &candidate.snapshot()
            && self.candidate_snapshot == snapshot.digest()
    }

    /// Stronger readiness gate binding the plan to an explicitly discovered candidate.
    ///
    /// Candidate discovery remains separate from policy selection: this only proves
    /// that the plan targets the same declared alternative and includes the candidate's
    /// known required closure as prerequisites.
    pub fn is_ready_against_candidate(&self, candidate: &RecoveryCandidate) -> bool {
        if !self.is_ready()
            || candidate.selection == RecoverySelectionState::Rejected
            || !candidate.is_resolvable()
            || self.unavailable != candidate.for_dependency
            || self.candidate != candidate.candidate
        {
            return false;
        }

        candidate
            .required_capabilities
            .iter()
            .filter(|capability| *capability != &candidate.candidate)
            .all(|capability| self.prerequisites.contains(capability))
    }
}

/// Canonical semantic snapshot of one recovery plan.
///
/// Lifecycle state is deliberately excluded: Draft -> Ready -> Executing ->
/// Succeeded/Failed/Verified is a mutable lifecycle transition, not a change to
/// the plan semantics that governed an execution. State must therefore be bound
/// separately by the consuming policy when needed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPlanSnapshotV1 {
    pub schema: String,
    pub id: String,
    pub unavailable: CapabilityId,
    pub candidate: CapabilityId,
    pub candidate_snapshot: String,
    pub prerequisites: Vec<CapabilityId>,
    /// Ordered recovery actions: order is part of plan identity.
    pub steps: Vec<String>,
    pub preconditions: Vec<String>,
    pub expected_evidence: Vec<String>,
    pub human_contribution: String,
    pub ai_contribution: String,
    pub claim_ceiling: String,
}

impl RecoveryPlanSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-plan-snapshot:v1";

    pub fn from_plan(plan: &RecoveryPlan) -> Self {
        let mut prerequisites = plan.prerequisites.clone();
        prerequisites.sort();
        prerequisites.dedup();

        let mut preconditions = plan.preconditions.clone();
        preconditions.sort();
        preconditions.dedup();

        let mut expected_evidence = plan.expected_evidence.clone();
        expected_evidence.sort();
        expected_evidence.dedup();

        Self {
            schema: Self::SCHEMA.into(),
            id: plan.id.clone(),
            unavailable: plan.unavailable.clone(),
            candidate: plan.candidate.clone(),
            candidate_snapshot: plan.candidate_snapshot.clone(),
            prerequisites,
            steps: plan.steps.clone(),
            preconditions,
            expected_evidence,
            human_contribution: plan.human_contribution.clone(),
            ai_contribution: plan.ai_contribution.clone(),
            claim_ceiling: plan.claim_ceiling.clone(),
        }
    }

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.id.is_empty()
            && !self.unavailable.0.is_empty()
            && !self.candidate.0.is_empty()
            && !self.candidate_snapshot.is_empty()
            && unique_nonempty_ids(&self.prerequisites)
            && !self.steps.is_empty()
            && unique_nonempty_strings(&self.steps)
            && unique_nonempty_strings(&self.preconditions)
            && unique_nonempty_strings(&self.expected_evidence)
            && !self.claim_ceiling.is_empty()
    }

    /// Stable project-specific canonical bytes for the fixed-field plan schema.
    ///
    /// Set-like collections are normalized before serialization. This is not
    /// presented as RFC 8785/JCS interoperability.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalSnapshot {
            schema: String,
            id: String,
            unavailable: CapabilityId,
            candidate: CapabilityId,
            candidate_snapshot: String,
            prerequisites: Vec<CapabilityId>,
            steps: Vec<String>,
            preconditions: Vec<String>,
            expected_evidence: Vec<String>,
            human_contribution: String,
            ai_contribution: String,
            claim_ceiling: String,
        }

        let mut prerequisites = self.prerequisites.clone();
        prerequisites.sort();
        prerequisites.dedup();

        let mut preconditions = self.preconditions.clone();
        preconditions.sort();
        preconditions.dedup();

        let mut expected_evidence = self.expected_evidence.clone();
        expected_evidence.sort();
        expected_evidence.dedup();

        let canonical = CanonicalSnapshot {
            schema: self.schema.clone(),
            id: self.id.clone(),
            unavailable: self.unavailable.clone(),
            candidate: self.candidate.clone(),
            candidate_snapshot: self.candidate_snapshot.clone(),
            prerequisites,
            steps: self.steps.clone(),
            preconditions,
            expected_evidence,
            human_contribution: self.human_contribution.clone(),
            ai_contribution: self.ai_contribution.clone(),
            claim_ceiling: self.claim_ceiling.clone(),
        };

        serde_json::to_vec(&canonical)
            .expect("recovery plan snapshot contains only serializable plan primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

impl RecoveryPlan {
    pub fn snapshot(&self) -> RecoveryPlanSnapshotV1 {
        RecoveryPlanSnapshotV1::from_plan(self)
    }

    /// Stronger binding that requires the persisted plan semantics to be
    /// reconstructible into the same canonical plan snapshot.
    pub fn is_exactly_bound_by_snapshot(&self, snapshot: &RecoveryPlanSnapshotV1) -> bool {
        snapshot.is_well_formed() && snapshot == &self.snapshot()
    }
}

/// External policy decision bound to one exact recovery plan and candidate.
///
/// This record does not establish that the referenced authority is legitimate.
/// It makes the admission/selection decision explicit, content-addressed, and
/// consumable only for the exact plan, candidate, purpose, consumer, and validity
/// window recorded here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPolicyDecisionSnapshotV1 {
    pub schema: String,
    pub id: String,
    pub decision: RecoveryPolicyDecisionV1,
    pub purpose: String,
    pub consumer: String,
    pub plan_id: String,
    pub plan_snapshot: String,
    pub candidate: CapabilityId,
    pub candidate_snapshot: String,
    /// Opaque reference to the external authority/issuer record.
    pub authority_reference: String,
    pub issued_at: String,
    pub valid_until: String,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryPolicyDecisionV1 {
    Admitted,
    Rejected,
}

impl RecoveryPolicyDecisionSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-policy-decision-snapshot:v1";

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.id.is_empty()
            && !self.purpose.is_empty()
            && !self.consumer.is_empty()
            && !self.plan_id.is_empty()
            && !self.plan_snapshot.is_empty()
            && !self.candidate.0.is_empty()
            && !self.candidate_snapshot.is_empty()
            && !self.authority_reference.is_empty()
            && is_canonical_utc_timestamp(&self.issued_at)
            && is_canonical_utc_timestamp(&self.valid_until)
            && self.issued_at.as_str() < self.valid_until.as_str()
            && !self.claim_ceiling.is_empty()
    }

    /// Whether this well-formed decision explicitly admits the recovery action.
    ///
    /// Rejected decisions remain canonical audit records but are never
    /// executable authorization.
    pub fn is_admitted(&self) -> bool {
        self.is_well_formed() && self.decision == RecoveryPolicyDecisionV1::Admitted
    }

    pub fn is_valid_at(&self, now: &str) -> bool {
        self.is_admitted()
            && is_canonical_utc_timestamp(now)
            && self.issued_at.as_str() <= now
            && now < self.valid_until.as_str()
    }

    /// A pre-execution decision must already be valid when the activity begins
    /// and must remain valid at the point the record is consumed.
    pub fn covers_execution(&self, execution: &RecoveryExecution, now: &str) -> bool {
        self.is_valid_at(now)
            && is_canonical_utc_timestamp(&execution.started_at)
            && execution
                .ended_at
                .as_deref()
                .is_some_and(is_canonical_utc_timestamp)
            && self.issued_at.as_str() <= execution.started_at.as_str()
            && execution.started_at.as_str() < self.valid_until.as_str()
            && execution
                .ended_at
                .as_deref()
                .is_some_and(|ended_at| ended_at <= now)
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self)
            .expect("recovery policy decision snapshot contains only serializable primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

/// Verification result for a completed recovery execution.
///
/// Verification is deliberately separate from execution: evidence can be
/// present while the recovered capability remains unavailable, out of scope,
/// or otherwise inconsistent with the declared postconditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryVerificationState {
    Pending,
    Passed,
    Failed,
    Inconclusive,
}

/// Canonical semantic input set for a recovery verification.
///
/// Set-like collections are sorted and deduplicated before serialization so
/// equivalent inputs produce the same digest regardless of insertion order.
/// The digest identifies the exact verification inputs; it does not prove
/// that those inputs are true or that the resulting capability is qualified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryVerificationSnapshotV1 {
    pub schema: String,
    pub execution_id: String,
    pub capability: CapabilityId,
    pub scope: String,
    pub expected_postconditions: Vec<String>,
    pub observed_postconditions: Vec<String>,
    /// Typed evidence references plus independent processing state.
    ///
    /// A PASS requires every reference to have a recomputed digest and verifier
    /// metadata; unresolved legacy strings are not admissible.
    pub evidence: Vec<EvidenceReferenceV1>,
    /// Digest of the exact profile/registry authorization metadata recorded for
    /// each processed evidence reference. An empty value is legacy/incomplete
    /// and therefore cannot match a snapshot produced by the current schema.
    #[serde(default)]
    pub evidence_authorization_snapshot: String,
    pub missing_postconditions: Vec<String>,
    pub contradictory_postconditions: Vec<String>,
    pub dependency_closure: Vec<CapabilityId>,
    pub unresolved_dependencies: Vec<CapabilityId>,
    pub dependency_snapshot: String,
    pub environment_snapshot: String,
    pub evidence_snapshot: String,
    pub evidence_coverage: RecoveryEvidenceCoverage,
}

impl RecoveryVerificationSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-verification-snapshot:v1";

    pub fn from_verification(verification: &RecoveryVerification) -> Self {
        fn sorted_strings(values: &[String]) -> Vec<String> {
            let mut values = values.to_vec();
            values.sort();
            values.dedup();
            values
        }

        fn sorted_ids(values: &[CapabilityId]) -> Vec<CapabilityId> {
            let mut values = values.to_vec();
            values.sort();
            values.dedup();
            values
        }

        Self {
            schema: Self::SCHEMA.into(),
            execution_id: verification.execution_id.clone(),
            capability: verification.capability.clone(),
            scope: verification.scope.clone(),
            expected_postconditions: sorted_strings(&verification.expected_postconditions),
            observed_postconditions: sorted_strings(&verification.observed_postconditions),
            evidence: {
                let mut values = verification
                    .evidence
                    .iter()
                    .map(|verification| verification.reference.clone())
                    .collect::<Vec<_>>();
                values.sort_by(|left, right| {
                    (
                        &left.artifact_type,
                        &left.context,
                        &left.digest,
                        &left.purpose,
                        &left.claim_ceiling,
                    )
                        .cmp(&(
                            &right.artifact_type,
                            &right.context,
                            &right.digest,
                            &right.purpose,
                            &right.claim_ceiling,
                        ))
                });
                values.dedup();
                values
            },
            evidence_authorization_snapshot: Self::evidence_authorization_snapshot(verification),
            missing_postconditions: sorted_strings(&verification.missing_postconditions),
            contradictory_postconditions: sorted_strings(
                &verification.contradictory_postconditions,
            ),
            dependency_closure: sorted_ids(&verification.dependency_closure),
            unresolved_dependencies: sorted_ids(&verification.unresolved_dependencies),
            dependency_snapshot: verification.dependency_snapshot.clone(),
            environment_snapshot: verification.environment_snapshot.clone(),
            evidence_snapshot: verification.evidence_snapshot.clone(),
            evidence_coverage: verification.evidence_coverage,
        }
    }

    /// Construct a verification snapshot only from structured component
    /// snapshots. Component identities are derived by their own producers first;
    /// this method binds those identities into the verification question.
    fn evidence_authorization_snapshot(verification: &RecoveryVerification) -> String {
        #[derive(Serialize)]
        struct CanonicalAuthorization {
            artifact_type: String,
            context: DigestContextV1,
            digest: String,
            purpose: Option<String>,
            claim_ceiling: String,
            profile_id: Option<String>,
            profile_version: Option<u32>,
            profile_digest: Option<String>,
            registry_id: Option<String>,
            registry_version: Option<u32>,
            registry_digest: Option<String>,
        }

        #[derive(Serialize)]
        struct CanonicalAuthorizationSnapshot {
            schema: String,
            evidence: Vec<CanonicalAuthorization>,
        }

        let mut evidence = verification
            .evidence
            .iter()
            .map(|verification| CanonicalAuthorization {
                artifact_type: verification.reference.artifact_type.clone(),
                context: verification.reference.context.clone(),
                digest: verification.reference.digest.clone(),
                purpose: verification.reference.purpose.clone(),
                claim_ceiling: verification.reference.claim_ceiling.clone(),
                profile_id: verification.profile_id.clone(),
                profile_version: verification.profile_version,
                profile_digest: verification.profile_digest.clone(),
                registry_id: verification.registry_id.clone(),
                registry_version: verification.registry_version,
                registry_digest: verification.registry_digest.clone(),
            })
            .collect::<Vec<_>>();

        evidence.sort_by(|left, right| {
            (
                &left.artifact_type,
                &left.context,
                &left.digest,
                &left.purpose,
                &left.claim_ceiling,
                &left.profile_id,
                left.profile_version,
                &left.profile_digest,
                &left.registry_id,
                left.registry_version,
                &left.registry_digest,
            )
                .cmp(&(
                    &right.artifact_type,
                    &right.context,
                    &right.digest,
                    &right.purpose,
                    &right.claim_ceiling,
                    &right.profile_id,
                    right.profile_version,
                    &right.profile_digest,
                    &right.registry_id,
                    right.registry_version,
                    &right.registry_digest,
                ))
        });
        evidence.dedup_by(|left, right| {
            left.artifact_type == right.artifact_type
                && left.context == right.context
                && left.digest == right.digest
                && left.purpose == right.purpose
                && left.claim_ceiling == right.claim_ceiling
                && left.profile_id == right.profile_id
                && left.profile_version == right.profile_version
                && left.profile_digest == right.profile_digest
                && left.registry_id == right.registry_id
                && left.registry_version == right.registry_version
                && left.registry_digest == right.registry_digest
        });

        let canonical = CanonicalAuthorizationSnapshot {
            schema: "sol-atlas:evidence-authorization-snapshot:v1".into(),
            evidence,
        };

        let digest = Sha256::digest(
            serde_json::to_vec(&canonical)
                .expect("evidence authorization snapshot is serializable"),
        );
        format!("sha256:{digest:x}")
    }

    pub fn from_verification_with_snapshots(
        verification: &RecoveryVerification,
        dependency: &DependencySnapshotV1,
        evidence: &EvidenceSnapshotV1,
        environment: &EnvironmentSnapshotV1,
    ) -> Result<Self, &'static str> {
        if dependency.root != verification.capability
            || evidence.subject != verification.capability
            || environment.subject != verification.capability
            || environment.scope != verification.scope
        {
            return Err("snapshot subject or scope does not match verification");
        }

        let mut snapshot = Self::from_verification(verification);
        snapshot.dependency_snapshot = dependency.digest();
        snapshot.evidence_snapshot = evidence.digest();
        snapshot.environment_snapshot = environment.digest();
        Ok(snapshot)
    }

    /// Stable JSON bytes for this fixed-field schema.
    ///
    /// This is a project-specific canonical form: it uses a fixed struct
    /// property order and normalizes set-like arrays before serialization.
    /// It does not claim full RFC 8785/JCS interoperability.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalEvidenceReference {
            artifact_type: String,
            context: crate::evidence_reference::DigestContextV1,
            digest: String,
            purpose: Option<String>,
            claim_ceiling: String,
        }

        #[derive(Serialize)]
        struct CanonicalSnapshot {
            schema: String,
            execution_id: String,
            capability: CapabilityId,
            scope: String,
            expected_postconditions: Vec<String>,
            observed_postconditions: Vec<String>,
            evidence: Vec<CanonicalEvidenceReference>,
            evidence_authorization_snapshot: String,
            missing_postconditions: Vec<String>,
            contradictory_postconditions: Vec<String>,
            dependency_closure: Vec<CapabilityId>,
            unresolved_dependencies: Vec<CapabilityId>,
            scope_snapshot: RecoveryResilienceScopeSnapshotV1,
            environment_snapshot: String,
            evidence_snapshot: String,
            evidence_coverage: RecoveryEvidenceCoverage,
        }

        fn sorted_strings(values: &[String]) -> Vec<String> {
            let mut values = values.to_vec();
            values.sort();
            values.dedup();
            values
        }

        fn sorted_ids(values: &[CapabilityId]) -> Vec<CapabilityId> {
            let mut values = values.to_vec();
            values.sort();
            values.dedup();
            values
        }

        let mut evidence = self
            .evidence
            .iter()
            .map(|reference| CanonicalEvidenceReference {
                artifact_type: reference.artifact_type.clone(),
                context: reference.context.clone(),
                digest: reference.digest.clone(),
                purpose: reference.purpose.clone(),
                claim_ceiling: reference.claim_ceiling.clone(),
            })
            .collect::<Vec<_>>();
        evidence.sort_by(|left, right| {
            (
                &left.artifact_type,
                &left.context,
                &left.digest,
                &left.purpose,
                &left.claim_ceiling,
            )
                .cmp(&(
                    &right.artifact_type,
                    &right.context,
                    &right.digest,
                    &right.purpose,
                    &right.claim_ceiling,
                ))
        });
        evidence.dedup_by(|left, right| {
            left.artifact_type == right.artifact_type
                && left.context == right.context
                && left.digest == right.digest
                && left.purpose == right.purpose
                && left.claim_ceiling == right.claim_ceiling
        });

        let canonical = CanonicalSnapshot {
            schema: self.schema.clone(),
            execution_id: self.execution_id.clone(),
            capability: self.capability.clone(),
            scope: self.scope.clone(),
            expected_postconditions: sorted_strings(&self.expected_postconditions),
            observed_postconditions: sorted_strings(&self.observed_postconditions),
            evidence,
            evidence_authorization_snapshot: self.evidence_authorization_snapshot.clone(),
            missing_postconditions: sorted_strings(&self.missing_postconditions),
            contradictory_postconditions: sorted_strings(&self.contradictory_postconditions),
            dependency_closure: sorted_ids(&self.dependency_closure),
            unresolved_dependencies: sorted_ids(&self.unresolved_dependencies),
            scope_snapshot: self.scope_snapshot.clone(),
            environment_snapshot: self.environment_snapshot.clone(),
            evidence_snapshot: self.evidence_snapshot.clone(),
            evidence_coverage: self.evidence_coverage,
        };

        serde_json::to_vec(&canonical)
            .expect("verification snapshot contains only serializable verification primitives")
    }

    /// SHA-256 identity of the canonical semantic verification inputs.
    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}
/// Result of checking whether a verification record can still be reused
/// against the exact inputs it originally verified.
///
/// Declares whether the verifier knows the submitted evidence set is complete.
///
/// `ClosedWorld` means the selected verification profile defines the supplied
/// bundle as the complete admissible evidence universe. `OpenWorld` means
/// absence from the bundle is not evidence of absence. `Unknown` prevents a
/// verifier from silently treating an incomplete search as a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryEvidenceCoverage {
    ClosedWorld,
    OpenWorld,
    Unknown,
}

impl RecoveryEvidenceCoverage {
    pub fn permits_pass(&self) -> bool {
        matches!(self, Self::ClosedWorld)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryVerificationValidity {
    Current,
    Stale,
    ScopeMismatch,
    DependencyDrift,
    EnvironmentDrift,
    EvidenceDrift,
    Superseded,
}

/// Canonical UTC timestamp accepted by recovery verification freshness checks.
///
/// The fixed second-resolution representation is intentionally narrower than
/// general RFC 3339: YYYY-MM-DDTHH:MM:SSZ. Fixed-width UTC timestamps can be
/// compared lexically without timezone or precision ambiguity.
fn is_canonical_utc_timestamp(value: &str) -> bool {
    fn digits(value: &[u8]) -> bool {
        value.iter().all(u8::is_ascii_digit)
    }

    fn leap_year(year: u32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    if value.len() != 20 {
        return false;
    }

    let bytes = value.as_bytes();
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
        || !digits(&bytes[0..4])
        || !digits(&bytes[5..7])
        || !digits(&bytes[8..10])
        || !digits(&bytes[11..13])
        || !digits(&bytes[14..16])
        || !digits(&bytes[17..19])
    {
        return false;
    }

    let year = value[0..4].parse::<u32>().ok();
    let month = value[5..7].parse::<u32>().ok();
    let day = value[8..10].parse::<u32>().ok();
    let hour = value[11..13].parse::<u32>().ok();
    let minute = value[14..16].parse::<u32>().ok();
    let second = value[17..19].parse::<u32>().ok();

    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) =
        (year, month, day, hour, minute, second)
    else {
        return false;
    };

    if !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }

    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year(year) => 29,
        2 => 28,
        _ => unreachable!(),
    };

    (1..=days_in_month).contains(&day)
}

/// Auditable verification of the post-execution capability state.
///
/// Snapshot bindings are deliberately explicit: a historical PASS must not
/// silently become a current claim after the capability scope, dependency
/// graph, environment, or evidence changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryVerification {
    pub execution_id: String,
    pub capability: CapabilityId,
    pub scope: String,
    pub expected_postconditions: Vec<String>,
    pub observed_postconditions: Vec<String>,
    pub evidence: Vec<EvidenceReferenceVerificationV1>,
    pub missing_postconditions: Vec<String>,
    pub contradictory_postconditions: Vec<String>,
    pub dependency_closure: Vec<CapabilityId>,
    pub unresolved_dependencies: Vec<CapabilityId>,
    /// Stable snapshot identity for the verification input set.
    pub verification_snapshot: String,
    /// Snapshot of the exact required dependency closure used by verification.
    pub dependency_snapshot: String,
    /// Snapshot of the environment in which verification was performed.
    pub environment_snapshot: String,
    /// Snapshot of the evidence set consumed by verification.
    pub evidence_snapshot: String,
    /// Explicit policy for whether the supplied evidence set is complete.
    pub evidence_coverage: RecoveryEvidenceCoverage,
    /// Canonical UTC timestamp at which this verification ceases to be reusable.
    pub valid_until: String,
    /// Optional lineage marker for a newer verification that supersedes this one.
    pub superseded_by: Option<String>,
    pub state: RecoveryVerificationState,
    pub verifier: String,
    pub verified_at: String,
    pub claim_ceiling: String,
    /// Digest of the exact concrete execution-result record consumed by verification.
    /// Empty is legacy/unbound; stronger consumers must require an exact match.
    #[serde(default)]
    pub execution_result_snapshot: String,
}

impl RecoveryVerification {
    /// Recompute the verification-input digest from the record itself.
    pub fn derived_snapshot(&self) -> RecoveryVerificationSnapshotV1 {
        RecoveryVerificationSnapshotV1::from_verification(self)
    }

    /// Whether the stored snapshot identity matches the record's semantic inputs.
    pub fn snapshot_matches_inputs(&self) -> bool {
        self.verification_snapshot == self.derived_snapshot().digest()
    }

    /// A passed verification requires complete declared postcondition coverage,
    /// no contradictions, and a fully resolved required dependency closure.
    ///
    /// This still does not grant external qualification.
    pub fn passes(&self) -> bool {
        self.state == RecoveryVerificationState::Passed
            && !self.execution_id.is_empty()
            && !self.capability.0.is_empty()
            && !self.scope.is_empty()
            && !self.expected_postconditions.is_empty()
            && !self.expected_postconditions.iter().any(|condition| {
                !self
                    .observed_postconditions
                    .iter()
                    .any(|observed| observed == condition)
            })
            && self.contradictory_postconditions.is_empty()
            && self.unresolved_dependencies.is_empty()
            && !self.dependency_closure.is_empty()
            && !self.evidence.is_empty()
            && self
                .evidence
                .iter()
                .all(|verification| verification.is_verified())
            && !self.verification_snapshot.is_empty()
            && !self
                .derived_snapshot()
                .evidence_authorization_snapshot
                .is_empty()
            && self.snapshot_matches_inputs()
            && !self.dependency_snapshot.is_empty()
            && !self.environment_snapshot.is_empty()
            && !self.evidence_snapshot.is_empty()
            && self.evidence_coverage.permits_pass()
            && is_canonical_utc_timestamp(&self.valid_until)
            && !self.verifier.is_empty()
            && is_canonical_utc_timestamp(&self.verified_at)
    }

    pub fn passes_with_bound_execution(&self, execution: &RecoveryExecution) -> bool {
        let snapshot = RecoveryExecutionResultSnapshotV1::from_execution(execution);
        self.passes()
            && execution.is_successful()
            && self.execution_id == execution.execution_id
            && snapshot.is_well_formed()
            && snapshot.input_snapshot == execution.input_snapshot
            && self.execution_result_snapshot == snapshot.digest()
    }

    /// Stronger verification gate binding the verification to the exact ready
    /// plan and the concrete execution that reports its result.
    ///
    /// Plan semantics remain separate from mutable lifecycle state and from the
    /// external policy that authorized candidate selection.
    pub fn passes_with_bound_plan_execution(
        &self,
        plan: &RecoveryPlan,
        execution: &RecoveryExecution,
    ) -> bool {
        self.passes_with_bound_execution(execution) && execution.is_successful_with_bound_plan(plan)
    }

    /// Stronger verification gate binding verification to the exact policy
    /// decision, candidate, plan, and concrete execution result.
    pub fn passes_with_bound_policy_decision(
        &self,
        plan: &RecoveryPlan,
        candidate: &RecoveryCandidate,
        decision: &RecoveryPolicyDecisionSnapshotV1,
        execution: &RecoveryExecution,
        now: &str,
    ) -> bool {
        self.passes_with_bound_execution(execution)
            && execution.is_successful_with_bound_policy_decision(plan, candidate, decision, now)
    }

    /// Stronger verification gate binding the verification to the exact
    /// candidate lineage, ready plan, and concrete execution result.
    pub fn passes_with_bound_candidate_execution(
        &self,
        plan: &RecoveryPlan,
        candidate: &RecoveryCandidate,
        execution: &RecoveryExecution,
    ) -> bool {
        self.passes_with_bound_execution(execution)
            && execution.is_successful_with_bound_candidate(plan, candidate)
    }

    /// Stronger gate requiring the structured dependency, evidence, and
    /// environment snapshots used to derive the verification record.
    ///
    /// This closes the remaining gap between "has digest-looking strings" and
    /// "those digests are demonstrably the hashes of the exact structured inputs."
    /// It still does not establish payload truth or external qualification.
    pub fn passes_with_bound_snapshots(
        &self,
        dependency: &DependencySnapshotV1,
        evidence: &EvidenceSnapshotV1,
        environment: &EnvironmentSnapshotV1,
    ) -> bool {
        if !self.passes() {
            return false;
        }

        if dependency.root != self.capability
            || !dependency.is_well_formed()
            || !evidence.is_well_formed()
            || evidence.subject != self.capability
            || evidence.coverage != self.evidence_coverage
            || !environment.is_well_formed()
            || environment.subject != self.capability
            || environment.scope != self.scope
        {
            return false;
        }

        let mut declared_closure = self.dependency_closure.clone();
        let mut bound_closure = dependency.nodes.clone();
        declared_closure.sort();
        bound_closure.sort();
        if declared_closure.len() != self.dependency_closure.len()
            || bound_closure.len() != dependency.nodes.len()
            || declared_closure != bound_closure
        {
            return false;
        }

        if self.dependency_snapshot != dependency.digest()
            || self.evidence_snapshot != evidence.digest()
            || self.environment_snapshot != environment.digest()
            || evidence.coverage != self.evidence_coverage
            || !evidence.coverage.permits_pass()
        {
            return false;
        }

        if !evidence.all_references_verified(&self.evidence) {
            return false;
        }

        match RecoveryVerificationSnapshotV1::from_verification_with_snapshots(
            self,
            dependency,
            evidence,
            environment,
        ) {
            Ok(snapshot) => self.verification_snapshot == snapshot.digest(),
            Err(_) => false,
        }
    }

    /// Stronger gate requiring every processed evidence reference to remain
    /// authorized by the exact supplied profile registry.
    ///
    /// The generic bound-snapshot gate intentionally permits unbound verified
    /// evidence. This method is the explicit policy boundary for consumers that
    /// require registry-backed admission as well as digest verification.
    pub fn passes_with_bound_snapshots_and_registry(
        &self,
        dependency: &DependencySnapshotV1,
        evidence: &EvidenceSnapshotV1,
        environment: &EnvironmentSnapshotV1,
        registry: &EvidenceReferenceProfileRegistryV1,
    ) -> bool {
        self.passes_with_bound_snapshots(dependency, evidence, environment)
            && self
                .evidence
                .iter()
                .all(|verification| verification.is_verified_against_registry(registry))
    }

    /// Check whether this verification remains bound to the exact scope and
    /// snapshots that were originally verified.
    ///
    /// Snapshot identifiers are intentionally opaque: their producer owns the
    /// digest/canonicalization scheme. A mismatch is drift, never an implicit
    /// re-verification.
    pub fn validity_against(
        &self,
        capability: &CapabilityId,
        scope: &str,
        verification_snapshot: &str,
        dependency_snapshot: &str,
        environment_snapshot: &str,
        evidence_snapshot: &str,
        now: &str,
    ) -> RecoveryVerificationValidity {
        if self.superseded_by.is_some() {
            return RecoveryVerificationValidity::Superseded;
        }
        if &self.capability != capability || self.scope != scope {
            return RecoveryVerificationValidity::ScopeMismatch;
        }
        if self.verification_snapshot != verification_snapshot {
            return RecoveryVerificationValidity::ScopeMismatch;
        }
        if self.dependency_snapshot != dependency_snapshot {
            return RecoveryVerificationValidity::DependencyDrift;
        }
        if self.environment_snapshot != environment_snapshot {
            return RecoveryVerificationValidity::EnvironmentDrift;
        }
        if self.evidence_snapshot != evidence_snapshot {
            return RecoveryVerificationValidity::EvidenceDrift;
        }
        if self.verification_snapshot.is_empty()
            || !self.scope_snapshot.is_well_formed()
            || self.environment_snapshot.is_empty()
            || self.evidence_snapshot.is_empty()
            || !is_canonical_utc_timestamp(&self.verified_at)
            || !is_canonical_utc_timestamp(&self.valid_until)
            || !is_canonical_utc_timestamp(now)
        {
            return RecoveryVerificationValidity::Stale;
        }
        if now >= self.valid_until.as_str() {
            return RecoveryVerificationValidity::Stale;
        }
        RecoveryVerificationValidity::Current
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoverySelectionState {
    /// Candidate has been discovered from explicit graph metadata.
    Discovered,
    /// An external policy/qualification layer has admitted the candidate.
    Admissible,
    /// An external policy layer has selected the candidate for execution.
    Selected,
    /// An external policy layer has rejected the candidate.
    Rejected,
}

/// An explicit recovery candidate derived from a declared alternative path.
///
/// The graph may discover and analyze candidates, but it never changes their
/// selection state. Policy, authority, and execution remain external concerns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryCandidate {
    /// Capability dependency that this candidate is declared to address.
    pub for_dependency: CapabilityId,
    /// Explicit candidate capability.
    pub candidate: CapabilityId,
    /// Deterministic required closure of the candidate, when fully resolvable.
    pub required_capabilities: Vec<CapabilityId>,
    /// Missing prerequisites prevent the candidate closure from being complete.
    pub missing_capabilities: Vec<CapabilityId>,
    /// Evidence references attached to the alternative declaration.
    pub evidence: Vec<String>,
    /// Optional qualification scoped to this candidate.
    pub qualification: Option<CapabilityQualification>,
    /// Selection state owned by an external policy layer.
    pub selection: RecoverySelectionState,
    /// Exact claim ceiling for this candidate record.
    pub claim_ceiling: String,
}

/// Canonical content identity for one recovery candidate.
///
/// Selection state is deliberately excluded: discovery/admission/selection are
/// policy lifecycle state, not intrinsic candidate content identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryCandidateSnapshotV1 {
    pub schema: String,
    pub for_dependency: CapabilityId,
    pub candidate: CapabilityId,
    pub required_capabilities: Vec<CapabilityId>,
    pub missing_capabilities: Vec<CapabilityId>,
    pub evidence: Vec<String>,
    pub qualification: Option<CapabilityQualification>,
    pub claim_ceiling: String,
}

impl RecoveryCandidateSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-candidate-snapshot:v1";

    pub fn from_candidate(candidate: &RecoveryCandidate) -> Self {
        let mut required_capabilities = candidate.required_capabilities.clone();
        required_capabilities.sort();
        required_capabilities.dedup();
        let mut missing_capabilities = candidate.missing_capabilities.clone();
        missing_capabilities.sort();
        missing_capabilities.dedup();
        let mut evidence = candidate.evidence.clone();
        evidence.sort();
        evidence.dedup();
        Self {
            schema: Self::SCHEMA.into(),
            for_dependency: candidate.for_dependency.clone(),
            candidate: candidate.candidate.clone(),
            required_capabilities,
            missing_capabilities,
            evidence,
            qualification: candidate.qualification.clone(),
            claim_ceiling: candidate.claim_ceiling.clone(),
        }
    }

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.for_dependency.0.is_empty()
            && !self.candidate.0.is_empty()
            && self.for_dependency != self.candidate
            && unique_nonempty_ids(&self.required_capabilities)
            && unique_nonempty_ids(&self.missing_capabilities)
            && !self
                .required_capabilities
                .iter()
                .any(|id| self.missing_capabilities.contains(id))
            && unique_nonempty_strings(&self.evidence)
            && !self.claim_ceiling.is_empty()
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self)
            .expect("recovery candidate snapshot contains only serializable primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

impl RecoveryCandidate {
    pub fn snapshot(&self) -> RecoveryCandidateSnapshotV1 {
        RecoveryCandidateSnapshotV1::from_candidate(self)
    }

    /// Structural integrity of an explicitly discovered candidate.
    ///
    /// Partial candidates may have missing prerequisites, but their identities
    /// and present/missing sets must remain unambiguous. Resolution is a
    /// separate stronger gate.
    pub fn is_well_formed(&self) -> bool {
        if self.for_dependency.0.is_empty()
            || self.candidate.0.is_empty()
            || self.for_dependency == self.candidate
            || self.claim_ceiling.is_empty()
        {
            return false;
        }

        if self
            .required_capabilities
            .iter()
            .chain(self.missing_capabilities.iter())
            .any(|id| id.0.is_empty())
        {
            return false;
        }

        let mut required = self.required_capabilities.clone();
        required.sort();
        required.dedup();
        if required.len() != self.required_capabilities.len() {
            return false;
        }

        let mut missing = self.missing_capabilities.clone();
        missing.sort();
        missing.dedup();
        if missing.len() != self.missing_capabilities.len() {
            return false;
        }

        !required.iter().any(|id| missing.contains(id))
    }

    /// Whether the candidate's own required closure is complete.
    pub fn is_resolvable(&self) -> bool {
        self.is_well_formed()
            && self.missing_capabilities.is_empty()
            && self.required_capabilities.contains(&self.candidate)
    }
}

/// Canonical semantic snapshot of the required dependency graph for one root capability.
///
/// Unlike the historical `dependency_snapshot: String` field on verification,
/// this record is constructed directly from the graph. It therefore commits to
/// the actual required nodes and directed dependency relations rather than to
/// an opaque producer-supplied label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencySnapshotV1 {
    pub schema: String,
    pub root: CapabilityId,
    pub nodes: Vec<CapabilityId>,
    pub edges: Vec<DependencySnapshotEdgeV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencySnapshotEdgeV1 {
    pub from: CapabilityId,
    pub to: CapabilityId,
    pub relation: DependencyKind,
}

impl DependencySnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:dependency-snapshot:v1";

    pub fn is_well_formed(&self) -> bool {
        if self.schema != Self::SCHEMA
            || self.root.0.is_empty()
            || self.nodes.is_empty()
            || self.nodes.iter().any(|id| id.0.is_empty())
            || !self.nodes.contains(&self.root)
        {
            return false;
        }

        let mut nodes = self.nodes.clone();
        nodes.sort();
        nodes.dedup();
        if nodes.len() != self.nodes.len() {
            return false;
        }

        let mut duplicate_nodes = self.duplicate_nodes.clone();
        duplicate_nodes.sort();
        duplicate_nodes.dedup();

        let mut edges = self.edges.clone();
        edges.sort_by(|left, right| {
            (&left.from, &left.to, &left.relation).cmp(&(&right.from, &right.to, &right.relation))
        });
        if edges.windows(2).any(|pair| pair[0] == pair[1]) {
            return false;
        }

        edges.iter().all(|edge| {
            edge.relation.is_required()
                && self.nodes.contains(&edge.from)
                && self.nodes.contains(&edge.to)
        })
    }

    /// Stable project-specific canonical bytes.
    ///
    /// Set-like node/edge collections are normalized before serialization.
    /// This is intentionally not presented as RFC 8785/JCS.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalSnapshot {
            schema: String,
            root: CapabilityId,
            nodes: Vec<CapabilityId>,
            edges: Vec<DependencySnapshotEdgeV1>,
        }

        let mut nodes = self.nodes.clone();
        nodes.sort();
        nodes.dedup();

        let mut edges = self.edges.clone();
        edges.sort_by(|left, right| {
            (&left.from, &left.to, &left.relation).cmp(&(&right.from, &right.to, &right.relation))
        });
        edges.dedup();

        let canonical = CanonicalSnapshot {
            schema: self.schema.clone(),
            root: self.root.clone(),
            nodes,
            edges,
        };

        serde_json::to_vec(&canonical)
            .expect("dependency snapshot contains only serializable graph primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGraphError {
    pub missing: Vec<CapabilityId>,
    /// Capability IDs declared more than once make the graph ambiguous.
    pub duplicate_ids: Vec<CapabilityId>,
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

/// Canonical snapshot of the dependency scope used by a resilience assessment.
///
/// Unlike DependencySnapshotV1, this record can represent an incomplete graph:
/// missing prerequisites are explicit rather than causing the whole analysis scope
/// to disappear behind an empty digest. Duplicate capability IDs remain invalid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryResilienceScopeSnapshotV1 {
    pub schema: String,
    pub root: CapabilityId,
    pub present_nodes: Vec<CapabilityId>,
    pub missing_nodes: Vec<CapabilityId>,
    pub duplicate_nodes: Vec<CapabilityId>,
    pub edges: Vec<DependencySnapshotEdgeV1>,
}

impl RecoveryResilienceScopeSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-resilience-scope:v1";

    pub fn is_well_formed(&self) -> bool {
        if self.schema != Self::SCHEMA
            || self.root.0.is_empty()
            || self.present_nodes.is_empty()
            || !self.present_nodes.contains(&self.root)
            || !unique_nonempty_ids(&self.present_nodes)
            || !unique_nonempty_ids(&self.missing_nodes)
            || !unique_nonempty_ids(&self.duplicate_nodes)
            || !self.duplicate_nodes.is_empty()
            || self.present_nodes.iter().any(|id| self.missing_nodes.contains(id))
            || self.present_nodes.iter().any(|id| self.duplicate_nodes.contains(id))
            || self.missing_nodes.iter().any(|id| self.duplicate_nodes.contains(id))
        {
            return false;
        }

        let mut edges = self.edges.clone();
        edges.sort_by(|left, right| {
            (&left.from, &left.to, &left.relation)
                .cmp(&(&right.from, &right.to, &right.relation))
        });
        if edges.windows(2).any(|pair| pair[0] == pair[1]) {
            return false;
        }

        edges.iter().all(|edge| {
            edge.relation.is_required()
                && self.present_nodes.contains(&edge.from)
                && (self.present_nodes.contains(&edge.to) || self.missing_nodes.contains(&edge.to))
        })
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalScope {
            schema: String,
            root: CapabilityId,
            present_nodes: Vec<CapabilityId>,
            missing_nodes: Vec<CapabilityId>,
            duplicate_nodes: Vec<CapabilityId>,
            edges: Vec<DependencySnapshotEdgeV1>,
        }

        let mut present_nodes = self.present_nodes.clone();
        present_nodes.sort();
        present_nodes.dedup();

        let mut missing_nodes = self.missing_nodes.clone();
        missing_nodes.sort();
        missing_nodes.dedup();

        let mut duplicate_nodes = self.duplicate_nodes.clone();
        duplicate_nodes.sort();
        duplicate_nodes.dedup();

        let mut edges = self.edges.clone();
        edges.sort_by(|left, right| {
            (&left.from, &left.to, &left.relation)
                .cmp(&(&right.from, &right.to, &right.relation))
        });
        edges.dedup();

        serde_json::to_vec(&CanonicalScope {
            schema: self.schema.clone(),
            root: self.root.clone(),
            present_nodes,
            missing_nodes,
            duplicate_nodes,
            edges,
        })
        .expect("recovery resilience scope contains only serializable graph primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

/// Canonical, root-scoped structural resilience assessment.
///
/// This is a graph analysis record, not a resilience or substitution claim. It
/// keeps structurally affected capabilities, unresolved graph regions, and
/// explicitly declared alternative candidates in separate sets. Alternative
/// candidates are snapshots only; no selection or substitution occurs here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryResilienceAssessmentV1 {
    pub schema: String,
    pub root: CapabilityId,
    pub unavailable: CapabilityId,
    /// Exact dependency scope snapshot used as the analysis scope, including
    /// explicit missing prerequisites when the graph is incomplete.
    pub scope_snapshot: RecoveryResilienceScopeSnapshotV1,
    /// Affected capabilities inside the root's declared required closure.
    pub affected: Vec<CapabilityId>,
    /// Unresolved capabilities inside the analyzed scope.
    pub unresolved: Vec<CapabilityId>,
    /// Explicit alternatives whose candidate closure is fully resolvable.
    pub alternatives: Vec<RecoveryCandidateSnapshotV1>,
    /// Explicit alternatives whose candidate closure remains incomplete.
    pub unresolved_alternatives: Vec<RecoveryCandidateSnapshotV1>,
    /// Structural-analysis claim ceiling; never a real-world resilience guarantee.
    pub claim_ceiling: String,
}

impl RecoveryResilienceAssessmentV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-resilience-assessment:v1";

    pub fn is_well_formed(&self) -> bool {
        if self.schema != Self::SCHEMA
            || self.root.0.is_empty()
            || self.unavailable.0.is_empty()
            || self.root == self.unavailable
            || self.scope_snapshot.root != self.root
            || !self.scope_snapshot.is_well_formed()
            || self.claim_ceiling.is_empty()
            || !unique_nonempty_ids(&self.affected)
            || !unique_nonempty_ids(&self.unresolved)
            || self
                .affected
                .iter()
                .any(|id| self.unresolved.contains(id))
            || self
                .affected
                .iter()
                .chain(self.unresolved.iter())
                .any(|id| !self.scope_snapshot.present_nodes.contains(id))
        {
            return false;
        }

        let mut candidate_keys = std::collections::BTreeSet::new();
        if self
            .alternatives
            .iter()
            .chain(self.unresolved_alternatives.iter())
            .any(|candidate| {
                !candidate.is_well_formed()
                    || candidate.for_dependency != self.unavailable
                    || !candidate_keys.insert((
                        candidate.for_dependency.clone(),
                        candidate.candidate.clone(),
                    ))
            })
        {
            return false;
        }

        self.alternatives
            .iter()
            .all(|candidate| candidate.missing_capabilities.is_empty())
            && self
                .unresolved_alternatives
                .iter()
                .all(|candidate| !candidate.missing_capabilities.is_empty())
    }

    /// Whether the analyzed required dependency scope is complete.
    ///
    /// A well-formed assessment may still be incomplete when required capabilities
    /// are absent from the supplied graph. This gate makes that distinction explicit.
    pub fn has_complete_scope(&self) -> bool {
        self.is_well_formed() && self.scope_snapshot.missing_nodes.is_empty()
    }

    /// Whether the assessment has a complete structural graph scope.
    ///
    /// This remains a structural gate only; it does not establish availability,
    /// equivalence, maintainability, reproducibility, or successful recovery.
    pub fn is_structurally_complete(&self) -> bool {
        self.has_complete_scope()
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalAssessment {
            schema: String,
            root: CapabilityId,
            unavailable: CapabilityId,
            scope_snapshot: RecoveryResilienceScopeSnapshotV1,
            affected: Vec<CapabilityId>,
            unresolved: Vec<CapabilityId>,
            alternatives: Vec<RecoveryCandidateSnapshotV1>,
            unresolved_alternatives: Vec<RecoveryCandidateSnapshotV1>,
            claim_ceiling: String,
        }

        let mut affected = self.affected.clone();
        affected.sort();
        affected.dedup();

        let mut unresolved = self.unresolved.clone();
        unresolved.sort();
        unresolved.dedup();

        let mut alternatives = self.alternatives.clone();
        alternatives.sort_by(|left, right| {
            (&left.for_dependency, &left.candidate).cmp(&(&right.for_dependency, &right.candidate))
        });
        alternatives.dedup();

        let mut unresolved_alternatives = self.unresolved_alternatives.clone();
        unresolved_alternatives.sort_by(|left, right| {
            (&left.for_dependency, &left.candidate)
                .cmp(&(&right.for_dependency, &right.candidate))
        });
        unresolved_alternatives.dedup();

        let canonical = CanonicalAssessment {
            schema: self.schema.clone(),
            root: self.root.clone(),
            unavailable: self.unavailable.clone(),
            scope_snapshot: self.scope_snapshot.clone(),
            affected,
            unresolved,
            alternatives,
            unresolved_alternatives,
            claim_ceiling: self.claim_ceiling.clone(),
        };

        serde_json::to_vec(&canonical)
            .expect("recovery resilience assessment contains only serializable primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }

    /// Re-derive the complete assessment from the supplied graph and reject any
    /// drift in structural impact, unresolved state, alternatives, or claim ceiling.
    pub fn is_exactly_bound_to_graph(&self, graph: &CapabilityGraph) -> bool {
        self.is_well_formed()
            && graph.resilience_assessment(&self.root, &self.unavailable) == *self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CapabilityGraph {
    pub capabilities: Vec<Capability>,
}

impl CapabilityGraph {
    /// Build a dependency snapshot from the actual required graph reachable from
    /// `root`. Alternatives and enabling-only relations are excluded because
    /// they are not part of the required closure.
    ///
    /// Missing required nodes fail closed instead of producing a partial digest.
    pub fn dependency_snapshot(
        &self,
        root: &CapabilityId,
    ) -> Result<DependencySnapshotV1, CapabilityGraphError> {
        use std::collections::{BTreeMap, BTreeSet, VecDeque};

        let duplicate_ids = self.duplicate_capability_ids();
        if !duplicate_ids.is_empty() {
            return Err(CapabilityGraphError {
                missing: vec![],
                duplicate_ids,
            });
        }

        let index = self
            .capabilities
            .iter()
            .map(|capability| (capability.id.clone(), capability))
            .collect::<BTreeMap<_, _>>();

        let mut queue = VecDeque::from([root.clone()]);
        let mut nodes = BTreeSet::new();
        let mut edges = BTreeSet::new();
        let mut missing = BTreeSet::new();

        while let Some(id) = queue.pop_front() {
            if !nodes.insert(id.clone()) {
                continue;
            }

            let Some(capability) = index.get(&id) else {
                missing.insert(id);
                continue;
            };

            for dependency in capability
                .dependencies
                .iter()
                .filter(|d| d.relation.is_required())
            {
                edges.insert((
                    capability.id.clone(),
                    dependency.capability.clone(),
                    dependency.relation,
                ));
                queue.push_back(dependency.capability.clone());
            }
        }

        if !missing.is_empty() {
            return Err(CapabilityGraphError {
                missing: missing.into_iter().collect(),
                duplicate_ids: vec![],
            });
        }

        Ok(DependencySnapshotV1 {
            schema: DependencySnapshotV1::SCHEMA.into(),
            root: root.clone(),
            nodes: nodes.into_iter().collect(),
            edges: edges
                .into_iter()
                .map(|(from, to, relation)| DependencySnapshotEdgeV1 { from, to, relation })
                .collect(),
        })
    }

    /// Return capabilities whose required closure depends on an unavailable
    /// capability.
    ///
    /// This is a declared dependency blast radius, not a prediction of real
    /// world impact. Explicit substitutes are intentionally not selected here;
    /// resilience policy remains a separate, auditable decision.
    pub fn affected_by(&self, unavailable: &CapabilityId) -> CapabilityImpact {
        use std::collections::{BTreeMap, BTreeSet, VecDeque};

        let duplicate_ids = self.duplicate_capability_ids();
        if !duplicate_ids.is_empty() {
            return CapabilityImpact {
                unavailable: unavailable.clone(),
                direct_affected: vec![],
                transitive_affected: vec![],
                unresolved: duplicate_ids,
                affected: vec![],
            };
        }

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

                for dependency in current
                    .dependencies
                    .iter()
                    .filter(|d| d.relation.is_required())
                {
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

    /// Enumerate explicit recovery candidates for a declared unavailable dependency.
    ///
    /// Candidate discovery is deterministic and never selects, ranks, or
    /// substitutes a candidate automatically. Each candidate's own closure is
    /// analyzed independently so missing recovery prerequisites remain visible.
    pub fn recovery_candidates(&self, unavailable: &CapabilityId) -> Vec<RecoveryCandidate> {
        if !self.duplicate_capability_ids().is_empty() {
            return Vec::new();
        }

        let mut candidates = self
            .capabilities
            .iter()
            .flat_map(|capability| {
                capability
                    .dependencies
                    .iter()
                    .filter(move |dependency| {
                        dependency.relation.is_required()
                            && dependency.capability == *unavailable
                    })
                    .flat_map(move |dependency| {
                        dependency.substitutes.iter().map(move |candidate| {
                            let (required_capabilities, missing_capabilities) =
                                self.required_closure_with_missing(candidate);

                            RecoveryCandidate {
                                for_dependency: dependency.capability.clone(),
                                candidate: candidate.clone(),
                                required_capabilities,
                                missing_capabilities,
                                evidence: Vec::new(),
                                qualification: None,
                                selection: RecoverySelectionState::Discovered,
                                claim_ceiling: "Declared recovery candidate only; equivalence, operational interchangeability, and successful recovery are not established.".into(),
                            }
                        })
                    })
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            (&left.for_dependency, &left.candidate).cmp(&(&right.for_dependency, &right.candidate))
        });
        candidates.dedup_by(|left, right| {
            left.for_dependency == right.for_dependency && left.candidate == right.candidate
        });
        candidates
    }

    /// Produce a root-scoped structural resilience assessment.
    ///
    /// Only explicitly declared alternatives reachable from the root's required
    /// closure are reported. Candidate closures with missing prerequisites are
    /// kept separate from resolvable alternatives, and no candidate is selected.
    pub fn resilience_assessment(
        &self,
        root: &CapabilityId,
        unavailable: &CapabilityId,
    ) -> RecoveryResilienceAssessmentV1 {
        use std::collections::BTreeSet;

        let scope_snapshot = self.resilience_scope_snapshot(root);
        let root_scope = scope_snapshot
            .present_nodes
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let impact = self.affected_by(unavailable);

        let mut affected = impact
            .affected
            .into_iter()
            .filter(|id| root_scope.contains(id))
            .collect::<Vec<_>>();
        affected.sort();
        affected.dedup();

        let mut unresolved = impact
            .unresolved
            .into_iter()
            .filter(|id| root_scope.contains(id))
            .collect::<Vec<_>>();
        unresolved.sort();
        unresolved.dedup();

        let root_declares_unavailable = self.capabilities.iter().any(|capability| {
            root_scope.contains(&capability.id)
                && capability.dependencies.iter().any(|dependency| {
                    dependency.relation.is_required() && dependency.capability == *unavailable
                })
        });

        let mut alternatives = Vec::new();
        let mut unresolved_alternatives = Vec::new();

        if root_declares_unavailable {
            for capability in &self.capabilities {
                if !root_scope.contains(&capability.id) {
                    continue;
                }

                for dependency in capability.dependencies.iter().filter(|dependency| {
                    dependency.relation.is_required() && dependency.capability == *unavailable
                }) {
                    for candidate_id in &dependency.substitutes {
                        let (required_capabilities, missing_capabilities) =
                            self.required_closure_with_missing(candidate_id);
                        let candidate = RecoveryCandidate {
                            for_dependency: dependency.capability.clone(),
                            candidate: candidate_id.clone(),
                            required_capabilities,
                            missing_capabilities,
                            evidence: Vec::new(),
                            qualification: None,
                            selection: RecoverySelectionState::Discovered,
                            claim_ceiling: "Declared recovery candidate only; equivalence and operational interchangeability are not established.".into(),
                        };
                        let snapshot = candidate.snapshot();

                        if candidate.is_resolvable() {
                            alternatives.push(snapshot);
                        } else {
                            unresolved_alternatives.push(snapshot);
                        }
                    }
                }
            }
        }

        alternatives.sort_by(|left, right| {
            (&left.for_dependency, &left.candidate)
                .cmp(&(&right.for_dependency, &right.candidate))
        });
        alternatives.dedup();
        unresolved_alternatives.sort_by(|left, right| {
            (&left.for_dependency, &left.candidate)
                .cmp(&(&right.for_dependency, &right.candidate))
        });
        unresolved_alternatives.dedup();

        RecoveryResilienceAssessmentV1 {
            schema: RecoveryResilienceAssessmentV1::SCHEMA.into(),
            root: root.clone(),
            unavailable: unavailable.clone(),
            scope_snapshot,
            affected,
            unresolved,
            alternatives,
            unresolved_alternatives,
            claim_ceiling: "Declared graph impact and explicit alternative-path analysis only; real-world resilience, equivalence, interchangeability, availability, sustainability, and successful recovery are not established.".into(),
        }
    }

    /// Build a deterministic root-scoped dependency record without discarding
    /// missing prerequisites. Duplicate capability IDs remain fail-closed.
    fn resilience_scope_snapshot(
        &self,
        root: &CapabilityId,
    ) -> RecoveryResilienceScopeSnapshotV1 {
        use std::collections::{BTreeMap, BTreeSet};

        let duplicate_nodes = self.duplicate_capability_ids();
        let (present_nodes, missing_nodes) = self.required_closure_with_missing(root);
        let index = self
            .capabilities
            .iter()
            .map(|capability| (capability.id.clone(), capability))
            .collect::<BTreeMap<_, _>>();
        let present = present_nodes.iter().cloned().collect::<BTreeSet<_>>();
        let missing = missing_nodes.iter().cloned().collect::<BTreeSet<_>>();
        let mut edges = BTreeSet::new();

        for id in &present_nodes {
            if let Some(capability) = index.get(id) {
                for dependency in capability
                    .dependencies
                    .iter()
                    .filter(|dependency| dependency.relation.is_required())
                {
                    if present.contains(&dependency.capability) || missing.contains(&dependency.capability) {
                        edges.insert((
                            capability.id.clone(),
                            dependency.capability.clone(),
                            dependency.relation,
                        ));
                    }
                }
            }
        }

        RecoveryResilienceScopeSnapshotV1 {
            schema: RecoveryResilienceScopeSnapshotV1::SCHEMA.into(),
            root: root.clone(),
            present_nodes,
            missing_nodes,
            duplicate_nodes,
            edges: edges
                .into_iter()
                .map(|(from, to, relation)| DependencySnapshotEdgeV1 { from, to, relation })
                .collect(),
        }
    }

    /// Compute a partial deterministic closure while preserving missing prerequisites.
    /// Present capabilities and absent prerequisites are kept in separate sets so
    /// recovery analysis can never imply that a missing candidate exists.
    fn required_closure_with_missing(
        &self,
        root: &CapabilityId,
    ) -> (Vec<CapabilityId>, Vec<CapabilityId>) {
        use std::collections::{BTreeMap, BTreeSet, VecDeque};

        let index = self
            .capabilities
            .iter()
            .map(|c| (c.id.clone(), c))
            .collect::<BTreeMap<_, _>>();
        let mut queue = VecDeque::from([root.clone()]);
        let mut seen = BTreeSet::new();
        let mut present = BTreeSet::new();
        let mut missing = BTreeSet::new();

        while let Some(id) = queue.pop_front() {
            if !seen.insert(id.clone()) {
                continue;
            }
            let Some(capability) = index.get(&id) else {
                missing.insert(id);
                continue;
            };
            present.insert(id);
            for dependency in capability
                .dependencies
                .iter()
                .filter(|d| d.relation.is_required())
            {
                queue.push_back(dependency.capability.clone());
            }
        }

        (present.into_iter().collect(), missing.into_iter().collect())
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
        let duplicate_ids = self.duplicate_capability_ids();
        if !duplicate_ids.is_empty() {
            return Err(CapabilityGraphError {
                missing: vec![],
                duplicate_ids,
            });
        }

        let (present, missing) = self.required_closure_with_missing(root);
        if !missing.is_empty() {
            return Err(CapabilityGraphError {
                missing,
                duplicate_ids: vec![],
            });
        }
        Ok(present)
    }

    fn duplicate_capability_ids(&self) -> Vec<CapabilityId> {
        use std::collections::BTreeSet;

        let mut seen = BTreeSet::new();
        let mut duplicates = BTreeSet::new();

        for capability in &self.capabilities {
            if !seen.insert(capability.id.clone()) {
                duplicates.insert(capability.id.clone());
            }
        }

        duplicates.into_iter().collect()
    }
}

/// Canonical semantic snapshot of the inputs that governed one recovery execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryExecutionSnapshotV1 {
    pub schema: String,
    pub plan_id: String,
    /// Digest of the canonical plan semantics bound to this execution input.
    /// Empty is legacy/unbound; stronger consumers must require an exact match.
    #[serde(default)]
    pub plan_snapshot: String,
    pub execution_id: String,
    pub unavailable: CapabilityId,
    pub candidate: CapabilityId,
    /// Digest of the canonical candidate content bound by the plan.
    /// Empty is legacy/unbound; stronger consumers must require an exact match.
    #[serde(default)]
    pub candidate_snapshot: String,
    pub prerequisites: Vec<CapabilityId>,
    /// Ordered execution steps: order is part of the input identity.
    pub steps: Vec<String>,
    pub preconditions: Vec<String>,
    pub expected_evidence: Vec<String>,
    pub observed_preconditions: Vec<String>,
    pub authorization: Option<String>,
    pub ai_assistance: Option<String>,
    pub plan_claim_ceiling: String,
    pub execution_claim_ceiling: String,
}

impl RecoveryExecutionSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-execution-snapshot:v1";

    /// Derive the exact input identity from the plan and the concrete execution.
    ///
    /// The execution's resulting state, evidence, and completion/failure markers
    /// are intentionally excluded: they are outputs of the activity, not inputs.
    pub fn from_plan_and_execution(plan: &RecoveryPlan, execution: &RecoveryExecution) -> Self {
        Self {
            schema: Self::SCHEMA.into(),
            plan_id: plan.id.clone(),
            plan_snapshot: plan.snapshot().digest(),
            execution_id: execution.execution_id.clone(),
            unavailable: plan.unavailable.clone(),
            candidate: plan.candidate.clone(),
            candidate_snapshot: plan.candidate_snapshot.clone(),
            prerequisites: plan.prerequisites.clone(),
            steps: plan.steps.clone(),
            preconditions: plan.preconditions.clone(),
            expected_evidence: plan.expected_evidence.clone(),
            observed_preconditions: execution.observed_preconditions.clone(),
            authorization: execution.authorization.clone(),
            ai_assistance: execution.ai_assistance.clone(),
            plan_claim_ceiling: plan.claim_ceiling.clone(),
            execution_claim_ceiling: execution.claim_ceiling.clone(),
        }
    }

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.plan_id.is_empty()
            && !self.plan_snapshot.is_empty()
            && !self.execution_id.is_empty()
            && !self.unavailable.0.is_empty()
            && !self.candidate.0.is_empty()
            && !self.candidate_snapshot.is_empty()
            && self.prerequisites.iter().all(|id| !id.0.is_empty())
            && !self.steps.is_empty()
            && self.steps.iter().all(|step| !step.is_empty())
            && !self
                .preconditions
                .iter()
                .any(|condition| condition.is_empty())
            && !self
                .expected_evidence
                .iter()
                .any(|reference| reference.is_empty())
            && !self
                .observed_preconditions
                .iter()
                .any(|condition| condition.is_empty())
            && self
                .authorization
                .as_ref()
                .is_none_or(|authorization| !authorization.is_empty())
            && self
                .ai_assistance
                .as_ref()
                .is_none_or(|assistance| !assistance.is_empty())
            && !self.plan_claim_ceiling.is_empty()
            && !self.execution_claim_ceiling.is_empty()
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalSnapshot {
            schema: String,
            plan_id: String,
            plan_snapshot: String,
            execution_id: String,
            unavailable: CapabilityId,
            candidate: CapabilityId,
            candidate_snapshot: String,
            prerequisites: Vec<CapabilityId>,
            steps: Vec<String>,
            preconditions: Vec<String>,
            expected_evidence: Vec<String>,
            observed_preconditions: Vec<String>,
            authorization: Option<String>,
            ai_assistance: Option<String>,
            plan_claim_ceiling: String,
            execution_claim_ceiling: String,
        }

        let canonical = CanonicalSnapshot {
            schema: self.schema.clone(),
            plan_id: self.plan_id.clone(),
            plan_snapshot: self.plan_snapshot.clone(),
            execution_id: self.execution_id.clone(),
            unavailable: self.unavailable.clone(),
            candidate: self.candidate.clone(),
            candidate_snapshot: self.candidate_snapshot.clone(),
            prerequisites: self.prerequisites.clone(),
            steps: self.steps.clone(),
            preconditions: self.preconditions.clone(),
            expected_evidence: self.expected_evidence.clone(),
            observed_preconditions: self.observed_preconditions.clone(),
            authorization: self.authorization.clone(),
            ai_assistance: self.ai_assistance.clone(),
            plan_claim_ceiling: self.plan_claim_ceiling.clone(),
            execution_claim_ceiling: self.execution_claim_ceiling.clone(),
        };

        serde_json::to_vec(&canonical)
            .expect("execution input snapshot contains only serializable primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

/// Evidence emitted by a concrete recovery execution.
///
/// Execution evidence is intentionally separate from the plan: a plan describes
/// intended actions, while this record describes what actually happened.
/// Canonical content identity for the concrete result of one recovery execution.
///
/// Input authorization and AI-assistance metadata are excluded because they belong
/// to the execution-input snapshot. This record binds the observed result instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryExecutionResultSnapshotV1 {
    pub schema: String,
    pub plan_id: String,
    /// Digest of the exact execution-input snapshot consumed by this result.
    /// Empty is legacy/unbound; stronger consumers must require an exact match.
    #[serde(default)]
    pub input_snapshot: String,
    pub execution_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub attempted_steps: Vec<String>,
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
    pub observed_preconditions: Vec<String>,
    pub evidence: Vec<String>,
    pub resulting_state: CapabilityState,
    pub failure_reason: Option<String>,
    pub claim_ceiling: String,
}

impl RecoveryExecutionResultSnapshotV1 {
    pub const SCHEMA: &'static str = "sol-atlas:recovery-execution-result-snapshot:v1";

    pub fn from_execution(execution: &RecoveryExecution) -> Self {
        Self {
            schema: Self::SCHEMA.into(),
            plan_id: execution.plan_id.clone(),
            input_snapshot: execution.input_snapshot.clone(),
            execution_id: execution.execution_id.clone(),
            started_at: execution.started_at.clone(),
            ended_at: execution.ended_at.clone(),
            attempted_steps: execution.attempted_steps.clone(),
            completed_steps: execution.completed_steps.clone(),
            failed_steps: execution.failed_steps.clone(),
            observed_preconditions: execution.observed_preconditions.clone(),
            evidence: execution.evidence.clone(),
            resulting_state: execution.resulting_state,
            failure_reason: execution.failure_reason.clone(),
            claim_ceiling: execution.claim_ceiling.clone(),
        }
    }

    pub fn is_well_formed(&self) -> bool {
        self.schema == Self::SCHEMA
            && !self.plan_id.is_empty()
            && !self.input_snapshot.is_empty()
            && !self.execution_id.is_empty()
            && is_canonical_utc_timestamp(&self.started_at)
            && self
                .ended_at
                .as_deref()
                .is_some_and(is_canonical_utc_timestamp)
            && !self.attempted_steps.is_empty()
            && self.attempted_steps.iter().all(|step| !step.is_empty())
            && unique_nonempty_strings(&self.attempted_steps)
            && self.completed_steps.iter().all(|step| !step.is_empty())
            && self.failed_steps.iter().all(|step| !step.is_empty())
            && unique_nonempty_strings(&self.completed_steps)
            && unique_nonempty_strings(&self.failed_steps)
            && self
                .observed_preconditions
                .iter()
                .all(|condition| !condition.is_empty())
            && unique_nonempty_strings(&self.observed_preconditions)
            && unique_nonempty_strings(&self.evidence)
            && !self.claim_ceiling.is_empty()
            && self
                .ended_at
                .as_ref()
                .is_some_and(|ended_at| self.started_at.as_str() <= ended_at.as_str())
            && {
                let attempted = self
                    .attempted_steps
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>();
                let completed = self
                    .completed_steps
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>();
                let failed = self
                    .failed_steps
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>();
                completed.is_subset(&attempted)
                    && failed.is_subset(&attempted)
                    && completed.is_disjoint(&failed)
            }
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self)
            .expect("recovery execution result snapshot contains only serializable primitives")
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryExecution {
    /// Recovery plan that authorized/defined the attempted execution.
    pub plan_id: String,
    /// Stable identifier for this concrete execution attempt.
    pub execution_id: String,
    /// Start timestamp/reference for the execution attempt.
    pub started_at: String,
    /// End timestamp/reference; absent while execution is still in progress.
    pub ended_at: Option<String>,
    /// Steps actually attempted, in recorded order.
    pub attempted_steps: Vec<String>,
    /// Steps confirmed completed.
    pub completed_steps: Vec<String>,
    /// Steps that failed or were abandoned.
    pub failed_steps: Vec<String>,
    /// Preconditions observed at execution time.
    pub observed_preconditions: Vec<String>,
    /// Evidence references generated by the execution.
    pub evidence: Vec<String>,
    /// Resulting declared state of the recovered capability.
    pub resulting_state: CapabilityState,
    /// Human/operator authorization reference.
    pub authorization: Option<String>,
    /// AI assistance record; does not itself confer authority.
    pub ai_assistance: Option<String>,
    /// Deterministic input snapshot used for the execution record.
    pub input_snapshot: String,
    /// Failure reason when execution did not complete successfully.
    pub failure_reason: Option<String>,
    /// Exact claim ceiling for this execution record.
    pub claim_ceiling: String,
}

impl RecoveryExecution {
    fn terminal_timestamps_are_well_formed(&self) -> bool {
        self.ended_at.as_deref().is_some_and(|ended_at| {
            is_canonical_utc_timestamp(&self.started_at)
                && is_canonical_utc_timestamp(ended_at)
                && self.started_at.as_str() <= ended_at
        })
    }

    fn attempted_steps_are_unique(&self) -> bool {
        let mut unique = self.attempted_steps.clone();
        unique.sort();
        unique.dedup();

        !self.attempted_steps.is_empty()
            && self.attempted_steps.iter().all(|step| !step.is_empty())
            && unique.len() == self.attempted_steps.len()
    }

    fn completed_and_failed_steps_are_consistent(&self) -> bool {
        let attempted = self
            .attempted_steps
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        let completed = self
            .completed_steps
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        let failed = self
            .failed_steps
            .iter()
            .collect::<std::collections::BTreeSet<_>>();

        self.completed_steps.iter().all(|step| !step.is_empty())
            && self.failed_steps.iter().all(|step| !step.is_empty())
            && completed.is_subset(&attempted)
            && failed.is_subset(&attempted)
            && completed.is_disjoint(&failed)
    }

    fn metadata_is_well_formed(&self) -> bool {
        unique_nonempty_strings(&self.evidence)
            && unique_nonempty_strings(&self.observed_preconditions)
            && self
                .authorization
                .as_ref()
                .is_none_or(|authorization| !authorization.is_empty())
            && self
                .ai_assistance
                .as_ref()
                .is_none_or(|assistance| !assistance.is_empty())
            && !self.claim_ceiling.is_empty()
    }

    /// Recompute and compare the concrete execution-input snapshot against its plan.
    pub fn input_snapshot_matches_plan(&self, plan: &RecoveryPlan) -> bool {
        let snapshot = RecoveryExecutionSnapshotV1::from_plan_and_execution(plan, self);
        snapshot.is_well_formed() && self.input_snapshot == snapshot.digest()
    }

    fn matches_plan_execution_contract(&self, plan: &RecoveryPlan) -> bool {
        self.attempted_steps == plan.steps
            && self.completed_steps == plan.steps
            && plan
                .preconditions
                .iter()
                .all(|condition| self.observed_preconditions.contains(condition))
    }

    /// Stronger success gate binding the execution to the exact ready recovery plan.
    ///
    /// In addition to the cryptographic input snapshot, the concrete execution
    /// must actually attempt and complete the declared plan steps and observe
    /// every declared precondition.
    ///
    /// This still does not establish recovery verification or external qualification.
    pub fn is_successful_with_bound_plan(&self, plan: &RecoveryPlan) -> bool {
        self.is_successful()
            && plan.is_ready()
            && plan.snapshot().is_well_formed()
            && self.plan_id == plan.id
            && self.matches_plan_execution_contract(plan)
            && self.input_snapshot_matches_plan(plan)
    }

    /// Stronger success gate that also binds the execution to an explicit
    /// candidate record at the point of consumption.
    ///
    /// Candidate discovery and policy selection remain external concerns.
    /// This gate only rejects a successful execution when the supplied
    /// candidate is malformed, unresolved, rejected, or inconsistent with
    /// the plan's declared alternative and prerequisite closure.
    pub fn is_successful_with_bound_candidate(
        &self,
        plan: &RecoveryPlan,
        candidate: &RecoveryCandidate,
    ) -> bool {
        self.is_successful()
            && plan.is_ready_against_candidate_snapshot(candidate, &candidate.snapshot())
            && self.plan_id == plan.id
            && self.matches_plan_execution_contract(plan)
            && self.input_snapshot_matches_plan(plan)
    }

    /// Stronger success gate binding the exact candidate, plan, and
    /// external admitted policy decision.
    ///
    /// The policy record is consumed, not interpreted as authority by this core:
    /// its authority reference remains opaque and must be validated by the
    /// external policy/issuer layer.
    pub fn is_successful_with_bound_policy_decision(
        &self,
        plan: &RecoveryPlan,
        candidate: &RecoveryCandidate,
        decision: &RecoveryPolicyDecisionSnapshotV1,
        now: &str,
    ) -> bool {
        self.is_successful_with_bound_candidate(plan, candidate)
            && decision.covers_execution(self, now)
            && decision.digest() == self.authorization.as_deref().unwrap_or_default()
            && decision.plan_id == plan.id
            && decision.plan_snapshot == plan.snapshot().digest()
            && decision.candidate == candidate.candidate
            && decision.candidate_snapshot == candidate.snapshot().digest()
    }

    /// Execution is complete only when it has an end marker and no failed steps.
    ///
    /// This does not establish verification or qualification.
    pub fn is_successful(&self) -> bool {
        self.terminal_timestamps_are_well_formed()
            && !self.plan_id.is_empty()
            && !self.execution_id.is_empty()
            && self.attempted_steps_are_unique()
            && self.completed_steps == self.attempted_steps
            && self.failed_steps.is_empty()
            && self.failure_reason.is_none()
            && self.metadata_is_well_formed()
            && !self.input_snapshot.is_empty()
    }

    /// A failed execution must preserve a terminal marker and an explicit
    /// failure indication rather than silently becoming an unsuccessful success.
    pub fn is_failed(&self) -> bool {
        self.terminal_timestamps_are_well_formed()
            && !self.plan_id.is_empty()
            && !self.execution_id.is_empty()
            && self.completed_and_failed_steps_are_consistent()
            && (self.failed_steps.iter().any(|step| !step.is_empty())
                || self
                    .failure_reason
                    .as_ref()
                    .is_some_and(|reason| !reason.is_empty()))
    }
}

#[cfg(test)]
mod graph_tests {
    use super::*;

    fn verified_test_evidence(label: &str) -> EvidenceReferenceVerificationV1 {
        let preimage = b"deterministic Sol Atlas test evidence";
        let digest = Sha256::digest(preimage);

        let reference = EvidenceReferenceV1::content_addressed(
            "test-evidence-record",
            DigestContextV1 {
                id: "sol-atlas:test-sha256:v1".into(),
                preimage_construction: "canonical test evidence bytes".into(),
                canonicalization: "sol-atlas-test-canonical-v1".into(),
                hash_algorithm: "SHA-256".into(),
                domain_separator: "sol-atlas:test-evidence:v1".into(),
                preimage_encoding: "UTF-8".into(),
                representation: DigestRepresentationV1::PrefixedLowerHex,
            },
            format!("sha256:{digest:x}"),
            "Exact test evidence identity only.",
        )
        .unwrap()
        .with_display_label(label);

        reference
            .verify_preimage(
                preimage,
                "deterministic-test-verifier",
                "2026-10-02T00:00:00Z",
                "Exact test evidence identity verified.",
            )
            .unwrap()
    }

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
    fn recovery_plan_can_bind_only_to_its_declared_resolvable_candidate() {
        let plan = RecoveryPlan {
            id: "plan-candidate-binding".into(),
            unavailable: CapabilityId("water".into()),
            candidate: CapabilityId("filter".into()),
            prerequisites: vec![
                CapabilityId("power".into()),
                CapabilityId("membrane".into()),
            ],
            steps: vec!["install".into(), "test".into()],
            preconditions: vec!["site prepared".into()],
            expected_evidence: vec!["evidence-001".into()],
            human_contribution: String::new(),
            ai_contribution: String::new(),
            state: RecoveryPlanState::Ready,
            claim_ceiling: "Exact recovery plan scope only.".into(),
            candidate_snapshot: String::new(),
        };
        let candidate = RecoveryCandidate {
            for_dependency: CapabilityId("water".into()),
            candidate: CapabilityId("filter".into()),
            required_capabilities: vec![
                CapabilityId("filter".into()),
                CapabilityId("power".into()),
                CapabilityId("membrane".into()),
            ],
            missing_capabilities: vec![],
            evidence: vec![],
            qualification: None,
            selection: RecoverySelectionState::Discovered,
            claim_ceiling: "Declared recovery candidate only.".into(),
        };

        assert!(plan.is_ready_against_candidate(&candidate));

        let mut wrong_candidate = candidate.clone();
        wrong_candidate.candidate = CapabilityId("other-filter".into());
        assert!(!plan.is_ready_against_candidate(&wrong_candidate));

        let mut missing_prerequisite = candidate.clone();
        missing_prerequisite
            .required_capabilities
            .push(CapabilityId("missing".into()));
        assert!(!plan.is_ready_against_candidate(&missing_prerequisite));

        let mut missing_candidate_from_closure = candidate.clone();
        missing_candidate_from_closure
            .required_capabilities
            .remove(0);
        assert!(!plan.is_ready_against_candidate(&missing_candidate_from_closure));

        let mut unresolved = candidate.clone();
        unresolved.missing_capabilities = vec![CapabilityId("membrane".into())];
        assert!(!plan.is_ready_against_candidate(&unresolved));

        let mut rejected = candidate.clone();
        rejected.selection = RecoverySelectionState::Rejected;
        assert!(!plan.is_ready_against_candidate(&rejected));

        let mut malformed = candidate.clone();
        malformed.claim_ceiling.clear();
        assert!(!plan.is_ready_against_candidate(&malformed));

        let mut duplicate_prerequisite = candidate.clone();
        duplicate_prerequisite
            .required_capabilities
            .push(CapabilityId("power".into()));
        assert!(!plan.is_ready_against_candidate(&duplicate_prerequisite));

        let mut self_substitute = candidate.clone();
        self_substitute.candidate = self_substitute.for_dependency.clone();
        assert!(!plan.is_ready_against_candidate(&self_substitute));

        let snapshot = candidate.snapshot();
        assert!(snapshot.is_well_formed());
        assert_eq!(snapshot.digest(), candidate.snapshot().digest());

        let mut malformed_snapshot = snapshot.clone();
        malformed_snapshot.candidate = CapabilityId(String::new());
        assert!(!malformed_snapshot.is_well_formed());

        let mut relabeled = candidate.clone();
        relabeled.claim_ceiling = "different ceiling".into();
        assert_ne!(snapshot.digest(), relabeled.snapshot().digest());

        let mut bound_plan = plan.clone();
        bound_plan.candidate_snapshot = snapshot.digest();
        let plan_snapshot = bound_plan.snapshot();
        assert!(plan_snapshot.is_well_formed());
        assert!(bound_plan.is_exactly_bound_by_snapshot(&plan_snapshot));

        let mut lifecycle_only = bound_plan.clone();
        lifecycle_only.state = RecoveryPlanState::Executing;
        assert_eq!(plan_snapshot.digest(), lifecycle_only.snapshot().digest());

        let mut changed_plan_semantics = bound_plan.clone();
        changed_plan_semantics.claim_ceiling = "different plan ceiling".into();
        assert_ne!(
            plan_snapshot.digest(),
            changed_plan_semantics.snapshot().digest()
        );

        let mut changed_contribution = bound_plan.clone();
        changed_contribution.human_contribution = "different operator context".into();
        assert_ne!(
            plan_snapshot.digest(),
            changed_contribution.snapshot().digest()
        );

        let mut malformed_plan_snapshot = plan_snapshot.clone();
        malformed_plan_snapshot.candidate_snapshot.clear();
        assert!(!malformed_plan_snapshot.is_well_formed());
        assert!(!bound_plan.is_exactly_bound_by_snapshot(&malformed_plan_snapshot));
    }

    #[test]
    fn execution_input_snapshot_binds_exact_plan_and_execution_inputs() {
        let plan = RecoveryPlan {
            id: "plan-input-binding".into(),
            unavailable: CapabilityId("water".into()),
            candidate: CapabilityId("filter".into()),
            prerequisites: vec![CapabilityId("power".into())],
            steps: vec!["install".into(), "test".into()],
            preconditions: vec!["site prepared".into()],
            expected_evidence: vec!["evidence-001".into()],
            human_contribution: String::new(),
            ai_contribution: String::new(),
            state: RecoveryPlanState::Ready,
            claim_ceiling: "Exact recovery plan scope only.".into(),
            candidate_snapshot: String::new(),
        };

        let mut execution = RecoveryExecution {
            plan_id: plan.id.clone(),
            execution_id: "execution-input-binding".into(),
            started_at: "2026-10-02T08:00:00Z".into(),
            ended_at: Some("2026-10-02T08:05:00Z".into()),
            attempted_steps: plan.steps.clone(),
            completed_steps: plan.steps.clone(),
            failed_steps: vec![],
            observed_preconditions: vec!["site prepared".into()],
            evidence: vec!["evidence-001".into()],
            resulting_state: CapabilityState::Demonstrated,
            authorization: Some("authorization-001".into()),
            ai_assistance: None,
            input_snapshot: String::new(),
            failure_reason: None,
            claim_ceiling: "Exact execution scope only.".into(),
        };

        execution.input_snapshot =
            RecoveryExecutionSnapshotV1::from_plan_and_execution(&plan, &execution).digest();

        let candidate = RecoveryCandidate {
            for_dependency: CapabilityId("water".into()),
            candidate: CapabilityId("filter".into()),
            required_capabilities: vec![
                CapabilityId("filter".into()),
                CapabilityId("power".into()),
            ],
            missing_capabilities: vec![],
            evidence: vec![],
            qualification: None,
            selection: RecoverySelectionState::Discovered,
            claim_ceiling: "Declared recovery candidate only.".into(),
        };

        let mut plan = plan;
        plan.candidate_snapshot = candidate.snapshot().digest();
        execution.input_snapshot =
            RecoveryExecutionSnapshotV1::from_plan_and_execution(&plan, &execution).digest();

        assert!(plan.snapshot().is_well_formed());
        assert!(execution.input_snapshot_matches_plan(&plan));
        assert!(execution.is_successful_with_bound_plan(&plan));
        assert!(execution.is_successful_with_bound_candidate(&plan, &candidate));

        let mut changed_candidate = candidate.clone();
        changed_candidate
            .required_capabilities
            .push(CapabilityId("membrane".into()));
        assert!(!execution.is_successful_with_bound_candidate(&plan, &changed_candidate));

        let mut stale_plan = plan.clone();
        stale_plan.candidate_snapshot = "sha256:stale-candidate".into();
        assert!(!execution.is_successful_with_bound_candidate(&stale_plan, &candidate));

        let mut rejected_candidate = candidate;
        rejected_candidate.selection = RecoverySelectionState::Rejected;
        assert!(!execution.is_successful_with_bound_candidate(&plan, &rejected_candidate));

        let mut changed_plan = plan.clone();
        changed_plan.steps[1] = "independent-test".into();
        assert!(!execution.input_snapshot_matches_plan(&changed_plan));

        let mut changed_candidate_binding = plan.clone();
        changed_candidate_binding.candidate_snapshot = "sha256:different-candidate".into();
        assert!(!execution.input_snapshot_matches_plan(&changed_candidate_binding));

        let mut substituted_steps = execution.clone();
        substituted_steps.attempted_steps[1] = "independent-test".into();
        substituted_steps.completed_steps = substituted_steps.attempted_steps.clone();
        assert!(!substituted_steps.is_successful_with_bound_plan(&plan));

        let mut missed_precondition = execution.clone();
        missed_precondition.observed_preconditions[0] = "site NOT prepared".into();
        missed_precondition.input_snapshot =
            RecoveryExecutionSnapshotV1::from_plan_and_execution(&plan, &missed_precondition)
                .digest();
        assert!(!missed_precondition.is_successful_with_bound_plan(&plan));

        let mut changed_inputs = execution.clone();
        changed_inputs.authorization = Some("different-authorization".into());
        assert!(!changed_inputs.input_snapshot_matches_plan(&plan));

        let mut malformed = RecoveryExecutionSnapshotV1::from_plan_and_execution(&plan, &execution);
        malformed.execution_claim_ceiling.clear();
        assert!(!malformed.is_well_formed());

        let result_snapshot = RecoveryExecutionResultSnapshotV1::from_execution(&execution);
        assert!(result_snapshot.is_well_formed());

        let mut malformed_result_snapshot = result_snapshot.clone();
        malformed_result_snapshot.claim_ceiling.clear();
        assert!(!malformed_result_snapshot.is_well_formed());

        let mut detached_result_snapshot = result_snapshot.clone();
        detached_result_snapshot.input_snapshot.clear();
        assert!(!detached_result_snapshot.is_well_formed());

        let mut changed_result = execution.clone();
        changed_result.resulting_state = CapabilityState::Deployed;
        assert_ne!(
            result_snapshot.digest(),
            RecoveryExecutionResultSnapshotV1::from_execution(&changed_result).digest()
        );
    }

    #[test]
    fn successful_execution_requires_exact_completed_steps_and_metadata() {
        let execution = RecoveryExecution {
            plan_id: "plan-001".into(),
            execution_id: "execution-001".into(),
            started_at: "2026-10-02T08:00:00Z".into(),
            ended_at: Some("2026-10-02T08:05:00Z".into()),
            attempted_steps: vec!["install".into(), "test".into()],
            completed_steps: vec!["install".into(), "test".into()],
            failed_steps: vec![],
            observed_preconditions: vec!["site ready".into()],
            evidence: vec!["evidence-001".into()],
            resulting_state: CapabilityState::Demonstrated,
            authorization: Some("auth-001".into()),
            ai_assistance: Some("planning assistance".into()),
            input_snapshot: "sha256:execution-inputs".into(),
            failure_reason: None,
            claim_ceiling: "Exact execution record only.".into(),
        };
        assert!(execution.is_successful());

        let mut wrong_completed = execution.clone();
        wrong_completed.completed_steps = vec!["install".into(), "publish".into()];
        assert!(!wrong_completed.is_successful());

        let mut incomplete = execution.clone();
        incomplete.input_snapshot.clear();
        assert!(!incomplete.is_successful());

        let mut blank_evidence = execution.clone();
        blank_evidence.evidence[0].clear();
        assert!(!blank_evidence.is_successful());

        let mut duplicate_evidence = execution.clone();
        duplicate_evidence.evidence.push("evidence-001".into());
        assert!(!duplicate_evidence.is_successful());

        let mut duplicate_observation = execution.clone();
        duplicate_observation
            .observed_preconditions
            .push("site ready".into());
        assert!(!duplicate_observation.is_successful());

        let mut blank_observation = execution.clone();
        blank_observation.observed_preconditions.push(String::new());
        assert!(!blank_observation.is_successful());

        let mut blank_authorization = execution.clone();
        blank_authorization.authorization = Some(String::new());
        assert!(!blank_authorization.is_successful());

        let mut blank_ai_assistance = execution.clone();
        blank_ai_assistance.ai_assistance = Some(String::new());
        assert!(!blank_ai_assistance.is_successful());

        let mut blank_claim_ceiling = execution.clone();
        blank_claim_ceiling.claim_ceiling.clear();
        assert!(!blank_claim_ceiling.is_successful());

        let mut malformed_time = execution.clone();
        malformed_time.started_at = "yesterday".into();
        assert!(!malformed_time.is_successful());
    }

    #[test]
    fn failed_execution_is_terminal_only_when_a_failure_is_recorded() {
        let mut execution = RecoveryExecution {
            plan_id: "plan-002".into(),
            execution_id: "execution-002".into(),
            started_at: "2026-10-02T09:00:00Z".into(),
            ended_at: Some("2026-10-02T09:02:00Z".into()),
            attempted_steps: vec!["prepare".into()],
            completed_steps: vec![],
            failed_steps: vec!["prepare".into()],
            observed_preconditions: vec![],
            evidence: vec![],
            resulting_state: CapabilityState::Conceptual,
            authorization: None,
            ai_assistance: None,
            input_snapshot: "sha256:execution-inputs".into(),
            failure_reason: Some("precondition failed".into()),
            claim_ceiling: "Exact failure record only.".into(),
        };
        assert!(execution.is_failed());

        execution.ended_at = None;
        assert!(!execution.is_failed());
    }

    #[test]
    fn execution_success_rejects_reversed_and_duplicate_steps() {
        let base = RecoveryExecution {
            plan_id: "plan-execution-integrity".into(),
            execution_id: "execution-integrity".into(),
            started_at: "2026-10-02T08:00:00Z".into(),
            ended_at: Some("2026-10-02T08:05:00Z".into()),
            attempted_steps: vec!["install".into(), "test".into()],
            completed_steps: vec!["install".into(), "test".into()],
            failed_steps: vec![],
            observed_preconditions: vec![],
            evidence: vec!["evidence-001".into()],
            resulting_state: CapabilityState::Demonstrated,
            authorization: Some("authorization-001".into()),
            ai_assistance: None,
            input_snapshot: "sha256:execution-inputs".into(),
            failure_reason: None,
            claim_ceiling: "Exact execution record only.".into(),
        };
        assert!(base.is_successful());

        let mut reversed = base.clone();
        reversed.ended_at = Some("2026-10-02T07:59:59Z".into());
        assert!(!reversed.is_successful());

        let mut duplicate = base.clone();
        duplicate.attempted_steps = vec!["install".into(), "install".into()];
        duplicate.completed_steps = duplicate.attempted_steps.clone();
        assert!(!duplicate.is_successful());

        let mut empty = base.clone();
        empty.attempted_steps[1].clear();
        empty.completed_steps = empty.attempted_steps.clone();
        assert!(!empty.is_successful());
    }

    #[test]
    fn failed_execution_rejects_malformed_terminal_metadata() {
        let mut execution = RecoveryExecution {
            plan_id: "plan-failure-integrity".into(),
            execution_id: "execution-failure-integrity".into(),
            started_at: "2026-10-02T09:00:00Z".into(),
            ended_at: Some("2026-10-02T09:02:00Z".into()),
            attempted_steps: vec!["prepare".into()],
            completed_steps: vec![],
            failed_steps: vec!["prepare".into()],
            observed_preconditions: vec![],
            evidence: vec![],
            resulting_state: CapabilityState::Conceptual,
            authorization: None,
            ai_assistance: None,
            input_snapshot: "sha256:execution-inputs".into(),
            failure_reason: Some("precondition failed".into()),
            claim_ceiling: "Exact failure record only.".into(),
        };
        assert!(execution.is_failed());

        execution.ended_at = Some("2026-10-02T08:59:59Z".into());
        assert!(!execution.is_failed());

        execution.ended_at = Some("2026-10-02T09:02:00Z".into());
        execution.failure_reason = Some(String::new());
        execution.failed_steps.clear();
        assert!(!execution.is_failed());
    }

    #[test]
    fn verification_snapshot_binds_derived_component_digests() {
        let graph = CapabilityGraph {
            capabilities: vec![cap("a", &["b"]), cap("b", &[])],
        };
        let dependency = graph
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap();

        let mut capability = cap("a", &[]);
        capability.evidence = vec![CapabilityEvidence {
            kind: EvidenceKind::Observed,
            reference: "evidence-a".into(),
            claim_ceiling: "exact observation".into(),
        }];
        let evidence =
            EvidenceSnapshotV1::from_capability(&capability, RecoveryEvidenceCoverage::ClosedWorld);
        let environment = EnvironmentSnapshotV1::from_facts(
            CapabilityId("a".into()),
            "site-1",
            vec![EnvironmentFactV1 {
                key: "temperature".into(),
                value: "20".into(),
                unit: Some("C".into()),
                source: "sensor-a".into(),
            }],
        );

        let verification = RecoveryVerification {
            execution_id: "execution-a".into(),
            capability: CapabilityId("a".into()),
            scope: "site-1".into(),
            expected_postconditions: vec!["operational".into()],
            observed_postconditions: vec!["operational".into()],
            evidence: vec![verified_test_evidence("evidence-a")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("a".into()), CapabilityId("b".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: String::new(),
            environment_snapshot: String::new(),
            evidence_snapshot: String::new(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "9999-12-31T23:59:59Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Pending,
            verifier: "test-verifier".into(),
            verified_at: "2026-10-02T00:00:00Z".into(),
            claim_ceiling: "Exact verification scope only.".into(),
            execution_result_snapshot: String::new(),
        };

        let snapshot = RecoveryVerificationSnapshotV1::from_verification_with_snapshots(
            &verification,
            &dependency,
            &evidence,
            &environment,
        )
        .unwrap();

        assert_eq!(snapshot.dependency_snapshot, dependency.digest());
        assert_eq!(snapshot.evidence_snapshot, evidence.digest());
        assert_eq!(snapshot.environment_snapshot, environment.digest());
    }

    #[test]
    fn verification_snapshot_binds_evidence_authorization_identity() {
        let verification = RecoveryVerification {
            execution_id: "authorization-binding".into(),
            capability: CapabilityId("a".into()),
            scope: "site-1".into(),
            expected_postconditions: vec!["operational".into()],
            observed_postconditions: vec!["operational".into()],
            evidence: vec![verified_test_evidence("evidence-a")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("a".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: "sha256:dependency".into(),
            environment_snapshot: "sha256:environment".into(),
            evidence_snapshot: "sha256:evidence".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "9999-12-31T23:59:59Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "test-verifier".into(),
            verified_at: "2026-10-02T00:00:00Z".into(),
            claim_ceiling: "Exact verification scope only.".into(),
            execution_result_snapshot: String::new(),
        };

        let base = RecoveryVerificationSnapshotV1::from_verification(&verification);
        assert!(!base.evidence_authorization_snapshot.is_empty());

        let mut profile_bound = verification.clone();
        profile_bound.evidence[0].profile_id = Some("profile-001".into());
        profile_bound.evidence[0].profile_version = Some(1);
        profile_bound.evidence[0].profile_digest =
            Some("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into());

        let profile_snapshot = RecoveryVerificationSnapshotV1::from_verification(&profile_bound);
        assert_ne!(
            base.evidence_authorization_snapshot,
            profile_snapshot.evidence_authorization_snapshot
        );
        assert_ne!(base.digest(), profile_snapshot.digest());

        let mut registry_bound = profile_bound.clone();
        registry_bound.evidence[0].registry_id = Some("registry-001".into());
        registry_bound.evidence[0].registry_version = Some(1);
        registry_bound.evidence[0].registry_digest =
            Some("sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into());

        let registry_snapshot = RecoveryVerificationSnapshotV1::from_verification(&registry_bound);
        assert_ne!(
            profile_snapshot.evidence_authorization_snapshot,
            registry_snapshot.evidence_authorization_snapshot
        );
        assert_ne!(profile_snapshot.digest(), registry_snapshot.digest());
    }

    #[test]
    fn bound_verification_requires_evidence_snapshot_coverage_alignment() {
        let graph = CapabilityGraph {
            capabilities: vec![cap("a", &[])],
        };
        let dependency = graph
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap();
        let evidence_reference = verified_test_evidence("coverage-check");
        let evidence = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                claim_ceiling: evidence_reference.reference.claim_ceiling.clone(),
                reference: evidence_reference.reference.clone(),
                unresolved_locator: None,
            }],
        );
        let environment = EnvironmentSnapshotV1::from_facts(
            CapabilityId("a".into()),
            "site-1",
            vec![EnvironmentFactV1 {
                key: "temperature".into(),
                value: "20".into(),
                unit: Some("C".into()),
                source: "sensor-a".into(),
            }],
        );
        let mut verification = RecoveryVerification {
            execution_id: "coverage-bound".into(),
            capability: CapabilityId("a".into()),
            scope: "site-1".into(),
            expected_postconditions: vec!["operational".into()],
            observed_postconditions: vec!["operational".into()],
            evidence: vec![evidence_reference],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("a".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: dependency.digest(),
            environment_snapshot: environment.digest(),
            evidence_snapshot: evidence.digest(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "9999-12-31T23:59:59Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "coverage-verifier".into(),
            verified_at: "2026-10-02T08:00:00Z".into(),
            claim_ceiling: "Exact coverage scope only.".into(),
            execution_result_snapshot: String::new(),
        };
        verification.verification_snapshot =
            RecoveryVerificationSnapshotV1::from_verification_with_snapshots(
                &verification,
                &dependency,
                &evidence,
                &environment,
            )
            .unwrap()
            .digest();
        assert!(verification.passes_with_bound_snapshots(&dependency, &evidence, &environment));

        let mut open_world = evidence.clone();
        open_world.coverage = RecoveryEvidenceCoverage::OpenWorld;
        let mut open_verification = verification.clone();
        open_verification.evidence_snapshot = open_world.digest();
        open_verification.verification_snapshot =
            RecoveryVerificationSnapshotV1::from_verification_with_snapshots(
                &open_verification,
                &dependency,
                &open_world,
                &environment,
            )
            .unwrap()
            .digest();
        assert!(!open_verification.passes_with_bound_snapshots(
            &dependency,
            &open_world,
            &environment
        ));

        let execution = RecoveryExecution {
            plan_id: "plan-coverage-bound".into(),
            execution_id: "coverage-bound".into(),
            started_at: "2026-10-02T07:59:00Z".into(),
            ended_at: Some("2026-10-02T08:00:00Z".into()),
            attempted_steps: vec!["verify".into()],
            completed_steps: vec!["verify".into()],
            failed_steps: vec![],
            observed_preconditions: vec![],
            evidence: vec!["coverage-check".into()],
            resulting_state: CapabilityState::Demonstrated,
            authorization: Some("verification-authorized".into()),
            ai_assistance: None,
            input_snapshot: "sha256:execution-inputs".into(),
            failure_reason: None,
            claim_ceiling: "Exact execution scope only.".into(),
        };
        let mut verification = verification;
        verification.execution_result_snapshot =
            RecoveryExecutionResultSnapshotV1::from_execution(&execution).digest();

        assert!(verification.passes_with_bound_execution(&execution));

        let mut changed_input = execution.clone();
        changed_input.input_snapshot = "sha256:different-input".into();
        assert!(!verification.passes_with_bound_execution(&changed_input));

        let mut wrong_execution_id = execution.clone();
        wrong_execution_id.execution_id = "different-execution".into();
        assert!(!verification.passes_with_bound_execution(&wrong_execution_id));

        let mut failed_execution = execution;
        failed_execution.failure_reason = Some("verification step failed".into());
        assert!(!verification.passes_with_bound_execution(&failed_execution));

        let candidate = RecoveryCandidate {
            for_dependency: CapabilityId("a".into()),
            candidate: CapabilityId("recovery".into()),
            required_capabilities: vec![CapabilityId("recovery".into())],
            missing_capabilities: vec![],
            evidence: vec![],
            qualification: None,
            selection: RecoverySelectionState::Discovered,
            claim_ceiling: "Declared recovery candidate only.".into(),
        };
        let mut plan = RecoveryPlan {
            id: "plan-coverage-bound".into(),
            unavailable: CapabilityId("a".into()),
            candidate: CapabilityId("recovery".into()),
            candidate_snapshot: candidate.snapshot().digest(),
            prerequisites: vec![CapabilityId("recovery".into())],
            steps: vec!["verify".into()],
            preconditions: vec![],
            expected_evidence: vec!["coverage-check".into()],
            human_contribution: String::new(),
            ai_contribution: String::new(),
            state: RecoveryPlanState::Ready,
            claim_ceiling: "Exact plan scope only.".into(),
        };
        let original_plan = plan.clone();

        let mut bound_execution = execution.clone();
        bound_execution.input_snapshot =
            RecoveryExecutionSnapshotV1::from_plan_and_execution(&plan, &bound_execution).digest();
        verification.execution_result_snapshot =
            RecoveryExecutionResultSnapshotV1::from_execution(&bound_execution).digest();

        assert!(verification.passes_with_bound_plan_execution(&plan, &bound_execution));
        assert!(verification.passes_with_bound_candidate_execution(
            &plan,
            &candidate,
            &bound_execution
        ));

        let decision = RecoveryPolicyDecisionSnapshotV1 {
            schema: RecoveryPolicyDecisionSnapshotV1::SCHEMA.into(),
            id: "permit-coverage-bound".into(),
            decision: RecoveryPolicyDecisionV1::Admitted,
            purpose: "recovery.execute".into(),
            consumer: "operator-001".into(),
            plan_id: plan.id.clone(),
            plan_snapshot: plan.snapshot().digest(),
            candidate: candidate.candidate.clone(),
            candidate_snapshot: candidate.snapshot().digest(),
            authority_reference: "authority-record-001".into(),
            issued_at: "2026-10-02T07:50:00Z".into(),
            valid_until: "2026-10-02T08:10:00Z".into(),
            claim_ceiling: "Exact recovery admission only.".into(),
        };

        bound_execution.authorization = Some(decision.digest());
        bound_execution.input_snapshot =
            RecoveryExecutionSnapshotV1::from_plan_and_execution(&plan, &bound_execution).digest();
        verification.execution_result_snapshot =
            RecoveryExecutionResultSnapshotV1::from_execution(&bound_execution).digest();

        assert!(decision.is_well_formed());
        assert!(decision.is_valid_at("2026-10-02T08:00:00Z"));
        assert!(!decision.is_valid_at("2026-10-02T08:10:00Z"));
        assert!(!decision.covers_execution(&bound_execution, "2026-10-02T07:58:00Z"));
        assert!(!decision.covers_execution(&bound_execution, "2026-10-02T07:59:00Z"));
        assert!(decision.covers_execution(&bound_execution, "2026-10-02T08:00:00Z"));
        let mut late_issued = decision.clone();
        late_issued.issued_at = "2026-10-02T08:01:00Z".into();
        assert!(!late_issued.covers_execution(&bound_execution, "2026-10-02T08:05:00Z"));
        assert!(bound_execution.is_successful_with_bound_policy_decision(
            &plan,
            &candidate,
            &decision,
            "2026-10-02T08:00:00Z"
        ));
        assert!(verification.passes_with_bound_policy_decision(
            &plan,
            &candidate,
            &decision,
            &bound_execution,
            "2026-10-02T08:00:00Z"
        ));

        let mut wrong_purpose = decision.clone();
        wrong_purpose.purpose = "different-purpose".into();
        assert_ne!(decision.digest(), wrong_purpose.digest());
        assert!(!bound_execution.is_successful_with_bound_policy_decision(
            &plan,
            &candidate,
            &wrong_purpose,
            "2026-10-02T08:00:00Z"
        ));

        let mut wrong_consumer = decision.clone();
        wrong_consumer.consumer = "different-consumer".into();
        assert!(!bound_execution.is_successful_with_bound_policy_decision(
            &plan,
            &candidate,
            &wrong_consumer,
            "2026-10-02T08:00:00Z"
        ));

        let mut expired = decision.clone();
        expired.valid_until = "2026-10-02T07:59:00Z".into();
        assert!(!bound_execution.is_successful_with_bound_policy_decision(
            &plan,
            &candidate,
            &expired,
            "2026-10-02T08:00:00Z"
        ));

        let mut rejected = decision.clone();
        rejected.decision = RecoveryPolicyDecisionV1::Rejected;
        assert!(rejected.is_well_formed());
        assert!(!rejected.is_admitted());
        assert!(!rejected.is_valid_at("2026-10-02T08:00:00Z"));
        assert_ne!(decision.digest(), rejected.digest());
        assert!(!bound_execution.is_successful_with_bound_policy_decision(
            &plan,
            &candidate,
            &rejected,
            "2026-10-02T08:00:00Z"
        ));

        plan.claim_ceiling = "mutated plan".into();
        assert!(!verification.passes_with_bound_plan_execution(&plan, &bound_execution));

        let mut stale_candidate = candidate;
        stale_candidate.claim_ceiling = "mutated candidate".into();
        assert!(!verification.passes_with_bound_candidate_execution(
            &original_plan,
            &stale_candidate,
            &bound_execution
        ));
    }

    #[test]
    fn evidence_snapshot_requires_bijective_reference_matching() {
        let verification = verified_test_evidence("duplicate-check");
        let mut snapshot = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                reference: verification.reference.clone(),
                claim_ceiling: verification.reference.claim_ceiling.clone(),
                unresolved_locator: None,
            }],
        );
        assert!(snapshot.all_references_verified(std::slice::from_ref(&verification)));

        snapshot.evidence.push(snapshot.evidence[0].clone());
        let duplicate_verifications = vec![verification.clone(), verification];
        assert!(!snapshot.all_references_verified(&duplicate_verifications));
    }

    #[test]
    fn provenance_complete_verification_requires_exact_bound_component_snapshots() {
        let graph = CapabilityGraph {
            capabilities: vec![cap("a", &["b"]), cap("b", &[])],
        };
        let dependency = graph
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap();

        let evidence_reference = verified_test_evidence("bound-evidence");
        let evidence = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                claim_ceiling: evidence_reference.reference.claim_ceiling.clone(),
                reference: evidence_reference.reference.clone(),
                unresolved_locator: None,
            }],
        );
        let environment = EnvironmentSnapshotV1::from_facts(
            CapabilityId("a".into()),
            "site-1",
            vec![EnvironmentFactV1 {
                key: "temperature".into(),
                value: "20".into(),
                unit: Some("C".into()),
                source: "sensor-a".into(),
            }],
        );

        assert!(evidence.all_references_verified(&[evidence_reference.clone()]));

        let mut verification = RecoveryVerification {
            execution_id: "execution-bound".into(),
            capability: CapabilityId("a".into()),
            scope: "site-1".into(),
            expected_postconditions: vec!["operational".into()],
            observed_postconditions: vec!["operational".into()],
            evidence: vec![evidence_reference],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: dependency.nodes.clone(),
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: dependency.digest(),
            environment_snapshot: environment.digest(),
            evidence_snapshot: evidence.digest(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "9999-12-31T23:59:59Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T00:00:00Z".into(),
            claim_ceiling: "Exact verification scope only.".into(),
            execution_result_snapshot: String::new(),
        };

        verification.verification_snapshot =
            RecoveryVerificationSnapshotV1::from_verification_with_snapshots(
                &verification,
                &dependency,
                &evidence,
                &environment,
            )
            .unwrap()
            .digest();

        assert!(verification.passes_with_bound_snapshots(&dependency, &evidence, &environment));

        let mut bad_snapshot = verification.clone();
        bad_snapshot.evidence_snapshot = "opaque-evidence-label".into();
        bad_snapshot.verification_snapshot = bad_snapshot.derived_snapshot().digest();
        assert!(bad_snapshot.passes());
        assert!(!bad_snapshot.passes_with_bound_snapshots(&dependency, &evidence, &environment));

        let mut changed_environment = environment.clone();
        changed_environment.facts[0].value = "21".into();
        assert!(!verification.passes_with_bound_snapshots(
            &dependency,
            &evidence,
            &changed_environment
        ));
    }

    #[test]
    fn duplicate_capability_ids_fail_closed_in_graph_operations() {
        let graph = CapabilityGraph {
            capabilities: vec![cap("a", &[]), cap("a", &[])],
        };

        let error = graph
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap_err();
        assert!(error.missing.is_empty());
        assert_eq!(error.duplicate_ids, vec![CapabilityId("a".into())]);

        let closure_error = graph
            .required_closure(&CapabilityId("a".into()))
            .unwrap_err();
        assert!(closure_error.missing.is_empty());
        assert_eq!(closure_error.duplicate_ids, vec![CapabilityId("a".into())]);

        let impact = graph.affected_by(&CapabilityId("a".into()));
        assert!(impact.direct_affected.is_empty());
        assert!(impact.transitive_affected.is_empty());
        assert!(impact.affected.is_empty());
        assert_eq!(impact.unresolved, vec![CapabilityId("a".into())]);

        assert!(
            graph
                .recovery_candidates(&CapabilityId("a".into()))
                .is_empty()
        );
    }

    #[test]
    fn recovery_plan_readiness_rejects_ambiguous_or_incomplete_structure() {
        let base = RecoveryPlan {
            id: "plan-ready".into(),
            unavailable: CapabilityId("water".into()),
            candidate: CapabilityId("filter".into()),
            prerequisites: vec![CapabilityId("power".into())],
            steps: vec!["install".into(), "test".into()],
            preconditions: vec!["site prepared".into()],
            expected_evidence: vec!["evidence-001".into()],
            human_contribution: "operator".into(),
            ai_contribution: "planning assistance".into(),
            state: RecoveryPlanState::Ready,
            claim_ceiling: "Exact recovery plan scope only.".into(),
            candidate_snapshot: String::new(),
        };
        assert!(base.is_ready());

        let mut duplicate_steps = base.clone();
        duplicate_steps.steps[1] = duplicate_steps.steps[0].clone();
        assert!(!duplicate_steps.is_ready());

        let mut blank_step = base.clone();
        blank_step.steps[0].clear();
        assert!(!blank_step.is_ready());

        let mut duplicate_prerequisite = base.clone();
        duplicate_prerequisite
            .prerequisites
            .push(CapabilityId("power".into()));
        assert!(!duplicate_prerequisite.is_ready());

        let mut duplicate_precondition = base.clone();
        duplicate_precondition
            .preconditions
            .push(duplicate_precondition.preconditions[0].clone());
        assert!(!duplicate_precondition.is_ready());

        let mut duplicate_expected_evidence = base.clone();
        duplicate_expected_evidence
            .expected_evidence
            .push(duplicate_expected_evidence.expected_evidence[0].clone());
        assert!(!duplicate_expected_evidence.is_ready());

        let mut blank_evidence = base;
        blank_evidence.expected_evidence[0].clear();
        assert!(!blank_evidence.is_ready());

        let mut blank_claim_ceiling = blank_evidence;
        blank_claim_ceiling.expected_evidence[0] = "evidence-001".into();
        blank_claim_ceiling.claim_ceiling.clear();
        assert!(!blank_claim_ceiling.is_ready());
    }

    #[test]
    fn dependency_snapshot_rejects_empty_capability_ids() {
        let snapshot = DependencySnapshotV1 {
            schema: DependencySnapshotV1::SCHEMA.into(),
            root: CapabilityId("a".into()),
            nodes: vec![CapabilityId("a".into()), CapabilityId(String::new())],
            edges: vec![],
        };
        assert!(!snapshot.is_well_formed());
    }

    #[test]
    fn failed_execution_rejects_unknown_or_conflicting_steps() {
        let base = RecoveryExecution {
            plan_id: "plan-failure-consistency".into(),
            execution_id: "execution-failure-consistency".into(),
            started_at: "2026-10-02T09:00:00Z".into(),
            ended_at: Some("2026-10-02T09:02:00Z".into()),
            attempted_steps: vec!["prepare".into(), "install".into()],
            completed_steps: vec!["prepare".into()],
            failed_steps: vec!["install".into()],
            observed_preconditions: vec![],
            evidence: vec![],
            resulting_state: CapabilityState::Conceptual,
            authorization: None,
            ai_assistance: None,
            input_snapshot: "sha256:execution-inputs".into(),
            failure_reason: Some("install failed".into()),
            claim_ceiling: "Exact failure record only.".into(),
        };
        assert!(base.is_failed());

        let mut unknown = base.clone();
        unknown.failed_steps = vec!["publish".into()];
        assert!(!unknown.is_failed());

        let mut conflict = base;
        conflict.failed_steps = vec!["prepare".into()];
        assert!(!conflict.is_failed());
    }

    #[test]
    fn component_snapshots_reject_malformed_schema_and_fields() {
        let mut environment = EnvironmentSnapshotV1::from_facts(
            CapabilityId("a".into()),
            "site-1",
            vec![EnvironmentFactV1 {
                key: "temperature".into(),
                value: "20".into(),
                unit: Some("C".into()),
                source: "sensor-a".into(),
            }],
        );
        assert!(environment.is_well_formed());
        environment.schema = "wrong-schema".into();
        assert!(!environment.is_well_formed());

        let mut evidence = EvidenceSnapshotV1 {
            schema: EvidenceSnapshotV1::SCHEMA.into(),
            subject: CapabilityId("a".into()),
            evidence: vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                reference: verified_test_evidence("malformed-component").reference,
                claim_ceiling: "different ceiling".into(),
                unresolved_locator: None,
            }],
            coverage: RecoveryEvidenceCoverage::ClosedWorld,
        };
        assert!(!evidence.is_well_formed());
        evidence.evidence[0].claim_ceiling = evidence.evidence[0].reference.claim_ceiling.clone();
        assert!(evidence.is_well_formed());
    }

    #[test]
    fn dependency_snapshot_rejects_duplicate_or_invalid_structure() {
        let mut duplicate_nodes = DependencySnapshotV1 {
            schema: DependencySnapshotV1::SCHEMA.into(),
            root: CapabilityId("a".into()),
            nodes: vec![CapabilityId("a".into()), CapabilityId("a".into())],
            edges: vec![],
        };
        assert!(!duplicate_nodes.is_well_formed());

        duplicate_nodes.nodes = vec![CapabilityId("a".into()), CapabilityId("b".into())];
        duplicate_nodes.edges = vec![DependencySnapshotEdgeV1 {
            from: CapabilityId("a".into()),
            to: CapabilityId("missing".into()),
            relation: DependencyKind::Required,
        }];
        assert!(!duplicate_nodes.is_well_formed());

        duplicate_nodes.edges.clear();
        duplicate_nodes.nodes = vec![CapabilityId("a".into())];
        duplicate_nodes.schema = "wrong-schema".into();
        assert!(!duplicate_nodes.is_well_formed());
    }

    #[test]
    fn dependency_snapshot_binds_exact_verification_closure() {
        let graph = CapabilityGraph {
            capabilities: vec![cap("a", &["b"]), cap("b", &[])],
        };
        let dependency = graph
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap();
        assert!(dependency.is_well_formed());

        let evidence_reference = verified_test_evidence("closure-binding");
        let evidence = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                claim_ceiling: evidence_reference.reference.claim_ceiling.clone(),
                reference: evidence_reference.reference.clone(),
                unresolved_locator: None,
            }],
        );
        let environment = EnvironmentSnapshotV1::from_facts(
            CapabilityId("a".into()),
            "site-1",
            vec![EnvironmentFactV1 {
                key: "temperature".into(),
                value: "20".into(),
                unit: Some("C".into()),
                source: "sensor-a".into(),
            }],
        );
        let mut verification = RecoveryVerification {
            execution_id: "closure-binding".into(),
            capability: CapabilityId("a".into()),
            scope: "site-1".into(),
            expected_postconditions: vec!["operational".into()],
            observed_postconditions: vec!["operational".into()],
            evidence: vec![evidence_reference],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: dependency.nodes.clone(),
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: dependency.digest(),
            environment_snapshot: environment.digest(),
            evidence_snapshot: evidence.digest(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "9999-12-31T23:59:59Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "closure-verifier".into(),
            verified_at: "2026-10-02T08:00:00Z".into(),
            claim_ceiling: "Exact closure scope only.".into(),
            execution_result_snapshot: String::new(),
        };
        verification.verification_snapshot =
            RecoveryVerificationSnapshotV1::from_verification_with_snapshots(
                &verification,
                &dependency,
                &evidence,
                &environment,
            )
            .unwrap()
            .digest();

        assert!(verification.passes_with_bound_snapshots(&dependency, &evidence, &environment));

        verification.dependency_closure = vec![CapabilityId("a".into())];
        verification.verification_snapshot =
            RecoveryVerificationSnapshotV1::from_verification_with_snapshots(
                &verification,
                &dependency,
                &evidence,
                &environment,
            )
            .unwrap()
            .digest();
        assert!(!verification.passes_with_bound_snapshots(&dependency, &evidence, &environment));
    }

    #[test]
    fn dependency_snapshot_is_derived_from_the_actual_required_graph() {
        let graph = CapabilityGraph {
            capabilities: vec![
                cap("a", &["b"]),
                cap("b", &["c"]),
                cap("c", &[]),
                cap("ignored", &[]),
            ],
        };

        let snapshot = graph
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap();

        assert_eq!(snapshot.root, CapabilityId("a".into()));
        assert_eq!(
            snapshot.nodes,
            vec![
                CapabilityId("a".into()),
                CapabilityId("b".into()),
                CapabilityId("c".into())
            ]
        );
        assert_eq!(snapshot.edges.len(), 2);
        assert_eq!(snapshot.edges[0].from, CapabilityId("a".into()));
        assert_eq!(snapshot.edges[0].to, CapabilityId("b".into()));
        assert_eq!(snapshot.edges[0].relation, DependencyKind::Required);
        assert!(!snapshot.nodes.contains(&CapabilityId("ignored".into())));
        assert!(snapshot.digest().starts_with("sha256:"));
    }

    #[test]
    fn dependency_snapshot_is_order_invariant_and_changes_on_graph_mutation() {
        let first = CapabilityGraph {
            capabilities: vec![cap("a", &["b"]), cap("b", &["c"]), cap("c", &[])],
        };
        let second = CapabilityGraph {
            capabilities: vec![cap("c", &[]), cap("b", &["c"]), cap("a", &["b"])],
        };

        let first_digest = first
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap()
            .digest();
        let second_digest = second
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap()
            .digest();
        assert_eq!(first_digest, second_digest);

        let mutated = CapabilityGraph {
            capabilities: vec![cap("a", &["b"]), cap("b", &["d"]), cap("c", &[])],
        };
        let mutated_result = mutated.dependency_snapshot(&CapabilityId("a".into()));
        assert_eq!(
            mutated_result.unwrap_err().missing,
            vec![CapabilityId("d".into())]
        );
    }

    #[test]
    fn dependency_snapshot_excludes_non_required_relations_and_alternatives() {
        let graph = CapabilityGraph {
            capabilities: vec![
                Capability {
                    id: CapabilityId("a".into()),
                    name: "a".into(),
                    description: String::new(),
                    state: CapabilityState::Demonstrated,
                    dependencies: vec![
                        CapabilityDependency {
                            capability: CapabilityId("b".into()),
                            relation: DependencyKind::Required,
                            substitutes: vec![CapabilityId("alternative".into())],
                        },
                        CapabilityDependency {
                            capability: CapabilityId("enabler".into()),
                            relation: DependencyKind::Enabling,
                            substitutes: vec![],
                        },
                    ],
                    evidence: vec![],
                    provenance: vec![],
                    locations: vec![],
                    qualification: None,
                    contribution: HumanAiContribution {
                        human: String::new(),
                        ai: String::new(),
                    },
                },
                cap("b", &[]),
                cap("enabler", &[]),
                cap("alternative", &[]),
            ],
        };

        let snapshot = graph
            .dependency_snapshot(&CapabilityId("a".into()))
            .unwrap();

        assert_eq!(
            snapshot.nodes,
            vec![CapabilityId("a".into()), CapabilityId("b".into())]
        );
        assert_eq!(snapshot.edges.len(), 1);
        assert_eq!(snapshot.edges[0].to, CapabilityId("b".into()));
    }

    #[test]
    fn legacy_capability_evidence_stays_unresolved_in_snapshots() {
        let mut capability = cap("a", &[]);
        capability.evidence = vec![CapabilityEvidence {
            kind: EvidenceKind::Observed,
            reference: "branch/main".into(),
            claim_ceiling: "Legacy reference only.".into(),
        }];

        let snapshot =
            EvidenceSnapshotV1::from_capability(&capability, RecoveryEvidenceCoverage::ClosedWorld);

        assert_eq!(snapshot.evidence.len(), 1);
        assert!(!snapshot.evidence[0].reference.is_well_formed());
        assert_eq!(
            snapshot.evidence[0].unresolved_locator.as_deref(),
            Some("branch/main")
        );
        assert!(!snapshot.all_references_verified(&[]));
    }

    #[test]
    fn typed_evidence_snapshot_digest_ignores_display_label() {
        let first = EvidenceReferenceV1::content_addressed(
            "test-evidence-record",
            DigestContextV1 {
                id: "sol-atlas:test-sha256:v1".into(),
                preimage_construction: "canonical test evidence bytes".into(),
                canonicalization: "sol-atlas-test-canonical-v1".into(),
                hash_algorithm: "SHA-256".into(),
                domain_separator: "sol-atlas:test-evidence:v1".into(),
                preimage_encoding: "UTF-8".into(),
                representation: DigestRepresentationV1::PrefixedLowerHex,
            },
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "Exact typed evidence only.",
        )
        .unwrap()
        .with_display_label("first label");
        let second = first.clone().with_display_label("second label");

        let first_snapshot = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                reference: first,
                claim_ceiling: "Exact typed evidence only.".into(),
                unresolved_locator: None,
            }],
        );
        let second_snapshot = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                reference: second,
                claim_ceiling: "Exact typed evidence only.".into(),
                unresolved_locator: None,
            }],
        );

        assert_eq!(first_snapshot.digest(), second_snapshot.digest());
    }

    #[test]
    fn evidence_snapshot_requires_verified_matching_references() {
        let verification = verified_test_evidence("typed-evidence");
        let snapshot = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                reference: verification.reference.clone(),
                claim_ceiling: verification.reference.claim_ceiling.clone(),
                unresolved_locator: None,
            }],
        );

        assert!(snapshot.all_references_verified(&[verification.clone()]));

        let mut unresolved = verification;
        unresolved.resolution = EvidenceReferenceResolutionV1::Unresolved;
        assert!(!snapshot.all_references_verified(&[unresolved]));
    }

    #[test]
    fn evidence_snapshot_is_derived_from_structured_capability_evidence() {
        let mut capability = cap("a", &[]);
        capability.evidence = vec![
            CapabilityEvidence {
                kind: EvidenceKind::Observed,
                reference: "evidence-b".into(),
                claim_ceiling: "b".into(),
            },
            CapabilityEvidence {
                kind: EvidenceKind::Scenario,
                reference: "fixture".into(),
                claim_ceiling: "scenario only".into(),
            },
            CapabilityEvidence {
                kind: EvidenceKind::Observed,
                reference: "evidence-b".into(),
                claim_ceiling: "b".into(),
            },
        ];

        let snapshot =
            EvidenceSnapshotV1::from_capability(&capability, RecoveryEvidenceCoverage::ClosedWorld);

        assert_eq!(snapshot.subject, CapabilityId("a".into()));
        assert_eq!(snapshot.evidence.len(), 2);
        assert_eq!(snapshot.evidence[0].kind, EvidenceKind::Observed);
        assert!(snapshot.digest().starts_with("sha256:"));
    }

    #[test]
    fn typed_evidence_snapshot_digest_changes_for_identity_mutations() {
        let reference = verified_test_evidence("mutation-test").reference;
        let base = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                claim_ceiling: reference.claim_ceiling.clone(),
                reference,
                unresolved_locator: None,
            }],
        );
        let base_digest = base.digest();

        let mut artifact_type = base.clone();
        artifact_type.evidence[0].reference.artifact_type = "other-artifact".into();
        assert_ne!(base_digest, artifact_type.digest());

        let mut digest = base.clone();
        digest.evidence[0].reference.digest =
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into();
        assert_ne!(base_digest, digest.digest());

        let mut context = base;
        context.evidence[0].reference.context.domain_separator = "other-domain:v1".into();
        assert_ne!(base_digest, context.digest());
    }

    #[test]
    fn evidence_snapshot_digest_changes_on_semantic_evidence_mutation() {
        let mut capability = cap("a", &[]);
        capability.evidence = vec![CapabilityEvidence {
            kind: EvidenceKind::Observed,
            reference: "evidence-a".into(),
            claim_ceiling: "exact observation".into(),
        }];

        let first =
            EvidenceSnapshotV1::from_capability(&capability, RecoveryEvidenceCoverage::ClosedWorld)
                .digest();

        capability.evidence[0].claim_ceiling = "broader claim".into();

        let second =
            EvidenceSnapshotV1::from_capability(&capability, RecoveryEvidenceCoverage::ClosedWorld)
                .digest();

        assert_ne!(first, second);
    }

    #[test]
    fn environment_snapshot_is_order_invariant_and_scope_sensitive() {
        let facts = vec![
            EnvironmentFactV1 {
                key: "temperature".into(),
                value: "20".into(),
                unit: Some("C".into()),
                source: "sensor-a".into(),
            },
            EnvironmentFactV1 {
                key: "pressure".into(),
                value: "101".into(),
                unit: Some("kPa".into()),
                source: "sensor-b".into(),
            },
        ];

        let first =
            EnvironmentSnapshotV1::from_facts(CapabilityId("a".into()), "site-1", facts.clone())
                .digest();

        let mut reversed = facts;
        reversed.reverse();
        let second =
            EnvironmentSnapshotV1::from_facts(CapabilityId("a".into()), "site-1", reversed)
                .digest();
        assert_eq!(first, second);

        let scoped = EnvironmentSnapshotV1::from_facts(
            CapabilityId("a".into()),
            "site-2",
            vec![
                EnvironmentFactV1 {
                    key: "temperature".into(),
                    value: "20".into(),
                    unit: Some("C".into()),
                    source: "sensor-a".into(),
                },
                EnvironmentFactV1 {
                    key: "pressure".into(),
                    value: "101".into(),
                    unit: Some("kPa".into()),
                    source: "sensor-b".into(),
                },
            ],
        )
        .digest();
        assert_ne!(first, scoped);
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
        assert_eq!(
            impact.affected,
            vec![CapabilityId("a".into()), CapabilityId("b".into())]
        );
        assert!(impact.unresolved.is_empty());

        let independent = graph.affected_by(&CapabilityId("independent".into()));
        assert!(independent.affected.is_empty());
        assert!(independent.direct_affected.is_empty());
        assert!(independent.transitive_affected.is_empty());
    }

    #[test]
    fn recovery_plan_does_not_imply_execution_or_success() {
        let plan = RecoveryPlan {
            id: "recovery-plan-001".into(),
            unavailable: CapabilityId("unavailable".into()),
            candidate: CapabilityId("recovery".into()),
            prerequisites: vec![CapabilityId("workshop".into())],
            steps: vec!["Prepare recovery path".into()],
            preconditions: vec!["Workshop operational".into()],
            expected_evidence: vec!["Operational run record".into()],
            human_contribution: "Operate and judge".into(),
            ai_contribution: "Analyze and plan".into(),
            state: RecoveryPlanState::Ready,
            claim_ceiling: "Plan only; execution and successful restoration are not established."
                .into(),
            candidate_snapshot: String::new(),
        };

        assert!(plan.is_ready());
        assert_eq!(plan.state, RecoveryPlanState::Ready);
        assert!(plan.claim_ceiling.contains("not established"));
    }

    #[test]
    fn recovery_candidates_are_discovered_without_selection() {
        let mut root = cap("root", &["unavailable"]);
        root.dependencies[0].substitutes = vec![CapabilityId("recovery".into())];

        let graph = CapabilityGraph {
            capabilities: vec![
                root,
                cap("unavailable", &[]),
                cap("recovery", &["recovery-prerequisite"]),
                cap("recovery-prerequisite", &[]),
            ],
        };

        let candidates = graph.recovery_candidates(&CapabilityId("unavailable".into()));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].candidate, CapabilityId("recovery".into()));
        assert_eq!(
            candidates[0].required_capabilities,
            vec![
                CapabilityId("recovery".into()),
                CapabilityId("recovery-prerequisite".into())
            ]
        );
        assert!(candidates[0].missing_capabilities.is_empty());
        assert_eq!(candidates[0].selection, RecoverySelectionState::Discovered);
        assert!(!candidates[0].claim_ceiling.is_empty());
    }

    #[test]
    fn recovery_candidate_missing_root_is_not_reported_as_present() {
        let mut root = cap("root", &["unavailable"]);
        root.dependencies[0].substitutes = vec![CapabilityId("missing-recovery".into())];

        let graph = CapabilityGraph {
            capabilities: vec![root, cap("unavailable", &[])],
        };

        let candidates = graph.recovery_candidates(&CapabilityId("unavailable".into()));
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].required_capabilities.is_empty());
        assert_eq!(
            candidates[0].missing_capabilities,
            vec![CapabilityId("missing-recovery".into())]
        );
        assert!(!candidates[0].is_resolvable());
    }

    #[test]
    fn recovery_candidate_missing_prerequisite_remains_unresolved() {
        let mut root = cap("root", &["unavailable"]);
        root.dependencies[0].substitutes = vec![CapabilityId("recovery".into())];

        let graph = CapabilityGraph {
            capabilities: vec![root, cap("unavailable", &[]), cap("recovery", &["missing"])],
        };

        let candidates = graph.recovery_candidates(&CapabilityId("unavailable".into()));
        assert_eq!(
            candidates[0].missing_capabilities,
            vec![CapabilityId("missing".into())]
        );
        assert!(!candidates[0].is_resolvable());
        assert_eq!(candidates[0].selection, RecoverySelectionState::Discovered);
    }

    #[test]
    fn unresolved_impact_is_scoped_to_relevant_gaps() {
        let graph = CapabilityGraph {
            capabilities: vec![
                cap("a", &["b", "unrelated-missing"]),
                cap("b", &["c"]),
                cap("c", &[]),
            ],
        };

        let impact = graph.affected_by(&CapabilityId("c".into()));
        assert_eq!(impact.direct_affected, vec![CapabilityId("b".into())]);
        assert_eq!(impact.transitive_affected, vec![CapabilityId("a".into())]);
        assert!(impact.unresolved.is_empty());
    }

    #[test]
    fn alternatives_are_explicit_candidates_not_selections() {
        let mut root = cap("a", &["b"]);
        root.dependencies[0].substitutes = vec![
            CapabilityId("alternative-2".into()),
            CapabilityId("alternative-1".into()),
            CapabilityId("alternative-1".into()),
        ];

        let paths = root.alternative_paths();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].for_dependency, CapabilityId("b".into()));
        assert_eq!(paths[0].candidate, CapabilityId("alternative-1".into()));
        assert_eq!(paths[1].candidate, CapabilityId("alternative-2".into()));
        assert!(paths[0].evidence.is_empty());
        assert!(paths[0].claim_ceiling.contains("not established"));
    }

    #[test]
    fn recovery_candidate_discovery_deduplicates_identical_declarations() {
        let mut root = cap("root", &["unavailable"]);
        root.dependencies[0].substitutes = vec![
            CapabilityId("recovery".into()),
            CapabilityId("recovery".into()),
        ];

        let graph = CapabilityGraph {
            capabilities: vec![root, cap("unavailable", &[]), cap("recovery", &[])],
        };

        let candidates = graph.recovery_candidates(&CapabilityId("unavailable".into()));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].candidate, CapabilityId("recovery".into()));
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
    fn resilience_assessment_is_root_scoped_and_separates_unresolved_alternatives() {
        let mut root = cap("root", &["unavailable"]);
        root.dependencies[0].substitutes = vec![
            CapabilityId("recovery-good".into()),
            CapabilityId("recovery-missing".into()),
        ];

        let mut unrelated = cap("unrelated", &["unavailable"]);
        unrelated.dependencies[0].substitutes = vec![CapabilityId("unrelated-recovery".into())];

        let graph = CapabilityGraph {
            capabilities: vec![
                root,
                unrelated,
                cap("unavailable", &[]),
                cap("recovery-good", &["good-prerequisite"]),
                cap("good-prerequisite", &[]),
                cap("recovery-missing", &["missing-prerequisite"]),
            ],
        };

        let assessment = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );

        assert!(assessment.is_well_formed());
        assert!(assessment.scope_snapshot.is_well_formed());
        assert!(assessment.scope_snapshot.missing_nodes.is_empty());
        assert_eq!(assessment.affected, vec![CapabilityId("root".into())]);
        assert!(assessment.unresolved.is_empty());
        assert_eq!(
            assessment
                .alternatives
                .iter()
                .map(|candidate| candidate.candidate.clone())
                .collect::<Vec<_>>(),
            vec![CapabilityId("recovery-good".into())]
        );
        assert_eq!(
            assessment
                .unresolved_alternatives
                .iter()
                .map(|candidate| candidate.candidate.clone())
                .collect::<Vec<_>>(),
            vec![CapabilityId("recovery-missing".into())]
        );
        assert!(assessment
            .alternatives
            .iter()
            .all(|candidate| candidate.missing_capabilities.is_empty()));
        assert!(assessment
            .unresolved_alternatives
            .iter()
            .all(|candidate| !candidate.missing_capabilities.is_empty()));
        assert_eq!(
            assessment
                .alternatives
                .first()
                .unwrap()
                .claim_ceiling,
            "Declared recovery candidate only; equivalence and operational interchangeability are not established."
        );
    }

    #[test]
    fn resilience_scope_rejects_duplicate_capability_ids() {
        let graph = CapabilityGraph {
            capabilities: vec![cap("root", &[]), cap("root", &[])],
        };
        let assessment = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );

        assert_eq!(
            assessment.scope_snapshot.duplicate_nodes,
            vec![CapabilityId("root".into())]
        );
        assert!(!assessment.scope_snapshot.is_well_formed());
        assert!(!assessment.is_well_formed());
        assert!(!assessment.is_exactly_bound_to_graph(&graph));
    }

    #[test]
    fn resilience_assessment_exposes_completion_gate() {
        let root = cap("root", &["unavailable", "missing-support"]);
        let graph = CapabilityGraph {
            capabilities: vec![root, cap("unavailable", &[])],
        };
        let assessment = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );

        assert!(assessment.is_well_formed());
        assert!(!assessment.has_complete_scope());
        assert!(!assessment.is_structurally_complete());
    }

    #[test]
    fn resilience_scope_preserves_missing_nodes_without_empty_digest() {
        let root = cap("root", &["unavailable", "missing-support"]);
        let graph = CapabilityGraph {
            capabilities: vec![root, cap("unavailable", &[])],
        };

        let assessment = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );

        assert!(assessment.is_well_formed());
        assert_eq!(
            assessment.scope_snapshot.missing_nodes,
            vec![CapabilityId("missing-support".into())]
        );
        assert!(assessment.scope_snapshot.digest().starts_with("sha256:"));
        assert!(assessment.is_exactly_bound_to_graph(&graph));
    }

    #[test]
    fn resilience_assessment_rejects_scope_root_or_membership_drift() {
        let graph = CapabilityGraph {
            capabilities: vec![cap("root", &["unavailable"]), cap("unavailable", &[])],
        };
        let mut assessment = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );

        assessment.scope_snapshot.root = CapabilityId("other".into());
        assert!(!assessment.is_well_formed());

        let mut assessment = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );
        assessment.affected = vec![CapabilityId("not-in-scope".into())];
        assert!(!assessment.is_well_formed());
    }

    #[test]
    fn resilience_assessment_rejects_duplicate_candidate_identity() {
        let mut root = cap("root", &["unavailable"]);
        root.dependencies[0].substitutes = vec![CapabilityId("recovery".into())];
        let graph = CapabilityGraph {
            capabilities: vec![root, cap("unavailable", &[]), cap("recovery", &[])],
        };
        let base = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );
        let candidate = base.alternatives[0].clone();

        let mut duplicate = base.clone();
        let mut altered = candidate.clone();
        altered.claim_ceiling = "different claim ceiling".into();
        duplicate.alternatives.push(altered);
        assert!(!duplicate.is_well_formed());

        let mut cross_partition = base;
        let mut unresolved = candidate;
        unresolved.missing_capabilities = vec![CapabilityId("missing".into())];
        cross_partition.unresolved_alternatives.push(unresolved);
        assert!(!cross_partition.is_well_formed());
    }

    #[test]
    fn resilience_assessment_does_not_import_unrelated_alternatives() {
        let mut root = cap("root", &["unavailable"]);
        root.dependencies[0].substitutes = vec![CapabilityId("recovery".into())];
        let mut unrelated = cap("unrelated", &["unavailable"]);
        unrelated.dependencies[0].substitutes = vec![CapabilityId("unrelated-recovery".into())];

        let graph = CapabilityGraph {
            capabilities: vec![
                root,
                unrelated,
                cap("unavailable", &[]),
                cap("recovery", &[]),
                cap("unrelated-recovery", &[]),
            ],
        };

        let assessment = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );

        assert_eq!(assessment.alternatives.len(), 1);
        assert_eq!(assessment.alternatives[0].candidate, CapabilityId("recovery".into()));
    }

    #[test]
    fn resilience_assessment_digest_is_order_invariant() {
        let mut root = cap("root", &["unavailable"]);
        root.dependencies[0].substitutes = vec![
            CapabilityId("recovery-b".into()),
            CapabilityId("recovery-a".into()),
        ];

        let graph = CapabilityGraph {
            capabilities: vec![root, cap("unavailable", &[]), cap("recovery-a", &[]), cap("recovery-b", &[])],
        };
        let first = graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );

        let mut reversed_root = cap("root", &["unavailable"]);
        reversed_root.dependencies[0].substitutes = vec![
            CapabilityId("recovery-a".into()),
            CapabilityId("recovery-b".into()),
        ];
        let reversed_graph = CapabilityGraph {
            capabilities: vec![reversed_root, cap("recovery-b", &[]), cap("unavailable", &[]), cap("recovery-a", &[])],
        };
        let second = reversed_graph.resilience_assessment(
            &CapabilityId("root".into()),
            &CapabilityId("unavailable".into()),
        );

        assert_eq!(first.digest(), second.digest());
        assert!(first.is_exactly_bound_to_graph(&graph));

        let mut drifted_root = cap("root", &["unavailable"]);
        drifted_root.dependencies[0].substitutes = vec![CapabilityId("recovery-c".into())];
        let drifted_graph = CapabilityGraph {
            capabilities: vec![drifted_root, cap("unavailable", &[]), cap("recovery-a", &[]), cap("recovery-b", &[]), cap("recovery-c", &[])],
        };
        assert!(!first.is_exactly_bound_to_graph(&drifted_graph));
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

        let error = graph
            .required_closure(&CapabilityId("a".into()))
            .unwrap_err();
        assert_eq!(error.missing, vec![CapabilityId("missing".into())]);
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

    #[test]
    fn open_world_or_unknown_evidence_cannot_pass() {
        let mut verification = RecoveryVerification {
            execution_id: "coverage-test".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-coverage".into(),
            expected_postconditions: vec!["potable water available".into()],
            observed_postconditions: vec!["potable water available".into()],
            evidence: vec![verified_test_evidence("water-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("water.purification".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: "deps-coverage".into(),
            environment_snapshot: "env-coverage".into(),
            evidence_snapshot: "evidence-coverage".into(),
            evidence_coverage: RecoveryEvidenceCoverage::OpenWorld,
            valid_until: "2026-10-03T00:00:00Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T12:00:00Z".into(),
            claim_ceiling: "Coverage test.".into(),
            execution_result_snapshot: String::new(),
        };
        verification.verification_snapshot = verification.derived_snapshot().digest();
        assert!(!verification.passes());
        verification.evidence_coverage = RecoveryEvidenceCoverage::Unknown;
        verification.verification_snapshot = verification.derived_snapshot().digest();
        assert!(!verification.passes());
        verification.evidence_coverage = RecoveryEvidenceCoverage::ClosedWorld;
        verification.verification_snapshot = verification.derived_snapshot().digest();
        assert!(verification.passes());
    }
    #[test]
    fn recovery_verification_snapshot_is_order_independent() {
        let mut a = RecoveryVerification {
            execution_id: "execution-snapshot".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-001".into(),
            expected_postconditions: vec!["b".into(), "a".into(), "a".into()],
            observed_postconditions: vec!["observed-b".into(), "observed-a".into()],
            evidence: vec![verified_test_evidence("evidence-1")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("dep-b".into()), CapabilityId("dep-a".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: "deps-1".into(),
            environment_snapshot: "env-1".into(),
            evidence_snapshot: "evidence-1".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "2026-10-03T00:00:00Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "runner".into(),
            verified_at: "2026-10-02T00:00:00Z".into(),
            claim_ceiling: "Exact inputs only.".into(),
            execution_result_snapshot: String::new(),
        };
        let digest = a.derived_snapshot().digest();
        a.verification_snapshot = digest.clone();
        let mut b = a.clone();
        b.expected_postconditions.reverse();
        b.dependency_closure.reverse();
        b.observed_postconditions.reverse();
        assert_eq!(
            a.derived_snapshot().canonical_bytes(),
            b.derived_snapshot().canonical_bytes()
        );
        assert_eq!(digest, b.derived_snapshot().digest());
        assert!(a.snapshot_matches_inputs());
        assert!(b.snapshot_matches_inputs());
    }

    #[test]
    fn recovery_verification_snapshot_changes_when_semantic_input_changes() {
        let mut verification = RecoveryVerification {
            execution_id: "execution-snapshot-mutation".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-001".into(),
            expected_postconditions: vec!["potable water available".into()],
            observed_postconditions: vec!["potable water available".into()],
            evidence: vec![verified_test_evidence("water-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("water.purification".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: "deps-1".into(),
            environment_snapshot: "env-1".into(),
            evidence_snapshot: "evidence-1".into(),
            valid_until: "2026-10-03T00:00:00Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "runner".into(),
            verified_at: "2026-10-02T00:00:00Z".into(),
            claim_ceiling: "Exact inputs only.".into(),
            execution_result_snapshot: String::new(),
        };
        let digest = verification.derived_snapshot().digest();
        verification.verification_snapshot = digest.clone();
        assert!(verification.snapshot_matches_inputs());
        verification
            .observed_postconditions
            .push("new observation".into());
        assert!(!verification.snapshot_matches_inputs());
        assert_ne!(digest, verification.derived_snapshot().digest());
    }
    #[test]
    fn recovery_verification_requires_verified_evidence_references() {
        let mut verification = RecoveryVerification {
            execution_id: "execution-evidence-binding".into(),
            capability: CapabilityId("a".into()),
            scope: "site-1".into(),
            expected_postconditions: vec!["operational".into()],
            observed_postconditions: vec!["operational".into()],
            evidence: vec![verified_test_evidence("binding-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("a".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: "deps-binding".into(),
            environment_snapshot: "env-binding".into(),
            evidence_snapshot: "evidence-binding".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "9999-12-31T23:59:59Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T00:00:00Z".into(),
            claim_ceiling: "Exact verification scope only.".into(),
            execution_result_snapshot: String::new(),
        };
        verification.verification_snapshot = verification.derived_snapshot().digest();
        assert!(verification.passes());

        verification.evidence[0].resolution = EvidenceReferenceResolutionV1::Unresolved;
        verification.verification_snapshot = verification.derived_snapshot().digest();
        assert!(!verification.passes());
    }

    #[test]
    fn recovery_verification_requires_complete_postconditions_and_dependencies() {
        let verification = RecoveryVerification {
            execution_id: "execution-verified".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-001".into(),
            expected_postconditions: vec![
                "potable water available".into(),
                "pump responding".into(),
            ],
            observed_postconditions: vec![
                "potable water available".into(),
                "pump responding".into(),
            ],
            evidence: vec![verified_test_evidence("water-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("water.purification".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: "deps-001".into(),
            environment_snapshot: "env-001".into(),
            evidence_snapshot: "evidence-001".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "2026-10-02T12:00:00Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T08:10:00Z".into(),
            claim_ceiling:
                "Exact execution and instance scope only; qualification is not established.".into(),
            execution_result_snapshot: String::new(),
        };
        let mut verification = verification;
        verification.verification_snapshot = verification.derived_snapshot().digest();

        assert!(verification.passes());
    }

    #[test]
    fn contradictory_or_unresolved_verification_cannot_pass() {
        let verification = RecoveryVerification {
            execution_id: "execution-ambiguous".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-002".into(),
            expected_postconditions: vec!["potable water available".into()],
            observed_postconditions: vec!["potable water available".into()],
            evidence: vec![verified_test_evidence("water-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec!["contamination detected".into()],
            dependency_closure: vec![],
            unresolved_dependencies: vec![CapabilityId("pump-maintenance".into())],
            verification_snapshot: "verification-inputs-002".into(),
            dependency_snapshot: "deps-002".into(),
            environment_snapshot: "env-002".into(),
            evidence_snapshot: "evidence-002".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "2026-10-02T12:00:00Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T08:10:00Z".into(),
            claim_ceiling: "Ambiguous result.".into(),
            execution_result_snapshot: String::new(),
        };

        assert!(!verification.passes());
    }

    #[test]
    fn recovery_verification_rejects_scope_and_snapshot_drift() {
        let verification = RecoveryVerification {
            execution_id: "execution-fresh".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-003".into(),
            expected_postconditions: vec!["potable water available".into()],
            observed_postconditions: vec!["potable water available".into()],
            evidence: vec![verified_test_evidence("water-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("water.purification".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: "verification-inputs-003".into(),
            dependency_snapshot: "deps-003".into(),
            environment_snapshot: "env-003".into(),
            evidence_snapshot: "evidence-003".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "2026-10-02T12:00:00Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T08:10:00Z".into(),
            claim_ceiling:
                "Exact execution and instance scope only; qualification is not established.".into(),
            execution_result_snapshot: String::new(),
        };

        assert_eq!(
            verification.validity_against(
                &CapabilityId("water.purification".into()),
                "instance-003",
                "verification-inputs-003",
                "deps-003",
                "env-003",
                "evidence-003",
                "2026-10-02T10:00:00Z",
            ),
            RecoveryVerificationValidity::Current
        );
        assert_eq!(
            verification.validity_against(
                &CapabilityId("water.purification".into()),
                "instance-003",
                "verification-inputs-003",
                "deps-004",
                "env-003",
                "evidence-003",
                "2026-10-02T10:00:00Z",
            ),
            RecoveryVerificationValidity::DependencyDrift
        );
        assert_eq!(
            verification.validity_against(
                &CapabilityId("water.purification".into()),
                "instance-004",
                "verification-inputs-003",
                "deps-003",
                "env-003",
                "evidence-003",
                "2026-10-02T10:00:00Z",
            ),
            RecoveryVerificationValidity::ScopeMismatch
        );
    }

    #[test]
    fn registry_bound_verification_requires_exact_registry_authorization() {
        let dependency = DependencySnapshotV1 {
            schema: DependencySnapshotV1::SCHEMA.into(),
            root: CapabilityId("a".into()),
            nodes: vec![CapabilityId("a".into())],
            edges: vec![],
        };
        let environment = EnvironmentSnapshotV1::from_facts(
            CapabilityId("a".into()),
            "site-1",
            vec![EnvironmentFactV1 {
                key: "temperature".into(),
                value: "20".into(),
                unit: Some("C".into()),
                source: "sensor-a".into(),
            }],
        );
        let preimage = b"registry-gated-evidence";
        let digest = Sha256::digest(preimage);
        let reference = EvidenceReferenceV1::content_addressed(
            "test-evidence-record",
            DigestContextV1 {
                id: "sol-atlas:registry-test:v1".into(),
                preimage_construction: "exact supplied test bytes".into(),
                canonicalization: "sol-atlas-registry-test-v1".into(),
                hash_algorithm: "SHA-256".into(),
                domain_separator: "sol-atlas:registry-test:v1".into(),
                preimage_encoding: "UTF-8".into(),
                representation: DigestRepresentationV1::PrefixedLowerHex,
            },
            format!("sha256:{digest:x}"),
            "Registry-authorized test evidence only.",
        )
        .unwrap()
        .with_purpose("recovery-verification")
        .unwrap();
        let profile = crate::evidence_reference::EvidenceReferenceProfileV1 {
            id: "registry-profile".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: reference.claim_ceiling.clone(),
        };
        let registry = EvidenceReferenceProfileRegistryV1 {
            id: "registry-001".into(),
            version: 1,
            profiles: vec![profile],
        };
        let processed = reference
            .verify_preimage_against_registry(
                &registry,
                preimage,
                "registry-verifier",
                "2026-10-02T08:00:00Z",
                "Registry-authorized test evidence only.",
            )
            .unwrap();
        let evidence = EvidenceSnapshotV1::from_entries(
            CapabilityId("a".into()),
            RecoveryEvidenceCoverage::ClosedWorld,
            vec![EvidenceSnapshotEntryV1 {
                kind: EvidenceKind::Observed,
                reference: processed.reference.clone(),
                claim_ceiling: processed.reference.claim_ceiling.clone(),
                unresolved_locator: None,
            }],
        );
        let mut verification = RecoveryVerification {
            execution_id: "execution-registry-gated".into(),
            capability: CapabilityId("a".into()),
            scope: "site-1".into(),
            expected_postconditions: vec!["operational".into()],
            observed_postconditions: vec!["operational".into()],
            evidence: vec![processed],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("a".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: String::new(),
            dependency_snapshot: dependency.digest(),
            environment_snapshot: environment.digest(),
            evidence_snapshot: evidence.digest(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "9999-12-31T23:59:59Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "registry-verifier".into(),
            verified_at: "2026-10-02T08:00:00Z".into(),
            claim_ceiling: "Exact registry-gated scope only.".into(),
            execution_result_snapshot: String::new(),
        };
        verification.verification_snapshot =
            RecoveryVerificationSnapshotV1::from_verification_with_snapshots(
                &verification,
                &dependency,
                &evidence,
                &environment,
            )
            .unwrap()
            .digest();

        assert!(verification.passes_with_bound_snapshots_and_registry(
            &dependency,
            &evidence,
            &environment,
            &registry
        ));

        let mut changed = registry.clone();
        changed.version = 2;
        assert!(!verification.passes_with_bound_snapshots_and_registry(
            &dependency,
            &evidence,
            &environment,
            &changed
        ));
    }

    #[test]
    fn canonical_utc_timestamp_rejects_ambiguous_or_invalid_values() {
        assert!(is_canonical_utc_timestamp("2026-10-02T12:00:00Z"));
        assert!(is_canonical_utc_timestamp("2000-02-29T00:00:00Z"));
        assert!(!is_canonical_utc_timestamp("2026-10-02T12:00:00+00:00"));
        assert!(!is_canonical_utc_timestamp("2026-10-02T12:00:00.000Z"));
        assert!(!is_canonical_utc_timestamp("2026-02-29T12:00:00Z"));
        assert!(!is_canonical_utc_timestamp("2026-13-01T12:00:00Z"));
        assert!(!is_canonical_utc_timestamp("2026-10-02T24:00:00Z"));
        assert!(!is_canonical_utc_timestamp("not-a-timestamp"));
    }

    #[test]
    fn recovery_verification_expires_at_declared_valid_until() {
        let verification = RecoveryVerification {
            execution_id: "execution-boundary".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-boundary".into(),
            expected_postconditions: vec!["potable water available".into()],
            observed_postconditions: vec!["potable water available".into()],
            evidence: vec![verified_test_evidence("water-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("water.purification".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: "verification-inputs-boundary".into(),
            dependency_snapshot: "deps-boundary".into(),
            environment_snapshot: "env-boundary".into(),
            evidence_snapshot: "evidence-boundary".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "2026-10-02T12:00:00Z".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T08:10:00Z".into(),
            claim_ceiling: "Exact verification scope only.".into(),
            execution_result_snapshot: String::new(),
        };

        assert_eq!(
            verification.validity_against(
                &CapabilityId("water.purification".into()),
                "instance-boundary",
                "verification-inputs-boundary",
                "deps-boundary",
                "env-boundary",
                "evidence-boundary",
                "2026-10-02T11:59:59Z",
            ),
            RecoveryVerificationValidity::Current
        );
        assert_eq!(
            verification.validity_against(
                &CapabilityId("water.purification".into()),
                "instance-boundary",
                "verification-inputs-boundary",
                "deps-boundary",
                "env-boundary",
                "evidence-boundary",
                "2026-10-02T12:00:00Z",
            ),
            RecoveryVerificationValidity::Stale
        );
    }

    #[test]
    fn recovery_verification_rejects_malformed_freshness_timestamps() {
        let mut verification = RecoveryVerification {
            execution_id: "execution-malformed-time".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-time".into(),
            expected_postconditions: vec!["potable water available".into()],
            observed_postconditions: vec!["potable water available".into()],
            evidence: vec![verified_test_evidence("water-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("water.purification".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: "verification-inputs-time".into(),
            dependency_snapshot: "deps-time".into(),
            environment_snapshot: "env-time".into(),
            evidence_snapshot: "evidence-time".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "tomorrow".into(),
            superseded_by: None,
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T08:10:00Z".into(),
            claim_ceiling: "Exact verification scope only.".into(),
            execution_result_snapshot: String::new(),
        };

        assert!(!verification.passes());
        verification.valid_until = "2026-10-02T12:00:00Z".into();
        verification.verification_snapshot = verification.derived_snapshot().digest();
        assert!(verification.passes());
    }

    #[test]
    fn recovery_verification_expires_and_can_be_superseded() {
        let verification = RecoveryVerification {
            execution_id: "execution-expiring".into(),
            capability: CapabilityId("water.purification".into()),
            scope: "instance-004".into(),
            expected_postconditions: vec!["potable water available".into()],
            observed_postconditions: vec!["potable water available".into()],
            evidence: vec![verified_test_evidence("water-test")],
            missing_postconditions: vec![],
            contradictory_postconditions: vec![],
            dependency_closure: vec![CapabilityId("water.purification".into())],
            unresolved_dependencies: vec![],
            verification_snapshot: "verification-inputs-004".into(),
            dependency_snapshot: "deps-004".into(),
            environment_snapshot: "env-004".into(),
            evidence_snapshot: "evidence-004".into(),
            evidence_coverage: RecoveryEvidenceCoverage::ClosedWorld,
            valid_until: "2026-10-02T12:00:00Z".into(),
            superseded_by: Some("execution-newer".into()),
            state: RecoveryVerificationState::Passed,
            verifier: "verification-runner".into(),
            verified_at: "2026-10-02T08:10:00Z".into(),
            claim_ceiling:
                "Exact execution and instance scope only; qualification is not established.".into(),
            execution_result_snapshot: String::new(),
        };

        assert_eq!(
            verification.validity_against(
                &CapabilityId("water.purification".into()),
                "instance-004",
                "verification-inputs-004",
                "deps-004",
                "env-004",
                "evidence-004",
                "2026-10-02T13:00:00Z",
            ),
            RecoveryVerificationValidity::Superseded
        );
    }
}
