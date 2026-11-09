// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-aware coupling between pump electricity and aggregate water-service trajectories.
//!
//! This is a bounded operating-point estimate, not a hydraulic pump-curve model or
//! electrical network solver. Power must be scoped to pump terminals; upstream power
//! measurements require an explicitly modeled delivery/loss adapter before use here.

use crate::dependency_logic::ScenarioBasis;
use crate::service_function::{
    FunctionMeasurement, ServiceFunctionAssessmentError, ServiceFunctionValidationError,
    build_evidence_index, validate_measurement,
};
use crate::service_water::WATER_RATE_UNIT;
use crate::service_water_trajectory::{
    WaterServiceTrajectory, WaterServiceTrajectoryAssessment, WaterServiceTrajectoryError,
    assess_trajectory_indexed,
};
use crate::spatial_economic::{EvidenceRecord, Quantity};
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const WATER_ENERGY_COUPLING_SCHEMA_VERSION: &str = "0.1.0";
pub const ELECTRIC_POWER_UNIT: &str = "kW";
pub const PUMP_SPECIFIC_ENERGY_UNIT: &str = "kWh/m3";

/// Where the electrical quantity is scoped. Only pump-terminal power is admissible
/// in this first adapter; feeder, substation, and generator power need explicit loss
/// and delivery modeling and must not be silently treated as pump-available power.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerMeasurementLocation {
    PumpTerminals,
    Feeder,
    Substation,
    Generator,
}

/// Paired power and specific-energy inputs for exactly one water trajectory interval.
/// The specific-energy entity must equal operating_regime_id; its provenance then
/// describes the declared operating regime instead of implying a universal pump constant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyCouplingStepInput {
    pub step_id: String,
    pub as_of: String,
    pub operating_regime_id: String,
    pub power_measurement_location: PowerMeasurementLocation,
    pub available_power: FunctionMeasurement,
    pub specific_energy: FunctionMeasurement,
}

