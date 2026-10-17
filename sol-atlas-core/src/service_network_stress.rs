// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Exhaustive bounded link-failure experiments for service-capacity networks.
//!
//! This layer moves from a single nominal flow calculation to explicit N-k stress
//! cases. It enumerates every combination of up to k currently active links, subject
//! to a caller-declared case budget; it never silently samples or truncates cases.
//! Scenario counts are stress-test cases, not failure probabilities.

use crate::service_network::{
    assess_service_capacity_networks, NetworkLinkAvailability,
    NetworkTopologyCompleteness, ServiceCapacityNetworkAssessment, ServiceCapacityNetworkError,
    ServiceCapacityNetworkScenario,
};
use crate::dependency_logic::ScenarioBasis;
use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SERVICE_NETWORK_STRESS_SCHEMA_VERSION: &str = "0.1.0";
pub const MAX_SIMULTANEOUS_LINK_FAILURES: usize = 3;
pub const MAX_EXHAUSTIVE_FAILURE_CASES: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkStressRequest {
    pub schema_version: String,
    /// Stable experiment ID; used to derive deterministic case IDs.
    pub id: String,
    pub baseline_scenario: ServiceCapacityNetworkScenario,
    /// Enumerate every combination of 1..=k active-link failures.
    pub max_simultaneous_failures: usize,
    /// Hard budget for the exhaustive set. Over-budget inputs fail instead of
    /// returning a misleading partial result.
    pub max_failure_cases: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceNetworkFailureCase {
    pub case_id: String,
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
    pub max_simultaneous_failures: usize,
    pub active_link_count: usize,
    /// Exact number of generated failure combinations. This equals evaluated count
    /// on every successful response: the sweep is exhaustive, not sampled.
    pub candidate_failure_case_count: usize,
    pub evaluated_failure_case_count: usize,
    pub exhaustive: bool,
    pub baseline: ServiceCapacityNetworkAssessment,
    pub worst_case_conservative_capacity: f64,
    pub worst_case_optimistic_capacity: f64,
    pub worst_case_conservative_capacity_loss: f64,
    pub worst_case_optimistic_capacity_loss: f64,
    /// All failure sets tied for the lowest conservative flow, stable sorted order.
    pub worst_case_conservative_failure_sets: Vec<Vec<String>>,
    /// All failure sets tied for the lowest optimistic flow, stable sorted order.
    pub worst_case_optimistic_failure_sets: Vec<Vec<String>>,
    /// Cases are sorted by failure order, then lexicographic failed-link IDs.
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
    NonMonotonicConservativeFlow,
    NonMonotonicOptimisticFlow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceNetworkStressError {
    InvalidInput(Vec<ServiceNetworkStressValidationError>),
    NetworkAssessment(ServiceCapacityNetworkError),
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
    if request.max_failure_cases == 0
        || request.max_failure_cases > MAX_EXHAUSTIVE_FAILURE_CASES
    {
        errors.push(ServiceNetworkStressValidationError::InvalidFailureCaseBudget);
    }
    if !errors.is_empty() {
        return Err(ServiceNetworkStressError::InvalidInput(errors));
    }

    let mut active_link_ids: Vec<String> = request.baseline_scenario.links.iter()
        .filter(|link| link.availability == NetworkLinkAvailability::Active)
        .map(|link| link.id.clone())
        .collect();
    active_link_ids.sort();
    if active_link_ids.is_empty() {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::NoActiveLinks,
        ]));
    }

    let candidate_count = count_failure_combinations(
        active_link_ids.len(),
        request.max_simultaneous_failures,
        request.max_failure_cases,
    );
    let Some(candidate_count) = candidate_count else {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ]));
    };
    if candidate_count > request.max_failure_cases {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ]));
    }

    let mut failure_sets = Vec::with_capacity(candidate_count);
    let maximum_order = request.max_simultaneous_failures.min(active_link_ids.len());
    for order in 1..=maximum_order {
        let mut selected = Vec::with_capacity(order);
        append_combinations(
            &active_link_ids,
            order,
            0,
            &mut selected,
            &mut failure_sets,
        );
    }
    if failure_sets.len() != candidate_count {
        return Err(ServiceNetworkStressError::InvalidInput(vec![
            ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
        ]));
    }

    let baseline_key = format!("sei-stress:{}:baseline", request.id);
    let mut baseline_input = request.baseline_scenario.clone();
    baseline_input.id = baseline_key.clone();

    let mut scenario_batch = Vec::with_capacity(candidate_count + 1);
    scenario_batch.push(baseline_input);
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
                "Explicit exhaustive stress experiment {}: force link {} inactive; this is a hypothetical failure, not an observation.",
                request.id, link.id
            );
            link.availability_supporting_evidence_ids.clear();
        }
        scenario_batch.push(candidate);
    }

    // One shared evidence corpus/index for the baseline and every failure case.
    let assessments = assess_service_capacity_networks(&scenario_batch, evidence_records)
        .map_err(ServiceNetworkStressError::NetworkAssessment)?;
    let mut by_id: BTreeMap<String, ServiceCapacityNetworkAssessment> = assessments.into_iter()
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

    for failed_link_ids in &failure_sets {
        // Failure set order determines generated IDs, so calculate this exact ID
        // from its ordinal in the deterministic enumeration.
        let index = cases.len() + 1;
        let case_id = format!("sei-stress:{}:case:{:05}", request.id, index);
        let Some(mut assessment) = by_id.remove(&case_id) else {
            return Err(ServiceNetworkStressError::InvalidInput(vec![
                ServiceNetworkStressValidationError::ScenarioBudgetExceeded,
            ]));
        };

        let scale = baseline_conservative.abs()
            .max(assessment.conservative_available_capacity.abs())
            .max(1.0);
        let tolerance = 1.0e-9 * scale;
        if assessment.conservative_available_capacity > baseline_conservative + tolerance {
            return Err(ServiceNetworkStressError::InvalidInput(vec![
                ServiceNetworkStressValidationError::NonMonotonicConservativeFlow,
            ]));
        }
        let optimistic_scale = baseline_optimistic.abs()
            .max(assessment.optimistic_available_capacity.abs())
            .max(1.0);
        if assessment.optimistic_available_capacity > baseline_optimistic + 1.0e-9 * optimistic_scale {
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

        // Keep generated scenario ID for audit, even though it is derived from the
        // experiment rather than persisted as an independent input scenario.
        assessment.scenario_id = case_id.clone();
        cases.push(ServiceNetworkFailureCase {
            case_id,
            failed_link_ids: failed_link_ids.clone(),
            assessment,
            conservative_capacity_loss,
            optimistic_capacity_loss,
        });
    }

    let mut worst_case_conservative_failure_sets: Vec<Vec<String>> = cases.iter()
        .filter(|case| case.assessment.conservative_available_capacity == worst_conservative)
        .map(|case| case.failed_link_ids.clone())
        .collect();
    let mut worst_case_optimistic_failure_sets: Vec<Vec<String>> = cases.iter()
        .filter(|case| case.assessment.optimistic_available_capacity == worst_optimistic)
        .map(|case| case.failed_link_ids.clone())
        .collect();
    worst_case_conservative_failure_sets.sort();
    worst_case_optimistic_failure_sets.sort();

    Ok(ServiceNetworkStressAssessment {
        schema_version: SERVICE_NETWORK_STRESS_SCHEMA_VERSION.into(),
        experiment_id: request.id.clone(),
        baseline_scenario_id: request.baseline_scenario.id.clone(),
        service_entity_id: baseline.service_entity_id.clone(),
        as_of: baseline.as_of.clone(),
        unit: baseline.unit.clone(),
        max_simultaneous_failures: maximum_order,
        active_link_count: active_link_ids.len(),
        candidate_failure_case_count: candidate_count,
        evaluated_failure_case_count: cases.len(),
        exhaustive: true,
        baseline,
        worst_case_conservative_capacity: worst_conservative,
        worst_case_optimistic_capacity: worst_optimistic,
        worst_case_conservative_capacity_loss: (baseline_conservative - worst_conservative).max(0.0),
        worst_case_optimistic_capacity_loss: (baseline_optimistic - worst_optimistic).max(0.0),
        worst_case_conservative_failure_sets,
        worst_case_optimistic_failure_sets,
        cases,
    })
}

