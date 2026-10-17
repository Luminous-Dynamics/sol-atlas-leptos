// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Paired baseline/candidate comparison across the same declared joint-failure suite.
//!
//! A candidate is not robustly better if any protected service floor may regress.
//! The comparator shares one validated evidence index, preserves per-side provenance,
//! never averages away a bad scenario, and does not interpret scenario counts as
//! probabilities or counterfactual results as causal field evidence.

use crate::service_equity::ServiceFloorStatus;
use crate::service_function::{ServiceFunctionAssessmentError, build_evidence_index};
use crate::service_water::WaterServiceState;
use crate::service_water_energy_stress::{
    WaterEnergyCommunityFloorOutcome, WaterEnergyMeasurementOverride,
    WaterEnergyStressCaseAssessment, WaterEnergyStressCaseKind, WaterEnergyStressSweepError,
    WaterEnergyStressSweepRequest, assess_water_energy_stress_sweep_indexed,
};
use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionComparisonRequest {
    pub schema_version: String,
    pub id: String,
    pub baseline: WaterEnergyStressSweepRequest,
    pub candidate: WaterEnergyStressSweepRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterEnergyStressInterventionEffect {
    RobustImprovement,
    RobustRegression,
    NoMaterialChange,
    ProtectedFloorRegression,
    MixedEffects,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterEnergyStressInterventionInterpretationBoundary {
    ModeledCounterfactualNotCausalEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaterEnergyStressInterventionValidationError {
    UnsupportedSchemaVersion,
    EmptyComparisonId,
    ServiceCountMismatch,
    ServiceRequestSetMismatch,
    ServiceEntityMismatch,
    ServiceTimeGridMismatch,
    CouplingTimeGridMismatch,
    ProtectedFloorSetMismatch,
    ProtectedFloorDefinitionMismatch,
    FailureScenarioSetMismatch,
    FailureScenarioKindMismatch,
    FailureScenarioRationaleMismatch,
    FailureOverrideSetMismatch,
    ServiceAssessmentCountMismatch,
    CommunitySetMismatch,
    StepAlignmentMismatch,
    NonFiniteDerivedValue,
    BoundsInconsistent,
    EmptyCandidateSet,
    EmptyCandidateId,
    DuplicateCandidateId,
    DuplicateCandidateExperimentId,
    ArithmeticOverflow,
    CandidateCountExceeded,
    InvalidBatchAssessmentBudget,
    BatchAssessmentBudgetExceeded,
    InvalidBatchOverrideBudget,
    BatchOverrideBudgetExceeded,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaterEnergyStressInterventionComparisonError {
    InvalidInput(Vec<WaterEnergyStressInterventionValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
    BaselineAssessment(WaterEnergyStressSweepError),
    CandidateAssessment(WaterEnergyStressSweepError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionStepComparison {
    pub service_request_id: String,
    pub community_id: String,
    pub step_id: String,
    pub as_of: String,
    pub step_hours: f64,
    pub baseline_state: WaterServiceState,
    pub candidate_state: WaterServiceState,
    pub baseline_floor_status: ServiceFloorStatus,
    pub candidate_floor_status: ServiceFloorStatus,
    pub possible_protected_floor_regression: bool,
    pub baseline_coverage_lower_bound: Option<f64>,
    pub baseline_coverage_upper_bound: Option<f64>,
    pub candidate_coverage_lower_bound: Option<f64>,
    pub candidate_coverage_upper_bound: Option<f64>,
    pub baseline_unmet_demand_lower_bound_m3: f64,
    pub baseline_unmet_demand_upper_bound_m3: f64,
    pub candidate_unmet_demand_lower_bound_m3: f64,
    pub candidate_unmet_demand_upper_bound_m3: f64,
    /// Positive means less modeled unmet demand under the candidate.
    pub unmet_demand_reduction_lower_bound_m3: f64,
    pub unmet_demand_reduction_upper_bound_m3: f64,
    pub baseline_customer_delivery_lower_bound_m3: f64,
    pub baseline_customer_delivery_upper_bound_m3: f64,
    pub candidate_customer_delivery_lower_bound_m3: f64,
    pub candidate_customer_delivery_upper_bound_m3: f64,
    /// Positive means more modeled customer delivery under the candidate.
    pub customer_delivery_delta_lower_bound_m3: f64,
    pub customer_delivery_delta_upper_bound_m3: f64,
    pub baseline_supporting_evidence_ids: Vec<String>,
    pub candidate_supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionServiceComparison {
    pub service_request_id: String,
    pub service_entity_id: String,
    pub community_id: String,
    pub beneficiary_weight: u64,
    pub essential_coverage_floor: f64,
    pub baseline_floor_status: ServiceFloorStatus,
    pub candidate_floor_status: ServiceFloorStatus,
    pub possible_protected_floor_regression: bool,
    pub baseline_unmet_demand_lower_bound_m3: f64,
    pub baseline_unmet_demand_upper_bound_m3: f64,
    pub candidate_unmet_demand_lower_bound_m3: f64,
    pub candidate_unmet_demand_upper_bound_m3: f64,
    pub unmet_demand_reduction_lower_bound_m3: f64,
    pub unmet_demand_reduction_upper_bound_m3: f64,
    pub baseline_customer_delivery_lower_bound_m3: f64,
    pub baseline_customer_delivery_upper_bound_m3: f64,
    pub candidate_customer_delivery_lower_bound_m3: f64,
    pub candidate_customer_delivery_upper_bound_m3: f64,
    pub customer_delivery_delta_lower_bound_m3: f64,
    pub customer_delivery_delta_upper_bound_m3: f64,
    pub baseline_maximum_normalized_deficit_lower_bound: f64,
    pub baseline_maximum_normalized_deficit_upper_bound: f64,
    pub candidate_maximum_normalized_deficit_lower_bound: f64,
    pub candidate_maximum_normalized_deficit_upper_bound: f64,
    pub steps: Vec<WaterEnergyStressInterventionStepComparison>,
    pub baseline_supporting_evidence_ids: Vec<String>,
    pub candidate_supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionCommunityComparison {
    pub community_id: String,
    pub service_cell_count: usize,
    pub beneficiary_weight: u64,
    pub baseline_floor_compliance_lower_bound: f64,
    pub baseline_floor_compliance_upper_bound: f64,
    pub candidate_floor_compliance_lower_bound: f64,
    pub candidate_floor_compliance_upper_bound: f64,
    pub floor_compliance_delta_lower_bound: f64,
    pub floor_compliance_delta_upper_bound: f64,
    pub possible_protected_floor_regression: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionCaseComparison {
    pub scenario_id: String,
    pub kind: WaterEnergyStressCaseKind,
    pub rationale: String,
    pub overrides_applied: Vec<WaterEnergyMeasurementOverride>,
    pub effect: WaterEnergyStressInterventionEffect,
    pub possible_protected_floor_regression: bool,
    pub baseline_total_unmet_demand_lower_bound_m3: f64,
    pub baseline_total_unmet_demand_upper_bound_m3: f64,
    pub candidate_total_unmet_demand_lower_bound_m3: f64,
    pub candidate_total_unmet_demand_upper_bound_m3: f64,
    pub unmet_demand_reduction_lower_bound_m3: f64,
    pub unmet_demand_reduction_upper_bound_m3: f64,
    pub baseline_customer_delivery_lower_bound_m3: f64,
    pub baseline_customer_delivery_upper_bound_m3: f64,
    pub candidate_customer_delivery_lower_bound_m3: f64,
    pub candidate_customer_delivery_upper_bound_m3: f64,
    pub customer_delivery_delta_lower_bound_m3: f64,
    pub customer_delivery_delta_upper_bound_m3: f64,
    pub baseline_floor_compliance_lower_bound: f64,
    pub baseline_floor_compliance_upper_bound: f64,
    pub candidate_floor_compliance_lower_bound: f64,
    pub candidate_floor_compliance_upper_bound: f64,
    pub floor_compliance_delta_lower_bound: f64,
    pub floor_compliance_delta_upper_bound: f64,
    pub baseline_maximum_normalized_deficit_lower_bound: f64,
    pub baseline_maximum_normalized_deficit_upper_bound: f64,
    pub candidate_maximum_normalized_deficit_lower_bound: f64,
    pub candidate_maximum_normalized_deficit_upper_bound: f64,
    pub communities: Vec<WaterEnergyStressInterventionCommunityComparison>,
    pub services: Vec<WaterEnergyStressInterventionServiceComparison>,
    pub baseline_supporting_evidence_ids: Vec<String>,
    pub candidate_supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionComparisonAssessment {
    pub schema_version: String,
    pub comparison_id: String,
    pub baseline_experiment_id: String,
    pub candidate_experiment_id: String,
    pub service_count: usize,
    pub declared_failure_scenario_count: usize,
    /// Includes one nominal baseline case plus each declared failure scenario.
    pub compared_case_count: usize,
    pub exhaustive_over_paired_declared_scenarios: bool,
    pub total_paired_service_step_assessments: usize,
    pub effect: WaterEnergyStressInterventionEffect,
    pub possible_protected_floor_regression: bool,
    /// Envelope across every nominal and declared-failure case; no case is averaged away.
    pub worst_case_unmet_demand_reduction_lower_bound_m3: f64,
    pub worst_case_unmet_demand_reduction_upper_bound_m3: f64,
    pub worst_case_customer_delivery_delta_lower_bound_m3: f64,
    pub worst_case_customer_delivery_delta_upper_bound_m3: f64,
    pub worst_case_floor_compliance_delta_lower_bound: f64,
    pub worst_case_floor_compliance_delta_upper_bound: f64,
    pub cases: Vec<WaterEnergyStressInterventionCaseComparison>,
    pub baseline_supporting_evidence_ids: Vec<String>,
    pub candidate_supporting_evidence_ids: Vec<String>,
    pub interpretation_boundary: WaterEnergyStressInterventionInterpretationBoundary,
}

fn aligned_override_sets(
    baseline: &[WaterEnergyMeasurementOverride],
    candidate: &[WaterEnergyMeasurementOverride],
) -> bool {
    if baseline.len() != candidate.len() {
        return false;
    }
    baseline.iter().all(|base| {
        candidate.iter().any(|cand| {
            base.service_request_id == cand.service_request_id
                && base.step_id == cand.step_id
                && base.target == cand.target
                && base.forced_value == cand.forced_value
                && base.forced_uncertainty_abs == cand.forced_uncertainty_abs
        })
    })
}

fn validate_paired_requests(
    request: &WaterEnergyStressInterventionComparisonRequest,
) -> Result<(), Vec<WaterEnergyStressInterventionValidationError>> {
    validate_paired_sweeps(
        &request.schema_version,
        &request.id,
        &request.baseline,
        &request.candidate,
    )
}

fn validate_paired_sweeps(
    schema_version: &str,
    id: &str,
    baseline: &WaterEnergyStressSweepRequest,
    candidate: &WaterEnergyStressSweepRequest,
) -> Result<(), Vec<WaterEnergyStressInterventionValidationError>> {
    let mut errors = Vec::new();
    if schema_version != WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION {
        errors.push(WaterEnergyStressInterventionValidationError::UnsupportedSchemaVersion);
    }
    if id.trim().is_empty() {
        errors.push(WaterEnergyStressInterventionValidationError::EmptyComparisonId);
    }

    let baseline_services: BTreeMap<&str, _> = baseline
        .baseline_services
        .iter()
        .map(|service| (service.id.as_str(), service))
        .collect();
    let candidate_services: BTreeMap<&str, _> = candidate
        .baseline_services
        .iter()
        .map(|service| (service.id.as_str(), service))
        .collect();
    if baseline_services.len() != baseline.baseline_services.len()
        || candidate_services.len() != candidate.baseline_services.len()
    {
        errors.push(WaterEnergyStressInterventionValidationError::ServiceRequestSetMismatch);
    }
    if baseline_services.len() != candidate_services.len()
        || baseline_services.keys().ne(candidate_services.keys())
    {
        errors.push(WaterEnergyStressInterventionValidationError::ServiceCountMismatch);
    }

    for (service_id, baseline_service) in &baseline_services {
        let Some(candidate_service) = candidate_services.get(service_id).copied() else {
            errors.push(WaterEnergyStressInterventionValidationError::ServiceRequestSetMismatch);
            continue;
        };
        if baseline_service.trajectory.service_entity_id
            != candidate_service.trajectory.service_entity_id
        {
            errors.push(WaterEnergyStressInterventionValidationError::ServiceEntityMismatch);
        }
        let baseline_steps = &baseline_service.trajectory.steps;
        let candidate_steps = &candidate_service.trajectory.steps;
        if baseline_steps.len() != candidate_steps.len() {
            errors.push(WaterEnergyStressInterventionValidationError::ServiceTimeGridMismatch);
        }
        for (base, cand) in baseline_steps.iter().zip(candidate_steps) {
            if base.id != cand.id || base.as_of != cand.as_of || base.step_hours != cand.step_hours
            {
                errors.push(WaterEnergyStressInterventionValidationError::ServiceTimeGridMismatch);
                break;
            }
        }

        let baseline_energy_steps: BTreeMap<&str, _> = baseline_service
            .steps
            .iter()
            .map(|step| (step.step_id.as_str(), step))
            .collect();
        let candidate_energy_steps: BTreeMap<&str, _> = candidate_service
            .steps
            .iter()
            .map(|step| (step.step_id.as_str(), step))
            .collect();
        if baseline_energy_steps.len() != baseline_service.steps.len()
            || candidate_energy_steps.len() != candidate_service.steps.len()
            || baseline_energy_steps
                .keys()
                .ne(candidate_energy_steps.keys())
        {
            errors.push(WaterEnergyStressInterventionValidationError::CouplingTimeGridMismatch);
        }
        for (step_id, base) in &baseline_energy_steps {
            match candidate_energy_steps.get(step_id).copied() {
                Some(cand) if base.as_of == cand.as_of => {}
                _ => errors
                    .push(WaterEnergyStressInterventionValidationError::CouplingTimeGridMismatch),
            }
        }
    }

    let baseline_floors: BTreeMap<&str, _> = baseline
        .protected_service_floors
        .iter()
        .map(|floor| (floor.service_request_id.as_str(), floor))
        .collect();
    let candidate_floors: BTreeMap<&str, _> = candidate
        .protected_service_floors
        .iter()
        .map(|floor| (floor.service_request_id.as_str(), floor))
        .collect();
    if baseline_floors.len() != baseline.protected_service_floors.len()
        || candidate_floors.len() != candidate.protected_service_floors.len()
        || baseline_floors.len() != candidate_floors.len()
        || baseline_floors.keys().ne(candidate_floors.keys())
    {
        errors.push(WaterEnergyStressInterventionValidationError::ProtectedFloorSetMismatch);
    }
    for (service_id, base) in &baseline_floors {
        let Some(cand) = candidate_floors.get(service_id).copied() else {
            errors.push(WaterEnergyStressInterventionValidationError::ProtectedFloorSetMismatch);
            continue;
        };
        if base.community_id != cand.community_id
            || base.beneficiary_weight != cand.beneficiary_weight
            || base.essential_coverage_floor != cand.essential_coverage_floor
        {
            errors.push(
                WaterEnergyStressInterventionValidationError::ProtectedFloorDefinitionMismatch,
            );
        }
    }

    let baseline_scenarios: BTreeMap<&str, _> = baseline
        .failure_scenarios
        .iter()
        .map(|scenario| (scenario.id.as_str(), scenario))
        .collect();
    let candidate_scenarios: BTreeMap<&str, _> = candidate
        .failure_scenarios
        .iter()
        .map(|scenario| (scenario.id.as_str(), scenario))
        .collect();
    if baseline_scenarios.len() != baseline.failure_scenarios.len()
        || candidate_scenarios.len() != candidate.failure_scenarios.len()
        || baseline_scenarios.len() != candidate_scenarios.len()
        || baseline_scenarios.keys().ne(candidate_scenarios.keys())
    {
        errors.push(WaterEnergyStressInterventionValidationError::FailureScenarioSetMismatch);
    }
    for (scenario_id, base) in &baseline_scenarios {
        let Some(cand) = candidate_scenarios.get(scenario_id).copied() else {
            errors.push(WaterEnergyStressInterventionValidationError::FailureScenarioSetMismatch);
            continue;
        };
        if base.kind != cand.kind {
            errors.push(WaterEnergyStressInterventionValidationError::FailureScenarioKindMismatch);
        }
        if base.rationale != cand.rationale {
            errors.push(
                WaterEnergyStressInterventionValidationError::FailureScenarioRationaleMismatch,
            );
        }
        if !aligned_override_sets(&base.overrides, &cand.overrides) {
            errors.push(WaterEnergyStressInterventionValidationError::FailureOverrideSetMismatch);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        errors.dedup();
        Err(errors)
    }
}

fn possible_floor_regression(baseline: ServiceFloorStatus, candidate: ServiceFloorStatus) -> bool {
    baseline != ServiceFloorStatus::BelowFloor && candidate != ServiceFloorStatus::MeetsFloor
}

fn effect_from_bounds(
    lower: f64,
    upper: f64,
    floor_regression: bool,
) -> WaterEnergyStressInterventionEffect {
    if floor_regression {
        return WaterEnergyStressInterventionEffect::ProtectedFloorRegression;
    }
    let tolerance = f64::EPSILON * 256.0 * lower.abs().max(upper.abs()).max(1.0);
    if lower > tolerance {
        WaterEnergyStressInterventionEffect::RobustImprovement
    } else if upper < -tolerance {
        WaterEnergyStressInterventionEffect::RobustRegression
    } else if lower >= -tolerance && upper <= tolerance {
        WaterEnergyStressInterventionEffect::NoMaterialChange
    } else {
        WaterEnergyStressInterventionEffect::Indeterminate
    }
}

fn delta_bounds(
    baseline_lower: f64,
    baseline_upper: f64,
    candidate_lower: f64,
    candidate_upper: f64,
) -> (f64, f64) {
    (
        baseline_lower - candidate_upper,
        baseline_upper - candidate_lower,
    )
}

fn delivery_delta_bounds(
    baseline_lower: f64,
    baseline_upper: f64,
    candidate_lower: f64,
    candidate_upper: f64,
) -> (f64, f64) {
    (
        candidate_lower - baseline_upper,
        candidate_upper - baseline_lower,
    )
}

fn find_matching_service<'a>(
    case: &'a WaterEnergyStressCaseAssessment,
    service_request_id: &str,
) -> Option<&'a crate::service_water_energy_stress::WaterEnergyStressServiceAssessment> {
    case.services
        .iter()
        .find(|service| service.service_request_id == service_request_id)
}

fn compare_case(
    baseline: &WaterEnergyStressCaseAssessment,
    candidate: &WaterEnergyStressCaseAssessment,
) -> Result<WaterEnergyStressInterventionCaseComparison, WaterEnergyStressInterventionComparisonError>
{
    if baseline.id != candidate.id || baseline.kind != candidate.kind {
        return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
            vec![WaterEnergyStressInterventionValidationError::FailureScenarioSetMismatch],
        ));
    }
    let mut baseline_support = BTreeSet::new();
    baseline_support.extend(baseline.supporting_evidence_ids.iter().cloned());
    let mut candidate_support = BTreeSet::new();
    candidate_support.extend(candidate.supporting_evidence_ids.iter().cloned());

    let (unmet_reduction_lower, unmet_reduction_upper) = delta_bounds(
        baseline.total_unmet_demand_volume_lower_bound_m3,
        baseline.total_unmet_demand_volume_upper_bound_m3,
        candidate.total_unmet_demand_volume_lower_bound_m3,
        candidate.total_unmet_demand_volume_upper_bound_m3,
    );
    let (delivery_delta_lower, delivery_delta_upper) = delivery_delta_bounds(
        baseline
            .services
            .iter()
            .map(|service| service.customer_delivered_volume_lower_bound_m3)
            .sum(),
        baseline
            .services
            .iter()
            .map(|service| service.customer_delivered_volume_upper_bound_m3)
            .sum(),
        candidate
            .services
            .iter()
            .map(|service| service.customer_delivered_volume_lower_bound_m3)
            .sum(),
        candidate
            .services
            .iter()
            .map(|service| service.customer_delivered_volume_upper_bound_m3)
            .sum(),
    );
    let (floor_delta_lower, floor_delta_upper) = delivery_delta_bounds(
        baseline.overall_floor_compliance_lower_bound,
        baseline.overall_floor_compliance_upper_bound,
        candidate.overall_floor_compliance_lower_bound,
        candidate.overall_floor_compliance_upper_bound,
    );

    let mut baseline_floor_by_community: BTreeMap<&str, &WaterEnergyCommunityFloorOutcome> =
        BTreeMap::new();
    let mut candidate_floor_by_community: BTreeMap<&str, &WaterEnergyCommunityFloorOutcome> =
        BTreeMap::new();
    for community in &baseline.communities {
        baseline_floor_by_community.insert(community.community_id.as_str(), community);
    }
    for community in &candidate.communities {
        candidate_floor_by_community.insert(community.community_id.as_str(), community);
    }
    if baseline_floor_by_community.len() != candidate_floor_by_community.len()
        || baseline_floor_by_community
            .keys()
            .ne(candidate_floor_by_community.keys())
    {
        return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
            vec![WaterEnergyStressInterventionValidationError::CommunitySetMismatch],
        ));
    }

    let mut communities = Vec::with_capacity(baseline_floor_by_community.len());
    for (community_id, base) in &baseline_floor_by_community {
        let cand = candidate_floor_by_community
            .get(community_id)
            .copied()
            .expect("validated community outcome exists");
        if base.total_beneficiary_weight != cand.total_beneficiary_weight
            || base.service_cell_count != cand.service_cell_count
        {
            return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
                vec![WaterEnergyStressInterventionValidationError::CommunitySetMismatch],
            ));
        }
        let (delta_lower, delta_upper) = delivery_delta_bounds(
            base.floor_compliance_lower_bound,
            base.floor_compliance_upper_bound,
            cand.floor_compliance_lower_bound,
            cand.floor_compliance_upper_bound,
        );
        communities.push(WaterEnergyStressInterventionCommunityComparison {
            community_id: (*community_id).to_owned(),
            service_cell_count: base.service_cell_count,
            beneficiary_weight: base.total_beneficiary_weight,
            baseline_floor_compliance_lower_bound: base.floor_compliance_lower_bound,
            baseline_floor_compliance_upper_bound: base.floor_compliance_upper_bound,
            candidate_floor_compliance_lower_bound: cand.floor_compliance_lower_bound,
            candidate_floor_compliance_upper_bound: cand.floor_compliance_upper_bound,
            floor_compliance_delta_lower_bound: delta_lower,
            floor_compliance_delta_upper_bound: delta_upper,
            possible_protected_floor_regression: cand.floor_compliance_lower_bound
                < base.floor_compliance_upper_bound,
        });
    }

    let mut services = Vec::with_capacity(baseline.services.len());
    let mut possible_regression = communities
        .iter()
        .any(|community| community.possible_protected_floor_regression);
    for base in &baseline.services {
        let Some(cand) = find_matching_service(candidate, &base.service_request_id) else {
            return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
                vec![WaterEnergyStressInterventionValidationError::ServiceAssessmentCountMismatch],
            ));
        };
        if base.service_entity_id != cand.service_entity_id
            || base.community_id != cand.community_id
            || base.beneficiary_weight != cand.beneficiary_weight
            || base.essential_coverage_floor != cand.essential_coverage_floor
            || base.steps.len() != cand.steps.len()
        {
            return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
                vec![WaterEnergyStressInterventionValidationError::StepAlignmentMismatch],
            ));
        }

        let (service_reduction_lower, service_reduction_upper) = delta_bounds(
            base.unmet_demand_volume_lower_bound_m3,
            base.unmet_demand_volume_upper_bound_m3,
            cand.unmet_demand_volume_lower_bound_m3,
            cand.unmet_demand_volume_upper_bound_m3,
        );
        let (service_delivery_delta_lower, service_delivery_delta_upper) = delivery_delta_bounds(
            base.customer_delivered_volume_lower_bound_m3,
            base.customer_delivered_volume_upper_bound_m3,
            cand.customer_delivered_volume_lower_bound_m3,
            cand.customer_delivered_volume_upper_bound_m3,
        );
        let mut step_comparisons = Vec::with_capacity(base.steps.len());
        let mut service_possible_regression =
            possible_floor_regression(base.floor_status, cand.floor_status);
        let mut baseline_service_evidence = BTreeSet::new();
        baseline_service_evidence.extend(base.supporting_evidence_ids.iter().cloned());
        let mut candidate_service_evidence = BTreeSet::new();
        candidate_service_evidence.extend(cand.supporting_evidence_ids.iter().cloned());

        for (base_step, cand_step) in base.steps.iter().zip(cand.steps.iter()) {
            if base_step.step_id != cand_step.step_id
                || base_step.as_of != cand_step.as_of
                || base_step.step_hours != cand_step.step_hours
            {
                return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
                    vec![WaterEnergyStressInterventionValidationError::StepAlignmentMismatch],
                ));
            }
            let floor_regression =
                possible_floor_regression(base_step.floor_status, cand_step.floor_status);
            service_possible_regression |= floor_regression;
            let (step_reduction_lower, step_reduction_upper) = delta_bounds(
                base_step.unmet_demand_volume_lower_bound_m3,
                base_step.unmet_demand_volume_upper_bound_m3,
                cand_step.unmet_demand_volume_lower_bound_m3,
                cand_step.unmet_demand_volume_upper_bound_m3,
            );
            let (step_delivery_lower, step_delivery_upper) = delivery_delta_bounds(
                base_step.customer_delivered_volume_lower_bound_m3,
                base_step.customer_delivered_volume_upper_bound_m3,
                cand_step.customer_delivered_volume_lower_bound_m3,
                cand_step.customer_delivered_volume_upper_bound_m3,
            );
            baseline_service_evidence.extend(base_step.supporting_evidence_ids.iter().cloned());
            candidate_service_evidence.extend(cand_step.supporting_evidence_ids.iter().cloned());
            step_comparisons.push(WaterEnergyStressInterventionStepComparison {
                service_request_id: base.service_request_id.clone(),
                community_id: base.community_id.clone(),
                step_id: base_step.step_id.clone(),
                as_of: base_step.as_of.clone(),
                step_hours: base_step.step_hours,
                baseline_state: base_step.service_state,
                candidate_state: cand_step.service_state,
                baseline_floor_status: base_step.floor_status,
                candidate_floor_status: cand_step.floor_status,
                possible_protected_floor_regression: floor_regression,
                baseline_coverage_lower_bound: base_step.coverage_lower_bound,
                baseline_coverage_upper_bound: base_step.coverage_upper_bound,
                candidate_coverage_lower_bound: cand_step.coverage_lower_bound,
                candidate_coverage_upper_bound: cand_step.coverage_upper_bound,
                baseline_unmet_demand_lower_bound_m3: base_step.unmet_demand_volume_lower_bound_m3,
                baseline_unmet_demand_upper_bound_m3: base_step.unmet_demand_volume_upper_bound_m3,
                candidate_unmet_demand_lower_bound_m3: cand_step.unmet_demand_volume_lower_bound_m3,
                candidate_unmet_demand_upper_bound_m3: cand_step.unmet_demand_volume_upper_bound_m3,
                unmet_demand_reduction_lower_bound_m3: step_reduction_lower,
                unmet_demand_reduction_upper_bound_m3: step_reduction_upper,
                baseline_customer_delivery_lower_bound_m3: base_step
                    .customer_delivered_volume_lower_bound_m3,
                baseline_customer_delivery_upper_bound_m3: base_step
                    .customer_delivered_volume_upper_bound_m3,
                candidate_customer_delivery_lower_bound_m3: cand_step
                    .customer_delivered_volume_lower_bound_m3,
                candidate_customer_delivery_upper_bound_m3: cand_step
                    .customer_delivered_volume_upper_bound_m3,
                customer_delivery_delta_lower_bound_m3: step_delivery_lower,
                customer_delivery_delta_upper_bound_m3: step_delivery_upper,
                baseline_supporting_evidence_ids: base_step.supporting_evidence_ids.clone(),
                candidate_supporting_evidence_ids: cand_step.supporting_evidence_ids.clone(),
            });
        }

        possible_regression |= service_possible_regression;
        services.push(WaterEnergyStressInterventionServiceComparison {
            service_request_id: base.service_request_id.clone(),
            service_entity_id: base.service_entity_id.clone(),
            community_id: base.community_id.clone(),
            beneficiary_weight: base.beneficiary_weight,
            essential_coverage_floor: base.essential_coverage_floor,
            baseline_floor_status: base.floor_status,
            candidate_floor_status: cand.floor_status,
            possible_protected_floor_regression: service_possible_regression,
            baseline_unmet_demand_lower_bound_m3: base.unmet_demand_volume_lower_bound_m3,
            baseline_unmet_demand_upper_bound_m3: base.unmet_demand_volume_upper_bound_m3,
            candidate_unmet_demand_lower_bound_m3: cand.unmet_demand_volume_lower_bound_m3,
            candidate_unmet_demand_upper_bound_m3: cand.unmet_demand_volume_upper_bound_m3,
            unmet_demand_reduction_lower_bound_m3: service_reduction_lower,
            unmet_demand_reduction_upper_bound_m3: service_reduction_upper,
            baseline_customer_delivery_lower_bound_m3: base
                .customer_delivered_volume_lower_bound_m3,
            baseline_customer_delivery_upper_bound_m3: base
                .customer_delivered_volume_upper_bound_m3,
            candidate_customer_delivery_lower_bound_m3: cand
                .customer_delivered_volume_lower_bound_m3,
            candidate_customer_delivery_upper_bound_m3: cand
                .customer_delivered_volume_upper_bound_m3,
            customer_delivery_delta_lower_bound_m3: service_delivery_delta_lower,
            customer_delivery_delta_upper_bound_m3: service_delivery_delta_upper,
            baseline_maximum_normalized_deficit_lower_bound: base
                .maximum_normalized_deficit_lower_bound,
            baseline_maximum_normalized_deficit_upper_bound: base
                .maximum_normalized_deficit_upper_bound,
            candidate_maximum_normalized_deficit_lower_bound: cand
                .maximum_normalized_deficit_lower_bound,
            candidate_maximum_normalized_deficit_upper_bound: cand
                .maximum_normalized_deficit_upper_bound,
            steps: step_comparisons,
            baseline_supporting_evidence_ids: baseline_service_evidence.into_iter().collect(),
            candidate_supporting_evidence_ids: candidate_service_evidence.into_iter().collect(),
        });
    }

    communities.sort_by(|left, right| left.community_id.cmp(&right.community_id));
    services.sort_by(|left, right| left.service_request_id.cmp(&right.service_request_id));

    let effect = effect_from_bounds(
        unmet_reduction_lower,
        unmet_reduction_upper,
        possible_regression,
    );
    let numbers = [
        unmet_reduction_lower,
        unmet_reduction_upper,
        delivery_delta_lower,
        delivery_delta_upper,
        floor_delta_lower,
        floor_delta_upper,
    ];
    if numbers.iter().any(|value| !value.is_finite()) {
        return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
            vec![WaterEnergyStressInterventionValidationError::NonFiniteDerivedValue],
        ));
    }
    if unmet_reduction_lower > unmet_reduction_upper
        || delivery_delta_lower > delivery_delta_upper
        || floor_delta_lower > floor_delta_upper
    {
        return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
            vec![WaterEnergyStressInterventionValidationError::BoundsInconsistent],
        ));
    }

    Ok(WaterEnergyStressInterventionCaseComparison {
        scenario_id: baseline.id.clone(),
        kind: baseline.kind,
        rationale: baseline.rationale.clone(),
        overrides_applied: baseline.overrides_applied.clone(),
        effect,
        possible_protected_floor_regression: possible_regression,
        baseline_total_unmet_demand_lower_bound_m3: baseline
            .total_unmet_demand_volume_lower_bound_m3,
        baseline_total_unmet_demand_upper_bound_m3: baseline
            .total_unmet_demand_volume_upper_bound_m3,
        candidate_total_unmet_demand_lower_bound_m3: candidate
            .total_unmet_demand_volume_lower_bound_m3,
        candidate_total_unmet_demand_upper_bound_m3: candidate
            .total_unmet_demand_volume_upper_bound_m3,
        unmet_demand_reduction_lower_bound_m3: unmet_reduction_lower,
        unmet_demand_reduction_upper_bound_m3: unmet_reduction_upper,
        baseline_customer_delivery_lower_bound_m3: baseline
            .services
            .iter()
            .map(|service| service.customer_delivered_volume_lower_bound_m3)
            .sum(),
        baseline_customer_delivery_upper_bound_m3: baseline
            .services
            .iter()
            .map(|service| service.customer_delivered_volume_upper_bound_m3)
            .sum(),
        candidate_customer_delivery_lower_bound_m3: candidate
            .services
            .iter()
            .map(|service| service.customer_delivered_volume_lower_bound_m3)
            .sum(),
        candidate_customer_delivery_upper_bound_m3: candidate
            .services
            .iter()
            .map(|service| service.customer_delivered_volume_upper_bound_m3)
            .sum(),
        customer_delivery_delta_lower_bound_m3: delivery_delta_lower,
        customer_delivery_delta_upper_bound_m3: delivery_delta_upper,
        baseline_floor_compliance_lower_bound: baseline.overall_floor_compliance_lower_bound,
        baseline_floor_compliance_upper_bound: baseline.overall_floor_compliance_upper_bound,
        candidate_floor_compliance_lower_bound: candidate.overall_floor_compliance_lower_bound,
        candidate_floor_compliance_upper_bound: candidate.overall_floor_compliance_upper_bound,
        floor_compliance_delta_lower_bound: floor_delta_lower,
        floor_compliance_delta_upper_bound: floor_delta_upper,
        baseline_maximum_normalized_deficit_lower_bound: baseline
            .maximum_normalized_deficit_lower_bound,
        baseline_maximum_normalized_deficit_upper_bound: baseline
            .maximum_normalized_deficit_upper_bound,
        candidate_maximum_normalized_deficit_lower_bound: candidate
            .maximum_normalized_deficit_lower_bound,
        candidate_maximum_normalized_deficit_upper_bound: candidate
            .maximum_normalized_deficit_upper_bound,
        communities,
        services,
        baseline_supporting_evidence_ids: baseline_support.into_iter().collect(),
        candidate_supporting_evidence_ids: candidate_support.into_iter().collect(),
    })
}

