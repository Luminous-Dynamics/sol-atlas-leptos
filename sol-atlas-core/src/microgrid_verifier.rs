// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Independent consistency checks for deterministic microgrid simulation outputs.
//!
//! The checker validates physical accounting, battery inventory transitions,
//! grid-boundary rules, and aggregate summaries from raw intervals. It does not
//! rerun the dispatch algorithm, prove the meter/source data is genuine, or
//! qualify a model for live-grid control.

use crate::microgrid::{
    BatteryConfig, MicrogridInterval, MicrogridSimulation, MicrogridTotals,
};

/// Fixed absolute-plus-relative tolerance for f64 energy-accounting checks.
const ABS_TOLERANCE_KWH: f64 = 1e-9;
const REL_TOLERANCE: f64 = 1e-9;

/// Results of independently recomputing interval balance residuals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicrogridVerificationReport {
    pub interval_count: usize,
    pub max_load_balance_residual_kwh: f64,
    pub max_generation_balance_residual_kwh: f64,
    pub max_battery_transition_residual_kwh: f64,
    /// Totals recomputed from the interval rows, not copied from the simulator.
    pub recomputed_totals: MicrogridTotals,
}

/// Fail-closed errors returned by the independent result checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicrogridVerificationError {
    EmptyProfile,
    ProfileLengthMismatch,
    IntervalCountMismatch,
    InvalidTimestep,
    InvalidBatteryConfig,
    NonFiniteOrNegativeFlow { interval: usize, field: &'static str },
    InputMismatch { interval: usize, field: &'static str },
    RenewablePriorityViolation { interval: usize },
    LoadBalanceViolation { interval: usize },
    GenerationBalanceViolation { interval: usize },
    BatteryEfficiencyViolation { interval: usize },
    BatteryTransitionViolation { interval: usize },
    BatteryStateOutOfBounds { interval: usize },
    ChargePowerLimitViolation { interval: usize },
    DischargePowerLimitViolation { interval: usize },
    SimultaneousChargeAndDischarge { interval: usize },
    GridBoundaryViolation { interval: usize },
    GridImportExportConflict { interval: usize },
    AggregateMismatch { field: &'static str },
}

impl std::fmt::Display for MicrogridVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyProfile => f.write_str("cannot verify an empty microgrid profile"),
            Self::ProfileLengthMismatch => {
                f.write_str("input profiles must have identical lengths")
            }
            Self::IntervalCountMismatch => {
                f.write_str("simulation interval count does not match the input profiles")
            }
            Self::InvalidTimestep => {
                f.write_str("expected and recorded timesteps must be finite, positive, and equal")
            }
            Self::InvalidBatteryConfig => {
                f.write_str("battery configuration is outside its physical domain")
            }
            Self::NonFiniteOrNegativeFlow { interval, field } => {
                write!(f, "interval {interval} has a non-finite or negative {field}")
            }
            Self::InputMismatch { interval, field } => {
                write!(f, "interval {interval} does not match supplied input {field}")
            }
            Self::RenewablePriorityViolation { interval } => {
                write!(f, "interval {interval} violates renewable-first load service")
            }
            Self::LoadBalanceViolation { interval } => {
                write!(f, "interval {interval} fails load-side energy balance")
            }
            Self::GenerationBalanceViolation { interval } => {
                write!(f, "interval {interval} fails renewable generation accounting")
            }
            Self::BatteryEfficiencyViolation { interval } => {
                write!(f, "interval {interval} violates battery efficiency accounting")
            }
            Self::BatteryTransitionViolation { interval } => {
                write!(f, "interval {interval} has an impossible battery inventory transition")
            }
            Self::BatteryStateOutOfBounds { interval } => {
                write!(f, "interval {interval} battery state is outside configured bounds")
            }
            Self::ChargePowerLimitViolation { interval } => {
                write!(f, "interval {interval} exceeds the battery charge power limit")
            }
            Self::DischargePowerLimitViolation { interval } => {
                write!(f, "interval {interval} exceeds the battery discharge power limit")
            }
            Self::SimultaneousChargeAndDischarge { interval } => {
                write!(f, "interval {interval} charges and discharges simultaneously")
            }
            Self::GridBoundaryViolation { interval } => {
                write!(f, "interval {interval} violates grid availability boundaries")
            }
            Self::GridImportExportConflict { interval } => {
                write!(f, "interval {interval} imports and exports energy simultaneously")
            }
            Self::AggregateMismatch { field } => {
                write!(f, "reported aggregate {field} differs from the recomputed total")
            }
        }
    }
}

impl std::error::Error for MicrogridVerificationError {}

