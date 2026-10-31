// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic inter-temporal storage accounting for electricity and useful heat.
//!
//! The model carries state of charge across ordered intervals, applies standing loss,
//! enforces power and capacity limits, distinguishes charging/discharging losses from
//! curtailed input and unmet demand, and verifies the store's energy balance. It is a
//! bounded accounting substrate, not an electrochemical/thermal mechanism simulator,
//! network solver, or claim about a specific commercial storage system.

use crate::energy_pathway::EvidenceClass;
use std::collections::BTreeSet;

/// External service carrier around a storage asset. The stored-state quantity is in kWh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageCarrier {
    Electricity,
    ThermalHeat,
}

/// Available external energy and requested service during one ordered interval.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageInterval {
    pub interval_id: String,
    pub duration_hours: f64,
    /// Energy offered to the store before input power/capacity limits, in kWh.
    pub charge_available_kwh: f64,
    /// Useful output requested from the store, in kWh at the discharge boundary.
    pub discharge_requested_kwh: f64,
}

/// Storage configuration plus a time-ordered series of intervals.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyStorageRequest {
    pub schema_version: String,
    pub storage_id: String,
    pub synthetic: bool,
    pub carrier: StorageCarrier,
    /// Fixed thermal service grade for a thermal store; must be absent for electrical storage.
    /// Charging/discharging is interpreted at this grade; thermal stratification and temperature
    /// changes are outside v0.1 and require a more detailed thermal-state model.
    #[serde(default)]
    pub thermal_temperature_c: Option<f64>,
    pub capacity_kwh: f64,
    pub opening_energy_kwh: f64,
    pub max_charge_power_kw: f64,
    pub max_discharge_power_kw: f64,
    /// Stored energy gained per unit of accepted charge-carrier energy; (0, 1].
    pub charge_efficiency: f64,
    /// Useful carrier energy delivered per unit of stored energy withdrawn; (0, 1].
    pub discharge_efficiency: f64,
    /// Fraction of state lost per hour; composed over each interval's duration.
    pub standing_loss_fraction_per_hour: f64,
    /// Desired final stored-energy floor. Failure is reported as a result, not hidden.
    pub terminal_reserve_kwh: f64,
    pub evidence_class: EvidenceClass,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub evidence_as_of: Option<String>,
    pub intervals: Vec<StorageInterval>,
}

/// Explicit failures return no partial trajectory.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnergyStorageError {
    EmptySchemaVersion,
    UnsupportedSchemaVersion { schema_version: String },
    EmptyStorageId,
    NoIntervals,
    InvalidEnergy { field: String },
    InvalidPower { field: String },
    InvalidEfficiency { field: String },
    InvalidStandingLoss,
    MissingThermalTemperatureGrade,
    UnexpectedThermalTemperatureGrade,
    InvalidThermalTemperatureGrade,
    OpeningStateExceedsCapacity,
    TerminalReserveExceedsCapacity,
    MissingEvidenceReference,
    EmptyIntervalId,
    DuplicateIntervalId { interval_id: String },
    InvalidIntervalDuration { interval_id: String },
    InvalidIntervalEnergy { interval_id: String, field: String },
    SimultaneousChargeAndDischarge { interval_id: String },
    NonFiniteDerivedValue { interval_id: String },
    InvalidClosingState { interval_id: String },
}

/// One state transition with explicit input, output, loss and service-deficit accounting.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct StorageIntervalAssessment {
    pub interval_id: String,
    pub duration_hours: f64,
    pub opening_energy_kwh: f64,
    pub standing_loss_kwh: f64,
    pub energy_after_standing_loss_kwh: f64,
    pub charge_available_kwh: f64,
    pub charge_accepted_kwh: f64,
    pub charge_curtailed_kwh: f64,
    pub stored_charge_gain_kwh: f64,
    pub charging_loss_kwh: f64,
    pub discharge_requested_kwh: f64,
    pub discharge_delivered_kwh: f64,
    pub discharge_unserved_kwh: f64,
    pub stored_energy_withdrawn_kwh: f64,
    pub discharging_loss_kwh: f64,
    pub closing_energy_kwh: f64,
    pub energy_balance_residual_kwh: f64,
}

