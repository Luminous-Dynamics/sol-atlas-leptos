// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Versioned, framework-neutral field-work projections and viewport intents.
//!
//! These types are presentation contracts, not canonical work-order records or
//! authorization decisions. Leptos and Bevy may consume the same projection;
//! authoritative services must independently validate every command.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Version of the serialized projection envelope, independent of app version.
pub const FIELD_JOB_PROJECTION_SCHEMA_VERSION: u16 = 1;

/// Provenance pointer for the authoritative source used to build this view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectionSourceV1 {
    pub authority_id: String,
    pub source_revision: u64,
    /// SHA-256 of the canonical source snapshot, encoded as 64 lowercase hex chars.
    pub source_digest_sha256: String,
}

/// A field-job view shared by textual and spatial UI surfaces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FieldJobProjectionV1 {
    pub schema_version: u16,
    pub job_id: String,
    pub job_revision: u64,
    pub trade: TradeKind,
    pub title: String,
    pub phase: FieldJobPhase,
    /// Transport is deliberately distinct from the physical work lifecycle.
    pub transport: TransportState,
    pub freshness: ProjectionFreshness,
    pub source: ProjectionSourceV1,
    pub assets: Vec<AssetRefV1>,
    pub evidence: Vec<EvidenceRefV1>,
    pub safety_gate: SafetyGateSummaryV1,
    /// Affordances for a viewport only; these do not confer command authority.
    pub offered_viewport_intents: Vec<ViewportIntentKind>,
}

impl FieldJobProjectionV1 {
    /// Validate structural consistency before a UI or renderer consumes a view.
    ///
    /// This is not a substitute for validating authoritative commands or source
    /// records. It rejects malformed/stale-shaped UI projections fail-closed.
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if self.schema_version != FIELD_JOB_PROJECTION_SCHEMA_VERSION {
            return Err(ProjectionError::UnsupportedSchemaVersion);
        }
        require_non_empty(&self.job_id, ProjectionError::MissingJobId)?;
        require_non_empty(&self.title, ProjectionError::MissingTitle)?;
        if self.job_revision == 0 {
            return Err(ProjectionError::InvalidJobRevision);
        }

        require_non_empty(
            &self.source.authority_id,
            ProjectionError::MissingAuthorityId,
        )?;
        if self.source.source_revision == 0 {
            return Err(ProjectionError::InvalidSourceRevision);
        }
        validate_sha256(
            &self.source.source_digest_sha256,
            ProjectionError::InvalidSourceDigest,
        )?;

        self.transport.validate()?;

        let mut asset_ids = BTreeSet::new();
        for asset in &self.assets {
            asset.validate()?;
            if !asset_ids.insert(asset.asset_id.as_str()) {
                return Err(ProjectionError::DuplicateAssetId);
            }
        }

        let mut evidence_ids = BTreeSet::new();
        for evidence in &self.evidence {
            evidence.validate()?;
            if !evidence_ids.insert(evidence.evidence_id.as_str()) {
                return Err(ProjectionError::DuplicateEvidenceId);
            }
        }

        self.safety_gate.validate()?;

