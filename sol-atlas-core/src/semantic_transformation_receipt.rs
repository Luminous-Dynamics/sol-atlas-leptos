// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Loss-accounted engineering transformation receipts.
//!
//! This module checks receipt structure, source/output inventory closure and
//! consistency of declared mapping metadata. It does not authenticate the
//! transformer, independently execute an equivalence oracle, or prove
//! engineering correctness.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const RECEIPT_SCHEMA: &str = "luminous.semantic-transformation-receipt.v0.1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRepresentation {
    pub native_artifact_id: String,
    pub payload_sha256: String,
    pub media_type: String,
    pub schema_id: String,
    pub schema_version: String,
    pub schema_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformationImplementation {
    pub implementation_id: String,
    pub implementation_version: String,
    pub implementation_build_sha256: String,
    pub mapping_profile_id: String,
    pub mapping_profile_sha256: String,
    pub configuration_sha256: String,
    pub parameters_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSemanticItem {
    pub semantic_id: String,
    pub semantic_kind: String,
    pub required_for_declared_purposes: bool,
    pub source_value_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSemanticItem {
    pub semantic_id: String,
    pub semantic_kind: String,
    pub origin: TargetSemanticOrigin,
    pub source_semantic_ids: Vec<String>,
    pub target_value_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetSemanticOrigin {
    SourceMapped,
    Derived,
    Generated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingDisposition {
    Preserved,
    Converted,
    Opaque,
    Dropped,
    Unsupported,
    Split,
    Merged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionKind {
    Unit,
    CoordinateFrame,
    Uncertainty,
    Enumeration,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rational {
    pub numerator: i128,
    pub denominator: i128,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitConversion {
    pub source_unit: String,
    pub target_unit: String,
    pub source_dimension: String,
    pub target_dimension: String,
    pub scale: Rational,
    pub offset: Rational,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateHandedness {
    RightHanded,
    LeftHanded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinateFrameConversion {
    pub source_crs_id: String,
    pub target_crs_id: String,
    pub source_frame_id: String,
    pub target_frame_id: String,
    pub source_axis_order: Vec<String>,
    pub target_axis_order: Vec<String>,
    pub source_handedness: CoordinateHandedness,
    pub target_handedness: CoordinateHandedness,
    pub source_vertical_datum: String,
    pub target_vertical_datum: String,
    pub operation_id: String,
    pub operation_parameters_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyBehavior {
    Preserved,
    Propagated,
    ConservativeEnvelope,
    Reduced,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UncertaintyConversion {
    pub source_model: String,
    pub target_model: String,
    pub behavior: UncertaintyBehavior,
    pub propagation_method: Option<String>,
    pub assumptions_sha256: Option<String>,
    pub covariance_sha256: Option<String>,
    pub target_bound_sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingMetadata {
    pub conversion_kind: Option<ConversionKind>,
    pub unit_conversion: Option<UnitConversion>,
    pub coordinate_conversion: Option<CoordinateFrameConversion>,
    pub uncertainty_conversion: Option<UncertaintyConversion>,
    pub rule_id: String,
    pub rule_version: String,
    pub rationale: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticItemMapping {
    pub source_semantic_id: String,
    pub disposition: MappingDisposition,
    pub target_semantic_ids: Vec<String>,
    pub mapping_group_id: Option<String>,
    pub metadata: MappingMetadata,
    pub loss_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleResult {
    Pass,
    Fail,
    Indeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquivalenceOracleEvidence {
    pub oracle_id: String,
    pub oracle_build_sha256: String,
    pub corpus_id: String,
    pub corpus_sha256: String,
    pub result_sha256: String,
    pub result: OracleResult,
    pub evaluated_at: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclaredTransformationClaim {
    EquivalentForDeclaredProfile,
    EquivalentWithDeclaredLoss,
    NotEquivalent,
    Indeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticTransformationReceipt {
    pub schema_version: String,
    pub receipt_id: String,
    pub declared_claim: DeclaredTransformationClaim,
    pub declared_purposes: Vec<String>,
    pub source: ArtifactRepresentation,
    pub target: ArtifactRepresentation,
    pub implementation: TransformationImplementation,
    pub source_inventory: Vec<SourceSemanticItem>,
    pub target_inventory: Vec<TargetSemanticItem>,
    pub mappings: Vec<SemanticItemMapping>,
    pub equivalence_oracle: Option<EquivalenceOracleEvidence>,
    pub action_authority: String,
    pub limitations: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformationReceiptDisposition {
    ReceiptConsistentForIndependentReview,
    DeclaredSemanticLoss,
    NotEquivalent,
    Indeterminate,
    InvalidReceipt,
}

impl TransformationReceiptDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReceiptConsistentForIndependentReview => {
                "receipt_consistent_for_independent_review"
            }
            Self::DeclaredSemanticLoss => "declared_semantic_loss",
            Self::NotEquivalent => "not_equivalent",
            Self::Indeterminate => "indeterminate",
            Self::InvalidReceipt => "invalid_receipt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TransformationFinding {
    pub severity: String,
    pub code: String,
    pub semantic_id: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransformationReceiptValidation {
    pub disposition: TransformationReceiptDisposition,
    pub receipt_id: String,
    pub receipt_representation_sha256: String,
    pub source_item_count: usize,
    pub target_item_count: usize,
    pub mapped_source_item_count: usize,
    pub unmapped_source_item_count: usize,
    pub decision_required_loss_count: usize,
    pub opaque_mapping_count: usize,
    pub dropped_mapping_count: usize,
    pub unsupported_mapping_count: usize,
    pub split_mapping_count: usize,
    pub merged_mapping_count: usize,
    pub oracle_evidence_present: bool,
    pub oracle_evidence_authenticated: String,
    pub semantic_equivalence: String,
    pub action_authority: String,
    pub claim_ceiling: String,
    pub findings: Vec<TransformationFinding>,
}

#[derive(Default)]
struct Findings {
    rows: BTreeSet<TransformationFinding>,
    decision_required_loss_count: usize,
    has_semantic_loss: bool,
    has_indeterminate: bool,
    not_equivalent: bool,
    invalid: bool,
}

impl Findings {
    fn push(
        &mut self,
        severity: &str,
        code: &str,
        semantic_id: &str,
        detail: impl Into<String>,
    ) {
        self.rows.insert(TransformationFinding {
            severity: severity.to_owned(),
            code: code.to_owned(),
            semantic_id: semantic_id.to_owned(),
            detail: detail.into(),
        });
    }

    fn invalid(&mut self, code: &str, id: &str, detail: impl Into<String>) {
        self.invalid = true;
        self.push("invalid", code, id, detail);
    }

    fn loss(&mut self, code: &str, id: &str, required: bool, detail: impl Into<String>) {
        self.has_semantic_loss = true;
        if required {
            self.decision_required_loss_count += 1;
        }
        self.push("loss", code, id, detail);
    }

    fn indeterminate(&mut self, code: &str, id: &str, detail: impl Into<String>) {
        self.has_indeterminate = true;
        self.push("incomplete", code, id, detail);
    }

    fn not_equivalent(&mut self, code: &str, id: &str, detail: impl Into<String>) {
        self.not_equivalent = true;
        self.push("invalid", code, id, detail);
    }

    fn disposition(&self) -> TransformationReceiptDisposition {
        if self.invalid {
            TransformationReceiptDisposition::InvalidReceipt
        } else if self.not_equivalent {
            TransformationReceiptDisposition::NotEquivalent
        } else if self.has_semantic_loss {
            TransformationReceiptDisposition::DeclaredSemanticLoss
        } else if self.has_indeterminate {
            TransformationReceiptDisposition::Indeterminate
        } else {
            TransformationReceiptDisposition::ReceiptConsistentForIndependentReview
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        if !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit() {
            return false;
        }
    }
    let parse = |start, end| {
        std::str::from_utf8(&bytes[start..end])
            .ok()
            .and_then(|part| part.parse::<u32>().ok())
    };
    let year = parse(0, 4).unwrap_or(0);
    let month = parse(5, 7).unwrap_or(0);
    let day = parse(8, 10).unwrap_or(0);
    let hour = parse(11, 13).unwrap_or(99);
    let minute = parse(14, 16).unwrap_or(99);
    let second = parse(17, 19).unwrap_or(99);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    day > 0 && day <= max_day
}

fn validate_artifact(
    artifact: &ArtifactRepresentation,
    label: &str,
    findings: &mut Findings,
) {
    for (field, value) in [
        ("native_artifact_id", artifact.native_artifact_id.as_str()),
        ("media_type", artifact.media_type.as_str()),
        ("schema_id", artifact.schema_id.as_str()),
        ("schema_version", artifact.schema_version.as_str()),
    ] {
        if !non_empty(value) {
            findings.invalid(
                "ARTIFACT_REPRESENTATION_FIELD_EMPTY",
                label,
                format!("{field} is required"),
            );
        }
    }
    for (field, value) in [
        ("payload_sha256", artifact.payload_sha256.as_str()),
        ("schema_sha256", artifact.schema_sha256.as_str()),
    ] {
        if !valid_sha256(value) {
            findings.invalid(
                "ARTIFACT_REPRESENTATION_DIGEST_INVALID",
                label,
                format!("{field} must be lowercase SHA-256"),
            );
        }
    }
}

fn validate_rational(
    value: &Rational,
    semantic_id: &str,
    name: &str,
    findings: &mut Findings,
) {
    if value.denominator <= 0 {
        findings.invalid(
            "RATIONAL_DENOMINATOR_NOT_POSITIVE",
            semantic_id,
            format!("{name} denominator must be positive"),
        );
    }
}

fn valid_axis_order(order: &[String]) -> bool {
    order.len() == 3
        && order
            .iter()
            .all(|axis| matches!(axis.as_str(), "x" | "y" | "z" | "east" | "north" | "up"))
        && order.iter().collect::<BTreeSet<_>>().len() == 3
}

fn validate_mapping_metadata(
    mapping: &SemanticItemMapping,
    required_for_declared_purposes: bool,
    findings: &mut Findings,
) {
    let id = &mapping.source_semantic_id;
    if !non_empty(&mapping.metadata.rule_id)
        || !non_empty(&mapping.metadata.rule_version)
        || !non_empty(&mapping.metadata.rationale)
    {
        findings.invalid(
            "MAPPING_RULE_METADATA_INCOMPLETE",
            id,
            "rule ID, version and rationale are required",
        );
    }

    match mapping.metadata.conversion_kind {
        Some(ConversionKind::Unit) => {
            let Some(unit) = mapping.metadata.unit_conversion.as_ref() else {
                findings.invalid(
                    "UNIT_CONVERSION_METADATA_MISSING",
                    id,
                    "unit conversion requires exact units, dimensions, scale and offset",
                );
                return;
            };
            for (field, value) in [
                ("source_unit", unit.source_unit.as_str()),
                ("target_unit", unit.target_unit.as_str()),
                ("source_dimension", unit.source_dimension.as_str()),
                ("target_dimension", unit.target_dimension.as_str()),
            ] {
                if !non_empty(value) {
                    findings.invalid(
                        "UNIT_CONVERSION_FIELD_EMPTY",
                        id,
                        format!("{field} is required"),
                    );
                }
            }
            if unit.source_dimension != unit.target_dimension {
                findings.not_equivalent(
                    "UNIT_DIMENSION_CHANGED",
                    id,
                    "unit conversion changes the physical dimension",
                );
            }
            validate_rational(&unit.scale, id, "scale", findings);
            validate_rational(&unit.offset, id, "offset", findings);
        }
        Some(ConversionKind::CoordinateFrame) => {
            let Some(coordinate) = mapping.metadata.coordinate_conversion.as_ref() else {
                findings.invalid(
                    "COORDINATE_CONVERSION_METADATA_MISSING",
                    id,
                    "coordinate conversion requires source/target CRS, frames, axis order, handedness, datum and operation",
                );
                return;
            };
            for (field, value) in [
                ("source_crs_id", coordinate.source_crs_id.as_str()),
                ("target_crs_id", coordinate.target_crs_id.as_str()),
                ("source_frame_id", coordinate.source_frame_id.as_str()),
                ("target_frame_id", coordinate.target_frame_id.as_str()),
                ("source_vertical_datum", coordinate.source_vertical_datum.as_str()),
                ("target_vertical_datum", coordinate.target_vertical_datum.as_str()),
                ("operation_id", coordinate.operation_id.as_str()),
            ] {
                if !non_empty(value) {
                    findings.invalid(
                        "COORDINATE_CONVERSION_FIELD_EMPTY",
                        id,
                        format!("{field} is required"),
                    );
                }
            }
            if !valid_axis_order(&coordinate.source_axis_order)
                || !valid_axis_order(&coordinate.target_axis_order)
            {
                findings.invalid(
                    "COORDINATE_AXIS_ORDER_INVALID",
                    id,
                    "source and target axis orders must each identify three distinct supported axes",
                );
            }
            if !valid_sha256(&coordinate.operation_parameters_sha256) {
                findings.invalid(
                    "COORDINATE_OPERATION_PARAMETERS_DIGEST_INVALID",
                    id,
                    "coordinate operation parameters must be bound by a lowercase SHA-256",
                );
            }
        }
        Some(ConversionKind::Uncertainty) => {
            let Some(uncertainty) = mapping.metadata.uncertainty_conversion.as_ref() else {
                findings.invalid(
                    "UNCERTAINTY_CONVERSION_METADATA_MISSING",
                    id,
                    "uncertainty conversion requires source/target models and an explicit behavior",
                );
                return;
            };
            if !non_empty(&uncertainty.source_model) || !non_empty(&uncertainty.target_model) {
                findings.invalid(
                    "UNCERTAINTY_MODEL_EMPTY",
                    id,
                    "source and target uncertainty models are required",
                );
            }
            match uncertainty.behavior {
                UncertaintyBehavior::Preserved => {}
                UncertaintyBehavior::Propagated => {
                    if uncertainty
                        .propagation_method
                        .as_deref()
                        .is_none_or(|value| !non_empty(value))
                        || uncertainty
                            .assumptions_sha256
                            .as_deref()
                            .is_none_or(|value| !valid_sha256(value))
                    {
                        findings.indeterminate(
                            "UNCERTAINTY_PROPAGATION_EVIDENCE_INCOMPLETE",
                            id,
                            "propagated uncertainty requires a method and assumptions digest",
                        );
                    }
                }
                UncertaintyBehavior::ConservativeEnvelope => {
                    if uncertainty
                        .target_bound_sha256
                        .as_deref()
                        .is_none_or(|value| !valid_sha256(value))
                    {
                        findings.indeterminate(
                            "UNCERTAINTY_ENVELOPE_EVIDENCE_INCOMPLETE",
                            id,
                            "a conservative envelope requires an exact target-bound digest",
                        );
                    }
                }
                UncertaintyBehavior::Reduced | UncertaintyBehavior::Unknown => {
                    findings.loss(
                        "UNCERTAINTY_REDUCED_OR_UNKNOWN",
                        id,
                        required_for_declared_purposes,
                        "uncertainty was reduced or its propagation behavior is unknown",
                    );
                }
            }
        }
        Some(ConversionKind::Enumeration | ConversionKind::Other) => {}
        None => {
            if mapping.metadata.unit_conversion.is_some()
                || mapping.metadata.coordinate_conversion.is_some()
                || mapping.metadata.uncertainty_conversion.is_some()
            {
                findings.invalid(
                    "CONVERSION_METADATA_KIND_MISSING",
                    id,
                    "typed conversion metadata is present without a conversion kind",
                );
            }
        }
    }

    if mapping.metadata.conversion_kind != Some(ConversionKind::Unit)
        && mapping.metadata.unit_conversion.is_some()
    {
        findings.invalid(
            "UNIT_CONVERSION_KIND_MISMATCH",
            id,
            "unit conversion metadata must use conversion_kind=unit",
        );
    }
    if mapping.metadata.conversion_kind != Some(ConversionKind::CoordinateFrame)
        && mapping.metadata.coordinate_conversion.is_some()
    {
        findings.invalid(
            "COORDINATE_CONVERSION_KIND_MISMATCH",
            id,
            "coordinate conversion metadata must use conversion_kind=coordinate_frame",
        );
    }
    if mapping.metadata.conversion_kind != Some(ConversionKind::Uncertainty)
        && mapping.metadata.uncertainty_conversion.is_some()
    {
        findings.invalid(
            "UNCERTAINTY_CONVERSION_KIND_MISMATCH",
            id,
            "uncertainty metadata must use conversion_kind=uncertainty",
        );
    }
}

pub fn validate_semantic_transformation_receipt(
    receipt: &SemanticTransformationReceipt,
) -> TransformationReceiptValidation {
    let mut findings = Findings::default();
    if receipt.schema_version != RECEIPT_SCHEMA {
        findings.invalid(
            "UNSUPPORTED_RECEIPT_SCHEMA",
            "receipt",
            "unsupported semantic transformation receipt schema version",
        );
    }
    if !non_empty(&receipt.receipt_id) {
        findings.invalid("RECEIPT_ID_EMPTY", "receipt", "receipt ID is required");
    }
    if receipt.declared_purposes.is_empty()
        || receipt.declared_purposes.iter().any(|purpose| !non_empty(purpose))
        || receipt.declared_purposes.iter().collect::<BTreeSet<_>>().len()
            != receipt.declared_purposes.len()
    {
        findings.invalid(
            "DECLARED_PURPOSES_INVALID",
            "receipt",
            "declared purposes must be non-empty and unique",
        );
    }
    if receipt.action_authority != "none" {
        findings.invalid(
            "ACTION_AUTHORITY_ESCALATION",
            "receipt",
            "a transformation receipt cannot grant action authority",
        );
    }
    if receipt.limitations.is_empty() || receipt.limitations.iter().any(|item| !non_empty(item)) {
        findings.invalid(
            "LIMITATIONS_MISSING",
            "receipt",
            "receipt must state at least one limitation",
        );
    }

    validate_artifact(&receipt.source, "source", &mut findings);
    validate_artifact(&receipt.target, "target", &mut findings);

    for (field, value) in [
        ("implementation_id", receipt.implementation.implementation_id.as_str()),
        ("implementation_version", receipt.implementation.implementation_version.as_str()),
        ("mapping_profile_id", receipt.implementation.mapping_profile_id.as_str()),
    ] {
        if !non_empty(value) {
            findings.invalid(
                "IMPLEMENTATION_METADATA_EMPTY",
                "implementation",
                format!("{field} is required"),
            );
        }
    }
    for (field, value) in [
        (
            "implementation_build_sha256",
            receipt.implementation.implementation_build_sha256.as_str(),
        ),
        (
            "mapping_profile_sha256",
            receipt.implementation.mapping_profile_sha256.as_str(),
        ),
        (
            "configuration_sha256",
            receipt.implementation.configuration_sha256.as_str(),
        ),
        (
            "parameters_sha256",
            receipt.implementation.parameters_sha256.as_str(),
        ),
    ] {
        if !valid_sha256(value) {
            findings.invalid(
                "IMPLEMENTATION_DIGEST_INVALID",
                "implementation",
                format!("{field} must be lowercase SHA-256"),
            );
        }
    }

    let mut source_by_id = BTreeMap::new();
    for item in &receipt.source_inventory {
        if !non_empty(&item.semantic_id)
            || !non_empty(&item.semantic_kind)
            || !valid_sha256(&item.source_value_sha256)
        {
            findings.invalid(
                "SOURCE_SEMANTIC_ITEM_INVALID",
                &item.semantic_id,
                "source semantic item requires ID, kind and source value digest",
            );
        }
        if source_by_id.insert(item.semantic_id.as_str(), item).is_some() {
            findings.invalid(
                "DUPLICATE_SOURCE_SEMANTIC_ID",
                &item.semantic_id,
                "source semantic IDs must be unique",
            );
        }
    }

    let mut target_by_id = BTreeMap::new();
    for item in &receipt.target_inventory {
        if !non_empty(&item.semantic_id)
            || !non_empty(&item.semantic_kind)
            || !valid_sha256(&item.target_value_sha256)
        {
            findings.invalid(
                "TARGET_SEMANTIC_ITEM_INVALID",
                &item.semantic_id,
                "target semantic item requires ID, kind and target value digest",
            );
        }
        if target_by_id.insert(item.semantic_id.as_str(), item).is_some() {
            findings.invalid(
                "DUPLICATE_TARGET_SEMANTIC_ID",
                &item.semantic_id,
                "target semantic IDs must be unique",
            );
        }
        let unique_sources = item.source_semantic_ids.iter().collect::<BTreeSet<_>>();
        if unique_sources.len() != item.source_semantic_ids.len() {
            findings.invalid(
                "DUPLICATE_TARGET_SOURCE_REFERENCE",
                &item.semantic_id,
                "target source semantic references must be unique",
            );
        }
        if item.origin == TargetSemanticOrigin::SourceMapped && item.source_semantic_ids.is_empty() {
            findings.invalid(
                "SOURCE_MAPPED_TARGET_WITHOUT_SOURCE",
                &item.semantic_id,
                "source_mapped target items must identify at least one source semantic item",
            );
        }
    }

    let mut mapping_by_source = BTreeMap::new();
    let mut mapped_source_count = 0usize;
    let mut unmapped_decision_required_loss_count = 0usize;
    let mut opaque_count = 0usize;
    let mut dropped_count = 0usize;
    let mut unsupported_count = 0usize;
    let mut split_count = 0usize;
    let mut merged_count = 0usize;

    for mapping in &receipt.mappings {
        let id = &mapping.source_semantic_id;
        let Some(source_item) = source_by_id.get(id.as_str()).copied() else {
            findings.invalid(
                "MAPPING_SOURCE_NOT_IN_INVENTORY",
                id,
                "mapping references a source semantic item that is not in the source inventory",
            );
            continue;
        };
        if mapping_by_source.insert(id.as_str(), mapping).is_some() {
            findings.invalid(
                "DUPLICATE_SOURCE_MAPPING",
                id,
                "each source semantic item must have exactly one mapping disposition",
            );
            continue;
        }
        mapped_source_count += 1;

        let unique_targets = mapping.target_semantic_ids.iter().collect::<BTreeSet<_>>();
        if unique_targets.len() != mapping.target_semantic_ids.len() {
            findings.invalid(
                "DUPLICATE_TARGET_MAPPING",
                id,
                "target semantic IDs in one mapping must be unique",
            );
        }
        for target_id in &mapping.target_semantic_ids {
            if !target_by_id.contains_key(target_id.as_str()) {
                findings.invalid(
                    "MAPPING_TARGET_NOT_IN_INVENTORY",
                    id,
                    format!("target semantic item {target_id} is not in the target inventory"),
                );
            }
        }

        match mapping.disposition {
            MappingDisposition::Preserved => {
                if mapping.target_semantic_ids.len() != 1 {
                    findings.invalid(
                        "PRESERVED_MAPPING_CARDINALITY_INVALID",
                        id,
                        "preserved mapping requires exactly one target semantic item",
                    );
                }
                if mapping.loss_reason.is_some() {
                    findings.invalid(
                        "PRESERVED_MAPPING_HAS_LOSS_REASON",
                        id,
                        "preserved mapping cannot carry a loss reason",
                    );
                }
            }
            MappingDisposition::Converted => {
                if mapping.target_semantic_ids.len() != 1 {
                    findings.invalid(
                        "CONVERTED_MAPPING_CARDINALITY_INVALID",
                        id,
                        "converted mapping requires exactly one target semantic item",
                    );
                }
                if mapping.loss_reason.is_some() {
                    findings.invalid(
                        "CONVERTED_MAPPING_HAS_LOSS_REASON",
                        id,
                        "converted mapping must preserve semantics or describe a typed conversion; use a loss disposition otherwise",
                    );
                }
                if mapping.metadata.conversion_kind.is_none() {
                    findings.indeterminate(
                        "CONVERSION_KIND_UNSPECIFIED",
                        id,
                        "converted mapping has no typed conversion kind",
                    );
                }
            }
            MappingDisposition::Opaque => {
                opaque_count += 1;
                if mapping.target_semantic_ids.len() != 1 {
                    findings.invalid(
                        "OPAQUE_MAPPING_CARDINALITY_INVALID",
                        id,
                        "opaque mapping requires exactly one target semantic item",
                    );
                }
                if mapping.loss_reason.as_deref().is_none_or(|reason| !non_empty(reason)) {
                    findings.invalid(
                        "OPAQUE_MAPPING_REASON_MISSING",
                        id,
                        "opaque mapping requires an explicit reason",
                    );
                }
                findings.loss(
                    "OPAQUE_SEMANTIC_MAPPING",
                    id,
                    source_item.required_for_declared_purposes,
                    "source semantic content is retained without a validated interpretation",
                );
            }
            MappingDisposition::Dropped | MappingDisposition::Unsupported => {
                if !mapping.target_semantic_ids.is_empty() {
                    findings.invalid(
                        "UNREPRESENTED_MAPPING_HAS_TARGET",
                        id,
                        "dropped/unsupported mapping must not invent target semantic IDs",
                    );
                }
                if mapping.loss_reason.as_deref().is_none_or(|reason| !non_empty(reason)) {
                    findings.invalid(
                        "SEMANTIC_LOSS_REASON_MISSING",
                        id,
                        "dropped/unsupported mapping requires an explicit loss reason",
                    );
                }
                if mapping.disposition == MappingDisposition::Dropped {
                    dropped_count += 1;
                } else {
                    unsupported_count += 1;
                }
                findings.loss(
                    if mapping.disposition == MappingDisposition::Dropped {
                        "SOURCE_SEMANTIC_DROPPED"
                    } else {
                        "SOURCE_SEMANTIC_UNSUPPORTED"
                    },
                    id,
                    source_item.required_for_declared_purposes,
                    mapping.loss_reason.as_deref().unwrap_or("no loss reason supplied"),
                );
            }
            MappingDisposition::Split => {
                split_count += 1;
                if mapping.target_semantic_ids.len() < 2
                    || mapping.mapping_group_id.as_deref().is_none_or(|value| !non_empty(value))
                {
                    findings.invalid(
                        "SPLIT_MAPPING_GROUP_INVALID",
                        id,
                        "split mapping requires at least two target items and a mapping group ID",
                    );
                }
                if mapping.loss_reason.is_some() {
                    findings.loss(
                        "SPLIT_MAPPING_HAS_DECLARED_LOSS",
                        id,
                        source_item.required_for_declared_purposes,
                        mapping.loss_reason.as_deref().unwrap_or("split mapping has declared loss"),
                    );
                }
            }
            MappingDisposition::Merged => {
                merged_count += 1;
                if mapping.target_semantic_ids.len() != 1
                    || mapping.mapping_group_id.as_deref().is_none_or(|value| !non_empty(value))
                {
                    findings.invalid(
                        "MERGED_MAPPING_GROUP_INVALID",
                        id,
                        "merged mapping requires one target item and a mapping group ID",
                    );
                }
                if mapping.loss_reason.is_some() {
                    findings.loss(
                        "MERGED_MAPPING_HAS_DECLARED_LOSS",
                        id,
                        source_item.required_for_declared_purposes,
                        mapping.loss_reason.as_deref().unwrap_or("merged mapping has declared loss"),
                    );
                }
            }
        }
        validate_mapping_metadata(
            mapping,
            source_item.required_for_declared_purposes,
            &mut findings,
        );
    }

    let unmapped_source_count = source_by_id
        .keys()
        .filter(|source_id| !mapping_by_source.contains_key(**source_id))
        .count();
    for source_id in source_by_id.keys() {
        if !mapping_by_source.contains_key(*source_id) {
            let source = source_by_id[*source_id];
            findings.indeterminate(
                "SOURCE_SEMANTIC_ITEM_UNMAPPED",
                source.semantic_id.as_str(),
                "source semantic inventory item has no mapping disposition",
            );
            if source.required_for_declared_purposes {
                unmapped_decision_required_loss_count += 1;
            }
        }
    }

    for target in &receipt.target_inventory {
        match target.origin {
            TargetSemanticOrigin::SourceMapped => {
                for source_id in &target.source_semantic_ids {
                    let Some(mapping) = mapping_by_source.get(source_id.as_str()) else {
                        findings.invalid(
                            "TARGET_REFERENCES_UNMAPPED_SOURCE",
                            &target.semantic_id,
                            format!("target references unmapped source semantic item {source_id}"),
                        );
                        continue;
                    };
                    if !mapping.target_semantic_ids.iter().any(|target_id| target_id == &target.semantic_id) {
                        findings.invalid(
                            "TARGET_SOURCE_MAPPING_NOT_RECIPROCAL",
                            &target.semantic_id,
                            format!("source mapping for {source_id} does not name this target item"),
                        );
                    }
                }
            }
            TargetSemanticOrigin::Derived | TargetSemanticOrigin::Generated => {
                if target.source_semantic_ids.iter().any(|id| !source_by_id.contains_key(id.as_str())) {
                    findings.invalid(
                        "DERIVED_TARGET_SOURCE_UNKNOWN",
                        &target.semantic_id,
                        "derived/generated target references an unknown source semantic item",
                    );
                }
            }
        }
    }

    if let Some(oracle) = &receipt.equivalence_oracle {
        for (name, value) in [
            ("oracle_id", oracle.oracle_id.as_str()),
            ("corpus_id", oracle.corpus_id.as_str()),
        ] {
            if !non_empty(value) {
                findings.invalid(
                    "ORACLE_METADATA_EMPTY",
                    "equivalence_oracle",
                    format!("{name} is required"),
                );
            }
        }
        for (name, value) in [
            ("oracle_build_sha256", oracle.oracle_build_sha256.as_str()),
            ("corpus_sha256", oracle.corpus_sha256.as_str()),
            ("result_sha256", oracle.result_sha256.as_str()),
        ] {
            if !valid_sha256(value) {
                findings.invalid(
                    "ORACLE_DIGEST_INVALID",
                    "equivalence_oracle",
                    format!("{name} must be lowercase SHA-256"),
                );
            }
        }
        if !valid_utc_timestamp(&oracle.evaluated_at) {
            findings.invalid(
                "ORACLE_EVALUATION_TIME_INVALID",
                "equivalence_oracle",
                "oracle evaluation time must be canonical UTC whole-second timestamp",
            );
        }
        match oracle.result {
            OracleResult::Pass => findings.indeterminate(
                "ORACLE_RESULT_NOT_AUTHENTICATED_HERE",
                "equivalence_oracle",
                "oracle pass is a producer-supplied claim; this validator does not authenticate or execute the oracle",
            ),
            OracleResult::Fail => findings.not_equivalent(
                "INDEPENDENT_ORACLE_REPORTED_FAILURE",
                "equivalence_oracle",
                "the referenced equivalence oracle reports failure",
            ),
            OracleResult::Indeterminate => findings.indeterminate(
                "INDEPENDENT_ORACLE_INDETERMINATE",
                "equivalence_oracle",
                "the referenced equivalence oracle reports indeterminate",
            ),
        }
    } else if receipt.declared_claim == DeclaredTransformationClaim::EquivalentForDeclaredProfile {
        findings.invalid(
            "EQUIVALENCE_CLAIM_WITHOUT_ORACLE_EVIDENCE",
            "equivalence_oracle",
            "equivalent_for_declared_profile requires an external oracle result reference",
        );
    } else {
        findings.indeterminate(
            "ORACLE_EVIDENCE_NOT_SUPPLIED",
            "equivalence_oracle",
            "no independent equivalence oracle result is referenced",
        );
    }

    match receipt.declared_claim {
        DeclaredTransformationClaim::EquivalentForDeclaredProfile => {
            if findings.has_semantic_loss || findings.has_decision_required_loss {
                findings.not_equivalent(
                    "EQUIVALENCE_CLAIM_CONTRADICTS_LOSS",
                    "receipt",
                    "receipt claims full equivalence while its mapping inventory declares semantic loss",
                );
            }
        }
        DeclaredTransformationClaim::EquivalentWithDeclaredLoss => {
            if !findings.has_semantic_loss {
                findings.invalid(
                    "DECLARED_LOSS_WITHOUT_LOSS_RECORD",
                    "receipt",
                    "equivalent_with_declared_loss requires at least one explicit loss mapping",
                );
            }
        }
        DeclaredTransformationClaim::NotEquivalent => {
            findings.not_equivalent(
                "TRANSFORMER_DECLARED_NOT_EQUIVALENT",
                "receipt",
                "transformer declares that the output is not semantically equivalent",
            );
        }
        DeclaredTransformationClaim::Indeterminate => {
            findings.indeterminate(
                "TRANSFORMER_DECLARED_INDETERMINATE",
                "receipt",
                "transformer does not claim semantic equivalence",
            );
        }
    }

    let decision_required_loss_count =
        findings.decision_required_loss_count + unmapped_decision_required_loss_count;
    let disposition = findings.disposition();
    let findings_vec = findings.rows.into_iter().collect::<Vec<_>>();
    let receipt_representation_sha256 = serde_json::to_vec(receipt)
        .map(|bytes| sha256_hex(&bytes))
        .unwrap_or_default();

    TransformationReceiptValidation {
        disposition,
        receipt_id: receipt.receipt_id.clone(),
        receipt_representation_sha256,
        source_item_count: receipt.source_inventory.len(),
        target_item_count: receipt.target_inventory.len(),
        mapped_source_item_count: mapped_source_count,
        unmapped_source_item_count: unmapped_source_count,
        decision_required_loss_count,
        opaque_mapping_count: opaque_count,
        dropped_mapping_count: dropped_count,
        unsupported_mapping_count: unsupported_count,
        split_mapping_count: split_count,
        merged_mapping_count: merged_count,
        oracle_evidence_present: receipt.equivalence_oracle.is_some(),
        oracle_evidence_authenticated: "not_performed".into(),
        semantic_equivalence: "not_evaluated_by_receipt_validator".into(),
        action_authority: "none".into(),
        claim_ceiling: "mapping_integrity_only".into(),
        findings: findings_vec,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn artifact(id: &str, value: char, schema: &str) -> ArtifactRepresentation {
        ArtifactRepresentation {
            native_artifact_id: id.into(),
            payload_sha256: digest(value),
            media_type: "application/json".into(),
            schema_id: schema.into(),
            schema_version: "1.0".into(),
            schema_sha256: digest('f'),
        }
    }

    fn mapping(id: &str, target: &str, disposition: MappingDisposition) -> SemanticItemMapping {
        SemanticItemMapping {
            source_semantic_id: id.into(),
            disposition,
            target_semantic_ids: if target.is_empty() {
                Vec::new()
            } else {
                vec![target.into()]
            },
            mapping_group_id: None,
            metadata: MappingMetadata {
                conversion_kind: None,
                unit_conversion: None,
                coordinate_conversion: None,
                uncertainty_conversion: None,
                rule_id: "rule:identity".into(),
                rule_version: "1.0".into(),
                rationale: "Exact identity mapping in the synthetic fixture".into(),
            },
            loss_reason: None,
        }
    }

    fn receipt() -> SemanticTransformationReceipt {
        SemanticTransformationReceipt {
            schema_version: RECEIPT_SCHEMA.into(),
            receipt_id: "receipt-transform-001".into(),
            declared_claim: DeclaredTransformationClaim::EquivalentForDeclaredProfile,
            declared_purposes: vec!["engineering_review".into(), "impact_analysis".into()],
            source: artifact("source-ifc-model-001", 'a', "urn:synthetic:ifc-schema"),
            target: artifact("target-native-graph-001", 'b', "urn:synthetic:ieac-schema"),
            implementation: TransformationImplementation {
                implementation_id: "sol-atlas-ifc-adapter".into(),
                implementation_version: "0.1.0".into(),
                implementation_build_sha256: digest('c'),
                mapping_profile_id: "ifc-to-ieac-core".into(),
                mapping_profile_sha256: digest('d'),
                configuration_sha256: digest('e'),
                parameters_sha256: digest('f'),
            },
            source_inventory: vec![
                SourceSemanticItem {
                    semantic_id: "wall:W-001:global-placement".into(),
                    semantic_kind: "coordinate_frame".into(),
                    required_for_declared_purposes: true,
                    source_value_sha256: digest('1'),
                },
                SourceSemanticItem {
                    semantic_id: "wall:W-001:fire-rating".into(),
                    semantic_kind: "property".into(),
                    required_for_declared_purposes: true,
                    source_value_sha256: digest('2'),
                },
            ],
            target_inventory: vec![
                TargetSemanticItem {
                    semantic_id: "node:wall-001:placement".into(),
                    semantic_kind: "coordinate_frame".into(),
                    origin: TargetSemanticOrigin::SourceMapped,
                    source_semantic_ids: vec!["wall:W-001:global-placement".into()],
                    target_value_sha256: digest('3'),
                },
                TargetSemanticItem {
                    semantic_id: "node:wall-001:fire-rating".into(),
                    semantic_kind: "property".into(),
                    origin: TargetSemanticOrigin::SourceMapped,
                    source_semantic_ids: vec!["wall:W-001:fire-rating".into()],
                    target_value_sha256: digest('4'),
                },
            ],
            mappings: vec![
                mapping(
                    "wall:W-001:global-placement",
                    "node:wall-001:placement",
                    MappingDisposition::Preserved,
                ),
                mapping(
                    "wall:W-001:fire-rating",
                    "node:wall-001:fire-rating",
                    MappingDisposition::Preserved,
                ),
            ],
            equivalence_oracle: Some(EquivalenceOracleEvidence {
                oracle_id: "independent-ifc-ieac-oracle".into(),
                oracle_build_sha256: digest('5'),
                corpus_id: "ifc-ieac-semantic-v0.1".into(),
                corpus_sha256: digest('6'),
                result_sha256: digest('7'),
                result: OracleResult::Pass,
                evaluated_at: "2026-11-04T12:00:00Z".into(),
            }),
            action_authority: "none".into(),
            limitations: vec![
                "Synthetic-only fixture; no real IFC export is evaluated.".into(),
                "This validator checks receipt consistency, not engineering equivalence.".into(),
                "The oracle result is referenced but not authenticated here.".into(),
            ],
        }
    }

    #[test]
    fn closed_mapping_inventory_is_consistent_but_never_self_authenticates_equivalence() {
        let result = validate_semantic_transformation_receipt(&receipt());
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::Indeterminate
        );
        assert_eq!(result.source_item_count, 2);
        assert_eq!(result.target_item_count, 2);
        assert_eq!(result.mapped_source_item_count, 2);
        assert_eq!(result.unmapped_source_item_count, 0);
        assert_eq!(result.oracle_evidence_authenticated, "not_performed");
        assert_eq!(result.semantic_equivalence, "not_evaluated_by_receipt_validator");
        assert_eq!(result.action_authority, "none");
        assert_eq!(result.claim_ceiling, "mapping_integrity_only");
        assert!(result.findings.iter().any(|finding| {
            finding.code == "ORACLE_RESULT_NOT_AUTHENTICATED_HERE"
        }));
    }

    #[test]
    fn missing_source_mapping_is_indeterminate_not_silent() {
        let mut value = receipt();
        value.mappings.pop();
        value.target_inventory.pop();
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::Indeterminate
        );
        assert_eq!(result.unmapped_source_item_count, 1);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "SOURCE_SEMANTIC_ITEM_UNMAPPED"
        }));
    }

    #[test]
    fn duplicate_mapping_for_one_source_item_is_invalid() {
        let mut value = receipt();
        value.mappings.push(value.mappings[0].clone());
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::InvalidReceipt
        );
        assert!(result.findings.iter().any(|finding| {
            finding.code == "DUPLICATE_SOURCE_MAPPING"
        }));
    }

    #[test]
    fn mapping_to_unknown_target_is_invalid() {
        let mut value = receipt();
        value.mappings[0].target_semantic_ids = vec!["not-in-target-inventory".into()];
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::InvalidReceipt
        );
        assert!(result.findings.iter().any(|finding| {
            finding.code == "MAPPING_TARGET_NOT_IN_INVENTORY"
        }));
    }

    #[test]
    fn required_semantic_drop_blocks_equivalence_claim() {
        let mut value = receipt();
        value.mappings[0] =
            mapping("wall:W-001:global-placement", "", MappingDisposition::Dropped);
        value.mappings[0].loss_reason = Some("target profile has no coordinate frame".into());
        value.target_inventory.remove(0);
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::NotEquivalent
        );
        assert_eq!(result.dropped_mapping_count, 1);
        assert_eq!(result.decision_required_loss_count, 1);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "EQUIVALENCE_CLAIM_CONTRADICTS_LOSS"
        }));
    }

    #[test]
    fn unsupported_semantic_must_be_explicit_and_has_no_target() {
        let mut value = receipt();
        value.mappings[0] =
            mapping("wall:W-001:global-placement", "", MappingDisposition::Unsupported);
        value.mappings[0].loss_reason = Some("unsupported target coordinate model".into());
        value.target_inventory.remove(0);
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::NotEquivalent
        );
        assert_eq!(result.unsupported_mapping_count, 1);
    }

    #[test]
    fn unit_conversion_requires_exact_rational_metadata_and_same_dimension() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Converted;
        value.mappings[0].metadata.conversion_kind = Some(ConversionKind::Unit);
        value.mappings[0].metadata.unit_conversion = Some(UnitConversion {
            source_unit: "millimetre".into(),
            target_unit: "metre".into(),
            source_dimension: "length".into(),
            target_dimension: "length".into(),
            scale: Rational {
                numerator: 1,
                denominator: 1000,
            },
            offset: Rational {
                numerator: 0,
                denominator: 1,
            },
        });
        let result = validate_semantic_transformation_receipt(&value);
        assert!(!result.findings.iter().any(|finding| {
            finding.code == "UNIT_CONVERSION_METADATA_MISSING"
                || finding.code == "UNIT_DIMENSION_CHANGED"
        }));
        assert_eq!(result.action_authority, "none");
    }

    #[test]
    fn dimensional_mismatch_is_not_equivalent() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Converted;
        value.mappings[0].metadata.conversion_kind = Some(ConversionKind::Unit);
        value.mappings[0].metadata.unit_conversion = Some(UnitConversion {
            source_unit: "millimetre".into(),
            target_unit: "kilogram".into(),
            source_dimension: "length".into(),
            target_dimension: "mass".into(),
            scale: Rational {
                numerator: 1,
                denominator: 1000,
            },
            offset: Rational {
                numerator: 0,
                denominator: 1,
            },
        });
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::NotEquivalent
        );
        assert!(result.findings.iter().any(|finding| {
            finding.code == "UNIT_DIMENSION_CHANGED"
        }));
    }

    #[test]
    fn non_positive_rational_denominator_is_invalid() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Converted;
        value.mappings[0].metadata.conversion_kind = Some(ConversionKind::Unit);
        value.mappings[0].metadata.unit_conversion = Some(UnitConversion {
            source_unit: "millimetre".into(),
            target_unit: "metre".into(),
            source_dimension: "length".into(),
            target_dimension: "length".into(),
            scale: Rational {
                numerator: 1,
                denominator: 0,
            },
            offset: Rational {
                numerator: 0,
                denominator: 1,
            },
        });
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::InvalidReceipt
        );
        assert!(result.findings.iter().any(|finding| {
            finding.code == "RATIONAL_DENOMINATOR_NOT_POSITIVE"
        }));
    }

    #[test]
    fn coordinate_conversion_requires_crs_frames_axes_datum_and_parameters() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Converted;
        value.mappings[0].metadata.conversion_kind = Some(ConversionKind::CoordinateFrame);
        value.mappings[0].metadata.coordinate_conversion = Some(CoordinateFrameConversion {
            source_crs_id: "EPSG:26915".into(),
            target_crs_id: "EPSG:4326".into(),
            source_frame_id: "survey-local-frame-1".into(),
            target_frame_id: "wgs84-global".into(),
            source_axis_order: vec!["x".into(), "y".into(), "z".into()],
            target_axis_order: vec!["x".into(), "y".into(), "z".into()],
            source_handedness: CoordinateHandedness::RightHanded,
            target_handedness: CoordinateHandedness::RightHanded,
            source_vertical_datum: "NAVD88".into(),
            target_vertical_datum: "ellipsoidal-height".into(),
            operation_id: "coordinate-operation-001".into(),
            operation_parameters_sha256: digest('8'),
        });
        let result = validate_semantic_transformation_receipt(&value);
        assert!(!result.findings.iter().any(|finding| {
            finding.code.starts_with("COORDINATE_CONVERSION_")
        }));
    }

    #[test]
    fn dropped_uncertainty_is_a_decision_required_loss() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Converted;
        value.mappings[0].metadata.conversion_kind = Some(ConversionKind::Uncertainty);
        value.mappings[0].metadata.uncertainty_conversion = Some(UncertaintyConversion {
            source_model: "expanded-uncertainty-k2".into(),
            target_model: "point-estimate-only".into(),
            behavior: UncertaintyBehavior::Reduced,
            propagation_method: None,
            assumptions_sha256: None,
            covariance_sha256: None,
            target_bound_sha256: None,
        });
        value.declared_claim = DeclaredTransformationClaim::EquivalentWithDeclaredLoss;
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::DeclaredSemanticLoss
        );
        assert_eq!(result.decision_required_loss_count, 1);
    }

    #[test]
    fn split_mapping_requires_multiple_targets_and_group_identity() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Split;
        value.mappings[0].target_semantic_ids = vec![
            "node:wall-001:placement-origin".into(),
            "node:wall-001:placement-orientation".into(),
        ];
        value.mappings[0].mapping_group_id = Some("split:wall-001-placement".into());
        value.target_inventory.push(TargetSemanticItem {
            semantic_id: "node:wall-001:placement-orientation".into(),
            semantic_kind: "coordinate_frame_orientation".into(),
            origin: TargetSemanticOrigin::SourceMapped,
            source_semantic_ids: vec!["wall:W-001:global-placement".into()],
            target_value_sha256: digest('8'),
        });
        let result = validate_semantic_transformation_receipt(&value);
        assert!(!result.findings.iter().any(|finding| {
            finding.code == "SPLIT_MAPPING_GROUP_INVALID"
                || finding.code == "MAPPING_TARGET_NOT_IN_INVENTORY"
        }));
    }

    #[test]
    fn split_mapping_without_group_id_is_invalid() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Split;
        value.mappings[0].target_semantic_ids = vec![
            "node:wall-001:placement".into(),
            "node:wall-001:placement-extra".into(),
        ];
        value.target_inventory.push(TargetSemanticItem {
            semantic_id: "node:wall-001:placement-extra".into(),
            semantic_kind: "coordinate_frame_component".into(),
            origin: TargetSemanticOrigin::SourceMapped,
            source_semantic_ids: vec!["wall:W-001:global-placement".into()],
            target_value_sha256: digest('8'),
        });
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(result.disposition, TransformationReceiptDisposition::InvalidReceipt);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "SPLIT_MAPPING_GROUP_INVALID"
        }));
    }

    #[test]
    fn merged_mapping_requires_group_identity_and_reciprocal_source_links() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Merged;
        value.mappings[0].mapping_group_id = Some("merge:wall-material-properties".into());
        value.mappings[1].disposition = MappingDisposition::Merged;
        value.mappings[1].target_semantic_ids = vec!["node:wall-001:combined-properties".into()];
        value.mappings[1].mapping_group_id = Some("merge:wall-material-properties".into());
        value.target_inventory[1].semantic_id = "node:wall-001:combined-properties".into();
        value.target_inventory[1].source_semantic_ids = vec![
            "wall:W-001:global-placement".into(),
            "wall:W-001:fire-rating".into(),
        ];
        let result = validate_semantic_transformation_receipt(&value);
        assert!(!result.findings.iter().any(|finding| {
            finding.code == "MERGED_MAPPING_GROUP_INVALID"
                || finding.code == "TARGET_SOURCE_MAPPING_NOT_RECIPROCAL"
        }));
    }

    #[test]
    fn converted_mapping_without_a_typed_conversion_kind_is_indeterminate() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Converted;
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(result.disposition, TransformationReceiptDisposition::Indeterminate);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "CONVERSION_KIND_UNSPECIFIED"
        }));
    }

    #[test]
    fn uncertainty_propagation_requires_method_and_assumption_digest() {
        let mut value = receipt();
        value.mappings[0].disposition = MappingDisposition::Converted;
        value.mappings[0].metadata.conversion_kind = Some(ConversionKind::Uncertainty);
        value.mappings[0].metadata.uncertainty_conversion = Some(UncertaintyConversion {
            source_model: "expanded-uncertainty-k2".into(),
            target_model: "covariance-model".into(),
            behavior: UncertaintyBehavior::Propagated,
            propagation_method: Some("first-order propagation".into()),
            assumptions_sha256: None,
            covariance_sha256: Some(digest('9')),
            target_bound_sha256: None,
        });
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(result.disposition, TransformationReceiptDisposition::Indeterminate);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "UNCERTAINTY_PROPAGATION_EVIDENCE_INCOMPLETE"
        }));
    }

    #[test]
    fn oracle_pass_remains_unverified_and_cannot_upgrade_claim_ceiling() {
        let result = validate_semantic_transformation_receipt(&receipt());
        assert!(result.oracle_evidence_present);
        assert_eq!(result.oracle_evidence_authenticated, "not_performed");
        assert_eq!(result.claim_ceiling, "mapping_integrity_only");
    }

    #[test]
    fn full_equivalence_claim_without_oracle_reference_is_invalid() {
        let mut value = receipt();
        value.equivalence_oracle = None;
        let result = validate_semantic_transformation_receipt(&value);
        assert_eq!(
            result.disposition,
            TransformationReceiptDisposition::InvalidReceipt
        );
        assert!(result.findings.iter().any(|finding| {
            finding.code == "EQUIVALENCE_CLAIM_WITHOUT_ORACLE_EVIDENCE"
        }));
    }
}