fn scenario_tolerance(case: &WaterEnergyStressInterventionCaseComparison) -> f64 {
    f64::EPSILON
        * 256.0
        * case
            .unmet_demand_reduction_lower_bound_m3
            .abs()
            .max(case.unmet_demand_reduction_upper_bound_m3.abs())
            .max(1.0)
}

fn overall_effect(
    cases: &[WaterEnergyStressInterventionCaseComparison],
) -> WaterEnergyStressInterventionEffect {
    if cases
        .iter()
        .any(|case| case.possible_protected_floor_regression)
    {
        return WaterEnergyStressInterventionEffect::ProtectedFloorRegression;
    }
    if cases.is_empty() {
        return WaterEnergyStressInterventionEffect::Indeterminate;
    }
    let all_nonworsening = cases
        .iter()
        .all(|case| case.unmet_demand_reduction_lower_bound_m3 >= -scenario_tolerance(case));
    let any_robust_improvement = cases
        .iter()
        .any(|case| case.unmet_demand_reduction_lower_bound_m3 > scenario_tolerance(case));
    if all_nonworsening && any_robust_improvement {
        return WaterEnergyStressInterventionEffect::RobustImprovement;
    }
    let all_nonimproving = cases
        .iter()
        .all(|case| case.unmet_demand_reduction_upper_bound_m3 <= scenario_tolerance(case));
    let any_robust_regression = cases
        .iter()
        .any(|case| case.unmet_demand_reduction_upper_bound_m3 < -scenario_tolerance(case));
    if all_nonimproving && any_robust_regression {
        return WaterEnergyStressInterventionEffect::RobustRegression;
    }
    if cases.iter().all(|case| {
        case.unmet_demand_reduction_lower_bound_m3 >= -scenario_tolerance(case)
            && case.unmet_demand_reduction_upper_bound_m3 <= scenario_tolerance(case)
    }) {
        return WaterEnergyStressInterventionEffect::NoMaterialChange;
    }
    let has_improvement = cases
        .iter()
        .any(|case| case.unmet_demand_reduction_lower_bound_m3 > scenario_tolerance(case));
    let has_regression = cases
        .iter()
        .any(|case| case.unmet_demand_reduction_upper_bound_m3 < -scenario_tolerance(case));
    if has_improvement && has_regression {
        WaterEnergyStressInterventionEffect::MixedEffects
    } else {
        WaterEnergyStressInterventionEffect::Indeterminate
    }
}

