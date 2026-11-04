// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rights-aware heritage dataset intake and per-asset use decisions.
//!
//! This module prevents public availability, code licensing or paper
//! data-availability statements from being silently promoted into permission
//! for unrelated data assets or uses. It checks recorded evidence shape only;
//! it is not a legal opinion and does not authenticate licence documents.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const HERITAGE_DATA_INTAKE_SCHEMA_VERSION: &str = "luminous.heritage-data-intake.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Imagery,
    Orthomosaic,
    ElevationRaster,
    PointCloud,
    Mesh,
    VectorLayer,
    Annotation,
    ModelWeights,
    SourceCode,
    Documentation,
    SurveyRawData,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryAvailability {
    PubliclyListed,
    Downloadable,
    AccessByRequest,
    NotAvailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessTier {
    Public,
    Controlled,
    Restricted,
    Custodial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateSensitivity {
    None,
    Generalized,
    Approximate,
    ExactSensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsePurpose {
    LocalInspection,
    ResearchTraining,
    Evaluation,
    DerivativeStorage,
    Publication,
    Redistribution,
    CommercialUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    Granted,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurposePermission {
    pub purpose: UsePurpose,
    pub state: PermissionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetPackageRecord {
    pub package_id: String,
    pub title: String,
    pub version: String,
    pub source_uri: String,
    pub citation: String,
    pub availability: RepositoryAvailability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_byte_size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetAssetRecord {
    pub asset_id: String,
    pub package_id: String,
    pub file_name: String,
    pub kind: AssetKind,
    pub media_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
    /// A software repository licence does not automatically apply to data assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub licence_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub licence_uri: Option<String>,
    pub access_tier: AccessTier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_receipt_ref: Option<String>,
    pub coordinate_sensitivity: CoordinateSensitivity,
    pub permissions: Vec<PurposePermission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageDatasetRegistry {
    pub schema_version: String,
    pub registry_id: String,
    pub packages: Vec<DatasetPackageRecord>,
    pub assets: Vec<DatasetAssetRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UseDecision {
    AllowedByRecordedEvidence,
    DeniedByRecordedEvidence,
    BlockedUnknownPermission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetUseDecisionReport {
    pub registry_id: String,
    pub package_id: String,
    pub asset_id: String,
    pub purpose: UsePurpose,
    pub decision: UseDecision,
    pub reason: String,
    pub licence_id: Option<String>,
    pub rights_evidence_ref: Option<String>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatasetIntakeError {
    SchemaVersionMismatch,
    EmptyRegistryId,
    EmptyPackageField(String),
    DuplicatePackageId(String),
    DuplicateAssetId(String),
    EmptyAssetField(String),
    PackageReferenceMissing(String),
    ParentAssetReferenceMissing(String),
    ParentAssetCycle(String),
    InvalidSha256(String),
    DigestRequiresByteSize(String),
    ByteSizeWithoutDigest(String),
    EmptyLicenceId(String),
    EmptyLicenceUri(String),
    DuplicatePurposePermission(String),
    GrantedPermissionMissingEvidence(String),
    DeniedPermissionMissingCondition(String),
    EmptyPermissionEvidence(String),
    EmptyPermissionCondition(String),
    DeniedPermissionMissingEvidence(String),
    AccessReceiptMissing(String),
    EmptyAccessReceipt(String),
    InvalidPackageSha256(String),
    PackageDigestRequiresByteSize(String),
    PackageByteSizeWithoutDigest(String),
    ExactSensitiveCoordinateMustNotBePublic(String),
}

impl fmt::Display for DatasetIntakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DatasetIntakeError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn validate_heritage_dataset_registry(
    registry: &HeritageDatasetRegistry,
) -> Result<(), DatasetIntakeError> {
    if registry.schema_version != HERITAGE_DATA_INTAKE_SCHEMA_VERSION {
        return Err(DatasetIntakeError::SchemaVersionMismatch);
    }
    if !non_empty(&registry.registry_id) {
        return Err(DatasetIntakeError::EmptyRegistryId);
    }

    let mut package_ids = BTreeSet::new();
    for package in &registry.packages {
        if !non_empty(&package.package_id)
            || !non_empty(&package.title)
            || !non_empty(&package.version)
            || !non_empty(&package.source_uri)
            || !non_empty(&package.citation)
        {
            return Err(DatasetIntakeError::EmptyPackageField(
                package.package_id.clone(),
            ));
        }
        if !package_ids.insert(package.package_id.as_str()) {
            return Err(DatasetIntakeError::DuplicatePackageId(
                package.package_id.clone(),
            ));
        }
        match (&package.package_sha256, package.package_byte_size) {
            (Some(digest), Some(size)) if valid_sha256(digest) && size > 0 => {}
            (Some(digest), Some(_)) if !valid_sha256(digest) => {
                return Err(DatasetIntakeError::InvalidPackageSha256(
                    package.package_id.clone(),
                ));
            }
            (Some(_), None) => {
                return Err(DatasetIntakeError::PackageDigestRequiresByteSize(
                    package.package_id.clone(),
                ));
            }
            (None, Some(_)) => {
                return Err(DatasetIntakeError::PackageByteSizeWithoutDigest(
                    package.package_id.clone(),
                ));
            }
            _ => {}
        }
    }

    let mut asset_ids = BTreeSet::new();
    for asset in &registry.assets {
        if !non_empty(&asset.asset_id)
            || !non_empty(&asset.file_name)
            || !non_empty(&asset.media_type)
        {
            return Err(DatasetIntakeError::EmptyAssetField(asset.asset_id.clone()));
        }
        if !asset_ids.insert(asset.asset_id.as_str()) {
            return Err(DatasetIntakeError::DuplicateAssetId(asset.asset_id.clone()));
        }
        if !package_ids.contains(asset.package_id.as_str()) {
            return Err(DatasetIntakeError::PackageReferenceMissing(
                asset.asset_id.clone(),
            ));
        }
        match (&asset.sha256, asset.byte_size) {
            (Some(digest), Some(size)) if valid_sha256(digest) && size > 0 => {}
            (Some(digest), Some(_)) if !valid_sha256(digest) => {
                return Err(DatasetIntakeError::InvalidSha256(asset.asset_id.clone()));
            }
            (Some(_), None) => {
                return Err(DatasetIntakeError::DigestRequiresByteSize(
                    asset.asset_id.clone(),
                ));
            }
            (None, Some(_)) => {
                return Err(DatasetIntakeError::ByteSizeWithoutDigest(
                    asset.asset_id.clone(),
                ));
            }
            _ => {}
        }
        if asset
            .licence_id
            .as_deref()
            .is_some_and(|value| !non_empty(value))
        {
            return Err(DatasetIntakeError::EmptyLicenceId(asset.asset_id.clone()));
        }
        if asset
            .licence_uri
            .as_deref()
            .is_some_and(|value| !non_empty(value))
        {
            return Err(DatasetIntakeError::EmptyLicenceUri(asset.asset_id.clone()));
        }
        if asset.coordinate_sensitivity == CoordinateSensitivity::ExactSensitive
            && asset.access_tier == AccessTier::Public
        {
            return Err(DatasetIntakeError::ExactSensitiveCoordinateMustNotBePublic(
                asset.asset_id.clone(),
            ));
        }
        if asset.access_tier != AccessTier::Public
            && asset
                .access_receipt_ref
                .as_deref()
                .is_none_or(|value| !non_empty(value))
        {
            return Err(DatasetIntakeError::AccessReceiptMissing(
                asset.asset_id.clone(),
            ));
        }
        if asset
            .access_receipt_ref
            .as_deref()
            .is_some_and(|value| !non_empty(value))
        {
            return Err(DatasetIntakeError::EmptyAccessReceipt(
                asset.asset_id.clone(),
            ));
        }

        let mut purposes = BTreeSet::new();
        for permission in &asset.permissions {
            if !purposes.insert(permission.purpose) {
                return Err(DatasetIntakeError::DuplicatePurposePermission(
                    asset.asset_id.clone(),
                ));
            }
            match permission.state {
                PermissionState::Granted => {
                    if permission
                        .evidence_ref
                        .as_deref()
                        .is_none_or(|value| !non_empty(value))
                    {
                        return Err(DatasetIntakeError::GrantedPermissionMissingEvidence(
                            asset.asset_id.clone(),
                        ));
                    }
                }
                PermissionState::Denied => {
                    if permission
                        .condition_summary
                        .as_deref()
                        .is_none_or(|value| !non_empty(value))
                    {
                        return Err(DatasetIntakeError::DeniedPermissionMissingCondition(
                            asset.asset_id.clone(),
                        ));
                    }
                    if permission
                        .evidence_ref
                        .as_deref()
                        .is_none_or(|value| !non_empty(value))
                    {
                        return Err(DatasetIntakeError::DeniedPermissionMissingEvidence(
                            asset.asset_id.clone(),
                        ));
                    }
                }
                PermissionState::Unknown => {}
            }
            if permission
                .evidence_ref
                .as_deref()
                .is_some_and(|value| !non_empty(value))
            {
                return Err(DatasetIntakeError::EmptyPermissionEvidence(
                    asset.asset_id.clone(),
                ));
            }
            if permission
                .condition_summary
                .as_deref()
                .is_some_and(|value| !non_empty(value))
            {
                return Err(DatasetIntakeError::EmptyPermissionCondition(
                    asset.asset_id.clone(),
                ));
            }
        }
    }

    let mut parent_by_child = std::collections::BTreeMap::<&str, &str>::new();
    for asset in &registry.assets {
        if let Some(parent_id) = asset.parent_asset_id.as_deref() {
            if !asset_ids.contains(parent_id) {
                return Err(DatasetIntakeError::ParentAssetReferenceMissing(
                    asset.asset_id.clone(),
                ));
            }
            parent_by_child.insert(&asset.asset_id, parent_id);
        }
    }
    for asset in &registry.assets {
        let mut visited = BTreeSet::new();
        let mut current = asset.asset_id.as_str();
        while let Some(parent) = parent_by_child.get(current) {
            if !visited.insert(current) {
                return Err(DatasetIntakeError::ParentAssetCycle(asset.asset_id.clone()));
            }
            current = parent;
        }
        if !visited.insert(current) {
            return Err(DatasetIntakeError::ParentAssetCycle(asset.asset_id.clone()));
        }
    }

    Ok(())
}

pub fn evaluate_asset_use(
    registry: &HeritageDatasetRegistry,
    asset_id: &str,
    purpose: UsePurpose,
) -> Result<AssetUseDecisionReport, DatasetIntakeError> {
    validate_heritage_dataset_registry(registry)?;
    let asset = registry
        .assets
        .iter()
        .find(|asset| asset.asset_id == asset_id)
        .ok_or_else(|| DatasetIntakeError::EmptyAssetField(asset_id.to_owned()))?;
    let package = registry
        .packages
        .iter()
        .find(|package| package.package_id == asset.package_id)
        .expect("registry validation guarantees package references");

    let permission = asset
        .permissions
        .iter()
        .find(|permission| permission.purpose == purpose);

    let (decision, reason, evidence_ref) = match permission {
        Some(permission) if permission.state == PermissionState::Granted => (
            UseDecision::AllowedByRecordedEvidence,
            "A granted permission with a recorded evidence reference is present; authenticity and legal interpretation remain external."
                .to_owned(),
            permission.evidence_ref.clone(),
        ),
        Some(permission) if permission.state == PermissionState::Denied => (
            UseDecision::DeniedByRecordedEvidence,
            permission
                .condition_summary
                .clone()
                .unwrap_or_else(|| "Use denied by recorded rights decision.".to_owned()),
            permission.evidence_ref.clone(),
        ),
        _ => (
            UseDecision::BlockedUnknownPermission,
            "No granted permission for this exact asset and purpose; public availability and software-code licences do not imply data-use permission."
                .to_owned(),
            None,
        ),
    };

    Ok(AssetUseDecisionReport {
        registry_id: registry.registry_id.clone(),
        package_id: package.package_id.clone(),
        asset_id: asset.asset_id.clone(),
        purpose,
        decision,
        reason,
        licence_id: asset.licence_id.clone(),
        rights_evidence_ref: evidence_ref,
        claim_ceiling: "recorded_permission_presence_only_not_legal_authentication".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(id: &str, kind: AssetKind, permissions: Vec<PurposePermission>) -> DatasetAssetRecord {
        DatasetAssetRecord {
            asset_id: id.to_owned(),
            package_id: "pkg-1".to_owned(),
            file_name: format!("{id}.zip"),
            kind,
            media_type: "application/zip".to_owned(),
            sha256: None,
            byte_size: None,
            licence_id: None,
            licence_uri: None,
            access_tier: AccessTier::Public,
            access_receipt_ref: None,
            coordinate_sensitivity: CoordinateSensitivity::None,
            permissions,
            parent_asset_id: None,
        }
    }

    fn granted(purpose: UsePurpose, evidence: &str) -> PurposePermission {
        PurposePermission {
            purpose,
            state: PermissionState::Granted,
            evidence_ref: Some(evidence.to_owned()),
            condition_summary: None,
        }
    }

    fn registry() -> HeritageDatasetRegistry {
        HeritageDatasetRegistry {
            schema_version: HERITAGE_DATA_INTAKE_SCHEMA_VERSION.to_owned(),
            registry_id: "heritage-registry-1".to_owned(),
            packages: vec![DatasetPackageRecord {
                package_id: "pkg-1".to_owned(),
                title: "Synthetic masonry package".to_owned(),
                version: "v1".to_owned(),
                source_uri: "https://example.org/dataset".to_owned(),
                citation: "Synthetic test package".to_owned(),
                availability: RepositoryAvailability::Downloadable,
            }],
            assets: vec![asset(
                "images-1",
                AssetKind::Imagery,
                vec![granted(
                    UsePurpose::LocalInspection,
                    "rights://local-inspection/1",
                )],
            )],
        }
    }

    #[test]
    fn public_download_does_not_grant_training_or_redistribution() {
        let registry = registry();
        let training = evaluate_asset_use(&registry, "images-1", UsePurpose::ResearchTraining)
            .expect("valid registry");
        let redistribution = evaluate_asset_use(&registry, "images-1", UsePurpose::Redistribution)
            .expect("valid registry");
        assert_eq!(training.decision, UseDecision::BlockedUnknownPermission);
        assert_eq!(
            redistribution.decision,
            UseDecision::BlockedUnknownPermission
        );
    }

    #[test]
    fn granted_permission_requires_asset_specific_evidence() {
        let mut registry = registry();
        registry.assets[0].permissions.push(PurposePermission {
            purpose: UsePurpose::ResearchTraining,
            state: PermissionState::Granted,
            evidence_ref: None,
            condition_summary: None,
        });
        assert!(matches!(
            validate_heritage_dataset_registry(&registry),
            Err(DatasetIntakeError::GrantedPermissionMissingEvidence(_))
        ));
    }

    #[test]
    fn code_license_does_not_grant_separate_dataset_asset() {
        let mut registry = registry();
        registry.assets[0].licence_id = Some("GPL-3.0-only".to_owned());
        registry.assets[0].kind = AssetKind::Imagery;
        let report = evaluate_asset_use(&registry, "images-1", UsePurpose::CommercialUse).unwrap();
        assert_eq!(report.decision, UseDecision::BlockedUnknownPermission);
        assert_eq!(report.licence_id.as_deref(), Some("GPL-3.0-only"));
    }

    #[test]
    fn explicitly_denied_commercial_use_is_not_overridden_by_research_permission() {
        let mut registry = registry();
        registry.assets[0].permissions.push(granted(
            UsePurpose::ResearchTraining,
            "rights://research-only/1",
        ));
        registry.assets[0].permissions.push(PurposePermission {
            purpose: UsePurpose::CommercialUse,
            state: PermissionState::Denied,
            evidence_ref: Some("rights://commercial-denial/1".to_owned()),
            condition_summary: Some("Non-commercial use only.".to_owned()),
        });
        let research =
            evaluate_asset_use(&registry, "images-1", UsePurpose::ResearchTraining).unwrap();
        let commercial =
            evaluate_asset_use(&registry, "images-1", UsePurpose::CommercialUse).unwrap();
        assert_eq!(research.decision, UseDecision::AllowedByRecordedEvidence);
        assert_eq!(commercial.decision, UseDecision::DeniedByRecordedEvidence);
    }

    #[test]
    fn duplicate_permission_purpose_is_rejected() {
        let mut registry = registry();
        registry.assets[0]
            .permissions
            .push(granted(UsePurpose::LocalInspection, "rights://second/1"));
        assert!(matches!(
            validate_heritage_dataset_registry(&registry),
            Err(DatasetIntakeError::DuplicatePurposePermission(_))
        ));
    }

    #[test]
    fn denied_permission_requires_a_rights_evidence_reference() {
        let mut registry = registry();
        registry.assets[0].permissions.push(PurposePermission {
            purpose: UsePurpose::Redistribution,
            state: PermissionState::Denied,
            evidence_ref: None,
            condition_summary: Some("Redistribution prohibited.".to_owned()),
        });
        assert!(matches!(
            validate_heritage_dataset_registry(&registry),
            Err(DatasetIntakeError::DeniedPermissionMissingEvidence(_))
        ));
    }

    #[test]
    fn restricted_asset_requires_an_access_receipt() {
        let mut registry = registry();
        registry.assets[0].access_tier = AccessTier::Controlled;
        assert!(matches!(
            validate_heritage_dataset_registry(&registry),
            Err(DatasetIntakeError::AccessReceiptMissing(_))
        ));
        registry.assets[0].access_receipt_ref = Some("custody://access/approved-1".to_owned());
        assert!(validate_heritage_dataset_registry(&registry).is_ok());
    }

    #[test]
    fn package_digest_and_size_must_be_recorded_together() {
        let mut registry = registry();
        registry.packages[0].package_sha256 = Some("a".repeat(64));
        assert!(matches!(
            validate_heritage_dataset_registry(&registry),
            Err(DatasetIntakeError::PackageDigestRequiresByteSize(_))
        ));
        registry.packages[0].package_byte_size = Some(512);
        assert!(validate_heritage_dataset_registry(&registry).is_ok());
    }

    #[test]
    fn exact_sensitive_coordinates_cannot_be_public() {
        let mut registry = registry();
        registry.assets[0].coordinate_sensitivity = CoordinateSensitivity::ExactSensitive;
        assert!(matches!(
            validate_heritage_dataset_registry(&registry),
            Err(DatasetIntakeError::ExactSensitiveCoordinateMustNotBePublic(
                _
            ))
        ));
    }

    #[test]
    fn asset_digest_and_byte_size_must_be_recorded_together() {
        let mut registry = registry();
        registry.assets[0].sha256 = Some("a".repeat(64));
        assert!(matches!(
            validate_heritage_dataset_registry(&registry),
            Err(DatasetIntakeError::DigestRequiresByteSize(_))
        ));
        registry.assets[0].byte_size = Some(128);
        assert!(validate_heritage_dataset_registry(&registry).is_ok());
    }

    #[test]
    fn code_and_data_assets_keep_separate_permission_records() {
        let mut registry = registry();
        let mut code = asset(
            "code-1",
            AssetKind::SourceCode,
            vec![granted(
                UsePurpose::ResearchTraining,
                "licence://GPL-3.0-only",
            )],
        );
        code.licence_id = Some("GPL-3.0-only".to_owned());
        code.parent_asset_id = Some("images-1".to_owned());
        registry.assets.push(code);
        let image_use =
            evaluate_asset_use(&registry, "images-1", UsePurpose::ResearchTraining).unwrap();
        let code_use =
            evaluate_asset_use(&registry, "code-1", UsePurpose::ResearchTraining).unwrap();
        assert_eq!(image_use.decision, UseDecision::BlockedUnknownPermission);
        assert_eq!(code_use.decision, UseDecision::AllowedByRecordedEvidence);
    }

    #[test]
    fn parent_asset_cycles_are_rejected() {
        let mut registry = registry();
        let mut code = asset("code-1", AssetKind::SourceCode, Vec::new());
        code.parent_asset_id = Some("images-1".to_owned());
        registry.assets[0].parent_asset_id = Some("code-1".to_owned());
        registry.assets.push(code);
        assert!(matches!(
            validate_heritage_dataset_registry(&registry),
            Err(DatasetIntakeError::ParentAssetCycle(_))
        ));
    }

    #[test]
    fn unknown_rights_remain_blocked_even_when_package_is_downloadable() {
        let mut registry = registry();
        registry.packages[0].availability = RepositoryAvailability::Downloadable;
        registry.assets[0].permissions.clear();
        let report =
            evaluate_asset_use(&registry, "images-1", UsePurpose::ResearchTraining).unwrap();
        assert_eq!(report.decision, UseDecision::BlockedUnknownPermission);
    }

    #[test]
    fn registry_schema_and_synthetic_fixture_are_valid_json() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/heritage-data-intake-v0.1.schema.json"
        ))
        .expect("schema JSON");
        let fixture: HeritageDatasetRegistry = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-data-intake-synthetic-v0.1.json"
        ))
        .expect("registry fixture");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(validate_heritage_dataset_registry(&fixture).is_ok());
    }
}
