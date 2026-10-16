// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Reproducible, dependency-free microbenchmark for spatial-economic intervention
//! comparison. Run with:
//! SEI_BENCH_SAMPLES=100 cargo bench --bench spatial_economic_intervention
//!
//! This harness reports observations; it does not assert that targets have passed.

use serde::{Deserialize, Serialize};
use sol_atlas_core::common_cause::{
    assess_common_cause_domains, correlate_common_cause_with_dependency_groups,
    CommonCauseBasis, CommonCauseCondition, CommonCauseDomain, CommonCauseScenario,
    CommonCauseState, FailureMechanism,
};
use sol_atlas_core::dependency_logic::{
    AvailabilityScenario, DependencyGroup, DependencyGroupState, DependencyLogic,
    EntityOperationalState, ScenarioBasis, ScenarioEntityCondition,
};
use sol_atlas_core::intervention::{compare_intervention, InterventionPlan};
use sol_atlas_core::spatial_economic::{
    EvidenceKind, EvidenceRecord, EvidenceState, Provenance, SpatialContext, TimeContext,
    EVIDENCE_SCHEMA_VERSION,
};
use sol_atlas_core::service_function::{
    assess_service_functions, FunctionMeasurement, ServiceFunctionScenario, ServiceFunctionState,
    SERVICE_FUNCTION_SCHEMA_VERSION,
};
use sol_atlas_core::spatial_economic::Quantity;
use sol_atlas_core::spatial_graph::{DependencyGraph, RelationKind, SpatialRelation};
use std::collections::BTreeMap;
use std::env;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct SampleSummary {
    groups: usize,
    relations: usize,
    evidence_records: usize,
    common_cause_evidence_records: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    recovered_groups_per_run: usize,
    result_bytes: usize,
    common_cause_overlap_pairs: usize,
    common_cause_p50_micros: u128,
    common_cause_p95_micros: u128,
    common_cause_max_micros: u128,
    functional_assessments_per_run: usize,
    functional_evidence_records: usize,
    functional_result_bytes: usize,
    functional_p50_micros: u128,
    functional_p95_micros: u128,
    functional_max_micros: u128,
}

fn evidence(id: String, feeder_id: &str) -> EvidenceRecord {
    EvidenceRecord {
        id,
        schema_version: EVIDENCE_SCHEMA_VERSION.into(),
        kind: EvidenceKind::Observation,
        state: EvidenceState::Asserted,
        issuer_id: "did:example:sei-benchmark".into(),
        subject_id: feeder_id.into(),
        spatial: SpatialContext {
            entity_id: feeder_id.into(),
            geometry_uri: None,
            crs: None,
        },
        time: TimeContext {
            occurred_at: "2026-10-15T10:00:00Z".into(),
            recorded_at: "2026-10-15T10:01:00Z".into(),
            valid_from: None,
            valid_until: None,
        },
        statement: "Synthetic benchmark relationship evidence".into(),
        quantity: None,
        money: None,
        provenance: Provenance {
            method: "synthetic_benchmark".into(),
            source_artifacts: Vec::new(),
            derivation_id: None,
        },
        supersedes: None,
        access_policy: "policy:synthetic-benchmark".into(),
    }
}

fn quantity_evidence(
    id: String,
    entity_id: &str,
    value: f64,
    unit: &str,
    uncertainty_abs: f64,
) -> EvidenceRecord {
    let mut record = evidence(id, entity_id);
    record.quantity = Some(Quantity {
        value,
        unit: unit.into(),
        uncertainty_abs: Some(uncertainty_abs),
    });
    record
}

