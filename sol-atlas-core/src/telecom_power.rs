// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Bounded site-backup autonomy for the telecom resilience profile.
//!
//! This module compares the usable backup-energy interval at battery output
//! terminals with the radio-equipment load interval at the equipment input
//! terminals. It is an interval accounting calculation, not a battery
//! electrochemistry model, generator model, grid load-flow model, or proof
//! that the site provides end-user service.

use crate::dependency_logic::ScenarioBasis;
use crate::service_function::{
    FunctionMeasurement, ServiceFunctionAssessmentError, ServiceFunctionValidationError,
    build_evidence_index, validate_measurement,
};
use crate::spatial_economic::EvidenceRecord;
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};

pub const TELECOM_POWER_AUTONOMY_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsableEnergyMeasurementPoint {
    BatteryOutputTerminals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadioLoadMeasurementPoint {
    RadioAccessEquipmentInputTerminals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelecomPowerAutonomyStatus {
    /// Even the conservative autonomy bound covers the declared outage horizon.
    Sufficient,
    /// Even the optimistic autonomy bound is shorter than the outage horizon.
    Insufficient,
    /// The uncertainty interval straddles the outage horizon.
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelecomSitePowerAutonomyScenario {
    pub schema_version: String,
    pub id: String,
    /// Canonical UTC timestamp: YYYY-MM-DDTHH:MM:SSZ.
    pub as_of: String,
    pub site_id: String,
    pub outage_horizon_hours: f64,
    pub usable_energy_measurement_point: UsableEnergyMeasurementPoint,
    pub radio_load_measurement_point: RadioLoadMeasurementPoint,
    /// Usable energy at battery output terminals, in exact unit kWh.
    pub usable_backup_energy: FunctionMeasurement,
    /// Aggregate radio-access equipment load at its input terminals, in exact unit kW.
    /// This is not feeder, generator, substation, or upstream grid power.
    pub radio_equipment_load: FunctionMeasurement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelecomSitePowerAutonomyAssessment {
    pub schema_version: String,
    pub scenario_id: String,
    pub site_id: String,
    pub as_of: String,
    pub outage_horizon_hours: f64,
    pub usable_backup_energy_lower_kwh: f64,
    pub usable_backup_energy_upper_kwh: f64,
    pub radio_equipment_load_lower_kw: f64,
    pub radio_equipment_load_upper_kw: f64,
    pub autonomy_lower_hours: f64,
    pub autonomy_upper_hours: f64,
    pub status: TelecomPowerAutonomyStatus,
    pub usable_energy_basis: ScenarioBasis,
    pub load_basis: ScenarioBasis,
    pub usable_energy_evidence_ids: Vec<String>,
    pub load_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelecomSitePowerAutonomyValidationError {
    UnsupportedSchemaVersion,
    EmptyScenarioId,
    InvalidAsOfTimestamp,
    EmptySiteId,
    InvalidOutageHorizon,
    MeasurementEntityMismatch,
    UsableEnergyUnitMismatch,
    RadioLoadUnitMismatch,
    NegativeUsableEnergyLowerBound,
    NonPositiveRadioLoadLowerBound,
    NonFiniteDerivedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelecomSitePowerAutonomyError {
    InvalidInput(Vec<TelecomSitePowerAutonomyValidationError>),
    Measurement(ServiceFunctionAssessmentError),
}

/// Evaluate lower/upper autonomy bounds from admissible measurement intervals.
///
/// For usable energy E in kWh and equipment load P in kW:
///
/// - lower autonomy = E_lower / P_upper
/// - upper autonomy = E_upper / P_lower
///
/// Unknown or uncalibrated inputs must be represented with an Assumed basis
/// and no supporting evidence IDs. Observed inputs are checked using the
/// shared service-function evidence validator before the arithmetic runs.
pub fn assess_telecom_site_power_autonomy(
    scenario: &TelecomSitePowerAutonomyScenario,
    evidence_records: &[EvidenceRecord],
) -> Result<TelecomSitePowerAutonomyAssessment, TelecomSitePowerAutonomyError> {
    let mut errors = Vec::new();

    if scenario.schema_version != TELECOM_POWER_AUTONOMY_SCHEMA_VERSION {
        errors.push(TelecomSitePowerAutonomyValidationError::UnsupportedSchemaVersion);
    }
    if scenario.id.trim().is_empty() {
        errors.push(TelecomSitePowerAutonomyValidationError::EmptyScenarioId);
    }
    if !is_canonical_utc_timestamp(&scenario.as_of) {
        errors.push(TelecomSitePowerAutonomyValidationError::InvalidAsOfTimestamp);
    }
    if scenario.site_id.trim().is_empty() {
        errors.push(TelecomSitePowerAutonomyValidationError::EmptySiteId);
    }
    if !scenario.outage_horizon_hours.is_finite() || scenario.outage_horizon_hours <= 0.0 {
        errors.push(TelecomSitePowerAutonomyValidationError::InvalidOutageHorizon);
    }
    if scenario.usable_backup_energy.entity_id != scenario.site_id
        || scenario.radio_equipment_load.entity_id != scenario.site_id
    {
        errors.push(TelecomSitePowerAutonomyValidationError::MeasurementEntityMismatch);
    }
    if scenario.usable_backup_energy.quantity.unit != "kWh" {
        errors.push(TelecomSitePowerAutonomyValidationError::UsableEnergyUnitMismatch);
    }
    if scenario.radio_equipment_load.quantity.unit != "kW" {
        errors.push(TelecomSitePowerAutonomyValidationError::RadioLoadUnitMismatch);
    }
    if !errors.is_empty() {
        return Err(TelecomSitePowerAutonomyError::InvalidInput(errors));
    }

    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(TelecomSitePowerAutonomyError::Measurement)?;
    let mut measurement_errors: Vec<ServiceFunctionValidationError> = Vec::new();
    validate_measurement(
        &scenario.usable_backup_energy,
        &scenario.as_of,
        &evidence_by_id,
        &mut measurement_errors,
    );
    validate_measurement(
        &scenario.radio_equipment_load,
        &scenario.as_of,
        &evidence_by_id,
        &mut measurement_errors,
    );
    if !measurement_errors.is_empty() {
        return Err(TelecomSitePowerAutonomyError::Measurement(
            ServiceFunctionAssessmentError::InvalidInput(measurement_errors),
        ));
    }

    let energy_value = scenario.usable_backup_energy.quantity.value;
    let energy_uncertainty = scenario
        .usable_backup_energy
        .quantity
        .uncertainty_abs
        .unwrap_or(0.0);
    let load_value = scenario.radio_equipment_load.quantity.value;
    let load_uncertainty = scenario
        .radio_equipment_load
        .quantity
        .uncertainty_abs
        .unwrap_or(0.0);

    let energy_lower = energy_value - energy_uncertainty;
    let energy_upper = energy_value + energy_uncertainty;
    let load_lower = load_value - load_uncertainty;
    let load_upper = load_value + load_uncertainty;

    if !energy_lower.is_finite()
        || !energy_upper.is_finite()
        || !load_lower.is_finite()
        || !load_upper.is_finite()
    {
        return Err(TelecomSitePowerAutonomyError::InvalidInput(vec![
            TelecomSitePowerAutonomyValidationError::NonFiniteDerivedValue,
        ]));
    }
    if energy_lower < 0.0 {
        return Err(TelecomSitePowerAutonomyError::InvalidInput(vec![
            TelecomSitePowerAutonomyValidationError::NegativeUsableEnergyLowerBound,
        ]));
    }
    if load_lower <= 0.0 {
        return Err(TelecomSitePowerAutonomyError::InvalidInput(vec![
            TelecomSitePowerAutonomyValidationError::NonPositiveRadioLoadLowerBound,
        ]));
    }

    let autonomy_lower = energy_lower / load_upper;
    let autonomy_upper = energy_upper / load_lower;
    if !autonomy_lower.is_finite()
        || !autonomy_upper.is_finite()
        || autonomy_lower < 0.0
        || autonomy_upper < autonomy_lower
    {
        return Err(TelecomSitePowerAutonomyError::InvalidInput(vec![
            TelecomSitePowerAutonomyValidationError::NonFiniteDerivedValue,
        ]));
    }

    let status = if autonomy_lower >= scenario.outage_horizon_hours {
        TelecomPowerAutonomyStatus::Sufficient
    } else if autonomy_upper < scenario.outage_horizon_hours {
        TelecomPowerAutonomyStatus::Insufficient
    } else {
        TelecomPowerAutonomyStatus::Indeterminate
    };

    Ok(TelecomSitePowerAutonomyAssessment {
        schema_version: TELECOM_POWER_AUTONOMY_SCHEMA_VERSION.to_owned(),
        scenario_id: scenario.id.clone(),
        site_id: scenario.site_id.clone(),
        as_of: scenario.as_of.clone(),
        outage_horizon_hours: scenario.outage_horizon_hours,
        usable_backup_energy_lower_kwh: energy_lower,
        usable_backup_energy_upper_kwh: energy_upper,
        radio_equipment_load_lower_kw: load_lower,
        radio_equipment_load_upper_kw: load_upper,
        autonomy_lower_hours: autonomy_lower,
        autonomy_upper_hours: autonomy_upper,
        status,
        usable_energy_basis: scenario.usable_backup_energy.basis,
        load_basis: scenario.radio_equipment_load.basis,
        usable_energy_evidence_ids: scenario
            .usable_backup_energy
            .supporting_evidence_ids
            .clone(),
        load_evidence_ids: scenario.radio_equipment_load.supporting_evidence_ids.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_economic::Quantity;

    const AS_OF: &str = "2026-10-30T10:00:00Z";

    fn measurement(value: f64, uncertainty: Option<f64>, unit: &str) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: "site-001".to_owned(),
            quantity: Quantity {
                value,
                unit: unit.to_owned(),
                uncertainty_abs: uncertainty,
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic Tier-0 site-power fixture; not field evidence.".to_owned(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario(
        energy: f64,
        energy_uncertainty: Option<f64>,
        load: f64,
        load_uncertainty: Option<f64>,
        horizon: f64,
    ) -> TelecomSitePowerAutonomyScenario {
        TelecomSitePowerAutonomyScenario {
            schema_version: TELECOM_POWER_AUTONOMY_SCHEMA_VERSION.to_owned(),
            id: "scenario-001".to_owned(),
            as_of: AS_OF.to_owned(),
            site_id: "site-001".to_owned(),
            outage_horizon_hours: horizon,
            usable_energy_measurement_point: UsableEnergyMeasurementPoint::BatteryOutputTerminals,
            radio_load_measurement_point:
                RadioLoadMeasurementPoint::RadioAccessEquipmentInputTerminals,
            usable_backup_energy: measurement(energy, energy_uncertainty, "kWh"),
            radio_equipment_load: measurement(load, load_uncertainty, "kW"),
        }
    }

    #[test]
    fn committed_json_example_is_accepted_and_preserves_uncertainty_bounds() {
        let input: TelecomSitePowerAutonomyScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/telecom-site-power-autonomy.json"
        ))
        .unwrap();
        let result = assess_telecom_site_power_autonomy(&input, &[]).unwrap();
        assert_eq!(result.usable_backup_energy_lower_kwh, 10.0);
        assert_eq!(result.usable_backup_energy_upper_kwh, 14.0);
        assert_eq!(result.radio_equipment_load_lower_kw, 1.75);
        assert_eq!(result.radio_equipment_load_upper_kw, 2.25);
        assert_eq!(result.autonomy_lower_hours, 10.0 / 2.25);
        assert_eq!(result.autonomy_upper_hours, 14.0 / 1.75);
        assert_eq!(result.status, TelecomPowerAutonomyStatus::Sufficient);
    }

    #[test]
    fn point_interval_sufficiently_covers_outage_horizon() {
        let result = assess_telecom_site_power_autonomy(
            &scenario(12.0, None, 2.0, None, 5.0),
            &[],
        )
        .unwrap();
        assert_eq!(result.autonomy_lower_hours, 6.0);
        assert_eq!(result.autonomy_upper_hours, 6.0);
        assert_eq!(result.status, TelecomPowerAutonomyStatus::Sufficient);
    }

    #[test]
    fn point_interval_insufficient_for_outage_horizon() {
        let result = assess_telecom_site_power_autonomy(
            &scenario(4.0, None, 2.0, None, 3.0),
            &[],
        )
        .unwrap();
        assert_eq!(result.autonomy_lower_hours, 2.0);
        assert_eq!(result.autonomy_upper_hours, 2.0);
        assert_eq!(result.status, TelecomPowerAutonomyStatus::Insufficient);
    }

    #[test]
    fn overlapping_autonomy_interval_is_indeterminate() {
        let result = assess_telecom_site_power_autonomy(
            &scenario(5.0, Some(1.0), 2.0, Some(0.5), 3.0),
            &[],
        )
        .unwrap();
        assert_eq!(result.autonomy_lower_hours, 4.0 / 2.5);
        assert_eq!(result.autonomy_upper_hours, 6.0 / 1.5);
        assert_eq!(result.status, TelecomPowerAutonomyStatus::Indeterminate);
    }

    #[test]
    fn zero_usable_energy_is_insufficient_not_a_validation_error() {
        let result = assess_telecom_site_power_autonomy(
            &scenario(0.0, None, 2.0, None, 1.0),
            &[],
        )
        .unwrap();
        assert_eq!(result.autonomy_lower_hours, 0.0);
        assert_eq!(result.autonomy_upper_hours, 0.0);
        assert_eq!(result.status, TelecomPowerAutonomyStatus::Insufficient);
    }

    #[test]
    fn lower_load_bound_must_be_positive() {
        let err = assess_telecom_site_power_autonomy(
            &scenario(5.0, None, 1.0, Some(1.0), 2.0),
            &[],
        );
        assert!(matches!(
            err,
            Err(TelecomSitePowerAutonomyError::InvalidInput(errors))
                if errors.contains(
                    &TelecomSitePowerAutonomyValidationError::NonPositiveRadioLoadLowerBound
                )
        ));
    }

    #[test]
    fn usable_energy_and_load_units_are_exact_and_explicit() {
        let mut wrong_energy = scenario(5.0, None, 1.0, None, 2.0);
        wrong_energy.usable_backup_energy.quantity.unit = "Wh".to_owned();
        assert!(matches!(
            assess_telecom_site_power_autonomy(&wrong_energy, &[]),
            Err(TelecomSitePowerAutonomyError::InvalidInput(errors))
                if errors.contains(
                    &TelecomSitePowerAutonomyValidationError::UsableEnergyUnitMismatch
                )
        ));

        let mut wrong_load = scenario(5.0, None, 1.0, None, 2.0);
        wrong_load.radio_equipment_load.quantity.unit = "W".to_owned();
        assert!(matches!(
            assess_telecom_site_power_autonomy(&wrong_load, &[]),
            Err(TelecomSitePowerAutonomyError::InvalidInput(errors))
                if errors.contains(&TelecomSitePowerAutonomyValidationError::RadioLoadUnitMismatch)
        ));
    }

    #[test]
    fn mismatched_measurement_entity_is_rejected() {
        let mut input = scenario(5.0, None, 1.0, None, 2.0);
        input.radio_equipment_load.entity_id = "feeder-01".to_owned();
        assert!(matches!(
            assess_telecom_site_power_autonomy(&input, &[]),
            Err(TelecomSitePowerAutonomyError::InvalidInput(errors))
                if errors.contains(
                    &TelecomSitePowerAutonomyValidationError::MeasurementEntityMismatch
                )
        ));
    }

    #[test]
    fn invalid_horizon_and_timestamp_are_rejected() {
        let mut zero_horizon = scenario(5.0, None, 1.0, None, 0.0);
        assert!(matches!(
            assess_telecom_site_power_autonomy(&zero_horizon, &[]),
            Err(TelecomSitePowerAutonomyError::InvalidInput(errors))
                if errors.contains(&TelecomSitePowerAutonomyValidationError::InvalidOutageHorizon)
        ));

        zero_horizon = scenario(5.0, None, 1.0, None, 2.0);
        zero_horizon.as_of = "2026-10-30T10:00:00+02:00".to_owned();
        assert!(matches!(
            assess_telecom_site_power_autonomy(&zero_horizon, &[]),
            Err(TelecomSitePowerAutonomyError::InvalidInput(errors))
                if errors.contains(&TelecomSitePowerAutonomyValidationError::InvalidAsOfTimestamp)
        ));
    }

    #[test]
    fn negative_energy_lower_bound_is_rejected_not_clamped() {
        let err = assess_telecom_site_power_autonomy(
            &scenario(1.0, Some(2.0), 1.0, None, 1.0),
            &[],
        );
        assert!(matches!(
            err,
            Err(TelecomSitePowerAutonomyError::InvalidInput(errors))
                if errors.contains(
                    &TelecomSitePowerAutonomyValidationError::NegativeUsableEnergyLowerBound
                )
        ));
    }

    #[test]
    fn observed_measurement_requires_shared_evidence_reference() {
        let mut input = scenario(5.0, None, 1.0, None, 2.0);
        input.usable_backup_energy.basis = ScenarioBasis::Observed;
        assert!(matches!(
            assess_telecom_site_power_autonomy(&input, &[]),
            Err(TelecomSitePowerAutonomyError::Measurement(
                ServiceFunctionAssessmentError::InvalidInput(errors)
            )) if errors.contains(
                &ServiceFunctionValidationError::MissingEvidenceForObservedMeasurement
            )
        ));
    }

    #[test]
    fn assumed_measurement_cannot_smuggle_evidence_ids() {
        let mut input = scenario(5.0, None, 1.0, None, 2.0);
        input.radio_equipment_load.supporting_evidence_ids = vec!["unverified-id".to_owned()];
        assert!(matches!(
            assess_telecom_site_power_autonomy(&input, &[]),
            Err(TelecomSitePowerAutonomyError::Measurement(
                ServiceFunctionAssessmentError::InvalidInput(errors)
            )) if errors.contains(
                &ServiceFunctionValidationError::EvidenceForbiddenForAssumedMeasurement
            )
        ));
    }

    #[test]
    fn overflowed_autonomy_interval_is_rejected() {
        let err = assess_telecom_site_power_autonomy(
            &scenario(1.0e308, None, 1.0e-308, None, 1.0),
            &[],
        );
        assert!(matches!(
            err,
            Err(TelecomSitePowerAutonomyError::InvalidInput(errors))
                if errors.contains(&TelecomSitePowerAutonomyValidationError::NonFiniteDerivedValue)
        ));
    }
}
