// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Typed deterministic selector for a complete finite energy-dispatch Pareto frontier.
//!
//! This module is the semantic kernel for the Energy Dispatch Pareto-Selector DSL.
//! It does not parse policy source, generate schedules, change physical feasibility,
//! or authorize dispatch. The oracle owns frontier generation and hard constraints;
//! the independent verifier owns schedule rechecking.

use crate::energy_dispatch::{
    DispatchIntervalDecision, DispatchObjective, DispatchParetoAlternative,
    EnergyDispatchAssessment, MAX_PARETO_FRONTIER,
};
use std::cmp::Ordering;

pub const POLICY_SCHEMA_VERSION: &str = "energy-dispatch-policy-v0.1";
pub const ASSESSMENT_SCHEMA_VERSION: &str = "energy-dispatch-report-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DispatchObjectiveId {
    EssentialUnservedQuanta,
    FlexibleUnservedQuanta,
    OperatingCostUnits,
    GridCarbonGrams,
    CurtailedPvQuanta,
    TerminalSocQuanta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveDirection {
    Minimize,
    Maximize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveAggregation {
    HorizonTotal,
    TerminalValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchTieBreak {
    /// Lexicographically minimize the complete stable interval/action record.
    StableScheduleV1,
    /// Reserved to make unsupported serialized policies fail explicitly.
    Unsupported(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectiveTier {
    pub objective: DispatchObjectiveId,
    pub direction: ObjectiveDirection,
    pub aggregation: ObjectiveAggregation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchSelectionPolicy {
    pub policy_id: String,
    pub schema_version: String,
    pub objective_tiers: Vec<ObjectiveTier>,
    pub tie_break: DispatchTieBreak,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchSelectionError {
    EmptyPolicyId,
    UnsupportedPolicySchemaVersion { actual: String },
    EmptyObjectiveTiers,
    EssentialServiceMustBeFirst,
    DuplicateObjective { objective: DispatchObjectiveId },
    UnsupportedObjectiveSemantics {
        objective: DispatchObjectiveId,
        direction: ObjectiveDirection,
        aggregation: ObjectiveAggregation,
    },
    UnsupportedTieBreak { name: String },
    UnsupportedAssessmentSchemaVersion { actual: String },
    OracleFrontierIncomplete,
    FrontierCountMismatch { declared: usize, actual: usize },
    EmptyFrontier,
    FrontierExceedsMaximum { actual: usize, maximum: usize },
    EmptyAlternativeSchedule { index: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DispatchSelection {
    pub policy_id: String,
    pub selected_frontier_index: usize,
    pub selected_objective: DispatchObjective,
    pub selected_schedule: Vec<DispatchIntervalDecision>,
    pub frontier_count: usize,
    /// True when two or more frontier alternatives tied on every declared tier.
    pub stable_tie_break_applied: bool,
}

impl DispatchSelectionPolicy {
    /// A conservative default policy: essential service, flexible service,
    /// operating cost, carbon, curtailed PV, then terminal state of charge.
    pub fn critical_service_first_v1() -> Self {
        Self {
            policy_id: "critical-service-first-v1".to_owned(),
            schema_version: POLICY_SCHEMA_VERSION.to_owned(),
            objective_tiers: vec![
                ObjectiveTier {
                    objective: DispatchObjectiveId::EssentialUnservedQuanta,
                    direction: ObjectiveDirection::Minimize,
                    aggregation: ObjectiveAggregation::HorizonTotal,
                },
                ObjectiveTier {
                    objective: DispatchObjectiveId::FlexibleUnservedQuanta,
                    direction: ObjectiveDirection::Minimize,
                    aggregation: ObjectiveAggregation::HorizonTotal,
                },
                ObjectiveTier {
                    objective: DispatchObjectiveId::OperatingCostUnits,
                    direction: ObjectiveDirection::Minimize,
                    aggregation: ObjectiveAggregation::HorizonTotal,
                },
                ObjectiveTier {
                    objective: DispatchObjectiveId::GridCarbonGrams,
                    direction: ObjectiveDirection::Minimize,
                    aggregation: ObjectiveAggregation::HorizonTotal,
                },
                ObjectiveTier {
                    objective: DispatchObjectiveId::CurtailedPvQuanta,
                    direction: ObjectiveDirection::Minimize,
                    aggregation: ObjectiveAggregation::HorizonTotal,
                },
                ObjectiveTier {
                    objective: DispatchObjectiveId::TerminalSocQuanta,
                    direction: ObjectiveDirection::Maximize,
                    aggregation: ObjectiveAggregation::TerminalValue,
                },
            ],
            tie_break: DispatchTieBreak::StableScheduleV1,
        }
    }

    pub fn validate(&self) -> Result<(), DispatchSelectionError> {
        if self.policy_id.trim().is_empty() {
            return Err(DispatchSelectionError::EmptyPolicyId);
        }
        if self.schema_version != POLICY_SCHEMA_VERSION {
            return Err(DispatchSelectionError::UnsupportedPolicySchemaVersion {
                actual: self.schema_version.clone(),
            });
        }
        let Some(first) = self.objective_tiers.first() else {
            return Err(DispatchSelectionError::EmptyObjectiveTiers);
        };
        if first.objective != DispatchObjectiveId::EssentialUnservedQuanta
            || first.direction != ObjectiveDirection::Minimize
            || first.aggregation != ObjectiveAggregation::HorizonTotal
        {
            return Err(DispatchSelectionError::EssentialServiceMustBeFirst);
        }

        for (index, tier) in self.objective_tiers.iter().enumerate() {
            if self.objective_tiers[..index]
                .iter()
                .any(|prior| prior.objective == tier.objective)
            {
                return Err(DispatchSelectionError::DuplicateObjective {
                    objective: tier.objective,
                });
            }
            let supported = match tier.objective {
                DispatchObjectiveId::EssentialUnservedQuanta
                | DispatchObjectiveId::FlexibleUnservedQuanta
                | DispatchObjectiveId::OperatingCostUnits
                | DispatchObjectiveId::GridCarbonGrams
                | DispatchObjectiveId::CurtailedPvQuanta => {
                    tier.direction == ObjectiveDirection::Minimize
                        && tier.aggregation == ObjectiveAggregation::HorizonTotal
                }
                DispatchObjectiveId::TerminalSocQuanta => {
                    tier.direction == ObjectiveDirection::Maximize
                        && tier.aggregation == ObjectiveAggregation::TerminalValue
                }
            };
            if !supported {
                return Err(DispatchSelectionError::UnsupportedObjectiveSemantics {
                    objective: tier.objective,
                    direction: tier.direction,
                    aggregation: tier.aggregation,
                });
            }
        }

        match &self.tie_break {
            DispatchTieBreak::StableScheduleV1 => Ok(()),
            DispatchTieBreak::Unsupported(name) => {
                Err(DispatchSelectionError::UnsupportedTieBreak { name: name.clone() })
            }
        }
    }
}

/// Select one schedule from an oracle-produced, complete Pareto frontier.
///
/// The assessment must be the direct output of the exact bounded oracle for the
/// case being evaluated. The completion flags are not cryptographic evidence;
/// callers must bind the assessment to its exact request/oracle identity in the
/// surrounding study/receipt layer. The selected schedule should still be passed
/// to `verify_dispatch_schedule` before any downstream use.
pub fn select_dispatch_alternative(
    policy: &DispatchSelectionPolicy,
    assessment: &EnergyDispatchAssessment,
) -> Result<DispatchSelection, DispatchSelectionError> {
    policy.validate()?;

    if assessment.assessment_schema_version != ASSESSMENT_SCHEMA_VERSION {
        return Err(DispatchSelectionError::UnsupportedAssessmentSchemaVersion {
            actual: assessment.assessment_schema_version.clone(),
        });
    }
    if !assessment.search_is_exact || !assessment.pareto_frontier_complete {
        return Err(DispatchSelectionError::OracleFrontierIncomplete);
    }
    let actual_count = assessment.pareto_alternatives.len();
    if assessment.pareto_frontier_count != actual_count {
        return Err(DispatchSelectionError::FrontierCountMismatch {
            declared: assessment.pareto_frontier_count,
            actual: actual_count,
        });
    }
    if actual_count == 0 {
        return Err(DispatchSelectionError::EmptyFrontier);
    }
    if actual_count > MAX_PARETO_FRONTIER {
        return Err(DispatchSelectionError::FrontierExceedsMaximum {
            actual: actual_count,
            maximum: MAX_PARETO_FRONTIER,
        });
    }
    for (index, alternative) in assessment.pareto_alternatives.iter().enumerate() {
        if alternative.schedule.is_empty() {
            return Err(DispatchSelectionError::EmptyAlternativeSchedule { index });
        }
    }

    let mut best_indices = vec![0_usize];
    for candidate_index in 1..actual_count {
        match compare_objectives(
            &assessment.pareto_alternatives[candidate_index].objective,
            &assessment.pareto_alternatives[best_indices[0]].objective,
            &policy.objective_tiers,
        ) {
            Ordering::Less => {
                best_indices.clear();
                best_indices.push(candidate_index);
            }
            Ordering::Equal => best_indices.push(candidate_index),
            Ordering::Greater => {}
        }
    }

    // The stable key is evaluated only inside the best equivalence class of the
    // declared objective tiers. It never overrules a declared objective.
    let selected_index = *best_indices
        .iter()
        .min_by(|left, right| {
            compare_schedules(
                &assessment.pareto_alternatives[**left].schedule,
                &assessment.pareto_alternatives[**right].schedule,
            )
        })
        .expect("a non-empty frontier always has a best index");

    let selected: &DispatchParetoAlternative =
        &assessment.pareto_alternatives[selected_index];

    Ok(DispatchSelection {
        policy_id: policy.policy_id.clone(),
        selected_frontier_index: selected_index,
        selected_objective: selected.objective.clone(),
        selected_schedule: selected.schedule.clone(),
        frontier_count: actual_count,
        stable_tie_break_applied: best_indices.len() > 1,
    })
}

fn compare_objectives(
    left: &DispatchObjective,
    right: &DispatchObjective,
    tiers: &[ObjectiveTier],
) -> Ordering {
    for tier in tiers {
        let left_value = objective_value(left, tier.objective);
        let right_value = objective_value(right, tier.objective);
        let comparison = left_value.cmp(&right_value);
        let comparison = match tier.direction {
            ObjectiveDirection::Minimize => comparison,
            ObjectiveDirection::Maximize => comparison.reverse(),
        };
        if comparison != Ordering::Equal {
            return comparison;
        }
    }
    Ordering::Equal
}

fn objective_value(objective: &DispatchObjective, id: DispatchObjectiveId) -> u64 {
    match id {
        DispatchObjectiveId::EssentialUnservedQuanta => objective.essential_unserved_quanta,
        DispatchObjectiveId::FlexibleUnservedQuanta => objective.flexible_unserved_quanta,
        DispatchObjectiveId::OperatingCostUnits => objective.operating_cost_units,
        DispatchObjectiveId::GridCarbonGrams => objective.grid_carbon_grams,
        DispatchObjectiveId::CurtailedPvQuanta => objective.curtailed_pv_quanta,
        DispatchObjectiveId::TerminalSocQuanta => u64::from(objective.terminal_soc_quanta),
    }
}

/// Versioned stable schedule ordering. The sequence is chronological and is
/// compared field-by-field using only exact strings, integers and booleans.
fn compare_schedules(
    left: &[DispatchIntervalDecision],
    right: &[DispatchIntervalDecision],
) -> Ordering {
    for (left_step, right_step) in left.iter().zip(right) {
        let comparison = compare_steps(left_step, right_step);
        if comparison != Ordering::Equal {
            return comparison;
        }
    }
    left.len().cmp(&right.len())
}

fn compare_steps(
    left: &DispatchIntervalDecision,
    right: &DispatchIntervalDecision,
) -> Ordering {
    left.interval_id
        .cmp(&right.interval_id)
        .then_with(|| left.start_unix_seconds.cmp(&right.start_unix_seconds))
        .then_with(|| left.duration_seconds.cmp(&right.duration_seconds))
        .then_with(|| left.grid_outage.cmp(&right.grid_outage))
        .then_with(|| left.pv_available_quanta.cmp(&right.pv_available_quanta))
        .then_with(|| left.pv_used_quanta.cmp(&right.pv_used_quanta))
        .then_with(|| left.pv_curtailed_quanta.cmp(&right.pv_curtailed_quanta))
        .then_with(|| left.grid_import_quanta.cmp(&right.grid_import_quanta))
        .then_with(|| {
            left.battery_soc_before_quanta
                .cmp(&right.battery_soc_before_quanta)
        })
        .then_with(|| {
            left.battery_charge_input_quanta
                .cmp(&right.battery_charge_input_quanta)
        })
        .then_with(|| {
            left.battery_soc_withdrawal_quanta
                .cmp(&right.battery_soc_withdrawal_quanta)
        })
        .then_with(|| {
            left.battery_discharge_delivered_quanta
                .cmp(&right.battery_discharge_delivered_quanta)
        })
        .then_with(|| left.battery_soc_after_quanta.cmp(&right.battery_soc_after_quanta))
        .then_with(|| left.essential_demand_quanta.cmp(&right.essential_demand_quanta))
        .then_with(|| left.essential_served_quanta.cmp(&right.essential_served_quanta))
        .then_with(|| left.essential_unserved_quanta.cmp(&right.essential_unserved_quanta))
        .then_with(|| left.flexible_demand_quanta.cmp(&right.flexible_demand_quanta))
        .then_with(|| left.flexible_served_quanta.cmp(&right.flexible_served_quanta))
        .then_with(|| left.flexible_unserved_quanta.cmp(&right.flexible_unserved_quanta))
        .then_with(|| {
            left.grid_cost_units_per_quantum
                .cmp(&right.grid_cost_units_per_quantum)
        })
        .then_with(|| {
            left.grid_carbon_grams_per_quantum
                .cmp(&right.grid_carbon_grams_per_quantum)
        })
        .then_with(|| {
            left.bus_energy_balance_residual_quanta
                .cmp(&right.bus_energy_balance_residual_quanta)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy_dispatch::{EnergyDispatchRequest, assess_energy_dispatch};
    use crate::energy_dispatch_verifier::verify_dispatch_schedule;

    const FIXTURE: &str =
        include_str!("../../schemas/examples/energy-dispatch-pv-outage.synthetic.json");

    fn frozen_assessment() -> (EnergyDispatchRequest, EnergyDispatchAssessment) {
        let request: EnergyDispatchRequest =
            serde_json::from_str(FIXTURE).expect("frozen dispatch request must parse");
        let assessment =
            assess_energy_dispatch(&request).expect("exact finite oracle should solve fixture");
        (request, assessment)
    }

    fn policy_with(objective_tiers: Vec<ObjectiveTier>) -> DispatchSelectionPolicy {
        DispatchSelectionPolicy {
            policy_id: "test-policy".to_owned(),
            schema_version: POLICY_SCHEMA_VERSION.to_owned(),
            objective_tiers,
            tie_break: DispatchTieBreak::StableScheduleV1,
        }
    }

    fn essential_and_cost_tiers() -> Vec<ObjectiveTier> {
        vec![
            ObjectiveTier {
                objective: DispatchObjectiveId::EssentialUnservedQuanta,
                direction: ObjectiveDirection::Minimize,
                aggregation: ObjectiveAggregation::HorizonTotal,
            },
            ObjectiveTier {
                objective: DispatchObjectiveId::OperatingCostUnits,
                direction: ObjectiveDirection::Minimize,
                aggregation: ObjectiveAggregation::HorizonTotal,
            },
        ]
    }

    #[test]
    fn essential_service_outweighs_any_cost_improvement() {
        let (_request, mut assessment) = frozen_assessment();
        let mut costly_but_served = assessment.pareto_alternatives[0].clone();
        let mut cheap_but_unserved = costly_but_served.clone();
        costly_but_served.objective.essential_unserved_quanta = 0;
        costly_but_served.objective.operating_cost_units = u64::MAX;
        cheap_but_unserved.objective.essential_unserved_quanta = 1;
        cheap_but_unserved.objective.operating_cost_units = 0;
        assessment.pareto_alternatives = vec![cheap_but_unserved, costly_but_served];
        assessment.pareto_frontier_count = 2;

        let policy = policy_with(essential_and_cost_tiers());
        let selection =
            select_dispatch_alternative(&policy, &assessment).expect("policy should select");
        assert_eq!(selection.selected_objective.essential_unserved_quanta, 0);
        assert_eq!(selection.selected_objective.operating_cost_units, u64::MAX);
        assert_eq!(selection.selected_frontier_index, 1);
    }

    #[test]
    fn cost_breaks_ties_only_after_essential_service() {
        let (_request, mut assessment) = frozen_assessment();
        let mut higher_cost = assessment.pareto_alternatives[0].clone();
        let mut lower_cost = higher_cost.clone();
        higher_cost.objective.essential_unserved_quanta = 0;
        higher_cost.objective.operating_cost_units = 11;
        lower_cost.objective.essential_unserved_quanta = 0;
        lower_cost.objective.operating_cost_units = 8;
        assessment.pareto_alternatives = vec![higher_cost, lower_cost];
        assessment.pareto_frontier_count = 2;

        let policy = policy_with(essential_and_cost_tiers());
        let selection =
            select_dispatch_alternative(&policy, &assessment).expect("policy should select");
        assert_eq!(selection.selected_objective.operating_cost_units, 8);
    }

    #[test]
    fn stable_tie_break_is_independent_of_frontier_input_order() {
        let (_request, mut assessment) = frozen_assessment();
        let mut later_schedule = assessment.pareto_alternatives[0].clone();
        let mut earlier_schedule = later_schedule.clone();
        later_schedule.objective.essential_unserved_quanta = 0;
        earlier_schedule.objective.essential_unserved_quanta = 0;
        later_schedule.objective.operating_cost_units = 10;
        earlier_schedule.objective.operating_cost_units = 10;
        later_schedule.schedule[0].interval_id = "z-interval".to_owned();
        earlier_schedule.schedule[0].interval_id = "a-interval".to_owned();
        assessment.pareto_alternatives = vec![later_schedule.clone(), earlier_schedule.clone()];
        assessment.pareto_frontier_count = 2;

        let policy = policy_with(essential_and_cost_tiers());
        let first =
            select_dispatch_alternative(&policy, &assessment).expect("policy should select");
        assert!(first.stable_tie_break_applied);
        assert_eq!(first.selected_schedule[0].interval_id, "a-interval");

        assessment.pareto_alternatives = vec![earlier_schedule, later_schedule];
        let second =
            select_dispatch_alternative(&policy, &assessment).expect("policy should select");
        assert!(second.stable_tie_break_applied);
        assert_eq!(second.selected_schedule[0].interval_id, "a-interval");
        assert_eq!(first.selected_objective, second.selected_objective);
    }

    #[test]
    fn policy_must_put_essential_service_first() {
        let policy = policy_with(vec![ObjectiveTier {
            objective: DispatchObjectiveId::OperatingCostUnits,
            direction: ObjectiveDirection::Minimize,
            aggregation: ObjectiveAggregation::HorizonTotal,
        }]);
        assert_eq!(
            policy.validate(),
            Err(DispatchSelectionError::EssentialServiceMustBeFirst)
        );
    }

    #[test]
    fn policy_rejects_duplicate_objective_tiers() {
        let mut tiers = essential_and_cost_tiers();
        tiers.push(tiers[1].clone());
        let policy = policy_with(tiers);
        assert!(matches!(
            policy.validate(),
            Err(DispatchSelectionError::DuplicateObjective { .. })
        ));
    }

    #[test]
    fn policy_rejects_unregistered_direction_and_aggregation() {
        let policy = policy_with(vec![
            essential_and_cost_tiers()[0].clone(),
            ObjectiveTier {
                objective: DispatchObjectiveId::TerminalSocQuanta,
                direction: ObjectiveDirection::Minimize,
                aggregation: ObjectiveAggregation::HorizonTotal,
            },
        ]);
        assert!(matches!(
            policy.validate(),
            Err(DispatchSelectionError::UnsupportedObjectiveSemantics { .. })
        ));
    }

    #[test]
    fn selector_rejects_incomplete_frontier() {
        let (_request, mut assessment) = frozen_assessment();
        assessment.pareto_frontier_complete = false;
        assert_eq!(
            select_dispatch_alternative(
                &DispatchSelectionPolicy::critical_service_first_v1(),
                &assessment
            ),
            Err(DispatchSelectionError::OracleFrontierIncomplete)
        );
    }

    #[test]
    fn selector_rejects_frontier_count_mismatch() {
        let (_request, mut assessment) = frozen_assessment();
        assessment.pareto_frontier_count += 1;
        assert!(matches!(
            select_dispatch_alternative(
                &DispatchSelectionPolicy::critical_service_first_v1(),
                &assessment
            ),
            Err(DispatchSelectionError::FrontierCountMismatch { .. })
        ));
    }

    #[test]
    fn selector_rejects_empty_frontier() {
        let (_request, mut assessment) = frozen_assessment();
        assessment.pareto_alternatives.clear();
        assessment.pareto_frontier_count = 0;
        assert_eq!(
            select_dispatch_alternative(
                &DispatchSelectionPolicy::critical_service_first_v1(),
                &assessment
            ),
            Err(DispatchSelectionError::EmptyFrontier)
        );
    }

    #[test]
    fn selected_real_fixture_schedule_passes_independent_verification() {
        let (request, assessment) = frozen_assessment();
        let selection = select_dispatch_alternative(
            &DispatchSelectionPolicy::critical_service_first_v1(),
            &assessment,
        )
        .expect("complete frontier should be selectable");
        verify_dispatch_schedule(&request, &selection.selected_schedule)
            .expect("independent verifier must accept the selected schedule");
    }
}
