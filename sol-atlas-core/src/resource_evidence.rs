// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence envelopes for energy-resource claims.
//!
//! A parsed site or resource estimate is not automatically a verified asset,
//! observed output, available MW, deliverable capacity, or an operationally
//! admissible action. This module keeps source lineage, rights, time, spatial
//! support, uncertainty, origin and verification level attached to the claim.
//!
//! A SHA-256 value is an identity field supplied by the source adapter. This
//! module validates its shape; it does not fetch the source, recompute the
//! digest, authenticate the publisher, or verify the engineering truth.

use serde::{Deserialize, Serialize};

pub const RESOURCE_EVIDENCE_SCHEMA_VERSION: &str = "sol-atlas.resource-evidence.v0.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRights {
    PublicDomain,
    LicensedUse,
    Restricted,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOrigin {
    MeterTelemetry,
    RegulatoryRecord,
    EngineeringSurvey,
    ModelEstimate,
    PublicAggregate,
    Simulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationLevel {
    Unreviewed,
    StructurallyValidated,
    CrossSourceCorroborated,
    FieldValidated,
    OperatorAuthorized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialResolution {
    Unknown,
    Global,
    Country,
    AdministrativeArea,
    GridCell,
    Site,
    ExactAsset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQuantity {
    EstimatedPotentialMw,
    RegisteredNameplateMw,
    AvailableCapacityMw,
    MeteredOutputMwh,
    FossilReservesMboe,
    PhysicalEroiRatio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidencePurpose {
    ContextualDisplay,
    PortfolioScreening,
    EngineeringPlanning,
    OperationalEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceClaim {
    RegionalPotential,
    ReportedRegionalCapacity,
    ReportedRegionalReserves,
    EstimatedPotential,
    RegisteredNameplateCapacity,
    ObservedAvailableCapacity,
    ObservedGeneration,
    ReportedFossilReserves,
    PhysicalEroi,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceSource {
    pub source_id: String,
    pub publisher: String,
    /// Stable URI, document ID, API record ID, or rights-cleared archive locator.
    pub locator: String,
    /// SHA-256 of the exact source artifact, supplied by the acquisition adapter.
    pub content_sha256: String,
    pub rights: EvidenceRights,
    /// Required for LicensedUse; should name the applicable licence or agreement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub licence_id: Option<String>,
    pub retrieved_at_unix: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at_unix: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceMeasurement {
    pub quantity: EvidenceQuantity,
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lower_bound: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upper_bound: Option<f64>,
    /// Required for energy totals and physical EROI boundaries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_start_unix: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_end_unix: Option<i64>,
    /// Required to interpret a physical energy-return ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_boundary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceEvidenceEnvelope {
    pub schema_version: String,
    pub evidence_id: String,
    /// Stable Sol-Atlas asset/site identity when the source names one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    pub source: EvidenceSource,
    pub measurement: EvidenceMeasurement,
    pub origin: EvidenceOrigin,
    pub spatial_resolution: SpatialResolution,
    /// Name/code of the supporting site, grid cell, administrative area, etc.
    pub spatial_locator: String,
    pub verification: VerificationLevel,
    /// When a meter, survey or other observation was actually made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at_unix: Option<i64>,
    /// Explicit expiry for time-bounded status or operating evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until_unix: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationPolicy {
    pub purpose: EvidencePurpose,
    pub now_unix: i64,
    /// Optional policy-defined freshness limit; never inferred from a file timestamp.
    pub max_age_seconds: Option<i64>,
    /// When supplied, the evidence must refer to this exact asset identity.
    pub expected_asset_id: Option<String>,
}

impl QualificationPolicy {
    #[must_use]
    pub fn new(purpose: EvidencePurpose, now_unix: i64) -> Self {
        Self {
            purpose,
            now_unix,
            max_age_seconds: None,
            expected_asset_id: None,
        }
    }

    #[must_use]
    pub fn with_max_age_seconds(mut self, maximum: i64) -> Self {
        self.max_age_seconds = Some(maximum);
        self
    }

    #[must_use]
    pub fn for_asset(mut self, asset_id: impl Into<String>) -> Self {
        self.expected_asset_id = Some(asset_id.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceFreshness {
    Current,
    Stale,
    Expired,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBlocker {
    UnsupportedSchemaVersion,
    MissingEvidenceIdentity,
    MissingAssetIdentity,
    MissingSourceIdentity,
    MissingSourceLocator,
    MissingSpatialLocator,
    MalformedSourceDigest,
    RightsNotAdmitted,
    MissingLicence,
    InvalidSourceTime,
    InvalidObservationTime,
    ExpiredEvidence,
    StaleEvidence,
    MissingTemporalReference,
    SpatialSupportTooCoarse,
    AssetIdentityMismatch,
    InvalidQuantity,
    InvalidUncertaintyInterval,
    MissingMeasurementPeriod,
    MissingPhysicalSystemBoundary,
    InsufficientVerification,
    OperationalEvidenceNotMetered,
    OperationalEvidenceNotFieldValidated,
    OperationalEvidenceNotTimeBound,
    NoSupportedClaim,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAssessment {
    pub purpose: EvidencePurpose,
    pub admitted_for_purpose: bool,
    pub freshness: EvidenceFreshness,
    pub allowed_claims: Vec<ResourceClaim>,
    pub blockers: Vec<EvidenceBlocker>,
    /// Always false here: network deliverability is a separate system-level proof.
    pub deliverable_capacity_claim_allowed: bool,
    pub limitations: Vec<&'static str>,
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn uncertainty_is_valid(measurement: &EvidenceMeasurement) -> bool {
    match (measurement.lower_bound, measurement.upper_bound) {
        (None, None) => true,
        (Some(lower), Some(upper)) => {
            lower.is_finite()
                && upper.is_finite()
                && lower <= measurement.value
                && measurement.value <= upper
                && lower <= upper
        }
        _ => false,
    }
}

fn has_asset_spatial_support(resolution: SpatialResolution) -> bool {
    resolution >= SpatialResolution::Site
}

fn freshness_for(
    envelope: &ResourceEvidenceEnvelope,
    policy: &QualificationPolicy,
    blockers: &mut Vec<EvidenceBlocker>,
) -> EvidenceFreshness {
    let temporal_reference = envelope
        .observed_at_unix
        .or(envelope.source.published_at_unix);

    if envelope.source.retrieved_at_unix <= 0
        || envelope.source.retrieved_at_unix > policy.now_unix
        || envelope
            .source
            .published_at_unix
            .is_some_and(|published| published <= 0 || published > envelope.source.retrieved_at_unix)
    {
        blockers.push(EvidenceBlocker::InvalidSourceTime);
    }

    if let Some(observed) = envelope.observed_at_unix {
        if observed <= 0
            || observed > policy.now_unix
            || observed > envelope.source.retrieved_at_unix
        {
            blockers.push(EvidenceBlocker::InvalidObservationTime);
        }
    }

    if let Some(valid_until) = envelope.valid_until_unix {
        if valid_until <= policy.now_unix {
            blockers.push(EvidenceBlocker::ExpiredEvidence);
            return EvidenceFreshness::Expired;
        }
    }

    let Some(reference_time) = temporal_reference else {
        if policy.purpose != EvidencePurpose::ContextualDisplay {
            blockers.push(EvidenceBlocker::MissingTemporalReference);
        }
        return EvidenceFreshness::Unknown;
    };

    if reference_time <= 0 || reference_time > policy.now_unix {
        blockers.push(EvidenceBlocker::InvalidObservationTime);
        return EvidenceFreshness::Unknown;
    }

    if let Some(max_age) = policy.max_age_seconds {
        if max_age < 0 {
            blockers.push(EvidenceBlocker::StaleEvidence);
            return EvidenceFreshness::Stale;
        }
        if policy.now_unix.saturating_sub(reference_time) > max_age {
            blockers.push(EvidenceBlocker::StaleEvidence);
            return EvidenceFreshness::Stale;
        }
    }

    EvidenceFreshness::Current
}

fn claims_supported_by(envelope: &ResourceEvidenceEnvelope) -> Vec<ResourceClaim> {
    let measurement = &envelope.measurement;
    let asset_specific = has_asset_spatial_support(envelope.spatial_resolution);

    match (envelope.origin, measurement.quantity) {
        (
            EvidenceOrigin::ModelEstimate | EvidenceOrigin::Simulation,
            EvidenceQuantity::EstimatedPotentialMw,
        ) => vec![ResourceClaim::EstimatedPotential],
        (EvidenceOrigin::PublicAggregate, EvidenceQuantity::EstimatedPotentialMw) => {
            vec![ResourceClaim::RegionalPotential]
        }
        (EvidenceOrigin::PublicAggregate, EvidenceQuantity::RegisteredNameplateMw) => {
            vec![ResourceClaim::ReportedRegionalCapacity]
        }
        (EvidenceOrigin::PublicAggregate, EvidenceQuantity::FossilReservesMboe) => {
            vec![ResourceClaim::ReportedRegionalReserves]
        }
        (
            EvidenceOrigin::RegulatoryRecord,
            EvidenceQuantity::RegisteredNameplateMw,
        ) if asset_specific
            && envelope.verification >= VerificationLevel::StructurallyValidated =>
        {
            vec![ResourceClaim::RegisteredNameplateCapacity]
        }
        (
            EvidenceOrigin::EngineeringSurvey,
            EvidenceQuantity::EstimatedPotentialMw,
        ) if asset_specific
            && envelope.verification >= VerificationLevel::CrossSourceCorroborated =>
        {
            vec![ResourceClaim::EstimatedPotential]
        }
        (
            EvidenceOrigin::MeterTelemetry,
            EvidenceQuantity::AvailableCapacityMw,
        ) if envelope.verification >= VerificationLevel::FieldValidated =>
        {
            vec![ResourceClaim::ObservedAvailableCapacity]
        }
        (
            EvidenceOrigin::MeterTelemetry,
            EvidenceQuantity::MeteredOutputMwh,
        ) if envelope.verification >= VerificationLevel::FieldValidated =>
        {
            vec![ResourceClaim::ObservedGeneration]
        }
        (
            EvidenceOrigin::RegulatoryRecord | EvidenceOrigin::EngineeringSurvey,
            EvidenceQuantity::FossilReservesMboe,
        ) if asset_specific
            && envelope.verification >= VerificationLevel::StructurallyValidated =>
        {
            vec![ResourceClaim::ReportedFossilReserves]
        }
        (
            EvidenceOrigin::MeterTelemetry | EvidenceOrigin::EngineeringSurvey,
            EvidenceQuantity::PhysicalEroiRatio,
        ) if measurement
            .system_boundary
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
            && measurement.period_start_unix.is_some()
            && measurement.period_end_unix.is_some()
            && envelope.verification >= VerificationLevel::CrossSourceCorroborated =>
        {
            vec![ResourceClaim::PhysicalEroi]
        }
        _ => Vec::new(),
    }
}

/// Qualify one source-bound resource claim for one declared use.
///
/// This is a deterministic policy check. It does not fetch sources, authenticate
/// publishers, recompute source digests, estimate grid deliverability, solve
/// network constraints, or authorize operational actions.
#[must_use]
pub fn assess_resource_evidence(
    envelope: &ResourceEvidenceEnvelope,
    policy: &QualificationPolicy,
) -> EvidenceAssessment {
    let mut blockers = Vec::new();

    if envelope.schema_version != RESOURCE_EVIDENCE_SCHEMA_VERSION {
        blockers.push(EvidenceBlocker::UnsupportedSchemaVersion);
    }
    if envelope.evidence_id.trim().is_empty() {
        blockers.push(EvidenceBlocker::MissingEvidenceIdentity);
    }
    if envelope.source.source_id.trim().is_empty() || envelope.source.publisher.trim().is_empty() {
        blockers.push(EvidenceBlocker::MissingSourceIdentity);
    }
    if envelope.source.locator.trim().is_empty() {
        blockers.push(EvidenceBlocker::MissingSourceLocator);
    }
    if envelope.spatial_locator.trim().is_empty() {
        blockers.push(EvidenceBlocker::MissingSpatialLocator);
    }
    if !valid_sha256(&envelope.source.content_sha256) {
        blockers.push(EvidenceBlocker::MalformedSourceDigest);
    }

    match envelope.source.rights {
        EvidenceRights::PublicDomain => {}
        EvidenceRights::LicensedUse => {
            if envelope
                .source
                .licence_id
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
            {
                blockers.push(EvidenceBlocker::MissingLicence);
            }
        }
        EvidenceRights::Restricted | EvidenceRights::Unknown => {
            blockers.push(EvidenceBlocker::RightsNotAdmitted);
        }
    }

    if !envelope.measurement.value.is_finite()
        || envelope.measurement.value < 0.0
        || (envelope.measurement.quantity == EvidenceQuantity::PhysicalEroiRatio
            && envelope.measurement.value <= 0.0)
    {
        blockers.push(EvidenceBlocker::InvalidQuantity);
    }
    if !uncertainty_is_valid(&envelope.measurement) {
        blockers.push(EvidenceBlocker::InvalidUncertaintyInterval);
    }
    if matches!(
        envelope.measurement.quantity,
        EvidenceQuantity::MeteredOutputMwh | EvidenceQuantity::PhysicalEroiRatio
    ) {
        match (
            envelope.measurement.period_start_unix,
            envelope.measurement.period_end_unix,
        ) {
            (Some(start), Some(end)) if start > 0 && start < end && end <= policy.now_unix => {}
            _ => blockers.push(EvidenceBlocker::MissingMeasurementPeriod),
        }
    }
    if envelope.measurement.quantity == EvidenceQuantity::PhysicalEroiRatio
        && envelope
            .measurement
            .system_boundary
            .as_ref()
            .is_none_or(|boundary| boundary.trim().is_empty())
    {
        blockers.push(EvidenceBlocker::MissingPhysicalSystemBoundary);
    }

    if matches!(
        envelope.spatial_resolution,
        SpatialResolution::ExactAsset | SpatialResolution::Site
    ) && envelope
        .asset_id
        .as_ref()
        .is_none_or(|asset_id| asset_id.trim().is_empty())
    {
        blockers.push(EvidenceBlocker::MissingAssetIdentity);
    }

    if let Some(expected) = &policy.expected_asset_id {
        if envelope.asset_id.as_deref() != Some(expected.as_str()) {
            blockers.push(EvidenceBlocker::AssetIdentityMismatch);
        }
        if !has_asset_spatial_support(envelope.spatial_resolution) {
            blockers.push(EvidenceBlocker::SpatialSupportTooCoarse);
        }
    }

    let freshness = freshness_for(envelope, policy, &mut blockers);

    match policy.purpose {
        EvidencePurpose::ContextualDisplay => {}
        EvidencePurpose::PortfolioScreening => {
            if envelope.verification < VerificationLevel::CrossSourceCorroborated {
                blockers.push(EvidenceBlocker::InsufficientVerification);
            }
        }
        EvidencePurpose::EngineeringPlanning => {
            if !has_asset_spatial_support(envelope.spatial_resolution) {
                blockers.push(EvidenceBlocker::SpatialSupportTooCoarse);
            }
            if envelope.verification < VerificationLevel::CrossSourceCorroborated {
                blockers.push(EvidenceBlocker::InsufficientVerification);
            }
            if envelope.measurement.lower_bound.is_none()
                || envelope.measurement.upper_bound.is_none()
            {
                blockers.push(EvidenceBlocker::InvalidUncertaintyInterval);
            }
        }
        EvidencePurpose::OperationalEvidence => {
            if envelope.spatial_resolution != SpatialResolution::ExactAsset {
                blockers.push(EvidenceBlocker::SpatialSupportTooCoarse);
            }
            if envelope.origin != EvidenceOrigin::MeterTelemetry
                || envelope.measurement.quantity != EvidenceQuantity::AvailableCapacityMw
            {
                blockers.push(EvidenceBlocker::OperationalEvidenceNotMetered);
            }
            if envelope.verification < VerificationLevel::FieldValidated {
                blockers.push(EvidenceBlocker::OperationalEvidenceNotFieldValidated);
            }
            if envelope.observed_at_unix.is_none() || envelope.valid_until_unix.is_none() {
                blockers.push(EvidenceBlocker::OperationalEvidenceNotTimeBound);
            }
        }
    }

    let mut allowed_claims = if blockers.is_empty() {
        claims_supported_by(envelope)
    } else {
        Vec::new()
    };

    if allowed_claims.is_empty() && blockers.is_empty() {
        blockers.push(EvidenceBlocker::NoSupportedClaim);
    }
    if policy.purpose == EvidencePurpose::OperationalEvidence
        && !allowed_claims.contains(&ResourceClaim::ObservedAvailableCapacity)
        && blockers.is_empty()
    {
        blockers.push(EvidenceBlocker::OperationalEvidenceNotFieldValidated);
    }
    if !blockers.is_empty() {
        allowed_claims.clear();
    }

    EvidenceAssessment {
        purpose: policy.purpose,
        admitted_for_purpose: blockers.is_empty() && !allowed_claims.is_empty(),
        freshness,
        allowed_claims,
        blockers,
        deliverable_capacity_claim_allowed: false,
        limitations: vec![
            "content_sha256_is_a_source_identity_claim_not_authenticity_proof",
            "resource_evidence_does_not_prove_interconnection_or_network_deliverability",
            "model_estimates_are_not_observations_or_nameplate_records",
            "operational_evidence_still_requires_separate_network_protection_and_operator_qualification",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_envelope() -> ResourceEvidenceEnvelope {
        ResourceEvidenceEnvelope {
            schema_version: RESOURCE_EVIDENCE_SCHEMA_VERSION.to_owned(),
            evidence_id: "ev-001".to_owned(),
            asset_id: Some("asset-001".to_owned()),
            source: EvidenceSource {
                source_id: "registry-record-7".to_owned(),
                publisher: "Independent registry".to_owned(),
                locator: "registry://records/7".to_owned(),
                content_sha256: "a".repeat(64),
                rights: EvidenceRights::LicensedUse,
                licence_id: Some("CC-BY-4.0".to_owned()),
                retrieved_at_unix: 1_700_000_000,
                published_at_unix: Some(1_699_000_000),
            },
            measurement: EvidenceMeasurement {
                quantity: EvidenceQuantity::RegisteredNameplateMw,
                value: 100.0,
                lower_bound: Some(99.0),
                upper_bound: Some(101.0),
                period_start_unix: None,
                period_end_unix: None,
                system_boundary: None,
            },
            origin: EvidenceOrigin::RegulatoryRecord,
            spatial_resolution: SpatialResolution::ExactAsset,
            spatial_locator: "asset-001/site-footprint".to_owned(),
            verification: VerificationLevel::StructurallyValidated,
            observed_at_unix: None,
            valid_until_unix: None,
        }
    }

    #[test]
    fn registry_record_can_support_nameplate_not_observed_or_deliverable_capacity() {
        let assessment = assess_resource_evidence(
            &base_envelope(),
            &QualificationPolicy::new(EvidencePurpose::EngineeringPlanning, 1_700_100_000)
                .for_asset("asset-001"),
        );
        assert!(assessment.admitted_for_purpose);
        assert_eq!(
            assessment.allowed_claims,
            vec![ResourceClaim::RegisteredNameplateCapacity]
        );
        assert!(!assessment.deliverable_capacity_claim_allowed);
        assert!(!assessment.allowed_claims.contains(&ResourceClaim::ObservedAvailableCapacity));
    }

    #[test]
    fn regional_model_estimate_never_becomes_asset_nameplate_or_observed_output() {
        let mut envelope = base_envelope();
        envelope.asset_id = None;
        envelope.origin = EvidenceOrigin::ModelEstimate;
        envelope.spatial_resolution = SpatialResolution::AdministrativeArea;
        envelope.spatial_locator = "ZA-WC".to_owned();
        envelope.measurement.quantity = EvidenceQuantity::EstimatedPotentialMw;
        envelope.verification = VerificationLevel::CrossSourceCorroborated;
        let assessment = assess_resource_evidence(
            &envelope,
            &QualificationPolicy::new(EvidencePurpose::PortfolioScreening, 1_700_100_000),
        );
        assert!(assessment.admitted_for_purpose);
        assert_eq!(
            assessment.allowed_claims,
            vec![ResourceClaim::EstimatedPotential]
        );
        assert!(!assessment.allowed_claims.contains(&ResourceClaim::RegisteredNameplateCapacity));
        assert!(!assessment.allowed_claims.contains(&ResourceClaim::ObservedGeneration));
    }

    #[test]
    fn coarse_regional_evidence_cannot_be_bound_to_a_specific_asset() {
        let mut envelope = base_envelope();
        envelope.origin = EvidenceOrigin::PublicAggregate;
        envelope.spatial_resolution = SpatialResolution::Country;
        envelope.spatial_locator = "ZA".to_owned();
        let assessment = assess_resource_evidence(
            &envelope,
            &QualificationPolicy::new(EvidencePurpose::EngineeringPlanning, 1_700_100_000)
                .for_asset("asset-001"),
        );
        assert!(!assessment.admitted_for_purpose);
        assert!(assessment.blockers.contains(&EvidenceBlocker::SpatialSupportTooCoarse));
    }

    #[test]
    fn operational_evidence_requires_fresh_metered_asset_specific_field_validation() {
        let mut envelope = base_envelope();
        envelope.source.source_id = "meter-asset-001".to_owned();
        envelope.source.publisher = "Grid operator".to_owned();
        envelope.measurement.quantity = EvidenceQuantity::AvailableCapacityMw;
        envelope.origin = EvidenceOrigin::MeterTelemetry;
        envelope.verification = VerificationLevel::FieldValidated;
        envelope.observed_at_unix = Some(1_700_099_000);
        envelope.valid_until_unix = Some(1_700_160_000);
        envelope.measurement.lower_bound = Some(90.0);
        envelope.measurement.upper_bound = Some(100.0);
        let assessment = assess_resource_evidence(
            &envelope,
            &QualificationPolicy::new(EvidencePurpose::OperationalEvidence, 1_700_100_000)
                .with_max_age_seconds(3_600)
                .for_asset("asset-001"),
        );
        assert!(assessment.admitted_for_purpose);
        assert_eq!(
            assessment.allowed_claims,
            vec![ResourceClaim::ObservedAvailableCapacity]
        );
        assert!(!assessment.deliverable_capacity_claim_allowed);
    }

    #[test]
    fn unknown_or_restricted_rights_block_even_contextual_display() {
        let mut envelope = base_envelope();
        envelope.source.rights = EvidenceRights::Unknown;
        let assessment = assess_resource_evidence(
            &envelope,
            &QualificationPolicy::new(EvidencePurpose::ContextualDisplay, 1_700_100_000),
        );
        assert!(!assessment.admitted_for_purpose);
        assert!(assessment.blockers.contains(&EvidenceBlocker::RightsNotAdmitted));
    }

    #[test]
    fn source_digest_is_shape_checked_but_not_claimed_as_authenticity() {
        let mut envelope = base_envelope();
        envelope.source.content_sha256 = "not-a-digest".to_owned();
        let assessment = assess_resource_evidence(
            &envelope,
            &QualificationPolicy::new(EvidencePurpose::ContextualDisplay, 1_700_100_000),
        );
        assert!(assessment.blockers.contains(&EvidenceBlocker::MalformedSourceDigest));
        assert!(assessment.limitations.iter().any(|value| value.contains("authenticity")));
    }

    #[test]
    fn stale_and_expired_evidence_do_not_pass_planning_or_operational_gates() {
        let mut envelope = base_envelope();
        let stale = assess_resource_evidence(
            &envelope,
            &QualificationPolicy::new(EvidencePurpose::PortfolioScreening, 1_800_000_000)
                .with_max_age_seconds(86_400),
        );
        assert!(!stale.admitted_for_purpose);
        assert!(stale.blockers.contains(&EvidenceBlocker::StaleEvidence));

        envelope.valid_until_unix = Some(1_700_000_001);
        let expired = assess_resource_evidence(
            &envelope,
            &QualificationPolicy::new(EvidencePurpose::ContextualDisplay, 1_700_000_002),
        );
        assert!(!expired.admitted_for_purpose);
        assert!(expired.blockers.contains(&EvidenceBlocker::ExpiredEvidence));
    }

    #[test]
    fn physical_eroi_requires_a_declared_energy_boundary_and_time_period() {
        let mut envelope = base_envelope();
        envelope.measurement.quantity = EvidenceQuantity::PhysicalEroiRatio;
        envelope.measurement.value = 8.0;
        envelope.measurement.lower_bound = Some(7.5);
        envelope.measurement.upper_bound = Some(8.5);
        envelope.origin = EvidenceOrigin::EngineeringSurvey;
        envelope.verification = VerificationLevel::CrossSourceCorroborated;
        envelope.measurement.system_boundary = Some("mine-mouth-to-exported-electricity".to_owned());
        envelope.measurement.period_start_unix = Some(1_600_000_000);
        envelope.measurement.period_end_unix = Some(1_699_000_000);
        let assessment = assess_resource_evidence(
            &envelope,
            &QualificationPolicy::new(EvidencePurpose::EngineeringPlanning, 1_700_100_000)
                .for_asset("asset-001"),
        );
        assert!(assessment.admitted_for_purpose);
        assert_eq!(assessment.allowed_claims, vec![ResourceClaim::PhysicalEroi]);
    }
}
