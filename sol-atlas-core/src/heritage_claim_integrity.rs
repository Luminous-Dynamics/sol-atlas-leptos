// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Archaeological construct-validity and claim-ceiling evaluation.
//!
//! This module prevents remote-sensing proxies, model outputs and simulations
//! from being promoted into stronger archaeological or historical claims
//! without the evidence classes, independence assessment and human review
//! required by the declared policy. It is a structural/policy evaluator, not
//! an authenticator of evidence, rights, independence audits or reviewer authority.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const HERITAGE_CLAIM_SCHEMA_VERSION: &str = "luminous.heritage-claim.v0.1";
pub const HERITAGE_CLAIM_REPORT_SCHEMA_VERSION: &str = "luminous.heritage-claim-report.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    MeasuredSignal,
    ProxyInterpretation,
    AnthropogenicFeatureCandidate,
    ArchaeologicalFeatureCandidate,
    ArchaeologicalSiteInterpretation,
    HistoricalProcessInterpretation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimCeiling {
    NoSupportedClaim,
    MeasuredSignal,
    ProxyInterpretation,
    AnthropogenicFeatureCandidate,
    ArchaeologicalFeatureCandidate,
    ArchaeologicalSiteInterpretation,
    HistoricalProcessInterpretation,
}

impl ClaimCeiling {
    fn rank(self) -> u8 {
        match self {
            Self::NoSupportedClaim => 0,
            Self::MeasuredSignal => 1,
            Self::ProxyInterpretation => 2,
            Self::AnthropogenicFeatureCandidate => 3,
            Self::ArchaeologicalFeatureCandidate => 4,
            Self::ArchaeologicalSiteInterpretation => 5,
            Self::HistoricalProcessInterpretation => 6,
        }
    }

    fn supports(self, claim: ClaimKind) -> bool {
        self.rank() >= claim.rank()
    }
}

