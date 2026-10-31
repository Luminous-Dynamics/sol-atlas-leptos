// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Capacity-aware directed flow composition for spatial-economic services.
//!
//! The evaluator combines edge capacities through conservation of flow instead
//! of summing nominal asset capacities. It reports a conservative max-flow bound
//! using active links and lower capacity bounds, plus an optimistic bound that
//! includes links with unknown availability and upper capacity bounds.
//!
//! This is a bounded network abstraction, not a hydraulic, electrical load-flow,
//! transport-traffic, probabilistic reliability, or causal-impact model.

use crate::dependency_logic::ScenarioBasis;
use crate::service_function::FunctionMeasurement;
use crate::spatial_economic::{EvidenceKind, EvidenceRecord, EvidenceState, Quantity};
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const SERVICE_NETWORK_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkTopologyCompleteness {
    /// The supplied edges represent the complete modeled route topology.
    Complete,
    /// Relevant edges or constraints are known to be omitted.
    Partial,
    /// Completeness has not been established.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkLinkAvailability {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkLink {
    pub id: String,
    pub from_entity_id: String,
    pub to_entity_id: String,
    /// Capacity measurement entity_id must equal this link ID.
    pub capacity: FunctionMeasurement,
    /// Explicit scenario condition; unknown links contribute only to the upper bound.
    pub availability: NetworkLinkAvailability,
    pub availability_basis: ScenarioBasis,
    pub availability_rationale: String,
    #[serde(default)]
    pub availability_supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceCapacityNetworkScenario {
    pub schema_version: String,
    pub id: String,
    /// Canonical UTC timestamp: YYYY-MM-DDTHH:MM:SSZ.
    pub as_of: String,
    pub service_entity_id: String,
    /// Completeness must be explicit; incomplete topology cannot justify an outage
    /// or adequacy conclusion, even when its modeled subgraph has zero or high flow.
    pub topology_completeness: NetworkTopologyCompleteness,
    pub source_entity_ids: Vec<String>,
    pub required_capacity: FunctionMeasurement,
    pub links: Vec<NetworkLink>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkAssessmentState {
    Adequate,
    Degraded,
    Unavailable,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceCapacityNetworkAssessment {
    pub schema_version: String,
    pub scenario_id: String,
    pub service_entity_id: String,
    pub as_of: String,
    pub topology_completeness: NetworkTopologyCompleteness,
    pub state: NetworkAssessmentState,
    pub unit: String,
    pub required_capacity_value: f64,
    pub required_capacity_lower_bound: f64,
    pub required_capacity_upper_bound: f64,
    /// Max flow in the declared graph with active links and conservative capacities.
    /// These are service-wide bounds only when topology_completeness is Complete.
    pub conservative_available_capacity: f64,
    /// Max flow in the declared graph with active plus unknown links and upper capacities.
    /// These are service-wide bounds only when topology_completeness is Complete.
    pub optimistic_available_capacity: f64,
    /// Omitted if topology completeness is not established or demand has a zero upper bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_lower_bound: Option<f64>,
    /// Omitted if topology completeness is not established or demand lower bound is zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_upper_bound: Option<f64>,
    /// Min-cut link IDs for the declared conservative graph, not necessarily service-wide
    /// when topology completeness is Partial or Unknown; sorted deterministically.
    pub conservative_min_cut_link_ids: Vec<String>,
    pub active_link_ids: Vec<String>,
    pub inactive_link_ids: Vec<String>,
    pub unknown_link_ids: Vec<String>,
    pub capacity_evidence_ids: Vec<String>,
    pub availability_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceCapacityNetworkValidationError {
    UnsupportedSchemaVersion,
    EmptyScenarioSet,
    DuplicateScenarioId,
    EmptyScenarioId,
    InvalidAsOfTimestamp,
    EmptyServiceEntityId,
    EmptySourceSet,
    EmptySourceEntityId,
    DuplicateSourceEntityId,
    SourceIsServiceEntity,
    EmptyLinkId,
    DuplicateLinkId,
    EmptyLinkEndpoint,
    SelfLoopLink,
    EmptyLinkRationale,
    LinkCapacityEntityMismatch,
    RequiredCapacityEntityMismatch,
    UnitMismatch,
    DuplicateEvidenceRecordId,
    InvalidEvidenceRecord,
    EmptyEvidenceId,
    UnknownEvidenceReference,
    MissingEvidenceForObservedMeasurement,
    EvidenceForbiddenForAssumedMeasurement,
    MissingEvidenceForObservedAvailability,
    EvidenceForbiddenForAssumedAvailability,
    InadmissibleSupportingEvidence,
    SupportingEvidenceWrongKind,
    SupportingEvidenceSubjectMismatch,
    SupportingEvidenceMissingQuantity,
    SupportingEvidenceQuantityMismatch,
    InvalidSupportingEvidenceTimestamp,
    EvidenceAfterAsOf,
    InvalidSupportingEvidenceValidity,
    OutsideSupportingEvidenceValidity,
    DuplicateSupportingEvidenceId,
    EmptyMeasurementEntityId,
    EmptyMeasurementRationale,
    EmptyUnit,
    NonFiniteValue,
    NegativeValue,
    NonFiniteUncertainty,
    NegativeUncertainty,
    NonFiniteDerivedValue,
    NegativeRequiredCapacity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceCapacityNetworkError {
    InvalidInput(Vec<ServiceCapacityNetworkValidationError>),
}

pub(crate) fn build_evidence_index<'a>(
    records: &'a [EvidenceRecord],
) -> Result<BTreeMap<&'a str, &'a EvidenceRecord>, Vec<ServiceCapacityNetworkValidationError>> {
    let mut index = BTreeMap::new();
    let mut errors = Vec::new();
    for record in records {
        if index.insert(record.id.as_str(), record).is_some() {
            errors.push(ServiceCapacityNetworkValidationError::DuplicateEvidenceRecordId);
        }
        if record.validate().is_err() {
            errors.push(ServiceCapacityNetworkValidationError::InvalidEvidenceRecord);
        }
    }
    if errors.is_empty() {
        Ok(index)
    } else {
        Err(errors)
    }
}

/// Evaluate directed source-to-service maximum flow and a deterministic min cut.
///
/// Unknown-availability links are excluded from the conservative graph but
/// included in the optimistic graph. Inactive links are excluded from both.
/// Every edge capacity and service requirement uses the same exact unit string;
/// this function performs no implicit unit conversion.
pub fn assess_service_capacity_network(
    scenario: &ServiceCapacityNetworkScenario,
    evidence_records: &[EvidenceRecord],
) -> Result<ServiceCapacityNetworkAssessment, ServiceCapacityNetworkError> {
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(ServiceCapacityNetworkError::InvalidInput)?;
    assess_service_capacity_network_indexed(scenario, &evidence_by_id)
}

/// Batch API for evaluating many service networks against one evidence corpus.
///
/// The evidence index is constructed once for the batch, scenarios are processed
/// in stable ID order, and duplicate scenario IDs are rejected. This makes the
/// scaling behavior explicit rather than requiring callers to rebuild indexes.
pub fn assess_service_capacity_networks(
    scenarios: &[ServiceCapacityNetworkScenario],
    evidence_records: &[EvidenceRecord],
) -> Result<Vec<ServiceCapacityNetworkAssessment>, ServiceCapacityNetworkError> {
    if scenarios.is_empty() {
        return Err(ServiceCapacityNetworkError::InvalidInput(vec![
            ServiceCapacityNetworkValidationError::EmptyScenarioSet,
        ]));
    }
    let evidence_by_id = build_evidence_index(evidence_records)
        .map_err(ServiceCapacityNetworkError::InvalidInput)?;
    let mut ids = BTreeSet::new();
    for scenario in scenarios {
        if !ids.insert(scenario.id.as_str()) {
            return Err(ServiceCapacityNetworkError::InvalidInput(vec![
                ServiceCapacityNetworkValidationError::DuplicateScenarioId,
            ]));
        }
    }
    let mut ordered: Vec<&ServiceCapacityNetworkScenario> = scenarios.iter().collect();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    ordered
        .into_iter()
        .map(|scenario| assess_service_capacity_network_indexed(scenario, &evidence_by_id))
        .collect()
}

pub(crate) fn assess_service_capacity_network_indexed(
    scenario: &ServiceCapacityNetworkScenario,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
) -> Result<ServiceCapacityNetworkAssessment, ServiceCapacityNetworkError> {
    let mut errors = Vec::new();
    if scenario.schema_version != SERVICE_NETWORK_SCHEMA_VERSION {
        errors.push(ServiceCapacityNetworkValidationError::UnsupportedSchemaVersion);
    }
    if scenario.id.trim().is_empty() {
        errors.push(ServiceCapacityNetworkValidationError::EmptyScenarioId);
    }
    if !is_canonical_utc_timestamp(&scenario.as_of) {
        errors.push(ServiceCapacityNetworkValidationError::InvalidAsOfTimestamp);
    }
    if scenario.service_entity_id.trim().is_empty() {
        errors.push(ServiceCapacityNetworkValidationError::EmptyServiceEntityId);
    }
    if scenario.source_entity_ids.is_empty() {
        errors.push(ServiceCapacityNetworkValidationError::EmptySourceSet);
    }
    if scenario.required_capacity.entity_id != scenario.service_entity_id {
        errors.push(ServiceCapacityNetworkValidationError::RequiredCapacityEntityMismatch);
    }
    if scenario.required_capacity.quantity.value.is_finite()
        && scenario.required_capacity.quantity.value < 0.0
    {
        errors.push(ServiceCapacityNetworkValidationError::NegativeRequiredCapacity);
    }
    validate_measurement(
        &scenario.required_capacity,
        &scenario.as_of,
        evidence_by_id,
        &mut errors,
    );

    let mut ordered_sources = scenario.source_entity_ids.clone();
    ordered_sources.sort();
    let mut source_ids = BTreeSet::new();
    for source in &scenario.source_entity_ids {
        if source.trim().is_empty() {
            errors.push(ServiceCapacityNetworkValidationError::EmptySourceEntityId);
        }
        if !source_ids.insert(source.as_str()) {
            errors.push(ServiceCapacityNetworkValidationError::DuplicateSourceEntityId);
        }
        if source == &scenario.service_entity_id {
            errors.push(ServiceCapacityNetworkValidationError::SourceIsServiceEntity);
        }
    }

    let mut link_ids = BTreeSet::new();
    for link in &scenario.links {
        if link.id.trim().is_empty() {
            errors.push(ServiceCapacityNetworkValidationError::EmptyLinkId);
        }
        if !link_ids.insert(link.id.as_str()) {
            errors.push(ServiceCapacityNetworkValidationError::DuplicateLinkId);
        }
        if link.from_entity_id.trim().is_empty() || link.to_entity_id.trim().is_empty() {
            errors.push(ServiceCapacityNetworkValidationError::EmptyLinkEndpoint);
        }
        if link.from_entity_id == link.to_entity_id {
            errors.push(ServiceCapacityNetworkValidationError::SelfLoopLink);
        }
        if link.availability_rationale.trim().is_empty() {
            errors.push(ServiceCapacityNetworkValidationError::EmptyLinkRationale);
        }
        if link.capacity.entity_id != link.id {
            errors.push(ServiceCapacityNetworkValidationError::LinkCapacityEntityMismatch);
        }
        if link.capacity.quantity.unit != scenario.required_capacity.quantity.unit {
            errors.push(ServiceCapacityNetworkValidationError::UnitMismatch);
        }
        validate_measurement(&link.capacity, &scenario.as_of, evidence_by_id, &mut errors);
        validate_availability_evidence(link, &scenario.as_of, evidence_by_id, &mut errors);
    }
    if !errors.is_empty() {
        return Err(ServiceCapacityNetworkError::InvalidInput(errors));
    }

    let mut all_links = scenario.links.clone();
    all_links.sort_by(|left, right| left.id.cmp(&right.id));
    let mut node_set = BTreeSet::new();
    node_set.insert(scenario.service_entity_id.clone());
    node_set.extend(ordered_sources.iter().cloned());
    for link in &all_links {
        node_set.insert(link.from_entity_id.clone());
        node_set.insert(link.to_entity_id.clone());
    }
    let node_ids: Vec<String> = node_set.into_iter().collect();
    let node_index: BTreeMap<&str, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect();
    let super_source = node_ids.len();
    let sink_index = node_index[scenario.service_entity_id.as_str()];
    let node_count = node_ids.len() + 1;

    let mut lower_edges = Vec::new();
    let mut upper_edges = Vec::new();
    let mut active_link_ids = Vec::new();
    let mut inactive_link_ids = Vec::new();
    let mut unknown_link_ids = Vec::new();
    let mut capacity_evidence_ids = scenario.required_capacity.supporting_evidence_ids.clone();
    let mut availability_evidence_ids = Vec::new();
    let mut total_upper_capacity = 0.0_f64;

    for link in &all_links {
        capacity_evidence_ids.extend(link.capacity.supporting_evidence_ids.iter().cloned());
        availability_evidence_ids.extend(link.availability_supporting_evidence_ids.iter().cloned());

        let value = link.capacity.quantity.value;
        let uncertainty = link.capacity.quantity.uncertainty_abs.unwrap_or(0.0);
        let lower = (value - uncertainty).max(0.0);
        let upper = value + uncertainty;
        if !lower.is_finite() || !upper.is_finite() {
            errors.push(ServiceCapacityNetworkValidationError::NonFiniteDerivedValue);
            continue;
        }
        total_upper_capacity += upper;
        if !total_upper_capacity.is_finite() {
            errors.push(ServiceCapacityNetworkValidationError::NonFiniteDerivedValue);
        }

        let from = node_index[link.from_entity_id.as_str()];
        let to = node_index[link.to_entity_id.as_str()];
        match link.availability {
            NetworkLinkAvailability::Active => {
                active_link_ids.push(link.id.clone());
                lower_edges.push(FlowInputEdge {
                    from,
                    to,
                    capacity: lower,
                    link_id: link.id.clone(),
                });
                upper_edges.push(FlowInputEdge {
                    from,
                    to,
                    capacity: upper,
                    link_id: link.id.clone(),
                });
            }
            NetworkLinkAvailability::Inactive => inactive_link_ids.push(link.id.clone()),
            NetworkLinkAvailability::Unknown => {
                unknown_link_ids.push(link.id.clone());
                upper_edges.push(FlowInputEdge {
                    from,
                    to,
                    capacity: upper,
                    link_id: link.id.clone(),
                });
            }
        }
    }
    if !errors.is_empty() {
        return Err(ServiceCapacityNetworkError::InvalidInput(errors));
    }

    // Virtual-source arcs are larger than all physical link capacities combined,
    // preventing them from being mistaken for a physical minimum cut.
    let virtual_source_capacity =
        (total_upper_capacity + 1.0).max(total_upper_capacity * (1.0 + 4.0 * f64::EPSILON));
    if !virtual_source_capacity.is_finite() || virtual_source_capacity <= total_upper_capacity {
        return Err(ServiceCapacityNetworkError::InvalidInput(vec![
            ServiceCapacityNetworkValidationError::NonFiniteDerivedValue,
        ]));
    }
    for source in &ordered_sources {
        let source_index = node_index[source.as_str()];
        lower_edges.push(FlowInputEdge {
            from: super_source,
            to: source_index,
            capacity: virtual_source_capacity,
            link_id: String::new(),
        });
        upper_edges.push(FlowInputEdge {
            from: super_source,
            to: source_index,
            capacity: virtual_source_capacity,
            link_id: String::new(),
        });
    }

    let lower_flow = max_flow_and_min_cut(node_count, &lower_edges, super_source, sink_index);
    let upper_flow = max_flow_and_min_cut(node_count, &upper_edges, super_source, sink_index);
    let required = &scenario.required_capacity.quantity;
    let demand_lower = (required.value - required.uncertainty_abs.unwrap_or(0.0)).max(0.0);
    let demand_upper = required.value + required.uncertainty_abs.unwrap_or(0.0);
    if !demand_lower.is_finite()
        || !demand_upper.is_finite()
        || !lower_flow.flow.is_finite()
        || !upper_flow.flow.is_finite()
    {
        return Err(ServiceCapacityNetworkError::InvalidInput(vec![
            ServiceCapacityNetworkValidationError::NonFiniteDerivedValue,
        ]));
    }

    let state = if scenario.topology_completeness != NetworkTopologyCompleteness::Complete {
        // A partial/unknown topology can omit a shared bottleneck or an alternate
        // route. Its max-flow result is only for the declared subgraph, not a
        // defensible bound on the actual service.
        NetworkAssessmentState::Indeterminate
    } else if lower_flow.flow >= demand_upper {
        NetworkAssessmentState::Adequate
    } else if upper_flow.flow == 0.0 && demand_lower > 0.0 {
        NetworkAssessmentState::Unavailable
    } else if lower_flow.flow > 0.0 && upper_flow.flow < demand_lower {
        NetworkAssessmentState::Degraded
    } else {
        NetworkAssessmentState::Indeterminate
    };
    let (coverage_lower_bound, coverage_upper_bound) =
        if scenario.topology_completeness == NetworkTopologyCompleteness::Complete {
            (
                (demand_upper > 0.0).then(|| lower_flow.flow / demand_upper),
                (demand_lower > 0.0).then(|| upper_flow.flow / demand_lower),
            )
        } else {
            (None, None)
        };
    if coverage_lower_bound.is_some_and(|value| !value.is_finite())
        || coverage_upper_bound.is_some_and(|value| !value.is_finite())
    {
        return Err(ServiceCapacityNetworkError::InvalidInput(vec![
            ServiceCapacityNetworkValidationError::NonFiniteDerivedValue,
        ]));
    }

    active_link_ids.sort();
    inactive_link_ids.sort();
    unknown_link_ids.sort();
    capacity_evidence_ids.sort();
    capacity_evidence_ids.dedup();
    availability_evidence_ids.sort();
    availability_evidence_ids.dedup();

    Ok(ServiceCapacityNetworkAssessment {
        schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
        scenario_id: scenario.id.clone(),
        service_entity_id: scenario.service_entity_id.clone(),
        as_of: scenario.as_of.clone(),
        topology_completeness: scenario.topology_completeness,
        state,
        unit: required.unit.clone(),
        required_capacity_value: required.value,
        required_capacity_lower_bound: demand_lower,
        required_capacity_upper_bound: demand_upper,
        conservative_available_capacity: lower_flow.flow,
        optimistic_available_capacity: upper_flow.flow,
        coverage_lower_bound,
        coverage_upper_bound,
        conservative_min_cut_link_ids: lower_flow.min_cut_link_ids,
        active_link_ids,
        inactive_link_ids,
        unknown_link_ids,
        capacity_evidence_ids,
        availability_evidence_ids,
    })
}

fn validate_measurement(
    measurement: &FunctionMeasurement,
    as_of: &str,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
    errors: &mut Vec<ServiceCapacityNetworkValidationError>,
) {
    if measurement.entity_id.trim().is_empty() {
        errors.push(ServiceCapacityNetworkValidationError::EmptyMeasurementEntityId);
    }
    if measurement.rationale.trim().is_empty() {
        errors.push(ServiceCapacityNetworkValidationError::EmptyMeasurementRationale);
    }
    let quantity = &measurement.quantity;
    if quantity.unit.trim().is_empty() {
        errors.push(ServiceCapacityNetworkValidationError::EmptyUnit);
    }
    if !quantity.value.is_finite() {
        errors.push(ServiceCapacityNetworkValidationError::NonFiniteValue);
    } else if quantity.value < 0.0 {
        errors.push(ServiceCapacityNetworkValidationError::NegativeValue);
    }
    if let Some(uncertainty) = quantity.uncertainty_abs {
        if !uncertainty.is_finite() {
            errors.push(ServiceCapacityNetworkValidationError::NonFiniteUncertainty);
        } else if uncertainty < 0.0 {
            errors.push(ServiceCapacityNetworkValidationError::NegativeUncertainty);
        }
    }

    let mut seen = BTreeSet::new();
    for evidence_id in &measurement.supporting_evidence_ids {
        if evidence_id.trim().is_empty() {
            errors.push(ServiceCapacityNetworkValidationError::EmptyEvidenceId);
        }
        if !seen.insert(evidence_id.as_str()) {
            errors.push(ServiceCapacityNetworkValidationError::DuplicateSupportingEvidenceId);
        }
    }

    match measurement.basis {
        ScenarioBasis::Assumed => {
            if !measurement.supporting_evidence_ids.is_empty() {
                errors.push(
                    ServiceCapacityNetworkValidationError::EvidenceForbiddenForAssumedMeasurement,
                );
            }
        }
        ScenarioBasis::Observed => {
            if measurement.supporting_evidence_ids.is_empty() {
                errors.push(
                    ServiceCapacityNetworkValidationError::MissingEvidenceForObservedMeasurement,
                );
            }
            for id in &measurement.supporting_evidence_ids {
                let Some(record) = evidence_by_id.get(id.as_str()) else {
                    errors.push(ServiceCapacityNetworkValidationError::UnknownEvidenceReference);
                    continue;
                };
                validate_observation_record(
                    record,
                    &measurement.entity_id,
                    Some(quantity),
                    as_of,
                    errors,
                );
            }
        }
    }
}

fn validate_availability_evidence(
    link: &NetworkLink,
    as_of: &str,
    evidence_by_id: &BTreeMap<&str, &EvidenceRecord>,
    errors: &mut Vec<ServiceCapacityNetworkValidationError>,
) {
    let mut seen = BTreeSet::new();
    for id in &link.availability_supporting_evidence_ids {
        if id.trim().is_empty() {
            errors.push(ServiceCapacityNetworkValidationError::EmptyEvidenceId);
        }
        if !seen.insert(id.as_str()) {
            errors.push(ServiceCapacityNetworkValidationError::DuplicateSupportingEvidenceId);
        }
    }
    match link.availability_basis {
        ScenarioBasis::Assumed => {
            if !link.availability_supporting_evidence_ids.is_empty() {
                errors.push(
                    ServiceCapacityNetworkValidationError::EvidenceForbiddenForAssumedAvailability,
                );
            }
        }
        ScenarioBasis::Observed => {
            if link.availability_supporting_evidence_ids.is_empty() {
                errors.push(
                    ServiceCapacityNetworkValidationError::MissingEvidenceForObservedAvailability,
                );
            }
            for id in &link.availability_supporting_evidence_ids {
                let Some(record) = evidence_by_id.get(id.as_str()) else {
                    errors.push(ServiceCapacityNetworkValidationError::UnknownEvidenceReference);
                    continue;
                };
                validate_observation_record(record, &link.id, None, as_of, errors);
            }
        }
    }
}

fn validate_observation_record(
    record: &EvidenceRecord,
    entity_id: &str,
    quantity: Option<&Quantity>,
    as_of: &str,
    errors: &mut Vec<ServiceCapacityNetworkValidationError>,
) {
    if !matches!(
        record.state,
        EvidenceState::Asserted | EvidenceState::Corroborated
    ) {
        errors.push(ServiceCapacityNetworkValidationError::InadmissibleSupportingEvidence);
    }
    if record.kind != EvidenceKind::Observation {
        errors.push(ServiceCapacityNetworkValidationError::SupportingEvidenceWrongKind);
    }
    if record.subject_id != entity_id || record.spatial.entity_id != entity_id {
        errors.push(ServiceCapacityNetworkValidationError::SupportingEvidenceSubjectMismatch);
    }
    if let Some(expected) = quantity {
        match &record.quantity {
            None => errors
                .push(ServiceCapacityNetworkValidationError::SupportingEvidenceMissingQuantity),
            Some(actual) => {
                if actual.value != expected.value
                    || actual.unit != expected.unit
                    || actual.uncertainty_abs != expected.uncertainty_abs
                {
                    errors.push(
                        ServiceCapacityNetworkValidationError::SupportingEvidenceQuantityMismatch,
                    );
                }
            }
        }
    }
    if !is_canonical_utc_timestamp(&record.time.occurred_at)
        || !is_canonical_utc_timestamp(&record.time.recorded_at)
    {
        errors.push(ServiceCapacityNetworkValidationError::InvalidSupportingEvidenceTimestamp);
        return;
    }
    if record.time.occurred_at.as_str() > as_of || record.time.recorded_at.as_str() > as_of {
        errors.push(ServiceCapacityNetworkValidationError::EvidenceAfterAsOf);
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
        errors.push(ServiceCapacityNetworkValidationError::InvalidSupportingEvidenceValidity);
    } else if start.is_some_and(|value| value > as_of) || end.is_some_and(|value| as_of >= value) {
        errors.push(ServiceCapacityNetworkValidationError::OutsideSupportingEvidenceValidity);
    }
}

#[derive(Debug, Clone)]
struct FlowInputEdge {
    from: usize,
    to: usize,
    capacity: f64,
    link_id: String,
}

#[derive(Debug, Clone)]
struct ResidualEdge {
    to: usize,
    reverse_index: usize,
    residual_capacity: f64,
    original_capacity: f64,
    link_id: String,
}

#[derive(Debug, Clone)]
struct FlowResult {
    flow: f64,
    min_cut_link_ids: Vec<String>,
}

fn max_flow_and_min_cut(
    node_count: usize,
    input_edges: &[FlowInputEdge],
    source: usize,
    sink: usize,
) -> FlowResult {
    let mut graph: Vec<Vec<ResidualEdge>> = vec![Vec::new(); node_count];
    let mut forward_positions = Vec::new();

    for edge in input_edges {
        if edge.capacity <= 0.0 {
            continue;
        }
        let forward_index = graph[edge.from].len();
        let reverse_index = graph[edge.to].len();
        graph[edge.from].push(ResidualEdge {
            to: edge.to,
            reverse_index,
            residual_capacity: edge.capacity,
            original_capacity: edge.capacity,
            link_id: edge.link_id.clone(),
        });
        graph[edge.to].push(ResidualEdge {
            to: edge.from,
            reverse_index: forward_index,
            residual_capacity: 0.0,
            original_capacity: 0.0,
            link_id: String::new(),
        });
        forward_positions.push((edge.from, forward_index));
    }

    let mut total_flow = 0.0_f64;
    loop {
        let mut level = vec![-1_i32; node_count];
        level[source] = 0;
        let mut queue = VecDeque::new();
        queue.push_back(source);
        while let Some(node) = queue.pop_front() {
            for edge in &graph[node] {
                if edge.residual_capacity > 0.0 && level[edge.to] < 0 {
                    level[edge.to] = level[node] + 1;
                    queue.push_back(edge.to);
                }
            }
        }
        if level[sink] < 0 {
            break;
        }

        let mut next_edge = vec![0_usize; node_count];
        loop {
            let pushed = send_blocking_flow(source, sink, &level, &mut next_edge, &mut graph);
            if pushed <= 0.0 {
                break;
            }
            total_flow += pushed;
            if !total_flow.is_finite() {
                return FlowResult {
                    flow: f64::INFINITY,
                    min_cut_link_ids: Vec::new(),
                };
            }
        }
    }

    let mut reachable = vec![false; node_count];
    reachable[source] = true;
    let mut queue = VecDeque::new();
    queue.push_back(source);
    while let Some(node) = queue.pop_front() {
        for edge in &graph[node] {
            if edge.residual_capacity > 0.0 && !reachable[edge.to] {
                reachable[edge.to] = true;
                queue.push_back(edge.to);
            }
        }
    }
    let mut cut = BTreeSet::new();
    for (from, edge_index) in forward_positions {
        let edge = &graph[from][edge_index];
        if !edge.link_id.is_empty()
            && edge.original_capacity > 0.0
            && reachable[from]
            && !reachable[edge.to]
        {
            cut.insert(edge.link_id.clone());
        }
    }
    FlowResult {
        flow: total_flow,
        min_cut_link_ids: cut.into_iter().collect(),
    }
}

/// Iterative level-graph DFS avoids a call-stack limit on long infrastructure chains.
fn send_blocking_flow(
    source: usize,
    sink: usize,
    level: &[i32],
    next_edge: &mut [usize],
    graph: &mut [Vec<ResidualEdge>],
) -> f64 {
    let mut nodes = vec![source];
    let mut path: Vec<(usize, usize)> = Vec::new();

    loop {
        let Some(&node) = nodes.last() else {
            return 0.0;
        };
        if node == sink {
            let pushed = path
                .iter()
                .map(|(from, edge_index)| graph[*from][*edge_index].residual_capacity)
                .fold(f64::MAX, f64::min);
            if pushed <= 0.0 {
                return 0.0;
            }
            for (from, edge_index) in &path {
                let (to, reverse_index) = {
                    let edge = &graph[*from][*edge_index];
                    (edge.to, edge.reverse_index)
                };
                graph[*from][*edge_index].residual_capacity -= pushed;
                if graph[*from][*edge_index].residual_capacity < 0.0 {
                    graph[*from][*edge_index].residual_capacity = 0.0;
                }
                graph[to][reverse_index].residual_capacity += pushed;
            }
            return pushed;
        }

        while next_edge[node] < graph[node].len() {
            let edge_index = next_edge[node];
            let edge = &graph[node][edge_index];
            if edge.residual_capacity > 0.0 && level[edge.to] == level[node] + 1 {
                break;
            }
            next_edge[node] += 1;
        }

        if next_edge[node] >= graph[node].len() {
            nodes.pop();
            if let Some((parent, _edge_index)) = path.pop() {
                next_edge[parent] += 1;
            } else {
                return 0.0;
            }
        } else {
            let edge_index = next_edge[node];
            let next_node = graph[node][edge_index].to;
            path.push((node, edge_index));
            nodes.push(next_node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_function::FunctionMeasurement;
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, Provenance, SpatialContext, TimeContext,
    };

    const AT: &str = "2026-10-16T10:00:00Z";

    fn measurement(
        entity_id: &str,
        value: f64,
        uncertainty_abs: Option<f64>,
        unit: &str,
    ) -> FunctionMeasurement {
        FunctionMeasurement {
            entity_id: entity_id.into(),
            quantity: Quantity {
                value,
                unit: unit.into(),
                uncertainty_abs,
            },
            basis: ScenarioBasis::Assumed,
            rationale: "Synthetic explicit capacity assumption".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn link(
        id: &str,
        from: &str,
        to: &str,
        capacity: f64,
        uncertainty: Option<f64>,
        availability: NetworkLinkAvailability,
    ) -> NetworkLink {
        NetworkLink {
            id: id.into(),
            from_entity_id: from.into(),
            to_entity_id: to.into(),
            capacity: measurement(id, capacity, uncertainty, "m3/h"),
            availability,
            availability_basis: ScenarioBasis::Assumed,
            availability_rationale: "Synthetic scenario assumption".into(),
            availability_supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario(
        links: Vec<NetworkLink>,
        required: f64,
        uncertainty: Option<f64>,
    ) -> ServiceCapacityNetworkScenario {
        ServiceCapacityNetworkScenario {
            schema_version: SERVICE_NETWORK_SCHEMA_VERSION.into(),
            id: "scenario:network-test".into(),
            as_of: AT.into(),
            service_entity_id: "service:irrigation".into(),
            topology_completeness: NetworkTopologyCompleteness::Complete,
            source_entity_ids: vec!["source:reservoir".into()],
            required_capacity: measurement("service:irrigation", required, uncertainty, "m3/h"),
            links,
        }
    }

    #[test]
    fn shared_trunk_bottleneck_caps_sum_of_parallel_routes() {
        let result = assess_service_capacity_network(
            &scenario(
                vec![
                    link(
                        "supply",
                        "source:reservoir",
                        "junction",
                        5.0,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                    link(
                        "route-a",
                        "junction",
                        "service:irrigation",
                        4.0,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                    link(
                        "route-b",
                        "junction",
                        "service:irrigation",
                        4.0,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                ],
                8.0,
                None,
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.conservative_available_capacity, 5.0);
        assert_eq!(result.optimistic_available_capacity, 5.0);
        assert_eq!(result.state, NetworkAssessmentState::Degraded);
        assert_eq!(result.conservative_min_cut_link_ids, vec!["supply"]);
    }

    #[test]
    fn parallel_routes_increase_flow_until_the_shared_trunk_binds() {
        let result = assess_service_capacity_network(
            &scenario(
                vec![
                    link(
                        "supply",
                        "source:reservoir",
                        "junction",
                        10.0,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                    link(
                        "route-a",
                        "junction",
                        "service:irrigation",
                        4.0,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                    link(
                        "route-b",
                        "junction",
                        "service:irrigation",
                        3.0,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                ],
                7.0,
                None,
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.conservative_available_capacity, 7.0);
        assert_eq!(result.state, NetworkAssessmentState::Adequate);
        assert_eq!(
            result.conservative_min_cut_link_ids,
            vec!["route-a", "route-b"]
        );
    }

    #[test]
    fn unknown_link_yields_interval_instead_of_false_outage() {
        let result = assess_service_capacity_network(
            &scenario(
                vec![
                    link(
                        "known",
                        "source:reservoir",
                        "junction",
                        8.0,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                    link(
                        "uncertain",
                        "junction",
                        "service:irrigation",
                        8.0,
                        None,
                        NetworkLinkAvailability::Unknown,
                    ),
                ],
                6.0,
                None,
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.conservative_available_capacity, 0.0);
        assert_eq!(result.optimistic_available_capacity, 8.0);
        assert_eq!(result.state, NetworkAssessmentState::Indeterminate);
        assert_eq!(result.unknown_link_ids, vec!["uncertain"]);
    }

    #[test]
    fn inactive_link_is_excluded_from_both_bounds() {
        let result = assess_service_capacity_network(
            &scenario(
                vec![link(
                    "closed",
                    "source:reservoir",
                    "service:irrigation",
                    12.0,
                    None,
                    NetworkLinkAvailability::Inactive,
                )],
                1.0,
                None,
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.conservative_available_capacity, 0.0);
        assert_eq!(result.optimistic_available_capacity, 0.0);
        assert_eq!(result.state, NetworkAssessmentState::Unavailable);
    }

    #[test]
    fn uncertain_capacity_must_cover_upper_demand_to_claim_adequacy() {
        let result = assess_service_capacity_network(
            &scenario(
                vec![link(
                    "edge",
                    "source:reservoir",
                    "service:irrigation",
                    10.0,
                    Some(1.0),
                    NetworkLinkAvailability::Active,
                )],
                10.0,
                Some(1.0),
            ),
            &[],
        )
        .unwrap();
        assert_eq!(result.conservative_available_capacity, 9.0);
        assert_eq!(result.required_capacity_upper_bound, 11.0);
        assert_eq!(result.state, NetworkAssessmentState::Indeterminate);
    }

    #[test]
    fn mismatched_units_and_duplicate_link_ids_fail_closed() {
        let mut mismatched = link(
            "edge",
            "source:reservoir",
            "service:irrigation",
            12.0,
            None,
            NetworkLinkAvailability::Active,
        );
        mismatched.capacity.quantity.unit = "L/s".into();
        assert!(matches!(
            assess_service_capacity_network(&scenario(vec![mismatched], 10.0, None), &[]),
            Err(ServiceCapacityNetworkError::InvalidInput(errors))
                if errors.contains(&ServiceCapacityNetworkValidationError::UnitMismatch)
        ));

        let duplicate = link(
            "edge",
            "source:reservoir",
            "service:irrigation",
            12.0,
            None,
            NetworkLinkAvailability::Active,
        );
        assert!(matches!(
            assess_service_capacity_network(&scenario(vec![duplicate.clone(), duplicate], 10.0, None), &[]),
            Err(ServiceCapacityNetworkError::InvalidInput(errors))
                if errors.contains(&ServiceCapacityNetworkValidationError::DuplicateLinkId)
        ));
    }

    #[test]
    fn incomplete_topology_never_becomes_adequate_or_unavailable() {
        let mut partial = scenario(vec![], 1.0, None);
        partial.topology_completeness = NetworkTopologyCompleteness::Partial;
        let partial_result = assess_service_capacity_network(&partial, &[]).unwrap();
        assert_eq!(partial_result.state, NetworkAssessmentState::Indeterminate);
        assert_eq!(partial_result.conservative_available_capacity, 0.0);
        assert_eq!(partial_result.coverage_lower_bound, None);
        assert_eq!(partial_result.coverage_upper_bound, None);

        let mut unknown = scenario(vec![], 1.0, None);
        unknown.topology_completeness = NetworkTopologyCompleteness::Unknown;
        let unknown_result = assess_service_capacity_network(&unknown, &[]).unwrap();
        assert_eq!(unknown_result.state, NetworkAssessmentState::Indeterminate);
    }

    #[test]
    fn malformed_scenario_time_fails_closed() {
        let mut invalid = scenario(vec![], 1.0, None);
        invalid.as_of = "2026-99-99T10:00:00Z".into();
        assert!(matches!(
            assess_service_capacity_network(&invalid, &[]),
            Err(ServiceCapacityNetworkError::InvalidInput(errors))
                if errors.contains(&ServiceCapacityNetworkValidationError::InvalidAsOfTimestamp)
        ));
    }

    #[test]
    fn derived_capacity_overflow_fails_closed() {
        let result = assess_service_capacity_network(
            &scenario(
                vec![
                    link(
                        "huge-a",
                        "source:reservoir",
                        "junction",
                        f64::MAX,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                    link(
                        "huge-b",
                        "junction",
                        "service:irrigation",
                        f64::MAX,
                        None,
                        NetworkLinkAvailability::Active,
                    ),
                ],
                10.0,
                None,
            ),
            &[],
        );
        assert!(matches!(
            result,
            Err(ServiceCapacityNetworkError::InvalidInput(errors))
                if errors.contains(&ServiceCapacityNetworkValidationError::NonFiniteDerivedValue)
        ));
    }

    #[test]
    fn observed_capacity_requires_matching_evidence() {
        let mut edge = link(
            "edge",
            "source:reservoir",
            "service:irrigation",
            7.0,
            None,
            NetworkLinkAvailability::Active,
        );
        edge.capacity.basis = ScenarioBasis::Observed;
        edge.capacity.supporting_evidence_ids = vec!["ev:edge".into()];
        let records = [evidence(
            "ev:edge",
            "edge",
            Quantity {
                value: 8.0,
                unit: "m3/h".into(),
                uncertainty_abs: None,
            },
            EvidenceState::Asserted,
            AT,
        )];
        assert!(matches!(
            assess_service_capacity_network(&scenario(vec![edge], 10.0, None), &records),
            Err(ServiceCapacityNetworkError::InvalidInput(errors))
                if errors.contains(&ServiceCapacityNetworkValidationError::SupportingEvidenceQuantityMismatch)
        ));
    }

    #[test]
    fn batch_network_assessment_sorts_scenario_ids() {
        let mut first = scenario(
            vec![link(
                "first-edge",
                "source:reservoir",
                "service:irrigation",
                4.0,
                None,
                NetworkLinkAvailability::Active,
            )],
            10.0,
            None,
        );
        first.id = "scenario:b".into();
        let mut second = scenario(
            vec![link(
                "second-edge",
                "source:reservoir",
                "service:irrigation",
                12.0,
                None,
                NetworkLinkAvailability::Active,
            )],
            10.0,
            None,
        );
        second.id = "scenario:a".into();
        let result = assess_service_capacity_networks(&[first, second], &[]).unwrap();
        assert_eq!(
            result
                .iter()
                .map(|item| item.scenario_id.as_str())
                .collect::<Vec<_>>(),
            vec!["scenario:a", "scenario:b"]
        );
        assert_eq!(result[0].state, NetworkAssessmentState::Adequate);
        assert_eq!(result[1].state, NetworkAssessmentState::Degraded);
    }

    #[test]
    fn batch_network_assessment_rejects_empty_or_duplicate_scenario_ids() {
        assert!(matches!(
            assess_service_capacity_networks(&[], &[]),
            Err(ServiceCapacityNetworkError::InvalidInput(errors))
                if errors.contains(&ServiceCapacityNetworkValidationError::EmptyScenarioSet)
        ));
        let one = scenario(vec![], 1.0, None);
        assert!(matches!(
            assess_service_capacity_networks(&[one.clone(), one], &[]),
            Err(ServiceCapacityNetworkError::InvalidInput(errors))
                if errors.contains(&ServiceCapacityNetworkValidationError::DuplicateScenarioId)
        ));
    }

    #[test]
    fn complete_network_fixture_matches_golden_max_flow_and_cut() {
        let input: ServiceCapacityNetworkScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/service-capacity-network-scenario.json"
        ))
        .unwrap();
        let expected: ServiceCapacityNetworkAssessment = serde_json::from_str(include_str!(
            "../../schemas/examples/service-capacity-network-assessment.json"
        ))
        .unwrap();
        let actual = assess_service_capacity_network(&input, &[]).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.conservative_min_cut_link_ids, vec!["supply"]);
        assert_eq!(actual.conservative_available_capacity, 5.0);
        assert_eq!(actual.optimistic_available_capacity, 5.0);
    }

    fn brute_force_min_cut_capacity(
        scenario: &ServiceCapacityNetworkScenario,
        optimistic: bool,
    ) -> f64 {
        let mut node_set = BTreeSet::new();
        node_set.insert(scenario.service_entity_id.clone());
        node_set.extend(scenario.source_entity_ids.iter().cloned());
        for edge in &scenario.links {
            node_set.insert(edge.from_entity_id.clone());
            node_set.insert(edge.to_entity_id.clone());
        }
        let node_ids: Vec<String> = node_set.into_iter().collect();
        assert!(node_ids.len() < usize::BITS as usize - 1);
        let node_index: BTreeMap<&str, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.as_str(), index))
            .collect();
        let sink_index = node_index[scenario.service_entity_id.as_str()];
        let total_upper: f64 = scenario
            .links
            .iter()
            .map(|edge| {
                edge.capacity.quantity.value + edge.capacity.quantity.uncertainty_abs.unwrap_or(0.0)
            })
            .sum();
        let virtual_capacity = total_upper + 1.0;
        let mut best = f64::INFINITY;

        for mask in 0_usize..(1_usize << node_ids.len()) {
            if mask & (1_usize << sink_index) != 0 {
                continue;
            }
            let on_source_side = |id: &str| mask & (1_usize << node_index[id]) != 0;
            let mut cut_capacity = 0.0;
            for source in &scenario.source_entity_ids {
                if !on_source_side(source) {
                    cut_capacity += virtual_capacity;
                }
            }
            for edge in &scenario.links {
                let included = edge.availability == NetworkLinkAvailability::Active
                    || (optimistic && edge.availability == NetworkLinkAvailability::Unknown);
                if !included {
                    continue;
                }
                let value = edge.capacity.quantity.value;
                let uncertainty = edge.capacity.quantity.uncertainty_abs.unwrap_or(0.0);
                let capacity = if optimistic {
                    value + uncertainty
                } else {
                    (value - uncertainty).max(0.0)
                };
                if on_source_side(&edge.from_entity_id) && !on_source_side(&edge.to_entity_id) {
                    cut_capacity += capacity;
                }
            }
            best = best.min(cut_capacity);
        }
        best
    }

    #[test]
    fn long_chain_does_not_require_recursive_call_stack() {
        let edge_count = 10_000;
        let mut links = Vec::with_capacity(edge_count);
        for index in 0..edge_count {
            let from = if index == 0 {
                "source:reservoir".to_owned()
            } else {
                format!("chain-node-{:05}", index - 1)
            };
            let to = if index + 1 == edge_count {
                "service:irrigation".to_owned()
            } else {
                format!("chain-node-{index:05}")
            };
            let id = format!("chain-link-{index:05}");
            links.push(link(
                &id,
                &from,
                &to,
                2.0,
                None,
                NetworkLinkAvailability::Active,
            ));
        }
        let result = assess_service_capacity_network(&scenario(links, 1.0, None), &[]).unwrap();
        assert_eq!(result.conservative_available_capacity, 2.0);
        assert_eq!(result.optimistic_available_capacity, 2.0);
        assert_eq!(result.state, NetworkAssessmentState::Adequate);
    }

    #[test]
    fn source_and_link_input_order_does_not_change_the_assessment() {
        let mut input = scenario(
            vec![
                link(
                    "z-route",
                    "source:other-reservoir",
                    "service:irrigation",
                    3.0,
                    None,
                    NetworkLinkAvailability::Active,
                ),
                link(
                    "a-route",
                    "source:reservoir",
                    "service:irrigation",
                    4.0,
                    None,
                    NetworkLinkAvailability::Active,
                ),
            ],
            10.0,
            None,
        );
        input
            .source_entity_ids
            .push("source:other-reservoir".into());
        let baseline = assess_service_capacity_network(&input, &[]).unwrap();

        input.source_entity_ids.reverse();
        input.links.reverse();
        let reordered = assess_service_capacity_network(&input, &[]).unwrap();
        assert_eq!(baseline, reordered);
    }

    #[test]
    fn max_flow_matches_independent_exhaustive_min_cut_oracle_on_small_networks() {
        let nodes = [
            "source:reservoir",
            "node:a",
            "node:b",
            "node:c",
            "service:irrigation",
        ];
        let arcs = [
            (0, 1),
            (0, 2),
            (1, 2),
            (2, 1),
            (1, 3),
            (2, 3),
            (3, 4),
            (1, 4),
            (2, 4),
            (3, 1),
            (0, 3),
            (2, 4),
        ];
        for seed in 0..64_usize {
            let mut links = Vec::new();
            for (index, (from, to)) in arcs.iter().copied().enumerate() {
                let value = ((seed + index * 3) % 7 + 1) as f64;
                let availability = match (seed + index) % 3 {
                    0 => NetworkLinkAvailability::Active,
                    1 => NetworkLinkAvailability::Inactive,
                    _ => NetworkLinkAvailability::Unknown,
                };
                links.push(link(
                    &format!("edge-{index:02}"),
                    nodes[from],
                    nodes[to],
                    value,
                    Some(0.5),
                    availability,
                ));
            }
            let input = scenario(links, 10.0, None);
            let expected_lower = brute_force_min_cut_capacity(&input, false);
            let expected_upper = brute_force_min_cut_capacity(&input, true);
            let actual = assess_service_capacity_network(&input, &[]).unwrap();
            assert!(
                (actual.conservative_available_capacity - expected_lower).abs() < 1.0e-9,
                "conservative max flow differs from exhaustive min cut on seed {seed}"
            );
            assert!(
                (actual.optimistic_available_capacity - expected_upper).abs() < 1.0e-9,
                "optimistic max flow differs from exhaustive min cut on seed {seed}"
            );
            assert!(
                actual.optimistic_available_capacity >= actual.conservative_available_capacity,
                "optimistic capacity must not fall below conservative capacity on seed {seed}"
            );

            let link_by_id: BTreeMap<&str, &NetworkLink> = input
                .links
                .iter()
                .map(|edge| (edge.id.as_str(), edge))
                .collect();
            let cut_capacity: f64 = actual
                .conservative_min_cut_link_ids
                .iter()
                .map(|id| {
                    let edge = link_by_id[id.as_str()];
                    (edge.capacity.quantity.value
                        - edge.capacity.quantity.uncertainty_abs.unwrap_or(0.0))
                    .max(0.0)
                })
                .sum();
            assert!(
                (cut_capacity - actual.conservative_available_capacity).abs() < 1.0e-9,
                "reported minimum cut does not certify flow on seed {seed}"
            );
        }
    }

    #[test]
    fn service_network_json_fixture_matches_golden_max_flow_and_cut() {
        let input: ServiceCapacityNetworkScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/service-capacity-network-scenario.json"
        ))
        .unwrap();
        let expected: ServiceCapacityNetworkAssessment = serde_json::from_str(include_str!(
            "../../schemas/examples/service-capacity-network-assessment.json"
        ))
        .unwrap();
        let actual = assess_service_capacity_network(&input, &[]).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.conservative_min_cut_link_ids, vec!["supply"]);
        assert_eq!(actual.conservative_available_capacity, 5.0);
        assert_eq!(actual.optimistic_available_capacity, 5.0);
    }

    #[test]
    fn same_input_is_deterministic() {
        let scenario = scenario(
            vec![
                link(
                    "z-route",
                    "source:reservoir",
                    "service:irrigation",
                    3.0,
                    None,
                    NetworkLinkAvailability::Active,
                ),
                link(
                    "a-route",
                    "source:reservoir",
                    "service:irrigation",
                    4.0,
                    None,
                    NetworkLinkAvailability::Active,
                ),
            ],
            10.0,
            None,
        );
        let first = assess_service_capacity_network(&scenario, &[]).unwrap();
        let second = assess_service_capacity_network(&scenario, &[]).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );
        assert_eq!(first.conservative_available_capacity, 7.0);
        assert_eq!(
            first.conservative_min_cut_link_ids,
            vec!["a-route", "z-route"]
        );
    }

    fn evidence(
        id: &str,
        entity_id: &str,
        quantity: Quantity,
        state: EvidenceState,
        recorded_at: &str,
    ) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state,
            issuer_id: "did:example:service-network-tests".into(),
            subject_id: entity_id.into(),
            spatial: SpatialContext {
                entity_id: entity_id.into(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: AT.into(),
                recorded_at: recorded_at.into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic capacity measurement".into(),
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
}
