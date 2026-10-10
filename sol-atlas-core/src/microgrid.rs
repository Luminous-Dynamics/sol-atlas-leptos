// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic interval-level microgrid dispatch.
//!
//! Inputs are interval energies in kWh with a common interval duration. This is
//! an auditable accounting baseline, not a generation forecast, optimizer, or
//! grid-protection controller.

use serde::{Deserialize, Serialize};

/// Battery state and rate limits. State of charge is stored energy in kWh.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BatteryConfig {
    pub capacity_kwh: f64,
    pub initial_soc_kwh: f64,
    pub minimum_soc_kwh: f64,
    pub max_charge_power_kw: f64,
    pub max_discharge_power_kw: f64,
    /// Stored fraction of energy sent into the battery, in (0, 1].
    pub charge_efficiency: f64,
    /// Delivered fraction of energy withdrawn from storage, in (0, 1].
    pub discharge_efficiency: f64,
}

/// Interval-level energy flows, all measured in kWh.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicrogridInterval {
    pub load_kwh: f64,
    pub renewable_generation_kwh: f64,
    pub renewable_to_load_kwh: f64,
    pub battery_charge_input_kwh: f64,
    pub battery_charge_stored_kwh: f64,
    pub battery_discharge_withdrawn_kwh: f64,
    pub battery_discharge_to_load_kwh: f64,
    pub grid_import_kwh: f64,
    pub grid_export_kwh: f64,
    pub renewable_curtailed_kwh: f64,
    pub unserved_load_kwh: f64,
    pub battery_soc_kwh: f64,
}

/// Aggregate flows across the simulated horizon, all in kWh.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicrogridTotals {
    pub load_kwh: f64,
    pub renewable_generation_kwh: f64,
    pub renewable_to_load_kwh: f64,
    pub battery_charge_input_kwh: f64,
    pub battery_charge_stored_kwh: f64,
    pub battery_discharge_withdrawn_kwh: f64,
    pub battery_discharge_to_load_kwh: f64,
    pub grid_import_kwh: f64,
    pub grid_export_kwh: f64,
    pub renewable_curtailed_kwh: f64,
    pub unserved_load_kwh: f64,
    pub battery_energy_losses_kwh: f64,
    pub final_battery_soc_kwh: f64,
}

/// Output of a deterministic dispatch run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicrogridSimulation {
    pub timestep_hours: f64,
    pub intervals: Vec<MicrogridInterval>,
    pub totals: MicrogridTotals,
}

/// Input validation failures. Invalid inputs are rejected rather than clamped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicrogridError {
    EmptyProfile,
    ProfileLengthMismatch,
    InvalidTimestep,
    InvalidEnergyInput,
    InvalidBatteryConfig,
    NonFiniteResult,
}

impl std::fmt::Display for MicrogridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyProfile => "microgrid profiles must not be empty",
            Self::ProfileLengthMismatch => "microgrid profiles must have identical lengths",
            Self::InvalidTimestep => "timestep must be finite and positive",
            Self::InvalidEnergyInput => "load and generation must be finite and non-negative",
            Self::InvalidBatteryConfig => "battery configuration is outside its physical domain",
            Self::NonFiniteResult => "microgrid calculation produced a non-finite result",
        };
        f.write_str(message)
    }
}

impl std::error::Error for MicrogridError {}

impl BatteryConfig {
    fn is_valid(self) -> bool {
        let values = [
            self.capacity_kwh,
            self.initial_soc_kwh,
            self.minimum_soc_kwh,
            self.max_charge_power_kw,
            self.max_discharge_power_kw,
            self.charge_efficiency,
            self.discharge_efficiency,
        ];
        values.iter().all(|value| value.is_finite())
            && self.capacity_kwh >= 0.0
            && self.minimum_soc_kwh >= 0.0
            && self.initial_soc_kwh >= self.minimum_soc_kwh
            && self.initial_soc_kwh <= self.capacity_kwh
            && self.max_charge_power_kw >= 0.0
            && self.max_discharge_power_kw >= 0.0
            && self.charge_efficiency > 0.0
            && self.charge_efficiency <= 1.0
            && self.discharge_efficiency > 0.0
            && self.discharge_efficiency <= 1.0
    }
}

