// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Explicit joint water/electricity failure scenarios with protected service floors.
//!
//! This evaluator runs every caller-declared scenario; it does not infer causal links,
//! enumerate undeclared failures, or assign scenario probabilities. Overrides are
//! adverse interval changes to named per-step inputs and become scenario assumptions,
//! never observations. The baseline trajectories are assessed independently: shared physical
//! source/tank/pump capacity is not globally conserved across cells, so shared asset IDs are
//! rejected and callers must preallocate shared resources before using this evaluator. Beneficiary
//! weights describe caller-defined service-cell exposure, not necessarily unique people.

use crate::dependency_logic::ScenarioBasis;
use crate::service_equity::ServiceFloorStatus;
use crate::service_function::{
    FunctionMeasurement, ServiceFunctionAssessmentError, build_evidence_index,
};
use crate::service_water::WaterServiceState;
use crate::service_water_energy_coupling::{
    WaterEnergyCouplingError, WaterEnergyCouplingRequest,
    assess_water_energy_coupled_trajectory_indexed,
};
use crate::spatial_economic::{EvidenceRecord, Quantity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const WATER_ENERGY_STRESS_SCHEMA_VERSION: &str = "0.1.0";
pub const MAX_DECLARED_WATER_ENERGY_FAILURE_SCENARIOS: usize = 10_000;
pub const MAX_WATER_ENERGY_FAILURE_OVERRIDES: usize = 100_000;
// Hard bound on (baseline + every declared failure case) * service-step count.
pub const MAX_WATER_ENERGY_SERVICE_STEP_ASSESSMENTS: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterEnergyFailureTarget {
    SourceFlowRate,
    PumpMechanicalCapacity,
    PumpAvailablePower,
    PumpSpecificEnergy,
    TreatmentCapacity,
    DistributionCapacity,
    DemandRate,
    DistributionLossFraction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterEnergyFailureScenarioKind {
    CommonCause,
    HazardBundle,
    Other,
}

/// A single adverse input change applied to one named service/trajectory interval.
/// Units and entity identity are inherited from the referenced baseline measurement;
/// values never receive implicit unit conversion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyMeasurementOverride {
    pub service_request_id: String,
    pub step_id: String,
    pub target: WaterEnergyFailureTarget,
    pub forced_value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forced_uncertainty_abs: Option<f64>,
}

/// A named, explicitly declared joint event. This is an assumed scenario mapping,
/// not a claim that the event will cause these input changes in the real world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyDeclaredFailureScenario {
    pub id: String,
    pub kind: WaterEnergyFailureScenarioKind,
    pub rationale: String,
    pub overrides: Vec<WaterEnergyMeasurementOverride>,
}

