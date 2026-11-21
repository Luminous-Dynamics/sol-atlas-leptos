// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Robust comparison of the same intervention plans across a declared scenario set.
//!
//! Scenario counts are unweighted case counts, not event probabilities. Plans are
//! ranked by worst-case conservative flow improvement, then the number of scenarios
//! improved, then fewer worsened scenarios and stable plan ID. This is decision
//! support, not a capital-budget optimizer, causal estimate, or welfare score.

use crate::service_network::{NetworkTopologyCompleteness, ServiceCapacityNetworkScenario};
use crate::service_network_intervention::{
    ServiceNetworkInterventionComparison, ServiceNetworkInterventionError,
    ServiceNetworkInterventionPlan, ServiceNetworkInterventionRequest,
    compare_service_network_interventions,
};
use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SERVICE_NETWORK_PORTFOLIO_SCHEMA_VERSION: &str = "0.1.0";
pub const NETWORK_PORTFOLIO_RANKING_METHOD: &str = "worst_case_conservative_capacity_gain_desc_then_scenarios_improved_desc_then_scenarios_worsened_asc_then_plan_id_asc";
pub const NETWORK_PORTFOLIO_PARETO_OBJECTIVES: [&str; 4] = [
    "worst_case_conservative_capacity_gain:maximize",
    "minimum_post_intervention_conservative_capacity:maximize",
    "scenarios_improved:maximize",
    "scenarios_worsened:minimize",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkPortfolioRequest {
    pub comparison_id: String,
    /// Alternate conditions for the same service/network topology.
    pub scenarios: Vec<ServiceCapacityNetworkScenario>,
    /// Same intervention candidates evaluated independently against each scenario.
    pub plans: Vec<ServiceNetworkInterventionPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkLinkScenarioFrequency {
    pub link_id: String,
    pub scenario_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkPlanRobustness {
    /// Deterministic display rank from ranking_method; not a universal utility score.
    pub rank: usize,
    /// Non-dominated layer: 1 is the Pareto frontier, 2 is dominated only by front 1, etc.
    pub pareto_front: usize,
    /// All candidate plans that Pareto-dominate this plan, sorted by plan ID.
    pub pareto_dominated_by_plan_ids: Vec<String>,
    pub plan_id: String,
    pub rationale: String,
    pub scenario_count: usize,
    /// Minimum conservative flow delta across all declared scenarios.
    pub worst_case_conservative_capacity_gain: f64,
    /// Minimum optimistic flow delta across all declared scenarios.
    pub worst_case_optimistic_capacity_gain: f64,
    /// Highest conservative flow delta across all declared scenarios.
    pub best_case_conservative_capacity_gain: f64,
    pub scenarios_improved: usize,
    pub scenarios_unchanged: usize,
    pub scenarios_worsened: usize,
    /// Minimum conservative post-intervention flow across all scenarios.
    pub minimum_post_intervention_conservative_capacity: f64,
    /// Link IDs present in the baseline lower-bound minimum cut, counted by scenario.
    pub recurring_baseline_cut_links: Vec<NetworkLinkScenarioFrequency>,
    /// Link IDs remaining in the post-intervention lower-bound minimum cut.
    pub recurring_post_intervention_cut_links: Vec<NetworkLinkScenarioFrequency>,
    /// Complete per-scenario evidence-traceable comparisons, sorted by scenario ID.
    pub scenario_comparisons: Vec<ServiceNetworkInterventionComparison>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkPortfolioComparison {
    pub schema_version: String,
    pub comparison_id: String,
    pub service_entity_id: String,
    pub unit: String,
    pub scenario_count: usize,
    pub plan_count: usize,
    /// Scenario IDs in stable sorted order. No implied probabilities or weights.
    pub scenario_ids: Vec<String>,
    pub ranking_method: String,
    /// Explicit objective directions used for Pareto diagnostics; scenarios remain unweighted.
    pub pareto_objectives: Vec<String>,
    pub plans: Vec<ServiceNetworkPlanRobustness>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceNetworkPortfolioValidationError {
    EmptyComparisonId,
    EmptyScenarioSet,
    DuplicateScenarioId,
    IncompleteScenarioTopology,
    MixedServiceEntity,
    MixedCapacityUnit,
    DifferentSourceSets,
    DifferentLinkTopology,
    EmptyPlanSet,
    DuplicatePlanId,
    ComparisonMatrixTooLarge,
    NonFiniteAggregate,
    DuplicateScenarioResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceNetworkPortfolioError {
    InvalidInput(Vec<ServiceNetworkPortfolioValidationError>),
    InterventionComparison(ServiceNetworkInterventionError),
}

/// Evaluate candidate plans across a common topology under multiple explicit scenarios.
///
/// All scenarios must have complete topology, the same target service, unit, source
/// set and link ID/end-point structure. Capacities and availability may differ.
/// Each plan is applied independently to each scenario. The rank is based on the
/// worst-case gain in conservative max flow; scenario frequencies are not probabilities.
pub fn compare_service_network_portfolio(
    request: &ServiceNetworkPortfolioRequest,
    evidence_records: &[EvidenceRecord],
) -> Result<ServiceNetworkPortfolioComparison, ServiceNetworkPortfolioError> {
    let mut errors = Vec::new();
    if request.comparison_id.trim().is_empty() {
        errors.push(ServiceNetworkPortfolioValidationError::EmptyComparisonId);
    }
    if request.scenarios.is_empty() {
        errors.push(ServiceNetworkPortfolioValidationError::EmptyScenarioSet);
    }
    if request.plans.is_empty() {
        errors.push(ServiceNetworkPortfolioValidationError::EmptyPlanSet);
    }

    let mut scenario_ids = BTreeSet::new();
    for scenario in &request.scenarios {
        if !scenario_ids.insert(scenario.id.as_str()) {
            errors.push(ServiceNetworkPortfolioValidationError::DuplicateScenarioId);
        }
        if scenario.topology_completeness != NetworkTopologyCompleteness::Complete {
            errors.push(ServiceNetworkPortfolioValidationError::IncompleteScenarioTopology);
        }
    }

    let mut plan_ids = BTreeSet::new();
    for plan in &request.plans {
        if !plan_ids.insert(plan.id.as_str()) {
            errors.push(ServiceNetworkPortfolioValidationError::DuplicatePlanId);
        }
    }

    if let Some(first) = request.scenarios.first() {
        let expected_sources: BTreeSet<&str> =
            first.source_entity_ids.iter().map(String::as_str).collect();
        let expected_topology: BTreeMap<&str, (&str, &str)> = first
            .links
            .iter()
            .map(|link| {
                (
                    link.id.as_str(),
                    (link.from_entity_id.as_str(), link.to_entity_id.as_str()),
                )
            })
            .collect();
        for scenario in &request.scenarios {
            if scenario.service_entity_id != first.service_entity_id {
                errors.push(ServiceNetworkPortfolioValidationError::MixedServiceEntity);
            }
            if scenario.required_capacity.quantity.unit != first.required_capacity.quantity.unit {
                errors.push(ServiceNetworkPortfolioValidationError::MixedCapacityUnit);
            }
            let sources: BTreeSet<&str> = scenario
                .source_entity_ids
                .iter()
                .map(String::as_str)
                .collect();
            if sources != expected_sources {
                errors.push(ServiceNetworkPortfolioValidationError::DifferentSourceSets);
            }
            let topology: BTreeMap<&str, (&str, &str)> = scenario
                .links
                .iter()
                .map(|link| {
                    (
                        link.id.as_str(),
                        (link.from_entity_id.as_str(), link.to_entity_id.as_str()),
                    )
                })
                .collect();
            if topology != expected_topology {
                errors.push(ServiceNetworkPortfolioValidationError::DifferentLinkTopology);
            }
        }
    }

    if request
        .scenarios
        .len()
        .checked_mul(request.plans.len())
        .is_none()
    {
        errors.push(ServiceNetworkPortfolioValidationError::ComparisonMatrixTooLarge);
    }

    if !errors.is_empty() {
        errors.sort_by_key(|error| *error as u8);
        errors.dedup();
        return Err(ServiceNetworkPortfolioError::InvalidInput(errors));
    }

    let mut scenario_ids: Vec<String> = request
        .scenarios
        .iter()
        .map(|scenario| scenario.id.clone())
        .collect();
    scenario_ids.sort();

    let mut ordered_scenarios: Vec<_> = request.scenarios.iter().collect();
    ordered_scenarios.sort_by(|left, right| left.id.cmp(&right.id));
    let requests: Vec<ServiceNetworkInterventionRequest> = ordered_scenarios
        .iter()
        .flat_map(|scenario| {
            request
                .plans
                .iter()
                .map(move |plan| ServiceNetworkInterventionRequest {
                    scenario: (**scenario).clone(),
                    plan: plan.clone(),
                })
        })
        .collect();

    let comparisons = compare_service_network_interventions(&requests, evidence_records)
        .map_err(ServiceNetworkPortfolioError::InterventionComparison)?;
    let mut by_key = BTreeMap::new();
    for comparison in comparisons {
        let key = (
            comparison.intervention_id.clone(),
            comparison.baseline_scenario_id.clone(),
        );
        if by_key.insert(key, comparison).is_some() {
            return Err(ServiceNetworkPortfolioError::InvalidInput(vec![
                ServiceNetworkPortfolioValidationError::DuplicateScenarioResult,
            ]));
        }
    }

    let mut plan_results = Vec::with_capacity(request.plans.len());
    for plan in &request.plans {
        let mut scenario_comparisons = Vec::with_capacity(request.scenarios.len());
        for scenario_id in &scenario_ids {
            let key = (plan.id.clone(), scenario_id.clone());
            let Some(comparison) = by_key.remove(&key) else {
                return Err(ServiceNetworkPortfolioError::InvalidInput(vec![
                    ServiceNetworkPortfolioValidationError::DuplicateScenarioResult,
                ]));
            };
            scenario_comparisons.push(comparison);
        }
        let deltas: Vec<f64> = scenario_comparisons
            .iter()
            .map(|comparison| comparison.conservative_capacity_delta)
            .collect();
        let optimistic_deltas: Vec<f64> = scenario_comparisons
            .iter()
            .map(|comparison| comparison.optimistic_capacity_delta)
            .collect();
        let minimum_post_intervention_conservative_capacity = scenario_comparisons
            .iter()
            .map(|comparison| comparison.with_intervention.conservative_available_capacity)
            .fold(f64::INFINITY, f64::min);
        let worst_case_conservative_capacity_gain =
            deltas.iter().copied().fold(f64::INFINITY, f64::min);
        let worst_case_optimistic_capacity_gain = optimistic_deltas
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let best_case_conservative_capacity_gain =
            deltas.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        if !minimum_post_intervention_conservative_capacity.is_finite()
            || !worst_case_conservative_capacity_gain.is_finite()
            || !worst_case_optimistic_capacity_gain.is_finite()
            || !best_case_conservative_capacity_gain.is_finite()
        {
            return Err(ServiceNetworkPortfolioError::InvalidInput(vec![
                ServiceNetworkPortfolioValidationError::NonFiniteAggregate,
            ]));
        }

        let mut scenarios_improved = 0;
        let mut scenarios_unchanged = 0;
        let mut scenarios_worsened = 0;
        let mut baseline_cut_counts = BTreeMap::<String, usize>::new();
        let mut post_cut_counts = BTreeMap::<String, usize>::new();

        for comparison in &scenario_comparisons {
            if comparison.conservative_capacity_delta > 0.0
                && comparison.optimistic_capacity_delta >= 0.0
            {
                scenarios_improved += 1;
            } else if comparison.conservative_capacity_delta == 0.0
                && comparison.optimistic_capacity_delta == 0.0
            {
                scenarios_unchanged += 1;
            } else if comparison.conservative_capacity_delta < 0.0
                || comparison.optimistic_capacity_delta < 0.0
            {
                scenarios_worsened += 1;
            }
            for link_id in &comparison.baseline.conservative_min_cut_link_ids {
                *baseline_cut_counts.entry(link_id.clone()).or_default() += 1;
            }
            for link_id in &comparison.with_intervention.conservative_min_cut_link_ids {
                *post_cut_counts.entry(link_id.clone()).or_default() += 1;
            }
        }

        let mut recurring_baseline_cut_links: Vec<_> = baseline_cut_counts
            .into_iter()
            .map(|(link_id, scenario_count)| NetworkLinkScenarioFrequency {
                link_id,
                scenario_count,
            })
            .collect();
        recurring_baseline_cut_links.sort_by(|left, right| {
            right
                .scenario_count
                .cmp(&left.scenario_count)
                .then_with(|| left.link_id.cmp(&right.link_id))
        });
        let mut recurring_post_intervention_cut_links: Vec<_> = post_cut_counts
            .into_iter()
            .map(|(link_id, scenario_count)| NetworkLinkScenarioFrequency {
                link_id,
                scenario_count,
            })
            .collect();
        recurring_post_intervention_cut_links.sort_by(|left, right| {
            right
                .scenario_count
                .cmp(&left.scenario_count)
                .then_with(|| left.link_id.cmp(&right.link_id))
        });

        plan_results.push(ServiceNetworkPlanRobustness {
            rank: 0,
            pareto_front: 0,
            pareto_dominated_by_plan_ids: Vec::new(),
            plan_id: plan.id.clone(),
            rationale: plan.rationale.clone(),
            scenario_count: scenario_comparisons.len(),
            worst_case_conservative_capacity_gain,
            worst_case_optimistic_capacity_gain,
            best_case_conservative_capacity_gain,
            scenarios_improved,
            scenarios_unchanged,
            scenarios_worsened,
            minimum_post_intervention_conservative_capacity,
            recurring_baseline_cut_links,
            recurring_post_intervention_cut_links,
            scenario_comparisons,
        });
    }

    assign_pareto_fronts(&mut plan_results);

    plan_results.sort_by(|left, right| {
        right
            .worst_case_conservative_capacity_gain
            .total_cmp(&left.worst_case_conservative_capacity_gain)
            .then_with(|| right.scenarios_improved.cmp(&left.scenarios_improved))
            .then_with(|| left.scenarios_worsened.cmp(&right.scenarios_worsened))
            .then_with(|| left.plan_id.cmp(&right.plan_id))
    });
    for (index, plan) in plan_results.iter_mut().enumerate() {
        plan.rank = index + 1;
    }

    Ok(ServiceNetworkPortfolioComparison {
        schema_version: SERVICE_NETWORK_PORTFOLIO_SCHEMA_VERSION.into(),
        comparison_id: request.comparison_id.clone(),
        service_entity_id: request.scenarios[0].service_entity_id.clone(),
        unit: request.scenarios[0].required_capacity.quantity.unit.clone(),
        scenario_count: request.scenarios.len(),
        plan_count: request.plans.len(),
        scenario_ids,
        ranking_method: NETWORK_PORTFOLIO_RANKING_METHOD.into(),
        pareto_objectives: NETWORK_PORTFOLIO_PARETO_OBJECTIVES
            .iter()
            .map(|objective| (*objective).to_owned())
            .collect(),
        plans: plan_results,
    })
}

fn pareto_dominates(
    left: &ServiceNetworkPlanRobustness,
    right: &ServiceNetworkPlanRobustness,
) -> bool {
    let no_worse = left.worst_case_conservative_capacity_gain
        >= right.worst_case_conservative_capacity_gain
        && left.minimum_post_intervention_conservative_capacity
            >= right.minimum_post_intervention_conservative_capacity
        && left.scenarios_improved >= right.scenarios_improved
        && left.scenarios_worsened <= right.scenarios_worsened;
    let strictly_better = left.worst_case_conservative_capacity_gain
        > right.worst_case_conservative_capacity_gain
        || left.minimum_post_intervention_conservative_capacity
            > right.minimum_post_intervention_conservative_capacity
        || left.scenarios_improved > right.scenarios_improved
        || left.scenarios_worsened < right.scenarios_worsened;
    no_worse && strictly_better
}

/// Annotate the non-dominated layers without replacing the existing deterministic
/// display order. Every dimension is explicit; there is no hidden weighted score.
fn assign_pareto_fronts(plans: &mut [ServiceNetworkPlanRobustness]) {
    for index in 0..plans.len() {
        let mut dominators: Vec<String> = (0..plans.len())
            .filter(|other| *other != index && pareto_dominates(&plans[*other], &plans[index]))
            .map(|other| plans[other].plan_id.clone())
            .collect();
        dominators.sort();
        plans[index].pareto_dominated_by_plan_ids = dominators;
        plans[index].pareto_front = 0;
    }

    let mut remaining: BTreeSet<usize> = (0..plans.len()).collect();
    let mut front = 1_usize;
    while !remaining.is_empty() {
        let current_front: Vec<usize> = remaining
            .iter()
            .copied()
            .filter(|index| {
                !remaining
                    .iter()
                    .any(|other| other != index && pareto_dominates(&plans[*other], &plans[*index]))
            })
            .collect();

        // Strict dominance is acyclic for finite values. Fail safely if that
        // invariant is ever broken rather than loop or emit unranked records.
        if current_front.is_empty() {
            for index in remaining.iter().copied() {
                plans[index].pareto_front = front;
            }
            break;
        }
        for index in current_front {
            plans[index].pareto_front = front;
            remaining.remove(&index);
        }
        front += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::service_function::FunctionMeasurement;
    use crate::service_network::{
        NetworkLink, NetworkLinkAvailability, NetworkTopologyCompleteness,
        SERVICE_NETWORK_SCHEMA_VERSION, ServiceCapacityNetworkScenario,
    };
    use crate::service_network_intervention::{
        NetworkInterventionEffect, NetworkLinkInterventionOverride,
        SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION, ServiceNetworkInterventionPlan,
    };
    use crate::spatial_economic::Quantity;
    use std::collections::BTreeMap;

    fn measurement(entity_id: &str, value: f64) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity_id.into(),
            quantity: Quantity {
                value,
                unit: "m3/h".into(),
                uncertainty_abs: None,
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic portfolio assumption".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn link(
        id: &str,
        from: &str,
        to: &str,
        capacity: f64,
        availability: NetworkLinkAvailability,
    ) -> NetworkLink {
        NetworkLink {
            id: id.into(),
            from_entity_id: from.into(),
            to_entity_id: to.into(),
            capacity: measurement(id, capacity),
            availability,
            availability_basis: ScenarioBasis::Assumed,
            availability_rationale: "Synthetic scenario status".into(),
            availability_supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario(
        id: &str,
        supply: f64,
        backup: NetworkLinkAvailability,
    ) -> ServiceCapacityNetworkScenario {
        ServiceCapacityNetworkScenario {
            schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
            id: id.into(),
            as_of: "2026-10-16T10:00:00Z".into(),
            service_entity_id: "service:irrigation".into(),
            topology_completeness: NetworkTopologyCompleteness::Complete,
            source_entity_ids: vec!["source:reservoir".into()],
            required_capacity: measurement("service:irrigation", 8.0),
            links: vec![
                link(
                    "supply",
                    "source:reservoir",
                    "junction",
                    supply,
                    NetworkLinkAvailability::Active,
                ),
                link(
                    "route-a",
                    "junction",
                    "service:irrigation",
                    4.0,
                    NetworkLinkAvailability::Active,
                ),
                link("route-b", "junction", "service:irrigation", 4.0, backup),
            ],
        }
    }

    fn plan(
        id: &str,
        overrides: Vec<(&str, Option<NetworkLinkAvailability>, Option<f64>)>,
    ) -> ServiceNetworkInterventionPlan {
        ServiceNetworkInterventionPlan {
            schema_version: SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION.into(),
            id: id.into(),
            rationale: "Synthetic intervention candidate; no work is asserted to be performed"
                .into(),
            link_overrides: overrides
                .into_iter()
                .map(|(link_id, availability, capacity_value)| {
                    (
                        link_id.to_owned(),
                        NetworkLinkInterventionOverride {
                            availability,
                            capacity_value,
                            capacity_uncertainty_abs: None,
                            rationale: "Synthetic explicit intervention assumption".into(),
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    fn synthetic_robustness(
        plan_id: &str,
        worst_case_gain: f64,
        minimum_post_capacity: f64,
        improved: usize,
        worsened: usize,
    ) -> ServiceNetworkPlanRobustness {
        ServiceNetworkPlanRobustness {
            rank: 0,
            pareto_front: 0,
            pareto_dominated_by_plan_ids: Vec::new(),
            plan_id: plan_id.into(),
            rationale: "Synthetic Pareto unit test".into(),
            scenario_count: 3,
            worst_case_conservative_capacity_gain: worst_case_gain,
            worst_case_optimistic_capacity_gain: worst_case_gain,
            best_case_conservative_capacity_gain: worst_case_gain,
            scenarios_improved: improved,
            scenarios_unchanged: 3 - improved - worsened,
            scenarios_worsened: worsened,
            minimum_post_intervention_conservative_capacity: minimum_post_capacity,
            recurring_baseline_cut_links: Vec::new(),
            recurring_post_intervention_cut_links: Vec::new(),
            scenario_comparisons: Vec::new(),
        }
    }

    #[test]
    fn pareto_front_exposes_real_tradeoffs_without_collapsing_to_one_score() {
        let mut plans = vec![
            synthetic_robustness("high-gain", 5.0, 6.0, 3, 0),
            synthetic_robustness("high-floor", 4.0, 7.0, 2, 0),
            synthetic_robustness("dominated", 3.0, 6.0, 2, 1),
            synthetic_robustness("weakest", 0.0, 2.0, 0, 2),
        ];
        assign_pareto_fronts(&mut plans);
        let by_id: BTreeMap<&str, &ServiceNetworkPlanRobustness> = plans
            .iter()
            .map(|plan| (plan.plan_id.as_str(), plan))
            .collect();

        assert_eq!(by_id["high-gain"].pareto_front, 1);
        assert_eq!(by_id["high-floor"].pareto_front, 1);
        assert_eq!(by_id["dominated"].pareto_front, 2);
        assert_eq!(by_id["weakest"].pareto_front, 3);
        assert_eq!(
            by_id["dominated"].pareto_dominated_by_plan_ids,
            vec!["high-floor".to_owned(), "high-gain".to_owned()]
        );
        assert!(by_id["high-gain"].pareto_dominated_by_plan_ids.is_empty());
        assert!(by_id["high-floor"].pareto_dominated_by_plan_ids.is_empty());
    }

    #[test]
    fn pareto_fronts_are_invariant_to_candidate_input_order() {
        let candidates = vec![
            synthetic_robustness("a", 5.0, 6.0, 3, 0),
            synthetic_robustness("b", 4.0, 7.0, 2, 0),
            synthetic_robustness("c", 3.0, 6.0, 2, 1),
        ];
        let mut first = candidates.clone();
        assign_pareto_fronts(&mut first);
        let mut second = candidates;
        second.reverse();
        assign_pareto_fronts(&mut second);

        let summarize = |items: &[ServiceNetworkPlanRobustness]| {
            items
                .iter()
                .map(|item| {
                    (
                        item.plan_id.clone(),
                        (item.pareto_front, item.pareto_dominated_by_plan_ids.clone()),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        assert_eq!(summarize(&first), summarize(&second));
    }

    #[test]
    fn identical_objectives_do_not_dominate_each_other() {
        let mut plans = vec![
            synthetic_robustness("tie-a", 3.0, 7.0, 2, 0),
            synthetic_robustness("tie-b", 3.0, 7.0, 2, 0),
        ];
        assign_pareto_fronts(&mut plans);
        assert!(plans.iter().all(|plan| plan.pareto_front == 1));
        assert!(
            plans
                .iter()
                .all(|plan| plan.pareto_dominated_by_plan_ids.is_empty())
        );
    }

    #[test]
    fn json_portfolio_fixture_ranks_the_robust_combination_first() {
        let request: ServiceNetworkPortfolioRequest = serde_json::from_str(include_str!(
            "../../schemas/examples/service-network-portfolio-request.json"
        ))
        .unwrap();
        let result = compare_service_network_portfolio(&request, &[]).unwrap();
        assert_eq!(result.scenario_count, 2);
        assert_eq!(result.plan_count, 3);
        assert_eq!(
            result.pareto_objectives,
            NETWORK_PORTFOLIO_PARETO_OBJECTIVES
                .iter()
                .map(|objective| (*objective).to_owned())
                .collect::<Vec<_>>()
        );
        assert!(result.plans.iter().all(|plan| plan.pareto_front >= 1));
        assert_eq!(result.plans[0].plan_id, "robust-combined");
        assert_eq!(result.plans[0].worst_case_conservative_capacity_gain, 4.0);
        assert_eq!(result.plans[0].scenarios_improved, 2);
        assert_eq!(result.plans[0].scenarios_worsened, 0);
    }

    #[test]
    fn ranking_prefers_the_plan_with_a_positive_worst_case_gain() {
        let request = ServiceNetworkPortfolioRequest {
            comparison_id: "portfolio:irrigation".into(),
            scenarios: vec![
                scenario("scenario:normal", 10.0, NetworkLinkAvailability::Inactive),
                scenario("scenario:drought", 4.0, NetworkLinkAvailability::Active),
            ],
            plans: vec![
                plan(
                    "open-backup",
                    vec![("route-b", Some(NetworkLinkAvailability::Active), None)],
                ),
                plan("upgrade-supply", vec![("supply", None, Some(8.0))]),
                plan(
                    "robust-combined",
                    vec![
                        ("route-b", Some(NetworkLinkAvailability::Active), None),
                        ("supply", None, Some(8.0)),
                    ],
                ),
            ],
        };
        let result = compare_service_network_portfolio(&request, &[]).unwrap();
        assert_eq!(result.plans[0].plan_id, "robust-combined");
        assert_eq!(result.plans[0].worst_case_conservative_capacity_gain, 4.0);
        assert_eq!(result.plans[0].scenarios_improved, 2);
        assert_eq!(result.plans[1].worst_case_conservative_capacity_gain, 0.0);
        assert_eq!(result.plans[2].worst_case_conservative_capacity_gain, 0.0);
        assert_eq!(
            result.scenario_ids,
            vec!["scenario:drought", "scenario:normal"]
        );
        assert!(
            result.plans[0]
                .scenario_comparisons
                .iter()
                .all(|item| { item.effect == NetworkInterventionEffect::BoundsImproved })
        );
    }

    #[test]
    fn recurring_min_cuts_expose_bottlenecks_across_scenarios() {
        let request = ServiceNetworkPortfolioRequest {
            comparison_id: "portfolio:cuts".into(),
            scenarios: vec![
                scenario("scenario:normal", 10.0, NetworkLinkAvailability::Inactive),
                scenario("scenario:drought", 4.0, NetworkLinkAvailability::Active),
            ],
            plans: vec![plan(
                "restore-and-upgrade",
                vec![
                    ("route-b", Some(NetworkLinkAvailability::Active), None),
                    ("supply", None, Some(8.0)),
                ],
            )],
        };
        let result = compare_service_network_portfolio(&request, &[]).unwrap();
        assert!(
            result.plans[0]
                .recurring_baseline_cut_links
                .iter()
                .any(|item| item.link_id == "supply" && item.scenario_count >= 1)
        );
        assert!(
            result.plans[0]
                .scenario_comparisons
                .iter()
                .all(|comparison| comparison.baseline_scenario_id.starts_with("scenario:"))
        );
    }

    #[test]
    fn incomplete_scenario_topology_fails_closed() {
        let mut incomplete = scenario("scenario:unknown", 4.0, NetworkLinkAvailability::Active);
        incomplete.topology_completeness = NetworkTopologyCompleteness::Unknown;
        let request = ServiceNetworkPortfolioRequest {
            comparison_id: "portfolio:invalid".into(),
            scenarios: vec![incomplete],
            plans: vec![plan(
                "activate",
                vec![("route-b", Some(NetworkLinkAvailability::Active), None)],
            )],
        };
        assert!(matches!(
            compare_service_network_portfolio(&request, &[]),
            Err(ServiceNetworkPortfolioError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkPortfolioValidationError::IncompleteScenarioTopology)
        ));
    }

    #[test]
    fn mixed_units_or_topologies_fail_closed() {
        let mut wrong_unit = scenario("scenario:b", 4.0, NetworkLinkAvailability::Active);
        wrong_unit.required_capacity.quantity.unit = "L/s".into();
        let request = ServiceNetworkPortfolioRequest {
            comparison_id: "portfolio:unit".into(),
            scenarios: vec![
                scenario("scenario:a", 5.0, NetworkLinkAvailability::Active),
                wrong_unit,
            ],
            plans: vec![plan(
                "activate",
                vec![("route-b", Some(NetworkLinkAvailability::Active), None)],
            )],
        };
        assert!(matches!(
            compare_service_network_portfolio(&request, &[]),
            Err(ServiceNetworkPortfolioError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkPortfolioValidationError::MixedCapacityUnit)
        ));
    }

    #[test]
    fn duplicate_scenario_and_plan_ids_fail_closed() {
        let one = scenario("scenario:dup", 4.0, NetworkLinkAvailability::Inactive);
        let request = ServiceNetworkPortfolioRequest {
            comparison_id: "portfolio:duplicates".into(),
            scenarios: vec![one.clone(), one],
            plans: vec![
                plan(
                    "same",
                    vec![("route-b", Some(NetworkLinkAvailability::Active), None)],
                ),
                plan("same", vec![("supply", None, Some(8.0))]),
            ],
        };
        assert!(matches!(
            compare_service_network_portfolio(&request, &[]),
            Err(ServiceNetworkPortfolioError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkPortfolioValidationError::DuplicateScenarioId)
                    && errors.contains(&ServiceNetworkPortfolioValidationError::DuplicatePlanId)
        ));
    }

    #[test]
    fn ranking_and_json_are_deterministic() {
        let request = ServiceNetworkPortfolioRequest {
            comparison_id: "portfolio:determinism".into(),
            scenarios: vec![
                scenario("scenario:b", 4.0, NetworkLinkAvailability::Active),
                scenario("scenario:a", 5.0, NetworkLinkAvailability::Inactive),
            ],
            plans: vec![
                plan(
                    "z-plan",
                    vec![("route-b", Some(NetworkLinkAvailability::Active), None)],
                ),
                plan("a-plan", vec![("supply", None, Some(8.0))]),
            ],
        };
        let first = compare_service_network_portfolio(&request, &[]).unwrap();
        let second = compare_service_network_portfolio(&request, &[]).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );
    }
}