fn tolerance(left: f64, right: f64) -> f64 {
    ABS_TOLERANCE_KWH + REL_TOLERANCE * left.abs().max(right.abs())
}

fn nearly_equal(left: f64, right: f64) -> bool {
    left.is_finite() && right.is_finite() && (left - right).abs() <= tolerance(left, right)
}

fn finite_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn validate_battery(battery: BatteryConfig) -> bool {
    let values = [
        battery.capacity_kwh,
        battery.initial_soc_kwh,
        battery.minimum_soc_kwh,
        battery.max_charge_power_kw,
        battery.max_discharge_power_kw,
        battery.charge_efficiency,
        battery.discharge_efficiency,
    ];
    values.iter().all(|value| value.is_finite())
        && battery.capacity_kwh >= 0.0
        && battery.minimum_soc_kwh >= 0.0
        && battery.initial_soc_kwh >= battery.minimum_soc_kwh
        && battery.initial_soc_kwh <= battery.capacity_kwh
        && battery.max_charge_power_kw >= 0.0
        && battery.max_discharge_power_kw >= 0.0
        && battery.charge_efficiency > 0.0
        && battery.charge_efficiency <= 1.0
        && battery.discharge_efficiency > 0.0
        && battery.discharge_efficiency <= 1.0
}

fn interval_flows(interval: &MicrogridInterval) -> [(&'static str, f64); 12] {
    [
        ("load_kwh", interval.load_kwh),
        ("renewable_generation_kwh", interval.renewable_generation_kwh),
        ("renewable_to_load_kwh", interval.renewable_to_load_kwh),
        ("battery_charge_input_kwh", interval.battery_charge_input_kwh),
        ("battery_charge_stored_kwh", interval.battery_charge_stored_kwh),
        ("battery_discharge_withdrawn_kwh", interval.battery_discharge_withdrawn_kwh),
        ("battery_discharge_to_load_kwh", interval.battery_discharge_to_load_kwh),
        ("grid_import_kwh", interval.grid_import_kwh),
        ("grid_export_kwh", interval.grid_export_kwh),
        ("renewable_curtailed_kwh", interval.renewable_curtailed_kwh),
        ("unserved_load_kwh", interval.unserved_load_kwh),
        ("battery_soc_kwh", interval.battery_soc_kwh),
    ]
}

/// Validate raw simulation rows and independently recompute all aggregates.
///
/// `load_kwh`, `renewable_generation_kwh`, and `grid_available` must be the
/// exact same input vectors used for the simulation. The caller should bind
/// those vectors to a validated EnergyStudyManifest before treating this
/// report as evidence for a particular study.
///
/// This checker intentionally does not call `simulate_microgrid` or reuse the
/// simulator's aggregate totals as its source of truth.
pub fn verify_microgrid_simulation(
    load_kwh: &[f64],
    renewable_generation_kwh: &[f64],
    grid_available: &[bool],
    expected_timestep_hours: f64,
    battery: BatteryConfig,
    simulation: &MicrogridSimulation,
) -> Result<MicrogridVerificationReport, MicrogridVerificationError> {
    if load_kwh.is_empty() {
        return Err(MicrogridVerificationError::EmptyProfile);
    }
    if load_kwh.len() != renewable_generation_kwh.len()
        || load_kwh.len() != grid_available.len()
    {
        return Err(MicrogridVerificationError::ProfileLengthMismatch);
    }
    if load_kwh.len() != simulation.intervals.len() {
        return Err(MicrogridVerificationError::IntervalCountMismatch);
    }
    if !expected_timestep_hours.is_finite()
        || expected_timestep_hours <= 0.0
        || !simulation.timestep_hours.is_finite()
        || simulation.timestep_hours <= 0.0
        || !nearly_equal(expected_timestep_hours, simulation.timestep_hours)
    {
        return Err(MicrogridVerificationError::InvalidTimestep);
    }
    if !validate_battery(battery) {
        return Err(MicrogridVerificationError::InvalidBatteryConfig);
    }
    if load_kwh
        .iter()
        .chain(renewable_generation_kwh.iter())
        .any(|value| !finite_nonnegative(*value))
    {
        return Err(MicrogridVerificationError::InputMismatch {
            interval: 0,
            field: "load or renewable generation",
        });
    }

    let dt = expected_timestep_hours;
    let mut prior_soc_kwh = battery.initial_soc_kwh;
    let mut max_load_residual: f64 = 0.0;
    let mut max_generation_residual: f64 = 0.0;
    let mut max_battery_residual: f64 = 0.0;

    for (index, ((interval, (&expected_load, &expected_generation)), &grid_is_available)) in
        simulation
            .intervals
            .iter()
            .zip(load_kwh.iter().zip(renewable_generation_kwh.iter()))
            .zip(grid_available.iter())
            .enumerate()
    {
        for (field, value) in interval_flows(interval) {
            if !finite_nonnegative(value) {
                return Err(MicrogridVerificationError::NonFiniteOrNegativeFlow {
                    interval: index,
                    field,
                });
            }
        }

        if !nearly_equal(interval.load_kwh, expected_load) {
            return Err(MicrogridVerificationError::InputMismatch {
                interval: index,
                field: "load_kwh",
            });
        }
        if !nearly_equal(interval.renewable_generation_kwh, expected_generation) {
            return Err(MicrogridVerificationError::InputMismatch {
                interval: index,
                field: "renewable_generation_kwh",
            });
        }

        let renewable_direct_expected = expected_load.min(expected_generation);
        if !nearly_equal(interval.renewable_to_load_kwh, renewable_direct_expected) {
            return Err(MicrogridVerificationError::RenewablePriorityViolation {
                interval: index,
            });
        }

        let load_supply = interval.renewable_to_load_kwh
            + interval.battery_discharge_to_load_kwh
            + interval.grid_import_kwh
            + interval.unserved_load_kwh;
        let load_residual = expected_load - load_supply;
        max_load_residual = max_load_residual.max(load_residual.abs());
        if !nearly_equal(expected_load, load_supply) {
            return Err(MicrogridVerificationError::LoadBalanceViolation {
                interval: index,
            });
        }

        let generation_use = interval.renewable_to_load_kwh
            + interval.battery_charge_input_kwh
            + interval.grid_export_kwh
            + interval.renewable_curtailed_kwh;
        let generation_residual = expected_generation - generation_use;
        max_generation_residual = max_generation_residual.max(generation_residual.abs());
        if !nearly_equal(expected_generation, generation_use) {
            return Err(MicrogridVerificationError::GenerationBalanceViolation {
                interval: index,
            });
        }

        if interval.battery_charge_input_kwh > 0.0
            && interval.battery_discharge_to_load_kwh > 0.0
        {
            return Err(MicrogridVerificationError::SimultaneousChargeAndDischarge {
                interval: index,
            });
        }

        if !nearly_equal(
            interval.battery_charge_stored_kwh,
            interval.battery_charge_input_kwh * battery.charge_efficiency,
        ) || !nearly_equal(
            interval.battery_discharge_withdrawn_kwh,
            interval.battery_discharge_to_load_kwh / battery.discharge_efficiency,
        ) {
            return Err(MicrogridVerificationError::BatteryEfficiencyViolation {
                interval: index,
            });
        }

        if interval.battery_charge_input_kwh > battery.max_charge_power_kw * dt
            + tolerance(interval.battery_charge_input_kwh, battery.max_charge_power_kw * dt)
        {
            return Err(MicrogridVerificationError::ChargePowerLimitViolation {
                interval: index,
            });
        }
        if interval.battery_discharge_to_load_kwh > battery.max_discharge_power_kw * dt
            + tolerance(interval.battery_discharge_to_load_kwh, battery.max_discharge_power_kw * dt)
        {
            return Err(MicrogridVerificationError::DischargePowerLimitViolation {
                interval: index,
            });
        }

        let expected_soc_kwh = prior_soc_kwh + interval.battery_charge_stored_kwh
            - interval.battery_discharge_withdrawn_kwh;
        let transition_residual = interval.battery_soc_kwh - expected_soc_kwh;
        max_battery_residual = max_battery_residual.max(transition_residual.abs());
        if !nearly_equal(interval.battery_soc_kwh, expected_soc_kwh) {
            return Err(MicrogridVerificationError::BatteryTransitionViolation {
                interval: index,
            });
        }
        if interval.battery_soc_kwh < battery.minimum_soc_kwh
            - tolerance(interval.battery_soc_kwh, battery.minimum_soc_kwh)
            || interval.battery_soc_kwh > battery.capacity_kwh
                + tolerance(interval.battery_soc_kwh, battery.capacity_kwh)
            || interval.battery_charge_stored_kwh
                > (battery.capacity_kwh - prior_soc_kwh).max(0.0)
                    + tolerance(interval.battery_charge_stored_kwh, battery.capacity_kwh)
            || interval.battery_discharge_to_load_kwh
                > (prior_soc_kwh - battery.minimum_soc_kwh).max(0.0)
                    * battery.discharge_efficiency
                    + tolerance(interval.battery_discharge_to_load_kwh, prior_soc_kwh)
        {
            return Err(MicrogridVerificationError::BatteryStateOutOfBounds {
                interval: index,
            });
        }

        if grid_is_available {
            if interval.unserved_load_kwh > tolerance(interval.unserved_load_kwh, 0.0)
                || interval.renewable_curtailed_kwh
                    > tolerance(interval.renewable_curtailed_kwh, 0.0)
            {
                return Err(MicrogridVerificationError::GridBoundaryViolation {
                    interval: index,
                });
            }
            if interval.grid_import_kwh > 0.0 && interval.grid_export_kwh > 0.0 {
                return Err(MicrogridVerificationError::GridImportExportConflict {
                    interval: index,
                });
            }
        } else if interval.grid_import_kwh > tolerance(interval.grid_import_kwh, 0.0)
            || interval.grid_export_kwh > tolerance(interval.grid_export_kwh, 0.0)
        {
            return Err(MicrogridVerificationError::GridBoundaryViolation {
                interval: index,
            });
        }

        prior_soc_kwh = interval.battery_soc_kwh;
    }

    let sum = |select: fn(&MicrogridInterval) -> f64| -> f64 {
        simulation.intervals.iter().map(select).sum()
    };
    let recomputed = MicrogridTotals {
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
        final_battery_soc_kwh: prior_soc_kwh,
    };

    compare_aggregate("load_kwh", simulation.totals.load_kwh, recomputed.load_kwh)?;
    compare_aggregate(
        "renewable_generation_kwh",
        simulation.totals.renewable_generation_kwh,
        recomputed.renewable_generation_kwh,
    )?;
    compare_aggregate(
        "renewable_to_load_kwh",
        simulation.totals.renewable_to_load_kwh,
        recomputed.renewable_to_load_kwh,
    )?;
    compare_aggregate(
        "battery_charge_input_kwh",
        simulation.totals.battery_charge_input_kwh,
        recomputed.battery_charge_input_kwh,
    )?;
    compare_aggregate(
        "battery_charge_stored_kwh",
        simulation.totals.battery_charge_stored_kwh,
        recomputed.battery_charge_stored_kwh,
    )?;
    compare_aggregate(
        "battery_discharge_withdrawn_kwh",
        simulation.totals.battery_discharge_withdrawn_kwh,
        recomputed.battery_discharge_withdrawn_kwh,
    )?;
    compare_aggregate(
        "battery_discharge_to_load_kwh",
        simulation.totals.battery_discharge_to_load_kwh,
        recomputed.battery_discharge_to_load_kwh,
    )?;
    compare_aggregate("grid_import_kwh", simulation.totals.grid_import_kwh, recomputed.grid_import_kwh)?;
    compare_aggregate("grid_export_kwh", simulation.totals.grid_export_kwh, recomputed.grid_export_kwh)?;
    compare_aggregate(
        "renewable_curtailed_kwh",
        simulation.totals.renewable_curtailed_kwh,
        recomputed.renewable_curtailed_kwh,
    )?;
    compare_aggregate(
        "unserved_load_kwh",
        simulation.totals.unserved_load_kwh,
        recomputed.unserved_load_kwh,
    )?;
    compare_aggregate(
        "battery_energy_losses_kwh",
        simulation.totals.battery_energy_losses_kwh,
        recomputed.battery_energy_losses_kwh,
    )?;
    compare_aggregate(
        "final_battery_soc_kwh",
        simulation.totals.final_battery_soc_kwh,
        recomputed.final_battery_soc_kwh,
    )?;

    Ok(MicrogridVerificationReport {
        interval_count: simulation.intervals.len(),
        max_load_balance_residual_kwh: max_load_residual,
        max_generation_balance_residual_kwh: max_generation_residual,
        max_battery_transition_residual_kwh: max_battery_residual,
        recomputed_totals: recomputed,
    })
}

fn compare_aggregate(
    field: &'static str,
    reported: f64,
    recomputed: f64,
) -> Result<(), MicrogridVerificationError> {
    if !nearly_equal(reported, recomputed) {
        return Err(MicrogridVerificationError::AggregateMismatch { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microgrid::simulate_microgrid;

    fn battery() -> BatteryConfig {
        BatteryConfig {
            capacity_kwh: 8.0,
            initial_soc_kwh: 4.0,
            minimum_soc_kwh: 1.0,
            max_charge_power_kw: 3.0,
            max_discharge_power_kw: 3.0,
            charge_efficiency: 0.9,
            discharge_efficiency: 0.9,
        }
    }

    fn valid_simulation() -> (
        Vec<f64>,
        Vec<f64>,
        Vec<bool>,
        f64,
        BatteryConfig,
        MicrogridSimulation,
    ) {
        let load = vec![2.0, 5.0, 3.0, 4.0];
        let generation = vec![5.0, 1.0, 1.0, 0.0];
        let grid = vec![true, false, true, true];
        let dt = 1.0;
        let config = battery();
        let simulation =
            simulate_microgrid(&load, &generation, &grid, dt, config).unwrap();
        (load, generation, grid, dt, config, simulation)
    }

    #[test]
    fn accepts_valid_rows_and_recomputes_totals() {
        let (load, generation, grid, dt, config, simulation) = valid_simulation();
        let report =
            verify_microgrid_simulation(&load, &generation, &grid, dt, config, &simulation)
                .unwrap();
        assert_eq!(report.interval_count, 4);
        assert!(report.max_load_balance_residual_kwh <= 1e-9);
        assert!(report.max_generation_balance_residual_kwh <= 1e-9);
        assert!(report.max_battery_transition_residual_kwh <= 1e-9);
        assert!(nearly_equal(
            report.recomputed_totals.load_kwh,
            load.iter().sum()
        ));
        assert!(nearly_equal(
            report.recomputed_totals.final_battery_soc_kwh,
            simulation.intervals.last().unwrap().battery_soc_kwh
        ));
    }

    #[test]
    fn rejects_forged_summary_even_when_interval_rows_are_unchanged() {
        let (load, generation, grid, dt, config, mut simulation) = valid_simulation();
        simulation.totals.unserved_load_kwh += 0.5;
        assert_eq!(
            verify_microgrid_simulation(&load, &generation, &grid, dt, config, &simulation)
                .unwrap_err(),
            MicrogridVerificationError::AggregateMismatch {
                field: "unserved_load_kwh"
            }
        );
    }

    #[test]
    fn rejects_grid_import_forged_during_outage_even_if_energy_balances() {
        let mut config = battery();
        config.capacity_kwh = 0.0;
        config.initial_soc_kwh = 0.0;
        config.minimum_soc_kwh = 0.0;
        config.max_charge_power_kw = 0.0;
        config.max_discharge_power_kw = 0.0;
        let load = [5.0];
        let generation = [1.0];
        let grid = [false];
        let mut simulation =
            simulate_microgrid(&load, &generation, &grid, 1.0, config).unwrap();
        // Preserve load accounting while forging unavailable grid supply.
        simulation.intervals[0].grid_import_kwh = 4.0;
        simulation.intervals[0].unserved_load_kwh = 0.0;
        simulation.totals.grid_import_kwh = 4.0;
        simulation.totals.unserved_load_kwh = 0.0;
        assert_eq!(
            verify_microgrid_simulation(&load, &generation, &grid, 1.0, config, &simulation)
                .unwrap_err(),
            MicrogridVerificationError::GridBoundaryViolation { interval: 0 }
        );
    }

    #[test]
    fn rejects_impossible_battery_state_transition() {
        let (load, generation, grid, dt, config, mut simulation) = valid_simulation();
        simulation.intervals[0].battery_soc_kwh += 0.25;
        assert_eq!(
            verify_microgrid_simulation(&load, &generation, &grid, dt, config, &simulation)
                .unwrap_err(),
            MicrogridVerificationError::BatteryTransitionViolation { interval: 0 }
        );
    }

    #[test]
    fn rejects_unreported_input_change() {
        let (load, generation, grid, dt, config, mut simulation) = valid_simulation();
        simulation.intervals[1].renewable_generation_kwh += 0.25;
        assert_eq!(
            verify_microgrid_simulation(&load, &generation, &grid, dt, config, &simulation)
                .unwrap_err(),
            MicrogridVerificationError::InputMismatch {
                interval: 1,
                field: "renewable_generation_kwh"
            }
        );
    }

    #[test]
    fn rejects_non_finite_or_out_of_domain_configuration() {
        let (load, generation, grid, dt, config, simulation) = valid_simulation();
        let mut bad_config = config;
        bad_config.charge_efficiency = f64::NAN;
        assert_eq!(
            verify_microgrid_simulation(&load, &generation, &grid, dt, bad_config, &simulation)
                .unwrap_err(),
            MicrogridVerificationError::InvalidBatteryConfig
        );
        let mut bad_result = simulation.clone();
        bad_result.intervals[0].grid_import_kwh = f64::INFINITY;
        assert_eq!(
            verify_microgrid_simulation(&load, &generation, &grid, dt, config, &bad_result)
                .unwrap_err(),
            MicrogridVerificationError::NonFiniteOrNegativeFlow {
                interval: 0,
                field: "grid_import_kwh"
            }
        );
    }
}
