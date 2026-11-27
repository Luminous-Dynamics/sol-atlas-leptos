// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Independent, fail-closed audit of a proposed EV route and charging plan.
//!
//! This module consumes upstream energy/capability intervals. It does not
//! estimate vehicle energy, optimize routes, operate chargers, or control a
//! vehicle. A passing result means only that declared bounds and evidence
//! references satisfy this bounded audit contract.

use crate::ev_evidence_admission::{
    EvidenceAttestation, EvidenceTrustStore, EvidenceVerificationRequest, validate_trust_store,
    verify_evidence,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub const INPUT_SCHEMA_VERSION: &str = "ev-route-assurance-input/v0.1";
pub const ASSESSMENT_SCHEMA_VERSION: &str = "ev-route-assurance-assessment/v0.1";
const EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    Observed,
    QualifiedModel,
    Forecast,
    Predicted,
    Scenario,
    Synthetic,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    pub reference_id: String,
    pub digest_sha256: String,
    pub source_class: EvidenceClass,
    #[serde(default)]
    pub observed_at_unix_s: Option<i64>,
    #[serde(default)]
    pub valid_until_unix_s: Option<i64>,
    #[serde(default)]
    pub attestation: Option<EvidenceAttestation>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericInterval {
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VehicleEvidence {
    pub vehicle_configuration_id: String,
    pub connector_ids: Vec<String>,
    pub configuration_evidence: EvidenceRef,
    pub energy_model_evidence: EvidenceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackCapability {
    pub pack_id: String,
    pub usable_energy_wh: NumericInterval,
    pub initial_energy_wh: NumericInterval,
    pub reserve_floor_wh: f64,
    pub charge_acceptance_power_w: NumericInterval,
    pub capability_evidence: EvidenceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteState {
    OpenObserved,
    ClosedObserved,
    ForecastOpen,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriveLeg {
    pub kind: String,
    pub event_id: String,
    pub planned_start_unix_s: i64,
    pub duration_seconds: u64,
    pub route_state: RouteState,
    pub energy_consumption_wh: NumericInterval,
    pub route_evidence: EvidenceRef,
    pub energy_estimate_evidence: EvidenceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChargerOperationalStatus {
    OperationalObserved,
    OfflineObserved,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridConnectionStage {
    Energized,
    SignedAgreement,
    CapacityAllocated,
    ApplicationSubmitted,
    BudgetQuote,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteConnection {
    pub grid_connection_stage: GridConnectionStage,
    #[serde(default)]
    pub site_import_power_w: Option<NumericInterval>,
    pub connection_evidence: EvidenceRef,
    #[serde(default)]
    pub import_power_evidence: Option<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Charger {
    pub charger_id: String,
    pub connector_id: String,
    pub operational_status: ChargerOperationalStatus,
    pub status_evidence: EvidenceRef,
    pub configuration_evidence: EvidenceRef,
    /// DC output power available at the vehicle connector.
    pub evse_output_power_w: NumericInterval,
    pub evse_power_evidence: EvidenceRef,
    /// Converts site import power (AC) to power at the vehicle connector.
    pub site_to_connector_efficiency: NumericInterval,
    pub conversion_evidence: EvidenceRef,
    pub site_connection: SiteConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChargingStop {
    pub kind: String,
    pub event_id: String,
    pub planned_start_unix_s: i64,
    pub duration_seconds: u64,
    pub requested_charge_energy_wh: f64,
    pub charger: Charger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlanEvent {
    DriveLeg(Box<DriveLeg>),
    ChargingStop(Box<ChargingStop>),
}

impl PlanEvent {
    fn event_id(&self) -> &str {
        match self {
            Self::DriveLeg(value) => &value.event_id,
            Self::ChargingStop(value) => &value.event_id,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::DriveLeg(_) => "drive_leg",
            Self::ChargingStop(_) => "charging_stop",
        }
    }

    fn planned_start(&self) -> i64 {
        match self {
            Self::DriveLeg(value) => value.planned_start_unix_s,
            Self::ChargingStop(value) => value.planned_start_unix_s,
        }
    }

    fn duration_seconds(&self) -> u64 {
        match self {
            Self::DriveLeg(value) => value.duration_seconds,
            Self::ChargingStop(value) => value.duration_seconds,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutePlan {
    pub schema_version: String,
    pub plan_id: String,
    pub assessed_at_unix_s: i64,
    pub vehicle: VehicleEvidence,
    pub pack: PackCapability,
    pub events: Vec<PlanEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssessmentDisposition {
    FeasibleUnderDeclaredBounds,
    Infeasible,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventDisposition {
    WithinDeclaredBounds,
    Infeasible,
    Indeterminate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAssessment {
    pub event_id: String,
    pub event_kind: String,
    pub disposition: EventDisposition,
    pub energy_after_wh: NumericInterval,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_charge_wh: Option<NumericInterval>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBinding {
    pub role: String,
    pub reference_id: String,
    pub digest_sha256: String,
    pub source_class: EvidenceClass,
    pub evidence_role: String,
    pub subject_id: String,
    pub verification_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_at_unix_s: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until_unix_s: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    pub schema_version: String,
    pub plan_id: String,
    pub trust_policy_id: String,
    pub disposition: AssessmentDisposition,
    pub reserve_floor_wh: f64,
    pub final_energy_wh: NumericInterval,
    pub event_assessments: Vec<EventAssessment>,
    pub consumed_evidence: Vec<EvidenceBinding>,
    pub findings: Vec<Finding>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReceipt {
    /// SHA-256 of the exact input file bytes, including whitespace.
    pub input_sha256: String,
    /// SHA-256 of the exact separately supplied trust-store file bytes.
    pub trust_store_sha256: String,
    /// Canonical typed trust-store digest pinned by the deployment policy.
    pub trust_store_canonical_sha256: String,
    /// SHA-256 of the compact serialized assessment object.
    pub assessment_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentEnvelope {
    pub receipt: RunReceipt,
    pub assessment: Assessment,
}

/// Parse, assess and bind a result to the exact input and trust-store bytes.
pub fn assess_json(
    input_bytes: &[u8],
    trust_store_bytes: &[u8],
    expected_trust_store_sha256: &str,
) -> Result<AssessmentEnvelope, String> {
    let input: RoutePlan = serde_json::from_slice(input_bytes)
        .map_err(|error| format!("invalid input JSON: {error}"))?;
    let trust_store: EvidenceTrustStore = serde_json::from_slice(trust_store_bytes)
        .map_err(|error| format!("invalid evidence trust-store JSON: {error}"))?;
    validate_trust_store(&trust_store)?;
    let canonical_trust_store_bytes = serde_json::to_vec(&trust_store)
        .map_err(|error| format!("cannot canonicalize evidence trust store: {error}"))?;
    let canonical_trust_store_sha256 = sha256_hex(&canonical_trust_store_bytes);
    if !is_sha256_hex(expected_trust_store_sha256)
        || canonical_trust_store_sha256 != expected_trust_store_sha256
    {
        return Err(
            "canonical evidence trust-store digest does not match the deployment-pinned SHA-256"
                .into(),
        );
    }
    let assessment = assess(&input, &trust_store, expected_trust_store_sha256)?;
    let assessment_bytes = serde_json::to_vec(&assessment)
        .map_err(|error| format!("could not serialize assessment: {error}"))?;
    Ok(AssessmentEnvelope {
        receipt: RunReceipt {
            input_sha256: sha256_hex(input_bytes),
            trust_store_sha256: sha256_hex(trust_store_bytes),
            trust_store_canonical_sha256: expected_trust_store_sha256.to_owned(),
            assessment_sha256: sha256_hex(&assessment_bytes),
        },
        assessment,
    })
}

/// Audit a parsed route plan against an independently supplied trust store.
/// Invalid structure/numerics/trust policy are rejected; missing or unauthenticated
/// source evidence returns INDETERMINATE rather than a false pass.
pub fn assess(
    input: &RoutePlan,
    trust_store: &EvidenceTrustStore,
    expected_trust_store_sha256: &str,
) -> Result<Assessment, String> {
    validate_trust_store(trust_store)?;
    let canonical_trust_store_bytes = serde_json::to_vec(trust_store)
        .map_err(|error| format!("cannot canonicalize evidence trust store: {error}"))?;
    if !is_sha256_hex(expected_trust_store_sha256)
        || sha256_hex(&canonical_trust_store_bytes) != expected_trust_store_sha256
    {
        return Err(
            "canonical evidence trust-store digest does not match the deployment-pinned SHA-256"
                .into(),
        );
    }
    if input.schema_version != INPUT_SCHEMA_VERSION {
        return Err(format!(
            "unsupported schema_version: expected {INPUT_SCHEMA_VERSION}"
        ));
    }
    require_identifier(&input.plan_id, "plan_id")?;
    if input.events.is_empty() {
        return Err("events must contain at least one route or charging event".into());
    }
    require_identifier(
        &input.vehicle.vehicle_configuration_id,
        "vehicle.vehicle_configuration_id",
    )?;
    if input.vehicle.connector_ids.is_empty() {
        return Err("vehicle.connector_ids must not be empty".into());
    }
    let mut connector_ids = HashSet::new();
    for connector_id in &input.vehicle.connector_ids {
        require_identifier(connector_id, "vehicle.connector_ids[]")?;
        if !connector_ids.insert(connector_id) {
            return Err(format!("duplicate vehicle connector_id: {connector_id}"));
        }
    }
    require_identifier(&input.pack.pack_id, "pack.pack_id")?;

    validate_interval(input.pack.usable_energy_wh, "pack.usable_energy_wh")?;
    validate_interval(input.pack.initial_energy_wh, "pack.initial_energy_wh")?;
    validate_interval(
        input.pack.charge_acceptance_power_w,
        "pack.charge_acceptance_power_w",
    )?;
    validate_nonnegative(input.pack.reserve_floor_wh, "pack.reserve_floor_wh")?;
    if input.pack.initial_energy_wh.upper > input.pack.usable_energy_wh.upper + EPSILON {
        return Err("initial stored energy exceeds the upper usable-capacity bound".into());
    }

    let mut findings = Vec::new();
    let mut any_infeasible = false;
    let mut any_indeterminate = false;

    let vehicle_configuration_qualified = evidence_class_qualified(
        &input.vehicle.configuration_evidence,
        "vehicle_configuration",
        &input.vehicle.vehicle_configuration_id,
        trust_store,
        input.assessed_at_unix_s,
        "vehicle.configuration_evidence",
    )?;
    let vehicle_energy_model_qualified = evidence_class_qualified(
        &input.vehicle.energy_model_evidence,
        "vehicle_energy_model",
        &input.vehicle.vehicle_configuration_id,
        trust_store,
        input.assessed_at_unix_s,
        "vehicle.energy_model_evidence",
    )?;
    let pack_capability_qualified = evidence_class_qualified(
        &input.pack.capability_evidence,
        "pack_capability",
        &input.pack.pack_id,
        trust_store,
        input.assessed_at_unix_s,
        "pack.capability_evidence",
    )?;

    if !vehicle_configuration_qualified {
        push_finding(
            &mut findings,
            None,
            "VEHICLE_CONFIGURATION_EVIDENCE_UNQUALIFIED",
            "Vehicle configuration lineage is synthetic, forecast, predicted, scenario, or unknown.",
        );
        any_indeterminate = true;
    }
    if !vehicle_energy_model_qualified {
        push_finding(
            &mut findings,
            None,
            "VEHICLE_ENERGY_MODEL_EVIDENCE_UNQUALIFIED",
            "The upstream vehicle energy model is not observed or independently qualified.",
        );
        any_indeterminate = true;
    }
    if !pack_capability_qualified {
        push_finding(
            &mut findings,
            None,
            "PACK_CAPABILITY_EVIDENCE_UNQUALIFIED",
            "The upstream pack capability descriptor is not observed or independently qualified.",
        );
        any_indeterminate = true;
    }

    let mut energy_bounds_qualified = vehicle_configuration_qualified
        && vehicle_energy_model_qualified
        && pack_capability_qualified;
    let mut energy = input.pack.initial_energy_wh;
    if input.pack.reserve_floor_wh > input.pack.usable_energy_wh.upper + EPSILON {
        push_finding(
            &mut findings,
            None,
            "RESERVE_EXCEEDS_PACK_CAPACITY",
            "The required reserve exceeds the maximum declared usable pack capacity.",
        );
        if pack_capability_qualified {
            any_infeasible = true;
        } else {
            any_indeterminate = true;
        }
    }
    if energy.upper + EPSILON < input.pack.reserve_floor_wh {
        push_finding(
            &mut findings,
            None,
            "INITIAL_ENERGY_BELOW_RESERVE",
            "Even the upper initial-energy bound is below the reserve floor.",
        );
        if pack_capability_qualified {
            any_infeasible = true;
        } else {
            any_indeterminate = true;
        }
    } else if energy.lower + EPSILON < input.pack.reserve_floor_wh {
        push_finding(
            &mut findings,
            None,
            "INITIAL_RESERVE_NOT_GUARANTEED",
            "The initial-energy interval crosses the reserve floor.",
        );
        any_indeterminate = true;
    }
    if energy.lower > input.pack.usable_energy_wh.lower + EPSILON {
        push_finding(
            &mut findings,
            None,
            "INITIAL_CAPACITY_UNCERTAINTY",
            "Initial stored energy exceeds the lower usable-capacity bound.",
        );
        any_indeterminate = true;
    }

    let mut event_ids = HashSet::new();
    let mut previous_end: Option<i64> = None;
    let mut event_assessments = Vec::with_capacity(input.events.len());

    for event in &input.events {
        let event_id = event.event_id();
        require_identifier(event_id, "events[].event_id")?;
        if !event_ids.insert(event_id.to_owned()) {
            return Err(format!("duplicate event_id: {event_id}"));
        }

        let start = event.planned_start();
        let duration = event.duration_seconds();
        if duration == 0 {
            return Err(format!("{event_id}: duration_seconds must be positive"));
        }
        if start < input.assessed_at_unix_s {
            return Err(format!(
                "{event_id}: planned start precedes assessment time"
            ));
        }
        let duration_i64 = i64::try_from(duration)
            .map_err(|_| format!("{event_id}: duration_seconds exceeds supported range"))?;
        let end = start
            .checked_add(duration_i64)
            .ok_or_else(|| format!("{event_id}: event end timestamp overflows"))?;
        if previous_end.is_some_and(|prior_end| start < prior_end) {
            return Err(format!("{event_id}: events overlap or are out of order"));
        }
        previous_end = Some(end);

        let mut event_infeasible = false;
        let mut event_indeterminate = false;
        let mut delivered_charge = None;

        match event {
            PlanEvent::DriveLeg(leg) => {
                if leg.kind != "drive_leg" {
                    return Err(format!("{event_id}: kind must be drive_leg"));
                }
                validate_interval(
                    leg.energy_consumption_wh,
                    &format!("{event_id}.energy_consumption_wh"),
                )?;
                let leg_energy_qualified = evidence_class_qualified(
                    &leg.energy_estimate_evidence,
                    "leg_energy_estimate",
                    event_id,
                    trust_store,
                    input.assessed_at_unix_s,
                    &format!("{event_id}.energy_estimate_evidence"),
                )?;
                if !leg_energy_qualified {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "ENERGY_ESTIMATE_EVIDENCE_UNQUALIFIED",
                        "The leg-energy interval is not backed by observed or qualified-model evidence.",
                    );
                    event_indeterminate = true;
                }
                energy_bounds_qualified &= leg_energy_qualified;
                match leg.route_state {
                    RouteState::ClosedObserved => {
                        if evidence_current_for_window(
                            &leg.route_evidence,
                            input.assessed_at_unix_s,
                            end,
                            "route_state",
                            event_id,
                            trust_store,
                            &format!("{event_id}.route_evidence"),
                        )? {
                            push_finding(
                                &mut findings,
                                Some(event_id),
                                "ROUTE_CLOSED",
                                "The source-qualified route evidence declares this leg closed.",
                            );
                            event_infeasible = true;
                        } else {
                            push_finding(
                                &mut findings,
                                Some(event_id),
                                "ROUTE_CLOSURE_EVIDENCE_STALE",
                                "A closure claim is not current for the planned traversal window.",
                            );
                            event_indeterminate = true;
                        }
                    }
                    RouteState::OpenObserved => {
                        if !evidence_current_for_window(
                            &leg.route_evidence,
                            input.assessed_at_unix_s,
                            end,
                            "route_state",
                            event_id,
                            trust_store,
                            &format!("{event_id}.route_evidence"),
                        )? {
                            push_finding(
                                &mut findings,
                                Some(event_id),
                                "ROUTE_OPEN_EVIDENCE_STALE",
                                "The route-open observation is missing, future-dated, expired, or not an observed source.",
                            );
                            event_indeterminate = true;
                        }
                    }
                    RouteState::ForecastOpen | RouteState::Unknown => {
                        evidence_reference_valid(
                            &leg.route_evidence,
                            &format!("{event_id}.route_evidence"),
                        )?;
                        push_finding(
                            &mut findings,
                            Some(event_id),
                            "ROUTE_NOT_OBSERVED_OPEN",
                            "The route is forecast-open or unknown; feasibility remains conditional.",
                        );
                        event_indeterminate = true;
                    }
                }

                let raw_energy_lower = energy.lower - leg.energy_consumption_wh.upper;
                let raw_energy_upper = energy.upper - leg.energy_consumption_wh.lower;
                if raw_energy_upper < 0.0 {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "ENERGY_INTERVAL_DEPLETED",
                        "Even the upper remaining-energy bound is negative after this leg.",
                    );
                } else if raw_energy_lower < 0.0 {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "ENERGY_LOWER_BOUND_DEPLETED",
                        "The lower remaining-energy bound is negative; the interval has been clamped at zero.",
                    );
                    event_indeterminate = true;
                }
                energy.lower = raw_energy_lower.max(0.0);
                energy.upper = raw_energy_upper.max(0.0);
                if energy.upper + EPSILON < input.pack.reserve_floor_wh {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "RESERVE_DEFINITELY_BREACHED",
                        "Even the upper remaining-energy bound falls below the reserve floor.",
                    );
                    if energy_bounds_qualified {
                        event_infeasible = true;
                    } else {
                        event_indeterminate = true;
                    }
                } else if energy.lower + EPSILON < input.pack.reserve_floor_wh {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "RESERVE_NOT_GUARANTEED",
                        "The remaining-energy interval crosses the reserve floor.",
                    );
                    event_indeterminate = true;
                }
            }
            PlanEvent::ChargingStop(stop) => {
                if stop.kind != "charging_stop" {
                    return Err(format!("{event_id}: kind must be charging_stop"));
                }
                validate_nonnegative(
                    stop.requested_charge_energy_wh,
                    &format!("{event_id}.requested_charge_energy_wh"),
                )?;
                if stop.requested_charge_energy_wh <= EPSILON {
                    return Err(format!(
                        "{event_id}: requested charge energy must be positive"
                    ));
                }
                let charger = &stop.charger;
                require_identifier(&charger.charger_id, &format!("{event_id}.charger_id"))?;
                require_identifier(&charger.connector_id, &format!("{event_id}.connector_id"))?;

                let connector_compatible = input
                    .vehicle
                    .connector_ids
                    .iter()
                    .any(|connector| connector == &charger.connector_id);
                if !connector_compatible {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "CONNECTOR_INCOMPATIBLE",
                        "The charger connector is not present in the exact vehicle configuration.",
                    );
                    if vehicle_configuration_qualified {
                        event_infeasible = true;
                    } else {
                        event_indeterminate = true;
                    }
                }

                let charger_configuration_qualified = evidence_class_qualified(
                    &charger.configuration_evidence,
                    "charger_configuration",
                    &charger.charger_id,
                    trust_store,
                    input.assessed_at_unix_s,
                    &format!("{event_id}.charger.configuration_evidence"),
                )?;
                let evse_power_qualified = evidence_class_qualified(
                    &charger.evse_power_evidence,
                    "evse_power_rating",
                    &charger.charger_id,
                    trust_store,
                    input.assessed_at_unix_s,
                    &format!("{event_id}.charger.evse_power_evidence"),
                )?;
                let conversion_qualified = evidence_class_qualified(
                    &charger.conversion_evidence,
                    "conversion_efficiency_model",
                    &charger.charger_id,
                    trust_store,
                    input.assessed_at_unix_s,
                    &format!("{event_id}.charger.conversion_evidence"),
                )?;
                if !charger_configuration_qualified {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "CHARGER_CONFIGURATION_EVIDENCE_UNQUALIFIED",
                        "Charger equipment/configuration lineage is not observed or independently qualified.",
                    );
                    event_indeterminate = true;
                }
                if !evse_power_qualified {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "EVSE_POWER_EVIDENCE_UNQUALIFIED",
                        "The EVSE output-power interval is not observed or independently qualified.",
                    );
                    event_indeterminate = true;
                }
                if !conversion_qualified {
                    push_finding(
                        &mut findings,
                        Some(event_id),
                        "CONVERSION_EVIDENCE_UNQUALIFIED",
                        "The site-to-connector conversion-efficiency interval is not observed or independently qualified.",
                    );
                    event_indeterminate = true;
                }
                validate_interval(
                    charger.evse_output_power_w,
                    &format!("{event_id}.charger.evse_output_power_w"),
                )?;
                validate_interval(
                    charger.site_to_connector_efficiency,
                    &format!("{event_id}.charger.site_to_connector_efficiency"),
                )?;
                if charger.site_to_connector_efficiency.upper > 1.0 + EPSILON {
                    return Err(format!("{event_id}: conversion efficiency cannot exceed 1"));
                }

                let charger_status_current = evidence_current_for_window(
                    &charger.status_evidence,
                    input.assessed_at_unix_s,
                    end,
                    "charger_status",
                    &charger.charger_id,
                    trust_store,
                    &format!("{event_id}.charger.status_evidence"),
                )?;
                match charger.operational_status {
                    ChargerOperationalStatus::OfflineObserved => {
                        if charger_status_current {
                            push_finding(
                                &mut findings,
                                Some(event_id),
                                "CHARGER_OFFLINE",
                                "The source-qualified charger status is offline for the planned stop.",
                            );
                            event_infeasible = true;
                        } else {
                            push_finding(
                                &mut findings,
                                Some(event_id),
                                "CHARGER_STATUS_STALE",
                                "The offline status observation is not current for the planned stop.",
                            );
                            event_indeterminate = true;
                        }
                    }
                    ChargerOperationalStatus::OperationalObserved => {
                        if !charger_status_current {
                            push_finding(
                                &mut findings,
                                Some(event_id),
                                "CHARGER_OPERATION_NOT_CURRENT",
                                "Operational status is not source-qualified and valid through the planned stop.",
                            );
                            event_indeterminate = true;
                        }
                    }
                    ChargerOperationalStatus::Unknown => {
                        evidence_reference_valid(
                            &charger.status_evidence,
                            &format!("{event_id}.charger.status_evidence"),
                        )?;
                        push_finding(
                            &mut findings,
                            Some(event_id),
                            "CHARGER_OPERATION_UNKNOWN",
                            "No current operational observation establishes charger availability.",
                        );
                        event_indeterminate = true;
                    }
                }

                let site = &charger.site_connection;
                let mut site_import = None;
                let mut site_connection_current = false;
                let mut site_import_evidence_current = false;
                match site.grid_connection_stage {
                    GridConnectionStage::Energized => {
                        site_connection_current = evidence_current_for_window(
                            &site.connection_evidence,
                            input.assessed_at_unix_s,
                            end,
                            "site_connection",
                            &charger.charger_id,
                            trust_store,
                            &format!("{event_id}.site_connection_evidence"),
                        )?;
                        if !site_connection_current {
                            push_finding(
                                &mut findings,
                                Some(event_id),
                                "GRID_ENERGIZATION_EVIDENCE_NOT_CURRENT",
                                "The site connection evidence does not establish an energized connection valid through the planned stop.",
                            );
                            event_indeterminate = true;
                        }
                        match (&site.site_import_power_w, &site.import_power_evidence) {
                            (Some(power), Some(evidence)) => {
                                validate_interval(
                                    *power,
                                    &format!("{event_id}.site_import_power_w"),
                                )?;
                                site_import_evidence_current = evidence_current_for_window(
                                    evidence,
                                    input.assessed_at_unix_s,
                                    end,
                                    "site_import_power",
                                    &charger.charger_id,
                                    trust_store,
                                    &format!("{event_id}.site_import_power_evidence"),
                                )?;
                                if !site_import_evidence_current {
                                    push_finding(
                                        &mut findings,
                                        Some(event_id),
                                        "SITE_IMPORT_POWER_EVIDENCE_NOT_CURRENT",
                                        "The site import-power evidence is not an observed, current bound valid through the stop.",
                                    );
                                    event_indeterminate = true;
                                }
                                site_import = Some(*power);
                            }
                            _ => {
                                push_finding(
                                    &mut findings,
                                    Some(event_id),
                                    "SITE_IMPORT_POWER_UNKNOWN",
                                    "Energization is not accompanied by an evidenced site import-power interval.",
                                );
                                event_indeterminate = true;
                            }
                        }
                    }
                    _ => {
                        evidence_reference_valid(
                            &site.connection_evidence,
                            &format!("{event_id}.site_connection_evidence"),
                        )?;
                        push_finding(
                            &mut findings,
                            Some(event_id),
                            "SITE_NOT_EVIDENCED_ENERGIZED",
                            "An application, budget quote, allocation, or agreement is not proof of energized import capacity.",
                        );
                        event_indeterminate = true;
                    }
                }

                let power_bounds_qualified = pack_capability_qualified
                    && charger_configuration_qualified
                    && evse_power_qualified
                    && conversion_qualified
                    && site_connection_current
                    && site_import_evidence_current;
                let charging_service_current = power_bounds_qualified
                    && charger_status_current
                    && connector_compatible
                    && vehicle_configuration_qualified
                    && charger.operational_status == ChargerOperationalStatus::OperationalObserved;

                if let Some(import_power) = site_import {
                    let site_connector_power_lower =
                        import_power.lower * charger.site_to_connector_efficiency.lower;
                    let site_connector_power_upper =
                        import_power.upper * charger.site_to_connector_efficiency.upper;
                    let guaranteed_connector_power = site_connector_power_lower
                        .min(charger.evse_output_power_w.lower)
                        .min(input.pack.charge_acceptance_power_w.lower);
                    let possible_connector_power = site_connector_power_upper
                        .min(charger.evse_output_power_w.upper)
                        .min(input.pack.charge_acceptance_power_w.upper);
                    let physical_charge_lower =
                        guaranteed_connector_power * duration as f64 / 3600.0;
                    let physical_charge_upper = possible_connector_power * duration as f64 / 3600.0;
                    // An unqualified source can never contribute to the conservative
                    // lower-energy path. Preserve only a conditional upper possibility.
                    let charge_lower = if charging_service_current {
                        stop.requested_charge_energy_wh.min(physical_charge_lower)
                    } else {
                        0.0
                    };
                    let charge_upper = if power_bounds_qualified {
                        stop.requested_charge_energy_wh.min(physical_charge_upper)
                    } else {
                        stop.requested_charge_energy_wh
                    };

                    if power_bounds_qualified
                        && physical_charge_upper + EPSILON < stop.requested_charge_energy_wh
                    {
                        push_finding(
                            &mut findings,
                            Some(event_id),
                            "CHARGE_TARGET_PHYSICALLY_UNAVAILABLE",
                            "Even the upper qualified site/EVSE/vehicle power bound cannot deliver the requested energy within the declared stop duration.",
                        );
                        event_infeasible = true;
                    } else if charge_lower + EPSILON < stop.requested_charge_energy_wh {
                        push_finding(
                            &mut findings,
                            Some(event_id),
                            "CHARGE_TARGET_NOT_GUARANTEED",
                            "The available lower bound, status evidence or site evidence cannot guarantee the requested charge energy.",
                        );
                        event_indeterminate = true;
                    }

                    let resulting_lower = energy.lower + charge_lower;
                    let resulting_upper = energy.upper + charge_upper;
                    if resulting_lower > input.pack.usable_energy_wh.upper + EPSILON {
                        push_finding(
                            &mut findings,
                            Some(event_id),
                            "CHARGE_EXCEEDS_PACK_CAPACITY",
                            "Even the lower resulting-energy bound exceeds the upper usable-capacity bound.",
                        );
                        if energy_bounds_qualified && charging_service_current {
                            event_infeasible = true;
                        } else {
                            event_indeterminate = true;
                        }
                    } else if resulting_lower > input.pack.usable_energy_wh.lower + EPSILON
                        || resulting_upper > input.pack.usable_energy_wh.upper + EPSILON
                    {
                        push_finding(
                            &mut findings,
                            Some(event_id),
                            "CHARGE_CAPACITY_NOT_GUARANTEED",
                            "The resulting-energy interval crosses or exceeds the declared usable-capacity interval.",
                        );
                        event_indeterminate = true;
                    }

                    energy = NumericInterval {
                        lower: resulting_lower,
                        upper: resulting_upper,
                    };
                    delivered_charge = Some(NumericInterval {
                        lower: charge_lower,
                        upper: charge_upper,
                    });
                    energy_bounds_qualified &= charging_service_current;
                } else {
                    delivered_charge = Some(NumericInterval {
                        lower: 0.0,
                        upper: stop.requested_charge_energy_wh,
                    });
                    energy_bounds_qualified = false;
                }
            }
        }

        any_infeasible |= event_infeasible;
        any_indeterminate |= event_indeterminate;
        let disposition = if event_infeasible {
            EventDisposition::Infeasible
        } else if event_indeterminate {
            EventDisposition::Indeterminate
        } else {
            EventDisposition::WithinDeclaredBounds
        };
        event_assessments.push(EventAssessment {
            event_id: event_id.to_owned(),
            event_kind: event.kind().to_owned(),
            disposition,
            energy_after_wh: energy,
            delivered_charge_wh: delivered_charge,
        });
    }

    let disposition = if any_infeasible {
        AssessmentDisposition::Infeasible
    } else if any_indeterminate {
        AssessmentDisposition::Indeterminate
    } else {
        AssessmentDisposition::FeasibleUnderDeclaredBounds
    };

    Ok(Assessment {
        schema_version: ASSESSMENT_SCHEMA_VERSION.into(),
        plan_id: input.plan_id.clone(),
        trust_policy_id: trust_store.policy_id.clone(),
        disposition,
        reserve_floor_wh: input.pack.reserve_floor_wh,
        final_energy_wh: energy,
        event_assessments,
        consumed_evidence: collect_evidence(input, trust_store),
        findings,
        claim_ceiling: "Proposal audit only. This result requires signatures from keys in the separately supplied trust store, but signatures establish issuer-bound claims rather than sensor truth, model validity, grid approval, route legality, vehicle safety, or actuation authority. The referenced artifact bytes are not fetched or independently re-hashed by this module.".into(),
    })
}

fn collect_evidence(input: &RoutePlan, trust_store: &EvidenceTrustStore) -> Vec<EvidenceBinding> {
    fn add(
        output: &mut Vec<EvidenceBinding>,
        report_role: String,
        evidence_role: &str,
        subject_id: &str,
        evidence: &EvidenceRef,
        trust_store: &EvidenceTrustStore,
        assessed_at_unix_s: i64,
    ) {
        let verification = verify_evidence(
            EvidenceVerificationRequest {
                reference_id: &evidence.reference_id,
                digest_sha256: &evidence.digest_sha256,
                source_class: evidence_class_label(evidence.source_class),
                observed_at_unix_s: evidence.observed_at_unix_s,
                valid_until_unix_s: evidence.valid_until_unix_s,
                attestation: evidence.attestation.as_ref(),
                expected_evidence_role: evidence_role,
                expected_subject_id: subject_id,
                assessed_at_unix_s,
            },
            trust_store,
        );
        output.push(EvidenceBinding {
            role: report_role,
            reference_id: evidence.reference_id.clone(),
            digest_sha256: evidence.digest_sha256.clone(),
            source_class: evidence.source_class,
            evidence_role: evidence_role.to_owned(),
            subject_id: subject_id.to_owned(),
            verification_status: verification.status,
            issuer_id: verification.issuer_id,
            issuer_role: verification.issuer_role,
            key_id: verification.key_id,
            observed_at_unix_s: evidence.observed_at_unix_s,
            valid_until_unix_s: evidence.valid_until_unix_s,
        });
    }

    let mut output = Vec::new();
    let assessed_at = input.assessed_at_unix_s;
    add(
        &mut output,
        "vehicle.configuration".into(),
        "vehicle_configuration",
        &input.vehicle.vehicle_configuration_id,
        &input.vehicle.configuration_evidence,
        trust_store,
        assessed_at,
    );
    add(
        &mut output,
        "vehicle.energy_model".into(),
        "vehicle_energy_model",
        &input.vehicle.vehicle_configuration_id,
        &input.vehicle.energy_model_evidence,
        trust_store,
        assessed_at,
    );
    add(
        &mut output,
        "pack.capability".into(),
        "pack_capability",
        &input.pack.pack_id,
        &input.pack.capability_evidence,
        trust_store,
        assessed_at,
    );
    for event in &input.events {
        match event {
            PlanEvent::DriveLeg(leg) => {
                add(
                    &mut output,
                    format!("{}.route", leg.event_id),
                    "route_state",
                    &leg.event_id,
                    &leg.route_evidence,
                    trust_store,
                    assessed_at,
                );
                add(
                    &mut output,
                    format!("{}.energy_estimate", leg.event_id),
                    "leg_energy_estimate",
                    &leg.event_id,
                    &leg.energy_estimate_evidence,
                    trust_store,
                    assessed_at,
                );
            }
            PlanEvent::ChargingStop(stop) => {
                let prefix = format!("{}.charger", stop.event_id);
                let charger_id = &stop.charger.charger_id;
                add(
                    &mut output,
                    format!("{prefix}.status"),
                    "charger_status",
                    charger_id,
                    &stop.charger.status_evidence,
                    trust_store,
                    assessed_at,
                );
                add(
                    &mut output,
                    format!("{prefix}.configuration"),
                    "charger_configuration",
                    charger_id,
                    &stop.charger.configuration_evidence,
                    trust_store,
                    assessed_at,
                );
                add(
                    &mut output,
                    format!("{prefix}.evse_power"),
                    "evse_power_rating",
                    charger_id,
                    &stop.charger.evse_power_evidence,
                    trust_store,
                    assessed_at,
                );
                add(
                    &mut output,
                    format!("{prefix}.conversion"),
                    "conversion_efficiency_model",
                    charger_id,
                    &stop.charger.conversion_evidence,
                    trust_store,
                    assessed_at,
                );
                add(
                    &mut output,
                    format!("{prefix}.site_connection"),
                    "site_connection",
                    charger_id,
                    &stop.charger.site_connection.connection_evidence,
                    trust_store,
                    assessed_at,
                );
                if let Some(evidence) = &stop.charger.site_connection.import_power_evidence {
                    add(
                        &mut output,
                        format!("{prefix}.site_import_power"),
                        "site_import_power",
                        charger_id,
                        evidence,
                        trust_store,
                        assessed_at,
                    );
                }
            }
        }
    }
    output
}

fn require_identifier(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    Ok(())
}

fn validate_nonnegative(value: f64, label: &str) -> Result<(), String> {
    if !value.is_finite() || value < 0.0 {
        return Err(format!("{label} must be finite and non-negative"));
    }
    Ok(())
}

fn validate_interval(value: NumericInterval, label: &str) -> Result<(), String> {
    validate_nonnegative(value.lower, &format!("{label}.lower"))?;
    validate_nonnegative(value.upper, &format!("{label}.upper"))?;
    if value.lower > value.upper {
        return Err(format!("{label}.lower must not exceed upper"));
    }
    Ok(())
}

fn evidence_reference_valid(value: &EvidenceRef, label: &str) -> Result<(), String> {
    require_identifier(&value.reference_id, &format!("{label}.reference_id"))?;
    if value.digest_sha256.len() != 64
        || !value
            .digest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(format!(
            "{label}.digest_sha256 must be 64 lowercase hexadecimal characters"
        ));
    }
    if let (Some(observed), Some(valid_until)) =
        (value.observed_at_unix_s, value.valid_until_unix_s)
        && valid_until < observed
    {
        return Err(format!("{label}: validity ends before observation time"));
    }
    Ok(())
}

fn evidence_class_label(value: EvidenceClass) -> &'static str {
    match value {
        EvidenceClass::Observed => "observed",
        EvidenceClass::QualifiedModel => "qualified_model",
        EvidenceClass::Forecast => "forecast",
        EvidenceClass::Predicted => "predicted",
        EvidenceClass::Scenario => "scenario",
        EvidenceClass::Synthetic => "synthetic",
        EvidenceClass::Unknown => "unknown",
    }
}

fn evidence_class_qualified(
    value: &EvidenceRef,
    expected_role: &str,
    expected_subject_id: &str,
    trust_store: &EvidenceTrustStore,
    assessed_at: i64,
    label: &str,
) -> Result<bool, String> {
    evidence_reference_valid(value, label)?;
    let verification = verify_evidence(
        EvidenceVerificationRequest {
            reference_id: &value.reference_id,
            digest_sha256: &value.digest_sha256,
            source_class: evidence_class_label(value.source_class),
            observed_at_unix_s: value.observed_at_unix_s,
            valid_until_unix_s: value.valid_until_unix_s,
            attestation: value.attestation.as_ref(),
            expected_evidence_role: expected_role,
            expected_subject_id,
            assessed_at_unix_s: assessed_at,
        },
        trust_store,
    );
    Ok(verification.authenticated()
        && matches!(
            value.source_class,
            EvidenceClass::Observed | EvidenceClass::QualifiedModel
        ))
}

fn evidence_current_for_window(
    value: &EvidenceRef,
    assessed_at: i64,
    planned_end: i64,
    expected_role: &str,
    expected_subject_id: &str,
    trust_store: &EvidenceTrustStore,
    label: &str,
) -> Result<bool, String> {
    evidence_reference_valid(value, label)?;
    let verification = verify_evidence(
        EvidenceVerificationRequest {
            reference_id: &value.reference_id,
            digest_sha256: &value.digest_sha256,
            source_class: evidence_class_label(value.source_class),
            observed_at_unix_s: value.observed_at_unix_s,
            valid_until_unix_s: value.valid_until_unix_s,
            attestation: value.attestation.as_ref(),
            expected_evidence_role: expected_role,
            expected_subject_id,
            assessed_at_unix_s: assessed_at,
        },
        trust_store,
    );
    Ok(value.source_class == EvidenceClass::Observed
        && verification.authenticated()
        && value
            .observed_at_unix_s
            .is_some_and(|observed| observed <= assessed_at)
        && value
            .valid_until_unix_s
            .is_some_and(|valid_until| valid_until >= planned_end))
}

fn push_finding(findings: &mut Vec<Finding>, event_id: Option<&str>, code: &str, detail: &str) {
    findings.push(Finding {
        event_id: event_id.map(str::to_owned),
        code: code.to_owned(),
        detail: detail.to_owned(),
    });
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    use ed25519_dalek::{Signer, SigningKey};

    fn test_signing_key() -> SigningKey {
        SigningKey::from_bytes(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ])
    }

    fn test_trust_store() -> EvidenceTrustStore {
        let key = test_signing_key();
        EvidenceTrustStore {
            schema_version: crate::ev_evidence_admission::TRUST_STORE_SCHEMA_VERSION.into(),
            policy_id: "test-only-ev-route-trust-policy".into(),
            trusted_keys: vec![crate::ev_evidence_admission::TrustedEvidenceKey {
                key_id: "test-key-001".into(),
                issuer_id: "test-issuer-001".into(),
                issuer_role: "engineering_evidence_issuer".into(),
                public_key_ed25519_hex: key
                    .verifying_key()
                    .to_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
                valid_from_unix_s: 0,
                valid_until_unix_s: Some(20_000),
                revoked: false,
                authorized_evidence_roles: vec![
                    "vehicle_configuration".into(),
                    "vehicle_energy_model".into(),
                    "pack_capability".into(),
                    "route_state".into(),
                    "leg_energy_estimate".into(),
                    "charger_status".into(),
                    "charger_configuration".into(),
                    "evse_power_rating".into(),
                    "conversion_efficiency_model".into(),
                    "site_connection".into(),
                    "site_import_power".into(),
                ],
            }],
        }
    }

    fn evidence(
        reference_id: &str,
        source_class: EvidenceClass,
        seed: u8,
        evidence_role: &str,
        subject_id: &str,
    ) -> EvidenceRef {
        let digest_sha256 = format!("{seed:02x}").repeat(32);
        let mut attestation = EvidenceAttestation {
            evidence_role: evidence_role.into(),
            subject_id: subject_id.into(),
            issuer_id: "test-issuer-001".into(),
            issuer_role: "engineering_evidence_issuer".into(),
            key_id: "test-key-001".into(),
            issued_at_unix_s: 900,
            signature_ed25519_hex: None,
        };
        let message = crate::ev_evidence_admission::signature_message(
            reference_id,
            &digest_sha256,
            evidence_class_label(source_class),
            Some(800),
            Some(20_000),
            &attestation,
        )
        .expect("evidence signing payload");
        attestation.signature_ed25519_hex = Some(
            test_signing_key()
                .sign(&message)
                .to_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        );
        EvidenceRef {
            reference_id: reference_id.into(),
            digest_sha256,
            source_class,
            observed_at_unix_s: Some(800),
            valid_until_unix_s: Some(20_000),
            attestation: Some(attestation),
        }
    }

    fn assess_test_plan(input: &RoutePlan) -> Result<Assessment, String> {
        let store = test_trust_store();
        let bytes = serde_json::to_vec(&store).expect("serialize test trust store");
        let expected_sha256 = sha256_hex(&bytes);
        assess(input, &store, &expected_sha256)
    }

    fn test_trust_store_bytes() -> Vec<u8> {
        serde_json::to_vec(&test_trust_store()).expect("serialize test trust store")
    }

    fn qualified_plan() -> RoutePlan {
        RoutePlan {
            schema_version: INPUT_SCHEMA_VERSION.into(),
            plan_id: "EV-ROUTE-KNOWN-001".into(),
            assessed_at_unix_s: 900,
            vehicle: VehicleEvidence {
                vehicle_configuration_id: "EV-REF-01-CONFIG-A".into(),
                connector_ids: vec!["CCS2".into()],
                configuration_evidence: evidence(
                    "mycelix:vehicle-config",
                    EvidenceClass::Observed,
                    1,
                    "vehicle_configuration",
                    "EV-REF-01-CONFIG-A",
                ),
                energy_model_evidence: evidence(
                    "symtropy:ev-energy-model",
                    EvidenceClass::QualifiedModel,
                    2,
                    "vehicle_energy_model",
                    "EV-REF-01-CONFIG-A",
                ),
            },
            pack: PackCapability {
                pack_id: "PACK-REF-01-A".into(),
                usable_energy_wh: NumericInterval {
                    lower: 60_000.0,
                    upper: 60_000.0,
                },
                initial_energy_wh: NumericInterval {
                    lower: 45_000.0,
                    upper: 45_000.0,
                },
                reserve_floor_wh: 10_000.0,
                charge_acceptance_power_w: NumericInterval {
                    lower: 40_000.0,
                    upper: 40_000.0,
                },
                capability_evidence: evidence(
                    "symthaea:pack-capability",
                    EvidenceClass::QualifiedModel,
                    3,
                    "pack_capability",
                    "PACK-REF-01-A",
                ),
            },
            events: vec![
                PlanEvent::DriveLeg(Box::new(DriveLeg {
                    kind: "drive_leg".into(),
                    event_id: "LEG-1".into(),
                    planned_start_unix_s: 1_000,
                    duration_seconds: 1_800,
                    route_state: RouteState::OpenObserved,
                    energy_consumption_wh: NumericInterval {
                        lower: 20_000.0,
                        upper: 20_000.0,
                    },
                    route_evidence: evidence(
                        "sol-atlas:road-state-1",
                        EvidenceClass::Observed,
                        4,
                        "route_state",
                        "LEG-1",
                    ),
                    energy_estimate_evidence: evidence(
                        "symtropy:leg-energy-1",
                        EvidenceClass::QualifiedModel,
                        5,
                        "leg_energy_estimate",
                        "LEG-1",
                    ),
                })),
                PlanEvent::ChargingStop(Box::new(ChargingStop {
                    kind: "charging_stop".into(),
                    event_id: "CHARGE-1".into(),
                    planned_start_unix_s: 2_800,
                    duration_seconds: 1_800,
                    requested_charge_energy_wh: 20_000.0,
                    charger: Charger {
                        charger_id: "N3-CHARGER-01".into(),
                        connector_id: "CCS2".into(),
                        operational_status: ChargerOperationalStatus::OperationalObserved,
                        status_evidence: evidence(
                            "sol-atlas:charger-status",
                            EvidenceClass::Observed,
                            6,
                            "charger_status",
                            "N3-CHARGER-01",
                        ),
                        configuration_evidence: evidence(
                            "mycelix:charger-config",
                            EvidenceClass::Observed,
                            7,
                            "charger_configuration",
                            "N3-CHARGER-01",
                        ),
                        evse_output_power_w: NumericInterval {
                            lower: 50_000.0,
                            upper: 50_000.0,
                        },
                        evse_power_evidence: evidence(
                            "mycelix:evse-rating",
                            EvidenceClass::Observed,
                            8,
                            "evse_power_rating",
                            "N3-CHARGER-01",
                        ),
                        site_to_connector_efficiency: NumericInterval {
                            lower: 0.9,
                            upper: 0.9,
                        },
                        conversion_evidence: evidence(
                            "symtropy:conversion-model",
                            EvidenceClass::QualifiedModel,
                            9,
                            "conversion_efficiency_model",
                            "N3-CHARGER-01",
                        ),
                        site_connection: SiteConnection {
                            grid_connection_stage: GridConnectionStage::Energized,
                            site_import_power_w: Some(NumericInterval {
                                lower: 60_000.0,
                                upper: 60_000.0,
                            }),
                            connection_evidence: evidence(
                                "sol-atlas:site-energization",
                                EvidenceClass::Observed,
                                10,
                                "site_connection",
                                "N3-CHARGER-01",
                            ),
                            import_power_evidence: Some(evidence(
                                "sol-atlas:site-import-limit",
                                EvidenceClass::Observed,
                                11,
                                "site_import_power",
                                "N3-CHARGER-01",
                            )),
                        },
                    },
                })),
                PlanEvent::DriveLeg(Box::new(DriveLeg {
                    kind: "drive_leg".into(),
                    event_id: "LEG-2".into(),
                    planned_start_unix_s: 4_600,
                    duration_seconds: 1_800,
                    route_state: RouteState::OpenObserved,
                    energy_consumption_wh: NumericInterval {
                        lower: 30_000.0,
                        upper: 30_000.0,
                    },
                    route_evidence: evidence(
                        "sol-atlas:road-state-2",
                        EvidenceClass::Observed,
                        12,
                        "route_state",
                        "LEG-2",
                    ),
                    energy_estimate_evidence: evidence(
                        "symtropy:leg-energy-2",
                        EvidenceClass::QualifiedModel,
                        13,
                        "leg_energy_estimate",
                        "LEG-2",
                    ),
                })),
            ],
        }
    }

    #[test]
    fn qualified_route_plan_passes_only_under_declared_bounds() {
        let result = assess_test_plan(&qualified_plan()).expect("valid plan");
        assert_eq!(
            result.disposition,
            AssessmentDisposition::FeasibleUnderDeclaredBounds
        );
        assert!((result.final_energy_wh.lower - 15_000.0).abs() < EPSILON);
        assert!((result.final_energy_wh.upper - 15_000.0).abs() < EPSILON);
        assert!(result.findings.is_empty());
        assert!(
            result
                .consumed_evidence
                .iter()
                .all(|binding| binding.verification_status == "AUTHENTICATED")
        );
    }

    #[test]
    fn synthetic_fixture_is_never_promoted_to_feasible() {
        let bytes = include_str!("../../schemas/examples/ev-route-assurance-v0.1.synthetic.json");
        let trust_bytes = test_trust_store_bytes();
        let parsed_trust_store: EvidenceTrustStore =
            serde_json::from_slice(&trust_bytes).expect("test trust store JSON");
        let canonical_trust_store_bytes =
            serde_json::to_vec(&parsed_trust_store).expect("canonical trust store");
        let trust_store_pin = sha256_hex(&canonical_trust_store_bytes);
        let envelope = assess_json(bytes.as_bytes(), &trust_bytes, &trust_store_pin)
            .expect("fixture is valid JSON");
        assert_eq!(
            envelope.assessment.disposition,
            AssessmentDisposition::Indeterminate
        );
        assert!(envelope.assessment.final_energy_wh.lower < 10_000.0);
        assert!((envelope.assessment.final_energy_wh.upper - 15_000.0).abs() < EPSILON);
        assert_eq!(envelope.receipt.input_sha256.len(), 64);
        assert_eq!(envelope.receipt.assessment_sha256.len(), 64);
        assert_eq!(envelope.receipt.trust_store_sha256.len(), 64);

        assert_eq!(
            envelope.assessment.trust_policy_id,
            "test-only-ev-route-trust-policy"
        );
        assert_eq!(envelope.assessment.consumed_evidence.len(), 13);
    }

    #[test]
    fn trust_store_substitution_is_rejected_without_the_pinned_digest() {
        let bytes = include_str!("../../schemas/examples/ev-route-assurance-v0.1.synthetic.json");
        let trust_bytes = test_trust_store_bytes();
        assert!(assess_json(bytes.as_bytes(), &trust_bytes, &"00".repeat(32)).is_err());
    }

    #[test]
    fn connector_mismatch_is_infeasible() {
        let mut plan = qualified_plan();
        if let PlanEvent::ChargingStop(stop) = &mut plan.events[1] {
            stop.charger.connector_id = "CHAdeMO".into();
        }
        let result = assess_test_plan(&plan).expect("validly structured plan");
        assert_eq!(result.disposition, AssessmentDisposition::Infeasible);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.code == "CONNECTOR_INCOMPATIBLE")
        );
    }

    #[test]
    fn an_agreement_is_not_energization() {
        let mut plan = qualified_plan();
        if let PlanEvent::ChargingStop(stop) = &mut plan.events[1] {
            stop.charger.site_connection.grid_connection_stage =
                GridConnectionStage::SignedAgreement;
        }
        let result = assess_test_plan(&plan).expect("validly structured plan");
        assert_eq!(result.disposition, AssessmentDisposition::Indeterminate);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.code == "SITE_NOT_EVIDENCED_ENERGIZED")
        );
    }

    #[test]
    fn stale_charger_status_is_indeterminate_not_available() {
        let mut plan = qualified_plan();
        if let PlanEvent::ChargingStop(stop) = &mut plan.events[1] {
            stop.charger.status_evidence.valid_until_unix_s = Some(2_000);
        }
        let result = assess_test_plan(&plan).expect("validly structured plan");
        assert_eq!(result.disposition, AssessmentDisposition::Indeterminate);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.code == "CHARGER_OPERATION_NOT_CURRENT")
        );
    }

    #[test]
    fn definite_reserve_shortfall_is_infeasible() {
        let mut plan = qualified_plan();
        if let PlanEvent::DriveLeg(leg) = &mut plan.events[2] {
            leg.energy_consumption_wh = NumericInterval {
                lower: 40_000.0,
                upper: 40_000.0,
            };
        }
        let result = assess_test_plan(&plan).expect("validly structured plan");
        assert_eq!(result.disposition, AssessmentDisposition::Infeasible);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.code == "RESERVE_DEFINITELY_BREACHED")
        );
    }

    #[test]
    fn planned_charge_must_fit_conservative_power_bound() {
        let mut plan = qualified_plan();
        if let PlanEvent::ChargingStop(stop) = &mut plan.events[1] {
            stop.requested_charge_energy_wh = 21_000.0;
        }
        let result = assess_test_plan(&plan).expect("validly structured plan");
        assert_eq!(result.disposition, AssessmentDisposition::Infeasible);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.code == "CHARGE_TARGET_PHYSICALLY_UNAVAILABLE")
        );
    }

    #[test]
    fn malformed_digest_is_rejected() {
        let mut plan = qualified_plan();
        plan.pack.capability_evidence.digest_sha256 = "not-a-digest".into();
        assert!(assess_test_plan(&plan).is_err());
    }

    #[test]
    fn fabricated_observed_evidence_cannot_produce_a_feasible_route() {
        let mut plan = qualified_plan();
        plan.vehicle.configuration_evidence.attestation = None;
        let result = assess_test_plan(&plan).expect("structurally valid plan");
        assert_eq!(result.disposition, AssessmentDisposition::Indeterminate);
        assert!(result.consumed_evidence.iter().any(|binding| {
            binding.role == "vehicle.configuration" && binding.verification_status == "UNATTESTED"
        }));
    }

    #[test]
    fn digest_tampering_breaks_route_evidence_admission() {
        let mut plan = qualified_plan();
        plan.vehicle.configuration_evidence.digest_sha256 = "ff".repeat(32);
        let result = assess_test_plan(&plan).expect("structurally valid plan");
        assert_eq!(result.disposition, AssessmentDisposition::Indeterminate);
        assert!(result.consumed_evidence.iter().any(|binding| {
            binding.role == "vehicle.configuration"
                && binding.verification_status == "REJECTED_SIGNATURE"
        }));
    }

    #[test]
    fn overlapping_events_are_rejected() {
        let mut plan = qualified_plan();
        if let PlanEvent::ChargingStop(stop) = &mut plan.events[1] {
            stop.planned_start_unix_s = 2_700;
        }
        assert!(assess_test_plan(&plan).is_err());
    }
}
