// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Multi-stream heat-pump accounting with an explicit first-law balance.
//!
//! Unlike a single-stream converter, a heat pump moves heat from a lower-temperature
//! source and adds compressor work. This reference model keeps the source-heat stream,
//! compressor electricity, auxiliary electricity, distribution loss, and useful heat
//! separate. It is a steady-state energy-accounting model, not a refrigerant-cycle
//! simulator, transient thermal model, test certification, or performance predictor.

use crate::energy_pathway::EvidenceClass;

/// Temperatures at the heat pump's declared source and useful-delivery boundaries.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeatPumpTemperatures {
    /// Heat-source temperature at the evaporator boundary, in degrees Celsius.
    pub source_c: f64,
    /// Required heat-delivery temperature at the condenser boundary, in degrees Celsius.
    pub delivery_c: f64,
}

/// Available input energy and hard throughput limits for one declared interval.
///
/// All energy values are kWh over the same interval. Source heat and compressor
/// electricity are different input streams. Auxiliary electricity is accounted as
/// a separate external input and is not silently transformed into delivered heat.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeatPumpRequest {
    pub schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub temperatures: HeatPumpTemperatures,
    pub source_heat_available_kwh: f64,
    pub max_source_heat_kwh: f64,
    pub compressor_electricity_available_kwh: f64,
    pub max_compressor_electricity_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    /// Fraction of gross condenser heat lost between the heat-pump and service boundary.
    pub distribution_loss_fraction: f64,
    pub evidence_class: EvidenceClass,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub evidence_as_of: Option<String>,
}

/// Explicit failure reasons. Invalid cases return no partial numerical assessment.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HeatPumpError {
    EmptySchemaVersion,
    UnsupportedSchemaVersion { schema_version: String },
    EmptyScenarioId,
    InvalidEnergy { field: String },
    InvalidLossFraction,
    InvalidTemperatures,
    MissingEvidenceReference,
    NoCompressorWorkForHeatTransfer,
    NonFiniteDerivedValue,
    CarnotCopExceeded {
        observed_cop: f64,
        carnot_cop: f64,
    },
}

/// Steady-state assessment of one heat-pump interval.
///
/// First law at the heat-pump boundary:
///
/// `source_heat_used + compressor_electricity_used = gross_heat_output`.
///
/// At the useful-service boundary:
///
/// `source_heat_used + compressor_electricity_used = useful_heat + distribution_loss`.
///
/// Auxiliary electricity is reported separately. The compressor COP uses gross condenser
/// heat divided by compressor electricity. The system COP uses useful delivered heat divided
/// by compressor plus auxiliary electricity. Carnot compares the core COP with the ideal
/// heating-cycle bound for the declared source and delivery reservoir temperatures.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct HeatPumpAssessment {
    pub assessment_schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub temperatures: HeatPumpTemperatures,
    pub source_heat_available_kwh: f64,
    pub source_heat_used_kwh: f64,
    pub source_heat_curtailed_kwh: f64,
    pub compressor_electricity_available_kwh: f64,
    pub compressor_electricity_used_kwh: f64,
    pub compressor_electricity_curtailed_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    pub gross_heat_output_kwh: f64,
    pub distribution_loss_kwh: f64,
    pub useful_heat_delivered_kwh: f64,
    pub total_electricity_consumed_kwh: f64,
    pub compressor_cop: Option<f64>,
    pub system_cop: Option<f64>,
    pub carnot_cop_upper_bound: f64,
    pub first_law_balance_residual_kwh: f64,
    pub evidence_class: EvidenceClass,
    pub evidence_ref: Option<String>,
    pub evidence_as_of: Option<String>,
}

const NUMERIC_TOLERANCE: f64 = 1.0e-9;
const CELSIUS_TO_KELVIN: f64 = 273.15;

fn finite_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

