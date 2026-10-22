// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Independent post-hoc verifier for finite dispatch schedules.
//!
//! This module does not call the optimizer or the baseline policy. It reconstructs
//! interval balances and storage transitions directly from the request and schedule.
//! It is a consistency check of the declared model, not physical-meter authentication.

use crate::energy_dispatch::{DispatchIntervalDecision, EnergyDispatchRequest};

/// Recalculate hard constraints without trusting the generator's recorded residuals.
pub fn verify_dispatch_schedule(
    request: &EnergyDispatchRequest,
    schedule: &[DispatchIntervalDecision],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let battery = &request.battery;

    if schedule.len() != request.intervals.len() {
        errors.push(format!(
            "schedule length {} does not match interval count {}",
            schedule.len(),
            request.intervals.len()
        ));
    }
    if battery.charge_efficiency_denominator == 0
        || battery.discharge_efficiency_denominator == 0
        || battery.charge_efficiency_numerator == 0
        || battery.discharge_efficiency_numerator == 0
        || battery.charge_efficiency_numerator > battery.charge_efficiency_denominator
        || battery.discharge_efficiency_numerator > battery.discharge_efficiency_denominator
    {
        errors.push("invalid efficiency ratio prevents independent state verification".to_owned());
        return Err(errors);
    }

    let mut expected_soc = battery.initial_soc_quanta;
    let mut expected_start = None::<i64>;
    for (index, (step, input)) in schedule.iter().zip(&request.intervals).enumerate() {
        let label = format!("interval[{index}] ({})", input.interval_id);

        if input.duration_seconds == 0 {
            errors.push(format!("{label}: request duration is zero"));
        }
        if let Some(expected) = expected_start {
            if input.start_unix_seconds != expected {
                errors.push(format!("{label}: request chronology has a gap or overlap"));
            }
        }
        expected_start = match i64::try_from(input.duration_seconds) {
            Ok(duration) => input.start_unix_seconds.checked_add(duration),
            Err(_) => None,
        };
        if expected_start.is_none() {
            errors.push(format!("{label}: request chronology overflows i64 epoch seconds"));
        }

        if step.interval_id != input.interval_id
            || step.start_unix_seconds != input.start_unix_seconds
            || step.duration_seconds != input.duration_seconds
            || step.grid_outage != input.grid_outage
        {
            errors.push(format!("{label}: schedule interval metadata differs from request"));
        }
        if step.pv_available_quanta != input.pv_available_quanta {
            errors.push(format!("{label}: PV availability differs from request"));
        }
        if step.grid_import_quanta
            > if input.grid_outage {
                0
            } else {
                input.grid_import_limit_quanta
            }
        {
            errors.push(format!("{label}: grid import exceeds limit or occurs during outage"));
        }

        if step.battery_soc_before_quanta != expected_soc {
            errors.push(format!("{label}: SOC does not continue from the prior interval"));
        }
        if step.battery_soc_before_quanta > battery.capacity_quanta
            || step.battery_soc_after_quanta > battery.capacity_quanta
        {
            errors.push(format!("{label}: state of charge exceeds battery capacity"));
        }
        if step.battery_soc_withdrawal_quanta > step.battery_soc_before_quanta
            || step.battery_soc_withdrawal_quanta > battery.discharge_withdrawal_limit_quanta
        {
            errors.push(format!("{label}: battery withdrawal exceeds available SOC or limit"));
        }
        if step.battery_charge_input_quanta > battery.charge_input_limit_quanta {
            errors.push(format!("{label}: battery charge input exceeds its limit"));
        }
        if step.battery_charge_input_quanta > 0 && step.battery_soc_withdrawal_quanta > 0 {
            errors.push(format!("{label}: simultaneous charging and discharging is prohibited"));
        }

        let charge_gain = (u64::from(step.battery_charge_input_quanta)
            * u64::from(battery.charge_efficiency_numerator)
            / u64::from(battery.charge_efficiency_denominator)) as u32;
        let soc_after = u32::from(step.battery_soc_before_quanta)
            .checked_sub(u32::from(step.battery_soc_withdrawal_quanta))
            .and_then(|soc| soc.checked_add(charge_gain));
        match soc_after {
            Some(calculated) if calculated == u32::from(step.battery_soc_after_quanta) => {}
            _ => errors.push(format!("{label}: storage state transition does not reconcile")),
        }

        let discharge_output = (u64::from(step.battery_soc_withdrawal_quanta)
            * u64::from(battery.discharge_efficiency_numerator)
            / u64::from(battery.discharge_efficiency_denominator)) as u16;
        if step.battery_discharge_delivered_quanta != discharge_output {
            errors.push(format!("{label}: discharged bus energy does not match efficiency rule"));
        }

        if step.essential_served_quanta > step.essential_demand_quanta
            || step.essential_served_quanta + step.essential_unserved_quanta
                != step.essential_demand_quanta
        {
            errors.push(format!("{label}: essential service does not reconcile to demand"));
        }
        if step.flexible_served_quanta > step.flexible_demand_quanta
            || step.flexible_served_quanta + step.flexible_unserved_quanta
                != step.flexible_demand_quanta
        {
            errors.push(format!("{label}: flexible service does not reconcile to demand"));
        }

        if step.pv_used_quanta > step.pv_available_quanta
            || step.pv_used_quanta + step.pv_curtailed_quanta != step.pv_available_quanta
        {
            errors.push(format!("{label}: PV use and curtailment do not reconcile"));
        }

        let supply = u64::from(step.pv_used_quanta)
            + u64::from(step.grid_import_quanta)
            + u64::from(step.battery_discharge_delivered_quanta);
        let use_energy = u64::from(step.essential_served_quanta)
            + u64::from(step.flexible_served_quanta)
            + u64::from(step.battery_charge_input_quanta);
        let residual = supply as i64 - use_energy as i64;
        if residual != 0 || step.bus_energy_balance_residual_quanta != residual {
            errors.push(format!(
                "{label}: independently calculated bus-balance residual is {residual}"
            ));
        }

        expected_soc = step.battery_soc_after_quanta;
    }

    if let Some(last) = schedule.last() {
        if last.battery_soc_after_quanta < battery.terminal_reserve_quanta {
            errors.push(format!(
                "terminal reserve not met: final SOC {}, required {}",
                last.battery_soc_after_quanta, battery.terminal_reserve_quanta
            ));
        }
    } else {
        errors.push("cannot verify an empty schedule".to_owned());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../schemas/examples/energy-dispatch-pv-outage.synthetic.json");

    fn fixture_request() -> EnergyDispatchRequest {
        serde_json::from_str(FIXTURE).expect("valid frozen request fixture")
    }

    #[test]
    fn verifies_every_oracle_frontier_schedule_independently() {
        let request = fixture_request();
        let assessment = crate::energy_dispatch::assess_energy_dispatch(&request)
            .expect("oracle solves frozen case");
        for alternative in assessment.pareto_alternatives {
            verify_dispatch_schedule(&request, &alternative.schedule)
                .expect("independent verifier accepts oracle output");
        }
    }

    #[test]
    fn rejects_grid_import_during_outage_even_when_recorded_balance_is_zero() {
        let request = fixture_request();
        let assessment = crate::energy_dispatch::assess_energy_dispatch(&request)
            .expect("oracle solves frozen case");
        let mut schedule = assessment.pareto_alternatives[0].schedule.clone();
        schedule[1].grid_import_quanta = 1;
        let errors = verify_dispatch_schedule(&request, &schedule).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("grid import")));
        assert!(errors.iter().any(|error| error.contains("bus-balance residual")));
    }

    #[test]
    fn rejects_forged_zero_residual_when_energy_does_not_balance() {
        let request = fixture_request();
        let assessment = crate::energy_dispatch::assess_energy_dispatch(&request)
            .expect("oracle solves frozen case");
        let mut schedule = assessment.pareto_alternatives[0].schedule.clone();
        schedule[0].bus_energy_balance_residual_quanta = 0;
        schedule[0].pv_used_quanta = 0;
        let errors = verify_dispatch_schedule(&request, &schedule).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("bus-balance residual")));
        assert!(errors.iter().any(|error| error.contains("PV use")));
    }

    #[test]
    fn rejects_broken_soc_continuity_and_terminal_reserve() {
        let request = fixture_request();
        let assessment = crate::energy_dispatch::assess_energy_dispatch(&request)
            .expect("oracle solves frozen case");
        let mut schedule = assessment.pareto_alternatives[0].schedule.clone();
        schedule[0].battery_soc_after_quanta = 0;
        schedule[1].battery_soc_before_quanta = 2;
        schedule[1].battery_soc_withdrawal_quanta = 0;
        schedule[1].battery_discharge_delivered_quanta = 0;
        schedule[1].essential_served_quanta = 0;
        schedule[1].essential_unserved_quanta = 2;
        schedule[1].bus_energy_balance_residual_quanta = 0;
        let errors = verify_dispatch_schedule(&request, &schedule).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("SOC does not continue")));
        assert!(errors.iter().any(|error| error.contains("state transition")));
        assert!(errors.iter().any(|error| error.contains("terminal reserve")));
    }

    #[test]
    fn rejects_wrong_schedule_length() {
        let request = fixture_request();
        let errors = verify_dispatch_schedule(&request, &[]).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("schedule length")));
    }
}
