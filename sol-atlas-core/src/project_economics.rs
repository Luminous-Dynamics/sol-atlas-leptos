// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Transparent project-level lifecycle economics for energy systems.
//!
//! This is a simplified, pre-tax, unlevered model. Inputs must use a consistent
//! currency basis and a discount rate consistent with nominal/real cost inputs.
//! It excludes financing structure, taxes, incentives, inflation changes,
//! salvage value, and grid/network value unless represented in the supplied
//! energy value. LCOE alone is not a system-level grid optimization objective.

use serde::{Deserialize, Serialize};

const MAX_LIFETIME_YEARS: u32 = 200;

/// A scheduled replacement or major refurbishment cost, paid at the end of year.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReplacementCost {
    pub year: u32,
    pub cost_usd: f64,
}

/// Inputs for an annual energy-project cash-flow model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectEconomicsInput {
    pub initial_capital_cost_usd: f64,
    pub fixed_annual_om_cost_usd: f64,
    pub variable_om_cost_per_kwh_usd: f64,
    /// Net energy delivered in year one, before subsequent annual degradation.
    pub annual_generation_kwh: f64,
    /// Value per delivered kWh (for example, a tariff or modeled avoided cost).
    pub energy_value_usd_per_kwh: f64,
    /// Fractional reduction in annual generation each year, in [0, 1).
    pub annual_generation_degradation_fraction: f64,
    /// Annual discount rate. Use a rate consistent with the currency/inflation basis.
    pub discount_rate: f64,
    pub lifetime_years: u32,
    pub replacement_costs: Vec<ReplacementCost>,
    pub end_of_life_decommissioning_cost_usd: f64,
}

/// Auditable cash flow for one project year. Monetary fields are USD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnualProjectCashFlow {
    pub year: u32,
    pub generation_kwh: f64,
    pub energy_value_usd: f64,
    pub fixed_om_cost_usd: f64,
    pub variable_om_cost_usd: f64,
    pub replacement_cost_usd: f64,
    pub decommissioning_cost_usd: f64,
    pub net_cash_flow_usd: f64,
    pub discounted_net_cash_flow_usd: f64,
}

/// Lifecycle economics. LCOE is discounted lifecycle cost divided by discounted
/// delivered generation; NPV uses the same annual generation/value assumptions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectEconomicsResult {
    pub lifecycle_cost_present_value_usd: f64,
    pub discounted_generation_kwh: f64,
    pub lcoe_usd_per_kwh: f64,
    pub net_present_value_usd: f64,
    pub annual_cash_flows: Vec<AnnualProjectCashFlow>,
}

/// Fail-closed input and arithmetic errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectEconomicsError {
    InvalidLifetime,
    InvalidFinancialInput,
    InvalidGenerationInput,
    InvalidReplacementSchedule,
    NonFiniteResult,
    NoDiscountedGeneration,
}

impl std::fmt::Display for ProjectEconomicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidLifetime => "project lifetime must be between 1 and 200 years",
            Self::InvalidFinancialInput => "financial inputs must be finite and non-negative",
            Self::InvalidGenerationInput => "generation and degradation inputs are outside their physical domain",
            Self::InvalidReplacementSchedule => "replacement costs must occur inside the project lifetime",
            Self::NonFiniteResult => "project economics produced a non-finite result",
            Self::NoDiscountedGeneration => "project has no positive discounted generation",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ProjectEconomicsError {}

