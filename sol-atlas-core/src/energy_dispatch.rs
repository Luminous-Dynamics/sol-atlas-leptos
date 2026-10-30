// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Exact bounded dispatch oracle for tiny, discrete microgrid cases.
//!
//! "Exact" applies only to the complete finite model declared by the request.
//! This is not a continuous-physics simulator, a capacity-expansion model, a
//! network power-flow solver, a forecast, or evidence that a schedule is safe
//! or optimal outside the declared model.

use crate::energy_pathway::EvidenceClass;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BTreeSet;

pub const MAX_CANDIDATE_EXPANSIONS: u64 = 1_000_000;
pub const MAX_PARETO_FRONTIER: usize = 128;

/// Finite-model storage configuration. Charge limits are bus-side input quanta;
/// discharge limits are internal state-of-charge withdrawal quanta.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchBattery {
    pub capacity_quanta: u16,
    pub initial_soc_quanta: u16,
    pub charge_input_limit_quanta: u16,
    pub discharge_withdrawal_limit_quanta: u16,
    pub charge_efficiency_numerator: u16,
    pub charge_efficiency_denominator: u16,
    pub discharge_efficiency_numerator: u16,
    pub discharge_efficiency_denominator: u16,
    pub terminal_reserve_quanta: u16,
}

/// A single chronological dispatch interval. Energy caps are per-interval
/// energy quantities, not instantaneous power ratings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchIntervalInput {
    pub interval_id: String,
    pub start_unix_seconds: i64,
    pub duration_seconds: u64,
    pub grid_outage: bool,
    pub pv_available_quanta: u16,
    pub grid_import_limit_quanta: u16,
    pub essential_demand_quanta: u16,
    pub flexible_demand_quanta: u16,
    /// Exact integer operating-cost units per imported grid quantum.
    pub grid_cost_units_per_quantum: u32,
    /// Exact integer grams of declared carbon per imported grid quantum.
    pub grid_carbon_grams_per_quantum: u32,
}

/// Complete finite dispatch problem; all energy choices are integer quanta.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyDispatchRequest {
    pub schema_version: String,
    pub case_id: String,
    pub synthetic: bool,
    pub evidence_class: EvidenceClass,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub evidence_as_of: Option<String>,
    /// Used only to express the chosen integer schedule in physical kWh.
    /// It never participates in search, feasibility or objective comparison.
    pub energy_quantum_kwh: f64,
    pub max_candidate_expansions: u64,
    pub battery: DispatchBattery,
    pub intervals: Vec<DispatchIntervalInput>,
}

/// One complete, physically balanced interval decision in the declared model.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DispatchIntervalDecision {
    pub interval_id: String,
    pub start_unix_seconds: i64,
    pub duration_seconds: u64,
    pub grid_outage: bool,
    pub pv_available_quanta: u16,
    pub pv_used_quanta: u16,
    pub pv_curtailed_quanta: u16,
    pub grid_import_quanta: u16,
    pub battery_soc_before_quanta: u16,
    pub battery_charge_input_quanta: u16,
    pub battery_soc_withdrawal_quanta: u16,
    pub battery_discharge_delivered_quanta: u16,
    pub battery_soc_after_quanta: u16,
    pub essential_demand_quanta: u16,
    pub essential_served_quanta: u16,
    pub essential_unserved_quanta: u16,
    pub flexible_demand_quanta: u16,
    pub flexible_served_quanta: u16,
    pub flexible_unserved_quanta: u16,
    pub grid_cost_units_per_quantum: u32,
    pub grid_carbon_grams_per_quantum: u32,
    /// Exactly zero by construction: all bus-side quantities are integer quanta.
    pub bus_energy_balance_residual_quanta: i64,
}

