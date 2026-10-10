// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Fail-closed, elemental nutrient accounting for regenerative agriculture planning.
//!
//! This module is deliberately an accounting primitive, not an agronomic recommender.
//! Amounts are plant-available (or seasonally available) elemental nutrients in kg/ha
//! for one explicitly defined planning period. Convert fertilizer-label P₂O₅ and K₂O
//! values before constructing a profile. Unknown values stay unknown; they are never
//! silently treated as zero. A complete balance is arithmetic completeness, not proof
//! that the inputs are accurate or that an application rate is agronomically safe.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

const P_PER_P2O5: f64 = 0.4364;
const K_PER_K2O: f64 = 0.8301;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    /// Directly measured or documented (for example, a dated laboratory result).
    Measured,
    /// An estimate whose method and source are recorded.
    Estimated,
    /// A deliberately hypothetical input used for scenario analysis.
    Scenario,
    /// No defensible numeric value is available.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BalanceStatus {
    /// Demand and all three supply components have numeric values.
    /// This does not mean the source evidence is measured or independently validated.
    Complete,
    /// At least one required value is unknown, so a deficit/surplus is not reported.
    Incomplete,
}

/// One nutrient amount and its provenance.
///
/// For known values, `source_id` and `method` are mandatory so a value cannot be
/// detached from its evidence. For `Unknown`, the amount, source, and method must be
/// absent. Use a documented, explicit zero when the evidence establishes zero supply.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NutrientMeasurement {
    /// kg elemental nutrient / ha over the budget's declared planning period.
    pub kg_per_ha: Option<f64>,
    pub evidence: EvidenceClass,
    pub source_id: Option<String>,
    pub method: Option<String>,
}

impl NutrientMeasurement {
    /// Construct a known measurement, estimate, or scenario value.
    pub fn known(
        kg_per_ha: f64,
        evidence: EvidenceClass,
        source_id: impl Into<String>,
        method: impl Into<String>,
    ) -> Result<Self, NutrientModelError> {
        let measurement = Self {
            kg_per_ha: Some(kg_per_ha),
            evidence,
            source_id: Some(source_id.into()),
            method: Some(method.into()),
        };
        measurement.validate("measurement")?;
        Ok(measurement)
    }

    /// Represent a real data gap. This value will block a complete balance.
    pub fn unknown() -> Self {
        Self {
            kg_per_ha: None,
            evidence: EvidenceClass::Unknown,
            source_id: None,
            method: None,
        }
    }