/// Compare two interventions over the same declared scenarios and protected service floors.
///
/// One evidence corpus is validated/indexed once and reused across both sweeps. A result is
/// marked RobustImprovement only if every declared case has a nonnegative lower bound on
/// reduced unmet demand and at least one case improves robustly, with no possible protected
/// service-floor regression at any service or sampled interval. It does not prove execution,
/// physical delivery, calibrated probabilities, hydraulic accuracy or causal field benefits.
pub fn compare_water_energy_stress_interventions(
    request: &WaterEnergyStressInterventionComparisonRequest,
    evidence_records: &[EvidenceRecord],
) -> Result<
    WaterEnergyStressInterventionComparisonAssessment,
    WaterEnergyStressInterventionComparisonError,
> {
    validate_paired_requests(request)
        .map_err(WaterEnergyStressInterventionComparisonError::InvalidInput)?;
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(WaterEnergyStressInterventionComparisonError::EvidenceValidation)?;
    let baseline = assess_water_energy_stress_sweep_indexed(&request.baseline, &evidence_by_id)
        .map_err(WaterEnergyStressInterventionComparisonError::BaselineAssessment)?;
    let candidate = assess_water_energy_stress_sweep_indexed(&request.candidate, &evidence_by_id)
        .map_err(WaterEnergyStressInterventionComparisonError::CandidateAssessment)?;

    let mut baseline_cases: Vec<&WaterEnergyStressCaseAssessment> = vec![&baseline.baseline];
    baseline_cases.extend(baseline.cases.iter());
    let mut candidate_cases: Vec<&WaterEnergyStressCaseAssessment> = vec![&candidate.baseline];
    candidate_cases.extend(candidate.cases.iter());
    baseline_cases.sort_by(|left, right| left.id.cmp(&right.id));
    candidate_cases.sort_by(|left, right| left.id.cmp(&right.id));
    if baseline_cases.len() != candidate_cases.len() {
        return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
            vec![WaterEnergyStressInterventionValidationError::FailureScenarioSetMismatch],
        ));
    }

    let mut cases = Vec::with_capacity(baseline_cases.len());
    for (base, cand) in baseline_cases.into_iter().zip(candidate_cases.into_iter()) {
        cases.push(compare_case(base, cand)?);
    }
    let effect = overall_effect(&cases);
    let mut baseline_evidence = BTreeSet::new();
    let mut candidate_evidence = BTreeSet::new();
    for case in &cases {
        baseline_evidence.extend(case.baseline_supporting_evidence_ids.iter().cloned());
        candidate_evidence.extend(case.candidate_supporting_evidence_ids.iter().cloned());
    }

    let worst_case_unmet_lower = cases
        .iter()
        .map(|case| case.unmet_demand_reduction_lower_bound_m3)
        .fold(f64::INFINITY, f64::min);
    let worst_case_unmet_upper = cases
        .iter()
        .map(|case| case.unmet_demand_reduction_upper_bound_m3)
        .fold(f64::NEG_INFINITY, f64::max);
    let worst_case_delivery_lower = cases
        .iter()
        .map(|case| case.customer_delivery_delta_lower_bound_m3)
        .fold(f64::INFINITY, f64::min);
    let worst_case_delivery_upper = cases
        .iter()
        .map(|case| case.customer_delivery_delta_upper_bound_m3)
        .fold(f64::NEG_INFINITY, f64::max);
    let worst_case_floor_lower = cases
        .iter()
        .map(|case| case.floor_compliance_delta_lower_bound)
        .fold(f64::INFINITY, f64::min);
    let worst_case_floor_upper = cases
        .iter()
        .map(|case| case.floor_compliance_delta_upper_bound)
        .fold(f64::NEG_INFINITY, f64::max);
    let scalars = [
        worst_case_unmet_lower,
        worst_case_unmet_upper,
        worst_case_delivery_lower,
        worst_case_delivery_upper,
        worst_case_floor_lower,
        worst_case_floor_upper,
    ];
    if scalars.iter().any(|value| !value.is_finite()) {
        return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
            vec![WaterEnergyStressInterventionValidationError::NonFiniteDerivedValue],
        ));
    }
    if worst_case_unmet_lower > worst_case_unmet_upper
        || worst_case_delivery_lower > worst_case_delivery_upper
        || worst_case_floor_lower > worst_case_floor_upper
    {
        return Err(WaterEnergyStressInterventionComparisonError::InvalidInput(
            vec![WaterEnergyStressInterventionValidationError::BoundsInconsistent],
        ));
    }

    Ok(WaterEnergyStressInterventionComparisonAssessment {
        schema_version: WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION.to_owned(),
        comparison_id: request.id.clone(),
        baseline_experiment_id: baseline.experiment_id,
        candidate_experiment_id: candidate.experiment_id,
        service_count: baseline.service_count,
        declared_failure_scenario_count: baseline.declared_failure_scenario_count,
        compared_case_count: cases.len(),
        exhaustive_over_paired_declared_scenarios: baseline.exhaustive_over_declared_scenarios
            && candidate.exhaustive_over_declared_scenarios
            && cases.len() == baseline.declared_failure_scenario_count + 1,
        total_paired_service_step_assessments: baseline
            .total_service_step_assessments
            .checked_add(candidate.total_service_step_assessments)
            .ok_or_else(|| {
                WaterEnergyStressInterventionComparisonError::InvalidInput(vec![
                    WaterEnergyStressInterventionValidationError::NonFiniteDerivedValue,
                ])
            })?,
        effect,
        possible_protected_floor_regression: cases
            .iter()
            .any(|case| case.possible_protected_floor_regression),
        worst_case_unmet_demand_reduction_lower_bound_m3: worst_case_unmet_lower,
        worst_case_unmet_demand_reduction_upper_bound_m3: worst_case_unmet_upper,
        worst_case_customer_delivery_delta_lower_bound_m3: worst_case_delivery_lower,
        worst_case_customer_delivery_delta_upper_bound_m3: worst_case_delivery_upper,
        worst_case_floor_compliance_delta_lower_bound: worst_case_floor_lower,
        worst_case_floor_compliance_delta_upper_bound: worst_case_floor_upper,
        cases,
        baseline_supporting_evidence_ids: baseline_evidence.into_iter().collect(),
        candidate_supporting_evidence_ids: candidate_evidence.into_iter().collect(),
        interpretation_boundary:
            WaterEnergyStressInterventionInterpretationBoundary::
                ModeledCounterfactualNotCausalEvidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::service_function::FunctionMeasurement;
    use crate::service_water::{DIMENSIONLESS_UNIT, WATER_RATE_UNIT, WATER_VOLUME_UNIT};
    use crate::service_water_energy_coupling::{
        ELECTRIC_POWER_UNIT, PUMP_SPECIFIC_ENERGY_UNIT, PowerMeasurementLocation,
        WATER_ENERGY_COUPLING_SCHEMA_VERSION, WaterEnergyCouplingRequest,
        WaterEnergyCouplingStepInput,
    };
    use crate::service_water_energy_stress::{
        ProtectedWaterServiceFloor, WATER_ENERGY_STRESS_SCHEMA_VERSION,
        WaterEnergyDeclaredFailureScenario, WaterEnergyFailureScenarioKind,
        WaterEnergyFailureTarget, WaterEnergyMeasurementOverride, WaterEnergyStressSweepRequest,
    };
    use crate::service_water_trajectory::{
        WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION, WaterServiceTrajectory, WaterServiceTrajectoryStep,
    };
    use crate::spatial_economic::Quantity;

    const TIMES: [&str; 2] = ["2026-10-16T09:00:00Z", "2026-10-16T10:00:00Z"];

    fn measurement(entity: &str, value: f64, unit: &str) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity.to_owned(),
            quantity: Quantity {
                value,
                unit: unit.to_owned(),
                uncertainty_abs: None,
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic comparison input.".to_owned(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn service_request(id: &str, service_entity: &str, storage: f64) -> WaterEnergyCouplingRequest {
        let water_steps = TIMES
            .iter()
            .enumerate()
            .map(|(index, as_of)| WaterServiceTrajectoryStep {
                id: format!("step-{index}"),
                as_of: (*as_of).to_owned(),
                step_hours: 1.0,
                source_flow_rate: measurement("source", 100.0, WATER_RATE_UNIT),
                pump_capacity: measurement("pump", 100.0, WATER_RATE_UNIT),
                treatment_capacity: measurement("treatment", 100.0, WATER_RATE_UNIT),
                distribution_capacity: measurement("distribution", 120.0, WATER_RATE_UNIT),
                demand_rate: measurement(service_entity, 100.0, WATER_RATE_UNIT),
                distribution_loss_fraction: measurement("distribution", 0.0, DIMENSIONLESS_UNIT),
            })
            .collect();
        let energy_steps = TIMES
            .iter()
            .enumerate()
            .map(|(index, as_of)| WaterEnergyCouplingStepInput {
                step_id: format!("step-{index}"),
                as_of: (*as_of).to_owned(),
                operating_regime_id: "pump-regime".to_owned(),
                power_measurement_location: PowerMeasurementLocation::PumpTerminals,
                available_power: measurement("pump", 50.0, ELECTRIC_POWER_UNIT),
                specific_energy: measurement("pump-regime", 0.5, PUMP_SPECIFIC_ENERGY_UNIT),
            })
            .collect();
        WaterEnergyCouplingRequest {
            schema_version: WATER_ENERGY_COUPLING_SCHEMA_VERSION.to_owned(),
            id: id.to_owned(),
            trajectory: WaterServiceTrajectory {
                schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
                id: format!("trajectory-{id}"),
                service_entity_id: service_entity.to_owned(),
                initial_storage: measurement("tank", storage, WATER_VOLUME_UNIT),
                storage_capacity: measurement("tank", 200.0, WATER_VOLUME_UNIT),
                steps: water_steps,
            },
            steps: energy_steps,
        }
    }

    pub(super) fn sweep_request(id: &str, storage: f64) -> WaterEnergyStressSweepRequest {
        let overrides = TIMES
            .iter()
            .enumerate()
            .map(|(index, _)| WaterEnergyMeasurementOverride {
                service_request_id: "cell-a".to_owned(),
                step_id: format!("step-{index}"),
                target: WaterEnergyFailureTarget::PumpAvailablePower,
                forced_value: 0.0,
                forced_uncertainty_abs: None,
            })
            .collect();
        WaterEnergyStressSweepRequest {
            schema_version: WATER_ENERGY_STRESS_SCHEMA_VERSION.to_owned(),
            id: id.to_owned(),
            baseline_services: vec![service_request("cell-a", "community-a", storage)],
            failure_scenarios: vec![WaterEnergyDeclaredFailureScenario {
                id: "grid-outage".to_owned(),
                kind: WaterEnergyFailureScenarioKind::CommonCause,
                rationale: "Same synthetic pump-terminal power failure.".to_owned(),
                overrides,
            }],
            protected_service_floors: vec![ProtectedWaterServiceFloor {
                service_request_id: "cell-a".to_owned(),
                community_id: "community-a".to_owned(),
                beneficiary_weight: 100,
                essential_coverage_floor: 0.8,
            }],
            max_failure_scenarios: 10,
            max_service_step_assessments: 100,
        }
    }

    fn request(
        baseline_storage: f64,
        candidate_storage: f64,
    ) -> WaterEnergyStressInterventionComparisonRequest {
        WaterEnergyStressInterventionComparisonRequest {
            schema_version: WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION.to_owned(),
            id: "paired-sweep-comparison".to_owned(),
            baseline: sweep_request("baseline-experiment", baseline_storage),
            candidate: sweep_request("candidate-experiment", candidate_storage),
        }
    }

    #[test]
    fn storage_intervention_is_robustly_better_across_the_same_failure_suite() {
        let input = request(0.0, 200.0);
        let output = compare_water_energy_stress_interventions(&input, &[])
            .expect("aligned baseline/candidate sweeps");
        assert_eq!(
            output.effect,
            WaterEnergyStressInterventionEffect::RobustImprovement
        );
        assert_eq!(output.declared_failure_scenario_count, 1);
        assert_eq!(output.compared_case_count, 2);
        assert_eq!(
            output
                .cases
                .iter()
                .find(|case| case.scenario_id == "grid-outage")
                .unwrap()
                .effect,
            WaterEnergyStressInterventionEffect::RobustImprovement
        );
        assert!(!output.possible_protected_floor_regression);
        assert!(output.worst_case_unmet_demand_reduction_lower_bound_m3 >= 0.0);
        assert_eq!(
            output.interpretation_boundary,
            WaterEnergyStressInterventionInterpretationBoundary::
                ModeledCounterfactualNotCausalEvidence
        );
    }

    #[test]
    fn protected_floor_regression_overrides_aggregate_delivery_improvement() {
        let input = request(200.0, 0.0);
        let output = compare_water_energy_stress_interventions(&input, &[])
            .expect("aligned baseline/candidate sweeps");
        assert_eq!(
            output.effect,
            WaterEnergyStressInterventionEffect::ProtectedFloorRegression
        );
        assert!(output.possible_protected_floor_regression);
        let outage = output
            .cases
            .iter()
            .find(|case| case.scenario_id == "grid-outage")
            .unwrap();
        assert!(outage.possible_protected_floor_regression);
        assert!(outage.customer_delivery_delta_upper_bound_m3 < 0.0);
    }

    #[test]
    fn rejects_scenario_set_mismatch() {
        let mut input = request(0.0, 200.0);
        input.candidate.failure_scenarios[0].id = "other-outage".to_owned();
        let result = compare_water_energy_stress_interventions(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyStressInterventionComparisonError::InvalidInput(errors))
                if errors.contains(
                    &WaterEnergyStressInterventionValidationError::FailureScenarioSetMismatch,
                )
        ));
    }

    #[test]
    fn rejects_different_protected_floors() {
        let mut input = request(0.0, 200.0);
        input.candidate.protected_service_floors[0].essential_coverage_floor = 0.7;
        let result = compare_water_energy_stress_interventions(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyStressInterventionComparisonError::InvalidInput(errors))
                if errors.contains(
                    &WaterEnergyStressInterventionValidationError::ProtectedFloorDefinitionMismatch,
                )
        ));
    }

    #[test]
    fn overlapping_delta_bounds_are_indeterminate() {
        let mut input = request(0.0, 0.0);
        for step in &mut input.baseline.baseline_services[0].steps {
            step.available_power.quantity.value = 20.0;
        }
        for step in &mut input.candidate.baseline_services[0].steps {
            step.available_power.quantity.value = 20.0;
            step.available_power.quantity.uncertainty_abs = Some(10.0);
        }
        let output = compare_water_energy_stress_interventions(&input, &[])
            .expect("overlapping intervals are evaluable");
        assert_eq!(
            output.effect,
            WaterEnergyStressInterventionEffect::Indeterminate
        );
    }

    #[test]
    fn reordering_scenario_and_override_inputs_preserves_output() {
        let input = request(0.0, 200.0);
        let mut shuffled = input.clone();
        shuffled.baseline.failure_scenarios[0].overrides.reverse();
        shuffled.candidate.failure_scenarios[0].overrides.reverse();
        let first = compare_water_energy_stress_interventions(&input, &[]).expect("original sweep");
        let second =
            compare_water_energy_stress_interventions(&shuffled, &[]).expect("permuted sweep");
        assert_eq!(first, second);
    }

    #[test]
    fn duplicate_evidence_ids_are_rejected_by_one_shared_index() {
        let input = request(0.0, 200.0);
        let duplicate = crate::spatial_economic::EvidenceRecord {
            id: "same-id".to_owned(),
            schema_version: crate::spatial_economic::EVIDENCE_SCHEMA_VERSION.to_owned(),
            kind: crate::spatial_economic::EvidenceKind::Observation,
            state: crate::spatial_economic::EvidenceState::Asserted,
            issuer_id: "did:example:stress-comparison".to_owned(),
            subject_id: "source".to_owned(),
            spatial: crate::spatial_economic::SpatialContext {
                entity_id: "source".to_owned(),
                geometry_uri: None,
                crs: None,
            },
            time: crate::spatial_economic::TimeContext {
                occurred_at: TIMES[0].to_owned(),
                recorded_at: TIMES[0].to_owned(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic duplicated record.".to_owned(),
            quantity: Some(Quantity {
                value: 100.0,
                unit: WATER_RATE_UNIT.to_owned(),
                uncertainty_abs: None,
            }),
            money: None,
            provenance: crate::spatial_economic::Provenance {
                method: "synthetic_test".to_owned(),
                source_artifacts: Vec::new(),
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:synthetic".to_owned(),
        };
        let result =
            compare_water_energy_stress_interventions(&input, &[duplicate.clone(), duplicate]);
        assert!(matches!(
            result,
            Err(WaterEnergyStressInterventionComparisonError::EvidenceValidation(
                ServiceFunctionAssessmentError::InvalidInput(errors)
            )) if errors.contains(
                &crate::service_function::ServiceFunctionValidationError::DuplicateEvidenceRecordId
            )
        ));
    }
}

/// Versioned request for comparing many intervention candidates against one fixed
/// baseline and one fixed declared joint-failure suite.
pub const WATER_ENERGY_STRESS_INTERVENTION_BATCH_SCHEMA_VERSION: &str = "0.1.0";
pub const MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_CANDIDATES: usize = 1_000;
pub const MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_SERVICE_STEP_ASSESSMENTS: usize = 1_000_000;
pub const MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_OVERRIDES: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionBatchCandidate {
    /// Stable caller-defined candidate key; used to make output order deterministic.
    pub candidate_id: String,
    pub experiment: WaterEnergyStressSweepRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionBatchRequest {
    pub schema_version: String,
    pub id: String,
    pub baseline: WaterEnergyStressSweepRequest,
    pub candidates: Vec<WaterEnergyStressInterventionBatchCandidate>,
    /// Caller budget across baseline plus every candidate sweep; hard-capped by this module.
    pub max_total_service_step_assessments: usize,
    /// Caller budget across the baseline and every candidate's declared scenario overrides.
    pub max_total_declared_failure_overrides: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionCandidateSummary {
    pub candidate_id: String,
    pub candidate_experiment_id: String,
    pub declared_failure_scenario_count: usize,
    /// One nominal case plus all paired declared failure cases.
    pub compared_case_count: usize,
    pub exhaustive_over_paired_declared_scenarios: bool,
    pub total_paired_service_step_assessments: usize,
    pub effect: WaterEnergyStressInterventionEffect,
    pub possible_protected_floor_regression: bool,
    pub worst_case_unmet_demand_reduction_lower_bound_m3: f64,
    pub worst_case_unmet_demand_reduction_upper_bound_m3: f64,
    pub worst_case_customer_delivery_delta_lower_bound_m3: f64,
    pub worst_case_customer_delivery_delta_upper_bound_m3: f64,
    pub worst_case_floor_compliance_delta_lower_bound: f64,
    pub worst_case_floor_compliance_delta_upper_bound: f64,
    pub baseline_supporting_evidence_ids: Vec<String>,
    pub candidate_supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressInterventionBatchAssessment {
    pub schema_version: String,
    pub batch_id: String,
    pub baseline_experiment_id: String,
    pub requested_candidate_count: usize,
    pub evaluated_candidate_count: usize,
    /// The implementation evaluates the baseline exactly once for the whole batch.
    pub baseline_sweep_evaluation_count: usize,
    pub candidate_sweep_evaluation_count: usize,
    pub total_service_step_assessments: usize,
    pub candidates: Vec<WaterEnergyStressInterventionCandidateSummary>,
    pub baseline_supporting_evidence_ids: Vec<String>,
    pub interpretation_boundary: WaterEnergyStressInterventionInterpretationBoundary,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaterEnergyStressInterventionBatchError {
    InvalidInput(Vec<WaterEnergyStressInterventionValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
    BaselineAssessment(WaterEnergyStressSweepError),
    CandidateAssessment {
        candidate_id: String,
        source: WaterEnergyStressSweepError,
    },
    Comparison {
        candidate_id: String,
        source: WaterEnergyStressInterventionComparisonError,
    },
}

/// Compare multiple candidates over a single declared scenario suite. Unlike looping over
/// the single-candidate comparator, this evaluates and validates the baseline only once,
/// shares one evidence index, and returns compact summaries rather than the full per-step
/// trace for every candidate. Candidate and baseline traces are still calculated internally
/// for correctness; consumers needing a detailed audit trace can use the single-candidate
/// API for selected candidates after shortlisting.
///
/// This is exhaustive over the request's declared failure cases only. It does not attach
/// probabilities to scenarios, establish hydraulic accuracy, or prove causal effects.
pub fn compare_water_energy_stress_intervention_batch(
    request: &WaterEnergyStressInterventionBatchRequest,
    evidence_records: &[EvidenceRecord],
) -> Result<WaterEnergyStressInterventionBatchAssessment, WaterEnergyStressInterventionBatchError> {
    let mut errors = Vec::new();
    if request.schema_version != WATER_ENERGY_STRESS_INTERVENTION_BATCH_SCHEMA_VERSION {
        errors.push(WaterEnergyStressInterventionValidationError::UnsupportedSchemaVersion);
    }
    if request.id.trim().is_empty() {
        errors.push(WaterEnergyStressInterventionValidationError::EmptyComparisonId);
    }
    if request.candidates.is_empty() {
        errors.push(WaterEnergyStressInterventionValidationError::EmptyCandidateSet);
    }
    if request.candidates.len() > MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_CANDIDATES {
        errors.push(WaterEnergyStressInterventionValidationError::CandidateCountExceeded);
    }
    if request.max_total_service_step_assessments == 0
        || request.max_total_service_step_assessments
            > MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_SERVICE_STEP_ASSESSMENTS
    {
        errors.push(WaterEnergyStressInterventionValidationError::InvalidBatchAssessmentBudget);
    }
    if request.max_total_declared_failure_overrides == 0
        || request.max_total_declared_failure_overrides
            > MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_OVERRIDES
    {
        errors.push(WaterEnergyStressInterventionValidationError::InvalidBatchOverrideBudget);
    }

    let mut candidate_ids = BTreeSet::new();
    let mut experiment_ids = BTreeSet::new();
    for candidate in &request.candidates {
        if candidate.candidate_id.trim().is_empty() {
            errors.push(WaterEnergyStressInterventionValidationError::EmptyCandidateId);
        } else if !candidate_ids.insert(candidate.candidate_id.clone()) {
            errors.push(WaterEnergyStressInterventionValidationError::DuplicateCandidateId);
        }
        if !experiment_ids.insert(candidate.experiment.id.clone()) {
            errors
                .push(WaterEnergyStressInterventionValidationError::DuplicateCandidateExperimentId);
        }
    }
    if !errors.is_empty() {
        errors.sort();
        errors.dedup();
        return Err(WaterEnergyStressInterventionBatchError::InvalidInput(
            errors,
        ));
    }

    // Enforce aggregate budgets before evaluating any candidate. Individual sweep budgets
    // alone do not constrain work multiplied across a large candidate set.
    let service_count = request.baseline.baseline_services.len();
    let steps_per_service = request
        .baseline
        .baseline_services
        .first()
        .map(|service| service.trajectory.steps.len())
        .unwrap_or(0);
    let cases_per_sweep = request
        .baseline
        .failure_scenarios
        .len()
        .checked_add(1)
        .ok_or_else(|| {
            WaterEnergyStressInterventionBatchError::InvalidInput(vec![
                WaterEnergyStressInterventionValidationError::ArithmeticOverflow,
            ])
        })?;
    let steps_per_sweep = service_count
        .checked_mul(steps_per_service)
        .and_then(|value| value.checked_mul(cases_per_sweep))
        .ok_or_else(|| {
            WaterEnergyStressInterventionBatchError::InvalidInput(vec![
                WaterEnergyStressInterventionValidationError::ArithmeticOverflow,
            ])
        })?;
    let all_sweep_count = request.candidates.len().checked_add(1).ok_or_else(|| {
        WaterEnergyStressInterventionBatchError::InvalidInput(vec![
            WaterEnergyStressInterventionValidationError::ArithmeticOverflow,
        ])
    })?;
    let estimated_total_step_assessments = steps_per_sweep
        .checked_mul(all_sweep_count)
        .ok_or_else(|| {
            WaterEnergyStressInterventionBatchError::InvalidInput(vec![
                WaterEnergyStressInterventionValidationError::ArithmeticOverflow,
            ])
        })?;
    if estimated_total_step_assessments > request.max_total_service_step_assessments
        || estimated_total_step_assessments
            > MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_SERVICE_STEP_ASSESSMENTS
    {
        return Err(WaterEnergyStressInterventionBatchError::InvalidInput(vec![
            WaterEnergyStressInterventionValidationError::BatchAssessmentBudgetExceeded,
        ]));
    }

    let mut total_override_count = 0usize;
    for sweep in std::iter::once(&request.baseline).chain(
        request
            .candidates
            .iter()
            .map(|candidate| &candidate.experiment),
    ) {
        let sweep_overrides = sweep
            .failure_scenarios
            .iter()
            .try_fold(0usize, |sum, scenario| {
                sum.checked_add(scenario.overrides.len())
            })
            .ok_or_else(|| {
                WaterEnergyStressInterventionBatchError::InvalidInput(vec![
                    WaterEnergyStressInterventionValidationError::ArithmeticOverflow,
                ])
            })?;
        total_override_count = total_override_count
            .checked_add(sweep_overrides)
            .ok_or_else(|| {
                WaterEnergyStressInterventionBatchError::InvalidInput(vec![
                    WaterEnergyStressInterventionValidationError::ArithmeticOverflow,
                ])
            })?;
    }
    if total_override_count > request.max_total_declared_failure_overrides
        || total_override_count > MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_OVERRIDES
    {
        return Err(WaterEnergyStressInterventionBatchError::InvalidInput(vec![
            WaterEnergyStressInterventionValidationError::BatchOverrideBudgetExceeded,
        ]));
    }

    // Validate every candidate against the same baseline and scenario suite before paying
    // for the baseline sweep or any candidate evaluation.
    let mut candidates: Vec<&WaterEnergyStressInterventionBatchCandidate> =
        request.candidates.iter().collect();
    candidates.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    for candidate in &candidates {
        let comparison_id = format!("{}::{}", request.id, candidate.candidate_id);
        validate_paired_sweeps(
            WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION,
            &comparison_id,
            &request.baseline,
            &candidate.experiment,
        )
        .map_err(WaterEnergyStressInterventionBatchError::InvalidInput)?;
    }

    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(WaterEnergyStressInterventionBatchError::EvidenceValidation)?;
    let baseline = assess_water_energy_stress_sweep_indexed(&request.baseline, &evidence_by_id)
        .map_err(WaterEnergyStressInterventionBatchError::BaselineAssessment)?;

    let mut baseline_evidence = BTreeSet::new();
    baseline_evidence.extend(baseline.baseline.supporting_evidence_ids.iter().cloned());
    for case in &baseline.cases {
        baseline_evidence.extend(case.supporting_evidence_ids.iter().cloned());
    }

    let mut total_service_step_assessments = baseline.total_service_step_assessments;
    let mut summaries = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let candidate_assessment =
            assess_water_energy_stress_sweep_indexed(&candidate.experiment, &evidence_by_id)
                .map_err(
                    |source| WaterEnergyStressInterventionBatchError::CandidateAssessment {
                        candidate_id: candidate.candidate_id.clone(),
                        source,
                    },
                )?;
        total_service_step_assessments = total_service_step_assessments
            .checked_add(candidate_assessment.total_service_step_assessments)
            .ok_or_else(|| {
                WaterEnergyStressInterventionBatchError::InvalidInput(vec![
                    WaterEnergyStressInterventionValidationError::ArithmeticOverflow,
                ])
            })?;
        summaries.push(compact_intervention_summary(
            &candidate.candidate_id,
            &baseline,
            &candidate_assessment,
        )?);
    }

    Ok(WaterEnergyStressInterventionBatchAssessment {
        schema_version: WATER_ENERGY_STRESS_INTERVENTION_BATCH_SCHEMA_VERSION.to_owned(),
        batch_id: request.id.clone(),
        baseline_experiment_id: baseline.experiment_id,
        requested_candidate_count: request.candidates.len(),
        evaluated_candidate_count: summaries.len(),
        baseline_sweep_evaluation_count: 1,
        candidate_sweep_evaluation_count: summaries.len(),
        total_service_step_assessments,
        candidates: summaries,
        baseline_supporting_evidence_ids: baseline_evidence.into_iter().collect(),
        interpretation_boundary:
            WaterEnergyStressInterventionInterpretationBoundary::ModeledCounterfactualNotCausalEvidence,
    })
}

fn compact_intervention_summary(
    candidate_id: &str,
    baseline: &crate::service_water_energy_stress::WaterEnergyStressSweepAssessment,
    candidate: &crate::service_water_energy_stress::WaterEnergyStressSweepAssessment,
) -> Result<WaterEnergyStressInterventionCandidateSummary, WaterEnergyStressInterventionBatchError>
{
    let mut baseline_cases: Vec<&WaterEnergyStressCaseAssessment> = vec![&baseline.baseline];
    baseline_cases.extend(baseline.cases.iter());
    let mut candidate_cases: Vec<&WaterEnergyStressCaseAssessment> = vec![&candidate.baseline];
    candidate_cases.extend(candidate.cases.iter());
    baseline_cases.sort_by(|left, right| left.id.cmp(&right.id));
    candidate_cases.sort_by(|left, right| left.id.cmp(&right.id));
    if baseline_cases.len() != candidate_cases.len() {
        return Err(WaterEnergyStressInterventionBatchError::InvalidInput(vec![
            WaterEnergyStressInterventionValidationError::FailureScenarioSetMismatch,
        ]));
    }

    let mut cases = Vec::with_capacity(baseline_cases.len());
    for (base, cand) in baseline_cases.into_iter().zip(candidate_cases.into_iter()) {
        cases.push(compare_case(base, cand).map_err(|source| {
            WaterEnergyStressInterventionBatchError::Comparison {
                candidate_id: candidate_id.to_owned(),
                source,
            }
        })?);
    }
    let worst_unmet_lower = cases
        .iter()
        .map(|case| case.unmet_demand_reduction_lower_bound_m3)
        .fold(f64::INFINITY, f64::min);
    let worst_unmet_upper = cases
        .iter()
        .map(|case| case.unmet_demand_reduction_upper_bound_m3)
        .fold(f64::NEG_INFINITY, f64::max);
    let worst_delivery_lower = cases
        .iter()
        .map(|case| case.customer_delivery_delta_lower_bound_m3)
        .fold(f64::INFINITY, f64::min);
    let worst_delivery_upper = cases
        .iter()
        .map(|case| case.customer_delivery_delta_upper_bound_m3)
        .fold(f64::NEG_INFINITY, f64::max);
    let worst_floor_lower = cases
        .iter()
        .map(|case| case.floor_compliance_delta_lower_bound)
        .fold(f64::INFINITY, f64::min);
    let worst_floor_upper = cases
        .iter()
        .map(|case| case.floor_compliance_delta_upper_bound)
        .fold(f64::NEG_INFINITY, f64::max);
    let scalars = [
        worst_unmet_lower,
        worst_unmet_upper,
        worst_delivery_lower,
        worst_delivery_upper,
        worst_floor_lower,
        worst_floor_upper,
    ];
    if scalars.iter().any(|value| !value.is_finite()) {
        return Err(WaterEnergyStressInterventionBatchError::InvalidInput(vec![
            WaterEnergyStressInterventionValidationError::NonFiniteDerivedValue,
        ]));
    }
    if worst_unmet_lower > worst_unmet_upper
        || worst_delivery_lower > worst_delivery_upper
        || worst_floor_lower > worst_floor_upper
    {
        return Err(WaterEnergyStressInterventionBatchError::InvalidInput(vec![
            WaterEnergyStressInterventionValidationError::BoundsInconsistent,
        ]));
    }

    let mut baseline_evidence = BTreeSet::new();
    let mut candidate_evidence = BTreeSet::new();
    for case in &cases {
        baseline_evidence.extend(case.baseline_supporting_evidence_ids.iter().cloned());
        candidate_evidence.extend(case.candidate_supporting_evidence_ids.iter().cloned());
    }
    Ok(WaterEnergyStressInterventionCandidateSummary {
        candidate_id: candidate_id.to_owned(),
        candidate_experiment_id: candidate.experiment_id.clone(),
        declared_failure_scenario_count: baseline.declared_failure_scenario_count,
        compared_case_count: cases.len(),
        exhaustive_over_paired_declared_scenarios: baseline.exhaustive_over_declared_scenarios
            && candidate.exhaustive_over_declared_scenarios
            && cases.len() == baseline.declared_failure_scenario_count + 1,
        total_paired_service_step_assessments: baseline
            .total_service_step_assessments
            .checked_add(candidate.total_service_step_assessments)
            .ok_or_else(|| {
                WaterEnergyStressInterventionBatchError::InvalidInput(vec![
                    WaterEnergyStressInterventionValidationError::ArithmeticOverflow,
                ])
            })?,
        effect: overall_effect(&cases),
        possible_protected_floor_regression: cases
            .iter()
            .any(|case| case.possible_protected_floor_regression),
        worst_case_unmet_demand_reduction_lower_bound_m3: worst_unmet_lower,
        worst_case_unmet_demand_reduction_upper_bound_m3: worst_unmet_upper,
        worst_case_customer_delivery_delta_lower_bound_m3: worst_delivery_lower,
        worst_case_customer_delivery_delta_upper_bound_m3: worst_delivery_upper,
        worst_case_floor_compliance_delta_lower_bound: worst_floor_lower,
        worst_case_floor_compliance_delta_upper_bound: worst_floor_upper,
        baseline_supporting_evidence_ids: baseline_evidence.into_iter().collect(),
        candidate_supporting_evidence_ids: candidate_evidence.into_iter().collect(),
    })
}

#[cfg(test)]
mod batch_tests {
    use super::*;

    fn request() -> WaterEnergyStressInterventionBatchRequest {
        WaterEnergyStressInterventionBatchRequest {
            schema_version: WATER_ENERGY_STRESS_INTERVENTION_BATCH_SCHEMA_VERSION.to_owned(),
            id: "batch-storage-shortlist".to_owned(),
            baseline: tests::sweep_request("baseline", 0.0),
            max_total_service_step_assessments:
                MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_SERVICE_STEP_ASSESSMENTS,
            max_total_declared_failure_overrides:
                MAX_WATER_ENERGY_STRESS_INTERVENTION_BATCH_OVERRIDES,
            candidates: vec![
                WaterEnergyStressInterventionBatchCandidate {
                    candidate_id: "z-storage-200".to_owned(),
                    experiment: tests::sweep_request("candidate-z", 200.0),
                },
                WaterEnergyStressInterventionBatchCandidate {
                    candidate_id: "a-storage-100".to_owned(),
                    experiment: tests::sweep_request("candidate-a", 100.0),
                },
            ],
        }
    }

    #[test]
    fn evaluates_baseline_once_and_returns_compact_sorted_candidate_summaries() {
        let output =
            compare_water_energy_stress_intervention_batch(&request(), &[]).expect("valid batch");
        assert_eq!(output.baseline_sweep_evaluation_count, 1);
        assert_eq!(output.candidate_sweep_evaluation_count, 2);
        assert_eq!(output.evaluated_candidate_count, 2);
        assert_eq!(output.candidates[0].candidate_id, "a-storage-100");
        assert_eq!(output.candidates[1].candidate_id, "z-storage-200");
        assert!(
            output
                .candidates
                .iter()
                .all(|candidate| candidate.compared_case_count == 2)
        );
        assert!(
            output
                .candidates
                .iter()
                .all(|candidate| candidate.exhaustive_over_paired_declared_scenarios)
        );
    }

    #[test]
    fn candidate_input_permutation_produces_identical_batch_output() {
        let first = request();
        let mut reversed = request();
        reversed.candidates.reverse();
        let left =
            compare_water_energy_stress_intervention_batch(&first, &[]).expect("first valid batch");
        let right = compare_water_energy_stress_intervention_batch(&reversed, &[])
            .expect("permuted valid batch");
        assert_eq!(left, right);
        assert_eq!(
            serde_json::to_vec(&left).expect("serialize left"),
            serde_json::to_vec(&right).expect("serialize right")
        );
    }

    #[test]
    fn duplicate_candidate_ids_fail_closed_before_evaluation() {
        let mut input = request();
        input.candidates[1].candidate_id = input.candidates[0].candidate_id.clone();
        assert!(matches!(
            compare_water_energy_stress_intervention_batch(&input, &[]),
            Err(WaterEnergyStressInterventionBatchError::InvalidInput(errors))
                if errors.contains(&WaterEnergyStressInterventionValidationError::DuplicateCandidateId)
        ));
    }

    #[test]
    fn aggregate_service_step_budget_fails_closed_before_evaluation() {
        let mut input = request();
        input.max_total_service_step_assessments = 1;
        assert!(matches!(
            compare_water_energy_stress_intervention_batch(&input, &[]),
            Err(WaterEnergyStressInterventionBatchError::InvalidInput(errors))
                if errors.contains(&WaterEnergyStressInterventionValidationError::BatchAssessmentBudgetExceeded)
        ));
    }

    #[test]
    fn aggregate_override_budget_fails_closed_before_evaluation() {
        let mut input = request();
        input.max_total_declared_failure_overrides = 1;
        assert!(matches!(
            compare_water_energy_stress_intervention_batch(&input, &[]),
            Err(WaterEnergyStressInterventionBatchError::InvalidInput(errors))
                if errors.contains(&WaterEnergyStressInterventionValidationError::BatchOverrideBudgetExceeded)
        ));
    }

    #[test]
    fn compact_summaries_match_detailed_single_candidate_comparisons() {
        let input = request();
        let batch =
            compare_water_energy_stress_intervention_batch(&input, &[]).expect("valid batch");
        for summary in &batch.candidates {
            let candidate = input
                .candidates
                .iter()
                .find(|candidate| candidate.candidate_id == summary.candidate_id)
                .expect("input candidate exists");
            let detailed_request = WaterEnergyStressInterventionComparisonRequest {
                schema_version: WATER_ENERGY_STRESS_INTERVENTION_SCHEMA_VERSION.to_owned(),
                id: format!("detailed-{}", candidate.candidate_id),
                baseline: input.baseline.clone(),
                candidate: candidate.experiment.clone(),
            };
            let detailed = compare_water_energy_stress_interventions(&detailed_request, &[])
                .expect("detailed comparator must accept paired candidates");
            assert_eq!(summary.effect, detailed.effect);
            assert_eq!(
                summary.possible_protected_floor_regression,
                detailed.possible_protected_floor_regression
            );
            assert_eq!(summary.compared_case_count, detailed.compared_case_count);
            assert_eq!(
                summary.worst_case_unmet_demand_reduction_lower_bound_m3,
                detailed.worst_case_unmet_demand_reduction_lower_bound_m3
            );
            assert_eq!(
                summary.worst_case_unmet_demand_reduction_upper_bound_m3,
                detailed.worst_case_unmet_demand_reduction_upper_bound_m3
            );
            assert_eq!(
                summary.worst_case_customer_delivery_delta_lower_bound_m3,
                detailed.worst_case_customer_delivery_delta_lower_bound_m3
            );
            assert_eq!(
                summary.worst_case_customer_delivery_delta_upper_bound_m3,
                detailed.worst_case_customer_delivery_delta_upper_bound_m3
            );
            assert_eq!(
                summary.worst_case_floor_compliance_delta_lower_bound,
                detailed.worst_case_floor_compliance_delta_lower_bound
            );
            assert_eq!(
                summary.worst_case_floor_compliance_delta_upper_bound,
                detailed.worst_case_floor_compliance_delta_upper_bound
            );
            assert_eq!(
                summary.baseline_supporting_evidence_ids,
                detailed.baseline_supporting_evidence_ids
            );
            assert_eq!(
                summary.candidate_supporting_evidence_ids,
                detailed.candidate_supporting_evidence_ids
            );
        }
    }

    #[test]
    fn batch_rejects_candidate_with_a_different_declared_failure_suite() {
        let mut input = request();
        input.candidates[0].experiment.failure_scenarios[0].id =
            "candidate-only-grid-outage".to_owned();
        assert!(matches!(
            compare_water_energy_stress_intervention_batch(&input, &[]),
            Err(WaterEnergyStressInterventionBatchError::InvalidInput(errors))
                if errors.contains(&WaterEnergyStressInterventionValidationError::FailureScenarioSetMismatch)
        ));
    }
}
