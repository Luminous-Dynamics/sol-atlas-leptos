// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Transparent heuristic baseline and objective-vector regret against the exact oracle.
//!
//! Policy v0.1 is deliberately simple: direct PV first, then grid for loads, then
//! storage discharge for remaining demand; charge from PV surplus only. It is not
//! advertised as SOTA. Its role is to establish a reproducible baseline whose
//! service-quality, cost and carbon trade-offs can be measured against exact small cases.

use crate::energy_dispatch::{
    DispatchIntervalDecision, DispatchObjective, EnergyDispatchAssessment, EnergyDispatchError,
    EnergyDispatchRequest, assess_energy_dispatch, objective_for_schedule,
};
use serde::Serialize;

pub const BASELINE_POLICY_VERSION: &str = "pv-grid-battery-priority-v0.1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchObjectiveDeltas {
    /// Baseline minus lexicographic oracle; positive means more essential energy unserved.
    pub essential_unserved_quanta: i128,
    pub flexible_unserved_quanta: i128,
    pub operating_cost_units: i128,
    pub grid_carbon_grams: i128,
    pub curtailed_pv_quanta: i128,
    /// Oracle terminal SOC minus baseline terminal SOC; positive means baseline leaves less.
    pub terminal_soc_shortfall_quanta: i128,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DispatchRegretAssessment {
    pub assessment_schema_version: String,
    pub case_id: String,
    pub synthetic: bool,
    pub policy_version: String,
    pub claim_ceiling: String,
    pub oracle_is_exact: bool,
    pub oracle_pareto_frontier_complete: bool,
    pub oracle_action_expansions: u64,
    pub oracle_feasible_complete_schedules: u64,
    pub lexicographic_oracle_objective: DispatchObjective,
    /// A failed heuristic is reported explicitly rather than omitted from the corpus.
    pub baseline_feasible: bool,
    pub baseline_failure: Option<String>,
    pub baseline_objective: Option<DispatchObjective>,
    pub objective_deltas_vs_lexicographic_oracle: Option<DispatchObjectiveDeltas>,
    pub baseline_schedule: Option<Vec<DispatchIntervalDecision>>,
    pub lexicographic_oracle_schedule: Vec<DispatchIntervalDecision>,
    pub interpretation: String,
}

fn positive_i128(value: u64) -> i128 {
    i128::from(value)
}

fn objective_deltas(
    baseline: &DispatchObjective,
    oracle: &DispatchObjective,
) -> DispatchObjectiveDeltas {
    DispatchObjectiveDeltas {
        essential_unserved_quanta: positive_i128(baseline.essential_unserved_quanta)
            - positive_i128(oracle.essential_unserved_quanta),
        flexible_unserved_quanta: positive_i128(baseline.flexible_unserved_quanta)
            - positive_i128(oracle.flexible_unserved_quanta),
        operating_cost_units: positive_i128(baseline.operating_cost_units)
            - positive_i128(oracle.operating_cost_units),
        grid_carbon_grams: positive_i128(baseline.grid_carbon_grams)
            - positive_i128(oracle.grid_carbon_grams),
        curtailed_pv_quanta: positive_i128(baseline.curtailed_pv_quanta)
            - positive_i128(oracle.curtailed_pv_quanta),
        terminal_soc_shortfall_quanta: i128::from(oracle.terminal_soc_quanta)
            - i128::from(baseline.terminal_soc_quanta),
    }
}

fn charge_gain(input: u16, request: &EnergyDispatchRequest) -> u16 {
    ((u64::from(input)
        * u64::from(request.battery.charge_efficiency_numerator))
        / u64::from(request.battery.charge_efficiency_denominator)) as u16
}

fn discharge_output(withdrawal: u16, request: &EnergyDispatchRequest) -> u16 {
    ((u64::from(withdrawal)
        * u64::from(request.battery.discharge_efficiency_numerator))
        / u64::from(request.battery.discharge_efficiency_denominator)) as u16
}

