// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Counterfactual comparison for explicit assumed service-network interventions.
//!
//! Plans may change declared link availability or link capacity. Outputs compare
//! the resulting flow bounds and minimum cuts; they do not establish that work was
//! performed, that a modeled edge is physically accurate, or that a real outcome
//! was causally produced by the proposed intervention.

use crate::dependency_logic::ScenarioBasis;
use crate::service_network::{
    NetworkLinkAvailability, NetworkTopologyCompleteness, ServiceCapacityNetworkAssessment,
    ServiceCapacityNetworkError, ServiceCapacityNetworkScenario,
    assess_service_capacity_network_indexed, build_evidence_index,
};
use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkLinkInterventionOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub availability: Option<NetworkLinkAvailability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity_value: Option<f64>,
    /// If omitted with capacity_value present, the assumed capacity is exact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity_uncertainty_abs: Option<f64>,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkInterventionPlan {
    pub schema_version: String,
    pub id: String,
    pub rationale: String,
    /// Map key is the stable link ID. Every override is a hypothetical assumption.
    pub link_overrides: BTreeMap<String, NetworkLinkInterventionOverride>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkInterventionRequest {
    pub scenario: ServiceCapacityNetworkScenario,
    pub plan: ServiceNetworkInterventionPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkInterventionEffect {
    /// Conservative flow bound rises and the optimistic bound does not fall.
    BoundsImproved,
    /// Optimistic flow bound falls and the conservative bound does not rise.
    BoundsDegraded,
    /// Only the optimistic bound increases; the conservative bound is unchanged.
    PossibleImprovementOnly,
    /// Only the optimistic bound decreases; the conservative bound is unchanged.
    PossibleDegradationOnly,
    /// Conservative bound falls while the optimistic bound remains unchanged.
    ConservativeBoundRegressed,
    /// Conservative and optimistic bounds move in conflicting directions.
    MixedBoundMovement,
    /// Neither modeled flow bound changes.
    BoundsUnchanged,
    /// The graph is incomplete, so a service-wide impact cannot be concluded.
    TopologyIncomplete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkInterventionComparison {
    pub schema_version: String,
    pub intervention_id: String,
    pub intervention_rationale: String,
    pub baseline_scenario_id: String,
    pub intervention_scenario_id: String,
    pub as_of: String,
    pub link_overrides: BTreeMap<String, NetworkLinkInterventionOverride>,
    pub baseline: ServiceCapacityNetworkAssessment,
    pub with_intervention: ServiceCapacityNetworkAssessment,
    /// Difference between modeled conservative lower-flow bounds.
    pub conservative_capacity_delta: f64,
    /// Difference between modeled optimistic upper-flow bounds.
    pub optimistic_capacity_delta: f64,
    pub effect: NetworkInterventionEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceNetworkInterventionValidationError {
    UnsupportedSchemaVersion,
    EmptyPlanId,
    EmptyPlanRationale,
    EmptyOverrides,
    EmptyOverrideLinkId,
    UnknownOverrideLinkId,
    EmptyOverrideRationale,
    OverrideMustChangeSomething,
    UncertaintyWithoutCapacityOverride,
    NonFiniteCapacityOverride,
    NegativeCapacityOverride,
    NonFiniteCapacityUncertainty,
    NegativeCapacityUncertainty,
    NonFiniteDelta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceNetworkInterventionError {
    EmptyRequestSet,
    DuplicateComparisonKey,
    InvalidPlan(Vec<ServiceNetworkInterventionValidationError>),
    BaselineAssessment(ServiceCapacityNetworkError),
    InterventionAssessment(ServiceCapacityNetworkError),
}

/// Compare a baseline capacity network with explicit hypothetical link overrides.
///
/// Availability or capacity changes are always scenario assumptions. A capacity
/// override retains the link's declared unit, replaces its observed/assumed
/// measurement with an assumed value, and removes the replaced capacity evidence
/// IDs from the candidate result. Existing evidence on untouched inputs is retained.
pub fn compare_service_network_intervention(
    scenario: &ServiceCapacityNetworkScenario,
    evidence_records: &[EvidenceRecord],
    plan: &ServiceNetworkInterventionPlan,
) -> Result<ServiceNetworkInterventionComparison, ServiceNetworkInterventionError> {
    validate_plan(scenario, plan)?;
    let evidence_by_id = build_evidence_index(evidence_records).map_err(|errors| {
        ServiceNetworkInterventionError::BaselineAssessment(
            ServiceCapacityNetworkError::InvalidInput(errors),
        )
    })?;
    compare_network_intervention_indexed(scenario, &evidence_by_id, plan)
}

/// Batch API for multiple network counterfactuals over a shared evidence corpus.
///
/// The corpus is structurally validated and indexed once; requests are returned in
/// stable (scenario ID, plan ID) order. Repeated comparison keys are rejected.
pub fn compare_service_network_interventions(
    requests: &[ServiceNetworkInterventionRequest],
    evidence_records: &[EvidenceRecord],
) -> Result<Vec<ServiceNetworkInterventionComparison>, ServiceNetworkInterventionError> {
    if requests.is_empty() {
        return Err(ServiceNetworkInterventionError::EmptyRequestSet);
    }
    let mut comparison_keys = std::collections::BTreeSet::new();
    for request in requests {
        validate_plan(&request.scenario, &request.plan)?;
        if !comparison_keys.insert((request.scenario.id.as_str(), request.plan.id.as_str())) {
            return Err(ServiceNetworkInterventionError::DuplicateComparisonKey);
        }
    }
    let evidence_by_id = build_evidence_index(evidence_records).map_err(|errors| {
        ServiceNetworkInterventionError::BaselineAssessment(
            ServiceCapacityNetworkError::InvalidInput(errors),
        )
    })?;
    let mut ordered: Vec<&ServiceNetworkInterventionRequest> = requests.iter().collect();
    ordered.sort_by(|left, right| {
        left.scenario
            .id
            .cmp(&right.scenario.id)
            .then_with(|| left.plan.id.cmp(&right.plan.id))
    });
    ordered
        .into_iter()
        .map(|request| {
            compare_network_intervention_indexed(&request.scenario, &evidence_by_id, &request.plan)
        })
        .collect()
}

fn validate_plan(
    scenario: &ServiceCapacityNetworkScenario,
    plan: &ServiceNetworkInterventionPlan,
) -> Result<(), ServiceNetworkInterventionError> {
    let mut errors = Vec::new();
    if plan.schema_version != SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION {
        errors.push(ServiceNetworkInterventionValidationError::UnsupportedSchemaVersion);
    }
    if plan.id.trim().is_empty() {
        errors.push(ServiceNetworkInterventionValidationError::EmptyPlanId);
    }
    if plan.rationale.trim().is_empty() {
        errors.push(ServiceNetworkInterventionValidationError::EmptyPlanRationale);
    }
    if plan.link_overrides.is_empty() {
        errors.push(ServiceNetworkInterventionValidationError::EmptyOverrides);
    }

    for (link_id, override_value) in &plan.link_overrides {
        if link_id.trim().is_empty() {
            errors.push(ServiceNetworkInterventionValidationError::EmptyOverrideLinkId);
        }
        if !scenario.links.iter().any(|link| link.id == *link_id) {
            errors.push(ServiceNetworkInterventionValidationError::UnknownOverrideLinkId);
        }
        if override_value.rationale.trim().is_empty() {
            errors.push(ServiceNetworkInterventionValidationError::EmptyOverrideRationale);
        }
        if override_value.availability.is_none() && override_value.capacity_value.is_none() {
            errors.push(ServiceNetworkInterventionValidationError::OverrideMustChangeSomething);
        }
        if override_value.capacity_uncertainty_abs.is_some()
            && override_value.capacity_value.is_none()
        {
            errors.push(
                ServiceNetworkInterventionValidationError::UncertaintyWithoutCapacityOverride,
            );
        }
        if let Some(value) = override_value.capacity_value {
            if !value.is_finite() {
                errors.push(ServiceNetworkInterventionValidationError::NonFiniteCapacityOverride);
            } else if value < 0.0 {
                errors.push(ServiceNetworkInterventionValidationError::NegativeCapacityOverride);
            }
        }
        if let Some(uncertainty) = override_value.capacity_uncertainty_abs {
            if !uncertainty.is_finite() {
                errors
                    .push(ServiceNetworkInterventionValidationError::NonFiniteCapacityUncertainty);
            } else if uncertainty < 0.0 {
                errors.push(ServiceNetworkInterventionValidationError::NegativeCapacityUncertainty);
            }
        }
    }
    if !errors.is_empty() {
        Err(ServiceNetworkInterventionError::InvalidPlan(errors))
    } else {
        Ok(())
    }
}

fn compare_network_intervention_indexed(
    scenario: &ServiceCapacityNetworkScenario,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
    plan: &ServiceNetworkInterventionPlan,
) -> Result<ServiceNetworkInterventionComparison, ServiceNetworkInterventionError> {
    let baseline = assess_service_capacity_network_indexed(scenario, evidence_by_id)
        .map_err(ServiceNetworkInterventionError::BaselineAssessment)?;
    let mut candidate = scenario.clone();
    for link in &mut candidate.links {
        let Some(override_value) = plan.link_overrides.get(&link.id) else {
            continue;
        };
        if let Some(availability) = override_value.availability {
            link.availability = availability;
            link.availability_basis = ScenarioBasis::Assumed;
            link.availability_rationale = override_value.rationale.clone();
            link.availability_supporting_evidence_ids.clear();
        }
        if let Some(capacity) = override_value.capacity_value {
            link.capacity.quantity.value = capacity;
            link.capacity.quantity.uncertainty_abs = override_value.capacity_uncertainty_abs;
            link.capacity.basis = ScenarioBasis::Assumed;
            link.capacity.rationale = override_value.rationale.clone();
            link.capacity.supporting_evidence_ids.clear();
        }
    }
    candidate.id = format!("{}:intervention:{}", scenario.id, plan.id);
    let with_intervention = assess_service_capacity_network_indexed(&candidate, evidence_by_id)
        .map_err(ServiceNetworkInterventionError::InterventionAssessment)?;
    let conservative_capacity_delta = with_intervention.conservative_available_capacity
        - baseline.conservative_available_capacity;
    let optimistic_capacity_delta =
        with_intervention.optimistic_available_capacity - baseline.optimistic_available_capacity;
    if !conservative_capacity_delta.is_finite() || !optimistic_capacity_delta.is_finite() {
        return Err(ServiceNetworkInterventionError::InvalidPlan(vec![
            ServiceNetworkInterventionValidationError::NonFiniteDelta,
        ]));
    }

    let effect = classify_effect(
        scenario.topology_completeness,
        conservative_capacity_delta,
        optimistic_capacity_delta,
    );
    Ok(ServiceNetworkInterventionComparison {
        schema_version: SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION.into(),
        intervention_id: plan.id.clone(),
        intervention_rationale: plan.rationale.clone(),
        baseline_scenario_id: scenario.id.clone(),
        intervention_scenario_id: candidate.id.clone(),
        as_of: scenario.as_of.clone(),
        link_overrides: plan.link_overrides.clone(),
        baseline,
        with_intervention,
        conservative_capacity_delta,
        optimistic_capacity_delta,
        effect,
    })
}

fn classify_effect(
    topology: NetworkTopologyCompleteness,
    conservative_delta: f64,
    optimistic_delta: f64,
) -> NetworkInterventionEffect {
    if topology != NetworkTopologyCompleteness::Complete {
        return NetworkInterventionEffect::TopologyIncomplete;
    }
    if conservative_delta == 0.0 && optimistic_delta == 0.0 {
        NetworkInterventionEffect::BoundsUnchanged
    } else if conservative_delta > 0.0 && optimistic_delta >= 0.0 {
        NetworkInterventionEffect::BoundsImproved
    } else if conservative_delta == 0.0 && optimistic_delta > 0.0 {
        NetworkInterventionEffect::PossibleImprovementOnly
    } else if conservative_delta == 0.0 && optimistic_delta < 0.0 {
        NetworkInterventionEffect::PossibleDegradationOnly
    } else if conservative_delta < 0.0 && optimistic_delta == 0.0 {
        NetworkInterventionEffect::ConservativeBoundRegressed
    } else if conservative_delta <= 0.0 && optimistic_delta < 0.0 {
        NetworkInterventionEffect::BoundsDegraded
    } else {
        NetworkInterventionEffect::MixedBoundMovement
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_function::FunctionMeasurement;
    use crate::service_network::{
        NetworkAssessmentState, NetworkLink, NetworkTopologyCompleteness,
        SERVICE_NETWORK_SCHEMA_VERSION,
    };
    use crate::spatial_economic::Quantity;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct FixturePlan {
        plan: ServiceNetworkInterventionPlan,
    }

    #[derive(Debug, Deserialize)]
    struct FixtureComparison {
        comparison: ServiceNetworkInterventionComparison,
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
            capacity: FunctionMeasurement {
                entity_id: id.into(),
                quantity: Quantity {
                    value: capacity,
                    unit: "m3/h".into(),
                    uncertainty_abs: None,
                },
                basis: ScenarioBasis::Assumed,
                rationale: "Synthetic baseline capacity".into(),
                supporting_evidence_ids: Vec::new(),
            },
            availability,
            availability_basis: ScenarioBasis::Assumed,
            availability_rationale: "Synthetic baseline availability".into(),
            availability_supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario() -> ServiceCapacityNetworkScenario {
        ServiceCapacityNetworkScenario {
            schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
            id: "scenario:network-intervention".into(),
            as_of: "2026-10-16T10:00:00Z".into(),
            service_entity_id: "service:irrigation".into(),
            topology_completeness: NetworkTopologyCompleteness::Complete,
            source_entity_ids: vec!["source:reservoir".into()],
            required_capacity: FunctionMeasurement {
                entity_id: "service:irrigation".into(),
                quantity: Quantity {
                    value: 8.0,
                    unit: "m3/h".into(),
                    uncertainty_abs: None,
                },
                basis: ScenarioBasis::Assumed,
                rationale: "Synthetic service demand".into(),
                supporting_evidence_ids: Vec::new(),
            },
            links: vec![
                link(
                    "supply",
                    "source:reservoir",
                    "junction",
                    10.0,
                    NetworkLinkAvailability::Active,
                ),
                link(
                    "route-a",
                    "junction",
                    "service:irrigation",
                    4.0,
                    NetworkLinkAvailability::Active,
                ),
                link(
                    "route-b",
                    "junction",
                    "service:irrigation",
                    4.0,
                    NetworkLinkAvailability::Inactive,
                ),
            ],
        }
    }

    fn plan(
        id: &str,
        overrides: BTreeMap<String, NetworkLinkInterventionOverride>,
    ) -> ServiceNetworkInterventionPlan {
        ServiceNetworkInterventionPlan {
            schema_version: SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION.into(),
            id: id.into(),
            rationale: "Synthetic what-if intervention; not a claim of execution".into(),
            link_overrides: overrides,
        }
    }

    fn activate_link(_link_id: &str) -> NetworkLinkInterventionOverride {
        NetworkLinkInterventionOverride {
            availability: Some(NetworkLinkAvailability::Active),
            capacity_value: None,
            capacity_uncertainty_abs: None,
            rationale: "Synthetic scenario assumption: route restored".into(),
        }
    }

    #[test]
    fn restoring_parallel_route_increases_end_to_end_capacity() {
        let comparison = compare_service_network_intervention(
            &scenario(),
            &[],
            &plan(
                "restore-route-b",
                [("route-b".into(), activate_link("route-b"))].into(),
            ),
        )
        .unwrap();
        assert_eq!(comparison.baseline.conservative_available_capacity, 4.0);
        assert_eq!(
            comparison.with_intervention.conservative_available_capacity,
            8.0
        );
        assert_eq!(comparison.conservative_capacity_delta, 4.0);
        assert_eq!(comparison.baseline.state, NetworkAssessmentState::Degraded);
        assert_eq!(
            comparison.with_intervention.state,
            NetworkAssessmentState::Adequate
        );
        assert_eq!(comparison.effect, NetworkInterventionEffect::BoundsImproved);
    }

    #[test]
    fn capacity_upgrade_changes_the_limiting_cut() {
        let mut input = scenario();
        input.links[0].capacity.quantity.value = 5.0;
        input.links[2].availability = NetworkLinkAvailability::Active;
        input.required_capacity.quantity.value = 8.0;
        let plan = plan(
            "upgrade-supply",
            [(
                "supply".into(),
                NetworkLinkInterventionOverride {
                    availability: None,
                    capacity_value: Some(10.0),
                    capacity_uncertainty_abs: None,
                    rationale: "Synthetic supply upgrade assumption".into(),
                },
            )]
            .into(),
        );
        let comparison = compare_service_network_intervention(&input, &[], &plan).unwrap();
        assert_eq!(
            comparison.baseline.conservative_min_cut_link_ids,
            vec!["supply"]
        );
        assert_eq!(
            comparison.with_intervention.conservative_min_cut_link_ids,
            vec!["route-a", "route-b"]
        );
        assert_eq!(comparison.baseline.conservative_available_capacity, 5.0);
        assert_eq!(
            comparison.with_intervention.conservative_available_capacity,
            8.0
        );
        assert_eq!(comparison.effect, NetworkInterventionEffect::BoundsImproved);
    }

    #[test]
    fn inactive_unknown_and_active_route_scenarios_preserve_uncertainty_semantics() {
        let mut input = scenario();
        input.links[2].availability = NetworkLinkAvailability::Unknown;
        let base = compare_service_network_intervention(
            &input,
            &[],
            &plan(
                "activate-route",
                [("route-b".into(), activate_link("route-b"))].into(),
            ),
        )
        .unwrap();
        assert_eq!(base.baseline.conservative_available_capacity, 4.0);
        assert_eq!(base.baseline.optimistic_available_capacity, 8.0);
        assert_eq!(base.effect, NetworkInterventionEffect::BoundsImproved);
    }

    #[test]
    fn effect_classifier_covers_unchanged_degraded_possible_and_mixed_bounds() {
        let supply_only = plan(
            "raise-nonlimiting-supply",
            [(
                "supply".into(),
                NetworkLinkInterventionOverride {
                    availability: None,
                    capacity_value: Some(15.0),
                    capacity_uncertainty_abs: None,
                    rationale: "Synthetic non-limiting supply upgrade".into(),
                },
            )]
            .into(),
        );
        let unchanged =
            compare_service_network_intervention(&scenario(), &[], &supply_only).unwrap();
        assert_eq!(unchanged.conservative_capacity_delta, 0.0);
        assert_eq!(unchanged.optimistic_capacity_delta, 0.0);
        assert_eq!(unchanged.effect, NetworkInterventionEffect::BoundsUnchanged);

        let deactivate = plan(
            "close-only-route",
            [(
                "route-a".into(),
                NetworkLinkInterventionOverride {
                    availability: Some(NetworkLinkAvailability::Inactive),
                    capacity_value: None,
                    capacity_uncertainty_abs: None,
                    rationale: "Synthetic route closure".into(),
                },
            )]
            .into(),
        );
        let degraded = compare_service_network_intervention(&scenario(), &[], &deactivate).unwrap();
        assert_eq!(degraded.conservative_capacity_delta, -4.0);
        assert_eq!(degraded.optimistic_capacity_delta, -4.0);
        assert_eq!(degraded.effect, NetworkInterventionEffect::BoundsDegraded);

        let mut unknown_route = scenario();
        unknown_route.links[1].availability = NetworkLinkAvailability::Unknown;
        let improve_possible = plan(
            "increase-unknown-route-capacity",
            [(
                "route-a".into(),
                NetworkLinkInterventionOverride {
                    availability: None,
                    capacity_value: Some(8.0),
                    capacity_uncertainty_abs: None,
                    rationale: "Synthetic capacity assumption on a route of unknown state".into(),
                },
            )]
            .into(),
        );
        let possible_improvement =
            compare_service_network_intervention(&unknown_route, &[], &improve_possible).unwrap();
        assert_eq!(possible_improvement.conservative_capacity_delta, 0.0);
        assert_eq!(possible_improvement.optimistic_capacity_delta, 4.0);
        assert_eq!(
            possible_improvement.effect,
            NetworkInterventionEffect::PossibleImprovementOnly
        );

        let reduce_possible = plan(
            "reduce-unknown-route-capacity",
            [(
                "route-a".into(),
                NetworkLinkInterventionOverride {
                    availability: None,
                    capacity_value: Some(2.0),
                    capacity_uncertainty_abs: None,
                    rationale: "Synthetic capacity downgrade on a route of unknown state".into(),
                },
            )]
            .into(),
        );
        let possible_degradation =
            compare_service_network_intervention(&unknown_route, &[], &reduce_possible).unwrap();
        assert_eq!(possible_degradation.conservative_capacity_delta, 0.0);
        assert_eq!(possible_degradation.optimistic_capacity_delta, -2.0);
        assert_eq!(
            possible_degradation.effect,
            NetworkInterventionEffect::PossibleDegradationOnly
        );

        let mut mixed_input = scenario();
        mixed_input.links[2].availability = NetworkLinkAvailability::Unknown;
        let mixed_plan = plan(
            "uncertain-redundancy-shift",
            [
                (
                    "route-a".into(),
                    NetworkLinkInterventionOverride {
                        availability: Some(NetworkLinkAvailability::Inactive),
                        capacity_value: None,
                        capacity_uncertainty_abs: None,
                        rationale: "Synthetic scenario removes the known route".into(),
                    },
                ),
                (
                    "route-b".into(),
                    NetworkLinkInterventionOverride {
                        availability: None,
                        capacity_value: Some(10.0),
                        capacity_uncertainty_abs: None,
                        rationale: "Synthetic scenario increases the unknown alternate's capacity"
                            .into(),
                    },
                ),
            ]
            .into(),
        );
        let mixed = compare_service_network_intervention(&mixed_input, &[], &mixed_plan).unwrap();
        assert_eq!(mixed.conservative_capacity_delta, -4.0);
        assert_eq!(mixed.optimistic_capacity_delta, 2.0);
        assert_eq!(mixed.effect, NetworkInterventionEffect::MixedBoundMovement);
    }

    #[test]
    fn degradation_of_only_the_conservative_bound_is_not_mislabeled_unchanged() {
        let mut input = scenario();
        input.links[2].availability = NetworkLinkAvailability::Unknown;
        let plan = plan(
            "make-known-route-uncertain",
            [(
                "route-a".into(),
                NetworkLinkInterventionOverride {
                    availability: Some(NetworkLinkAvailability::Unknown),
                    capacity_value: None,
                    capacity_uncertainty_abs: None,
                    rationale: "Synthetic scenario assumption: route confidence reduced.".into(),
                },
            )]
            .into(),
        );
        let comparison = compare_service_network_intervention(&input, &[], &plan).unwrap();
        assert_eq!(
            comparison.with_intervention.conservative_available_capacity,
            0.0
        );
        assert_eq!(
            comparison.with_intervention.optimistic_available_capacity,
            8.0
        );
        assert_eq!(comparison.conservative_capacity_delta, -4.0);
        assert_eq!(comparison.optimistic_capacity_delta, 0.0);
        assert_eq!(
            comparison.effect,
            NetworkInterventionEffect::ConservativeBoundRegressed
        );
    }

    #[test]
    fn incomplete_topology_never_claims_a_service_wide_intervention_effect() {
        let mut input = scenario();
        input.topology_completeness = NetworkTopologyCompleteness::Partial;
        let comparison = compare_service_network_intervention(
            &input,
            &[],
            &plan(
                "restore-route-b",
                [("route-b".into(), activate_link("route-b"))].into(),
            ),
        )
        .unwrap();
        assert_eq!(
            comparison.effect,
            NetworkInterventionEffect::TopologyIncomplete
        );
        assert_eq!(
            comparison.baseline.state,
            NetworkAssessmentState::Indeterminate
        );
        assert_eq!(
            comparison.with_intervention.state,
            NetworkAssessmentState::Indeterminate
        );
        assert_eq!(comparison.baseline.coverage_lower_bound, None);
        assert_eq!(comparison.with_intervention.coverage_upper_bound, None);
    }

    #[test]
    fn invalid_plans_fail_closed() {
        assert!(matches!(
            compare_service_network_intervention(&scenario(), &[], &plan("empty", BTreeMap::new())),
            Err(ServiceNetworkInterventionError::InvalidPlan(errors))
                if errors.contains(&ServiceNetworkInterventionValidationError::EmptyOverrides)
        ));
        assert!(matches!(
            compare_service_network_intervention(
                &scenario(),
                &[],
                &plan("unknown", [("missing-link".into(), activate_link("missing-link"))].into()),
            ),
            Err(ServiceNetworkInterventionError::InvalidPlan(errors))
                if errors.contains(&ServiceNetworkInterventionValidationError::UnknownOverrideLinkId)
        ));
    }

    #[test]
    fn batch_intervention_comparison_sorts_scenario_plan_pairs() {
        let mut scenario_b = scenario();
        scenario_b.id = "scenario:b".into();
        let mut scenario_a = scenario();
        scenario_a.id = "scenario:a".into();
        let restore = plan(
            "restore-route",
            [("route-b".into(), activate_link("route-b"))].into(),
        );
        let requests = vec![
            ServiceNetworkInterventionRequest {
                scenario: scenario_b,
                plan: restore.clone(),
            },
            ServiceNetworkInterventionRequest {
                scenario: scenario_a,
                plan: restore,
            },
        ];
        let results = compare_service_network_interventions(&requests, &[]).unwrap();
        assert_eq!(
            results
                .iter()
                .map(|item| item.baseline_scenario_id.as_str())
                .collect::<Vec<_>>(),
            vec!["scenario:a", "scenario:b"]
        );
        assert!(results.iter().all(|item| {
            item.effect == NetworkInterventionEffect::BoundsImproved
                && item.baseline.conservative_available_capacity == 4.0
                && item.with_intervention.conservative_available_capacity == 8.0
        }));
    }

    #[test]
    fn batch_intervention_rejects_empty_and_duplicate_comparison_keys() {
        assert_eq!(
            compare_service_network_interventions(&[], &[]),
            Err(ServiceNetworkInterventionError::EmptyRequestSet)
        );
        let request = ServiceNetworkInterventionRequest {
            scenario: scenario(),
            plan: plan(
                "restore-route",
                [("route-b".into(), activate_link("route-b"))].into(),
            ),
        };
        assert_eq!(
            compare_service_network_interventions(&[request.clone(), request], &[]),
            Err(ServiceNetworkInterventionError::DuplicateComparisonKey)
        );
    }

    #[test]
    fn network_intervention_fixture_matches_golden_comparison() {
        let input: ServiceCapacityNetworkScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/service-capacity-network-scenario.json"
        ))
        .unwrap();
        let fixture: FixturePlan = serde_json::from_str(include_str!(
            "../../schemas/examples/service-network-intervention-plan.json"
        ))
        .unwrap();
        let expected: FixtureComparison = serde_json::from_str(include_str!(
            "../../schemas/examples/service-network-intervention-comparison.json"
        ))
        .unwrap();
        let comparison = compare_service_network_intervention(&input, &[], &fixture.plan).unwrap();
        assert_eq!(comparison, expected.comparison);
        assert_eq!(comparison.effect, NetworkInterventionEffect::BoundsImproved);
    }
}