/// Minimize the first five dimensions; maximize terminal state of charge.
/// Reserve is a hard constraint and never a soft penalty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchObjective {
    pub essential_unserved_quanta: u64,
    pub flexible_unserved_quanta: u64,
    pub operating_cost_units: u64,
    pub grid_carbon_grams: u64,
    pub curtailed_pv_quanta: u64,
    pub terminal_soc_quanta: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DispatchParetoAlternative {
    pub objective: DispatchObjective,
    pub schedule: Vec<DispatchIntervalDecision>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EnergyDispatchAssessment {
    pub assessment_schema_version: String,
    pub case_id: String,
    pub synthetic: bool,
    pub evidence_class: EvidenceClass,
    pub evidence_ref: Option<String>,
    pub evidence_as_of: Option<String>,
    pub claim_ceiling: String,
    pub energy_quantum_kwh: f64,
    /// True only because a successful assessment completed exhaustive enumeration.
    pub search_is_exact: bool,
    /// True only if no cap was hit and the full non-dominated frontier was retained.
    pub pareto_frontier_complete: bool,
    pub evaluated_action_expansions: u64,
    pub feasible_complete_schedules: u64,
    pub pareto_frontier_count: usize,
    pub pareto_alternatives: Vec<DispatchParetoAlternative>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnergyDispatchError {
    EmptyCaseId,
    UnsupportedSchemaVersion { schema_version: String },
    NoIntervals,
    MissingEvidenceReference,
    InvalidEnergyQuantum,
    InvalidCandidateExpansionCap { requested: u64, hard_max: u64 },
    EmptyIntervalId,
    DuplicateIntervalId { interval_id: String },
    InvalidIntervalDuration { interval_id: String },
    ChronologyGapOrOverlap {
        interval_id: String,
        expected_start_unix_seconds: i64,
        actual_start_unix_seconds: i64,
    },
    ChronologyOverflow { interval_id: String },
    InvalidBatteryConfiguration { reason: String },
    InvalidEfficiencyRatio { name: String },
    CandidateExpansionCapExceeded { cap: u64 },
    ParetoFrontierCapExceeded { cap: usize },
    ArithmeticOverflow { context: String },
    NoFeasibleSchedules,
}

fn validate_efficiency(
    numerator: u16,
    denominator: u16,
    name: &str,
) -> Result<(), EnergyDispatchError> {
    if denominator == 0 || numerator == 0 || numerator > denominator {
        return Err(EnergyDispatchError::InvalidEfficiencyRatio {
            name: name.to_owned(),
        });
    }
    Ok(())
}

fn add_u64(total: &mut u64, amount: u64, context: &str) -> Result<(), EnergyDispatchError> {
    *total = total
        .checked_add(amount)
        .ok_or_else(|| EnergyDispatchError::ArithmeticOverflow {
            context: context.to_owned(),
        })?;
    Ok(())
}

fn multiply_u64(lhs: u64, rhs: u64, context: &str) -> Result<u64, EnergyDispatchError> {
    lhs.checked_mul(rhs)
        .ok_or_else(|| EnergyDispatchError::ArithmeticOverflow {
            context: context.to_owned(),
        })
}

impl DispatchObjective {
    fn weakly_dominates(&self, other: &Self) -> bool {
        self.essential_unserved_quanta <= other.essential_unserved_quanta
            && self.flexible_unserved_quanta <= other.flexible_unserved_quanta
            && self.operating_cost_units <= other.operating_cost_units
            && self.grid_carbon_grams <= other.grid_carbon_grams
            && self.curtailed_pv_quanta <= other.curtailed_pv_quanta
            && self.terminal_soc_quanta >= other.terminal_soc_quanta
    }

    fn strictly_dominates(&self, other: &Self) -> bool {
        self.weakly_dominates(other) && self != other
    }
}

fn objective_for_schedule(
    request: &EnergyDispatchRequest,
    schedule: &[DispatchIntervalDecision],
) -> Result<DispatchObjective, EnergyDispatchError> {
    let mut objective = DispatchObjective {
        essential_unserved_quanta: 0,
        flexible_unserved_quanta: 0,
        operating_cost_units: 0,
        grid_carbon_grams: 0,
        curtailed_pv_quanta: 0,
        terminal_soc_quanta: schedule
            .last()
            .map(|step| step.battery_soc_after_quanta)
            .unwrap_or(request.battery.initial_soc_quanta),
    };

    for (step, input) in schedule.iter().zip(&request.intervals) {
        add_u64(
            &mut objective.essential_unserved_quanta,
            u64::from(step.essential_unserved_quanta),
            "essential unserved total",
        )?;
        add_u64(
            &mut objective.flexible_unserved_quanta,
            u64::from(step.flexible_unserved_quanta),
            "flexible unserved total",
        )?;
        add_u64(
            &mut objective.operating_cost_units,
            multiply_u64(
                u64::from(step.grid_import_quanta),
                u64::from(input.grid_cost_units_per_quantum),
                "interval operating cost",
            )?,
            "operating cost total",
        )?;
        add_u64(
            &mut objective.grid_carbon_grams,
            multiply_u64(
                u64::from(step.grid_import_quanta),
                u64::from(input.grid_carbon_grams_per_quantum),
                "interval grid carbon",
            )?,
            "grid carbon total",
        )?;
        add_u64(
            &mut objective.curtailed_pv_quanta,
            u64::from(step.pv_curtailed_quanta),
            "curtailed PV total",
        )?;
    }
    Ok(objective)
}

#[cfg(test)]
type TestObjectiveVector = (u64, u64, u64, u64, u64, u16);

struct ExactSearch<'a> {
    request: &'a EnergyDispatchRequest,
    evaluated_action_expansions: u64,
    feasible_complete_schedules: u64,
    frontier: Vec<DispatchParetoAlternative>,
    // Captured only in test builds so the independent enumerator can compare
    // the full feasible objective-vector set, not just count and Pareto members.
    #[cfg(test)]
    all_feasible_objective_vectors: BTreeSet<TestObjectiveVector>,
}

impl<'a> ExactSearch<'a> {
    fn tick(&mut self) -> Result<(), EnergyDispatchError> {
        self.evaluated_action_expansions = self
            .evaluated_action_expansions
            .checked_add(1)
            .ok_or_else(|| EnergyDispatchError::ArithmeticOverflow {
                context: "action-expansion counter".to_owned(),
            })?;
        if self.evaluated_action_expansions > self.request.max_candidate_expansions {
            return Err(EnergyDispatchError::CandidateExpansionCapExceeded {
                cap: self.request.max_candidate_expansions,
            });
        }
        Ok(())
    }

    fn record_schedule(
        &mut self,
        schedule: &[DispatchIntervalDecision],
    ) -> Result<(), EnergyDispatchError> {
        self.feasible_complete_schedules = self
            .feasible_complete_schedules
            .checked_add(1)
            .ok_or_else(|| EnergyDispatchError::ArithmeticOverflow {
                context: "complete-schedule counter".to_owned(),
            })?;
        let objective = objective_for_schedule(self.request, schedule)?;
        #[cfg(test)]
        self.all_feasible_objective_vectors.insert((
            objective.essential_unserved_quanta,
            objective.flexible_unserved_quanta,
            objective.operating_cost_units,
            objective.grid_carbon_grams,
            objective.curtailed_pv_quanta,
            objective.terminal_soc_quanta,
        ));

        // Deterministic first-witness retention for duplicate objective vectors.
        // Enumeration order is fixed by ascending integer loops.
        if self
            .frontier
            .iter()
            .any(|known| known.objective.weakly_dominates(&objective))
        {
            return Ok(());
        }

        self.frontier
            .retain(|known| !objective.strictly_dominates(&known.objective));
        self.frontier.push(DispatchParetoAlternative {
            objective,
            schedule: schedule.to_vec(),
        });
        if self.frontier.len() > MAX_PARETO_FRONTIER {
            return Err(EnergyDispatchError::ParetoFrontierCapExceeded {
                cap: MAX_PARETO_FRONTIER,
            });
        }
        Ok(())
    }

    fn visit(
        &mut self,
        interval_index: usize,
        soc_before: u16,
        path: &mut Vec<DispatchIntervalDecision>,
    ) -> Result<(), EnergyDispatchError> {
        if interval_index == self.request.intervals.len() {
            if soc_before >= self.request.battery.terminal_reserve_quanta {
                self.record_schedule(path)?;
            }
            return Ok(());
        }

        // Copy the interval because this recursion mutably updates search counters
        // and the Pareto set while iterating through its finite action space.
        let input = self.request.intervals[interval_index].clone();
        let battery = self.request.battery.clone();
        let grid_limit = if input.grid_outage {
            0
        } else {
            input.grid_import_limit_quanta
        };
        let withdrawal_limit = battery.discharge_withdrawal_limit_quanta.min(soc_before);

        for grid_import in 0..=grid_limit {
            self.tick()?;
            for soc_withdrawal in 0..=withdrawal_limit {
                self.tick()?;
                for charge_input in 0..=battery.charge_input_limit_quanta {
                    self.tick()?;
                    // Simultaneous charging and discharging is prohibited by this
                    // v0.1 model, rather than priced or merely discouraged.
                    if soc_withdrawal > 0 && charge_input > 0 {
                        continue;
                    }
                    let charge_gain = (u64::from(charge_input)
                        * u64::from(battery.charge_efficiency_numerator)
                        / u64::from(battery.charge_efficiency_denominator))
                        as u16;
                    let soc_after_charge = u32::from(soc_before)
                        - u32::from(soc_withdrawal)
                        + u32::from(charge_gain);
                    if soc_after_charge > u32::from(battery.capacity_quanta) {
                        continue;
                    }
                    let discharge_delivered = (u64::from(soc_withdrawal)
                        * u64::from(battery.discharge_efficiency_numerator)
                        / u64::from(battery.discharge_efficiency_denominator))
                        as u16;

                    for essential_served in 0..=input.essential_demand_quanta {
                        self.tick()?;
                        for flexible_served in 0..=input.flexible_demand_quanta {
                            self.tick()?;
                            // Exact integer bus balance:
                            // PV-used + grid + battery-output = essential + flexible + charge.
                            let load_plus_charge = u64::from(essential_served)
                                + u64::from(flexible_served)
                                + u64::from(charge_input);
                            let non_pv_supply =
                                u64::from(grid_import) + u64::from(discharge_delivered);
                            if load_plus_charge < non_pv_supply {
                                continue;
                            }
                            let pv_used = load_plus_charge - non_pv_supply;
                            if pv_used > u64::from(input.pv_available_quanta) {
                                continue;
                            }
                            let pv_curtailed =
                                u64::from(input.pv_available_quanta) - pv_used;
                            let step = DispatchIntervalDecision {
                                interval_id: input.interval_id.clone(),
                                start_unix_seconds: input.start_unix_seconds,
                                duration_seconds: input.duration_seconds,
                                grid_outage: input.grid_outage,
                                pv_available_quanta: input.pv_available_quanta,
                                pv_used_quanta: pv_used as u16,
                                pv_curtailed_quanta: pv_curtailed as u16,
                                grid_import_quanta: grid_import,
                                battery_soc_before_quanta: soc_before,
                                battery_charge_input_quanta: charge_input,
                                battery_soc_withdrawal_quanta: soc_withdrawal,
                                battery_discharge_delivered_quanta: discharge_delivered,
                                battery_soc_after_quanta: soc_after_charge as u16,
                                essential_demand_quanta: input.essential_demand_quanta,
                                essential_served_quanta: essential_served,
                                essential_unserved_quanta: input
                                    .essential_demand_quanta
                                    - essential_served,
                                flexible_demand_quanta: input.flexible_demand_quanta,
                                flexible_served_quanta: flexible_served,
                                flexible_unserved_quanta: input
                                    .flexible_demand_quanta
                                    - flexible_served,
                                grid_cost_units_per_quantum: input.grid_cost_units_per_quantum,
                                grid_carbon_grams_per_quantum: input.grid_carbon_grams_per_quantum,
                                bus_energy_balance_residual_quanta: 0,
                            };
                            path.push(step);
                            self.visit(interval_index + 1, step_soc_after(path), path)?;
                            path.pop();
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn step_soc_after(path: &[DispatchIntervalDecision]) -> u16 {
    path.last()
        .map(|step| step.battery_soc_after_quanta)
        .unwrap_or(0)
}

/// Exhaustively enumerate the complete bounded model and return its non-dominated
/// feasible schedules. Any configured search or frontier cap hit is an error; no
/// partial frontier is returned with an "exact" label.
pub fn assess_energy_dispatch(
    request: &EnergyDispatchRequest,
) -> Result<EnergyDispatchAssessment, EnergyDispatchError> {
    if request.schema_version != "energy-dispatch-v0.1" {
        return Err(EnergyDispatchError::UnsupportedSchemaVersion {
            schema_version: request.schema_version.clone(),
        });
    }
    if request.case_id.trim().is_empty() {
        return Err(EnergyDispatchError::EmptyCaseId);
    }
    if !request.energy_quantum_kwh.is_finite() || request.energy_quantum_kwh <= 0.0 {
        return Err(EnergyDispatchError::InvalidEnergyQuantum);
    }
    if request.intervals.is_empty() {
        return Err(EnergyDispatchError::NoIntervals);
    }
    if request.max_candidate_expansions == 0
        || request.max_candidate_expansions > MAX_CANDIDATE_EXPANSIONS
    {
        return Err(EnergyDispatchError::InvalidCandidateExpansionCap {
            requested: request.max_candidate_expansions,
            hard_max: MAX_CANDIDATE_EXPANSIONS,
        });
    }
    if request.evidence_class != EvidenceClass::AnalystAssumption
        && request
            .evidence_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
    {
        return Err(EnergyDispatchError::MissingEvidenceReference);
    }

    let battery = &request.battery;
    if battery.initial_soc_quanta > battery.capacity_quanta
        || battery.terminal_reserve_quanta > battery.capacity_quanta
    {
        return Err(EnergyDispatchError::InvalidBatteryConfiguration {
            reason: "initial state and terminal reserve must not exceed capacity".to_owned(),
        });
    }
    validate_efficiency(
        battery.charge_efficiency_numerator,
        battery.charge_efficiency_denominator,
        "charge",
    )?;
    validate_efficiency(
        battery.discharge_efficiency_numerator,
        battery.discharge_efficiency_denominator,
        "discharge",
    )?;

    let mut ids = BTreeSet::new();
    let mut expected_start = None::<i64>;
    for interval in &request.intervals {
        if interval.interval_id.trim().is_empty() {
            return Err(EnergyDispatchError::EmptyIntervalId);
        }
        if !ids.insert(interval.interval_id.as_str()) {
            return Err(EnergyDispatchError::DuplicateIntervalId {
                interval_id: interval.interval_id.clone(),
            });
        }
        if interval.duration_seconds == 0 {
            return Err(EnergyDispatchError::InvalidIntervalDuration {
                interval_id: interval.interval_id.clone(),
            });
        }
        if let Some(expected) = expected_start {
            if interval.start_unix_seconds != expected {
                return Err(EnergyDispatchError::ChronologyGapOrOverlap {
                    interval_id: interval.interval_id.clone(),
                    expected_start_unix_seconds: expected,
                    actual_start_unix_seconds: interval.start_unix_seconds,
                });
            }
        }
        expected_start = Some(
            interval
                .start_unix_seconds
                .checked_add(i64::try_from(interval.duration_seconds).map_err(|_| {
                    EnergyDispatchError::ChronologyOverflow {
                        interval_id: interval.interval_id.clone(),
                    }
                })?)
                .ok_or_else(|| EnergyDispatchError::ChronologyOverflow {
                    interval_id: interval.interval_id.clone(),
                })?,
        );
    }

    let mut search = ExactSearch {
        request,
        evaluated_action_expansions: 0,
        feasible_complete_schedules: 0,
        frontier: Vec::new(),
        #[cfg(test)]
        all_feasible_objective_vectors: BTreeSet::new(),
    };
    let mut path = Vec::with_capacity(request.intervals.len());
    search.visit(0, battery.initial_soc_quanta, &mut path)?;
    if search.feasible_complete_schedules == 0 || search.frontier.is_empty() {
        return Err(EnergyDispatchError::NoFeasibleSchedules);
    }

    search.frontier.sort_by_key(|alternative| {
        let objective = &alternative.objective;
        (
            objective.essential_unserved_quanta,
            objective.flexible_unserved_quanta,
            objective.operating_cost_units,
            objective.grid_carbon_grams,
            objective.curtailed_pv_quanta,
            Reverse(objective.terminal_soc_quanta),
        )
    });

    Ok(EnergyDispatchAssessment {
        assessment_schema_version: "energy-dispatch-report-v1".to_owned(),
        case_id: request.case_id.clone(),
        synthetic: request.synthetic,
        evidence_class: request.evidence_class,
        evidence_ref: request.evidence_ref.clone(),
        evidence_as_of: request.evidence_as_of.clone(),
        claim_ceiling: concat!(
            "exact only over the declared finite quantized model; not a continuous ",
            "physics, network-flow, forecast, safety, or real-world optimality claim; ",
            "evidence metadata is not independently authenticated"
        )
        .to_owned(),
        energy_quantum_kwh: request.energy_quantum_kwh,
        search_is_exact: true,
        pareto_frontier_complete: true,
        evaluated_action_expansions: search.evaluated_action_expansions,
        feasible_complete_schedules: search.feasible_complete_schedules,
        pareto_frontier_count: search.frontier.len(),
        pareto_alternatives: search.frontier,
    })
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
    fn frozen_pv_charge_then_outage_case_has_zero_essential_shortfall() {
        let request = fixture_request();
        let result = assess_energy_dispatch(&request).expect("finite case is feasible");
        assert!(result.search_is_exact);
        assert!(result.pareto_frontier_complete);
        assert!(result.feasible_complete_schedules > 0);
        assert!(result.evaluated_action_expansions <= request.max_candidate_expansions);
        assert!(!result.pareto_alternatives.is_empty());

        let best = &result.pareto_alternatives[0];
        assert_eq!(best.objective.essential_unserved_quanta, 0);
        assert_eq!(best.objective.flexible_unserved_quanta, 0);
        assert_eq!(best.objective.operating_cost_units, 0);
        assert_eq!(best.objective.grid_carbon_grams, 0);
        assert_eq!(best.objective.curtailed_pv_quanta, 0);
        assert_eq!(best.objective.terminal_soc_quanta, 0);
        assert_eq!(best.schedule.len(), 2);
        assert_eq!(best.schedule[0].battery_charge_input_quanta, 2);
        assert_eq!(best.schedule[0].pv_used_quanta, 2);
        assert_eq!(best.schedule[1].battery_soc_withdrawal_quanta, 2);
        assert_eq!(best.schedule[1].essential_served_quanta, 2);
        assert_eq!(best.schedule[1].grid_import_quanta, 0);
    }

    #[test]
    fn outage_never_allows_grid_import_and_every_bus_balance_closes() {
        let result = assess_energy_dispatch(&fixture_request()).expect("valid case");
        for alternative in result.pareto_alternatives {
            for step in alternative.schedule {
                if step.grid_outage {
                    assert_eq!(step.grid_import_quanta, 0);
                }
                assert_eq!(step.bus_energy_balance_residual_quanta, 0);
                assert_eq!(
                    step.pv_used_quanta + step.pv_curtailed_quanta,
                    step.pv_available_quanta
                );
                assert_eq!(
                    step.essential_served_quanta + step.essential_unserved_quanta,
                    step.essential_demand_quanta
                );
                assert_eq!(
                    step.flexible_served_quanta + step.flexible_unserved_quanta,
                    step.flexible_demand_quanta
                );
                assert!(!(step.battery_charge_input_quanta > 0
                    && step.battery_soc_withdrawal_quanta > 0));
            }
        }
    }

    #[test]
    fn reported_frontier_is_pairwise_nondominated() {
        let result = assess_energy_dispatch(&fixture_request()).expect("valid case");
        for (index, left) in result.pareto_alternatives.iter().enumerate() {
            for (other_index, right) in result.pareto_alternatives.iter().enumerate() {
                if index != other_index {
                    assert!(!left.objective.strictly_dominates(&right.objective));
                }
            }
        }
    }

    #[test]
    fn candidate_cap_fails_closed_without_a_partial_exact_result() {
        let mut request = fixture_request();
        request.max_candidate_expansions = 1;
        assert!(matches!(
            assess_energy_dispatch(&request),
            Err(EnergyDispatchError::CandidateExpansionCapExceeded { cap: 1 })
        ));
    }

    #[test]
    fn invalid_reserve_and_efficiency_are_rejected() {
        let mut reserve = fixture_request();
        reserve.battery.terminal_reserve_quanta = reserve.battery.capacity_quanta + 1;
        assert!(matches!(
            assess_energy_dispatch(&reserve),
            Err(EnergyDispatchError::InvalidBatteryConfiguration { .. })
        ));

        let mut efficiency = fixture_request();
        efficiency.battery.discharge_efficiency_numerator = 0;
        assert!(matches!(
            assess_energy_dispatch(&efficiency),
            Err(EnergyDispatchError::InvalidEfficiencyRatio { .. })
        ));
    }

    #[test]
    fn chronology_gap_is_rejected() {
        let mut request = fixture_request();
        request.intervals[1].start_unix_seconds += 1;
        assert!(matches!(
            assess_energy_dispatch(&request),
            Err(EnergyDispatchError::ChronologyGapOrOverlap { .. })
        ));
    }

    #[test]
    fn deterministic_64_case_sweep_rechecks_state_balance_frontier_and_receipts() {
        for seed in 0..64_u16 {
            let mut request = fixture_request();
            request.case_id = format!("synthetic-dispatch-sweep-{seed}");
            request.max_candidate_expansions = 100_000;
            request.battery.capacity_quanta = 4;
            request.battery.initial_soc_quanta = (seed / 3) % 5;
            request.battery.charge_input_limit_quanta = 3;
            request.battery.discharge_withdrawal_limit_quanta = 3;
            request.battery.terminal_reserve_quanta = seed % 5;
            request.battery.charge_efficiency_numerator = 1;
            request.battery.charge_efficiency_denominator = if seed % 2 == 0 { 1 } else { 2 };
            request.battery.discharge_efficiency_numerator = 1;
            request.battery.discharge_efficiency_denominator = if seed % 3 == 0 { 1 } else { 2 };

            request.intervals[0].pv_available_quanta = seed % 4;
            request.intervals[0].grid_import_limit_quanta = seed % 3;
            request.intervals[0].essential_demand_quanta = (seed / 2) % 3;
            request.intervals[0].flexible_demand_quanta = (seed / 3) % 2;
            request.intervals[0].grid_cost_units_per_quantum = u32::from(seed % 3 + 1);
            request.intervals[0].grid_carbon_grams_per_quantum = u32::from(seed % 4);
            request.intervals[1].grid_outage = seed % 2 == 0;
            request.intervals[1].pv_available_quanta = (seed / 3) % 4;
            request.intervals[1].grid_import_limit_quanta = (seed / 4) % 3;
            request.intervals[1].essential_demand_quanta = (seed / 5) % 4;
            request.intervals[1].flexible_demand_quanta = (seed / 7) % 3;
            request.intervals[1].grid_cost_units_per_quantum = u32::from(seed % 5);
            request.intervals[1].grid_carbon_grams_per_quantum = u32::from(seed % 6);

            let result = assess_energy_dispatch(&request);
            let assessment = match result {
                Ok(assessment) => assessment,
                Err(EnergyDispatchError::NoFeasibleSchedules) => continue,
                Err(other) => panic!("seed {seed} must not fail for an unrelated reason: {other:?}"),
            };
            assert!(assessment.search_is_exact);
            assert!(assessment.pareto_frontier_complete);
            assert!(assessment.evaluated_action_expansions <= request.max_candidate_expansions);

            for (index, alternative) in assessment.pareto_alternatives.iter().enumerate() {
                let mut expected_soc = request.battery.initial_soc_quanta;
                for (step, input) in alternative.schedule.iter().zip(&request.intervals) {
                    assert_eq!(step.battery_soc_before_quanta, expected_soc);
                    assert!(step.battery_soc_after_quanta <= request.battery.capacity_quanta);
                    assert!(step.battery_soc_withdrawal_quanta <= step.battery_soc_before_quanta);
                    assert!(!(step.battery_charge_input_quanta > 0
                        && step.battery_soc_withdrawal_quanta > 0));
                    let gained = u64::from(step.battery_charge_input_quanta)
                        * u64::from(request.battery.charge_efficiency_numerator)
                        / u64::from(request.battery.charge_efficiency_denominator);
                    let expected_after = u32::from(step.battery_soc_before_quanta)
                        - u32::from(step.battery_soc_withdrawal_quanta)
                        + gained as u32;
                    assert_eq!(u32::from(step.battery_soc_after_quanta), expected_after);
                    let expected_discharge = u64::from(step.battery_soc_withdrawal_quanta)
                        * u64::from(request.battery.discharge_efficiency_numerator)
                        / u64::from(request.battery.discharge_efficiency_denominator);
                    assert_eq!(u64::from(step.battery_discharge_delivered_quanta), expected_discharge);
                    assert_eq!(step.grid_outage, input.grid_outage);
                    if input.grid_outage {
                        assert_eq!(step.grid_import_quanta, 0);
                    }
                    assert_eq!(
                        u64::from(step.pv_used_quanta)
                            + u64::from(step.grid_import_quanta)
                            + u64::from(step.battery_discharge_delivered_quanta),
                        u64::from(step.essential_served_quanta)
                            + u64::from(step.flexible_served_quanta)
                            + u64::from(step.battery_charge_input_quanta)
                    );
                    assert_eq!(
                        step.pv_used_quanta + step.pv_curtailed_quanta,
                        step.pv_available_quanta
                    );
                    assert_eq!(
                        step.essential_served_quanta + step.essential_unserved_quanta,
                        step.essential_demand_quanta
                    );
                    assert_eq!(
                        step.flexible_served_quanta + step.flexible_unserved_quanta,
                        step.flexible_demand_quanta
                    );
                    assert_eq!(step.bus_energy_balance_residual_quanta, 0);
                    expected_soc = step.battery_soc_after_quanta;
                }
                assert!(expected_soc >= request.battery.terminal_reserve_quanta);
                for (other_index, other) in assessment.pareto_alternatives.iter().enumerate() {
                    if index != other_index {
                        assert!(!alternative.objective.strictly_dominates(&other.objective));
                    }
                }
            }

            let encoded_first = serde_json::to_vec(&assessment).expect("assessment serializes");
            let rerun = assess_energy_dispatch(&request).expect("same case remains feasible");
            let encoded_second = serde_json::to_vec(&rerun).expect("rerun serializes");
            assert_eq!(encoded_first, encoded_second, "seed {seed} must be deterministic");
        }
    }

    #[test]
    fn non_assumption_evidence_requires_a_reference() {
        let mut request = fixture_request();
        request.evidence_class = EvidenceClass::PublishedModel;
        request.evidence_ref = None;
        assert!(matches!(
            assess_energy_dispatch(&request),
            Err(EnergyDispatchError::MissingEvidenceReference)
        ));
    }
}


/// Deliberately simple test-only reference enumerator. It shares the request and
/// objective data types, but does not call the production recursive search,
/// objective calculator, Pareto helper, or independent schedule verifier.
#[cfg(test)]
mod independent_reference_tests {
    use super::*;
    use std::collections::BTreeSet;

    type ObjectiveKey = TestObjectiveVector;

    const REFERENCE_EXPANSION_BUDGET: u64 = 500_000;

    #[derive(Debug, Default)]
    struct ReferenceResult {
        expansion_limit: u64,
        feasible_complete_schedules: u64,
        explored_local_actions: u64,
        all_objective_vectors: BTreeSet<ObjectiveKey>,
        pareto_objectives: BTreeSet<ObjectiveKey>,
    }

    impl ReferenceResult {
        fn new(expansion_limit: u64) -> Self {
            Self {
                expansion_limit,
                feasible_complete_schedules: 0,
                explored_local_actions: 0,
                all_objective_vectors: BTreeSet::new(),
                pareto_objectives: BTreeSet::new(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ReferenceError {
        BudgetExceeded { limit: u64 },
        CounterOverflow,
    }

    fn key_from_objective(objective: &DispatchObjective) -> ObjectiveKey {
        (
            objective.essential_unserved_quanta,
            objective.flexible_unserved_quanta,
            objective.operating_cost_units,
            objective.grid_carbon_grams,
            objective.curtailed_pv_quanta,
            objective.terminal_soc_quanta,
        )
    }

    fn reference_dominates(a: &ObjectiveKey, b: &ObjectiveKey) -> bool {
        let no_worse = a.0 <= b.0
            && a.1 <= b.1
            && a.2 <= b.2
            && a.3 <= b.3
            && a.4 <= b.4
            && a.5 >= b.5;
        let strictly_better = a.0 < b.0
            || a.1 < b.1
            || a.2 < b.2
            || a.3 < b.3
            || a.4 < b.4
            || a.5 > b.5;
        no_worse && strictly_better
    }

    fn spend_reference_action(result: &mut ReferenceResult) -> Result<(), ReferenceError> {
        result.explored_local_actions = result
            .explored_local_actions
            .checked_add(1)
            .ok_or(ReferenceError::CounterOverflow)?;
        if result.explored_local_actions > result.expansion_limit {
            return Err(ReferenceError::BudgetExceeded {
                limit: result.expansion_limit,
            });
        }
        Ok(())
    }

    /// Re-enumerate every finite action tuple for each interval, then compose
    /// feasible transitions chronologically. This intentionally favors clarity
    /// over the production oracle's in-place Pareto pruning.
    fn reference_visit(
        request: &EnergyDispatchRequest,
        interval_index: usize,
        soc_at_start: u16,
        accumulated: (u64, u64, u64, u64, u64),
        result: &mut ReferenceResult,
    ) -> Result<(), ReferenceError> {
        if interval_index == request.intervals.len() {
            if soc_at_start >= request.battery.terminal_reserve_quanta {
                result.feasible_complete_schedules = result
                    .feasible_complete_schedules
                    .checked_add(1)
                    .ok_or(ReferenceError::CounterOverflow)?;
                let objective = (
                    accumulated.0,
                    accumulated.1,
                    accumulated.2,
                    accumulated.3,
                    accumulated.4,
                    soc_at_start,
                );
                result.all_objective_vectors.insert(objective);
            }
            return Ok(());
        }

        let interval = &request.intervals[interval_index];
        let battery = &request.battery;
        let allowed_grid = if interval.grid_outage {
            0
        } else {
            interval.grid_import_limit_quanta
        };
        let allowed_withdrawal =
            std::cmp::min(soc_at_start, battery.discharge_withdrawal_limit_quanta);

        for grid_import in 0..=allowed_grid {
            for soc_withdrawal in 0..=allowed_withdrawal {
                for charge_input in 0..=battery.charge_input_limit_quanta {
                    if soc_withdrawal != 0 && charge_input != 0 {
                        continue;
                    }

                    let charge_gain = (u64::from(charge_input)
                        * u64::from(battery.charge_efficiency_numerator))
                        / u64::from(battery.charge_efficiency_denominator);
                    let soc_after = u32::from(soc_at_start)
                        - u32::from(soc_withdrawal)
                        + u32::try_from(charge_gain).expect("small reference case");
                    if soc_after > u32::from(battery.capacity_quanta) {
                        continue;
                    }

                    let discharge_delivery = (u64::from(soc_withdrawal)
                        * u64::from(battery.discharge_efficiency_numerator))
                        / u64::from(battery.discharge_efficiency_denominator);
                    let non_pv_supply = u64::from(grid_import) + discharge_delivery;

                    for essential_served in 0..=interval.essential_demand_quanta {
                        for flexible_served in 0..=interval.flexible_demand_quanta {
                            spend_reference_action(result)?;

                            let consumption_and_charge = u64::from(essential_served)
                                + u64::from(flexible_served)
                                + u64::from(charge_input);
                            // Grid and battery are not curtailable in this model.
                            // The residual is therefore the only admissible PV use.
                            let Some(pv_used) =
                                consumption_and_charge.checked_sub(non_pv_supply)
                            else {
                                continue;
                            };
                            if pv_used > u64::from(interval.pv_available_quanta) {
                                continue;
                            }

                            let pv_spill =
                                u64::from(interval.pv_available_quanta) - pv_used;
                            let next_accumulated = (
                                accumulated.0
                                    + u64::from(
                                        interval.essential_demand_quanta - essential_served,
                                    ),
                                accumulated.1
                                    + u64::from(
                                        interval.flexible_demand_quanta - flexible_served,
                                    ),
                                accumulated.2
                                    + u64::from(grid_import)
                                        * u64::from(interval.grid_cost_units_per_quantum),
                                accumulated.3
                                    + u64::from(grid_import)
                                        * u64::from(interval.grid_carbon_grams_per_quantum),
                                accumulated.4 + pv_spill,
                            );
                            reference_visit(
                                request,
                                interval_index + 1,
                                u16::try_from(soc_after).expect("SOC is capacity-bounded"),
                                next_accumulated,
                                result,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn enumerate_reference(
        request: &EnergyDispatchRequest,
    ) -> Result<ReferenceResult, ReferenceError> {
        enumerate_reference_with_budget(request, REFERENCE_EXPANSION_BUDGET)
    }

    fn enumerate_reference_with_budget(
        request: &EnergyDispatchRequest,
        expansion_limit: u64,
    ) -> Result<ReferenceResult, ReferenceError> {
        let mut result = ReferenceResult::new(expansion_limit);
        reference_visit(
            request,
            0,
            request.battery.initial_soc_quanta,
            (0, 0, 0, 0, 0),
            &mut result,
        )?;
        result.pareto_objectives = result
            .all_objective_vectors
            .iter()
            .filter(|candidate| {
                !result
                    .all_objective_vectors
                    .iter()
                    .any(|challenger| reference_dominates(challenger, candidate))
            })
            .copied()
            .collect();
        Ok(result)
    }

    fn production_frontier_keys(assessment: &EnergyDispatchAssessment) -> BTreeSet<ObjectiveKey> {
        assessment
            .pareto_alternatives
            .iter()
            .map(|alternative| key_from_objective(&alternative.objective))
            .collect()
    }

    fn compare_assessment_to_reference(
        request: &EnergyDispatchRequest,
        assessment: &EnergyDispatchAssessment,
        reference: &ReferenceResult,
        production_objective_vectors: &BTreeSet<ObjectiveKey>,
    ) -> Result<(), String> {
        if reference.feasible_complete_schedules == 0 {
            return Err("assessment supplied for a reference-infeasible request".to_owned());
        }
        if !assessment.search_is_exact || !assessment.pareto_frontier_complete {
            return Err("production did not label the result exact and complete".to_owned());
        }
        if assessment.case_id != request.case_id {
            return Err("assessment case identity differs from request".to_owned());
        }
        if assessment.feasible_complete_schedules != reference.feasible_complete_schedules {
            return Err(format!(
                "feasible schedule count differs: production={}, reference={}",
                assessment.feasible_complete_schedules,
                reference.feasible_complete_schedules
            ));
        }
        if production_objective_vectors != &reference.all_objective_vectors {
            return Err(format!(
                "complete feasible objective-vector sets differ: production={production_objective_vectors:?}, reference={:?}",
                reference.all_objective_vectors
            ));
        }
        let production_keys = production_frontier_keys(assessment);
        if production_keys.len() != assessment.pareto_alternatives.len() {
            return Err("production frontier contains duplicate objective vectors".to_owned());
        }
        if production_keys != reference.pareto_objectives {
            return Err(format!(
                "Pareto objective sets differ: production={production_keys:?}, reference={:?}",
                reference.pareto_objectives
            ));
        }
        Ok(())
    }

    fn production_objective_vectors(
        request: &EnergyDispatchRequest,
    ) -> Result<BTreeSet<ObjectiveKey>, String> {
        // The public assessment validates the request first. This second test-only
        // search captures every feasible objective vector before Pareto pruning.
        let mut search = ExactSearch {
            request,
            evaluated_action_expansions: 0,
            feasible_complete_schedules: 0,
            frontier: Vec::new(),
            all_feasible_objective_vectors: BTreeSet::new(),
        };
        let mut path = Vec::with_capacity(request.intervals.len());
        search
            .visit(0, request.battery.initial_soc_quanta, &mut path)
            .map_err(|error| format!("production objective-vector trace failed: {error:?}"))?;
        Ok(search.all_feasible_objective_vectors)
    }

    fn compare_reference_to_production(
        request: &EnergyDispatchRequest,
    ) -> Result<(), String> {
        let reference = enumerate_reference(request)
            .map_err(|error| format!("reference enumeration failed: {error:?}"))?;
        let production_objective_vectors = production_objective_vectors(request)?;
        match assess_energy_dispatch(request) {
            Ok(assessment) => {
                if reference.feasible_complete_schedules == 0 {
                    return Err(
                        "production reported a feasible assessment, reference found none"
                            .to_owned(),
                    );
                }
                compare_assessment_to_reference(
                    request,
                    &assessment,
                    &reference,
                    &production_objective_vectors,
                )
            }
            Err(EnergyDispatchError::NoFeasibleSchedules)
                if reference.feasible_complete_schedules == 0 =>
            {
                Ok(())
            }
            Err(error) => Err(format!(
                "production returned {error:?} while reference found {} feasible schedules",
                reference.feasible_complete_schedules
            )),
        }
    }

    fn small_request(interval_count: usize) -> EnergyDispatchRequest {
        let mut request: EnergyDispatchRequest = serde_json::from_str(
            include_str!("../../schemas/examples/energy-dispatch-pv-outage.synthetic.json"),
        )
        .expect("frozen reference request must parse");
        request.case_id = "independent-reference-case".to_owned();
        request.intervals.truncate(interval_count);
        request.max_candidate_expansions = MAX_CANDIDATE_EXPANSIONS;
        request.battery.capacity_quanta = 2;
        request.battery.initial_soc_quanta = 1;
        request.battery.charge_input_limit_quanta = 1;
        request.battery.discharge_withdrawal_limit_quanta = 1;
        request.battery.charge_efficiency_numerator = 1;
        request.battery.charge_efficiency_denominator = 1;
        request.battery.discharge_efficiency_numerator = 1;
        request.battery.discharge_efficiency_denominator = 1;
        request.battery.terminal_reserve_quanta = 0;
        request
    }

    #[test]
    fn reference_matches_hand_calculated_one_interval_known_answer() {
        let mut request = small_request(1);
        request.battery.initial_soc_quanta = 0;
        request.intervals[0].pv_available_quanta = 1;
        request.intervals[0].grid_outage = false;
        request.intervals[0].grid_import_limit_quanta = 1;
        request.intervals[0].essential_demand_quanta = 1;
        request.intervals[0].flexible_demand_quanta = 0;
        request.intervals[0].grid_cost_units_per_quantum = 7;
        request.intervals[0].grid_carbon_grams_per_quantum = 9;

        compare_reference_to_production(&request).expect("reference and production should agree");
        let reference = enumerate_reference(&request).expect("bounded reference case");
        assert!(reference.feasible_complete_schedules > 0);
        assert!(reference.pareto_objectives.contains(&(0, 0, 0, 0, 0, 0)));
    }

    #[test]
    fn reference_detects_a_removed_oracle_frontier_member() {
        let mut request = small_request(1);
        request.battery.capacity_quanta = 0;
        request.battery.initial_soc_quanta = 0;
        request.battery.charge_input_limit_quanta = 0;
        request.battery.discharge_withdrawal_limit_quanta = 0;
        request.intervals[0].pv_available_quanta = 0;
        request.intervals[0].grid_outage = false;
        request.intervals[0].grid_import_limit_quanta = 1;
        request.intervals[0].essential_demand_quanta = 1;
        request.intervals[0].flexible_demand_quanta = 0;
        request.intervals[0].grid_cost_units_per_quantum = 7;
        request.intervals[0].grid_carbon_grams_per_quantum = 3;

        let reference = enumerate_reference(&request).expect("bounded reference case");
        let mut corrupted = assess_energy_dispatch(&request).expect("production case is feasible");
        let production_objective_vectors =
            production_objective_vectors(&request).expect("bounded production trace");
        assert_eq!(reference.all_objective_vectors, production_objective_vectors);
        assert_eq!(reference.pareto_objectives.len(), 2);
        assert_eq!(corrupted.pareto_alternatives.len(), 2);
        corrupted.pareto_alternatives.pop();
        assert!(
            compare_assessment_to_reference(
                &request,
                &corrupted,
                &reference,
                &production_objective_vectors,
            )
            .is_err(),
            "reference comparator must detect a missing Pareto alternative"
        );
    }

    #[test]
    fn independently_audits_a_small_exhaustive_request_domain() {
        let mut case_id = 0_u64;
        for capacity in 0..=2_u16 {
            for initial_soc in 0..=capacity {
                for reserve in 0..=capacity {
                    for outage in [false, true] {
                        for pv in 0..=1_u16 {
                            for grid_limit in 0..=1_u16 {
                                for essential_demand in 0..=1_u16 {
                                    for flexible_demand in 0..=1_u16 {
                                        let mut request = small_request(1);
                                        request.case_id = format!("exhaustive-{case_id}");
                                        case_id += 1;
                                        request.battery.capacity_quanta = capacity;
                                        request.battery.initial_soc_quanta = initial_soc;
                                        request.battery.terminal_reserve_quanta = reserve;
                                        request.battery.charge_input_limit_quanta = 1;
                                        request.battery.discharge_withdrawal_limit_quanta = 1;
                                        request.intervals[0].grid_outage = outage;
                                        request.intervals[0].pv_available_quanta = pv;
                                        request.intervals[0].grid_import_limit_quanta = grid_limit;
                                        request.intervals[0].essential_demand_quanta = essential_demand;
                                        request.intervals[0].flexible_demand_quanta = flexible_demand;
                                        compare_reference_to_production(&request).unwrap_or_else(
                                            |error| panic!("case {}: {error}", request.case_id),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(case_id, 448);
    }

    #[test]
    fn independently_audits_one_thousand_deterministic_generated_cases() {
        for seed in 0..1_000_u64 {
            let interval_count = if seed % 2 == 0 { 1 } else { 2 };
            let mut request = small_request(interval_count);
            request.case_id = format!("reference-generated-{seed}");
            let capacity = (seed % 3) as u16;
            request.battery.capacity_quanta = capacity;
            request.battery.initial_soc_quanta = ((seed / 3) % u64::from(capacity + 1)) as u16;
            request.battery.terminal_reserve_quanta =
                ((seed / 5) % u64::from(capacity + 1)) as u16;
            request.battery.charge_input_limit_quanta = (seed % 2) as u16;
            request.battery.discharge_withdrawal_limit_quanta = ((seed / 7) % 2) as u16;
            request.battery.charge_efficiency_numerator = 1;
            request.battery.charge_efficiency_denominator =
                if seed % 4 == 0 { 2 } else { 1 };
            request.battery.discharge_efficiency_numerator = 1;
            request.battery.discharge_efficiency_denominator =
                if seed % 6 == 0 { 2 } else { 1 };

            for (index, interval) in request.intervals.iter_mut().enumerate() {
                let offset = seed + (index as u64 * 37);
                interval.grid_outage = offset % 5 == 0;
                interval.pv_available_quanta = (offset % 3) as u16;
                interval.grid_import_limit_quanta = ((offset / 3) % 3) as u16;
                interval.essential_demand_quanta = ((offset / 5) % 3) as u16;
                interval.flexible_demand_quanta = ((offset / 7) % 2) as u16;
                interval.grid_cost_units_per_quantum = (offset % 4) as u32;
                interval.grid_carbon_grams_per_quantum = ((offset / 2) % 5) as u32;
            }

            compare_reference_to_production(&request).unwrap_or_else(|error| {
                panic!("seed {seed}, request={request:?}: {error}");
            });
        }
    }

    #[test]
    fn reference_enumerator_budget_failure_is_explicit() {
        let request = small_request(1);
        let error = enumerate_reference_with_budget(&request, 0)
            .expect_err("reference must not return a partial frontier after its budget is hit");
        assert_eq!(error, ReferenceError::BudgetExceeded { limit: 0 });
    }
}