/// Build a deterministic feasible schedule where the policy can satisfy the
/// hard terminal reserve. Failure is a benchmark outcome, not a fabricated schedule.
fn build_baseline_schedule(
    request: &EnergyDispatchRequest,
) -> Result<Vec<DispatchIntervalDecision>, String> {
    let battery = &request.battery;
    let mut soc = battery.initial_soc_quanta;
    let mut schedule = Vec::with_capacity(request.intervals.len());

    for input in &request.intervals {
        let pv_available = input.pv_available_quanta;
        let mut pv_remaining = pv_available;

        // Allocate direct PV to essential demand first, then flexible demand.
        let essential_pv = input.essential_demand_quanta.min(pv_remaining);
        pv_remaining -= essential_pv;
        let flexible_pv = input.flexible_demand_quanta.min(pv_remaining);
        pv_remaining -= flexible_pv;

        let mut essential_remaining = input.essential_demand_quanta - essential_pv;
        let mut flexible_remaining = input.flexible_demand_quanta - flexible_pv;
        let mut grid_remaining = if input.grid_outage {
            0
        } else {
            input.grid_import_limit_quanta
        };

        // Prefer available grid energy to battery discharge. This deliberately
        // transparent rule will reveal cost/carbon regret on suitable cases.
        let essential_grid = essential_remaining.min(grid_remaining);
        essential_remaining -= essential_grid;
        grid_remaining -= essential_grid;
        let flexible_grid = flexible_remaining.min(grid_remaining);
        flexible_remaining -= flexible_grid;
        grid_remaining -= flexible_grid;
        let grid_import = essential_grid + flexible_grid;

        // Preserve the declared terminal reserve at every step. If the opening
        // SOC is already below reserve, do not discharge until PV charging restores it.
        let withdrawable = battery
            .discharge_withdrawal_limit_quanta
            .min(soc.saturating_sub(battery.terminal_reserve_quanta));
        let remaining_demand = u32::from(essential_remaining) + u32::from(flexible_remaining);
        let mut selected_withdrawal = 0_u16;
        let mut selected_delivery = 0_u16;

        // Select the smallest withdrawal achieving the largest non-wasted integer
        // delivery that does not exceed the remaining demand.
        for withdrawal in 1..=withdrawable {
            let delivered = discharge_output(withdrawal, request);
            if delivered > 0
                && u32::from(delivered) <= remaining_demand
                && delivered > selected_delivery
            {
                selected_withdrawal = withdrawal;
                selected_delivery = delivered;
            }
        }

        let essential_battery = essential_remaining.min(selected_delivery);
        essential_remaining -= essential_battery;
        let delivery_after_essential = selected_delivery - essential_battery;
        let flexible_battery = flexible_remaining.min(delivery_after_essential);
        flexible_remaining -= flexible_battery;
        let essential_served = essential_pv + essential_grid + essential_battery;
        let flexible_served = flexible_pv + flexible_grid + flexible_battery;
        let essential_unserved = input.essential_demand_quanta - essential_served;
        let flexible_unserved = input.flexible_demand_quanta - flexible_served;

        // No simultaneous charge/discharge. Charge from leftover PV only, and
        // choose the greatest bus-side input that yields a positive gain without
        // exceeding storage capacity.
        let mut charge_input = 0_u16;
        let mut gained_soc = 0_u16;
        if selected_withdrawal == 0 && pv_remaining > 0 {
            let upper = pv_remaining.min(battery.charge_input_limit_quanta);
            for candidate in (1..=upper).rev() {
                let gain = charge_gain(candidate, request);
                if gain > 0
                    && u32::from(soc) + u32::from(gain)
                        <= u32::from(battery.capacity_quanta)
                {
                    charge_input = candidate;
                    gained_soc = gain;
                    break;
                }
            }
        }

        let soc_after = u32::from(soc) - u32::from(selected_withdrawal)
            + u32::from(gained_soc);
        if soc_after > u32::from(battery.capacity_quanta) {
            return Err(format!(
                "interval {} would exceed battery capacity",
                input.interval_id
            ));
        }
        let soc_after = soc_after as u16;
        let pv_used = pv_available - pv_remaining + charge_input;
        if pv_used > pv_available {
            return Err(format!(
                "interval {} uses more PV than available",
                input.interval_id
            ));
        }
        let pv_curtailed = pv_available - pv_used;
        let battery_discharge_delivered = discharge_output(selected_withdrawal, request);
        let bus_supply = u64::from(pv_used) + u64::from(grid_import)
            + u64::from(battery_discharge_delivered);
        let bus_use = u64::from(essential_served) + u64::from(flexible_served)
            + u64::from(charge_input);
        if bus_supply != bus_use {
            return Err(format!(
                "interval {} fails integer bus balance: supply={bus_supply}, use={bus_use}",
                input.interval_id
            ));
        }

        schedule.push(DispatchIntervalDecision {
            interval_id: input.interval_id.clone(),
            start_unix_seconds: input.start_unix_seconds,
            duration_seconds: input.duration_seconds,
            grid_outage: input.grid_outage,
            pv_available_quanta: pv_available,
            pv_used_quanta: pv_used,
            pv_curtailed_quanta: pv_curtailed,
            grid_import_quanta: grid_import,
            battery_soc_before_quanta: soc,
            battery_charge_input_quanta: charge_input,
            battery_soc_withdrawal_quanta: selected_withdrawal,
            battery_discharge_delivered_quanta: battery_discharge_delivered,
            battery_soc_after_quanta: soc_after,
            essential_demand_quanta: input.essential_demand_quanta,
            essential_served_quanta: essential_served,
            essential_unserved_quanta: essential_unserved,
            flexible_demand_quanta: input.flexible_demand_quanta,
            flexible_served_quanta: flexible_served,
            flexible_unserved_quanta: flexible_unserved,
            grid_cost_units_per_quantum: input.grid_cost_units_per_quantum,
            grid_carbon_grams_per_quantum: input.grid_carbon_grams_per_quantum,
            bus_energy_balance_residual_quanta: 0,
        });
        soc = soc_after;
    }

    if soc < battery.terminal_reserve_quanta {
        return Err(format!(
            "terminal reserve unmet: final SOC {soc}, required {}",
            battery.terminal_reserve_quanta
        ));
    }
    Ok(schedule)
}

