// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Deterministic, scenario-only civic-energy demonstration.
//!
//! This module is a small end-to-end proof slice for the Network State demo.
//! It models energy accounting and critical-load allocation; it is not a
//! forecast, cost optimizer, market, meter attestation, or grid controller.
//! The verifier recomputes balances from interval rows without rerunning the
//! dispatch function. It lives in the same crate, so it is not an external or
//! organizationally independent verifier.

const EPS: f64 = 1e-8;
pub const INTERVAL_HOURS: f64 = 0.25;
pub const MAX_CRITICAL_UNSERVED_KWH: f64 = 0.0;
pub const CASE_ID: &str = "community-energy-cloud-evening-outage-v1";
pub const EVIDENCE_CLASS: &str = "synthetic scenario; no live meter observations";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyIntervalInput {
    pub interval: u32,
    pub load_kwh: f64,
    pub critical_load_kwh: f64,
    pub renewable_generation_kwh: f64,
    pub grid_available: bool,
}

pub const SCENARIO_INPUTS: [EnergyIntervalInput; 8] = [
    EnergyIntervalInput {
        interval: 0,
        load_kwh: 2.0,
        critical_load_kwh: 1.0,
        renewable_generation_kwh: 2.5,
        grid_available: true,
    },
    EnergyIntervalInput {
        interval: 1,
        load_kwh: 2.5,
        critical_load_kwh: 1.0,
        renewable_generation_kwh: 1.2,
        grid_available: true,
    },
    EnergyIntervalInput {
        interval: 2,
        load_kwh: 3.0,
        critical_load_kwh: 1.2,
        renewable_generation_kwh: 0.9,
        grid_available: true,
    },
    EnergyIntervalInput {
        interval: 3,
        load_kwh: 4.0,
        critical_load_kwh: 1.5,
        renewable_generation_kwh: 1.0,
        grid_available: true,
    },
    EnergyIntervalInput {
        interval: 4,
        load_kwh: 5.0,
        critical_load_kwh: 1.5,
        renewable_generation_kwh: 0.5,
        grid_available: false,
    },
    EnergyIntervalInput {
        interval: 5,
        load_kwh: 5.0,
        critical_load_kwh: 1.5,
        renewable_generation_kwh: 0.0,
        grid_available: false,
    },
    EnergyIntervalInput {
        interval: 6,
        load_kwh: 3.5,
        critical_load_kwh: 1.2,
        renewable_generation_kwh: 0.0,
        grid_available: true,
    },
    EnergyIntervalInput {
        interval: 7,
        load_kwh: 3.0,
        critical_load_kwh: 1.0,
        renewable_generation_kwh: 0.0,
        grid_available: true,
    },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatteryConfiguration {
    pub name: &'static str,
    pub capacity_kwh: f64,
    pub initial_soc_kwh: f64,
    pub minimum_soc_kwh: f64,
    pub max_charge_power_kw: f64,
    pub max_discharge_power_kw: f64,
    pub charge_efficiency: f64,
    pub discharge_efficiency: f64,
}

impl BatteryConfiguration {
    pub const BASELINE: Self = Self {
        name: "Baseline equipment",
        capacity_kwh: 6.0,
        initial_soc_kwh: 3.0,
        minimum_soc_kwh: 1.0,
        max_charge_power_kw: 2.0,
        max_discharge_power_kw: 2.0,
        charge_efficiency: 0.92,
        discharge_efficiency: 0.92,
    };

    /// A deliberately oversized sensitivity candidate, not a costed proposal.
    pub const RESILIENCE_CANDIDATE: Self = Self {
        name: "Resilience sensitivity",
        capacity_kwh: 20.0,
        initial_soc_kwh: 16.0,
        minimum_soc_kwh: 1.0,
        max_charge_power_kw: 8.0,
        max_discharge_power_kw: 8.0,
        charge_efficiency: 0.92,
        discharge_efficiency: 0.92,
    };

    fn validate(self) -> Result<(), String> {
        let values = [
            self.capacity_kwh,
            self.initial_soc_kwh,
            self.minimum_soc_kwh,
            self.max_charge_power_kw,
            self.max_discharge_power_kw,
            self.charge_efficiency,
            self.discharge_efficiency,
        ];
        if values.iter().any(|v| !v.is_finite())
            || self.minimum_soc_kwh < 0.0
            || self.initial_soc_kwh < self.minimum_soc_kwh
            || self.initial_soc_kwh > self.capacity_kwh
            || self.max_charge_power_kw < 0.0
            || self.max_discharge_power_kw < 0.0
            || !(0.0 < self.charge_efficiency && self.charge_efficiency <= 1.0)
            || !(0.0 < self.discharge_efficiency && self.discharge_efficiency <= 1.0)
        {
            return Err("invalid battery configuration".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntervalResult {
    pub input: EnergyIntervalInput,
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
    pub critical_served_kwh: f64,
    pub critical_unserved_kwh: f64,
    pub noncritical_served_kwh: f64,
    pub noncritical_unserved_kwh: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimulationTotals {
    pub total_load_kwh: f64,
    pub total_renewable_generation_kwh: f64,
    pub total_grid_import_kwh: f64,
    pub total_grid_export_kwh: f64,
    pub total_renewable_curtailed_kwh: f64,
    pub total_unserved_load_kwh: f64,
    pub total_critical_load_kwh: f64,
    pub total_critical_unserved_kwh: f64,
    pub total_noncritical_unserved_kwh: f64,
    pub final_battery_soc_kwh: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimulationResult {
    pub case_id: &'static str,
    pub configuration: BatteryConfiguration,
    pub intervals: Vec<IntervalResult>,
    pub totals: SimulationTotals,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerificationReport {
    pub interval_count: usize,
    pub max_load_balance_residual_kwh: f64,
    pub max_generation_balance_residual_kwh: f64,
    pub max_battery_state_residual_kwh: f64,
    pub accounting_consistent: bool,
    pub critical_service_limit_satisfied: bool,
}

pub fn simulate(configuration: BatteryConfiguration) -> Result<SimulationResult, String> {
    configuration.validate()?;
    let mut state_of_charge = configuration.initial_soc_kwh;
    let mut intervals = Vec::with_capacity(SCENARIO_INPUTS.len());

    for input in SCENARIO_INPUTS {
        if !input.load_kwh.is_finite()
            || !input.critical_load_kwh.is_finite()
            || !input.renewable_generation_kwh.is_finite()
            || input.load_kwh < 0.0
            || input.critical_load_kwh < 0.0
            || input.critical_load_kwh > input.load_kwh
            || input.renewable_generation_kwh < 0.0
        {
            return Err(format!("invalid scenario input at interval {}", input.interval));
        }

        let renewable_to_load = input.load_kwh.min(input.renewable_generation_kwh);
        let surplus = (input.renewable_generation_kwh - renewable_to_load).max(0.0);
        let deficit = (input.load_kwh - renewable_to_load).max(0.0);
        let mut charge_input = 0.0;
        let mut charge_stored = 0.0;
        let mut discharge_withdrawn = 0.0;
        let mut discharge_to_load = 0.0;
        let mut grid_import = 0.0;
        let mut grid_export = 0.0;
        let mut curtailed = 0.0;
        let mut unserved = 0.0;

        if deficit > EPS {
            let available_withdrawal = (state_of_charge - configuration.minimum_soc_kwh)
                .max(0.0)
                .min(configuration.max_discharge_power_kw * INTERVAL_HOURS);
            let available_delivery = available_withdrawal * configuration.discharge_efficiency;
            discharge_to_load = deficit.min(available_delivery);
            discharge_withdrawn = discharge_to_load / configuration.discharge_efficiency;
            state_of_charge -= discharge_withdrawn;
            let remaining = (deficit - discharge_to_load).max(0.0);
            if input.grid_available {
                grid_import = remaining;
            } else {
                unserved = remaining;
            }
        } else {
            let charge_room_input = (configuration.capacity_kwh - state_of_charge).max(0.0)
                / configuration.charge_efficiency;
            charge_input = surplus
                .min(configuration.max_charge_power_kw * INTERVAL_HOURS)
                .min(charge_room_input);
            charge_stored = charge_input * configuration.charge_efficiency;
            state_of_charge += charge_stored;
            let remaining_surplus = (surplus - charge_input).max(0.0);
            if input.grid_available {
                grid_export = remaining_surplus;
            } else {
                curtailed = remaining_surplus;
            }
        }

        let total_served = (input.load_kwh - unserved).max(0.0);
        let critical_served = input.critical_load_kwh.min(total_served);
        let critical_unserved = (input.critical_load_kwh - critical_served).max(0.0);
        let noncritical_load = input.load_kwh - input.critical_load_kwh;
        let noncritical_served = (total_served - critical_served).max(0.0).min(noncritical_load);
        let noncritical_unserved = (noncritical_load - noncritical_served).max(0.0);

        intervals.push(IntervalResult {
            input,
            renewable_to_load_kwh: renewable_to_load,
            battery_charge_input_kwh: charge_input,
            battery_charge_stored_kwh: charge_stored,
            battery_discharge_withdrawn_kwh: discharge_withdrawn,
            battery_discharge_to_load_kwh: discharge_to_load,
            grid_import_kwh: grid_import,
            grid_export_kwh: grid_export,
            renewable_curtailed_kwh: curtailed,
            unserved_load_kwh: unserved,
            battery_soc_kwh: state_of_charge,
            critical_served_kwh: critical_served,
            critical_unserved_kwh: critical_unserved,
            noncritical_served_kwh: noncritical_served,
            noncritical_unserved_kwh: noncritical_unserved,
        });
    }

    let totals = aggregate_totals(&intervals, state_of_charge);
    Ok(SimulationResult { case_id: CASE_ID, configuration, intervals, totals })
}

/// Independently recompute physical/service accounting from row outputs.
/// This function does not call or replay simulate().
pub fn verify(
    configuration: BatteryConfiguration,
    expected_inputs: &[EnergyIntervalInput],
    candidate: &SimulationResult,
) -> Result<VerificationReport, String> {
    configuration.validate()?;
    if candidate.case_id != CASE_ID || candidate.configuration != configuration {
        return Err("candidate identity/configuration mismatch".into());
    }
    if candidate.intervals.len() != expected_inputs.len()
        || expected_inputs.len() != SCENARIO_INPUTS.len()
    {
        return Err("interval count differs from the frozen scenario".into());
    }

    let mut prior_soc = configuration.initial_soc_kwh;
    let mut max_load_residual: f64 = 0.0;
    let mut max_generation_residual: f64 = 0.0;
    let mut max_soc_residual: f64 = 0.0;
    let mut critical_unserved = 0.0;
    let mut noncritical_unserved = 0.0;
    for (position, (input, row)) in expected_inputs.iter().zip(&candidate.intervals).enumerate() {
        if *input != SCENARIO_INPUTS[position] || row.input != *input {
            return Err(format!("input binding mismatch at interval {}", position));
        }
        let values = [
            row.renewable_to_load_kwh,
            row.battery_charge_input_kwh,
            row.battery_charge_stored_kwh,
            row.battery_discharge_withdrawn_kwh,
            row.battery_discharge_to_load_kwh,
            row.grid_import_kwh,
            row.grid_export_kwh,
            row.renewable_curtailed_kwh,
            row.unserved_load_kwh,
            row.battery_soc_kwh,
            row.critical_served_kwh,
            row.critical_unserved_kwh,
            row.noncritical_served_kwh,
            row.noncritical_unserved_kwh,
        ];
        if values.iter().any(|v| !v.is_finite() || *v < -EPS) {
            return Err(format!("invalid flow value at interval {}", position));
        }
        if row.battery_charge_input_kwh > EPS && row.battery_discharge_withdrawn_kwh > EPS {
            return Err(format!("simultaneous charge and discharge at interval {}", position));
        }
        let input = *input;
        let renewable_to_load_expected = input.load_kwh.min(input.renewable_generation_kwh);
        if !near(row.renewable_to_load_kwh, renewable_to_load_expected)
            || row.renewable_to_load_kwh > input.load_kwh + EPS
            || row.renewable_to_load_kwh > input.renewable_generation_kwh + EPS
        {
            return Err(format!("renewable-to-load allocation mismatch at interval {}", position));
        }
        if row.battery_charge_input_kwh > configuration.max_charge_power_kw * INTERVAL_HOURS + EPS
            || row.battery_discharge_withdrawn_kwh
                > configuration.max_discharge_power_kw * INTERVAL_HOURS + EPS
        {
            return Err(format!("battery power limit exceeded at interval {}", position));
        }
        let input_surplus = (input.renewable_generation_kwh - input.load_kwh).max(0.0);
        let input_deficit = (input.load_kwh - input.renewable_generation_kwh).max(0.0);
        if row.battery_charge_input_kwh > input_surplus + EPS
            || row.grid_export_kwh > input_surplus + EPS
            || row.battery_discharge_to_load_kwh > input_deficit + EPS
        {
            return Err(format!(
                "flow direction exceeds available surplus/deficit at interval {}",
                position
            ));
        }
        if input.grid_available {
            if row.grid_import_kwh > input_deficit + EPS
                || row.grid_export_kwh > EPS && input_deficit > EPS
            {
                return Err(format!("grid flow direction mismatch at interval {}", position));
            }
        } else if row.grid_import_kwh > EPS || row.grid_export_kwh > EPS {
            return Err(format!("grid flow recorded during outage at interval {}", position));
        }
        if !input.grid_available && (row.grid_import_kwh > EPS || row.grid_export_kwh > EPS) {
            return Err(format!("grid flow recorded during outage at interval {}", position));
        }
        if row.battery_soc_kwh < configuration.minimum_soc_kwh - EPS
            || row.battery_soc_kwh > configuration.capacity_kwh + EPS
        {
            return Err(format!("battery state outside bounds at interval {}", position));
        }
        if !near(
            row.battery_charge_stored_kwh,
            row.battery_charge_input_kwh * configuration.charge_efficiency,
        ) || !near(
            row.battery_discharge_to_load_kwh,
            row.battery_discharge_withdrawn_kwh * configuration.discharge_efficiency,
        ) {
            return Err(format!("battery efficiency accounting mismatch at interval {}", position));
        }

        let load_left = row.renewable_to_load_kwh
            + row.battery_discharge_to_load_kwh
            + row.grid_import_kwh
            + row.unserved_load_kwh;
        let generation_right = row.renewable_to_load_kwh
            + row.battery_charge_input_kwh
            + row.grid_export_kwh
            + row.renewable_curtailed_kwh;
        let load_residual = (input.load_kwh - load_left).abs();
        let generation_residual = (input.renewable_generation_kwh - generation_right).abs();
        let soc_residual = (prior_soc + row.battery_charge_stored_kwh
            - row.battery_discharge_withdrawn_kwh - row.battery_soc_kwh).abs();
        max_load_residual = max_load_residual.max(load_residual);
        max_generation_residual = max_generation_residual.max(generation_residual);
        max_soc_residual = max_soc_residual.max(soc_residual);
        if load_residual > EPS || generation_residual > EPS || soc_residual > EPS {
            return Err(format!("physical balance failed at interval {}", position));
        }

        let served = input.load_kwh - row.unserved_load_kwh;
        let expected_critical_served = input.critical_load_kwh.min(served);
        let expected_critical_unserved =
            (input.critical_load_kwh - expected_critical_served).max(0.0);
        let noncritical_load = input.load_kwh - input.critical_load_kwh;
        let expected_noncritical_served =
            (served - expected_critical_served).max(0.0).min(noncritical_load);
        let expected_noncritical_unserved =
            (noncritical_load - expected_noncritical_served).max(0.0);
        if !near(row.critical_served_kwh, expected_critical_served)
            || !near(row.critical_unserved_kwh, expected_critical_unserved)
            || !near(row.noncritical_served_kwh, expected_noncritical_served)
            || !near(row.noncritical_unserved_kwh, expected_noncritical_unserved)
        {
            return Err(format!("service allocation mismatch at interval {}", position));
        }
        critical_unserved += expected_critical_unserved;
        noncritical_unserved += expected_noncritical_unserved;
        prior_soc = row.battery_soc_kwh;
    }

    // Recompute the aggregate contract separately from the simulator helper.
    // Sharing the simulator's aggregation function here would allow a bug in
    // that helper to validate its own output.
    let recomputed = SimulationTotals {
        total_load_kwh: expected_inputs.iter().map(|row| row.load_kwh).sum(),
        total_renewable_generation_kwh: expected_inputs
            .iter()
            .map(|row| row.renewable_generation_kwh)
            .sum(),
        total_grid_import_kwh: candidate.intervals.iter().map(|row| row.grid_import_kwh).sum(),
        total_grid_export_kwh: candidate.intervals.iter().map(|row| row.grid_export_kwh).sum(),
        total_renewable_curtailed_kwh: candidate
            .intervals
            .iter()
            .map(|row| row.renewable_curtailed_kwh)
            .sum(),
        total_unserved_load_kwh: candidate
            .intervals
            .iter()
            .map(|row| row.unserved_load_kwh)
            .sum(),
        total_critical_load_kwh: expected_inputs.iter().map(|row| row.critical_load_kwh).sum(),
        total_critical_unserved_kwh: critical_unserved,
        total_noncritical_unserved_kwh: noncritical_unserved,
        final_battery_soc_kwh: prior_soc,
    };
    if !totals_near(&recomputed, &candidate.totals) {
        return Err("reported aggregate totals differ from recomputed interval totals".into());
    }

    Ok(VerificationReport {
        interval_count: expected_inputs.len(),
        max_load_balance_residual_kwh: max_load_residual,
        max_generation_balance_residual_kwh: max_generation_residual,
        max_battery_state_residual_kwh: max_soc_residual,
        accounting_consistent: true,
        critical_service_limit_satisfied: critical_unserved <= MAX_CRITICAL_UNSERVED_KWH + EPS,
    })
}

pub fn tampered_aggregate_rejected(
    configuration: BatteryConfiguration,
    candidate: &SimulationResult,
) -> bool {
    let mut tampered = candidate.clone();
    tampered.totals.total_grid_import_kwh += 1.0;
    verify(configuration, &SCENARIO_INPUTS, &tampered).is_err()
}

fn aggregate_totals(rows: &[IntervalResult], final_soc: f64) -> SimulationTotals {
    let mut totals = SimulationTotals {
        total_load_kwh: 0.0,
        total_renewable_generation_kwh: 0.0,
        total_grid_import_kwh: 0.0,
        total_grid_export_kwh: 0.0,
        total_renewable_curtailed_kwh: 0.0,
        total_unserved_load_kwh: 0.0,
        total_critical_load_kwh: 0.0,
        total_critical_unserved_kwh: 0.0,
        total_noncritical_unserved_kwh: 0.0,
        final_battery_soc_kwh: final_soc,
    };
    for row in rows {
        totals.total_load_kwh += row.input.load_kwh;
        totals.total_renewable_generation_kwh += row.input.renewable_generation_kwh;
        totals.total_grid_import_kwh += row.grid_import_kwh;
        totals.total_grid_export_kwh += row.grid_export_kwh;
        totals.total_renewable_curtailed_kwh += row.renewable_curtailed_kwh;
        totals.total_unserved_load_kwh += row.unserved_load_kwh;
        totals.total_critical_load_kwh += row.input.critical_load_kwh;
        totals.total_critical_unserved_kwh += row.critical_unserved_kwh;
        totals.total_noncritical_unserved_kwh += row.noncritical_unserved_kwh;
    }
    totals
}

fn totals_near(a: &SimulationTotals, b: &SimulationTotals) -> bool {
    near(a.total_load_kwh, b.total_load_kwh)
        && near(a.total_renewable_generation_kwh, b.total_renewable_generation_kwh)
        && near(a.total_grid_import_kwh, b.total_grid_import_kwh)
        && near(a.total_grid_export_kwh, b.total_grid_export_kwh)
        && near(a.total_renewable_curtailed_kwh, b.total_renewable_curtailed_kwh)
        && near(a.total_unserved_load_kwh, b.total_unserved_load_kwh)
        && near(a.total_critical_load_kwh, b.total_critical_load_kwh)
        && near(a.total_critical_unserved_kwh, b.total_critical_unserved_kwh)
        && near(a.total_noncritical_unserved_kwh, b.total_noncritical_unserved_kwh)
        && near(a.final_battery_soc_kwh, b.final_battery_soc_kwh)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= EPS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernanceVoteFixture {
    pub eligible_members: u8,
    pub yes: u8,
    pub no: u8,
    pub abstain: u8,
    pub quorum_percent: u8,
    pub approval_percent: u8,
}

impl GovernanceVoteFixture {
    pub const NETWORK_STATE_DEMO: Self = Self {
        eligible_members: 5,
        yes: 4,
        no: 1,
        abstain: 0,
        quorum_percent: 25,
        approval_percent: 60,
    };

    pub fn cast_ballots(self) -> u8 {
        self.yes.saturating_add(self.no).saturating_add(self.abstain)
    }

    pub fn quorum_met(self) -> bool {
        self.eligible_members > 0
            && u16::from(self.cast_ballots()) * 100
                >= u16::from(self.eligible_members) * u16::from(self.quorum_percent)
    }

    pub fn approval_met(self) -> bool {
        let decisive = self.yes.saturating_add(self.no);
        decisive > 0
            && u16::from(self.yes) * 100 >= u16::from(decisive) * u16::from(self.approval_percent)
    }

    pub fn passed(self) -> bool {
        self.quorum_met() && self.approval_met()
    }

    pub fn approval_rate_percent(self) -> u8 {
        let decisive = self.yes.saturating_add(self.no);
        if decisive == 0 {
            0
        } else {
            ((u16::from(self.yes) * 100) / u16::from(decisive)) as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_is_accounting_valid_but_fails_critical_service_bound() {
        let run = simulate(BatteryConfiguration::BASELINE).unwrap();
        let report = verify(BatteryConfiguration::BASELINE, &SCENARIO_INPUTS, &run).unwrap();
        assert!(report.accounting_consistent);
        assert!(!report.critical_service_limit_satisfied);
        assert!(run.totals.total_critical_unserved_kwh > 0.0);
    }

    #[test]
    fn resilience_sensitivity_passes_critical_service_bound_but_sheds_noncritical_load() {
        let run = simulate(BatteryConfiguration::RESILIENCE_CANDIDATE).unwrap();
        let report =
            verify(BatteryConfiguration::RESILIENCE_CANDIDATE, &SCENARIO_INPUTS, &run).unwrap();
        assert!(report.accounting_consistent);
        assert!(report.critical_service_limit_satisfied);
        assert!(run.totals.total_noncritical_unserved_kwh > 0.0);
    }

    #[test]
    fn independent_verifier_rejects_forged_aggregate_total() {
        let run = simulate(BatteryConfiguration::RESILIENCE_CANDIDATE).unwrap();
        assert!(tampered_aggregate_rejected(BatteryConfiguration::RESILIENCE_CANDIDATE, &run));
    }

    #[test]
    fn outage_never_imports_or_exports_grid_energy() {
        let run = simulate(BatteryConfiguration::RESILIENCE_CANDIDATE).unwrap();
        for row in run.intervals.iter().filter(|r| !r.input.grid_available) {
            assert_eq!(row.grid_import_kwh, 0.0);
            assert_eq!(row.grid_export_kwh, 0.0);
        }
    }

    #[test]
    fn verifier_rejects_grid_import_during_a_frozen_outage() {
        let mut run = simulate(BatteryConfiguration::RESILIENCE_CANDIDATE).unwrap();
        run.intervals[4].grid_import_kwh = 0.25;
        assert!(verify(
            BatteryConfiguration::RESILIENCE_CANDIDATE,
            &SCENARIO_INPUTS,
            &run
        )
        .is_err());
    }

    #[test]
    fn verifier_rejects_charge_power_limit_violation() {
        let mut run = simulate(BatteryConfiguration::RESILIENCE_CANDIDATE).unwrap();
        run.intervals[0].battery_charge_input_kwh = 2.01;
        assert!(verify(
            BatteryConfiguration::RESILIENCE_CANDIDATE,
            &SCENARIO_INPUTS,
            &run
        )
        .is_err());
    }

    #[test]
    fn frozen_scenario_known_answer_preserves_the_tradeoff() {
        let baseline = simulate(BatteryConfiguration::BASELINE).unwrap();
        let resilience = simulate(BatteryConfiguration::RESILIENCE_CANDIDATE).unwrap();
        assert!((baseline.totals.total_critical_unserved_kwh - 1.6168).abs() < 1e-8);
        assert!(resilience.totals.total_critical_unserved_kwh.abs() < 1e-8);
        assert!((resilience.totals.total_noncritical_unserved_kwh - 5.82).abs() < 1e-8);
    }

    #[test]
    fn governance_fixture_preserves_dissent_and_passes_its_declared_rule() {
        let vote = GovernanceVoteFixture::NETWORK_STATE_DEMO;
        assert_eq!(vote.yes, 4);
        assert_eq!(vote.no, 1);
        assert!(vote.passed());
        assert_eq!(vote.approval_rate_percent(), 80);
    }
}
