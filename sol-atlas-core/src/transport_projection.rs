// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Versioned, renderer-neutral transport projection types.
//!
//! This module is a read-only view contract. It is not a transport optimizer,
//! shipment event authority, route database, reservation service, or live API.
//! Physical movement, shipment milestones, communication contacts, evidence
//! relations, and infrastructure dependencies are represented by distinct types
//! so a consumer cannot accidentally interpret one graph's edge as another's.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const TRANSPORT_PROJECTION_SCHEMA_V1: &str = "luminous.sol-atlas.transport-projection.v1";
pub const SYNTHETIC_RELATIVE_TIME_PROFILE_V1: &str = "SYNTHETIC_RELATIVE_SECONDS_V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceKindV1 {
    Observed,
    Curated,
    Derived,
    Scenario,
}

/// Qualification is a projection-readiness label, not a truth score.
/// AcceptedForProjection does not authenticate an issuer or prove physical truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceQualificationV1 {
    AcceptedForProjection,
    ScenarioOnly,
    Unknown,
    Stale,
    Conflicted,
    Unavailable,
    Unqualified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportEvidenceV1 {
    pub id: String,
    /// Optional ID of the projected object to which this claim pertains.
    pub claim_about_id: Option<String>,
    pub kind: EvidenceKindV1,
    pub qualification: EvidenceQualificationV1,
    /// Stable source locator or an explicit synthetic fixture reference.
    pub source_ref: String,
    pub source_profile_id: String,
    pub source_version: Option<String>,
    /// Optional claimed SHA-256 of the exact source record bytes. This module
    /// validates its shape only; it does not fetch bytes or recompute the hash.
    pub content_sha256: Option<String>,
    /// Kept separate from observation time and repository bookkeeping time.
    pub source_event_time_epoch_seconds: Option<u64>,
    pub observed_at_epoch_seconds: Option<u64>,
    pub repository_recorded_at_epoch_seconds: Option<u64>,
    pub valid_from_epoch_seconds: Option<u64>,
    pub valid_until_epoch_seconds: Option<u64>,
    /// Every populated timestamp in this projection uses this explicit profile.
    pub time_profile_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeoPointMicrodegreesV1 {
    pub latitude_microdegrees: i32,
    pub longitude_microdegrees: i32,
}

impl GeoPointMicrodegreesV1 {
    pub fn is_valid(self) -> bool {
        (-90_000_000..=90_000_000).contains(&self.latitude_microdegrees)
            && (-180_000_000..=180_000_000).contains(&self.longitude_microdegrees)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PhysicalNodeKindV1 {
    Producer,
    Warehouse,
    Community,
    IntermodalTerminal,
    RailTerminal,
    Port,
    CanalTerminal,
    CanalLock,
    Airport,
    DeliveryPoint,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalNodeV1 {
    pub id: String,
    pub label: String,
    pub kind: PhysicalNodeKindV1,
    /// Missing geography stays missing; the renderer must not infer a point.
    pub geography: Option<GeoPointMicrodegreesV1>,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TransportModeV1 {
    Road,
    Rail,
    InlandWaterway,
    Maritime,
    Air,
    Walking,
    Cycling,
    Pipeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalOperationV1 {
    Movement { mode: TransportModeV1 },
    Transfer {
        from_mode: TransportModeV1,
        to_mode: TransportModeV1,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkSegmentStateV1 {
    ReportedOpen,
    ReportedRestricted,
    ReportedClosed,
    Proposed,
    AssumedForScenario,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CapacityUnitV1 {
    Passengers,
    Vehicles,
    Tonnes,
    Teu,
    Pallets,
    Shipments,
    Slots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapacityBasisV1 {
    Concurrent,
    PerHour,
    PerShift,
    PerDay,
    Total,
    Unspecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityValueV1 {
    pub amount: u64,
    pub unit: CapacityUnitV1,
    /// Never infer a rate period or concurrent-use meaning from the quantity unit.
    pub basis: CapacityBasisV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalSegmentV1 {
    pub id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub operation: PhysicalOperationV1,
    pub state: NetworkSegmentStateV1,
    pub distance_meters: Option<u64>,
    pub expected_duration_seconds: Option<u64>,
    /// A sourced or scenario capacity claim, not a live availability/reservation.
    pub capacity: Option<CapacityValueV1>,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ShipmentMilestoneKindV1 {
    Planned,
    ForecastArrival,
    ReservationProposed,
    ReservationAccepted,
    Dispatched,
    ArrivalReported,
    DeliveryConfirmed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventTimeMeaningV1 {
    Planned,
    Forecast,
    Occurred,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipmentMilestoneV1 {
    pub id: String,
    pub shipment_ref: String,
    pub leg_id: Option<String>,
    pub milestone: ShipmentMilestoneKindV1,
    pub time_meaning: EventTimeMeaningV1,
    pub event_time_epoch_seconds: Option<u64>,
    /// Repository record time is bookkeeping, not the event's occurrence time.
    pub repository_recorded_at_epoch_seconds: Option<u64>,
    pub time_profile_id: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommunicationEndpointKindV1 {
    OperatorControl,
    DispatchCenter,
    Vehicle,
    TerminalGateway,
    Relay,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunicationEndpointV1 {
    pub id: String,
    pub kind: CommunicationEndpointKindV1,
    pub label: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommunicationDirectionV1 {
    Directed,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommunicationContactStateV1 {
    Scheduled,
    Predicted,
    Opportunistic,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunicationContactV1 {
    pub id: String,
    pub from_endpoint_id: String,
    pub to_endpoint_id: String,
    pub direction: CommunicationDirectionV1,
    pub state: CommunicationContactStateV1,
    pub start_epoch_seconds: u64,
    pub end_epoch_seconds: u64,
    pub time_profile_id: String,
    pub capacity_bytes: Option<u64>,
    pub latency_milliseconds: Option<u64>,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceRelationKindV1 {
    Supports,
    Conflicts,
    Supersedes,
    DerivedFrom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRelationV1 {
    pub id: String,
    pub from_evidence_id: String,
    pub to_evidence_id: String,
    pub relation: EvidenceRelationKindV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InfrastructureAssetKindV1 {
    Power,
    Maintenance,
    Storage,
    Communications,
    Repair,
    Water,
    Workforce,
    HandlingEquipment,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfrastructureAssetV1 {
    pub id: String,
    pub kind: InfrastructureAssetKindV1,
    pub label: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfrastructureDependencyV1 {
    pub id: String,
    pub dependent_asset_id: String,
    pub prerequisite_asset_id: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportProjectionV1 {
    pub schema_version: String,
    pub projection_id: String,
    pub source_manifest_ref: String,
    /// An optional claimed manifest hash. Shape is checked, bytes are not fetched.
    pub source_manifest_sha256: Option<String>,
    pub generated_at_epoch_seconds: Option<u64>,
    pub time_profile_id: String,
    pub evidence_records: Vec<TransportEvidenceV1>,
    pub physical_nodes: Vec<PhysicalNodeV1>,
    pub physical_segments: Vec<PhysicalSegmentV1>,
    pub shipment_milestones: Vec<ShipmentMilestoneV1>,
    pub communication_endpoints: Vec<CommunicationEndpointV1>,
    pub communication_contacts: Vec<CommunicationContactV1>,
    pub evidence_relations: Vec<EvidenceRelationV1>,
    pub infrastructure_assets: Vec<InfrastructureAssetV1>,
    pub infrastructure_dependencies: Vec<InfrastructureDependencyV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportProjectionError {
    UnsupportedSchemaVersion,
    MissingIdentityOrSourceReference,
    InvalidSha256Shape,
    InvalidEvidenceValidityInterval,
    InvalidGeography,
    DuplicateOrBlankIdentifier,
    MissingEvidenceReference,
    UnknownEvidenceReference,
    UnknownPhysicalNode,
    UnknownPhysicalSegment,
    UnknownCommunicationEndpoint,
    UnknownInfrastructureAsset,
    InvalidCapacity,
    InvalidContactWindow,
    InvalidTimeProfile,
    InvalidMilestoneTimeMeaning,
    SelfReferentialDependency,
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn is_sha256_shape(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn unique_nonempty_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    ids.into_iter()
        .all(|id| nonempty(id) && seen.insert(id))
}

fn validate_evidence_refs(
    refs: &[String],
    known: &BTreeSet<&str>,
) -> Result<(), TransportProjectionError> {
    if refs.is_empty() {
        return Err(TransportProjectionError::MissingEvidenceReference);
    }
    if refs.iter().any(|id| !known.contains(id.as_str())) {
        return Err(TransportProjectionError::UnknownEvidenceReference);
    }
    Ok(())
}

/// Validate structural coherence only. It does not authenticate publishers,
/// verify the claimed hashes against source bytes, establish physical truth,
/// resolve conflicts, or qualify data for a live operational decision.
pub fn validate_transport_projection(
    projection: &TransportProjectionV1,
) -> Result<(), TransportProjectionError> {
    if projection.schema_version != TRANSPORT_PROJECTION_SCHEMA_V1 {
        return Err(TransportProjectionError::UnsupportedSchemaVersion);
    }
    if !nonempty(&projection.projection_id)
        || !nonempty(&projection.source_manifest_ref)
        || !nonempty(&projection.time_profile_id)
    {
        return Err(TransportProjectionError::MissingIdentityOrSourceReference);
    }
    if projection
        .source_manifest_sha256
        .as_deref()
        .is_some_and(|digest| !is_sha256_shape(digest))
    {
        return Err(TransportProjectionError::InvalidSha256Shape);
    }

    if !unique_nonempty_ids(projection.evidence_records.iter().map(|item| item.id.as_str()))
        || !unique_nonempty_ids(projection.physical_nodes.iter().map(|item| item.id.as_str()))
        || !unique_nonempty_ids(projection.physical_segments.iter().map(|item| item.id.as_str()))
        || !unique_nonempty_ids(
            projection
                .shipment_milestones
                .iter()
                .map(|item| item.id.as_str()),
        )
        || !unique_nonempty_ids(
            projection
                .communication_endpoints
                .iter()
                .map(|item| item.id.as_str()),
        )
        || !unique_nonempty_ids(
            projection
                .communication_contacts
                .iter()
                .map(|item| item.id.as_str()),
        )
        || !unique_nonempty_ids(
            projection
                .evidence_relations
                .iter()
                .map(|item| item.id.as_str()),
        )
        || !unique_nonempty_ids(
            projection
                .infrastructure_assets
                .iter()
                .map(|item| item.id.as_str()),
        )
        || !unique_nonempty_ids(
            projection
                .infrastructure_dependencies
                .iter()
                .map(|item| item.id.as_str()),
        )
    {
        return Err(TransportProjectionError::DuplicateOrBlankIdentifier);
    }

    for record in &projection.evidence_records {
        if !nonempty(&record.source_ref)
            || !nonempty(&record.source_profile_id)
            || !nonempty(&record.time_profile_id)
            || record
                .claim_about_id
                .as_deref()
                .is_some_and(|id| !nonempty(id))
        {
            return Err(TransportProjectionError::MissingIdentityOrSourceReference);
        }
        if record.time_profile_id != projection.time_profile_id {
            return Err(TransportProjectionError::InvalidTimeProfile);
        }
        if record
            .content_sha256
            .as_deref()
            .is_some_and(|digest| !is_sha256_shape(digest))
        {
            return Err(TransportProjectionError::InvalidSha256Shape);
        }
        if record.valid_from_epoch_seconds.is_some() != record.valid_until_epoch_seconds.is_some()
            || record
                .valid_from_epoch_seconds
                .zip(record.valid_until_epoch_seconds)
                .is_some_and(|(start, end)| start >= end)
        {
            return Err(TransportProjectionError::InvalidEvidenceValidityInterval);
        }
    }

    let evidence_ids: BTreeSet<&str> = projection
        .evidence_records
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let physical_node_ids: BTreeSet<&str> = projection
        .physical_nodes
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let physical_segment_ids: BTreeSet<&str> = projection
        .physical_segments
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let communication_endpoint_ids: BTreeSet<&str> = projection
        .communication_endpoints
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let infrastructure_asset_ids: BTreeSet<&str> = projection
        .infrastructure_assets
        .iter()
        .map(|item| item.id.as_str())
        .collect();

    for node in &projection.physical_nodes {
        if !nonempty(&node.label) {
            return Err(TransportProjectionError::MissingIdentityOrSourceReference);
        }
        if node.geography.is_some_and(|point| !point.is_valid()) {
            return Err(TransportProjectionError::InvalidGeography);
        }
        validate_evidence_refs(&node.evidence_ids, &evidence_ids)?;
    }

    for segment in &projection.physical_segments {
        if !physical_node_ids.contains(segment.from_node_id.as_str())
            || !physical_node_ids.contains(segment.to_node_id.as_str())
        {
            return Err(TransportProjectionError::UnknownPhysicalNode);
        }
        if segment.capacity.is_some_and(|capacity| capacity.amount == 0) {
            return Err(TransportProjectionError::InvalidCapacity);
        }
        validate_evidence_refs(&segment.evidence_ids, &evidence_ids)?;
    }

    for milestone in &projection.shipment_milestones {
        if !nonempty(&milestone.shipment_ref) {
            return Err(TransportProjectionError::MissingIdentityOrSourceReference);
        }
        if milestone
            .leg_id
            .as_deref()
            .is_some_and(|id| !physical_segment_ids.contains(id))
        {
            return Err(TransportProjectionError::UnknownPhysicalSegment);
        }
        if milestone.time_profile_id != projection.time_profile_id {
            return Err(TransportProjectionError::InvalidTimeProfile);
        }
        let expected_time_meaning = match milestone.milestone {
            ShipmentMilestoneKindV1::Planned => EventTimeMeaningV1::Planned,
            ShipmentMilestoneKindV1::ForecastArrival => EventTimeMeaningV1::Forecast,
            ShipmentMilestoneKindV1::ReservationProposed
            | ShipmentMilestoneKindV1::ReservationAccepted
            | ShipmentMilestoneKindV1::Dispatched
            | ShipmentMilestoneKindV1::ArrivalReported
            | ShipmentMilestoneKindV1::DeliveryConfirmed
            | ShipmentMilestoneKindV1::Cancelled => EventTimeMeaningV1::Occurred,
        };
        if milestone.time_meaning != expected_time_meaning {
            return Err(TransportProjectionError::InvalidMilestoneTimeMeaning);
        }
        validate_evidence_refs(&milestone.evidence_ids, &evidence_ids)?;
    }

    for endpoint in &projection.communication_endpoints {
        if !nonempty(&endpoint.label) {
            return Err(TransportProjectionError::MissingIdentityOrSourceReference);
        }
        validate_evidence_refs(&endpoint.evidence_ids, &evidence_ids)?;
    }

    for contact in &projection.communication_contacts {
        if !communication_endpoint_ids.contains(contact.from_endpoint_id.as_str())
            || !communication_endpoint_ids.contains(contact.to_endpoint_id.as_str())
        {
            return Err(TransportProjectionError::UnknownCommunicationEndpoint);
        }
        if contact.start_epoch_seconds >= contact.end_epoch_seconds {
            return Err(TransportProjectionError::InvalidContactWindow);
        }
        if contact.time_profile_id != projection.time_profile_id {
            return Err(TransportProjectionError::InvalidTimeProfile);
        }
        if contact.capacity_bytes == Some(0) {
            return Err(TransportProjectionError::InvalidCapacity);
        }
        validate_evidence_refs(&contact.evidence_ids, &evidence_ids)?;
    }

    for relation in &projection.evidence_relations {
        if !evidence_ids.contains(relation.from_evidence_id.as_str())
            || !evidence_ids.contains(relation.to_evidence_id.as_str())
        {
            return Err(TransportProjectionError::UnknownEvidenceReference);
        }
    }

    for asset in &projection.infrastructure_assets {
        if !nonempty(&asset.label) {
            return Err(TransportProjectionError::MissingIdentityOrSourceReference);
        }
        validate_evidence_refs(&asset.evidence_ids, &evidence_ids)?;
    }

    for dependency in &projection.infrastructure_dependencies {
        if dependency.dependent_asset_id == dependency.prerequisite_asset_id {
            return Err(TransportProjectionError::SelfReferentialDependency);
        }
        if !infrastructure_asset_ids.contains(dependency.dependent_asset_id.as_str())
            || !infrastructure_asset_ids.contains(dependency.prerequisite_asset_id.as_str())
        {
            return Err(TransportProjectionError::UnknownInfrastructureAsset);
        }
        validate_evidence_refs(&dependency.evidence_ids, &evidence_ids)?;
    }

    Ok(())
}

/// Return a copy with every record collection sorted by stable ID. This is a
/// normalization utility for reproducible serialization, not a cryptographic
/// canonical-JSON or hash implementation.
pub fn normalized_transport_projection(
    mut projection: TransportProjectionV1,
) -> TransportProjectionV1 {
    projection.evidence_records.sort_by(|a, b| a.id.cmp(&b.id));
    projection.physical_nodes.sort_by(|a, b| a.id.cmp(&b.id));
    projection.physical_segments.sort_by(|a, b| a.id.cmp(&b.id));
    projection
        .shipment_milestones
        .sort_by(|a, b| a.id.cmp(&b.id));
    projection
        .communication_endpoints
        .sort_by(|a, b| a.id.cmp(&b.id));
    projection
        .communication_contacts
        .sort_by(|a, b| a.id.cmp(&b.id));
    projection.evidence_relations.sort_by(|a, b| a.id.cmp(&b.id));
    projection
        .infrastructure_assets
        .sort_by(|a, b| a.id.cmp(&b.id));
    projection
        .infrastructure_dependencies
        .sort_by(|a, b| a.id.cmp(&b.id));
    projection
}

/// Serialize a normalized projection for deterministic comparison in fixtures.
/// Source evidence and object values still need explicit source/encoding rules
/// before these bytes can be treated as a cross-implementation canonical digest.
pub fn normalized_transport_projection_json(
    projection: &TransportProjectionV1,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&normalized_transport_projection(projection.clone()))
}

/// Synthetic example only. It has no real coordinates, source observations, or
/// South African capacity values. Two contradictory rail-capacity claims remain
/// visible and the rail segment therefore has no selected numeric capacity.
pub fn synthetic_freight_corridor_v1() -> TransportProjectionV1 {
    let time_profile = SYNTHETIC_RELATIVE_TIME_PROFILE_V1.to_owned();
    let base_evidence = TransportEvidenceV1 {
        id: "scenario-baseline".to_owned(),
        claim_about_id: None,
        kind: EvidenceKindV1::Scenario,
        qualification: EvidenceQualificationV1::ScenarioOnly,
        source_ref: "synthetic://transport-projection/v1/demo".to_owned(),
        source_profile_id: "sol-atlas.transport-projection.synthetic.v1".to_owned(),
        source_version: Some("1.0".to_owned()),
        content_sha256: None,
        source_event_time_epoch_seconds: None,
        observed_at_epoch_seconds: None,
        repository_recorded_at_epoch_seconds: None,
        valid_from_epoch_seconds: None,
        valid_until_epoch_seconds: None,
        time_profile_id: time_profile.clone(),
    };
    let capacity_claim_a = TransportEvidenceV1 {
        id: "capacity-claim-a".to_owned(),
        claim_about_id: Some("rail-haul".to_owned()),
        kind: EvidenceKindV1::Derived,
        qualification: EvidenceQualificationV1::Conflicted,
        source_ref: "synthetic://transport-projection/v1/rail-capacity-a".to_owned(),
        source_profile_id: "synthetic-capacity-model-a-v1".to_owned(),
        source_version: Some("1.0".to_owned()),
        content_sha256: None,
        source_event_time_epoch_seconds: None,
        observed_at_epoch_seconds: None,
        repository_recorded_at_epoch_seconds: None,
        valid_from_epoch_seconds: None,
        valid_until_epoch_seconds: None,
        time_profile_id: time_profile.clone(),
    };
    let capacity_claim_b = TransportEvidenceV1 {
        id: "capacity-claim-b".to_owned(),
        claim_about_id: Some("rail-haul".to_owned()),
        kind: EvidenceKindV1::Derived,
        qualification: EvidenceQualificationV1::Conflicted,
        source_ref: "synthetic://transport-projection/v1/rail-capacity-b".to_owned(),
        source_profile_id: "synthetic-capacity-model-b-v1".to_owned(),
        source_version: Some("1.0".to_owned()),
        content_sha256: None,
        source_event_time_epoch_seconds: None,
        observed_at_epoch_seconds: None,
        repository_recorded_at_epoch_seconds: None,
        valid_from_epoch_seconds: None,
        valid_until_epoch_seconds: None,
        time_profile_id: time_profile.clone(),
    };

    let baseline_ref = vec!["scenario-baseline".to_owned()];
    let mut projection = TransportProjectionV1 {
        schema_version: TRANSPORT_PROJECTION_SCHEMA_V1.to_owned(),
        projection_id: "synthetic-freight-corridor-v1".to_owned(),
        source_manifest_ref: "synthetic://transport-projection/v1/manifest".to_owned(),
        source_manifest_sha256: None,
        generated_at_epoch_seconds: None,
        time_profile_id: time_profile.clone(),
        evidence_records: vec![base_evidence, capacity_claim_a, capacity_claim_b],
        physical_nodes: vec![
            PhysicalNodeV1 {
                id: "producer-a".to_owned(),
                label: "Synthetic producer".to_owned(),
                kind: PhysicalNodeKindV1::Producer,
                geography: None,
                evidence_ids: baseline_ref.clone(),
            },
            PhysicalNodeV1 {
                id: "inland-terminal-a".to_owned(),
                label: "Synthetic inland intermodal terminal".to_owned(),
                kind: PhysicalNodeKindV1::IntermodalTerminal,
                geography: None,
                evidence_ids: baseline_ref.clone(),
            },
            PhysicalNodeV1 {
                id: "port-a".to_owned(),
                label: "Synthetic seaport".to_owned(),
                kind: PhysicalNodeKindV1::Port,
                geography: None,
                evidence_ids: baseline_ref.clone(),
            },
            PhysicalNodeV1 {
                id: "receiver-a".to_owned(),
                label: "Synthetic receiving hub".to_owned(),
                kind: PhysicalNodeKindV1::DeliveryPoint,
                geography: None,
                evidence_ids: baseline_ref.clone(),
            },
        ],
        physical_segments: vec![
            PhysicalSegmentV1 {
                id: "road-feeder".to_owned(),
                from_node_id: "producer-a".to_owned(),
                to_node_id: "inland-terminal-a".to_owned(),
                operation: PhysicalOperationV1::Movement {
                    mode: TransportModeV1::Road,
                },
                state: NetworkSegmentStateV1::AssumedForScenario,
                distance_meters: Some(40_000),
                expected_duration_seconds: Some(3_600),
                capacity: Some(CapacityValueV1 {
                    amount: 24,
                    unit: CapacityUnitV1::Shipments,
                    basis: CapacityBasisV1::Unspecified,
                }),
                evidence_ids: baseline_ref.clone(),
            },
            PhysicalSegmentV1 {
                id: "road-rail-transfer".to_owned(),
                from_node_id: "inland-terminal-a".to_owned(),
                to_node_id: "inland-terminal-a".to_owned(),
                operation: PhysicalOperationV1::Transfer {
                    from_mode: TransportModeV1::Road,
                    to_mode: TransportModeV1::Rail,
                },
                state: NetworkSegmentStateV1::AssumedForScenario,
                distance_meters: None,
                expected_duration_seconds: Some(1_800),
                capacity: Some(CapacityValueV1 {
                    amount: 4,
                    unit: CapacityUnitV1::Shipments,
                    basis: CapacityBasisV1::Unspecified,
                }),
                evidence_ids: baseline_ref.clone(),
            },
            PhysicalSegmentV1 {
                id: "rail-haul".to_owned(),
                from_node_id: "inland-terminal-a".to_owned(),
                to_node_id: "port-a".to_owned(),
                operation: PhysicalOperationV1::Movement {
                    mode: TransportModeV1::Rail,
                },
                state: NetworkSegmentStateV1::AssumedForScenario,
                distance_meters: Some(600_000),
                expected_duration_seconds: Some(28_800),
                capacity: None,
                evidence_ids: vec![
                    "capacity-claim-a".to_owned(),
                    "capacity-claim-b".to_owned(),
                ],
            },
            PhysicalSegmentV1 {
                id: "rail-maritime-transfer".to_owned(),
                from_node_id: "port-a".to_owned(),
                to_node_id: "port-a".to_owned(),
                operation: PhysicalOperationV1::Transfer {
                    from_mode: TransportModeV1::Rail,
                    to_mode: TransportModeV1::Maritime,
                },
                state: NetworkSegmentStateV1::AssumedForScenario,
                distance_meters: None,
                expected_duration_seconds: Some(5_400),
                capacity: Some(CapacityValueV1 {
                    amount: 4,
                    unit: CapacityUnitV1::Shipments,
                    basis: CapacityBasisV1::Unspecified,
                }),
                evidence_ids: baseline_ref.clone(),
            },
            PhysicalSegmentV1 {
                id: "sea-leg".to_owned(),
                from_node_id: "port-a".to_owned(),
                to_node_id: "receiver-a".to_owned(),
                operation: PhysicalOperationV1::Movement {
                    mode: TransportModeV1::Maritime,
                },
                state: NetworkSegmentStateV1::AssumedForScenario,
                distance_meters: Some(3_000_000),
                expected_duration_seconds: Some(172_800),
                capacity: Some(CapacityValueV1 {
                    amount: 24,
                    unit: CapacityUnitV1::Shipments,
                    basis: CapacityBasisV1::Unspecified,
                }),
                evidence_ids: baseline_ref.clone(),
            },
        ],
        shipment_milestones: vec![
            ShipmentMilestoneV1 {
                id: "shipment-planned".to_owned(),
                shipment_ref: "shipment-synthetic-1".to_owned(),
                leg_id: Some("road-feeder".to_owned()),
                milestone: ShipmentMilestoneKindV1::Planned,
                time_meaning: EventTimeMeaningV1::Planned,
                event_time_epoch_seconds: Some(3_600),
                repository_recorded_at_epoch_seconds: Some(3_000),
                time_profile_id: time_profile.clone(),
                evidence_ids: baseline_ref.clone(),
            },
            ShipmentMilestoneV1 {
                id: "shipment-dispatched".to_owned(),
                shipment_ref: "shipment-synthetic-1".to_owned(),
                leg_id: Some("road-feeder".to_owned()),
                milestone: ShipmentMilestoneKindV1::Dispatched,
                time_meaning: EventTimeMeaningV1::Occurred,
                event_time_epoch_seconds: Some(3_650),
                repository_recorded_at_epoch_seconds: Some(3_660),
                time_profile_id: time_profile.clone(),
                evidence_ids: baseline_ref.clone(),
            },
            ShipmentMilestoneV1 {
                id: "shipment-forecast-arrival".to_owned(),
                shipment_ref: "shipment-synthetic-1".to_owned(),
                leg_id: Some("road-feeder".to_owned()),
                milestone: ShipmentMilestoneKindV1::ForecastArrival,
                time_meaning: EventTimeMeaningV1::Forecast,
                event_time_epoch_seconds: Some(7_200),
                repository_recorded_at_epoch_seconds: Some(3_660),
                time_profile_id: time_profile.clone(),
                evidence_ids: baseline_ref.clone(),
            },
            ShipmentMilestoneV1 {
                id: "shipment-arrival-reported".to_owned(),
                shipment_ref: "shipment-synthetic-1".to_owned(),
                leg_id: Some("road-feeder".to_owned()),
                milestone: ShipmentMilestoneKindV1::ArrivalReported,
                time_meaning: EventTimeMeaningV1::Occurred,
                event_time_epoch_seconds: Some(7_050),
                repository_recorded_at_epoch_seconds: Some(7_100),
                time_profile_id: time_profile.clone(),
                evidence_ids: baseline_ref.clone(),
            },
        ],
        communication_endpoints: vec![
            CommunicationEndpointV1 {
                id: "dispatch-center-a".to_owned(),
                kind: CommunicationEndpointKindV1::DispatchCenter,
                label: "Synthetic dispatch center".to_owned(),
                evidence_ids: baseline_ref.clone(),
            },
            CommunicationEndpointV1 {
                id: "terminal-gateway-a".to_owned(),
                kind: CommunicationEndpointKindV1::TerminalGateway,
                label: "Synthetic terminal gateway".to_owned(),
                evidence_ids: baseline_ref.clone(),
            },
        ],
        communication_contacts: vec![CommunicationContactV1 {
            id: "dispatch-contact-window".to_owned(),
            from_endpoint_id: "dispatch-center-a".to_owned(),
            to_endpoint_id: "terminal-gateway-a".to_owned(),
            direction: CommunicationDirectionV1::Directed,
            state: CommunicationContactStateV1::Scheduled,
            start_epoch_seconds: 3_000,
            end_epoch_seconds: 3_900,
            time_profile_id: time_profile.clone(),
            capacity_bytes: Some(65_536),
            latency_milliseconds: Some(100),
            evidence_ids: baseline_ref.clone(),
        }],
        evidence_relations: vec![EvidenceRelationV1 {
            id: "rail-capacity-conflict".to_owned(),
            from_evidence_id: "capacity-claim-a".to_owned(),
            to_evidence_id: "capacity-claim-b".to_owned(),
            relation: EvidenceRelationKindV1::Conflicts,
        }],
        infrastructure_assets: vec![
            InfrastructureAssetV1 {
                id: "terminal-handling-a".to_owned(),
                kind: InfrastructureAssetKindV1::HandlingEquipment,
                label: "Synthetic terminal handling system".to_owned(),
                evidence_ids: baseline_ref.clone(),
            },
            InfrastructureAssetV1 {
                id: "terminal-power-a".to_owned(),
                kind: InfrastructureAssetKindV1::Power,
                label: "Synthetic terminal power supply".to_owned(),
                evidence_ids: baseline_ref.clone(),
            },
        ],
        infrastructure_dependencies: vec![InfrastructureDependencyV1 {
            id: "handling-requires-power".to_owned(),
            dependent_asset_id: "terminal-handling-a".to_owned(),
            prerequisite_asset_id: "terminal-power-a".to_owned(),
            evidence_ids: baseline_ref,
        }],
    };

    projection.evidence_records.sort_by(|a, b| a.id.cmp(&b.id));
    projection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_fixture_is_structurally_valid_and_explicitly_synthetic() {
        let projection = synthetic_freight_corridor_v1();
        assert_eq!(
            projection.time_profile_id,
            SYNTHETIC_RELATIVE_TIME_PROFILE_V1
        );
        assert!(projection.physical_nodes.iter().all(|node| node.geography.is_none()));
        assert!(projection
            .evidence_records
            .iter()
            .all(|record| record.qualification != EvidenceQualificationV1::AcceptedForProjection));
        assert_eq!(validate_transport_projection(&projection), Ok(()));
    }

    #[test]
    fn normalized_json_is_invariant_to_collection_order() {
        let original = synthetic_freight_corridor_v1();
        let before = normalized_transport_projection_json(&original).expect("serialize");
        let mut reordered = original;
        reordered.physical_nodes.reverse();
        reordered.physical_segments.reverse();
        reordered.shipment_milestones.reverse();
        reordered.communication_contacts.reverse();
        reordered.communication_endpoints.reverse();
        reordered.evidence_records.reverse();
        reordered.evidence_relations.reverse();
        reordered.infrastructure_assets.reverse();
        reordered.infrastructure_dependencies.reverse();
        let after = normalized_transport_projection_json(&reordered).expect("serialize");
        assert_eq!(before, after);
    }

    #[test]
    fn duplicate_node_identifiers_are_rejected() {
        let mut projection = synthetic_freight_corridor_v1();
        projection.physical_nodes[1].id = projection.physical_nodes[0].id.clone();
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::DuplicateOrBlankIdentifier)
        );
    }

    #[test]
    fn unknown_evidence_reference_is_rejected() {
        let mut projection = synthetic_freight_corridor_v1();
        projection.physical_nodes[0].evidence_ids = vec!["missing-source".to_owned()];
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::UnknownEvidenceReference)
        );
    }

    #[test]
    fn invalid_geographic_coordinates_are_rejected() {
        let mut projection = synthetic_freight_corridor_v1();
        projection.physical_nodes[0].geography = Some(GeoPointMicrodegreesV1 {
            latitude_microdegrees: 90_000_001,
            longitude_microdegrees: 0,
        });
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::InvalidGeography)
        );
    }

    #[test]
    fn conflicting_capacity_claims_are_preserved_without_selecting_a_value() {
        let projection = synthetic_freight_corridor_v1();
        let rail = projection
            .physical_segments
            .iter()
            .find(|segment| segment.id == "rail-haul")
            .expect("rail segment");
        assert_eq!(rail.capacity, None);
        assert_eq!(rail.evidence_ids.len(), 2);
        assert_eq!(
            projection
                .evidence_records
                .iter()
                .filter(|record| record.qualification == EvidenceQualificationV1::Conflicted)
                .count(),
            2
        );
        assert_eq!(projection.evidence_relations[0].relation, EvidenceRelationKindV1::Conflicts);
        assert_eq!(validate_transport_projection(&projection), Ok(()));
    }

    #[test]
    fn contact_windows_must_be_nonempty_and_use_projection_time_profile() {
        let mut projection = synthetic_freight_corridor_v1();
        projection.communication_contacts[0].end_epoch_seconds =
            projection.communication_contacts[0].start_epoch_seconds;
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::InvalidContactWindow)
        );

        let mut projection = synthetic_freight_corridor_v1();
        projection.communication_contacts[0].time_profile_id = "other-clock".to_owned();
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::InvalidTimeProfile)
        );
    }

    #[test]
    fn milestone_kind_and_time_meaning_must_agree() {
        let mut projection = synthetic_freight_corridor_v1();
        projection.shipment_milestones[0].time_meaning = EventTimeMeaningV1::Forecast;
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::InvalidMilestoneTimeMeaning)
        );
    }

    #[test]
    fn evidence_timestamps_must_use_the_projection_time_profile() {
        let mut projection = synthetic_freight_corridor_v1();
        projection.evidence_records[0].time_profile_id = "unmapped-source-clock".to_owned();
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::InvalidTimeProfile)
        );
    }

    #[test]
    fn physical_and_communication_edges_remain_distinct_in_serialized_contract() {
        let projection = synthetic_freight_corridor_v1();
        let serialized = String::from_utf8(
            normalized_transport_projection_json(&projection).expect("serialize"),
        )
        .expect("UTF-8 JSON");
        assert!(serialized.contains("\"physical_segments\""));
        assert!(serialized.contains("\"communication_contacts\""));
        assert!(!serialized.contains("\"contact_window_as_physical_leg\""));
    }

    #[test]
    fn one_sided_or_reversed_evidence_validity_interval_is_rejected() {
        let mut projection = synthetic_freight_corridor_v1();
        projection.evidence_records[0].valid_from_epoch_seconds = Some(200);
        projection.evidence_records[0].valid_until_epoch_seconds = None;
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::InvalidEvidenceValidityInterval)
        );

        let mut projection = synthetic_freight_corridor_v1();
        projection.evidence_records[0].valid_from_epoch_seconds = Some(300);
        projection.evidence_records[0].valid_until_epoch_seconds = Some(300);
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::InvalidEvidenceValidityInterval)
        );
    }

    #[test]
    fn malformed_source_hash_is_rejected_without_claiming_hash_verification() {
        let mut projection = synthetic_freight_corridor_v1();
        projection.source_manifest_sha256 = Some("not-a-sha256".to_owned());
        assert_eq!(
            validate_transport_projection(&projection),
            Err(TransportProjectionError::InvalidSha256Shape)
        );
    }
}
