// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! One-snapshot shared-resource allocation with evidence-checked interval inputs.
//!
//! The evaluator declares every physical capacity once, shares it across connected service
//! cells, and uses equal-coverage progressive filling to preserve transparent floor outcomes.
//! It is a bounded allocation abstraction, not a hydraulic, electrical-load-flow, network,
//! storage-dynamics, probabilistic-reliability, or causal-impact solver.

use crate::dependency_logic::ScenarioBasis;
use crate::service_equity::ServiceFloorStatus;
use crate::service_function::{
    FunctionMeasurement, ServiceFunctionAssessmentError, ServiceFunctionValidationError,
    build_evidence_index, validate_measurement,
};
use crate::spatial_economic::{EvidenceRecord, Quantity};
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SHARED_RESOURCE_ALLOCATION_SCHEMA_VERSION: &str = "0.1.0";
pub const MAX_SHARED_RESOURCE_SERVICES: usize = 1_000;
pub const MAX_SHARED_RESOURCE_ASSETS: usize = 1_000;
pub const MAX_SHARED_RESOURCE_USAGES: usize = 20_000;
/// Conservative budget for service/usage passes, resource freezing scans, and final audit.
pub const MAX_SHARED_RESOURCE_WORK_UNITS: usize = 60_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedResourceAllocationPolicy {
    /// Raise active services by equal coverage fractions until a shared resource saturates;
    /// freeze every active service that consumes it, then continue with remaining services.
    EqualCoverageProgressiveFill,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedResourceServiceDemand {
    pub id: String,
    pub service_entity_id: String,
    pub community_id: String,
    pub beneficiary_weight: u64,
    pub delivery_rate_unit: String,
    /// Demand interval must remain strictly positive after uncertainty is applied.
    pub demand_rate: FunctionMeasurement,
    pub essential_coverage_floor: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedResourceCapacity {
    /// Each physical resource appears exactly once in this table.
    pub id: String,
    pub capacity: FunctionMeasurement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedResourceUsage {
    pub id: String,
    pub resource_id: String,
    pub service_id: String,
    /// Unit must be "<resource capacity unit> per (<delivery rate unit>)".
    /// E.g. "kW per (m3/h)". This is a declared coefficient, not a validated physical
    /// network relationship unless its evidence and operating regime support that meaning.
    pub resource_use_per_delivery_rate: FunctionMeasurement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedResourceAllocationRequest {
    pub schema_version: String,
    pub id: String,
    /// Canonical UTC timestamp. This contract evaluates one aligned snapshot only.
    pub as_of: String,
    pub policy: SharedResourceAllocationPolicy,
    pub services: Vec<SharedResourceServiceDemand>,
    pub resources: Vec<SharedResourceCapacity>,
    pub usages: Vec<SharedResourceUsage>,
    /// Caller budget for bounded solver work; must be positive and <= the hard limit.
    pub max_work_units: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SharedResourceAllocationValidationError {
    UnsupportedSchemaVersion,
    EmptyRequestId,
    InvalidAsOfTimestamp,
    EmptyServiceSet,
    EmptyResourceSet,
    EmptyUsageSet,
    ServiceCountExceeded,
    ResourceCountExceeded,
    UsageCountExceeded,
    WorkBudgetInvalid,
    WorkBudgetExceeded,
    WorkEstimateOverflow,
    EmptyServiceId,
    DuplicateServiceId,
    EmptyServiceEntityId,
    DuplicateServiceEntityId,
    EmptyCommunityId,
    ZeroBeneficiaryWeight,
    EmptyDeliveryRateUnit,
    DemandEntityMismatch,
    DemandUnitMismatch,
    DeliveryRateUnitMismatch,
    InvalidCoverageFloor,
    DemandLowerBoundNonPositive,
    EmptyResourceId,
    DuplicateResourceId,
    ResourceCapacityEntityMismatch,
    EmptyUsageId,
    DuplicateUsageId,
    UnknownUsageResource,
    UnknownUsageService,
    DuplicateServiceResourceUsage,
    ServiceWithoutResourceConstraint,
    ResourceWithoutServiceConstraint,
    UsageCoefficientEntityMismatch,
    UsageCoefficientUnitMismatch,
    NonPositiveResourceUseCoefficient,
    NonFiniteIntervalBound,
    NonFiniteAllocationResult,
    AllocationDidNotConverge,
    CapacityConstraintViolated,
    BeneficiaryWeightOverflow,
    InconsistentCoverageBounds,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SharedResourceAllocationError {
    InvalidInput(Vec<SharedResourceAllocationValidationError>),
    MeasurementValidation(Vec<ServiceFunctionValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedResourceServiceAssessment {
    pub service_id: String,
    pub service_entity_id: String,
    pub community_id: String,
    pub beneficiary_weight: u64,
    pub delivery_rate_unit: String,
    pub demand_rate_lower_bound: f64,
    pub demand_rate_upper_bound: f64,
    /// Allocation at low capacity, high demand, and high resource-use coefficients.
    /// This is a policy output, not a physical lower bound on delivered rate.
    pub conservative_allocated_delivery_rate: f64,
    /// Allocation at high capacity, low demand, and low resource-use coefficients.
    /// This is a policy output, not a physical upper bound on delivered rate.
    pub optimistic_allocated_delivery_rate: f64,
    pub coverage_lower_bound: f64,
    pub coverage_upper_bound: f64,
    pub essential_coverage_floor: f64,
    pub floor_status: ServiceFloorStatus,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedResourceAssessment {
    pub resource_id: String,
    pub capacity_unit: String,
    pub capacity_lower_bound: f64,
    pub capacity_upper_bound: f64,
    pub conservative_allocated_capacity: f64,
    pub conservative_residual_capacity: f64,
    pub conservative_constraint_binding: bool,
    pub optimistic_allocated_capacity: f64,
    pub optimistic_residual_capacity: f64,
    pub optimistic_constraint_binding: bool,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedResourceCommunityAssessment {
    pub community_id: String,
    pub service_count: usize,
    pub beneficiary_weight: u64,
    pub floor_compliance_lower_bound: f64,
    pub floor_compliance_upper_bound: f64,
    pub meets_floor_weight: u64,
    pub below_floor_weight: u64,
    pub indeterminate_floor_weight: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedResourceAllocationInterpretationBoundary {
    EqualCoverageAllocationNotPhysicalNetworkSimulation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedResourceAllocationAssessment {
    pub schema_version: String,
    pub request_id: String,
    pub as_of: String,
    pub policy: SharedResourceAllocationPolicy,
    pub service_count: usize,
    pub resource_count: usize,
    pub usage_constraint_count: usize,
    pub estimated_work_units: usize,
    pub conservative_allocation_iterations: usize,
    pub optimistic_allocation_iterations: usize,
    pub total_beneficiary_weight: u64,
    pub meets_floor_weight: u64,
    pub below_floor_weight: u64,
    pub indeterminate_floor_weight: u64,
    pub floor_compliance_lower_bound: f64,
    pub floor_compliance_upper_bound: f64,
    pub services: Vec<SharedResourceServiceAssessment>,
    pub resources: Vec<SharedResourceAssessment>,
    pub communities: Vec<SharedResourceCommunityAssessment>,
    pub supporting_evidence_ids: Vec<String>,
    pub interpretation_boundary: SharedResourceAllocationInterpretationBoundary,
}

#[derive(Debug, Clone, Copy)]
struct MeasurementInterval {
    lower: f64,
    upper: f64,
}

#[derive(Debug, Clone)]
struct IndexedUsage {
    resource_index: usize,
    service_index: usize,
    coefficient: MeasurementInterval,
}

#[derive(Debug, Clone)]
struct AllocationSide {
    coverage: Vec<f64>,
    delivery_rate: Vec<f64>,
    resource_used: Vec<f64>,
    iterations: usize,
}

fn interval(
    measurement: &FunctionMeasurement,
) -> Result<MeasurementInterval, SharedResourceAllocationValidationError> {
    let uncertainty = measurement.quantity.uncertainty_abs.unwrap_or(0.0);
    let lower = (measurement.quantity.value - uncertainty).max(0.0);
    let upper = measurement.quantity.value + uncertainty;
    if !lower.is_finite() || !upper.is_finite() || lower > upper {
        return Err(SharedResourceAllocationValidationError::NonFiniteIntervalBound);
    }
    Ok(MeasurementInterval { lower, upper })
}

fn numeric_tolerance(left: f64, right: f64) -> f64 {
    f64::EPSILON * 256.0 * left.abs().max(right.abs()).max(1.0)
}

fn run_progressive_fill(
    demand_rates: &[f64],
    resource_capacities: &[f64],
    usages: &[IndexedUsage],
    service_count: usize,
    resource_count: usize,
    conservative: bool,
) -> Result<AllocationSide, SharedResourceAllocationValidationError> {
    let mut usages_by_resource = vec![Vec::<&IndexedUsage>::new(); resource_count];
    for usage in usages {
        usages_by_resource[usage.resource_index].push(usage);
    }
    let mut coverage = vec![0.0_f64; service_count];
    let mut active = vec![true; service_count];
    let mut iterations = 0usize;

    while active.iter().any(|is_active| *is_active) {
        iterations = iterations
            .checked_add(1)
            .ok_or(SharedResourceAllocationValidationError::AllocationDidNotConverge)?;
        if iterations > service_count.saturating_add(1) {
            return Err(SharedResourceAllocationValidationError::AllocationDidNotConverge);
        }

        let mut resource_used = vec![0.0_f64; resource_count];
        let mut active_slope = vec![0.0_f64; resource_count];
        for usage in usages {
            let coefficient = if conservative {
                usage.coefficient.upper
            } else {
                usage.coefficient.lower
            };
            let service = usage.service_index;
            let used = coefficient * coverage[service] * demand_rates[service];
            if !used.is_finite() {
                return Err(SharedResourceAllocationValidationError::NonFiniteAllocationResult);
            }
            resource_used[usage.resource_index] += used;
            if active[service] {
                active_slope[usage.resource_index] += coefficient * demand_rates[service];
            }
        }
        if resource_used
            .iter()
            .chain(active_slope.iter())
            .any(|value| !value.is_finite())
        {
            return Err(SharedResourceAllocationValidationError::NonFiniteAllocationResult);
        }

        let mut delta = 1.0_f64;
        let active_count = active.iter().filter(|is_active| **is_active).count();
        if active_count == 0 {
            break;
        }
        for (index, is_active) in active.iter().enumerate() {
            if *is_active {
                delta = delta.min((1.0 - coverage[index]).max(0.0));
            }
        }
        for resource_index in 0..resource_count {
            let slope = active_slope[resource_index];
            if slope <= 0.0 {
                continue;
            }
            let capacity = resource_capacities[resource_index];
            let remaining = capacity - resource_used[resource_index];
            if remaining < -numeric_tolerance(capacity, resource_used[resource_index]) {
                return Err(SharedResourceAllocationValidationError::CapacityConstraintViolated);
            }
            delta = delta.min(remaining.max(0.0) / slope);
        }
        if !delta.is_finite() || delta < 0.0 {
            return Err(SharedResourceAllocationValidationError::NonFiniteAllocationResult);
        }

        for index in 0..service_count {
            if active[index] {
                coverage[index] = (coverage[index] + delta).min(1.0);
            }
        }

        let mut new_used = vec![0.0_f64; resource_count];
        for usage in usages {
            let coefficient = if conservative {
                usage.coefficient.upper
            } else {
                usage.coefficient.lower
            };
            new_used[usage.resource_index] +=
                coefficient * demand_rates[usage.service_index] * coverage[usage.service_index];
        }
        if new_used.iter().any(|value| !value.is_finite()) {
            return Err(SharedResourceAllocationValidationError::NonFiniteAllocationResult);
        }

        let mut froze_service = false;
        for index in 0..service_count {
            if active[index] && coverage[index] >= 1.0 - numeric_tolerance(coverage[index], 1.0) {
                coverage[index] = 1.0;
                active[index] = false;
                froze_service = true;
            }
        }

        for resource_index in 0..resource_count {
            let capacity = resource_capacities[resource_index];
            let remaining = capacity - new_used[resource_index];
            if remaining < -numeric_tolerance(capacity, new_used[resource_index]) {
                return Err(SharedResourceAllocationValidationError::CapacityConstraintViolated);
            }
            if remaining <= numeric_tolerance(capacity, new_used[resource_index]) {
                for usage in &usages_by_resource[resource_index] {
                    let coefficient = if conservative {
                        usage.coefficient.upper
                    } else {
                        usage.coefficient.lower
                    };
                    if active[usage.service_index]
                        && coefficient * demand_rates[usage.service_index] > 0.0
                    {
                        active[usage.service_index] = false;
                        froze_service = true;
                    }
                }
            }
        }

        if !froze_service && active.iter().any(|is_active| *is_active) {
            return Err(SharedResourceAllocationValidationError::AllocationDidNotConverge);
        }
    }

    let delivery_rate = coverage
        .iter()
        .zip(demand_rates)
        .map(|(fraction, demand)| fraction * demand)
        .collect::<Vec<_>>();
    let mut resource_used = vec![0.0_f64; resource_count];
    for usage in usages {
        let coefficient = if conservative {
            usage.coefficient.upper
        } else {
            usage.coefficient.lower
        };
        resource_used[usage.resource_index] += coefficient * delivery_rate[usage.service_index];
    }
    if delivery_rate
        .iter()
        .chain(resource_used.iter())
        .any(|value| !value.is_finite())
    {
        return Err(SharedResourceAllocationValidationError::NonFiniteAllocationResult);
    }
    for index in 0..resource_count {
        if resource_used[index]
            > resource_capacities[index]
                + numeric_tolerance(resource_used[index], resource_capacities[index])
        {
            return Err(SharedResourceAllocationValidationError::CapacityConstraintViolated);
        }
    }

    Ok(AllocationSide {
        coverage,
        delivery_rate,
        resource_used,
        iterations,
    })
}

fn classify_floor(lower: f64, upper: f64, floor: f64) -> ServiceFloorStatus {
    let tolerance = numeric_tolerance(lower.max(upper), floor);
    if lower + tolerance >= floor {
        ServiceFloorStatus::MeetsFloor
    } else if upper + tolerance < floor {
        ServiceFloorStatus::BelowFloor
    } else {
        ServiceFloorStatus::Indeterminate
    }
}

#[derive(Debug, Default)]
struct CommunityAccumulator {
    service_count: usize,
    total_weight: u64,
    meets_weight: u64,
    below_weight: u64,
    indeterminate_weight: u64,
}

/// Allocates one declared set of capacities across all connected service cells. The asset
/// table is unique by resource ID; all services that consume that resource share the same
/// inequality. The output is conditional on the supplied resource-usage coefficients and
/// equal-coverage progressive-fill policy.
///
/// Coverage intervals describe policy outcomes at uncertainty endpoints. They are not
/// bounds over every possible allocation policy and do not imply that the declared usage
/// coefficients reproduce hydraulic, electrical, or network physics.
pub fn assess_shared_resource_allocation(
    request: &SharedResourceAllocationRequest,
    evidence_records: &[EvidenceRecord],
) -> Result<SharedResourceAllocationAssessment, SharedResourceAllocationError> {
    let mut errors = Vec::new();
    if request.schema_version != SHARED_RESOURCE_ALLOCATION_SCHEMA_VERSION {
        errors.push(SharedResourceAllocationValidationError::UnsupportedSchemaVersion);
    }
    if request.id.trim().is_empty() {
        errors.push(SharedResourceAllocationValidationError::EmptyRequestId);
    }
    if !is_canonical_utc_timestamp(&request.as_of) {
        errors.push(SharedResourceAllocationValidationError::InvalidAsOfTimestamp);
    }
    if request.services.is_empty() {
        errors.push(SharedResourceAllocationValidationError::EmptyServiceSet);
    }
    if request.resources.is_empty() {
        errors.push(SharedResourceAllocationValidationError::EmptyResourceSet);
    }
    if request.usages.is_empty() {
        errors.push(SharedResourceAllocationValidationError::EmptyUsageSet);
    }
    if request.services.len() > MAX_SHARED_RESOURCE_SERVICES {
        errors.push(SharedResourceAllocationValidationError::ServiceCountExceeded);
    }
    if request.resources.len() > MAX_SHARED_RESOURCE_ASSETS {
        errors.push(SharedResourceAllocationValidationError::ResourceCountExceeded);
    }
    if request.usages.len() > MAX_SHARED_RESOURCE_USAGES {
        errors.push(SharedResourceAllocationValidationError::UsageCountExceeded);
    }
    if request.max_work_units == 0 || request.max_work_units > MAX_SHARED_RESOURCE_WORK_UNITS {
        errors.push(SharedResourceAllocationValidationError::WorkBudgetInvalid);
    }
    let estimated_work_units = request
        .services
        .len()
        .checked_add(1)
        .and_then(|iterations| iterations.checked_mul(request.usages.len()))
        .and_then(|passes| passes.checked_mul(5))
        .ok_or_else(|| {
            SharedResourceAllocationError::InvalidInput(vec![
                SharedResourceAllocationValidationError::WorkEstimateOverflow,
            ])
        })?;
    if estimated_work_units > request.max_work_units
        || estimated_work_units > MAX_SHARED_RESOURCE_WORK_UNITS
    {
        errors.push(SharedResourceAllocationValidationError::WorkBudgetExceeded);
    }

    let mut service_by_id = BTreeMap::new();
    let mut service_entities = BTreeSet::new();
    for service in &request.services {
        if service.id.trim().is_empty() {
            errors.push(SharedResourceAllocationValidationError::EmptyServiceId);
        } else if service_by_id.insert(service.id.as_str(), service).is_some() {
            errors.push(SharedResourceAllocationValidationError::DuplicateServiceId);
        }
        if service.service_entity_id.trim().is_empty() {
            errors.push(SharedResourceAllocationValidationError::EmptyServiceEntityId);
        } else if !service_entities.insert(service.service_entity_id.as_str()) {
            errors.push(SharedResourceAllocationValidationError::DuplicateServiceEntityId);
        }
        if service.community_id.trim().is_empty() {
            errors.push(SharedResourceAllocationValidationError::EmptyCommunityId);
        }
        if service.beneficiary_weight == 0 {
            errors.push(SharedResourceAllocationValidationError::ZeroBeneficiaryWeight);
        }
        if service.delivery_rate_unit.trim().is_empty() {
            errors.push(SharedResourceAllocationValidationError::EmptyDeliveryRateUnit);
        }
        if service.demand_rate.entity_id != service.service_entity_id {
            errors.push(SharedResourceAllocationValidationError::DemandEntityMismatch);
        }
        if service.demand_rate.quantity.unit != service.delivery_rate_unit {
            errors.push(SharedResourceAllocationValidationError::DemandUnitMismatch);
        }
        if !service.essential_coverage_floor.is_finite()
            || !(0.0..=1.0).contains(&service.essential_coverage_floor)
        {
            errors.push(SharedResourceAllocationValidationError::InvalidCoverageFloor);
        }
    }

    let mut resource_by_id = BTreeMap::new();
    for resource in &request.resources {
        if resource.id.trim().is_empty() {
            errors.push(SharedResourceAllocationValidationError::EmptyResourceId);
        } else if resource_by_id
            .insert(resource.id.as_str(), resource)
            .is_some()
        {
            errors.push(SharedResourceAllocationValidationError::DuplicateResourceId);
        }
        if resource.capacity.entity_id != resource.id {
            errors.push(SharedResourceAllocationValidationError::ResourceCapacityEntityMismatch);
        }
    }

    let mut usage_ids = BTreeSet::new();
    let mut service_resource_pairs = BTreeSet::new();
    let mut service_usage_counts = BTreeMap::<&str, usize>::new();
    let mut resource_usage_counts = BTreeMap::<&str, usize>::new();
    for usage in &request.usages {
        if usage.id.trim().is_empty() {
            errors.push(SharedResourceAllocationValidationError::EmptyUsageId);
        } else if !usage_ids.insert(usage.id.as_str()) {
            errors.push(SharedResourceAllocationValidationError::DuplicateUsageId);
        }
        if !resource_by_id.contains_key(usage.resource_id.as_str()) {
            errors.push(SharedResourceAllocationValidationError::UnknownUsageResource);
        }
        if !service_by_id.contains_key(usage.service_id.as_str()) {
            errors.push(SharedResourceAllocationValidationError::UnknownUsageService);
        }
        if !service_resource_pairs.insert((usage.service_id.as_str(), usage.resource_id.as_str())) {
            errors.push(SharedResourceAllocationValidationError::DuplicateServiceResourceUsage);
        }
        *service_usage_counts
            .entry(usage.service_id.as_str())
            .or_default() += 1;
        *resource_usage_counts
            .entry(usage.resource_id.as_str())
            .or_default() += 1;
        if usage.resource_use_per_delivery_rate.entity_id != usage.id {
            errors.push(SharedResourceAllocationValidationError::UsageCoefficientEntityMismatch);
        }
        if let (Some(resource), Some(service)) = (
            resource_by_id.get(usage.resource_id.as_str()),
            service_by_id.get(usage.service_id.as_str()),
        ) {
            let expected = format!(
                "{} per ({})",
                resource.capacity.quantity.unit, service.delivery_rate_unit
            );
            if usage.resource_use_per_delivery_rate.quantity.unit != expected {
                errors.push(SharedResourceAllocationValidationError::UsageCoefficientUnitMismatch);
            }
        }
    }
    for service in &request.services {
        if service_usage_counts
            .get(service.id.as_str())
            .copied()
            .unwrap_or(0)
            == 0
        {
            errors.push(SharedResourceAllocationValidationError::ServiceWithoutResourceConstraint);
        }
    }
    for resource in &request.resources {
        if resource_usage_counts
            .get(resource.id.as_str())
            .copied()
            .unwrap_or(0)
            == 0
        {
            errors.push(SharedResourceAllocationValidationError::ResourceWithoutServiceConstraint);
        }
    }
    let delivery_units = request
        .services
        .iter()
        .map(|service| service.delivery_rate_unit.as_str())
        .collect::<BTreeSet<_>>();
    if delivery_units.len() > 1 {
        errors.push(SharedResourceAllocationValidationError::DeliveryRateUnitMismatch);
    }
    if !errors.is_empty() {
        errors.sort();
        errors.dedup();
        return Err(SharedResourceAllocationError::InvalidInput(errors));
    }

    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(SharedResourceAllocationError::EvidenceValidation)?;
    let mut measurement_errors = Vec::new();
    for service in &request.services {
        validate_measurement(
            &service.demand_rate,
            &request.as_of,
            &evidence_by_id,
            &mut measurement_errors,
        );
    }
    for resource in &request.resources {
        validate_measurement(
            &resource.capacity,
            &request.as_of,
            &evidence_by_id,
            &mut measurement_errors,
        );
    }
    for usage in &request.usages {
        validate_measurement(
            &usage.resource_use_per_delivery_rate,
            &request.as_of,
            &evidence_by_id,
            &mut measurement_errors,
        );
    }
    if !measurement_errors.is_empty() {
        measurement_errors.sort_by_key(|error| format!("{error:?}"));
        measurement_errors.dedup();
        return Err(SharedResourceAllocationError::MeasurementValidation(
            measurement_errors,
        ));
    }

    let mut services: Vec<&SharedResourceServiceDemand> = request.services.iter().collect();
    services.sort_by(|a, b| a.id.cmp(&b.id));
    let mut resources: Vec<&SharedResourceCapacity> = request.resources.iter().collect();
    resources.sort_by(|a, b| a.id.cmp(&b.id));
    let service_indices = services
        .iter()
        .enumerate()
        .map(|(index, service)| (service.id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let resource_indices = resources
        .iter()
        .enumerate()
        .map(|(index, resource)| (resource.id.as_str(), index))
        .collect::<BTreeMap<_, _>>();

    let mut demand_intervals = Vec::with_capacity(services.len());
    let mut all_evidence = BTreeSet::new();
    let mut service_evidence = BTreeMap::<String, BTreeSet<String>>::new();
    let mut resource_evidence = BTreeMap::<String, BTreeSet<String>>::new();
    for service in &services {
        let range = interval(&service.demand_rate)
            .map_err(|error| SharedResourceAllocationError::InvalidInput(vec![error]))?;
        if range.lower <= 0.0 {
            return Err(SharedResourceAllocationError::InvalidInput(vec![
                SharedResourceAllocationValidationError::DemandLowerBoundNonPositive,
            ]));
        }
        demand_intervals.push(range);
        service_evidence
            .entry(service.id.clone())
            .or_default()
            .extend(service.demand_rate.supporting_evidence_ids.iter().cloned());
        all_evidence.extend(service.demand_rate.supporting_evidence_ids.iter().cloned());
    }

    let mut resource_intervals = Vec::with_capacity(resources.len());
    for resource in &resources {
        let range = interval(&resource.capacity)
            .map_err(|error| SharedResourceAllocationError::InvalidInput(vec![error]))?;
        resource_intervals.push(range);
        resource_evidence
            .entry(resource.id.clone())
            .or_default()
            .extend(resource.capacity.supporting_evidence_ids.iter().cloned());
        all_evidence.extend(resource.capacity.supporting_evidence_ids.iter().cloned());
    }

    let mut ordered_usages: Vec<&SharedResourceUsage> = request.usages.iter().collect();
    ordered_usages.sort_by(|a, b| {
        (&a.resource_id, &a.service_id, &a.id).cmp(&(&b.resource_id, &b.service_id, &b.id))
    });
    let mut indexed_usages = Vec::with_capacity(ordered_usages.len());
    for usage in ordered_usages {
        let coefficient = interval(&usage.resource_use_per_delivery_rate)
            .map_err(|error| SharedResourceAllocationError::InvalidInput(vec![error]))?;
        if usage.resource_use_per_delivery_rate.quantity.value <= 0.0 || coefficient.upper <= 0.0 {
            return Err(SharedResourceAllocationError::InvalidInput(vec![
                SharedResourceAllocationValidationError::NonPositiveResourceUseCoefficient,
            ]));
        }
        let resource_index = *resource_indices
            .get(usage.resource_id.as_str())
            .expect("validated resource reference");
        let service_index = *service_indices
            .get(usage.service_id.as_str())
            .expect("validated service reference");
        indexed_usages.push(IndexedUsage {
            resource_index,
            service_index,
            coefficient,
        });
        let refs = usage
            .resource_use_per_delivery_rate
            .supporting_evidence_ids
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        service_evidence
            .entry(usage.service_id.clone())
            .or_default()
            .extend(refs.iter().cloned());
        resource_evidence
            .entry(usage.resource_id.clone())
            .or_default()
            .extend(refs.iter().cloned());
        service_evidence
            .entry(usage.service_id.clone())
            .or_default()
            .extend(
                resources[resource_index]
                    .capacity
                    .supporting_evidence_ids
                    .iter()
                    .cloned(),
            );
        all_evidence.extend(refs);
    }

    let conservative_demands = demand_intervals
        .iter()
        .map(|range| range.upper)
        .collect::<Vec<_>>();
    let optimistic_demands = demand_intervals
        .iter()
        .map(|range| range.lower)
        .collect::<Vec<_>>();
    let conservative_capacities = resource_intervals
        .iter()
        .map(|range| range.lower)
        .collect::<Vec<_>>();
    let optimistic_capacities = resource_intervals
        .iter()
        .map(|range| range.upper)
        .collect::<Vec<_>>();
    let conservative = run_progressive_fill(
        &conservative_demands,
        &conservative_capacities,
        &indexed_usages,
        services.len(),
        resources.len(),
        true,
    )
    .map_err(|error| SharedResourceAllocationError::InvalidInput(vec![error]))?;
    let optimistic = run_progressive_fill(
        &optimistic_demands,
        &optimistic_capacities,
        &indexed_usages,
        services.len(),
        resources.len(),
        false,
    )
    .map_err(|error| SharedResourceAllocationError::InvalidInput(vec![error]))?;

    let mut service_assessments = Vec::with_capacity(services.len());
    let mut total_weight = 0_u64;
    let mut meets_weight = 0_u64;
    let mut below_weight = 0_u64;
    let mut indeterminate_weight = 0_u64;
    let mut community_acc = BTreeMap::<String, CommunityAccumulator>::new();
    for (index, service) in services.iter().enumerate() {
        let lower = conservative.coverage[index].clamp(0.0, 1.0);
        let upper = optimistic.coverage[index].clamp(0.0, 1.0);
        if lower > upper + numeric_tolerance(lower, upper) {
            return Err(SharedResourceAllocationError::InvalidInput(vec![
                SharedResourceAllocationValidationError::InconsistentCoverageBounds,
            ]));
        }
        let status = classify_floor(lower, upper, service.essential_coverage_floor);
        total_weight = total_weight
            .checked_add(service.beneficiary_weight)
            .ok_or_else(|| {
                SharedResourceAllocationError::InvalidInput(vec![
                    SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
                ])
            })?;
        let community = community_acc
            .entry(service.community_id.clone())
            .or_default();
        community.service_count += 1;
        community.total_weight = community
            .total_weight
            .checked_add(service.beneficiary_weight)
            .ok_or_else(|| {
                SharedResourceAllocationError::InvalidInput(vec![
                    SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
                ])
            })?;
        match status {
            ServiceFloorStatus::MeetsFloor => {
                meets_weight = meets_weight
                    .checked_add(service.beneficiary_weight)
                    .ok_or_else(|| {
                        SharedResourceAllocationError::InvalidInput(vec![
                            SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
                        ])
                    })?;
                community.meets_weight = community
                    .meets_weight
                    .checked_add(service.beneficiary_weight)
                    .ok_or_else(|| {
                        SharedResourceAllocationError::InvalidInput(vec![
                            SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
                        ])
                    })?;
            }
            ServiceFloorStatus::BelowFloor => {
                below_weight = below_weight
                    .checked_add(service.beneficiary_weight)
                    .ok_or_else(|| {
                        SharedResourceAllocationError::InvalidInput(vec![
                            SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
                        ])
                    })?;
                community.below_weight = community
                    .below_weight
                    .checked_add(service.beneficiary_weight)
                    .ok_or_else(|| {
                        SharedResourceAllocationError::InvalidInput(vec![
                            SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
                        ])
                    })?;
            }
            ServiceFloorStatus::Indeterminate => {
                indeterminate_weight = indeterminate_weight
                    .checked_add(service.beneficiary_weight)
                    .ok_or_else(|| {
                        SharedResourceAllocationError::InvalidInput(vec![
                            SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
                        ])
                    })?;
                community.indeterminate_weight = community
                    .indeterminate_weight
                    .checked_add(service.beneficiary_weight)
                    .ok_or_else(|| {
                        SharedResourceAllocationError::InvalidInput(vec![
                            SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
                        ])
                    })?;
            }
        }
        service_assessments.push(SharedResourceServiceAssessment {
            service_id: service.id.clone(),
            service_entity_id: service.service_entity_id.clone(),
            community_id: service.community_id.clone(),
            beneficiary_weight: service.beneficiary_weight,
            delivery_rate_unit: service.delivery_rate_unit.clone(),
            demand_rate_lower_bound: demand_intervals[index].lower,
            demand_rate_upper_bound: demand_intervals[index].upper,
            conservative_allocated_delivery_rate: conservative.delivery_rate[index],
            optimistic_allocated_delivery_rate: optimistic.delivery_rate[index],
            coverage_lower_bound: lower,
            coverage_upper_bound: upper,
            essential_coverage_floor: service.essential_coverage_floor,
            floor_status: status,
            supporting_evidence_ids: service_evidence
                .get(service.id.as_str())
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect(),
        });
    }
    if total_weight == 0 || meets_weight + below_weight + indeterminate_weight != total_weight {
        return Err(SharedResourceAllocationError::InvalidInput(vec![
            SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
        ]));
    }

    let mut resource_assessments = Vec::with_capacity(resources.len());
    for (index, resource) in resources.iter().enumerate() {
        let lower = resource_intervals[index].lower;
        let upper = resource_intervals[index].upper;
        let conservative_used = conservative.resource_used[index];
        let optimistic_used = optimistic.resource_used[index];
        let conservative_residual = lower - conservative_used;
        let optimistic_residual = upper - optimistic_used;
        if conservative_residual < -numeric_tolerance(lower, conservative_used)
            || optimistic_residual < -numeric_tolerance(upper, optimistic_used)
        {
            return Err(SharedResourceAllocationError::InvalidInput(vec![
                SharedResourceAllocationValidationError::CapacityConstraintViolated,
            ]));
        }
        resource_assessments.push(SharedResourceAssessment {
            resource_id: resource.id.clone(),
            capacity_unit: resource.capacity.quantity.unit.clone(),
            capacity_lower_bound: lower,
            capacity_upper_bound: upper,
            conservative_allocated_capacity: conservative_used,
            conservative_residual_capacity: conservative_residual.max(0.0),
            conservative_constraint_binding: conservative_residual
                <= numeric_tolerance(lower, conservative_used)
                && conservative_used > 0.0,
            optimistic_allocated_capacity: optimistic_used,
            optimistic_residual_capacity: optimistic_residual.max(0.0),
            optimistic_constraint_binding: optimistic_residual
                <= numeric_tolerance(upper, optimistic_used)
                && optimistic_used > 0.0,
            supporting_evidence_ids: resource_evidence
                .get(resource.id.as_str())
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect(),
        });
    }

    let mut community_assessments = Vec::with_capacity(community_acc.len());
    for (community_id, aggregate) in community_acc {
        if aggregate.total_weight == 0
            || aggregate.meets_weight + aggregate.below_weight + aggregate.indeterminate_weight
                != aggregate.total_weight
        {
            return Err(SharedResourceAllocationError::InvalidInput(vec![
                SharedResourceAllocationValidationError::BeneficiaryWeightOverflow,
            ]));
        }
        community_assessments.push(SharedResourceCommunityAssessment {
            community_id,
            service_count: aggregate.service_count,
            beneficiary_weight: aggregate.total_weight,
            floor_compliance_lower_bound: aggregate.meets_weight as f64
                / aggregate.total_weight as f64,
            floor_compliance_upper_bound: (aggregate.meets_weight + aggregate.indeterminate_weight)
                as f64
                / aggregate.total_weight as f64,
            meets_floor_weight: aggregate.meets_weight,
            below_floor_weight: aggregate.below_weight,
            indeterminate_floor_weight: aggregate.indeterminate_weight,
        });
    }
    community_assessments.sort_by(|a, b| a.community_id.cmp(&b.community_id));

    Ok(SharedResourceAllocationAssessment {
        schema_version: SHARED_RESOURCE_ALLOCATION_SCHEMA_VERSION.to_owned(),
        request_id: request.id.clone(),
        as_of: request.as_of.clone(),
        policy: request.policy,
        service_count: services.len(),
        resource_count: resources.len(),
        usage_constraint_count: indexed_usages.len(),
        estimated_work_units,
        conservative_allocation_iterations: conservative.iterations,
        optimistic_allocation_iterations: optimistic.iterations,
        total_beneficiary_weight: total_weight,
        meets_floor_weight: meets_weight,
        below_floor_weight: below_weight,
        indeterminate_floor_weight: indeterminate_weight,
        floor_compliance_lower_bound: meets_weight as f64 / total_weight as f64,
        floor_compliance_upper_bound: (meets_weight + indeterminate_weight) as f64 / total_weight as f64,
        services: service_assessments,
        resources: resource_assessments,
        communities: community_assessments,
        supporting_evidence_ids: all_evidence.into_iter().collect(),
        interpretation_boundary: SharedResourceAllocationInterpretationBoundary::EqualCoverageAllocationNotPhysicalNetworkSimulation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceRecord, EvidenceState, Provenance,
        SpatialContext, TimeContext,
    };

    const AS_OF: &str = "2026-10-16T00:00:00Z";
    const RATE_UNIT: &str = "m3/h";

    fn measurement(
        entity: &str,
        value: f64,
        uncertainty: Option<f64>,
        unit: &str,
    ) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity.to_owned(),
            quantity: Quantity {
                value,
                unit: unit.to_owned(),
                uncertainty_abs: uncertainty,
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic allocation assumption; not physical evidence.".to_owned(),
            supporting_evidence_ids: Vec::new(),
        }
    }
    fn service(id: &str, community: &str, weight: u64, floor: f64) -> SharedResourceServiceDemand {
        let entity = format!("service-entity-{id}");
        SharedResourceServiceDemand {
            id: id.to_owned(),
            service_entity_id: entity.clone(),
            community_id: community.to_owned(),
            beneficiary_weight: weight,
            delivery_rate_unit: RATE_UNIT.to_owned(),
            demand_rate: measurement(&entity, 100.0, None, RATE_UNIT),
            essential_coverage_floor: floor,
        }
    }
    fn resource(
        id: &str,
        value: f64,
        uncertainty: Option<f64>,
        unit: &str,
    ) -> SharedResourceCapacity {
        SharedResourceCapacity {
            id: id.to_owned(),
            capacity: measurement(id, value, uncertainty, unit),
        }
    }
    fn usage(
        id: &str,
        resource_id: &str,
        service_id: &str,
        value: f64,
        unit: &str,
    ) -> SharedResourceUsage {
        SharedResourceUsage {
            id: id.to_owned(),
            resource_id: resource_id.to_owned(),
            service_id: service_id.to_owned(),
            resource_use_per_delivery_rate: measurement(id, value, None, unit),
        }
    }
    fn observed_evidence(id: &str, entity: &str, quantity: Quantity) -> EvidenceRecord {
        EvidenceRecord {
            id: id.to_owned(),
            schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:shared-resource-tests".to_owned(),
            subject_id: entity.to_owned(),
            spatial: SpatialContext {
                entity_id: entity.to_owned(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: AS_OF.to_owned(),
                recorded_at: AS_OF.to_owned(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic observed capacity for provenance test.".to_owned(),
            quantity: Some(quantity),
            money: None,
            provenance: Provenance {
                method: "synthetic_test".to_owned(),
                source_artifacts: Vec::new(),
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:synthetic-test".to_owned(),
        }
    }

    fn request() -> SharedResourceAllocationRequest {
        SharedResourceAllocationRequest {
            schema_version: SHARED_RESOURCE_ALLOCATION_SCHEMA_VERSION.to_owned(),
            id: "shared-water-power-allocation".to_owned(),
            as_of: AS_OF.to_owned(),
            policy: SharedResourceAllocationPolicy::EqualCoverageProgressiveFill,
            services: vec![
                service("service-a", "community-a", 60, 0.3),
                service("service-b", "community-b", 40, 0.8),
            ],
            resources: vec![
                resource("shared-water", 100.0, None, "m3/h"),
                resource("shared-pump-power", 40.0, None, "kW"),
            ],
            usages: vec![
                usage(
                    "water-a",
                    "shared-water",
                    "service-a",
                    1.0,
                    "m3/h per (m3/h)",
                ),
                usage(
                    "water-b",
                    "shared-water",
                    "service-b",
                    1.0,
                    "m3/h per (m3/h)",
                ),
                usage(
                    "power-a",
                    "shared-pump-power",
                    "service-a",
                    0.5,
                    "kW per (m3/h)",
                ),
                usage(
                    "power-b",
                    "shared-pump-power",
                    "service-b",
                    0.5,
                    "kW per (m3/h)",
                ),
            ],
            max_work_units: MAX_SHARED_RESOURCE_WORK_UNITS,
        }
    }

    #[test]
    fn shared_resource_is_allocated_once_across_service_cells() {
        let output =
            assess_shared_resource_allocation(&request(), &[]).expect("valid shared allocation");
        assert_eq!(output.service_count, 2);
        assert_eq!(output.resource_count, 2);
        assert_eq!(output.usage_constraint_count, 4);
        assert!((output.services[0].coverage_lower_bound - 0.4).abs() < 1e-9);
        assert!((output.services[1].coverage_lower_bound - 0.4).abs() < 1e-9);
        let water = output
            .resources
            .iter()
            .find(|r| r.resource_id == "shared-water")
            .unwrap();
        assert!((water.conservative_allocated_capacity - 80.0).abs() < 1e-9);
        assert!((water.conservative_residual_capacity - 20.0).abs() < 1e-9);
        let power = output
            .resources
            .iter()
            .find(|r| r.resource_id == "shared-pump-power")
            .unwrap();
        assert!((power.conservative_allocated_capacity - 40.0).abs() < 1e-9);
        assert!(power.conservative_constraint_binding);
    }

    #[test]
    fn protected_floor_outcomes_are_distributionally_reconciled() {
        let output =
            assess_shared_resource_allocation(&request(), &[]).expect("valid shared allocation");
        assert_eq!(output.meets_floor_weight, 60);
        assert_eq!(output.below_floor_weight, 40);
        assert_eq!(output.indeterminate_floor_weight, 0);
        assert!((output.floor_compliance_lower_bound - 0.6).abs() < 1e-9);
        assert_eq!(
            output.services[0].floor_status,
            ServiceFloorStatus::MeetsFloor
        );
        assert_eq!(
            output.services[1].floor_status,
            ServiceFloorStatus::BelowFloor
        );
    }

    #[test]
    fn uncertainty_produces_coverage_interval_and_indeterminate_floor() {
        let mut input = request();
        input.resources[1].capacity.quantity.uncertainty_abs = Some(10.0);
        input.services[0].essential_coverage_floor = 0.4;
        input.services[1].essential_coverage_floor = 0.4;
        let output =
            assess_shared_resource_allocation(&input, &[]).expect("valid uncertain allocation");
        for service in output.services {
            assert!((service.coverage_lower_bound - 0.3).abs() < 1e-9);
            assert!((service.coverage_upper_bound - 0.5).abs() < 1e-9);
            assert_eq!(service.floor_status, ServiceFloorStatus::Indeterminate);
        }
        assert_eq!(output.indeterminate_floor_weight, 100);
    }

    #[test]
    fn independent_resources_allow_full_coverage() {
        let mut input = request();
        input.resources = vec![
            resource("water-a", 100.0, None, "m3/h"),
            resource("water-b", 100.0, None, "m3/h"),
        ];
        input.usages = vec![
            usage("use-a", "water-a", "service-a", 1.0, "m3/h per (m3/h)"),
            usage("use-b", "water-b", "service-b", 1.0, "m3/h per (m3/h)"),
        ];
        let output =
            assess_shared_resource_allocation(&input, &[]).expect("valid independent resources");
        assert!(
            output
                .services
                .iter()
                .all(|service| (service.coverage_lower_bound - 1.0).abs() < 1e-9)
        );
    }

    #[test]
    fn duplicate_physical_resource_ids_fail_closed() {
        let mut input = request();
        input.resources.push(input.resources[0].clone());
        assert!(matches!(assess_shared_resource_allocation(&input, &[]),
            Err(SharedResourceAllocationError::InvalidInput(errors))
                if errors.contains(&SharedResourceAllocationValidationError::DuplicateResourceId)));
    }

    #[test]
    fn coefficient_unit_mismatch_fails_closed() {
        let mut input = request();
        input.usages[0].resource_use_per_delivery_rate.quantity.unit = "kW per (m3/h)".to_owned();
        assert!(matches!(assess_shared_resource_allocation(&input, &[]),
            Err(SharedResourceAllocationError::InvalidInput(errors))
                if errors.contains(&SharedResourceAllocationValidationError::UsageCoefficientUnitMismatch)));
    }

    #[test]
    fn over_budget_requests_fail_closed() {
        let mut input = request();
        input.max_work_units = 1;
        assert!(matches!(assess_shared_resource_allocation(&input, &[]),
            Err(SharedResourceAllocationError::InvalidInput(errors))
                if errors.contains(&SharedResourceAllocationValidationError::WorkBudgetExceeded)));
    }

    #[test]
    fn input_permutation_produces_identical_assessment() {
        let input = request();
        let mut permuted = request();
        permuted.services.reverse();
        permuted.resources.reverse();
        permuted.usages.reverse();
        let left = assess_shared_resource_allocation(&input, &[]).expect("valid input");
        let right =
            assess_shared_resource_allocation(&permuted, &[]).expect("valid permuted input");
        assert_eq!(left, right);
        assert_eq!(
            serde_json::to_vec(&left).expect("serialize left"),
            serde_json::to_vec(&right).expect("serialize right")
        );
    }

    #[test]
    fn observed_measurements_require_matching_evidence() {
        let mut input = request();
        input.services[0].demand_rate.basis = ScenarioBasis::Observed;
        assert!(matches!(assess_shared_resource_allocation(&input, &[]),
            Err(SharedResourceAllocationError::MeasurementValidation(errors))
                if errors.contains(&ServiceFunctionValidationError::MissingEvidenceForObservedMeasurement)));
    }

    #[test]
    fn saturated_resource_freezes_its_consumers_but_not_unrelated_services() {
        let mut input = request();
        input.services = vec![
            service("service-a", "community-a", 1, 0.4),
            service("service-b", "community-b", 1, 0.4),
            service("service-c", "community-c", 1, 0.8),
        ];
        input.resources = vec![
            resource("shared-limited", 100.0, None, "m3/h"),
            resource("independent", 100.0, None, "m3/h"),
        ];
        input.usages = vec![
            usage(
                "limited-a",
                "shared-limited",
                "service-a",
                1.0,
                "m3/h per (m3/h)",
            ),
            usage(
                "limited-b",
                "shared-limited",
                "service-b",
                1.0,
                "m3/h per (m3/h)",
            ),
            usage(
                "independent-c",
                "independent",
                "service-c",
                1.0,
                "m3/h per (m3/h)",
            ),
        ];
        let output =
            assess_shared_resource_allocation(&input, &[]).expect("valid mixed constraints");
        let coverage = output
            .services
            .iter()
            .map(|service| (service.service_id.as_str(), service.coverage_lower_bound))
            .collect::<BTreeMap<_, _>>();
        assert!((coverage["service-a"] - 0.5).abs() < 1e-9);
        assert!((coverage["service-b"] - 0.5).abs() < 1e-9);
        assert!((coverage["service-c"] - 1.0).abs() < 1e-9);
        assert!(output.conservative_allocation_iterations >= 2);
    }

    #[test]
    fn progressive_fill_matches_small_discrete_lexicographic_fairness_oracle() {
        let mut input = request();
        input.services = vec![
            service("service-a", "community-a", 1, 0.0),
            service("service-b", "community-b", 1, 0.0),
            service("service-c", "community-c", 1, 0.0),
        ];
        input.resources = vec![
            resource("shared-limited", 100.0, None, "m3/h"),
            resource("independent", 100.0, None, "m3/h"),
        ];
        input.usages = vec![
            usage(
                "limited-a",
                "shared-limited",
                "service-a",
                1.0,
                "m3/h per (m3/h)",
            ),
            usage(
                "limited-b",
                "shared-limited",
                "service-b",
                1.0,
                "m3/h per (m3/h)",
            ),
            usage(
                "independent-c",
                "independent",
                "service-c",
                1.0,
                "m3/h per (m3/h)",
            ),
        ];
        let output = assess_shared_resource_allocation(&input, &[]).expect("valid oracle fixture");
        let observed = output
            .services
            .iter()
            .map(|service| service.coverage_lower_bound)
            .collect::<Vec<_>>();
        assert!((observed[0] - 0.5).abs() < 1e-9);
        assert!((observed[1] - 0.5).abs() < 1e-9);
        assert!((observed[2] - 1.0).abs() < 1e-9);

        // Enumerate a discrete 0.1-coverage grid as an independent small-instance oracle.
        // Feasibility: 100*c_a + 100*c_b <= 100; 100*c_c <= 100.
        let mut best = [0.0_f64; 3];
        for a in 0..=10 {
            for b in 0..=10 {
                for c in 0..=10 {
                    let ca = a as f64 / 10.0;
                    let cb = b as f64 / 10.0;
                    let cc = c as f64 / 10.0;
                    if 100.0 * ca + 100.0 * cb > 100.0 + 1e-9 || 100.0 * cc > 100.0 + 1e-9 {
                        continue;
                    }
                    let mut candidate = [ca, cb, cc];
                    candidate.sort_by(f64::total_cmp);
                    if candidate[0] > best[0]
                        || (candidate[0] == best[0] && candidate[1] > best[1])
                        || (candidate[0] == best[0]
                            && candidate[1] == best[1]
                            && candidate[2] > best[2])
                    {
                        best = candidate;
                    }
                }
            }
        }
        let mut observed_sorted = observed;
        observed_sorted.sort_by(f64::total_cmp);
        assert!((observed_sorted[0] - best[0]).abs() < 1e-9);
        assert!((observed_sorted[1] - best[1]).abs() < 1e-9);
        assert!((observed_sorted[2] - best[2]).abs() < 1e-9);
        assert_eq!(best, [0.5, 0.5, 1.0]);
    }

    #[test]
    fn each_service_provenance_includes_evidence_for_its_shared_resource_capacity() {
        let mut input = request();
        input.resources[0].capacity.basis = ScenarioBasis::Observed;
        input.resources[0].capacity.supporting_evidence_ids =
            vec!["evidence:shared-water".to_owned()];
        let evidence = observed_evidence(
            "evidence:shared-water",
            "shared-water",
            Quantity {
                value: 100.0,
                unit: "m3/h".to_owned(),
                uncertainty_abs: None,
            },
        );
        let output = assess_shared_resource_allocation(&input, &[evidence])
            .expect("observed shared capacity must be admissible");
        assert!(output.services.iter().all(|service| {
            service
                .supporting_evidence_ids
                .iter()
                .any(|id| id == "evidence:shared-water")
        }));
        assert!(output.resources.iter().any(|resource| {
            resource.resource_id == "shared-water"
                && resource
                    .supporting_evidence_ids
                    .iter()
                    .any(|id| id == "evidence:shared-water")
        }));
        assert!(
            output
                .supporting_evidence_ids
                .iter()
                .any(|id| id == "evidence:shared-water")
        );
    }

    #[test]
    fn deterministic_dense_cases_match_closed_form_progressive_fill_oracle() {
        // Independent oracle for complete-bipartite cases: every service consumes every
        // resource, so equal coverage is bounded by min(1, capacity_r / sum_i(demand_i * use_ir)).
        // A deterministic LCG avoids a new dependency and makes failures exactly reproducible.
        let mut seed = 0x4d59_5df4_d0f3_3173_u64;
        let mut next = || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            seed
        };

        for case_index in 0..100 {
            let service_count = 2 + (next() % 7) as usize;
            let resource_count = 1 + (next() % 5) as usize;
            let mut input = request();
            input.id = format!("closed-form-oracle-case-{case_index}");
            input.services.clear();
            input.resources.clear();
            input.usages.clear();

            let mut demands = Vec::with_capacity(service_count);
            for service_index in 0..service_count {
                let demand = (50 + next() % 101) as f64;
                demands.push(demand);
                let mut item = service(
                    &format!("service-{service_index}"),
                    &format!("community-{}", service_index % 3),
                    1 + (next() % 10),
                    0.0,
                );
                item.demand_rate.quantity.value = demand;
                input.services.push(item);
            }

            let mut capacities = Vec::with_capacity(resource_count);
            for resource_index in 0..resource_count {
                let capacity = (500 + next() % 1_501) as f64;
                capacities.push(capacity);
                input.resources.push(resource(
                    &format!("resource-{resource_index}"),
                    capacity,
                    None,
                    "m3/h",
                ));
            }

            let mut coefficients = vec![vec![0.0_f64; resource_count]; service_count];
            for resource_index in 0..resource_count {
                for service_index in 0..service_count {
                    let coefficient = 0.5 + (next() % 101) as f64 / 100.0;
                    coefficients[service_index][resource_index] = coefficient;
                    input.usages.push(usage(
                        &format!("use-{resource_index}-{service_index}"),
                        &format!("resource-{resource_index}"),
                        &format!("service-{service_index}"),
                        coefficient,
                        "m3/h per (m3/h)",
                    ));
                }
            }

            let expected_coverage = (0..resource_count)
                .map(|resource_index| {
                    let total_usage = (0..service_count)
                        .map(|service_index| {
                            demands[service_index] * coefficients[service_index][resource_index]
                        })
                        .sum::<f64>();
                    capacities[resource_index] / total_usage
                })
                .fold(1.0_f64, f64::min);

            let output =
                assess_shared_resource_allocation(&input, &[]).expect("generated case is valid");
            assert_eq!(output.services.len(), service_count, "case {case_index}");
            assert_eq!(output.resources.len(), resource_count, "case {case_index}");
            for service_output in &output.services {
                assert!(
                    (service_output.coverage_lower_bound - expected_coverage).abs() < 1e-8,
                    "case {case_index}, service {}: expected {expected_coverage}, got {}",
                    service_output.service_id,
                    service_output.coverage_lower_bound
                );
                let index = input
                    .services
                    .iter()
                    .position(|service| service.id == service_output.service_id)
                    .expect("generated service exists");
                assert!(
                    (service_output.conservative_allocated_delivery_rate
                        - demands[index] * expected_coverage)
                        .abs()
                        < 1e-7,
                    "case {case_index}, service {} rate",
                    service_output.service_id
                );
            }
            for resource_output in &output.resources {
                assert!(
                    resource_output.conservative_allocated_capacity
                        <= resource_output.capacity_lower_bound + 1e-7,
                    "case {case_index}, resource {} overspent",
                    resource_output.resource_id
                );
            }

            let mut permuted = input.clone();
            permuted.services.reverse();
            permuted.resources.reverse();
            permuted.usages.reverse();
            let permuted_output = assess_shared_resource_allocation(&permuted, &[])
                .expect("permuted generated case is valid");
            assert_eq!(output, permuted_output, "case {case_index} permutation");
        }
    }

    #[test]
    fn deterministic_sparse_graphs_match_exhaustive_leximin_oracle() {
        // Tiny independent feasibility oracle. With three services, unit demands and unit
        // coefficients, capacities that are integer multiples of demand induce progressive-fill
        // breakpoints on a 1/6 grid. Enumerating the 1/30 grid therefore includes every possible
        // breakpoint for these instances while being independent of the production fill routine.
        const GRID: i32 = 30;
        const SERVICE_COUNT: usize = 3;
        const RESOURCE_COUNT: usize = 4;

        let mut seed = 0x243f_6a88_85a3_08d3_u64;
        let mut next = || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            seed
        };

        for case_index in 0..100 {
            let mut input = request();
            input.id = format!("sparse-leximin-oracle-case-{case_index}");
            input.services.clear();
            input.resources.clear();
            input.usages.clear();

            for service_index in 0..SERVICE_COUNT {
                input.services.push(service(
                    &format!("service-{service_index}"),
                    &format!("community-{}", service_index % 2),
                    1,
                    0.0,
                ));
            }

            let mut adjacency = vec![vec![false; SERVICE_COUNT]; RESOURCE_COUNT];
            for resource in 0..RESOURCE_COUNT {
                for service in 0..SERVICE_COUNT {
                    adjacency[resource][service] = next() % 100 < 55;
                }
            }
            // Ensure both sides of the incidence graph are nonempty without making it dense.
            for service in 0..SERVICE_COUNT {
                if !(0..RESOURCE_COUNT).any(|resource| adjacency[resource][service]) {
                    adjacency[service % RESOURCE_COUNT][service] = true;
                }
            }
            for resource in 0..RESOURCE_COUNT {
                if !adjacency[resource].iter().any(|connected| *connected) {
                    adjacency[resource][resource % SERVICE_COUNT] = true;
                }
            }

            let mut capacity_units = Vec::with_capacity(RESOURCE_COUNT);
            for resource_index in 0..RESOURCE_COUNT {
                let consumers = adjacency[resource_index]
                    .iter()
                    .filter(|value| **value)
                    .count();
                let units = 1 + (next() % consumers as u64) as i32;
                capacity_units.push(units);
                input.resources.push(resource(
                    &format!("resource-{resource_index}"),
                    100.0 * units as f64,
                    None,
                    "m3/h",
                ));
                for service_index in 0..SERVICE_COUNT {
                    if adjacency[resource_index][service_index] {
                        input.usages.push(usage(
                            &format!("use-{resource_index}-{service_index}"),
                            &format!("resource-{resource_index}"),
                            &format!("service-{service_index}"),
                            1.0,
                            "m3/h per (m3/h)",
                        ));
                    }
                }
            }

            // Maximize the sorted coverage vector lexicographically over a finite grid. The
            // complete search is feasible here (31^3 points) and does not call the production
            // progressive-fill algorithm or reuse its internal state.
            let mut best = [0_i32; SERVICE_COUNT];
            let mut found_feasible = false;
            for a in 0..=GRID {
                for b in 0..=GRID {
                    for c in 0..=GRID {
                        let candidate = [a, b, c];
                        let feasible = (0..RESOURCE_COUNT).all(|resource| {
                            let used_grid_units = (0..SERVICE_COUNT)
                                .filter(|service| adjacency[resource][*service])
                                .map(|service| candidate[service])
                                .sum::<i32>();
                            used_grid_units <= capacity_units[resource] * GRID
                        });
                        if !feasible {
                            continue;
                        }
                        let mut sorted = candidate;
                        sorted.sort_unstable();
                        if !found_feasible || sorted > best {
                            best = sorted;
                            found_feasible = true;
                        }
                    }
                }
            }
            assert!(
                found_feasible,
                "case {case_index} must have a feasible allocation"
            );

            let output = assess_shared_resource_allocation(&input, &[])
                .expect("generated sparse graph must be valid");
            let mut actual = output
                .services
                .iter()
                .map(|service| (service.coverage_lower_bound * GRID as f64).round() as i32)
                .collect::<Vec<_>>();
            actual.sort_unstable();
            assert_eq!(
                actual, best,
                "case {case_index}: sparse leximin coverage differs; adjacency={adjacency:?}, capacity_units={capacity_units:?}"
            );
            for service in &output.services {
                assert!(
                    (service.coverage_lower_bound * GRID as f64
                        - (service.coverage_lower_bound * GRID as f64).round())
                    .abs()
                        < 1e-7,
                    "case {case_index}: a coverage breakpoint was outside the exact 1/30 grid"
                );
            }
            for resource in &output.resources {
                assert!(
                    resource.conservative_allocated_capacity
                        <= resource.capacity_lower_bound + 1e-7,
                    "case {case_index}: resource {} was overspent",
                    resource.resource_id
                );
            }

            let mut permuted = input.clone();
            permuted.services.reverse();
            permuted.resources.reverse();
            permuted.usages.reverse();
            let permuted_output = assess_shared_resource_allocation(&permuted, &[])
                .expect("permuted generated sparse graph must be valid");
            assert_eq!(output, permuted_output, "case {case_index}: permutation");
        }
    }

    #[test]
    fn deterministic_heterogeneous_sparse_graphs_match_independent_grid_max_min_oracle() {
        // This differential oracle deliberately does not reuse run_progressive_fill or its
        // saturation bookkeeping. It enumerates all three-service coverage vectors on a finite
        // grid, checks the original sparse resource inequalities directly, and maximizes the
        // minimum coverage. Heterogeneous demands and usage coefficients exercise a gap left by
        // the unit-demand/unit-coefficient oracle above. This is a bounded discretized oracle,
        // not a continuous LP oracle or a proof of every later leximin stage.
        const GRID: i32 = 40;
        const SERVICE_COUNT: usize = 3;
        const RESOURCE_COUNT: usize = 4;
        const CASE_COUNT: usize = 32;
        const COEFFICIENTS: [f64; 4] = [0.5, 1.0, 1.5, 2.0];

        let mut seed = 0xb7e1_5162_8aed_2a6b_u64;
        let mut next = || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            seed
        };

        for case_index in 0..CASE_COUNT {
            let mut input = request();
            input.id = format!("heterogeneous-sparse-max-min-case-{case_index}");
            input.services.clear();
            input.resources.clear();
            input.usages.clear();

            let mut demand_values = Vec::with_capacity(SERVICE_COUNT);
            for service_index in 0..SERVICE_COUNT {
                let demand = 50.0 + (next() % 11) as f64 * 10.0;
                demand_values.push(demand);
                let mut item = service(
                    &format!("service-{service_index}"),
                    &format!("community-{}", service_index % 2),
                    1,
                    0.0,
                );
                item.demand_rate = measurement(&item.service_entity_id, demand, None, RATE_UNIT);
                input.services.push(item);
            }

            let mut adjacency = vec![vec![false; SERVICE_COUNT]; RESOURCE_COUNT];
            let mut coefficients = vec![vec![1.0_f64; SERVICE_COUNT]; RESOURCE_COUNT];
            for resource_index in 0..RESOURCE_COUNT {
                for service_index in 0..SERVICE_COUNT {
                    adjacency[resource_index][service_index] = next() % 100 < 55;
                    coefficients[resource_index][service_index] =
                        COEFFICIENTS[(next() % COEFFICIENTS.len() as u64) as usize];
                }
            }

            // Guarantee every service is constrained and every declared resource is used.
            // Keep those repairs explicit so the generated fixture itself remains auditable.
            for service_index in 0..SERVICE_COUNT {
                if !(0..RESOURCE_COUNT).any(|resource| adjacency[resource][service_index]) {
                    adjacency[service_index % RESOURCE_COUNT][service_index] = true;
                }
            }
            for resource_index in 0..RESOURCE_COUNT {
                if !adjacency[resource_index].iter().any(|connected| *connected) {
                    adjacency[resource_index][resource_index % SERVICE_COUNT] = true;
                }
            }

            let mut capacity_values = Vec::with_capacity(RESOURCE_COUNT);
            for resource_index in 0..RESOURCE_COUNT {
                let capacity = 20.0 + (next() % 10) as f64 * 20.0;
                capacity_values.push(capacity);
                input.resources.push(resource(
                    &format!("resource-{resource_index}"),
                    capacity,
                    None,
                    RATE_UNIT,
                ));
                for service_index in 0..SERVICE_COUNT {
                    if adjacency[resource_index][service_index] {
                        input.usages.push(usage(
                            &format!("use-{resource_index}-{service_index}"),
                            &format!("resource-{resource_index}"),
                            &format!("service-{service_index}"),
                            coefficients[resource_index][service_index],
                            "m3/h per (m3/h)",
                        ));
                    }
                }
            }

            // Independent grid oracle for the first (max-min) leximin stage. Each candidate is
            // judged directly against every resource inequality, not by a copy of the production
            // saturation/freeze algorithm.
            let mut best_minimum = 0_i32;
            let mut found_feasible = false;
            for a in 0..=GRID {
                for b in 0..=GRID {
                    for c in 0..=GRID {
                        let candidate = [a, b, c];
                        let feasible = (0..RESOURCE_COUNT).all(|resource_index| {
                            let used = (0..SERVICE_COUNT)
                                .filter(|service_index| adjacency[resource_index][*service_index])
                                .map(|service_index| {
                                    coefficients[resource_index][service_index]
                                        * demand_values[service_index]
                                        * candidate[service_index] as f64
                                        / GRID as f64
                                })
                                .sum::<f64>();
                            used <= capacity_values[resource_index] + 1e-9
                        });
                        if !feasible {
                            continue;
                        }
                        found_feasible = true;
                        best_minimum = best_minimum.max(*candidate.iter().min().unwrap());
                    }
                }
            }
            assert!(
                found_feasible,
                "case {case_index} must have a feasible origin"
            );

            let output = assess_shared_resource_allocation(&input, &[])
                .expect("generated heterogeneous sparse graph must be valid");
            let actual_minimum = output
                .services
                .iter()
                .map(|item| item.coverage_lower_bound)
                .fold(f64::INFINITY, f64::min);
            let oracle_minimum = best_minimum as f64 / GRID as f64;
            assert!(
                (actual_minimum - oracle_minimum).abs() <= 1.0 / GRID as f64 + 1e-6,
                "case {case_index}: max-min coverage differs from the independent grid oracle; actual={actual_minimum}, grid={oracle_minimum}, adjacency={adjacency:?}, demand={demand_values:?}, coefficients={coefficients:?}, capacities={capacity_values:?}"
            );

            for item in &output.resources {
                assert!(
                    item.conservative_allocated_capacity <= item.capacity_lower_bound + 1e-7,
                    "case {case_index}: resource {} was overspent",
                    item.resource_id
                );
            }

            let mut permuted = input.clone();
            permuted.services.reverse();
            permuted.resources.reverse();
            permuted.usages.reverse();
            let permuted_output = assess_shared_resource_allocation(&permuted, &[])
                .expect("permuted heterogeneous sparse graph must be valid");
            assert_eq!(output, permuted_output, "case {case_index}: permutation");
        }
    }
}
