// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rights-aware source metadata, immutable in-memory asset revisions, and
//! deterministic byte-verification receipts for Heritage Intelligence.
//!
//! This module deliberately does not fetch remote assets, validate source
//! licences on behalf of their issuers, parse raster/point-cloud payloads, or
//! establish archaeological truth. Callers must authorize each intended use
//! separately and run heavy geospatial processing in a pinned adapter.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const SOURCE_ASSET_SCHEMA_VERSION: &str = "luminous.heritage-source-asset.v0.1";
pub const INGESTION_RECEIPT_SCHEMA_VERSION: &str = "luminous.heritage-ingestion-receipt.v0.1";

/// SHA-256 of the exact bytes supplied by the caller, encoded as lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

/// Validate the scheme component of an absolute URI (RFC 3986 §3.1).
/// This validates syntax only; it does not test reachability or dereference the URI.
fn has_uri_scheme(value: &str) -> bool {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
}

/// SHA-256 digests are 32 bytes encoded as exactly 64 hexadecimal characters.
fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsState {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeritageUse {
    View,
    Research,
    Training,
    Evaluation,
    Publication,
    Redistribution,
    Commercial,
}

/// Increasing values represent increasingly privileged access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessTier {
    Public,
    Registered,
    ApprovedResearch,
    Custodial,
    Restricted,
    Withheld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExactGeometryPolicy {
    Restricted,
    CustodialApprovalRequired,
    ApprovedResearchOnly,
    PublicApproved,
    Withheld,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RightsProfile {
    pub state: RightsState,
    pub permitted_uses: BTreeSet<HeritageUse>,
    pub access_tier: AccessTier,
    pub exact_geometry_policy: ExactGeometryPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licence_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_or_authority_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_due_at: Option<String>,
    #[serde(default)]
    pub restrictions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessContext {
    /// Highest access tier explicitly granted to the current caller.
    pub max_tier: AccessTier,
    /// Explicit approval for access to exact rather than generalised geometry.
    pub exact_geometry_authorized: bool,
}

impl AccessContext {
    pub const fn public() -> Self {
        Self {
            max_tier: AccessTier::Public,
            exact_geometry_authorized: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightsDenial {
    RightsNotVerified(RightsState),
    PurposeNotPermitted(HeritageUse),
    AccessTierInsufficient {
        required: AccessTier,
        granted: AccessTier,
    },
    ExactGeometryNotAuthorized,
    ExactGeometryTierInsufficient {
        required: AccessTier,
        granted: AccessTier,
    },
    ExactGeometryWithheld,
    WithheldAsset,
    InvalidRightsProfile,
}

impl fmt::Display for RightsDenial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RightsNotVerified(state) => {
                write!(f, "source rights are not verified: {state:?}")
            }
            Self::PurposeNotPermitted(purpose) => {
                write!(f, "source terms do not permit this use: {purpose:?}")
            }
            Self::AccessTierInsufficient { required, granted } => {
                write!(f, "access tier {granted:?} does not satisfy {required:?}")
            }
            Self::ExactGeometryNotAuthorized => {
                write!(f, "exact-geometry access requires explicit authorization")
            }
            Self::ExactGeometryTierInsufficient { required, granted } => {
                write!(
                    f,
                    "exact-geometry tier {granted:?} does not satisfy {required:?}"
                )
            }
            Self::ExactGeometryWithheld => write!(f, "exact geometry is withheld"),
            Self::WithheldAsset => write!(f, "asset access is withheld"),
            Self::InvalidRightsProfile => write!(f, "rights profile is internally inconsistent"),
        }
    }
}

impl std::error::Error for RightsDenial {}

impl RightsProfile {
    /// Fail-closed authorization for one specific use and access context.
    ///
    /// Licence/use permission, access tier, and exact-coordinate authorization
    /// are separate checks. Passing this function is not a permit to visit,
    /// survey, excavate, or disturb a place.
    pub fn authorize(
        &self,
        purpose: HeritageUse,
        context: AccessContext,
        exact_geometry: bool,
    ) -> Result<(), RightsDenial> {
        if self.state != RightsState::Verified {
            return Err(RightsDenial::RightsNotVerified(self.state));
        }
        if self.access_tier == AccessTier::Withheld {
            return Err(RightsDenial::WithheldAsset);
        }
        if !self.permitted_uses.contains(&purpose) {
            return Err(RightsDenial::PurposeNotPermitted(purpose));
        }
        if context.max_tier == AccessTier::Withheld || context.max_tier < self.access_tier {
            return Err(RightsDenial::AccessTierInsufficient {
                required: self.access_tier,
                granted: context.max_tier,
            });
        }

        if exact_geometry {
            if !context.exact_geometry_authorized {
                return Err(RightsDenial::ExactGeometryNotAuthorized);
            }

            let required_tier = match self.exact_geometry_policy {
                ExactGeometryPolicy::Restricted => AccessTier::Restricted,
                ExactGeometryPolicy::CustodialApprovalRequired => AccessTier::Custodial,
                ExactGeometryPolicy::ApprovedResearchOnly => {
                    if purpose != HeritageUse::Research {
                        return Err(RightsDenial::PurposeNotPermitted(purpose));
                    }
                    AccessTier::ApprovedResearch
                }
                ExactGeometryPolicy::PublicApproved => AccessTier::Public,
                ExactGeometryPolicy::Withheld => {
                    return Err(RightsDenial::ExactGeometryWithheld);
                }
            };

            if context.max_tier < required_tier {
                return Err(RightsDenial::ExactGeometryTierInsufficient {
                    required: required_tier,
                    granted: context.max_tier,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeritageModality {
    AirborneLidar,
    DroneLidar,
    SpaceborneLidar,
    Sar,
    OpticalMultispectral,
    Hyperspectral,
    Thermal,
    Dem,
    HistoricalAerial,
    HistoricalMap,
    Photogrammetry,
    GroundPenetratingRadar,
    Magnetometry,
    ElectricalResistivity,
    FieldSurvey,
    OralOrCommunityHistory,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrbitDirection {
    Ascending,
    Descending,
    NotApplicable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderIdentity {
    pub provider_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetIdentity {
    pub asset_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_item_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stac_asset_key: Option<String>,
    pub media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_length: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcquisitionInterval {
    pub start_at: String,
    pub end_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_level: Option<String>,
    #[serde(default)]
    pub bands_or_polarisations: Vec<String>,
    pub orbit_direction: OrbitDirection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialMetadata {
    /// Source bounding box in the declared CRS: 2D [minx, miny, maxx, maxy]
    /// or 3D [minx, miny, minz, maxx, maxy, maxz]. Axis semantics remain
    /// those of the declared CRS; this layer does not silently reproject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,
    pub crs_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_datum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_resolution_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub point_density_points_per_m2: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positional_accuracy_m: Option<f64>,
    pub geometry_access_tier: AccessTier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_fraction: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodata_fraction: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_error_m: Option<f64>,
    #[serde(default)]
    pub known_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionProvenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_dataset_version: Option<String>,
    pub ingested_at: String,
    pub ingestion_pipeline_id: String,
    pub ingestion_pipeline_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingestion_manifest_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_receipt_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceAssetMetadata {
    pub schema_version: String,
    pub source_id: String,
    pub provider: ProviderIdentity,
    pub asset: AssetIdentity,
    pub modality: HeritageModality,
    pub acquisition: AcquisitionInterval,
    pub spatial: SpatialMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<QualityMetadata>,
    pub rights: RightsProfile,
    pub provenance: IngestionProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataValidationError {
    UnsupportedSchemaVersion,
    EmptySourceId,
    EmptyProviderId,
    EmptyProviderName,
    InvalidProviderHomepage,
    EmptyAssetReference,
    EmptyMediaType,
    InvalidStacItemUri,
    InvalidAcquisitionStart,
    InvalidAcquisitionEnd,
    AcquisitionEndsBeforeItStarts,
    InvalidCrsUri,
    InvalidBoundingBox,
    InvalidNativeResolution,
    MissingNativeResolution,
    InvalidPointDensity,
    MissingPointDensity,
    InvalidPositionalAccuracy,
    MissingVerticalDatum,
    EmptyVerticalDatum,
    InvalidContentDigest,
    InvalidQualityFraction,
    InvalidRegistrationError,
    InvalidIngestionTimestamp,
    InvalidRightsReviewTimestamp,
    InvalidLicenceUri,
    ProvenanceDigestMismatch,
    EmptyIngestionPipelineId,
    EmptyIngestionPipelineVersion,
    EmptyPermittedUses,
    InconsistentWithheldRights,
}

impl fmt::Display for MetadataValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source asset metadata validation failed: {self:?}")
    }
}

impl std::error::Error for MetadataValidationError {}

impl SourceAssetMetadata {
    /// Validate domain invariants without downloading or interpreting the asset.
    pub fn validate(&self) -> Result<(), Vec<MetadataValidationError>> {
        let mut errors = Vec::new();

        if self.schema_version != SOURCE_ASSET_SCHEMA_VERSION {
            errors.push(MetadataValidationError::UnsupportedSchemaVersion);
        }
        if self.source_id.trim().is_empty() {
            errors.push(MetadataValidationError::EmptySourceId);
        }
        if self.provider.provider_id.trim().is_empty() {
            errors.push(MetadataValidationError::EmptyProviderId);
        }
        if self.provider.name.trim().is_empty() {
            errors.push(MetadataValidationError::EmptyProviderName);
        }
        if self
            .provider
            .homepage
            .as_deref()
            .is_some_and(|value| !has_uri_scheme(value))
        {
            errors.push(MetadataValidationError::InvalidProviderHomepage);
        }
        if self.asset.asset_ref.trim().is_empty() {
            errors.push(MetadataValidationError::EmptyAssetReference);
        }
        if self.asset.media_type.trim().is_empty() {
            errors.push(MetadataValidationError::EmptyMediaType);
        }
        if self
            .asset
            .stac_item_uri
            .as_deref()
            .is_some_and(|value| !has_uri_scheme(value))
        {
            errors.push(MetadataValidationError::InvalidStacItemUri);
        }

        let start = parse_rfc3339_nanos(&self.acquisition.start_at);
        let end = parse_rfc3339_nanos(&self.acquisition.end_at);
        if start.is_none() {
            errors.push(MetadataValidationError::InvalidAcquisitionStart);
        }
        if end.is_none() {
            errors.push(MetadataValidationError::InvalidAcquisitionEnd);
        }
        if matches!((start, end), (Some(a), Some(b)) if b < a) {
            errors.push(MetadataValidationError::AcquisitionEndsBeforeItStarts);
        }

        if !has_uri_scheme(&self.spatial.crs_uri) {
            errors.push(MetadataValidationError::InvalidCrsUri);
        }
        if let Some(bbox) = self.spatial.bbox.as_ref() {
            if !matches!(bbox.len(), 4 | 6) || bbox.iter().any(|value| !value.is_finite()) {
                errors.push(MetadataValidationError::InvalidBoundingBox);
            }
        }
        if requires_native_resolution(self.modality) && self.spatial.native_resolution_m.is_none() {
            errors.push(MetadataValidationError::MissingNativeResolution);
        }
        if let Some(resolution) = self.spatial.native_resolution_m {
            if !resolution.is_finite() || resolution <= 0.0 {
                errors.push(MetadataValidationError::InvalidNativeResolution);
            }
        }
        if requires_point_density(self.modality)
            && self.spatial.point_density_points_per_m2.is_none()
        {
            errors.push(MetadataValidationError::MissingPointDensity);
        }
        if let Some(density) = self.spatial.point_density_points_per_m2 {
            if !density.is_finite() || density <= 0.0 {
                errors.push(MetadataValidationError::InvalidPointDensity);
            }
        }
        if let Some(accuracy) = self.spatial.positional_accuracy_m {
            if !accuracy.is_finite() || accuracy < 0.0 {
                errors.push(MetadataValidationError::InvalidPositionalAccuracy);
            }
        }

        if requires_vertical_datum(self.modality) {
            match self.spatial.vertical_datum.as_deref() {
                None => errors.push(MetadataValidationError::MissingVerticalDatum),
                Some(value) if value.trim().is_empty() => {
                    errors.push(MetadataValidationError::EmptyVerticalDatum);
                }
                Some(_) => {}
            }
        }

        if let Some(digest) = self.asset.content_sha256.as_deref() {
            if !is_sha256_hex(digest) {
                errors.push(MetadataValidationError::InvalidContentDigest);
            }
        }
        if let Some(digest) = self.provenance.content_sha256.as_deref() {
            if !is_sha256_hex(digest) {
                errors.push(MetadataValidationError::InvalidContentDigest);
            }
            if self
                .asset
                .content_sha256
                .as_deref()
                .is_some_and(|asset_digest| !asset_digest.eq_ignore_ascii_case(digest))
            {
                errors.push(MetadataValidationError::ProvenanceDigestMismatch);
            }
        }

        if self.quality.as_ref().is_some_and(|quality| {
            [quality.cloud_fraction, quality.nodata_fraction]
                .into_iter()
                .flatten()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        }) {
            errors.push(MetadataValidationError::InvalidQualityFraction);
        }
        if self.quality.as_ref().is_some_and(|quality| {
            quality
                .registration_error_m
                .is_some_and(|value| !value.is_finite() || value < 0.0)
        }) {
            errors.push(MetadataValidationError::InvalidRegistrationError);
        }

        if parse_rfc3339_nanos(&self.provenance.ingested_at).is_none() {
            errors.push(MetadataValidationError::InvalidIngestionTimestamp);
        }
        if self
            .rights
            .review_due_at
            .as_deref()
            .is_some_and(|value| parse_rfc3339_nanos(value).is_none())
        {
            errors.push(MetadataValidationError::InvalidRightsReviewTimestamp);
        }
        if self
            .rights
            .licence_uri
            .as_deref()
            .is_some_and(|value| !has_uri_scheme(value))
        {
            errors.push(MetadataValidationError::InvalidLicenceUri);
        }
        if self.provenance.ingestion_pipeline_id.trim().is_empty() {
            errors.push(MetadataValidationError::EmptyIngestionPipelineId);
        }
        if self.provenance.ingestion_pipeline_version.trim().is_empty() {
            errors.push(MetadataValidationError::EmptyIngestionPipelineVersion);
        }
        if self.rights.permitted_uses.is_empty() {
            errors.push(MetadataValidationError::EmptyPermittedUses);
        }
        if (self.rights.access_tier == AccessTier::Withheld
            || self.spatial.geometry_access_tier == AccessTier::Withheld)
            && self.rights.state == RightsState::Verified
        {
            errors.push(MetadataValidationError::InconsistentWithheldRights);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Deterministic digest of this typed metadata representation.
    ///
    /// Serde struct field order and BTreeSet ordering make this representation
    /// stable for the current schema version. A schema change requires a new
    /// version and must not reuse old digests as if they were equivalent.
    pub fn metadata_sha256(&self) -> Result<String, serde_json::Error> {
        let bytes = serde_json::to_vec(self)?;
        Ok(sha256_hex(&bytes))
    }
}

fn requires_native_resolution(modality: HeritageModality) -> bool {
    matches!(
        modality,
        HeritageModality::Sar
            | HeritageModality::OpticalMultispectral
            | HeritageModality::Hyperspectral
            | HeritageModality::Thermal
            | HeritageModality::Dem
            | HeritageModality::HistoricalAerial
            | HeritageModality::Photogrammetry
    )
}

fn requires_point_density(modality: HeritageModality) -> bool {
    matches!(
        modality,
        HeritageModality::AirborneLidar | HeritageModality::DroneLidar
    )
}

fn requires_vertical_datum(modality: HeritageModality) -> bool {
    matches!(
        modality,
        HeritageModality::AirborneLidar
            | HeritageModality::DroneLidar
            | HeritageModality::SpaceborneLidar
            | HeritageModality::Dem
    )
}

fn parse_rfc3339_nanos(value: &str) -> Option<i128> {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return None;
    }

    let year = parse_digits(bytes, 0, 4)? as i64;
    let month = parse_digits(bytes, 5, 2)? as i64;
    let day = parse_digits(bytes, 8, 2)? as i64;
    let hour = parse_digits(bytes, 11, 2)? as i64;
    let minute = parse_digits(bytes, 14, 2)? as i64;
    let second = parse_digits(bytes, 17, 2)? as i64;

    if !(1..=12).contains(&month)
        || day < 1
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }

    let mut cursor = 19;
    let mut fractional_nanos = 0_i64;
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        let fraction_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        let fraction_len = cursor - fraction_start;
        if fraction_len == 0 || fraction_len > 9 {
            return None;
        }
        let fraction = parse_digits(bytes, fraction_start, fraction_len)? as i64;
        fractional_nanos = fraction * 10_i64.pow((9 - fraction_len) as u32);
    }

    let offset_seconds: i64 = match bytes.get(cursor) {
        Some(b'Z') if cursor + 1 == bytes.len() => 0,
        Some(sign @ (b'+' | b'-')) if cursor + 6 == bytes.len() => {
            if bytes.get(cursor + 3) != Some(&b':') {
                return None;
            }
            let offset_hour = parse_digits(bytes, cursor + 1, 2)? as i64;
            let offset_minute = parse_digits(bytes, cursor + 4, 2)? as i64;
            if offset_hour > 23 || offset_minute > 59 {
                return None;
            }
            let magnitude = offset_hour * 3600 + offset_minute * 60;
            if *sign == b'+' { magnitude } else { -magnitude }
        }
        _ => return None,
    };

    // Howard Hinnant's civil-date conversion. The result is timezone-neutral;
    // the RFC 3339 offset is subtracted to obtain UTC.
    let adjusted_year = year - if month <= 2 { 1 } else { 0 };
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146_097 + day_of_era - 719_468;

    let local_seconds = days_since_epoch * 86_400 + hour * 3_600 + minute * 60 + second;
    let utc_seconds = local_seconds - offset_seconds;
    Some(i128::from(utc_seconds) * 1_000_000_000 + i128::from(fractional_nanos))
}

fn parse_digits(bytes: &[u8], start: usize, length: usize) -> Option<u32> {
    let end = start.checked_add(length)?;
    let slice = bytes.get(start..end)?;
    if !slice.iter().all(u8::is_ascii_digit) {
        return None;
    }
    slice.iter().try_fold(0_u32, |value, digit| {
        value.checked_mul(10)?.checked_add(u32::from(*digit - b'0'))
    })
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetVersion {
    pub version: usize,
    pub source_id: String,
    pub asset_ref: String,
    pub metadata_sha256: String,
    pub content_sha256: Option<String>,
    pub rights_state: RightsState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationOutcome {
    Added {
        version: usize,
    },
    AlreadyPresent {
        version: usize,
    },
    NewRevision {
        version: usize,
        previous_version: usize,
    },
}

/// In-memory append-only registry reference implementation.
///
/// Durability, concurrent writers, signed provider statements, and distributed
/// replication are deliberately left to a later storage adapter. Within this
/// process, prior revisions are never overwritten.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRegistry {
    revisions: BTreeMap<String, Vec<AssetVersion>>,
}

impl AssetRegistry {
    pub fn register(
        &mut self,
        metadata: &SourceAssetMetadata,
    ) -> Result<RegistrationOutcome, RegistryError> {
        metadata
            .validate()
            .map_err(RegistryError::InvalidMetadata)?;
        let metadata_sha256 = metadata
            .metadata_sha256()
            .map_err(|error| RegistryError::Serialization(error.to_string()))?;
        let key = registry_key(&metadata.source_id, &metadata.asset.asset_ref);
        let versions = self.revisions.entry(key).or_default();

        if let Some(existing) = versions
            .iter()
            .find(|version| version.metadata_sha256 == metadata_sha256)
        {
            return Ok(RegistrationOutcome::AlreadyPresent {
                version: existing.version,
            });
        }

        let previous_version = versions.last().map(|version| version.version);
        let version = previous_version.unwrap_or(0) + 1;
        versions.push(AssetVersion {
            version,
            source_id: metadata.source_id.clone(),
            asset_ref: metadata.asset.asset_ref.clone(),
            metadata_sha256,
            content_sha256: metadata.asset.content_sha256.clone(),
            rights_state: metadata.rights.state,
        });

        Ok(match previous_version {
            Some(previous_version) => RegistrationOutcome::NewRevision {
                version,
                previous_version,
            },
            None => RegistrationOutcome::Added { version },
        })
    }

    pub fn versions(&self, source_id: &str, asset_ref: &str) -> &[AssetVersion] {
        self.revisions
            .get(&registry_key(source_id, asset_ref))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn asset_count(&self) -> usize {
        self.revisions.len()
    }
}

fn registry_key(source_id: &str, asset_ref: &str) -> String {
    format!("{source_id}\u{1f}{asset_ref}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    InvalidMetadata(Vec<MetadataValidationError>),
    Serialization(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "asset registry error: {self:?}")
    }
}

impl std::error::Error for RegistryError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionReceipt {
    pub schema_version: String,
    pub source_id: String,
    pub asset_ref: String,
    pub retrieved_at: String,
    pub byte_length: u64,
    pub content_sha256: String,
    pub metadata_sha256: String,
    pub pipeline_id: String,
    pub pipeline_version: String,
    pub receipt_sha256: String,
}

#[derive(Serialize)]
struct ReceiptPayload<'a> {
    schema_version: &'a str,
    source_id: &'a str,
    asset_ref: &'a str,
    retrieved_at: &'a str,
    byte_length: u64,
    content_sha256: &'a str,
    metadata_sha256: &'a str,
    pipeline_id: &'a str,
    pipeline_version: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptError {
    InvalidMetadata(Vec<MetadataValidationError>),
    MissingExpectedDigest,
    InvalidExpectedDigest,
    DigestMismatch { expected: String, actual: String },
    InvalidRetrievalTimestamp,
    EmptyPipelineId,
    EmptyPipelineVersion,
    Serialization(String),
    RightsDenied(RightsDenial),
    ByteLengthMismatch { expected: u64, actual: u64 },
}

impl fmt::Display for ReceiptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ingestion receipt error: {self:?}")
    }
}

impl std::error::Error for ReceiptError {}

/// Apply both asset-level rights and spatial geometry access controls.
pub fn authorize_metadata_use(
    metadata: &SourceAssetMetadata,
    purpose: HeritageUse,
    context: AccessContext,
    exact_geometry: bool,
) -> Result<(), RightsDenial> {
    metadata
        .rights
        .authorize(purpose, context, exact_geometry)?;
    if metadata.spatial.geometry_access_tier == AccessTier::Withheld {
        return Err(RightsDenial::WithheldAsset);
    }
    if context.max_tier < metadata.spatial.geometry_access_tier {
        return Err(RightsDenial::AccessTierInsufficient {
            required: metadata.spatial.geometry_access_tier,
            granted: context.max_tier,
        });
    }
    Ok(())
}

/// Verify exact supplied bytes against the source snapshot and create a
/// deterministic receipt after authorizing the one requested use and access
/// context. Authorization here does not extend to other purposes, publication,
/// redistribution, land access, field survey, excavation, or physical disturbance.
pub fn verify_artifact_bytes(
    metadata: &SourceAssetMetadata,
    bytes: &[u8],
    retrieved_at: &str,
    pipeline_id: &str,
    pipeline_version: &str,
    purpose: HeritageUse,
    access: AccessContext,
    exact_geometry: bool,
) -> Result<IngestionReceipt, ReceiptError> {
    metadata.validate().map_err(ReceiptError::InvalidMetadata)?;
    authorize_metadata_use(metadata, purpose, access, exact_geometry)
        .map_err(ReceiptError::RightsDenied)?;

    let expected = metadata
        .asset
        .content_sha256
        .as_deref()
        .ok_or(ReceiptError::MissingExpectedDigest)?;
    if !is_sha256_hex(expected) {
        return Err(ReceiptError::InvalidExpectedDigest);
    }
    if parse_rfc3339_nanos(retrieved_at).is_none() {
        return Err(ReceiptError::InvalidRetrievalTimestamp);
    }
    if pipeline_id.trim().is_empty() {
        return Err(ReceiptError::EmptyPipelineId);
    }
    if pipeline_version.trim().is_empty() {
        return Err(ReceiptError::EmptyPipelineVersion);
    }

    let actual_length = u64::try_from(bytes.len())
        .map_err(|error| ReceiptError::Serialization(error.to_string()))?;
    if let Some(expected_length) = metadata.asset.byte_length {
        if expected_length != actual_length {
            return Err(ReceiptError::ByteLengthMismatch {
                expected: expected_length,
                actual: actual_length,
            });
        }
    }

    let actual = sha256_hex(bytes);
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(ReceiptError::DigestMismatch {
            expected: expected.to_ascii_lowercase(),
            actual,
        });
    }

    let metadata_sha256 = metadata
        .metadata_sha256()
        .map_err(|error| ReceiptError::Serialization(error.to_string()))?;
    let byte_length = actual_length;

    let payload = ReceiptPayload {
        schema_version: INGESTION_RECEIPT_SCHEMA_VERSION,
        source_id: &metadata.source_id,
        asset_ref: &metadata.asset.asset_ref,
        retrieved_at,
        byte_length,
        content_sha256: &actual,
        metadata_sha256: &metadata_sha256,
        pipeline_id,
        pipeline_version,
    };
    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|error| ReceiptError::Serialization(error.to_string()))?;
    let receipt_sha256 = sha256_hex(&payload_bytes);

    Ok(IngestionReceipt {
        schema_version: INGESTION_RECEIPT_SCHEMA_VERSION.to_owned(),
        source_id: metadata.source_id.clone(),
        asset_ref: metadata.asset.asset_ref.clone(),
        retrieved_at: retrieved_at.to_owned(),
        byte_length,
        content_sha256: actual,
        metadata_sha256,
        pipeline_id: pipeline_id.to_owned(),
        pipeline_version: pipeline_version.to_owned(),
        receipt_sha256,
    })
}

/// STAC envelope that retains the complete parsed Item, including unknown
/// extensions and provider-specific fields. No field is silently projected
/// away by this adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct StacItemEnvelope {
    original: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StacItemError {
    InvalidJson(String),
    NotFeature,
    MissingStacVersion,
    MissingItemId,
    MissingAssets,
    InvalidAsset(String),
}

impl fmt::Display for StacItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "STAC Item error: {self:?}")
    }
}

impl std::error::Error for StacItemError {}

impl StacItemEnvelope {
    pub fn parse(json: &str) -> Result<Self, StacItemError> {
        let original: Value = serde_json::from_str(json)
            .map_err(|error| StacItemError::InvalidJson(error.to_string()))?;
        let object = original.as_object().ok_or(StacItemError::NotFeature)?;

        if object.get("type").and_then(Value::as_str) != Some("Feature") {
            return Err(StacItemError::NotFeature);
        }
        if object
            .get("stac_version")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(StacItemError::MissingStacVersion);
        }
        if object
            .get("id")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(StacItemError::MissingItemId);
        }

        let assets = object
            .get("assets")
            .and_then(Value::as_object)
            .ok_or(StacItemError::MissingAssets)?;
        if assets.is_empty() {
            return Err(StacItemError::MissingAssets);
        }
        for (key, value) in assets {
            let valid_href = value
                .as_object()
                .and_then(|asset| asset.get("href"))
                .and_then(Value::as_str)
                .is_some_and(|href| !href.trim().is_empty());
            if !valid_href {
                return Err(StacItemError::InvalidAsset(key.clone()));
            }
        }

        Ok(Self { original })
    }

    pub fn item_id(&self) -> &str {
        self.original
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }

    pub fn asset_keys(&self) -> Vec<&str> {
        self.original
            .get("assets")
            .and_then(Value::as_object)
            .map(|assets| assets.keys().map(String::as_str).collect())
            .unwrap_or_default()
    }

    pub fn original(&self) -> &Value {
        &self.original
    }

    pub fn into_original(self) -> Value {
        self.original
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_metadata() -> SourceAssetMetadata {
        let mut permitted_uses = BTreeSet::new();
        permitted_uses.insert(HeritageUse::View);
        permitted_uses.insert(HeritageUse::Research);
        permitted_uses.insert(HeritageUse::Evaluation);

        SourceAssetMetadata {
            schema_version: SOURCE_ASSET_SCHEMA_VERSION.to_owned(),
            source_id: "source-synthetic-01".to_owned(),
            provider: ProviderIdentity {
                provider_id: "provider-synthetic".to_owned(),
                name: "Synthetic Test Provider".to_owned(),
                homepage: Some("https://example.invalid".to_owned()),
                contact_ref: None,
            },
            asset: AssetIdentity {
                asset_ref: "urn:synthetic:asset:lidar:01".to_owned(),
                stac_item_uri: Some("https://example.invalid/stac/items/synthetic-01".to_owned()),
                stac_asset_key: Some("lidar".to_owned()),
                media_type: "application/octet-stream".to_owned(),
                upstream_asset_id: Some("synthetic-upstream-01".to_owned()),
                content_sha256: Some(sha256_hex(b"synthetic point cloud bytes")),
                byte_length: Some(b"synthetic point cloud bytes".len() as u64),
            },
            modality: HeritageModality::AirborneLidar,
            acquisition: AcquisitionInterval {
                start_at: "2026-10-01T00:00:00Z".to_owned(),
                end_at: "2026-10-01T00:10:00Z".to_owned(),
                platform: Some("synthetic-platform".to_owned()),
                sensor: Some("synthetic-lidar".to_owned()),
                product_level: Some("synthetic-ground-model".to_owned()),
                bands_or_polarisations: Vec::new(),
                orbit_direction: OrbitDirection::NotApplicable,
                acquisition_notes: None,
            },
            spatial: SpatialMetadata {
                bbox: Some(vec![0.1, 0.1, 0.2, 0.2]),
                crs_uri: "https://www.opengis.net/def/crs/OGC/1.3/CRS84".to_owned(),
                vertical_datum: Some("synthetic-vertical-datum".to_owned()),
                native_resolution_m: Some(0.1),
                point_density_points_per_m2: Some(12.0),
                positional_accuracy_m: Some(0.1),
                geometry_access_tier: AccessTier::ApprovedResearch,
                coverage_notes: Some("Synthetic fixture only".to_owned()),
            },
            quality: Some(QualityMetadata {
                cloud_fraction: Some(0.0),
                nodata_fraction: Some(0.0),
                registration_error_m: Some(0.05),
                known_issues: Vec::new(),
            }),
            rights: RightsProfile {
                state: RightsState::Verified,
                permitted_uses,
                access_tier: AccessTier::ApprovedResearch,
                exact_geometry_policy: ExactGeometryPolicy::ApprovedResearchOnly,
                licence_uri: Some("https://example.invalid/licence/synthetic".to_owned()),
                attribution_text: Some("Synthetic fixture only".to_owned()),
                consent_or_authority_ref: Some("urn:synthetic:authority:01".to_owned()),
                jurisdiction: Some("synthetic".to_owned()),
                review_due_at: Some("2027-11-02T10:00:00Z".to_owned()),
                restrictions: Vec::new(),
            },
            provenance: IngestionProvenance {
                upstream_dataset_version: Some("synthetic-v1".to_owned()),
                ingested_at: "2026-11-02T10:00:00Z".to_owned(),
                ingestion_pipeline_id: "heritage-fixture-ingestor".to_owned(),
                ingestion_pipeline_version: "0.1.0".to_owned(),
                ingestion_manifest_ref: Some("urn:synthetic:manifest:01".to_owned()),
                retrieval_receipt_ref: None,
                content_sha256: None,
            },
        }
    }

    #[test]
    fn uri_scheme_validation_accepts_absolute_uris_and_rejects_malformed_values() {
        for valid in [
            "https://example.org/data/item.json",
            "urn:ogc:def:crs:EPSG::4326",
            "mailto:heritage@example.org",
            "file:///tmp/asset.laz",
        ] {
            assert!(has_uri_scheme(valid), "expected valid URI scheme: {valid}");
        }
        for invalid in [
            "",
            "relative/path/item.json",
            ":missing-scheme",
            "1http://example.org",
            "http ://example.org",
            "https://example.org/with space",
            " https://example.org",
        ] {
            assert!(!has_uri_scheme(invalid), "expected invalid URI: {invalid}");
        }
    }

    #[test]
    fn sha256_digest_validation_requires_exactly_64_hexadecimal_digits() {
        let digest = sha256_hex(b"heritage-source");
        assert!(is_sha256_hex(&digest));
        assert!(is_sha256_hex(&digest.to_ascii_uppercase()));
        assert!(!is_sha256_hex(&digest[..63]));
        assert!(!is_sha256_hex(&format!("{}g", &digest[..63])));
        assert!(!is_sha256_hex(&"0".repeat(63)));
    }

    #[test]
    fn sha256_matches_standard_known_answer_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn valid_source_asset_metadata_passes_domain_validation() {
        assert_eq!(fixture_metadata().validate(), Ok(()));
    }

    #[test]
    fn source_metadata_serializes_with_versioned_nested_contract_shape() {
        let value = serde_json::to_value(fixture_metadata()).unwrap();
        assert_eq!(value["schema_version"], SOURCE_ASSET_SCHEMA_VERSION);
        assert_eq!(value["provider"]["provider_id"], "provider-synthetic");
        assert_eq!(value["asset"]["stac_asset_key"], "lidar");
        assert_eq!(value["acquisition"]["orbit_direction"], "not_applicable");
        assert_eq!(
            value["spatial"]["geometry_access_tier"],
            "approved_research"
        );
        assert_eq!(
            value["rights"]["exact_geometry_policy"],
            "approved_research_only"
        );
        assert!(value["quality"].is_object());
        assert!(value["provenance"]["ingested_at"].is_string());
        assert!(value["provider"].get("contact_ref").is_none());
    }

    #[test]
    fn rfc3339_timestamp_ordering_respects_timezone_offsets() {
        let start = parse_rfc3339_nanos("2026-01-01T00:00:00+02:00").unwrap();
        let end = parse_rfc3339_nanos("2025-12-31T22:00:01Z").unwrap();
        assert!(end > start);
        assert!(parse_rfc3339_nanos("2026-02-30T00:00:00Z").is_none());
        assert!(parse_rfc3339_nanos("2026-01-01T00:00:00.123456789Z").is_some());
        assert!(parse_rfc3339_nanos("2026-01-01T00:00:00.1234567890Z").is_none());
    }

    #[test]
    fn antimeridian_bbox_is_preserved_and_malformed_bbox_is_rejected() {
        let mut metadata = fixture_metadata();
        metadata.spatial.bbox = Some(vec![170.0, -10.0, -170.0, 10.0]);
        assert_eq!(metadata.validate(), Ok(()));

        metadata.spatial.bbox = Some(vec![170.0, -10.0, -170.0]);
        assert!(
            metadata
                .validate()
                .unwrap_err()
                .contains(&MetadataValidationError::InvalidBoundingBox)
        );
    }

    #[test]
    fn elevation_modalities_require_an_explicit_vertical_datum() {
        let mut metadata = fixture_metadata();
        metadata.spatial.vertical_datum = None;
        assert!(
            metadata
                .validate()
                .unwrap_err()
                .contains(&MetadataValidationError::MissingVerticalDatum)
        );

        metadata.modality = HeritageModality::OpticalMultispectral;
        assert_eq!(metadata.validate(), Ok(()));
    }

    #[test]
    fn malformed_content_digest_is_rejected() {
        let mut metadata = fixture_metadata();
        metadata.asset.content_sha256 = Some("not-a-digest".to_owned());
        assert!(
            metadata
                .validate()
                .unwrap_err()
                .contains(&MetadataValidationError::InvalidContentDigest)
        );
    }

    #[test]
    fn unknown_or_pending_rights_fail_closed() {
        let mut profile = fixture_metadata().rights;
        let context = AccessContext {
            max_tier: AccessTier::Restricted,
            exact_geometry_authorized: true,
        };

        profile.state = RightsState::Unknown;
        assert_eq!(
            profile.authorize(HeritageUse::Research, context, false),
            Err(RightsDenial::RightsNotVerified(RightsState::Unknown))
        );

        profile.state = RightsState::Pending;
        assert_eq!(
            profile.authorize(HeritageUse::Research, context, false),
            Err(RightsDenial::RightsNotVerified(RightsState::Pending))
        );

        profile.state = RightsState::Denied;
        assert_eq!(
            profile.authorize(HeritageUse::Research, context, false),
            Err(RightsDenial::RightsNotVerified(RightsState::Denied))
        );
    }

    #[test]
    fn each_purpose_requires_its_own_permission() {
        let profile = fixture_metadata().rights;
        let context = AccessContext {
            max_tier: AccessTier::ApprovedResearch,
            exact_geometry_authorized: false,
        };
        assert!(
            profile
                .authorize(HeritageUse::Research, context, false)
                .is_ok()
        );
        assert_eq!(
            profile.authorize(HeritageUse::Training, context, false),
            Err(RightsDenial::PurposeNotPermitted(HeritageUse::Training))
        );
        assert_eq!(
            profile.authorize(HeritageUse::Publication, context, false),
            Err(RightsDenial::PurposeNotPermitted(HeritageUse::Publication))
        );
    }

    #[test]
    fn exact_geometry_requires_separate_approval_and_tier() {
        let profile = fixture_metadata().rights;
        let unapproved = AccessContext {
            max_tier: AccessTier::Restricted,
            exact_geometry_authorized: false,
        };
        assert_eq!(
            profile.authorize(HeritageUse::Research, unapproved, true),
            Err(RightsDenial::ExactGeometryNotAuthorized)
        );

        let public_approved = AccessContext {
            max_tier: AccessTier::Public,
            exact_geometry_authorized: true,
        };
        assert_eq!(
            profile.authorize(HeritageUse::Research, public_approved, true),
            Err(RightsDenial::AccessTierInsufficient {
                required: AccessTier::ApprovedResearch,
                granted: AccessTier::Public,
            })
        );

        let approved_research = AccessContext {
            max_tier: AccessTier::ApprovedResearch,
            exact_geometry_authorized: true,
        };
        assert!(
            profile
                .authorize(HeritageUse::Research, approved_research, true)
                .is_ok()
        );
        assert_eq!(
            profile.authorize(HeritageUse::Evaluation, approved_research, true),
            Err(RightsDenial::PurposeNotPermitted(HeritageUse::Evaluation))
        );
    }

    #[test]
    fn spatial_geometry_tier_is_enforced_separately_from_asset_rights() {
        let mut metadata = fixture_metadata();
        metadata.rights.access_tier = AccessTier::Public;
        metadata.spatial.geometry_access_tier = AccessTier::Custodial;
        let context = AccessContext {
            max_tier: AccessTier::Public,
            exact_geometry_authorized: true,
        };
        assert_eq!(
            authorize_metadata_use(&metadata, HeritageUse::Research, context, false),
            Err(RightsDenial::AccessTierInsufficient {
                required: AccessTier::Custodial,
                granted: AccessTier::Public,
            })
        );
    }

    #[test]
    fn withheld_exact_geometry_never_passes() {
        let mut profile = fixture_metadata().rights;
        profile.exact_geometry_policy = ExactGeometryPolicy::Withheld;
        let context = AccessContext {
            max_tier: AccessTier::Restricted,
            exact_geometry_authorized: true,
        };
        assert_eq!(
            profile.authorize(HeritageUse::Research, context, true),
            Err(RightsDenial::ExactGeometryWithheld)
        );
    }

    #[test]
    fn receipt_matches_exact_bytes_and_is_deterministic() {
        let bytes = b"synthetic point cloud bytes";
        let metadata = fixture_metadata();
        let first = verify_artifact_bytes(
            &metadata,
            bytes,
            "2026-11-02T10:00:00Z",
            "heritage-ingestor",
            "0.1.0",
            HeritageUse::Research,
            AccessContext {
                max_tier: AccessTier::ApprovedResearch,
                exact_geometry_authorized: true,
            },
            true,
        )
        .unwrap();
        let second = verify_artifact_bytes(
            &metadata,
            bytes,
            "2026-11-02T10:00:00Z",
            "heritage-ingestor",
            "0.1.0",
            HeritageUse::Research,
            AccessContext {
                max_tier: AccessTier::ApprovedResearch,
                exact_geometry_authorized: true,
            },
            true,
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.byte_length, bytes.len() as u64);
        assert_eq!(first.content_sha256, sha256_hex(bytes));
        assert_eq!(first.receipt_sha256.len(), 64);
    }

    #[test]
    fn receipt_rejects_declared_byte_length_mismatch() {
        let mut metadata = fixture_metadata();
        metadata.asset.byte_length = Some(1);
        assert_eq!(
            verify_artifact_bytes(
                &metadata,
                b"synthetic point cloud bytes",
                "2026-11-02T10:00:00Z",
                "heritage-ingestor",
                "0.1.0",
                HeritageUse::Research,
                AccessContext {
                    max_tier: AccessTier::ApprovedResearch,
                    exact_geometry_authorized: true,
                },
                true,
            ),
            Err(ReceiptError::ByteLengthMismatch {
                expected: 1,
                actual: 27,
            })
        );
    }

    #[test]
    fn receipt_detects_same_reference_with_changed_bytes() {
        let metadata = fixture_metadata();
        let error = verify_artifact_bytes(
            &metadata,
            b"synthetic point cloud byteX",
            "2026-11-02T10:00:00Z",
            "heritage-ingestor",
            "0.1.0",
            HeritageUse::Research,
            AccessContext {
                max_tier: AccessTier::ApprovedResearch,
                exact_geometry_authorized: true,
            },
            true,
        )
        .unwrap_err();
        assert!(matches!(error, ReceiptError::DigestMismatch { .. }));
    }

    #[test]
    fn receipt_denies_unverified_rights_even_when_bytes_match() {
        let mut metadata = fixture_metadata();
        metadata.rights.state = RightsState::Unknown;
        let error = verify_artifact_bytes(
            &metadata,
            b"synthetic point cloud bytes",
            "2026-11-02T10:00:00Z",
            "heritage-ingestor",
            "0.1.0",
            HeritageUse::Research,
            AccessContext {
                max_tier: AccessTier::Restricted,
                exact_geometry_authorized: true,
            },
            true,
        )
        .unwrap_err();
        assert_eq!(
            error,
            ReceiptError::RightsDenied(RightsDenial::RightsNotVerified(RightsState::Unknown))
        );
    }

    #[test]
    fn receipt_requires_expected_digest_and_valid_retrieval_time() {
        let mut metadata = fixture_metadata();
        metadata.asset.content_sha256 = None;
        assert_eq!(
            verify_artifact_bytes(
                &metadata,
                b"bytes",
                "2026-11-02T10:00:00Z",
                "heritage-ingestor",
                "0.1.0",
                HeritageUse::Research,
                AccessContext {
                    max_tier: AccessTier::ApprovedResearch,
                    exact_geometry_authorized: true
                },
                true
            ),
            Err(ReceiptError::MissingExpectedDigest)
        );

        metadata.asset.content_sha256 = Some(sha256_hex(b"bytes"));
        assert_eq!(
            verify_artifact_bytes(
                &metadata,
                b"bytes",
                "not-a-timestamp",
                "heritage-ingestor",
                "0.1.0",
                HeritageUse::Research,
                AccessContext {
                    max_tier: AccessTier::ApprovedResearch,
                    exact_geometry_authorized: true
                },
                true
            ),
            Err(ReceiptError::InvalidRetrievalTimestamp)
        );
    }

    #[test]
    fn registry_is_idempotent_and_preserves_changed_revisions() {
        let mut metadata = fixture_metadata();
        let mut registry = AssetRegistry::default();

        assert_eq!(
            registry.register(&metadata),
            Ok(RegistrationOutcome::Added { version: 1 })
        );
        assert_eq!(
            registry.register(&metadata),
            Ok(RegistrationOutcome::AlreadyPresent { version: 1 })
        );

        metadata.asset.content_sha256 = Some(sha256_hex(b"changed source bytes"));
        metadata.provenance.upstream_dataset_version = Some("synthetic-v2".to_owned());
        assert_eq!(
            registry.register(&metadata),
            Ok(RegistrationOutcome::NewRevision {
                version: 2,
                previous_version: 1,
            })
        );

        let versions = registry.versions(&metadata.source_id, &metadata.asset.asset_ref);
        assert_eq!(versions.len(), 2);
        assert_ne!(versions[0].content_sha256, versions[1].content_sha256);
        assert_eq!(versions[0].version, 1);
        assert_eq!(versions[1].version, 2);
    }

    #[test]
    fn registry_quarantines_invalid_metadata_without_mutating_history() {
        let mut metadata = fixture_metadata();
        let mut registry = AssetRegistry::default();
        metadata.spatial.crs_uri = "not-a-crs".to_owned();
        assert!(registry.register(&metadata).is_err());
        assert_eq!(registry.asset_count(), 0);
    }

    #[test]
    fn stac_item_preserves_unknown_extensions_and_all_assets() {
        let json = r#"{
            "type": "Feature",
            "stac_version": "1.0.0",
            "stac_extensions": ["https://example.org/custom/v1.0.0/schema.json"],
            "id": "synthetic-item-01",
            "geometry": null,
            "bbox": null,
            "properties": {"datetime": "2026-10-01T00:00:00Z", "custom:provider_flag": true},
            "links": [],
            "assets": {
                "lidar": {"href": "https://example.invalid/lidar.laz", "type": "application/vnd.laszip"},
                "metadata": {"href": "https://example.invalid/metadata.json", "type": "application/json"}
            },
            "custom:unknown_extension": {"preserve": true}
        }"#;

        let item = StacItemEnvelope::parse(json).unwrap();
        assert_eq!(item.item_id(), "synthetic-item-01");
        assert_eq!(item.asset_keys().len(), 2);
        assert_eq!(
            item.original()["custom:unknown_extension"]["preserve"],
            Value::Bool(true)
        );
        assert_eq!(
            item.original()["properties"]["custom:provider_flag"],
            Value::Bool(true)
        );
    }

    #[test]
    fn stac_item_rejects_invalid_envelopes_and_assets() {
        assert_eq!(
            StacItemEnvelope::parse(r#"{"type":"Feature"}"#),
            Err(StacItemError::MissingStacVersion)
        );

        let invalid_asset = r#"{
            "type":"Feature",
            "stac_version":"1.0.0",
            "id":"bad-item",
            "assets":{"missing-href":{"type":"application/json"}},
            "links":[],
            "properties":{},
            "geometry":null
        }"#;
        assert_eq!(
            StacItemEnvelope::parse(invalid_asset),
            Err(StacItemError::InvalidAsset("missing-href".to_owned()))
        );
    }
}