/// One trajectory must describe service for this beneficiary group; the evaluator
/// will not reuse a single trajectory as if it were spatially disaggregated to others.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtectedWaterServiceFloor {
    pub service_request_id: String,
    pub community_id: String,
    pub beneficiary_weight: u64,
    pub essential_coverage_floor: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressSweepRequest {
    pub schema_version: String,
    pub id: String,
    /// Distinct service-cell trajectories on one shared ordered time grid.
    pub baseline_services: Vec<WaterEnergyCouplingRequest>,
    /// Every declared scenario is evaluated; no random sampling or truncation.
    pub failure_scenarios: Vec<WaterEnergyDeclaredFailureScenario>,
    /// Exactly one protected service floor per baseline service request.
    pub protected_service_floors: Vec<ProtectedWaterServiceFloor>,
    /// Caller budget. Exceeding it fails closed.
    pub max_failure_scenarios: usize,
    /// Caller budget for all per-step assessments, including the nominal baseline.
    pub max_service_step_assessments: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterEnergyStressCaseKind {
    Baseline,
    CommonCause,
    HazardBundle,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterEnergyStressValidationError {
    UnsupportedSchemaVersion,
    EmptyExperimentId,
    EmptyBaselineServices,
    EmptyFailureScenarioSet,
    EmptyProtectedFloorSet,
    InvalidScenarioBudget,
    ScenarioBudgetExceeded,
    FailureOverrideBudgetExceeded,
    InvalidStepAssessmentBudget,
    StepAssessmentBudgetExceeded,
    ServiceRequestIdEmpty,
    DuplicateServiceRequestId,
    DuplicateServiceEntityId,
    SharedPhysicalAssetAcrossServiceCells,
    EmptyServiceTrajectory,
    ServiceTimeGridMismatch,
    ProtectedFloorCountMismatch,
    EmptyFloorServiceRequestId,
    UnknownFloorServiceRequest,
    DuplicateFloorServiceRequest,
    EmptyCommunityId,
    ZeroBeneficiaryWeight,
    InvalidCoverageFloor,
    FailureScenarioIdEmpty,
    DuplicateFailureScenarioId,
    ReservedFailureScenarioId,
    EmptyFailureRationale,
    EmptyFailureOverrides,
    EmptyOverrideServiceRequestId,
    UnknownOverrideServiceRequest,
    EmptyOverrideStepId,
    UnknownOverrideStepId,
    DuplicateOverrideTarget,
    NonFiniteForcedValue,
    NegativeForcedValue,
    NonFiniteForcedUncertainty,
    NegativeForcedUncertainty,
    NonFiniteForcedInterval,
    InvalidDistributionLossInterval,
    NonPositiveSpecificEnergyLowerBound,
    OverrideNotAdverse,
    WeightOverflow,
    NonFiniteAggregate,
    BoundsInconsistent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaterEnergyStressSweepError {
    InvalidInput(Vec<WaterEnergyStressValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
    CouplingAssessment {
        scenario_id: String,
        service_request_id: String,
        source: WaterEnergyCouplingError,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressStepAssessment {
    pub step_id: String,
    pub as_of: String,
    pub step_hours: f64,
    pub service_state: WaterServiceState,
    pub coverage_lower_bound: Option<f64>,
    pub coverage_upper_bound: Option<f64>,
    pub essential_coverage_floor: f64,
    pub floor_status: ServiceFloorStatus,
    pub unmet_demand_volume_lower_bound_m3: f64,
    pub unmet_demand_volume_upper_bound_m3: f64,
    pub customer_delivered_volume_lower_bound_m3: f64,
    pub customer_delivered_volume_upper_bound_m3: f64,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressServiceAssessment {
    pub service_request_id: String,
    pub service_entity_id: String,
    pub community_id: String,
    pub beneficiary_weight: u64,
    pub essential_coverage_floor: f64,
    /// Whole-horizon classification: any definite sampled floor breach is BelowFloor.
    pub floor_status: ServiceFloorStatus,
    pub initial_state: WaterServiceState,
    pub final_state: WaterServiceState,
    pub first_adequate_as_of: Option<String>,
    pub adequacy_maintained_through_last_sample: bool,
    pub unmet_demand_volume_lower_bound_m3: f64,
    pub unmet_demand_volume_upper_bound_m3: f64,
    pub customer_delivered_volume_lower_bound_m3: f64,
    pub customer_delivered_volume_upper_bound_m3: f64,
    pub maximum_normalized_deficit_lower_bound: f64,
    pub maximum_normalized_deficit_upper_bound: f64,
    pub steps: Vec<WaterEnergyStressStepAssessment>,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyCommunityFloorOutcome {
    pub community_id: String,
    pub service_cell_count: usize,
    pub total_beneficiary_weight: u64,
    pub meets_floor_weight: u64,
    pub below_floor_weight: u64,
    pub indeterminate_weight: u64,
    pub floor_compliance_lower_bound: f64,
    pub floor_compliance_upper_bound: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressCaseAssessment {
    pub id: String,
    pub kind: WaterEnergyStressCaseKind,
    pub rationale: String,
    pub overrides_applied: Vec<WaterEnergyMeasurementOverride>,
    pub service_cell_count: usize,
    pub total_beneficiary_weight: u64,
    pub meets_floor_weight: u64,
    pub below_floor_weight: u64,
    pub indeterminate_weight: u64,
    pub overall_floor_compliance_lower_bound: f64,
    pub overall_floor_compliance_upper_bound: f64,
    pub total_unmet_demand_volume_lower_bound_m3: f64,
    pub total_unmet_demand_volume_upper_bound_m3: f64,
    pub maximum_normalized_deficit_lower_bound: f64,
    pub maximum_normalized_deficit_upper_bound: f64,
    pub communities: Vec<WaterEnergyCommunityFloorOutcome>,
    pub services: Vec<WaterEnergyStressServiceAssessment>,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyStressSweepAssessment {
    pub schema_version: String,
    pub experiment_id: String,
    pub service_count: usize,
    pub declared_failure_scenario_count: usize,
    pub candidate_failure_scenario_count: usize,
    pub evaluated_failure_scenario_count: usize,
    pub declared_failure_override_count: usize,
    pub evaluated_failure_override_count: usize,
    /// True means every caller-declared case was evaluated; it does not mean all
    /// physically possible outages were enumerated.
    pub exhaustive_over_declared_scenarios: bool,
    pub total_service_step_assessments: usize,
    pub baseline: WaterEnergyStressCaseAssessment,
    pub cases: Vec<WaterEnergyStressCaseAssessment>,
    /// Extremes across declared failure cases only. Scenario counts are not probabilities.
    pub worst_case_unmet_demand_lower_bound_m3: f64,
    pub worst_case_unmet_demand_upper_bound_m3: f64,
    pub worst_case_floor_compliance_lower_bound: f64,
    pub worst_case_floor_compliance_upper_bound: f64,
    pub worst_case_maximum_normalized_deficit_lower_bound: f64,
    pub worst_case_maximum_normalized_deficit_upper_bound: f64,
}

#[derive(Debug, Default)]
struct CommunityAccumulator {
    service_cell_count: usize,
    total_weight: u64,
    meets_weight: u64,
    below_weight: u64,
    indeterminate_weight: u64,
}

impl CommunityAccumulator {
    fn add(
        &mut self,
        weight: u64,
        status: ServiceFloorStatus,
    ) -> Result<(), WaterEnergyStressValidationError> {
        self.service_cell_count += 1;
        self.total_weight = self
            .total_weight
            .checked_add(weight)
            .ok_or(WaterEnergyStressValidationError::WeightOverflow)?;
        match status {
            ServiceFloorStatus::MeetsFloor => {
                self.meets_weight = self
                    .meets_weight
                    .checked_add(weight)
                    .ok_or(WaterEnergyStressValidationError::WeightOverflow)?;
            }
            ServiceFloorStatus::BelowFloor => {
                self.below_weight = self
                    .below_weight
                    .checked_add(weight)
                    .ok_or(WaterEnergyStressValidationError::WeightOverflow)?;
            }
            ServiceFloorStatus::Indeterminate => {
                self.indeterminate_weight = self
                    .indeterminate_weight
                    .checked_add(weight)
                    .ok_or(WaterEnergyStressValidationError::WeightOverflow)?;
            }
        }
        Ok(())
    }

    fn finish(self, community_id: String) -> WaterEnergyCommunityFloorOutcome {
        WaterEnergyCommunityFloorOutcome {
            community_id,
            service_cell_count: self.service_cell_count,
            total_beneficiary_weight: self.total_weight,
            meets_floor_weight: self.meets_weight,
            below_floor_weight: self.below_weight,
            indeterminate_weight: self.indeterminate_weight,
            floor_compliance_lower_bound: self.meets_weight as f64 / self.total_weight as f64,
            floor_compliance_upper_bound: (self.meets_weight + self.indeterminate_weight) as f64
                / self.total_weight as f64,
        }
    }
}

fn measurement_interval(value: f64, uncertainty: Option<f64>) -> (f64, f64) {
    let uncertainty = uncertainty.unwrap_or(0.0);
    ((value - uncertainty).max(0.0), value + uncertainty)
}

fn measurement_for_target<'a>(
    request: &'a WaterEnergyCouplingRequest,
    step_id: &str,
    target: WaterEnergyFailureTarget,
) -> Option<&'a FunctionMeasurement> {
    match target {
        WaterEnergyFailureTarget::SourceFlowRate => request
            .trajectory
            .steps
            .iter()
            .find(|step| step.id == step_id)
            .map(|step| &step.source_flow_rate),
        WaterEnergyFailureTarget::PumpMechanicalCapacity => request
            .trajectory
            .steps
            .iter()
            .find(|step| step.id == step_id)
            .map(|step| &step.pump_capacity),
        WaterEnergyFailureTarget::PumpAvailablePower => request
            .steps
            .iter()
            .find(|step| step.step_id == step_id)
            .map(|step| &step.available_power),
        WaterEnergyFailureTarget::PumpSpecificEnergy => request
            .steps
            .iter()
            .find(|step| step.step_id == step_id)
            .map(|step| &step.specific_energy),
        WaterEnergyFailureTarget::TreatmentCapacity => request
            .trajectory
            .steps
            .iter()
            .find(|step| step.id == step_id)
            .map(|step| &step.treatment_capacity),
        WaterEnergyFailureTarget::DistributionCapacity => request
            .trajectory
            .steps
            .iter()
            .find(|step| step.id == step_id)
            .map(|step| &step.distribution_capacity),
        WaterEnergyFailureTarget::DemandRate => request
            .trajectory
            .steps
            .iter()
            .find(|step| step.id == step_id)
            .map(|step| &step.demand_rate),
        WaterEnergyFailureTarget::DistributionLossFraction => request
            .trajectory
            .steps
            .iter()
            .find(|step| step.id == step_id)
            .map(|step| &step.distribution_loss_fraction),
    }
}

fn measurement_for_target_mut<'a>(
    request: &'a mut WaterEnergyCouplingRequest,
    step_id: &str,
    target: WaterEnergyFailureTarget,
) -> Option<&'a mut FunctionMeasurement> {
    match target {
        WaterEnergyFailureTarget::SourceFlowRate => request
            .trajectory
            .steps
            .iter_mut()
            .find(|step| step.id == step_id)
            .map(|step| &mut step.source_flow_rate),
        WaterEnergyFailureTarget::PumpMechanicalCapacity => request
            .trajectory
            .steps
            .iter_mut()
            .find(|step| step.id == step_id)
            .map(|step| &mut step.pump_capacity),
        WaterEnergyFailureTarget::PumpAvailablePower => request
            .steps
            .iter_mut()
            .find(|step| step.step_id == step_id)
            .map(|step| &mut step.available_power),
        WaterEnergyFailureTarget::PumpSpecificEnergy => request
            .steps
            .iter_mut()
            .find(|step| step.step_id == step_id)
            .map(|step| &mut step.specific_energy),
        WaterEnergyFailureTarget::TreatmentCapacity => request
            .trajectory
            .steps
            .iter_mut()
            .find(|step| step.id == step_id)
            .map(|step| &mut step.treatment_capacity),
        WaterEnergyFailureTarget::DistributionCapacity => request
            .trajectory
            .steps
            .iter_mut()
            .find(|step| step.id == step_id)
            .map(|step| &mut step.distribution_capacity),
        WaterEnergyFailureTarget::DemandRate => request
            .trajectory
            .steps
            .iter_mut()
            .find(|step| step.id == step_id)
            .map(|step| &mut step.demand_rate),
        WaterEnergyFailureTarget::DistributionLossFraction => request
            .trajectory
            .steps
            .iter_mut()
            .find(|step| step.id == step_id)
            .map(|step| &mut step.distribution_loss_fraction),
    }
}

fn target_is_capacity(target: WaterEnergyFailureTarget) -> bool {
    matches!(
        target,
        WaterEnergyFailureTarget::SourceFlowRate
            | WaterEnergyFailureTarget::PumpMechanicalCapacity
            | WaterEnergyFailureTarget::PumpAvailablePower
            | WaterEnergyFailureTarget::TreatmentCapacity
            | WaterEnergyFailureTarget::DistributionCapacity
    )
}

fn target_is_adverse_energy_or_requirement(target: WaterEnergyFailureTarget) -> bool {
    matches!(
        target,
        WaterEnergyFailureTarget::PumpSpecificEnergy
            | WaterEnergyFailureTarget::DemandRate
            | WaterEnergyFailureTarget::DistributionLossFraction
    )
}

fn floor_status(
    coverage_lower: Option<f64>,
    coverage_upper: Option<f64>,
    floor: f64,
) -> ServiceFloorStatus {
    if coverage_lower.is_some_and(|coverage| coverage >= floor) {
        ServiceFloorStatus::MeetsFloor
    } else if coverage_upper.is_some_and(|coverage| coverage < floor) {
        ServiceFloorStatus::BelowFloor
    } else {
        ServiceFloorStatus::Indeterminate
    }
}

fn whole_trajectory_floor_status(statuses: &[ServiceFloorStatus]) -> ServiceFloorStatus {
    if statuses
        .iter()
        .any(|status| *status == ServiceFloorStatus::BelowFloor)
    {
        ServiceFloorStatus::BelowFloor
    } else if !statuses.is_empty()
        && statuses
            .iter()
            .all(|status| *status == ServiceFloorStatus::MeetsFloor)
    {
        ServiceFloorStatus::MeetsFloor
    } else {
        ServiceFloorStatus::Indeterminate
    }
}

fn validate_sweep_request(
    request: &WaterEnergyStressSweepRequest,
) -> Result<usize, Vec<WaterEnergyStressValidationError>> {
    let mut errors = Vec::new();
    if request.schema_version != WATER_ENERGY_STRESS_SCHEMA_VERSION {
        errors.push(WaterEnergyStressValidationError::UnsupportedSchemaVersion);
    }
    if request.id.trim().is_empty() {
        errors.push(WaterEnergyStressValidationError::EmptyExperimentId);
    }
    if request.baseline_services.is_empty() {
        errors.push(WaterEnergyStressValidationError::EmptyBaselineServices);
    }
    if request.failure_scenarios.is_empty() {
        errors.push(WaterEnergyStressValidationError::EmptyFailureScenarioSet);
    }
    if request.protected_service_floors.is_empty() {
        errors.push(WaterEnergyStressValidationError::EmptyProtectedFloorSet);
    }
    if request.max_failure_scenarios == 0
        || request.max_failure_scenarios > MAX_DECLARED_WATER_ENERGY_FAILURE_SCENARIOS
    {
        errors.push(WaterEnergyStressValidationError::InvalidScenarioBudget);
    }
    if request.max_service_step_assessments == 0
        || request.max_service_step_assessments > MAX_WATER_ENERGY_SERVICE_STEP_ASSESSMENTS
    {
        errors.push(WaterEnergyStressValidationError::InvalidStepAssessmentBudget);
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    if request.failure_scenarios.len() > request.max_failure_scenarios {
        errors.push(WaterEnergyStressValidationError::ScenarioBudgetExceeded);
    }

    let mut request_ids = BTreeSet::new();
    let mut service_entities = BTreeSet::new();
    let mut physical_component_owners: BTreeMap<&str, &str> = BTreeMap::new();
    let mut reference_grid: Option<Vec<(String, String, f64)>> = None;
    let mut step_assessment_count = 0_usize;
    for service in &request.baseline_services {
        if service.id.trim().is_empty() {
            errors.push(WaterEnergyStressValidationError::ServiceRequestIdEmpty);
        }
        if !request_ids.insert(service.id.as_str()) {
            errors.push(WaterEnergyStressValidationError::DuplicateServiceRequestId);
        }
        if !service_entities.insert(service.trajectory.service_entity_id.as_str()) {
            errors.push(WaterEnergyStressValidationError::DuplicateServiceEntityId);
        }
        let mut physical_component_ids = BTreeSet::new();
        physical_component_ids.insert(service.trajectory.initial_storage.entity_id.as_str());
        physical_component_ids.insert(service.trajectory.storage_capacity.entity_id.as_str());
        for step in &service.trajectory.steps {
            physical_component_ids.insert(step.source_flow_rate.entity_id.as_str());
            physical_component_ids.insert(step.pump_capacity.entity_id.as_str());
            physical_component_ids.insert(step.treatment_capacity.entity_id.as_str());
            physical_component_ids.insert(step.distribution_capacity.entity_id.as_str());
            physical_component_ids.insert(step.distribution_loss_fraction.entity_id.as_str());
        }
        for component_id in physical_component_ids {
            if component_id.trim().is_empty() {
                continue;
            }
            match physical_component_owners.get(component_id) {
                Some(owner) if *owner != service.id.as_str() => {
                    errors.push(
                        WaterEnergyStressValidationError::SharedPhysicalAssetAcrossServiceCells,
                    );
                }
                Some(_) => {}
                None => {
                    physical_component_owners.insert(component_id, service.id.as_str());
                }
            }
        }
        if service.trajectory.steps.is_empty() {
            errors.push(WaterEnergyStressValidationError::EmptyServiceTrajectory);
        }
        let grid = service
            .trajectory
            .steps
            .iter()
            .map(|step| (step.id.clone(), step.as_of.clone(), step.step_hours))
            .collect::<Vec<_>>();
        if let Some(reference) = &reference_grid {
            if reference.len() != grid.len()
                || reference
                    .iter()
                    .zip(grid.iter())
                    .any(|(left, right)| left.1 != right.1 || left.2 != right.2)
            {
                errors.push(WaterEnergyStressValidationError::ServiceTimeGridMismatch);
            }
        } else {
            reference_grid = Some(grid);
        }
        step_assessment_count =
            match step_assessment_count.checked_add(service.trajectory.steps.len()) {
                Some(count) => count,
                None => {
                    errors.push(WaterEnergyStressValidationError::StepAssessmentBudgetExceeded);
                    usize::MAX
                }
            };
    }

    let mut floor_request_ids = BTreeSet::new();
    for floor in &request.protected_service_floors {
        if floor.service_request_id.trim().is_empty() {
            errors.push(WaterEnergyStressValidationError::EmptyFloorServiceRequestId);
        }
        if !request_ids.contains(floor.service_request_id.as_str()) {
            errors.push(WaterEnergyStressValidationError::UnknownFloorServiceRequest);
        }
        if !floor_request_ids.insert(floor.service_request_id.as_str()) {
            errors.push(WaterEnergyStressValidationError::DuplicateFloorServiceRequest);
        }
        if floor.community_id.trim().is_empty() {
            errors.push(WaterEnergyStressValidationError::EmptyCommunityId);
        }
        if floor.beneficiary_weight == 0 {
            errors.push(WaterEnergyStressValidationError::ZeroBeneficiaryWeight);
        }
        if !floor.essential_coverage_floor.is_finite()
            || !(0.0..=1.0).contains(&floor.essential_coverage_floor)
        {
            errors.push(WaterEnergyStressValidationError::InvalidCoverageFloor);
        }
    }
    if request.protected_service_floors.len() != request.baseline_services.len()
        || floor_request_ids.len() != request_ids.len()
    {
        errors.push(WaterEnergyStressValidationError::ProtectedFloorCountMismatch);
    }

    let mut scenario_ids = BTreeSet::new();
    let mut total_declared_overrides = 0_usize;
    for scenario in &request.failure_scenarios {
        total_declared_overrides = total_declared_overrides
            .checked_add(scenario.overrides.len())
            .unwrap_or(usize::MAX);
        if scenario.id.trim().is_empty() {
            errors.push(WaterEnergyStressValidationError::FailureScenarioIdEmpty);
        }
        if !scenario_ids.insert(scenario.id.as_str()) {
            errors.push(WaterEnergyStressValidationError::DuplicateFailureScenarioId);
        }
        if scenario.id == "baseline" {
            errors.push(WaterEnergyStressValidationError::ReservedFailureScenarioId);
        }
        if scenario.rationale.trim().is_empty() {
            errors.push(WaterEnergyStressValidationError::EmptyFailureRationale);
        }
        if scenario.overrides.is_empty() {
            errors.push(WaterEnergyStressValidationError::EmptyFailureOverrides);
        }
        let mut override_keys = BTreeSet::new();
        for override_value in &scenario.overrides {
            if override_value.service_request_id.trim().is_empty() {
                errors.push(WaterEnergyStressValidationError::EmptyOverrideServiceRequestId);
            }
            let Some(service) = request
                .baseline_services
                .iter()
                .find(|service| service.id == override_value.service_request_id)
            else {
                errors.push(WaterEnergyStressValidationError::UnknownOverrideServiceRequest);
                continue;
            };
            if override_value.step_id.trim().is_empty() {
                errors.push(WaterEnergyStressValidationError::EmptyOverrideStepId);
            }
            if !service
                .trajectory
                .steps
                .iter()
                .any(|step| step.id == override_value.step_id)
            {
                errors.push(WaterEnergyStressValidationError::UnknownOverrideStepId);
                continue;
            }
            if !override_keys.insert((
                override_value.service_request_id.as_str(),
                override_value.step_id.as_str(),
                override_value.target,
            )) {
                errors.push(WaterEnergyStressValidationError::DuplicateOverrideTarget);
                continue;
            }

            if !override_value.forced_value.is_finite() {
                errors.push(WaterEnergyStressValidationError::NonFiniteForcedValue);
                continue;
            }
            if override_value.forced_value < 0.0 {
                errors.push(WaterEnergyStressValidationError::NegativeForcedValue);
                continue;
            }
            if let Some(uncertainty) = override_value.forced_uncertainty_abs {
                if !uncertainty.is_finite() {
                    errors.push(WaterEnergyStressValidationError::NonFiniteForcedUncertainty);
                    continue;
                }
                if uncertainty < 0.0 {
                    errors.push(WaterEnergyStressValidationError::NegativeForcedUncertainty);
                    continue;
                }
            }
            let (new_lower, new_upper) = measurement_interval(
                override_value.forced_value,
                override_value.forced_uncertainty_abs,
            );
            if !new_upper.is_finite() {
                errors.push(WaterEnergyStressValidationError::NonFiniteForcedInterval);
                continue;
            }
            if override_value.target == WaterEnergyFailureTarget::DistributionLossFraction
                && new_upper >= 1.0
            {
                errors.push(WaterEnergyStressValidationError::InvalidDistributionLossInterval);
                continue;
            }
            if override_value.target == WaterEnergyFailureTarget::PumpSpecificEnergy
                && new_lower <= 0.0
            {
                errors.push(WaterEnergyStressValidationError::NonPositiveSpecificEnergyLowerBound);
                continue;
            }
            let Some(current) =
                measurement_for_target(service, &override_value.step_id, override_value.target)
            else {
                errors.push(WaterEnergyStressValidationError::UnknownOverrideStepId);
                continue;
            };
            let (old_lower, old_upper) =
                measurement_interval(current.quantity.value, current.quantity.uncertainty_abs);
            let adverse_and_changed = if target_is_capacity(override_value.target) {
                new_lower <= old_lower
                    && new_upper <= old_upper
                    && (new_lower < old_lower || new_upper < old_upper)
            } else if target_is_adverse_energy_or_requirement(override_value.target) {
                new_lower >= old_lower
                    && new_upper >= old_upper
                    && (new_lower > old_lower || new_upper > old_upper)
            } else {
                false
            };
            if !adverse_and_changed {
                errors.push(WaterEnergyStressValidationError::OverrideNotAdverse);
            }
        }
    }

    if total_declared_overrides > MAX_WATER_ENERGY_FAILURE_OVERRIDES {
        errors.push(WaterEnergyStressValidationError::FailureOverrideBudgetExceeded);
    }

    let total_step_assessments = step_assessment_count
        .checked_mul(request.failure_scenarios.len().saturating_add(1))
        .unwrap_or(usize::MAX);
    if total_step_assessments > request.max_service_step_assessments
        || total_step_assessments > MAX_WATER_ENERGY_SERVICE_STEP_ASSESSMENTS
    {
        errors.push(WaterEnergyStressValidationError::StepAssessmentBudgetExceeded);
    }
    if errors.is_empty() {
        Ok(total_step_assessments)
    } else {
        Err(errors)
    }
}

fn apply_scenario_overrides(
    baseline_services: &[WaterEnergyCouplingRequest],
    scenario: &WaterEnergyDeclaredFailureScenario,
) -> Result<Vec<WaterEnergyCouplingRequest>, WaterEnergyStressValidationError> {
    let mut changed = baseline_services.to_vec();
    for override_value in &scenario.overrides {
        let service = changed
            .iter_mut()
            .find(|service| service.id == override_value.service_request_id)
            .ok_or(WaterEnergyStressValidationError::UnknownOverrideServiceRequest)?;
        let measurement =
            measurement_for_target_mut(service, &override_value.step_id, override_value.target)
                .ok_or(WaterEnergyStressValidationError::UnknownOverrideStepId)?;
        measurement.quantity = Quantity {
            value: override_value.forced_value,
            unit: measurement.quantity.unit.clone(),
            uncertainty_abs: override_value.forced_uncertainty_abs,
        };
        measurement.basis = ScenarioBasis::Assumed;
        measurement.rationale = format!(
            "Declared failure scenario {}: {}; modeled override, not an observation.",
            scenario.id, scenario.rationale
        );
        measurement.supporting_evidence_ids.clear();
    }
    Ok(changed)
}

fn evaluate_case(
    id: String,
    kind: WaterEnergyStressCaseKind,
    rationale: String,
    overrides_applied: Vec<WaterEnergyMeasurementOverride>,
    services: &[WaterEnergyCouplingRequest],
    floors_by_request_id: &BTreeMap<&str, &ProtectedWaterServiceFloor>,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<WaterEnergyStressCaseAssessment, WaterEnergyStressSweepError> {
    let mut overrides_applied = overrides_applied;
    overrides_applied.sort_by(|left, right| {
        left.service_request_id
            .cmp(&right.service_request_id)
            .then_with(|| left.step_id.cmp(&right.step_id))
            .then_with(|| left.target.cmp(&right.target))
    });
    let mut service_results = Vec::with_capacity(services.len());
    let mut community_accumulators: BTreeMap<String, CommunityAccumulator> = BTreeMap::new();
    let mut total_weight = 0_u64;
    let mut meets_weight = 0_u64;
    let mut below_weight = 0_u64;
    let mut indeterminate_weight = 0_u64;
    let mut total_unmet_lower = 0.0;
    let mut total_unmet_upper = 0.0;
    let mut maximum_deficit_lower: f64 = 0.0;
    let mut maximum_deficit_upper: f64 = 0.0;
    let mut evidence_ids = BTreeSet::new();

    let mut ordered_services: Vec<&WaterEnergyCouplingRequest> = services.iter().collect();
    ordered_services.sort_by(|left, right| left.id.cmp(&right.id));
    for service in ordered_services {
        let assessment = assess_water_energy_coupled_trajectory_indexed(service, evidence_by_id)
            .map_err(|source| WaterEnergyStressSweepError::CouplingAssessment {
                scenario_id: id.clone(),
                service_request_id: service.id.clone(),
                source,
            })?;
        let floor = floors_by_request_id
            .get(service.id.as_str())
            .copied()
            .expect("validated floor exists for every baseline service");

        let step_floor_statuses = assessment
            .water_assessment
            .steps
            .iter()
            .map(|step| {
                floor_status(
                    step.coverage_lower_bound,
                    step.coverage_upper_bound,
                    floor.essential_coverage_floor,
                )
            })
            .collect::<Vec<_>>();
        let service_floor_status = whole_trajectory_floor_status(&step_floor_statuses);
        let mut service_evidence = BTreeSet::new();
        service_evidence.extend(assessment.supporting_evidence_ids.iter().cloned());
        let mut service_steps = Vec::with_capacity(assessment.water_assessment.steps.len());
        for (index, step) in assessment.water_assessment.steps.iter().enumerate() {
            let mut step_evidence = BTreeSet::new();
            step_evidence.extend(step.supporting_evidence_ids.iter().cloned());
            step_evidence.extend(
                assessment.steps[index]
                    .supporting_evidence_ids
                    .iter()
                    .cloned(),
            );
            service_evidence.extend(step_evidence.iter().cloned());
            let demand_lower = step.demand_volume_lower_bound;
            let demand_upper = step.demand_volume_upper_bound;
            let deficit_lower =
                (demand_lower - step.customer_delivered_volume_upper_bound).max(0.0);
            let deficit_upper =
                (demand_upper - step.customer_delivered_volume_lower_bound).max(0.0);
            service_steps.push(WaterEnergyStressStepAssessment {
                step_id: step.step_id.clone(),
                as_of: step.as_of.clone(),
                step_hours: step.step_hours,
                service_state: step.state,
                coverage_lower_bound: step.coverage_lower_bound,
                coverage_upper_bound: step.coverage_upper_bound,
                essential_coverage_floor: floor.essential_coverage_floor,
                floor_status: step_floor_statuses[index],
                unmet_demand_volume_lower_bound_m3: deficit_lower,
                unmet_demand_volume_upper_bound_m3: deficit_upper,
                customer_delivered_volume_lower_bound_m3: step
                    .customer_delivered_volume_lower_bound,
                customer_delivered_volume_upper_bound_m3: step
                    .customer_delivered_volume_upper_bound,
                supporting_evidence_ids: step_evidence.into_iter().collect(),
            });
        }

        let mut community = community_accumulators
            .remove(floor.community_id.as_str())
            .unwrap_or_default();
        community
            .add(floor.beneficiary_weight, service_floor_status)
            .map_err(|error| WaterEnergyStressSweepError::InvalidInput(vec![error]))?;
        community_accumulators.insert(floor.community_id.clone(), community);

        total_weight = total_weight
            .checked_add(floor.beneficiary_weight)
            .ok_or_else(|| {
                WaterEnergyStressSweepError::InvalidInput(vec![
                    WaterEnergyStressValidationError::WeightOverflow,
                ])
            })?;
        match service_floor_status {
            ServiceFloorStatus::MeetsFloor => {
                meets_weight = meets_weight
                    .checked_add(floor.beneficiary_weight)
                    .ok_or_else(|| {
                        WaterEnergyStressSweepError::InvalidInput(vec![
                            WaterEnergyStressValidationError::WeightOverflow,
                        ])
                    })?;
            }
            ServiceFloorStatus::BelowFloor => {
                below_weight = below_weight
                    .checked_add(floor.beneficiary_weight)
                    .ok_or_else(|| {
                        WaterEnergyStressSweepError::InvalidInput(vec![
                            WaterEnergyStressValidationError::WeightOverflow,
                        ])
                    })?;
            }
            ServiceFloorStatus::Indeterminate => {
                indeterminate_weight = indeterminate_weight
                    .checked_add(floor.beneficiary_weight)
                    .ok_or_else(|| {
                    WaterEnergyStressSweepError::InvalidInput(vec![
                        WaterEnergyStressValidationError::WeightOverflow,
                    ])
                })?;
            }
        }

        total_unmet_lower += assessment.water_assessment.unmet_demand_volume_lower_bound;
        total_unmet_upper += assessment.water_assessment.unmet_demand_volume_upper_bound;
        maximum_deficit_lower = maximum_deficit_lower.max(
            assessment
                .water_assessment
                .maximum_normalized_deficit_lower_bound,
        );
        maximum_deficit_upper = maximum_deficit_upper.max(
            assessment
                .water_assessment
                .maximum_normalized_deficit_upper_bound,
        );
        evidence_ids.extend(service_evidence.iter().cloned());

        service_results.push(WaterEnergyStressServiceAssessment {
            service_request_id: service.id.clone(),
            service_entity_id: assessment.water_assessment.service_entity_id.clone(),
            community_id: floor.community_id.clone(),
            beneficiary_weight: floor.beneficiary_weight,
            essential_coverage_floor: floor.essential_coverage_floor,
            floor_status: service_floor_status,
            initial_state: assessment.water_assessment.initial_state,
            final_state: assessment.water_assessment.final_state,
            first_adequate_as_of: assessment.water_assessment.first_adequate_as_of.clone(),
            adequacy_maintained_through_last_sample: assessment
                .water_assessment
                .adequacy_maintained_through_last_sample,
            unmet_demand_volume_lower_bound_m3: assessment
                .water_assessment
                .unmet_demand_volume_lower_bound,
            unmet_demand_volume_upper_bound_m3: assessment
                .water_assessment
                .unmet_demand_volume_upper_bound,
            customer_delivered_volume_lower_bound_m3: assessment
                .water_assessment
                .customer_delivered_volume_lower_bound,
            customer_delivered_volume_upper_bound_m3: assessment
                .water_assessment
                .customer_delivered_volume_upper_bound,
            maximum_normalized_deficit_lower_bound: assessment
                .water_assessment
                .maximum_normalized_deficit_lower_bound,
            maximum_normalized_deficit_upper_bound: assessment
                .water_assessment
                .maximum_normalized_deficit_upper_bound,
            steps: service_steps,
            supporting_evidence_ids: service_evidence.into_iter().collect(),
        });
    }

    let mut communities = community_accumulators
        .into_iter()
        .map(|(id, accumulator)| accumulator.finish(id))
        .collect::<Vec<_>>();
    communities.sort_by(|left, right| left.community_id.cmp(&right.community_id));
    service_results.sort_by(|left, right| left.service_request_id.cmp(&right.service_request_id));

    for value in [
        total_unmet_lower,
        total_unmet_upper,
        maximum_deficit_lower,
        maximum_deficit_upper,
    ] {
        if !value.is_finite() {
            return Err(WaterEnergyStressSweepError::InvalidInput(vec![
                WaterEnergyStressValidationError::NonFiniteAggregate,
            ]));
        }
    }
    if total_unmet_lower > total_unmet_upper
        || maximum_deficit_lower > maximum_deficit_upper
        || total_weight == 0
        || meets_weight
            .checked_add(below_weight)
            .and_then(|partial| partial.checked_add(indeterminate_weight))
            != Some(total_weight)
    {
        return Err(WaterEnergyStressSweepError::InvalidInput(vec![
            WaterEnergyStressValidationError::BoundsInconsistent,
        ]));
    }

    Ok(WaterEnergyStressCaseAssessment {
        id,
        kind,
        rationale,
        overrides_applied,
        service_cell_count: service_results.len(),
        total_beneficiary_weight: total_weight,
        meets_floor_weight: meets_weight,
        below_floor_weight: below_weight,
        indeterminate_weight,
        overall_floor_compliance_lower_bound: meets_weight as f64 / total_weight as f64,
        overall_floor_compliance_upper_bound: (meets_weight + indeterminate_weight) as f64
            / total_weight as f64,
        total_unmet_demand_volume_lower_bound_m3: total_unmet_lower,
        total_unmet_demand_volume_upper_bound_m3: total_unmet_upper,
        maximum_normalized_deficit_lower_bound: maximum_deficit_lower,
        maximum_normalized_deficit_upper_bound: maximum_deficit_upper,
        communities,
        services: service_results,
        supporting_evidence_ids: evidence_ids.into_iter().collect(),
    })
}

/// Evaluate baseline plus every explicitly declared adverse joint-failure case.
///
/// The evidence corpus is validated and indexed once across the entire sweep. Service
/// requests must each represent one protected service cell and share a time grid.
/// Every scenario is evaluated; exceeding the scenario or service-step budget returns
/// an error rather than silently sampling. Results summarize worst case across declared
/// scenarios only and never reinterpret case counts as calibrated probabilities.
pub fn assess_water_energy_stress_sweep(
    request: &WaterEnergyStressSweepRequest,
    evidence_records: &[EvidenceRecord],
) -> Result<WaterEnergyStressSweepAssessment, WaterEnergyStressSweepError> {
    let total_step_assessments =
        validate_sweep_request(request).map_err(WaterEnergyStressSweepError::InvalidInput)?;
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(WaterEnergyStressSweepError::EvidenceValidation)?;
    assess_water_energy_stress_sweep_validated(request, total_step_assessments, &evidence_by_id)
}

pub(crate) fn assess_water_energy_stress_sweep_indexed(
    request: &WaterEnergyStressSweepRequest,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<WaterEnergyStressSweepAssessment, WaterEnergyStressSweepError> {
    let total_step_assessments =
        validate_sweep_request(request).map_err(WaterEnergyStressSweepError::InvalidInput)?;
    assess_water_energy_stress_sweep_validated(request, total_step_assessments, evidence_by_id)
}

fn assess_water_energy_stress_sweep_validated(
    request: &WaterEnergyStressSweepRequest,
    total_step_assessments: usize,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<WaterEnergyStressSweepAssessment, WaterEnergyStressSweepError> {
    let floors_by_request_id = request
        .protected_service_floors
        .iter()
        .map(|floor| (floor.service_request_id.as_str(), floor))
        .collect::<BTreeMap<_, _>>();
    let baseline = evaluate_case(
        "baseline".to_owned(),
        WaterEnergyStressCaseKind::Baseline,
        "Declared nominal inputs; not a claim of current physical operation.".to_owned(),
        Vec::new(),
        &request.baseline_services,
        &floors_by_request_id,
        evidence_by_id,
    )?;

    let mut ordered_scenarios: Vec<&WaterEnergyDeclaredFailureScenario> =
        request.failure_scenarios.iter().collect();
    ordered_scenarios.sort_by(|left, right| left.id.cmp(&right.id));
    let mut cases = Vec::with_capacity(ordered_scenarios.len());
    for scenario in ordered_scenarios {
        let altered_services = apply_scenario_overrides(&request.baseline_services, scenario)
            .map_err(|error| WaterEnergyStressSweepError::InvalidInput(vec![error]))?;
        let kind = match scenario.kind {
            WaterEnergyFailureScenarioKind::CommonCause => WaterEnergyStressCaseKind::CommonCause,
            WaterEnergyFailureScenarioKind::HazardBundle => WaterEnergyStressCaseKind::HazardBundle,
            WaterEnergyFailureScenarioKind::Other => WaterEnergyStressCaseKind::Other,
        };
        cases.push(evaluate_case(
            scenario.id.clone(),
            kind,
            scenario.rationale.clone(),
            scenario.overrides.clone(),
            &altered_services,
            &floors_by_request_id,
            evidence_by_id,
        )?);
    }

    let worst_case_unmet_lower = cases
        .iter()
        .map(|case| case.total_unmet_demand_volume_lower_bound_m3)
        .fold(0.0_f64, f64::max);
    let worst_case_unmet_upper = cases
        .iter()
        .map(|case| case.total_unmet_demand_volume_upper_bound_m3)
        .fold(0.0_f64, f64::max);
    let worst_case_floor_lower = cases
        .iter()
        .map(|case| case.overall_floor_compliance_lower_bound)
        .fold(1.0_f64, f64::min);
    let worst_case_floor_upper = cases
        .iter()
        .map(|case| case.overall_floor_compliance_upper_bound)
        .fold(1.0_f64, f64::min);
    let worst_case_deficit_lower = cases
        .iter()
        .map(|case| case.maximum_normalized_deficit_lower_bound)
        .fold(0.0_f64, f64::max);
    let worst_case_deficit_upper = cases
        .iter()
        .map(|case| case.maximum_normalized_deficit_upper_bound)
        .fold(0.0_f64, f64::max);
    if [
        worst_case_unmet_lower,
        worst_case_unmet_upper,
        worst_case_floor_lower,
        worst_case_floor_upper,
        worst_case_deficit_lower,
        worst_case_deficit_upper,
    ]
    .into_iter()
    .any(|value| !value.is_finite())
        || worst_case_unmet_lower > worst_case_unmet_upper
        || worst_case_floor_lower > worst_case_floor_upper
        || worst_case_deficit_lower > worst_case_deficit_upper
    {
        return Err(WaterEnergyStressSweepError::InvalidInput(vec![
            WaterEnergyStressValidationError::BoundsInconsistent,
        ]));
    }

    Ok(WaterEnergyStressSweepAssessment {
        schema_version: WATER_ENERGY_STRESS_SCHEMA_VERSION.to_owned(),
        experiment_id: request.id.clone(),
        service_count: request.baseline_services.len(),
        declared_failure_scenario_count: request.failure_scenarios.len(),
        candidate_failure_scenario_count: request.failure_scenarios.len(),
        evaluated_failure_scenario_count: cases.len(),
        declared_failure_override_count: request
            .failure_scenarios
            .iter()
            .map(|scenario| scenario.overrides.len())
            .sum(),
        evaluated_failure_override_count: cases
            .iter()
            .map(|case| case.overrides_applied.len())
            .sum(),
        exhaustive_over_declared_scenarios: cases.len() == request.failure_scenarios.len(),
        total_service_step_assessments: total_step_assessments,
        baseline,
        cases,
        worst_case_unmet_demand_lower_bound_m3: worst_case_unmet_lower,
        worst_case_unmet_demand_upper_bound_m3: worst_case_unmet_upper,
        worst_case_floor_compliance_lower_bound: worst_case_floor_lower,
        worst_case_floor_compliance_upper_bound: worst_case_floor_upper,
        worst_case_maximum_normalized_deficit_lower_bound: worst_case_deficit_lower,
        worst_case_maximum_normalized_deficit_upper_bound: worst_case_deficit_upper,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_water::{DIMENSIONLESS_UNIT, WATER_RATE_UNIT, WATER_VOLUME_UNIT};
    use crate::service_water_energy_coupling::{
        ELECTRIC_POWER_UNIT, PUMP_SPECIFIC_ENERGY_UNIT, PowerMeasurementLocation,
        WATER_ENERGY_COUPLING_SCHEMA_VERSION, WaterEnergyCouplingStepInput,
    };
    use crate::service_water_trajectory::{
        WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION, WaterServiceTrajectory, WaterServiceTrajectoryStep,
    };

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
            rationale: "Synthetic input only.".to_owned(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn service_request(id: &str, service_entity: &str) -> WaterEnergyCouplingRequest {
        let steps = TIMES
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
                initial_storage: measurement("tank", 0.0, WATER_VOLUME_UNIT),
                storage_capacity: measurement("tank", 100.0, WATER_VOLUME_UNIT),
                steps,
            },
            steps: energy_steps,
        }
    }

    fn request() -> WaterEnergyStressSweepRequest {
        let baseline = service_request("cell-a", "community-a");
        WaterEnergyStressSweepRequest {
            schema_version: WATER_ENERGY_STRESS_SCHEMA_VERSION.to_owned(),
            id: "synthetic-joint-stress".to_owned(),
            baseline_services: vec![baseline],
            failure_scenarios: vec![WaterEnergyDeclaredFailureScenario {
                id: "grid-outage".to_owned(),
                kind: WaterEnergyFailureScenarioKind::CommonCause,
                rationale: "Declared synthetic power interruption at the pump terminals."
                    .to_owned(),
                overrides: vec![
                    WaterEnergyMeasurementOverride {
                        service_request_id: "cell-a".to_owned(),
                        step_id: "step-0".to_owned(),
                        target: WaterEnergyFailureTarget::PumpAvailablePower,
                        forced_value: 0.0,
                        forced_uncertainty_abs: None,
                    },
                    WaterEnergyMeasurementOverride {
                        service_request_id: "cell-a".to_owned(),
                        step_id: "step-1".to_owned(),
                        target: WaterEnergyFailureTarget::PumpAvailablePower,
                        forced_value: 0.0,
                        forced_uncertainty_abs: None,
                    },
                ],
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

    #[test]
    fn evaluates_every_declared_joint_failure_case() {
        let output = assess_water_energy_stress_sweep(&request(), &[]).expect("valid stress sweep");
        assert_eq!(output.declared_failure_scenario_count, 1);
        assert_eq!(output.candidate_failure_scenario_count, 1);
        assert_eq!(output.evaluated_failure_scenario_count, 1);
        assert!(output.exhaustive_over_declared_scenarios);
        assert_eq!(output.cases[0].id, "grid-outage");
    }

    #[test]
    fn power_failure_breaches_floor_without_turning_case_count_into_probability() {
        let output = assess_water_energy_stress_sweep(&request(), &[]).expect("valid stress sweep");
        assert_eq!(
            output.baseline.services[0].floor_status,
            ServiceFloorStatus::MeetsFloor
        );
        assert_eq!(
            output.cases[0].services[0].floor_status,
            ServiceFloorStatus::BelowFloor
        );
        assert_eq!(output.cases[0].below_floor_weight, 100);
        assert_eq!(output.cases[0].meets_floor_weight, 0);
        assert_eq!(output.cases[0].indeterminate_weight, 0);
        assert_eq!(output.worst_case_floor_compliance_lower_bound, 0.0);
        assert_eq!(output.cases.len(), 1);
    }

    #[test]
    fn input_order_does_not_change_sorted_scenario_results() {
        let mut input = request();
        input
            .failure_scenarios
            .push(WaterEnergyDeclaredFailureScenario {
                id: "reduced-source".to_owned(),
                kind: WaterEnergyFailureScenarioKind::HazardBundle,
                rationale: "Declared synthetic source interruption.".to_owned(),
                overrides: vec![WaterEnergyMeasurementOverride {
                    service_request_id: "cell-a".to_owned(),
                    step_id: "step-0".to_owned(),
                    target: WaterEnergyFailureTarget::SourceFlowRate,
                    forced_value: 10.0,
                    forced_uncertainty_abs: None,
                }],
            });
        let mut shuffled = input.clone();
        shuffled.failure_scenarios.reverse();
        let first = assess_water_energy_stress_sweep(&input, &[]).expect("valid sweep");
        let second = assess_water_energy_stress_sweep(&shuffled, &[]).expect("permuted sweep");
        assert_eq!(first.cases, second.cases);
        assert_eq!(
            first
                .cases
                .iter()
                .map(|case| case.id.as_str())
                .collect::<Vec<_>>(),
            vec!["grid-outage", "reduced-source"]
        );
    }

    #[test]
    fn override_input_order_does_not_change_serialized_case_results() {
        let input = request();
        let mut shuffled = input.clone();
        shuffled.failure_scenarios[0].overrides.reverse();
        let first = assess_water_energy_stress_sweep(&input, &[]).expect("valid sweep");
        let second = assess_water_energy_stress_sweep(&shuffled, &[]).expect("permuted sweep");
        assert_eq!(first.cases, second.cases);
        assert_eq!(
            serde_json::to_vec(&first).expect("serialize first"),
            serde_json::to_vec(&second).expect("serialize second")
        );
    }

    #[test]
    fn rejects_shared_physical_asset_ids_across_independent_service_cells() {
        let mut input = request();
        let mut second = service_request("cell-b", "community-b");
        let shared_source = input.baseline_services[0].trajectory.steps[0]
            .source_flow_rate
            .entity_id
            .clone();
        for step in &mut second.trajectory.steps {
            step.source_flow_rate.entity_id = shared_source.clone();
        }
        input.baseline_services.push(second);
        input
            .protected_service_floors
            .push(ProtectedWaterServiceFloor {
                service_request_id: "cell-b".to_owned(),
                community_id: "community-b".to_owned(),
                beneficiary_weight: 50,
                essential_coverage_floor: 0.8,
            });
        let result = assess_water_energy_stress_sweep(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyStressSweepError::InvalidInput(errors))
                if errors.contains(
                    &WaterEnergyStressValidationError::SharedPhysicalAssetAcrossServiceCells
                )
        ));
    }

    #[test]
    fn rejects_duplicate_scenario_ids() {
        let mut input = request();
        input
            .failure_scenarios
            .push(input.failure_scenarios[0].clone());
        let result = assess_water_energy_stress_sweep(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyStressSweepError::InvalidInput(errors))
                if errors.contains(&WaterEnergyStressValidationError::DuplicateFailureScenarioId)
        ));
    }

    #[test]
    fn rejects_budgets_instead_of_silently_sampling() {
        let mut input = request();
        input.max_failure_scenarios = 0;
        let result = assess_water_energy_stress_sweep(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyStressSweepError::InvalidInput(errors))
                if errors.contains(&WaterEnergyStressValidationError::InvalidScenarioBudget)
        ));
    }

    #[test]
    fn rejects_a_scenario_that_improves_instead_of_degrades_capacity() {
        let mut input = request();
        input.failure_scenarios[0].overrides[0].forced_value = 60.0;
        let result = assess_water_energy_stress_sweep(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyStressSweepError::InvalidInput(errors))
                if errors.contains(&WaterEnergyStressValidationError::OverrideNotAdverse)
        ));
    }

    #[test]
    fn indeterminate_floor_weight_is_never_counted_as_meet_or_below() {
        let mut input = request();
        for step in &mut input.baseline_services[0].steps {
            step.available_power.quantity.value = 41.0;
            step.available_power.quantity.uncertainty_abs = Some(10.0);
        }
        for override_value in &mut input.failure_scenarios[0].overrides {
            override_value.forced_value = 30.0;
            override_value.forced_uncertainty_abs = Some(10.0);
        }
        let output = assess_water_energy_stress_sweep(&input, &[]).expect("valid uncertain sweep");
        let case = &output.cases[0];
        assert_eq!(
            case.meets_floor_weight + case.below_floor_weight + case.indeterminate_weight,
            case.total_beneficiary_weight
        );
        assert_eq!(case.meets_floor_weight, 0);
        assert_eq!(case.indeterminate_weight, 100);
    }

    #[test]
    fn output_reconciles_step_deficits_and_preserves_override_manifest() {
        let input = request();
        let output = assess_water_energy_stress_sweep(&input, &[]).expect("valid sweep");
        let case = &output.cases[0];
        assert_eq!(case.overrides_applied, input.failure_scenarios[0].overrides);
        let step_lower: f64 = case.services[0]
            .steps
            .iter()
            .map(|step| step.unmet_demand_volume_lower_bound_m3)
            .sum();
        let step_upper: f64 = case.services[0]
            .steps
            .iter()
            .map(|step| step.unmet_demand_volume_upper_bound_m3)
            .sum();
        assert!((step_lower - case.services[0].unmet_demand_volume_lower_bound_m3).abs() < 1e-9);
        assert!((step_upper - case.services[0].unmet_demand_volume_upper_bound_m3).abs() < 1e-9);
        assert_eq!(
            case.services[0].steps[0].floor_status,
            ServiceFloorStatus::BelowFloor
        );
    }
}
