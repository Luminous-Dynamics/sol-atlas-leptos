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
    DigestContextV1, DigestRepresentationV1, EvidenceReferenceCanonicalV1,
    EvidenceReferenceProfileRegistryV1, EvidenceReferenceResolutionV1, EvidenceReferenceV1,
    EvidenceReferenceVerificationV1,
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
        self.evidence.len() == verifications.len()
            && self.evidence.iter().all(|entry| {
                entry.reference.is_well_formed()
                    && entry.unresolved_locator.is_none()
                    && verifications.iter().any(|verification| {
                        verification.is_verified()
                            && verification
                                .reference
                                .same_content_identity(&entry.reference)
                            && verification.reference.purpose == entry.reference.purpose
                            && verification.reference.claim_ceiling == entry.claim_ceiling
                    })
            })
            && verifications.iter().all(|verification| {
                verification.is_verified()
                    && self.evidence.iter().any(|entry| {
                        entry
                            .reference
                            .same_content_identity(&verification.reference)
                            && entry.reference.purpose == verification.reference.purpose
                            && entry.claim_ceiling == verification.reference.claim_ceiling
                            && entry.unresolved_locator.is_none()
                    })
            })
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub id: String,
    pub unavailable: CapabilityId,
    pub candidate: CapabilityId,
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
    pub fn is_ready(&self) -> bool {
        self.state == RecoveryPlanState::Ready
            && !self.id.is_empty()
            && !self.unavailable.0.is_empty()
            && !self.candidate.0.is_empty()
            && !self.steps.is_empty()
            && !self.expected_evidence.is_empty()
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
            dependency_snapshot: String,
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
            dependency_snapshot: self.dependency_snapshot.clone(),
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
            || evidence.subject != self.capability
            || environment.subject != self.capability
            || environment.scope != self.scope
        {
            return false;
        }

        if self.dependency_snapshot != dependency.digest()
            || self.evidence_snapshot != evidence.digest()
            || self.environment_snapshot != environment.digest()
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
            || self.dependency_snapshot.is_empty()
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

impl RecoveryCandidate {
    /// Whether the candidate's own required closure is complete.
    pub fn is_resolvable(&self) -> bool {
        self.missing_capabilities.is_empty()
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
        candidates
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

/// Evidence emitted by a concrete recovery execution.
///
/// Execution evidence is intentionally separate from the plan: a plan describes
/// intended actions, while this record describes what actually happened.
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
                && self.started_at <= ended_at
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
            && !self.evidence.is_empty()
            && !self.input_snapshot.is_empty()
            && !self.claim_ceiling.is_empty()
    }

    /// A failed execution must preserve a terminal marker and an explicit
    /// failure indication rather than silently becoming an unsuccessful success.
    pub fn is_failed(&self) -> bool {
        self.terminal_timestamps_are_well_formed()
            && !self.plan_id.is_empty()
            && !self.execution_id.is_empty()
            && (self.failed_steps.iter().any(|step| !step.is_empty())
                || self.failure_reason.as_ref().is_some_and(|reason| !reason.is_empty()))
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

        assert!(graph
            .recovery_candidates(&CapabilityId("a".into()))
            .is_empty());
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