        let mut intents = BTreeSet::new();
        for intent in &self.offered_viewport_intents {
            if !intents.insert(*intent) {
                return Err(ProjectionError::DuplicateViewportIntent);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TradeKind {
    Electrical,
    Plumbing,
    MechanicalRepair,
    HvacRefrigeration,
    Construction,
    GeneralMaintenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FieldJobPhase {
    Assigned,
    Assessing,
    FindingRecorded,
    PlanApproved,
    WorkReportedInProgress,
    CompletionReported,
    VerificationRecorded,
    HandoverRecorded,
    Blocked,
    NeedsInformation,
    ReworkRequired,
    Disputed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase", deny_unknown_fields)]
pub enum TransportState {
    LocalOnly,
    ReadyToSubmit,
    OutcomeUnknown { operation_id: String },
    Received { receipt_ref: String },
    Rejected { reason_code: String },
    ConflictNeedsReview { conflict_ref: String },
}

impl TransportState {
    fn validate(&self) -> Result<(), ProjectionError> {
        match self {
            Self::LocalOnly | Self::ReadyToSubmit => Ok(()),
            Self::OutcomeUnknown { operation_id } => require_non_empty(
                operation_id,
                ProjectionError::UnknownOutcomeMissingOperationId,
            ),
            Self::Received { receipt_ref } => {
                require_non_empty(receipt_ref, ProjectionError::ReceivedMissingReceipt)
            }
            Self::Rejected { reason_code } => {
                require_non_empty(reason_code, ProjectionError::RejectedMissingReasonCode)
            }
            Self::ConflictNeedsReview { conflict_ref } => {
                require_non_empty(conflict_ref, ProjectionError::ConflictMissingReference)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectionFreshness {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetRefV1 {
    pub asset_id: String,
    pub asset_revision: u64,
    pub label: String,
    pub relationship: AssetRelationship,
}

impl AssetRefV1 {
    fn validate(&self) -> Result<(), ProjectionError> {
        require_non_empty(&self.asset_id, ProjectionError::MissingAssetId)?;
        require_non_empty(&self.label, ProjectionError::MissingAssetLabel)?;
        if self.asset_revision == 0 {
            return Err(ProjectionError::InvalidAssetRevision);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetRelationship {
    Primary,
    Component,
    Related,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceRefV1 {
    pub evidence_id: String,
    pub kind: EvidenceKind,
    /// URI/opaque stable pointer to the source record, not a display label.
    pub source_ref: String,
    pub digest_sha256: String,
    pub captured_at_unix_ms: Option<u64>,
    pub unit: Option<String>,
    pub instrument_ref: Option<String>,
    pub procedure_revision: Option<u64>,
    pub model_ref: Option<String>,
}

impl EvidenceRefV1 {
    fn validate(&self) -> Result<(), ProjectionError> {
        require_non_empty(&self.evidence_id, ProjectionError::MissingEvidenceId)?;
        require_non_empty(&self.source_ref, ProjectionError::MissingEvidenceSource)?;
        validate_sha256(&self.digest_sha256, ProjectionError::InvalidEvidenceDigest)?;

        if self.kind == EvidenceKind::Measured && self.unit.as_deref().is_none_or(str::is_empty) {
            return Err(ProjectionError::MeasuredEvidenceMissingUnit);
        }
        if self.kind == EvidenceKind::Modelled
            && self.model_ref.as_deref().is_none_or(str::is_empty)
        {
            return Err(ProjectionError::ModelledEvidenceMissingModelRef);
        }
        if self.procedure_revision == Some(0) {
            return Err(ProjectionError::InvalidProcedureRevision);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceKind {
    Reported,
    Measured,
    Imported,
    Modelled,
    Verified,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SafetyGateSummaryV1 {
    pub status: SafetyGateStatus,
    pub policy_ref: Option<String>,
    pub policy_revision: Option<u64>,
    pub missing_requirements: Vec<String>,
}

impl SafetyGateSummaryV1 {
    fn validate(&self) -> Result<(), ProjectionError> {
        if self.policy_revision == Some(0) {
            return Err(ProjectionError::InvalidPolicyRevision);
        }
        match self.status {
            SafetyGateStatus::Blocked => {
                if self.policy_ref.as_deref().is_none_or(str::is_empty)
                    || self.policy_revision.is_none()
                    || self.missing_requirements.is_empty()
                    || self
                        .missing_requirements
                        .iter()
                        .any(|reason| reason.trim().is_empty())
                {
                    return Err(ProjectionError::BlockedGateMissingReason);
                }
            }
            SafetyGateStatus::AwaitingQualifiedReview => {
                if self.policy_ref.as_deref().is_none_or(str::is_empty)
                    || self.policy_revision.is_none()
                    || !self.missing_requirements.is_empty()
                {
                    return Err(ProjectionError::InvalidReviewReadyGate);
                }
            }
            SafetyGateStatus::Unknown => {}
        }
        Ok(())
    }
}

/// None of these values means "physically safe" or authorizes work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SafetyGateStatus {
    Unknown,
    Blocked,
    AwaitingQualifiedReview,
}

/// Viewport affordance kinds are hints to render controls, never authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewportIntentKind {
    SelectAsset,
    FocusAsset,
    SelectComponent,
    SelectFlow,
    SetTimelineCursor,
    RequestJobComparison,
}

/// Typed user intent emitted by a Bevy viewport or another spatial surface.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "intent",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ViewportIntentV1 {
    SelectAsset {
        asset_id: String,
    },
    FocusAsset {
        asset_id: String,
    },
    SelectComponent {
        component_id: String,
    },
    SelectFlow {
        flow_id: String,
    },
    SetTimelineCursor {
        at_unix_ms: u64,
    },
    RequestJobComparison {
        job_id: String,
        baseline_revision: u64,
        candidate_revision: u64,
    },
}

impl ViewportIntentV1 {
    pub fn kind(&self) -> ViewportIntentKind {
        match self {
            Self::SelectAsset { .. } => ViewportIntentKind::SelectAsset,
            Self::FocusAsset { .. } => ViewportIntentKind::FocusAsset,
            Self::SelectComponent { .. } => ViewportIntentKind::SelectComponent,
            Self::SelectFlow { .. } => ViewportIntentKind::SelectFlow,
            Self::SetTimelineCursor { .. } => ViewportIntentKind::SetTimelineCursor,
            Self::RequestJobComparison { .. } => ViewportIntentKind::RequestJobComparison,
        }
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        match self {
            Self::SelectAsset { asset_id } | Self::FocusAsset { asset_id } => {
                require_non_empty(asset_id, ProjectionError::MissingIntentTarget)
            }
            Self::SelectComponent { component_id } => {
                require_non_empty(component_id, ProjectionError::MissingIntentTarget)
            }
            Self::SelectFlow { flow_id } => {
                require_non_empty(flow_id, ProjectionError::MissingIntentTarget)
            }
            Self::SetTimelineCursor { .. } => Ok(()),
            Self::RequestJobComparison {
                job_id,
                baseline_revision,
                candidate_revision,
            } => {
                require_non_empty(job_id, ProjectionError::MissingIntentTarget)?;
                if *baseline_revision == 0
                    || *candidate_revision == 0
                    || baseline_revision == candidate_revision
                {
                    return Err(ProjectionError::InvalidComparisonRevisions);
                }
                Ok(())
            }
        }
    }

    /// Check that the intent is represented by this projection's affordances.
    ///
    /// This is a client-side consistency guard only. The authoritative command
    /// handler must still re-check actor rights, revisions and domain rules.
    pub fn validate_for_projection(
        &self,
        projection: &FieldJobProjectionV1,
    ) -> Result<(), ProjectionError> {
        projection.validate()?;
        self.validate()?;
        if !projection.offered_viewport_intents.contains(&self.kind()) {
            return Err(ProjectionError::IntentNotOffered);
        }
        match self {
            Self::SelectAsset { asset_id } | Self::FocusAsset { asset_id } => {
                if !projection
                    .assets
                    .iter()
                    .any(|asset| asset.asset_id == *asset_id)
                {
                    return Err(ProjectionError::IntentTargetMissingFromProjection);
                }
            }
            Self::SelectComponent { component_id } => {
                if !projection.assets.iter().any(|asset| {
                    asset.asset_id == *component_id
                        && asset.relationship == AssetRelationship::Component
                }) {
                    return Err(ProjectionError::IntentTargetMissingFromProjection);
                }
            }
            Self::RequestJobComparison { job_id, .. } if job_id != &projection.job_id => {
                return Err(ProjectionError::ComparisonJobMismatch);
            }
            Self::SelectFlow { .. }
            | Self::SetTimelineCursor { .. }
            | Self::RequestJobComparison { .. } => {}
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    UnsupportedSchemaVersion,
    MissingJobId,
    MissingTitle,
    InvalidJobRevision,
    MissingAuthorityId,
    InvalidSourceRevision,
    InvalidSourceDigest,
    UnknownOutcomeMissingOperationId,
    ReceivedMissingReceipt,
    RejectedMissingReasonCode,
    ConflictMissingReference,
    MissingAssetId,
    MissingAssetLabel,
    InvalidAssetRevision,
    DuplicateAssetId,
    MissingEvidenceId,
    MissingEvidenceSource,
    InvalidEvidenceDigest,
    MeasuredEvidenceMissingUnit,
    ModelledEvidenceMissingModelRef,
    InvalidProcedureRevision,
    InvalidPolicyRevision,
    BlockedGateMissingReason,
    InvalidReviewReadyGate,
    DuplicateEvidenceId,
    DuplicateViewportIntent,
    MissingIntentTarget,
    IntentNotOffered,
    IntentTargetMissingFromProjection,
    ComparisonJobMismatch,
    InvalidComparisonRevisions,
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ProjectionError {}

fn require_non_empty(value: &str, error: ProjectionError) -> Result<(), ProjectionError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str, error: ProjectionError) -> Result<(), ProjectionError> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid { Ok(()) } else { Err(error) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_projection() -> FieldJobProjectionV1 {
        FieldJobProjectionV1 {
            schema_version: FIELD_JOB_PROJECTION_SCHEMA_VERSION,
            job_id: "job-demo-01".into(),
            job_revision: 3,
            trade: TradeKind::Electrical,
            title: "Synthetic electrical inspection".into(),
            phase: FieldJobPhase::Assessing,
            transport: TransportState::LocalOnly,
            freshness: ProjectionFreshness::Fresh,
            source: ProjectionSourceV1 {
                authority_id: "demo-authority".into(),
                source_revision: 8,
                source_digest_sha256: "a".repeat(64),
            },
            assets: vec![AssetRefV1 {
                asset_id: "asset-demo-01".into(),
                asset_revision: 2,
                label: "Demo distribution board".into(),
                relationship: AssetRelationship::Primary,
            }],
            evidence: vec![EvidenceRefV1 {
                evidence_id: "evidence-demo-01".into(),
                kind: EvidenceKind::Measured,
                source_ref: "urn:demo:evidence:01".into(),
                digest_sha256: "b".repeat(64),
                captured_at_unix_ms: Some(1_800_000_000_000),
                unit: Some("V".into()),
                instrument_ref: Some("instrument-demo".into()),
                procedure_revision: Some(4),
                model_ref: None,
            }],
            safety_gate: SafetyGateSummaryV1 {
                status: SafetyGateStatus::Blocked,
                policy_ref: Some("demo-policy".into()),
                policy_revision: Some(4),
                missing_requirements: vec!["qualified review has not occurred".into()],
            },
            offered_viewport_intents: vec![
                ViewportIntentKind::SelectAsset,
                ViewportIntentKind::FocusAsset,
                ViewportIntentKind::SetTimelineCursor,
            ],
        }
    }

    #[test]
    fn accepts_valid_versioned_projection() {
        assert_eq!(valid_projection().validate(), Ok(()));
    }

    #[test]
    fn rejects_unknown_schema_and_zero_revision() {
        let mut projection = valid_projection();
        projection.schema_version = 2;
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::UnsupportedSchemaVersion)
        );

        let mut projection = valid_projection();
        projection.job_revision = 0;
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::InvalidJobRevision)
        );
    }

    #[test]
    fn unknown_transport_outcome_requires_stable_operation_id() {
        let mut projection = valid_projection();
        projection.transport = TransportState::OutcomeUnknown {
            operation_id: " ".into(),
        };
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::UnknownOutcomeMissingOperationId)
        );
    }

    #[test]
    fn received_transport_state_requires_authoritative_receipt_reference() {
        let mut projection = valid_projection();
        projection.transport = TransportState::Received {
            receipt_ref: " ".into(),
        };
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::ReceivedMissingReceipt)
        );

        let mut projection = valid_projection();
        projection.transport = TransportState::Rejected {
            reason_code: String::new(),
        };
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::RejectedMissingReasonCode)
        );

        let mut projection = valid_projection();
        projection.transport = TransportState::ConflictNeedsReview {
            conflict_ref: " ".into(),
        };
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::ConflictMissingReference)
        );
    }

    #[test]
    fn measured_and_modelled_evidence_need_type_specific_provenance() {
        let mut projection = valid_projection();
        projection.evidence[0].unit = None;
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::MeasuredEvidenceMissingUnit)
        );

        let mut projection = valid_projection();
        projection.evidence[0].kind = EvidenceKind::Modelled;
        projection.evidence[0].model_ref = None;
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::ModelledEvidenceMissingModelRef)
        );
    }

    #[test]
    fn safety_summary_never_promotes_missing_requirements_to_review_ready() {
        let mut projection = valid_projection();
        projection.safety_gate.status = SafetyGateStatus::AwaitingQualifiedReview;
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::InvalidReviewReadyGate)
        );
    }

    #[test]
    fn projection_rejects_duplicate_asset_and_evidence_ids() {
        let mut projection = valid_projection();
        projection.assets.push(projection.assets[0].clone());
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::DuplicateAssetId)
        );

        let mut projection = valid_projection();
        projection.evidence.push(projection.evidence[0].clone());
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::DuplicateEvidenceId)
        );
    }

    #[test]
    fn viewport_intents_are_typed_and_reject_invalid_comparison() {
        let selection = ViewportIntentV1::SelectAsset {
            asset_id: "asset-demo-01".into(),
        };
        assert_eq!(selection.kind(), ViewportIntentKind::SelectAsset);
        assert_eq!(selection.validate(), Ok(()));

        let invalid = ViewportIntentV1::RequestJobComparison {
            job_id: "job-demo-01".into(),
            baseline_revision: 2,
            candidate_revision: 2,
        };
        assert_eq!(
            invalid.validate(),
            Err(ProjectionError::InvalidComparisonRevisions)
        );
    }

    #[test]
    fn transport_and_work_phase_serialize_as_separate_dimensions() {
        let projection = valid_projection();
        let value = serde_json::to_value(&projection).expect("projection serializes");
        assert_eq!(value["phase"], "assessing");
        assert_eq!(value["transport"]["state"], "localOnly");
        let decoded: FieldJobProjectionV1 =
            serde_json::from_value(value).expect("projection deserializes");
        assert_eq!(decoded, projection);
    }

    #[test]
    fn sha256_provenance_requires_canonical_lowercase_hex() {
        let mut projection = valid_projection();
        projection.source.source_digest_sha256 = "A".repeat(64);
        assert_eq!(
            projection.validate(),
            Err(ProjectionError::InvalidSourceDigest)
        );
    }

    #[test]
    fn frozen_projection_fixture_deserializes_and_validates() {
        let projection: FieldJobProjectionV1 =
            serde_json::from_str(include_str!("../fixtures/field-job-projection-v1.json"))
                .expect("fixture matches the versioned projection wire contract");
        assert_eq!(projection.validate(), Ok(()));
        assert_eq!(projection.job_id, "job-demo-01");
    }

    #[test]
    fn frozen_viewport_intent_fixture_is_typed_and_valid() {
        let intents: Vec<ViewportIntentV1> =
            serde_json::from_str(include_str!("../fixtures/viewport-intents-v1.json"))
                .expect("fixture matches the tagged viewport-intent contract");
        assert_eq!(intents.len(), 6);
        for intent in &intents {
            assert_eq!(intent.validate(), Ok(()));
        }
        assert_eq!(
            intents
                .iter()
                .map(ViewportIntentV1::kind)
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
    }

    #[test]
    fn viewport_intents_must_be_offered_and_target_the_current_projection() {
        let projection: FieldJobProjectionV1 =
            serde_json::from_str(include_str!("../fixtures/field-job-projection-v1.json"))
                .expect("valid fixture");
        let selected = ViewportIntentV1::SelectAsset {
            asset_id: "asset-demo-01".into(),
        };
        assert_eq!(selected.validate_for_projection(&projection), Ok(()));

        let missing_asset = ViewportIntentV1::FocusAsset {
            asset_id: "asset-not-in-projection".into(),
        };
        assert_eq!(
            missing_asset.validate_for_projection(&projection),
            Err(ProjectionError::IntentTargetMissingFromProjection)
        );

        let unavailable = ViewportIntentV1::SelectFlow {
            flow_id: "flow-demo-01".into(),
        };
        assert_eq!(
            unavailable.validate_for_projection(&projection),
            Err(ProjectionError::IntentNotOffered)
        );

        let mut comparison_projection = projection.clone();
        comparison_projection
            .offered_viewport_intents
            .push(ViewportIntentKind::RequestJobComparison);
        let wrong_job = ViewportIntentV1::RequestJobComparison {
            job_id: "different-job".into(),
            baseline_revision: 2,
            candidate_revision: 3,
        };
        assert_eq!(
            wrong_job.validate_for_projection(&comparison_projection),
            Err(ProjectionError::ComparisonJobMismatch)
        );
    }
}