    fn validate(&self, path: &str) -> Result<(), NutrientModelError> {
        if let Some(value) = self.kg_per_ha {
            if !value.is_finite() {
                return Err(NutrientModelError::new(path, "amount must be finite"));
            }
            if value < 0.0 {
                return Err(NutrientModelError::new(path, "amount cannot be negative"));
            }
        }

        match self.evidence {
            EvidenceClass::Unknown => {
                if self.kg_per_ha.is_some() {
                    return Err(NutrientModelError::new(
                        path,
                        "unknown evidence must not contain a numeric amount",
                    ));
                }
                if self.source_id.is_some() || self.method.is_some() {
                    return Err(NutrientModelError::new(
                        path,
                        "unknown evidence must not claim a source or method",
                    ));
                }
            }
            EvidenceClass::Measured | EvidenceClass::Estimated | EvidenceClass::Scenario => {
                if self.kg_per_ha.is_none() {
                    return Err(NutrientModelError::new(
                        path,
                        "known evidence must contain a numeric amount",
                    ));
                }
                if self
                    .source_id
                    .as_deref()
                    .map_or(true, |value| value.trim().is_empty())
                {
                    return Err(NutrientModelError::new(
                        path,
                        "known evidence requires a non-empty source_id",
                    ));
                }
                if self
                    .method
                    .as_deref()
                    .map_or(true, |value| value.trim().is_empty())
                {
                    return Err(NutrientModelError::new(
                        path,
                        "known evidence requires a non-empty method",
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Nitrogen, phosphorus, and potassium values on one shared unit/time basis.
///
/// Phosphorus and potassium are represented as elemental P and K, not P₂O₅ or K₂O.
/// Each nutrient carries independent provenance and may independently be unknown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NutrientProfile {
    pub nitrogen: NutrientMeasurement,
    pub phosphorus: NutrientMeasurement,
    pub potassium: NutrientMeasurement,
}

impl NutrientProfile {
    pub fn new(
        nitrogen: NutrientMeasurement,
        phosphorus: NutrientMeasurement,
        potassium: NutrientMeasurement,
    ) -> Self {
        Self {
            nitrogen,
            phosphorus,
            potassium,
        }
    }

    /// An explicit profile of data gaps. This does not mean zero nutrient supply.
    pub fn unknown() -> Self {
        Self::new(
            NutrientMeasurement::unknown(),
            NutrientMeasurement::unknown(),
            NutrientMeasurement::unknown(),
        )
    }

    fn validate(&self, path: &str) -> Result<(), NutrientModelError> {
        self.nitrogen.validate(&format!("{path}.nitrogen"))?;
        self.phosphorus.validate(&format!("{path}.phosphorus"))?;
        self.potassium.validate(&format!("{path}.potassium"))?;
        Ok(())
    }
}

/// Identity and area for one crop-season budget. Stable identifiers and an explicit
/// period prevent values from unrelated fields, crops, or seasons being combined.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetContext {
    pub site_id: String,
    pub crop_id: String,
    pub period_id: String,
    pub area_hectares: f64,
}

impl BudgetContext {
    fn validate(&self) -> Result<(), NutrientModelError> {
        for (path, value) in [
            ("context.site_id", self.site_id.as_str()),
            ("context.crop_id", self.crop_id.as_str()),
            ("context.period_id", self.period_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(NutrientModelError::new(path, "identifier cannot be empty"));
            }
        }
        if !self.area_hectares.is_finite() || self.area_hectares <= 0.0 {
            return Err(NutrientModelError::new(
                "context.area_hectares",
                "area must be finite and greater than zero",
            ));
        }
        Ok(())
    }
}

/// One per-hectare seasonal accounting exercise.
///
/// Supply values must be the estimated amount available during the same period as the
/// crop demand, not simply total nutrient content. Inputs must use the same soil/field,
/// area, crop-season, and elemental nutrient basis. This model does not estimate
/// mineralization, fixation, losses, crop removal, or the safety of any amendment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NutrientBudgetInput {
    pub context: BudgetContext,
    pub crop_demand: NutrientProfile,
    pub existing_soil_supply: NutrientProfile,
    pub mineral_fertilizer_supply: NutrientProfile,
    pub organic_amendment_supply: NutrientProfile,
}

impl NutrientBudgetInput {
    fn validate(&self) -> Result<(), NutrientModelError> {
        self.context.validate()?;
        self.crop_demand.validate("crop_demand")?;
        self.existing_soil_supply.validate("existing_soil_supply")?;
        self.mineral_fertilizer_supply
            .validate("mineral_fertilizer_supply")?;
        self.organic_amendment_supply
            .validate("organic_amendment_supply")?;
        Ok(())
    }
}

/// One nutrient's result. Deficit and surplus are reported only when every required
/// value is known; they are not fertilizer application-rate recommendations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NutrientBalance {
    pub status: BalanceStatus,
    pub crop_demand_kg_per_ha: Option<f64>,
    pub soil_supply_kg_per_ha: Option<f64>,
    pub mineral_fertilizer_supply_kg_per_ha: Option<f64>,
    pub organic_amendment_supply_kg_per_ha: Option<f64>,
    /// Sum of the available, known supply entries. This is a partial subtotal if the
    /// status is Incomplete, and must not be interpreted as total available supply.
    pub known_supply_subtotal_kg_per_ha: Option<f64>,
    /// max(demand - all supplies, 0), only when status is Complete.
    pub deficit_kg_per_ha: Option<f64>,
    /// max(all supplies - demand, 0), only when status is Complete.
    pub surplus_kg_per_ha: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NutrientBudget {
    /// Preserve input values and their evidence so persisted results remain auditable.
    pub inputs: NutrientBudgetInput,
    pub nitrogen: NutrientBalance,
    pub phosphorus: NutrientBalance,
    pub potassium: NutrientBalance,
}

/// Invalid or incomplete model input. Errors fail closed instead of silently coercing
/// negative, non-finite, or unprovenanced amounts into apparently valid outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NutrientModelError {
    pub path: String,
    pub reason: &'static str,
}

impl NutrientModelError {
    fn new(path: impl Into<String>, reason: &'static str) -> Self {
        Self {
            path: path.into(),
            reason,
        }
    }
}

impl fmt::Display for NutrientModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.reason)
    }
}

