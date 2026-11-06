// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Paired, uncertainty-aware counterfactual comparison of water-service trajectories.
//! A modeled delta is not evidence that an intervention was executed or caused an outcome.

use crate::service_function::{ServiceFunctionAssessmentError, build_evidence_index};
use crate::service_water::WaterServiceState;
use crate::service_water_trajectory::{
    WaterServiceTrajectory, WaterServiceTrajectoryAssessment, WaterServiceTrajectoryError,
    WaterServiceTrajectoryStepAssessment, assess_trajectory_indexed,
};
use crate::spatial_economic::EvidenceRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const WATER_SERVICE_TRAJECTORY_INTERVENTION_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceTrajectoryInterventionRequest {
    pub schema_version: String,
    pub id: String,
    pub rationale: String,
    pub baseline: WaterServiceTrajectory,
    /// Explicit candidate assumptions or candidate observed evidence, not proof of execution.
    pub candidate: WaterServiceTrajectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterTrajectoryInterventionEffect {
    RobustImprovement,
    RobustRegression,
    NoMaterialChange,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterTrajectoryRecoveryComparison {
    CandidateEarlier,
    BaselineEarlier,
    BothSameSample,
    CandidateOnlyWithinWindow,
    BaselineOnlyWithinWindow,
    NeitherWithinWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterTrajectoryInterpretationBoundary {
    ModeledCounterfactualNotCausalEvidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterTrajectoryInterventionStepComparison {
    pub as_of: String,
    pub baseline_step_id: String,
    pub candidate_step_id: String,
    pub baseline_state: WaterServiceState,
    pub candidate_state: WaterServiceState,
    pub baseline_unmet_demand_lower_bound: f64,
    pub baseline_unmet_demand_upper_bound: f64,
    pub candidate_unmet_demand_lower_bound: f64,
    pub candidate_unmet_demand_upper_bound: f64,
    /// Baseline deficit minus candidate deficit; positive means modeled deficit fell.
    pub unmet_demand_reduction_lower_bound: f64,
    pub unmet_demand_reduction_upper_bound: f64,
    pub baseline_supporting_evidence_ids: Vec<String>,
    pub candidate_supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterServiceTrajectoryInterventionComparison {
    pub schema_version: String,
    pub comparison_id: String,
    pub rationale: String,
    pub service_entity_id: String,
    pub baseline_trajectory_id: String,
    pub candidate_trajectory_id: String,
    pub rate_unit: String,
    pub volume_unit: String,
    pub time_unit: String,
    pub paired_interval_count: usize,
    pub baseline_unmet_demand_lower_bound: f64,
    pub baseline_unmet_demand_upper_bound: f64,
    pub candidate_unmet_demand_lower_bound: f64,
    pub candidate_unmet_demand_upper_bound: f64,
    /// Difference in total unmet-demand volume; a positive lower bound is robust improvement.
    pub unmet_demand_reduction_lower_bound: f64,
    pub unmet_demand_reduction_upper_bound: f64,
    /// Candidate minus baseline delivered volume; positive means greater modeled delivery.
    pub customer_delivery_delta_lower_bound: f64,
    pub customer_delivery_delta_upper_bound: f64,
    pub effect: WaterTrajectoryInterventionEffect,
    pub recovery_comparison: WaterTrajectoryRecoveryComparison,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_first_adequate_as_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_first_adequate_as_of: Option<String>,
    pub interpretation_boundary: WaterTrajectoryInterpretationBoundary,
    pub steps: Vec<WaterTrajectoryInterventionStepComparison>,
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterTrajectoryInterventionValidationError {
    UnsupportedSchemaVersion,
    EmptyRequestSet,
    DuplicateComparisonId,
    EmptyComparisonId,
    EmptyRationale,
    SameBaselineAndCandidateId,
    ServiceEntityMismatch,
    StepCountMismatch,
    TimeGridMismatch,
    StepDurationMismatch,
    NonFiniteDelta,
    NonFiniteAggregate,
    BoundsInconsistent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterServiceTrajectoryInterventionError {
    InvalidInput(Vec<WaterTrajectoryInterventionValidationError>),
    EvidenceValidation(ServiceFunctionAssessmentError),
    TrajectoryAssessment(WaterServiceTrajectoryError),
}

fn validate_request(
    request: &WaterServiceTrajectoryInterventionRequest,
) -> Result<(), Vec<WaterTrajectoryInterventionValidationError>> {
    let mut errors = Vec::new();
    if request.schema_version != WATER_SERVICE_TRAJECTORY_INTERVENTION_SCHEMA_VERSION {
        errors.push(WaterTrajectoryInterventionValidationError::UnsupportedSchemaVersion);
    }
    if request.id.trim().is_empty() {
        errors.push(WaterTrajectoryInterventionValidationError::EmptyComparisonId);
    }
    if request.rationale.trim().is_empty() {
        errors.push(WaterTrajectoryInterventionValidationError::EmptyRationale);
    }
    if request.baseline.id == request.candidate.id {
        errors.push(WaterTrajectoryInterventionValidationError::SameBaselineAndCandidateId);
    }
    if request.baseline.service_entity_id != request.candidate.service_entity_id {
        errors.push(WaterTrajectoryInterventionValidationError::ServiceEntityMismatch);
    }
    if request.baseline.steps.len() != request.candidate.steps.len() {
        errors.push(WaterTrajectoryInterventionValidationError::StepCountMismatch);
    } else {
        for (baseline, candidate) in request.baseline.steps.iter().zip(&request.candidate.steps) {
            if baseline.as_of != candidate.as_of {
                errors.push(WaterTrajectoryInterventionValidationError::TimeGridMismatch);
                break;
            }
            if baseline.step_hours != candidate.step_hours {
                errors.push(WaterTrajectoryInterventionValidationError::StepDurationMismatch);
                break;
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn deficit_bounds(step: &WaterServiceTrajectoryStepAssessment) -> (f64, f64) {
    (
        (step.demand_volume_lower_bound - step.customer_delivered_volume_upper_bound).max(0.0),
        (step.demand_volume_upper_bound - step.customer_delivered_volume_lower_bound).max(0.0),
    )
}

fn recovery_comparison(
    baseline: &WaterServiceTrajectoryAssessment,
    candidate: &WaterServiceTrajectoryAssessment,
) -> WaterTrajectoryRecoveryComparison {
    match (
        baseline.first_adequate_as_of.as_deref(),
        candidate.first_adequate_as_of.as_deref(),
    ) {
        (Some(base), Some(cand)) if cand < base => {
            WaterTrajectoryRecoveryComparison::CandidateEarlier
        }
        (Some(base), Some(cand)) if base < cand => {
            WaterTrajectoryRecoveryComparison::BaselineEarlier
        }
        (Some(_), Some(_)) => WaterTrajectoryRecoveryComparison::BothSameSample,
        (None, Some(_)) => WaterTrajectoryRecoveryComparison::CandidateOnlyWithinWindow,
        (Some(_), None) => WaterTrajectoryRecoveryComparison::BaselineOnlyWithinWindow,
        (None, None) => WaterTrajectoryRecoveryComparison::NeitherWithinWindow,
    }
}

fn compare_indexed(
    request: &WaterServiceTrajectoryInterventionRequest,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<WaterServiceTrajectoryInterventionComparison, WaterServiceTrajectoryInterventionError> {
    validate_request(request).map_err(WaterServiceTrajectoryInterventionError::InvalidInput)?;
    let baseline = assess_trajectory_indexed(&request.baseline, evidence_by_id)
        .map_err(WaterServiceTrajectoryInterventionError::TrajectoryAssessment)?;
    let candidate = assess_trajectory_indexed(&request.candidate, evidence_by_id)
        .map_err(WaterServiceTrajectoryInterventionError::TrajectoryAssessment)?;

    let reduction_lower =
        baseline.unmet_demand_volume_lower_bound - candidate.unmet_demand_volume_upper_bound;
    let reduction_upper =
        baseline.unmet_demand_volume_upper_bound - candidate.unmet_demand_volume_lower_bound;
    let delivery_delta_lower = candidate.customer_delivered_volume_lower_bound
        - baseline.customer_delivered_volume_upper_bound;
    let delivery_delta_upper = candidate.customer_delivered_volume_upper_bound
        - baseline.customer_delivered_volume_lower_bound;
    let tol = f64::EPSILON
        * 256.0
        * baseline
            .unmet_demand_volume_upper_bound
            .max(candidate.unmet_demand_volume_upper_bound)
            .max(baseline.customer_delivered_volume_upper_bound)
            .max(candidate.customer_delivered_volume_upper_bound)
            .max(1.0);

    if [
        reduction_lower,
        reduction_upper,
        delivery_delta_lower,
        delivery_delta_upper,
    ]
    .into_iter()
    .any(|value| !value.is_finite())
    {
        return Err(WaterServiceTrajectoryInterventionError::InvalidInput(vec![
            WaterTrajectoryInterventionValidationError::NonFiniteDelta,
        ]));
    }
    if reduction_lower > reduction_upper + tol
        || delivery_delta_lower > delivery_delta_upper + tol
        || baseline.unmet_demand_volume_lower_bound > baseline.unmet_demand_volume_upper_bound + tol
        || candidate.unmet_demand_volume_lower_bound
            > candidate.unmet_demand_volume_upper_bound + tol
    {
        return Err(WaterServiceTrajectoryInterventionError::InvalidInput(vec![
            WaterTrajectoryInterventionValidationError::BoundsInconsistent,
        ]));
    }

    let effect = if reduction_lower > tol {
        WaterTrajectoryInterventionEffect::RobustImprovement
    } else if reduction_upper < -tol {
        WaterTrajectoryInterventionEffect::RobustRegression
    } else if reduction_lower.abs() <= tol && reduction_upper.abs() <= tol {
        WaterTrajectoryInterventionEffect::NoMaterialChange
    } else {
        WaterTrajectoryInterventionEffect::Indeterminate
    };

    let mut evidence_ids = BTreeSet::new();
    let mut steps = Vec::with_capacity(baseline.steps.len());
    for (base_step, cand_step) in baseline.steps.iter().zip(&candidate.steps) {
        let (base_deficit_lower, base_deficit_upper) = deficit_bounds(base_step);
        let (cand_deficit_lower, cand_deficit_upper) = deficit_bounds(cand_step);
        let step_reduction_lower = base_deficit_lower - cand_deficit_upper;
        let step_reduction_upper = base_deficit_upper - cand_deficit_lower;
        if [
            base_deficit_lower,
            base_deficit_upper,
            cand_deficit_lower,
            cand_deficit_upper,
            step_reduction_lower,
            step_reduction_upper,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
        {
            return Err(WaterServiceTrajectoryInterventionError::InvalidInput(vec![
                WaterTrajectoryInterventionValidationError::NonFiniteDelta,
            ]));
        }
        if step_reduction_lower > step_reduction_upper + tol {
            return Err(WaterServiceTrajectoryInterventionError::InvalidInput(vec![
                WaterTrajectoryInterventionValidationError::BoundsInconsistent,
            ]));
        }
        evidence_ids.extend(base_step.supporting_evidence_ids.iter().cloned());
        evidence_ids.extend(cand_step.supporting_evidence_ids.iter().cloned());
        steps.push(WaterTrajectoryInterventionStepComparison {
            as_of: base_step.as_of.clone(),
            baseline_step_id: base_step.step_id.clone(),
            candidate_step_id: cand_step.step_id.clone(),
            baseline_state: base_step.state,
            candidate_state: cand_step.state,
            baseline_unmet_demand_lower_bound: base_deficit_lower,
            baseline_unmet_demand_upper_bound: base_deficit_upper,
            candidate_unmet_demand_lower_bound: cand_deficit_lower,
            candidate_unmet_demand_upper_bound: cand_deficit_upper,
            unmet_demand_reduction_lower_bound: step_reduction_lower,
            unmet_demand_reduction_upper_bound: step_reduction_upper,
            baseline_supporting_evidence_ids: base_step.supporting_evidence_ids.clone(),
            candidate_supporting_evidence_ids: cand_step.supporting_evidence_ids.clone(),
        });
    }

    let recovery_comparison = recovery_comparison(&baseline, &candidate);
    let baseline_first_adequate_as_of = baseline.first_adequate_as_of.clone();
    let candidate_first_adequate_as_of = candidate.first_adequate_as_of.clone();

    Ok(WaterServiceTrajectoryInterventionComparison {
        schema_version: WATER_SERVICE_TRAJECTORY_INTERVENTION_SCHEMA_VERSION.to_owned(),
        comparison_id: request.id.clone(),
        rationale: request.rationale.clone(),
        service_entity_id: baseline.service_entity_id.clone(),
        baseline_trajectory_id: baseline.trajectory_id.clone(),
        candidate_trajectory_id: candidate.trajectory_id.clone(),
        rate_unit: baseline.rate_unit,
        volume_unit: baseline.volume_unit,
        time_unit: baseline.time_unit,
        paired_interval_count: steps.len(),
        baseline_unmet_demand_lower_bound: baseline.unmet_demand_volume_lower_bound,
        baseline_unmet_demand_upper_bound: baseline.unmet_demand_volume_upper_bound,
        candidate_unmet_demand_lower_bound: candidate.unmet_demand_volume_lower_bound,
        candidate_unmet_demand_upper_bound: candidate.unmet_demand_volume_upper_bound,
        unmet_demand_reduction_lower_bound: reduction_lower,
        unmet_demand_reduction_upper_bound: reduction_upper,
        customer_delivery_delta_lower_bound: delivery_delta_lower,
        customer_delivery_delta_upper_bound: delivery_delta_upper,
        effect,
        recovery_comparison,
        baseline_first_adequate_as_of,
        candidate_first_adequate_as_of,
        interpretation_boundary:
            WaterTrajectoryInterpretationBoundary::ModeledCounterfactualNotCausalEvidence,
        steps,
        supporting_evidence_ids: evidence_ids.into_iter().collect(),
    })
}

/// Compare baseline/candidate trajectory pairs against one shared evidence corpus.
/// The corpus is validated/indexed once and outputs sort by stable comparison ID.
pub fn compare_water_service_trajectory_interventions(
    requests: &[WaterServiceTrajectoryInterventionRequest],
    evidence_records: &[EvidenceRecord],
) -> Result<
    Vec<WaterServiceTrajectoryInterventionComparison>,
    WaterServiceTrajectoryInterventionError,
> {
    if requests.is_empty() {
        return Err(WaterServiceTrajectoryInterventionError::InvalidInput(vec![
            WaterTrajectoryInterventionValidationError::EmptyRequestSet,
        ]));
    }
    let mut ids = BTreeSet::new();
    for request in requests {
        validate_request(request).map_err(WaterServiceTrajectoryInterventionError::InvalidInput)?;
        if !ids.insert(request.id.as_str()) {
            return Err(WaterServiceTrajectoryInterventionError::InvalidInput(vec![
                WaterTrajectoryInterventionValidationError::DuplicateComparisonId,
            ]));
        }
    }
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(WaterServiceTrajectoryInterventionError::EvidenceValidation)?;
    let mut ordered: Vec<&WaterServiceTrajectoryInterventionRequest> = requests.iter().collect();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    ordered
        .into_iter()
        .map(|request| compare_indexed(request, &evidence_by_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::service_function::FunctionMeasurement;
    use crate::service_water::{DIMENSIONLESS_UNIT, WATER_RATE_UNIT, WATER_VOLUME_UNIT};
    use crate::service_water_trajectory::{
        WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION, WaterServiceTrajectoryStep,
    };
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceRecord, EvidenceState, Provenance, Quantity,
        SpatialContext, TimeContext,
    };

    fn measurement(
        entity: &str,
        value: f64,
        unit: &str,
        uncertainty: Option<f64>,
    ) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity.to_owned(),
            quantity: Quantity {
                value,
                unit: unit.to_owned(),
                uncertainty_abs: uncertainty,
            },
            basis: ScenarioBasis::Assumed,
            rationale: format!("Synthetic counterfactual assumption for {entity}"),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn step(
        id: &str,
        hour: u32,
        source_rate: f64,
        uncertainty: Option<f64>,
    ) -> WaterServiceTrajectoryStep {
        let distribution = "distribution-a";
        WaterServiceTrajectoryStep {
            id: id.to_owned(),
            as_of: format!("2026-10-16T{hour:02}:00:00Z"),
            step_hours: 1.0,
            source_flow_rate: measurement("source-a", source_rate, WATER_RATE_UNIT, uncertainty),
            pump_capacity: measurement("pump-a", 100.0, WATER_RATE_UNIT, None),
            treatment_capacity: measurement("treatment-a", 100.0, WATER_RATE_UNIT, None),
            distribution_capacity: measurement(distribution, 120.0, WATER_RATE_UNIT, None),
            demand_rate: measurement("service-north", 100.0, WATER_RATE_UNIT, None),
            distribution_loss_fraction: measurement(distribution, 0.0, DIMENSIONLESS_UNIT, None),
        }
    }

    fn trajectory(
        id: &str,
        outage_source: f64,
        outage_uncertainty: Option<f64>,
    ) -> WaterServiceTrajectory {
        WaterServiceTrajectory {
            schema_version: WATER_SERVICE_TRAJECTORY_SCHEMA_VERSION.to_owned(),
            id: id.to_owned(),
            service_entity_id: "service-north".to_owned(),
            initial_storage: measurement("tank-a", 50.0, WATER_VOLUME_UNIT, None),
            storage_capacity: measurement("tank-a", 150.0, WATER_VOLUME_UNIT, None),
            steps: vec![
                step("normal", 10, 100.0, None),
                step("outage-1", 11, outage_source, outage_uncertainty),
                step("outage-2", 12, outage_source, outage_uncertainty),
                step("recovery", 13, 100.0, None),
            ],
        }
    }

    fn request(
        id: &str,
        baseline: WaterServiceTrajectory,
        candidate: WaterServiceTrajectory,
    ) -> WaterServiceTrajectoryInterventionRequest {
        WaterServiceTrajectoryInterventionRequest {
            schema_version: WATER_SERVICE_TRAJECTORY_INTERVENTION_SCHEMA_VERSION.to_owned(),
            id: id.to_owned(),
            rationale: "Synthetic intervention scenario; no field effect is implied.".to_owned(),
            baseline,
            candidate,
        }
    }

    #[test]
    fn robust_intervention_reduces_unmet_demand_and_compares_recovery() {
        let input = request(
            "pump-backup",
            trajectory("base", 0.0, None),
            trajectory("backup", 50.0, None),
        );
        let result = compare_water_service_trajectory_interventions(&[input], &[]).unwrap();
        assert_eq!(result[0].baseline_unmet_demand_lower_bound, 150.0);
        assert_eq!(result[0].candidate_unmet_demand_lower_bound, 50.0);
        assert_eq!(result[0].unmet_demand_reduction_lower_bound, 100.0);
        assert_eq!(result[0].unmet_demand_reduction_upper_bound, 100.0);
        assert_eq!(
            result[0]
                .steps
                .iter()
                .map(|step| step.unmet_demand_reduction_lower_bound)
                .sum::<f64>(),
            result[0].unmet_demand_reduction_lower_bound
        );
        assert_eq!(
            result[0].effect,
            WaterTrajectoryInterventionEffect::RobustImprovement
        );
        assert_eq!(
            result[0].recovery_comparison,
            WaterTrajectoryRecoveryComparison::BothSameSample
        );
        assert_eq!(
            result[0].interpretation_boundary,
            WaterTrajectoryInterpretationBoundary::ModeledCounterfactualNotCausalEvidence
        );
    }

    #[test]
    fn uncertainty_that_overlaps_baseline_benefit_is_indeterminate() {
        let input = request(
            "uncertain-backup",
            trajectory("base", 0.0, None),
            trajectory("candidate", 50.0, Some(50.0)),
        );
        let result = compare_water_service_trajectory_interventions(&[input], &[]).unwrap();
        assert_eq!(
            result[0].effect,
            WaterTrajectoryInterventionEffect::Indeterminate
        );
        assert!(result[0].unmet_demand_reduction_lower_bound <= 0.0);
        assert!(result[0].unmet_demand_reduction_upper_bound > 0.0);
    }

    #[test]
    fn regression_is_detected_when_candidate_is_worse() {
        let input = request(
            "worse",
            trajectory("base", 50.0, None),
            trajectory("candidate", 0.0, None),
        );
        let result = compare_water_service_trajectory_interventions(&[input], &[]).unwrap();
        assert_eq!(
            result[0].effect,
            WaterTrajectoryInterventionEffect::RobustRegression
        );
        assert!(result[0].unmet_demand_reduction_upper_bound < 0.0);
    }

    #[test]
    fn rejects_time_grid_mismatch() {
        let mut candidate = trajectory("candidate", 50.0, None);
        candidate.steps[2].as_of = "2026-10-16T14:00:00Z".to_owned();
        let input = request("mismatch", trajectory("base", 0.0, None), candidate);
        assert!(matches!(
            compare_water_service_trajectory_interventions(&[input], &[]),
            Err(WaterServiceTrajectoryInterventionError::InvalidInput(errors))
                if errors.contains(&WaterTrajectoryInterventionValidationError::TimeGridMismatch)
        ));
    }

    fn source_evidence(id: &str, value: f64, at: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.to_owned(),
            schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:water-intervention-test".to_owned(),
            subject_id: "source-a".to_owned(),
            spatial: SpatialContext {
                entity_id: "source-a".to_owned(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: at.to_owned(),
                recorded_at: at.to_owned(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic source-flow observation for evidence-attribution test."
                .to_owned(),
            quantity: Some(Quantity {
                value,
                unit: WATER_RATE_UNIT.to_owned(),
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
    fn paired_step_keeps_baseline_and_candidate_evidence_separate() {
        let mut baseline = trajectory("baseline-evidence", 0.0, None);
        let mut candidate = trajectory("candidate-evidence", 50.0, None);
        baseline.steps[1].source_flow_rate.basis = ScenarioBasis::Observed;
        baseline.steps[1].source_flow_rate.supporting_evidence_ids =
            vec!["baseline-flow".to_owned()];
        candidate.steps[1].source_flow_rate.basis = ScenarioBasis::Observed;
        candidate.steps[1].source_flow_rate.supporting_evidence_ids =
            vec!["candidate-flow".to_owned()];
        let evidence = [
            source_evidence("baseline-flow", 0.0, "2026-10-16T11:00:00Z"),
            source_evidence("candidate-flow", 50.0, "2026-10-16T11:00:00Z"),
        ];
        let input = request("evidence-pair", baseline, candidate);
        let result = compare_water_service_trajectory_interventions(&[input], &evidence).unwrap();
        assert_eq!(
            result[0].steps[1].baseline_supporting_evidence_ids,
            vec!["baseline-flow".to_owned()]
        );
        assert_eq!(
            result[0].steps[1].candidate_supporting_evidence_ids,
            vec!["candidate-flow".to_owned()]
        );
        assert_eq!(
            result[0].supporting_evidence_ids,
            vec!["baseline-flow".to_owned(), "candidate-flow".to_owned()]
        );
    }

    #[test]
    fn identical_trajectories_are_no_material_change() {
        let input = request(
            "no-change",
            trajectory("baseline-same", 0.0, None),
            trajectory("candidate-same", 0.0, None),
        );
        let result = compare_water_service_trajectory_interventions(&[input], &[]).unwrap();
        assert_eq!(
            result[0].effect,
            WaterTrajectoryInterventionEffect::NoMaterialChange
        );
        assert_eq!(result[0].unmet_demand_reduction_lower_bound, 0.0);
        assert_eq!(result[0].unmet_demand_reduction_upper_bound, 0.0);
    }

    #[test]
    fn candidate_only_recovery_is_distinguished_within_observation_window() {
        let mut baseline = trajectory("baseline-stays-down", 0.0, None);
        baseline.steps[3].source_flow_rate.quantity.value = 0.0;
        let candidate = trajectory("candidate-recovers", 50.0, None);
        let input = request("candidate-recovers-only", baseline, candidate);
        let result = compare_water_service_trajectory_interventions(&[input], &[]).unwrap();
        assert_eq!(
            result[0].recovery_comparison,
            WaterTrajectoryRecoveryComparison::CandidateOnlyWithinWindow
        );
        assert_eq!(result[0].baseline_first_adequate_as_of, None);
        assert_eq!(
            result[0].candidate_first_adequate_as_of.as_deref(),
            Some("2026-10-16T13:00:00Z")
        );
    }

    #[test]
    fn batch_is_sorted_and_duplicate_comparison_ids_fail_closed() {
        let first = request(
            "zeta",
            trajectory("base-z", 0.0, None),
            trajectory("candidate-z", 50.0, None),
        );
        let second = request(
            "alpha",
            trajectory("base-a", 0.0, None),
            trajectory("candidate-a", 50.0, None),
        );
        let sorted =
            compare_water_service_trajectory_interventions(&[first.clone(), second.clone()], &[])
                .unwrap();
        assert_eq!(sorted[0].comparison_id, "alpha");
        assert_eq!(sorted[1].comparison_id, "zeta");
        let reversed =
            compare_water_service_trajectory_interventions(&[second, first], &[]).unwrap();
        assert_eq!(
            serde_json::to_vec(&sorted).unwrap(),
            serde_json::to_vec(&reversed).unwrap()
        );
        let duplicates = [
            request(
                "same",
                trajectory("base-1", 0.0, None),
                trajectory("candidate-1", 50.0, None),
            ),
            request(
                "same",
                trajectory("base-2", 0.0, None),
                trajectory("candidate-2", 50.0, None),
            ),
        ];
        assert!(matches!(
            compare_water_service_trajectory_interventions(&duplicates, &[]),
            Err(WaterServiceTrajectoryInterventionError::InvalidInput(errors))
                if errors.contains(
                    &WaterTrajectoryInterventionValidationError::DuplicateComparisonId,
                )
        ));
    }
}
