// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Study-scoped maintenance availability projection.
//!
//! This module consumes already-admitted source references and derives a
//! deterministic interval trace for one asset and one work order. It does not
//! implement an asset registry, lifecycle ledger, cryptographic verifier,
//! operating procedure, or physical equipment model.
//!
//! Time is represented as Unix seconds and intervals are half-open:
//! start is included and end is excluded. Evidence digests are opaque exact
//! source identifiers; this module does not verify their construction/signers.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

pub const MAINTENANCE_AVAILABILITY_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudyWindow {
    pub start_unix_seconds: i64,
    pub end_unix_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetIdentity {
    pub asset_id: String,
    pub configuration_digest: String,
}

/// A reference to an independently stored source event or acceptance record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    pub reference: String,
    pub digest: String,
    pub observed_at_unix_seconds: i64,
    pub configuration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceWorkOrder {
    pub work_order_id: String,
    /// Exact pre-maintenance asset/configuration identity.
    pub target: AssetIdentity,
    pub planned_start_unix_seconds: i64,
    pub planned_end_unix_seconds: i64,
    pub source_event: EvidenceRef,
    /// Must be admitted before actual isolation or the planned outage start.
    pub authorization: Option<EvidenceRef>,
    /// If absent, this is a planned scenario beginning at planned_start.
    pub isolation: Option<EvidenceRef>,
    /// Completion evidence binds the exact post-work configuration.
    pub completion: Option<EvidenceRef>,
    /// None means the configuration is unchanged.
    pub post_work_configuration_digest: Option<String>,
    pub inspection: Option<EvidenceRef>,
    pub return_to_service_authorization: Option<EvidenceRef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityState {
    Available,
    PlannedOutage,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AvailabilityInterval {
    pub start_unix_seconds: i64,
    pub end_unix_seconds: i64,
    pub state: AvailabilityState,
    pub configuration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionDisposition {
    Complete,
    Incomplete,
    NotEvaluable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceAvailabilityTraceV1 {
    pub schema_version: u16,
    pub study_id: String,
    pub study_window: StudyWindow,
    pub target: AssetIdentity,
    pub work_order_id: String,
    pub source_event_refs: Vec<EvidenceRef>,
    pub intervals: Vec<AvailabilityInterval>,
    pub disposition: ProjectionDisposition,
    pub late_start: bool,
    pub late_completion: bool,
    pub late_return_to_service: bool,
    pub overdue_unresolved: bool,
    pub claim_ceiling: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionRequest {
    pub study_id: String,
    pub study_window: StudyWindow,
    pub target: AssetIdentity,
    pub baseline_available: EvidenceRef,
    pub work_order: MaintenanceWorkOrder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    EmptyStudyId,
    EmptyAssetIdentity,
    EmptyWorkOrderId,
    InvalidStudyWindow,
    InvalidPlannedWindow,
    InvalidEvidenceReference,
    DuplicateEvidenceReference,
    BaselineAfterStudyStart,
    BaselineAfterMaintenanceStart,
    AssetIdentityMismatch,
    EvidenceConfigurationMismatch,
    AuthorizationAfterMaintenanceStart,
    InvalidEvidenceOrder,
    ReturnBeforeStudyStart,
    CompletionWithoutIsolation,
    InspectionWithoutCompletion,
    ReturnAuthorizationWithoutInspection,
    ConfigurationChangeWithoutCompletion,
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyStudyId => "study ID is empty",
            Self::EmptyAssetIdentity => "asset ID or configuration digest is empty",
            Self::EmptyWorkOrderId => "work-order ID is empty",
            Self::InvalidStudyWindow => "study window must have start < end",
            Self::InvalidPlannedWindow => "planned window must have start < end",
            Self::InvalidEvidenceReference => "evidence reference or digest is empty",
            Self::DuplicateEvidenceReference => "duplicate evidence reference identity",
            Self::BaselineAfterStudyStart => "baseline evidence is later than study start",
            Self::BaselineAfterMaintenanceStart => "baseline evidence does not predate maintenance",
            Self::AssetIdentityMismatch => "asset identity does not match the study target",
            Self::EvidenceConfigurationMismatch => "evidence is bound to the wrong configuration",
            Self::AuthorizationAfterMaintenanceStart => {
                "authorization was observed after maintenance start"
            }
            Self::InvalidEvidenceOrder => "maintenance evidence timestamps are inconsistent",
            Self::ReturnBeforeStudyStart => "return-to-service occurred before the study window",
            Self::CompletionWithoutIsolation => {
                "completion evidence exists without isolation evidence"
            }
            Self::InspectionWithoutCompletion => {
                "inspection evidence exists without completion evidence"
            }
            Self::ReturnAuthorizationWithoutInspection => {
                "return-to-service authority exists without inspection acceptance"
            }
            Self::ConfigurationChangeWithoutCompletion => {
                "post-work configuration changes without completion evidence"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ProjectionError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceValidationError {
    UnsupportedSchemaVersion,
    EmptyStudyId,
    InvalidStudyWindow,
    InvalidTarget,
    EmptyWorkOrderId,
    EmptyClaimCeiling,
    MissingEvidenceReferences,
    InvalidEvidenceReference,
    DuplicateEvidenceReference,
    MissingIntervals,
    InvalidInterval,
    InvalidIntervalConfiguration,
    IntervalOutsideStudy,
    IntervalGap,
    IntervalOverlap,
    IncompleteCoverage,
}

impl fmt::Display for TraceValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnsupportedSchemaVersion => "unsupported maintenance trace schema version",
            Self::EmptyStudyId => "trace study ID is empty",
            Self::InvalidStudyWindow => "trace study window must have start < end",
            Self::InvalidTarget => "trace asset identity is empty",
            Self::EmptyWorkOrderId => "trace work-order ID is empty",
            Self::EmptyClaimCeiling => "trace claim ceiling is empty",
            Self::MissingEvidenceReferences => "trace has no source evidence references",
            Self::InvalidEvidenceReference => "trace contains an empty evidence reference or digest",
            Self::DuplicateEvidenceReference => "trace contains duplicate evidence references",
            Self::MissingIntervals => "trace has no availability intervals",
            Self::InvalidInterval => "trace contains an empty or reversed interval",
            Self::InvalidIntervalConfiguration => "trace interval has an empty configuration digest",
            Self::IntervalOutsideStudy => "trace interval lies outside its study window",
            Self::IntervalGap => "trace availability intervals contain a gap",
            Self::IntervalOverlap => "trace availability intervals overlap",
            Self::IncompleteCoverage => "trace intervals do not cover the complete study window",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TraceValidationError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn validate_identity(identity: &AssetIdentity) -> Result<(), ProjectionError> {
    if !non_empty(&identity.asset_id) || !non_empty(&identity.configuration_digest) {
        return Err(ProjectionError::EmptyAssetIdentity);
    }
    Ok(())
}

fn validate_evidence(reference: &EvidenceRef) -> Result<(), ProjectionError> {
    if !non_empty(&reference.reference) || !non_empty(&reference.digest) {
        return Err(ProjectionError::InvalidEvidenceReference);
    }
    if !non_empty(&reference.configuration_digest) {
        return Err(ProjectionError::EvidenceConfigurationMismatch);
    }
    Ok(())
}

fn require_configuration(
    evidence: &EvidenceRef,
    expected_digest: &str,
) -> Result<(), ProjectionError> {
    validate_evidence(evidence)?;
    if evidence.configuration_digest != expected_digest {
        return Err(ProjectionError::EvidenceConfigurationMismatch);
    }
    Ok(())
}

fn push_interval(
    intervals: &mut Vec<AvailabilityInterval>,
    start: i64,
    end: i64,
    state: AvailabilityState,
    configuration_digest: &str,
) {
    if start < end {
        intervals.push(AvailabilityInterval {
            start_unix_seconds: start,
            end_unix_seconds: end,
            state,
            configuration_digest: configuration_digest.to_owned(),
        });
    }
}

/// Validate the structural integrity of a serialized study-window trace.
///
/// This checks schema/version, identifiers, source-reference uniqueness, and
/// complete non-overlapping interval coverage. It does not authenticate source
/// digests, verify signer authority, or independently reproduce the projection.
pub fn validate_trace_structure(
    trace: &MaintenanceAvailabilityTraceV1,
) -> Result<(), TraceValidationError> {
    if trace.schema_version != MAINTENANCE_AVAILABILITY_SCHEMA_VERSION {
        return Err(TraceValidationError::UnsupportedSchemaVersion);
    }
    if !non_empty(&trace.study_id) {
        return Err(TraceValidationError::EmptyStudyId);
    }
    if trace.study_window.start_unix_seconds >= trace.study_window.end_unix_seconds {
        return Err(TraceValidationError::InvalidStudyWindow);
    }
    if !non_empty(&trace.target.asset_id)
        || !non_empty(&trace.target.configuration_digest)
    {
        return Err(TraceValidationError::InvalidTarget);
    }
    if !non_empty(&trace.work_order_id) {
        return Err(TraceValidationError::EmptyWorkOrderId);
    }
    if !non_empty(&trace.claim_ceiling) {
        return Err(TraceValidationError::EmptyClaimCeiling);
    }
    if trace.source_event_refs.is_empty() {
        return Err(TraceValidationError::MissingEvidenceReferences);
    }

    let mut seen_references = HashSet::new();
    for evidence in &trace.source_event_refs {
        if !non_empty(&evidence.reference)
            || !non_empty(&evidence.digest)
            || !non_empty(&evidence.configuration_digest)
        {
            return Err(TraceValidationError::InvalidEvidenceReference);
        }
        if !seen_references.insert(evidence.reference.as_str()) {
            return Err(TraceValidationError::DuplicateEvidenceReference);
        }
    }

    if trace.intervals.is_empty() {
        return Err(TraceValidationError::MissingIntervals);
    }

    let study_start = trace.study_window.start_unix_seconds;
    let study_end = trace.study_window.end_unix_seconds;
    let mut expected_start = study_start;

    for interval in &trace.intervals {
        if interval.start_unix_seconds >= interval.end_unix_seconds {
            return Err(TraceValidationError::InvalidInterval);
        }
        if !non_empty(&interval.configuration_digest) {
            return Err(TraceValidationError::InvalidIntervalConfiguration);
        }
        if interval.start_unix_seconds < study_start || interval.end_unix_seconds > study_end {
            return Err(TraceValidationError::IntervalOutsideStudy);
        }
        if interval.start_unix_seconds > expected_start {
            return Err(TraceValidationError::IntervalGap);
        }
        if interval.start_unix_seconds < expected_start {
            return Err(TraceValidationError::IntervalOverlap);
        }
        expected_start = interval.end_unix_seconds;
    }

    if expected_start != study_end {
        return Err(TraceValidationError::IncompleteCoverage);
    }

    Ok(())
}

/// Project one maintenance work order into a deterministic study-window trace.
///
/// A planned end is never a restoration event. Restoration occurs only at the
/// return-to-service authorization timestamp, after completion and inspection
/// evidence for the exact resulting configuration are present. Missing
/// authorization yields Unknown; missing completion, inspection, or return
/// evidence leaves the asset in PlannedOutage through the study end.
///
/// This is a software projection, not proof that source references are authentic
/// or that a real asset is safe, energized, healthy, or operationally available.
pub fn project_maintenance_availability(
    request: &ProjectionRequest,
) -> Result<MaintenanceAvailabilityTraceV1, ProjectionError> {
    if !non_empty(&request.study_id) {
        return Err(ProjectionError::EmptyStudyId);
    }
    validate_identity(&request.target)?;
    validate_identity(&request.work_order.target)?;

    let window = &request.study_window;
    if window.start_unix_seconds >= window.end_unix_seconds {
        return Err(ProjectionError::InvalidStudyWindow);
    }

    let work_order = &request.work_order;
    if !non_empty(&work_order.work_order_id) {
        return Err(ProjectionError::EmptyWorkOrderId);
    }
    if work_order.planned_start_unix_seconds >= work_order.planned_end_unix_seconds {
        return Err(ProjectionError::InvalidPlannedWindow);
    }
    if work_order.target != request.target {
        return Err(ProjectionError::AssetIdentityMismatch);
    }

    require_configuration(
        &request.baseline_available,
        &request.target.configuration_digest,
    )?;
    if request.baseline_available.observed_at_unix_seconds > window.start_unix_seconds {
        return Err(ProjectionError::BaselineAfterStudyStart);
    }

    require_configuration(
        &work_order.source_event,
        &request.target.configuration_digest,
    )?;
    if let Some(authorization) = &work_order.authorization {
        require_configuration(authorization, &request.target.configuration_digest)?;
    }
    if let Some(isolation) = &work_order.isolation {
        require_configuration(isolation, &request.target.configuration_digest)?;
    }

    let post_work_configuration = work_order
        .post_work_configuration_digest
        .as_deref()
        .unwrap_or(&request.target.configuration_digest);
    if !non_empty(post_work_configuration) {
        return Err(ProjectionError::EvidenceConfigurationMismatch);
    }

    if let Some(completion) = &work_order.completion {
        require_configuration(completion, post_work_configuration)?;
    }
    if let Some(inspection) = &work_order.inspection {
        require_configuration(inspection, post_work_configuration)?;
    }
    if let Some(return_authorization) = &work_order.return_to_service_authorization {
        require_configuration(return_authorization, post_work_configuration)?;
    }

    if work_order.post_work_configuration_digest.is_some() && work_order.completion.is_none() {
        return Err(ProjectionError::ConfigurationChangeWithoutCompletion);
    }
    if work_order.completion.is_some() && work_order.isolation.is_none() {
        return Err(ProjectionError::CompletionWithoutIsolation);
    }
    if work_order.inspection.is_some() && work_order.completion.is_none() {
        return Err(ProjectionError::InspectionWithoutCompletion);
    }
    if work_order.return_to_service_authorization.is_some() && work_order.inspection.is_none() {
        return Err(ProjectionError::ReturnAuthorizationWithoutInspection);
    }

    let outage_start = work_order
        .isolation
        .as_ref()
        .map(|evidence| evidence.observed_at_unix_seconds)
        .unwrap_or(work_order.planned_start_unix_seconds);

    if request.baseline_available.observed_at_unix_seconds > outage_start {
        return Err(ProjectionError::BaselineAfterMaintenanceStart);
    }
    if let Some(authorization) = &work_order.authorization {
        if authorization.observed_at_unix_seconds > outage_start {
            return Err(ProjectionError::AuthorizationAfterMaintenanceStart);
        }
        if authorization.observed_at_unix_seconds < work_order.source_event.observed_at_unix_seconds
        {
            return Err(ProjectionError::InvalidEvidenceOrder);
        }
    }

    if let (Some(isolation), Some(completion)) = (&work_order.isolation, &work_order.completion) {
        if completion.observed_at_unix_seconds < isolation.observed_at_unix_seconds {
            return Err(ProjectionError::InvalidEvidenceOrder);
        }
    }
    if let (Some(completion), Some(inspection)) = (&work_order.completion, &work_order.inspection) {
        if inspection.observed_at_unix_seconds < completion.observed_at_unix_seconds {
            return Err(ProjectionError::InvalidEvidenceOrder);
        }
    }
    if let (Some(inspection), Some(return_authorization)) = (
        &work_order.inspection,
        &work_order.return_to_service_authorization,
    ) {
        if return_authorization.observed_at_unix_seconds < inspection.observed_at_unix_seconds {
            return Err(ProjectionError::InvalidEvidenceOrder);
        }
    }

    let mut evidence_refs = vec![
        request.baseline_available.clone(),
        work_order.source_event.clone(),
    ];
    for evidence in [
        work_order.authorization.as_ref(),
        work_order.isolation.as_ref(),
        work_order.completion.as_ref(),
        work_order.inspection.as_ref(),
        work_order.return_to_service_authorization.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        evidence_refs.push(evidence.clone());
    }

    let mut seen_references = HashSet::new();
    if evidence_refs
        .iter()
        .any(|evidence| !seen_references.insert(evidence.reference.as_str()))
    {
        return Err(ProjectionError::DuplicateEvidenceReference);
    }

    let disposition = if work_order.authorization.is_none() {
        ProjectionDisposition::NotEvaluable
    } else if work_order.completion.is_some()
        && work_order.inspection.is_some()
        && work_order.return_to_service_authorization.is_some()
    {
        ProjectionDisposition::Complete
    } else {
        ProjectionDisposition::Incomplete
    };

    // A final authorization cannot repair a missing authorization for the
    // maintenance operation itself. Keep the remainder of the window Unknown.
    let return_at = if work_order.authorization.is_some()
        && work_order.completion.is_some()
        && work_order.inspection.is_some()
    {
        work_order
            .return_to_service_authorization
            .as_ref()
            .map(|evidence| evidence.observed_at_unix_seconds)
    } else {
        None
    };
    if return_at.is_some_and(|timestamp| timestamp < window.start_unix_seconds) {
        return Err(ProjectionError::ReturnBeforeStudyStart);
    }

    let mut intervals = Vec::with_capacity(3);
    let clipped_outage_start = outage_start.max(window.start_unix_seconds);
    push_interval(
        &mut intervals,
        window.start_unix_seconds,
        clipped_outage_start.min(window.end_unix_seconds),
        AvailabilityState::Available,
        &request.target.configuration_digest,
    );

    let outage_state = if work_order.authorization.is_some() {
        AvailabilityState::PlannedOutage
    } else {
        AvailabilityState::Unknown
    };
    let outage_end = return_at.unwrap_or(window.end_unix_seconds);
    push_interval(
        &mut intervals,
        clipped_outage_start,
        outage_end.min(window.end_unix_seconds),
        outage_state,
        &request.target.configuration_digest,
    );

    if let Some(return_at) = return_at {
        push_interval(
            &mut intervals,
            return_at.max(window.start_unix_seconds),
            window.end_unix_seconds,
            AvailabilityState::Available,
            post_work_configuration,
        );
    }

    let late_start = work_order.isolation.as_ref().is_some_and(|evidence| {
        evidence.observed_at_unix_seconds > work_order.planned_start_unix_seconds
    });
    let late_completion = work_order.completion.as_ref().is_some_and(|evidence| {
        evidence.observed_at_unix_seconds > work_order.planned_end_unix_seconds
    });
    let late_return_to_service =
        return_at.is_some_and(|timestamp| timestamp > work_order.planned_end_unix_seconds);
    let overdue_unresolved = window.end_unix_seconds > work_order.planned_end_unix_seconds
        && return_at.is_none_or(|timestamp| timestamp > window.end_unix_seconds);

    Ok(MaintenanceAvailabilityTraceV1 {
        schema_version: MAINTENANCE_AVAILABILITY_SCHEMA_VERSION,
        study_id: request.study_id.clone(),
        study_window: window.clone(),
        target: request.target.clone(),
        work_order_id: work_order.work_order_id.clone(),
        source_event_refs: evidence_refs,
        intervals,
        disposition,
        late_start,
        late_completion,
        late_return_to_service,
        overdue_unresolved,
        claim_ceiling: String::from(
            "Study-scoped software projection for one asset/work order; not asset-health, safety, regulatory, grid-code, switching, energization, or field qualification.",
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD_CONFIG: &str = "config:transformer-rev-a";
    const NEW_CONFIG: &str = "config:transformer-rev-b";

    fn evidence(reference: &str, at: i64, configuration_digest: &str) -> EvidenceRef {
        EvidenceRef {
            reference: reference.to_owned(),
            digest: format!("opaque-digest:{reference}"),
            observed_at_unix_seconds: at,
            configuration_digest: configuration_digest.to_owned(),
        }
    }

    fn request() -> ProjectionRequest {
        ProjectionRequest {
            study_id: "synthetic-three-bus-case".to_owned(),
            study_window: StudyWindow {
                start_unix_seconds: 0,
                end_unix_seconds: 50,
            },
            target: AssetIdentity {
                asset_id: "transformer-t1".to_owned(),
                configuration_digest: OLD_CONFIG.to_owned(),
            },
            baseline_available: evidence("baseline-available", -1, OLD_CONFIG),
            work_order: MaintenanceWorkOrder {
                work_order_id: "wo-17".to_owned(),
                target: AssetIdentity {
                    asset_id: "transformer-t1".to_owned(),
                    configuration_digest: OLD_CONFIG.to_owned(),
                },
                planned_start_unix_seconds: 10,
                planned_end_unix_seconds: 20,
                source_event: evidence("work-order-created", 1, OLD_CONFIG),
                authorization: Some(evidence("work-authorized", 2, OLD_CONFIG)),
                isolation: Some(evidence("asset-isolated", 11, OLD_CONFIG)),
                completion: Some(evidence("work-completed", 28, NEW_CONFIG)),
                post_work_configuration_digest: Some(NEW_CONFIG.to_owned()),
                inspection: Some(evidence("inspection-accepted", 31, NEW_CONFIG)),
                return_to_service_authorization: Some(evidence(
                    "return-authorized",
                    33,
                    NEW_CONFIG,
                )),
            },
        }
    }

    #[test]
    fn complete_trace_restores_only_after_exact_config_acceptance_and_authority() {
        let trace = project_maintenance_availability(&request()).expect("valid trace");

        assert_eq!(trace.disposition, ProjectionDisposition::Complete);
        assert_eq!(
            trace.intervals,
            vec![
                AvailabilityInterval {
                    start_unix_seconds: 0,
                    end_unix_seconds: 11,
                    state: AvailabilityState::Available,
                    configuration_digest: OLD_CONFIG.to_owned(),
                },
                AvailabilityInterval {
                    start_unix_seconds: 11,
                    end_unix_seconds: 33,
                    state: AvailabilityState::PlannedOutage,
                    configuration_digest: OLD_CONFIG.to_owned(),
                },
                AvailabilityInterval {
                    start_unix_seconds: 33,
                    end_unix_seconds: 50,
                    state: AvailabilityState::Available,
                    configuration_digest: NEW_CONFIG.to_owned(),
                },
            ]
        );
        assert!(trace.late_start);
        assert!(trace.late_completion);
        assert!(trace.late_return_to_service);
        assert!(!trace.overdue_unresolved);
    }

    #[test]
    fn planned_end_never_restores_asset_without_acceptance_and_authority() {
        let mut input = request();
        input.work_order.completion = None;
        input.work_order.post_work_configuration_digest = None;
        input.work_order.inspection = None;
        input.work_order.return_to_service_authorization = None;

        let trace = project_maintenance_availability(&input).expect("incomplete is representable");
        assert_eq!(trace.disposition, ProjectionDisposition::Incomplete);
        assert_eq!(trace.intervals.len(), 2);
        assert_eq!(trace.intervals[1].state, AvailabilityState::PlannedOutage);
        assert_eq!(trace.intervals[1].end_unix_seconds, 50);
        assert!(trace.overdue_unresolved);
    }

    #[test]
    fn missing_work_authorization_projects_unknown_not_available() {
        let mut input = request();
        input.work_order.authorization = None;
        input.work_order.isolation = None;
        input.work_order.completion = None;
        input.work_order.post_work_configuration_digest = None;
        input.work_order.inspection = None;
        input.work_order.return_to_service_authorization = None;

        let trace = project_maintenance_availability(&input).expect("unknown is representable");
        assert_eq!(trace.disposition, ProjectionDisposition::NotEvaluable);
        assert_eq!(trace.intervals[1].state, AvailabilityState::Unknown);
    }

    #[test]
    fn return_evidence_does_not_override_missing_work_authorization() {
        let mut input = request();
        input.work_order.authorization = None;

        let trace = project_maintenance_availability(&input).expect("unknown is representable");
        assert_eq!(trace.disposition, ProjectionDisposition::NotEvaluable);
        assert_eq!(trace.intervals.len(), 2);
        assert_eq!(trace.intervals[1].state, AvailabilityState::Unknown);
        assert_eq!(trace.intervals[1].end_unix_seconds, 50);
    }

    #[test]
    fn wrong_replacement_configuration_is_rejected() {
        let mut input = request();
        input.work_order.inspection = Some(evidence("inspection-wrong-rev", 31, OLD_CONFIG));

        assert_eq!(
            project_maintenance_availability(&input),
            Err(ProjectionError::EvidenceConfigurationMismatch)
        );
    }

    #[test]
    fn return_authority_without_inspection_is_rejected() {
        let mut input = request();
        input.work_order.inspection = None;

        assert_eq!(
            project_maintenance_availability(&input),
            Err(ProjectionError::ReturnAuthorizationWithoutInspection)
        );
    }

    #[test]
    fn authorization_after_isolation_is_rejected() {
        let mut input = request();
        input.work_order.authorization = Some(evidence("late-authorization", 12, OLD_CONFIG));

        assert_eq!(
            project_maintenance_availability(&input),
            Err(ProjectionError::AuthorizationAfterMaintenanceStart)
        );
    }

    #[test]
    fn duplicate_source_reference_is_rejected() {
        let mut input = request();
        input.work_order.source_event.reference = "baseline-available".to_owned();

        assert_eq!(
            project_maintenance_availability(&input),
            Err(ProjectionError::DuplicateEvidenceReference)
        );
    }

    #[test]
    fn baseline_for_another_configuration_is_rejected() {
        let mut input = request();
        input.baseline_available.configuration_digest = NEW_CONFIG.to_owned();

        assert_eq!(
            project_maintenance_availability(&input),
            Err(ProjectionError::EvidenceConfigurationMismatch)
        );
    }

    #[test]
    fn invalid_time_order_is_rejected() {
        let mut input = request();
        input.work_order.inspection = Some(evidence("inspection-too-early", 27, NEW_CONFIG));

        assert_eq!(
            project_maintenance_availability(&input),
            Err(ProjectionError::InvalidEvidenceOrder)
        );
    }

    #[test]
    fn return_before_study_window_is_rejected() {
        let mut input = request();
        input.study_window.start_unix_seconds = 35;

        assert_eq!(
            project_maintenance_availability(&input),
            Err(ProjectionError::ReturnBeforeStudyStart)
        );
    }

    #[test]
    fn generated_trace_has_complete_non_overlapping_study_coverage() {
        let trace = project_maintenance_availability(&request()).expect("valid trace");
        assert_eq!(validate_trace_structure(&trace), Ok(()));
    }

    #[test]
    fn trace_validator_rejects_interval_gap() {
        let mut trace = project_maintenance_availability(&request()).expect("valid trace");
        trace.intervals[1].start_unix_seconds += 1;

        assert_eq!(
            validate_trace_structure(&trace),
            Err(TraceValidationError::IntervalGap)
        );
    }

    #[test]
    fn trace_validator_rejects_interval_overlap() {
        let mut trace = project_maintenance_availability(&request()).expect("valid trace");
        trace.intervals[1].start_unix_seconds -= 1;

        assert_eq!(
            validate_trace_structure(&trace),
            Err(TraceValidationError::IntervalOverlap)
        );
    }

    #[test]
    fn trace_validator_rejects_duplicate_evidence_references() {
        let mut trace = project_maintenance_availability(&request()).expect("valid trace");
        trace.source_event_refs[1].reference = trace.source_event_refs[0].reference.clone();

        assert_eq!(
            validate_trace_structure(&trace),
            Err(TraceValidationError::DuplicateEvidenceReference)
        );
    }

    #[test]
    fn trace_validator_rejects_incomplete_study_coverage() {
        let mut trace = project_maintenance_availability(&request()).expect("valid trace");
        trace.intervals[2].end_unix_seconds -= 1;

        assert_eq!(
            validate_trace_structure(&trace),
            Err(TraceValidationError::IncompleteCoverage)
        );
    }

    #[test]
    fn return_after_study_horizon_remains_unresolved_at_horizon() {
        let mut input = request();
        input.study_window.end_unix_seconds = 50;
        input.work_order.return_to_service_authorization =
            Some(evidence("return-authorized-after-horizon", 55, NEW_CONFIG));

        let trace = project_maintenance_availability(&input).expect("valid future restoration");
        assert_eq!(trace.intervals.last().unwrap().end_unix_seconds, 50);
        assert_eq!(
            trace.intervals.last().unwrap().state,
            AvailabilityState::PlannedOutage
        );
        assert!(trace.late_return_to_service);
        assert!(trace.overdue_unresolved);
    }

    #[test]
    fn planned_window_must_be_nonempty() {
        let mut input = request();
        input.work_order.planned_end_unix_seconds = input.work_order.planned_start_unix_seconds;

        assert_eq!(
            project_maintenance_availability(&input),
            Err(ProjectionError::InvalidPlannedWindow)
        );
    }
}
