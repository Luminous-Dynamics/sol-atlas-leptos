// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Boundary-explicit thermal exergy for declared isothermal heat streams.
//!
//! This evaluator computes idealized work potential for heat supplied at one
//! declared temperature relative to an explicit reference environment:
//! B = Q * (1 - T0/T), with absolute temperatures in kelvin. It does not estimate
//! real device efficiency, model finite-temperature heat exchange, or replace
//! first-law energy conservation.

use crate::energy_pathway::EvidenceClass;

/// One isothermal heat stream and its explicit reference environment.
///
/// Temperatures are accepted in Celsius and converted to kelvin before calculation.
/// This v0.1 model only supports heat at or above the reference-environment temperature;
/// cold-stream/refrigeration exergy and non-isothermal profiles require separate contracts.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThermalExergyRequest {
    pub schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    /// Heat quantity crossing the declared boundary, in kWh.
    pub heat_quantity_kwh: f64,
    /// Assumed isothermal heat-stream temperature, in degrees Celsius.
    pub heat_temperature_c: f64,
    /// Explicit environment/reference temperature, in degrees Celsius.
    pub reference_environment_temperature_c: f64,
    pub evidence_class: EvidenceClass,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub evidence_as_of: Option<String>,
}

/// Validation errors fail closed: no partial or silently defaulted result.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ThermalExergyError {
    EmptySchemaVersion,
    UnsupportedSchemaVersion { schema_version: String },
    EmptyScenarioId,
    InvalidHeatQuantity,
    InvalidAbsoluteTemperature { field: String },
    HeatBelowReference {
        heat_temperature_kelvin: f64,
        reference_temperature_kelvin: f64,
    },
    MissingEvidenceReference,
    NonFiniteDerivedValue,
}

/// Energy quantity and idealized thermal exergy are kept as separate report fields.
///
/// thermal_exergy_kwh is a reversible-work upper bound for heat at the stated
/// boundary/environment, not delivered work. energy_minus_exergy_kwh is a quality
/// distinction, not an energy loss.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ThermalExergyAssessment {
    pub assessment_schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub heat_quantity_kwh: f64,
    pub heat_temperature_c: f64,
    pub reference_environment_temperature_c: f64,
    pub heat_temperature_kelvin: f64,
    pub reference_environment_temperature_kelvin: f64,
    /// The dimensionless factor 1 - T0/T; not a measured device efficiency.
    pub exergy_fraction: f64,
    /// Idealized thermal exergy B = Q * (1 - T0/T), in kWh.
    pub thermal_exergy_kwh: f64,
    /// Energy quantity minus exergy; not a dissipative or conversion loss.
    pub energy_minus_exergy_kwh: f64,
    pub evidence_class: EvidenceClass,
    pub evidence_ref: Option<String>,
    pub evidence_as_of: Option<String>,
}

const CELSIUS_TO_KELVIN: f64 = 273.15;

fn finite_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