fn count_failure_combinations(
    link_count: usize,
    max_order: usize,
    cap: usize,
) -> Option<usize> {
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
    use crate::service_network::{NetworkLink, NetworkLinkAvailability, NetworkTopologyCompleteness, ServiceCapacityNetworkScenario, SERVICE_NETWORK_SCHEMA_VERSION};
    use crate::service_function::FunctionMeasurement;
    use crate::spatial_economic::Quantity;

    fn measurement(entity_id: &str, value: f64) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity_id.into(),
            quantity: Quantity { value, unit: "m3/h".into(), uncertainty_abs: Some(0.0) },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic stress test assumption".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn link(id: &str, from: &str, to: &str, capacity: f64, availability: NetworkLinkAvailability) -> NetworkLink {
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
                    link("supply", "source:reservoir", "junction", 10.0, NetworkLinkAvailability::Active),
                    link("route-a", "junction", "service:water", 4.0, NetworkLinkAvailability::Active),
                    link("route-b", "junction", "service:water", 3.0, NetworkLinkAvailability::Active),
                    link("uncertain", "junction", "service:water", 2.0, NetworkLinkAvailability::Unknown),
                    link("closed", "junction", "service:water", 100.0, NetworkLinkAvailability::Inactive),
                ],
            },
            max_simultaneous_failures: order,
            max_failure_cases: budget,
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
        assert_eq!(result.worst_case_conservative_failure_sets, vec![vec!["supply".to_owned()]]);
    }

    #[test]
    fn double_failure_sweep_enumerates_all_pairs_without_probability_claim() {
        let result = assess_service_network_stress(&request(2, 10), &[]).unwrap();
        assert_eq!(result.candidate_failure_case_count, 6);
        assert_eq!(result.evaluated_failure_case_count, 6);
        assert!(result.exhaustive);
        assert!(result.cases.iter().all(|case| case.failed_link_ids.len() <= 2));
        assert_eq!(result.cases[3].failed_link_ids, vec!["route-a", "route-b"]);
    }

    #[test]
    fn unknown_and_inactive_links_are_not_failure_candidates() {
        let result = assess_service_network_stress(&request(3, 10), &[]).unwrap();
        assert_eq!(result.active_link_count, 3);
        assert!(result.cases.iter().all(|case| !case.failed_link_ids.contains(&"uncertain".to_owned())));
        assert!(result.cases.iter().all(|case| !case.failed_link_ids.contains(&"closed".to_owned())));
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
    fn source_input_order_does_not_change_failure_case_manifest() {
        let first = assess_service_network_stress(&request(2, 10), &[]).unwrap();
        let mut reordered = request(2, 10);
        reordered.baseline_scenario.links.reverse();
        let second = assess_service_network_stress(&reordered, &[]).unwrap();
        assert_eq!(first, second);
    }
}
