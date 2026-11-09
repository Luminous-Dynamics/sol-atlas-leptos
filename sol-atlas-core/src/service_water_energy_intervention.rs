// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Paired counterfactual comparison for electricity-coupled water-service trajectories.
//!
//! The comparison preserves separate evidence sets for the baseline and candidate
//! at every shared interval. It evaluates modeled deltas; it does not establish
//! intervention execution, field delivery, or causal impact.

use crate::service_function::{ServiceFunctionAssessmentError, build_evidence_index};
use crate::service_water::WaterServiceState;
use crate::service_water_energy_coupling::{
    WaterEnergyCoupledTrajectoryAssessment, WaterEnergyCouplingError, WaterEnergyCouplingRequest,
    assess_water_energy_coupled_trajectory_indexed,
};
use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const WATER_ENERGY_INTERVENTION_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyInterventionComparisonRequest {
    pub schema_version: String,
    pub id: String,
    pub baseline: WaterEnergyCouplingRequest,
    pub candidate: WaterEnergyCouplingRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterEnergyInterventionEffect {
    RobustImprovement,
    RobustRegression,
    NoMaterialChange,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterEnergyInterventionInterpretationBoundary {
    ModeledCounterfactualNotCausalEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterEnergyInterventionValidationError {
    UnsupportedSchemaVersion,
    EmptyComparisonId,
    ServiceEntityMismatch,
    StepCountMismatch,
    StepIdMismatch,
    StepTimeMismatch,
    StepDurationMismatch,
    NonFiniteDerivedValue,
    BoundsInconsistent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaterEnergyInterventionComparisonError {
    InvalidInput(Vec<WaterEnergyInterventionValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
    BaselineAssessment(WaterEnergyCouplingError),
    CandidateAssessment(WaterEnergyCouplingError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyInterventionStepComparison {
    pub step_id: String,
    pub as_of: String,
    pub baseline_state: WaterServiceState,
    pub candidate_state: WaterServiceState,
    pub baseline_unmet_demand_lower_bound: f64,
    pub baseline_unmet_demand_upper_bound: f64,
    pub candidate_unmet_demand_lower_bound: f64,
    pub candidate_unmet_demand_upper_bound: f64,
    /// Positive means less modeled unmet demand in the candidate.
    pub unmet_demand_reduction_lower_bound: f64,
    pub unmet_demand_reduction_upper_bound: f64,
    /// Positive means more modeled customer delivery in the candidate.
    pub customer_delivery_delta_lower_bound: f64,
    pub customer_delivery_delta_upper_bound: f64,
    pub baseline_supporting_evidence_ids: Vec<String>,
    pub candidate_supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterEnergyInterventionComparisonAssessment {
    pub schema_version: String,
    pub comparison_id: String,
    pub service_entity_id: String,
    pub effect: WaterEnergyInterventionEffect,
    pub total_unmet_demand_reduction_lower_bound: f64,
    pub total_unmet_demand_reduction_upper_bound: f64,
    pub total_customer_delivery_delta_lower_bound: f64,
    pub total_customer_delivery_delta_upper_bound: f64,
    pub baseline: WaterEnergyCoupledTrajectoryAssessment,
    pub candidate: WaterEnergyCoupledTrajectoryAssessment,
    pub steps: Vec<WaterEnergyInterventionStepComparison>,
    pub supporting_evidence_ids: Vec<String>,
    pub interpretation_boundary: WaterEnergyInterventionInterpretationBoundary,
}

fn deficit_bounds(
    demand_lower: f64,
    demand_upper: f64,
    delivered_lower: f64,
    delivered_upper: f64,
) -> (f64, f64) {
    (
        (demand_lower - delivered_upper).max(0.0),
        (demand_upper - delivered_lower).max(0.0),
    )
}

fn effect_from_bounds(lower: f64, upper: f64) -> WaterEnergyInterventionEffect {
    let tolerance = f64::EPSILON * 256.0 * lower.abs().max(upper.abs()).max(1.0);
    if lower > tolerance {
        WaterEnergyInterventionEffect::RobustImprovement
    } else if upper < -tolerance {
        WaterEnergyInterventionEffect::RobustRegression
    } else if lower >= -tolerance && upper <= tolerance {
        WaterEnergyInterventionEffect::NoMaterialChange
    } else {
        WaterEnergyInterventionEffect::Indeterminate
    }
}

fn validate_aligned_trajectories(
    request: &WaterEnergyInterventionComparisonRequest,
) -> Result<(), Vec<WaterEnergyInterventionValidationError>> {
    let mut errors = Vec::new();
    if request.schema_version != WATER_ENERGY_INTERVENTION_SCHEMA_VERSION {
        errors.push(WaterEnergyInterventionValidationError::UnsupportedSchemaVersion);
    }
    if request.id.trim().is_empty() {
        errors.push(WaterEnergyInterventionValidationError::EmptyComparisonId);
    }
    if request.baseline.trajectory.service_entity_id
        != request.candidate.trajectory.service_entity_id
    {
        errors.push(WaterEnergyInterventionValidationError::ServiceEntityMismatch);
    }
    let baseline = &request.baseline.trajectory.steps;
    let candidate = &request.candidate.trajectory.steps;
    if baseline.len() != candidate.len() {
        errors.push(WaterEnergyInterventionValidationError::StepCountMismatch);
    }
    for (base, cand) in baseline.iter().zip(candidate.iter()) {
        if base.id != cand.id {
            errors.push(WaterEnergyInterventionValidationError::StepIdMismatch);
        }
        if base.as_of != cand.as_of {
            errors.push(WaterEnergyInterventionValidationError::StepTimeMismatch);
        }
        if base.step_hours != cand.step_hours {
            errors.push(WaterEnergyInterventionValidationError::StepDurationMismatch);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Compare baseline and candidate coupled trajectories on an identical service and
/// time grid. A robust improvement requires the total lower bound on reduced unmet
/// demand to be strictly positive beyond floating-point tolerance. Overlapping
/// uncertainty is indeterminate; every interval retains both sides' water and energy
/// evidence references. Results are modeled counterfactuals, not causal evidence.
pub fn compare_water_energy_trajectory_interventions(
    request: &WaterEnergyInterventionComparisonRequest,
    evidence_records: &[EvidenceRecord],
) -> Result<WaterEnergyInterventionComparisonAssessment, WaterEnergyInterventionComparisonError> {
    validate_aligned_trajectories(request)
        .map_err(WaterEnergyInterventionComparisonError::InvalidInput)?;

    // Validate and index evidence once for both sides of the counterfactual.
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(WaterEnergyInterventionComparisonError::EvidenceValidation)?;
    let baseline =
        assess_water_energy_coupled_trajectory_indexed(&request.baseline, &evidence_by_id)
            .map_err(WaterEnergyInterventionComparisonError::BaselineAssessment)?;
    let candidate =
        assess_water_energy_coupled_trajectory_indexed(&request.candidate, &evidence_by_id)
            .map_err(WaterEnergyInterventionComparisonError::CandidateAssessment)?;

    let mut steps = Vec::with_capacity(baseline.water_assessment.steps.len());
    let mut total_reduction_lower = 0.0;
    let mut total_reduction_upper = 0.0;
    let mut total_delivery_delta_lower = 0.0;
    let mut total_delivery_delta_upper = 0.0;
    let mut all_evidence_ids = BTreeSet::new();

    for index in 0..baseline.water_assessment.steps.len() {
        let base_water = &baseline.water_assessment.steps[index];
        let cand_water = &candidate.water_assessment.steps[index];
        let base_energy = &baseline.steps[index];
        let cand_energy = &candidate.steps[index];

        let (base_deficit_lower, base_deficit_upper) = deficit_bounds(
            base_water.demand_volume_lower_bound,
            base_water.demand_volume_upper_bound,
            base_water.customer_delivered_volume_lower_bound,
            base_water.customer_delivered_volume_upper_bound,
        );
        let (cand_deficit_lower, cand_deficit_upper) = deficit_bounds(
            cand_water.demand_volume_lower_bound,
            cand_water.demand_volume_upper_bound,
            cand_water.customer_delivered_volume_lower_bound,
            cand_water.customer_delivered_volume_upper_bound,
        );
        let reduction_lower = base_deficit_lower - cand_deficit_upper;
        let reduction_upper = base_deficit_upper - cand_deficit_lower;
        let delivery_delta_lower = cand_water.customer_delivered_volume_lower_bound
            - base_water.customer_delivered_volume_upper_bound;
        let delivery_delta_upper = cand_water.customer_delivered_volume_upper_bound
            - base_water.customer_delivered_volume_lower_bound;

        if [
            reduction_lower,
            reduction_upper,
            delivery_delta_lower,
            delivery_delta_upper,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
        {
            return Err(WaterEnergyInterventionComparisonError::InvalidInput(vec![
                WaterEnergyInterventionValidationError::NonFiniteDerivedValue,
            ]));
        }
        if reduction_lower > reduction_upper || delivery_delta_lower > delivery_delta_upper {
            return Err(WaterEnergyInterventionComparisonError::InvalidInput(vec![
                WaterEnergyInterventionValidationError::BoundsInconsistent,
            ]));
        }

        let mut base_evidence = BTreeSet::new();
        base_evidence.extend(base_water.supporting_evidence_ids.iter().cloned());
        base_evidence.extend(base_energy.supporting_evidence_ids.iter().cloned());
        let mut cand_evidence = BTreeSet::new();
        cand_evidence.extend(cand_water.supporting_evidence_ids.iter().cloned());
        cand_evidence.extend(cand_energy.supporting_evidence_ids.iter().cloned());
        all_evidence_ids.extend(base_evidence.iter().cloned());
        all_evidence_ids.extend(cand_evidence.iter().cloned());

        steps.push(WaterEnergyInterventionStepComparison {
            step_id: base_water.step_id.clone(),
            as_of: base_water.as_of.clone(),
            baseline_state: base_water.state,
            candidate_state: cand_water.state,
            baseline_unmet_demand_lower_bound: base_deficit_lower,
            baseline_unmet_demand_upper_bound: base_deficit_upper,
            candidate_unmet_demand_lower_bound: cand_deficit_lower,
            candidate_unmet_demand_upper_bound: cand_deficit_upper,
            unmet_demand_reduction_lower_bound: reduction_lower,
            unmet_demand_reduction_upper_bound: reduction_upper,
            customer_delivery_delta_lower_bound: delivery_delta_lower,
            customer_delivery_delta_upper_bound: delivery_delta_upper,
            baseline_supporting_evidence_ids: base_evidence.into_iter().collect(),
            candidate_supporting_evidence_ids: cand_evidence.into_iter().collect(),
        });

        total_reduction_lower += reduction_lower;
        total_reduction_upper += reduction_upper;
        total_delivery_delta_lower += delivery_delta_lower;
        total_delivery_delta_upper += delivery_delta_upper;
    }

    if [
        total_reduction_lower,
        total_reduction_upper,
        total_delivery_delta_lower,
        total_delivery_delta_upper,
    ]
    .into_iter()
    .any(|value| !value.is_finite())
    {
        return Err(WaterEnergyInterventionComparisonError::InvalidInput(vec![
            WaterEnergyInterventionValidationError::NonFiniteDerivedValue,
        ]));
    }
    if total_reduction_lower > total_reduction_upper
        || total_delivery_delta_lower > total_delivery_delta_upper
    {
        return Err(WaterEnergyInterventionComparisonError::InvalidInput(vec![
            WaterEnergyInterventionValidationError::BoundsInconsistent,
        ]));
    }

    all_evidence_ids.extend(baseline.supporting_evidence_ids.iter().cloned());
    all_evidence_ids.extend(candidate.supporting_evidence_ids.iter().cloned());

    Ok(WaterEnergyInterventionComparisonAssessment {
        schema_version: WATER_ENERGY_INTERVENTION_SCHEMA_VERSION.to_owned(),
        comparison_id: request.id.clone(),
        service_entity_id: request.baseline.trajectory.service_entity_id.clone(),
        effect: effect_from_bounds(total_reduction_lower, total_reduction_upper),
        total_unmet_demand_reduction_lower_bound: total_reduction_lower,
        total_unmet_demand_reduction_upper_bound: total_reduction_upper,
        total_customer_delivery_delta_lower_bound: total_delivery_delta_lower,
        total_customer_delivery_delta_upper_bound: total_delivery_delta_upper,
        baseline,
        candidate,
        steps,
        supporting_evidence_ids: all_evidence_ids.into_iter().collect(),
        interpretation_boundary:
            WaterEnergyInterventionInterpretationBoundary::ModeledCounterfactualNotCausalEvidence,
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
        WATER_ENERGY_COUPLING_SCHEMA_VERSION, WaterEnergyCouplingStepInput,
    };
    use crate::service_water_trajectory::{
        WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION, WaterServiceTrajectory, WaterServiceTrajectoryStep,
    };
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceRecord, EvidenceState, Provenance, Quantity,
        SpatialContext, TimeContext,
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
            rationale: "Synthetic comparison assumption.".to_owned(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn coupling_request(id: &str, power: f64) -> WaterEnergyCouplingRequest {
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
                demand_rate: measurement("community", 100.0, WATER_RATE_UNIT),
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
                available_power: measurement("pump", power, ELECTRIC_POWER_UNIT),
                specific_energy: measurement("pump-regime", 0.5, PUMP_SPECIFIC_ENERGY_UNIT),
            })
            .collect();
        WaterEnergyCouplingRequest {
            schema_version: WATER_ENERGY_COUPLING_SCHEMA_VERSION.to_owned(),
            id: id.to_owned(),
            trajectory: WaterServiceTrajectory {
                schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
                id: format!("trajectory-{id}"),
                service_entity_id: "community".to_owned(),
                initial_storage: measurement("tank", 0.0, WATER_VOLUME_UNIT),
                storage_capacity: measurement("tank", 100.0, WATER_VOLUME_UNIT),
                steps,
            },
            steps: energy_steps,
        }
    }

    fn comparison(
        baseline_power: f64,
        candidate_power: f64,
    ) -> WaterEnergyInterventionComparisonRequest {
        WaterEnergyInterventionComparisonRequest {
            schema_version: WATER_ENERGY_INTERVENTION_SCHEMA_VERSION.to_owned(),
            id: "comparison".to_owned(),
            baseline: coupling_request("baseline", baseline_power),
            candidate: coupling_request("candidate", candidate_power),
        }
    }

    fn observed(id: &str, entity: &str, value: f64, unit: &str, at: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.to_owned(),
            schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:sei-tests".to_owned(),
            subject_id: entity.to_owned(),
            spatial: SpatialContext {
                entity_id: entity.to_owned(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: at.to_owned(),
                recorded_at: at.to_owned(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic observation used to test provenance wiring.".to_owned(),
            quantity: Some(Quantity {
                value,
                unit: unit.to_owned(),
                uncertainty_abs: None,
            }),
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

    #[test]
    fn classifies_robust_improvement_from_reduced_unmet_demand() {
        let output = compare_water_energy_trajectory_interventions(&comparison(20.0, 40.0), &[])
            .expect("aligned synthetic comparison");
        assert_eq!(
            output.effect,
            WaterEnergyInterventionEffect::RobustImprovement
        );
        assert!((output.total_unmet_demand_reduction_lower_bound - 80.0).abs() < 1e-9);
        assert!((output.total_unmet_demand_reduction_upper_bound - 80.0).abs() < 1e-9);
        assert_eq!(
            output.interpretation_boundary,
            WaterEnergyInterventionInterpretationBoundary::ModeledCounterfactualNotCausalEvidence
        );
    }

    #[test]
    fn classifies_robust_regression_when_candidate_has_less_power() {
        let output = compare_water_energy_trajectory_interventions(&comparison(40.0, 10.0), &[])
            .expect("aligned synthetic comparison");
        assert_eq!(
            output.effect,
            WaterEnergyInterventionEffect::RobustRegression
        );
        assert!((output.total_unmet_demand_reduction_upper_bound + 120.0).abs() < 1e-9);
    }

    #[test]
    fn identical_trajectories_are_no_material_change() {
        let output = compare_water_energy_trajectory_interventions(&comparison(40.0, 40.0), &[])
            .expect("aligned synthetic comparison");
        assert_eq!(
            output.effect,
            WaterEnergyInterventionEffect::NoMaterialChange
        );
        assert_eq!(output.total_unmet_demand_reduction_lower_bound, 0.0);
        assert_eq!(output.total_unmet_demand_reduction_upper_bound, 0.0);
    }

    #[test]
    fn overlapping_capacity_intervals_remain_indeterminate() {
        let mut input = comparison(40.0, 40.0);
        for step in &mut input.baseline.steps {
            step.available_power.quantity.uncertainty_abs = Some(10.0);
        }
        for step in &mut input.candidate.steps {
            step.available_power.quantity.uncertainty_abs = Some(20.0);
        }
        let output = compare_water_energy_trajectory_interventions(&input, &[])
            .expect("overlapping intervals remain evaluable");
        assert_eq!(output.effect, WaterEnergyInterventionEffect::Indeterminate);
        assert!(output.total_unmet_demand_reduction_lower_bound < 0.0);
        assert!(output.total_unmet_demand_reduction_upper_bound > 0.0);
    }

    #[test]
    fn refuses_misaligned_time_grids() {
        let mut input = comparison(20.0, 40.0);
        input.candidate.trajectory.steps[0].as_of = "2026-10-16T08:00:00Z".to_owned();
        let result = compare_water_energy_trajectory_interventions(&input, &[]);
        assert!(matches!(
            result,
            Err(WaterEnergyInterventionComparisonError::InvalidInput(errors))
                if errors.contains(&WaterEnergyInterventionValidationError::StepTimeMismatch)
        ));
    }

    #[test]
    fn validates_shared_evidence_corpus_before_assessing_either_side() {
        let input = comparison(20.0, 40.0);
        let duplicate = observed("duplicate", "source", 100.0, WATER_RATE_UNIT, TIMES[0]);
        let result =
            compare_water_energy_trajectory_interventions(&input, &[duplicate.clone(), duplicate]);
        assert!(matches!(
            result,
            Err(WaterEnergyInterventionComparisonError::EvidenceValidation(
                ServiceFunctionAssessmentError::InvalidInput(errors)
            )) if errors.contains(
                &crate::service_function::ServiceFunctionValidationError::DuplicateEvidenceRecordId
            )
        ));
    }

    #[test]
    fn keeps_water_and_energy_provenance_separate_for_each_side() {
        let mut input = comparison(20.0, 40.0);
        let mut evidence = Vec::new();

        for side in ["baseline", "candidate"] {
            let coupling = if side == "baseline" {
                &mut input.baseline
            } else {
                &mut input.candidate
            };
            for index in 0..2 {
                let at = TIMES[index];
                let water_id = format!("{side}-water-source-{index}");
                coupling.trajectory.steps[index].source_flow_rate.basis = ScenarioBasis::Observed;
                coupling.trajectory.steps[index]
                    .source_flow_rate
                    .supporting_evidence_ids = vec![water_id.clone()];
                evidence.push(observed(&water_id, "source", 100.0, WATER_RATE_UNIT, at));

                let power_id = format!("{side}-terminal-power-{index}");
                let power_value = coupling.steps[index].available_power.quantity.value;
                coupling.steps[index].available_power.basis = ScenarioBasis::Observed;
                coupling.steps[index]
                    .available_power
                    .supporting_evidence_ids = vec![power_id.clone()];
                evidence.push(observed(
                    &power_id,
                    "pump",
                    power_value,
                    ELECTRIC_POWER_UNIT,
                    at,
                ));
            }
        }

        let output = compare_water_energy_trajectory_interventions(&input, &evidence)
            .expect("observed inputs have exact matching evidence");
        let first = &output.steps[0];
        assert!(
            first
                .baseline_supporting_evidence_ids
                .contains(&"baseline-water-source-0".to_owned())
        );
        assert!(
            first
                .baseline_supporting_evidence_ids
                .contains(&"baseline-terminal-power-0".to_owned())
        );
        assert!(
            first
                .candidate_supporting_evidence_ids
                .contains(&"candidate-water-source-0".to_owned())
        );
        assert!(
            first
                .candidate_supporting_evidence_ids
                .contains(&"candidate-terminal-power-0".to_owned())
        );
        assert!(
            !first
                .baseline_supporting_evidence_ids
                .contains(&"candidate-terminal-power-0".to_owned())
        );
        assert!(
            !first
                .candidate_supporting_evidence_ids
                .contains(&"baseline-terminal-power-0".to_owned())
        );
    }
}