/// Complete state trajectory and terminal reserve result.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyStorageAssessment {
    pub assessment_schema_version: String,
    pub storage_id: String,
    pub carrier: StorageCarrier,
    pub thermal_temperature_c: Option<f64>,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub initial_energy_kwh: f64,
    pub final_energy_kwh: f64,
    pub capacity_kwh: f64,
    pub terminal_reserve_kwh: f64,
    pub terminal_reserve_met: bool,
    pub terminal_reserve_shortfall_kwh: f64,
    pub total_charge_accepted_kwh: f64,
    pub total_charge_curtailed_kwh: f64,
    pub total_discharge_delivered_kwh: f64,
    pub total_discharge_unserved_kwh: f64,
    pub total_standing_loss_kwh: f64,
    pub total_charging_loss_kwh: f64,
    pub total_discharging_loss_kwh: f64,
    pub total_loss_kwh: f64,
    pub trajectory_balance_residual_kwh: f64,
    pub evidence_class: EvidenceClass,
    pub evidence_ref: Option<String>,
    pub evidence_as_of: Option<String>,
    pub intervals: Vec<StorageIntervalAssessment>,
}

const NUMERIC_TOLERANCE: f64 = 1.0e-9;

fn finite_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn checked_accumulate(
    total: &mut f64,
    amount: f64,
    interval_id: &str,
) -> Result<(), EnergyStorageError> {
    *total += amount;
    if !total.is_finite() {
        return Err(EnergyStorageError::NonFiniteDerivedValue {
            interval_id: interval_id.to_owned(),
        });
    }
    Ok(())
}