fn functional_fixture(
    count: usize,
) -> (Vec<ServiceFunctionScenario>, Vec<EvidenceRecord>) {
    let mut scenarios = Vec::with_capacity(count);
    let mut records = Vec::with_capacity(count.saturating_mul(2));
    for index in 0..count {
        let service_id = format!("service-capacity-{index:06}");
        let capacity_entity = format!("pump-capacity-{index:06}");
        let capacity_evidence_id = format!("capacity-evidence-{index:06}");
        let requirement_evidence_id = format!("requirement-evidence-{index:06}");
        let capacity_quantity = Quantity {
            value: 70.0,
            unit: "m3/h".into(),
            uncertainty_abs: Some(5.0),
        };
        let requirement_quantity = Quantity {
            value: 100.0,
            unit: "m3/h".into(),
            uncertainty_abs: Some(10.0),
        };
        records.push(quantity_evidence(
            capacity_evidence_id.clone(),
            &capacity_entity,
            capacity_quantity.value,
            &capacity_quantity.unit,
            capacity_quantity.uncertainty_abs.unwrap_or_default(),
        ));
        records.push(quantity_evidence(
            requirement_evidence_id.clone(),
            &service_id,
            requirement_quantity.value,
            &requirement_quantity.unit,
            requirement_quantity.uncertainty_abs.unwrap_or_default(),
        ));
        scenarios.push(ServiceFunctionScenario {
            schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
            id: format!("functional-scenario-{index:06}"),
            as_of: "2026-10-15T12:00:00Z".into(),
            service_entity_id: service_id.clone(),
            available_capacity: FunctionMeasurement {
                entity_id: capacity_entity,
                quantity: capacity_quantity,
                basis: ScenarioBasis::Observed,
                rationale: "Synthetic observed capacity fixture".into(),
                supporting_evidence_ids: vec![capacity_evidence_id],
            },
            required_capacity: FunctionMeasurement {
                entity_id: service_id,
                quantity: requirement_quantity,
                basis: ScenarioBasis::Observed,
                rationale: "Synthetic observed demand fixture".into(),
                supporting_evidence_ids: vec![requirement_evidence_id],
            },
        });
    }
    (scenarios, records)
}

fn assumed(state: EntityOperationalState, rationale: &str) -> ScenarioEntityCondition {
    ScenarioEntityCondition {
        state,
        basis: ScenarioBasis::Assumed,
        rationale: rationale.into(),
        supporting_evidence_ids: Vec::new(),
    }
}

fn fixture(groups_count: usize) -> (
    DependencyGraph,
    Vec<DependencyGroup>,
    AvailabilityScenario,
    InterventionPlan,
    Vec<EvidenceRecord>,
) {
    let mut relations = Vec::with_capacity(groups_count.saturating_mul(2));
    let mut groups = Vec::with_capacity(groups_count);
    let mut evidence_records = Vec::with_capacity(groups_count.saturating_mul(2));

    for index in 0..groups_count {
        let service_id = format!("service-{index:06}");
        let primary_relation_id = format!("relation-{index:06}-primary");
        let backup_relation_id = format!("relation-{index:06}-backup");
        let primary_evidence_id = format!("evidence-{index:06}-primary");
        let backup_evidence_id = format!("evidence-{index:06}-backup");

        relations.push(SpatialRelation {
            id: primary_relation_id.clone(),
            subject_entity_id: service_id.clone(),
            kind: RelationKind::SuppliedBy,
            object_entity_id: "feeder-primary".into(),
            valid_from: None,
            valid_until: None,
            supporting_evidence_ids: vec![primary_evidence_id.clone()],
        });
        relations.push(SpatialRelation {
            id: backup_relation_id.clone(),
            subject_entity_id: service_id.clone(),
            kind: RelationKind::SuppliedBy,
            object_entity_id: "feeder-backup".into(),
            valid_from: None,
            valid_until: None,
            supporting_evidence_ids: vec![backup_evidence_id.clone()],
        });
        groups.push(DependencyGroup {
            id: format!("group-{index:06}"),
            dependent_entity_id: service_id,
            logic: DependencyLogic::AnyOne,
            relation_ids: vec![primary_relation_id, backup_relation_id],
        });
        evidence_records.push(evidence(primary_evidence_id, "feeder-primary"));
        evidence_records.push(evidence(backup_evidence_id, "feeder-backup"));
    }

    let baseline = AvailabilityScenario {
        id: "benchmark-baseline".into(),
        as_of: "2026-10-15T12:00:00Z".into(),
        entity_conditions: [
            (
                "feeder-primary".into(),
                assumed(
                    EntityOperationalState::Unavailable,
                    "Synthetic baseline: primary feeder unavailable",
                ),
            ),
            (
                "feeder-backup".into(),
                assumed(
                    EntityOperationalState::Unavailable,
                    "Synthetic baseline: backup feeder unavailable",
                ),
            ),
        ]
        .into_iter()
        .collect::<BTreeMap<_, _>>(),
    };
    let intervention = InterventionPlan {
        id: "benchmark-restore-backup".into(),
        rationale: "Synthetic intervention: assume backup feeder becomes available".into(),
        condition_overrides: [(
            "feeder-backup".into(),
            assumed(
                EntityOperationalState::Available,
                "Synthetic intervention assumption: backup feeder available",
            ),
        )]
        .into_iter()
        .collect(),
    };

    (
        DependencyGraph { relations },
        groups,
        baseline,
        intervention,
        evidence_records,
    )
}

