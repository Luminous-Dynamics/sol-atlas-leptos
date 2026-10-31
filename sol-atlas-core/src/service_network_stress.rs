// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Exhaustive bounded link-failure experiments for service-capacity networks.
//!
//! This layer moves from a single nominal flow calculation to explicit N-k stress
//! cases. It enumerates every combination of up to k currently active links, subject
//! to a caller-declared case budget; it never silently samples or truncates cases.
//! Scenario counts are stress-test cases, not failure probabilities.

use crate::dependency_logic::ScenarioBasis;
use crate::service_network::{
    NetworkLinkAvailability, NetworkTopologyCompleteness, ServiceCapacityNetworkAssessment,
    ServiceCapacityNetworkError, ServiceCapacityNetworkScenario, assess_service_capacity_networks,
};
use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SERVICE_NETWORK_STRESS_SCHEMA_VERSION: &str = "0.1.0";
pub const MAX_SIMULTANEOUS_LINK_FAILURES: usize = 3;
pub const MAX_EXHAUSTIVE_FAILURE_CASES: usize = 10_000;
/// Hard cap on (baseline + failure cases) multiplied by modeled link count.
/// This bounds the peak cloned graph footprint, not just the number of scenarios.
pub const MAX_STRESS_CASE_LINK_REFERENCES: usize = 2_000_000;
const FAILURE_TIE_RELATIVE_TOLERANCE: f64 = 1.0e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclaredFailureSetKind {
    CommonCause,
    HazardBundle,
    Other,
}

