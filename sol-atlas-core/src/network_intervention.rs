// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-preserving what-if comparisons for service-capacity networks.
//!
//! Every intervention is an explicit assumption. A candidate cannot rewrite the
//! baseline assessment, and results rank changes in modeled conservative flow—not
//! causal impact, cost-effectiveness, probability, or authorization to act.

use crate::service_network::{
    assess_service_capacity_network_indexed, build_evidence_index,
    NetworkAssessmentState, NetworkLinkAvailability, NetworkTopologyCompleteness,
    ServiceCapacityNetworkAssessment, ServiceCapacityNetworkError,
    ServiceCapacityNetworkScenario, ServiceCapacityNetworkValidationError,
};
use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const NETWORK_INTERVENTION_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NetworkInterventionAction {
    /// Explicitly model a currently unknown or inactive link as active.
    AssumeLinkActive { link_id: String },
    /// Set a larger planned link capacity as an explicit assumption. Existing
    /// observed capacity evidence is not inherited by the candidate value.
    IncreaseLinkCapacity {
        link_id: String,
        target_capacity_value: f64,
        target_uncertainty_abs: f64,
        unit: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkInterventionPlan {
    pub id: String,
    pub rationale: String,
    pub action: NetworkInterventionAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkInterventionComparison {
    pub schema_version: String,
    pub baseline_scenario_id: String,
    pub intervention_id: String,
    pub rationale: String,
    pub action: NetworkInterventionAction,
    pub baseline_state: NetworkAssessmentState,
    pub after_state: NetworkAssessmentState,
    pub baseline_conservative_capacity: f64,
    pub after_conservative_capacity: f64,
    pub conservative_capacity_gain: f64,
    pub baseline_optimistic_capacity: f64,
    pub after_optimistic_capacity: f64,
    pub optimistic_capacity_gain: f64,
    pub baseline_min_cut_link_ids: Vec<String>,
    pub after_min_cut_link_ids: Vec<String>,
    pub baseline_capacity_evidence_ids: Vec<String>,
    pub after_capacity_evidence_ids: Vec<String>,
    pub baseline_availability_evidence_ids: Vec<String>,
    pub after_availability_evidence_ids: Vec<String>,
    pub topology_complete: bool,
    /// This is an engineering ranking only; cost, equity and rights are not scored.
    pub ranking_basis: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkInterventionValidationError {
    IncompleteBaselineTopology,
    EmptyPlanSet,
    EmptyInterventionId,
    DuplicateInterventionId,
    EmptyRationale,
    EmptyLinkId,
    UnknownLink,
    LinkAlreadyActive,
    LinkNotActive,
    TargetCapacityMustIncrease,
    InvalidTargetCapacity,
    UnitMismatch,
    NonFiniteFlowGain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkInterventionError {
    InvalidBaseline(ServiceCapacityNetworkError),
    InvalidInput(Vec<NetworkInterventionValidationError>),
}

/// Compare explicit what-if plans against one baseline and rank by conservative
/// flow gain, then by stable intervention ID.
///
/// The shared evidence corpus is indexed once for baseline and all candidates.
/// Candidate capacity changes become assumed values with no inherited evidence IDs.
/// A complete topology is mandatory: partial topology cannot support a ranking that
/// purports to describe the whole service.
pub fn compare_network_interventions(
    baseline: &ServiceCapacityNetworkScenario,
    plans: &[NetworkInterventionPlan],
    evidence_records: &[EvidenceRecord],
) -> Result<Vec<NetworkInterventionComparison>, NetworkInterventionError> {
    if baseline.topology_completeness != NetworkTopologyCompleteness::Complete {
        return Err(NetworkInterventionError::InvalidInput(vec![
            NetworkInterventionValidationError::IncompleteBaselineTopology,
        ]));
    }
    if plans.is_empty() {
        return Err(NetworkInterventionError::InvalidInput(vec![
            NetworkInterventionValidationError::EmptyPlanSet,
        ]));
    }

    let evidence_index = build_evidence_index(evidence_records)
        .map_err(|errors| NetworkInterventionError::InvalidBaseline(
            ServiceCapacityNetworkError::InvalidInput(errors),
        ))?;
    let baseline_assessment = assess_service_capacity_network_indexed(baseline, &evidence_index)
        .map_err(NetworkInterventionError::InvalidBaseline)?;

    let mut ids = BTreeSet::new();
    let mut ordered_plans: Vec<&NetworkInterventionPlan> = plans.iter().collect();
    for plan in plans {
        if plan.id.trim().is_empty() {
            return Err(NetworkInterventionError::InvalidInput(vec![
                NetworkInterventionValidationError::EmptyInterventionId,
            ]));
        }
        if !ids.insert(plan.id.as_str()) {
            return Err(NetworkInterventionError::InvalidInput(vec![
                NetworkInterventionValidationError::DuplicateInterventionId,
            ]));
        }
        if plan.rationale.trim().is_empty() {
            return Err(NetworkInterventionError::InvalidInput(vec![
                NetworkInterventionValidationError::EmptyRationale,
            ]));
        }
        validate_action(baseline, plan)?;
    }
    ordered_plans.sort_by(|left, right| left.id.cmp(&right.id));

    let mut comparisons = Vec::with_capacity(ordered_plans.len());
    for plan in ordered_plans {
        let candidate = apply_plan(baseline, plan)?;
        let after = assess_service_capacity_network_indexed(&candidate, &evidence_index)
            .map_err(NetworkInterventionError::InvalidBaseline)?;
        let conservative_capacity_gain =
            after.conservative_available_capacity - baseline_assessment.conservative_available_capacity;
        let optimistic_capacity_gain =
            after.optimistic_available_capacity - baseline_assessment.optimistic_available_capacity;
        if !conservative_capacity_gain.is_finite() || !optimistic_capacity_gain.is_finite() {
            return Err(NetworkInterventionError::InvalidInput(vec![
                NetworkInterventionValidationError::NonFiniteFlowGain,
            ]));
        }
        comparisons.push(NetworkInterventionComparison {
            schema_version: NETWORK_INTERVENTION_SCHEMA_VERSION.into(),
            baseline_scenario_id: baseline.id.clone(),
            intervention_id: plan.id.clone(),
            rationale: plan.rationale.clone(),
            action: plan.action.clone(),
            baseline_state: baseline_assessment.state,
            after_state: after.state,
            baseline_conservative_capacity: baseline_assessment.conservative_available_capacity,
            after_conservative_capacity: after.conservative_available_capacity,
            conservative_capacity_gain,
            baseline_optimistic_capacity: baseline_assessment.optimistic_available_capacity,
            after_optimistic_capacity: after.optimistic_available_capacity,
            optimistic_capacity_gain,
            baseline_min_cut_link_ids: baseline_assessment.conservative_min_cut_link_ids.clone(),
            after_min_cut_link_ids: after.conservative_min_cut_link_ids,
            baseline_capacity_evidence_ids: baseline_assessment.capacity_evidence_ids.clone(),
            after_capacity_evidence_ids: after.capacity_evidence_ids,
            baseline_availability_evidence_ids: baseline_assessment.availability_evidence_ids.clone(),
            after_availability_evidence_ids: after.availability_evidence_ids,
            topology_complete: true,
            ranking_basis: format!(
                "descending conservative capacity gain in {}; ties break by intervention ID; excludes cost, equity and rights effects",
                after.unit
            ),
        });
    }

    comparisons.sort_by(|left, right| {
        right.conservative_capacity_gain.total_cmp(&left.conservative_capacity_gain)
            .then_with(|| left.intervention_id.cmp(&right.intervention_id))
    });
    Ok(comparisons)
}

fn validate_action(
    baseline: &ServiceCapacityNetworkScenario,
    plan: &NetworkInterventionPlan,
) -> Result<(), NetworkInterventionError> {
    match &plan.action {
        NetworkInterventionAction::AssumeLinkActive { link_id } => {
            if link_id.trim().is_empty() {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::EmptyLinkId,
                ]));
            }
            let Some(link) = baseline.links.iter().find(|link| link.id == *link_id) else {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::UnknownLink,
                ]));
            };
            if link.availability == NetworkLinkAvailability::Active {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::LinkAlreadyActive,
                ]));
            }
        }
        NetworkInterventionAction::IncreaseLinkCapacity {
            link_id, target_capacity_value, target_uncertainty_abs, unit,
        } => {
            if link_id.trim().is_empty() {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::EmptyLinkId,
                ]));
            }
            let Some(link) = baseline.links.iter().find(|link| link.id == *link_id) else {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::UnknownLink,
                ]));
            };
            if link.availability != NetworkLinkAvailability::Active {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::LinkNotActive,
                ]));
            }
            if !target_capacity_value.is_finite()
                || *target_capacity_value < 0.0
                || !target_uncertainty_abs.is_finite()
                || *target_uncertainty_abs < 0.0
            {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::InvalidTargetCapacity,
                ]));
            }
            if unit != &link.capacity.quantity.unit {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::UnitMismatch,
                ]));
            }
            if *target_capacity_value <= link.capacity.quantity.value {
                return Err(NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::TargetCapacityMustIncrease,
                ]));
            }
        }
    }
    Ok(())
}