impl Error for NutrientModelError {}

/// Convert fertilizer-label phosphorus oxide (P₂O₅) to elemental phosphorus (P).
///
/// Uses the standard mass fraction 0.4364. Input and output units are kg/ha.
pub fn elemental_p_from_p2o5(p2o5_kg_per_ha: f64) -> Result<f64, NutrientModelError> {
    convert_oxide(p2o5_kg_per_ha, P_PER_P2O5, "p2o5_kg_per_ha")
}

/// Convert fertilizer-label potassium oxide (K₂O) to elemental potassium (K).
///
/// Uses the standard mass fraction 0.8301. Input and output units are kg/ha.
pub fn elemental_k_from_k2o(k2o_kg_per_ha: f64) -> Result<f64, NutrientModelError> {
    convert_oxide(k2o_kg_per_ha, K_PER_K2O, "k2o_kg_per_ha")
}

fn convert_oxide(value: f64, factor: f64, path: &str) -> Result<f64, NutrientModelError> {
    if !value.is_finite() {
        return Err(NutrientModelError::new(path, "amount must be finite"));
    }
    if value < 0.0 {
        return Err(NutrientModelError::new(path, "amount cannot be negative"));
    }
    Ok(value * factor)
}

/// Calculate auditable nutrient accounting from explicit inputs.
///
/// If any required value for a nutrient is unknown, its deficit and surplus remain
/// `None`. This function intentionally does not infer a safe or optimal application
/// rate, and it does not substitute for soil tests, agronomist review, or field trials.
pub fn calculate_budget(input: &NutrientBudgetInput) -> Result<NutrientBudget, NutrientModelError> {
    input.validate()?;

    Ok(NutrientBudget {
        inputs: input.clone(),
        nitrogen: calculate_one(
            "nitrogen",
            &input.crop_demand.nitrogen,
            &input.existing_soil_supply.nitrogen,
            &input.mineral_fertilizer_supply.nitrogen,
            &input.organic_amendment_supply.nitrogen,
        )?,
        phosphorus: calculate_one(
            "phosphorus",
            &input.crop_demand.phosphorus,
            &input.existing_soil_supply.phosphorus,
            &input.mineral_fertilizer_supply.phosphorus,
            &input.organic_amendment_supply.phosphorus,
        )?,
        potassium: calculate_one(
            "potassium",
            &input.crop_demand.potassium,
            &input.existing_soil_supply.potassium,
            &input.mineral_fertilizer_supply.potassium,
            &input.organic_amendment_supply.potassium,
        )?,
    })
}

