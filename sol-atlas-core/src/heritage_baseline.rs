// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fail-closed input-channel and rights checks for heritage baseline runs.
//! This does not read raster pixels or interpret licence text.

use crate::heritage::{AccessContext, HeritageUse, RightsDenial, RightsProfile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelSemantic {
    Red, Green, Blue, NearInfrared, ShortwaveInfrared, Thermal,
    SarVv, SarVh, Elevation, ValidityMask, SiteGeometryMask, Other,
}

impl ChannelSemantic {
    fn is_mask(self) -> bool {
        matches!(self, Self::ValidityMask | Self::SiteGeometryMask)
    }

    fn is_physical_measurement(self) -> bool {
        matches!(self, Self::Red | Self::Green | Self::Blue | Self::NearInfrared
            | Self::ShortwaveInfrared | Self::Thermal | Self::SarVv
            | Self::SarVh | Self::Elevation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceChannel {
    pub index: usize,
    pub semantic: ChannelSemantic,
    pub units: String,
    pub data_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInputChannel {
    pub position: usize,
    pub semantic: ChannelSemantic,
    pub units: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelMapping {
    pub model_position: usize,
    pub source_channel_index: usize,
    pub source_semantic: ChannelSemantic,
    pub target_semantic: ChannelSemantic,
    pub transform_id: Option<String>,
    pub transform_sha256: Option<String>,
    pub reviewed: bool,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelDecision {
    Compatible,
    Adapted,
    Blocked,
    Unreviewed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelContractIssue {
    MissingModelChannelMapping { model_position: usize },
    ExtraModelChannelMapping { model_position: usize },
    DuplicateSourceChannelIndex { channel_index: usize },
    DuplicateModelChannelPosition { model_position: usize },
    SourceChannelNotFound { channel_index: usize },
    SourceSemanticDeclarationMismatch { channel_index: usize },
    TargetSemanticMismatch { model_position: usize },
    EmptySourceUnits { channel_index: usize },
    EmptyModelUnits { model_position: usize },
    MissingUnitTransform { model_position: usize },
    UnreviewedTransform { model_position: usize },
    InvalidTransformDigest { model_position: usize },
    EmptyTransformEvidence { model_position: usize },
    MaskCannotRepresentPhysicalBand {
        source_semantic: ChannelSemantic,
        target_semantic: ChannelSemantic,
    },
    SemanticChangeWithoutTransform { model_position: usize },
    CompatibleDecisionContainsTransform { model_position: usize },
    AdaptedDecisionHasNoTransformation,
    BlockedDecisionHasNoBlockingIssue,
    UnreviewedDecisionContainsReviewedMapping,
}

impl fmt::Display for ChannelContractIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "baseline channel contract issue: {self:?}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelContractReport {
    pub decision: ChannelDecision,
    pub source_channel_count: usize,
    pub model_input_channel_count: usize,
    pub mapping_count: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelContractError {
    EmptySourceChannels,
    EmptyModelInputContract,
    EmptyMappings,
    InvalidContract(Vec<ChannelContractIssue>),
}

impl fmt::Display for ChannelContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "baseline channel contract invalid: {self:?}")
    }
}

impl std::error::Error for ChannelContractError {}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Validate the mapping from source channel identities to model input roles.
///
/// Validity/site masks can never represent physical spectral, thermal, SAR or
/// elevation measurements. Unit/semantic transformations require a reviewed
/// transform ID, digest and evidence reference. A blocked decision is valid
/// only when at least one concrete incompatibility is recorded.
pub fn validate_channel_contract(
    source_channels: &[SourceChannel],
    model_channels: &[ModelInputChannel],
    mappings: &[ChannelMapping],
    decision: ChannelDecision,
) -> Result<ChannelContractReport, ChannelContractError> {
    if source_channels.is_empty() {
        return Err(ChannelContractError::EmptySourceChannels);
    }
    if model_channels.is_empty() {
        return Err(ChannelContractError::EmptyModelInputContract);
    }
    if mappings.is_empty() {
        return Err(ChannelContractError::EmptyMappings);
    }

    let mut issues = Vec::new();
    let mut source_indices = BTreeSet::new();
    for channel in source_channels {
        if !source_indices.insert(channel.index) {
            issues.push(ChannelContractIssue::DuplicateSourceChannelIndex {
                channel_index: channel.index,
            });
        }
        if channel.units.trim().is_empty() {
            issues.push(ChannelContractIssue::EmptySourceUnits {
                channel_index: channel.index,
            });
        }
    }

    let mut model_positions = BTreeSet::new();
    for channel in model_channels {
        if !model_positions.insert(channel.position) {
            issues.push(ChannelContractIssue::DuplicateModelChannelPosition {
                model_position: channel.position,
            });
        }
        if channel.units.trim().is_empty() {
            issues.push(ChannelContractIssue::EmptyModelUnits {
                model_position: channel.position,
            });
        }
    }

    let mut mapped_positions = BTreeSet::new();
    let mut has_transform = false;
    let mut has_unreviewed_mapping = false;

    for mapping in mappings {
        if !mapped_positions.insert(mapping.model_position) {
            issues.push(ChannelContractIssue::DuplicateModelChannelPosition {
                model_position: mapping.model_position,
            });
        }

        let Some(model_channel) = model_channels
            .iter()
            .find(|channel| channel.position == mapping.model_position)
        else {
            issues.push(ChannelContractIssue::ExtraModelChannelMapping {
                model_position: mapping.model_position,
            });
            continue;
        };

        if mapping.target_semantic != model_channel.semantic {
            issues.push(ChannelContractIssue::TargetSemanticMismatch {
                model_position: mapping.model_position,
            });
        }

        let Some(source_channel) = source_channels
            .iter()
            .find(|channel| channel.index == mapping.source_channel_index)
        else {
            issues.push(ChannelContractIssue::SourceChannelNotFound {
                channel_index: mapping.source_channel_index,
            });
            continue;
        };

        if mapping.source_semantic != source_channel.semantic {
            issues.push(ChannelContractIssue::SourceSemanticDeclarationMismatch {
                channel_index: mapping.source_channel_index,
            });
        }

        if source_channel.semantic.is_mask() && model_channel.semantic.is_physical_measurement() {
            issues.push(ChannelContractIssue::MaskCannotRepresentPhysicalBand {
                source_semantic: source_channel.semantic,
                target_semantic: model_channel.semantic,
            });
            continue;
        }

        let semantic_matches = source_channel.semantic == model_channel.semantic;
        let units_match = source_channel.units == model_channel.units;
        let requires_transform = !semantic_matches || !units_match;

        if requires_transform {
            has_transform = true;
            let transform_id_valid = mapping
                .transform_id
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            let transform_digest_valid = mapping
                .transform_sha256
                .as_deref()
                .is_some_and(is_sha256);

            if !transform_id_valid || !transform_digest_valid {
                if !semantic_matches {
                    issues.push(ChannelContractIssue::SemanticChangeWithoutTransform {
                        model_position: mapping.model_position,
                    });
                } else {
                    issues.push(ChannelContractIssue::MissingUnitTransform {
                        model_position: mapping.model_position,
                    });
                }
            }
            if !mapping.reviewed {
                has_unreviewed_mapping = true;
                issues.push(ChannelContractIssue::UnreviewedTransform {
                    model_position: mapping.model_position,
                });
            }
            if !transform_digest_valid && transform_id_valid {
                issues.push(ChannelContractIssue::InvalidTransformDigest {
                    model_position: mapping.model_position,
                });
            }
            if mapping.evidence_ref.trim().is_empty() {
                issues.push(ChannelContractIssue::EmptyTransformEvidence {
                    model_position: mapping.model_position,
                });
            }
        } else if mapping.transform_id.is_some() || mapping.transform_sha256.is_some() {
            has_transform = true;
            issues.push(ChannelContractIssue::CompatibleDecisionContainsTransform {
                model_position: mapping.model_position,
            });
        }

        if !mapping.reviewed {
            has_unreviewed_mapping = true;
            if !issues.iter().any(|issue| matches!(
                issue,
                ChannelContractIssue::UnreviewedTransform { model_position }
                    if *model_position == mapping.model_position
            )) {
                issues.push(ChannelContractIssue::UnreviewedTransform {
                    model_position: mapping.model_position,
                });
            }
        }
    }

    for model_channel in model_channels {
        if !mapped_positions.contains(&model_channel.position) {
            issues.push(ChannelContractIssue::MissingModelChannelMapping {
                model_position: model_channel.position,
            });
        }
    }

    match decision {
        ChannelDecision::Compatible => {
            if has_transform && issues.is_empty() {
                issues.push(ChannelContractIssue::CompatibleDecisionContainsTransform {
                    model_position: 0,
                });
            }
            if has_unreviewed_mapping && issues.is_empty() {
                issues.push(ChannelContractIssue::UnreviewedTransform {
                    model_position: 0,
                });
            }
            if !issues.is_empty() {
                return Err(ChannelContractError::InvalidContract(issues));
            }
        }
        ChannelDecision::Adapted => {
            if !issues.is_empty() {
                return Err(ChannelContractError::InvalidContract(issues));
            }
            if !has_transform {
                return Err(ChannelContractError::InvalidContract(vec![
                    ChannelContractIssue::AdaptedDecisionHasNoTransformation,
                ]));
            }
        }
        ChannelDecision::Blocked => {
            if issues.is_empty() {
                return Err(ChannelContractError::InvalidContract(vec![
                    ChannelContractIssue::BlockedDecisionHasNoBlockingIssue,
                ]));
            }
        }
        ChannelDecision::Unreviewed => {
            if !has_unreviewed_mapping {
                return Err(ChannelContractError::InvalidContract(vec![
                    ChannelContractIssue::UnreviewedDecisionContainsReviewedMapping,
                ]));
            }
        }
    }

    Ok(ChannelContractReport {
        decision,
        source_channel_count: source_channels.len(),
        model_input_channel_count: model_channels.len(),
        mapping_count: mappings.len(),
        issues: issues.iter().map(ToString::to_string).collect(),
    })
}

/// Authorize one baseline data use using the source-registry rights profile.
pub fn authorize_baseline_data_use(
    rights: &RightsProfile,
    purpose: HeritageUse,
    access: AccessContext,
) -> Result<(), RightsDenial> {
    rights.authorize(purpose, access, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb_plus_mask() -> Vec<SourceChannel> {
        vec![
            SourceChannel { index: 0, semantic: ChannelSemantic::Red, units: "8-bit display value".into(), data_type: "uint8".into() },
            SourceChannel { index: 1, semantic: ChannelSemantic::Green, units: "8-bit display value".into(), data_type: "uint8".into() },
            SourceChannel { index: 2, semantic: ChannelSemantic::Blue, units: "8-bit display value".into(), data_type: "uint8".into() },
            SourceChannel { index: 3, semantic: ChannelSemantic::ValidityMask, units: "0=invalid,255=valid".into(), data_type: "uint8".into() },
        ]
    }

    fn rgb_model() -> Vec<ModelInputChannel> {
        vec![
            ModelInputChannel { position: 0, semantic: ChannelSemantic::Red, units: "8-bit display value".into() },
            ModelInputChannel { position: 1, semantic: ChannelSemantic::Green, units: "8-bit display value".into() },
            ModelInputChannel { position: 2, semantic: ChannelSemantic::Blue, units: "8-bit display value".into() },
        ]
    }

    fn rgb_mapping() -> Vec<ChannelMapping> {
        (0..3).map(|index| {
            let semantic = [ChannelSemantic::Red, ChannelSemantic::Green, ChannelSemantic::Blue][index];
            ChannelMapping {
                model_position: index,
                source_channel_index: index,
                source_semantic: semantic,
                target_semantic: semantic,
                transform_id: None,
                transform_sha256: None,
                reviewed: true,
                evidence_ref: format!("urn:synthetic:channel-review:{index}"),
            }
        }).collect()
    }

    #[test]
    fn exact_rgb_contract_is_compatible() {
        let report = validate_channel_contract(
            &rgb_plus_mask(), &rgb_model(), &rgb_mapping(), ChannelDecision::Compatible
        ).unwrap();
        assert_eq!(report.decision, ChannelDecision::Compatible);
        assert_eq!(report.mapping_count, 3);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn validity_mask_cannot_satisfy_nir_input() {
        let mut model = rgb_model();
        model.push(ModelInputChannel {
            position: 3, semantic: ChannelSemantic::NearInfrared, units: "reflectance".into()
        });
        let mut mappings = rgb_mapping();
        mappings.push(ChannelMapping {
            model_position: 3,
            source_channel_index: 3,
            source_semantic: ChannelSemantic::ValidityMask,
            target_semantic: ChannelSemantic::NearInfrared,
            transform_id: None,
            transform_sha256: None,
            reviewed: false,
            evidence_ref: "synthetic negative control".into(),
        });

        let report = validate_channel_contract(
            &rgb_plus_mask(), &model, &mappings, ChannelDecision::Blocked
        ).unwrap();
        assert!(report.issues.iter().any(|issue| issue.contains("MaskCannotRepresentPhysicalBand")));
    }

    #[test]
    fn missing_nir_channel_is_rejected_for_compatible_model() {
        let mut model = rgb_model();
        model.push(ModelInputChannel {
            position: 3, semantic: ChannelSemantic::NearInfrared, units: "reflectance".into()
        });
        assert!(matches!(
            validate_channel_contract(&rgb_plus_mask(), &model, &rgb_mapping(), ChannelDecision::Compatible),
            Err(ChannelContractError::InvalidContract(_))
        ));
    }

    #[test]
    fn unit_conversion_requires_reviewed_transform_and_digest() {
        let source = vec![SourceChannel {
            index: 0, semantic: ChannelSemantic::Red, units: "uint8 DN".into(), data_type: "uint8".into()
        }];
        let model = vec![ModelInputChannel {
            position: 0, semantic: ChannelSemantic::Red, units: "surface reflectance".into()
        }];
        let invalid = vec![ChannelMapping {
            model_position: 0,
            source_channel_index: 0,
            source_semantic: ChannelSemantic::Red,
            target_semantic: ChannelSemantic::Red,
            transform_id: None,
            transform_sha256: None,
            reviewed: false,
            evidence_ref: String::new(),
        }];
        assert!(matches!(
            validate_channel_contract(&source, &model, &invalid, ChannelDecision::Adapted),
            Err(ChannelContractError::InvalidContract(_))
        ));

        let valid = vec![ChannelMapping {
            model_position: 0,
            source_channel_index: 0,
            source_semantic: ChannelSemantic::Red,
            target_semantic: ChannelSemantic::Red,
            transform_id: Some("uint8-to-reflectance-v1".into()),
            transform_sha256: Some("a".repeat(64)),
            reviewed: true,
            evidence_ref: "urn:synthetic:preprocessing-review:1".into(),
        }];
        assert_eq!(
            validate_channel_contract(&source, &model, &valid, ChannelDecision::Adapted).unwrap().decision,
            ChannelDecision::Adapted
        );
    }

    #[test]
    fn duplicate_or_missing_channel_mappings_fail_closed() {
        let mut duplicate = rgb_mapping();
        duplicate[1].model_position = 0;
        assert!(matches!(
            validate_channel_contract(&rgb_plus_mask(), &rgb_model(), &duplicate, ChannelDecision::Compatible),
            Err(ChannelContractError::InvalidContract(_))
        ));
    }

    #[test]
    fn a_transform_cannot_be_hidden_under_compatible_status() {
        let source = vec![SourceChannel {
            index: 0, semantic: ChannelSemantic::Red, units: "uint8 DN".into(), data_type: "uint8".into()
        }];
        let model = vec![ModelInputChannel {
            position: 0, semantic: ChannelSemantic::Red, units: "surface reflectance".into()
        }];
        let mapping = vec![ChannelMapping {
            model_position: 0,
            source_channel_index: 0,
            source_semantic: ChannelSemantic::Red,
            target_semantic: ChannelSemantic::Red,
            transform_id: Some("unreviewed-scaling".into()),
            transform_sha256: Some("a".repeat(64)),
            reviewed: true,
            evidence_ref: "urn:synthetic:transform:1".into(),
        }];
        assert!(matches!(
            validate_channel_contract(&source, &model, &mapping, ChannelDecision::Compatible),
            Err(ChannelContractError::InvalidContract(_))
        ));
    }

    #[test]
    fn blocked_decision_requires_a_concrete_issue() {
        assert!(matches!(
            validate_channel_contract(&rgb_plus_mask(), &rgb_model(), &rgb_mapping(), ChannelDecision::Blocked),
            Err(ChannelContractError::InvalidContract(_))
        ));
    }
}
