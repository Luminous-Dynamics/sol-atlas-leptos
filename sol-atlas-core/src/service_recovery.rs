// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-aware service recovery trajectories.
//!
//! This module turns a sequence of validated service-capacity snapshots into
//! bounded recovery timing and normalized unmet-service burden. It reports only
//! what the sampled time series supports. It is not a continuous-time simulator,
//! an outage probability model, or proof that any intervention caused recovery.

use crate::service_function::{
    ServiceFunctionAssessment, ServiceFunctionAssessmentError, ServiceFunctionScenario,
    ServiceFunctionState, assess_service_functions,
};
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SERVICE_RECOVERY_TRAJECTORY_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceRecoveryTrajectory {
    pub schema_version: String,
    pub id: String,
    pub service_entity_id: String,
    /// Timestamp of the first measured disruption snapshot.
    pub disruption_started_at: String,
    /// Any adjacent observations farther apart than this are rejected so the
    /// integrated metric cannot silently bridge arbitrarily large missing windows.
    pub max_sample_gap_seconds: u64,
    /// Time-stamped capacity/demand observations or explicit assumed snapshots.
    pub snapshots: Vec<ServiceFunctionScenario>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceRecoveryTrajectoryAssessment {
    pub schema_version: String,
    pub trajectory_id: String,
    pub service_entity_id: String,
    pub unit: String,
    pub disruption_started_at: String,
    pub observation_ended_at: String,
    pub sample_count: usize,
    pub maximum_observed_sample_gap_seconds: u64,
    pub initial_state: ServiceFunctionState,
    pub final_state: ServiceFunctionState,
    /// The previous sample before first robustly adequate sampled state.
    /// This is a sample bracket, not proof of the exact physical recovery instant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preceding_sample_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preceding_sample_elapsed_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_adequate_sample_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_adequate_sample_elapsed_seconds: Option<u64>,
    /// True when every sampled assessment from the first adequate sample through
    /// the end of this observation window remains adequate. No future guarantee.
    pub observed_adequacy_maintained_through_end: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_adequacy_duration_seconds: Option<u64>,
    /// Integral of lower/upper normalized service-deficit fractions over time,
    /// using trapezoidal integration between samples. Units are equivalent
    /// full-service-deficit seconds, not physical delivered volume.
    pub normalized_deficit_seconds_lower_bound: f64,
    pub normalized_deficit_seconds_upper_bound: f64,
    pub maximum_normalized_deficit_lower_bound: f64,
    pub maximum_normalized_deficit_upper_bound: f64,
    /// Evidence IDs cited by any snapshot, stable and deduplicated.
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRecoveryTrajectoryValidationError {
    UnsupportedSchemaVersion,
    EmptyTrajectoryId,
    EmptyServiceEntityId,
    InvalidDisruptionStart,
    ZeroMaximumSampleGap,
    TooFewSamples,
    EmptySnapshotId,
    DuplicateSnapshotId,
    SnapshotServiceMismatch,
    SnapshotBeforeDisruptionStart,
    FirstSnapshotMustMatchDisruptionStart,
    DuplicateSnapshotTimestamp,
    SamplingGapExceeded,
    UnitMismatch,
    CapacityEntityMismatch,
    InitialDisruptionNotEstablished,
    NonFiniteIntegratedDeficit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceRecoveryTrajectoryError {
    InvalidInput(Vec<ServiceRecoveryTrajectoryValidationError>),
    SnapshotAssessment(ServiceFunctionAssessmentError),
}

#[derive(Debug, Clone, Copy)]
struct DeficitBounds {
    lower: f64,
    upper: f64,
}

pub(crate) fn utc_epoch_seconds(timestamp: &str) -> Option<i64> {
    if !is_canonical_utc_timestamp(timestamp) {
        return None;
    }
    let bytes = timestamp.as_bytes();
    let number = |range: std::ops::Range<usize>| -> i64 {
        bytes[range]
            .iter()
            .fold(0_i64, |acc, byte| acc * 10 + i64::from(*byte - b'0'))
    };
    let year_raw = number(0..4);
    let month = number(5..7);
    let day = number(8..10);
    let hour = number(11..13);
    let minute = number(14..16);
    let second = number(17..19);

    // Gregorian civil date to days since 1970-01-01 (integer arithmetic only).
    let year = year_raw - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146_097 + day_of_era - 719_468;
    Some(days_since_epoch * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn deficit_bounds(assessment: &ServiceFunctionAssessment) -> DeficitBounds {
    let requirement_lower = assessment.required_capacity_lower_bound;
    let requirement_upper = assessment.required_capacity_upper_bound;
    let lower = if requirement_lower > 0.0 {
        (1.0 - assessment.available_capacity_upper_bound / requirement_lower)
            .max(0.0)
            .min(1.0)
    } else {
        0.0
    };
    let upper = if requirement_upper > 0.0 {
        (1.0 - assessment.available_capacity_lower_bound / requirement_upper)
            .max(0.0)
            .min(1.0)
    } else {
        0.0
    };
    DeficitBounds { lower, upper }
}

/// Analyze a time series of individually evidence-validated service snapshots.
///
/// Inputs may be supplied in any order; the output uses chronological order.
/// Integrated deficit is the trapezoidal area under normalized shortage bounds.
/// Large gaps are rejected rather than silently interpolated.
pub fn assess_service_recovery_trajectory(
    trajectory: &ServiceRecoveryTrajectory,
    evidence_records: &[crate::spatial_economic::EvidenceRecord],
) -> Result<ServiceRecoveryTrajectoryAssessment, ServiceRecoveryTrajectoryError> {
    let mut errors = Vec::new();
    if trajectory.schema_version != SERVICE_RECOVERY_TRAJECTORY_SCHEMA_VERSION {
        errors.push(ServiceRecoveryTrajectoryValidationError::UnsupportedSchemaVersion);
    }
    if trajectory.id.trim().is_empty() {
        errors.push(ServiceRecoveryTrajectoryValidationError::EmptyTrajectoryId);
    }
    if trajectory.service_entity_id.trim().is_empty() {
        errors.push(ServiceRecoveryTrajectoryValidationError::EmptyServiceEntityId);
    }
    if utc_epoch_seconds(&trajectory.disruption_started_at).is_none() {
        errors.push(ServiceRecoveryTrajectoryValidationError::InvalidDisruptionStart);
    }
    if trajectory.max_sample_gap_seconds == 0 {
        errors.push(ServiceRecoveryTrajectoryValidationError::ZeroMaximumSampleGap);
    }
    if trajectory.snapshots.len() < 2 {
        errors.push(ServiceRecoveryTrajectoryValidationError::TooFewSamples);
    }

    let mut snapshot_ids = BTreeSet::new();
    let mut sorted_snapshots = trajectory.snapshots.clone();
    for snapshot in &trajectory.snapshots {
        if snapshot.id.trim().is_empty() {
            errors.push(ServiceRecoveryTrajectoryValidationError::EmptySnapshotId);
        }
        if !snapshot_ids.insert(snapshot.id.as_str()) {
            errors.push(ServiceRecoveryTrajectoryValidationError::DuplicateSnapshotId);
        }
        if snapshot.service_entity_id != trajectory.service_entity_id {
            errors.push(ServiceRecoveryTrajectoryValidationError::SnapshotServiceMismatch);
        }
        if utc_epoch_seconds(&snapshot.as_of).is_none() {
            // The downstream service evaluator reports the malformed timestamp with
            // its native typed error. Do not attempt elapsed-time calculation here.
            continue;
        }
        if snapshot.as_of < trajectory.disruption_started_at {
            errors.push(ServiceRecoveryTrajectoryValidationError::SnapshotBeforeDisruptionStart);
        }
    }

    sorted_snapshots.sort_by(|left, right| {
        left.as_of
            .cmp(&right.as_of)
            .then_with(|| left.id.cmp(&right.id))
    });
    if let Some(first) = sorted_snapshots.first() {
        if first.as_of != trajectory.disruption_started_at {
            errors.push(
                ServiceRecoveryTrajectoryValidationError::FirstSnapshotMustMatchDisruptionStart,
            );
        }
    }
    for pair in sorted_snapshots.windows(2) {
        if pair[0].as_of == pair[1].as_of {
            errors.push(ServiceRecoveryTrajectoryValidationError::DuplicateSnapshotTimestamp);
        }
    }

    if let Some(first) = sorted_snapshots.first() {
        for snapshot in &sorted_snapshots {
            if snapshot.available_capacity.quantity.unit != first.available_capacity.quantity.unit
                || snapshot.required_capacity.quantity.unit != first.required_capacity.quantity.unit
            {
                errors.push(ServiceRecoveryTrajectoryValidationError::UnitMismatch);
                break;
            }
            if snapshot.available_capacity.entity_id != first.available_capacity.entity_id {
                errors.push(ServiceRecoveryTrajectoryValidationError::CapacityEntityMismatch);
                break;
            }
        }
    }
    if !errors.is_empty() {
        return Err(ServiceRecoveryTrajectoryError::InvalidInput(errors));
    }

    let assessments = assess_service_functions(&sorted_snapshots, evidence_records)
        .map_err(ServiceRecoveryTrajectoryError::SnapshotAssessment)?;
    let mut chronological: Vec<ServiceFunctionAssessment> = assessments;
    chronological.sort_by(|left, right| left.as_of.cmp(&right.as_of));

    if !matches!(
        chronological.first().map(|snapshot| snapshot.state),
        Some(ServiceFunctionState::Degraded | ServiceFunctionState::Unavailable)
    ) {
        return Err(ServiceRecoveryTrajectoryError::InvalidInput(vec![
            ServiceRecoveryTrajectoryValidationError::InitialDisruptionNotEstablished,
        ]));
    }

    let start_epoch = utc_epoch_seconds(&trajectory.disruption_started_at)
        .expect("canonical timestamp was validated");
    let mut maximum_gap = 0_u64;
    for pair in chronological.windows(2) {
        let left = utc_epoch_seconds(&pair[0].as_of).expect("assessed timestamp is canonical");
        let right = utc_epoch_seconds(&pair[1].as_of).expect("assessed timestamp is canonical");
        let gap = u64::try_from(right - left).unwrap_or(0);
        maximum_gap = maximum_gap.max(gap);
        if gap > trajectory.max_sample_gap_seconds {
            return Err(ServiceRecoveryTrajectoryError::InvalidInput(vec![
                ServiceRecoveryTrajectoryValidationError::SamplingGapExceeded,
            ]));
        }
    }

    let initial_state = chronological[0].state;
    let final_state = chronological.last().expect("at least two snapshots").state;
    let first_adequate_index = chronological
        .iter()
        .position(|snapshot| snapshot.state == ServiceFunctionState::Adequate);
    let (
        preceding_sample_at,
        preceding_sample_elapsed_seconds,
        first_adequate_sample_at,
        first_adequate_sample_elapsed_seconds,
        observed_adequacy_maintained_through_end,
        observed_adequacy_duration_seconds,
    ) = match first_adequate_index {
        Some(index) => {
            let adequate = &chronological[index];
            let adequate_epoch =
                utc_epoch_seconds(&adequate.as_of).expect("assessed timestamp is canonical");
            let preceding = (index > 0).then(|| &chronological[index - 1]);
            let all_later_adequate = chronological[index..]
                .iter()
                .all(|snapshot| snapshot.state == ServiceFunctionState::Adequate);
            let duration = utc_epoch_seconds(&chronological.last().unwrap().as_of)
                .expect("assessed timestamp is canonical")
                - adequate_epoch;
            (
                preceding.map(|snapshot| snapshot.as_of.clone()),
                preceding.map(|snapshot| {
                    let epoch = utc_epoch_seconds(&snapshot.as_of)
                        .expect("assessed timestamp is canonical");
                    u64::try_from(epoch - start_epoch).unwrap_or(0)
                }),
                Some(adequate.as_of.clone()),
                Some(u64::try_from(adequate_epoch - start_epoch).unwrap_or(0)),
                all_later_adequate,
                all_later_adequate.then_some(u64::try_from(duration).unwrap_or(0)),
            )
        }
        None => (None, None, None, None, false, None),
    };

    let mut deficit_samples = Vec::with_capacity(chronological.len());
    let mut evidence_ids = BTreeSet::new();
    for snapshot in &chronological {
        deficit_samples.push(deficit_bounds(snapshot));
        evidence_ids.extend(snapshot.available_capacity_evidence_ids.iter().cloned());
        evidence_ids.extend(snapshot.required_capacity_evidence_ids.iter().cloned());
    }

    let mut deficit_lower_integral = 0.0_f64;
    let mut deficit_upper_integral = 0.0_f64;
    for index in 0..chronological.len() - 1 {
        let left_epoch = utc_epoch_seconds(&chronological[index].as_of)
            .expect("assessed timestamp is canonical");
        let right_epoch = utc_epoch_seconds(&chronological[index + 1].as_of)
            .expect("assessed timestamp is canonical");
        let duration = (right_epoch - left_epoch) as f64;
        deficit_lower_integral +=
            duration * (deficit_samples[index].lower + deficit_samples[index + 1].lower) / 2.0;
        deficit_upper_integral +=
            duration * (deficit_samples[index].upper + deficit_samples[index + 1].upper) / 2.0;
    }

    if !deficit_lower_integral.is_finite()
        || !deficit_upper_integral.is_finite()
        || deficit_lower_integral < 0.0
        || deficit_upper_integral < deficit_lower_integral
    {
        return Err(ServiceRecoveryTrajectoryError::InvalidInput(vec![
            ServiceRecoveryTrajectoryValidationError::NonFiniteIntegratedDeficit,
        ]));
    }

    Ok(ServiceRecoveryTrajectoryAssessment {
        schema_version: SERVICE_RECOVERY_TRAJECTORY_SCHEMA_VERSION.into(),
        trajectory_id: trajectory.id.clone(),
        service_entity_id: trajectory.service_entity_id.clone(),
        unit: chronological[0].unit.clone(),
        disruption_started_at: trajectory.disruption_started_at.clone(),
        observation_ended_at: chronological
            .last()
            .expect("at least two snapshots")
            .as_of
            .clone(),
        sample_count: chronological.len(),
        maximum_observed_sample_gap_seconds: maximum_gap,
        initial_state,
        final_state,
        preceding_sample_at,
        preceding_sample_elapsed_seconds,
        first_adequate_sample_at,
        first_adequate_sample_elapsed_seconds,
        observed_adequacy_maintained_through_end,
        observed_adequacy_duration_seconds,
        normalized_deficit_seconds_lower_bound: deficit_lower_integral,
        normalized_deficit_seconds_upper_bound: deficit_upper_integral,
        maximum_normalized_deficit_lower_bound: deficit_samples
            .iter()
            .map(|x| x.lower)
            .fold(0.0, f64::max),
        maximum_normalized_deficit_upper_bound: deficit_samples
            .iter()
            .map(|x| x.upper)
            .fold(0.0, f64::max),
        evidence_ids: evidence_ids.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::service_function::{FunctionMeasurement, SERVICE_FUNCTION_SCHEMA_VERSION};
    use crate::spatial_economic::Quantity;

    const START: &str = "2026-10-16T10:00:00Z";

    fn measurement(entity: &str, value: f64, unit: &str) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity.into(),
            quantity: Quantity {
                value,
                unit: unit.into(),
                uncertainty_abs: Some(0.0),
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic recovery trajectory assumption".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn sample(id: &str, at: &str, capacity: f64, demand: f64) -> ServiceFunctionScenario {
        ServiceFunctionScenario {
            schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
            id: id.into(),
            as_of: at.into(),
            service_entity_id: "service:water".into(),
            available_capacity: measurement("meter:water-output", capacity, "m3/h"),
            required_capacity: measurement("service:water", demand, "m3/h"),
        }
    }

    fn trajectory(snapshots: Vec<ServiceFunctionScenario>) -> ServiceRecoveryTrajectory {
        ServiceRecoveryTrajectory {
            schema_version: SERVICE_RECOVERY_TRAJECTORY_SCHEMA_VERSION.into(),
            id: "trajectory:water-recovery".into(),
            service_entity_id: "service:water".into(),
            disruption_started_at: START.into(),
            max_sample_gap_seconds: 10,
            snapshots,
        }
    }

    #[test]
    fn json_recovery_trajectory_matches_golden_assessment() {
        let trajectory: ServiceRecoveryTrajectory = serde_json::from_str(include_str!(
            "../../schemas/examples/service-recovery-trajectory.json"
        ))
        .unwrap();
        let expected: ServiceRecoveryTrajectoryAssessment = serde_json::from_str(include_str!(
            "../../schemas/examples/service-recovery-trajectory-assessment.json"
        ))
        .unwrap();
        let actual = assess_service_recovery_trajectory(&trajectory, &[]).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn computes_recovery_bracket_and_normalized_deficit_area() {
        let samples = vec![
            sample("sample-0", START, 4.0, 8.0),
            sample("sample-1", "2026-10-16T10:00:10Z", 6.0, 8.0),
            sample("sample-2", "2026-10-16T10:00:20Z", 8.0, 8.0),
            sample("sample-3", "2026-10-16T10:00:30Z", 8.0, 8.0),
        ];
        let result = assess_service_recovery_trajectory(&trajectory(samples), &[]).unwrap();
        assert_eq!(result.initial_state, ServiceFunctionState::Degraded);
        assert_eq!(result.final_state, ServiceFunctionState::Adequate);
        assert_eq!(result.preceding_sample_elapsed_seconds, Some(10));
        assert_eq!(result.first_adequate_sample_elapsed_seconds, Some(20));
        assert_eq!(result.observed_adequacy_maintained_through_end, true);
        assert_eq!(result.observed_adequacy_duration_seconds, Some(10));
        assert_eq!(result.normalized_deficit_seconds_lower_bound, 5.0);
        assert_eq!(result.normalized_deficit_seconds_upper_bound, 5.0);
        assert_eq!(result.maximum_normalized_deficit_lower_bound, 0.5);
        assert_eq!(result.maximum_normalized_deficit_upper_bound, 0.5);
    }

    #[test]
    fn recovery_is_not_called_sustained_when_service_degrades_again() {
        let samples = vec![
            sample("sample-0", START, 4.0, 8.0),
            sample("sample-1", "2026-10-16T10:00:10Z", 8.0, 8.0),
            sample("sample-2", "2026-10-16T10:00:20Z", 6.0, 8.0),
            sample("sample-3", "2026-10-16T10:00:30Z", 8.0, 8.0),
        ];
        let result = assess_service_recovery_trajectory(&trajectory(samples), &[]).unwrap();
        assert_eq!(result.first_adequate_sample_elapsed_seconds, Some(10));
        assert_eq!(result.observed_adequacy_maintained_through_end, false);
        assert_eq!(result.observed_adequacy_duration_seconds, None);
    }

    #[test]
    fn sample_input_order_does_not_change_trajectory_results() {
        let ordered = vec![
            sample("sample-0", START, 4.0, 8.0),
            sample("sample-1", "2026-10-16T10:00:10Z", 6.0, 8.0),
            sample("sample-2", "2026-10-16T10:00:20Z", 8.0, 8.0),
        ];
        let reversed: Vec<_> = ordered.iter().cloned().rev().collect();
        assert_eq!(
            assess_service_recovery_trajectory(&trajectory(ordered), &[]).unwrap(),
            assess_service_recovery_trajectory(&trajectory(reversed), &[]).unwrap()
        );
    }

    #[test]
    fn uncertain_capacity_produces_bounded_deficit_not_false_precision() {
        let mut low = sample("sample-0", START, 7.0, 10.0);
        low.available_capacity.quantity.uncertainty_abs = Some(2.0);
        let mut high = sample("sample-1", "2026-10-16T10:00:10Z", 10.0, 10.0);
        high.available_capacity.quantity.uncertainty_abs = Some(1.0);
        high.required_capacity.quantity.uncertainty_abs = Some(1.0);
        let result = assess_service_recovery_trajectory(&trajectory(vec![low, high]), &[]).unwrap();
        assert!(
            result.normalized_deficit_seconds_lower_bound
                <= result.normalized_deficit_seconds_upper_bound
        );
        assert!(
            result.maximum_normalized_deficit_lower_bound
                <= result.maximum_normalized_deficit_upper_bound
        );
    }

    #[test]
    fn elapsed_seconds_are_correct_across_calendar_boundaries() {
        let trajectory = ServiceRecoveryTrajectory {
            schema_version: SERVICE_RECOVERY_TRAJECTORY_SCHEMA_VERSION.into(),
            id: "trajectory:year-boundary".into(),
            service_entity_id: "service:water".into(),
            disruption_started_at: "2026-12-31T23:59:59Z".into(),
            max_sample_gap_seconds: 1,
            snapshots: vec![
                ServiceFunctionScenario {
                    schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
                    id: "before-year-boundary".into(),
                    as_of: "2026-12-31T23:59:59Z".into(),
                    service_entity_id: "service:water".into(),
                    available_capacity: measurement("meter:water-output", 4.0, "m3/h"),
                    required_capacity: measurement("service:water", 8.0, "m3/h"),
                },
                ServiceFunctionScenario {
                    schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
                    id: "after-year-boundary".into(),
                    as_of: "2027-01-01T00:00:00Z".into(),
                    service_entity_id: "service:water".into(),
                    available_capacity: measurement("meter:water-output", 8.0, "m3/h"),
                    required_capacity: measurement("service:water", 8.0, "m3/h"),
                },
            ],
        };
        let result = assess_service_recovery_trajectory(&trajectory, &[]).unwrap();
        assert_eq!(result.maximum_observed_sample_gap_seconds, 1);
        assert_eq!(result.first_adequate_sample_elapsed_seconds, Some(1));
        assert_eq!(result.normalized_deficit_seconds_lower_bound, 0.25);
        assert_eq!(result.normalized_deficit_seconds_upper_bound, 0.25);
    }

    #[test]
    fn excessive_sampling_gap_fails_closed() {
        let result = assess_service_recovery_trajectory(
            &trajectory(vec![
                sample("sample-0", START, 4.0, 8.0),
                sample("sample-1", "2026-10-16T10:00:11Z", 8.0, 8.0),
            ]),
            &[],
        );
        assert!(matches!(
            result,
            Err(ServiceRecoveryTrajectoryError::InvalidInput(errors))
                if errors.contains(&ServiceRecoveryTrajectoryValidationError::SamplingGapExceeded)
        ));
    }

    #[test]
    fn initial_adequate_or_indeterminate_state_does_not_claim_recovery() {
        let adequate = assess_service_recovery_trajectory(
            &trajectory(vec![
                sample("sample-0", START, 8.0, 8.0),
                sample("sample-1", "2026-10-16T10:00:10Z", 8.0, 8.0),
            ]),
            &[],
        );
        assert!(matches!(
            adequate,
            Err(ServiceRecoveryTrajectoryError::InvalidInput(errors))
                if errors.contains(&ServiceRecoveryTrajectoryValidationError::InitialDisruptionNotEstablished)
        ));

        let mut indeterminate_sample = sample("sample-0", START, 7.0, 8.0);
        indeterminate_sample
            .available_capacity
            .quantity
            .uncertainty_abs = Some(2.0);
        let indeterminate = assess_service_recovery_trajectory(
            &trajectory(vec![
                indeterminate_sample,
                sample("sample-1", "2026-10-16T10:00:10Z", 8.0, 8.0),
            ]),
            &[],
        );
        assert!(matches!(
            indeterminate,
            Err(ServiceRecoveryTrajectoryError::InvalidInput(errors))
                if errors.contains(&ServiceRecoveryTrajectoryValidationError::InitialDisruptionNotEstablished)
        ));
    }

    #[test]
    fn duplicate_timestamps_and_mismatched_services_are_rejected() {
        assert!(matches!(
            assess_service_recovery_trajectory(
                &trajectory(vec![
                    sample("sample-0", START, 4.0, 8.0),
                    sample("sample-1", START, 8.0, 8.0),
                ]),
                &[],
            ),
            Err(ServiceRecoveryTrajectoryError::InvalidInput(errors))
                if errors.contains(&ServiceRecoveryTrajectoryValidationError::DuplicateSnapshotTimestamp)
        ));

        let mut wrong = sample("sample-1", "2026-10-16T10:00:10Z", 8.0, 8.0);
        wrong.service_entity_id = "service:other".into();
        assert!(matches!(
            assess_service_recovery_trajectory(&trajectory(vec![
                sample("sample-0", START, 4.0, 8.0),
                wrong,
            ]), &[]),
            Err(ServiceRecoveryTrajectoryError::InvalidInput(errors))
                if errors.contains(&ServiceRecoveryTrajectoryValidationError::SnapshotServiceMismatch)
        ));
    }
}
