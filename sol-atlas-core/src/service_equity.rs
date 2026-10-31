// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Distributional service-floor assessment for community resilience.
//!
//! This module evaluates who is above, below or uncertain relative to an explicit
//! essential-service coverage floor. It deliberately reports per-community outcomes
//! and lower/upper bounds rather than compressing distribution into one welfare score.
//! Beneficiary weights represent caller-defined service-cell exposure weights; they
//! are not guaranteed to be unique people or comparable across datasets.

use crate::service_function::{
    ServiceFunctionAssessment, ServiceFunctionAssessmentError, ServiceFunctionScenario,
    assess_service_functions,
};
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SERVICE_EQUITY_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceEquityCell {
    /// Stable row ID within this distributional assessment.
    pub id: String,
    /// Reference to a ServiceFunctionScenario.id that genuinely describes delivery
    /// for this service/community cell. Reuse across communities explicitly assumes
    /// the referenced scenario's coverage applies to each reused community.
    pub service_scenario_id: String,
    /// Region, community, facility catchment, or other declared beneficiary group.
    pub community_id: String,
    /// Caller-defined comparable weight (for example, qualified population served).
    /// This is not guaranteed to identify unique beneficiaries across service cells.
    pub beneficiary_weight: u64,
    /// Required conservative coverage floor, from 0 to 1 inclusive.
    pub essential_coverage_floor: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceEquityScenario {
    pub schema_version: String,
    pub id: String,
    /// Canonical UTC timestamp: YYYY-MM-DDTHH:MM:SSZ.
    pub as_of: String,
    pub cells: Vec<ServiceEquityCell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceFloorStatus {
    /// Conservative coverage lower bound meets the floor.
    MeetsFloor,
    /// Even the optimistic coverage upper bound remains below the floor.
    BelowFloor,
    /// Current measurement uncertainty can change whether the floor is met.
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceEquityCellAssessment {
    pub cell_id: String,
    pub service_scenario_id: String,
    pub service_entity_id: String,
    pub community_id: String,
    pub beneficiary_weight: u64,
    pub essential_coverage_floor: f64,
    pub service_state: crate::service_function::ServiceFunctionState,
    pub coverage_lower_bound: Option<f64>,
    pub coverage_upper_bound: Option<f64>,
    pub floor_status: ServiceFloorStatus,
    pub available_capacity_evidence_ids: Vec<String>,
    pub required_capacity_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunityServiceFloorAssessment {
    pub community_id: String,
    pub cell_count: usize,
    pub total_beneficiary_weight: u64,
    pub meets_floor_weight: u64,
    pub below_floor_weight: u64,
    pub indeterminate_weight: u64,
    /// Guaranteed fraction of this community's weighted service cells meeting floor.
    pub floor_compliance_lower_bound: f64,
    /// Maximum fraction that could meet floor if all indeterminate cells resolve favorably.
    pub floor_compliance_upper_bound: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceEquityAssessment {
    pub schema_version: String,
    pub scenario_id: String,
    pub as_of: String,
    pub community_count: usize,
    pub cell_count: usize,
    pub total_beneficiary_weight: u64,
    pub meets_floor_weight: u64,
    pub below_floor_weight: u64,
    pub indeterminate_weight: u64,
    /// Weighted service-cell floor compliance; this is not unique-population coverage.
    pub overall_floor_compliance_lower_bound: f64,
    /// Includes all indeterminate weight as potentially meeting floor.
    pub overall_floor_compliance_upper_bound: f64,
    /// Bounds on the difference between community-level floor-compliance fractions.
    pub community_compliance_gap_lower_bound: f64,
    pub community_compliance_gap_upper_bound: f64,
    pub communities: Vec<CommunityServiceFloorAssessment>,
    /// Sorted by community ID, then service scenario ID, then cell ID.
    pub cells: Vec<ServiceEquityCellAssessment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceEquityValidationError {
    UnsupportedSchemaVersion,
    EmptyScenarioId,
    InvalidAsOfTimestamp,
    EmptyCellSet,
    EmptyCellId,
    DuplicateCellId,
    EmptyServiceScenarioId,
    EmptyCommunityId,
    ZeroBeneficiaryWeight,
    InvalidCoverageFloor,
    DuplicateServiceCommunityCell,
    UnknownServiceScenario,
    ServiceTimestampMismatch,
    WeightOverflow,
    NonFiniteAggregate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceEquityError {
    InvalidInput(Vec<ServiceEquityValidationError>),
    ServiceAssessment(ServiceFunctionAssessmentError),
}

#[derive(Debug, Default)]
struct CommunityAccumulator {
    cell_count: usize,
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
    ) -> Result<(), ServiceEquityValidationError> {
        self.cell_count += 1;
        self.total_weight = self
            .total_weight
            .checked_add(weight)
            .ok_or(ServiceEquityValidationError::WeightOverflow)?;
        match status {
            ServiceFloorStatus::MeetsFloor => {
                self.meets_weight = self
                    .meets_weight
                    .checked_add(weight)
                    .ok_or(ServiceEquityValidationError::WeightOverflow)?;
            }
            ServiceFloorStatus::BelowFloor => {
                self.below_weight = self
                    .below_weight
                    .checked_add(weight)
                    .ok_or(ServiceEquityValidationError::WeightOverflow)?;
            }
            ServiceFloorStatus::Indeterminate => {
                self.indeterminate_weight = self
                    .indeterminate_weight
                    .checked_add(weight)
                    .ok_or(ServiceEquityValidationError::WeightOverflow)?;
            }
        }
        Ok(())
    }

    fn finish(self, community_id: String) -> CommunityServiceFloorAssessment {
        let lower = self.meets_weight as f64 / self.total_weight as f64;
        let upper =
            (self.meets_weight + self.indeterminate_weight) as f64 / self.total_weight as f64;
        CommunityServiceFloorAssessment {
            community_id,
            cell_count: self.cell_count,
            total_beneficiary_weight: self.total_weight,
            meets_floor_weight: self.meets_weight,
            below_floor_weight: self.below_weight,
            indeterminate_weight: self.indeterminate_weight,
            floor_compliance_lower_bound: lower,
            floor_compliance_upper_bound: upper,
        }
    }
}

/// Assess service-floor distribution across communities for one explicit snapshot.
///
/// Service capacity/demand scenarios are batch-evaluated against one evidence corpus.
/// Each input scenario must already represent the service capacity/demand applicable
/// to its mapped community cell: this function does not spatially disaggregate a
/// network's flow among communities. Reusing a scenario across communities explicitly
/// applies the same assessed coverage to each row.
/// The result includes weighted and per-community bounds plus a distributional gap
/// interval. It does not infer causal effects, uniquely count people across services,
/// or replace human review of rights and allocation decisions.
pub fn assess_service_equity(
    scenario: &ServiceEquityScenario,
    service_scenarios: &[ServiceFunctionScenario],
    evidence_records: &[crate::spatial_economic::EvidenceRecord],
) -> Result<ServiceEquityAssessment, ServiceEquityError> {
    let mut errors = Vec::new();
    if scenario.schema_version != SERVICE_EQUITY_SCHEMA_VERSION {
        errors.push(ServiceEquityValidationError::UnsupportedSchemaVersion);
    }
    if scenario.id.trim().is_empty() {
        errors.push(ServiceEquityValidationError::EmptyScenarioId);
    }
    if !is_canonical_utc_timestamp(&scenario.as_of) {
        errors.push(ServiceEquityValidationError::InvalidAsOfTimestamp);
    }
    if scenario.cells.is_empty() {
        errors.push(ServiceEquityValidationError::EmptyCellSet);
    }

    let mut cell_ids = BTreeSet::new();
    let mut service_community_pairs = BTreeSet::new();
    let mut referenced_scenario_ids = BTreeSet::new();
    for cell in &scenario.cells {
        if cell.id.trim().is_empty() {
            errors.push(ServiceEquityValidationError::EmptyCellId);
        }
        if !cell_ids.insert(cell.id.as_str()) {
            errors.push(ServiceEquityValidationError::DuplicateCellId);
        }
        if cell.service_scenario_id.trim().is_empty() {
            errors.push(ServiceEquityValidationError::EmptyServiceScenarioId);
        }
        if cell.community_id.trim().is_empty() {
            errors.push(ServiceEquityValidationError::EmptyCommunityId);
        }
        if cell.beneficiary_weight == 0 {
            errors.push(ServiceEquityValidationError::ZeroBeneficiaryWeight);
        }
        if !cell.essential_coverage_floor.is_finite()
            || !(0.0..=1.0).contains(&cell.essential_coverage_floor)
        {
            errors.push(ServiceEquityValidationError::InvalidCoverageFloor);
        }
        if !service_community_pairs.insert((
            cell.service_scenario_id.as_str(),
            cell.community_id.as_str(),
        )) {
            errors.push(ServiceEquityValidationError::DuplicateServiceCommunityCell);
        }
        referenced_scenario_ids.insert(cell.service_scenario_id.as_str());
    }

    let mut service_ids = BTreeSet::new();
    for service in service_scenarios {
        service_ids.insert(service.id.as_str());
        if service.as_of != scenario.as_of {
            errors.push(ServiceEquityValidationError::ServiceTimestampMismatch);
        }
    }
    for id in &referenced_scenario_ids {
        if !service_ids.contains(id) {
            errors.push(ServiceEquityValidationError::UnknownServiceScenario);
        }
    }

    if !errors.is_empty() {
        errors.sort_by_key(|error| *error as u8);
        errors.dedup();
        return Err(ServiceEquityError::InvalidInput(errors));
    }

    let assessments = assess_service_functions(service_scenarios, evidence_records)
        .map_err(ServiceEquityError::ServiceAssessment)?;
    let assessments_by_id: BTreeMap<&str, &ServiceFunctionAssessment> = assessments
        .iter()
        .map(|assessment| (assessment.scenario_id.as_str(), assessment))
        .collect();

    let mut cells = Vec::with_capacity(scenario.cells.len());
    let mut groups: BTreeMap<String, CommunityAccumulator> = BTreeMap::new();
    let mut total_weight = 0_u64;
    let mut meets_weight = 0_u64;
    let mut below_weight = 0_u64;
    let mut indeterminate_weight = 0_u64;

    let mut ordered_cells: Vec<&ServiceEquityCell> = scenario.cells.iter().collect();
    ordered_cells.sort_by(|left, right| {
        left.community_id
            .cmp(&right.community_id)
            .then_with(|| left.service_scenario_id.cmp(&right.service_scenario_id))
            .then_with(|| left.id.cmp(&right.id))
    });

    for cell in ordered_cells {
        let service = assessments_by_id[cell.service_scenario_id.as_str()];
        let lower = service.coverage_lower_bound;
        let upper = service.coverage_upper_bound;
        let status = if lower.is_some_and(|value| value >= cell.essential_coverage_floor) {
            ServiceFloorStatus::MeetsFloor
        } else if upper.is_some_and(|value| value < cell.essential_coverage_floor) {
            ServiceFloorStatus::BelowFloor
        } else {
            ServiceFloorStatus::Indeterminate
        };

        total_weight = total_weight
            .checked_add(cell.beneficiary_weight)
            .ok_or_else(|| {
                ServiceEquityError::InvalidInput(vec![ServiceEquityValidationError::WeightOverflow])
            })?;
        match status {
            ServiceFloorStatus::MeetsFloor => {
                meets_weight = meets_weight
                    .checked_add(cell.beneficiary_weight)
                    .ok_or_else(|| {
                        ServiceEquityError::InvalidInput(vec![
                            ServiceEquityValidationError::WeightOverflow,
                        ])
                    })?
            }
            ServiceFloorStatus::BelowFloor => {
                below_weight = below_weight
                    .checked_add(cell.beneficiary_weight)
                    .ok_or_else(|| {
                        ServiceEquityError::InvalidInput(vec![
                            ServiceEquityValidationError::WeightOverflow,
                        ])
                    })?
            }
            ServiceFloorStatus::Indeterminate => {
                indeterminate_weight = indeterminate_weight
                    .checked_add(cell.beneficiary_weight)
                    .ok_or_else(|| {
                        ServiceEquityError::InvalidInput(vec![
                            ServiceEquityValidationError::WeightOverflow,
                        ])
                    })?
            }
        }
        groups
            .entry(cell.community_id.clone())
            .or_default()
            .add(cell.beneficiary_weight, status)
            .map_err(|error| ServiceEquityError::InvalidInput(vec![error]))?;

        cells.push(ServiceEquityCellAssessment {
            cell_id: cell.id.clone(),
            service_scenario_id: cell.service_scenario_id.clone(),
            service_entity_id: service.service_entity_id.clone(),
            community_id: cell.community_id.clone(),
            beneficiary_weight: cell.beneficiary_weight,
            essential_coverage_floor: cell.essential_coverage_floor,
            service_state: service.state,
            coverage_lower_bound: lower,
            coverage_upper_bound: upper,
            floor_status: status,
            available_capacity_evidence_ids: service.available_capacity_evidence_ids.clone(),
            required_capacity_evidence_ids: service.required_capacity_evidence_ids.clone(),
        });
    }

    if total_weight == 0 {
        return Err(ServiceEquityError::InvalidInput(vec![
            ServiceEquityValidationError::ZeroBeneficiaryWeight,
        ]));
    }
    let overall_floor_compliance_lower_bound = meets_weight as f64 / total_weight as f64;
    let overall_floor_compliance_upper_bound =
        (meets_weight + indeterminate_weight) as f64 / total_weight as f64;
    let communities: Vec<CommunityServiceFloorAssessment> = groups
        .into_iter()
        .map(|(id, aggregate)| aggregate.finish(id))
        .collect();

    let community_compliance_gap_lower_bound;
    let community_compliance_gap_upper_bound;
    if communities.len() <= 1 {
        community_compliance_gap_lower_bound = 0.0;
        community_compliance_gap_upper_bound = 0.0;
    } else {
        let max_lower = communities
            .iter()
            .map(|c| c.floor_compliance_lower_bound)
            .fold(0.0_f64, f64::max);
        let min_lower = communities
            .iter()
            .map(|c| c.floor_compliance_lower_bound)
            .fold(1.0_f64, f64::min);
        let max_upper = communities
            .iter()
            .map(|c| c.floor_compliance_upper_bound)
            .fold(0.0_f64, f64::max);
        let min_upper = communities
            .iter()
            .map(|c| c.floor_compliance_upper_bound)
            .fold(1.0_f64, f64::min);
        community_compliance_gap_lower_bound = (max_lower - min_upper).max(0.0);
        community_compliance_gap_upper_bound = (max_upper - min_lower).max(0.0).min(1.0);
    }

    if !overall_floor_compliance_lower_bound.is_finite()
        || !overall_floor_compliance_upper_bound.is_finite()
        || !community_compliance_gap_lower_bound.is_finite()
        || !community_compliance_gap_upper_bound.is_finite()
    {
        return Err(ServiceEquityError::InvalidInput(vec![
            ServiceEquityValidationError::NonFiniteAggregate,
        ]));
    }

    Ok(ServiceEquityAssessment {
        schema_version: SERVICE_EQUITY_SCHEMA_VERSION.into(),
        scenario_id: scenario.id.clone(),
        as_of: scenario.as_of.clone(),
        community_count: communities.len(),
        cell_count: cells.len(),
        total_beneficiary_weight: total_weight,
        meets_floor_weight: meets_weight,
        below_floor_weight: below_weight,
        indeterminate_weight: indeterminate_weight,
        overall_floor_compliance_lower_bound,
        overall_floor_compliance_upper_bound,
        community_compliance_gap_lower_bound,
        community_compliance_gap_upper_bound,
        communities,
        cells,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_logic::ScenarioBasis;
    use crate::service_function::SERVICE_FUNCTION_SCHEMA_VERSION;
    use crate::spatial_economic::Quantity;

    const AT: &str = "2026-10-17T10:00:00Z";

    fn measurement(
        entity: &str,
        value: f64,
        uncertainty: Option<f64>,
    ) -> crate::service_function::FunctionMeasurement {
        crate::service_function::FunctionMeasurement {
            entity_id: entity.into(),
            quantity: Quantity {
                value,
                unit: "m3/h".into(),
                uncertainty_abs: uncertainty,
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic service-equity scenario assumption".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn function_scenario(
        id: &str,
        service: &str,
        capacity: f64,
        capacity_uncertainty: Option<f64>,
        demand: f64,
        demand_uncertainty: Option<f64>,
    ) -> ServiceFunctionScenario {
        ServiceFunctionScenario {
            schema_version: SERVICE_FUNCTION_SCHEMA_VERSION.into(),
            id: id.into(),
            as_of: AT.into(),
            service_entity_id: service.into(),
            available_capacity: measurement(
                &format!("capacity:{service}"),
                capacity,
                capacity_uncertainty,
            ),
            required_capacity: measurement(service, demand, demand_uncertainty),
        }
    }

    fn cell(
        id: &str,
        service_scenario_id: &str,
        community: &str,
        weight: u64,
        floor: f64,
    ) -> ServiceEquityCell {
        ServiceEquityCell {
            id: id.into(),
            service_scenario_id: service_scenario_id.into(),
            community_id: community.into(),
            beneficiary_weight: weight,
            essential_coverage_floor: floor,
        }
    }

    fn scenario(cells: Vec<ServiceEquityCell>) -> ServiceEquityScenario {
        ServiceEquityScenario {
            schema_version: SERVICE_EQUITY_SCHEMA_VERSION.into(),
            id: "equity:water-floor".into(),
            as_of: AT.into(),
            cells,
        }
    }

    #[test]
    fn distribution_reports_floor_met_below_and_indeterminate_weight_separately() {
        let services = vec![
            function_scenario("service:a", "water:north", 9.0, None, 10.0, None),
            function_scenario("service:b", "water:south", 2.0, None, 10.0, None),
            function_scenario("service:c", "water:east", 7.0, Some(2.0), 10.0, Some(1.0)),
        ];
        let input = scenario(vec![
            cell("north-cell", "service:a", "north", 6, 0.8),
            cell("south-cell", "service:b", "south", 2, 0.8),
            cell("east-cell", "service:c", "east", 2, 0.8),
        ]);
        let result = assess_service_equity(&input, &services, &[]).unwrap();
        assert_eq!(result.total_beneficiary_weight, 10);
        assert_eq!(result.meets_floor_weight, 6);
        assert_eq!(result.below_floor_weight, 2);
        assert_eq!(result.indeterminate_weight, 2);
        assert_eq!(result.overall_floor_compliance_lower_bound, 0.6);
        assert_eq!(result.overall_floor_compliance_upper_bound, 0.8);
        assert_eq!(result.communities.len(), 3);
        assert_eq!(result.cells[0].community_id, "east");
        assert_eq!(
            result.cells[0].floor_status,
            ServiceFloorStatus::Indeterminate
        );
        assert_eq!(result.cells[1].floor_status, ServiceFloorStatus::MeetsFloor);
        assert_eq!(result.cells[2].floor_status, ServiceFloorStatus::BelowFloor);
        assert_eq!(result.community_compliance_gap_lower_bound, 1.0);
        assert_eq!(result.community_compliance_gap_upper_bound, 1.0);
    }

    #[test]
    fn indeterminate_service_weight_is_never_counted_as_definitely_served_or_definitely_below_floor()
     {
        let service = function_scenario(
            "service:uncertain",
            "water:uncertain",
            8.0,
            Some(2.0),
            10.0,
            Some(1.0),
        );
        let result = assess_service_equity(
            &scenario(vec![cell(
                "uncertain",
                "service:uncertain",
                "district",
                15,
                0.8,
            )]),
            &[service],
            &[],
        )
        .unwrap();
        assert_eq!(result.meets_floor_weight, 0);
        assert_eq!(result.below_floor_weight, 0);
        assert_eq!(result.indeterminate_weight, 15);
        assert_eq!(result.overall_floor_compliance_lower_bound, 0.0);
        assert_eq!(result.overall_floor_compliance_upper_bound, 1.0);
    }

    #[test]
    fn exact_coverage_floor_boundary_counts_as_meeting_the_floor() {
        let service =
            function_scenario("service:boundary", "water:boundary", 8.0, None, 10.0, None);
        let result = assess_service_equity(
            &scenario(vec![cell(
                "boundary-cell",
                "service:boundary",
                "district",
                3,
                0.8,
            )]),
            &[service],
            &[],
        )
        .unwrap();
        assert_eq!(result.cells[0].coverage_lower_bound, Some(0.8));
        assert_eq!(result.cells[0].floor_status, ServiceFloorStatus::MeetsFloor);
        assert_eq!(result.meets_floor_weight, 3);
        assert_eq!(result.indeterminate_weight, 0);
    }

    #[test]
    fn zero_requirement_and_absent_coverage_ratio_do_not_imply_floor_success() {
        let service = function_scenario(
            "service:zero-demand",
            "water:zero-demand",
            10.0,
            None,
            0.0,
            None,
        );
        let result = assess_service_equity(
            &scenario(vec![cell(
                "zero-demand-cell",
                "service:zero-demand",
                "district",
                4,
                0.5,
            )]),
            &[service],
            &[],
        )
        .unwrap();
        assert_eq!(result.cells[0].coverage_lower_bound, None);
        assert_eq!(result.cells[0].coverage_upper_bound, None);
        assert_eq!(
            result.cells[0].floor_status,
            ServiceFloorStatus::Indeterminate
        );
        assert_eq!(result.meets_floor_weight, 0);
        assert_eq!(result.indeterminate_weight, 4);
    }

    #[test]
    fn community_gap_bounds_capture_uncertainty_between_groups() {
        let services = vec![
            function_scenario("met", "water:met", 10.0, None, 10.0, None),
            function_scenario(
                "uncertain",
                "water:uncertain",
                8.0,
                Some(2.0),
                10.0,
                Some(1.0),
            ),
        ];
        let result = assess_service_equity(
            &scenario(vec![
                cell("met-cell", "met", "served-community", 10, 0.8),
                cell(
                    "uncertain-cell",
                    "uncertain",
                    "uncertain-community",
                    10,
                    0.8,
                ),
            ]),
            &services,
            &[],
        )
        .unwrap();
        assert_eq!(result.community_compliance_gap_lower_bound, 0.0);
        assert_eq!(result.community_compliance_gap_upper_bound, 1.0);
    }

    #[test]
    fn beneficiary_weights_not_cell_count_drive_overall_bounds() {
        let services = vec![
            function_scenario("served", "water:served", 10.0, None, 10.0, None),
            function_scenario("failed", "water:failed", 0.0, None, 10.0, None),
            function_scenario("served-2", "water:served-2", 10.0, None, 10.0, None),
        ];
        let result = assess_service_equity(
            &scenario(vec![
                cell("large-served", "served", "large", 90, 0.8),
                cell("small-failed", "failed", "small", 1, 0.8),
                cell("small-served", "served-2", "small", 9, 0.8),
            ]),
            &services,
            &[],
        )
        .unwrap();
        assert_eq!(result.cell_count, 3);
        assert_eq!(result.meets_floor_weight, 99);
        assert_eq!(result.below_floor_weight, 1);
        assert_eq!(result.overall_floor_compliance_lower_bound, 0.99);
        assert_eq!(result.communities[0].community_id, "large");
        assert_eq!(result.communities[1].floor_compliance_lower_bound, 0.9);
    }

    #[test]
    fn input_order_does_not_change_serialized_assessment() {
        let services = vec![
            function_scenario("scenario:b", "water:b", 9.0, None, 10.0, None),
            function_scenario("scenario:a", "water:a", 2.0, None, 10.0, None),
        ];
        let first = scenario(vec![
            cell("b-cell", "scenario:b", "west", 2, 0.8),
            cell("a-cell", "scenario:a", "east", 2, 0.8),
        ]);
        let mut second = first.clone();
        second.cells.reverse();
        let result_a = assess_service_equity(&first, &services, &[]).unwrap();
        let result_b = assess_service_equity(&second, &services, &[]).unwrap();
        assert_eq!(
            serde_json::to_vec(&result_a).unwrap(),
            serde_json::to_vec(&result_b).unwrap()
        );
    }

    #[test]
    fn service_equity_json_fixtures_match_distributional_golden_output() {
        let input: ServiceEquityScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/service-equity-scenario.json"
        ))
        .unwrap();
        let expected: ServiceEquityAssessment = serde_json::from_str(include_str!(
            "../../schemas/examples/service-equity-assessment.json"
        ))
        .unwrap();
        let services = vec![
            function_scenario("service:a", "water:north", 9.0, None, 10.0, None),
            function_scenario("service:b", "water:south", 2.0, None, 10.0, None),
            function_scenario("service:c", "water:east", 7.0, Some(2.0), 10.0, Some(1.0)),
        ];
        let actual = assess_service_equity(&input, &services, &[]).unwrap();
        // The JSON decimal is a one-ULP representation of the computed 5/11
        // ratio on this parser/toolchain. Keep the tolerance local to this one
        // derived scalar; every other fixture field remains exact.
        let actual_east_lower = actual.cells[0].coverage_lower_bound.unwrap();
        let expected_east_lower = expected.cells[0].coverage_lower_bound.unwrap();
        assert!(
            (actual_east_lower - expected_east_lower).abs() <= f64::EPSILON,
            "east lower coverage ratio differs beyond one ULP: actual={actual_east_lower:?}, expected={expected_east_lower:?}"
        );
        let mut expected = expected;
        expected.cells[0].coverage_lower_bound = Some(actual_east_lower);
        assert_eq!(actual, expected);
        assert_eq!(actual.overall_floor_compliance_lower_bound, 0.6);
        assert_eq!(actual.overall_floor_compliance_upper_bound, 0.8);
    }

    #[test]
    fn unknown_service_reference_and_timestamp_mismatch_fail_closed() {
        let valid_service = function_scenario("known", "water:service", 10.0, None, 10.0, None);
        let unknown_reference = assess_service_equity(
            &scenario(vec![cell("unknown-cell", "missing", "community", 1, 0.8)]),
            &[valid_service.clone()],
            &[],
        );
        assert!(
            matches!(unknown_reference, Err(ServiceEquityError::InvalidInput(errors))
            if errors.contains(&ServiceEquityValidationError::UnknownServiceScenario))
        );

        let mut other_time = valid_service;
        other_time.as_of = "2026-10-17T11:00:00Z".into();
        let mismatch = assess_service_equity(
            &scenario(vec![cell("known-cell", "known", "community", 1, 0.8)]),
            &[other_time],
            &[],
        );
        assert!(
            matches!(mismatch, Err(ServiceEquityError::InvalidInput(errors))
            if errors.contains(&ServiceEquityValidationError::ServiceTimestampMismatch))
        );
    }

    #[test]
    fn invalid_floor_duplicate_cell_and_weight_overflow_fail_closed() {
        let service = function_scenario("service", "water:service", 10.0, None, 10.0, None);
        let invalid_floor = assess_service_equity(
            &scenario(vec![cell("cell", "service", "district", 1, 1.01)]),
            &[service.clone()],
            &[],
        );
        assert!(
            matches!(invalid_floor, Err(ServiceEquityError::InvalidInput(errors))
            if errors.contains(&ServiceEquityValidationError::InvalidCoverageFloor))
        );

        let duplicate = assess_service_equity(
            &scenario(vec![
                cell("same", "service", "district", u64::MAX, 0.8),
                cell("same", "service", "district-2", 1, 0.8),
            ]),
            &[service.clone()],
            &[],
        );
        assert!(
            matches!(duplicate, Err(ServiceEquityError::InvalidInput(errors))
            if errors.contains(&ServiceEquityValidationError::DuplicateCellId))
        );

        let overflow = assess_service_equity(
            &scenario(vec![
                cell("first", "service", "district-a", u64::MAX, 0.8),
                cell("second", "service", "district-b", 1, 0.8),
            ]),
            &[service],
            &[],
        );
        assert!(
            matches!(overflow, Err(ServiceEquityError::InvalidInput(errors))
            if errors.contains(&ServiceEquityValidationError::WeightOverflow))
        );
    }
}