/// Compare a fixed, transparent heuristic against the exact model's explicit
/// lexicographic reference alternative. Each metric delta is reported separately;
/// no arbitrary weighted sum is used to hide essential-service shortfall.
pub fn assess_dispatch_regret(
    request: &EnergyDispatchRequest,
) -> Result<DispatchRegretAssessment, EnergyDispatchError> {
    let oracle: EnergyDispatchAssessment = assess_energy_dispatch(request)?;
    for alternative in &oracle.pareto_alternatives {
        crate::energy_dispatch_verifier::verify_dispatch_schedule(request, &alternative.schedule)
            .map_err(|errors| EnergyDispatchError::ScheduleVerificationFailed {
                reason: errors.join("; "),
            })?;
    }
    let reference = oracle
        .pareto_alternatives
        .first()
        .ok_or(EnergyDispatchError::NoFeasibleSchedules)?;

    let baseline_result = build_baseline_schedule(request).and_then(|schedule| {
        crate::energy_dispatch_verifier::verify_dispatch_schedule(request, &schedule)
            .map_err(|errors| {
                format!("independent verifier rejected baseline: {}", errors.join("; "))
            })?;
        Ok(schedule)
    });
    match baseline_result {
        Ok(schedule) => {
            let objective = objective_for_schedule(request, &schedule)?;
            Ok(DispatchRegretAssessment {
                assessment_schema_version: "energy-dispatch-regret-v1".to_owned(),
                case_id: request.case_id.clone(),
                synthetic: request.synthetic,
                policy_version: BASELINE_POLICY_VERSION.to_owned(),
                claim_ceiling: concat!(
                    "baseline comparison is exact only for the declared finite quantized ",
                    "model and selected lexicographic reference; it is not a continuous ",
                    "physics or real-world optimality claim"
                )
                .to_owned(),
                oracle_is_exact: oracle.search_is_exact,
                oracle_pareto_frontier_complete: oracle.pareto_frontier_complete,
                oracle_action_expansions: oracle.evaluated_action_expansions,
                oracle_feasible_complete_schedules: oracle.feasible_complete_schedules,
                lexicographic_oracle_objective: reference.objective.clone(),
                baseline_feasible: true,
                baseline_failure: None,
                objective_deltas_vs_lexicographic_oracle: Some(objective_deltas(
                    &objective,
                    &reference.objective,
                )),
                baseline_objective: Some(objective),
                baseline_schedule: Some(schedule),
                lexicographic_oracle_schedule: reference.schedule.clone(),
                interpretation: concat!(
                    "Metric deltas are reported independently. Positive deltas indicate ",
                    "worse baseline outcomes for minimized metrics; terminal SOC uses ",
                    "oracle SOC minus baseline SOC. Negative deltas on one dimension can ",
                    "coexist with worse outcomes on another and must not be collapsed ",
                    "without an explicit policy."
                )
                .to_owned(),
            })
        }
        Err(reason) => Ok(DispatchRegretAssessment {
            assessment_schema_version: "energy-dispatch-regret-v1".to_owned(),
            case_id: request.case_id.clone(),
            synthetic: request.synthetic,
            policy_version: BASELINE_POLICY_VERSION.to_owned(),
            claim_ceiling: concat!(
                "baseline infeasibility is observed for this heuristic; the exact oracle ",
                "is scoped to the declared finite model and cannot certify real-world safety"
            )
            .to_owned(),
            oracle_is_exact: oracle.search_is_exact,
            oracle_pareto_frontier_complete: oracle.pareto_frontier_complete,
            oracle_action_expansions: oracle.evaluated_action_expansions,
            oracle_feasible_complete_schedules: oracle.feasible_complete_schedules,
            lexicographic_oracle_objective: reference.objective.clone(),
            baseline_feasible: false,
            baseline_failure: Some(reason),
            baseline_objective: None,
            objective_deltas_vs_lexicographic_oracle: None,
            baseline_schedule: None,
            lexicographic_oracle_schedule: reference.schedule.clone(),
            interpretation: concat!(
                "The heuristic failed a hard constraint; no objective regret vector ",
                "is fabricated. ",
                "Count this case as a baseline feasibility failure."
            )
            .to_owned(),
        }),
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
    fn baseline_matches_the_frozen_pv_charge_outage_known_answer() {
        let request = fixture_request();
        let report = assess_dispatch_regret(&request).expect("exact oracle and baseline");
        assert!(report.oracle_is_exact);
        assert!(report.oracle_pareto_frontier_complete);
        assert!(report.baseline_feasible);
        assert_eq!(
            report.lexicographic_oracle_objective.essential_unserved_quanta,
            0
        );
        assert_eq!(
            report.baseline_objective.as_ref().unwrap().essential_unserved_quanta,
            0
        );
        assert_eq!(
            report
                .objective_deltas_vs_lexicographic_oracle
                .as_ref()
                .unwrap()
                .essential_unserved_quanta,
            0
        );
        assert_eq!(report.baseline_schedule.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn grid_first_rule_exposes_cost_regret_against_storage_dispatch() {
        let mut request = fixture_request();
        request.case_id = "synthetic-cost-regret".to_owned();
        request.intervals.truncate(1);
        request.battery.capacity_quanta = 2;
        request.battery.initial_soc_quanta = 1;
        request.battery.charge_input_limit_quanta = 1;
        request.battery.discharge_withdrawal_limit_quanta = 1;
        request.battery.terminal_reserve_quanta = 0;
        request.intervals[0].pv_available_quanta = 0;
        request.intervals[0].grid_outage = false;
        request.intervals[0].grid_import_limit_quanta = 1;
        request.intervals[0].essential_demand_quanta = 1;
        request.intervals[0].flexible_demand_quanta = 0;
        request.intervals[0].grid_cost_units_per_quantum = 10;
        request.intervals[0].grid_carbon_grams_per_quantum = 10;

        let report = assess_dispatch_regret(&request).expect("bounded exact oracle");
        assert!(report.baseline_feasible);
        assert_eq!(
            report.baseline_objective.as_ref().unwrap().operating_cost_units,
            10
        );
        assert_eq!(report.lexicographic_oracle_objective.operating_cost_units, 0);
        assert_eq!(
            report.objective_deltas_vs_lexicographic_oracle.as_ref().unwrap().operating_cost_units,
            10
        );
    }

    #[test]
    fn baseline_failure_is_explicit_when_only_grid_charging_can_meet_reserve() {
        let mut request = fixture_request();
        request.case_id = "synthetic-baseline-reserve-failure".to_owned();
        request.intervals.truncate(1);
        request.battery.capacity_quanta = 1;
        request.battery.initial_soc_quanta = 0;
        request.battery.charge_input_limit_quanta = 1;
        request.battery.discharge_withdrawal_limit_quanta = 1;
        request.battery.terminal_reserve_quanta = 1;
        request.intervals[0].pv_available_quanta = 0;
        request.intervals[0].grid_outage = false;
        request.intervals[0].grid_import_limit_quanta = 1;
        request.intervals[0].essential_demand_quanta = 0;
        request.intervals[0].flexible_demand_quanta = 0;

        let report = assess_dispatch_regret(&request).expect("oracle can import to charge");
        assert!(!report.baseline_feasible);
        assert!(report.baseline_failure.is_some());
        assert!(report.baseline_objective.is_none());
        assert!(report.objective_deltas_vs_lexicographic_oracle.is_none());
        assert_eq!(report.lexicographic_oracle_objective.terminal_soc_quanta, 1);
    }

    #[test]
    fn baseline_schedule_never_imports_grid_during_outage() {
        let report =
            assess_dispatch_regret(&fixture_request()).expect("frozen fixture is feasible");
        for step in report.baseline_schedule.unwrap() {
            if step.grid_outage {
                assert_eq!(step.grid_import_quanta, 0);
            }
            assert_eq!(step.bus_energy_balance_residual_quanta, 0);
            assert_eq!(
                step.essential_served_quanta + step.essential_unserved_quanta,
                step.essential_demand_quanta
            );
            assert_eq!(
                step.flexible_served_quanta + step.flexible_unserved_quanta,
                step.flexible_demand_quanta
            );
        }
    }
}
