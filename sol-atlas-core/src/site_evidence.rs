// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Source provenance, policy-specific freshness, rights and spatial-coverage checks.
//!
//! This module does not verify the underlying truth of a dataset or perform GIS
//! operations. It checks whether supplied source metadata meets a caller-defined
//! evidence policy. A missing or non-covering layer is not evidence of absence.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAuthority {
    GovernmentRecord,
    LicensedProfessionalSurvey,
    PeerReviewedModel,
    RemoteSensingProvider,
    CommunityObservation,
    CommercialProvider,
    Unverified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataRights {
    PublicDomain,
    OpenLicense,
    Restricted,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDomain {
    LandRights,
    PlanningPermission,
    EnvironmentalProtection,
    Hazard,
    Geotechnical,
    Utilities,
    TransitAccess,
    CommunityContext,
    GeneralSpatial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisOrder {
    Xy,
    LonLat,
    LatLon,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Extent2D {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpatialMetadata {
    /// CRS identifier such as an EPSG code or a full CRS URI.
    pub crs: String,
    pub axis_order: AxisOrder,
    pub horizontal_unit: String,
    /// Bounds are expressed in this spatial record's declared CRS and axis order.
    pub bounds: Option<Extent2D>,
    /// Approximate ground resolution in metres, if meaningful for the source.
    pub resolution_meters: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub source_id: String,
    pub publisher: String,
    pub dataset_name: String,
    pub revision: String,
    pub fact_domain: EvidenceDomain,
    pub jurisdiction: Option<String>,
    /// SHA-256 of the exact ingested payload, when available. Format is checked here;
    /// bytes must be hashed by the ingestion layer.
    pub payload_sha256: Option<String>,
    pub authority: SourceAuthority,
    pub rights: DataRights,
    #[serde(default)]
    pub license_id: Option<String>,
    pub retrieved_at_unix_seconds: i64,
    /// Source data's own observation/publication/effective time. Not retrieval time.
    pub data_as_of_unix_seconds: Option<i64>,
    pub valid_until_unix_seconds: Option<i64>,
    pub spatial: Option<SpatialMetadata>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidencePolicy {
    pub evaluation_time_unix_seconds: i64,
    pub required_fact_domain: EvidenceDomain,
    pub required_jurisdiction: Option<String>,
    /// Policy-specific maximum age for source data. None means the caller accepts no
    /// automatic age cutoff; source-specific authority rules still apply.
    pub maximum_data_age_seconds: Option<i64>,
    pub require_data_timestamp: bool,
    pub require_payload_digest: bool,
    pub accepted_authorities: BTreeSet<SourceAuthority>,
    pub accepted_rights: BTreeSet<DataRights>,
    pub require_spatial_metadata: bool,
    pub required_crs: Option<String>,
    pub required_axis_order: Option<AxisOrder>,
    pub required_extent_crs: Option<String>,
    pub required_extent: Option<Extent2D>,
    /// Maximum acceptable ground resolution. A smaller value is more detailed.
    pub maximum_resolution_meters: Option<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceReadiness {
    Ready,
    Stale,
    Expired,
    FutureDated,
    UnknownFreshness,
    MissingDigest,
    RightsUnknown,
    RightsNotAllowed,
    LicenseIdentifierMissing,
    FactDomainMismatch,
    JurisdictionUnknown,
    JurisdictionMismatch,
    AuthorityNotAccepted,
    SpatialMetadataMissing,
    SpatialMetadataInvalid,
    CrsMismatch,
    AxisOrderUnknown,
    AxisOrderMismatch,
    CoverageUnknown,
    CoverageGap,
    ResolutionUnknown,
    ResolutionInsufficient,
    InvalidMetadata,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceAssessment {
    pub source_id: String,
    pub readiness: EvidenceReadiness,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvidencePolicyError {
    InvalidEvaluationTime(i64),
    InvalidMaximumAge(i64),
    EmptyAuthorityPolicy,
    EmptyRightsPolicy,
    EmptyRequiredCrs,
    EmptyRequiredExtentCrs,
    EmptyRequiredJurisdiction,
    MissingRequiredExtentCrs,
    ConflictingRequiredCrs,
    EmptyRequiredAxisPolicy,
    InvalidMaximumResolution(f64),
    InvalidRequiredExtent,
}

impl fmt::Display for EvidencePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEvaluationTime(value) => {
                write!(f, "evaluation timestamp must be non-negative, got {value}")
            }
            Self::InvalidMaximumAge(value) => {
                write!(f, "maximum data age must be non-negative, got {value}")
            }
            Self::EmptyAuthorityPolicy => {
                write!(
                    f,
                    "policy must name at least one accepted source-authority class"
                )
            }
            Self::EmptyRightsPolicy => {
                write!(
                    f,
                    "policy must name at least one accepted data-rights class"
                )
            }
            Self::EmptyRequiredCrs => write!(f, "required CRS must not be empty"),
            Self::EmptyRequiredExtentCrs => {
                write!(f, "required extent CRS must not be empty")
            }
            Self::EmptyRequiredJurisdiction => {
                write!(f, "required jurisdiction must not be empty")
            }
            Self::MissingRequiredExtentCrs => {
                write!(
                    f,
                    "required extent must declare its CRS or inherit an explicit required CRS"
                )
            }
            Self::ConflictingRequiredCrs => {
                write!(f, "required CRS and required extent CRS must agree")
            }
            Self::EmptyRequiredAxisPolicy => {
                write!(f, "required axis order must not be Unknown")
            }
            Self::InvalidMaximumResolution(value) => {
                write!(
                    f,
                    "maximum resolution must be finite and positive, got {value}"
                )
            }
            Self::InvalidRequiredExtent => {
                write!(f, "required extent must contain finite, ordered bounds")
            }
        }
    }
}

impl std::error::Error for EvidencePolicyError {}

fn valid_extent(extent: Extent2D) -> bool {
    [extent.min_x, extent.min_y, extent.max_x, extent.max_y]
        .into_iter()
        .all(f64::is_finite)
        && extent.min_x <= extent.max_x
        && extent.min_y <= extent.max_y
}

fn validate_policy(policy: &EvidencePolicy) -> Result<(), EvidencePolicyError> {
    if policy.evaluation_time_unix_seconds < 0 {
        return Err(EvidencePolicyError::InvalidEvaluationTime(
            policy.evaluation_time_unix_seconds,
        ));
    }
    if let Some(age) = policy.maximum_data_age_seconds {
        if age < 0 {
            return Err(EvidencePolicyError::InvalidMaximumAge(age));
        }
    }
    if policy
        .required_jurisdiction
        .as_ref()
        .is_some_and(|jurisdiction| jurisdiction.trim().is_empty())
    {
        return Err(EvidencePolicyError::EmptyRequiredJurisdiction);
    }
    if policy.accepted_authorities.is_empty() {
        return Err(EvidencePolicyError::EmptyAuthorityPolicy);
    }
    if policy.accepted_rights.is_empty() {
        return Err(EvidencePolicyError::EmptyRightsPolicy);
    }
    if policy
        .required_crs
        .as_ref()
        .is_some_and(|crs| crs.trim().is_empty())
    {
        return Err(EvidencePolicyError::EmptyRequiredCrs);
    }
    if policy
        .required_extent_crs
        .as_ref()
        .is_some_and(|crs| crs.trim().is_empty())
    {
        return Err(EvidencePolicyError::EmptyRequiredExtentCrs);
    }
    if policy.required_axis_order == Some(AxisOrder::Unknown) {
        return Err(EvidencePolicyError::EmptyRequiredAxisPolicy);
    }
    if let Some(resolution) = policy.maximum_resolution_meters {
        if !resolution.is_finite() || resolution <= 0.0 {
            return Err(EvidencePolicyError::InvalidMaximumResolution(resolution));
        }
    }
    if policy
        .required_extent
        .is_some_and(|extent| !valid_extent(extent))
    {
        return Err(EvidencePolicyError::InvalidRequiredExtent);
    }
    if policy.required_extent.is_some()
        && policy.required_extent_crs.is_none()
        && policy.required_crs.is_none()
    {
        return Err(EvidencePolicyError::MissingRequiredExtentCrs);
    }
    if let (Some(crs), Some(extent_crs)) = (&policy.required_crs, &policy.required_extent_crs) {
        if crs != extent_crs {
            return Err(EvidencePolicyError::ConflictingRequiredCrs);
        }
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn note_failure(
    assessment: &mut SourceAssessment,
    readiness: EvidenceReadiness,
    reason: impl Into<String>,
) {
    if assessment.readiness == EvidenceReadiness::Ready {
        assessment.readiness = readiness;
    }
    assessment.reasons.push(reason.into());
}

fn assess_spatial(
    assessment: &mut SourceAssessment,
    spatial: Option<&SpatialMetadata>,
    policy: &EvidencePolicy,
) {
    let spatial_required = policy.require_spatial_metadata
        || policy.required_crs.is_some()
        || policy.required_axis_order.is_some()
        || policy.required_extent.is_some()
        || policy.maximum_resolution_meters.is_some();

    let Some(spatial) = spatial else {
        if spatial_required {
            note_failure(
                assessment,
                EvidenceReadiness::SpatialMetadataMissing,
                "required spatial metadata is missing",
            );
        }
        return;
    };

    if spatial.crs.trim().is_empty()
        || spatial.horizontal_unit.trim().is_empty()
        || spatial.bounds.is_some_and(|extent| !valid_extent(extent))
        || spatial
            .resolution_meters
            .is_some_and(|resolution| !resolution.is_finite() || resolution <= 0.0)
    {
        note_failure(
            assessment,
            EvidenceReadiness::SpatialMetadataInvalid,
            "spatial metadata contains empty identifiers or invalid bounds/units/resolution",
        );
    }

    if spatial.axis_order == AxisOrder::Unknown && spatial_required {
        note_failure(
            assessment,
            EvidenceReadiness::AxisOrderUnknown,
            "spatial axis order is unknown",
        );
    }
    if let Some(required_crs) = &policy.required_crs {
        if &spatial.crs != required_crs {
            note_failure(
                assessment,
                EvidenceReadiness::CrsMismatch,
                format!(
                    "source CRS {} does not match required CRS {required_crs}",
                    spatial.crs
                ),
            );
        }
    }
    if let Some(required_axis) = policy.required_axis_order {
        if spatial.axis_order != required_axis {
            note_failure(
                assessment,
                EvidenceReadiness::AxisOrderMismatch,
                format!(
                    "source axis order {:?} does not match required axis order {:?}",
                    spatial.axis_order, required_axis
                ),
            );
        }
    }

    if let Some(required_extent) = policy.required_extent {
        let required_crs = policy
            .required_extent_crs
            .as_ref()
            .or(policy.required_crs.as_ref());
        if let Some(required_crs) = required_crs {
            if &spatial.crs != required_crs {
                note_failure(
                    assessment,
                    EvidenceReadiness::CrsMismatch,
                    "source extent cannot be compared because its CRS differs from the requested extent CRS",
                );
                return;
            }
        }
        let Some(source_extent) = spatial.bounds else {
            note_failure(
                assessment,
                EvidenceReadiness::CoverageUnknown,
                "source does not declare spatial coverage bounds",
            );
            return;
        };
        if source_extent.min_x > required_extent.min_x
            || source_extent.min_y > required_extent.min_y
            || source_extent.max_x < required_extent.max_x
            || source_extent.max_y < required_extent.max_y
        {
            note_failure(
                assessment,
                EvidenceReadiness::CoverageGap,
                "declared source bounds do not fully cover the requested extent",
            );
        }
    }

    if let Some(maximum) = policy.maximum_resolution_meters {
        match spatial.resolution_meters {
            None => note_failure(
                assessment,
                EvidenceReadiness::ResolutionUnknown,
                "required ground resolution is not declared",
            ),
            Some(resolution) if resolution > maximum => note_failure(
                assessment,
                EvidenceReadiness::ResolutionInsufficient,
                format!(
                    "source resolution {resolution} m is coarser than the required maximum {maximum} m"
                ),
            ),
            Some(_) => {}
        }
    }
}

/// Assess whether source metadata is admissible for a specific declared use.
///
/// This function does not interpret the source's content. A Ready result only means
/// its metadata meets this policy: it is not proof of land rights, absence of hazards,
/// ecological clearance, planning permission, or engineering safety.
pub fn assess_source(
    source: &SourceRecord,
    policy: &EvidencePolicy,
) -> Result<SourceAssessment, EvidencePolicyError> {
    validate_policy(policy)?;

    let mut result = SourceAssessment {
        source_id: source.source_id.clone(),
        readiness: EvidenceReadiness::Ready,
        reasons: Vec::new(),
    };

    if source.source_id.trim().is_empty()
        || source.publisher.trim().is_empty()
        || source.dataset_name.trim().is_empty()
        || source.revision.trim().is_empty()
        || source
            .jurisdiction
            .as_ref()
            .is_some_and(|jurisdiction| jurisdiction.trim().is_empty())
        || source
            .payload_sha256
            .as_ref()
            .is_some_and(|digest| !valid_sha256(digest))
        || source.valid_until_unix_seconds.is_some_and(|expiry| {
            source
                .data_as_of_unix_seconds
                .is_some_and(|as_of| expiry < as_of)
        })
        || source.spatial.as_ref().is_some_and(|spatial| {
            spatial.crs.trim().is_empty()
                || spatial.horizontal_unit.trim().is_empty()
                || spatial.bounds.is_some_and(|bounds| !valid_extent(bounds))
                || spatial
                    .resolution_meters
                    .is_some_and(|resolution| !resolution.is_finite() || resolution <= 0.0)
        })
    {
        note_failure(
            &mut result,
            EvidenceReadiness::InvalidMetadata,
            "source record has missing required identifiers or malformed metadata",
        );
    }

    if source.retrieved_at_unix_seconds > policy.evaluation_time_unix_seconds
        || source
            .data_as_of_unix_seconds
            .is_some_and(|as_of| as_of > policy.evaluation_time_unix_seconds)
    {
        note_failure(
            &mut result,
            EvidenceReadiness::FutureDated,
            "source retrieval/data timestamp is after the policy evaluation time",
        );
    }

    if source
        .valid_until_unix_seconds
        .is_some_and(|expiry| expiry < policy.evaluation_time_unix_seconds)
    {
        note_failure(
            &mut result,
            EvidenceReadiness::Expired,
            "source is past its declared validity/expiry time",
        );
    }

    match source.data_as_of_unix_seconds {
        None if policy.require_data_timestamp => note_failure(
            &mut result,
            EvidenceReadiness::UnknownFreshness,
            "source data timestamp is missing; freshness cannot be established",
        ),
        Some(as_of) => {
            if let Some(max_age) = policy.maximum_data_age_seconds {
                let age = policy.evaluation_time_unix_seconds.saturating_sub(as_of);
                if age > max_age {
                    note_failure(
                        &mut result,
                        EvidenceReadiness::Stale,
                        format!("source data age {age} s exceeds policy maximum {max_age} s"),
                    );
                }
            }
        }
        None => {}
    }

    if policy.require_payload_digest && source.payload_sha256.is_none() {
        note_failure(
            &mut result,
            EvidenceReadiness::MissingDigest,
            "policy requires a digest of the exact source payload",
        );
    }

    if source.rights == DataRights::Unknown {
        note_failure(
            &mut result,
            EvidenceReadiness::RightsUnknown,
            "source data rights are unknown",
        );
    } else if !policy.accepted_rights.contains(&source.rights) {
        note_failure(
            &mut result,
            EvidenceReadiness::RightsNotAllowed,
            format!(
                "source rights {:?} are not allowed by this policy",
                source.rights
            ),
        );
    }

    if matches!(
        source.rights,
        DataRights::OpenLicense | DataRights::Restricted
    ) && source
        .license_id
        .as_ref()
        .is_none_or(|license| license.trim().is_empty())
    {
        note_failure(
            &mut result,
            EvidenceReadiness::LicenseIdentifierMissing,
            "open or restricted rights class has no license/terms identifier",
        );
    }

    if source.fact_domain != policy.required_fact_domain {
        note_failure(
            &mut result,
            EvidenceReadiness::FactDomainMismatch,
            format!(
                "source fact domain {:?} does not match required domain {:?}",
                source.fact_domain, policy.required_fact_domain
            ),
        );
    }

    if let Some(required_jurisdiction) = &policy.required_jurisdiction {
        match &source.jurisdiction {
            None => note_failure(
                &mut result,
                EvidenceReadiness::JurisdictionUnknown,
                "policy requires jurisdictional source evidence but the source jurisdiction is unknown",
            ),
            Some(jurisdiction) if jurisdiction != required_jurisdiction => note_failure(
                &mut result,
                EvidenceReadiness::JurisdictionMismatch,
                format!(
                    "source jurisdiction {jurisdiction} does not match required jurisdiction {required_jurisdiction}"
                ),
            ),
            Some(_) => {}
        }
    }

    if source.authority == SourceAuthority::Unverified
        || !policy.accepted_authorities.contains(&source.authority)
    {
        note_failure(
            &mut result,
            EvidenceReadiness::AuthorityNotAccepted,
            format!(
                "source authority {:?} is not accepted for this fact type",
                source.authority
            ),
        );
    }

    assess_spatial(&mut result, source.spatial.as_ref(), policy);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn policy() -> EvidencePolicy {
        EvidencePolicy {
            evaluation_time_unix_seconds: 1_792_195_200,
            required_fact_domain: EvidenceDomain::PlanningPermission,
            required_jurisdiction: Some("ZA-GP-JHB".into()),
            maximum_data_age_seconds: Some(90 * 24 * 60 * 60),
            require_data_timestamp: true,
            require_payload_digest: true,
            accepted_authorities: [SourceAuthority::GovernmentRecord].into_iter().collect(),
            accepted_rights: [DataRights::PublicDomain, DataRights::OpenLicense]
                .into_iter()
                .collect(),
            require_spatial_metadata: true,
            required_crs: Some("EPSG:4326".into()),
            required_axis_order: Some(AxisOrder::LonLat),
            required_extent_crs: Some("EPSG:4326".into()),
            required_extent: Some(Extent2D {
                min_x: 10.0,
                min_y: 20.0,
                max_x: 30.0,
                max_y: 40.0,
            }),
            maximum_resolution_meters: Some(100.0),
        }
    }

    fn source(id: &str) -> SourceRecord {
        SourceRecord {
            source_id: id.into(),
            publisher: "Synthetic authority".into(),
            dataset_name: "Synthetic planning data".into(),
            revision: "fixture-1".into(),
            fact_domain: EvidenceDomain::PlanningPermission,
            jurisdiction: Some("ZA-GP-JHB".into()),
            payload_sha256: Some("a".repeat(64)),
            authority: SourceAuthority::GovernmentRecord,
            rights: DataRights::OpenLicense,
            license_id: Some("CC-BY-4.0".into()),
            retrieved_at_unix_seconds: 1_792_195_200,
            data_as_of_unix_seconds: Some(1_792_195_200),
            valid_until_unix_seconds: None,
            spatial: Some(SpatialMetadata {
                crs: "EPSG:4326".into(),
                axis_order: AxisOrder::LonLat,
                horizontal_unit: "degree".into(),
                bounds: Some(Extent2D {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 50.0,
                    max_y: 50.0,
                }),
                resolution_meters: Some(30.0),
            }),
        }
    }

    #[test]
    fn matching_fresh_source_is_ready_for_this_policy() {
        let result = assess_source(&source("fresh"), &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::Ready);
        assert!(result.reasons.is_empty());
    }

    #[test]
    fn stale_data_is_not_made_usable_by_recent_retrieval() {
        let mut item = source("stale");
        item.data_as_of_unix_seconds = Some(1_700_000_000);
        item.retrieved_at_unix_seconds = 1_792_195_200;
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::Stale);
    }

    #[test]
    fn unknown_timestamp_cannot_pass_required_freshness() {
        let mut item = source("unknown-date");
        item.data_as_of_unix_seconds = None;
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::UnknownFreshness);
    }

    #[test]
    fn unknown_or_restricted_rights_do_not_pass_open_data_policy() {
        let mut item = source("unknown-rights");
        item.rights = DataRights::Unknown;
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::RightsUnknown);

        item.rights = DataRights::Restricted;
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::RightsNotAllowed);
    }

    #[test]
    fn crs_mismatch_cannot_be_silently_compared() {
        let mut item = source("wrong-crs");
        item.spatial.as_mut().unwrap().crs = "EPSG:3857".into();
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::CrsMismatch);
    }

    #[test]
    fn declared_coverage_gap_is_not_treated_as_no_constraint() {
        let mut item = source("partial-coverage");
        item.spatial.as_mut().unwrap().bounds = Some(Extent2D {
            min_x: 15.0,
            min_y: 25.0,
            max_x: 25.0,
            max_y: 35.0,
        });
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::CoverageGap);
    }

    #[test]
    fn expired_sources_are_held() {
        let mut item = source("expired");
        item.valid_until_unix_seconds = Some(1_792_195_199);
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::Expired);
    }

    #[test]
    fn invalid_digest_or_axis_order_is_not_accepted() {
        let mut item = source("invalid-digest");
        item.payload_sha256 = Some("not-a-digest".into());
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::InvalidMetadata);

        item = source("unknown-axis");
        item.spatial.as_mut().unwrap().axis_order = AxisOrder::Unknown;
        let result = assess_source(&item, &policy()).unwrap();
        assert_eq!(result.readiness, EvidenceReadiness::AxisOrderUnknown);
    }

    #[derive(Deserialize)]
    struct BenchmarkCase {
        case_id: String,
        source: SourceRecord,
        policy: EvidencePolicy,
        expected: EvidenceReadiness,
    }

    #[test]
    fn synthetic_source_benchmark_matches_expected_readiness() {
        let fixture =
            include_str!("../../docs/research/fixtures/site-evidence-benchmark-v0.1.jsonl");
        let mut count = 0;
        for line in fixture.lines().filter(|line| !line.trim().is_empty()) {
            let case: BenchmarkCase =
                serde_json::from_str(line).expect("valid source evidence JSONL");
            let result = assess_source(&case.source, &case.policy)
                .unwrap_or_else(|error| panic!("{}: {error}", case.case_id));
            assert_eq!(result.readiness, case.expected, "{}", case.case_id);
            count += 1;
        }
        assert!(count >= 8, "benchmark must cover at least eight cases");
    }
}
