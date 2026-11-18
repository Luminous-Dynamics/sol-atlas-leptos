// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-preserving territorial-rights and stewardship interchange types.
//!
//! This module is deliberately a record/validation boundary, not a land registry,
//! legal rules engine, title adjudicator, or authority to grant/transfer rights.
//! It models assertions, their bases, disputes, spatial/temporal scope, source
//! evidence, and stewardship obligations without collapsing them into one score.
//!
//! Geometry is referenced by stable feature URI so callers can use OGC APIs,
//! GeoJSON, GeoPackage, or another jurisdictionally appropriate encoding.
//! Timestamps are RFC 3339 strings; CRSs and vertical datums are explicit IDs.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const TERRITORIAL_RECORD_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterestKind {
    SurfaceLand,
    SubsurfaceMineral,
    Groundwater,
    SurfaceWater,
    Access,
    Grazing,
    Conservation,
    CulturalHeritage,
    Servitude,
    Infrastructure,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    Right,
    Responsibility,
    Restriction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimBasis {
    Asserted,
    CommunityAttested,
    AdministrativeRecord,
    CourtDecision,
    ContractualInstrument,
    StatutoryInstrument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeState {
    Unassessed,
    NoChallengeObservedWithinFrontier,
    Challenged,
    UnderReview,
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeCoverage {
    CompleteWithinDeclaredFrontier,
    Partial,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureClass {
    Public,
    Restricted,
    CommunityControlled,
    CulturallySensitive,
}

impl Default for DisclosureClass {
    fn default() -> Self {
        Self::Restricted
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    RegistryExtract,
    CourtOrder,
    Permit,
    StatutoryInstrument,
    Survey,
    CommunityTestimony,
    CommunityDecision,
    ChallengeSearchReport,
    Contract,
    EnvironmentalMeasurement,
    RemoteSensingObservation,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityKind {
    Registry,
    CourtOrTribunal,
    StatutoryRegulator,
    CustomaryInstitution,
    CommunityGovernanceBody,
    ContractingParty,
    ProfessionalVerifier,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeState {
    Submitted,
    UnderReview,
    Resolved,
    Rejected,
    Withdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationKind {
    RehabilitateLand,
    RestoreHabitat,
    ProtectGroundwater,
    MonitorWaterQuality,
    StabilizeGround,
    ManageTailings,
    MaintainAccess,
    ShareBenefits,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationState {
    Open,
    InProgress,
    AwaitingEvidence,
    VerifiedComplete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReference {
    pub id: String,
    pub kind: EvidenceKind,
    /// URI or opaque reference resolved through an authorised data provider.
    pub source_ref: String,
    /// RFC 3339 capture/receipt time, not necessarily the instrument's issue date.
    pub captured_at: String,
    /// SHA-256 of the exact retained representation, when available.
    pub sha256: Option<String>,
    #[serde(default)]
    pub disclosure_class: DisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityReference {
    pub id: String,
    /// Institution or governance-body identifier; avoid embedding personal data.
    pub issuer_ref: String,
    pub kind: AuthorityKind,
    pub jurisdiction: String,
    pub scope: String,
    pub source_evidence_id: String,
    pub valid_from: String,
    pub valid_until: Option<String>,
    #[serde(default)]
    pub disclosure_class: DisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeReference {
    pub id: String,
    pub raised_by_ref: String,
    pub filed_at: String,
    pub state: ChallengeState,
    pub evidence_ids: Vec<String>,
    #[serde(default)]
    pub disclosure_class: DisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeAssessment {
    pub id: String,
    /// Exact scope in which challenge records were sought, not a global absence claim.
    pub scope_ref: String,
    /// Exact source/evidence frontier identifier used for the assessment.
    pub frontier_ref: String,
    pub assessed_at: String,
    pub coverage: ChallengeCoverage,
    pub source_refs: Vec<String>,
    pub evidence_ids: Vec<String>,
    #[serde(default)]
    pub disclosure_class: DisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialExtent {
    /// Stable URI to a feature in an OGC API Features collection or equivalent.
    pub feature_ref: String,
    /// For example, an EPSG identifier or OGC CRS URI.
    pub horizontal_crs: String,
    /// Optional vertical extent; never infer underground geometry from a surface polygon.
    pub vertical_min_m: Option<f64>,
    pub vertical_max_m: Option<f64>,
    pub vertical_datum: Option<String>,
    /// Source-reported or computed horizontal uncertainty in metres.
    pub horizontal_uncertainty_m: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalExtent {
    /// RFC 3339 instant or jurisdictional date, with the representation documented.
    pub valid_from: String,
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerritorialAssertion {
    pub id: String,
    /// Stable party/organisation/community reference, not a person's name or ID.
    pub holder_ref: String,
    pub interest_kind: InterestKind,
    /// Exact source-system code; preserves jurisdictional subtypes through round trips.
    pub source_interest_type_code: Option<String>,
    /// Versioned profile/code-system URI that defines the source code.
    pub source_interest_type_profile_ref: Option<String>,
    pub relationship: RelationshipKind,
    /// Multiple bases can coexist. None is an automatic legal priority ordering.
    pub bases: Vec<ClaimBasis>,
    /// Dispute status is orthogonal to the basis: a registered record can be challenged.
    pub dispute_state: DisputeState,
    pub spatial: SpatialExtent,
    pub temporal: TemporalExtent,
    pub evidence_ids: Vec<String>,
    /// Multiple independent authorities may support distinct bases for one assertion.
    pub authority_ids: Vec<String>,
    pub challenge_ids: Vec<String>,
    /// Required for a bounded no-challenge observation; never implies global absence.
    pub challenge_assessment_id: Option<String>,
    #[serde(default)]
    pub disclosure_class: DisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StewardshipObligation {
    pub id: String,
    /// Reference to the upstream governance/commitment record; this projection is not canonical.
    pub governance_record_ref: String,
    pub kind: ObligationKind,
    pub description: String,
    pub spatial_feature_ref: String,
    pub responsible_party_ref: String,
    pub originating_evidence_id: String,
    pub due_at: String,
    /// Optional quantitative outcome, with an explicit unit.
    pub target_value: Option<f64>,
    pub target_unit: Option<String>,
    pub verification_protocol_ref: String,
    /// Must be distinct from the responsible party for verified completion.
    pub independent_verifier_ref: Option<String>,
    pub state: ObligationState,
    pub evidence_ids: Vec<String>,
    #[serde(default)]
    pub disclosure_class: DisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerritorialRecord {
    pub schema_version: u16,
    pub territory_ref: String,
    pub created_at: String,
    pub evidence: Vec<EvidenceReference>,
    pub authorities: Vec<AuthorityReference>,
    pub challenges: Vec<ChallengeReference>,
    pub challenge_assessments: Vec<ChallengeAssessment>,
    pub assertions: Vec<TerritorialAssertion>,
    pub obligations: Vec<StewardshipObligation>,
}

impl TerritorialRecord {
    /// Validate structural integrity and provenance links.
    ///
    /// This does not authenticate signatures, adjudicate legal effect, determine
    /// representative legitimacy, validate geometry, or establish consent.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();

        if self.schema_version != TERRITORIAL_RECORD_SCHEMA_VERSION {
            issues.push(format!(
                "unsupported schema_version {}; expected {}",
                self.schema_version, TERRITORIAL_RECORD_SCHEMA_VERSION
            ));
        }
        required("territory_ref", &self.territory_ref, &mut issues);
        required("created_at", &self.created_at, &mut issues);

        let mut evidence_ids = HashSet::new();
        for evidence in &self.evidence {
            unique_id("evidence", &evidence.id, &mut evidence_ids, &mut issues);
            required("evidence.source_ref", &evidence.source_ref, &mut issues);
            required("evidence.captured_at", &evidence.captured_at, &mut issues);
            if let Some(digest) = &evidence.sha256 {
                if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    issues.push(format!("evidence {} has an invalid SHA-256 digest", evidence.id));
                }
            }
        }

        let mut authority_ids = HashSet::new();
        for authority in &self.authorities {
            unique_id("authority", &authority.id, &mut authority_ids, &mut issues);
            required("authority.issuer_ref", &authority.issuer_ref, &mut issues);
            required("authority.jurisdiction", &authority.jurisdiction, &mut issues);
            required("authority.scope", &authority.scope, &mut issues);
            required("authority.valid_from", &authority.valid_from, &mut issues);
            if !evidence_ids.contains(&authority.source_evidence_id) {
                issues.push(format!(
                    "authority {} references missing source evidence {}",
                    authority.id, authority.source_evidence_id
                ));
            }
        }

        let mut challenge_ids = HashSet::new();
        for challenge in &self.challenges {
            unique_id("challenge", &challenge.id, &mut challenge_ids, &mut issues);
            required("challenge.raised_by_ref", &challenge.raised_by_ref, &mut issues);
            required("challenge.filed_at", &challenge.filed_at, &mut issues);
            if challenge.evidence_ids.is_empty() {
                issues.push(format!("challenge {} has no evidence references", challenge.id));
            }
            for evidence_id in &challenge.evidence_ids {
                if !evidence_ids.contains(evidence_id) {
                    issues.push(format!(
                        "challenge {} references missing evidence {}",
                        challenge.id, evidence_id
                    ));
                }
            }
        }

        let mut challenge_assessment_ids = HashSet::new();
        for assessment in &self.challenge_assessments {
            unique_id(
                "challenge_assessment",
                &assessment.id,
                &mut challenge_assessment_ids,
                &mut issues,
            );
            required("challenge_assessment.scope_ref", &assessment.scope_ref, &mut issues);
            required("challenge_assessment.frontier_ref", &assessment.frontier_ref, &mut issues);
            required("challenge_assessment.assessed_at", &assessment.assessed_at, &mut issues);
            if assessment.source_refs.is_empty()
                || assessment.source_refs.iter().any(|source| source.trim().is_empty())
            {
                issues.push(format!(
                    "challenge assessment {} has no valid source references",
                    assessment.id
                ));
            }
            if assessment.evidence_ids.is_empty() {
                issues.push(format!(
                    "challenge assessment {} has no evidence references",
                    assessment.id
                ));
            }
            let has_search_report = assessment.evidence_ids.iter().any(|evidence_id| {
                evidence_by_id.get(evidence_id.as_str()).is_some_and(|evidence| {
                    evidence.kind == EvidenceKind::ChallengeSearchReport
                })
            });
            if !has_search_report {
                issues.push(format!(
                    "challenge assessment {} lacks challenge-search report evidence",
                    assessment.id
                ));
            }
            for evidence_id in &assessment.evidence_ids {
                if !evidence_ids.contains(evidence_id) {
                    issues.push(format!(
                        "challenge assessment {} references missing evidence {}",
                        assessment.id, evidence_id
                    ));
                }
            }
        }

        let challenge_assessment_by_id = self
            .challenge_assessments
            .iter()
            .map(|assessment| (assessment.id.as_str(), assessment))
            .collect::<std::collections::HashMap<_, _>>();

        let authority_by_id = self
            .authorities
            .iter()
            .map(|authority| (authority.id.as_str(), authority))
            .collect::<std::collections::HashMap<_, _>>();
        let evidence_by_id = self
            .evidence
            .iter()
            .map(|evidence| (evidence.id.as_str(), evidence))
            .collect::<std::collections::HashMap<_, _>>();

        let mut assertion_ids = HashSet::new();
        for assertion in &self.assertions {
            unique_id("assertion", &assertion.id, &mut assertion_ids, &mut issues);
            required("assertion.holder_ref", &assertion.holder_ref, &mut issues);
            match (
                assertion.source_interest_type_code.as_deref(),
                assertion.source_interest_type_profile_ref.as_deref(),
            ) {
                (Some(code), Some(profile)) => {
                    required("assertion.source_interest_type_code", code, &mut issues);
                    required(
                        "assertion.source_interest_type_profile_ref",
                        profile,
                        &mut issues,
                    );
                }
                (None, None) if assertion.interest_kind == InterestKind::Other => {
                    issues.push(format!(
                        "assertion {} uses other interest_kind without preserving its source code and profile",
                        assertion.id
                    ));
                }
                (None, None) => {}
                _ => issues.push(format!(
                    "assertion {} has an incomplete source interest-type code/profile pair",
                    assertion.id
                )),
            }
            required(
                "assertion.spatial.feature_ref",
                &assertion.spatial.feature_ref,
                &mut issues,
            );
            required(
                "assertion.spatial.horizontal_crs",
                &assertion.spatial.horizontal_crs,
                &mut issues,
            );
            required(
                "assertion.temporal.valid_from",
                &assertion.temporal.valid_from,
                &mut issues,
            );
            if !assertion.spatial.horizontal_uncertainty_m.is_finite()
                || assertion.spatial.horizontal_uncertainty_m < 0.0
            {
                issues.push(format!(
                    "assertion {} has invalid horizontal uncertainty",
                    assertion.id
                ));
            }
            match (assertion.spatial.vertical_min_m, assertion.spatial.vertical_max_m) {
                (Some(min), Some(max)) if min.is_finite() && max.is_finite() && min <= max => {
                    required(
                        "assertion.spatial.vertical_datum",
                        assertion.spatial.vertical_datum.as_deref().unwrap_or(""),
                        &mut issues,
                    );
                }
                (None, None) => {}
                _ => issues.push(format!(
                    "assertion {} has an incomplete or invalid vertical extent",
                    assertion.id
                )),
            }
            if assertion.bases.is_empty() {
                issues.push(format!("assertion {} has no declared basis", assertion.id));
            }
            if assertion.evidence_ids.is_empty() {
                issues.push(format!("assertion {} has no evidence references", assertion.id));
            }
            for evidence_id in &assertion.evidence_ids {
                if !evidence_ids.contains(evidence_id) {
                    issues.push(format!(
                        "assertion {} references missing evidence {}",
                        assertion.id, evidence_id
                    ));
                }
            }
            for authority_id in &assertion.authority_ids {
                if !authority_ids.contains(authority_id) {
                    issues.push(format!(
                        "assertion {} references missing authority {}",
                        assertion.id, authority_id
                    ));
                }
            }
            for challenge_id in &assertion.challenge_ids {
                if !challenge_ids.contains(challenge_id) {
                    issues.push(format!(
                        "assertion {} references missing challenge {}",
                        assertion.id, challenge_id
                    ));
                }
            }
            if let Some(assessment_id) = &assertion.challenge_assessment_id {
                if !challenge_assessment_ids.contains(assessment_id) {
                    issues.push(format!(
                        "assertion {} references missing challenge assessment {}",
                        assertion.id, assessment_id
                    ));
                }
            }

            let linked_challenges = self
                .challenges
                .iter()
                .filter(|challenge| {
                    assertion.challenge_ids.iter().any(|id| id == &challenge.id)
                })
                .collect::<Vec<_>>();
            let has_submitted_challenge = linked_challenges
                .iter()
                .any(|challenge| challenge.state == ChallengeState::Submitted);
            let has_under_review_challenge = linked_challenges
                .iter()
                .any(|challenge| challenge.state == ChallengeState::UnderReview);
            let all_linked_challenges_terminal = !linked_challenges.is_empty()
                && linked_challenges.iter().all(|challenge| {
                    matches!(
                        challenge.state,
                        ChallengeState::Resolved
                            | ChallengeState::Rejected
                            | ChallengeState::Withdrawn
                    )
                });

            match assertion.dispute_state {
                DisputeState::NoChallengeObservedWithinFrontier => {
                    if !linked_challenges.is_empty() {
                        issues.push(format!(
                            "assertion {} says no challenge observed while linked challenge records exist",
                            assertion.id
                        ));
                    }
                    let complete_assessment = assertion
                        .challenge_assessment_id
                        .as_deref()
                        .and_then(|id| challenge_assessment_by_id.get(id))
                        .is_some_and(|assessment| {
                            assessment.coverage == ChallengeCoverage::CompleteWithinDeclaredFrontier
                        });
                    if !complete_assessment {
                        issues.push(format!(
                            "assertion {} claims no challenge observed without a complete declared challenge frontier",
                            assertion.id
                        ));
                    }
                }
                DisputeState::Challenged if !has_submitted_challenge => {
                    issues.push(format!(
                        "assertion {} says challenged without a submitted linked challenge",
                        assertion.id
                    ));
                }
                DisputeState::UnderReview
                    if has_submitted_challenge || !has_under_review_challenge =>
                {
                    issues.push(format!(
                        "assertion {} says under_review without exclusively under-review active challenges",
                        assertion.id
                    ));
                }
                DisputeState::Resolved if !all_linked_challenges_terminal => {
                    issues.push(format!(
                        "assertion {} says resolved while one or more linked challenges remain active or no challenge is linked",
                        assertion.id
                    ));
                }
                _ => {}
            }

            for basis in &assertion.bases {
                match basis {
                    ClaimBasis::AdministrativeRecord => {
                        let has_register_authority = assertion.authority_ids.iter().any(|id| {
                            authority_by_id.get(id.as_str()).is_some_and(|authority| {
                                matches!(
                                    authority.kind,
                                    AuthorityKind::Registry | AuthorityKind::StatutoryRegulator
                                ) && evidence_by_id
                                    .get(authority.source_evidence_id.as_str())
                                    .is_some_and(|evidence| {
                                        matches!(
                                            evidence.kind,
                                            EvidenceKind::RegistryExtract | EvidenceKind::Permit
                                        )
                                    })
                            })
                        });
                        if !has_register_authority {
                            issues.push(format!(
                                "assertion {} declares administrative_record without matching registry/regulator authority and source evidence",
                                assertion.id
                            ));
                        }
                    }
                    ClaimBasis::CourtDecision => {
                        let has_court_authority = assertion.authority_ids.iter().any(|id| {
                            authority_by_id.get(id.as_str()).is_some_and(|authority| {
                                authority.kind == AuthorityKind::CourtOrTribunal
                                    && evidence_by_id
                                        .get(authority.source_evidence_id.as_str())
                                        .is_some_and(|evidence| {
                                            evidence.kind == EvidenceKind::CourtOrder
                                        })
                            })
                        });
                        if !has_court_authority {
                            issues.push(format!(
                                "assertion {} declares court_decision without a court/tribunal authority and court-order evidence",
                                assertion.id
                            ));
                        }
                    }
                    ClaimBasis::CommunityAttested => {
                        let has_community_decision_authority = assertion.authority_ids.iter().any(|id| {
                            authority_by_id.get(id.as_str()).is_some_and(|authority| {
                                matches!(
                                    authority.kind,
                                    AuthorityKind::CustomaryInstitution
                                        | AuthorityKind::CommunityGovernanceBody
                                ) && evidence_by_id
                                    .get(authority.source_evidence_id.as_str())
                                    .is_some_and(|evidence| {
                                        evidence.kind == EvidenceKind::CommunityDecision
                                    })
                            })
                        });
                        if !has_community_decision_authority {
                            issues.push(format!(
                                "assertion {} declares community_attested without community-governance/customary authority and linked community-decision evidence",
                                assertion.id
                            ));
                        }
                        if !assertion.evidence_ids.iter().any(|id| {
                            evidence_by_id.get(id.as_str()).is_some_and(|evidence| {
                                evidence.kind == EvidenceKind::CommunityDecision
                            })
                        }) {
                            issues.push(format!(
                                "assertion {} declares community_attested without community-decision evidence",
                                assertion.id
                            ));
                        }
                    }
                    ClaimBasis::ContractualInstrument => {
                        let has_contract_authority = assertion.authority_ids.iter().any(|id| {
                            authority_by_id.get(id.as_str()).is_some_and(|authority| {
                                authority.kind == AuthorityKind::ContractingParty
                                    && evidence_by_id
                                        .get(authority.source_evidence_id.as_str())
                                        .is_some_and(|evidence| {
                                            evidence.kind == EvidenceKind::Contract
                                        })
                            })
                        });
                        if !has_contract_authority {
                            issues.push(format!(
                                "assertion {} declares contractual_instrument without contracting-party authority and linked contract evidence",
                                assertion.id
                            ));
                        }
                        if !assertion.evidence_ids.iter().any(|id| {
                            evidence_by_id
                                .get(id.as_str())
                                .is_some_and(|evidence| evidence.kind == EvidenceKind::Contract)
                        }) {
                            issues.push(format!(
                                "assertion {} declares contractual_instrument without contract evidence",
                                assertion.id
                            ));
                        }
                    }
                    ClaimBasis::StatutoryInstrument => {
                        let has_statutory_authority = assertion.authority_ids.iter().any(|id| {
                            authority_by_id.get(id.as_str()).is_some_and(|authority| {
                                authority.kind == AuthorityKind::StatutoryRegulator
                                    && evidence_by_id
                                        .get(authority.source_evidence_id.as_str())
                                        .is_some_and(|evidence| {
                                            evidence.kind == EvidenceKind::StatutoryInstrument
                                        })
                            })
                        });
                        if !has_statutory_authority {
                            issues.push(format!(
                                "assertion {} declares statutory_instrument without statutory-regulator authority and linked statutory-instrument evidence",
                                assertion.id
                            ));
                        }
                        if !assertion.evidence_ids.iter().any(|id| {
                            evidence_by_id.get(id.as_str()).is_some_and(|evidence| {
                                evidence.kind == EvidenceKind::StatutoryInstrument
                            })
                        }) {
                            issues.push(format!(
                                "assertion {} declares statutory_instrument without statutory-instrument evidence",
                                assertion.id
                            ));
                        }
                    }
                    ClaimBasis::Asserted => {}
                }
            }
        }

        let mut obligation_ids = HashSet::new();
        for obligation in &self.obligations {
            unique_id("obligation", &obligation.id, &mut obligation_ids, &mut issues);
            required(
                "obligation.governance_record_ref",
                &obligation.governance_record_ref,
                &mut issues,
            );
            required("obligation.description", &obligation.description, &mut issues);
            required(
                "obligation.spatial_feature_ref",
                &obligation.spatial_feature_ref,
                &mut issues,
            );
            required(
                "obligation.responsible_party_ref",
                &obligation.responsible_party_ref,
                &mut issues,
            );
            required("obligation.due_at", &obligation.due_at, &mut issues);
            required(
                "obligation.verification_protocol_ref",
                &obligation.verification_protocol_ref,
                &mut issues,
            );
            if !evidence_ids.contains(&obligation.originating_evidence_id) {
                issues.push(format!(
                    "obligation {} references missing originating evidence {}",
                    obligation.id, obligation.originating_evidence_id
                ));
            }
            if let Some(value) = obligation.target_value {
                if !value.is_finite() {
                    issues.push(format!("obligation {} has a non-finite target", obligation.id));
                }
                if obligation.target_unit.as_deref().unwrap_or("").trim().is_empty() {
                    issues.push(format!(
                        "obligation {} has a target value without a unit",
                        obligation.id
                    ));
                }
            } else if obligation.target_unit.is_some() {
                issues.push(format!(
                    "obligation {} has a target unit without a target value",
                    obligation.id
                ));
            }
            for evidence_id in &obligation.evidence_ids {
                if !evidence_ids.contains(evidence_id) {
                    issues.push(format!(
                        "obligation {} references missing evidence {}",
                        obligation.id, evidence_id
                    ));
                }
            }
            if obligation.state == ObligationState::VerifiedComplete {
                let verifier = obligation
                    .independent_verifier_ref
                    .as_deref()
                    .unwrap_or("");
                if verifier.trim().is_empty()
                    || verifier == obligation.responsible_party_ref
                    || obligation.evidence_ids.is_empty()
                {
                    issues.push(format!(
                        "obligation {} is verified_complete without independent verifier and completion evidence",
                        obligation.id
                    ));
                }
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

fn required(field: &str, value: &str, issues: &mut Vec<String>) {
    if value.trim().is_empty() {
        issues.push(format!("{field} must not be empty"));
    }
}

fn unique_id(
    kind: &str,
    value: &str,
    seen: &mut HashSet<String>,
    issues: &mut Vec<String>,
) {
    if value.trim().is_empty() {
        issues.push(format!("{kind} id must not be empty"));
    } else if !seen.insert(value.to_owned()) {
        issues.push(format!("duplicate {kind} id {value}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(id: &str, kind: EvidenceKind) -> EvidenceReference {
        EvidenceReference {
            id: id.into(),
            kind,
            source_ref: format!("urn:test:evidence:{id}"),
            captured_at: "2026-11-18T10:00:00Z".into(),
            sha256: Some("a".repeat(64)),
            disclosure_class: DisclosureClass::Restricted,
        }
    }

    fn assertion(bases: Vec<ClaimBasis>) -> TerritorialAssertion {
        TerritorialAssertion {
            id: "assertion-1".into(),
            holder_ref: "party:community-1".into(),
            interest_kind: InterestKind::SurfaceLand,
            source_interest_type_code: None,
            source_interest_type_profile_ref: None,
            relationship: RelationshipKind::Right,
            bases,
            dispute_state: DisputeState::Unassessed,
            spatial: SpatialExtent {
                feature_ref: "urn:ogc:feature:parcel-1".into(),
                horizontal_crs: "https://www.opengis.net/def/crs/EPSG/0/4326".into(),
                vertical_min_m: None,
                vertical_max_m: None,
                vertical_datum: None,
                horizontal_uncertainty_m: 4.5,
            },
            temporal: TemporalExtent {
                valid_from: "1996-06-26".into(),
                valid_until: None,
            },
            evidence_ids: vec!["evidence-1".into()],
            authority_ids: Vec::new(),
            challenge_ids: Vec::new(),
            challenge_assessment_id: None,
            disclosure_class: DisclosureClass::CommunityControlled,
        }
    }

    fn record(assertions: Vec<TerritorialAssertion>) -> TerritorialRecord {
        TerritorialRecord {
            schema_version: TERRITORIAL_RECORD_SCHEMA_VERSION,
            territory_ref: "urn:test:territory-1".into(),
            created_at: "2026-11-18T10:00:00Z".into(),
            evidence: vec![evidence("evidence-1", EvidenceKind::CommunityTestimony)],
            authorities: Vec::new(),
            challenges: Vec::new(),
            challenge_assessments: Vec::new(),
            assertions,
            obligations: Vec::new(),
        }
    }

    #[test]
    fn asserted_claim_does_not_require_or_imply_registered_title() {
        let value = record(vec![assertion(vec![ClaimBasis::Asserted])]);
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn testimony_alone_does_not_establish_community_attestation() {
        let value = record(vec![assertion(vec![ClaimBasis::CommunityAttested])]);
        let issues = value.validate().expect_err(
            "community testimony alone must not be promoted to a community decision",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("community_attested without community-decision evidence")
        }));
    }

    #[test]
    fn no_challenge_observation_requires_a_complete_declared_frontier() {
        let mut value = record(vec![assertion(vec![ClaimBasis::Asserted])]);
        value.assertions[0].dispute_state = DisputeState::NoChallengeObservedWithinFrontier;
        let issues = value.validate().expect_err(
            "a no-challenge observation requires a complete declared evidence frontier",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("without a complete declared challenge frontier")
        }));

        value.evidence.push(evidence(
            "challenge-search-1",
            EvidenceKind::ChallengeSearchReport,
        ));
        value.challenge_assessments.push(ChallengeAssessment {
            id: "challenge-frontier-1".into(),
            scope_ref: "urn:test:scope:registry-and-tribunal-records".into(),
            frontier_ref: "urn:test:frontier:za:2026-11-18".into(),
            assessed_at: "2026-11-18T10:10:00Z".into(),
            coverage: ChallengeCoverage::CompleteWithinDeclaredFrontier,
            source_refs: vec![
                "urn:test:source:registry".into(),
                "urn:test:source:tribunal".into(),
            ],
            evidence_ids: vec!["challenge-search-1".into()],
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].challenge_assessment_id = Some("challenge-frontier-1".into());
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn active_challenge_cannot_be_hidden_by_no_challenge_observation() {
        let mut value = record(vec![assertion(vec![ClaimBasis::Asserted])]);
        value.evidence.push(evidence(
            "challenge-search-1",
            EvidenceKind::ChallengeSearchReport,
        ));
        value.challenge_assessments.push(ChallengeAssessment {
            id: "challenge-frontier-1".into(),
            scope_ref: "urn:test:scope:registry-and-tribunal-records".into(),
            frontier_ref: "urn:test:frontier:za:2026-11-18".into(),
            assessed_at: "2026-11-18T10:10:00Z".into(),
            coverage: ChallengeCoverage::CompleteWithinDeclaredFrontier,
            source_refs: vec!["urn:test:source:registry".into()],
            evidence_ids: vec!["challenge-search-1".into()],
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].dispute_state = DisputeState::NoChallengeObservedWithinFrontier;
        value.assertions[0].challenge_assessment_id = Some("challenge-frontier-1".into());
        value.challenges.push(ChallengeReference {
            id: "challenge-1".into(),
            raised_by_ref: "party:challenger".into(),
            filed_at: "2026-11-18T10:05:00Z".into(),
            state: ChallengeState::Submitted,
            evidence_ids: vec!["evidence-1".into()],
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].challenge_ids = vec!["challenge-1".into()];
        let issues = value.validate().expect_err(
            "an active challenge must contradict a no-challenge observation",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("no challenge observed while linked challenge records exist")
        }));
    }

    #[test]
    fn resolved_state_requires_every_linked_challenge_to_be_terminal() {
        let mut value = record(vec![assertion(vec![ClaimBasis::Asserted])]);
        value.challenges.push(ChallengeReference {
            id: "challenge-closed".into(),
            raised_by_ref: "party:challenger-1".into(),
            filed_at: "2026-11-17T10:05:00Z".into(),
            state: ChallengeState::Rejected,
            evidence_ids: vec!["evidence-1".into()],
            disclosure_class: DisclosureClass::Restricted,
        });
        value.challenges.push(ChallengeReference {
            id: "challenge-open".into(),
            raised_by_ref: "party:challenger-2".into(),
            filed_at: "2026-11-18T10:05:00Z".into(),
            state: ChallengeState::Submitted,
            evidence_ids: vec!["evidence-1".into()],
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].challenge_ids =
            vec!["challenge-closed".into(), "challenge-open".into()];
        value.assertions[0].dispute_state = DisputeState::Resolved;
        let issues = value.validate().expect_err(
            "one closed challenge must not conceal another active challenge",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("resolved while one or more linked challenges remain active")
        }));
    }

    #[test]
    fn community_decision_requires_scoped_community_authority() {
        let mut value = record(vec![assertion(vec![ClaimBasis::CommunityAttested])]);
        value.evidence.push(evidence("decision-1", EvidenceKind::CommunityDecision));
        value.assertions[0].evidence_ids.push("decision-1".into());
        let issues = value.validate().expect_err(
            "community decision evidence without a scoped community authority must not pass",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("without community-governance/customary authority")
        }));
        value.authorities.push(AuthorityReference {
            id: "community-authority".into(),
            issuer_ref: "institution:community-governance".into(),
            kind: AuthorityKind::CommunityGovernanceBody,
            jurisdiction: "ZA".into(),
            scope: "decision for this territory".into(),
            source_evidence_id: "decision-1".into(),
            valid_from: "2026-11-18".into(),
            valid_until: None,
            disclosure_class: DisclosureClass::CommunityControlled,
        });
        value.assertions[0].authority_ids = vec!["community-authority".into()];
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn administrative_record_requires_registry_or_regulator_evidence() {
        let mut value = record(vec![assertion(vec![ClaimBasis::AdministrativeRecord])]);
        assert!(value.validate().is_err());
        value.evidence.push(evidence("registry-1", EvidenceKind::RegistryExtract));
        value.authorities.push(AuthorityReference {
            id: "authority-1".into(),
            issuer_ref: "institution:registry".into(),
            kind: AuthorityKind::Registry,
            jurisdiction: "ZA".into(),
            scope: "land registration".into(),
            source_evidence_id: "registry-1".into(),
            valid_from: "2026-01-01".into(),
            valid_until: None,
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].authority_ids = vec!["authority-1".into()];
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn a_recorded_claim_can_remain_disputed() {
        let mut value = record(vec![assertion(vec![
            ClaimBasis::AdministrativeRecord,
            ClaimBasis::CommunityAttested,
        ])]);
        value.evidence.push(evidence("registry-1", EvidenceKind::RegistryExtract));
        value.evidence.push(evidence("challenge-1", EvidenceKind::CommunityDecision));
        value.assertions[0].evidence_ids.push("challenge-1".into());
        value.authorities.push(AuthorityReference {
            id: "authority-1".into(),
            issuer_ref: "institution:registry".into(),
            kind: AuthorityKind::Registry,
            jurisdiction: "ZA".into(),
            scope: "land registration".into(),
            source_evidence_id: "registry-1".into(),
            valid_from: "2026-01-01".into(),
            valid_until: None,
            disclosure_class: DisclosureClass::Restricted,
        });
        value.authorities.push(AuthorityReference {
            id: "community-authority".into(),
            issuer_ref: "institution:community-governance".into(),
            kind: AuthorityKind::CommunityGovernanceBody,
            jurisdiction: "ZA".into(),
            scope: "community decision for this asserted territory".into(),
            source_evidence_id: "challenge-1".into(),
            valid_from: "2026-11-18".into(),
            valid_until: None,
            disclosure_class: DisclosureClass::CommunityControlled,
        });
        value.challenges.push(ChallengeReference {
            id: "challenge-record-1".into(),
            raised_by_ref: "party:community-2".into(),
            filed_at: "2026-11-18T10:05:00Z".into(),
            state: ChallengeState::Submitted,
            evidence_ids: vec!["challenge-1".into()],
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].authority_ids = vec![
            "authority-1".into(),
            "community-authority".into(),
        ];
        value.assertions[0].dispute_state = DisputeState::Challenged;
        value.assertions[0].challenge_ids = vec!["challenge-record-1".into()];
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn court_decision_requires_court_order_and_court_authority() {
        let mut value = record(vec![assertion(vec![ClaimBasis::CourtDecision])]);
        assert!(value.validate().is_err());
        value.evidence.push(evidence("court-1", EvidenceKind::CourtOrder));
        value.authorities.push(AuthorityReference {
            id: "court-authority".into(),
            issuer_ref: "institution:court".into(),
            kind: AuthorityKind::CourtOrTribunal,
            jurisdiction: "ZA".into(),
            scope: "case-specific order".into(),
            source_evidence_id: "court-1".into(),
            valid_from: "2026-11-18".into(),
            valid_until: None,
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].authority_ids = vec!["court-authority".into()];
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn contractual_instrument_requires_contracting_party_authority() {
        let mut value = record(vec![assertion(vec![ClaimBasis::ContractualInstrument])]);
        value.evidence.push(evidence("contract-1", EvidenceKind::Contract));
        value.assertions[0].evidence_ids.push("contract-1".into());
        let issues = value.validate().expect_err(
            "a contract document alone must not establish the authority basis",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("without contracting-party authority")
        }));

        value.authorities.push(AuthorityReference {
            id: "contract-authority".into(),
            issuer_ref: "institution:contracting-party".into(),
            kind: AuthorityKind::ContractingParty,
            jurisdiction: "ZA".into(),
            scope: "rights and obligations defined by contract-1".into(),
            source_evidence_id: "contract-1".into(),
            valid_from: "2026-11-18".into(),
            valid_until: None,
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].authority_ids = vec!["contract-authority".into()];
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn statutory_instrument_requires_statutory_regulator_authority() {
        let mut value = record(vec![assertion(vec![ClaimBasis::StatutoryInstrument])]);
        value.evidence.push(evidence("statute-1", EvidenceKind::StatutoryInstrument));
        value.assertions[0].evidence_ids.push("statute-1".into());
        value.authorities.push(AuthorityReference {
            id: "wrong-authority".into(),
            issuer_ref: "institution:other".into(),
            kind: AuthorityKind::Other,
            jurisdiction: "ZA".into(),
            scope: "unrelated authority".into(),
            source_evidence_id: "statute-1".into(),
            valid_from: "2026-11-18".into(),
            valid_until: None,
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].authority_ids = vec!["wrong-authority".into()];
        let issues = value.validate().expect_err(
            "a statutory instrument linked to a non-regulator must not pass",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("without statutory-regulator authority")
        }));

        value.authorities.push(AuthorityReference {
            id: "regulator-authority".into(),
            issuer_ref: "institution:statutory-regulator".into(),
            kind: AuthorityKind::StatutoryRegulator,
            jurisdiction: "ZA".into(),
            scope: "statutory instrument administration".into(),
            source_evidence_id: "statute-1".into(),
            valid_from: "2026-11-18".into(),
            valid_until: None,
            disclosure_class: DisclosureClass::Restricted,
        });
        value.assertions[0].authority_ids.push("regulator-authority".into());
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn verified_obligation_requires_independent_verifier_and_evidence() {
        let mut value = record(Vec::new());
        value.evidence.push(evidence(
            "obligation-origin",
            EvidenceKind::Permit,
        ));
        value.evidence.push(evidence(
            "completion-proof",
            EvidenceKind::EnvironmentalMeasurement,
        ));
        value.obligations.push(StewardshipObligation {
            id: "restore-1".into(),
            governance_record_ref: "urn:mycelix:stewardship:commitment:restore-1".into(),
            kind: ObligationKind::RestoreHabitat,
            description: "Restore disturbed riparian habitat".into(),
            spatial_feature_ref: "urn:ogc:feature:site-1".into(),
            responsible_party_ref: "party:operator".into(),
            originating_evidence_id: "obligation-origin".into(),
            due_at: "2030-12-31".into(),
            target_value: Some(80.0),
            target_unit: Some("percent_native_cover".into()),
            verification_protocol_ref: "urn:test:protocol:habitat-v1".into(),
            independent_verifier_ref: Some("party:independent-verifier".into()),
            state: ObligationState::VerifiedComplete,
            evidence_ids: vec!["completion-proof".into()],
            disclosure_class: DisclosureClass::Restricted,
        });
        assert_eq!(value.validate(), Ok(()));
        value.obligations[0].independent_verifier_ref =
            Some("party:operator".into());
        assert!(value.validate().is_err());
    }

    #[test]
    fn stewardship_projection_requires_upstream_governance_record_reference() {
        let mut value = record(Vec::new());
        value.evidence.push(evidence("obligation-origin", EvidenceKind::Permit));
        value.obligations.push(StewardshipObligation {
            id: "restore-1".into(),
            governance_record_ref: String::new(),
            kind: ObligationKind::RestoreHabitat,
            description: "Restore disturbed riparian habitat".into(),
            spatial_feature_ref: "urn:ogc:feature:site-1".into(),
            responsible_party_ref: "party:operator".into(),
            originating_evidence_id: "obligation-origin".into(),
            due_at: "2030-12-31".into(),
            target_value: None,
            target_unit: None,
            verification_protocol_ref: "urn:test:protocol:habitat-v1".into(),
            independent_verifier_ref: None,
            state: ObligationState::Open,
            evidence_ids: Vec::new(),
            disclosure_class: DisclosureClass::Restricted,
        });
        let issues = value.validate().expect_err(
            "a Sol-Atlas obligation projection must retain its upstream governance record",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("obligation.governance_record_ref must not be empty")
        }));
    }

    #[test]
    fn jurisdictional_interest_code_must_survive_other_mapping() {
        let mut value = record(vec![assertion(vec![ClaimBasis::Asserted])]);
        value.assertions[0].interest_kind = InterestKind::Other;
        let issues = value.validate().expect_err(
            "generic Other must not erase a jurisdiction-specific interest type",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("without preserving its source code and profile")
        }));

        value.assertions[0].source_interest_type_code = Some("CUSTOMARY_GRAZING_V2".into());
        value.assertions[0].source_interest_type_profile_ref =
            Some("urn:example:jurisdiction-profile:za-customary:v2".into());
        assert_eq!(value.validate(), Ok(()));
    }

    #[test]
    fn source_interest_type_code_and_profile_must_be_a_complete_pair() {
        let mut value = record(vec![assertion(vec![ClaimBasis::Asserted])]);
        value.assertions[0].source_interest_type_code = Some("WATER_USE_CLASS_A".into());
        let issues = value.validate().expect_err(
            "a source code without its profile namespace is ambiguous",
        );
        assert!(issues.iter().any(|issue| {
            issue.contains("incomplete source interest-type code/profile pair")
        }));
    }

    #[test]
    fn invalid_digest_and_negative_uncertainty_are_rejected() {
        let mut value = record(vec![assertion(vec![ClaimBasis::Asserted])]);
        value.evidence[0].sha256 = Some("not-a-digest".into());
        value.assertions[0].spatial.horizontal_uncertainty_m = -1.0;
        assert!(value.validate().is_err());
    }

    #[test]
    fn culturally_sensitive_records_are_not_public_by_default() {
        let item = EvidenceReference {
            id: "private-evidence".into(),
            kind: EvidenceKind::CommunityTestimony,
            source_ref: "urn:test:private".into(),
            captured_at: "2026-11-18T10:00:00Z".into(),
            sha256: None,
            disclosure_class: DisclosureClass::default(),
        };
        assert_eq!(item.disclosure_class, DisclosureClass::Restricted);
    }
}
