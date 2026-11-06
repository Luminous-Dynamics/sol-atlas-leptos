// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-aware functional adequacy for services with measurable capacity.
//!
//! Binary "available/unavailable" models hide partial service. This module
//! compares an available capacity interval with a required-capacity interval,
//! returning explicit coverage bounds and a qualitative state. It does not
//! infer failure probabilities, convert units, or prove causal impact.

use crate::dependency_logic::ScenarioBasis;
use crate::spatial_economic::{EvidenceKind, EvidenceRecord, EvidenceState, Quantity};
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SERVICE_FUNCTION_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionMeasurement {
    /// Entity whose capacity or requirement this value describes.
    pub entity_id: String,
    pub quantity: Quantity,
    pub basis: ScenarioBasis,
    pub rationale: String,
    #[serde(default)]
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceFunctionScenario {
    pub schema_version: String,
    pub id: String,
    /// Canonical UTC timestamp: YYYY-MM-DDTHH:MM:SSZ.
    pub as_of: String,
    pub service_entity_id: String,
    /// Measured or assumed usable capacity, in an explicitly named unit.
    pub available_capacity: FunctionMeasurement,
    /// Minimum capacity needed to meet the service requirement, in the same unit.
    pub required_capacity: FunctionMeasurement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceFunctionState {
    /// The conservative capacity lower bound meets the requirement upper bound.
    Adequate,
    /// The service has definitely non-zero capacity, but even its upper bound
    /// remains below the requirement lower bound.
    Degraded,
    /// The capacity upper bound is zero and the requirement lower bound is > 0.
    Unavailable,
    /// Measurement uncertainty overlaps the adequacy boundary, or a zero-capacity
    /// versus partial-capacity conclusion cannot be safely distinguished.
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceFunctionAssessment {
    pub schema_version: String,
    pub scenario_id: String,
    pub service_entity_id: String,
    pub as_of: String,
    pub state: ServiceFunctionState,
    pub unit: String,
    pub available_capacity_value: f64,
    pub available_capacity_lower_bound: f64,
    pub available_capacity_upper_bound: f64,
    pub required_capacity_value: f64,
    pub required_capacity_lower_bound: f64,
    pub required_capacity_upper_bound: f64,
    /// Point estimate of capacity / requirement; absent when requirement is zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_ratio: Option<f64>,
    /// Conservative ratio: capacity lower bound / requirement upper bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_lower_bound: Option<f64>,
    /// Optimistic ratio: capacity upper bound / requirement lower bound.
    /// Absent if requirement lower bound is zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_upper_bound: Option<f64>,
    pub available_capacity_basis: ScenarioBasis,
    pub required_capacity_basis: ScenarioBasis,
    pub available_capacity_evidence_ids: Vec<String>,
    pub required_capacity_evidence_ids: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceFunctionValidationError {
    UnsupportedSchemaVersion,
    EmptyScenarioSet,
    DuplicateScenarioId,
    EmptyScenarioId,
    InvalidAsOfTimestamp,
    EmptyServiceEntityId,
    EmptyMeasurementEntityId,
    EmptyMeasurementRationale,
    EmptyUnit,
    NonFiniteValue,
    NegativeValue,
    NonFiniteUncertainty,
    NegativeUncertainty,
    NonFiniteDerivedValue,
    UnitMismatch,
    RequirementEntityMismatch,
    DuplicateEvidenceRecordId,
    InvalidEvidenceRecord,
    EmptySupportingEvidenceId,
    DuplicateSupportingEvidenceId,
    UnknownSupportingEvidence,
    MissingEvidenceForObservedMeasurement,
    EvidenceForbiddenForAssumedMeasurement,
    InadmissibleSupportingEvidence,
    SupportingEvidenceWrongKind,
    SupportingEvidenceSubjectMismatch,
    SupportingEvidenceMissingQuantity,
    SupportingEvidenceQuantityMismatch,
    InvalidSupportingEvidenceTimestamp,
    ObservationAfterAsOf,
    EvidenceRecordedAfterAsOf,
    InvalidSupportingEvidenceValidity,
    OutsideSupportingEvidenceValidity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceFunctionAssessmentError {
    InvalidInput(Vec<ServiceFunctionValidationError>),
}

/// Evaluate capacity against demand using conservative interval comparisons.
///
/// Observation evidence must match the exact quantity, unit and uncertainty carried
/// in the measurement. This deliberately avoids a second, conflicting value being
/// attached to the same evidence ID. Unit conversion must happen in an explicitly
/// versioned adapter before this call; this evaluator never guesses conversions.
pub fn assess_service_function(
    scenario: &ServiceFunctionScenario,
    evidence_records: &[EvidenceRecord],
) -> Result<ServiceFunctionAssessment, ServiceFunctionAssessmentError> {
    let evidence_by_id = build_evidence_index(evidence_records)?;
    assess_service_function_indexed(scenario, &evidence_by_id)
}

/// Batch API for a corpus of service-capacity assessments.
///
/// Evidence is validated and indexed once for the entire batch. Inputs and outputs
/// are sorted by stable scenario ID; duplicate IDs fail closed. This keeps large
/// evidence-backed assessments from revalidating the full corpus once per service.
pub fn assess_service_functions(
    scenarios: &[ServiceFunctionScenario],
    evidence_records: &[EvidenceRecord],
) -> Result<Vec<ServiceFunctionAssessment>, ServiceFunctionAssessmentError> {
    if scenarios.is_empty() {
        return Err(ServiceFunctionAssessmentError::InvalidInput(vec![
            ServiceFunctionValidationError::EmptyScenarioSet,
        ]));
    }

    let evidence_by_id = build_evidence_index(evidence_records)?;
    let mut scenario_ids = BTreeSet::new();
    for scenario in scenarios {
        if !scenario_ids.insert(scenario.id.as_str()) {
            return Err(ServiceFunctionAssessmentError::InvalidInput(vec![
                ServiceFunctionValidationError::DuplicateScenarioId,
            ]));
        }
    }

    let mut ordered: Vec<&ServiceFunctionScenario> = scenarios.iter().collect();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    ordered
        .into_iter()
        .map(|scenario| assess_service_function_indexed(scenario, &evidence_by_id))
        .collect()
}

pub(crate) fn build_evidence_index<'a>(
    evidence_records: &'a [EvidenceRecord],
) -> Result<BTreeMap<&'a str, &'a EvidenceRecord>, ServiceFunctionAssessmentError> {
    let mut errors = Vec::new();
    let mut evidence_by_id = BTreeMap::new();
    for record in evidence_records {
        if evidence_by_id.insert(record.id.as_str(), record).is_some() {
            errors.push(ServiceFunctionValidationError::DuplicateEvidenceRecordId);
        }
        if record.validate().is_err() {
            errors.push(ServiceFunctionValidationError::InvalidEvidenceRecord);
        }
    }
    if errors.is_empty() {
        Ok(evidence_by_id)
    } else {
        Err(ServiceFunctionAssessmentError::InvalidInput(errors))
    }
}

fn assess_service_function_indexed(
    scenario: &ServiceFunctionScenario,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<ServiceFunctionAssessment, ServiceFunctionAssessmentError> {
    let mut errors = Vec::new();
    if scenario.schema_version != SERVICE_FUNCTION_SCHEMA_VERSION {
        errors.push(ServiceFunctionValidationError::UnsupportedSchemaVersion);
    }
    if scenario.id.trim().is_empty() {
        errors.push(ServiceFunctionValidationError::EmptyScenarioId);
    }
    if !is_canonical_utc_timestamp(&scenario.as_of) {
        errors.push(ServiceFunctionValidationError::InvalidAsOfTimestamp);
    }
    if scenario.service_entity_id.trim().is_empty() {
        errors.push(ServiceFunctionValidationError::EmptyServiceEntityId);
    }

    validate_measurement(
        &scenario.available_capacity,
        &scenario.as_of,
        evidence_by_id,
        &mut errors,
    );
    validate_measurement(
        &scenario.required_capacity,
        &scenario.as_of,
        evidence_by_id,
        &mut errors,
    );

    if scenario.available_capacity.quantity.unit != scenario.required_capacity.quantity.unit {
        errors.push(ServiceFunctionValidationError::UnitMismatch);
    }
    if scenario.required_capacity.entity_id != scenario.service_entity_id {
        errors.push(ServiceFunctionValidationError::RequirementEntityMismatch);
    }

    if !errors.is_empty() {
        return Err(ServiceFunctionAssessmentError::InvalidInput(errors));
    }

    let capacity = &scenario.available_capacity.quantity;
    let requirement = &scenario.required_capacity.quantity;
    let capacity_uncertainty = capacity.uncertainty_abs.unwrap_or(0.0);
    let requirement_uncertainty = requirement.uncertainty_abs.unwrap_or(0.0);
    let capacity_lower = (capacity.value - capacity_uncertainty).max(0.0);
    let capacity_upper = capacity.value + capacity_uncertainty;
    let requirement_lower = (requirement.value - requirement_uncertainty).max(0.0);
    let requirement_upper = requirement.value + requirement_uncertainty;

    let state = if capacity_lower >= requirement_upper {
        ServiceFunctionState::Adequate
    } else if capacity_upper == 0.0 && requirement_lower > 0.0 {
        ServiceFunctionState::Unavailable
    } else if capacity_lower > 0.0 && capacity_upper < requirement_lower {
        ServiceFunctionState::Degraded
    } else {
        ServiceFunctionState::Indeterminate
    };

    let coverage_ratio = (requirement.value > 0.0).then(|| capacity.value / requirement.value);
    let coverage_lower_bound =
        (requirement_upper > 0.0).then(|| capacity_lower / requirement_upper);
    let coverage_upper_bound =
        (requirement_lower > 0.0).then(|| capacity_upper / requirement_lower);

    if !capacity_lower.is_finite()
        || !capacity_upper.is_finite()
        || !requirement_lower.is_finite()
        || !requirement_upper.is_finite()
        || coverage_ratio.is_some_and(|value| !value.is_finite())
        || coverage_lower_bound.is_some_and(|value| !value.is_finite())
        || coverage_upper_bound.is_some_and(|value| !value.is_finite())
    {
        return Err(ServiceFunctionAssessmentError::InvalidInput(vec![
            ServiceFunctionValidationError::NonFiniteDerivedValue,
        ]));
    }

    Ok(ServiceFunctionAssessment {
        schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
        scenario_id: scenario.id.clone(),
        service_entity_id: scenario.service_entity_id.clone(),
        as_of: scenario.as_of.clone(),
        state,
        unit: capacity.unit.clone(),
        available_capacity_value: capacity.value,
        available_capacity_lower_bound: capacity_lower,
        available_capacity_upper_bound: capacity_upper,
        required_capacity_value: requirement.value,
        required_capacity_lower_bound: requirement_lower,
        required_capacity_upper_bound: requirement_upper,
        coverage_ratio,
        coverage_lower_bound,
        coverage_upper_bound,
        available_capacity_basis: scenario.available_capacity.basis,
        required_capacity_basis: scenario.required_capacity.basis,
        available_capacity_evidence_ids: scenario
            .available_capacity
            .supporting_evidence_ids
            .clone(),
        required_capacity_evidence_ids: scenario.required_capacity.supporting_evidence_ids.clone(),
        rationale: format!(
            "{}; {}",
            scenario.available_capacity.rationale, scenario.required_capacity.rationale
        ),
    })
}

pub(crate) fn validate_measurement(
    measurement: &FunctionMeasurement,
    as_of: &str,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
    errors: &mut Vec<ServiceFunctionValidationError>,
) {
    if measurement.entity_id.trim().is_empty() {
        errors.push(ServiceFunctionValidationError::EmptyMeasurementEntityId);
    }
    if measurement.rationale.trim().is_empty() {
        errors.push(ServiceFunctionValidationError::EmptyMeasurementRationale);
    }
    if measurement.quantity.unit.trim().is_empty() {
        errors.push(ServiceFunctionValidationError::EmptyUnit);
    }
    if !measurement.quantity.value.is_finite() {
        errors.push(ServiceFunctionValidationError::NonFiniteValue);
    } else if measurement.quantity.value < 0.0 {
        errors.push(ServiceFunctionValidationError::NegativeValue);
    }
    if let Some(uncertainty) = measurement.quantity.uncertainty_abs {
        if !uncertainty.is_finite() {
            errors.push(ServiceFunctionValidationError::NonFiniteUncertainty);
        } else if uncertainty < 0.0 {
            errors.push(ServiceFunctionValidationError::NegativeUncertainty);
        }
    }

    let mut seen = BTreeSet::new();
    for evidence_id in &measurement.supporting_evidence_ids {
        if evidence_id.trim().is_empty() {
            errors.push(ServiceFunctionValidationError::EmptySupportingEvidenceId);
        }
        if !seen.insert(evidence_id.as_str()) {
            errors.push(ServiceFunctionValidationError::DuplicateSupportingEvidenceId);
        }
    }

    match measurement.basis {
        ScenarioBasis::Assumed => {
            if !measurement.supporting_evidence_ids.is_empty() {
                errors.push(ServiceFunctionValidationError::EvidenceForbiddenForAssumedMeasurement);
            }
        }
        ScenarioBasis::Observed => {
            if measurement.supporting_evidence_ids.is_empty() {
                errors.push(ServiceFunctionValidationError::MissingEvidenceForObservedMeasurement);
            }
            for evidence_id in &measurement.supporting_evidence_ids {
                let Some(record) = evidence_by_id.get(evidence_id.as_str()) else {
                    errors.push(ServiceFunctionValidationError::UnknownSupportingEvidence);
                    continue;
                };

                if record.state != EvidenceState::Asserted
                    && record.state != EvidenceState::Corroborated
                {
                    errors.push(ServiceFunctionValidationError::InadmissibleSupportingEvidence);
                }
                if record.kind != EvidenceKind::Observation {
                    errors.push(ServiceFunctionValidationError::SupportingEvidenceWrongKind);
                }
                if record.subject_id != measurement.entity_id
                    || record.spatial.entity_id != measurement.entity_id
                {
                    errors.push(ServiceFunctionValidationError::SupportingEvidenceSubjectMismatch);
                }
                match &record.quantity {
                    None => errors
                        .push(ServiceFunctionValidationError::SupportingEvidenceMissingQuantity),
                    Some(quantity) => {
                        if quantity.value != measurement.quantity.value
                            || quantity.unit != measurement.quantity.unit
                            || quantity.uncertainty_abs != measurement.quantity.uncertainty_abs
                        {
                            errors.push(
                                ServiceFunctionValidationError::SupportingEvidenceQuantityMismatch,
                            );
                        }
                    }
                }
                validate_evidence_validity(record, as_of, errors);
            }
        }
    }
}

fn validate_evidence_validity(
    record: &EvidenceRecord,
    as_of: &str,
    errors: &mut Vec<ServiceFunctionValidationError>,
) {
    // This evaluator intentionally uses the graph's canonical UTC-second profile
    // so as-of comparisons are unambiguous and do not depend on local timezone.
    if !is_canonical_utc_timestamp(&record.time.occurred_at)
        || !is_canonical_utc_timestamp(&record.time.recorded_at)
    {
        errors.push(ServiceFunctionValidationError::InvalidSupportingEvidenceTimestamp);
        return;
    }
    if record.time.occurred_at.as_str() > as_of {
        errors.push(ServiceFunctionValidationError::ObservationAfterAsOf);
        return;
    }
    if record.time.recorded_at.as_str() > as_of {
        errors.push(ServiceFunctionValidationError::EvidenceRecordedAfterAsOf);
        return;
    }

    let start = record.time.valid_from.as_deref();
    let end = record.time.valid_until.as_deref();
    let invalid = start.is_some_and(|value| !is_canonical_utc_timestamp(value))
        || end.is_some_and(|value| !is_canonical_utc_timestamp(value))
        || matches!((start, end), (Some(first), Some(last)) if {
            is_canonical_utc_timestamp(first)
                && is_canonical_utc_timestamp(last)
                && first >= last
        });
    if invalid {
        errors.push(ServiceFunctionValidationError::InvalidSupportingEvidenceValidity);
    } else if start.is_some_and(|value| value > as_of) || end.is_some_and(|value| as_of >= value) {
        errors.push(ServiceFunctionValidationError::OutsideSupportingEvidenceValidity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceState, Provenance, SpatialContext,
        TimeContext,
    };

    const AT: &str = "2026-10-16T09:00:00Z";

    fn measurement(
        entity_id: &str,
        value: f64,
        unit: &str,
        uncertainty_abs: Option<f64>,
    ) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity_id.into(),
            quantity: Quantity {
                value,
                unit: unit.into(),
                uncertainty_abs,
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic explicitly assumed capacity".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario(
        capacity: FunctionMeasurement,
        required: FunctionMeasurement,
    ) -> ServiceFunctionScenario {
        ServiceFunctionScenario {
            schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
            id: "synthetic:service-capacity".into(),
            as_of: AT.into(),
            service_entity_id: "service:irrigation".into(),
            available_capacity: capacity,
            required_capacity: required,
        }
    }

    fn evidence(
        id: &str,
        entity_id: &str,
        quantity: Quantity,
        state: EvidenceState,
    ) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state,
            issuer_id: "did:example:service-function-tests".into(),
            subject_id: entity_id.into(),
            spatial: SpatialContext {
                entity_id: entity_id.into(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: AT.into(),
                recorded_at: AT.into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic measurement for unit tests".into(),
            quantity: Some(quantity),
            money: None,
            provenance: Provenance {
                method: "synthetic_test".into(),
                source_artifacts: Vec::new(),
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:synthetic-test".into(),
        }
    }

    #[test]
    fn exact_full_capacity_is_adequate() {
        let result = assess_service_function(
            &scenario(
                measurement("asset:pump", 10.0, "m3/h", None),
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.state, ServiceFunctionState::Adequate);
        assert_eq!(result.coverage_ratio, Some(1.0));
        assert_eq!(result.coverage_lower_bound, Some(1.0));
        assert_eq!(result.coverage_upper_bound, Some(1.0));
    }

    #[test]
    fn exact_partial_capacity_is_degraded_not_binary_unavailable() {
        let result = assess_service_function(
            &scenario(
                measurement("asset:pump", 7.0, "m3/h", None),
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.state, ServiceFunctionState::Degraded);
        assert_eq!(result.coverage_ratio, Some(0.7));
        assert_eq!(result.available_capacity_value, 7.0);
    }

    #[test]
    fn zero_capacity_is_unavailable() {
        let result = assess_service_function(
            &scenario(
                measurement("asset:pump", 0.0, "m3/h", None),
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.state, ServiceFunctionState::Unavailable);
        assert_eq!(result.coverage_ratio, Some(0.0));
    }

    #[test]
    fn overlapping_capacity_and_demand_uncertainty_is_indeterminate() {
        let result = assess_service_function(
            &scenario(
                measurement("asset:pump", 9.5, "m3/h", Some(1.0)),
                measurement("service:irrigation", 10.0, "m3/h", Some(0.5)),
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.state, ServiceFunctionState::Indeterminate);
        assert!(result.coverage_lower_bound.unwrap() < 1.0);
        assert!(result.coverage_upper_bound.unwrap() > 1.0);
    }

    #[test]
    fn robust_capacity_interval_must_cover_requirement_upper_bound() {
        let result = assess_service_function(
            &scenario(
                measurement("asset:pump", 12.0, "m3/h", Some(1.0)),
                measurement("service:irrigation", 10.0, "m3/h", Some(1.0)),
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.state, ServiceFunctionState::Adequate);
        assert_eq!(result.available_capacity_lower_bound, 11.0);
        assert_eq!(result.required_capacity_upper_bound, 11.0);
    }

    #[test]
    fn derived_bounds_that_overflow_are_rejected() {
        let result = assess_service_function(
            &scenario(
                measurement("asset:pump", f64::MAX, "m3/h", Some(f64::MAX)),
                measurement("service:irrigation", 1.0, "m3/h", None),
            ),
            &[],
        );
        assert!(matches!(
            result,
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::NonFiniteDerivedValue)
        ));
    }

    #[test]
    fn mismatched_units_fail_closed_without_implicit_conversion() {
        let result = assess_service_function(
            &scenario(
                measurement("asset:pump", 10.0, "m3/h", None),
                measurement("service:irrigation", 10.0, "L/s", None),
            ),
            &[],
        );
        assert!(matches!(
            result,
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::UnitMismatch)
        ));
    }

    #[test]
    fn observed_measurements_require_matching_admissible_evidence() {
        let quantity = Quantity {
            value: 7.0,
            unit: "m3/h".into(),
            uncertainty_abs: Some(0.5),
        };
        let mut capacity = measurement("asset:pump", 7.0, "m3/h", Some(0.5));
        capacity.basis = ScenarioBasis::Observed;
        capacity.supporting_evidence_ids = vec!["ev:pump-capacity".into()];
        let result = assess_service_function(
            &scenario(
                capacity,
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[evidence(
                "ev:pump-capacity",
                "asset:pump",
                quantity,
                EvidenceState::Asserted,
            )],
        )
        .unwrap();
        assert_eq!(result.state, ServiceFunctionState::Degraded);
        assert_eq!(
            result.available_capacity_evidence_ids,
            vec!["ev:pump-capacity"]
        );
    }

    #[test]
    fn forecast_evidence_cannot_substitute_for_an_observed_capacity() {
        let mut capacity = measurement("asset:pump", 7.0, "m3/h", None);
        capacity.basis = ScenarioBasis::Observed;
        capacity.supporting_evidence_ids = vec!["ev:pump-capacity".into()];
        let mut forecast = evidence(
            "ev:pump-capacity",
            "asset:pump",
            Quantity {
                value: 7.0,
                unit: "m3/h".into(),
                uncertainty_abs: None,
            },
            EvidenceState::Asserted,
        );
        forecast.kind = EvidenceKind::Forecast;
        let result = assess_service_function(
            &scenario(
                capacity,
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[forecast],
        );
        assert!(matches!(
            result,
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::SupportingEvidenceWrongKind)
        ));
    }

    #[test]
    fn required_capacity_must_belong_to_the_target_service() {
        let invalid = ServiceFunctionScenario {
            schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
            id: "synthetic:wrong-demand-entity".into(),
            as_of: AT.into(),
            service_entity_id: "service:irrigation".into(),
            available_capacity: measurement("asset:pump", 7.0, "m3/h", None),
            required_capacity: measurement("service:another", 10.0, "m3/h", None),
        };
        assert!(matches!(
            assess_service_function(&invalid, &[]),
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::RequirementEntityMismatch)
        ));
    }

    #[test]
    fn wrong_quantity_or_subject_evidence_is_rejected() {
        let mut capacity = measurement("asset:pump", 7.0, "m3/h", None);
        capacity.basis = ScenarioBasis::Observed;
        capacity.supporting_evidence_ids = vec!["ev:pump-capacity".into()];
        let result = assess_service_function(
            &scenario(
                capacity,
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[evidence(
                "ev:pump-capacity",
                "asset:other-pump",
                Quantity {
                    value: 8.0,
                    unit: "m3/h".into(),
                    uncertainty_abs: None,
                },
                EvidenceState::Asserted,
            )],
        );
        assert!(matches!(
            result,
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::SupportingEvidenceSubjectMismatch)
                    && errors.contains(&ServiceFunctionValidationError::SupportingEvidenceQuantityMismatch)
        ));
    }

    #[test]
    fn expired_evidence_is_rejected() {
        let mut record = evidence(
            "ev:pump-capacity",
            "asset:pump",
            Quantity {
                value: 7.0,
                unit: "m3/h".into(),
                uncertainty_abs: None,
            },
            EvidenceState::Asserted,
        );
        record.time.valid_until = Some("2026-10-16T08:00:00Z".into());
        let mut capacity = measurement("asset:pump", 7.0, "m3/h", None);
        capacity.basis = ScenarioBasis::Observed;
        capacity.supporting_evidence_ids = vec!["ev:pump-capacity".into()];
        let result = assess_service_function(
            &scenario(
                capacity,
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[record],
        );
        assert!(matches!(
            result,
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::OutsideSupportingEvidenceValidity)
        ));
    }

    #[test]
    fn observed_evidence_recorded_after_as_of_is_rejected() {
        let mut record = evidence(
            "ev:pump-capacity",
            "asset:pump",
            Quantity {
                value: 7.0,
                unit: "m3/h".into(),
                uncertainty_abs: None,
            },
            EvidenceState::Asserted,
        );
        record.time.recorded_at = "2026-10-16T10:00:00Z".into();
        let mut capacity = measurement("asset:pump", 7.0, "m3/h", None);
        capacity.basis = ScenarioBasis::Observed;
        capacity.supporting_evidence_ids = vec!["ev:pump-capacity".into()];
        let result = assess_service_function(
            &scenario(
                capacity,
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[record],
        );
        assert!(matches!(
            result,
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::EvidenceRecordedAfterAsOf)
        ));
    }

    #[test]
    fn noncanonical_observation_time_is_rejected_for_as_of_assessment() {
        let mut record = evidence(
            "ev:pump-capacity",
            "asset:pump",
            Quantity {
                value: 7.0,
                unit: "m3/h".into(),
                uncertainty_abs: None,
            },
            EvidenceState::Asserted,
        );
        record.time.occurred_at = "2026-10-16T09:00:00+00:00".into();
        let mut capacity = measurement("asset:pump", 7.0, "m3/h", None);
        capacity.basis = ScenarioBasis::Observed;
        capacity.supporting_evidence_ids = vec!["ev:pump-capacity".into()];
        let result = assess_service_function(
            &scenario(
                capacity,
                measurement("service:irrigation", 10.0, "m3/h", None),
            ),
            &[record],
        );
        assert!(matches!(
            result,
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::InvalidSupportingEvidenceTimestamp)
        ));
    }

    #[test]
    fn zero_requirement_has_no_infinite_coverage_ratio() {
        let result = assess_service_function(
            &scenario(
                measurement("asset:pump", 10.0, "m3/h", None),
                measurement("service:irrigation", 0.0, "m3/h", None),
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.state, ServiceFunctionState::Adequate);
        assert_eq!(result.coverage_ratio, None);
        assert_eq!(result.coverage_upper_bound, None);
    }

    #[test]
    fn batch_assessment_indexes_evidence_once_and_sorts_by_scenario_id() {
        let mut first = scenario(
            measurement("asset:pump-a", 7.0, "m3/h", None),
            measurement("service:a", 10.0, "m3/h", None),
        );
        first.id = "scenario:b".into();
        first.service_entity_id = "service:a".into();
        let mut second = scenario(
            measurement("asset:pump-b", 12.0, "m3/h", None),
            measurement("service:b", 10.0, "m3/h", None),
        );
        second.id = "scenario:a".into();
        second.service_entity_id = "service:b".into();

        let batch = assess_service_functions(&[first.clone(), second.clone()], &[]).unwrap();
        assert_eq!(
            batch
                .iter()
                .map(|item| item.scenario_id.as_str())
                .collect::<Vec<_>>(),
            vec!["scenario:a", "scenario:b"]
        );
        assert_eq!(batch[0].state, ServiceFunctionState::Adequate);
        assert_eq!(batch[1].state, ServiceFunctionState::Degraded);

        let singleton_a = assess_service_function(&second, &[]).unwrap();
        let singleton_b = assess_service_function(&first, &[]).unwrap();
        assert_eq!(batch, vec![singleton_a, singleton_b]);
    }

    #[test]
    fn batch_assessment_rejects_empty_and_duplicate_scenario_sets() {
        assert!(matches!(
            assess_service_functions(&[], &[]),
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::EmptyScenarioSet)
        ));

        let one = scenario(
            measurement("asset:pump", 7.0, "m3/h", None),
            measurement("service:irrigation", 10.0, "m3/h", None),
        );
        assert!(matches!(
            assess_service_functions(&[one.clone(), one], &[]),
            Err(ServiceFunctionAssessmentError::InvalidInput(errors))
                if errors.contains(&ServiceFunctionValidationError::DuplicateScenarioId)
        ));
    }

    #[test]
    fn service_function_fixtures_match_golden_capacity_assessment() {
        let scenario: ServiceFunctionScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/service-function-scenario.json"
        ))
        .unwrap();
        let expected: ServiceFunctionAssessment = serde_json::from_str(include_str!(
            "../../schemas/examples/service-function-assessment.json"
        ))
        .unwrap();

        let actual = assess_service_function(&scenario, &[]).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.state, ServiceFunctionState::Degraded);
        assert!(actual.coverage_lower_bound.unwrap() < 1.0);
        assert!(actual.coverage_upper_bound.unwrap() < 1.0);
    }

    #[test]
    fn same_input_returns_identical_serialized_assessment() {
        let input = scenario(
            measurement("asset:pump", 7.0, "m3/h", Some(0.25)),
            measurement("service:irrigation", 10.0, "m3/h", Some(0.2)),
        );
        let first = assess_service_function(&input, &[]).unwrap();
        let second = assess_service_function(&input, &[]).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );
    }
}