/// Assess one bounded heat-pump interval.
///
/// This model rejects a claimed core COP above the Carnot heating bound. That bound is
/// necessary, not sufficient: a passing case is not evidence that a real device achieves
/// the supplied energy flows or that the COP is economically useful at this duty cycle.
pub fn assess_heat_pump(
    request: &HeatPumpRequest,
) -> Result<HeatPumpAssessment, HeatPumpError> {
    if request.schema_version.trim().is_empty() {
        return Err(HeatPumpError::EmptySchemaVersion);
    }
    if request.schema_version != "heat-pump-v0.1" {
        return Err(HeatPumpError::UnsupportedSchemaVersion {
            schema_version: request.schema_version.clone(),
        });
    }
    if request.scenario_id.trim().is_empty() {
        return Err(HeatPumpError::EmptyScenarioId);
    }

    for (field, value) in [
        ("source_heat_available_kwh", request.source_heat_available_kwh),
        ("max_source_heat_kwh", request.max_source_heat_kwh),
        (
            "compressor_electricity_available_kwh",
            request.compressor_electricity_available_kwh,
        ),
        (
            "max_compressor_electricity_kwh",
            request.max_compressor_electricity_kwh,
        ),
        ("auxiliary_electricity_kwh", request.auxiliary_electricity_kwh),
    ] {
        if !finite_nonnegative(value) {
            return Err(HeatPumpError::InvalidEnergy {
                field: field.to_owned(),
            });
        }
    }

    if !request.distribution_loss_fraction.is_finite()
        || !(0.0..=1.0).contains(&request.distribution_loss_fraction)
    {
        return Err(HeatPumpError::InvalidLossFraction);
    }
    let source_kelvin = request.temperatures.source_c + CELSIUS_TO_KELVIN;
    let delivery_kelvin = request.temperatures.delivery_c + CELSIUS_TO_KELVIN;
    if !request.temperatures.source_c.is_finite()
        || !request.temperatures.delivery_c.is_finite()
        || source_kelvin <= 0.0
        || delivery_kelvin <= source_kelvin
    {
        return Err(HeatPumpError::InvalidTemperatures);
    }
    if request.evidence_class != EvidenceClass::AnalystAssumption
        && request
            .evidence_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
    {
        return Err(HeatPumpError::MissingEvidenceReference);
    }

    let source_heat_used = request
        .source_heat_available_kwh
        .min(request.max_source_heat_kwh);
    let source_heat_curtailed = request.source_heat_available_kwh - source_heat_used;
    let compressor_electricity_used = request
        .compressor_electricity_available_kwh
        .min(request.max_compressor_electricity_kwh);
    let compressor_electricity_curtailed =
        request.compressor_electricity_available_kwh - compressor_electricity_used;

    if source_heat_used > 0.0 && compressor_electricity_used <= 0.0 {
        return Err(HeatPumpError::NoCompressorWorkForHeatTransfer);
    }

    let gross_heat_output = source_heat_used + compressor_electricity_used;
    let distribution_loss = gross_heat_output * request.distribution_loss_fraction;
    let useful_heat_delivered = gross_heat_output - distribution_loss;
    let total_electricity_consumed =
        compressor_electricity_used + request.auxiliary_electricity_kwh;
    let compressor_cop = if compressor_electricity_used > 0.0 {
        Some(gross_heat_output / compressor_electricity_used)
    } else {
        None
    };
    let system_cop = if total_electricity_consumed > 0.0 {
        Some(useful_heat_delivered / total_electricity_consumed)
    } else {
        None
    };
    let carnot_cop_upper_bound = delivery_kelvin / (delivery_kelvin - source_kelvin);
    let first_law_balance_residual =
        source_heat_used + compressor_electricity_used - distribution_loss - useful_heat_delivered;

    for value in [
        gross_heat_output,
        distribution_loss,
        useful_heat_delivered,
        total_electricity_consumed,
        carnot_cop_upper_bound,
        first_law_balance_residual,
    ] {
        if !value.is_finite() {
            return Err(HeatPumpError::NonFiniteDerivedValue);
        }
    }
    if let Some(observed_cop) = compressor_cop {
        if !observed_cop.is_finite() {
            return Err(HeatPumpError::NonFiniteDerivedValue);
        }
        if observed_cop > carnot_cop_upper_bound + NUMERIC_TOLERANCE {
            return Err(HeatPumpError::CarnotCopExceeded {
                observed_cop,
                carnot_cop: carnot_cop_upper_bound,
            });
        }
    }
    if let Some(system_cop) = system_cop {
        if !system_cop.is_finite() {
            return Err(HeatPumpError::NonFiniteDerivedValue);
        }
    }

    Ok(HeatPumpAssessment {
        assessment_schema_version: "1".to_owned(),
        scenario_id: request.scenario_id.clone(),
        synthetic: request.synthetic,
        claim_ceiling: concat!(
            "calculated steady-state scenario; no physical meter verification ",
            "or source authentication",
        )
        .to_owned(),
        temperatures: request.temperatures,
        source_heat_available_kwh: request.source_heat_available_kwh,
        source_heat_used_kwh: source_heat_used,
        source_heat_curtailed_kwh: source_heat_curtailed,
        compressor_electricity_available_kwh: request.compressor_electricity_available_kwh,
        compressor_electricity_used_kwh: compressor_electricity_used,
        compressor_electricity_curtailed_kwh: compressor_electricity_curtailed,
        auxiliary_electricity_kwh: request.auxiliary_electricity_kwh,
        gross_heat_output_kwh: gross_heat_output,
        distribution_loss_kwh: distribution_loss,
        useful_heat_delivered_kwh: useful_heat_delivered,
        total_electricity_consumed_kwh: total_electricity_consumed,
        compressor_cop,
        system_cop,
        carnot_cop_upper_bound,
        first_law_balance_residual_kwh: first_law_balance_residual,
        evidence_class: request.evidence_class,
        evidence_ref: request.evidence_ref.clone(),
        evidence_as_of: request.evidence_as_of.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heat_pump_request() -> HeatPumpRequest {
        HeatPumpRequest {
            schema_version: "heat-pump-v0.1".to_owned(),
            scenario_id: "synthetic-heat-pump-known-answer".to_owned(),
            synthetic: true,
            temperatures: HeatPumpTemperatures {
                source_c: 20.0,
                delivery_c: 80.0,
            },
            source_heat_available_kwh: 30.0,
            max_source_heat_kwh: 30.0,
            compressor_electricity_available_kwh: 10.0,
            max_compressor_electricity_kwh: 10.0,
            auxiliary_electricity_kwh: 1.0,
            distribution_loss_fraction: 0.1,
            evidence_class: EvidenceClass::AnalystAssumption,
            evidence_ref: None,
            evidence_as_of: None,
        }
    }

    #[test]
    fn multi_stream_first_law_and_system_cop_close() {
        let assessment = assess_heat_pump(&heat_pump_request()).unwrap();
        assert!((assessment.gross_heat_output_kwh - 40.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.distribution_loss_kwh - 4.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.useful_heat_delivered_kwh - 36.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.compressor_cop.unwrap() - 4.0).abs() < NUMERIC_TOLERANCE);
        assert!((assessment.system_cop.unwrap() - (36.0 / 11.0)).abs() < NUMERIC_TOLERANCE);
        assert!(assessment.first_law_balance_residual_kwh.abs() < NUMERIC_TOLERANCE);
        assert!(assessment.compressor_cop.unwrap() < assessment.carnot_cop_upper_bound);
    }

    #[test]
    fn throughput_caps_curtail_both_streams_explicitly() {
        let mut request = heat_pump_request();
        request.max_source_heat_kwh = 20.0;
        request.max_compressor_electricity_kwh = 5.0;
        let assessment = assess_heat_pump(&request).unwrap();
        assert_eq!(assessment.source_heat_used_kwh, 20.0);
        assert_eq!(assessment.source_heat_curtailed_kwh, 10.0);
        assert_eq!(assessment.compressor_electricity_used_kwh, 5.0);
        assert_eq!(assessment.compressor_electricity_curtailed_kwh, 5.0);
        assert!((assessment.useful_heat_delivered_kwh - 22.5).abs() < NUMERIC_TOLERANCE);
        assert!(assessment.first_law_balance_residual_kwh.abs() < NUMERIC_TOLERANCE);
    }

    #[test]
    fn zero_inputs_cannot_create_heat() {
        let mut request = heat_pump_request();
        request.source_heat_available_kwh = 0.0;
        request.max_source_heat_kwh = 0.0;
        request.compressor_electricity_available_kwh = 0.0;
        request.max_compressor_electricity_kwh = 0.0;
        request.auxiliary_electricity_kwh = 0.0;
        let assessment = assess_heat_pump(&request).unwrap();
        assert_eq!(assessment.gross_heat_output_kwh, 0.0);
        assert_eq!(assessment.useful_heat_delivered_kwh, 0.0);
        assert_eq!(assessment.compressor_cop, None);
        assert_eq!(assessment.system_cop, None);
    }

    #[test]
    fn source_heat_without_compressor_work_is_rejected() {
        let mut request = heat_pump_request();
        request.compressor_electricity_available_kwh = 0.0;
        request.max_compressor_electricity_kwh = 0.0;
        assert!(matches!(
            assess_heat_pump(&request),
            Err(HeatPumpError::NoCompressorWorkForHeatTransfer)
        ));
    }

    #[test]
    fn impossible_cop_fails_carnot_negative_control() {
        let mut request = heat_pump_request();
        request.source_heat_available_kwh = 100.0;
        request.max_source_heat_kwh = 100.0;
        request.compressor_electricity_available_kwh = 1.0;
        request.max_compressor_electricity_kwh = 1.0;
        assert!(matches!(
            assess_heat_pump(&request),
            Err(HeatPumpError::CarnotCopExceeded { .. })
        ));
    }

    #[test]
    fn invalid_temperatures_fail_closed() {
        let mut request = heat_pump_request();
        request.temperatures.source_c = 90.0;
        request.temperatures.delivery_c = 80.0;
        assert!(matches!(
            assess_heat_pump(&request),
            Err(HeatPumpError::InvalidTemperatures)
        ));
    }

    #[test]
    fn missing_evidence_reference_fails_for_non_assumption_class() {
        let mut request = heat_pump_request();
        request.evidence_class = EvidenceClass::OperatingPlant;
        assert!(matches!(
            assess_heat_pump(&request),
            Err(HeatPumpError::MissingEvidenceReference)
        ));
    }

    #[test]
    fn invalid_numeric_values_fail_closed() {
        let mut request = heat_pump_request();
        request.auxiliary_electricity_kwh = f64::NAN;
        assert!(matches!(
            assess_heat_pump(&request),
            Err(HeatPumpError::InvalidEnergy { .. })
        ));
        let mut request = heat_pump_request();
        request.distribution_loss_fraction = 1.01;
        assert!(matches!(
            assess_heat_pump(&request),
            Err(HeatPumpError::InvalidLossFraction)
        ));
    }

    #[test]
    fn unsupported_version_fails_closed() {
        let mut request = heat_pump_request();
        request.schema_version = "future-v9".to_owned();
        assert!(matches!(
            assess_heat_pump(&request),
            Err(HeatPumpError::UnsupportedSchemaVersion { .. })
        ));
    }

    #[test]
    fn output_is_deterministic() {
        let a = serde_json::to_vec(&assess_heat_pump(&heat_pump_request()).unwrap()).unwrap();
        let b = serde_json::to_vec(&assess_heat_pump(&heat_pump_request()).unwrap()).unwrap();
        assert_eq!(a, b);
    }
}