fn percentile(sorted: &[Duration], percentile: usize) -> Duration {
    let last = sorted.len().saturating_sub(1);
    let index = (sorted.len().saturating_mul(percentile).saturating_add(99) / 100)
        .saturating_sub(1)
        .min(last);
    sorted[index]
}

fn main() {
    let warmup_iterations = env::var("SEI_BENCH_WARMUP")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3usize);
    let measured_iterations = env::var("SEI_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(20usize)
        .max(1);

    for groups_count in [100usize, 1_000, 10_000] {
        let (graph, groups, baseline, intervention, evidence_records) = fixture(groups_count);
        let reference = compare_intervention(
            &graph,
            &groups,
            &baseline,
            &intervention,
            &evidence_records,
        )
        .expect("synthetic fixture must evaluate");
        assert_eq!(reference.summary.recovered, groups_count);
        let reference_bytes = serde_json::to_vec(&reference)
            .expect("deterministic comparison serializes");
        let common_cause_domain = CommonCauseDomain {
            id: "domain:shared-feeder".into(),
            cause_entity_id: "grid:shared-feeder".into(),
            mechanism: FailureMechanism::SharedPower,
            member_entity_ids: vec!["feeder-primary".into(), "feeder-backup".into()],
            supporting_evidence_ids: vec!["evidence-common-cause-domain".into()],
            valid_from: None,
            valid_until: None,
        };
        let common_cause_scenario = CommonCauseScenario {
            id: "benchmark-common-cause".into(),
            as_of: baseline.as_of.clone(),
            conditions: [(
                "domain:shared-feeder".into(),
                CommonCauseCondition {
                    state: CommonCauseState::Active,
                    basis: CommonCauseBasis::Assumed,
                    rationale: "Synthetic benchmark: a shared power failure mode is active".into(),
                    supporting_evidence_ids: Vec::new(),
                },
            )]
            .into_iter()
            .collect(),
        };
        let common_cause_evidence = [evidence(
            "evidence-common-cause-domain".into(),
            "domain:shared-feeder",
        )];
        let reference_common_cause = assess_common_cause_domains(
            std::slice::from_ref(&common_cause_domain),
            &common_cause_scenario,
            &common_cause_evidence,
        )
        .expect("synthetic common-cause fixture must evaluate");
        let intervention_groups = reference
            .group_deltas
            .iter()
            .map(|delta| delta.with_intervention.clone())
            .collect::<Vec<_>>();
        let reference_overlaps = correlate_common_cause_with_dependency_groups(
            &reference_common_cause,
            &intervention_groups,
        )
        .expect("same-time synthetic assessments must correlate");
        assert_eq!(reference_overlaps.len(), groups_count);
        assert!(reference_overlaps.iter().all(|overlap| {
            overlap.independent_redundancy_warning
                && overlap.currently_exposed_relation_ids.len() == 2
        }));
        let reference_overlap_bytes =
            serde_json::to_vec(&reference_overlaps).expect("overlaps serialize");

        let (functional_scenarios, functional_evidence) = functional_fixture(groups_count);
        let reference_functional = assess_service_functions(
            &functional_scenarios,
            &functional_evidence,
        )
        .expect("synthetic functional-adequacy fixtures must evaluate");
        assert_eq!(reference_functional.len(), groups_count);
        assert!(reference_functional
            .iter()
            .all(|assessment| assessment.state == ServiceFunctionState::Degraded));
        let reference_functional_bytes = serde_json::to_vec(&reference_functional)
            .expect("functional assessments serialize");

        for _ in 0..warmup_iterations {
            let result = compare_intervention(
                &graph,
                &groups,
                &baseline,
                &intervention,
                &evidence_records,
            )
            .expect("synthetic fixture must evaluate");
            assert_eq!(result.summary.recovered, groups_count);
            assert!(result
                .group_deltas
                .iter()
                .all(|delta| delta.with_intervention.state == DependencyGroupState::Satisfied));
            assert_eq!(
                serde_json::to_vec(&result).expect("comparison serializes"),
                reference_bytes
            );
            let common = assess_common_cause_domains(
                std::slice::from_ref(&common_cause_domain),
                &common_cause_scenario,
                &common_cause_evidence,
            )
            .expect("synthetic common-cause fixture must evaluate");
            let overlaps = correlate_common_cause_with_dependency_groups(
                &common,
                &intervention_groups,
            )
            .expect("same-time synthetic assessments must correlate");
            assert_eq!(overlaps.len(), groups_count);
            assert_eq!(
                serde_json::to_vec(&overlaps).expect("overlaps serialize"),
                reference_overlap_bytes
            );

            let functional = assess_service_functions(
                &functional_scenarios,
                &functional_evidence,
            )
            .expect("synthetic functional-adequacy fixtures must evaluate");
            assert_eq!(functional.len(), groups_count);
            assert!(functional
                .iter()
                .all(|assessment| assessment.state == ServiceFunctionState::Degraded));
            assert_eq!(
                serde_json::to_vec(&functional).expect("functional assessments serialize"),
                reference_functional_bytes
            );
        }

        let mut samples = Vec::with_capacity(measured_iterations);
        let mut common_cause_samples = Vec::with_capacity(measured_iterations);
        let mut functional_samples = Vec::with_capacity(measured_iterations);
        let mut recovered = 0;
        for _ in 0..measured_iterations {
            let start = Instant::now();
            let result = compare_intervention(
                &graph,
                &groups,
                &baseline,
                &intervention,
                &evidence_records,
            )
            .expect("synthetic fixture must evaluate");
            samples.push(start.elapsed());
            recovered = result.summary.recovered;
            assert_eq!(recovered, groups_count);
            assert_eq!(
                serde_json::to_vec(&result).expect("comparison serializes"),
                reference_bytes
            );

            let common_start = Instant::now();
            let common = assess_common_cause_domains(
                std::slice::from_ref(&common_cause_domain),
                &common_cause_scenario,
                &common_cause_evidence,
            )
            .expect("synthetic common-cause fixture must evaluate");
            let overlaps = correlate_common_cause_with_dependency_groups(
                &common,
                &intervention_groups,
            )
            .expect("same-time synthetic assessments must correlate");
            common_cause_samples.push(common_start.elapsed());
            assert_eq!(overlaps.len(), groups_count);
            assert_eq!(
                serde_json::to_vec(&overlaps).expect("overlaps serialize"),
                reference_overlap_bytes
            );

            let functional_start = Instant::now();
            let functional = assess_service_functions(
                &functional_scenarios,
                &functional_evidence,
            )
            .expect("synthetic functional-adequacy fixtures must evaluate");
            functional_samples.push(functional_start.elapsed());
            assert_eq!(functional.len(), groups_count);
            assert!(functional
                .iter()
                .all(|assessment| assessment.state == ServiceFunctionState::Degraded));
            assert_eq!(
                serde_json::to_vec(&functional).expect("functional assessments serialize"),
                reference_functional_bytes
            );
        }
        samples.sort_unstable();
        common_cause_samples.sort_unstable();
        functional_samples.sort_unstable();

        let summary = SampleSummary {
            groups: groups_count,
            relations: graph.relations.len(),
            evidence_records: evidence_records.len(),
            common_cause_evidence_records: common_cause_evidence.len(),
            warmup_iterations,
            measured_iterations,
            p50_micros: percentile(&samples, 50).as_micros(),
            p95_micros: percentile(&samples, 95).as_micros(),
            max_micros: samples.last().copied().unwrap_or_default().as_micros(),
            recovered_groups_per_run: recovered,
            result_bytes: reference_bytes.len(),
            common_cause_overlap_pairs: reference_overlaps.len(),
            common_cause_p50_micros: percentile(&common_cause_samples, 50).as_micros(),
            common_cause_p95_micros: percentile(&common_cause_samples, 95).as_micros(),
            common_cause_max_micros: common_cause_samples
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            functional_assessments_per_run: reference_functional.len(),
            functional_evidence_records: functional_evidence.len(),
            functional_result_bytes: reference_functional_bytes.len(),
            functional_p50_micros: percentile(&functional_samples, 50).as_micros(),
            functional_p95_micros: percentile(&functional_samples, 95).as_micros(),
            functional_max_micros: functional_samples
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
        };
        println!(
            "{}",
            serde_json::to_string(&summary).expect("benchmark summary serializes")
        );
    }
}
