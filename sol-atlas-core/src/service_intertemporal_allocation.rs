// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Bounded exact reference model for horizon-wide shared-resource allocation.
//!
//! The model works in declared integer quantity quanta and exhaustively enumerates
//! only small, preflight-bounded cases. Inventory balance, per-period throughput,
//! storage bounds, and terminal reserves are hard feasibility constraints.
//!
//! This is an exact oracle for small synthetic/modelled cases, not a scalable LP/QP
//! solver, hydraulic/electrical physics model, outage-probability estimator, or
//! operational authority. Missing required inputs produce IndeterminateInput; an
//! exhausted search budget never returns a provisional allocation as optimal.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

pub const INTERTEMPORAL_ALLOCATION_SCHEMA_VERSION: &str = "0.1.0";
pub const MAX_INTERTEMPORAL_STEPS: usize = 8;
pub const MAX_INTERTEMPORAL_SERVICES: usize = 8;
pub const MAX_INTERTEMPORAL_RESOURCES: usize = 8;
pub const MAX_INTERTEMPORAL_USAGES: usize = 64;
pub const MAX_INTERTEMPORAL_SEARCH_NODES: usize = 1_000_000;
const BASIS_POINTS: u128 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntertemporalLineageBasis {
    Synthetic,
    Assumed,
    Observed,
    Forecast,
    Scenario,
}

/// References remain opaque: this module preserves lineage but does not authenticate it.
/// Canonical evidence admission remains owned by the existing evidence layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalLineage {
    pub basis: IntertemporalLineageBasis,
    pub evidence_ids: Vec<String>,
    pub assumption_ids: Vec<String>,
}