/// Simulate priority dispatch for a grid-connected or islandable site.
///
/// Renewable output serves load first. Surplus charges the battery subject to
/// its power and capacity limits, then exports. During a deficit, the battery
/// discharges subject to its power and minimum-SOC limits. Remaining deficit
/// is imported if the grid is available or counted as unserved during an outage.
/// Surplus remaining after charging is exported only when the grid is available;
/// otherwise it is reported as curtailed renewable energy.
///
/// All three profiles must have the same non-zero length. Each energy value
/// describes the whole interval, whose duration is specified in hours.
pub fn simulate_microgrid(
    load_kwh: &[f64],
    renewable_generation_kwh: &[f64],
    grid_available: &[bool],
    timestep_hours: f64,
    battery: BatteryConfig,
) -> Result<MicrogridSimulation, MicrogridError> {
    if load_kwh.is_empty() {
        return Err(MicrogridError::EmptyProfile);
    }
    if load_kwh.len() != renewable_generation_kwh.len()
        || load_kwh.len() != grid_available.len()
    {
        return Err(MicrogridError::ProfileLengthMismatch);
    }
    if !timestep_hours.is_finite() || timestep_hours <= 0.0 {
        return Err(MicrogridError::InvalidTimestep);
    }
    if !battery.is_valid() {
        return Err(MicrogridError::InvalidBatteryConfig);
    }
    if load_kwh
        .iter()
        .chain(renewable_generation_kwh.iter())
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(MicrogridError::InvalidEnergyInput);
    }

    let mut soc_kwh = battery.initial_soc_kwh;
    let mut intervals = Vec::with_capacity(load_kwh.len());

    for ((&load, &generation), &grid_is_available) in load_kwh
        .iter()
        .zip(renewable_generation_kwh)
        .zip(grid_available)
    {
        let renewable_to_load = load.min(generation);
        let surplus = (generation - load).max(0.0);
        let deficit = (load - generation).max(0.0);

        let charge_input = if surplus > 0.0 {
            let power_limit = battery.max_charge_power_kw * timestep_hours;
            let capacity_limit =
                ((battery.capacity_kwh - soc_kwh) / battery.charge_efficiency).max(0.0);
            surplus.min(power_limit).min(capacity_limit)
        } else {
            0.0
        };
        let charge_stored = charge_input * battery.charge_efficiency;
        soc_kwh = (soc_kwh + charge_stored).min(battery.capacity_kwh);

        let available_soc = (soc_kwh - battery.minimum_soc_kwh).max(0.0);
        let discharge_to_load = if deficit > 0.0 {
            let power_limit = battery.max_discharge_power_kw * timestep_hours;
            let energy_limit = available_soc * battery.discharge_efficiency;
            deficit.min(power_limit).min(energy_limit)
        } else {
            0.0
        };
        let discharge_withdrawn = discharge_to_load / battery.discharge_efficiency;
        soc_kwh = (soc_kwh - discharge_withdrawn).max(battery.minimum_soc_kwh);

        let remaining_deficit = (deficit - discharge_to_load).max(0.0);
        let grid_import = if grid_is_available { remaining_deficit } else { 0.0 };
        let unserved_load = if grid_is_available { 0.0 } else { remaining_deficit };
        let remaining_surplus = (surplus - charge_input).max(0.0);
        let grid_export = if grid_is_available { remaining_surplus } else { 0.0 };
        let renewable_curtailed = if grid_is_available { 0.0 } else { remaining_surplus };

        intervals.push(MicrogridInterval {
            load_kwh: load,
            renewable_generation_kwh: generation,
            renewable_to_load_kwh: renewable_to_load,
            battery_charge_input_kwh: charge_input,
            battery_charge_stored_kwh: charge_stored,
            battery_discharge_withdrawn_kwh: discharge_withdrawn,
            battery_discharge_to_load_kwh: discharge_to_load,
            grid_import_kwh: grid_import,
            grid_export_kwh: grid_export,
            renewable_curtailed_kwh: renewable_curtailed,
            unserved_load_kwh: unserved_load,
            battery_soc_kwh: soc_kwh,
        });
    }

    let sum = |select: fn(&MicrogridInterval) -> f64| -> f64 {
        intervals.iter().map(select).sum()
    };
    let totals = MicrogridTotals {
        load_kwh: sum(|i| i.load_kwh),
        renewable_generation_kwh: sum(|i| i.renewable_generation_kwh),
        renewable_to_load_kwh: sum(|i| i.renewable_to_load_kwh),
        battery_charge_input_kwh: sum(|i| i.battery_charge_input_kwh),
        battery_charge_stored_kwh: sum(|i| i.battery_charge_stored_kwh),
        battery_discharge_withdrawn_kwh: sum(|i| i.battery_discharge_withdrawn_kwh),
        battery_discharge_to_load_kwh: sum(|i| i.battery_discharge_to_load_kwh),
        grid_import_kwh: sum(|i| i.grid_import_kwh),
        grid_export_kwh: sum(|i| i.grid_export_kwh),
        renewable_curtailed_kwh: sum(|i| i.renewable_curtailed_kwh),
        unserved_load_kwh: sum(|i| i.unserved_load_kwh),
        battery_energy_losses_kwh: sum(|i| {
            (i.battery_charge_input_kwh - i.battery_charge_stored_kwh)
                + (i.battery_discharge_withdrawn_kwh - i.battery_discharge_to_load_kwh)
        }),
        final_battery_soc_kwh: soc_kwh,
    };

    let values = [
        totals.load_kwh,
        totals.renewable_generation_kwh,
        totals.renewable_to_load_kwh,
        totals.battery_charge_input_kwh,
        totals.battery_charge_stored_kwh,
        totals.battery_discharge_withdrawn_kwh,
        totals.battery_discharge_to_load_kwh,
        totals.grid_import_kwh,
        totals.grid_export_kwh,
        totals.renewable_curtailed_kwh,
        totals.unserved_load_kwh,
        totals.battery_energy_losses_kwh,
        totals.final_battery_soc_kwh,
    ];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(MicrogridError::NonFiniteResult);
    }

    Ok(MicrogridSimulation { timestep_hours, intervals, totals })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn battery() -> BatteryConfig {
        BatteryConfig {
            capacity_kwh: 8.0,
            initial_soc_kwh: 0.0,
            minimum_soc_kwh: 0.0,
            max_charge_power_kw: 4.0,
            max_discharge_power_kw: 4.0,
            charge_efficiency: 0.9,
            discharge_efficiency: 0.9,
        }
    }

    #[test]
    fn surplus_charges_battery_before_exporting() {
        let result = simulate_microgrid(&[2.0], &[5.0], &[true], 1.0, battery()).unwrap();
        let interval = &result.intervals[0];
        assert_eq!(interval.renewable_to_load_kwh, 2.0);
        assert_eq!(interval.battery_charge_input_kwh, 3.0);
        assert!((interval.battery_charge_stored_kwh - 2.7).abs() < 1e-12);
        assert_eq!(interval.grid_export_kwh, 0.0);
        assert!((interval.battery_soc_kwh - 2.7).abs() < 1e-12);
    }

    #[test]
    fn charge_power_and_capacity_limits_are_respected() {
        let mut config = battery();
        config.capacity_kwh = 1.0;
        config.max_charge_power_kw = 0.5;
        let result = simulate_microgrid(&[0.0], &[10.0], &[true], 1.0, config).unwrap();
        let interval = &result.intervals[0];
        assert_eq!(interval.battery_charge_input_kwh, 0.5);
        assert!((interval.battery_soc_kwh - 0.45).abs() < 1e-12);
        assert!((interval.grid_export_kwh - 9.5).abs() < 1e-12);
    }

    #[test]
    fn deficit_uses_battery_then_grid() {
        let mut config = battery();
        config.initial_soc_kwh = 4.0;
        let result = simulate_microgrid(&[5.0], &[2.0], &[true], 1.0, config).unwrap();
        let interval = &result.intervals[0];
        assert_eq!(interval.renewable_to_load_kwh, 2.0);
        assert!((interval.battery_discharge_to_load_kwh - 3.0).abs() < 1e-12);
        assert!((interval.battery_discharge_withdrawn_kwh - (3.0 / 0.9)).abs() < 1e-12);
        assert_eq!(interval.grid_import_kwh, 0.0);
        assert_eq!(interval.unserved_load_kwh, 0.0);
        assert!((interval.battery_soc_kwh - (4.0 - 3.0 / 0.9)).abs() < 1e-12);
    }

    #[test]
    fn outage_records_unserved_load_without_grid_import() {
        let mut config = battery();
        config.capacity_kwh = 0.0;
        config.max_charge_power_kw = 0.0;
        config.max_discharge_power_kw = 0.0;
        let result = simulate_microgrid(&[5.0], &[1.0], &[false], 1.0, config).unwrap();
        assert_eq!(result.totals.grid_import_kwh, 0.0);
        assert_eq!(result.totals.unserved_load_kwh, 4.0);
    }

    #[test]
    fn surplus_during_grid_outage_is_curtailed_not_exported() {
        let mut config = battery();
        config.capacity_kwh = 0.0;
        config.max_charge_power_kw = 0.0;
        let result = simulate_microgrid(&[0.0], &[10.0], &[false], 1.0, config).unwrap();
        let interval = &result.intervals[0];
        assert_eq!(interval.grid_export_kwh, 0.0);
        assert_eq!(interval.renewable_curtailed_kwh, 10.0);
        assert_eq!(result.totals.renewable_curtailed_kwh, 10.0);
    }

    #[test]
    fn balances_hold_for_every_interval() {
        let result = simulate_microgrid(
            &[2.0, 6.0, 4.0, 3.0],
            &[7.0, 1.0, 4.0, 0.0],
            &[true, false, true, true],
            1.0,
            battery(),
        )
        .unwrap();

        for interval in &result.intervals {
            let load_supply = interval.renewable_to_load_kwh
                + interval.battery_discharge_to_load_kwh
                + interval.grid_import_kwh
                + interval.unserved_load_kwh;
            let renewable_use = interval.renewable_to_load_kwh
                + interval.battery_charge_input_kwh
                + interval.grid_export_kwh
                + interval.renewable_curtailed_kwh;
            assert!((interval.load_kwh - load_supply).abs() < 1e-10);
            assert!((interval.renewable_generation_kwh - renewable_use).abs() < 1e-10);
        }
    }

    #[test]
    fn rejects_malformed_profiles_and_battery_settings() {
        assert_eq!(
            simulate_microgrid(&[], &[], &[], 1.0, battery()).unwrap_err(),
            MicrogridError::EmptyProfile
        );
        assert_eq!(
            simulate_microgrid(&[1.0], &[], &[true], 1.0, battery()).unwrap_err(),
            MicrogridError::ProfileLengthMismatch
        );
        assert_eq!(
            simulate_microgrid(&[f64::NAN], &[0.0], &[true], 1.0, battery()).unwrap_err(),
            MicrogridError::InvalidEnergyInput
        );
        let mut config = battery();
        config.charge_efficiency = 0.0;
        assert_eq!(
            simulate_microgrid(&[0.0], &[1.0], &[true], 1.0, config).unwrap_err(),
            MicrogridError::InvalidBatteryConfig
        );
    }
}