/// Calculate discounted lifecycle LCOE and pre-tax project NPV.
///
/// Year zero contains the initial capital cost. Operating cash flows, replacement
/// costs, decommissioning costs, and delivered generation are discounted at the
/// end of each modeled year. Energy degradation starts after year one.
pub fn evaluate_project(
    input: &ProjectEconomicsInput,
) -> Result<ProjectEconomicsResult, ProjectEconomicsError> {
    if input.lifetime_years == 0 || input.lifetime_years > MAX_LIFETIME_YEARS {
        return Err(ProjectEconomicsError::InvalidLifetime);
    }

    let non_negative_finite = [
        input.initial_capital_cost_usd,
        input.fixed_annual_om_cost_usd,
        input.variable_om_cost_per_kwh_usd,
        input.energy_value_usd_per_kwh,
        input.end_of_life_decommissioning_cost_usd,
        input.discount_rate,
    ]
    .into_iter()
    .all(|value| value.is_finite() && value >= 0.0);
    if !non_negative_finite {
        return Err(ProjectEconomicsError::InvalidFinancialInput);
    }

    if !input.annual_generation_kwh.is_finite()
        || input.annual_generation_kwh <= 0.0
        || !input.annual_generation_degradation_fraction.is_finite()
        || !(0.0..1.0).contains(&input.annual_generation_degradation_fraction)
    {
        return Err(ProjectEconomicsError::InvalidGenerationInput);
    }

    if input.replacement_costs.iter().any(|cost| {
        cost.year == 0
            || cost.year > input.lifetime_years
            || !cost.cost_usd.is_finite()
            || cost.cost_usd < 0.0
    }) {
        return Err(ProjectEconomicsError::InvalidReplacementSchedule);
    }

    let mut annual_cash_flows = Vec::with_capacity(input.lifetime_years as usize);
    let mut lifecycle_cost_pv = input.initial_capital_cost_usd;
    let mut discounted_generation = 0.0;
    let mut discounted_operating_value = 0.0;
    let degradation_factor = 1.0 - input.annual_generation_degradation_fraction;

    for year in 1..=input.lifetime_years {
        let year_index = year as i32;
        let discount_factor = (1.0 + input.discount_rate).powi(year_index);
        let generation = input.annual_generation_kwh
            * degradation_factor.powi((year - 1) as i32);
        let energy_value = generation * input.energy_value_usd_per_kwh;
        let variable_om = generation * input.variable_om_cost_per_kwh_usd;
        let replacement_cost: f64 = input
            .replacement_costs
            .iter()
            .filter(|cost| cost.year == year)
            .map(|cost| cost.cost_usd)
            .sum();
        let decommissioning_cost = if year == input.lifetime_years {
            input.end_of_life_decommissioning_cost_usd
        } else {
            0.0
        };
        let undiscounted_cost =
            input.fixed_annual_om_cost_usd + variable_om + replacement_cost + decommissioning_cost;
        let net_cash_flow = energy_value - undiscounted_cost;
        let discounted_cost = undiscounted_cost / discount_factor;
        let discounted_energy = generation / discount_factor;
        let discounted_net_cash_flow = net_cash_flow / discount_factor;

        lifecycle_cost_pv += discounted_cost;
        discounted_generation += discounted_energy;
        discounted_operating_value += discounted_net_cash_flow;
        annual_cash_flows.push(AnnualProjectCashFlow {
            year,
            generation_kwh: generation,
            energy_value_usd: energy_value,
            fixed_om_cost_usd: input.fixed_annual_om_cost_usd,
            variable_om_cost_usd: variable_om,
            replacement_cost_usd: replacement_cost,
            decommissioning_cost_usd: decommissioning_cost,
            net_cash_flow_usd: net_cash_flow,
            discounted_net_cash_flow_usd: discounted_net_cash_flow,
        });
    }

    if !lifecycle_cost_pv.is_finite()
        || !discounted_generation.is_finite()
        || !discounted_operating_value.is_finite()
        || annual_cash_flows.iter().any(|year| {
            [
                year.generation_kwh,
                year.energy_value_usd,
                year.fixed_om_cost_usd,
                year.variable_om_cost_usd,
                year.replacement_cost_usd,
                year.decommissioning_cost_usd,
                year.net_cash_flow_usd,
                year.discounted_net_cash_flow_usd,
            ]
            .iter()
            .any(|value| !value.is_finite())
        })
    {
        return Err(ProjectEconomicsError::NonFiniteResult);
    }
    if discounted_generation <= 0.0 {
        return Err(ProjectEconomicsError::NoDiscountedGeneration);
    }

    let lcoe = lifecycle_cost_pv / discounted_generation;
    let npv = -input.initial_capital_cost_usd + discounted_operating_value;
    if !lcoe.is_finite() || !npv.is_finite() {
        return Err(ProjectEconomicsError::NonFiniteResult);
    }

    Ok(ProjectEconomicsResult {
        lifecycle_cost_present_value_usd: lifecycle_cost_pv,
        discounted_generation_kwh: discounted_generation,
        lcoe_usd_per_kwh: lcoe,
        net_present_value_usd: npv,
        annual_cash_flows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> ProjectEconomicsInput {
        ProjectEconomicsInput {
            initial_capital_cost_usd: 1_000.0,
            fixed_annual_om_cost_usd: 50.0,
            variable_om_cost_per_kwh_usd: 0.0,
            annual_generation_kwh: 1_000.0,
            energy_value_usd_per_kwh: 0.8,
            annual_generation_degradation_fraction: 0.0,
            discount_rate: 0.0,
            lifetime_years: 2,
            replacement_costs: vec![],
            end_of_life_decommissioning_cost_usd: 0.0,
        }
    }

    #[test]
    fn computes_discounted_lcoe_and_npv_for_simple_project() {
        let result = evaluate_project(&baseline()).unwrap();
        assert!((result.lifecycle_cost_present_value_usd - 1_100.0).abs() < 1e-9);
        assert!((result.discounted_generation_kwh - 2_000.0).abs() < 1e-9);
        assert!((result.lcoe_usd_per_kwh - 0.55).abs() < 1e-12);
        assert!((result.net_present_value_usd - 500.0).abs() < 1e-9);
    }

    #[test]
    fn applies_degradation_and_scheduled_replacement_costs() {
        let mut input = baseline();
        input.initial_capital_cost_usd = 100.0;
        input.fixed_annual_om_cost_usd = 0.0;
        input.annual_generation_kwh = 100.0;
        input.energy_value_usd_per_kwh = 0.0;
        input.annual_generation_degradation_fraction = 0.1;
        input.lifetime_years = 2;
        input.replacement_costs = vec![ReplacementCost {
            year: 2,
            cost_usd: 20.0,
        }];
        input.end_of_life_decommissioning_cost_usd = 10.0;
        let result = evaluate_project(&input).unwrap();
        assert!((result.discounted_generation_kwh - 190.0).abs() < 1e-9);
        assert!((result.lifecycle_cost_present_value_usd - 130.0).abs() < 1e-9);
        assert!((result.lcoe_usd_per_kwh - (130.0 / 190.0)).abs() < 1e-12);
    }

    #[test]
    fn discounts_costs_generation_and_operating_value_consistently() {
        let mut input = baseline();
        input.discount_rate = 0.1;
        let result = evaluate_project(&input).unwrap();
        let discounted_generation = 1_000.0 / 1.1 + 1_000.0 / 1.21;
        let discounted_cost = 1_000.0 + 50.0 / 1.1 + 50.0 / 1.21;
        let discounted_value = 800.0 / 1.1 + 800.0 / 1.21;
        assert!((result.discounted_generation_kwh - discounted_generation).abs() < 1e-9);
        assert!((result.lifecycle_cost_present_value_usd - discounted_cost).abs() < 1e-9);
        assert!((result.net_present_value_usd - (-1_000.0 + discounted_value - 50.0/1.1 - 50.0/1.21)).abs() < 1e-9);
    }

    #[test]
    fn rejects_invalid_cash_flows_and_replacement_years() {
        let mut input = baseline();
        input.annual_generation_kwh = f64::NAN;
        assert_eq!(
            evaluate_project(&input).unwrap_err(),
            ProjectEconomicsError::InvalidGenerationInput
        );
        let mut input = baseline();
        input.replacement_costs = vec![ReplacementCost { year: 0, cost_usd: 1.0 }];
        assert_eq!(
            evaluate_project(&input).unwrap_err(),
            ProjectEconomicsError::InvalidReplacementSchedule
        );
    }
}
