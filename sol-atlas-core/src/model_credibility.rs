// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Conservative handoff from a model-credibility monitor to Sol Atlas service
//! assessments. A byte digest is provenance metadata, not a signature or proof
//! that a source observation is true. Abstentions are indeterminate and never
//! exported as usable capacity.

use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MODEL_CREDIBILITY_HANDOFF_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelDisposition {
    UseModel,
    RejectUnsafeInterval,
    AbstainMissingObservation,
    AbstainStaleTelemetry,
    AbstainResidualOutOfEnvelope,
    AbstainRecoveryHoldoff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapacityDisposition {
    EligibleForAssessment,
    Rejected,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundedCapacityInterval {
    pub lower: f64,
    pub upper: f64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCredibilityHandoff {
    pub schema_version: String,
    pub decision_id: String,
    pub model_id: String,
    pub asset_id: String,
    /// Canonical UTC timestamp of the model decision, not necessarily the measurement.
    pub as_of_utc: String,
    /// Both bounds use this exact unit. No implicit conversion is performed.
    pub capacity_unit: String,
    pub capacity_lower: f64,
    pub capacity_upper: f64,
    pub disposition: ModelDisposition,
    pub source_commit_sha: String,
    pub calibration_set_sha256: String,
    pub generated_case_set_sha256: String,
    /// Evidence IDs for the model decision and observation/fallback state.
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelCredibilityHandoffError {
    UnsupportedSchemaVersion,
    BlankIdentity,
    InvalidTimestamp,
    InvalidCapacityUnit,
    InvalidCapacityInterval,
    InvalidSourceCommit,
    InvalidCalibrationDigest,
    InvalidCaseSetDigest,
    MissingEvidence,
    DuplicateEvidenceId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCredibilityHandoffAssessment {
    pub schema_version: String,
    pub decision_id: String,
    pub model_id: String,
    pub asset_id: String,
    pub as_of_utc: String,
    pub disposition: ModelDisposition,
    pub status: ModelCapacityDisposition,
    pub reason_code: String,
    pub capacity_unit: String,
    /// Present only when the upstream disposition is UseModel.
    /// None means indeterminate/rejected, never zero capacity.
    pub eligible_capacity: Option<BoundedCapacityInterval>,
    pub source_commit_sha: String,
    pub calibration_set_sha256: String,
    pub generated_case_set_sha256: String,
    pub supporting_evidence_ids: Vec<String>,
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value == value.to_ascii_lowercase()
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn nonblank(value: &str) -> bool {
    !value.trim().is_empty() && value == value.trim()
}

fn validate_handoff(
    input: &ModelCredibilityHandoff,
) -> Result<(), ModelCredibilityHandoffError> {
    if input.schema_version != MODEL_CREDIBILITY_HANDOFF_SCHEMA_VERSION {
        return Err(ModelCredibilityHandoffError::UnsupportedSchemaVersion);
    }
    if !nonblank(&input.decision_id)
        || !nonblank(&input.model_id)
        || !nonblank(&input.asset_id)
    {
        return Err(ModelCredibilityHandoffError::BlankIdentity);
    }
    if !is_canonical_utc_timestamp(&input.as_of_utc) {
        return Err(ModelCredibilityHandoffError::InvalidTimestamp);
    }
    if !nonblank(&input.capacity_unit) {
        return Err(ModelCredibilityHandoffError::InvalidCapacityUnit);
    }
    if !input.capacity_lower.is_finite()
        || !input.capacity_upper.is_finite()
        || input.capacity_lower < 0.0
        || input.capacity_upper < input.capacity_lower
    {
        return Err(ModelCredibilityHandoffError::InvalidCapacityInterval);
    }
    if !is_lower_hex(&input.source_commit_sha, 40) {
        return Err(ModelCredibilityHandoffError::InvalidSourceCommit);
    }
    if !is_lower_hex(&input.calibration_set_sha256, 64) {
        return Err(ModelCredibilityHandoffError::InvalidCalibrationDigest);
    }
    if !is_lower_hex(&input.generated_case_set_sha256, 64) {
        return Err(ModelCredibilityHandoffError::InvalidCaseSetDigest);
    }
    if input.supporting_evidence_ids.is_empty()
        || input.supporting_evidence_ids.iter().any(|id| !nonblank(id))
    {
        return Err(ModelCredibilityHandoffError::MissingEvidence);
    }
    let evidence_set = input.supporting_evidence_ids.iter().collect::<BTreeSet<_>>();
    if evidence_set.len() != input.supporting_evidence_ids.len() {
        return Err(ModelCredibilityHandoffError::DuplicateEvidenceId);
    }
    Ok(())
}

/// Maps the upstream disposition without treating abstention as zero capacity.
/// This validates payload shape and digest syntax; it does not authenticate the
/// producer, verify that a referenced commit exists, or prove evidence truth.
pub fn assess_model_credibility_handoff(
    input: &ModelCredibilityHandoff,
) -> Result<ModelCredibilityHandoffAssessment, ModelCredibilityHandoffError> {
    validate_handoff(input)?;

    let (status, reason_code, eligible_capacity) = match input.disposition {
        ModelDisposition::UseModel => (
            ModelCapacityDisposition::EligibleForAssessment,
            "MODEL_DISPOSITION_ADMITTED",
            Some(BoundedCapacityInterval {
                lower: input.capacity_lower,
                upper: input.capacity_upper,
                unit: input.capacity_unit.clone(),
            }),
        ),
        ModelDisposition::RejectUnsafeInterval => (
            ModelCapacityDisposition::Rejected,
            "MODEL_INTERVAL_REJECTED",
            None,
        ),
        ModelDisposition::AbstainMissingObservation => (
            ModelCapacityDisposition::Indeterminate,
            "MODEL_ABSTAINED_MISSING_OBSERVATION",
            None,
        ),
        ModelDisposition::AbstainStaleTelemetry => (
            ModelCapacityDisposition::Indeterminate,
            "MODEL_ABSTAINED_STALE_TELEMETRY",
            None,
        ),
        ModelDisposition::AbstainResidualOutOfEnvelope => (
            ModelCapacityDisposition::Indeterminate,
            "MODEL_ABSTAINED_RESIDUAL_OUT_OF_ENVELOPE",
            None,
        ),
        ModelDisposition::AbstainRecoveryHoldoff => (
            ModelCapacityDisposition::Indeterminate,
            "MODEL_ABSTAINED_RECOVERY_HOLDOFF",
            None,
        ),
    };

    Ok(ModelCredibilityHandoffAssessment {
        schema_version: MODEL_CREDIBILITY_HANDOFF_SCHEMA_VERSION.to_owned(),
        decision_id: input.decision_id.clone(),
        model_id: input.model_id.clone(),
        asset_id: input.asset_id.clone(),
        as_of_utc: input.as_of_utc.clone(),
        disposition: input.disposition,
        status,
        reason_code: reason_code.to_owned(),
        capacity_unit: input.capacity_unit.clone(),
        eligible_capacity,
        source_commit_sha: input.source_commit_sha.clone(),
        calibration_set_sha256: input.calibration_set_sha256.clone(),
        generated_case_set_sha256: input.generated_case_set_sha256.clone(),
        supporting_evidence_ids: input.supporting_evidence_ids.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(disposition: ModelDisposition) -> ModelCredibilityHandoff {
        ModelCredibilityHandoff {
            schema_version: MODEL_CREDIBILITY_HANDOFF_SCHEMA_VERSION.to_owned(),
            decision_id: "decision-001".to_owned(),
            model_id: "pump-state-estimator-v1".to_owned(),
            asset_id: "pump-17".to_owned(),
            as_of_utc: "2026-10-19T20:00:00Z".to_owned(),
            capacity_unit: "m3/h".to_owned(),
            capacity_lower: 8.0,
            capacity_upper: 10.0,
            disposition,
            source_commit_sha: "a".repeat(40),
            calibration_set_sha256: "b".repeat(64),
            generated_case_set_sha256: "c".repeat(64),
            supporting_evidence_ids: vec!["measurement-window-17".to_owned()],
        }
    }

    #[test]
    fn admitted_model_exports_only_its_explicit_bounded_capacity() {
        let input = fixture(ModelDisposition::UseModel);
        let result = assess_model_credibility_handoff(&input).expect("valid handoff");
        assert_eq!(ModelCapacityDisposition::EligibleForAssessment, result.status);
        assert_eq!("MODEL_DISPOSITION_ADMITTED", result.reason_code);
        assert_eq!(
            Some(BoundedCapacityInterval {
                lower: 8.0,
                upper: 10.0,
                unit: "m3/h".to_owned(),
            }),
            result.eligible_capacity
        );
    }

    #[test]
    fn every_abstention_is_indeterminate_and_never_becomes_zero_capacity() {
        for disposition in [
            ModelDisposition::AbstainMissingObservation,
            ModelDisposition::AbstainStaleTelemetry,
            ModelDisposition::AbstainResidualOutOfEnvelope,
            ModelDisposition::AbstainRecoveryHoldoff,
        ] {
            let result = assess_model_credibility_handoff(&fixture(disposition))
                .expect("valid abstention envelope");
            assert_eq!(ModelCapacityDisposition::Indeterminate, result.status);
            assert!(result.eligible_capacity.is_none());
            assert!(!result.reason_code.is_empty());
        }
    }

    #[test]
    fn explicit_unsafe_rejection_is_not_confused_with_unknown_evidence() {
        let result = assess_model_credibility_handoff(&fixture(
            ModelDisposition::RejectUnsafeInterval,
        ))
        .expect("valid rejection envelope");
        assert_eq!(ModelCapacityDisposition::Rejected, result.status);
        assert!(result.eligible_capacity.is_none());
    }

    #[test]
    fn malformed_identity_time_units_bounds_and_provenance_fail_closed() {
        let mut input = fixture(ModelDisposition::UseModel);
        input.schema_version = "9.0.0".to_owned();
        assert_eq!(
            Err(ModelCredibilityHandoffError::UnsupportedSchemaVersion),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.decision_id.clear();
        assert_eq!(
            Err(ModelCredibilityHandoffError::BlankIdentity),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.capacity_unit = "   ".to_owned();
        assert_eq!(
            Err(ModelCredibilityHandoffError::InvalidCapacityUnit),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.as_of_utc = "yesterday".to_owned();
        assert_eq!(
            Err(ModelCredibilityHandoffError::InvalidTimestamp),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.capacity_lower = f64::NAN;
        assert_eq!(
            Err(ModelCredibilityHandoffError::InvalidCapacityInterval),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.capacity_lower = 11.0;
        assert_eq!(
            Err(ModelCredibilityHandoffError::InvalidCapacityInterval),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.source_commit_sha = "not-a-commit".to_owned();
        assert_eq!(
            Err(ModelCredibilityHandoffError::InvalidSourceCommit),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.calibration_set_sha256 = "not-a-digest".to_owned();
        assert_eq!(
            Err(ModelCredibilityHandoffError::InvalidCalibrationDigest),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.generated_case_set_sha256 = "not-a-digest".to_owned();
        assert_eq!(
            Err(ModelCredibilityHandoffError::InvalidCaseSetDigest),
            assess_model_credibility_handoff(&input)
        );
    }

    #[test]
    fn missing_or_duplicate_evidence_references_fail_closed() {
        let mut input = fixture(ModelDisposition::AbstainMissingObservation);
        input.supporting_evidence_ids.clear();
        assert_eq!(
            Err(ModelCredibilityHandoffError::MissingEvidence),
            assess_model_credibility_handoff(&input)
        );

        let mut input = fixture(ModelDisposition::UseModel);
        input.supporting_evidence_ids = vec!["same".to_owned(), "same".to_owned()];
        assert_eq!(
            Err(ModelCredibilityHandoffError::DuplicateEvidenceId),
            assess_model_credibility_handoff(&input)
        );
    }

    #[test]
    fn serialized_handoff_and_assessment_are_deterministic() {
        let input = fixture(ModelDisposition::AbstainStaleTelemetry);
        let first = assess_model_credibility_handoff(&input).expect("first assessment");
        let second = assess_model_credibility_handoff(&input).expect("second assessment");
        assert_eq!(
            serde_json::to_vec(&first).expect("serialize first"),
            serde_json::to_vec(&second).expect("serialize second")
        );
        let bytes = serde_json::to_vec(&first).expect("serialize assessment");
        let decoded: ModelCredibilityHandoffAssessment =
            serde_json::from_slice(&bytes).expect("deserialize assessment");
        assert_eq!(first, decoded);
    }
}