/// Assess a deterministic, ordered storage trajectory.
///
/// Within each interval, standing losses are applied first. The store then either
/// charges or discharges (not both); this avoids hiding artificial round-trip arbitrage
/// within one timestep. Charging is limited by offered energy, input-side power and
/// remaining capacity. Discharge is limited by demand, output-side power and retained
/// state. Missing/unmet service is reported explicitly. Energy is never created by the
/// store's state transition, and terminal reserve shortfall is reported rather than
/// rewritten into a feasible result.
pub fn assess_energy_storage(
    request: &EnergyStorageRequest,
) -> Result<EnergyStorageAssessment, EnergyStorageError> {
    if request.schema_version.trim().is_empty() {
        return Err(EnergyStorageError::EmptySchemaVersion);
    }
    if request.schema_version != "energy-storage-v0.1" {
        return Err(EnergyStorageError::UnsupportedSchemaVersion {
            schema_version: request.schema_version.clone(),
        });
    }
    if request.storage_id.trim().is_empty() {
        return Err(EnergyStorageError::EmptyStorageId);
    }
    if request.intervals.is_empty() {
        return Err(EnergyStorageError::NoIntervals);
    }
    for (field, value) in [
        ("capacity_kwh", request.capacity_kwh),
        ("opening_energy_kwh", request.opening_energy_kwh),
        ("terminal_reserve_kwh", request.terminal_reserve_kwh),
    ] {
        if !finite_nonnegative(value) {
            return Err(EnergyStorageError::InvalidEnergy {
                field: field.to_owned(),
            });
        }
    }
    for (field, value) in [
        ("max_charge_power_kw", request.max_charge_power_kw),
        ("max_discharge_power_kw", request.max_discharge_power_kw),
    ] {
        if !finite_nonnegative(value) {
            return Err(EnergyStorageError::InvalidPower {
                field: field.to_owned(),
            });
        }
    }
    for (field, value) in [
        ("charge_efficiency", request.charge_efficiency),
        ("discharge_efficiency", request.discharge_efficiency),
    ] {
        if !value.is_finite() || value <= 0.0 || value > 1.0 {
            return Err(EnergyStorageError::InvalidEfficiency {
                field: field.to_owned(),
            });
        }
    }
    if !request.standing_loss_fraction_per_hour.is_finite()
        || !(0.0..=1.0).contains(&request.standing_loss_fraction_per_hour)
    {
        return Err(EnergyStorageError::InvalidStandingLoss);
    }
    match (request.carrier, request.thermal_temperature_c) {
        (StorageCarrier::Electricity, None) => {}
        (StorageCarrier::Electricity, Some(_)) => {
            return Err(EnergyStorageError::UnexpectedThermalTemperatureGrade);
        }
        (StorageCarrier::ThermalHeat, None) => {
            return Err(EnergyStorageError::MissingThermalTemperatureGrade);
        }
        (StorageCarrier::ThermalHeat, Some(temperature_c)) => {
            if !temperature_c.is_finite() || temperature_c + 273.15 <= 0.0 {
                return Err(EnergyStorageError::InvalidThermalTemperatureGrade);
            }
        }
    }
    if request.opening_energy_kwh > request.capacity_kwh {
        return Err(EnergyStorageError::OpeningStateExceedsCapacity);
    }
    if request.terminal_reserve_kwh > request.capacity_kwh {
        return Err(EnergyStorageError::TerminalReserveExceedsCapacity);
    }
    if request.evidence_class != EvidenceClass::AnalystAssumption
        && request
            .evidence_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
    {
        return Err(EnergyStorageError::MissingEvidenceReference);
    }

    let mut ids = BTreeSet::new();
    for interval in &request.intervals {
        if interval.interval_id.trim().is_empty() {
            return Err(EnergyStorageError::EmptyIntervalId);
        }
        if !ids.insert(interval.interval_id.as_str()) {
            return Err(EnergyStorageError::DuplicateIntervalId {
                interval_id: interval.interval_id.clone(),
            });
        }
        if !interval.duration_hours.is_finite() || interval.duration_hours <= 0.0 {
            return Err(EnergyStorageError::InvalidIntervalDuration {
                interval_id: interval.interval_id.clone(),
            });
        }
        for (field, value) in [
            ("charge_available_kwh", interval.charge_available_kwh),
            ("discharge_requested_kwh", interval.discharge_requested_kwh),
        ] {
            if !finite_nonnegative(value) {
                return Err(EnergyStorageError::InvalidIntervalEnergy {
                    interval_id: interval.interval_id.clone(),
                    field: field.to_owned(),
                });
            }
        }
        if interval.charge_available_kwh > 0.0 && interval.discharge_requested_kwh > 0.0 {
            return Err(EnergyStorageError::SimultaneousChargeAndDischarge {
                interval_id: interval.interval_id.clone(),
            });
        }
    }

    let initial_energy = request.opening_energy_kwh;
    let mut state = initial_energy;
    let mut total_charge_accepted = 0.0;
    let mut total_charge_curtailed = 0.0;
    let mut total_discharge_delivered = 0.0;
    let mut total_discharge_unserved = 0.0;
    let mut total_standing_loss = 0.0;
    let mut total_charging_loss = 0.0;
    let mut total_discharging_loss = 0.0;
    let mut interval_results = Vec::with_capacity(request.intervals.len());

    for interval in &request.intervals {
        let opening = state;
        let retention_fraction =
            (1.0 - request.standing_loss_fraction_per_hour).powf(interval.duration_hours);
        let after_standing_loss = opening * retention_fraction;
        let standing_loss = opening - after_standing_loss;
        let charge_power_energy_limit = request.max_charge_power_kw * interval.duration_hours;
        let discharge_power_energy_limit =
            request.max_discharge_power_kw * interval.duration_hours;
        if !retention_fraction.is_finite()
            || !after_standing_loss.is_finite()
            || !standing_loss.is_finite()
            || !charge_power_energy_limit.is_finite()
            || !discharge_power_energy_limit.is_finite()
        {
            return Err(EnergyStorageError::NonFiniteDerivedValue {
                interval_id: interval.interval_id.clone(),
            });
        }

        let mut charge_accepted = 0.0;
        let mut charge_curtailed = interval.charge_available_kwh;
        let mut stored_charge_gain = 0.0;
        let mut charging_loss = 0.0;
        let mut discharge_delivered = 0.0;
        let mut discharge_unserved = interval.discharge_requested_kwh;
        let mut stored_energy_withdrawn = 0.0;
        let mut discharging_loss = 0.0;

        if interval.charge_available_kwh > 0.0 {
            let room = (request.capacity_kwh - after_standing_loss).max(0.0);
            let input_limit_from_capacity = room / request.charge_efficiency;
            if !input_limit_from_capacity.is_finite() {
                return Err(EnergyStorageError::NonFiniteDerivedValue {
                    interval_id: interval.interval_id.clone(),
                });
            }
            charge_accepted = interval
                .charge_available_kwh
                .min(charge_power_energy_limit)
                .min(input_limit_from_capacity);
            charge_curtailed = interval.charge_available_kwh - charge_accepted;
            stored_charge_gain = charge_accepted * request.charge_efficiency;
            charging_loss = charge_accepted - stored_charge_gain;
            state = after_standing_loss + stored_charge_gain;
        } else if interval.discharge_requested_kwh > 0.0 {
            let output_available = after_standing_loss * request.discharge_efficiency;
            discharge_delivered = interval
                .discharge_requested_kwh
                .min(discharge_power_energy_limit)
                .min(output_available);
            discharge_unserved = interval.discharge_requested_kwh - discharge_delivered;
            stored_energy_withdrawn = discharge_delivered / request.discharge_efficiency;
            discharging_loss = stored_energy_withdrawn - discharge_delivered;
            state = after_standing_loss - stored_energy_withdrawn;
        } else {
            state = after_standing_loss;
        }

        // Remove only tiny floating-point boundary error; substantive violations fail closed.
        if state < 0.0 && state.abs() <= NUMERIC_TOLERANCE {
            state = 0.0;
        }
        if state > request.capacity_kwh
            && state - request.capacity_kwh <= NUMERIC_TOLERANCE
        {
            state = request.capacity_kwh;
        }
        if !state.is_finite()
            || state < 0.0
            || state > request.capacity_kwh + NUMERIC_TOLERANCE
        {
            return Err(EnergyStorageError::InvalidClosingState {
                interval_id: interval.interval_id.clone(),
            });
        }

        let balance_residual = opening
            + charge_accepted
            - state
            - standing_loss
            - charging_loss
            - discharge_delivered
            - discharging_loss;
        for value in [
            charge_accepted,
            charge_curtailed,
            stored_charge_gain,
            charging_loss,
            discharge_delivered,
            discharge_unserved,
            stored_energy_withdrawn,
            discharging_loss,
            balance_residual,
        ] {
            if !value.is_finite() {
                return Err(EnergyStorageError::NonFiniteDerivedValue {
                    interval_id: interval.interval_id.clone(),
                });
            }
        }

        checked_accumulate(
            &mut total_charge_accepted,
            charge_accepted,
            &interval.interval_id,
        )?;
        checked_accumulate(
            &mut total_charge_curtailed,
            charge_curtailed,
            &interval.interval_id,
        )?;
        checked_accumulate(
            &mut total_discharge_delivered,
            discharge_delivered,
            &interval.interval_id,
        )?;
        checked_accumulate(
            &mut total_discharge_unserved,
            discharge_unserved,
            &interval.interval_id,
        )?;
        checked_accumulate(
            &mut total_standing_loss,
            standing_loss,
            &interval.interval_id,
        )?;
        checked_accumulate(
            &mut total_charging_loss,
            charging_loss,
            &interval.interval_id,
        )?;
        checked_accumulate(
            &mut total_discharging_loss,
            discharging_loss,
            &interval.interval_id,
        )?;

        interval_results.push(StorageIntervalAssessment {
            interval_id: interval.interval_id.clone(),
            duration_hours: interval.duration_hours,
            opening_energy_kwh: opening,
            standing_loss_kwh: standing_loss,
            energy_after_standing_loss_kwh: after_standing_loss,
            charge_available_kwh: interval.charge_available_kwh,
            charge_accepted_kwh: charge_accepted,
            charge_curtailed_kwh: charge_curtailed,
            stored_charge_gain_kwh: stored_charge_gain,
            charging_loss_kwh: charging_loss,
            discharge_requested_kwh: interval.discharge_requested_kwh,
            discharge_delivered_kwh: discharge_delivered,
            discharge_unserved_kwh: discharge_unserved,
            stored_energy_withdrawn_kwh: stored_energy_withdrawn,
            discharging_loss_kwh: discharging_loss,
            closing_energy_kwh: state,
            energy_balance_residual_kwh: balance_residual,
        });
    }

    let total_loss = total_standing_loss + total_charging_loss + total_discharging_loss;
    let trajectory_balance_residual = initial_energy
        + total_charge_accepted
        - state
        - total_loss
        - total_discharge_delivered;
    if !total_loss.is_finite() || !trajectory_balance_residual.is_finite() {
        return Err(EnergyStorageError::NonFiniteDerivedValue {
            interval_id: request.intervals[request.intervals.len() - 1]
                .interval_id
                .clone(),
        });
    }
    let terminal_reserve_shortfall = (request.terminal_reserve_kwh - state).max(0.0);

    Ok(EnergyStorageAssessment {
        assessment_schema_version: "1".to_owned(),
        storage_id: request.storage_id.clone(),
        carrier: request.carrier,
        thermal_temperature_c: request.thermal_temperature_c,
        synthetic: request.synthetic,
        claim_ceiling: concat!(
            "calculated state trajectory; no device metering ",
            "or physical validation",
        )
        .to_owned(),
        initial_energy_kwh: initial_energy,
        final_energy_kwh: state,
        capacity_kwh: request.capacity_kwh,
        terminal_reserve_kwh: request.terminal_reserve_kwh,
        terminal_reserve_met: terminal_reserve_shortfall == 0.0,
        terminal_reserve_shortfall_kwh: terminal_reserve_shortfall,
        total_charge_accepted_kwh: total_charge_accepted,
        total_charge_curtailed_kwh: total_charge_curtailed,
        total_discharge_delivered_kwh: total_discharge_delivered,
        total_discharge_unserved_kwh: total_discharge_unserved,
        total_standing_loss_kwh: total_standing_loss,
        total_charging_loss_kwh: total_charging_loss,
        total_discharging_loss_kwh: total_discharging_loss,
        total_loss_kwh: total_loss,
        trajectory_balance_residual_kwh: trajectory_balance_residual,
        evidence_class: request.evidence_class,
        evidence_ref: request.evidence_ref.clone(),
        evidence_as_of: request.evidence_as_of.clone(),
        intervals: interval_results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(intervals: Vec<StorageInterval>) -> EnergyStorageRequest {
        EnergyStorageRequest {
            schema_version: "energy-storage-v0.1".to_owned(),
            storage_id: "test-store".to_owned(),
            synthetic: true,
            carrier: StorageCarrier::Electricity,
            thermal_temperature_c: None,
            capacity_kwh: 20.0,
            opening_energy_kwh: 0.0,
            max_charge_power_kw: 12.0,
            max_discharge_power_kw: 10.0,
            charge_efficiency: 0.9,
            discharge_efficiency: 0.8,
            standing_loss_fraction_per_hour: 0.01,
            terminal_reserve_kwh: 0.0,
            evidence_class: EvidenceClass::AnalystAssumption,
            evidence_ref: None,
            evidence_as_of: None,
            intervals,
        }
    }

    fn interval(id: &str, duration_hours: f64, charge: f64, discharge: f64) -> StorageInterval {
        StorageInterval {
            interval_id: id.to_owned(),
            duration_hours,
            charge_available_kwh: charge,
            discharge_requested_kwh: discharge,
        }
    }

    #[test]
    fn state_continuity_efficiency_standing_loss_and_balance_close() {
        let assessment = assess_energy_storage(&request(vec![
            interval("charge", 1.0, 10.0, 0.0),
            interval("discharge", 1.0, 0.0, 4.0),
        ]))
        .unwrap();
        assert!((assessment.intervals[0].closing_energy_kwh - 9.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.intervals[1].standing_loss_kwh - 0.09).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.intervals[1].closing_energy_kwh - 3.91).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.total_charge_accepted_kwh - 10.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.total_discharge_delivered_kwh - 4.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.total_loss_kwh - 2.09).abs() < NUMERIC_TOLERANCE);
        assert!(assessment.trajectory_balance_residual_kwh.abs() < NUMERIC_TOLERANCE);
        assert!(assessment
            .intervals
            .iter()
            .all(|row| row.energy_balance_residual_kwh.abs() < NUMERIC_TOLERANCE));
    }

    #[test]
    fn charge_is_limited_by_power_and_remaining_capacity() {
        let mut request = request(vec![interval("charge", 1.0, 10.0, 0.0)]);
        request.capacity_kwh = 5.0;
        request.opening_energy_kwh = 3.0;
        request.standing_loss_fraction_per_hour = 0.0;
        let assessment = assess_energy_storage(&request).unwrap();
        let expected_accepted = 2.0 / request.charge_efficiency;
        let expected_curtailed = 10.0 - expected_accepted;
        assert!(
            (assessment.intervals[0].charge_accepted_kwh - expected_accepted).abs()
                < NUMERIC_TOLERANCE
        );
        assert!(
            (assessment.intervals[0].charge_curtailed_kwh - expected_curtailed).abs()
                < NUMERIC_TOLERANCE
        );
        assert!((assessment.final_energy_kwh - 5.0).abs() < NUMERIC_TOLERANCE);
        assert!(assessment.trajectory_balance_residual_kwh.abs() < NUMERIC_TOLERANCE);
    }

    #[test]
    fn discharge_power_limit_reports_unserved_demand() {
        let mut request = request(vec![interval("discharge", 1.0, 0.0, 10.0)]);
        request.opening_energy_kwh = 4.0;
        request.max_discharge_power_kw = 2.0;
        request.standing_loss_fraction_per_hour = 0.0;
        let assessment = assess_energy_storage(&request).unwrap();
        assert_eq!(assessment.intervals[0].discharge_delivered_kwh, 2.0);
        assert_eq!(assessment.intervals[0].discharge_unserved_kwh, 8.0);
        assert!((assessment.final_energy_kwh - 1.5).abs() < NUMERIC_TOLERANCE);
        assert!(assessment.trajectory_balance_residual_kwh.abs() < NUMERIC_TOLERANCE);
    }


    #[test]
    fn thermal_storage_requires_explicit_temperature_grade() {
        let mut request = request(vec![interval("idle", 1.0, 0.0, 0.0)]);
        request.carrier = StorageCarrier::ThermalHeat;
        assert!(matches!(
            assess_energy_storage(&request),
            Err(EnergyStorageError::MissingThermalTemperatureGrade)
        ));

        request.thermal_temperature_c = Some(80.0);
        let assessment = assess_energy_storage(&request).unwrap();
        assert_eq!(assessment.thermal_temperature_c, Some(80.0));
    }

    #[test]
    fn thermal_grade_is_invalid_for_electric_store_and_validates_absolute_temperature() {
        let mut request = request(vec![interval("idle", 1.0, 0.0, 0.0)]);
        request.thermal_temperature_c = Some(25.0);
        assert!(matches!(
            assess_energy_storage(&request),
            Err(EnergyStorageError::UnexpectedThermalTemperatureGrade)
        ));

        let mut request = request(vec![interval("idle", 1.0, 0.0, 0.0)]);
        request.carrier = StorageCarrier::ThermalHeat;
        request.thermal_temperature_c = Some(-300.0);
        assert!(matches!(
            assess_energy_storage(&request),
            Err(EnergyStorageError::InvalidThermalTemperatureGrade)
        ));
    }

    #[test]
    fn no_energy_input_cannot_create_storage_energy() {
        let mut request = request(vec![interval("idle", 1.0, 0.0, 0.0)]);
        request.standing_loss_fraction_per_hour = 0.0;
        let assessment = assess_energy_storage(&request).unwrap();
        assert_eq!(assessment.final_energy_kwh, 0.0);
        assert_eq!(assessment.total_discharge_delivered_kwh, 0.0);
    }

    #[test]
    fn same_interval_charge_and_discharge_fails_closed() {
        let request = request(vec![interval("ambiguous", 1.0, 1.0, 1.0)]);
        assert!(matches!(
            assess_energy_storage(&request),
            Err(EnergyStorageError::SimultaneousChargeAndDischarge { .. })
        ));
    }

    #[test]
    fn opening_state_and_terminal_reserve_bounds_are_checked() {
        let mut request = request(vec![interval("idle", 1.0, 0.0, 0.0)]);
        request.opening_energy_kwh = 21.0;
        assert!(matches!(
            assess_energy_storage(&request),
            Err(EnergyStorageError::OpeningStateExceedsCapacity)
        ));
        let mut request = request(vec![interval("idle", 1.0, 0.0, 0.0)]);
        request.terminal_reserve_kwh = 21.0;
        assert!(matches!(
            assess_energy_storage(&request),
            Err(EnergyStorageError::TerminalReserveExceedsCapacity)
        ));
    }

    #[test]
    fn invalid_efficiencies_and_nonfinite_inputs_fail_closed() {
        let mut request = request(vec![interval("charge", 1.0, 1.0, 0.0)]);
        request.charge_efficiency = 0.0;
        assert!(matches!(
            assess_energy_storage(&request),
            Err(EnergyStorageError::InvalidEfficiency { .. })
        ));
        let mut request = request(vec![interval("charge", 1.0, f64::NAN, 0.0)]);
        assert!(matches!(
            assess_energy_storage(&request),
            Err(EnergyStorageError::InvalidIntervalEnergy { .. })
        ));
    }

    #[test]
    fn evidence_classes_require_references_and_version_is_pinned() {
        let mut operating_plant_request = request(vec![interval("idle", 1.0, 0.0, 0.0)]);
        operating_plant_request.evidence_class = EvidenceClass::OperatingPlant;
        assert!(matches!(
            assess_energy_storage(&operating_plant_request),
            Err(EnergyStorageError::MissingEvidenceReference)
        ));
        let mut future_version_request = request(vec![interval("idle", 1.0, 0.0, 0.0)]);
        future_version_request.schema_version = "future-v9".to_owned();
        assert!(matches!(
            assess_energy_storage(&future_version_request),
            Err(EnergyStorageError::UnsupportedSchemaVersion { .. })
        ));
    }

    #[test]
    fn terminal_reserve_shortfall_is_reported_not_hidden() {
        let mut request = request(vec![interval("discharge", 1.0, 0.0, 2.0)]);
        request.opening_energy_kwh = 4.0;
        request.terminal_reserve_kwh = 3.0;
        request.standing_loss_fraction_per_hour = 0.0;
        let assessment = assess_energy_storage(&request).unwrap();
        assert!(!assessment.terminal_reserve_met);
        assert!((assessment.terminal_reserve_shortfall_kwh - 1.5).abs() < NUMERIC_TOLERANCE);
    }

    #[test]
    fn repeat_runs_are_deterministic() {
        let request = request(vec![
            interval("charge", 1.0, 10.0, 0.0),
            interval("discharge", 1.0, 0.0, 4.0),
        ]);
        let first = serde_json::to_vec(&assess_energy_storage(&request).unwrap()).unwrap();
        let second = serde_json::to_vec(&assess_energy_storage(&request).unwrap()).unwrap();
        assert_eq!(first, second);
    }
}