/// A caller-declared event that jointly disables a named set of network links.
/// The mapping and rationale are explicit assumptions; the evaluator does not
/// infer that the event physically causes these links to fail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeclaredNetworkFailureSet {
    pub id: String,
    pub kind: DeclaredFailureSetKind,
    pub rationale: String,
    pub failed_link_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkStressRequest {
    pub schema_version: String,
    /// Stable experiment ID; used to derive deterministic case IDs.
    pub id: String,
    pub baseline_scenario: ServiceCapacityNetworkScenario,
    /// Enumerate every combination of 1..=k active-link failures.
    pub max_simultaneous_failures: usize,
    /// Hard budget for link-combination plus declared-event cases. Over-budget
    /// inputs fail instead of returning a misleading partial result.
    pub max_failure_cases: usize,
    /// Optional explicit common-cause/hazard bundles, evaluated once each in
    /// addition to the exhaustive single-/double-/triple-link combinations.
    #[serde(default)]
    pub declared_failure_sets: Vec<DeclaredNetworkFailureSet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceNetworkFailureCaseOrigin {
    EnumeratedLinkCombination,
    DeclaredCommonCause,
    DeclaredHazardBundle,
    DeclaredOther,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkFailureCase {
    pub case_id: String,
    pub origin: ServiceNetworkFailureCaseOrigin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_set_id: Option<String>,
    pub rationale: String,
    /// Link IDs forced inactive by this synthetic stress case.
    pub failed_link_ids: Vec<String>,
    pub assessment: ServiceCapacityNetworkAssessment,
    /// Baseline minus post-failure flow, floored at zero.
    pub conservative_capacity_loss: f64,
    pub optimistic_capacity_loss: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkStressAssessment {
    pub schema_version: String,
    pub experiment_id: String,
    pub baseline_scenario_id: String,
    pub service_entity_id: String,
    pub as_of: String,
    pub unit: String,
    /// Maximum order of enumerated link combinations; declared event bundles may
    /// disable more links together but are reported separately by origin.
    pub max_simultaneous_failures: usize,
    pub active_link_count: usize,
    pub declared_failure_set_count: usize,
    /// Exact number of generated link combinations plus declared-event cases. This equals evaluated count
    /// on every successful response: the sweep is exhaustive, not sampled.
    pub candidate_failure_case_count: usize,
    pub evaluated_failure_case_count: usize,
    pub exhaustive: bool,
    pub baseline: ServiceCapacityNetworkAssessment,
    pub worst_case_conservative_capacity: f64,
    pub worst_case_optimistic_capacity: f64,
    pub worst_case_conservative_capacity_loss: f64,
    pub worst_case_optimistic_capacity_loss: f64,
    /// Failure sets whose conservative flow ties the minimum within numeric tolerance, sorted deterministically.
    pub worst_case_conservative_failure_sets: Vec<Vec<String>>,
    /// Failure sets whose optimistic flow ties the minimum within numeric tolerance, sorted deterministically.
    pub worst_case_optimistic_failure_sets: Vec<Vec<String>>,
    /// Enumerated link combinations (failure order then link IDs) come first,
    /// followed by caller-declared event bundles sorted by declared event ID.
    pub cases: Vec<ServiceNetworkFailureCase>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceNetworkStressValidationError {
    UnsupportedSchemaVersion,
    EmptyExperimentId,
    IncompleteTopology,
    InvalidFailureOrder,
    FailureOrderTooLarge,
    InvalidFailureCaseBudget,
    NoActiveLinks,
    ScenarioBudgetExceeded,
    ScenarioMemoryBudgetExceeded,
    EmptyDeclaredFailureSetId,
    DuplicateDeclaredFailureSetId,
    EmptyDeclaredFailureSetRationale,
    EmptyDeclaredFailureSetLinks,
    EmptyDeclaredFailureLinkId,
    DuplicateDeclaredFailureLinkId,
    UnknownDeclaredFailureLinkId,
    AlreadyInactiveDeclaredFailureLinkId,
    DuplicateBaselineLinkId,
    AssessmentCountMismatch,
    NonMonotonicConservativeFlow,
    NonMonotonicOptimisticFlow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceNetworkStressError {
    InvalidInput(Vec<ServiceNetworkStressValidationError>),
    NetworkAssessment(ServiceCapacityNetworkError),
}

#[derive(Debug, Clone)]
struct FailureCaseSpec {
    case_id: String,
    origin: ServiceNetworkFailureCaseOrigin,
    failure_set_id: Option<String>,
    rationale: String,
    failed_link_ids: Vec<String>,
}

/// Enumerate all single-, double-, or triple-link failures within an explicit budget.
///
/// Requires a complete topology. Each forced outage is explicitly tagged as an
/// assumed scenario override and loses any observational availability evidence for
/// the overridden link. Capacity observations are left untouched. A successful
/// output is exhaustive; if the combinations exceed max_failure_cases (or the
/// global safety cap), this function returns an error rather than dropping cases.
pub fn assess_service_network_stress(
    request: &ServiceNetworkStressRequest,
    evidence_records: &[EvidenceRecord],
) -> Result<ServiceNetworkStressAssessment, ServiceNetworkStressError> {
    let mut errors = Vec::new();
    if request.schema_version != SERVICE_NETWORK_STRESS_SCHEMA_VERSION {
        errors.push(ServiceNetworkStressValidationError::UnsupportedSchemaVersion);
    }
    if request.id.trim().is_empty() {
        errors.push(ServiceNetworkStressValidationError::EmptyExperimentId);
    }
    if request.baseline_scenario.topology_completeness != NetworkTopologyCompleteness::Complete {
        errors.push(ServiceNetworkStressValidationError::IncompleteTopology);
    }
    if request.max_simultaneous_failures == 0 {
        errors.push(ServiceNetworkStressValidationError::InvalidFailureOrder);
    } else if request.max_simultaneous_failures > MAX_SIMULTANEOUS_LINK_FAILURES {
        errors.push(ServiceNetworkStressValidationError::FailureOrderTooLarge);
    }
    if request.max_failure_cases == 0 || request.max_failure_cases > MAX_EXHAUSTIVE_FAILURE_CASES {
        errors.push(ServiceNetworkStressValidationError::InvalidFailureCaseBudget);
    }
    if !errors.is_empty() {
        return Err(ServiceNetworkStressError::InvalidInput(errors));
    }

    let mut availability_by_link_id = BTreeMap::new();
    for link in &request.baseline_scenario.links {
        if availability_by_link_id
            .insert(link.id.as_str(), link.availability)
            .is_some()
        {
            errors.push(ServiceNetworkStressValidationError::DuplicateBaselineLinkId);
        }
    }
    let mut declared_set_ids = BTreeSet::new();
    for failure_set in &request.declared_failure_sets {
        if failure_set.id.trim().is_empty() {
            errors.push(ServiceNetworkStressValidationError::EmptyDeclaredFailureSetId);
        }
        if !declared_set_ids.insert(failure_set.id.as_str()) {
            errors.push(ServiceNetworkStressValidationError::DuplicateDeclaredFailureSetId);
        }
        if failure_set.rationale.trim().is_empty() {
            errors.push(ServiceNetworkStressValidationError::EmptyDeclaredFailureSetRationale);
        }
        if failure_set.failed_link_ids.is_empty() {
            errors.push(ServiceNetworkStressValidationError::EmptyDeclaredFailureSetLinks);
        }
        let mut failed_link_ids = BTreeSet::new();
        for link_id in &failure_set.failed_link_ids {
            if link_id.trim().is_empty() {
                errors.push(ServiceNetworkStressValidationError::EmptyDeclaredFailureLinkId);
            }
            if !failed_link_ids.insert(link_id.as_str()) {
                errors.push(ServiceNetworkStressValidationError::DuplicateDeclaredFailureLinkId);
            }
            match availability_by_link_id.get(link_id.as_str()) {
                None => {
                    errors.push(ServiceNetworkStressValidationError::UnknownDeclaredFailureLinkId)
                }
                Some(availability) if *availability == NetworkLinkAvailability::Inactive => {
                    errors.push(
                        ServiceNetworkStressValidationError::AlreadyInactiveDeclaredFailureLinkId,
                    );
                }
                Some(_) => {}
            }
        }
    }
    if !errors.is_empty() {
        return Err(ServiceNetworkStressError::InvalidInput(errors));
    }

    let mut active_link_ids: Vec<String> = request
        .baseline_scenario
        .links
        .iter()
        .filter(|link| link.availability == NetworkLinkAvailability::Active)
        .map(|link| link.id.clone())
        .collect();
    active_link_ids.sort();
    if active_link_ids.is_empty() {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::NoActiveLinks,
        ]));
    }

    let link_combination_count = count_failure_combinations(
        active_link_ids.len(),
        request.max_simultaneous_failures,
        request.max_failure_cases,
    );
    let Some(link_combination_count) = link_combination_count else {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ]));
    };
    let Some(candidate_count) =
        link_combination_count.checked_add(request.declared_failure_sets.len())
    else {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ]));
    };
    if candidate_count > request.max_failure_cases || candidate_count > MAX_EXHAUSTIVE_FAILURE_CASES
    {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ]));
    }
    let Some(scenario_count) = candidate_count.checked_add(1) else {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ]));
    };
    let modeled_link_references = scenario_count.checked_mul(request.baseline_scenario.links.len());
    if modeled_link_references.is_none_or(|count| count > MAX_STRESS_CASE_LINK_REFERENCES) {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioMemoryBudgetExceeded,
        ]));
    }

    let mut failure_sets = Vec::with_capacity(link_combination_count);
    let maximum_order = request.max_simultaneous_failures.min(active_link_ids.len());
    for order in 1..=maximum_order {
        let mut selected = Vec::with_capacity(order);
        append_combinations(&active_link_ids, order, 0, &mut selected, &mut failure_sets);
    }
    if failure_sets.len() != link_combination_count {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ]));
    }

    let baseline_key = format!("sei-stress:{}:baseline", request.id);
    let mut baseline_input = request.baseline_scenario.clone();
    baseline_input.id = baseline_key.clone();

    let mut scenario_batch = Vec::with_capacity(scenario_count);
    scenario_batch.push(baseline_input);
    let mut case_specs = Vec::with_capacity(candidate_count);

    for (index, failed_link_ids) in failure_sets.iter().enumerate() {
        let case_id = format!("sei-stress:{}:case:{:05}", request.id, index + 1);
        let failed: BTreeSet<&str> = failed_link_ids.iter().map(String::as_str).collect();
        let mut candidate = request.baseline_scenario.clone();
        candidate.id = case_id.clone();

        for link in &mut candidate.links {
            if !failed.contains(link.id.as_str()) {
                continue;
            }
            link.availability = NetworkLinkAvailability::Inactive;
            link.availability_basis = ScenarioBasis::Assumed;
            link.availability_rationale = format!(
                "Explicit exhaustive link-failure experiment {}: force link {} inactive; this is a hypothetical failure, not an observation.",
                request.id, link.id
            );
            link.availability_supporting_evidence_ids.clear();
        }
        case_specs.push(FailureCaseSpec {
            case_id,
            origin: ServiceNetworkFailureCaseOrigin::EnumeratedLinkCombination,
            failure_set_id: None,
            rationale: format!(
                "Exhaustively enumerated link-failure combination: {}",
                failed_link_ids.join(", ")
            ),
            failed_link_ids: failed_link_ids.clone(),
        });
        scenario_batch.push(candidate);
    }

    let mut declared_sets: Vec<&DeclaredNetworkFailureSet> =
        request.declared_failure_sets.iter().collect();
    declared_sets.sort_by(|left, right| left.id.cmp(&right.id));
    for failure_set in declared_sets {
        let mut failed_link_ids = failure_set.failed_link_ids.clone();
        failed_link_ids.sort();
        let failed: BTreeSet<&str> = failed_link_ids.iter().map(String::as_str).collect();
        let case_id = format!("sei-stress:{}:declared:{}", request.id, failure_set.id);
        let mut candidate = request.baseline_scenario.clone();
        candidate.id = case_id.clone();

        for link in &mut candidate.links {
            if !failed.contains(link.id.as_str()) {
                continue;
            }
            link.availability = NetworkLinkAvailability::Inactive;
            link.availability_basis = ScenarioBasis::Assumed;
            link.availability_rationale = format!(
                "Declared stress event {}: {}; link {} is forced inactive as a hypothetical scenario, not an observation.",
                failure_set.id, failure_set.rationale, link.id
            );
            link.availability_supporting_evidence_ids.clear();
        }

        let origin = match failure_set.kind {
            DeclaredFailureSetKind::CommonCause => {
                ServiceNetworkFailureCaseOrigin::DeclaredCommonCause
            }
            DeclaredFailureSetKind::HazardBundle => {
                ServiceNetworkFailureCaseOrigin::DeclaredHazardBundle
            }
            DeclaredFailureSetKind::Other => ServiceNetworkFailureCaseOrigin::DeclaredOther,
        };
        case_specs.push(FailureCaseSpec {
            case_id,
            origin,
            failure_set_id: Some(failure_set.id.clone()),
            rationale: failure_set.rationale.clone(),
            failed_link_ids,
        });
        scenario_batch.push(candidate);
    }

    // One shared evidence corpus/index for the baseline and every failure case.
    let assessments = assess_service_capacity_networks(&scenario_batch, evidence_records)
        .map_err(ServiceNetworkStressError::NetworkAssessment)?;
    let mut by_id: BTreeMap<String, ServiceCapacityNetworkAssessment> = assessments
        .into_iter()
        .map(|assessment| (assessment.scenario_id.clone(), assessment))
        .collect();
    let mut baseline = by_id.remove(&baseline_key).ok_or_else(|| {
        ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ])
    })?;
    baseline.scenario_id = request.baseline_scenario.id.clone();

    let baseline_conservative = baseline.conservative_available_capacity;
    let baseline_optimistic = baseline.optimistic_available_capacity;
    let mut cases = Vec::with_capacity(candidate_count);
    let mut worst_conservative = f64::INFINITY;
    let mut worst_optimistic = f64::INFINITY;

    for spec in &case_specs {
        let Some(mut assessment) = by_id.remove(&spec.case_id) else {
            return Err(ServiceNetworkStressError::InvalidInput(vec![
                ServiceNetworkStressValidationError::AssessmentCountMismatch,
            ]));
        };

        let scale = baseline_conservative
            .abs()
            .max(assessment.conservative_available_capacity.abs())
            .max(1.0);
        let tolerance = FAILURE_TIE_RELATIVE_TOLERANCE * scale;
        if assessment.conservative_available_capacity > baseline_conservative + tolerance {
            return Err(ServiceNetworkStressError::InvalidInput(vec![
                ServiceNetworkStressValidationError::NonMonotonicConservativeFlow,
            ]));
        }
        let optimistic_scale = baseline_optimistic
            .abs()
            .max(assessment.optimistic_available_capacity.abs())
            .max(1.0);
        if assessment.optimistic_available_capacity
            > baseline_optimistic + FAILURE_TIE_RELATIVE_TOLERANCE * optimistic_scale
        {
            return Err(ServiceNetworkStressError::InvalidInput(vec![
                ServiceNetworkStressValidationError::NonMonotonicOptimisticFlow,
            ]));
        }

        let conservative_capacity_loss =
            (baseline_conservative - assessment.conservative_available_capacity).max(0.0);
        let optimistic_capacity_loss =
            (baseline_optimistic - assessment.optimistic_available_capacity).max(0.0);
        worst_conservative = worst_conservative.min(assessment.conservative_available_capacity);
        worst_optimistic = worst_optimistic.min(assessment.optimistic_available_capacity);
        assessment.scenario_id = spec.case_id.clone();

        cases.push(ServiceNetworkFailureCase {
            case_id: spec.case_id.clone(),
            origin: spec.origin,
            failure_set_id: spec.failure_set_id.clone(),
            rationale: spec.rationale.clone(),
            failed_link_ids: spec.failed_link_ids.clone(),
            assessment,
            conservative_capacity_loss,
            optimistic_capacity_loss,
        });
    }
    if !by_id.is_empty() {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::AssessmentCountMismatch,
        ]));
    }

    let mut worst_case_conservative_failure_sets: Vec<Vec<String>> = cases
        .iter()
        .filter(|case| {
            capacities_tie(
                case.assessment.conservative_available_capacity,
                worst_conservative,
            )
        })
        .map(|case| case.failed_link_ids.clone())
        .collect();
    let mut worst_case_optimistic_failure_sets: Vec<Vec<String>> = cases
        .iter()
        .filter(|case| {
            capacities_tie(
                case.assessment.optimistic_available_capacity,
                worst_optimistic,
            )
        })
        .map(|case| case.failed_link_ids.clone())
        .collect();
    worst_case_conservative_failure_sets.sort();
    worst_case_conservative_failure_sets.dedup();
    worst_case_optimistic_failure_sets.sort();
    worst_case_optimistic_failure_sets.dedup();

    Ok(ServiceNetworkStressAssessment {
        schema_version: SERVICE_NETWORK_STRESS_SCHEMA_VERSION.into(),
        experiment_id: request.id.clone(),
        baseline_scenario_id: request.baseline_scenario.id.clone(),
        service_entity_id: baseline.service_entity_id.clone(),
        as_of: baseline.as_of.clone(),
        unit: baseline.unit.clone(),
        max_simultaneous_failures: maximum_order,
        active_link_count: active_link_ids.len(),
        declared_failure_set_count: request.declared_failure_sets.len(),
        candidate_failure_case_count: candidate_count,
        evaluated_failure_case_count: cases.len(),
        exhaustive: true,
        baseline,
        worst_case_conservative_capacity: worst_conservative,
        worst_case_optimistic_capacity: worst_optimistic,
        worst_case_conservative_capacity_loss: (baseline_conservative - worst_conservative)
            .max(0.0),
        worst_case_optimistic_capacity_loss: (baseline_optimistic - worst_optimistic).max(0.0),
        worst_case_conservative_failure_sets,
        worst_case_optimistic_failure_sets,
        cases,
    })
}

