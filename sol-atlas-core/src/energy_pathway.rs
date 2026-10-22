// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-aware single-stream source-to-service energy accounting.
//!
//! This v0.1 reference evaluator deliberately does not simulate storage state,
//! heat-pump COP, grid power flow, dynamic thermal transients, or plant reliability.
//! It keeps stream energy, curtailed flow, conversion loss, and external auxiliary
//! electricity as separate quantities. A calculated result is not a measured plant
//! result; evidence metadata is carried through but not independently authenticated.

use std::collections::BTreeSet;

/// Energy form carried by the pathway. Values are all expressed in kWh of that form;
/// a form transition's efficiency and boundary must be supplied explicitly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyForm {
    Electricity,
    ThermalHeat,
    Mechanical,
    Chemical,
    Radiant,
}

/// Evidence class attached to a stage's declared efficiency and operating assumptions.
/// This enum is provenance metadata, not an automatic multiplier or quality score.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    AnalystAssumption,
    PublishedModel,
    ComponentTest,
    FieldPilot,
    OperatingPlant,
    ReplicatedFleet,
    IndependentlyAudited,
}

/// Temperatures for a thermal-to-electric heat-engine bound, in degrees Celsius.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThermalEngineLimits {
    pub hot_reservoir_c: f64,
    pub cold_reservoir_c: f64,
}

/// One declared conversion, distribution, or service-delivery stage.
///
/// `efficiency` means gross energy in `output_form` divided by accepted input
/// energy in `input_form`; it must be in [0, 1]. Heat pumps with COP > 1 require
/// a separate multi-stream model and are deliberately unsupported here.
/// `max_input_kwh` is a hard, explicit throughput bound over this scenario's
/// time window. `auxiliary_electricity_kwh_per_input_kwh` is external electrical
/// demand per unit of accepted stage input; it is tracked separately from the
/// primary stream to avoid silently mixing thermal and electrical quantities.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyPathwayStage {
    pub stage_id: String,
    pub input_form: EnergyForm,
    pub output_form: EnergyForm,
    pub efficiency: f64,
    pub max_input_kwh: f64,
    pub auxiliary_electricity_kwh_per_input_kwh: f64,
    pub evidence_class: EvidenceClass,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub evidence_as_of: Option<String>,
    #[serde(default)]
    pub thermal_engine_limits: Option<ThermalEngineLimits>,
}

/// A single-stream pathway. Storage is not a source or converter in this contract:
/// charge/discharge requires an explicit stateful model in a later version.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyPathwayRequest {
    pub schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub source_form: EnergyForm,
    pub source_energy_kwh: f64,
    pub stages: Vec<EnergyPathwayStage>,
}

/// Stage disposition after the explicit input capacity is enforced.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyPathwayStageResult {
    pub stage_id: String,
    pub input_form: EnergyForm,
    pub output_form: EnergyForm,
    pub available_input_kwh: f64,
    pub accepted_input_kwh: f64,
    pub curtailed_input_kwh: f64,
    pub gross_output_kwh: f64,
    pub conversion_loss_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    pub efficiency: f64,
    pub evidence_class: EvidenceClass,
    pub evidence_ref: Option<String>,
    pub evidence_as_of: Option<String>,
}

/// The evaluator's explicit refusal reasons. Errors return no partial assessment.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnergyPathwayError {
    EmptySchemaVersion,
    EmptyScenarioId,
    NoStages,
    EmptyStageId,
    DuplicateStageId { stage_id: String },
    InvalidSourceEnergy,
    InvalidEfficiency { stage_id: String },
    InvalidStageCapacity { stage_id: String },
    InvalidAuxiliaryRatio { stage_id: String },
    MissingEvidenceReference { stage_id: String },
    FormMismatch {
        stage_id: String,
        expected: EnergyForm,
        actual: EnergyForm,
    },
    MissingThermalEngineLimits { stage_id: String },
    InvalidThermalReservoirs { stage_id: String },
    CarnotBoundExceeded {
        stage_id: String,
        efficiency: f64,
        maximum_efficiency: f64,
    },
    NonFiniteDerivedValue { stage_id: String },
}