fn calculate_one(
    path: &str,
    demand: &NutrientMeasurement,
    soil: &NutrientMeasurement,
    mineral: &NutrientMeasurement,
    organic: &NutrientMeasurement,
) -> Result<NutrientBalance, NutrientModelError> {
    let demand_value = demand.kg_per_ha;
    let soil_value = soil.kg_per_ha;
    let mineral_value = mineral.kg_per_ha;
    let organic_value = organic.kg_per_ha;

    let known_supply_values = [soil_value, mineral_value, organic_value];
    let any_supply_known = known_supply_values.iter().any(Option::is_some);
    let partial_total = known_supply_values
        .iter()
        .flatten()
        .try_fold(0.0, |sum, value| {
            let next = sum + *value;
            next.is_finite().then_some(next)
        })
        .ok_or_else(|| NutrientModelError::new(path, "supply subtotal overflow"))?;

    // Only the all-known branch reports a deficit or surplus. Unknown inputs are
    // excluded from the partial subtotal above, never substituted with numeric zero.
    let (deficit, surplus, status) = match (
        demand_value,
        soil_value,
        mineral_value,
        organic_value,
    ) {
        (Some(demand), Some(_), Some(_), Some(_)) => (
            Some((demand - partial_total).max(0.0)),
            Some((partial_total - demand).max(0.0)),
            BalanceStatus::Complete,
        ),
        _ => (None, None, BalanceStatus::Incomplete),
    };

    Ok(NutrientBalance {
        status,
        crop_demand_kg_per_ha: demand_value,
        soil_supply_kg_per_ha: soil_value,
        mineral_fertilizer_supply_kg_per_ha: mineral_value,
        organic_amendment_supply_kg_per_ha: organic_value,
        known_supply_subtotal_kg_per_ha: any_supply_known.then_some(partial_total),
        deficit_kg_per_ha: deficit,
        surplus_kg_per_ha: surplus,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(value: f64) -> NutrientMeasurement {
        NutrientMeasurement::known(
            value,
            EvidenceClass::Measured,
            "lab-or-ledger-001",
            "documented test input",
        )
        .unwrap()
    }

    fn profile(n: f64, p: f64, k: f64) -> NutrientProfile {
        NutrientProfile::new(known(n), known(p), known(k))
    }

    fn complete_input() -> NutrientBudgetInput {
        NutrientBudgetInput {
            context: BudgetContext {
                site_id: "field-north-001".into(),
                crop_id: "maize".into(),
                period_id: "2026-main-season".into(),
                area_hectares: 2.5,
            },
            crop_demand: profile(120.0, 30.0, 80.0),
            existing_soil_supply: profile(30.0, 5.0, 20.0),
            mineral_fertilizer_supply: profile(60.0, 10.0, 25.0),
            organic_amendment_supply: profile(20.0, 20.0, 35.0),
        }
    }

    #[test]
    fn complete_budget_reports_independent_deficits_and_surpluses() {
        let result = calculate_budget(&complete_input()).unwrap();

        assert_eq!(result.nitrogen.status, BalanceStatus::Complete);
        assert_eq!(result.nitrogen.known_supply_subtotal_kg_per_ha, Some(110.0));
        assert_eq!(result.nitrogen.deficit_kg_per_ha, Some(10.0));
        assert_eq!(result.nitrogen.surplus_kg_per_ha, Some(0.0));

        assert_eq!(result.phosphorus.known_supply_subtotal_kg_per_ha, Some(35.0));
        assert_eq!(result.phosphorus.deficit_kg_per_ha, Some(0.0));
        assert_eq!(result.phosphorus.surplus_kg_per_ha, Some(5.0));

        assert_eq!(result.potassium.known_supply_subtotal_kg_per_ha, Some(80.0));
        assert_eq!(result.potassium.deficit_kg_per_ha, Some(0.0));
        assert_eq!(result.potassium.surplus_kg_per_ha, Some(0.0));
    }

    #[test]
    fn unknown_supply_never_becomes_zero_or_a_false_deficit() {
        let mut input = complete_input();
        input.organic_amendment_supply.nitrogen = NutrientMeasurement::unknown();

        let result = calculate_budget(&input).unwrap();
        assert_eq!(result.nitrogen.status, BalanceStatus::Incomplete);
        assert_eq!(result.nitrogen.known_supply_subtotal_kg_per_ha, Some(90.0));
        assert_eq!(result.nitrogen.deficit_kg_per_ha, None);
        assert_eq!(result.nitrogen.surplus_kg_per_ha, None);

        // A gap in nitrogen evidence does not erase a complete phosphorus calculation.
        assert_eq!(result.phosphorus.status, BalanceStatus::Complete);
        assert_eq!(result.phosphorus.surplus_kg_per_ha, Some(5.0));
    }

    #[test]
    fn explicit_verified_zero_is_distinct_from_unknown() {
        let zero = known(0.0);
        assert_eq!(zero.kg_per_ha, Some(0.0));
        assert_eq!(zero.evidence, EvidenceClass::Measured);
        assert_eq!(NutrientMeasurement::unknown().kg_per_ha, None);
    }

    #[test]
    fn invalid_measurements_fail_closed_even_after_deserialization() {
        let mut input = complete_input();
        input.existing_soil_supply.potassium.kg_per_ha = Some(f64::NAN);
        assert!(calculate_budget(&input).is_err());

        input.existing_soil_supply.potassium.kg_per_ha = Some(-1.0);
        assert!(calculate_budget(&input).is_err());

        input.existing_soil_supply.potassium.kg_per_ha = Some(f64::INFINITY);
        assert!(calculate_budget(&input).is_err());
    }

    #[test]
    fn known_measurements_require_provenance() {
        let result = NutrientMeasurement::known(5.0, EvidenceClass::Measured, " ", "soil test");
        assert!(result.is_err());

        let result = NutrientMeasurement::known(5.0, EvidenceClass::Estimated, "model-1", "");
        assert!(result.is_err());
    }

    #[test]
    fn unknown_measurement_cannot_smuggle_a_zero() {
        let invalid = NutrientMeasurement {
            kg_per_ha: Some(0.0),
            evidence: EvidenceClass::Unknown,
            source_id: None,
            method: None,
        };
        let mut input = complete_input();
        input.organic_amendment_supply.nitrogen = invalid;
        assert!(calculate_budget(&input).is_err());
    }

    #[test]
    fn oxide_conversions_use_elemental_mass_basis() {
        assert!((elemental_p_from_p2o5(100.0).unwrap() - 43.64).abs() < 1e-10);
        assert!((elemental_k_from_k2o(100.0).unwrap() - 83.01).abs() < 1e-10);
        assert!(elemental_p_from_p2o5(-0.1).is_err());
        assert!(elemental_k_from_k2o(f64::NAN).is_err());
    }

    #[test]
    fn overflowed_supply_subtotal_is_rejected() {
        let mut input = complete_input();
        input.existing_soil_supply.nitrogen = known(f64::MAX);
        input.mineral_fertilizer_supply.nitrogen = known(f64::MAX);
        assert!(calculate_budget(&input).is_err());
    }

    #[test]
    fn invalid_budget_context_is_rejected() {
        let mut input = complete_input();
        input.context.site_id = "  ".into();
        assert!(calculate_budget(&input).is_err());

        input = complete_input();
        input.context.area_hectares = 0.0;
        assert!(calculate_budget(&input).is_err());

        input.context.area_hectares = f64::INFINITY;
        assert!(calculate_budget(&input).is_err());
    }

    #[test]
    fn result_retains_context_and_provenance_for_auditability() {
        let input = complete_input();
        let result = calculate_budget(&input).unwrap();
        assert_eq!(result.inputs, input);
        assert_eq!(
            result.inputs.existing_soil_supply.nitrogen.source_id.as_deref(),
            Some("lab-or-ledger-001")
        );
    }

    #[test]
    fn serde_roundtrip_preserves_provenance_and_unknowns() {
        let mut input = complete_input();
        input.organic_amendment_supply.nitrogen = NutrientMeasurement::unknown();

        let encoded = serde_json::to_string(&input).unwrap();
        let decoded: NutrientBudgetInput = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, input);
        assert!(calculate_budget(&decoded).unwrap().nitrogen.deficit_kg_per_ha.is_none());
    }
}