/// Values are integer quanta **per interval**, not rates. A null value means unknown,
/// never zero. Unit strings describe the quantity represented by one integer quantum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalQuantitySeries {
    pub unit: String,
    pub values: Vec<Option<u64>>,
    pub lineage: IntertemporalLineage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalTimeStep {
    pub id: String,
    pub start_unix_seconds: i64,
    pub end_unix_seconds_exclusive: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntertemporalAllocationPolicy {
    /// Maximize the lexicographically sorted service-interval coverage fractions.
    /// Essential-floor shortfall and weighted delivered quanta break ties.
    EqualCoverageLeximin,
    /// Maximize weighted delivered quanta for lower numeric priority ranks first.
    /// Essential-floor shortfall and total delivery break ties.
    FixedPriority,
    /// Minimize weighted essential-floor shortfall first, then maximize weighted
    /// delivered quanta and leximin coverage.
    EssentialFloorFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntertemporalDesignRole {
    Baseline,
    Candidate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalServiceDemand {
    pub id: String,
    pub community_id: String,
    /// Explicit weighting for objective aggregation; this is not a count of unique people.
    pub beneficiary_weight: u64,
    /// Lower ranks are served first only under FixedPriority.
    pub priority_rank: u16,
    pub demand: IntertemporalQuantitySeries,
    /// Floor as integer basis points: 10_000 means 100% of demand in each interval.
    pub essential_floor_basis_points: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalInventorySpec {
    /// Null means opening stock is unknown; it must not be interpreted as empty.
    pub opening_stock_quanta: Option<u64>,
    pub maximum_stock_quanta: u64,
    pub minimum_reserve_quanta: u64,
    pub terminal_reserve_quanta: u64,
    pub opening_stock_lineage: IntertemporalLineage,
    pub storage_bounds_lineage: IntertemporalLineage,
    pub inflow: IntertemporalQuantitySeries,
    /// Declared physical/modelled losses per interval. No loss is inferred.
    pub losses: IntertemporalQuantitySeries,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalResource {
    pub id: String,
    /// Per-interval throughput limit. For inventories this is the maximum withdrawal
    /// flow in each interval; stock is independently bounded by InventorySpec.
    pub capacity: IntertemporalQuantitySeries,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inventory: Option<IntertemporalInventorySpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalResourceUsage {
    pub id: String,
    pub resource_id: String,
    pub service_id: String,
    pub resource_unit: String,
    pub service_unit: String,
    /// Resource quanta used per service-delivery quantum. Rational coefficients are
    /// rounded upward per usage row, so integer accounting never understates usage.
    pub usage_numerator: Option<u64>,
    pub usage_denominator: Option<u64>,
    pub lineage: IntertemporalLineage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalAllocationRequest {
    pub schema_version: String,
    pub id: String,
    pub comparison_pair_id: String,
    pub design_role: IntertemporalDesignRole,
    pub scenario_id: String,
    /// Named stress/failure bundle IDs; bundle counts are not probabilities.
    pub failure_bundle_ids: Vec<String>,
    pub time_steps: Vec<IntertemporalTimeStep>,
    pub policy: IntertemporalAllocationPolicy,
    pub services: Vec<IntertemporalServiceDemand>,
    pub resources: Vec<IntertemporalResource>,
    pub usages: Vec<IntertemporalResourceUsage>,
    /// Caller-selected exhaustive search budget, bounded again by the hard limit.
    pub max_search_nodes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntertemporalAllocationDisposition {
    Optimal,
    IndeterminateInput,
    Infeasible,
    SearchCapExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntertemporalFloorStatus {
    MeetsFloor,
    BelowFloor,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalServiceStepResult {
    pub service_id: String,
    pub community_id: String,
    pub step_id: String,
    pub demand_quanta: Option<u64>,
    pub allocated_quanta: Option<u64>,
    pub essential_floor_quanta: Option<u64>,
    pub coverage_basis_points: Option<u16>,
    pub floor_status: IntertemporalFloorStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalResourceStepResult {
    pub resource_id: String,
    pub step_id: String,
    pub resource_unit: String,
    pub capacity_quanta: u64,
    pub consumed_quanta: u64,
    pub residual_capacity_quanta: u64,
    /// Inventory columns are null for non-inventory flow resources.
    pub opening_stock_quanta: Option<u64>,
    pub inflow_quanta: Option<u64>,
    pub loss_quanta: Option<u64>,
    pub withdrawal_quanta: Option<u64>,
    pub spill_quanta: Option<u64>,
    pub ending_stock_quanta: Option<u64>,
    pub balance_equation_holds: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalAllocationAssessment {
    pub schema_version: String,
    pub request_id: String,
    pub comparison_pair_id: String,
    pub design_role: IntertemporalDesignRole,
    pub scenario_id: String,
    pub failure_bundle_ids: Vec<String>,
    pub policy: IntertemporalAllocationPolicy,
    pub disposition: IntertemporalAllocationDisposition,
    /// None means the complete Cartesian space overflowed the counter or was
    /// deliberately not calculated; an over-budget search never emits a plan.
    pub search_space_size: Option<u64>,
    pub search_nodes_examined: u64,
    pub feasible_plan_count: Option<u64>,
    pub search_complete: bool,
    pub service_steps: Vec<IntertemporalServiceStepResult>,
    pub resource_steps: Vec<IntertemporalResourceStepResult>,
    pub input_blockers: Vec<String>,
    pub supporting_evidence_ids: Vec<String>,
    pub assumption_ids: Vec<String>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntertemporalValidationIssue {
    pub code: String,
    pub path: String,
}

const CLAIM_CEILING: &str = "Exact bounded allocation and declared integer-quanta inventory accounting only; no hydraulic/electrical physics, failure probability, unique-person welfare, causal impact, or operational authority.";

fn push_issue(issues: &mut Vec<IntertemporalValidationIssue>, code: &str, path: impl Into<String>) {
    issues.push(IntertemporalValidationIssue {
        code: code.to_owned(),
        path: path.into(),
    });
}

fn validate_lineage(
    lineage: &IntertemporalLineage,
    path: &str,
    issues: &mut Vec<IntertemporalValidationIssue>,
) {
    let mut seen_evidence = BTreeSet::new();
    for id in &lineage.evidence_ids {
        if id.trim().is_empty() {
            push_issue(issues, "lineage_evidence_id_empty", path);
        }
        if !seen_evidence.insert(id) {
            push_issue(issues, "lineage_evidence_id_duplicate", path);
        }
    }
    let mut seen_assumptions = BTreeSet::new();
    for id in &lineage.assumption_ids {
        if id.trim().is_empty() {
            push_issue(issues, "lineage_assumption_id_empty", path);
        }
        if !seen_assumptions.insert(id) {
            push_issue(issues, "lineage_assumption_id_duplicate", path);
        }
    }

    match lineage.basis {
        IntertemporalLineageBasis::Synthetic => {
            if !lineage.evidence_ids.is_empty() || !lineage.assumption_ids.is_empty() {
                push_issue(issues, "synthetic_lineage_must_not_claim_source_evidence", path);
            }
        }
        IntertemporalLineageBasis::Assumed => {
            if lineage.assumption_ids.is_empty() || !lineage.evidence_ids.is_empty() {
                push_issue(issues, "assumed_lineage_requires_assumption_ids_only", path);
            }
        }
        IntertemporalLineageBasis::Observed | IntertemporalLineageBasis::Forecast => {
            if lineage.evidence_ids.is_empty() {
                push_issue(issues, "evidence_backed_lineage_requires_evidence_ids", path);
            }
        }
        IntertemporalLineageBasis::Scenario => {
            if lineage.assumption_ids.is_empty() {
                push_issue(issues, "scenario_lineage_requires_assumption_ids", path);
            }
        }
    }
}

fn validate_series(
    series: &IntertemporalQuantitySeries,
    expected_len: usize,
    path: &str,
    issues: &mut Vec<IntertemporalValidationIssue>,
) {
    if series.unit.trim().is_empty() {
        push_issue(issues, "quantity_series_unit_empty", path);
    }
    if series.values.len() != expected_len {
        push_issue(issues, "quantity_series_length_mismatch", path);
    }
    validate_lineage(&series.lineage, &format!("{path}.lineage"), issues);
}

/// Structural/domain admission only. Unknown numeric values are permitted here so they can
/// produce a non-rankable IndeterminateInput assessment rather than silently becoming zero.
pub fn validate_intertemporal_allocation_request(
    request: &IntertemporalAllocationRequest,
) -> Vec<IntertemporalValidationIssue> {
    let mut issues = Vec::new();
    if request.schema_version != INTERTEMPORAL_ALLOCATION_SCHEMA_VERSION {
        push_issue(&mut issues, "unsupported_schema_version", "schema_version");
    }
    if request.id.trim().is_empty() {
        push_issue(&mut issues, "request_id_empty", "id");
    }
    if request.comparison_pair_id.trim().is_empty() {
        push_issue(&mut issues, "comparison_pair_id_empty", "comparison_pair_id");
    }
    if request.scenario_id.trim().is_empty() {
        push_issue(&mut issues, "scenario_id_empty", "scenario_id");
    }
    let mut seen_failures = BTreeSet::new();
    for id in &request.failure_bundle_ids {
        if id.trim().is_empty() {
            push_issue(&mut issues, "failure_bundle_id_empty", "failure_bundle_ids");
        }
        if !seen_failures.insert(id) {
            push_issue(&mut issues, "failure_bundle_id_duplicate", "failure_bundle_ids");
        }
    }
    if request.time_steps.is_empty() {
        push_issue(&mut issues, "time_grid_empty", "time_steps");
    }
    if request.time_steps.len() > MAX_INTERTEMPORAL_STEPS {
        push_issue(&mut issues, "time_grid_too_large_for_exact_reference", "time_steps");
    }
    let mut seen_steps = BTreeSet::new();
    for (index, step) in request.time_steps.iter().enumerate() {
        if step.id.trim().is_empty() {
            push_issue(&mut issues, "time_step_id_empty", format!("time_steps[{index}].id"));
        }
        if !seen_steps.insert(&step.id) {
            push_issue(&mut issues, "time_step_id_duplicate", format!("time_steps[{index}].id"));
        }
        if step.start_unix_seconds >= step.end_unix_seconds_exclusive {
            push_issue(&mut issues, "time_step_interval_invalid", format!("time_steps[{index}].start_unix_seconds"));
        }
        if index > 0
            && request.time_steps[index - 1].end_unix_seconds_exclusive != step.start_unix_seconds
        {
            push_issue(&mut issues, "time_grid_not_contiguous", format!("time_steps[{index}]"));
        }
    }
    if request.services.is_empty() {
        push_issue(&mut issues, "service_set_empty", "services");
    }
    if request.services.len() > MAX_INTERTEMPORAL_SERVICES {
        push_issue(&mut issues, "service_count_exceeds_exact_reference_limit", "services");
    }
    if request.resources.is_empty() {
        push_issue(&mut issues, "resource_set_empty", "resources");
    }
    if request.resources.len() > MAX_INTERTEMPORAL_RESOURCES {
        push_issue(&mut issues, "resource_count_exceeds_exact_reference_limit", "resources");
    }
    if request.usages.is_empty() {
        push_issue(&mut issues, "usage_set_empty", "usages");
    }
    if request.usages.len() > MAX_INTERTEMPORAL_USAGES {
        push_issue(&mut issues, "usage_count_exceeds_exact_reference_limit", "usages");
    }
    if request.max_search_nodes == 0 || request.max_search_nodes > MAX_INTERTEMPORAL_SEARCH_NODES {
        push_issue(&mut issues, "search_budget_out_of_range", "max_search_nodes");
    }

    let step_count = request.time_steps.len();
    let mut service_by_id = BTreeMap::<String, &IntertemporalServiceDemand>::new();
    for (index, service) in request.services.iter().enumerate() {
        let base = format!("services[{index}]");
        if service.id.trim().is_empty() {
            push_issue(&mut issues, "service_id_empty", format!("{base}.id"));
        }
        if service.community_id.trim().is_empty() {
            push_issue(&mut issues, "community_id_empty", format!("{base}.community_id"));
        }
        if service.beneficiary_weight == 0 {
            push_issue(&mut issues, "beneficiary_weight_zero", format!("{base}.beneficiary_weight"));
        }
        if service.essential_floor_basis_points > 10_000 {
            push_issue(&mut issues, "essential_floor_out_of_range", format!("{base}.essential_floor_basis_points"));
        }
        validate_series(&service.demand, step_count, &format!("{base}.demand"), &mut issues);
        for (step_index, value) in service.demand.values.iter().enumerate() {
            if *value == Some(0) {
                push_issue(
                    &mut issues,
                    "zero_demand_interval_not_supported_in_v1",
                    format!("{base}.demand.values[{step_index}]"),
                );
            }
        }
        if service_by_id.insert(service.id.clone(), service).is_some() {
            push_issue(&mut issues, "service_id_duplicate", format!("{base}.id"));
        }
    }

    let mut resource_by_id = BTreeMap::<String, &IntertemporalResource>::new();
    for (index, resource) in request.resources.iter().enumerate() {
        let base = format!("resources[{index}]");
        if resource.id.trim().is_empty() {
            push_issue(&mut issues, "resource_id_empty", format!("{base}.id"));
        }
        validate_series(&resource.capacity, step_count, &format!("{base}.capacity"), &mut issues);
        if let Some(inventory) = &resource.inventory {
            if inventory.maximum_stock_quanta == 0
                || inventory.minimum_reserve_quanta > inventory.maximum_stock_quanta
                || inventory.terminal_reserve_quanta > inventory.maximum_stock_quanta
            {
                push_issue(&mut issues, "inventory_storage_bounds_invalid", format!("{base}.inventory"));
            }
            if inventory
                .opening_stock_quanta
                .is_some_and(|opening| opening > inventory.maximum_stock_quanta)
            {
                push_issue(&mut issues, "opening_stock_exceeds_maximum", format!("{base}.inventory.opening_stock_quanta"));
            }
            validate_lineage(
                &inventory.opening_stock_lineage,
                &format!("{base}.inventory.opening_stock_lineage"),
                &mut issues,
            );
            validate_lineage(
                &inventory.storage_bounds_lineage,
                &format!("{base}.inventory.storage_bounds_lineage"),
                &mut issues,
            );
            validate_series(&inventory.inflow, step_count, &format!("{base}.inventory.inflow"), &mut issues);
            validate_series(&inventory.losses, step_count, &format!("{base}.inventory.losses"), &mut issues);
            if inventory.inflow.unit != resource.capacity.unit {
                push_issue(&mut issues, "inventory_inflow_unit_mismatch", format!("{base}.inventory.inflow.unit"));
            }
            if inventory.losses.unit != resource.capacity.unit {
                push_issue(&mut issues, "inventory_loss_unit_mismatch", format!("{base}.inventory.losses.unit"));
            }
        }
        if resource_by_id.insert(resource.id.clone(), resource).is_some() {
            push_issue(&mut issues, "resource_id_duplicate", format!("{base}.id"));
        }
    }

    let mut seen_usage_ids = BTreeSet::new();
    let mut seen_resource_service_pairs = BTreeSet::new();
    let mut used_services = BTreeSet::new();
    let mut used_resources = BTreeSet::new();
    for (index, usage) in request.usages.iter().enumerate() {
        let base = format!("usages[{index}]");
        if usage.id.trim().is_empty() {
            push_issue(&mut issues, "usage_id_empty", format!("{base}.id"));
        }
        if !seen_usage_ids.insert(usage.id.clone()) {
            push_issue(&mut issues, "usage_id_duplicate", format!("{base}.id"));
        }
        validate_lineage(&usage.lineage, &format!("{base}.lineage"), &mut issues);
        let resource = resource_by_id.get(&usage.resource_id).copied();
        let service = service_by_id.get(&usage.service_id).copied();
        if resource.is_none() {
            push_issue(&mut issues, "usage_resource_unknown", format!("{base}.resource_id"));
        }
        if service.is_none() {
            push_issue(&mut issues, "usage_service_unknown", format!("{base}.service_id"));
        }
        if let Some(resource) = resource {
            if usage.resource_unit != resource.capacity.unit {
                push_issue(&mut issues, "usage_resource_unit_mismatch", format!("{base}.resource_unit"));
            }
        }
        if let Some(service) = service {
            if usage.service_unit != service.demand.unit {
                push_issue(&mut issues, "usage_service_unit_mismatch", format!("{base}.service_unit"));
            }
        }
        match (usage.usage_numerator, usage.usage_denominator) {
            (Some(numerator), Some(denominator)) if numerator > 0 && denominator > 0 => {}
            (None, None) => {}
            _ => push_issue(&mut issues, "usage_ratio_invalid_or_partially_unknown", format!("{base}.usage_numerator")),
        }
        if !seen_resource_service_pairs.insert((usage.resource_id.clone(), usage.service_id.clone())) {
            push_issue(&mut issues, "duplicate_resource_service_usage", base.clone());
        }
        used_services.insert(usage.service_id.clone());
        used_resources.insert(usage.resource_id.clone());
    }
    for service in request.services.iter() {
        if !used_services.contains(&service.id) {
            push_issue(&mut issues, "service_without_resource_constraint", format!("services.{}", service.id));
        }
    }
    for resource in request.resources.iter() {
        if !used_resources.contains(&resource.id) {
            push_issue(&mut issues, "resource_without_service_constraint", format!("resources.{}", resource.id));
        }
    }

    issues.sort_by(|left, right| (&left.code, &left.path).cmp(&(&right.code, &right.path)));
    issues.dedup();
    issues
}

#[derive(Debug, Clone, Copy)]
struct CoverageFraction {
    numerator: u64,
    denominator: u64,
}

impl PartialEq for CoverageFraction {
    fn eq(&self, other: &Self) -> bool {
        (self.numerator as u128) * (other.denominator as u128)
            == (other.numerator as u128) * (self.denominator as u128)
    }
}
impl Eq for CoverageFraction {}
impl PartialOrd for CoverageFraction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CoverageFraction {
    fn cmp(&self, other: &Self) -> Ordering {
        ((self.numerator as u128) * (other.denominator as u128))
            .cmp(&((other.numerator as u128) * (self.denominator as u128)))
    }
}

#[derive(Debug, Clone)]
struct IndexedUsage {
    resource_index: usize,
    service_index: usize,
    numerator: u64,
    denominator: u64,
}

fn essential_floor_quanta(demand: u64, floor_basis_points: u16) -> u64 {
    let product = (demand as u128) * (floor_basis_points as u128);
    ((product + BASIS_POINTS - 1) / BASIS_POINTS) as u64
}

fn weighted_floor_shortfall(
    allocation: &[u64],
    services: &[&IntertemporalServiceDemand],
    demands: &[Vec<u64>],
    step_count: usize,
) -> u128 {
    let mut score = 0_u128;
    for (service_index, service) in services.iter().enumerate() {
        for step_index in 0..step_count {
            let target = essential_floor_quanta(
                demands[service_index][step_index],
                service.essential_floor_basis_points,
            );
            let delivered = allocation[service_index * step_count + step_index];
            let shortfall = target.saturating_sub(delivered);
            score += (shortfall as u128) * (service.beneficiary_weight as u128);
        }
    }
    score
}

fn weighted_delivered_quanta(
    allocation: &[u64],
    services: &[&IntertemporalServiceDemand],
    step_count: usize,
) -> u128 {
    let mut score = 0_u128;
    for (service_index, service) in services.iter().enumerate() {
        for step_index in 0..step_count {
            score += (allocation[service_index * step_count + step_index] as u128)
                * (service.beneficiary_weight as u128);
        }
    }
    score
}

fn leximin_coverage(
    allocation: &[u64],
    demands: &[Vec<u64>],
    step_count: usize,
) -> Vec<CoverageFraction> {
    let mut fractions = Vec::with_capacity(allocation.len());
    for service_index in 0..demands.len() {
        for step_index in 0..step_count {
            fractions.push(CoverageFraction {
                numerator: allocation[service_index * step_count + step_index],
                denominator: demands[service_index][step_index],
            });
        }
    }
    fractions.sort();
    fractions
}

fn priority_delivery_vector(
    allocation: &[u64],
    services: &[&IntertemporalServiceDemand],
    step_count: usize,
) -> Vec<u128> {
    let priorities: BTreeSet<u16> = services.iter().map(|service| service.priority_rank).collect();
    priorities
        .into_iter()
        .map(|priority| {
            let mut sum = 0_u128;
            for (service_index, service) in services.iter().enumerate() {
                if service.priority_rank == priority {
                    for step_index in 0..step_count {
                        sum += (allocation[service_index * step_count + step_index] as u128)
                            * (service.beneficiary_weight as u128);
                    }
                }
            }
            sum
        })
        .collect()
}

fn compare_allocations(
    policy: IntertemporalAllocationPolicy,
    left: &[u64],
    right: &[u64],
    services: &[&IntertemporalServiceDemand],
    demands: &[Vec<u64>],
    step_count: usize,
) -> Ordering {
    let left_shortfall = weighted_floor_shortfall(left, services, demands, step_count);
    let right_shortfall = weighted_floor_shortfall(right, services, demands, step_count);
    let left_delivered = weighted_delivered_quanta(left, services, step_count);
    let right_delivered = weighted_delivered_quanta(right, services, step_count);
    let left_leximin = leximin_coverage(left, demands, step_count);
    let right_leximin = leximin_coverage(right, demands, step_count);

    let primary = match policy {
        IntertemporalAllocationPolicy::EqualCoverageLeximin => {
            left_leximin.cmp(&right_leximin)
        }
        IntertemporalAllocationPolicy::FixedPriority => {
            priority_delivery_vector(left, services, step_count)
                .cmp(&priority_delivery_vector(right, services, step_count))
        }
        IntertemporalAllocationPolicy::EssentialFloorFirst => {
            right_shortfall.cmp(&left_shortfall)
        }
    };
    if primary != Ordering::Equal {
        return primary;
    }

    let secondary = match policy {
        IntertemporalAllocationPolicy::EqualCoverageLeximin => {
            right_shortfall
                .cmp(&left_shortfall)
                .then_with(|| left_delivered.cmp(&right_delivered))
        }
        IntertemporalAllocationPolicy::FixedPriority => {
            right_shortfall
                .cmp(&left_shortfall)
                .then_with(|| left_delivered.cmp(&right_delivered))
                .then_with(|| left_leximin.cmp(&right_leximin))
        }
        IntertemporalAllocationPolicy::EssentialFloorFirst => {
            left_delivered
                .cmp(&right_delivered)
                .then_with(|| left_leximin.cmp(&right_leximin))
        }
    };
    if secondary != Ordering::Equal {
        return secondary;
    }

    // Explicit deterministic tie-break over canonical service-ID / time-step order.
    // This is a reproducibility rule, not a claim that IDs have normative priority.
    right.cmp(left)
}

#[derive(Debug, Clone)]
struct CandidateSlot {
    demand: u64,
}

fn build_unknown_paths(request: &IntertemporalAllocationRequest) -> Vec<String> {
    let mut unknown = Vec::new();
    for service in &request.services {
        for (index, value) in service.demand.values.iter().enumerate() {
            if value.is_none() {
                let step_id = request.time_steps.get(index).map(|step| step.id.as_str()).unwrap_or("unknown-step");
                unknown.push(format!("services.{}.demand[{step_id}]", service.id));
            }
        }
    }
    for resource in &request.resources {
        for (index, value) in resource.capacity.values.iter().enumerate() {
            if value.is_none() {
                let step_id = request.time_steps.get(index).map(|step| step.id.as_str()).unwrap_or("unknown-step");
                unknown.push(format!("resources.{}.capacity[{step_id}]", resource.id));
            }
        }
        if let Some(inventory) = &resource.inventory {
            if inventory.opening_stock_quanta.is_none() {
                unknown.push(format!("resources.{}.inventory.opening_stock", resource.id));
            }
            for (index, value) in inventory.inflow.values.iter().enumerate() {
                if value.is_none() {
                    let step_id = request.time_steps.get(index).map(|step| step.id.as_str()).unwrap_or("unknown-step");
                    unknown.push(format!("resources.{}.inventory.inflow[{step_id}]", resource.id));
                }
            }
            for (index, value) in inventory.losses.values.iter().enumerate() {
                if value.is_none() {
                    let step_id = request.time_steps.get(index).map(|step| step.id.as_str()).unwrap_or("unknown-step");
                    unknown.push(format!("resources.{}.inventory.losses[{step_id}]", resource.id));
                }
            }
        }
    }
    for usage in &request.usages {
        if usage.usage_numerator.is_none() && usage.usage_denominator.is_none() {
            unknown.push(format!("usages.{}.coefficient", usage.id));
        }
    }
    unknown.sort();
    unknown.dedup();
    unknown
}

fn collect_lineage_references(
    request: &IntertemporalAllocationRequest,
) -> (Vec<String>, Vec<String>) {
    let mut evidence = BTreeSet::new();
    let mut assumptions = BTreeSet::new();
    let mut add = |lineage: &IntertemporalLineage| {
        evidence.extend(lineage.evidence_ids.iter().cloned());
        assumptions.extend(lineage.assumption_ids.iter().cloned());
    };
    for service in &request.services {
        add(&service.demand.lineage);
    }
    for resource in &request.resources {
        add(&resource.capacity.lineage);
        if let Some(inventory) = &resource.inventory {
            add(&inventory.opening_stock_lineage);
            add(&inventory.storage_bounds_lineage);
            add(&inventory.inflow.lineage);
            add(&inventory.losses.lineage);
        }
    }
    for usage in &request.usages {
        add(&usage.lineage);
    }
    (evidence.into_iter().collect(), assumptions.into_iter().collect())
}

fn service_step_skeleton(
    request: &IntertemporalAllocationRequest,
    services_sorted: &[&IntertemporalServiceDemand],
) -> Vec<IntertemporalServiceStepResult> {
    let mut output = Vec::new();
    for service in services_sorted {
        for (step_index, step) in request.time_steps.iter().enumerate() {
            let demand = service.demand.values.get(step_index).copied().flatten();
            let floor = demand.map(|value| essential_floor_quanta(value, service.essential_floor_basis_points));
            output.push(IntertemporalServiceStepResult {
                service_id: service.id.clone(),
                community_id: service.community_id.clone(),
                step_id: step.id.clone(),
                demand_quanta: demand,
                allocated_quanta: None,
                essential_floor_quanta: floor,
                coverage_basis_points: None,
                floor_status: IntertemporalFloorStatus::Indeterminate,
            });
        }
    }
    output
}

fn empty_assessment(
    request: &IntertemporalAllocationRequest,
    disposition: IntertemporalAllocationDisposition,
    search_space_size: Option<u64>,
    search_nodes_examined: u64,
    feasible_plan_count: Option<u64>,
    search_complete: bool,
    input_blockers: Vec<String>,
    services_sorted: &[&IntertemporalServiceDemand],
    evidence_ids: &[String],
    assumption_ids: &[String],
) -> IntertemporalAllocationAssessment {
    IntertemporalAllocationAssessment {
        schema_version: INTERTEMPORAL_ALLOCATION_SCHEMA_VERSION.to_owned(),
        request_id: request.id.clone(),
        comparison_pair_id: request.comparison_pair_id.clone(),
        design_role: request.design_role,
        scenario_id: request.scenario_id.clone(),
        failure_bundle_ids: {
            let mut ids = request.failure_bundle_ids.clone();
            ids.sort();
            ids
        },
        policy: request.policy,
        disposition,
        search_space_size,
        search_nodes_examined,
        feasible_plan_count,
        search_complete,
        service_steps: service_step_skeleton(request, services_sorted),
        resource_steps: Vec::new(),
        input_blockers,
        supporting_evidence_ids: evidence_ids.to_vec(),
        assumption_ids: assumption_ids.to_vec(),
        claim_ceiling: CLAIM_CEILING.to_owned(),
    }
}

fn resource_quanta_for_delivery_checked(
    delivered_quanta: u64,
    numerator: u64,
    denominator: u64,
) -> Option<u64> {
    let product = (delivered_quanta as u128).checked_mul(numerator as u128)?;
    let divisor = denominator as u128;
    let rounded = product.checked_add(divisor - 1)?.checked_div(divisor)?;
    u64::try_from(rounded).ok()
}

fn build_resource_step_results(
    allocation: &[u64],
    services_sorted: &[&IntertemporalServiceDemand],
    resources_sorted: &[&IntertemporalResource],
    usages_indexed: &[IndexedUsage],
    step_count: usize,
    step_ids: &[String],
) -> Option<Vec<IntertemporalResourceStepResult>> {
    let mut used = vec![vec![0_u128; step_count]; resources_sorted.len()];
    for usage in usages_indexed {
        for step_index in 0..step_count {
            let amount = allocation[usage.service_index * step_count + step_index];
            let consumed = resource_quanta_for_delivery_checked(amount, usage.numerator, usage.denominator)?;
            used[usage.resource_index][step_index] =
                used[usage.resource_index][step_index].checked_add(consumed as u128)?;
        }
    }

    let mut output = Vec::with_capacity(resources_sorted.len() * step_count);
    for (resource_index, resource) in resources_sorted.iter().enumerate() {
        let inventory = resource.inventory.as_ref();
        let mut stock = inventory.and_then(|item| item.opening_stock_quanta);
        for step_index in 0..step_count {
            let capacity = resource.capacity.values[step_index]?;
            if used[resource_index][step_index] > capacity as u128 {
                return None;
            }
            let consumed = u64::try_from(used[resource_index][step_index]).ok()?;
            let mut row = IntertemporalResourceStepResult {
                resource_id: resource.id.clone(),
                step_id: step_ids[step_index].clone(),
                resource_unit: resource.capacity.unit.clone(),
                capacity_quanta: capacity,
                consumed_quanta: consumed,
                residual_capacity_quanta: capacity - consumed,
                opening_stock_quanta: None,
                inflow_quanta: None,
                loss_quanta: None,
                withdrawal_quanta: None,
                spill_quanta: None,
                ending_stock_quanta: None,
                balance_equation_holds: None,
            };

            if let Some(inventory) = inventory {
                let opening = stock?;
                let inflow = inventory.inflow.values[step_index]?;
                let loss = inventory.losses.values[step_index]?;
                let total_available = (opening as u128).checked_add(inflow as u128)?;
                if (loss as u128) > total_available {
                    return None;
                }
                let after_loss = total_available - (loss as u128);
                if (consumed as u128) > after_loss {
                    return None;
                }
                let after_withdrawal = after_loss - (consumed as u128);
                let maximum = inventory.maximum_stock_quanta as u128;
                let spill = after_withdrawal.saturating_sub(maximum);
                let ending = after_withdrawal - spill;
                if ending < inventory.minimum_reserve_quanta as u128 {
                    return None;
                }
                if step_index + 1 == step_count && ending < inventory.terminal_reserve_quanta as u128 {
                    return None;
                }
                let ending_u64 = u64::try_from(ending).ok()?;
                let spill_u64 = u64::try_from(spill).ok()?;
                let balance_left = total_available;
                let balance_right = (loss as u128)
                    .checked_add(consumed as u128)?
                    .checked_add(spill)?
                    .checked_add(ending)?;
                row.opening_stock_quanta = Some(opening);
                row.inflow_quanta = Some(inflow);
                row.loss_quanta = Some(loss);
                row.withdrawal_quanta = Some(consumed);
                row.spill_quanta = Some(spill_u64);
                row.ending_stock_quanta = Some(ending_u64);
                row.balance_equation_holds = Some(balance_left == balance_right);
                if !row.balance_equation_holds.unwrap_or(false) {
                    return None;
                }
                stock = Some(ending_u64);
            }
            output.push(row);
        }
    }
    // services_sorted is already the canonical objective order; touching it here also documents
    // that allocation indexes refer to sorted services rather than caller input order.
    let _ = services_sorted;
    Some(output)
}

fn weighted_feasible_plan_is_preferred(
    policy: IntertemporalAllocationPolicy,
    candidate: &[u64],
    incumbent: &[u64],
    services: &[&IntertemporalServiceDemand],
    demands: &[Vec<u64>],
    step_count: usize,
) -> bool {
    compare_allocations(policy, candidate, incumbent, services, demands, step_count) == Ordering::Greater
}

fn enumerate_candidates(
    index: usize,
    slots: &[CandidateSlot],
    current: &mut [u64],
    request: &IntertemporalAllocationRequest,
    services_sorted: &[&IntertemporalServiceDemand],
    resources_sorted: &[&IntertemporalResource],
    usages_indexed: &[IndexedUsage],
    demands: &[Vec<u64>],
    step_ids: &[String],
    feasible_count: &mut u64,
    best: &mut Option<Vec<u64>>,
    best_resource_steps: &mut Option<Vec<IntertemporalResourceStepResult>>,
) {
    if index == slots.len() {
        let Some(ledger) = build_resource_step_results(
            current,
            services_sorted,
            resources_sorted,
            usages_indexed,
            step_ids.len(),
            step_ids,
        ) else {
            return;
        };
        *feasible_count = feasible_count.saturating_add(1);
        if best.as_ref().is_none_or(|incumbent| {
            weighted_feasible_plan_is_preferred(
                request.policy,
                current,
                incumbent,
                services_sorted,
                demands,
                step_ids.len(),
            )
        }) {
            *best = Some(current.to_vec());
            *best_resource_steps = Some(ledger);
        }
        return;
    }

    for amount in 0..=slots[index].demand {
        current[index] = amount;
        enumerate_candidates(
            index + 1,
            slots,
            current,
            request,
            services_sorted,
            resources_sorted,
            usages_indexed,
            demands,
            step_ids,
            feasible_count,
            best,
            best_resource_steps,
        );
    }
}

fn search_space_size(demands: &[Vec<u64>]) -> Option<u128> {
    let mut size = 1_u128;
    for service in demands {
        for demand in service {
            size = size.checked_mul((*demand as u128).checked_add(1)?)?;
        }
    }
    Some(size)
}

/// Exhaustive reference optimization. Inputs are validated first; unknown quantities yield an
/// indeterminate, non-rankable receipt; over-budget searches return no partial plan.
pub fn assess_intertemporal_resource_allocation(
    request: &IntertemporalAllocationRequest,
) -> Result<IntertemporalAllocationAssessment, Vec<IntertemporalValidationIssue>> {
    let issues = validate_intertemporal_allocation_request(request);
    if !issues.is_empty() {
        return Err(issues);
    }

    let mut services_sorted: Vec<&IntertemporalServiceDemand> = request.services.iter().collect();
    services_sorted.sort_by(|left, right| left.id.cmp(&right.id));
    let mut resources_sorted: Vec<&IntertemporalResource> = request.resources.iter().collect();
    resources_sorted.sort_by(|left, right| left.id.cmp(&right.id));
    let mut usages_sorted: Vec<&IntertemporalResourceUsage> = request.usages.iter().collect();
    usages_sorted.sort_by(|left, right| {
        (&left.resource_id, &left.service_id, &left.id).cmp(&(
            &right.resource_id,
            &right.service_id,
            &right.id,
        ))
    });

    let (evidence_ids, assumption_ids) = collect_lineage_references(request);
    let unknown = build_unknown_paths(request);
    if !unknown.is_empty() {
        return Ok(empty_assessment(
            request,
            IntertemporalAllocationDisposition::IndeterminateInput,
            None,
            0,
            None,
            false,
            unknown,
            &services_sorted,
            &evidence_ids,
            &assumption_ids,
        ));
    }

    let step_count = request.time_steps.len();
    let mut service_index = BTreeMap::new();
    let mut demands = Vec::<Vec<u64>>::with_capacity(services_sorted.len());
    for (index, service) in services_sorted.iter().enumerate() {
        service_index.insert(service.id.clone(), index);
        demands.push(
            service
                .demand
                .values
                .iter()
                .map(|value| value.expect("unknown inputs were handled above"))
                .collect(),
        );
    }
    let mut resource_index = BTreeMap::new();
    for (index, resource) in resources_sorted.iter().enumerate() {
        resource_index.insert(resource.id.clone(), index);
    }

    let space = search_space_size(&demands);
    let space_within_budget = space.is_some_and(|value| value <= request.max_search_nodes as u128);
    if !space_within_budget {
        return Ok(empty_assessment(
            request,
            IntertemporalAllocationDisposition::SearchCapExceeded,
            space.and_then(|value| u64::try_from(value).ok()),
            0,
            None,
            false,
            vec!["complete_search_space_exceeds_declared_budget".to_owned()],
            &services_sorted,
            &evidence_ids,
            &assumption_ids,
        ));
    }
    let exact_space = u64::try_from(space.expect("within-budget search space")).expect("search cap is bounded by usize");
    let indexed_usages: Vec<IndexedUsage> = usages_sorted
        .iter()
        .map(|usage| IndexedUsage {
            resource_index: *resource_index
                .get(&usage.resource_id)
                .expect("usage references validated resource"),
            service_index: *service_index
                .get(&usage.service_id)
                .expect("usage references validated service"),
            numerator: usage.usage_numerator.expect("unknown coefficient handled above"),
            denominator: usage.usage_denominator.expect("unknown coefficient handled above"),
        })
        .collect();

    let slots: Vec<CandidateSlot> = demands
        .iter()
        .flat_map(|service_demands| service_demands.iter().map(|demand| CandidateSlot { demand: *demand }))
        .collect();
    let mut current = vec![0_u64; slots.len()];
    let mut best = None;
    let mut best_resource_steps = None;
    let mut feasible_count = 0_u64;
    enumerate_candidates(
        0,
        &slots,
        &mut current,
        request,
        &services_sorted,
        &resources_sorted,
        &indexed_usages,
        &demands,
        &request.time_steps.iter().map(|step| step.id.clone()).collect::<Vec<_>>(),
        &mut feasible_count,
        &mut best,
        &mut best_resource_steps,
    );

    let Some(best_allocation) = best else {
        let mut result = empty_assessment(
            request,
            IntertemporalAllocationDisposition::Infeasible,
            Some(exact_space),
            exact_space,
            Some(0),
            true,
            vec!["no_feasible_plan_satisfies_all_resource_and_terminal_reserve_constraints".to_owned()],
            &services_sorted,
            &evidence_ids,
            &assumption_ids,
        );
        result.resource_steps.clear();
        return Ok(result);
    };

    let mut service_steps = Vec::with_capacity(slots.len());
    for (service_index, service) in services_sorted.iter().enumerate() {
        for (step_index, step) in request.time_steps.iter().enumerate() {
            let demand = demands[service_index][step_index];
            let allocated = best_allocation[service_index * step_count + step_index];
            let floor = essential_floor_quanta(demand, service.essential_floor_basis_points);
            let coverage = ((allocated as u128) * BASIS_POINTS / (demand as u128)) as u16;
            service_steps.push(IntertemporalServiceStepResult {
                service_id: service.id.clone(),
                community_id: service.community_id.clone(),
                step_id: step.id.clone(),
                demand_quanta: Some(demand),
                allocated_quanta: Some(allocated),
                essential_floor_quanta: Some(floor),
                coverage_basis_points: Some(coverage.min(10_000)),
                floor_status: if allocated >= floor {
                    IntertemporalFloorStatus::MeetsFloor
                } else {
                    IntertemporalFloorStatus::BelowFloor
                },
            });
        }
    }

    Ok(IntertemporalAllocationAssessment {
        schema_version: INTERTEMPORAL_ALLOCATION_SCHEMA_VERSION.to_owned(),
        request_id: request.id.clone(),
        comparison_pair_id: request.comparison_pair_id.clone(),
        design_role: request.design_role,
        scenario_id: request.scenario_id.clone(),
        failure_bundle_ids: {
            let mut ids = request.failure_bundle_ids.clone();
            ids.sort();
            ids
        },
        policy: request.policy,
        disposition: IntertemporalAllocationDisposition::Optimal,
        search_space_size: Some(exact_space),
        search_nodes_examined: exact_space,
        feasible_plan_count: Some(feasible_count),
        search_complete: true,
        service_steps,
        resource_steps: best_resource_steps.expect("best allocation has a feasible resource ledger"),
        input_blockers: Vec::new(),
        supporting_evidence_ids: evidence_ids,
        assumption_ids,
        claim_ceiling: CLAIM_CEILING.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::service_shared_resource_allocation::{
        assess_shared_resource_allocation, SharedResourceAllocationPolicy,
        SharedResourceAllocationRequest, SharedResourceCapacity, SharedResourceServiceDemand,
        SharedResourceUsage,
    };
    use crate::service_function::FunctionMeasurement;
    use crate::spatial_economic::{EvidenceRecord, Quantity};

    fn synthetic_lineage() -> IntertemporalLineage {
        IntertemporalLineage {
            basis: IntertemporalLineageBasis::Synthetic,
            evidence_ids: Vec::new(),
            assumption_ids: Vec::new(),
        }
    }

    fn series(unit: &str, values: Vec<Option<u64>>) -> IntertemporalQuantitySeries {
        IntertemporalQuantitySeries {
            unit: unit.to_owned(),
            values,
            lineage: synthetic_lineage(),
        }
    }

    fn time_steps() -> Vec<IntertemporalTimeStep> {
        vec![
            IntertemporalTimeStep { id: "t0".to_owned(), start_unix_seconds: 0, end_unix_seconds_exclusive: 60 },
            IntertemporalTimeStep { id: "t1".to_owned(), start_unix_seconds: 60, end_unix_seconds_exclusive: 120 },
        ]
    }

    fn service(id: &str, weight: u64, priority: u16, demand: Vec<Option<u64>>, floor_bps: u16) -> IntertemporalServiceDemand {
        IntertemporalServiceDemand {
            id: id.to_owned(),
            community_id: format!("community-{id}"),
            beneficiary_weight: weight,
            priority_rank: priority,
            demand: series("delivery-quanta", demand),
            essential_floor_basis_points: floor_bps,
        }
    }

    fn inventory_resource(
        id: &str,
        opening: Option<u64>,
        capacity: Vec<Option<u64>>,
        inflow: Vec<Option<u64>>,
        losses: Vec<Option<u64>>,
        max_stock: u64,
        min_reserve: u64,
        terminal_reserve: u64,
    ) -> IntertemporalResource {
        IntertemporalResource {
            id: id.to_owned(),
            capacity: series("water-quanta", capacity),
            inventory: Some(IntertemporalInventorySpec {
                opening_stock_quanta: opening,
                maximum_stock_quanta: max_stock,
                minimum_reserve_quanta: min_reserve,
                terminal_reserve_quanta: terminal_reserve,
                opening_stock_lineage: synthetic_lineage(),
                storage_bounds_lineage: synthetic_lineage(),
                inflow: series("water-quanta", inflow),
                losses: series("water-quanta", losses),
            }),
        }
    }

    fn usage(id: &str, resource_id: &str, service_id: &str, numerator: Option<u64>, denominator: Option<u64>) -> IntertemporalResourceUsage {
        IntertemporalResourceUsage {
            id: id.to_owned(),
            resource_id: resource_id.to_owned(),
            service_id: service_id.to_owned(),
            resource_unit: "water-quanta".to_owned(),
            service_unit: "delivery-quanta".to_owned(),
            usage_numerator: numerator,
            usage_denominator: denominator,
            lineage: synthetic_lineage(),
        }
    }

    fn tank_request(policy: IntertemporalAllocationPolicy) -> IntertemporalAllocationRequest {
        IntertemporalAllocationRequest {
            schema_version: INTERTEMPORAL_ALLOCATION_SCHEMA_VERSION.to_owned(),
            id: "two-step-tank".to_owned(),
            comparison_pair_id: "pair-tank-001".to_owned(),
            design_role: IntertemporalDesignRole::Candidate,
            scenario_id: "synthetic-late-inflow".to_owned(),
            failure_bundle_ids: vec!["feeder-derate-t1".to_owned()],
            time_steps: time_steps(),
            policy,
            services: vec![
                service("essential-a", 10, 0, vec![Some(1), Some(1)], 10_000),
                service("general-b", 1, 1, vec![Some(3), Some(3)], 10_000),
            ],
            resources: vec![
                inventory_resource("tank", Some(3), vec![Some(3), Some(3)], vec![Some(0), Some(3)], vec![Some(0), Some(0)], 4, 0, 2)
            ],
            usages: vec![
                usage("usage-a", "tank", "essential-a", Some(1), Some(1)),
                usage("usage-b", "tank", "general-b", Some(1), Some(1)),
            ],
            max_search_nodes: 100_000,
        }
    }

    #[test]
    fn two_step_planner_uses_later_inflow_and_preserves_terminal_reserve() {
        let request = tank_request(IntertemporalAllocationPolicy::EssentialFloorFirst);
        let result = assess_intertemporal_resource_allocation(&request).expect("valid fixture");
        assert_eq!(result.disposition, IntertemporalAllocationDisposition::Optimal);
        assert!(result.search_complete);
        assert_eq!(result.search_space_size, Some(4 * 4 * 2 * 2));
        assert_eq!(result.service_steps.len(), 4);

        let allocation: BTreeMap<_, _> = result.service_steps.iter()
            .map(|row| ((row.service_id.as_str(), row.step_id.as_str()), row.allocated_quanta.unwrap()))
            .collect();
        assert_eq!(allocation[("essential-a", "t0")], 1);
        assert_eq!(allocation[("essential-a", "t1")], 1);
        assert_eq!(allocation[("general-b", "t0")], 1);
        assert_eq!(allocation[("general-b", "t1")], 1);

        let rows: Vec<_> = result.resource_steps.iter().filter(|row| row.resource_id == "tank").collect();
        assert_eq!(rows[0].opening_stock_quanta, Some(3));
        assert_eq!(rows[0].inflow_quanta, Some(0));
        assert_eq!(rows[0].consumed_quanta, 2);
        assert_eq!(rows[0].ending_stock_quanta, Some(1));
        assert_eq!(rows[1].opening_stock_quanta, Some(1));
        assert_eq!(rows[1].inflow_quanta, Some(3));
        assert_eq!(rows[1].consumed_quanta, 2);
        assert_eq!(rows[1].ending_stock_quanta, Some(2));
        assert_eq!(rows[1].balance_equation_holds, Some(true));
        assert!(rows.iter().all(|row| row.residual_capacity_quanta <= row.capacity_quanta));
    }

    #[test]
    fn missing_capacity_is_indeterminate_not_zero_capacity() {
        let mut request = tank_request(IntertemporalAllocationPolicy::EssentialFloorFirst);
        request.resources[0].capacity.values[1] = None;
        let result = assess_intertemporal_resource_allocation(&request).expect("well-formed unknown input");
        assert_eq!(result.disposition, IntertemporalAllocationDisposition::IndeterminateInput);
        assert_eq!(result.search_nodes_examined, 0);
        assert!(!result.search_complete);
        assert!(result.input_blockers.iter().any(|field| field == "resources.tank.capacity[t1]"));
        assert!(result.resource_steps.is_empty());
        assert!(result.service_steps.iter().all(|row| row.floor_status == IntertemporalFloorStatus::Indeterminate));
        assert!(result.service_steps.iter().all(|row| row.allocated_quanta.is_none()));
    }

    #[test]
    fn search_cap_returns_no_partial_plan() {
        let mut request = tank_request(IntertemporalAllocationPolicy::EssentialFloorFirst);
        request.max_search_nodes = 5;
        let result = assess_intertemporal_resource_allocation(&request).expect("valid bounded input");
        assert_eq!(result.disposition, IntertemporalAllocationDisposition::SearchCapExceeded);
        assert_eq!(result.search_nodes_examined, 0);
        assert!(!result.search_complete);
        assert!(result.service_steps.iter().all(|row| row.allocated_quanta.is_none()));
        assert_eq!(result.resource_steps.len(), 0);
        assert!(result.input_blockers.contains(&"complete_search_space_exceeds_declared_budget".to_owned()));
    }

    #[test]
    fn scenario_and_service_resource_permutations_are_deterministic() {
        let request = tank_request(IntertemporalAllocationPolicy::EqualCoverageLeximin);
        let mut permuted = request.clone();
        permuted.services.reverse();
        permuted.resources.reverse();
        permuted.usages.reverse();
        let first = assess_intertemporal_resource_allocation(&request).unwrap();
        let second = assess_intertemporal_resource_allocation(&permuted).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn unknown_usage_ratio_is_not_fabricated_as_zero() {
        let mut request = tank_request(IntertemporalAllocationPolicy::EssentialFloorFirst);
        request.usages[0].usage_numerator = None;
        request.usages[0].usage_denominator = None;
        let result = assess_intertemporal_resource_allocation(&request).unwrap();
        assert_eq!(result.disposition, IntertemporalAllocationDisposition::IndeterminateInput);
        assert!(result.input_blockers.iter().any(|field| field == "usages.usage-a.coefficient"));
    }

    #[test]
    fn rational_usage_rounds_up_to_conserve_resource_quanta() {
        assert_eq!(resource_quanta_for_delivery_checked(1, 1, 2), Some(1));
        assert_eq!(resource_quanta_for_delivery_checked(2, 1, 2), Some(1));
        assert_eq!(resource_quanta_for_delivery_checked(3, 1, 2), Some(2));
    }

    fn assumed_measurement(entity_id: &str, value: f64, unit: &str, rationale: &str) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity_id.to_owned(),
            quantity: Quantity { value, unit: unit.to_owned(), uncertainty_abs: None },
            basis: ScenarioBasis::Assumed,
            rationale: rationale.to_owned(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    #[test]
    fn one_step_leximin_matches_shared_resource_progressive_fill() {
        let request = IntertemporalAllocationRequest {
            schema_version: INTERTEMPORAL_ALLOCATION_SCHEMA_VERSION.to_owned(),
            id: "one-step-parity".to_owned(),
            comparison_pair_id: "pair-one-step-parity".to_owned(),
            design_role: IntertemporalDesignRole::Candidate,
            scenario_id: "synthetic-one-step".to_owned(),
            failure_bundle_ids: Vec::new(),
            time_steps: vec![IntertemporalTimeStep { id: "t0".to_owned(), start_unix_seconds: 0, end_unix_seconds_exclusive: 60 }],
            policy: IntertemporalAllocationPolicy::EqualCoverageLeximin,
            services: vec![
                IntertemporalServiceDemand {
                    id: "service-a".to_owned(), community_id: "community-a".to_owned(), beneficiary_weight: 1, priority_rank: 0,
                    demand: series("delivery-quanta", vec![Some(4)]), essential_floor_basis_points: 5_000,
                },
                IntertemporalServiceDemand {
                    id: "service-b".to_owned(), community_id: "community-b".to_owned(), beneficiary_weight: 1, priority_rank: 1,
                    demand: series("delivery-quanta", vec![Some(4)]), essential_floor_basis_points: 5_000,
                },
            ],
            resources: vec![IntertemporalResource { id: "shared".to_owned(), capacity: series("resource-quanta", vec![Some(4)]), inventory: None }],
            usages: vec![
                IntertemporalResourceUsage { id: "usage-a".to_owned(), resource_id: "shared".to_owned(), service_id: "service-a".to_owned(), resource_unit: "resource-quanta".to_owned(), service_unit: "delivery-quanta".to_owned(), usage_numerator: Some(1), usage_denominator: Some(1), lineage: synthetic_lineage() },
                IntertemporalResourceUsage { id: "usage-b".to_owned(), resource_id: "shared".to_owned(), service_id: "service-b".to_owned(), resource_unit: "resource-quanta".to_owned(), service_unit: "delivery-quanta".to_owned(), usage_numerator: Some(1), usage_denominator: Some(1), lineage: synthetic_lineage() },
            ],
            max_search_nodes: 1_000,
        };
        let exact = assess_intertemporal_resource_allocation(&request).expect("small request");
        let exact_allocations: BTreeMap<_, _> = exact.service_steps.iter()
            .map(|row| (row.service_id.as_str(), row.allocated_quanta.unwrap()))
            .collect();

        let old_snapshot = SharedResourceAllocationRequest {
            schema_version: "0.1.0".to_owned(),
            id: "one-step-parity".to_owned(),
            as_of: "2026-10-20T00:00:00Z".to_owned(),
            policy: SharedResourceAllocationPolicy::EqualCoverageProgressiveFill,
            services: vec![
                SharedResourceServiceDemand {
                    id: "service-a".to_owned(), service_entity_id: "service-entity-a".to_owned(), community_id: "community-a".to_owned(), beneficiary_weight: 1,
                    delivery_rate_unit: "delivery-quanta".to_owned(),
                    demand_rate: assumed_measurement("service-entity-a", 4.0, "delivery-quanta", "Synthetic parity test."),
                    essential_coverage_floor: 0.5,
                },
                SharedResourceServiceDemand {
                    id: "service-b".to_owned(), service_entity_id: "service-entity-b".to_owned(), community_id: "community-b".to_owned(), beneficiary_weight: 1,
                    delivery_rate_unit: "delivery-quanta".to_owned(),
                    demand_rate: assumed_measurement("service-entity-b", 4.0, "delivery-quanta", "Synthetic parity test."),
                    essential_coverage_floor: 0.5,
                },
            ],
            resources: vec![SharedResourceCapacity {
                id: "shared".to_owned(),
                capacity: assumed_measurement("shared", 4.0, "resource-quanta", "Synthetic parity test."),
            }],
            usages: vec![
                SharedResourceUsage { id: "usage-a".to_owned(), resource_id: "shared".to_owned(), service_id: "service-a".to_owned(), resource_use_per_delivery_rate: assumed_measurement("usage-a", 1.0, "resource-quanta per (delivery-quanta)", "Synthetic parity test.") },
                SharedResourceUsage { id: "usage-b".to_owned(), resource_id: "shared".to_owned(), service_id: "service-b".to_owned(), resource_use_per_delivery_rate: assumed_measurement("usage-b", 1.0, "resource-quanta per (delivery-quanta)", "Synthetic parity test.") },
            ],
            max_work_units: 10_000,
        };
        let old = assess_shared_resource_allocation(&old_snapshot, &[]).expect("existing single-snapshot model");
        assert_eq!(exact_allocations["service-a"], 2);
        assert_eq!(exact_allocations["service-b"], 2);
        for service in old.services {
            let exact_rate = exact_allocations[service.service_id.as_str()] as f64;
            assert!((exact_rate - service.conservative_allocated_delivery_rate).abs() < 1e-9);
        }
    }
}