/// Summary of the declared resource-stream path and auxiliary electric demand.
///
/// `source_energy_kwh = conversion_loss_kwh + curtailed_energy_kwh
/// + gross_delivered_energy_kwh` (within floating-point tolerance) for the
/// single modelled stream. Auxiliary electricity is a separate external input.
/// When the final product is electricity, `net_electric_output_kwh` is gross
/// delivered electricity minus those auxiliary loads; it can be negative and
/// therefore signals net electric consumption under the declared boundary.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyPathwayAssessment {
    pub assessment_schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub source_form: EnergyForm,
    pub source_energy_kwh: f64,
    pub delivered_form: EnergyForm,
    pub gross_delivered_energy_kwh: f64,
    pub total_conversion_loss_kwh: f64,
    pub total_curtailed_energy_kwh: f64,
    pub total_auxiliary_electricity_kwh: f64,
    pub net_electric_output_kwh: Option<f64>,
    pub stream_balance_residual_kwh: f64,
    pub stages: Vec<EnergyPathwayStageResult>,
}

const NUMERIC_TOLERANCE: f64 = 1.0e-9;
const CELSIUS_TO_KELVIN: f64 = 273.15;

fn finite_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

/// Evaluate the declared source-to-service path.
///
/// The function never treats unknown capacity as infinite, never silently converts
/// an absent evidence reference into evidence, and never returns a truncated result
/// after a physical/accounting validation failure. This is an accounting oracle for
/// the declared scalar stream—not a grid simulator, plant-performance model, or
/// certification of source evidence.
pub fn assess_energy_pathway(
    request: &EnergyPathwayRequest,
) -> Result<EnergyPathwayAssessment, EnergyPathwayError> {
    if request.schema_version.trim().is_empty() {
        return Err(EnergyPathwayError::EmptySchemaVersion);
    }
    if request.scenario_id.trim().is_empty() {
        return Err(EnergyPathwayError::EmptyScenarioId);
    }
    if !finite_nonnegative(request.source_energy_kwh) {
        return Err(EnergyPathwayError::InvalidSourceEnergy);
    }
    if request.stages.is_empty() {
        return Err(EnergyPathwayError::NoStages);
    }

    let mut ids = BTreeSet::new();
    let mut expected_form = request.source_form;

    // Validate the complete chain before producing any output.
    for stage in &request.stages {
        if stage.stage_id.trim().is_empty() {
            return Err(EnergyPathwayError::EmptyStageId);
        }
        if !ids.insert(stage.stage_id.as_str()) {
            return Err(EnergyPathwayError::DuplicateStageId {
                stage_id: stage.stage_id.clone(),
            });
        }
        if stage.input_form != expected_form {
            return Err(EnergyPathwayError::FormMismatch {
                stage_id: stage.stage_id.clone(),
                expected: expected_form,
                actual: stage.input_form,
            });
        }
        if !stage.efficiency.is_finite() || !(0.0..=1.0).contains(&stage.efficiency) {
            return Err(EnergyPathwayError::InvalidEfficiency {
                stage_id: stage.stage_id.clone(),
            });
        }
        if !finite_nonnegative(stage.max_input_kwh) {
            return Err(EnergyPathwayError::InvalidStageCapacity {
                stage_id: stage.stage_id.clone(),
            });
        }
        if !finite_nonnegative(stage.auxiliary_electricity_kwh_per_input_kwh) {
            return Err(EnergyPathwayError::InvalidAuxiliaryRatio {
                stage_id: stage.stage_id.clone(),
            });
        }
        if stage.evidence_class != EvidenceClass::AnalystAssumption
            && stage
                .evidence_ref
                .as_deref()
                .is_none_or(|reference| reference.trim().is_empty())
        {
            return Err(EnergyPathwayError::MissingEvidenceReference {
                stage_id: stage.stage_id.clone(),
            });
        }
        if stage.input_form == EnergyForm::ThermalHeat
            && stage.output_form == EnergyForm::Electricity
        {
            let limits = stage
                .thermal_engine_limits
                .ok_or_else(|| EnergyPathwayError::MissingThermalEngineLimits {
                    stage_id: stage.stage_id.clone(),
                })?;
            let hot_kelvin = limits.hot_reservoir_c + CELSIUS_TO_KELVIN;
            let cold_kelvin = limits.cold_reservoir_c + CELSIUS_TO_KELVIN;
            if !limits.hot_reservoir_c.is_finite()
                || !limits.cold_reservoir_c.is_finite()
                || cold_kelvin <= 0.0
                || hot_kelvin <= cold_kelvin
            {
                return Err(EnergyPathwayError::InvalidThermalReservoirs {
                    stage_id: stage.stage_id.clone(),
                });
            }
            let carnot_max = 1.0 - cold_kelvin / hot_kelvin;
            if stage.efficiency > carnot_max + NUMERIC_TOLERANCE {
                return Err(EnergyPathwayError::CarnotBoundExceeded {
                    stage_id: stage.stage_id.clone(),
                    efficiency: stage.efficiency,
                    maximum_efficiency: carnot_max,
                });
            }
        }
        expected_form = stage.output_form;
    }

    let mut available_input = request.source_energy_kwh;
    let mut conversion_loss = 0.0_f64;
    let mut curtailed_energy = 0.0_f64;
    let mut auxiliary_electricity = 0.0_f64;
    let mut stage_results = Vec::with_capacity(request.stages.len());

    for stage in &request.stages {
        let accepted_input = available_input.min(stage.max_input_kwh);
        let curtailed_input = available_input - accepted_input;
        let gross_output = accepted_input * stage.efficiency;
        let stage_loss = accepted_input - gross_output;
        let stage_aux = accepted_input * stage.auxiliary_electricity_kwh_per_input_kwh;

        if !accepted_input.is_finite()
            || !curtailed_input.is_finite()
            || !gross_output.is_finite()
            || !stage_loss.is_finite()
            || !stage_aux.is_finite()
        {
            return Err(EnergyPathwayError::NonFiniteDerivedValue {
                stage_id: stage.stage_id.clone(),
            });
        }

        conversion_loss += stage_loss;
        curtailed_energy += curtailed_input;
        auxiliary_electricity += stage_aux;
        if !conversion_loss.is_finite()
            || !curtailed_energy.is_finite()
            || !auxiliary_electricity.is_finite()
        {
            return Err(EnergyPathwayError::NonFiniteDerivedValue {
                stage_id: stage.stage_id.clone(),
            });
        }

        stage_results.push(EnergyPathwayStageResult {
            stage_id: stage.stage_id.clone(),
            input_form: stage.input_form,
            output_form: stage.output_form,
            available_input_kwh: available_input,
            accepted_input_kwh: accepted_input,
            curtailed_input_kwh: curtailed_input,
            gross_output_kwh: gross_output,
            conversion_loss_kwh: stage_loss,
            auxiliary_electricity_kwh: stage_aux,
            efficiency: stage.efficiency,
            evidence_class: stage.evidence_class,
            evidence_ref: stage.evidence_ref.clone(),
            evidence_as_of: stage.evidence_as_of.clone(),
        });
        available_input = gross_output;
    }

    let delivered_form = expected_form;
    let gross_delivered = available_input;
    let balance_residual =
        request.source_energy_kwh - conversion_loss - curtailed_energy - gross_delivered;
    if !gross_delivered.is_finite() || !balance_residual.is_finite() {
        return Err(EnergyPathwayError::NonFiniteDerivedValue {
            stage_id: request.stages.last().expect("validated nonempty").stage_id.clone(),
        });
    }

    Ok(EnergyPathwayAssessment {
        assessment_schema_version: "1".to_owned(),
        scenario_id: request.scenario_id.clone(),
        synthetic: request.synthetic,
        claim_ceiling: "calculated scenario; no physical meter verification or source authentication"
            .to_owned(),
        source_form: request.source_form,
        source_energy_kwh: request.source_energy_kwh,
        delivered_form,
        gross_delivered_energy_kwh: gross_delivered,
        total_conversion_loss_kwh: conversion_loss,
        total_curtailed_energy_kwh: curtailed_energy,
        total_auxiliary_electricity_kwh: auxiliary_electricity,
        net_electric_output_kwh: (delivered_form == EnergyForm::Electricity)
            .then_some(gross_delivered - auxiliary_electricity),
        stream_balance_residual_kwh: balance_residual,
        stages: stage_results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(
        id: &str,
        input_form: EnergyForm,
        output_form: EnergyForm,
        efficiency: f64,
        max_input_kwh: f64,
        auxiliary_ratio: f64,
    ) -> EnergyPathwayStage {
        EnergyPathwayStage {
            stage_id: id.to_owned(),
            input_form,
            output_form,
            efficiency,
            max_input_kwh,
            auxiliary_electricity_kwh_per_input_kwh: auxiliary_ratio,
            evidence_class: EvidenceClass::AnalystAssumption,
            evidence_ref: None,
            evidence_as_of: None,
            thermal_engine_limits: None,
        }
    }

    fn request(source_energy_kwh: f64, stages: Vec<EnergyPathwayStage>) -> EnergyPathwayRequest {
        EnergyPathwayRequest {
            schema_version: "energy-pathway-v0.1".to_owned(),
            scenario_id: "unit-test".to_owned(),
            synthetic: true,
            source_form: stages[0].input_form,
            source_energy_kwh,
            stages,
        }
    }

    #[test]
    fn energy_and_auxiliary_accounting_are_separate() {
        let mut heat_engine = stage(
            "heat-engine",
            EnergyForm::ThermalHeat,
            EnergyForm::Electricity,
            0.4,
            100.0,
            0.03,
        );
        heat_engine.thermal_engine_limits = Some(ThermalEngineLimits {
            hot_reservoir_c: 500.0,
            cold_reservoir_c: 25.0,
        });
        let distribution = stage(
            "distribution",
            EnergyForm::Electricity,
            EnergyForm::Electricity,
            0.95,
            1_000.0,
            0.005,
        );
        let assessment =
            assess_energy_pathway(&request(100.0, vec![heat_engine, distribution])).unwrap();
        assert!((assessment.gross_delivered_energy_kwh - 38.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.total_conversion_loss_kwh - 62.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.total_auxiliary_electricity_kwh - 3.2).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.net_electric_output_kwh.unwrap() - 34.8).abs() < NUMERIC_TOLERANCE);
        assert!(assessment.stream_balance_residual_kwh.abs() < NUMERIC_TOLERANCE);
    }

    #[test]
    fn stage_capacity_curtailment_closes_balance() {
        let converter = stage(
            "converter",
            EnergyForm::Chemical,
            EnergyForm::Electricity,
            0.8,
            80.0,
            0.0,
        );
        let grid = stage(
            "distribution",
            EnergyForm::Electricity,
            EnergyForm::Electricity,
            0.95,
            50.0,
            0.0,
        );
        let assessment = assess_energy_pathway(&request(100.0, vec![converter, grid])).unwrap();
        assert!((assessment.gross_delivered_energy_kwh - 47.5).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.total_conversion_loss_kwh - 18.5).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.total_curtailed_energy_kwh - 34.0).abs() < NUMERIC_TOLERANCE);
        assert!(assessment.stream_balance_residual_kwh.abs() < NUMERIC_TOLERANCE);
    }

    #[test]
    fn zero_resource_cannot_create_output() {
        let stage = stage(
            "converter",
            EnergyForm::Chemical,
            EnergyForm::Electricity,
            0.9,
            100.0,
            0.0,
        );
        let assessment = assess_energy_pathway(&request(0.0, vec![stage])).unwrap();
        assert_eq!(assessment.gross_delivered_energy_kwh, 0.0);
        assert_eq!(assessment.total_conversion_loss_kwh, 0.0);
        assert_eq!(assessment.total_auxiliary_electricity_kwh, 0.0);
    }

    #[test]
    fn rejects_efficiency_above_one() {
        let stage = stage("bad", EnergyForm::Chemical, EnergyForm::Electricity, 1.01, 100.0, 0.0);
        assert!(matches!(
            assess_energy_pathway(&request(100.0, vec![stage])),
            Err(EnergyPathwayError::InvalidEfficiency { .. })
        ));
    }

    #[test]
    fn rejects_non_finite_efficiency() {
        let stage = stage(
            "bad",
            EnergyForm::Chemical,
            EnergyForm::Electricity,
            f64::NAN,
            100.0,
            0.0,
        );
        assert!(matches!(
            assess_energy_pathway(&request(100.0, vec![stage])),
            Err(EnergyPathwayError::InvalidEfficiency { .. })
        ));
    }

    #[test]
    fn rejects_stream_form_mismatch() {
        let first = stage("a", EnergyForm::Chemical, EnergyForm::Electricity, 0.8, 100.0, 0.0);
        let second = stage("b", EnergyForm::ThermalHeat, EnergyForm::Electricity, 0.4, 100.0, 0.0);
        assert!(matches!(
            assess_energy_pathway(&request(100.0, vec![first, second])),
            Err(EnergyPathwayError::FormMismatch { .. })
        ));
    }

    #[test]
    fn heat_engine_requires_reservoirs_and_carnot_bound() {
        let missing = stage(
            "engine",
            EnergyForm::ThermalHeat,
            EnergyForm::Electricity,
            0.4,
            100.0,
            0.0,
        );
        assert!(matches!(
            assess_energy_pathway(&request(100.0, vec![missing])),
            Err(EnergyPathwayError::MissingThermalEngineLimits { .. })
        ));

        let mut impossible = stage(
            "engine",
            EnergyForm::ThermalHeat,
            EnergyForm::Electricity,
            0.7,
            100.0,
            0.0,
        );
        impossible.thermal_engine_limits = Some(ThermalEngineLimits {
            hot_reservoir_c: 500.0,
            cold_reservoir_c: 25.0,
        });
        assert!(matches!(
            assess_energy_pathway(&request(100.0, vec![impossible])),
            Err(EnergyPathwayError::CarnotBoundExceeded { .. })
        ));
    }

    #[test]
    fn non_assumption_requires_evidence_reference() {
        let mut stage = stage(
            "converter",
            EnergyForm::Chemical,
            EnergyForm::Electricity,
            0.5,
            100.0,
            0.0,
        );
        stage.evidence_class = EvidenceClass::ComponentTest;
        assert!(matches!(
            assess_energy_pathway(&request(100.0, vec![stage])),
            Err(EnergyPathwayError::MissingEvidenceReference { .. })
        ));
    }

    #[test]
    fn duplicate_stage_ids_fail_closed() {
        let a = stage("same", EnergyForm::Chemical, EnergyForm::Electricity, 0.8, 100.0, 0.0);
        let b = stage("same", EnergyForm::Electricity, EnergyForm::Electricity, 0.9, 100.0, 0.0);
        assert!(matches!(
            assess_energy_pathway(&request(100.0, vec![a, b])),
            Err(EnergyPathwayError::DuplicateStageId { .. })
        ));
    }

    #[test]
    fn rejects_negative_auxiliary_energy_ratio() {
        let stage = stage(
            "converter",
            EnergyForm::Chemical,
            EnergyForm::Electricity,
            0.5,
            100.0,
            -0.1,
        );
        assert!(matches!(
            assess_energy_pathway(&request(100.0, vec![stage])),
            Err(EnergyPathwayError::InvalidAuxiliaryRatio { .. })
        ));
    }

    #[test]
    fn deterministic_serialization_on_repeat_run() {
        let stage = stage(
            "converter",
            EnergyForm::Chemical,
            EnergyForm::Electricity,
            0.5,
            100.0,
            0.02,
        );
        let request = request(100.0, vec![stage]);
        let a = serde_json::to_vec(&assess_energy_pathway(&request).unwrap()).unwrap();
        let b = serde_json::to_vec(&assess_energy_pathway(&request).unwrap()).unwrap();
        assert_eq!(a, b);
    }
}
