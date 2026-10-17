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
    CommonCauseBasis, CommonCauseCondition, CommonCauseDomain, CommonCauseScenario,
    CommonCauseState, FailureMechanism, assess_common_cause_domains,
    correlate_common_cause_with_dependency_groups,
};
use sol_atlas_core::dependency_logic::{
    AvailabilityScenario, DependencyGroup, DependencyGroupState, DependencyLogic,
    EntityOperationalState, ScenarioBasis, ScenarioEntityCondition,
};
use sol_atlas_core::intervention::{InterventionPlan, compare_intervention};
use sol_atlas_core::service_equity::{
    SERVICE_EQUITY_SCHEMA_VERSION, ServiceEquityCell, ServiceEquityScenario, ServiceFloorStatus,
    assess_service_equity,
};
use sol_atlas_core::service_function::{
    FunctionMeasurement, SERVICE_FUNCTION_SCHEMA_VERSION, ServiceFunctionScenario,
    ServiceFunctionState, assess_service_functions,
};
use sol_atlas_core::service_network::{
    NetworkAssessmentState, NetworkLink, NetworkLinkAvailability, NetworkTopologyCompleteness,
    SERVICE_NETWORK_SCHEMA_VERSION, ServiceCapacityNetworkScenario,
    assess_service_capacity_networks,
};
use sol_atlas_core::service_network_intervention::{
    NetworkInterventionEffect, NetworkLinkInterventionOverride,
    SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION, ServiceNetworkInterventionPlan,
    ServiceNetworkInterventionRequest, compare_service_network_interventions,
};
use sol_atlas_core::service_network_portfolio::{
    ServiceNetworkPortfolioRequest, compare_service_network_portfolio,
};
use sol_atlas_core::service_network_stress::{
    DeclaredFailureSetKind, DeclaredNetworkFailureSet, MAX_EXHAUSTIVE_FAILURE_CASES,
    SERVICE_NETWORK_STRESS_SCHEMA_VERSION, ServiceNetworkFailureCaseOrigin,
    ServiceNetworkStressRequest, assess_service_network_stress,
};
use sol_atlas_core::service_recovery::{
    SERVICE_RECOVERY_TRAJECTORY_SCHEMA_VERSION, ServiceRecoveryTrajectory,
    assess_service_recovery_trajectory,
};
use sol_atlas_core::spatial_economic::Quantity;
use sol_atlas_core::spatial_economic::{
    EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceRecord, EvidenceState, Provenance,
    SpatialContext, TimeContext,
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
    equity_cells_per_run: usize,
    equity_communities_per_run: usize,
    equity_total_beneficiary_weight: u64,
    equity_meets_floor_weight: u64,
    equity_below_floor_weight: u64,
    equity_indeterminate_weight: u64,
    equity_floor_compliance_lower_bound: f64,
    equity_floor_compliance_upper_bound: f64,
    equity_compliance_gap_lower_bound: f64,
    equity_compliance_gap_upper_bound: f64,
    equity_result_bytes: usize,
    equity_p50_micros: u128,
    equity_p95_micros: u128,
    equity_max_micros: u128,
    network_scenarios_per_run: usize,
    network_links_per_run: usize,
    network_evidence_records: usize,
    network_result_bytes: usize,
    network_p50_micros: u128,
    network_p95_micros: u128,
    network_max_micros: u128,
    network_interventions_per_run: usize,
    network_intervention_bounds_improved: usize,
    network_intervention_bounds_degraded: usize,
    network_intervention_possible_improvement_only: usize,
    network_intervention_possible_degradation_only: usize,
    network_intervention_conservative_bound_regressed: usize,
    network_intervention_mixed_bound_movement: usize,
    network_intervention_bounds_unchanged: usize,
    network_intervention_topology_incomplete: usize,
    network_intervention_result_bytes: usize,
    network_intervention_p50_micros: u128,
    network_intervention_p95_micros: u128,
    network_intervention_max_micros: u128,
    recovery_samples_per_run: usize,
    recovery_evidence_records: usize,
    recovery_result_bytes: usize,
    recovery_first_adequate_elapsed_seconds: Option<u64>,
    recovery_normalized_deficit_seconds: f64,
    recovery_p50_micros: u128,
    recovery_p95_micros: u128,
    recovery_max_micros: u128,
    stress_active_links: usize,
    stress_failure_cases_per_run: usize,
    stress_evidence_records: usize,
    stress_result_bytes: usize,
    stress_p50_micros: u128,
    stress_p95_micros: u128,
    stress_max_micros: u128,
    portfolio_plans_per_run: usize,
    portfolio_scenarios_per_run: usize,
    portfolio_pareto_front_one_count: usize,
    portfolio_dominated_plan_count: usize,
    portfolio_result_bytes: usize,
    portfolio_p50_micros: u128,
    portfolio_p95_micros: u128,
    portfolio_max_micros: u128,
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

fn quantity_evidence_at(
    id: String,
    entity_id: &str,
    value: f64,
    unit: &str,
    uncertainty_abs: f64,
    timestamp: &str,
) -> EvidenceRecord {
    let mut record = quantity_evidence(id, entity_id, value, unit, uncertainty_abs);
    record.time.occurred_at = timestamp.into();
    record.time.recorded_at = timestamp.into();
    record
}

fn recovery_timestamp(index: usize) -> String {
    // Bench sizes are capped at 10,000 samples (< 3 hours), so a fixed UTC date
    // and hour/minute/second formatter is sufficient for this synthetic fixture.
    let hour = index / 3_600;
    let minute = (index % 3_600) / 60;
    let second = index % 60;
    format!("2026-10-16T{hour:02}:{minute:02}:{second:02}Z")
}

fn recovery_fixture(count: usize) -> (ServiceRecoveryTrajectory, Vec<EvidenceRecord>) {
    let start = recovery_timestamp(0);
    let mut snapshots = Vec::with_capacity(count);
    let mut records = Vec::with_capacity(count.saturating_mul(2));
    for index in 0..count {
        let timestamp = recovery_timestamp(index);
        let snapshot_id = format!("recovery-sample-{index:06}");
        let capacity_value = if index + 2 < count {
            4.0
        } else if index + 1 < count {
            6.0
        } else {
            8.0
        };
        let capacity_evidence_id = format!("recovery-capacity-evidence-{index:06}");
        let demand_evidence_id = format!("recovery-demand-evidence-{index:06}");
        records.push(quantity_evidence_at(
            capacity_evidence_id.clone(),
            "meter:recovery-output",
            capacity_value,
            "m3/h",
            0.0,
            &timestamp,
        ));
        records.push(quantity_evidence_at(
            demand_evidence_id.clone(),
            "service:recovery-water",
            8.0,
            "m3/h",
            0.0,
            &timestamp,
        ));
        snapshots.push(ServiceFunctionScenario {
            schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
            id: snapshot_id,
            as_of: timestamp.clone(),
            service_entity_id: "service:recovery-water".into(),
            available_capacity: FunctionMeasurement {
                entity_id: "meter:recovery-output".into(),
                quantity: Quantity {
                    value: capacity_value,
                    unit: "m3/h".into(),
                    uncertainty_abs: Some(0.0),
                },
                basis: ScenarioBasis::Observed,
                rationale: "Synthetic recovery capacity sample".into(),
                supporting_evidence_ids: vec![capacity_evidence_id],
            },
            required_capacity: FunctionMeasurement {
                entity_id: "service:recovery-water".into(),
                quantity: Quantity {
                    value: 8.0,
                    unit: "m3/h".into(),
                    uncertainty_abs: Some(0.0),
                },
                basis: ScenarioBasis::Observed,
                rationale: "Synthetic service demand sample".into(),
                supporting_evidence_ids: vec![demand_evidence_id],
            },
        });
    }
    (
        ServiceRecoveryTrajectory {
            schema_version: SERVICE_RECOVERY_TRAJECTORY_SCHEMA_VERSION.into(),
            id: format!("recovery-trajectory-{count:06}"),
            service_entity_id: "service:recovery-water".into(),
            disruption_started_at: start,
            max_sample_gap_seconds: 1,
            snapshots,
        },
        records,
    )
}

fn functional_fixture(count: usize) -> (Vec<ServiceFunctionScenario>, Vec<EvidenceRecord>) {
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

fn network_measurement(
    entity_id: &str,
    evidence_id: &str,
    value: f64,
    unit: &str,
) -> FunctionMeasurement {
    FunctionMeasurement {
        entity_id: entity_id.into(),
        quantity: Quantity {
            value,
            unit: unit.into(),
            uncertainty_abs: Some(0.0),
        },
        basis: ScenarioBasis::Observed,
        rationale: "Synthetic observed service-network measurement".into(),
        supporting_evidence_ids: vec![evidence_id.into()],
    }
}

fn network_fixture(count: usize) -> (Vec<ServiceCapacityNetworkScenario>, Vec<EvidenceRecord>) {
    let mut scenarios = Vec::with_capacity(count);
    let mut records = Vec::with_capacity(count.saturating_mul(4));
    for index in 0..count {
        let suffix = format!("{index:06}");
        let source_id = format!("network-source-{suffix}");
        let hub_id = format!("network-junction-{suffix}");
        let service_id = format!("network-service-{suffix}");
        let definitions = [
            ("supply", source_id.as_str(), hub_id.as_str(), 5.0),
            ("route-a", hub_id.as_str(), service_id.as_str(), 4.0),
            ("route-b", hub_id.as_str(), service_id.as_str(), 4.0),
        ];
        let mut links = Vec::with_capacity(3);
        for (link_name, from, to, capacity) in definitions {
            let link_id = format!("network-link-{suffix}-{link_name}");
            let evidence_id = format!("network-evidence-{suffix}-{link_name}");
            records.push(quantity_evidence(
                evidence_id.clone(),
                &link_id,
                capacity,
                "m3/h",
                0.0,
            ));
            links.push(NetworkLink {
                id: link_id.clone(),
                from_entity_id: from.into(),
                to_entity_id: to.into(),
                capacity: network_measurement(&link_id, &evidence_id, capacity, "m3/h"),
                availability: NetworkLinkAvailability::Active,
                availability_basis: ScenarioBasis::Assumed,
                availability_rationale: "Synthetic active network link".into(),
                availability_supporting_evidence_ids: Vec::new(),
            });
        }
        let demand_evidence_id = format!("network-demand-evidence-{suffix}");
        records.push(quantity_evidence(
            demand_evidence_id.clone(),
            &service_id,
            8.0,
            "m3/h",
            0.0,
        ));
        scenarios.push(ServiceCapacityNetworkScenario {
            schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
            id: format!("network-scenario-{suffix}"),
            as_of: "2026-10-15T12:00:00Z".into(),
            service_entity_id: service_id.clone(),
            topology_completeness: NetworkTopologyCompleteness::Complete,
            source_entity_ids: vec![source_id],
            required_capacity: network_measurement(&service_id, &demand_evidence_id, 8.0, "m3/h"),
            links,
        });
    }
    (scenarios, records)
}

fn stress_fixture(active_links: usize) -> (ServiceNetworkStressRequest, Vec<EvidenceRecord>) {
    let source_id = "stress-source".to_owned();
    let service_id = "stress-service".to_owned();
    let mut links = Vec::with_capacity(active_links);
    let mut records = Vec::with_capacity(active_links + 1);
    for index in 0..active_links {
        let link_id = format!("stress-link-{index:04}");
        let evidence_id = format!("stress-capacity-evidence-{index:04}");
        records.push(quantity_evidence(
            evidence_id.clone(),
            &link_id,
            1.0,
            "m3/h",
            0.0,
        ));
        links.push(NetworkLink {
            id: link_id.clone(),
            from_entity_id: source_id.clone(),
            to_entity_id: service_id.clone(),
            capacity: network_measurement(&link_id, &evidence_id, 1.0, "m3/h"),
            availability: NetworkLinkAvailability::Active,
            availability_basis: ScenarioBasis::Assumed,
            availability_rationale: "Synthetic active route in exhaustive stress fixture".into(),
            availability_supporting_evidence_ids: Vec::new(),
        });
    }
    let demand_evidence = "stress-demand-evidence".to_owned();
    records.push(quantity_evidence(
        demand_evidence.clone(),
        &service_id,
        active_links as f64,
        "m3/h",
        0.0,
    ));
    let scenario = ServiceCapacityNetworkScenario {
        schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
        id: format!("stress-network-{active_links}"),
        as_of: "2026-10-15T12:00:00Z".into(),
        service_entity_id: service_id.clone(),
        topology_completeness: NetworkTopologyCompleteness::Complete,
        source_entity_ids: vec![source_id],
        required_capacity: network_measurement(
            &service_id,
            &demand_evidence,
            active_links as f64,
            "m3/h",
        ),
        links,
    };
    (
        ServiceNetworkStressRequest {
            schema_version: SERVICE_NETWORK_STRESS_SCHEMA_VERSION.into(),
            id: format!("stress-sweep-{active_links}"),
            baseline_scenario: scenario,
            max_simultaneous_failures: 2,
            max_failure_cases: MAX_EXHAUSTIVE_FAILURE_CASES,
            declared_failure_sets: vec![DeclaredNetworkFailureSet {
                id: "shared-trunk:declared-common-cause".into(),
                kind: DeclaredFailureSetKind::CommonCause,
                rationale:
                    "Synthetic stress event disables two nominally parallel routes together.".into(),
                failed_link_ids: vec!["stress-link-0000".into(), "stress-link-0001".into()],
            }],
        },
        records,
    )
}

fn network_intervention_requests(
    scenarios: &[ServiceCapacityNetworkScenario],
) -> Vec<ServiceNetworkInterventionRequest> {
    scenarios
        .iter()
        .map(|scenario| {
            let suffix = scenario
                .id
                .strip_prefix("network-scenario-")
                .unwrap_or(&scenario.id);
            let supply_link_id = format!("network-link-{suffix}-supply");
            let plan = ServiceNetworkInterventionPlan {
                schema_version: SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION.into(),
                id: format!("upgrade-supply-{suffix}"),
                rationale: "Synthetic counterfactual: raise supply trunk capacity".into(),
                link_overrides: [(
                    supply_link_id,
                    NetworkLinkInterventionOverride {
                        availability: None,
                        capacity_value: Some(10.0),
                        capacity_uncertainty_abs: None,
                        rationale: "Synthetic assumed supply-capacity upgrade".into(),
                    },
                )]
                .into_iter()
                .collect(),
            };
            ServiceNetworkInterventionRequest {
                scenario: scenario.clone(),
                plan,
            }
        })
        .collect()
}

fn portfolio_fixture(plan_count: usize, scenario_count: usize) -> ServiceNetworkPortfolioRequest {
    let service_id = "service:irrigation";
    let source_id = "source:reservoir";
    let mut scenarios = Vec::with_capacity(scenario_count);
    for index in 0..scenario_count {
        let suffix = format!("{index:03}");
        let supply_capacity = 5.0 + (index % 3) as f64;
        let backup_availability = match index % 3 {
            0 => NetworkLinkAvailability::Active,
            1 => NetworkLinkAvailability::Inactive,
            _ => NetworkLinkAvailability::Unknown,
        };
        let link =
            |id: &str, from: &str, to: &str, value: f64, availability: NetworkLinkAvailability| {
                NetworkLink {
                    id: id.into(),
                    from_entity_id: from.into(),
                    to_entity_id: to.into(),
                    capacity: FunctionMeasurement {
                        entity_id: id.into(),
                        quantity: Quantity {
                            value,
                            unit: "m3/h".into(),
                            uncertainty_abs: Some(0.0),
                        },
                        basis: ScenarioBasis::Assumed,
                        rationale: "Synthetic portfolio benchmark capacity".into(),
                        supporting_evidence_ids: Vec::new(),
                    },
                    availability,
                    availability_basis: ScenarioBasis::Assumed,
                    availability_rationale: "Synthetic portfolio benchmark state".into(),
                    availability_supporting_evidence_ids: Vec::new(),
                }
            };
        scenarios.push(ServiceCapacityNetworkScenario {
            schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
            id: format!("portfolio-scenario-{suffix}"),
            as_of: "2026-10-15T12:00:00Z".into(),
            service_entity_id: service_id.into(),
            topology_completeness: NetworkTopologyCompleteness::Complete,
            source_entity_ids: vec![source_id.into()],
            required_capacity: FunctionMeasurement {
                entity_id: service_id.into(),
                quantity: Quantity {
                    value: 8.0,
                    unit: "m3/h".into(),
                    uncertainty_abs: Some(0.0),
                },
                basis: ScenarioBasis::Assumed,
                rationale: "Synthetic portfolio benchmark demand".into(),
                supporting_evidence_ids: Vec::new(),
            },
            links: vec![
                link(
                    "supply",
                    source_id,
                    "junction",
                    supply_capacity,
                    NetworkLinkAvailability::Active,
                ),
                link(
                    "route-a",
                    "junction",
                    service_id,
                    4.0,
                    NetworkLinkAvailability::Active,
                ),
                link("route-b", "junction", service_id, 4.0, backup_availability),
            ],
        });
    }

    let mut plans = Vec::with_capacity(plan_count);
    for index in 0..plan_count {
        let mut link_overrides = BTreeMap::new();
        match index % 3 {
            0 => {
                link_overrides.insert(
                    "route-b".into(),
                    NetworkLinkInterventionOverride {
                        availability: Some(NetworkLinkAvailability::Active),
                        capacity_value: None,
                        capacity_uncertainty_abs: None,
                        rationale: "Synthetic plan activates backup route".into(),
                    },
                );
            }
            1 => {
                link_overrides.insert(
                    "supply".into(),
                    NetworkLinkInterventionOverride {
                        availability: None,
                        capacity_value: Some(8.0 + (index % 4) as f64),
                        capacity_uncertainty_abs: Some(0.0),
                        rationale: "Synthetic plan upgrades shared supply".into(),
                    },
                );
            }
            _ => {
                link_overrides.insert(
                    "route-b".into(),
                    NetworkLinkInterventionOverride {
                        availability: Some(NetworkLinkAvailability::Active),
                        capacity_value: None,
                        capacity_uncertainty_abs: None,
                        rationale: "Synthetic combined plan activates backup route".into(),
                    },
                );
                link_overrides.insert(
                    "supply".into(),
                    NetworkLinkInterventionOverride {
                        availability: None,
                        capacity_value: Some(8.0 + (index % 4) as f64),
                        capacity_uncertainty_abs: Some(0.0),
                        rationale: "Synthetic combined plan upgrades supply".into(),
                    },
                );
            }
        }
        plans.push(ServiceNetworkInterventionPlan {
            schema_version: SERVICE_NETWORK_INTERVENTION_SCHEMA_VERSION.into(),
            id: format!("portfolio-plan-{index:05}"),
            rationale: "Synthetic candidate only; no real work is implied".into(),
            link_overrides,
        });
    }

    ServiceNetworkPortfolioRequest {
        comparison_id: format!("portfolio-benchmark-{plan_count}x{scenario_count}"),
        scenarios,
        plans,
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

fn fixture(
    groups_count: usize,
) -> (
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
        let reference =
            compare_intervention(&graph, &groups, &baseline, &intervention, &evidence_records)
                .expect("synthetic fixture must evaluate");
        assert_eq!(reference.summary.recovered, groups_count);
        let reference_bytes =
            serde_json::to_vec(&reference).expect("deterministic comparison serializes");
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
            overlap.domain_evidence_admissible
                && overlap.independent_redundancy_warning
                && overlap.shared_prerequisite_entity_ids == vec!["feeder-backup", "feeder-primary"]
                && overlap.currently_exposed_relation_ids.len() == 2
        }));
        let reference_overlap_bytes =
            serde_json::to_vec(&reference_overlaps).expect("overlaps serialize");

        let (functional_scenarios, functional_evidence) = functional_fixture(groups_count);
        let reference_functional =
            assess_service_functions(&functional_scenarios, &functional_evidence)
                .expect("synthetic functional-adequacy fixtures must evaluate");
        assert_eq!(reference_functional.len(), groups_count);
        assert!(reference_functional.iter().all(|assessment| {
            assessment.state == ServiceFunctionState::Degraded
                && assessment.coverage_ratio == Some(0.7)
                && assessment.coverage_lower_bound == Some(65.0 / 110.0)
                && assessment.coverage_upper_bound == Some(75.0 / 90.0)
                && !assessment.available_capacity_evidence_ids.is_empty()
                && !assessment.required_capacity_evidence_ids.is_empty()
        }));
        let reference_functional_bytes =
            serde_json::to_vec(&reference_functional).expect("functional assessments serialize");

        let equity_scenario = ServiceEquityScenario {
            schema_version: SERVICE_EQUITY_SCHEMA_VERSION.into(),
            id: format!("equity-benchmark-{groups_count}"),
            as_of: "2026-10-15T12:00:00Z".into(),
            cells: functional_scenarios
                .iter()
                .enumerate()
                .map(|(index, service)| ServiceEquityCell {
                    id: format!("equity-cell-{index:06}"),
                    service_scenario_id: service.id.clone(),
                    community_id: format!("community-{:02}", index % 17),
                    beneficiary_weight: (index % 5 + 1) as u64,
                    essential_coverage_floor: match index % 3 {
                        0 => 0.5,
                        1 => 0.8,
                        _ => 0.9,
                    },
                })
                .collect(),
        };
        let reference_equity = assess_service_equity(
            &equity_scenario,
            &functional_scenarios,
            &functional_evidence,
        )
        .expect("synthetic distributional service-floor fixture must evaluate");
        assert_eq!(reference_equity.cells.len(), groups_count);
        assert_eq!(reference_equity.community_count, groups_count.min(17));
        let expected_equity_totals = equity_scenario.cells.iter().enumerate().fold(
            (0_u64, 0_u64, 0_u64, 0_u64),
            |(total, met, below, unknown), (index, cell)| {
                let weight = cell.beneficiary_weight;
                match index % 3 {
                    0 => (total + weight, met + weight, below, unknown),
                    1 => (total + weight, met, below, unknown + weight),
                    _ => (total + weight, met, below + weight, unknown),
                }
            },
        );
        assert_eq!(
            reference_equity.total_beneficiary_weight,
            expected_equity_totals.0
        );
        assert_eq!(
            reference_equity.meets_floor_weight,
            expected_equity_totals.1
        );
        assert_eq!(
            reference_equity.below_floor_weight,
            expected_equity_totals.2
        );
        assert_eq!(
            reference_equity.indeterminate_weight,
            expected_equity_totals.3
        );
        assert!(
            reference_equity
                .cells
                .iter()
                .all(|cell| match cell.floor_status {
                    ServiceFloorStatus::MeetsFloor =>
                        cell.coverage_lower_bound == Some(65.0 / 110.0),
                    ServiceFloorStatus::Indeterminate =>
                        cell.coverage_lower_bound == Some(65.0 / 110.0)
                            && cell.coverage_upper_bound == Some(75.0 / 90.0),
                    ServiceFloorStatus::BelowFloor =>
                        cell.coverage_upper_bound == Some(75.0 / 90.0),
                })
        );
        let reference_equity_bytes =
            serde_json::to_vec(&reference_equity).expect("equity assessment serializes");

        let (network_scenarios, network_evidence) = network_fixture(groups_count);
        let reference_network =
            assess_service_capacity_networks(&network_scenarios, &network_evidence)
                .expect("synthetic capacity network fixtures must evaluate");
        assert_eq!(reference_network.len(), groups_count);
        assert!(reference_network.iter().all(|assessment| {
            assessment.state == NetworkAssessmentState::Degraded
                && assessment.conservative_available_capacity == 5.0
                && assessment.optimistic_available_capacity == 5.0
                && assessment.conservative_min_cut_link_ids.len() == 1
                && assessment.conservative_min_cut_link_ids[0].ends_with("-supply")
        }));
        let reference_network_bytes =
            serde_json::to_vec(&reference_network).expect("network assessments serialize");

        let network_intervention_requests = network_intervention_requests(&network_scenarios);
        let reference_network_interventions = compare_service_network_interventions(
            &network_intervention_requests,
            &network_evidence,
        )
        .expect("synthetic network interventions must evaluate");
        assert_eq!(reference_network_interventions.len(), groups_count);
        assert!(reference_network_interventions.iter().all(|comparison| {
            comparison.effect == NetworkInterventionEffect::BoundsImproved
                && comparison.baseline.conservative_available_capacity == 5.0
                && comparison.with_intervention.conservative_available_capacity == 8.0
                && comparison.conservative_capacity_delta == 3.0
                && comparison.optimistic_capacity_delta == 3.0
                && !comparison
                    .with_intervention
                    .capacity_evidence_ids
                    .iter()
                    .any(|id| id.ends_with("-supply"))
        }));
        let reference_network_intervention_bytes =
            serde_json::to_vec(&reference_network_interventions)
                .expect("network intervention comparisons serialize");
        let network_intervention_bounds_improved = reference_network_interventions
            .iter()
            .filter(|item| item.effect == NetworkInterventionEffect::BoundsImproved)
            .count();
        let network_intervention_bounds_degraded = reference_network_interventions
            .iter()
            .filter(|item| item.effect == NetworkInterventionEffect::BoundsDegraded)
            .count();
        let network_intervention_possible_improvement_only = reference_network_interventions
            .iter()
            .filter(|item| item.effect == NetworkInterventionEffect::PossibleImprovementOnly)
            .count();
        let network_intervention_possible_degradation_only = reference_network_interventions
            .iter()
            .filter(|item| item.effect == NetworkInterventionEffect::PossibleDegradationOnly)
            .count();
        let network_intervention_conservative_bound_regressed = reference_network_interventions
            .iter()
            .filter(|item| item.effect == NetworkInterventionEffect::ConservativeBoundRegressed)
            .count();
        let network_intervention_mixed_bound_movement = reference_network_interventions
            .iter()
            .filter(|item| item.effect == NetworkInterventionEffect::MixedBoundMovement)
            .count();
        let network_intervention_bounds_unchanged = reference_network_interventions
            .iter()
            .filter(|item| item.effect == NetworkInterventionEffect::BoundsUnchanged)
            .count();
        let network_intervention_topology_incomplete = reference_network_interventions
            .iter()
            .filter(|item| item.effect == NetworkInterventionEffect::TopologyIncomplete)
            .count();
        assert_eq!(network_intervention_bounds_improved, groups_count);
        assert_eq!(network_intervention_bounds_degraded, 0);
        assert_eq!(network_intervention_possible_improvement_only, 0);
        assert_eq!(network_intervention_possible_degradation_only, 0);
        assert_eq!(network_intervention_conservative_bound_regressed, 0);
        assert_eq!(network_intervention_mixed_bound_movement, 0);
        assert_eq!(network_intervention_bounds_unchanged, 0);
        assert_eq!(network_intervention_topology_incomplete, 0);

        let stress_active_links = match groups_count {
            count if count <= 100 => 10,
            count if count <= 1_000 => 20,
            _ => 40,
        };
        let (stress_request, stress_evidence) = stress_fixture(stress_active_links);
        let reference_stress = assess_service_network_stress(&stress_request, &stress_evidence)
            .expect("synthetic exhaustive network stress fixture must evaluate");
        let expected_stress_cases =
            stress_active_links + (stress_active_links * (stress_active_links - 1) / 2) + 1;
        let expected_worst_sets = stress_active_links * (stress_active_links - 1) / 2;
        assert_eq!(
            reference_stress.candidate_failure_case_count,
            expected_stress_cases
        );
        assert_eq!(
            reference_stress.evaluated_failure_case_count,
            expected_stress_cases
        );
        assert_eq!(reference_stress.declared_failure_set_count, 1);
        assert!(reference_stress.exhaustive);
        let declared_common_cause = reference_stress
            .cases
            .iter()
            .find(|case| {
                case.origin == ServiceNetworkFailureCaseOrigin::DeclaredCommonCause
                    && case.failure_set_id.as_deref() == Some("shared-trunk:declared-common-cause")
            })
            .expect("declared common-cause bundle is present");
        assert_eq!(
            declared_common_cause.failed_link_ids,
            vec!["stress-link-0000", "stress-link-0001"]
        );
        assert_eq!(
            declared_common_cause
                .assessment
                .conservative_available_capacity,
            (stress_active_links - 2) as f64
        );
        assert_eq!(
            reference_stress.baseline.conservative_available_capacity,
            stress_active_links as f64
        );
        assert_eq!(
            reference_stress.worst_case_conservative_capacity,
            (stress_active_links - 2) as f64
        );
        assert_eq!(reference_stress.worst_case_conservative_capacity_loss, 2.0);
        assert_eq!(
            reference_stress.worst_case_conservative_failure_sets.len(),
            expected_worst_sets
        );
        let reference_stress_bytes =
            serde_json::to_vec(&reference_stress).expect("stress report serializes");

        let (recovery_trajectory, recovery_evidence) = recovery_fixture(groups_count);
        let reference_recovery =
            assess_service_recovery_trajectory(&recovery_trajectory, &recovery_evidence)
                .expect("synthetic recovery trajectory must evaluate");
        assert_eq!(reference_recovery.sample_count, groups_count);
        assert_eq!(
            reference_recovery.first_adequate_sample_elapsed_seconds,
            Some((groups_count - 1) as u64)
        );
        assert!(reference_recovery.observed_adequacy_maintained_through_end);
        let expected_deficit_seconds = groups_count as f64 * 0.5 - 1.0;
        assert_eq!(
            reference_recovery.normalized_deficit_seconds_lower_bound,
            expected_deficit_seconds
        );
        assert_eq!(
            reference_recovery.normalized_deficit_seconds_upper_bound,
            expected_deficit_seconds
        );
        let reference_recovery_bytes =
            serde_json::to_vec(&reference_recovery).expect("recovery trajectory serializes");

        // Bound plan x scenario cost independently from graph-size and N-k sweeps.
        let portfolio_plan_count = match groups_count {
            100 => 10,
            1_000 => 50,
            _ => 100,
        };
        let portfolio_scenario_count = 10;
        let portfolio_request = portfolio_fixture(portfolio_plan_count, portfolio_scenario_count);
        let reference_portfolio = compare_service_network_portfolio(&portfolio_request, &[])
            .expect("synthetic Pareto portfolio must evaluate");
        assert_eq!(reference_portfolio.plans.len(), portfolio_plan_count);
        assert!(
            reference_portfolio
                .plans
                .iter()
                .all(|plan| plan.pareto_front >= 1)
        );
        assert!(
            reference_portfolio
                .plans
                .iter()
                .any(|plan| plan.pareto_front == 1)
        );
        assert!(
            reference_portfolio
                .plans
                .iter()
                .any(|plan| plan.pareto_front > 1)
        );
        assert!(reference_portfolio.plans.iter().all(|plan| {
            plan.pareto_dominated_by_plan_ids
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        }));
        let reference_portfolio_bytes =
            serde_json::to_vec(&reference_portfolio).expect("portfolio comparison serializes");

        for _ in 0..warmup_iterations {
            let result =
                compare_intervention(&graph, &groups, &baseline, &intervention, &evidence_records)
                    .expect("synthetic fixture must evaluate");
            assert_eq!(result.summary.recovered, groups_count);
            assert!(
                result
                    .group_deltas
                    .iter()
                    .all(|delta| delta.with_intervention.state == DependencyGroupState::Satisfied)
            );
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
            let overlaps =
                correlate_common_cause_with_dependency_groups(&common, &intervention_groups)
                    .expect("same-time synthetic assessments must correlate");
            assert_eq!(overlaps.len(), groups_count);
            assert_eq!(
                serde_json::to_vec(&overlaps).expect("overlaps serialize"),
                reference_overlap_bytes
            );

            let functional = assess_service_functions(&functional_scenarios, &functional_evidence)
                .expect("synthetic functional-adequacy fixtures must evaluate");
            assert_eq!(functional.len(), groups_count);
            assert!(functional.iter().all(|assessment| {
                assessment.state == ServiceFunctionState::Degraded
                    && assessment.coverage_ratio == Some(0.7)
                    && assessment.coverage_lower_bound == Some(65.0 / 110.0)
                    && assessment.coverage_upper_bound == Some(75.0 / 90.0)
            }));
            assert_eq!(
                serde_json::to_vec(&functional).expect("functional assessments serialize"),
                reference_functional_bytes
            );

            let equity = assess_service_equity(
                &equity_scenario,
                &functional_scenarios,
                &functional_evidence,
            )
            .expect("synthetic distributional service-floor fixture must evaluate");
            assert_eq!(equity.cells.len(), groups_count);
            assert_eq!(equity.total_beneficiary_weight, expected_equity_totals.0);
            assert_eq!(equity.meets_floor_weight, expected_equity_totals.1);
            assert_eq!(equity.below_floor_weight, expected_equity_totals.2);
            assert_eq!(equity.indeterminate_weight, expected_equity_totals.3);
            assert_eq!(
                serde_json::to_vec(&equity).expect("equity assessment serializes"),
                reference_equity_bytes
            );

            let networks = assess_service_capacity_networks(&network_scenarios, &network_evidence)
                .expect("synthetic capacity network fixtures must evaluate");
            assert_eq!(networks.len(), groups_count);
            assert!(networks.iter().all(|assessment| {
                assessment.state == NetworkAssessmentState::Degraded
                    && assessment.conservative_available_capacity == 5.0
                    && assessment.optimistic_available_capacity == 5.0
            }));
            assert_eq!(
                serde_json::to_vec(&networks).expect("network assessments serialize"),
                reference_network_bytes
            );

            let comparisons = compare_service_network_interventions(
                &network_intervention_requests,
                &network_evidence,
            )
            .expect("synthetic network interventions must evaluate");
            assert_eq!(comparisons.len(), groups_count);
            assert!(comparisons.iter().all(|comparison| {
                comparison.effect == NetworkInterventionEffect::BoundsImproved
                    && comparison.conservative_capacity_delta == 3.0
                    && comparison.optimistic_capacity_delta == 3.0
                    && !comparison
                        .with_intervention
                        .capacity_evidence_ids
                        .iter()
                        .any(|id| id.ends_with("-supply"))
            }));
            assert_eq!(
                serde_json::to_vec(&comparisons)
                    .expect("network intervention comparisons serialize"),
                reference_network_intervention_bytes
            );

            let stress = assess_service_network_stress(&stress_request, &stress_evidence)
                .expect("synthetic exhaustive network stress fixture must evaluate");
            assert_eq!(stress.candidate_failure_case_count, expected_stress_cases);
            assert_eq!(stress.evaluated_failure_case_count, expected_stress_cases);
            assert_eq!(stress.declared_failure_set_count, 1);
            assert!(stress.exhaustive);
            assert!(stress.cases.iter().any(|case| {
                case.origin == ServiceNetworkFailureCaseOrigin::DeclaredCommonCause
                    && case.failed_link_ids == vec!["stress-link-0000", "stress-link-0001"]
                    && case.assessment.conservative_available_capacity
                        == (stress_active_links - 2) as f64
            }));
            assert_eq!(
                serde_json::to_vec(&stress).expect("stress report serializes"),
                reference_stress_bytes
            );

            let recovery =
                assess_service_recovery_trajectory(&recovery_trajectory, &recovery_evidence)
                    .expect("synthetic recovery trajectory must evaluate");
            assert_eq!(
                serde_json::to_vec(&recovery).expect("recovery trajectory serializes"),
                reference_recovery_bytes
            );

            let portfolio = compare_service_network_portfolio(&portfolio_request, &[])
                .expect("synthetic Pareto portfolio must evaluate");
            assert_eq!(portfolio.plans.len(), portfolio_plan_count);
            assert!(portfolio.plans.iter().all(|plan| plan.pareto_front >= 1));
            assert_eq!(
                serde_json::to_vec(&portfolio).expect("portfolio serializes"),
                reference_portfolio_bytes
            );
        }

        let mut samples = Vec::with_capacity(measured_iterations);
        let mut portfolio_samples = Vec::with_capacity(measured_iterations);
        let mut common_cause_samples = Vec::with_capacity(measured_iterations);
        let mut functional_samples = Vec::with_capacity(measured_iterations);
        let mut equity_samples = Vec::with_capacity(measured_iterations);
        let mut network_samples = Vec::with_capacity(measured_iterations);
        let mut network_intervention_samples = Vec::with_capacity(measured_iterations);
        let mut recovery_samples = Vec::with_capacity(measured_iterations);
        let mut stress_samples = Vec::with_capacity(measured_iterations);
        let mut recovered = 0;
        for _ in 0..measured_iterations {
            let start = Instant::now();
            let result =
                compare_intervention(&graph, &groups, &baseline, &intervention, &evidence_records)
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
            let overlaps =
                correlate_common_cause_with_dependency_groups(&common, &intervention_groups)
                    .expect("same-time synthetic assessments must correlate");
            common_cause_samples.push(common_start.elapsed());
            assert_eq!(overlaps.len(), groups_count);
            assert_eq!(
                serde_json::to_vec(&overlaps).expect("overlaps serialize"),
                reference_overlap_bytes
            );

            let functional_start = Instant::now();
            let functional = assess_service_functions(&functional_scenarios, &functional_evidence)
                .expect("synthetic functional-adequacy fixtures must evaluate");
            functional_samples.push(functional_start.elapsed());
            assert_eq!(functional.len(), groups_count);
            assert!(
                functional
                    .iter()
                    .all(|assessment| assessment.state == ServiceFunctionState::Degraded)
            );
            assert_eq!(
                serde_json::to_vec(&functional).expect("functional assessments serialize"),
                reference_functional_bytes
            );

            let equity_start = Instant::now();
            let equity = assess_service_equity(
                &equity_scenario,
                &functional_scenarios,
                &functional_evidence,
            )
            .expect("synthetic distributional service-floor fixture must evaluate");
            equity_samples.push(equity_start.elapsed());
            assert_eq!(equity.cells.len(), groups_count);
            assert_eq!(equity.total_beneficiary_weight, expected_equity_totals.0);
            assert_eq!(equity.meets_floor_weight, expected_equity_totals.1);
            assert_eq!(equity.below_floor_weight, expected_equity_totals.2);
            assert_eq!(equity.indeterminate_weight, expected_equity_totals.3);
            assert_eq!(
                serde_json::to_vec(&equity).expect("equity assessment serializes"),
                reference_equity_bytes
            );

            let network_start = Instant::now();
            let networks = assess_service_capacity_networks(&network_scenarios, &network_evidence)
                .expect("synthetic capacity network fixtures must evaluate");
            network_samples.push(network_start.elapsed());
            assert_eq!(networks.len(), groups_count);
            assert!(networks.iter().all(|assessment| {
                assessment.state == NetworkAssessmentState::Degraded
                    && assessment.conservative_available_capacity == 5.0
                    && assessment.optimistic_available_capacity == 5.0
            }));
            assert_eq!(
                serde_json::to_vec(&networks).expect("network assessments serialize"),
                reference_network_bytes
            );

            let intervention_start = Instant::now();
            let comparisons = compare_service_network_interventions(
                &network_intervention_requests,
                &network_evidence,
            )
            .expect("synthetic network interventions must evaluate");
            network_intervention_samples.push(intervention_start.elapsed());
            assert_eq!(comparisons.len(), groups_count);
            assert!(comparisons.iter().all(|comparison| {
                comparison.effect == NetworkInterventionEffect::BoundsImproved
                    && comparison.conservative_capacity_delta == 3.0
                    && comparison.optimistic_capacity_delta == 3.0
            }));
            assert_eq!(
                serde_json::to_vec(&comparisons)
                    .expect("network intervention comparisons serialize"),
                reference_network_intervention_bytes
            );

            let stress_start = Instant::now();
            let stress = assess_service_network_stress(&stress_request, &stress_evidence)
                .expect("synthetic exhaustive network stress fixture must evaluate");
            stress_samples.push(stress_start.elapsed());
            assert_eq!(stress.candidate_failure_case_count, expected_stress_cases);
            assert_eq!(stress.evaluated_failure_case_count, expected_stress_cases);
            assert_eq!(stress.worst_case_conservative_capacity_loss, 2.0);
            assert_eq!(
                serde_json::to_vec(&stress).expect("stress report serializes"),
                reference_stress_bytes
            );

            let recovery_start = Instant::now();
            let recovery =
                assess_service_recovery_trajectory(&recovery_trajectory, &recovery_evidence)
                    .expect("synthetic recovery trajectory must evaluate");
            recovery_samples.push(recovery_start.elapsed());
            assert_eq!(recovery.sample_count, groups_count);
            assert_eq!(
                recovery.normalized_deficit_seconds_lower_bound,
                expected_deficit_seconds
            );
            assert_eq!(
                serde_json::to_vec(&recovery).expect("recovery trajectory serializes"),
                reference_recovery_bytes
            );

            let portfolio_start = Instant::now();
            let portfolio = compare_service_network_portfolio(&portfolio_request, &[])
                .expect("synthetic Pareto portfolio must evaluate");
            portfolio_samples.push(portfolio_start.elapsed());
            assert_eq!(portfolio.plans.len(), portfolio_plan_count);
            assert!(portfolio.plans.iter().all(|plan| plan.pareto_front >= 1));
            assert_eq!(
                serde_json::to_vec(&portfolio).expect("portfolio serializes"),
                reference_portfolio_bytes
            );
        }
        samples.sort_unstable();
        common_cause_samples.sort_unstable();
        functional_samples.sort_unstable();
        equity_samples.sort_unstable();
        network_samples.sort_unstable();
        network_intervention_samples.sort_unstable();
        recovery_samples.sort_unstable();
        stress_samples.sort_unstable();
        portfolio_samples.sort_unstable();

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
            equity_cells_per_run: reference_equity.cell_count,
            equity_communities_per_run: reference_equity.community_count,
            equity_total_beneficiary_weight: reference_equity.total_beneficiary_weight,
            equity_meets_floor_weight: reference_equity.meets_floor_weight,
            equity_below_floor_weight: reference_equity.below_floor_weight,
            equity_indeterminate_weight: reference_equity.indeterminate_weight,
            equity_floor_compliance_lower_bound: reference_equity
                .overall_floor_compliance_lower_bound,
            equity_floor_compliance_upper_bound: reference_equity
                .overall_floor_compliance_upper_bound,
            equity_compliance_gap_lower_bound: reference_equity
                .community_compliance_gap_lower_bound,
            equity_compliance_gap_upper_bound: reference_equity
                .community_compliance_gap_upper_bound,
            equity_result_bytes: reference_equity_bytes.len(),
            equity_p50_micros: percentile(&equity_samples, 50).as_micros(),
            equity_p95_micros: percentile(&equity_samples, 95).as_micros(),
            equity_max_micros: equity_samples
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            network_scenarios_per_run: reference_network.len(),
            network_links_per_run: network_scenarios.iter().map(|item| item.links.len()).sum(),
            network_evidence_records: network_evidence.len(),
            network_result_bytes: reference_network_bytes.len(),
            network_p50_micros: percentile(&network_samples, 50).as_micros(),
            network_p95_micros: percentile(&network_samples, 95).as_micros(),
            network_max_micros: network_samples
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            network_interventions_per_run: reference_network_interventions.len(),
            network_intervention_bounds_improved,
            network_intervention_bounds_degraded,
            network_intervention_possible_improvement_only,
            network_intervention_possible_degradation_only,
            network_intervention_conservative_bound_regressed,
            network_intervention_mixed_bound_movement,
            network_intervention_bounds_unchanged,
            network_intervention_topology_incomplete,
            network_intervention_result_bytes: reference_network_intervention_bytes.len(),
            network_intervention_p50_micros: percentile(&network_intervention_samples, 50)
                .as_micros(),
            network_intervention_p95_micros: percentile(&network_intervention_samples, 95)
                .as_micros(),
            network_intervention_max_micros: network_intervention_samples
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            recovery_samples_per_run: reference_recovery.sample_count,
            recovery_evidence_records: recovery_evidence.len(),
            recovery_result_bytes: reference_recovery_bytes.len(),
            recovery_first_adequate_elapsed_seconds: reference_recovery
                .first_adequate_sample_elapsed_seconds,
            recovery_normalized_deficit_seconds: reference_recovery
                .normalized_deficit_seconds_lower_bound,
            recovery_p50_micros: percentile(&recovery_samples, 50).as_micros(),
            recovery_p95_micros: percentile(&recovery_samples, 95).as_micros(),
            recovery_max_micros: recovery_samples
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            stress_active_links,
            stress_failure_cases_per_run: reference_stress.evaluated_failure_case_count,
            stress_evidence_records: stress_evidence.len(),
            stress_result_bytes: reference_stress_bytes.len(),
            stress_p50_micros: percentile(&stress_samples, 50).as_micros(),
            stress_p95_micros: percentile(&stress_samples, 95).as_micros(),
            stress_max_micros: stress_samples
                .last()
                .copied()
                .unwrap_or_default()
                .as_micros(),
            portfolio_plans_per_run: reference_portfolio.plans.len(),
            portfolio_scenarios_per_run: reference_portfolio.scenario_count,
            portfolio_pareto_front_one_count: reference_portfolio
                .plans
                .iter()
                .filter(|plan| plan.pareto_front == 1)
                .count(),
            portfolio_dominated_plan_count: reference_portfolio
                .plans
                .iter()
                .filter(|plan| plan.pareto_front > 1)
                .count(),
            portfolio_result_bytes: reference_portfolio_bytes.len(),
            portfolio_p50_micros: percentile(&portfolio_samples, 50).as_micros(),
            portfolio_p95_micros: percentile(&portfolio_samples, 95).as_micros(),
            portfolio_max_micros: portfolio_samples
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
