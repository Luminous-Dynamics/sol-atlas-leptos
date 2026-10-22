// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Reproducible baseline-versus-intervention comparisons.
//!
//! A comparison is a modelled qualitative delta between two explicit scenario
//! inputs. It does not establish that a real intervention occurred, that the
//! world changed, or that the intervention caused an outcome.

use crate::dependency_logic::{
    AvailabilityScenario, DependencyGroup, DependencyGroupAssessment, DependencyGroupState,
    ScenarioBasis, ScenarioEntityCondition, ScenarioEvaluationError,
};
use crate::spatial_economic::EvidenceRecord;
use crate::spatial_graph::DependencyGraph;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionPlan {
    pub id: String,
    pub rationale: String,
    /// Explicit what-if overrides applied to a cloned baseline scenario.
    /// Overrides must be assumptions; an intervention proposal is not observed
    /// evidence. Entity IDs absent from this map retain baseline conditions.
    pub condition_overrides: BTreeMap<String, ScenarioEntityCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeChange {
    /// Before and after have the same qualitative state.
    Unchanged,
    /// Disrupted in the baseline, satisfied with the intervention assumptions.
    Recovered,
    /// Satisfied in the baseline, disrupted with the intervention assumptions.
    Degraded,
    /// Indeterminate in the baseline, satisfied with the intervention assumptions.
    ResolvedToSatisfied,
    /// Indeterminate in the baseline, disrupted with the intervention assumptions.
    ResolvedToDisrupted,
    /// A previously determinate state became indeterminate.
    BecameIndeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupInterventionDelta {
    pub group_id: String,
    pub dependent_entity_id: String,
    pub change: OutcomeChange,
    pub baseline: DependencyGroupAssessment,
    pub with_intervention: DependencyGroupAssessment,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionSummary {
    pub groups_compared: usize,
    pub recovered: usize,
    pub degraded: usize,
    pub resolved_to_satisfied: usize,
    pub resolved_to_disrupted: usize,
    pub became_indeterminate: usize,
    pub unchanged: usize,
}

pub const INTERVENTION_COMPARISON_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterventionComparison {
    pub schema_version: String,
    pub intervention_id: String,
    pub intervention_rationale: String,
    pub baseline_scenario_id: String,
    pub intervention_scenario_id: String,
    pub as_of: String,
    /// Exact qualitative baseline assumptions/observations used by the comparison.
    pub baseline_conditions: BTreeMap<String, ScenarioEntityCondition>,
    /// Hypothetical overrides; they are not claims that work occurred.
    pub intervention_overrides: BTreeMap<String, ScenarioEntityCondition>,
    pub group_deltas: Vec<GroupInterventionDelta>,
    pub summary: InterventionSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterventionComparisonError {
    EmptyPlanId,
    EmptyPlanRationale,
    EmptyOverrides,
    EmptyOverrideEntityId,
    EmptyOverrideRationale,
    OverrideMustBeAssumed,
    OverrideCannotClaimObservedEvidence,
    BaselineEvaluation(ScenarioEvaluationError),
    InterventionEvaluation(ScenarioEvaluationError),
    ComparisonResultCountMismatch,
}

fn classify_change(baseline: DependencyGroupState, after: DependencyGroupState) -> OutcomeChange {
    use DependencyGroupState::{Disrupted, Indeterminate, Satisfied};

    match (baseline, after) {
        (Satisfied, Satisfied) | (Disrupted, Disrupted) | (Indeterminate, Indeterminate) => {
            OutcomeChange::Unchanged
        }
        (Disrupted, Satisfied) => OutcomeChange::Recovered,
        (Satisfied, Disrupted) => OutcomeChange::Degraded,
        (Indeterminate, Satisfied) => OutcomeChange::ResolvedToSatisfied,
        (Indeterminate, Disrupted) => OutcomeChange::ResolvedToDisrupted,
        (Satisfied | Disrupted, Indeterminate) => OutcomeChange::BecameIndeterminate,
    }
}

/// Compare all dependency groups against a baseline and explicit intervention
/// assumptions. The batch evaluator builds evidence indexes once per scenario
/// and sorts groups by stable ID so output ordering is reproducible.
///
/// Conditions not overridden by the plan are inherited from the baseline.
/// Every override must be an explicit assumption with a non-empty rationale and
/// no supporting-evidence IDs; real observations belong in the baseline state.
pub fn compare_intervention(
    graph: &DependencyGraph,
    groups: &[DependencyGroup],
    baseline: &AvailabilityScenario,
    intervention: &InterventionPlan,
    evidence_records: &[EvidenceRecord],
) -> Result<InterventionComparison, InterventionComparisonError> {
    if intervention.id.trim().is_empty() {
        return Err(InterventionComparisonError::EmptyPlanId);
    }
    if intervention.rationale.trim().is_empty() {
        return Err(InterventionComparisonError::EmptyPlanRationale);
    }
    if intervention.condition_overrides.is_empty() {
        return Err(InterventionComparisonError::EmptyOverrides);
    }

    for (entity_id, condition) in &intervention.condition_overrides {
        if entity_id.trim().is_empty() {
            return Err(InterventionComparisonError::EmptyOverrideEntityId);
        }
        if condition.rationale.trim().is_empty() {
            return Err(InterventionComparisonError::EmptyOverrideRationale);
        }
        if condition.basis != ScenarioBasis::Assumed {
            return Err(InterventionComparisonError::OverrideMustBeAssumed);
        }
        if !condition.supporting_evidence_ids.is_empty() {
            return Err(InterventionComparisonError::OverrideCannotClaimObservedEvidence);
        }
    }

    let baseline_assessments = graph
        .evaluate_dependency_groups_at(groups, baseline, evidence_records)
        .map_err(InterventionComparisonError::BaselineEvaluation)?;

    let mut with_intervention = baseline.clone();
    with_intervention.id = format!("{}-with-{}", baseline.id, intervention.id);
    for (entity_id, condition) in &intervention.condition_overrides {
        with_intervention
            .entity_conditions
            .insert(entity_id.clone(), condition.clone());
    }

    let intervention_assessments = graph
        .evaluate_dependency_groups_at(groups, &with_intervention, evidence_records)
        .map_err(InterventionComparisonError::InterventionEvaluation)?;

    if baseline_assessments.len() != intervention_assessments.len() {
        // Defensive fail-closed guard; the batch evaluator should preserve length.
        return Err(InterventionComparisonError::ComparisonResultCountMismatch);
    }

    let mut summary = InterventionSummary {
        groups_compared: baseline_assessments.len(),
        ..InterventionSummary::default()
    };
    let mut group_deltas = Vec::with_capacity(baseline_assessments.len());

    for (before, after) in baseline_assessments
        .into_iter()
        .zip(intervention_assessments.into_iter())
    {
        let change = classify_change(before.state, after.state);
        match change {
            OutcomeChange::Unchanged => summary.unchanged += 1,
            OutcomeChange::Recovered => summary.recovered += 1,
            OutcomeChange::Degraded => summary.degraded += 1,
            OutcomeChange::ResolvedToSatisfied => summary.resolved_to_satisfied += 1,
            OutcomeChange::ResolvedToDisrupted => summary.resolved_to_disrupted += 1,
            OutcomeChange::BecameIndeterminate => summary.became_indeterminate += 1,
        }

        group_deltas.push(GroupInterventionDelta {
            group_id: before.group_id.clone(),
            dependent_entity_id: before.dependent_entity_id.clone(),
            change,
            baseline: before,
            with_intervention: after,
        });
    }

    Ok(InterventionComparison {
        schema_version: INTERVENTION_COMPARISON_SCHEMA_VERSION.into(),
        intervention_id: intervention.id.clone(),
        intervention_rationale: intervention.rationale.clone(),
        baseline_scenario_id: baseline.id.clone(),
        intervention_scenario_id: with_intervention.id,
        as_of: baseline.as_of.clone(),
        baseline_conditions: baseline.entity_conditions.clone(),
        intervention_overrides: intervention.condition_overrides.clone(),
        group_deltas,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::{DependencyLogic, EntityOperationalState};
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceRecord, EvidenceState, Provenance,
        SpatialContext, TimeContext,
    };
    use crate::spatial_graph::{RelationKind, SpatialRelation};
    use std::collections::BTreeMap;

    fn evidence(id: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:intervention-tests".into(),
            subject_id: "urn:example:feeder".into(),
            spatial: SpatialContext {
                entity_id: "urn:example:feeder".into(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: "2026-10-15T10:00:00Z".into(),
                recorded_at: "2026-10-15T10:01:00Z".into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic relationship evidence for unit tests".into(),
            quantity: None,
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

    fn condition(state: EntityOperationalState) -> ScenarioEntityCondition {
        ScenarioEntityCondition {
            state,
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic what-if condition".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario(
        id: &str,
        conditions: impl IntoIterator<Item = (String, ScenarioEntityCondition)>,
    ) -> AvailabilityScenario {
        AvailabilityScenario {
            id: id.into(),
            as_of: "2026-10-15T12:00:00Z".into(),
            entity_conditions: conditions.into_iter().collect(),
        }
    }

    fn relation(id: &str, prerequisite: &str, ev: &str) -> SpatialRelation {
        SpatialRelation {
            id: id.into(),
            subject_entity_id: "pump".into(),
            kind: RelationKind::SuppliedBy,
            object_entity_id: prerequisite.into(),
            valid_from: None,
            valid_until: None,
            supporting_evidence_ids: vec![ev.into()],
        }
    }

    fn fixture_graph_and_group() -> (DependencyGraph, DependencyGroup, Vec<EvidenceRecord>) {
        let graph = DependencyGraph {
            relations: vec![
                relation("pump-primary", "feeder-a", "ev-primary"),
                relation("pump-backup", "feeder-b", "ev-backup"),
            ],
        };
        let group = DependencyGroup {
            id: "pump-power".into(),
            dependent_entity_id: "pump".into(),
            logic: DependencyLogic::AnyOne,
            relation_ids: vec!["pump-primary".into(), "pump-backup".into()],
        };
        (
            graph,
            group,
            vec![evidence("ev-primary"), evidence("ev-backup")],
        )
    }

    fn plan(
        overrides: impl IntoIterator<Item = (String, ScenarioEntityCondition)>,
    ) -> InterventionPlan {
        InterventionPlan {
            id: "restore-backup-feeder".into(),
            rationale: "Synthetic plan for exercising comparison semantics".into(),
            condition_overrides: overrides.into_iter().collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn intervention_recovers_a_disrupted_redundant_group() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let baseline = scenario(
            "baseline",
            [
                (
                    "feeder-a".into(),
                    condition(EntityOperationalState::Unavailable),
                ),
                (
                    "feeder-b".into(),
                    condition(EntityOperationalState::Unavailable),
                ),
            ],
        );
        let plan = plan([(
            "feeder-b".into(),
            condition(EntityOperationalState::Available),
        )]);

        let result = compare_intervention(&graph, &[group], &baseline, &plan, &evidence).unwrap();
        assert_eq!(
            result.group_deltas[0].baseline.state,
            DependencyGroupState::Disrupted
        );
        assert_eq!(
            result.group_deltas[0].with_intervention.state,
            DependencyGroupState::Satisfied
        );
        assert_eq!(result.group_deltas[0].change, OutcomeChange::Recovered);
        assert_eq!(result.summary.recovered, 1);
        assert_eq!(result.summary.groups_compared, 1);
    }

    #[test]
    fn intervention_can_degrade_a_satisfied_group() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let baseline = scenario(
            "baseline",
            [
                (
                    "feeder-a".into(),
                    condition(EntityOperationalState::Available),
                ),
                (
                    "feeder-b".into(),
                    condition(EntityOperationalState::Available),
                ),
            ],
        );
        let plan = plan([
            (
                "feeder-a".into(),
                condition(EntityOperationalState::Unavailable),
            ),
            (
                "feeder-b".into(),
                condition(EntityOperationalState::Unavailable),
            ),
        ]);

        let result = compare_intervention(&graph, &[group], &baseline, &plan, &evidence).unwrap();
        assert_eq!(result.group_deltas[0].change, OutcomeChange::Degraded);
        assert_eq!(result.summary.degraded, 1);
    }

    #[test]
    fn intervention_can_resolve_indeterminate_baseline_to_satisfied() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let baseline = scenario("baseline", []);
        let plan = plan([(
            "feeder-b".into(),
            condition(EntityOperationalState::Available),
        )]);

        let result = compare_intervention(&graph, &[group], &baseline, &plan, &evidence).unwrap();
        assert_eq!(
            result.group_deltas[0].baseline.state,
            DependencyGroupState::Indeterminate
        );
        assert_eq!(
            result.group_deltas[0].with_intervention.state,
            DependencyGroupState::Satisfied
        );
        assert_eq!(
            result.group_deltas[0].change,
            OutcomeChange::ResolvedToSatisfied
        );
    }

    #[test]
    fn unknown_baseline_resolved_to_disruption_is_not_called_degradation() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let baseline = scenario("baseline", []);
        let plan = plan([
            (
                "feeder-a".into(),
                condition(EntityOperationalState::Unavailable),
            ),
            (
                "feeder-b".into(),
                condition(EntityOperationalState::Unavailable),
            ),
        ]);

        let result = compare_intervention(&graph, &[group], &baseline, &plan, &evidence).unwrap();
        assert_eq!(
            result.group_deltas[0].change,
            OutcomeChange::ResolvedToDisrupted
        );
        assert_eq!(result.summary.resolved_to_disrupted, 1);
        assert_eq!(result.summary.degraded, 0);
    }

    #[test]
    fn no_op_intervention_is_reported_as_unchanged() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let baseline = scenario(
            "baseline",
            [
                (
                    "feeder-a".into(),
                    condition(EntityOperationalState::Available),
                ),
                (
                    "feeder-b".into(),
                    condition(EntityOperationalState::Unavailable),
                ),
            ],
        );
        let plan = plan([(
            "feeder-b".into(),
            condition(EntityOperationalState::Unavailable),
        )]);

        let result = compare_intervention(&graph, &[group], &baseline, &plan, &evidence).unwrap();
        assert_eq!(result.group_deltas[0].change, OutcomeChange::Unchanged);
        assert_eq!(result.summary.unchanged, 1);
    }

    #[test]
    fn proposed_intervention_cannot_masquerade_as_observed_evidence() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let baseline = scenario(
            "baseline",
            [
                (
                    "feeder-a".into(),
                    condition(EntityOperationalState::Unavailable),
                ),
                (
                    "feeder-b".into(),
                    condition(EntityOperationalState::Unavailable),
                ),
            ],
        );
        let mut observed = condition(EntityOperationalState::Available);
        observed.basis = ScenarioBasis::Observed;
        observed.supporting_evidence_ids = vec!["ev-backup".into()];
        let plan = plan([("feeder-b".into(), observed)]);

        assert_eq!(
            compare_intervention(&graph, &[group], &baseline, &plan, &evidence),
            Err(InterventionComparisonError::OverrideMustBeAssumed)
        );
    }

    #[test]
    fn proposed_intervention_cannot_attach_observation_ids_to_assumption() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let baseline = scenario("baseline", []);
        let mut assumed = condition(EntityOperationalState::Available);
        assumed.supporting_evidence_ids = vec!["ev-backup".into()];
        let plan = plan([("feeder-b".into(), assumed)]);

        assert_eq!(
            compare_intervention(&graph, &[group], &baseline, &plan, &evidence),
            Err(InterventionComparisonError::OverrideCannotClaimObservedEvidence)
        );
    }

    #[test]
    fn duplicate_group_ids_fail_closed() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let baseline = scenario("baseline", []);
        let plan = plan([(
            "feeder-b".into(),
            condition(EntityOperationalState::Available),
        )]);

        assert_eq!(
            compare_intervention(&graph, &[group.clone(), group], &baseline, &plan, &evidence),
            Err(InterventionComparisonError::BaselineEvaluation(
                ScenarioEvaluationError::DuplicateGroupId
            ))
        );
    }

    #[test]
    fn result_order_is_stable_when_group_input_order_changes() {
        let (graph, group, evidence) = fixture_graph_and_group();
        let other = DependencyGroup {
            id: "z-later".into(),
            ..group.clone()
        };
        let earlier = DependencyGroup {
            id: "a-earlier".into(),
            ..group
        };
        let baseline = scenario(
            "baseline",
            [
                (
                    "feeder-a".into(),
                    condition(EntityOperationalState::Unavailable),
                ),
                (
                    "feeder-b".into(),
                    condition(EntityOperationalState::Unavailable),
                ),
            ],
        );
        let plan = plan([(
            "feeder-b".into(),
            condition(EntityOperationalState::Available),
        )]);

        let first = compare_intervention(
            &graph,
            &[other.clone(), earlier.clone()],
            &baseline,
            &plan,
            &evidence,
        )
        .unwrap();
        let second =
            compare_intervention(&graph, &[earlier, other], &baseline, &plan, &evidence).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first
                .group_deltas
                .iter()
                .map(|item| item.group_id.as_str())
                .collect::<Vec<_>>(),
            vec!["a-earlier", "z-later"]
        );
    }

    #[test]
    fn synthetic_fixture_compares_outage_baseline_with_intervention_plan() {
        #[derive(Deserialize)]
        struct GroupFixture {
            groups: Vec<DependencyGroup>,
        }

        let graph: DependencyGraph = serde_json::from_str(include_str!(
            "../../schemas/examples/water-system-dependencies.json"
        ))
        .unwrap();
        let groups: GroupFixture = serde_json::from_str(include_str!(
            "../../schemas/examples/water-system-dependency-groups.json"
        ))
        .unwrap();
        let baseline: AvailabilityScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/water-system-outage-baseline.json"
        ))
        .unwrap();
        let plan: InterventionPlan = serde_json::from_str(include_str!(
            "../../schemas/examples/water-system-intervention-plan.json"
        ))
        .unwrap();
        let plan_schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/spatial-economic-intervention-plan-v0.1.schema.json"
        ))
        .unwrap();
        let comparison_schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/spatial-economic-intervention-comparison-v0.1.schema.json"
        ))
        .unwrap();
        assert_eq!(
            comparison_schema["properties"]["schema_version"]["const"],
            serde_json::json!(INTERVENTION_COMPARISON_SCHEMA_VERSION)
        );
        assert!(
            comparison_schema["$defs"]["delta"]["properties"]["change"]["enum"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("recovered"))
        );
        assert_eq!(
            plan_schema["properties"]["condition_overrides"]["minProperties"],
            serde_json::json!(1)
        );
        assert_eq!(
            plan_schema["properties"]["condition_overrides"]["additionalProperties"]["properties"]
                ["basis"]["const"],
            serde_json::json!("assumed")
        );
        let evidence_records = [
            evidence("synthetic-evidence-pump-power"),
            evidence("synthetic-evidence-pump-backup-power"),
            evidence("synthetic-evidence-field-pump"),
            evidence("synthetic-evidence-warehouse-location"),
        ];

        let comparison =
            compare_intervention(&graph, &groups.groups, &baseline, &plan, &evidence_records)
                .unwrap();

        assert_eq!(comparison.summary.groups_compared, 2);
        assert_eq!(comparison.summary.recovered, 1);
        assert_eq!(comparison.summary.unchanged, 1);
        let pump = comparison
            .group_deltas
            .iter()
            .find(|delta| delta.group_id == "synthetic-pump-power-resilience")
            .unwrap();
        assert_eq!(pump.baseline.state, DependencyGroupState::Disrupted);
        assert_eq!(
            pump.with_intervention.state,
            DependencyGroupState::Satisfied
        );
        assert_eq!(pump.change, OutcomeChange::Recovered);
        let backup_member = pump
            .with_intervention
            .members
            .iter()
            .find(|member| member.relation_id == "synthetic-edge-pump-backup-power")
            .unwrap();
        assert_eq!(
            backup_member.supporting_evidence_ids,
            vec![String::from("synthetic-evidence-pump-backup-power")]
        );
    }

    #[test]
    fn all_nine_qualitative_state_transitions_are_classified_explicitly() {
        use DependencyGroupState::{Disrupted, Indeterminate, Satisfied};
        let cases = [
            ((Satisfied, Satisfied), OutcomeChange::Unchanged),
            ((Satisfied, Disrupted), OutcomeChange::Degraded),
            (
                (Satisfied, Indeterminate),
                OutcomeChange::BecameIndeterminate,
            ),
            ((Disrupted, Satisfied), OutcomeChange::Recovered),
            ((Disrupted, Disrupted), OutcomeChange::Unchanged),
            (
                (Disrupted, Indeterminate),
                OutcomeChange::BecameIndeterminate,
            ),
            (
                (Indeterminate, Satisfied),
                OutcomeChange::ResolvedToSatisfied,
            ),
            (
                (Indeterminate, Disrupted),
                OutcomeChange::ResolvedToDisrupted,
            ),
            ((Indeterminate, Indeterminate), OutcomeChange::Unchanged),
        ];
        for ((before, after), expected) in cases {
            assert_eq!(classify_change(before, after), expected);
        }
    }
}