/// Coupling request wraps the existing water trajectory; it does not duplicate or
/// reinterpret its source, storage, treatment, distribution, loss, or demand semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyCouplingRequest {
    pub schema_version: String,
    pub id: String,
    pub trajectory: WaterServiceTrajectory,
    pub steps: Vec<WaterEnergyCouplingStepInput>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterEnergyMeasurementRole {
    AvailablePower,
    SpecificEnergy,
    MechanicalPumpCapacity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterEnergyCouplingValidationError {
    UnsupportedSchemaVersion,
    EmptyRequestId,
    EmptyTrajectorySteps,
    StepCountMismatch,
    EmptyStepId,
    DuplicateTrajectoryStepId,
    DuplicateCouplingStepId,
    UnknownTrajectoryStep,
    MissingCouplingStep,
    InvalidTimestamp,
    StepTimeMismatch,
    EmptyOperatingRegimeId,
    PowerNotAtPumpTerminals,
    PowerEntityMismatch,
    SpecificEnergyRegimeMismatch,
    InvalidMeasurement {
        step_id: String,
        role: WaterEnergyMeasurementRole,
        errors: Vec<ServiceFunctionValidationError>,
    },
    UnitMismatch {
        step_id: String,
        role: WaterEnergyMeasurementRole,
        expected: &'static str,
        actual: String,
    },
    SpecificEnergyLowerBoundNotPositive,
    NonFiniteDerivedValue,
    BoundsInconsistent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaterEnergyCouplingError {
    InvalidInput(Vec<WaterEnergyCouplingValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
    TrajectoryAssessment(WaterServiceTrajectoryError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyCouplingStepAssessment {
    pub step_id: String,
    pub as_of: String,
    pub operating_regime_id: String,
    pub power_measurement_location: PowerMeasurementLocation,
    pub power_unit: String,
    pub specific_energy_unit: String,
    pub water_flow_unit: String,
    pub available_power_lower_bound_kw: f64,
    pub available_power_upper_bound_kw: f64,
    pub specific_energy_lower_bound_kwh_per_m3: f64,
    pub specific_energy_upper_bound_kwh_per_m3: f64,
    pub mechanical_pump_capacity_lower_bound_m3_per_h: f64,
    pub mechanical_pump_capacity_upper_bound_m3_per_h: f64,
    pub energy_limited_flow_lower_bound_m3_per_h: f64,
    pub energy_limited_flow_upper_bound_m3_per_h: f64,
    pub effective_pump_capacity_lower_bound_m3_per_h: f64,
    pub effective_pump_capacity_upper_bound_m3_per_h: f64,
    /// Evidence supporting the mechanical capacity, terminal power and operating
    /// specific energy inputs. Derived pump capacity itself remains a model output.
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterEnergyInterpretationBoundary {
    BoundedOperatingPointNotHydraulicSimulation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyCoupledTrajectoryAssessment {
    pub schema_version: String,
    pub coupling_id: String,
    pub power_unit: String,
    pub specific_energy_unit: String,
    pub water_flow_unit: String,
    pub water_assessment: WaterServiceTrajectoryAssessment,
    pub steps: Vec<WaterEnergyCouplingStepAssessment>,
    pub supporting_evidence_ids: Vec<String>,
    pub interpretation_boundary: WaterEnergyInterpretationBoundary,
}

fn interval_bounds(measurement: &FunctionMeasurement) -> Option<(f64, f64)> {
    let uncertainty = measurement.quantity.uncertainty_abs.unwrap_or(0.0);
    let lower = (measurement.quantity.value - uncertainty).max(0.0);
    let upper = measurement.quantity.value + uncertainty;
    (lower.is_finite() && upper.is_finite() && lower <= upper).then_some((lower, upper))
}

fn validate_unit(
    step_id: &str,
    role: WaterEnergyMeasurementRole,
    actual: &str,
    expected: &'static str,
    errors: &mut Vec<WaterEnergyCouplingValidationError>,
) {
    if actual != expected {
        errors.push(WaterEnergyCouplingValidationError::UnitMismatch {
            step_id: step_id.to_owned(),
            role,
            expected,
            actual: actual.to_owned(),
        });
    }
}

fn validate_one_measurement(
    step_id: &str,
    role: WaterEnergyMeasurementRole,
    measurement: &FunctionMeasurement,
    as_of: &str,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
    errors: &mut Vec<WaterEnergyCouplingValidationError>,
) {
    let mut measurement_errors = Vec::new();
    validate_measurement(measurement, as_of, evidence_by_id, &mut measurement_errors);
    if !measurement_errors.is_empty() {
        errors.push(WaterEnergyCouplingValidationError::InvalidMeasurement {
            step_id: step_id.to_owned(),
            role,
            errors: measurement_errors,
        });
    }
}

/// Evaluate an electricity-constrained water-service trajectory.
///
/// The energy-limited flow interval is [power_low / energy_high,
/// power_high / energy_low]. The effective pump-capacity interval is the
/// interval-wise minimum of that estimate and the independently declared
/// mechanical/nameplate capacity. Power must be measured or scenario-scoped at
/// pump terminals. The derived capacity is labelled as an assumption/model output;
/// input evidence remains attached to this wrapper's per-step provenance.
pub fn assess_water_energy_coupled_trajectory(
    request: &WaterEnergyCouplingRequest,
    evidence_records: &[EvidenceRecord],
) -> Result<WaterEnergyCoupledTrajectoryAssessment, WaterEnergyCouplingError> {
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(WaterEnergyCouplingError::EvidenceValidation)?;
    assess_water_energy_coupled_trajectory_indexed(request, &evidence_by_id)
}

pub(crate) fn assess_water_energy_coupled_trajectory_indexed(
    request: &WaterEnergyCouplingRequest,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<WaterEnergyCoupledTrajectoryAssessment, WaterEnergyCouplingError> {
    let mut errors = Vec::new();

    if request.schema_version != WATER_ENERGY_COUPLING_SCHEMA_VERSION {
        errors.push(WaterEnergyCouplingValidationError::UnsupportedSchemaVersion);
    }
    if request.id.trim().is_empty() {
        errors.push(WaterEnergyCouplingValidationError::EmptyRequestId);
    }
    if request.trajectory.steps.is_empty() {
        errors.push(WaterEnergyCouplingValidationError::EmptyTrajectorySteps);
    }
    if request.steps.len() != request.trajectory.steps.len() {
        errors.push(WaterEnergyCouplingValidationError::StepCountMismatch);
    }

    let mut trajectory_ids = BTreeSet::new();
    for step in &request.trajectory.steps {
        if step.id.trim().is_empty() {
            errors.push(WaterEnergyCouplingValidationError::EmptyStepId);
        }
        if !trajectory_ids.insert(step.id.as_str()) {
            errors.push(WaterEnergyCouplingValidationError::DuplicateTrajectoryStepId);
        }
    }

    let mut coupling_by_id: BTreeMap<&str, &WaterEnergyCouplingStepInput> = BTreeMap::new();
    for step in &request.steps {
        if step.step_id.trim().is_empty() {
            errors.push(WaterEnergyCouplingValidationError::EmptyStepId);
        }
        if coupling_by_id.insert(step.step_id.as_str(), step).is_some() {
            errors.push(WaterEnergyCouplingValidationError::DuplicateCouplingStepId);
        }
        if !trajectory_ids.contains(step.step_id.as_str()) {
            errors.push(WaterEnergyCouplingValidationError::UnknownTrajectoryStep);
        }
        if !is_canonical_utc_timestamp(&step.as_of) {
            errors.push(WaterEnergyCouplingValidationError::InvalidTimestamp);
        }
    }

    for water_step in &request.trajectory.steps {
        let Some(input) = coupling_by_id.get(water_step.id.as_str()).copied() else {
            errors.push(WaterEnergyCouplingValidationError::MissingCouplingStep);
            continue;
        };

        if input.as_of != water_step.as_of {
            errors.push(WaterEnergyCouplingValidationError::StepTimeMismatch);
        }
        if input.operating_regime_id.trim().is_empty() {
            errors.push(WaterEnergyCouplingValidationError::EmptyOperatingRegimeId);
        }
        if input.power_measurement_location != PowerMeasurementLocation::PumpTerminals {
            errors.push(WaterEnergyCouplingValidationError::PowerNotAtPumpTerminals);
        }
        if input.available_power.entity_id != water_step.pump_capacity.entity_id {
            errors.push(WaterEnergyCouplingValidationError::PowerEntityMismatch);
        }
        if input.specific_energy.entity_id != input.operating_regime_id {
            errors.push(WaterEnergyCouplingValidationError::SpecificEnergyRegimeMismatch);
        }

        validate_unit(
            &water_step.id,
            WaterEnergyMeasurementRole::MechanicalPumpCapacity,
            &water_step.pump_capacity.quantity.unit,
            WATER_RATE_UNIT,
            &mut errors,
        );
        validate_unit(
            &water_step.id,
            WaterEnergyMeasurementRole::AvailablePower,
            &input.available_power.quantity.unit,
            ELECTRIC_POWER_UNIT,
            &mut errors,
        );
        validate_unit(
            &water_step.id,
            WaterEnergyMeasurementRole::SpecificEnergy,
            &input.specific_energy.quantity.unit,
            PUMP_SPECIFIC_ENERGY_UNIT,
            &mut errors,
        );

        validate_one_measurement(
            &water_step.id,
            WaterEnergyMeasurementRole::MechanicalPumpCapacity,
            &water_step.pump_capacity,
            &water_step.as_of,
            evidence_by_id,
            &mut errors,
        );
        validate_one_measurement(
            &water_step.id,
            WaterEnergyMeasurementRole::AvailablePower,
            &input.available_power,
            &water_step.as_of,
            evidence_by_id,
            &mut errors,
        );
        validate_one_measurement(
            &water_step.id,
            WaterEnergyMeasurementRole::SpecificEnergy,
            &input.specific_energy,
            &water_step.as_of,
            evidence_by_id,
            &mut errors,
        );

        for measurement in [
            &water_step.pump_capacity,
            &input.available_power,
            &input.specific_energy,
        ] {
            if interval_bounds(measurement).is_none() {
                errors.push(WaterEnergyCouplingValidationError::NonFiniteDerivedValue);
            }
        }
        if interval_bounds(&input.specific_energy).is_some_and(|(lower, _)| lower <= 0.0) {
            errors.push(WaterEnergyCouplingValidationError::SpecificEnergyLowerBoundNotPositive);
        }
    }

    if !errors.is_empty() {
        return Err(WaterEnergyCouplingError::InvalidInput(errors));
    }

    let mut coupled_trajectory = request.trajectory.clone();
    let mut assessments = Vec::with_capacity(coupled_trajectory.steps.len());
    let mut all_evidence_ids = BTreeSet::new();

    for water_step in &mut coupled_trajectory.steps {
        // Structural validation above guarantees one matching, validated input.
        let input = coupling_by_id
            .get(water_step.id.as_str())
            .copied()
            .expect("validated coupling step exists");
        let (power_low, power_high) =
            interval_bounds(&input.available_power).expect("validated finite power interval");
        let (energy_low, energy_high) =
            interval_bounds(&input.specific_energy).expect("validated finite energy interval");
        let (mechanical_low, mechanical_high) = interval_bounds(&water_step.pump_capacity)
            .expect("validated finite mechanical capacity interval");

        if energy_low <= 0.0 {
            return Err(WaterEnergyCouplingError::InvalidInput(vec![
                WaterEnergyCouplingValidationError::SpecificEnergyLowerBoundNotPositive,
            ]));
        }

        let energy_flow_low = power_low / energy_high;
        let energy_flow_high = power_high / energy_low;
        let effective_low = mechanical_low.min(energy_flow_low);
        let effective_high = mechanical_high.min(energy_flow_high);
        if [
            energy_flow_low,
            energy_flow_high,
            effective_low,
            effective_high,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
        {
            return Err(WaterEnergyCouplingError::InvalidInput(vec![
                WaterEnergyCouplingValidationError::NonFiniteDerivedValue,
            ]));
        }
        if effective_low < 0.0 || effective_high < effective_low {
            return Err(WaterEnergyCouplingError::InvalidInput(vec![
                WaterEnergyCouplingValidationError::BoundsInconsistent,
            ]));
        }

        let midpoint = effective_low + (effective_high - effective_low) / 2.0;
        let uncertainty = (effective_high - effective_low) / 2.0;
        let mut step_evidence_ids = BTreeSet::new();
        step_evidence_ids.extend(
            water_step
                .pump_capacity
                .supporting_evidence_ids
                .iter()
                .cloned(),
        );
        step_evidence_ids.extend(
            input
                .available_power
                .supporting_evidence_ids
                .iter()
                .cloned(),
        );
        step_evidence_ids.extend(
            input
                .specific_energy
                .supporting_evidence_ids
                .iter()
                .cloned(),
        );
        all_evidence_ids.extend(step_evidence_ids.iter().cloned());

        assessments.push(WaterEnergyCouplingStepAssessment {
            step_id: water_step.id.clone(),
            as_of: water_step.as_of.clone(),
            operating_regime_id: input.operating_regime_id.clone(),
            power_measurement_location: input.power_measurement_location,
            power_unit: ELECTRIC_POWER_UNIT.to_owned(),
            specific_energy_unit: PUMP_SPECIFIC_ENERGY_UNIT.to_owned(),
            water_flow_unit: WATER_RATE_UNIT.to_owned(),
            available_power_lower_bound_kw: power_low,
            available_power_upper_bound_kw: power_high,
            specific_energy_lower_bound_kwh_per_m3: energy_low,
            specific_energy_upper_bound_kwh_per_m3: energy_high,
            mechanical_pump_capacity_lower_bound_m3_per_h: mechanical_low,
            mechanical_pump_capacity_upper_bound_m3_per_h: mechanical_high,
            energy_limited_flow_lower_bound_m3_per_h: energy_flow_low,
            energy_limited_flow_upper_bound_m3_per_h: energy_flow_high,
            effective_pump_capacity_lower_bound_m3_per_h: effective_low,
            effective_pump_capacity_upper_bound_m3_per_h: effective_high,
            supporting_evidence_ids: step_evidence_ids.into_iter().collect(),
        });

        // ScenarioBasis::Assumed is intentional: this quantity is derived, not an
        // observation. Provenance is retained in the step assessment above.
        water_step.pump_capacity = FunctionMeasurement {
            entity_id: water_step.pump_capacity.entity_id.clone(),
            quantity: Quantity {
                value: midpoint,
                unit: WATER_RATE_UNIT.to_owned(),
                uncertainty_abs: Some(uncertainty),
            },
            basis: ScenarioBasis::Assumed,
            rationale: concat!(
                "Derived capacity from pump-terminal power and regime specific energy; ",
                "not measured flow or a hydraulic pump curve.",
            )
            .to_owned(),
            supporting_evidence_ids: Vec::new(),
        };
    }

    let water_assessment = assess_trajectory_indexed(&coupled_trajectory, evidence_by_id)
        .map_err(WaterEnergyCouplingError::TrajectoryAssessment)?;
    all_evidence_ids.extend(water_assessment.supporting_evidence_ids.iter().cloned());

    Ok(WaterEnergyCoupledTrajectoryAssessment {
        schema_version: WATER_ENERGY_COUPLING_SCHEMA_VERSION.to_owned(),
        coupling_id: request.id.clone(),
        power_unit: ELECTRIC_POWER_UNIT.to_owned(),
        specific_energy_unit: PUMP_SPECIFIC_ENERGY_UNIT.to_owned(),
        water_flow_unit: WATER_RATE_UNIT.to_owned(),
        water_assessment,
        steps: assessments,
        supporting_evidence_ids: all_evidence_ids.into_iter().collect(),
        interpretation_boundary:
            WaterEnergyInterpretationBoundary::BoundedOperatingPointNotHydraulicSimulation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_water::{DIMENSIONLESS_UNIT, WATER_VOLUME_UNIT};
    use crate::service_water_trajectory::{
        WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION, WaterServiceTrajectoryStep,
    };

    const FIRST: &str = "2026-10-16T09:00:00Z";
    const SECOND: &str = "2026-10-16T10:00:00Z";

    fn measurement(
        entity_id: &str,
        value: f64,
        unit: &str,
        uncertainty_abs: Option<f64>,
    ) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity_id.to_owned(),
            quantity: Quantity {
                value,
                unit: unit.to_owned(),
                uncertainty_abs,
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Explicit synthetic test assumption.".to_owned(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn request(power_value: f64, energy_value: f64) -> WaterEnergyCouplingRequest {
        let trajectory = WaterServiceTrajectory {
            schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
            id: "water-trajectory".to_owned(),
            service_entity_id: "community".to_owned(),
            initial_storage: measurement("tank", 0.0, WATER_VOLUME_UNIT, None),
            storage_capacity: measurement("tank", 100.0, WATER_VOLUME_UNIT, None),
            steps: [FIRST, SECOND]
                .into_iter()
                .enumerate()
                .map(|(index, as_of)| WaterServiceTrajectoryStep {
                    id: format!("step-{index}"),
                    as_of: as_of.to_owned(),
                    step_hours: 1.0,
                    source_flow_rate: measurement("source", 100.0, WATER_RATE_UNIT, None),
                    pump_capacity: measurement("pump", 100.0, WATER_RATE_UNIT, None),
                    treatment_capacity: measurement("treatment", 100.0, WATER_RATE_UNIT, None),
                    distribution_capacity: measurement(
                        "distribution",
                        120.0,
                        WATER_RATE_UNIT,
                        None,
                    ),
                    demand_rate: measurement("community", 100.0, WATER_RATE_UNIT, None),
                    distribution_loss_fraction: measurement(
                        "distribution",
                        0.0,
                        DIMENSIONLESS_UNIT,
                        None,
                    ),
                })
                .collect(),
        };
        let steps = [FIRST, SECOND]
            .into_iter()
            .enumerate()
            .map(|(index, as_of)| WaterEnergyCouplingStepInput {
                step_id: format!("step-{index}"),
                as_of: as_of.to_owned(),
                operating_regime_id: "pump-regime".to_owned(),
                power_measurement_location: PowerMeasurementLocation::PumpTerminals,
                available_power: measurement("pump", power_value, ELECTRIC_POWER_UNIT, None),
                specific_energy: measurement(
                    "pump-regime",
                    energy_value,
                    PUMP_SPECIFIC_ENERGY_UNIT,
                    None,
                ),
            })
            .collect();
        WaterEnergyCouplingRequest {
            schema_version: WATER_ENERGY_COUPLING_SCHEMA_VERSION.to_owned(),
            id: "coupling-scenario".to_owned(),
            trajectory,
            steps,
        }
    }

    #[test]
    fn intersects_energy_bound_with_mechanical_capacity() {
        let output =
            assess_water_energy_coupled_trajectory(&request(40.0, 0.5), &[]).expect("valid model");
        let first = &output.steps[0];
        assert!((first.energy_limited_flow_lower_bound_m3_per_h - 80.0).abs() < 1e-9);
        assert!((first.energy_limited_flow_upper_bound_m3_per_h - 80.0).abs() < 1e-9);
        assert!((first.effective_pump_capacity_lower_bound_m3_per_h - 80.0).abs() < 1e-9);
        assert!((first.effective_pump_capacity_upper_bound_m3_per_h - 80.0).abs() < 1e-9);
        assert_eq!(output.water_assessment.step_count, 2);
        assert_eq!(
            output.interpretation_boundary,
            WaterEnergyInterpretationBoundary::BoundedOperatingPointNotHydraulicSimulation
        );
    }

    #[test]
    fn propagates_uncertainty_conservatively_before_intersecting_capacity() {
        let mut input = request(40.0, 0.5);
        for step in &mut input.steps {
            step.available_power.quantity.uncertainty_abs = Some(4.0);
            step.specific_energy.quantity.uncertainty_abs = Some(0.1);
        }
        for step in &mut input.trajectory.steps {
            step.pump_capacity.quantity.value = 90.0;
            step.pump_capacity.quantity.uncertainty_abs = Some(10.0);
        }
        let output = assess_water_energy_coupled_trajectory(&input, &[]).expect("valid model");
        let first = &output.steps[0];
        assert!((first.energy_limited_flow_lower_bound_m3_per_h - 60.0).abs() < 1e-9);
        assert!((first.energy_limited_flow_upper_bound_m3_per_h - 110.0).abs() < 1e-9);
        assert!((first.effective_pump_capacity_lower_bound_m3_per_h - 60.0).abs() < 1e-9);
        assert!((first.effective_pump_capacity_upper_bound_m3_per_h - 100.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_non_positive_specific_energy_lower_bound() {
        let mut input = request(40.0, 0.1);
        for step in &mut input.steps {
            step.specific_energy.quantity.uncertainty_abs = Some(0.1);
        }
        let result = assess_water_energy_coupled_trajectory(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyCouplingError::InvalidInput(errors))
                if errors.contains(
                    &WaterEnergyCouplingValidationError::SpecificEnergyLowerBoundNotPositive,
                )
        ));
    }

    #[test]
    fn rejects_power_measured_upstream_of_pump_terminals() {
        let mut input = request(40.0, 0.5);
        input.steps[0].power_measurement_location = PowerMeasurementLocation::Feeder;
        let result = assess_water_energy_coupled_trajectory(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyCouplingError::InvalidInput(errors))
                if errors.contains(&WaterEnergyCouplingValidationError::PowerNotAtPumpTerminals)
        ));
    }

    #[test]
    fn rejects_mismatched_units_without_guessing_conversion() {
        let mut input = request(40.0, 0.5);
        input.steps[0].available_power.quantity.unit = "W".to_owned();
        let result = assess_water_energy_coupled_trajectory(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyCouplingError::InvalidInput(errors))
                if errors.iter().any(|error| matches!(
                    error,
                    WaterEnergyCouplingValidationError::UnitMismatch {
                        role: WaterEnergyMeasurementRole::AvailablePower,
                        ..
                    }
                ))
        ));
    }

    #[test]
    fn rejects_time_grid_mismatch() {
        let mut input = request(40.0, 0.5);
        input.steps[0].as_of = SECOND.to_owned();
        let result = assess_water_energy_coupled_trajectory(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyCouplingError::InvalidInput(errors))
                if errors.contains(&WaterEnergyCouplingValidationError::StepTimeMismatch)
        ));
    }

    #[test]
    fn increasing_power_does_not_reduce_effective_flow_bounds() {
        let low = assess_water_energy_coupled_trajectory(&request(20.0, 0.5), &[])
            .expect("low-power model");
        let high = assess_water_energy_coupled_trajectory(&request(40.0, 0.5), &[])
            .expect("high-power model");
        assert!(
            low.steps[0].effective_pump_capacity_lower_bound_m3_per_h
                <= high.steps[0].effective_pump_capacity_lower_bound_m3_per_h
        );
        assert!(
            low.steps[0].effective_pump_capacity_upper_bound_m3_per_h
                <= high.steps[0].effective_pump_capacity_upper_bound_m3_per_h
        );
    }

    #[test]
    fn increasing_specific_energy_does_not_increase_energy_limited_flow() {
        let low = assess_water_energy_coupled_trajectory(&request(40.0, 0.5), &[])
            .expect("low-energy model");
        let high = assess_water_energy_coupled_trajectory(&request(40.0, 1.0), &[])
            .expect("high-energy model");
        assert!(
            low.steps[0].energy_limited_flow_lower_bound_m3_per_h
                >= high.steps[0].energy_limited_flow_lower_bound_m3_per_h
        );
        assert!(
            low.steps[0].energy_limited_flow_upper_bound_m3_per_h
                >= high.steps[0].energy_limited_flow_upper_bound_m3_per_h
        );
    }

    #[test]
    fn retains_model_boundary_even_for_synthetic_scenarios() {
        let output =
            assess_water_energy_coupled_trajectory(&request(40.0, 0.5), &[]).expect("valid model");
        assert!(output.supporting_evidence_ids.is_empty());
        assert!(
            output
                .steps
                .iter()
                .all(|step| step.supporting_evidence_ids.is_empty())
        );
    }
}
