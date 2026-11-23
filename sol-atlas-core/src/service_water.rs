// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-aware, bounded water-service balance accounting.
//!
//! This is a serial-stage volume-balance model, not a hydraulic solver. Source,
//! pump, and treatment rates compose through their minimum (never by summation).
//! The model accounts for a storage buffer, a distribution-rate bottleneck,
//! customer-side losses, and explicit spill. It does not calculate pressure,
//! pipe friction, water quality, contamination, or within-step transient behavior.

use crate::service_function::{
    FunctionMeasurement, ServiceFunctionAssessmentError, ServiceFunctionValidationError,
    build_evidence_index, validate_measurement,
};
use crate::spatial_economic::EvidenceRecord;
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const WATER_SERVICE_SCHEMA_VERSION: &str = "0.1.0";
pub const WATER_RATE_UNIT: &str = "m3/h";
pub const WATER_VOLUME_UNIT: &str = "m3";
pub const DIMENSIONLESS_UNIT: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterMeasurementRole {
    SourceFlowRate,
    PumpCapacity,
    TreatmentCapacity,
    DistributionCapacity,
    InitialStorage,
    StorageCapacity,
    DemandRate,
    DistributionLossFraction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterServiceState {
    Adequate,
    Degraded,
    Unavailable,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceScenario {
    pub schema_version: String,
    pub id: String,
    /// Canonical UTC timestamp: YYYY-MM-DDTHH:MM:SSZ.
    pub as_of: String,
    pub service_entity_id: String,
    /// Duration represented by the single aggregate balance step, in hours.
    pub step_hours: f64,
    /// Upstream source rate, pump capacity, and treatment capacity are serial stages.
    pub source_flow_rate: FunctionMeasurement,
    pub pump_capacity: FunctionMeasurement,
    pub treatment_capacity: FunctionMeasurement,
    /// Customer-side delivery is bounded by this rate after treatment and storage.
    pub distribution_capacity: FunctionMeasurement,
    /// Usable treated-water inventory downstream of treatment and upstream of distribution.
    /// Both storage measurements must identify the same tank/reservoir entity.
    pub initial_storage: FunctionMeasurement,
    pub storage_capacity: FunctionMeasurement,
    /// Required customer-delivered service rate, before no hidden unit conversion.
    pub demand_rate: FunctionMeasurement,
    /// Absolute uncertainty is a bounded interval on the dimensionless fraction [0, 1).
    pub distribution_loss_fraction: FunctionMeasurement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceBalancePath {
    /// Endpoint values for one fully specified conservative or optimistic realization.
    pub demand_volume: f64,
    pub treated_volume: f64,
    pub available_water_volume: f64,
    /// Volume leaving the storage/source balance toward the distribution system.
    pub raw_distribution_withdrawal: f64,
    /// Customer-side delivery after the selected loss fraction.
    pub customer_delivered_volume: f64,
    /// Water lost between distribution withdrawal and customer delivery.
    pub distribution_loss_volume: f64,
    pub ending_storage_volume: f64,
    pub spilled_volume: f64,
    /// Available water minus withdrawal, ending storage, and spill. Must be ~0.
    pub mass_balance_residual: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceAssessment {
    pub schema_version: String,
    pub scenario_id: String,
    pub service_entity_id: String,
    pub as_of: String,
    pub rate_unit: String,
    pub volume_unit: String,
    pub time_unit: String,
    pub state: WaterServiceState,
    pub step_hours: f64,
    pub demand_volume_lower_bound: f64,
    pub demand_volume_upper_bound: f64,
    pub treated_volume_lower_bound: f64,
    pub treated_volume_upper_bound: f64,
    /// Customer-delivery capacity from available water, serial stage caps, and loss.
    /// This is capacity for the interval, not a claim that this volume was delivered.
    pub customer_delivery_capacity_lower_bound: f64,
    pub customer_delivery_capacity_upper_bound: f64,
    /// Capacity / demand; absent if the corresponding demand bound is zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_lower_bound: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_upper_bound: Option<f64>,
    /// Each path is internally mass-balanced. Their demand values differ by design.
    pub conservative_path: WaterServiceBalancePath,
    pub optimistic_path: WaterServiceBalancePath,
    /// Sorted, deduplicated references from all eight source measurements.
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterServiceValidationError {
    UnsupportedSchemaVersion,
    EmptyScenarioSet,
    DuplicateScenarioId,
    EmptyScenarioId,
    InvalidAsOfTimestamp,
    EmptyServiceEntityId,
    InvalidStepHours,
    InvalidMeasurement {
        role: WaterMeasurementRole,
        errors: Vec<ServiceFunctionValidationError>,
    },
    UnitMismatch {
        role: WaterMeasurementRole,
        expected: &'static str,
        actual: String,
    },
    DuplicatePhysicalEntityId,
    StorageEntityMismatch,
    DemandEntityMismatch,
    LossMeasurementEntityMismatch,
    InvalidLossFractionInterval,
    StorageCapacityIntervalInconsistent,
    NonFiniteDerivedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterServiceError {
    InvalidInput(Vec<WaterServiceValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
}

#[derive(Debug, Clone, Copy)]
struct Interval {
    lower: f64,
    upper: f64,
}

#[derive(Debug, Clone, Copy)]
struct PathInputs {
    initial_storage: f64,
    storage_capacity: f64,
    treated_volume: f64,
    distribution_rate: f64,
    demand_volume: f64,
    loss_fraction: f64,
}

fn interval(measurement: &FunctionMeasurement) -> Interval {
    let uncertainty = measurement.quantity.uncertainty_abs.unwrap_or(0.0);
    Interval {
        lower: (measurement.quantity.value - uncertainty).max(0.0),
        upper: measurement.quantity.value + uncertainty,
    }
}

/// Assess one service against an evidence corpus. Observed measurements must cite
/// admissible, matching evidence through the shared service-function contract.
pub fn assess_water_service(
    scenario: &WaterServiceScenario,
    evidence_records: &[EvidenceRecord],
) -> Result<WaterServiceAssessment, WaterServiceError> {
    let evidence_by_id =
        build_evidence_index(evidence_records).map_err(WaterServiceError::EvidenceValidation)?;
    assess_water_service_indexed(scenario, &evidence_by_id)
}

/// Batch evaluator indexes and validates the evidence corpus once, rejects duplicate
/// scenario IDs, and returns results sorted by stable scenario ID.
pub fn assess_water_services(
    scenarios: &[WaterServiceScenario],
    evidence_records: &[EvidenceRecord],
) -> Result<Vec<WaterServiceAssessment>, WaterServiceError> {
    if scenarios.is_empty() {
        return Err(WaterServiceError::InvalidInput(vec![
            WaterServiceValidationError::EmptyScenarioSet,
        ]));
    }
    let evidence_by_id =
        build_evidence_index(evidence_records).map_err(WaterServiceError::EvidenceValidation)?;
    let mut seen = BTreeSet::new();
    for scenario in scenarios {
        if !seen.insert(scenario.id.as_str()) {
            return Err(WaterServiceError::InvalidInput(vec![
                WaterServiceValidationError::DuplicateScenarioId,
            ]));
        }
    }
    let mut ordered: Vec<&WaterServiceScenario> = scenarios.iter().collect();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    ordered
        .into_iter()
        .map(|scenario| assess_water_service_indexed(scenario, &evidence_by_id))
        .collect()
}

pub(crate) fn assess_water_service_indexed(
    scenario: &WaterServiceScenario,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<WaterServiceAssessment, WaterServiceError> {
    let mut errors = Vec::new();
    if scenario.schema_version != WATER_SERVICE_SCHEMA_VERSION {
        errors.push(WaterServiceValidationError::UnsupportedSchemaVersion);
    }
    if scenario.id.trim().is_empty() {
        errors.push(WaterServiceValidationError::EmptyScenarioId);
    }
    if !is_canonical_utc_timestamp(&scenario.as_of) {
        errors.push(WaterServiceValidationError::InvalidAsOfTimestamp);
    }
    if scenario.service_entity_id.trim().is_empty() {
        errors.push(WaterServiceValidationError::EmptyServiceEntityId);
    }
    if !scenario.step_hours.is_finite() || scenario.step_hours <= 0.0 {
        errors.push(WaterServiceValidationError::InvalidStepHours);
    }

    let measurements = [
        (
            WaterMeasurementRole::SourceFlowRate,
            &scenario.source_flow_rate,
            WATER_RATE_UNIT,
        ),
        (
            WaterMeasurementRole::PumpCapacity,
            &scenario.pump_capacity,
            WATER_RATE_UNIT,
        ),
        (
            WaterMeasurementRole::TreatmentCapacity,
            &scenario.treatment_capacity,
            WATER_RATE_UNIT,
        ),
        (
            WaterMeasurementRole::DistributionCapacity,
            &scenario.distribution_capacity,
            WATER_RATE_UNIT,
        ),
        (
            WaterMeasurementRole::InitialStorage,
            &scenario.initial_storage,
            WATER_VOLUME_UNIT,
        ),
        (
            WaterMeasurementRole::StorageCapacity,
            &scenario.storage_capacity,
            WATER_VOLUME_UNIT,
        ),
        (
            WaterMeasurementRole::DemandRate,
            &scenario.demand_rate,
            WATER_RATE_UNIT,
        ),
        (
            WaterMeasurementRole::DistributionLossFraction,
            &scenario.distribution_loss_fraction,
            DIMENSIONLESS_UNIT,
        ),
    ];
    for (role, measurement, expected_unit) in measurements {
        let mut measurement_errors = Vec::new();
        validate_measurement(
            measurement,
            &scenario.as_of,
            evidence_by_id,
            &mut measurement_errors,
        );
        if !measurement_errors.is_empty() {
            errors.push(WaterServiceValidationError::InvalidMeasurement {
                role,
                errors: measurement_errors,
            });
        }
        if measurement.quantity.unit != expected_unit {
            errors.push(WaterServiceValidationError::UnitMismatch {
                role,
                expected: expected_unit,
                actual: measurement.quantity.unit.clone(),
            });
        }
        let bounds = interval(measurement);
        if !bounds.lower.is_finite() || !bounds.upper.is_finite() {
            errors.push(WaterServiceValidationError::NonFiniteDerivedValue);
        }
    }

    if scenario.demand_rate.entity_id != scenario.service_entity_id {
        errors.push(WaterServiceValidationError::DemandEntityMismatch);
    }
    if scenario.initial_storage.entity_id != scenario.storage_capacity.entity_id {
        errors.push(WaterServiceValidationError::StorageEntityMismatch);
    }
    if scenario.distribution_loss_fraction.entity_id != scenario.distribution_capacity.entity_id {
        errors.push(WaterServiceValidationError::LossMeasurementEntityMismatch);
    }

    let physical_entity_ids = [
        scenario.source_flow_rate.entity_id.as_str(),
        scenario.pump_capacity.entity_id.as_str(),
        scenario.treatment_capacity.entity_id.as_str(),
        scenario.distribution_capacity.entity_id.as_str(),
        scenario.initial_storage.entity_id.as_str(),
    ];
    let unique_physical_ids: BTreeSet<&str> = physical_entity_ids.into_iter().collect();
    if unique_physical_ids.len() != physical_entity_ids.len() {
        errors.push(WaterServiceValidationError::DuplicatePhysicalEntityId);
    }
    if physical_entity_ids.contains(&scenario.service_entity_id.as_str()) {
        errors.push(WaterServiceValidationError::DuplicatePhysicalEntityId);
    }

    let loss_uncertainty = scenario
        .distribution_loss_fraction
        .quantity
        .uncertainty_abs
        .unwrap_or(0.0);
    let loss = Interval {
        lower: scenario.distribution_loss_fraction.quantity.value - loss_uncertainty,
        upper: scenario.distribution_loss_fraction.quantity.value + loss_uncertainty,
    };
    if loss.lower < 0.0 || loss.upper >= 1.0 {
        errors.push(WaterServiceValidationError::InvalidLossFractionInterval);
    }

    let initial = interval(&scenario.initial_storage);
    let tank_capacity = interval(&scenario.storage_capacity);
    if initial.upper > tank_capacity.lower {
        errors.push(WaterServiceValidationError::StorageCapacityIntervalInconsistent);
    }

    // Check all rate-to-volume products before calculations. This also blocks
    // overflow from apparently finite, but physically unusable, extreme inputs.
    if scenario.step_hours.is_finite() && scenario.step_hours > 0.0 {
        for rate in [
            interval(&scenario.source_flow_rate).upper,
            interval(&scenario.pump_capacity).upper,
            interval(&scenario.treatment_capacity).upper,
            interval(&scenario.distribution_capacity).upper,
            interval(&scenario.demand_rate).upper,
        ] {
            if !(rate * scenario.step_hours).is_finite() {
                errors.push(WaterServiceValidationError::NonFiniteDerivedValue);
            }
        }
        let demand_upper = interval(&scenario.demand_rate).upper * scenario.step_hours;
        if loss.upper < 1.0 && !(demand_upper / (1.0 - loss.upper)).is_finite() {
            errors.push(WaterServiceValidationError::NonFiniteDerivedValue);
        }
        let max_production = [
            interval(&scenario.source_flow_rate).upper,
            interval(&scenario.pump_capacity).upper,
            interval(&scenario.treatment_capacity).upper,
        ]
        .into_iter()
        .fold(f64::INFINITY, f64::min)
            * scenario.step_hours;
        if !(initial.upper + max_production).is_finite() {
            errors.push(WaterServiceValidationError::NonFiniteDerivedValue);
        }
    }

    if !errors.is_empty() {
        return Err(WaterServiceError::InvalidInput(errors));
    }

    let source = interval(&scenario.source_flow_rate);
    let pump = interval(&scenario.pump_capacity);
    let treatment = interval(&scenario.treatment_capacity);
    let distribution = interval(&scenario.distribution_capacity);
    let demand = interval(&scenario.demand_rate);
    let step = scenario.step_hours;

    // Serial stages compose by minimum, never by addition: a 65 m3/h treatment
    // plant cannot produce 80 m3/h just because upstream source/pump exceed it.
    let treatment_rate_lower = source.lower.min(pump.lower).min(treatment.lower);
    let treatment_rate_upper = source.upper.min(pump.upper).min(treatment.upper);
    let treated_lower = treatment_rate_lower * step;
    let treated_upper = treatment_rate_upper * step;
    let available_lower = initial.lower + treated_lower;
    let available_upper = initial.upper + treated_upper;
    let distribution_volume_lower = distribution.lower * step;
    let distribution_volume_upper = distribution.upper * step;

    let delivery_capacity_lower =
        available_lower.min(distribution_volume_lower) * (1.0 - loss.upper);
    let delivery_capacity_upper =
        available_upper.min(distribution_volume_upper) * (1.0 - loss.lower);
    let demand_volume_lower = demand.lower * step;
    let demand_volume_upper = demand.upper * step;

    let coverage_lower =
        (demand_volume_upper > 0.0).then_some(delivery_capacity_lower / demand_volume_upper);
    let coverage_upper =
        (demand_volume_lower > 0.0).then_some(delivery_capacity_upper / demand_volume_lower);

    if [
        treated_lower,
        treated_upper,
        available_lower,
        available_upper,
        delivery_capacity_lower,
        delivery_capacity_upper,
        demand_volume_lower,
        demand_volume_upper,
    ]
    .into_iter()
    .any(|value| !value.is_finite())
        || coverage_lower.is_some_and(|value| !value.is_finite())
        || coverage_upper.is_some_and(|value| !value.is_finite())
    {
        return Err(WaterServiceError::InvalidInput(vec![
            WaterServiceValidationError::NonFiniteDerivedValue,
        ]));
    }

    let tolerance = f64::EPSILON * 64.0 * demand_volume_upper.max(1.0);
    let state = if demand_volume_upper == 0.0 {
        // A zero denominator does not establish a positive service claim.
        WaterServiceState::Indeterminate
    } else if delivery_capacity_lower + tolerance >= demand_volume_upper {
        WaterServiceState::Adequate
    } else if delivery_capacity_upper == 0.0 && demand_volume_lower > 0.0 {
        WaterServiceState::Unavailable
    } else if delivery_capacity_upper + tolerance < demand_volume_lower
        && delivery_capacity_lower > 0.0
    {
        WaterServiceState::Degraded
    } else {
        WaterServiceState::Indeterminate
    };

    let conservative_path = simulate_path(
        PathInputs {
            initial_storage: initial.lower,
            storage_capacity: tank_capacity.lower,
            treated_volume: treated_lower,
            distribution_rate: distribution.lower,
            demand_volume: demand_volume_upper,
            loss_fraction: loss.upper,
        },
        step,
    );
    let optimistic_path = simulate_path(
        PathInputs {
            initial_storage: initial.upper,
            storage_capacity: tank_capacity.upper,
            treated_volume: treated_upper,
            distribution_rate: distribution.upper,
            demand_volume: demand_volume_lower,
            loss_fraction: loss.lower,
        },
        step,
    );

    let mut evidence_ids = BTreeSet::new();
    for (_, measurement, _) in measurements {
        evidence_ids.extend(measurement.supporting_evidence_ids.iter().cloned());
    }

    Ok(WaterServiceAssessment {
        schema_version: WATER_SERVICE_SCHEMA_VERSION.into(),
        scenario_id: scenario.id.clone(),
        service_entity_id: scenario.service_entity_id.clone(),
        as_of: scenario.as_of.clone(),
        rate_unit: WATER_RATE_UNIT.into(),
        volume_unit: WATER_VOLUME_UNIT.into(),
        time_unit: "h".into(),
        state,
        step_hours: step,
        demand_volume_lower_bound: demand_volume_lower,
        demand_volume_upper_bound: demand_volume_upper,
        treated_volume_lower_bound: treated_lower,
        treated_volume_upper_bound: treated_upper,
        customer_delivery_capacity_lower_bound: delivery_capacity_lower,
        customer_delivery_capacity_upper_bound: delivery_capacity_upper,
        coverage_lower_bound: coverage_lower,
        coverage_upper_bound: coverage_upper,
        conservative_path,
        optimistic_path,
        supporting_evidence_ids: evidence_ids.into_iter().collect(),
    })
}

fn simulate_path(inputs: PathInputs, step_hours: f64) -> WaterServiceBalancePath {
    let available = inputs.initial_storage + inputs.treated_volume;
    let distribution_limit = inputs.distribution_rate * step_hours;
    let demand_with_losses = inputs.demand_volume / (1.0 - inputs.loss_fraction);
    let withdrawal = available.min(distribution_limit).min(demand_with_losses);
    let delivered = withdrawal * (1.0 - inputs.loss_fraction);
    let distribution_loss = (withdrawal - delivered).max(0.0);
    let before_spill = (available - withdrawal).max(0.0);
    let ending_storage = before_spill.min(inputs.storage_capacity);
    let spill = (before_spill - ending_storage).max(0.0);
    let residual = available - delivered - distribution_loss - ending_storage - spill;

    WaterServiceBalancePath {
        demand_volume: inputs.demand_volume,
        treated_volume: inputs.treated_volume,
        available_water_volume: available,
        raw_distribution_withdrawal: withdrawal,
        customer_delivered_volume: delivered,
        distribution_loss_volume: distribution_loss,
        ending_storage_volume: ending_storage,
        spilled_volume: spill,
        mass_balance_residual: residual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::spatial_economic::Quantity;

    const AS_OF: &str = "2026-10-16T09:00:00Z";

    fn measurement(
        entity: &str,
        value: f64,
        unit: &str,
        uncertainty: Option<f64>,
    ) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity.into(),
            quantity: Quantity {
                value,
                unit: unit.into(),
                uncertainty_abs: uncertainty,
            },
            basis: ScenarioBasis::Assumed,
            rationale: format!("Synthetic test assumption for {entity}"),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario(id: &str) -> WaterServiceScenario {
        WaterServiceScenario {
            schema_version: WATER_SERVICE_SCHEMA_VERSION.into(),
            id: id.into(),
            as_of: AS_OF.into(),
            service_entity_id: "service-north".into(),
            step_hours: 1.0,
            source_flow_rate: measurement("source-a", 80.0, WATER_RATE_UNIT, None),
            pump_capacity: measurement("pump-a", 70.0, WATER_RATE_UNIT, None),
            treatment_capacity: measurement("treatment-a", 65.0, WATER_RATE_UNIT, None),
            distribution_capacity: measurement("distribution-a", 60.0, WATER_RATE_UNIT, None),
            initial_storage: measurement("tank-a", 100.0, WATER_VOLUME_UNIT, None),
            storage_capacity: measurement("tank-a", 500.0, WATER_VOLUME_UNIT, None),
            demand_rate: measurement("service-north", 100.0, WATER_RATE_UNIT, None),
            distribution_loss_fraction: measurement(
                "distribution-a",
                0.10,
                DIMENSIONLESS_UNIT,
                Some(0.02),
            ),
        }
    }

    #[test]
    fn serial_bottleneck_does_not_sum_stage_capacities_and_balance_closes() {
        let result = assess_water_service(&scenario("water-a"), &[]).unwrap();
        assert_eq!(result.treated_volume_lower_bound, 65.0);
        assert_eq!(result.treated_volume_upper_bound, 65.0);
        assert_eq!(result.customer_delivery_capacity_lower_bound, 52.8);
        assert!((result.customer_delivery_capacity_upper_bound - 55.2).abs() < 1e-12);
        assert_eq!(result.coverage_lower_bound, Some(0.528));
        assert_eq!(result.state, WaterServiceState::Degraded);
        assert!(result.conservative_path.mass_balance_residual.abs() < 1e-9);
        assert!(result.optimistic_path.mass_balance_residual.abs() < 1e-9);
        assert_eq!(result.conservative_path.ending_storage_volume, 105.0);
        assert!(
            (result.conservative_path.raw_distribution_withdrawal
                - result.conservative_path.customer_delivered_volume
                - result.conservative_path.distribution_loss_volume)
                .abs()
                < 1e-9
        );
        assert!(
            (result.optimistic_path.raw_distribution_withdrawal
                - result.optimistic_path.customer_delivered_volume
                - result.optimistic_path.distribution_loss_volume)
                .abs()
                < 1e-9
        );
        let mut more_source = scenario("water-more-source");
        more_source.source_flow_rate.quantity.value = 10_000.0;
        assert_eq!(
            assess_water_service(&more_source, &[])
                .unwrap()
                .treated_volume_lower_bound,
            65.0,
            "increasing source flow cannot exceed the pump/treatment serial bottleneck"
        );
    }

    #[test]
    fn excess_water_spills_and_storage_never_exceeds_capacity() {
        let mut input = scenario("water-spill");
        input.demand_rate.quantity.value = 1.0;
        input.storage_capacity.quantity.value = 120.0;
        let result = assess_water_service(&input, &[]).unwrap();
        assert_eq!(result.conservative_path.ending_storage_volume, 120.0);
        assert!(result.conservative_path.spilled_volume > 0.0);
        assert!(result.conservative_path.mass_balance_residual.abs() < 1e-9);
    }

    #[test]
    fn uncertainty_crossing_adequacy_boundary_is_indeterminate() {
        let mut input = scenario("water-uncertain");
        input.source_flow_rate.quantity.value = 100.0;
        input.pump_capacity.quantity.value = 100.0;
        input.treatment_capacity.quantity.value = 100.0;
        input.treatment_capacity.quantity.uncertainty_abs = Some(10.0);
        input.distribution_capacity.quantity.value = 100.0;
        input.initial_storage.quantity.value = 0.0;
        input.storage_capacity.quantity.value = 200.0;
        input.demand_rate.quantity.value = 100.0;
        input.distribution_loss_fraction.quantity.value = 0.10;
        input.distribution_loss_fraction.quantity.uncertainty_abs = Some(0.10);
        let result = assess_water_service(&input, &[]).unwrap();
        assert_eq!(result.state, WaterServiceState::Indeterminate);
        assert!(result.customer_delivery_capacity_lower_bound < 100.0);
        assert!(result.customer_delivery_capacity_upper_bound >= 100.0);
    }

    #[test]
    fn exact_units_are_required_without_implicit_conversion() {
        let mut input = scenario("water-units");
        input.source_flow_rate.quantity.unit = "L/s".into();
        let result = assess_water_service(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterServiceError::InvalidInput(errors))
                if errors.iter().any(|error| matches!(
                    error,
                    WaterServiceValidationError::UnitMismatch {
                        role: WaterMeasurementRole::SourceFlowRate,
                        ..
                    }
                ))
        ));
    }

    #[test]
    fn physically_inconsistent_storage_intervals_fail_closed() {
        let mut input = scenario("water-storage");
        input.initial_storage.quantity.value = 200.0;
        input.storage_capacity.quantity.value = 100.0;
        let result = assess_water_service(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterServiceError::InvalidInput(errors))
                if errors.contains(&WaterServiceValidationError::StorageCapacityIntervalInconsistent)
        ));
    }

    #[test]
    fn observed_measurement_requires_supporting_evidence() {
        let mut input = scenario("water-evidence");
        input.source_flow_rate.basis = ScenarioBasis::Observed;
        let result = assess_water_service(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterServiceError::InvalidInput(errors))
                if errors.iter().any(|error| matches!(
                    error,
                    WaterServiceValidationError::InvalidMeasurement {
                        role: WaterMeasurementRole::SourceFlowRate,
                        errors,
                    } if errors.contains(&ServiceFunctionValidationError::MissingEvidenceForObservedMeasurement)
                ))
        ));
    }

    #[test]
    fn batch_rejects_duplicate_ids_and_orders_outputs_stably() {
        let duplicate = [scenario("same"), scenario("same")];
        assert!(matches!(
            assess_water_services(&duplicate, &[]),
            Err(WaterServiceError::InvalidInput(errors))
                if errors.contains(&WaterServiceValidationError::DuplicateScenarioId)
        ));

        let inputs = [scenario("water-z"), scenario("water-a")];
        let results = assess_water_services(&inputs, &[]).unwrap();
        assert_eq!(results[0].scenario_id, "water-a");
        assert_eq!(results[1].scenario_id, "water-z");
        let bytes_a = serde_json::to_vec(&results).unwrap();
        let reversed = [scenario("water-a"), scenario("water-z")];
        let bytes_b = serde_json::to_vec(&assess_water_services(&reversed, &[]).unwrap()).unwrap();
        assert_eq!(bytes_a, bytes_b);
    }

    #[test]
    fn zero_demand_does_not_create_a_positive_adequacy_claim() {
        let mut input = scenario("water-zero-demand");
        input.demand_rate.quantity.value = 0.0;
        let result = assess_water_service(&input, &[]).unwrap();
        assert_eq!(result.state, WaterServiceState::Indeterminate);
        assert_eq!(result.coverage_lower_bound, None);
        assert_eq!(result.coverage_upper_bound, None);
    }
}