fn apply_plan(
    baseline: &ServiceCapacityNetworkScenario,
    plan: &NetworkInterventionPlan,
) -> Result<ServiceCapacityNetworkScenario, NetworkInterventionError> {
    let mut candidate = baseline.clone();
    match &plan.action {
        NetworkInterventionAction::AssumeLinkActive { link_id } => {
            let link = candidate.links.iter_mut().find(|link| link.id == *link_id)
                .ok_or_else(|| NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::UnknownLink,
                ]))?;
            link.availability = NetworkLinkAvailability::Active;
            link.availability_basis = crate::dependency_logic::ScenarioBasis::Assumed;
            link.availability_supporting_evidence_ids.clear();
            link.availability_rationale = plan.rationale.clone();
        }
        NetworkInterventionAction::IncreaseLinkCapacity {
            link_id, target_capacity_value, target_uncertainty_abs, unit: _,
        } => {
            let link = candidate.links.iter_mut().find(|link| link.id == *link_id)
                .ok_or_else(|| NetworkInterventionError::InvalidInput(vec![
                    NetworkInterventionValidationError::UnknownLink,
                ]))?;
            link.capacity.quantity.value = *target_capacity_value;
            link.capacity.quantity.uncertainty_abs = Some(*target_uncertainty_abs);
            link.capacity.basis = crate::dependency_logic::ScenarioBasis::Assumed;
            link.capacity.supporting_evidence_ids.clear();
            link.capacity.rationale = plan.rationale.clone();
        }
    }
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::service_function::FunctionMeasurement;
    use crate::service_network::{
        NetworkLink, NetworkLinkAvailability, NetworkTopologyCompleteness,
        ServiceCapacityNetworkScenario, SERVICE_NETWORK_SCHEMA_VERSION,
    };
    use crate::spatial_economic::{
        EvidenceKind, EvidenceState, Provenance, Quantity, SpatialContext, TimeContext,
        EVIDENCE_SCHEMA_VERSION,
    };
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct PlansFixture {
        plans: Vec<NetworkInterventionPlan>,
    }

    #[derive(Deserialize)]
    struct ComparisonsFixture {
        comparisons: Vec<NetworkInterventionComparison>,
    }

    const AT: &str = "2026-10-16T10:00:00Z";

    fn measurement(entity_id: &str, value: f64, unit: &str) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity_id.into(),
            quantity: Quantity { value, unit: unit.into(), uncertainty_abs: Some(0.0) },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic capacity assumption".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn link(id: &str, from: &str, to: &str, capacity: f64, state: NetworkLinkAvailability) -> NetworkLink {
        NetworkLink {
            id: id.into(),
            from_entity_id: from.into(),
            to_entity_id: to.into(),
            capacity: measurement(id, capacity, "m3/h"),
            availability: state,
            availability_basis: ScenarioBasis::Assumed,
            availability_rationale: "Synthetic scenario".into(),
            availability_supporting_evidence_ids: Vec::new(),
        }
    }

    fn baseline() -> ServiceCapacityNetworkScenario {
        ServiceCapacityNetworkScenario {
            schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
            id: "scenario:baseline".into(),
            as_of: AT.into(),
            service_entity_id: "service:water".into(),
            topology_completeness: NetworkTopologyCompleteness::Complete,
            source_entity_ids: vec!["source:reservoir".into()],
            required_capacity: measurement("service:water", 8.0, "m3/h"),
            links: vec![
                link("trunk", "source:reservoir", "junction", 5.0, NetworkLinkAvailability::Active),
                link("route-a", "junction", "service:water", 4.0, NetworkLinkAvailability::Active),
                link("route-b", "junction", "service:water", 4.0, NetworkLinkAvailability::Active),
                link("backup", "source:reservoir", "service:water", 5.0, NetworkLinkAvailability::Unknown),
            ],
        }
    }

    fn plan(id: &str, action: NetworkInterventionAction, rationale: &str) -> NetworkInterventionPlan {
        NetworkInterventionPlan { id: id.into(), rationale: rationale.into(), action }
    }

    fn evidence_for_trunk() -> EvidenceRecord {
        EvidenceRecord {
            id: "ev:observed-trunk".into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:network-intervention-tests".into(),
            subject_id: "trunk".into(),
            spatial: SpatialContext {
                entity_id: "trunk".into(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: AT.into(),
                recorded_at: AT.into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic observed trunk capacity".into(),
            quantity: Some(Quantity {
                value: 5.0,
                unit: "m3/h".into(),
                uncertainty_abs: Some(0.0),
            }),
            money: None,
            provenance: Provenance {
                method: "synthetic_test".into(),
                source_artifacts: Vec::new(),
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:synthetic-test".into(),
        }
    }

    #[test]
    fn network_intervention_fixture_matches_golden_ranked_results() {
        let baseline: ServiceCapacityNetworkScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/service-capacity-network-intervention-baseline.json"
        ))
        .unwrap();
        let plans: PlansFixture = serde_json::from_str(include_str!(
            "../../schemas/examples/network-intervention-plans.json"
        ))
        .unwrap();
        let expected: ComparisonsFixture = serde_json::from_str(include_str!(
            "../../schemas/examples/network-intervention-comparisons.json"
        ))
        .unwrap();

        let actual = compare_network_interventions(&baseline, &plans.plans, &[]).unwrap();
        assert_eq!(actual, expected.comparisons);
        assert_eq!(actual[0].intervention_id, "activate-backup");
        assert_eq!(actual[0].conservative_capacity_gain, 5.0);
        assert_eq!(actual[1].intervention_id, "capacity-upgrade");
        assert_eq!(actual[1].conservative_capacity_gain, 3.0);
    }

    #[test]
    fn candidate_actions_are_ranked_by_conservative_flow_gain() {
        let result = compare_network_interventions(
            &baseline(),
            &[
                plan(
                    "capacity-upgrade",
                    NetworkInterventionAction::IncreaseLinkCapacity {
                        link_id: "trunk".into(),
                        target_capacity_value: 9.0,
                        target_uncertainty_abs: 0.0,
                        unit: "m3/h".into(),
                    },
                    "Synthetic expansion of the shared upstream trunk.",
                ),
                plan(
                    "activate-backup",
                    NetworkInterventionAction::AssumeLinkActive { link_id: "backup".into() },
                    "Synthetic scenario assumes the backup route is restored.",
                ),
            ],
            &[],
        ).unwrap();
        assert_eq!(result[0].intervention_id, "activate-backup");
        assert_eq!(result[0].baseline_conservative_capacity, 5.0);
        assert_eq!(result[0].after_conservative_capacity, 10.0);
        assert_eq!(result[0].conservative_capacity_gain, 5.0);
        assert_eq!(result[0].after_state, NetworkAssessmentState::Adequate);
        assert_eq!(result[1].intervention_id, "capacity-upgrade");
        assert_eq!(result[1].conservative_capacity_gain, 3.0);
        assert_eq!(result[1].after_conservative_capacity, 8.0);
    }

    #[test]
    fn assumed_capacity_override_does_not_reuse_observation_evidence() {
        let mut input = baseline();
        let trunk = input.links.iter_mut().find(|link| link.id == "trunk").unwrap();
        trunk.capacity.basis = ScenarioBasis::Observed;
        trunk.capacity.supporting_evidence_ids = vec!["ev:observed-trunk".into()];
        let result = compare_network_interventions(
            &input,
            &[plan(
                "upgrade",
                NetworkInterventionAction::IncreaseLinkCapacity {
                    link_id: "trunk".into(),
                    target_capacity_value: 9.0,
                    target_uncertainty_abs: 0.0,
                    unit: "m3/h".into(),
                },
                "Explicit hypothetical trunk upgrade.",
            )],
            &[evidence_for_trunk()],
        ).unwrap();
        assert!(result[0].baseline_capacity_evidence_ids.contains(&"ev:observed-trunk".into()));
        assert!(!result[0].after_capacity_evidence_ids.contains(&"ev:observed-trunk".into()));
        assert_eq!(result[0].after_conservative_capacity, 8.0);
    }

    #[test]
    fn incomplete_topology_and_bad_candidates_fail_closed() {
        let mut incomplete = baseline();
        incomplete.topology_completeness = NetworkTopologyCompleteness::Unknown;
        assert!(matches!(
            compare_network_interventions(
                &incomplete,
                &[plan("activate", NetworkInterventionAction::AssumeLinkActive { link_id: "backup".into() }, "Hypothesis.")],
                &[],
            ),
            Err(NetworkInterventionError::InvalidInput(errors))
                if errors.contains(&NetworkInterventionValidationError::IncompleteBaselineTopology)
        ));

        assert!(matches!(
            compare_network_interventions(&baseline(), &[], &[]),
            Err(NetworkInterventionError::InvalidInput(errors))
                if errors.contains(&NetworkInterventionValidationError::EmptyPlanSet)
        ));
        assert!(matches!(
            compare_network_interventions(
                &baseline(),
                &[plan("bad", NetworkInterventionAction::AssumeLinkActive { link_id: "missing".into() }, "Hypothesis.")],
                &[],
            ),
            Err(NetworkInterventionError::InvalidInput(errors))
                if errors.contains(&NetworkInterventionValidationError::UnknownLink)
        ));
    }

    #[test]
    fn duplicate_plan_ids_and_non_increasing_capacity_are_rejected() {
        let duplicate = plan(
            "same",
            NetworkInterventionAction::AssumeLinkActive { link_id: "backup".into() },
            "Hypothesis.",
        );
        assert!(matches!(
            compare_network_interventions(&baseline(), &[duplicate.clone(), duplicate], &[]),
            Err(NetworkInterventionError::InvalidInput(errors))
                if errors.contains(&NetworkInterventionValidationError::DuplicateInterventionId)
        ));
        assert!(matches!(
            compare_network_interventions(
                &baseline(),
                &[plan(
                    "no-op",
                    NetworkInterventionAction::IncreaseLinkCapacity {
                        link_id: "trunk".into(),
                        target_capacity_value: 5.0,
                        target_uncertainty_abs: 0.0,
                        unit: "m3/h".into(),
                    },
                    "Hypothesis.",
                )],
                &[],
            ),
            Err(NetworkInterventionError::InvalidInput(errors))
                if errors.contains(&NetworkInterventionValidationError::TargetCapacityMustIncrease)
        ));
    }
}
