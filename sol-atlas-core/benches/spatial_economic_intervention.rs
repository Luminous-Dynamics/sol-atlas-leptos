// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Reproducible, dependency-free microbenchmark for spatial-economic intervention
//! comparison. Run with:
//! SEI_BENCH_SAMPLES=100 cargo bench --manifest-path sol-atlas-core/Cargo.toml --bench spatial_economic_intervention
//!
//! This harness reports observations; it does not assert that targets have passed.

use serde::{Deserialize, Serialize};
use sol_atlas_core::dependency_logic::{
    AvailabilityScenario, DependencyGroup, DependencyGroupState, DependencyLogic,
    EntityOperationalState, ScenarioBasis, ScenarioEntityCondition,
};
use sol_atlas_core::intervention::{compare_intervention, InterventionPlan};
use sol_atlas_core::spatial_economic::{
    EvidenceKind, EvidenceRecord, EvidenceState, Provenance, SpatialContext, TimeContext,
    EVIDENCE_SCHEMA_VERSION,
};
use sol_atlas_core::spatial_graph::{DependencyGraph, RelationKind, SpatialRelation};
use std::collections::BTreeMap;
use std::env;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct SampleSummary {
    groups: usize,
    relations: usize,
    evidence_records: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
    p50_micros: u128,
    p95_micros: u128,
    max_micros: u128,
    recovered_groups_per_run: usize,
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
        }

        let mut samples = Vec::with_capacity(measured_iterations);
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
        }
        samples.sort_unstable();

        let summary = SampleSummary {
            groups: groups_count,
            relations: graph.relations.len(),
            evidence_records: evidence_records.len(),
            warmup_iterations,
            measured_iterations,
            p50_micros: percentile(&samples, 50).as_micros(),
            p95_micros: percentile(&samples, 95).as_micros(),
            max_micros: samples.last().copied().unwrap_or_default().as_micros(),
            recovered_groups_per_run: recovered,
        };
        println!(
            "{}",
            serde_json::to_string(&summary).expect("benchmark summary serializes")
        );
    }
}