impl ClaimKind {
    fn rank(self) -> u8 {
        match self {
            Self::MeasuredSignal => 1,
            Self::ProxyInterpretation => 2,
            Self::AnthropogenicFeatureCandidate => 3,
            Self::ArchaeologicalFeatureCandidate => 4,
            Self::ArchaeologicalSiteInterpretation => 5,
            Self::HistoricalProcessInterpretation => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    RemoteSensingSignal,
    GeophysicalSignal,
    SurfaceFieldAssessment,
    ExcavationStratigraphy,
    DiagnosticMaterialAnalysis,
    ChronometricMeasurement,
    HistoricalDocument,
    ExpertInterpretation,
    CatalogueAssertion,
    ModelPrediction,
    Simulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOrigin {
    DirectObservation,
    HumanInterpretation,
    Catalogue,
    ModelOutput,
    Simulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    Supports,
    Contradicts,
    ContextOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceModality {
    SatelliteOptical,
    SatelliteSar,
    AirborneLidar,
    UavPhotogrammetry,
    ThermalImaging,
    GroundGeophysics,
    SurfaceFieldSurvey,
    Excavation,
    MaterialLaboratory,
    ChronometricLaboratory,
    HistoricalArchive,
    ExpertReview,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndependenceStatus {
    IndependentlyAssessed,
    Dependent,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsStatus {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanAdjudicationOutcome {
    Accepted,
    Rejected,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEvidence {
    pub evidence_id: String,
    pub evidence_kind: EvidenceKind,
    pub origin: EvidenceOrigin,
    pub relation: EvidenceRelation,
    pub modality: EvidenceModality,
    pub source_asset_sha256: String,
    /// Root identity shared by raw assets and all derived products from them.
    pub source_lineage_id: String,
    /// Acquisition campaign/event identity. Separate files from one capture
    /// campaign do not become independent observations.
    pub acquisition_group_id: String,
    /// Declared method family. Different software names do not imply different methods.
    pub method_family_id: String,
    pub independence_status: IndependenceStatus,
    pub independence_group_id: Option<String>,
    /// Digest of the external independence assessment, not proof authenticated here.
    pub independence_assessment_sha256: Option<String>,
    pub rights_status: RightsStatus,
    pub rights_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanAdjudication {
    pub reviewer_id: String,
    pub reviewer_scope: String,
    pub adjudicated_at: String,
    pub outcome: HumanAdjudicationOutcome,
    pub decision_ref: String,
    pub decision_sha256: String,
    /// Must be exactly the evidence set in the claim, not a subset or stale set.
    pub reviewed_evidence_ids: Vec<String>,
    pub rights_status: RightsStatus,
    pub rights_receipt_ref: Option<String>,
    /// Must be explicit when any evidence is marked as contradicting the claim.
    pub contradictions_resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageClaimInput {
    pub schema_version: String,
    pub claim_id: String,
    pub candidate_id: String,
    pub declared_claim: ClaimKind,
    pub proxy_family_id: Option<String>,
    pub evidence: Vec<ClaimEvidence>,
    pub adjudication: Option<HumanAdjudication>,
    /// Public coordinates are not part of this evaluator. This field controls
    /// whether a downstream public representation may expose exact geometry.
    pub coordinate_access_restricted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimDisposition {
    QualifiedAtDeclaredClaim,
    Overclaimed,
    Contradicted,
    Indeterminate,
    RejectedByAdjudication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimReason {
    NoDirectObservation,
    ProxyFamilyMissing,
    InsufficientIndependentEvidenceGroups,
    InsufficientDistinctModalities,
    ArchaeologicalFieldEvidenceMissing,
    HumanAdjudicationMissing,
    HumanAdjudicationEvidenceSetMismatch,
    HumanAdjudicationRightsNotVerified,
    HumanAdjudicationReferenceMissing,
    HumanAdjudicationDigestInvalid,
    ContradictoryEvidenceUnresolved,
    ChronologicalOrArchivalEvidenceMissing,
    EvidenceRightsNotVerified,
    EvidenceRightsReceiptMissing,
    NonIndependentEvidenceIgnored,
    ModelOrSimulationEvidenceIgnored,
    DuplicateEvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageClaimReport {
    pub schema_version: String,
    pub claim_id: String,
    pub candidate_id: String,
    pub declared_claim: ClaimKind,
    pub evidence_supported_ceiling: ClaimCeiling,
    pub disposition: ClaimDisposition,
    pub evidence_record_count: usize,
    pub direct_support_count: usize,
    pub independent_support_group_count: usize,
    pub distinct_support_modality_count: usize,
    pub archaeological_field_evidence_count: usize,
    pub chronological_or_archival_evidence_count: usize,
    pub model_or_simulation_evidence_count: usize,
    pub contradictory_evidence_count: usize,
    pub rights_blocked_evidence_count: usize,
    pub human_adjudication_applies: bool,
    pub adjudication_evidence_set_matches: bool,
    pub coordinate_access_restricted: bool,
    pub exact_coordinates_publication_allowed: bool,
    pub reasons: Vec<ClaimReason>,
    /// A claim-ceiling decision is not an archaeological fact or truth verdict.
    pub archaeological_fact_established: bool,
    pub historical_truth_established: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageClaimError {
    WrongSchemaVersion,
    EmptyClaimId,
    EmptyCandidateId,
    ProxyFamilyRequired,
    NoEvidence,
    TooManyEvidenceRecords,
    EmptyEvidenceId,
    DuplicateEvidenceId(String),
    InvalidSourceDigest { evidence_id: String },
    EmptySourceLineage { evidence_id: String },
    EmptyAcquisitionGroup { evidence_id: String },
    EmptyMethodFamily { evidence_id: String },
    InvalidIndependenceAssessment { evidence_id: String },
    InvalidIndependenceReference { evidence_id: String },
    InvalidRightsReceipt { evidence_id: String },
    EmptyReviewerId,
    EmptyReviewerScope,
    EmptyAdjudicationTimestamp,
    EmptyAdjudicationReference,
    InvalidAdjudicationDigest,
    InvalidAdjudicationRightsReceipt,
    DuplicateReviewedEvidenceId(String),
}

impl fmt::Display for HeritageClaimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage claim evaluation error: {self:?}")
    }
}

impl std::error::Error for HeritageClaimError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_direct_observation(item: &ClaimEvidence) -> bool {
    item.origin == EvidenceOrigin::DirectObservation
        && !matches!(
            item.evidence_kind,
            EvidenceKind::ExpertInterpretation
                | EvidenceKind::CatalogueAssertion
                | EvidenceKind::ModelPrediction
                | EvidenceKind::Simulation
        )
}

fn is_model_or_simulation(item: &ClaimEvidence) -> bool {
    matches!(
        item.origin,
        EvidenceOrigin::ModelOutput | EvidenceOrigin::Simulation
    ) || matches!(
        item.evidence_kind,
        EvidenceKind::ModelPrediction | EvidenceKind::Simulation
    )
}

fn is_archaeological_field_evidence(item: &ClaimEvidence) -> bool {
    is_direct_observation(item)
        && matches!(
            item.evidence_kind,
            EvidenceKind::SurfaceFieldAssessment
                | EvidenceKind::ExcavationStratigraphy
                | EvidenceKind::DiagnosticMaterialAnalysis
        )
}

fn is_chronological_or_archival(item: &ClaimEvidence) -> bool {
    is_direct_observation(item)
        && matches!(
            item.evidence_kind,
            EvidenceKind::ChronometricMeasurement | EvidenceKind::HistoricalDocument
        )
}

fn is_remote_or_geophysical_proxy(item: &ClaimEvidence) -> bool {
    is_direct_observation(item)
        && matches!(
            item.evidence_kind,
            EvidenceKind::RemoteSensingSignal | EvidenceKind::GeophysicalSignal
        )
        && matches!(
            item.modality,
            EvidenceModality::SatelliteOptical
                | EvidenceModality::SatelliteSar
                | EvidenceModality::AirborneLidar
                | EvidenceModality::UavPhotogrammetry
                | EvidenceModality::ThermalImaging
                | EvidenceModality::GroundGeophysics
        )
}

fn is_usable_support(item: &ClaimEvidence) -> bool {
    item.relation == EvidenceRelation::Supports
        && item.rights_status == RightsStatus::Verified
        && item.rights_receipt_ref.as_deref().is_some_and(non_empty)
        && is_direct_observation(item)
}

fn validate_input(input: &HeritageClaimInput) -> Result<(), HeritageClaimError> {
    if input.schema_version != HERITAGE_CLAIM_SCHEMA_VERSION {
        return Err(HeritageClaimError::WrongSchemaVersion);
    }
    if !non_empty(&input.claim_id) {
        return Err(HeritageClaimError::EmptyClaimId);
    }
    if !non_empty(&input.candidate_id) {
        return Err(HeritageClaimError::EmptyCandidateId);
    }
    if matches!(
        input.declared_claim,
        ClaimKind::ProxyInterpretation
            | ClaimKind::AnthropogenicFeatureCandidate
            | ClaimKind::ArchaeologicalFeatureCandidate
            | ClaimKind::ArchaeologicalSiteInterpretation
            | ClaimKind::HistoricalProcessInterpretation
    ) && input.declared_claim == ClaimKind::ProxyInterpretation
        && input
            .proxy_family_id
            .as_deref()
            .is_none_or(|value| !non_empty(value))
    {
        return Err(HeritageClaimError::ProxyFamilyRequired);
    }
    if input.evidence.is_empty() {
        return Err(HeritageClaimError::NoEvidence);
    }
    if input.evidence.len() > 100_000 {
        return Err(HeritageClaimError::TooManyEvidenceRecords);
    }

    let mut evidence_ids = BTreeSet::new();
    for item in &input.evidence {
        if !non_empty(&item.evidence_id) {
            return Err(HeritageClaimError::EmptyEvidenceId);
        }
        if !evidence_ids.insert(item.evidence_id.clone()) {
            return Err(HeritageClaimError::DuplicateEvidenceId(
                item.evidence_id.clone(),
            ));
        }
        if !valid_sha256(&item.source_asset_sha256) {
            return Err(HeritageClaimError::InvalidSourceDigest {
                evidence_id: item.evidence_id.clone(),
            });
        }
        if !non_empty(&item.source_lineage_id) {
            return Err(HeritageClaimError::EmptySourceLineage {
                evidence_id: item.evidence_id.clone(),
            });
        }
        if !non_empty(&item.acquisition_group_id) {
            return Err(HeritageClaimError::EmptyAcquisitionGroup {
                evidence_id: item.evidence_id.clone(),
            });
        }
        if !non_empty(&item.method_family_id) {
            return Err(HeritageClaimError::EmptyMethodFamily {
                evidence_id: item.evidence_id.clone(),
            });
        }
        match item.independence_status {
            IndependenceStatus::IndependentlyAssessed => {
                if item
                    .independence_group_id
                    .as_deref()
                    .is_none_or(|value| !non_empty(value))
                {
                    return Err(HeritageClaimError::InvalidIndependenceReference {
                        evidence_id: item.evidence_id.clone(),
                    });
                }
                if item
                    .independence_assessment_sha256
                    .as_deref()
                    .is_none_or(|value| !valid_sha256(value))
                {
                    return Err(HeritageClaimError::InvalidIndependenceAssessment {
                        evidence_id: item.evidence_id.clone(),
                    });
                }
            }
            IndependenceStatus::Dependent | IndependenceStatus::Unknown => {
                if item.independence_group_id.is_some()
                    || item.independence_assessment_sha256.is_some()
                {
                    return Err(HeritageClaimError::InvalidIndependenceReference {
                        evidence_id: item.evidence_id.clone(),
                    });
                }
            }
        }
        if item.rights_status == RightsStatus::Verified
            && item
                .rights_receipt_ref
                .as_deref()
                .is_none_or(|value| !non_empty(value))
        {
            return Err(HeritageClaimError::InvalidRightsReceipt {
                evidence_id: item.evidence_id.clone(),
            });
        }
    }

    if let Some(adjudication) = &input.adjudication {
        if !non_empty(&adjudication.reviewer_id) {
            return Err(HeritageClaimError::EmptyReviewerId);
        }
        if !non_empty(&adjudication.reviewer_scope) {
            return Err(HeritageClaimError::EmptyReviewerScope);
        }
        if !non_empty(&adjudication.adjudicated_at) {
            return Err(HeritageClaimError::EmptyAdjudicationTimestamp);
        }
        if !non_empty(&adjudication.decision_ref) {
            return Err(HeritageClaimError::EmptyAdjudicationReference);
        }
        if !valid_sha256(&adjudication.decision_sha256) {
            return Err(HeritageClaimError::InvalidAdjudicationDigest);
        }
        if adjudication.rights_status == RightsStatus::Verified
            && adjudication
                .rights_receipt_ref
                .as_deref()
                .is_none_or(|value| !non_empty(value))
        {
            return Err(HeritageClaimError::InvalidAdjudicationRightsReceipt);
        }
        let mut reviewed_ids = BTreeSet::new();
        for evidence_id in &adjudication.reviewed_evidence_ids {
            if !reviewed_ids.insert(evidence_id.clone()) {
                return Err(HeritageClaimError::DuplicateReviewedEvidenceId(
                    evidence_id.clone(),
                ));
            }
        }
    }
    Ok(())
}

/// Count only independently assessed direct support records whose source
/// lineage, acquisition group, method family, source digest and independence
/// group are all unique. This deliberately conservative check catches simple
/// duplicate/derived-output inflation; it does not authenticate the external
/// independence assessment or establish statistical independence.
fn independent_support_records<'a>(evidence: &'a [ClaimEvidence]) -> Vec<&'a ClaimEvidence> {
    let mut selected = Vec::new();
    let mut lineages = BTreeSet::new();
    let mut acquisitions = BTreeSet::new();
    let mut methods = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut groups = BTreeSet::new();

    let mut ordered = evidence.iter().collect::<Vec<_>>();
    ordered.sort_by(|a, b| a.evidence_id.cmp(&b.evidence_id));

    for item in ordered {
        if !is_usable_support(item)
            || item.independence_status != IndependenceStatus::IndependentlyAssessed
        {
            continue;
        }
        let group = item.independence_group_id.as_deref().unwrap_or_default();
        if lineages.contains(&item.source_lineage_id)
            || acquisitions.contains(&item.acquisition_group_id)
            || methods.contains(&item.method_family_id)
            || sources.contains(&item.source_asset_sha256)
            || groups.contains(group)
        {
            continue;
        }
        lineages.insert(item.source_lineage_id.clone());
        acquisitions.insert(item.acquisition_group_id.clone());
        methods.insert(item.method_family_id.clone());
        sources.insert(item.source_asset_sha256.clone());
        groups.insert(group.to_owned());
        selected.push(item);
    }
    selected
}

fn adjudication_matches_evidence(input: &HeritageClaimInput) -> bool {
    let Some(adjudication) = &input.adjudication else {
        return false;
    };
    let current = input
        .evidence
        .iter()
        .map(|item| item.evidence_id.clone())
        .collect::<BTreeSet<_>>();
    let reviewed = adjudication
        .reviewed_evidence_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    current == reviewed
}

fn adjudication_is_accepted(input: &HeritageClaimInput) -> bool {
    input.adjudication.as_ref().is_some_and(|item| {
        item.outcome == HumanAdjudicationOutcome::Accepted && adjudication_matches_evidence(input)
    })
}

fn adjudication_rights_verified(input: &HeritageClaimInput) -> bool {
    input.adjudication.as_ref().is_some_and(|item| {
        item.reviewer_id.trim().len() > 0
            && item.decision_ref.trim().len() > 0
            && item.rights_status == RightsStatus::Verified
            && item.rights_receipt_ref.as_deref().is_some_and(non_empty)
    })
}

fn evaluate_ceiling(
    input: &HeritageClaimInput,
    support: &[&ClaimEvidence],
    independent: &[&ClaimEvidence],
    modalities: &BTreeSet<EvidenceModality>,
    archaeology_count: usize,
    chronology_count: usize,
    contradiction_count: usize,
    adjudication_matches: bool,
) -> ClaimCeiling {
    let direct_count = support.len();
    if direct_count == 0 {
        return ClaimCeiling::NoSupportedClaim;
    }

    let mut ceiling = ClaimCeiling::MeasuredSignal;
    let has_proxy = support
        .iter()
        .any(|item| is_remote_or_geophysical_proxy(item));
    if has_proxy && input.proxy_family_id.as_deref().is_some_and(non_empty) {
        ceiling = ClaimCeiling::ProxyInterpretation;
    }

    let direct_field_assessment = support.iter().any(|item| {
        matches!(
            item.evidence_kind,
            EvidenceKind::SurfaceFieldAssessment
                | EvidenceKind::ExcavationStratigraphy
                | EvidenceKind::DiagnosticMaterialAnalysis
        )
    });
    if independent.len() >= 2 || direct_field_assessment {
        ceiling = ClaimCeiling::AnthropogenicFeatureCandidate;
    }

    if independent.len() >= 2 && modalities.len() >= 2 && archaeology_count >= 1 {
        ceiling = ClaimCeiling::ArchaeologicalFeatureCandidate;
    }

    let accepted = adjudication_is_accepted(input);
    let contradictions_resolved = input
        .adjudication
        .as_ref()
        .is_some_and(|item| item.contradictions_resolved);
    if ceiling.rank() >= ClaimCeiling::ArchaeologicalFeatureCandidate.rank()
        && accepted
        && adjudication_matches
        && adjudication_rights_verified(input)
        && (contradiction_count == 0 || contradictions_resolved)
    {
        ceiling = ClaimCeiling::ArchaeologicalSiteInterpretation;
    }

    if ceiling.rank() >= ClaimCeiling::ArchaeologicalSiteInterpretation.rank()
        && chronology_count >= 1
        && accepted
        && adjudication_matches
        && (contradiction_count == 0 || contradictions_resolved)
    {
        ceiling = ClaimCeiling::HistoricalProcessInterpretation;
    }
    ceiling
}

pub fn evaluate_heritage_claim(
    input: &HeritageClaimInput,
) -> Result<HeritageClaimReport, HeritageClaimError> {
    validate_input(input)?;

    let support = input
        .evidence
        .iter()
        .filter(|item| is_usable_support(item))
        .collect::<Vec<_>>();
    let direct_support_count = support.len();
    let independent = independent_support_records(&input.evidence);
    let modalities = independent
        .iter()
        .map(|item| item.modality)
        .collect::<BTreeSet<_>>();
    let archaeology_count = independent
        .iter()
        .filter(|item| is_archaeological_field_evidence(item))
        .count();
    let chronology_count = independent
        .iter()
        .filter(|item| is_chronological_or_archival(item))
        .count();
    let model_count = input
        .evidence
        .iter()
        .filter(|item| is_model_or_simulation(item))
        .count();
    let contradiction_count = input
        .evidence
        .iter()
        .filter(|item| {
            item.relation == EvidenceRelation::Contradicts
                && is_direct_observation(item)
                && item.rights_status == RightsStatus::Verified
                && item.rights_receipt_ref.as_deref().is_some_and(non_empty)
        })
        .count();
    let rights_blocked_count = input
        .evidence
        .iter()
        .filter(|item| {
            item.rights_status != RightsStatus::Verified
                || item
                    .rights_receipt_ref
                    .as_deref()
                    .is_none_or(|value| !non_empty(value))
        })
        .count();
    let adjudication_matches = adjudication_matches_evidence(input);
    let ceiling = evaluate_ceiling(
        input,
        &support,
        &independent,
        &modalities,
        archaeology_count,
        chronology_count,
        contradiction_count,
        adjudication_matches,
    );

    let accepted = adjudication_is_accepted(input);
    let contradictions_resolved = input
        .adjudication
        .as_ref()
        .is_some_and(|item| item.contradictions_resolved);
    let adjudication_rights_ok = adjudication_rights_verified(input);
    let has_unresolved_contradictions =
        contradiction_count > 0 && !(accepted && adjudication_matches && contradictions_resolved);
    let adjudication_set_invalid = input.adjudication.is_some() && !adjudication_matches;
    let adjudication_missing_for_site = matches!(
        input.declared_claim,
        ClaimKind::ArchaeologicalSiteInterpretation | ClaimKind::HistoricalProcessInterpretation
    ) && input.adjudication.is_none();

    let mut reasons = Vec::new();
    if direct_support_count == 0 {
        reasons.push(ClaimReason::NoDirectObservation);
    }
    if input.declared_claim == ClaimKind::ProxyInterpretation
        && input
            .proxy_family_id
            .as_deref()
            .is_none_or(|value| !non_empty(value))
    {
        reasons.push(ClaimReason::ProxyFamilyMissing);
    }
    if independent.len() < 2
        && input.declared_claim.rank() >= ClaimKind::AnthropogenicFeatureCandidate.rank()
    {
        reasons.push(ClaimReason::InsufficientIndependentEvidenceGroups);
    }
    if modalities.len() < 2
        && input.declared_claim.rank() >= ClaimKind::ArchaeologicalFeatureCandidate.rank()
    {
        reasons.push(ClaimReason::InsufficientDistinctModalities);
    }
    if archaeology_count == 0
        && input.declared_claim.rank() >= ClaimKind::ArchaeologicalFeatureCandidate.rank()
    {
        reasons.push(ClaimReason::ArchaeologicalFieldEvidenceMissing);
    }
    if adjudication_missing_for_site {
        reasons.push(ClaimReason::HumanAdjudicationMissing);
    }
    if adjudication_set_invalid {
        reasons.push(ClaimReason::HumanAdjudicationEvidenceSetMismatch);
    }
    if input.adjudication.is_some() && !adjudication_rights_ok {
        reasons.push(ClaimReason::HumanAdjudicationRightsNotVerified);
    }
    if let Some(item) = &input.adjudication {
        if !non_empty(&item.decision_ref) {
            reasons.push(ClaimReason::HumanAdjudicationReferenceMissing);
        }
        if !valid_sha256(&item.decision_sha256) {
            reasons.push(ClaimReason::HumanAdjudicationDigestInvalid);
        }
    }
    if has_unresolved_contradictions {
        reasons.push(ClaimReason::ContradictoryEvidenceUnresolved);
    }
    if chronology_count == 0 && input.declared_claim == ClaimKind::HistoricalProcessInterpretation {
        reasons.push(ClaimReason::ChronologicalOrArchivalEvidenceMissing);
    }
    if rights_blocked_count > 0 {
        reasons.push(ClaimReason::EvidenceRightsNotVerified);
    }
    if input.evidence.iter().any(|item| {
        item.rights_status == RightsStatus::Verified
            && item
                .rights_receipt_ref
                .as_deref()
                .is_none_or(|value| !non_empty(value))
    }) {
        reasons.push(ClaimReason::EvidenceRightsReceiptMissing);
    }
    if input.evidence.iter().any(|item| {
        item.relation == EvidenceRelation::Supports
            && item.independence_status != IndependenceStatus::IndependentlyAssessed
            && !is_archaeological_field_evidence(item)
    }) {
        reasons.push(ClaimReason::NonIndependentEvidenceIgnored);
    }
    if model_count > 0 {
        reasons.push(ClaimReason::ModelOrSimulationEvidenceIgnored);
    }

    reasons.sort_by_key(|item| *item as u8);
    reasons.dedup();

    let adjudication_rejected = input
        .adjudication
        .as_ref()
        .is_some_and(|item| item.outcome == HumanAdjudicationOutcome::Rejected);
    let indeterminate = rights_blocked_count > 0
        || adjudication_set_invalid
        || (input.adjudication.is_some() && !adjudication_rights_ok);
    let disposition = if adjudication_rejected && adjudication_matches {
        ClaimDisposition::RejectedByAdjudication
    } else if has_unresolved_contradictions {
        ClaimDisposition::Contradicted
    } else if indeterminate {
        ClaimDisposition::Indeterminate
    } else if !ceiling.supports(input.declared_claim) {
        ClaimDisposition::Overclaimed
    } else {
        ClaimDisposition::QualifiedAtDeclaredClaim
    };

    Ok(HeritageClaimReport {
        schema_version: HERITAGE_CLAIM_REPORT_SCHEMA_VERSION.to_owned(),
        claim_id: input.claim_id.clone(),
        candidate_id: input.candidate_id.clone(),
        declared_claim: input.declared_claim,
        evidence_supported_ceiling: ceiling,
        disposition,
        evidence_record_count: input.evidence.len(),
        direct_support_count,
        independent_support_group_count: independent.len(),
        distinct_support_modality_count: modalities.len(),
        archaeological_field_evidence_count: archaeology_count,
        chronological_or_archival_evidence_count: chronology_count,
        model_or_simulation_evidence_count: model_count,
        contradictory_evidence_count: contradiction_count,
        rights_blocked_evidence_count: rights_blocked_count,
        human_adjudication_applies: input.adjudication.is_some(),
        adjudication_evidence_set_matches: adjudication_matches,
        coordinate_access_restricted: input.coordinate_access_restricted,
        exact_coordinates_publication_allowed: !input.coordinate_access_restricted
            && rights_blocked_count == 0,
        reasons,
        archaeological_fact_established: false,
        historical_truth_established: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(
        evidence_id: &str,
        kind: EvidenceKind,
        modality: EvidenceModality,
        lineage: &str,
        acquisition: &str,
        method: &str,
        group: &str,
    ) -> ClaimEvidence {
        ClaimEvidence {
            evidence_id: evidence_id.into(),
            evidence_kind: kind,
            origin: EvidenceOrigin::DirectObservation,
            relation: EvidenceRelation::Supports,
            modality,
            source_asset_sha256: format!(
                "{:064x}",
                evidence_id.bytes().fold(0_u128, |value, byte| value
                    .wrapping_mul(257)
                    .wrapping_add(u128::from(byte)))
            ),
            source_lineage_id: lineage.into(),
            acquisition_group_id: acquisition.into(),
            method_family_id: method.into(),
            independence_status: IndependenceStatus::IndependentlyAssessed,
            independence_group_id: Some(group.into()),
            independence_assessment_sha256: Some("a".repeat(64)),
            rights_status: RightsStatus::Verified,
            rights_receipt_ref: Some(format!("urn:synthetic:rights:{evidence_id}")),
        }
    }

    fn base_input(claim: ClaimKind, evidence: Vec<ClaimEvidence>) -> HeritageClaimInput {
        HeritageClaimInput {
            schema_version: HERITAGE_CLAIM_SCHEMA_VERSION.into(),
            claim_id: "claim-001".into(),
            candidate_id: "candidate-001".into(),
            declared_claim: claim,
            proxy_family_id: Some("stone_wall_morphology".into()),
            evidence,
            adjudication: None,
            coordinate_access_restricted: true,
        }
    }

    fn two_independent_plus_field() -> Vec<ClaimEvidence> {
        vec![
            evidence(
                "lidar-1",
                EvidenceKind::RemoteSensingSignal,
                EvidenceModality::AirborneLidar,
                "lineage-lidar",
                "acq-lidar",
                "lidar_morphometry",
                "group-lidar",
            ),
            evidence(
                "field-1",
                EvidenceKind::SurfaceFieldAssessment,
                EvidenceModality::SurfaceFieldSurvey,
                "lineage-field",
                "acq-field",
                "field_archaeological_assessment",
                "group-field",
            ),
        ]
    }

    fn accepted_adjudication(evidence: &[ClaimEvidence]) -> HumanAdjudication {
        HumanAdjudication {
            reviewer_id: "reviewer-001".into(),
            reviewer_scope: "stone-built archaeological features".into(),
            adjudicated_at: "2026-11-04T17:00:00Z".into(),
            outcome: HumanAdjudicationOutcome::Accepted,
            decision_ref: "urn:synthetic:adjudication:001".into(),
            decision_sha256: "f".repeat(64),
            reviewed_evidence_ids: evidence
                .iter()
                .map(|item| item.evidence_id.clone())
                .collect(),
            rights_status: RightsStatus::Verified,
            rights_receipt_ref: Some("urn:synthetic:rights:reviewer".into()),
            contradictions_resolved: false,
        }
    }

    #[test]
    fn one_remote_sensor_supports_only_a_proxy_interpretation() {
        let input = base_input(
            ClaimKind::ProxyInterpretation,
            vec![evidence(
                "lidar-1",
                EvidenceKind::RemoteSensingSignal,
                EvidenceModality::AirborneLidar,
                "lineage-lidar",
                "acq-lidar",
                "lidar_morphometry",
                "group-lidar",
            )],
        );
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::ProxyInterpretation
        );
        assert_eq!(
            report.disposition,
            ClaimDisposition::QualifiedAtDeclaredClaim
        );
        assert!(!report.archaeological_fact_established);
    }

    #[test]
    fn two_independent_modalities_and_field_evidence_support_feature_candidate_only() {
        let input = base_input(
            ClaimKind::ArchaeologicalFeatureCandidate,
            two_independent_plus_field(),
        );
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::ArchaeologicalFeatureCandidate
        );
        assert_eq!(
            report.disposition,
            ClaimDisposition::QualifiedAtDeclaredClaim
        );
        assert!(!report.human_adjudication_applies);
        assert!(!report.archaeological_fact_established);
    }

    #[test]
    fn archaeological_site_interpretation_requires_exact_set_human_adjudication() {
        let mut input = base_input(
            ClaimKind::ArchaeologicalSiteInterpretation,
            two_independent_plus_field(),
        );
        input.adjudication = Some(accepted_adjudication(&input.evidence));
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::ArchaeologicalSiteInterpretation
        );
        assert_eq!(
            report.disposition,
            ClaimDisposition::QualifiedAtDeclaredClaim
        );
        assert!(report.adjudication_evidence_set_matches);
        assert!(!report.archaeological_fact_established);
    }

    #[test]
    fn stale_human_adjudication_is_invalidated_when_evidence_changes() {
        let mut input = base_input(
            ClaimKind::ArchaeologicalSiteInterpretation,
            two_independent_plus_field(),
        );
        input.adjudication = Some(accepted_adjudication(&input.evidence));
        input.evidence.push(evidence(
            "new-geophysics",
            EvidenceKind::GeophysicalSignal,
            EvidenceModality::GroundGeophysics,
            "lineage-geophysics",
            "acq-geophysics",
            "resistivity",
            "group-geophysics",
        ));
        let report = evaluate_heritage_claim(&input).unwrap();
        assert!(!report.adjudication_evidence_set_matches);
        assert_eq!(report.disposition, ClaimDisposition::Indeterminate);
        assert!(
            report
                .reasons
                .contains(&ClaimReason::HumanAdjudicationEvidenceSetMismatch)
        );
    }

    #[test]
    fn duplicated_source_lineage_cannot_inflate_independence_groups() {
        let mut items = two_independent_plus_field();
        items[1].source_lineage_id = items[0].source_lineage_id.clone();
        let input = base_input(ClaimKind::ArchaeologicalFeatureCandidate, items);
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(report.independent_support_group_count, 1);
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::AnthropogenicFeatureCandidate
        );
        assert_eq!(report.disposition, ClaimDisposition::Overclaimed);
    }

    #[test]
    fn same_acquisition_or_method_cannot_be_counted_as_independent() {
        let mut items = two_independent_plus_field();
        items[1].acquisition_group_id = items[0].acquisition_group_id.clone();
        let input = base_input(ClaimKind::AnthropogenicFeatureCandidate, items.clone());
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(report.independent_support_group_count, 1);

        items = two_independent_plus_field();
        items[1].method_family_id = items[0].method_family_id.clone();
        let input = base_input(ClaimKind::ArchaeologicalFeatureCandidate, items);
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(report.independent_support_group_count, 1);
    }

    #[test]
    fn model_and_simulation_outputs_never_count_as_empirical_confirmation() {
        let mut model = evidence(
            "model-1",
            EvidenceKind::ModelPrediction,
            EvidenceModality::AirborneLidar,
            "model-lineage",
            "model-run",
            "neural_network",
            "model-group",
        );
        model.origin = EvidenceOrigin::ModelOutput;
        let mut simulation = evidence(
            "simulation-1",
            EvidenceKind::Simulation,
            EvidenceModality::Other,
            "simulation-lineage",
            "simulation-run",
            "forward_simulation",
            "simulation-group",
        );
        simulation.origin = EvidenceOrigin::Simulation;
        let input = base_input(
            ClaimKind::AnthropogenicFeatureCandidate,
            vec![model, simulation],
        );
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(report.direct_support_count, 0);
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::NoSupportedClaim
        );
        assert_eq!(report.disposition, ClaimDisposition::Overclaimed);
        assert_eq!(report.model_or_simulation_evidence_count, 2);
    }

    #[test]
    fn catalogue_only_evidence_cannot_support_archaeological_claim() {
        let mut item = evidence(
            "catalogue-1",
            EvidenceKind::CatalogueAssertion,
            EvidenceModality::Other,
            "catalogue-lineage",
            "catalogue-record",
            "catalogue-import",
            "catalogue-group",
        );
        item.origin = EvidenceOrigin::Catalogue;
        item.independence_status = IndependenceStatus::Unknown;
        item.independence_group_id = None;
        item.independence_assessment_sha256 = None;
        let input = base_input(ClaimKind::ArchaeologicalFeatureCandidate, vec![item]);
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::NoSupportedClaim
        );
        assert_eq!(report.disposition, ClaimDisposition::Overclaimed);
    }

    #[test]
    fn unresolved_contradictory_evidence_prevents_higher_claim() {
        let mut items = two_independent_plus_field();
        let mut contrary = evidence(
            "contradictory-geo",
            EvidenceKind::GeophysicalSignal,
            EvidenceModality::GroundGeophysics,
            "lineage-geo",
            "acq-geo",
            "resistivity",
            "group-geo",
        );
        contrary.relation = EvidenceRelation::Contradicts;
        items.push(contrary);
        let input = base_input(ClaimKind::ArchaeologicalFeatureCandidate, items);
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(report.disposition, ClaimDisposition::Contradicted);
        assert_eq!(report.contradictory_evidence_count, 1);
        assert!(
            report
                .reasons
                .contains(&ClaimReason::ContradictoryEvidenceUnresolved)
        );
    }

    #[test]
    fn explicit_adjudication_can_resolve_a_contradiction_but_not_create_evidence() {
        let mut items = two_independent_plus_field();
        let mut contrary = evidence(
            "contradictory-geo",
            EvidenceKind::GeophysicalSignal,
            EvidenceModality::GroundGeophysics,
            "lineage-geo",
            "acq-geo",
            "resistivity",
            "group-geo",
        );
        contrary.relation = EvidenceRelation::Contradicts;
        items.push(contrary);
        let mut input = base_input(ClaimKind::ArchaeologicalSiteInterpretation, items);
        let mut adjudication = accepted_adjudication(&input.evidence);
        adjudication.contradictions_resolved = true;
        input.adjudication = Some(adjudication);
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::ArchaeologicalSiteInterpretation
        );
        assert_eq!(
            report.disposition,
            ClaimDisposition::QualifiedAtDeclaredClaim
        );
    }

    #[test]
    fn historical_process_claim_requires_chronology_and_adjudication() {
        let mut items = two_independent_plus_field();
        items.push(evidence(
            "chronology-1",
            EvidenceKind::ChronometricMeasurement,
            EvidenceModality::ChronometricLaboratory,
            "lineage-chronology",
            "acq-chronology",
            "radiocarbon",
            "group-chronology",
        ));
        let mut input = base_input(ClaimKind::HistoricalProcessInterpretation, items);
        input.adjudication = Some(accepted_adjudication(&input.evidence));
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::HistoricalProcessInterpretation
        );
        assert_eq!(
            report.disposition,
            ClaimDisposition::QualifiedAtDeclaredClaim
        );
        assert!(!report.historical_truth_established);
    }

    #[test]
    fn pending_or_denied_rights_withhold_claim_qualification() {
        let mut items = two_independent_plus_field();
        items[1].rights_status = RightsStatus::Pending;
        items[1].rights_receipt_ref = None;
        let input = base_input(ClaimKind::ArchaeologicalFeatureCandidate, items);
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(report.disposition, ClaimDisposition::Indeterminate);
        assert!(report.rights_blocked_evidence_count > 0);
    }

    #[test]
    fn restricted_coordinate_policy_is_preserved_in_report() {
        let input = base_input(
            ClaimKind::ProxyInterpretation,
            vec![evidence(
                "lidar-1",
                EvidenceKind::RemoteSensingSignal,
                EvidenceModality::AirborneLidar,
                "lineage-lidar",
                "acq-lidar",
                "lidar_morphometry",
                "group-lidar",
            )],
        );
        let report = evaluate_heritage_claim(&input).unwrap();
        assert!(report.coordinate_access_restricted);
        assert!(!report.exact_coordinates_publication_allowed);
    }

    #[test]
    fn duplicate_evidence_id_fails_closed() {
        let mut items = two_independent_plus_field();
        items[1].evidence_id = items[0].evidence_id.clone();
        let input = base_input(ClaimKind::ArchaeologicalFeatureCandidate, items);
        assert!(matches!(
            evaluate_heritage_claim(&input),
            Err(HeritageClaimError::DuplicateEvidenceId(_))
        ));
    }

    #[test]
    fn archaeological_feature_claim_can_use_field_evidence_without_proxy_label() {
        let input = HeritageClaimInput {
            schema_version: HERITAGE_CLAIM_SCHEMA_VERSION.into(),
            claim_id: "claim-field-only".into(),
            candidate_id: "candidate-field-only".into(),
            declared_claim: ClaimKind::ArchaeologicalFeatureCandidate,
            proxy_family_id: None,
            evidence: two_independent_plus_field(),
            adjudication: None,
            coordinate_access_restricted: false,
        };
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(
            report.evidence_supported_ceiling,
            ClaimCeiling::ArchaeologicalFeatureCandidate
        );
        assert_eq!(
            report.disposition,
            ClaimDisposition::QualifiedAtDeclaredClaim
        );
    }

    #[test]
    fn proxy_claim_requires_declared_proxy_family() {
        let mut input = base_input(
            ClaimKind::ProxyInterpretation,
            vec![evidence(
                "lidar-1",
                EvidenceKind::RemoteSensingSignal,
                EvidenceModality::AirborneLidar,
                "lineage-lidar",
                "acq-lidar",
                "lidar_morphometry",
                "group-lidar",
            )],
        );
        input.proxy_family_id = None;
        assert_eq!(
            evaluate_heritage_claim(&input),
            Err(HeritageClaimError::ProxyFamilyRequired)
        );
    }

    #[test]
    fn human_rejection_is_preserved() {
        let mut input = base_input(
            ClaimKind::ArchaeologicalSiteInterpretation,
            two_independent_plus_field(),
        );
        let mut adjudication = accepted_adjudication(&input.evidence);
        adjudication.outcome = HumanAdjudicationOutcome::Rejected;
        input.adjudication = Some(adjudication);
        let report = evaluate_heritage_claim(&input).unwrap();
        assert_eq!(report.disposition, ClaimDisposition::RejectedByAdjudication);
    }

    #[test]
    fn rights_and_evidence_identity_are_not_authenticity_proofs() {
        let input = base_input(
            ClaimKind::ArchaeologicalFeatureCandidate,
            two_independent_plus_field(),
        );
        let report = evaluate_heritage_claim(&input).unwrap();
        assert!(!report.archaeological_fact_established);
        assert!(!report.historical_truth_established);
    }
}