/// Compute idealized thermal exergy for one declared isothermal heat stream.
///
/// The reference environment is a mandatory scenario input, not a hidden constant.
/// The evaluator allows T = T0 (zero exergy) and rejects T < T0 because refrigeration
/// and cold exergy are outside this contract. All absolute temperatures must be > 0 K.
pub fn assess_thermal_exergy(
    request: &ThermalExergyRequest,
) -> Result<ThermalExergyAssessment, ThermalExergyError> {
    if request.schema_version.trim().is_empty() {
        return Err(ThermalExergyError::EmptySchemaVersion);
    }
    if request.schema_version != "thermal-exergy-v0.1" {
        return Err(ThermalExergyError::UnsupportedSchemaVersion {
            schema_version: request.schema_version.clone(),
        });
    }
    if request.scenario_id.trim().is_empty() {
        return Err(ThermalExergyError::EmptyScenarioId);
    }
    if !finite_nonnegative(request.heat_quantity_kwh) {
        return Err(ThermalExergyError::InvalidHeatQuantity);
    }
    if !request.heat_temperature_c.is_finite() {
        return Err(ThermalExergyError::InvalidAbsoluteTemperature {
            field: "heat_temperature_c".to_owned(),
        });
    }
    if !request.reference_environment_temperature_c.is_finite() {
        return Err(ThermalExergyError::InvalidAbsoluteTemperature {
            field: "reference_environment_temperature_c".to_owned(),
        });
    }

    let heat_temperature_kelvin = request.heat_temperature_c + CELSIUS_TO_KELVIN;
    let reference_environment_temperature_kelvin =
        request.reference_environment_temperature_c + CELSIUS_TO_KELVIN;
    for (field, value) in [
        ("heat_temperature_c", heat_temperature_kelvin),
        (
            "reference_environment_temperature_c",
            reference_environment_temperature_kelvin,
        ),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(ThermalExergyError::InvalidAbsoluteTemperature {
                field: field.to_owned(),
            });
        }
    }
    if heat_temperature_kelvin < reference_environment_temperature_kelvin {
        return Err(ThermalExergyError::HeatBelowReference {
            heat_temperature_kelvin,
            reference_temperature_kelvin: reference_environment_temperature_kelvin,
        });
    }
    if request.evidence_class != EvidenceClass::AnalystAssumption
        && request
            .evidence_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
    {
        return Err(ThermalExergyError::MissingEvidenceReference);
    }

    let exergy_fraction =
        (1.0 - reference_environment_temperature_kelvin / heat_temperature_kelvin)
            .clamp(0.0, 1.0);
    let thermal_exergy_kwh = request.heat_quantity_kwh * exergy_fraction;
    let energy_minus_exergy_kwh = request.heat_quantity_kwh - thermal_exergy_kwh;
    for value in [
        exergy_fraction,
        thermal_exergy_kwh,
        energy_minus_exergy_kwh,
    ] {
        if !value.is_finite() {
            return Err(ThermalExergyError::NonFiniteDerivedValue);
        }
    }

    Ok(ThermalExergyAssessment {
        assessment_schema_version: "1".to_owned(),
        scenario_id: request.scenario_id.clone(),
        synthetic: request.synthetic,
        claim_ceiling: concat!(
            "idealized exergy of a declared isothermal heat stream at the supplied boundary; ",
            "not delivered work, device efficiency, metered plant performance, or economic value",
        )
        .to_owned(),
        heat_quantity_kwh: request.heat_quantity_kwh,
        heat_temperature_c: request.heat_temperature_c,
        reference_environment_temperature_c: request.reference_environment_temperature_c,
        heat_temperature_kelvin,
        reference_environment_temperature_kelvin,
        exergy_fraction,
        thermal_exergy_kwh,
        energy_minus_exergy_kwh,
        evidence_class: request.evidence_class,
        evidence_ref: request.evidence_ref.clone(),
        evidence_as_of: request.evidence_as_of.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(heat_quantity_kwh: f64, heat_temperature_c: f64) -> ThermalExergyRequest {
        ThermalExergyRequest {
            schema_version: "thermal-exergy-v0.1".to_owned(),
            scenario_id: "synthetic-test".to_owned(),
            synthetic: true,
            heat_quantity_kwh,
            heat_temperature_c,
            reference_environment_temperature_c: 20.0,
            evidence_class: EvidenceClass::AnalystAssumption,
            evidence_ref: None,
            evidence_as_of: None,
        }
    }

    #[test]
    fn hand_derived_fixture_preserves_energy_and_exergy_as_distinct_quantities() {
        // 40 kWh at 80 C against a 20 C environment: B = Q * 60 / 353.15 K.
        let assessment = assess_thermal_exergy(&request(40.0, 80.0)).unwrap();
        assert!((assessment.thermal_exergy_kwh - 6.795979045731277).abs() < 1.0e-12);
        assert!((assessment.exergy_fraction - (60.0 / 353.15)).abs() < 1.0e-14);
        assert!((assessment.energy_minus_exergy_kwh - 33.20402095426872).abs() < 1.0e-12);
        assert!(assessment.thermal_exergy_kwh <= assessment.heat_quantity_kwh);
        assert!(assessment.synthetic);
    }

    #[test]
    fn zero_heat_has_zero_exergy() {
        let assessment = assess_thermal_exergy(&request(0.0, 80.0)).unwrap();
        assert_eq!(assessment.thermal_exergy_kwh, 0.0);
        assert_eq!(assessment.energy_minus_exergy_kwh, 0.0);
    }

    #[test]
    fn ambient_temperature_heat_has_zero_exergy() {
        let assessment = assess_thermal_exergy(&request(100.0, 20.0)).unwrap();
        assert_eq!(assessment.exergy_fraction, 0.0);
        assert_eq!(assessment.thermal_exergy_kwh, 0.0);
    }

    #[test]
    fn higher_temperature_increases_exergy_at_fixed_heat_quantity() {
        let quantities = [40.0, 40.0, 40.0, 40.0, 40.0];
        let temperatures = [20.0, 30.0, 50.0, 80.0, 150.0];
        let mut previous = 0.0;
        for (quantity, temperature) in quantities.into_iter().zip(temperatures) {
            let assessment = assess_thermal_exergy(&request(quantity, temperature)).unwrap();
            assert!(assessment.exergy_fraction >= 0.0);
            assert!(assessment.exergy_fraction <= 1.0);
            assert!(assessment.thermal_exergy_kwh >= previous);
            assert!(assessment.thermal_exergy_kwh <= quantity);
            previous = assessment.thermal_exergy_kwh;
        }
    }

    #[test]
    fn rejects_heat_below_reference_instead_of_misclassifying_cold_exergy() {
        let error = assess_thermal_exergy(&request(2.0, 10.0)).unwrap_err();
        assert!(matches!(error, ThermalExergyError::HeatBelowReference { .. }));
    }

    #[test]
    fn rejects_nonpositive_absolute_temperature() {
        let error = assess_thermal_exergy(&request(2.0, -273.15)).unwrap_err();
        assert!(matches!(error, ThermalExergyError::InvalidAbsoluteTemperature { .. }));
    }

    #[test]
    fn non_assumption_evidence_requires_a_reference() {
        let mut input = request(1.0, 80.0);
        input.evidence_class = EvidenceClass::PublishedModel;
        assert_eq!(
            assess_thermal_exergy(&input),
            Err(ThermalExergyError::MissingEvidenceReference)
        );
        input.evidence_ref = Some("https://example.invalid/model".to_owned());
        assert!(assess_thermal_exergy(&input).is_ok());
    }

    #[test]
    fn rejects_nonfinite_and_negative_heat() {
        for amount in [-1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                assess_thermal_exergy(&request(amount, 80.0)),
                Err(ThermalExergyError::InvalidHeatQuantity)
            );
        }
    }

    #[test]
    fn equal_temperature_streams_share_zero_factor_and_no_hidden_reference() {
        let mut input = request(1.0, 50.0);
        input.reference_environment_temperature_c = 50.0;
        let assessment = assess_thermal_exergy(&input).unwrap();
        assert_eq!(assessment.thermal_exergy_kwh, 0.0);
        assert_eq!(assessment.reference_environment_temperature_c, 50.0);
    }
}