fn capacities_tie(left: f64, right: f64) -> bool {
    (left - right).abs() <= FAILURE_TIE_RELATIVE_TOLERANCE * left.abs().max(right.abs()).max(1.0)
}

fn count_failure_combinations(link_count: usize, max_order: usize, cap: usize) -> Option<usize> {
    let mut total = 0usize;
    for order in 1..=max_order.min(link_count) {
        let count = bounded_binomial(link_count, order, cap)?;
        total = total.checked_add(count)?;
        if total > cap {
            return None;
        }
    }
    Some(total)
}

fn bounded_binomial(n: usize, k: usize, cap: usize) -> Option<usize> {
    let k = k.min(n.saturating_sub(k));
    let mut result = 1_u128;
    for index in 1..=k {
        let factor = n - k + index;
        result = result.checked_mul(factor as u128)? / index as u128;
        if result > cap as u128 {
            return None;
        }
    }
    Some(result as usize)
}

fn append_combinations(
    values: &[String],
    size: usize,
    start: usize,
    selected: &mut Vec<String>,
    output: &mut Vec<Vec<String>>,
) {
    if selected.len() == size {
        output.push(selected.clone());
        return;
    }
    let remaining = size - selected.len();
    if values.len().saturating_sub(start) < remaining {
        return;
    }
    let last_start = values.len() - remaining;
    for index in start..=last_start {
        selected.push(values[index].clone());
        append_combinations(values, size, index + 1, selected, output);
        selected.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_function::FunctionMeasurement;
    use crate::service_network::{
        NetworkLink, NetworkLinkAvailability, NetworkTopologyCompleteness,
        SERVICE_NETWORK_SCHEMA_VERSION, ServiceCapacityNetworkScenario,
    };
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceRecord, EvidenceState, Provenance, Quantity,
        SpatialContext, TimeContext,
    };

    fn measurement(entity_id: &str, value: f64) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity_id.into(),
            quantity: Quantity {
                value,
                unit: "m3/h".into(),
                uncertainty_abs: Some(0.0),
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic stress test assumption".into(),
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
            availability_rationale: "Synthetic link state assumption".into(),
            availability_supporting_evidence_ids: Vec::new(),
        }
    }

    fn request(order: usize, budget: usize) -> ServiceNetworkStressRequest {
        ServiceNetworkStressRequest {
            schema_version: SERVICE_NETWORK_STRESS_SCHEMA_VERSION.into(),
            id: "stress:shared-trunk".into(),
            baseline_scenario: ServiceCapacityNetworkScenario {
                schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
                id: "network:shared-trunk".into(),
                as_of: "2026-10-16T10:00:00Z".into(),
                service_entity_id: "service:water".into(),
                topology_completeness: NetworkTopologyCompleteness::Complete,
                source_entity_ids: vec!["source:reservoir".into()],
                required_capacity: measurement("service:water", 8.0),
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
                        "service:water",
                        4.0,
                        NetworkLinkAvailability::Active,
                    ),
                    link(
                        "route-b",
                        "junction",
                        "service:water",
                        3.0,
                        NetworkLinkAvailability::Active,
                    ),
                    link(
                        "uncertain",
                        "junction",
                        "service:water",
                        2.0,
                        NetworkLinkAvailability::Unknown,
                    ),
                    link(
                        "closed",
                        "junction",
                        "service:water",
                        100.0,
                        NetworkLinkAvailability::Inactive,
                    ),
                ],
            },
            max_simultaneous_failures: order,
            max_failure_cases: budget,
            declared_failure_sets: Vec::new(),
        }
    }

    #[test]
    fn single_failure_sweep_is_exhaustive_and_reports_critical_bottleneck() {
        let result = assess_service_network_stress(&request(1, 10), &[]).unwrap();
        assert_eq!(result.candidate_failure_case_count, 3);
        assert_eq!(result.evaluated_failure_case_count, 3);
        assert!(result.exhaustive);
        assert_eq!(result.baseline.conservative_available_capacity, 7.0);
        assert_eq!(result.worst_case_conservative_capacity, 0.0);
        assert_eq!(result.worst_case_conservative_capacity_loss, 7.0);
        assert_eq!(
            result.worst_case_conservative_failure_sets,
            vec![vec!["supply".to_owned()]]
        );
    }

    #[test]
    fn double_failure_sweep_enumerates_all_pairs_without_probability_claim() {
        let result = assess_service_network_stress(&request(2, 10), &[]).unwrap();
        assert_eq!(result.candidate_failure_case_count, 6);
        assert_eq!(result.evaluated_failure_case_count, 6);
        assert!(result.exhaustive);
        assert!(
            result
                .cases
                .iter()
                .all(|case| case.failed_link_ids.len() <= 2)
        );
        assert_eq!(result.cases[3].failed_link_ids, vec!["route-a", "route-b"]);
    }

    #[test]
    fn unknown_and_inactive_links_are_not_failure_candidates() {
        let result = assess_service_network_stress(&request(3, 10), &[]).unwrap();
        assert_eq!(result.active_link_count, 3);
        assert!(
            result
                .cases
                .iter()
                .all(|case| !case.failed_link_ids.contains(&"uncertain".to_owned()))
        );
        assert!(
            result
                .cases
                .iter()
                .all(|case| !case.failed_link_ids.contains(&"closed".to_owned()))
        );
    }

    #[test]
    fn incomplete_topology_is_rejected_for_network_stress_claims() {
        let mut input = request(1, 10);
        input.baseline_scenario.topology_completeness = NetworkTopologyCompleteness::Unknown;
        assert!(matches!(
            assess_service_network_stress(&input, &[]),
            Err(ServiceNetworkStressError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkStressValidationError::IncompleteTopology)
        ));
    }

    #[test]
    fn over_budget_stress_fails_instead_of_returning_partial_sweep() {
        let result = assess_service_network_stress(&request(2, 5), &[]);
        assert!(matches!(
            result,
            Err(ServiceNetworkStressError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkStressValidationError::ScenarioBudgetExceeded)
        ));
    }

    #[test]
    fn json_request_fixture_produces_exhaustive_n1_report() {
        let input: ServiceNetworkStressRequest = serde_json::from_str(include_str!(
            "../../schemas/examples/service-network-stress-request.json"
        ))
        .unwrap();
        let result = assess_service_network_stress(&input, &[]).unwrap();
        assert_eq!(result.baseline_scenario_id, "network:shared-trunk");
        assert_eq!(result.candidate_failure_case_count, 3);
        assert_eq!(result.evaluated_failure_case_count, 3);
        assert!(result.exhaustive);
        assert_eq!(result.baseline.conservative_available_capacity, 7.0);
        assert_eq!(result.worst_case_conservative_capacity, 0.0);
        assert_eq!(
            result.worst_case_conservative_failure_sets,
            vec![vec!["supply".to_owned()]]
        );
    }

    #[test]
    fn cloned_scenario_link_reference_budget_fails_before_materializing_cases() {
        let mut input = request(1, 2_000);
        input.baseline_scenario.links.clear();
        for index in 0..2_000 {
            input.baseline_scenario.links.push(link(
                &format!("link-{index:04}"),
                "source:reservoir",
                "service:water",
                1.0,
                NetworkLinkAvailability::Active,
            ));
        }
        assert!(matches!(
            assess_service_network_stress(&input, &[]),
            Err(ServiceNetworkStressError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkStressValidationError::ScenarioMemoryBudgetExceeded)
        ));
    }

    #[test]
    fn forced_availability_failures_remove_the_overridden_observation_reference() {
        let mut input = request(1, 10);
        input.baseline_scenario.links.truncate(1);
        input.baseline_scenario.links[0].to_entity_id = "service:water".into();
        input.baseline_scenario.links[0].availability_basis = ScenarioBasis::Observed;
        input.baseline_scenario.links[0].availability_supporting_evidence_ids =
            vec!["ev:supply-active".into()];
        let evidence = EvidenceRecord {
            id: "ev:supply-active".into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:stress-tests".into(),
            subject_id: "supply".into(),
            spatial: SpatialContext {
                entity_id: "supply".into(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: "2026-10-16T10:00:00Z".into(),
                recorded_at: "2026-10-16T10:00:00Z".into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic observed supply-link availability".into(),
            quantity: None,
            money: None,
            provenance: Provenance {
                method: "synthetic_test".into(),
                source_artifacts: Vec::new(),
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:synthetic-test".into(),
        };

        let result = assess_service_network_stress(&input, &[evidence]).unwrap();
        assert_eq!(
            result.baseline.availability_evidence_ids,
            vec!["ev:supply-active"]
        );
        assert_eq!(result.cases.len(), 1);
        assert!(
            result.cases[0]
                .assessment
                .availability_evidence_ids
                .is_empty()
        );
        assert_eq!(result.cases[0].failed_link_ids, vec!["supply"]);
    }

    #[test]
    fn declared_common_cause_bundle_is_separate_from_single_link_combinations() {
        let mut input = request(1, 10);
        input.declared_failure_sets = vec![DeclaredNetworkFailureSet {
            id: "shared-feeder:outage".into(),
            kind: DeclaredFailureSetKind::CommonCause,
            rationale: "Synthetic shared feeder fault disables both parallel routes.".into(),
            failed_link_ids: vec!["route-b".into(), "route-a".into()],
        }];

        let result = assess_service_network_stress(&input, &[]).unwrap();
        assert_eq!(result.candidate_failure_case_count, 4);
        assert_eq!(result.evaluated_failure_case_count, 4);
        assert_eq!(result.declared_failure_set_count, 1);
        assert!(result.exhaustive);

        let bundle = result
            .cases
            .iter()
            .find(|case| case.failure_set_id.as_deref() == Some("shared-feeder:outage"))
            .unwrap();
        assert_eq!(
            bundle.origin,
            ServiceNetworkFailureCaseOrigin::DeclaredCommonCause
        );
        assert_eq!(bundle.failed_link_ids, vec!["route-a", "route-b"]);
        assert_eq!(bundle.assessment.conservative_available_capacity, 0.0);
        assert_eq!(bundle.conservative_capacity_loss, 7.0);
        assert!(bundle.rationale.contains("shared feeder fault"));

        assert_eq!(
            result.worst_case_conservative_failure_sets,
            vec![
                vec!["route-a".to_owned(), "route-b".to_owned()],
                vec!["supply".to_owned()]
            ]
        );
    }

    #[test]
    fn declared_failure_sets_validate_ids_members_and_active_state() {
        let mut unknown = request(1, 10);
        unknown.declared_failure_sets = vec![DeclaredNetworkFailureSet {
            id: "bad-set".into(),
            kind: DeclaredFailureSetKind::CommonCause,
            rationale: "Synthetic invalid bundle.".into(),
            failed_link_ids: vec!["link-does-not-exist".into()],
        }];
        assert!(matches!(
            assess_service_network_stress(&unknown, &[]),
            Err(ServiceNetworkStressError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkStressValidationError::UnknownDeclaredFailureLinkId)
        ));

        let mut inactive = request(1, 10);
        inactive.declared_failure_sets = vec![DeclaredNetworkFailureSet {
            id: "inactive-bundle".into(),
            kind: DeclaredFailureSetKind::HazardBundle,
            rationale: "Synthetic invalid already-inactive bundle.".into(),
            failed_link_ids: vec!["closed".into()],
        }];
        assert!(matches!(
            assess_service_network_stress(&inactive, &[]),
            Err(ServiceNetworkStressError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkStressValidationError::AlreadyInactiveDeclaredFailureLinkId)
        ));

        let mut duplicate_set = request(1, 10);
        duplicate_set.declared_failure_sets = vec![
            DeclaredNetworkFailureSet {
                id: "same".into(),
                kind: DeclaredFailureSetKind::Other,
                rationale: "Synthetic first bundle.".into(),
                failed_link_ids: vec!["route-a".into()],
            },
            DeclaredNetworkFailureSet {
                id: "same".into(),
                kind: DeclaredFailureSetKind::Other,
                rationale: "Synthetic second bundle.".into(),
                failed_link_ids: vec!["route-b".into()],
            },
        ];
        assert!(matches!(
            assess_service_network_stress(&duplicate_set, &[]),
            Err(ServiceNetworkStressError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkStressValidationError::DuplicateDeclaredFailureSetId)
        ));
    }

    #[test]
    fn declared_failure_bundle_is_included_in_case_budget() {
        let mut input = request(1, 3);
        input.declared_failure_sets = vec![DeclaredNetworkFailureSet {
            id: "shared-feeder".into(),
            kind: DeclaredFailureSetKind::CommonCause,
            rationale: "Synthetic common-cause event.".into(),
            failed_link_ids: vec!["route-a".into(), "route-b".into()],
        }];
        assert!(matches!(
            assess_service_network_stress(&input, &[]),
            Err(ServiceNetworkStressError::InvalidInput(errors))
                if errors.contains(&ServiceNetworkStressValidationError::ScenarioBudgetExceeded)
        ));
    }

    #[test]
    fn declared_failure_manifest_is_deterministic_under_input_permutation() {
        let mut first = request(1, 10);
        first.declared_failure_sets = vec![
            DeclaredNetworkFailureSet {
                id: "zeta".into(),
                kind: DeclaredFailureSetKind::HazardBundle,
                rationale: "Synthetic second named event.".into(),
                failed_link_ids: vec!["route-b".into(), "route-a".into()],
            },
            DeclaredNetworkFailureSet {
                id: "alpha".into(),
                kind: DeclaredFailureSetKind::CommonCause,
                rationale: "Synthetic first named event.".into(),
                failed_link_ids: vec!["supply".into(), "route-a".into()],
            },
        ];
        let first_result = assess_service_network_stress(&first, &[]).unwrap();
        first.declared_failure_sets.reverse();
        first.baseline_scenario.links.reverse();
        for set in &mut first.declared_failure_sets {
            set.failed_link_ids.reverse();
        }
        let second_result = assess_service_network_stress(&first, &[]).unwrap();
        assert_eq!(first_result, second_result);
    }

    #[test]
    fn json_common_cause_request_fixture_enumerates_shared_failure() {
        let input: ServiceNetworkStressRequest = serde_json::from_str(include_str!(
            "../../schemas/examples/service-network-stress-common-cause-request.json"
        ))
        .unwrap();
        let result = assess_service_network_stress(&input, &[]).unwrap();
        assert_eq!(result.declared_failure_set_count, 1);
        assert_eq!(result.candidate_failure_case_count, 4);
        assert!(result.cases.iter().any(|case| {
            case.origin == ServiceNetworkFailureCaseOrigin::DeclaredCommonCause
                && case.failed_link_ids == vec!["route-a", "route-b"]
                && case.assessment.conservative_available_capacity == 0.0
        }));
    }

    #[test]
    fn source_input_order_does_not_change_failure_case_manifest() {
        let first = assess_service_network_stress(&request(2, 10), &[]).unwrap();
        let mut reordered = request(2, 10);
        reordered.baseline_scenario.links.reverse();
        let second = assess_service_network_stress(&reordered, &[]).unwrap();
        assert_eq!(first, second);
    }
}
