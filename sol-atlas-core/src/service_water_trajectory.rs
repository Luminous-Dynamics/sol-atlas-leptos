// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-aware time-dependent water-service resilience accounting.
//!
//! This is a bounded aggregate model. It carries the modeled storage interval
//! from one contiguous time step to the next and integrates unmet-demand volume.
//! It does not replace hydraulic simulation, estimate outage probabilities, or
//! establish observed physical delivery or intervention causality.

use crate::service_function::{
    FunctionMeasurement, ServiceFunctionAssessmentError, build_evidence_index,
};
use crate::service_recovery::utc_epoch_seconds;
use crate::service_water::{
    WATER_RATE_UNIT, WATER_SERVICE_SCHEMA_VERSION, WATER_VOLUME_UNIT, WaterServiceError,
    WaterServiceScenario, WaterServiceState, assess_water_service_indexed,
};
use crate::spatial_economic::EvidenceRecord;
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceTrajectoryStep {
    pub id: String,
    /// Timestamp at the end of the interval represented by this step.
    pub as_of: String,
    /// Must resolve to an integer number of seconds; adjacent steps must be contiguous.
    pub step_hours: f64,
    pub source_flow_rate: FunctionMeasurement,
    pub pump_capacity: FunctionMeasurement,
    pub treatment_capacity: FunctionMeasurement,
    pub distribution_capacity: FunctionMeasurement,
    pub demand_rate: FunctionMeasurement,
    pub distribution_loss_fraction: FunctionMeasurement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceTrajectory {
    pub schema_version: String,
    pub id: String,
    pub service_entity_id: String,
    /// Initial treated-water inventory before the first modeled interval.
    pub initial_storage: FunctionMeasurement,
    /// Stable physical storage capacity for all steps in this trajectory.
    pub storage_capacity: FunctionMeasurement,
    /// Ordered, contiguous intervals. Outages are explicit step assumptions or observations.
    pub steps: Vec<WaterServiceTrajectoryStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterStorageStateOrigin {
    InitialScenarioInput,
    DerivedFromPreviousStep,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceTrajectoryStepAssessment {
    pub step_id: String,
    pub storage_state_origin: WaterStorageStateOrigin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_source_step_id: Option<String>,
    pub as_of: String,
    pub step_hours: f64,
    pub state: WaterServiceState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_lower_bound: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_upper_bound: Option<f64>,
    pub demand_volume_lower_bound: f64,
    pub demand_volume_upper_bound: f64,
    pub customer_delivered_volume_lower_bound: f64,
    pub customer_delivered_volume_upper_bound: f64,
    pub ending_storage_lower_bound: f64,
    pub ending_storage_upper_bound: f64,
    pub conservative_mass_balance_residual: f64,
    pub optimistic_mass_balance_residual: f64,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceTrajectoryAssessment {
    pub schema_version: String,
    pub trajectory_id: String,
    pub service_entity_id: String,
    pub rate_unit: String,
    pub volume_unit: String,
    pub time_unit: String,
    pub step_count: usize,
    pub first_step_as_of: String,
    pub last_step_as_of: String,
    pub initial_state: WaterServiceState,
    pub final_state: WaterServiceState,
    /// First robustly adequate sampled interval; this is not a continuous-time recovery claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_adequate_step_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_adequate_as_of: Option<String>,
    /// Previous sampled step before the first robustly adequate interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preceding_step_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preceding_step_as_of: Option<String>,
    /// True only when every sampled step from first adequate through the final sample is adequate.
    pub adequacy_maintained_through_last_sample: bool,
    /// Sum of interval unmet-demand volume bounds; not a forecast beyond the last step.
    pub unmet_demand_volume_lower_bound: f64,
    pub unmet_demand_volume_upper_bound: f64,
    pub customer_delivered_volume_lower_bound: f64,
    pub customer_delivered_volume_upper_bound: f64,
    pub maximum_normalized_deficit_lower_bound: f64,
    pub maximum_normalized_deficit_upper_bound: f64,
    pub steps: Vec<WaterServiceTrajectoryStepAssessment>,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterServiceTrajectoryValidationError {
    UnsupportedSchemaVersion,
    EmptyTrajectorySet,
    DuplicateTrajectoryId,
    EmptyTrajectoryId,
    EmptyServiceEntityId,
    TooFewSteps,
    EmptyStepId,
    DuplicateStepId,
    InvalidTimestamp,
    InvalidStepHours,
    NonContiguousSteps,
    ComponentEntityChanged,
    ServiceEntityMismatch,
    StorageEntityMismatch,
    NonFiniteDerivedValue,
    NonFiniteAggregate,
    BoundsInconsistent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterServiceTrajectoryError {
    InvalidInput(Vec<WaterServiceTrajectoryValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
    StepAssessment(WaterServiceError),
}

fn derived_storage_measurement(
    entity_id: &str,
    low: f64,
    high: f64,
) -> Result<FunctionMeasurement, WaterServiceTrajectoryError> {
    if !low.is_finite() || !high.is_finite() || low < 0.0 || high < low {
        return Err(WaterServiceTrajectoryError::InvalidInput(vec![
            WaterServiceTrajectoryValidationError::NonFiniteDerivedValue,
        ]));
    }
    let midpoint = low + (high - low) / 2.0;
    let uncertainty = (high - low) / 2.0;
    Ok(FunctionMeasurement {
        entity_id: entity_id.to_owned(),
        quantity: crate::spatial_economic::Quantity {
            value: midpoint,
            unit: WATER_VOLUME_UNIT.to_owned(),
            uncertainty_abs: Some(uncertainty),
        },
        basis: crate::dependency_logic::ScenarioBasis::Assumed,
        rationale: "Modeled interval carried forward from the preceding step; not an observation."
            .to_owned(),
        supporting_evidence_ids: Vec::new(),
    })
}

fn validate_trajectory(
    trajectory: &WaterServiceTrajectory,
) -> Result<(), Vec<WaterServiceTrajectoryValidationError>> {
    let mut errors = Vec::new();
    if trajectory.schema_version != WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION {
        errors.push(WaterServiceTrajectoryValidationError::UnsupportedSchemaVersion);
    }
    if trajectory.id.trim().is_empty() {
        errors.push(WaterServiceTrajectoryValidationError::EmptyTrajectoryId);
    }
    if trajectory.service_entity_id.trim().is_empty() {
        errors.push(WaterServiceTrajectoryValidationError::EmptyServiceEntityId);
    }
    if trajectory.steps.len() < 2 {
        errors.push(WaterServiceTrajectoryValidationError::TooFewSteps);
    }
    if trajectory.initial_storage.entity_id != trajectory.storage_capacity.entity_id {
        errors.push(WaterServiceTrajectoryValidationError::StorageEntityMismatch);
    }

    let mut ids = BTreeSet::new();
    for step in &trajectory.steps {
        if step.id.trim().is_empty() {
            errors.push(WaterServiceTrajectoryValidationError::EmptyStepId);
        }
        if !ids.insert(step.id.as_str()) {
            errors.push(WaterServiceTrajectoryValidationError::DuplicateStepId);
        }
        if !is_canonical_utc_timestamp(&step.as_of) || utc_epoch_seconds(&step.as_of).is_none() {
            errors.push(WaterServiceTrajectoryValidationError::InvalidTimestamp);
        }
        if !step.step_hours.is_finite()
            || step.step_hours <= 0.0
            || !(step.step_hours * 3_600.0).is_finite()
            || (step.step_hours * 3_600.0).fract().abs() > 1e-9
        {
            errors.push(WaterServiceTrajectoryValidationError::InvalidStepHours);
        }
        if step.demand_rate.entity_id != trajectory.service_entity_id {
            errors.push(WaterServiceTrajectoryValidationError::ServiceEntityMismatch);
        }
        if step.distribution_loss_fraction.entity_id != step.distribution_capacity.entity_id {
            errors.push(WaterServiceTrajectoryValidationError::ComponentEntityChanged);
        }
    }

    if let Some(first) = trajectory.steps.first() {
        for later in trajectory.steps.iter().skip(1) {
            if first.source_flow_rate.entity_id != later.source_flow_rate.entity_id
                || first.pump_capacity.entity_id != later.pump_capacity.entity_id
                || first.treatment_capacity.entity_id != later.treatment_capacity.entity_id
                || first.distribution_capacity.entity_id != later.distribution_capacity.entity_id
                || first.demand_rate.entity_id != later.demand_rate.entity_id
                || first.distribution_loss_fraction.entity_id
                    != later.distribution_loss_fraction.entity_id
            {
                errors.push(WaterServiceTrajectoryValidationError::ComponentEntityChanged);
                break;
            }
        }
    }

    for pair in trajectory.steps.windows(2) {
        let Some(previous) = utc_epoch_seconds(&pair[0].as_of) else {
            continue;
        };
        let Some(current) = utc_epoch_seconds(&pair[1].as_of) else {
            continue;
        };
        let duration = pair[0].step_hours * 3_600.0;
        if !duration.is_finite()
            || duration <= 0.0
            || duration > u64::MAX as f64
            || (duration.fract().abs() > 1e-9)
            || current.saturating_sub(previous) != duration.round() as i64
        {
            errors.push(WaterServiceTrajectoryValidationError::NonContiguousSteps);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub(crate) fn assess_trajectory_indexed(
    trajectory: &WaterServiceTrajectory,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<WaterServiceTrajectoryAssessment, WaterServiceTrajectoryError> {
    validate_trajectory(trajectory).map_err(WaterServiceTrajectoryError::InvalidInput)?;

    let mut step_results = Vec::with_capacity(trajectory.steps.len());
    let mut evidence_ids = BTreeSet::new();
    let mut previous_storage_bounds: Option<(f64, f64)> = None;
    let mut demand_lower_total = 0.0;
    let mut demand_upper_total = 0.0;
    let mut delivery_lower_total = 0.0;
    let mut delivery_upper_total = 0.0;
    let mut deficit_lower_total = 0.0;
    let mut deficit_upper_total = 0.0;
    let mut maximum_deficit_lower: f64 = 0.0;
    let mut maximum_deficit_upper: f64 = 0.0;

    for (index, step) in trajectory.steps.iter().enumerate() {
        let initial_storage = if index == 0 {
            trajectory.initial_storage.clone()
        } else {
            let (low, high) =
                previous_storage_bounds.expect("previous step must establish storage");
            derived_storage_measurement(&trajectory.initial_storage.entity_id, low, high)?
        };
        let storage_state_origin = if index == 0 {
            WaterStorageStateOrigin::InitialScenarioInput
        } else {
            WaterStorageStateOrigin::DerivedFromPreviousStep
        };
        let storage_source_step_id = index
            .checked_sub(1)
            .map(|previous_index| trajectory.steps[previous_index].id.clone());
        let scenario = WaterServiceScenario {
            schema_version: WATER_SERVICE_SCHEMA_VERSION.to_owned(),
            id: format!("{}:{}", trajectory.id, step.id),
            as_of: step.as_of.clone(),
            service_entity_id: trajectory.service_entity_id.clone(),
            step_hours: step.step_hours,
            source_flow_rate: step.source_flow_rate.clone(),
            pump_capacity: step.pump_capacity.clone(),
            treatment_capacity: step.treatment_capacity.clone(),
            distribution_capacity: step.distribution_capacity.clone(),
            initial_storage,
            storage_capacity: trajectory.storage_capacity.clone(),
            demand_rate: step.demand_rate.clone(),
            distribution_loss_fraction: step.distribution_loss_fraction.clone(),
        };
        let assessment = assess_water_service_indexed(&scenario, evidence_by_id)
            .map_err(WaterServiceTrajectoryError::StepAssessment)?;

        evidence_ids.extend(assessment.supporting_evidence_ids.iter().cloned());

        let demand_lower = assessment.demand_volume_lower_bound;
        let demand_upper = assessment.demand_volume_upper_bound;
        let delivered_lower = assessment.conservative_path.customer_delivered_volume;
        let delivered_upper = assessment.optimistic_path.customer_delivered_volume;
        let deficit_lower = (demand_lower - delivered_upper).max(0.0);
        let deficit_upper = (demand_upper - delivered_lower).max(0.0);

        if [
            demand_lower,
            demand_upper,
            delivered_lower,
            delivered_upper,
            deficit_lower,
            deficit_upper,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
        {
            return Err(WaterServiceTrajectoryError::InvalidInput(vec![
                WaterServiceTrajectoryValidationError::NonFiniteDerivedValue,
            ]));
        }
        let step_tolerance = f64::EPSILON * 128.0 * demand_upper.max(delivered_upper).max(1.0);
        if demand_lower > demand_upper + step_tolerance
            || delivered_lower > delivered_upper + step_tolerance
            || assessment.conservative_path.ending_storage_volume
                > assessment.optimistic_path.ending_storage_volume + step_tolerance
        {
            return Err(WaterServiceTrajectoryError::InvalidInput(vec![
                WaterServiceTrajectoryValidationError::BoundsInconsistent,
            ]));
        }

        demand_lower_total += demand_lower;
        demand_upper_total += demand_upper;
        delivery_lower_total += delivered_lower;
        delivery_upper_total += delivered_upper;
        deficit_lower_total += deficit_lower;
        deficit_upper_total += deficit_upper;

        // Tight normalized-deficit bounds use the demand denominator that
        // minimizes/maximizes the ratio, independently of the volume bounds.
        let normalized_low = if demand_lower > 0.0 {
            (1.0 - delivered_upper / demand_lower).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let normalized_high = if demand_upper > 0.0 {
            (1.0 - delivered_lower / demand_upper).clamp(0.0, 1.0)
        } else {
            0.0
        };
        maximum_deficit_lower = maximum_deficit_lower.max(normalized_low);
        maximum_deficit_upper = maximum_deficit_upper.max(normalized_high);

        previous_storage_bounds = Some((
            assessment.conservative_path.ending_storage_volume,
            assessment.optimistic_path.ending_storage_volume,
        ));
        step_results.push(WaterServiceTrajectoryStepAssessment {
            step_id: step.id.clone(),
            storage_state_origin,
            storage_source_step_id,
            as_of: step.as_of.clone(),
            step_hours: step.step_hours,
            state: assessment.state,
            coverage_lower_bound: assessment.coverage_lower_bound,
            coverage_upper_bound: assessment.coverage_upper_bound,
            demand_volume_lower_bound: demand_lower,
            demand_volume_upper_bound: demand_upper,
            customer_delivered_volume_lower_bound: delivered_lower,
            customer_delivered_volume_upper_bound: delivered_upper,
            ending_storage_lower_bound: assessment.conservative_path.ending_storage_volume,
            ending_storage_upper_bound: assessment.optimistic_path.ending_storage_volume,
            conservative_mass_balance_residual: assessment.conservative_path.mass_balance_residual,
            optimistic_mass_balance_residual: assessment.optimistic_path.mass_balance_residual,
            supporting_evidence_ids: assessment.supporting_evidence_ids.clone(),
        });
    }

    if [
        demand_lower_total,
        demand_upper_total,
        delivery_lower_total,
        delivery_upper_total,
        deficit_lower_total,
        deficit_upper_total,
        maximum_deficit_lower,
        maximum_deficit_upper,
    ]
    .into_iter()
    .any(|value| !value.is_finite())
    {
        return Err(WaterServiceTrajectoryError::InvalidInput(vec![
            WaterServiceTrajectoryValidationError::NonFiniteAggregate,
        ]));
    }

    // Recovery means adequate after an observed modeled deficit, not simply an
    // initially healthy first sample. This avoids labelling baseline adequacy
    // as recovery when an outage occurs later in the trajectory.
    let aggregate_tolerance =
        f64::EPSILON * 256.0 * demand_upper_total.max(delivery_upper_total).max(1.0);
    if demand_lower_total > demand_upper_total + aggregate_tolerance
        || delivery_lower_total > delivery_upper_total + aggregate_tolerance
        || deficit_lower_total > deficit_upper_total + aggregate_tolerance
        || maximum_deficit_lower > maximum_deficit_upper + f64::EPSILON * 256.0
    {
        return Err(WaterServiceTrajectoryError::InvalidInput(vec![
            WaterServiceTrajectoryValidationError::BoundsInconsistent,
        ]));
    }

    let first_adequate_index = step_results.iter().enumerate().position(|(index, item)| {
        item.state == WaterServiceState::Adequate
            && step_results[..index]
                .iter()
                .any(|prior| prior.state != WaterServiceState::Adequate)
    });
    let first_adequate_step_id =
        first_adequate_index.map(|index| step_results[index].step_id.clone());
    let first_adequate_as_of = first_adequate_index.map(|index| step_results[index].as_of.clone());
    let preceding_step_id = first_adequate_index
        .and_then(|index| index.checked_sub(1))
        .map(|index| step_results[index].step_id.clone());
    let preceding_step_as_of = first_adequate_index
        .and_then(|index| index.checked_sub(1))
        .map(|index| step_results[index].as_of.clone());
    let adequacy_maintained_through_last_sample = first_adequate_index.is_some_and(|index| {
        step_results[index..]
            .iter()
            .all(|step| step.state == WaterServiceState::Adequate)
    });

    Ok(WaterServiceTrajectoryAssessment {
        schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
        trajectory_id: trajectory.id.clone(),
        service_entity_id: trajectory.service_entity_id.clone(),
        rate_unit: WATER_RATE_UNIT.to_owned(),
        volume_unit: WATER_VOLUME_UNIT.to_owned(),
        time_unit: "h".to_owned(),
        step_count: step_results.len(),
        first_step_as_of: step_results
            .first()
            .expect("validated non-empty")
            .as_of
            .clone(),
        last_step_as_of: step_results
            .last()
            .expect("validated non-empty")
            .as_of
            .clone(),
        initial_state: step_results.first().expect("validated non-empty").state,
        final_state: step_results.last().expect("validated non-empty").state,
        first_adequate_step_id,
        first_adequate_as_of,
        preceding_step_id,
        preceding_step_as_of,
        adequacy_maintained_through_last_sample,
        unmet_demand_volume_lower_bound: deficit_lower_total,
        unmet_demand_volume_upper_bound: deficit_upper_total,
        customer_delivered_volume_lower_bound: delivery_lower_total,
        customer_delivered_volume_upper_bound: delivery_upper_total,
        maximum_normalized_deficit_lower_bound: maximum_deficit_lower,
        maximum_normalized_deficit_upper_bound: maximum_deficit_upper,
        steps: step_results,
        supporting_evidence_ids: evidence_ids.into_iter().collect(),
    })
}

/// Evaluate a time series with one validated evidence index for all steps.
pub fn assess_water_service_trajectory(
    trajectory: &WaterServiceTrajectory,
    evidence_records: &[EvidenceRecord],
) -> Result<WaterServiceTrajectoryAssessment, WaterServiceTrajectoryError> {
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(WaterServiceTrajectoryError::EvidenceValidation)?;
    assess_trajectory_indexed(trajectory, &evidence_by_id)
}

/// Batch-evaluate trajectories against one shared evidence corpus; duplicate trajectory
/// IDs fail closed, and outputs are ordered by stable trajectory ID.
pub fn assess_water_service_trajectories(
    trajectories: &[WaterServiceTrajectory],
    evidence_records: &[EvidenceRecord],
) -> Result<Vec<WaterServiceTrajectoryAssessment>, WaterServiceTrajectoryError> {
    if trajectories.is_empty() {
        return Err(WaterServiceTrajectoryError::InvalidInput(vec![
            WaterServiceTrajectoryValidationError::EmptyTrajectorySet,
        ]));
    }
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(WaterServiceTrajectoryError::EvidenceValidation)?;
    let mut ids = BTreeSet::new();
    for trajectory in trajectories {
        if !ids.insert(trajectory.id.as_str()) {
            return Err(WaterServiceTrajectoryError::InvalidInput(vec![
                WaterServiceTrajectoryValidationError::DuplicateTrajectoryId,
            ]));
        }
    }
    let mut ordered: Vec<&WaterServiceTrajectory> = trajectories.iter().collect();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    ordered
        .into_iter()
        .map(|trajectory| assess_trajectory_indexed(trajectory, &evidence_by_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::service_water::DIMENSIONLESS_UNIT;
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceRecord, EvidenceState, Provenance, Quantity,
        SpatialContext, TimeContext,
    };

    fn measurement(entity: &str, value: f64, unit: &str) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity.to_owned(),
            quantity: Quantity {
                value,
                unit: unit.to_owned(),
                uncertainty_abs: None,
            },
            basis: ScenarioBasis::Assumed,
            rationale: format!("Synthetic trajectory assumption for {entity}"),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn step(id: &str, hour: u32, source_rate: f64) -> WaterServiceTrajectoryStep {
        WaterServiceTrajectoryStep {
            id: id.to_owned(),
            as_of: format!("2026-10-16T{hour:02}:00:00Z"),
            step_hours: 1.0,
            source_flow_rate: measurement("source-a", source_rate, WATER_RATE_UNIT),
            pump_capacity: measurement("pump-a", 100.0, WATER_RATE_UNIT),
            treatment_capacity: measurement("treatment-a", 100.0, WATER_RATE_UNIT),
            distribution_capacity: measurement("distribution-a", 120.0, WATER_RATE_UNIT),
            demand_rate: measurement("service-north", 100.0, WATER_RATE_UNIT),
            distribution_loss_fraction: measurement("distribution-a", 0.0, DIMENSIONLESS_UNIT),
        }
    }

    fn trajectory(id: &str) -> WaterServiceTrajectory {
        WaterServiceTrajectory {
            schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
            id: id.to_owned(),
            service_entity_id: "service-north".to_owned(),
            initial_storage: measurement("tank-a", 50.0, WATER_VOLUME_UNIT),
            storage_capacity: measurement("tank-a", 150.0, WATER_VOLUME_UNIT),
            steps: vec![
                step("normal", 10, 100.0),
                step("outage-1", 11, 0.0),
                step("outage-2", 12, 0.0),
                step("recovery", 13, 100.0),
            ],
        }
    }

    #[test]
    fn storage_carries_across_outage_and_recovery() {
        let result = assess_water_service_trajectory(&trajectory("north"), &[]).unwrap();
        assert_eq!(result.steps[0].state, WaterServiceState::Adequate);
        assert_eq!(result.steps[1].state, WaterServiceState::Degraded);
        assert_eq!(
            result.steps[1].storage_state_origin,
            WaterStorageStateOrigin::DerivedFromPreviousStep
        );
        assert_eq!(
            result.steps[1].storage_source_step_id.as_deref(),
            Some("normal")
        );
        assert_eq!(result.steps[2].state, WaterServiceState::Unavailable);
        assert_eq!(
            result.steps[2].storage_source_step_id.as_deref(),
            Some("outage-1")
        );
        assert_eq!(result.steps[3].state, WaterServiceState::Adequate);
        assert_eq!(result.unmet_demand_volume_lower_bound, 150.0);
        assert_eq!(result.unmet_demand_volume_upper_bound, 150.0);
        assert_eq!(result.customer_delivered_volume_lower_bound, 250.0);
        assert_eq!(result.customer_delivered_volume_upper_bound, 250.0);
        assert_eq!(result.first_adequate_step_id.as_deref(), Some("recovery"));
        assert_eq!(result.preceding_step_id.as_deref(), Some("outage-2"));
        assert!(result.adequacy_maintained_through_last_sample);
        assert!(result.steps.iter().all(|step| {
            step.conservative_mass_balance_residual.abs() <= 1e-9
                && step.optimistic_mass_balance_residual.abs() <= 1e-9
                && step.ending_storage_lower_bound <= 150.0
                && step.ending_storage_upper_bound <= 150.0
        }));
    }

    #[test]
    fn recovery_after_outage_is_bracketed_not_claimed_continuous() {
        let mut input = trajectory("recovery-bracket");
        input.steps.remove(0);
        input.steps[0] = step("outage-1", 11, 0.0);
        input.steps[1] = step("outage-2", 12, 0.0);
        input.steps[2] = step("recovery", 13, 100.0);
        let result = assess_water_service_trajectory(&input, &[]).unwrap();
        assert_eq!(result.first_adequate_step_id.as_deref(), Some("recovery"));
        assert_eq!(result.preceding_step_id.as_deref(), Some("outage-2"));
        assert!(result.adequacy_maintained_through_last_sample);
    }

    #[test]
    fn batch_is_order_invariant_and_duplicate_trajectory_ids_fail_closed() {
        let inputs = [trajectory("zeta"), trajectory("alpha")];
        let first = assess_water_service_trajectories(&inputs, &[]).unwrap();
        let second_inputs = [trajectory("alpha"), trajectory("zeta")];
        let second = assess_water_service_trajectories(&second_inputs, &[]).unwrap();
        assert_eq!(first[0].trajectory_id, "alpha");
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );
        assert!(matches!(
            assess_water_service_trajectories(&[trajectory("same"), trajectory("same")], &[]),
            Err(WaterServiceTrajectoryError::InvalidInput(errors))
                if errors.contains(&WaterServiceTrajectoryValidationError::DuplicateTrajectoryId)
        ));
    }

    #[test]
    fn irregular_time_series_is_rejected() {
        let mut input = trajectory("irregular");
        input.steps[2].as_of = "2026-10-16T14:00:00Z".to_owned();
        assert!(matches!(
            assess_water_service_trajectory(&input, &[]),
            Err(WaterServiceTrajectoryError::InvalidInput(errors))
                if errors.contains(&WaterServiceTrajectoryValidationError::NonContiguousSteps)
        ));
    }

    #[test]
    fn component_identity_cannot_silently_change_mid_trajectory() {
        let mut input = trajectory("identity");
        input.steps[1].pump_capacity.entity_id = "other-pump".to_owned();
        assert!(matches!(
            assess_water_service_trajectory(&input, &[]),
            Err(WaterServiceTrajectoryError::InvalidInput(errors))
                if errors.contains(&WaterServiceTrajectoryValidationError::ComponentEntityChanged)
        ));
    }

    #[test]
    fn evidence_required_when_initial_inventory_is_observed() {
        let mut input = trajectory("observed-storage");
        input.initial_storage.basis = crate::dependency_logic::ScenarioBasis::Observed;
        assert!(matches!(
            assess_water_service_trajectory(&input, &[]),
            Err(WaterServiceTrajectoryError::StepAssessment(_))
        ));
    }

    fn source_observation(id: &str, at: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.to_owned(),
            schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:water-trajectory-test".to_owned(),
            subject_id: "source-a".to_owned(),
            spatial: SpatialContext {
                entity_id: "source-a".to_owned(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: at.to_owned(),
                recorded_at: at.to_owned(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic observed source flow for traceability test.".to_owned(),
            quantity: Some(Quantity {
                value: 100.0,
                unit: WATER_RATE_UNIT.to_owned(),
                uncertainty_abs: None,
            }),
            money: None,
            provenance: Provenance {
                method: "synthetic_test".to_owned(),
                source_artifacts: Vec::new(),
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:synthetic-test".to_owned(),
        }
    }

    #[test]
    fn evidence_is_attributed_to_the_exact_step_and_trajectory() {
        let mut input = trajectory("evidence-trace");
        input.steps[0].source_flow_rate.basis = crate::dependency_logic::ScenarioBasis::Observed;
        input.steps[0].source_flow_rate.supporting_evidence_ids =
            vec!["source-observation".to_owned()];
        let evidence = source_observation("source-observation", &input.steps[0].as_of);
        let result = assess_water_service_trajectory(&input, &[evidence]).unwrap();
        assert_eq!(
            result.steps[0].supporting_evidence_ids,
            vec!["source-observation".to_owned()]
        );
        assert!(result.steps[1].supporting_evidence_ids.is_empty());
        assert_eq!(
            result.supporting_evidence_ids,
            vec!["source-observation".to_owned()]
        );
    }

    #[test]
    fn storage_uncertainty_is_carried_forward_as_a_bounded_model_state() {
        let mut input = trajectory("bounded-storage");
        input.initial_storage.quantity.uncertainty_abs = Some(10.0);
        let result = assess_water_service_trajectory(&input, &[]).unwrap();
        assert_eq!(result.steps[0].ending_storage_lower_bound, 40.0);
        assert_eq!(result.steps[0].ending_storage_upper_bound, 60.0);
        assert_eq!(result.steps[1].customer_delivered_volume_lower_bound, 40.0);
        assert_eq!(result.steps[1].customer_delivered_volume_upper_bound, 60.0);
        assert!(result.unmet_demand_volume_lower_bound <= result.unmet_demand_volume_upper_bound);
        assert!(
            result.customer_delivered_volume_lower_bound
                <= result.customer_delivered_volume_upper_bound
        );
    }

    #[test]
    fn normalized_deficit_bounds_use_adversarial_demand_denominators() {
        let mut input = trajectory("deficit-bounds");
        input.initial_storage.quantity.value = 0.0;
        for (index, item) in input.steps.iter_mut().enumerate() {
            *item = step(&format!("shortfall-{index}"), 10 + index as u32, 50.0);
            item.demand_rate.quantity.value = 100.0;
            item.demand_rate.quantity.uncertainty_abs = Some(20.0);
        }
        let result = assess_water_service_trajectory(&input, &[]).unwrap();
        assert_eq!(result.unmet_demand_volume_lower_bound, 120.0);
        assert_eq!(result.unmet_demand_volume_upper_bound, 280.0);
        assert_eq!(result.maximum_normalized_deficit_lower_bound, 0.375);
        assert!(
            (result.maximum_normalized_deficit_upper_bound - (1.0 - 50.0 / 120.0)).abs() < 1e-12
        );
        assert!(
            result.maximum_normalized_deficit_lower_bound
                <= result.maximum_normalized_deficit_upper_bound
        );
    }

    #[test]
    fn results_do_not_claim_positive_recovery_when_not_observed() {
        let mut input = trajectory("still-down");
        for (index, item) in input.steps.iter_mut().enumerate() {
            *item = step(&format!("offline-{index}"), 10 + index as u32, 0.0);
        }
        let result = assess_water_service_trajectory(&input, &[]).unwrap();
        assert_eq!(result.first_adequate_step_id, None);
        assert!(!result.adequacy_maintained_through_last_sample);
    }

    #[test]
    fn duplicate_step_ids_and_non_integer_second_durations_fail_closed() {
        let mut input = trajectory("bad-duration");
        input.steps[1].id = input.steps[0].id.clone();
        assert!(matches!(
            assess_water_service_trajectory(&input, &[]),
            Err(WaterServiceTrajectoryError::InvalidInput(errors))
                if errors.contains(&WaterServiceTrajectoryValidationError::DuplicateStepId)
        ));
        let mut input = trajectory("fractional-second");
        input.steps[0].step_hours = 0.0001;
        assert!(matches!(
            assess_water_service_trajectory(&input, &[]),
            Err(WaterServiceTrajectoryError::InvalidInput(errors))
                if errors.contains(&WaterServiceTrajectoryValidationError::InvalidStepHours)
        ));
    }
}
