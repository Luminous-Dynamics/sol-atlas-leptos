// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Fail-closed adapter from a Symtropy v0.1 energy candidate to a Sol-Atlas
//! exact-dispatch subproblem.
//!
//! Only the synthetic single-site electrical subset is supported: at most one
//! PV source, one grid import source, one battery, and essential/flexible
//! electric services expressed in kWh. The resulting dispatch solution does
//! not certify the full candidate's physics, economics, network, or safety.

use crate::energy_dispatch::{
    DispatchBattery, DispatchIntervalInput, EnergyDispatchRequest,
};
use crate::energy_pathway::EvidenceClass;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const MAPPING_SCHEMA_VERSION: &str = "energy-dispatch-mapping-v0.1";
const CANDIDATE_SCHEMA_VERSION: &str = "energy-system-candidate-v0.1";
const DISPATCH_SCHEMA_VERSION: &str = "energy-dispatch-v0.1";
const EFFICIENCY_ABSOLUTE_TOLERANCE: f64 = 1.0e-9;
const ENERGY_ABSOLUTE_TOLERANCE_KWH: f64 = 1.0e-9;

/// External, explicit assumptions needed to compile the bounded dispatch subproblem.
/// A missing tariff or grid-carbon factor is never silently interpreted as zero.
/// Zero is allowed only as an explicit value with a non-empty basis statement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchMappingConfig {
    pub schema_version: String,
    pub energy_quantum_kwh: f64,
    pub max_candidate_expansions: u64,
    pub grid_cost_units_per_quantum: u32,
    pub grid_carbon_grams_per_quantum: u32,
    pub factor_basis: String,
    pub synthetic: bool,
}

impl DispatchMappingConfig {
    /// Test-only defaults are intentionally explicit synthetic factors, not field data.
    pub fn synthetic_fixture() -> Self {
        Self {
            schema_version: MAPPING_SCHEMA_VERSION.to_owned(),
            energy_quantum_kwh: 1.0,
            max_candidate_expansions: 100_000,
            grid_cost_units_per_quantum: 0,
            grid_carbon_grams_per_quantum: 0,
            factor_basis: "Explicit synthetic fixture factors only; not a real tariff or emissions factor.".to_owned(),
            synthetic: true,
        }
    }
}

/// A reproducible mapping receipt around the reduced dispatch request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnergyDesignCompilation {
    pub receipt_schema_version: String,
    pub mapping_schema_version: String,
    pub candidate_id: String,
    pub candidate_sha256: String,
    pub mapping_config_sha256: String,
    pub synthetic: bool,
    pub supported_boundary: String,
    pub claim_ceiling: String,
    pub dispatch_request: EnergyDispatchRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnergyDesignAdapterError {
    pub reason: String,
}

impl EnergyDesignAdapterError {
    fn new(reason: impl Into<String>) -> Self {
        Self { reason: reason.into() }
    }
}

impl fmt::Display for EnergyDesignAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "energy design adapter rejected candidate: {}", self.reason)
    }
}

impl Error for EnergyDesignAdapterError {}

// The adapter intentionally models only fields it consumes. The complete candidate
// contract remains owned by Symtropy's schema; unknown document fields are ignored here
// only after the caller has separately validated the full candidate against that schema.
#[derive(Debug, Deserialize)]
struct CandidateDocument {
    schema_version: String,
    candidate_id: String,
    synthetic: bool,
    evidence_class: String,
    horizon: CandidateHorizon,
    components: Vec<CandidateComponent>,
    connections: Vec<CandidateConnection>,
    services: Vec<CandidateService>,
    scenarios: Vec<CandidateScenario>,
}

#[derive(Debug, Deserialize)]
struct CandidateHorizon {
    start_unix_seconds: i64,
    end_unix_seconds: i64,
    interval_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct CandidateComponent {
    component_id: String,
    kind: String,
    ports: Vec<CandidatePort>,
    parameter_claims: Vec<CandidateParameterClaim>,
}

#[derive(Debug, Deserialize)]
struct CandidatePort {
    port_id: String,
    direction: String,
    carrier: String,
    unit: String,
    temperature_grade_c: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct CandidateParameterClaim {
    parameter_id: String,
    value: Value,
    unit: String,
    evidence_class: String,
    source_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CandidateConnection {
    connection_id: String,
    from_component: String,
    from_port: String,
    to_component: String,
    to_port: String,
    carrier: String,
}

#[derive(Debug, Deserialize)]
struct CandidateService {
    service_id: String,
    service_kind: String,
    component_id: String,
    essential: bool,
    unit: String,
    minimum_delivery_temperature_c: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct CandidateScenario {
    scenario_id: String,
    evidence_class: String,
    intervals: Vec<CandidateScenarioInterval>,
}

#[derive(Debug, Deserialize)]
struct CandidateScenarioInterval {
    interval_id: String,
    start_unix_seconds: i64,
    duration_seconds: u64,
    grid_outage: bool,
    resource_available_kwh: Vec<CandidateResourceAvailability>,
    service_requested: Vec<CandidateServiceRequest>,
}

#[derive(Debug, Deserialize)]
struct CandidateResourceAvailability {
    component_id: String,
    available_kwh: f64,
}

#[derive(Debug, Deserialize)]
struct CandidateServiceRequest {
    service_id: String,
    requested_quantity: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceKind {
    Photovoltaic,
    GridImport,
}

#[derive(Clone, Debug)]
struct SourceDescription {
    kind: SourceKind,
    energy_cap_kwh: f64,
    power_cap_kw: Option<f64>,
}

#[derive(Clone, Debug)]
struct BatteryDescription {
    capacity_kwh: f64,
    initial_soc_kwh: f64,
    terminal_reserve_kwh: f64,
    max_charge_power_kw: f64,
    max_discharge_power_kw: f64,
    charge_efficiency: f64,
    discharge_efficiency: f64,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn finite_nonnegative(value: f64, label: &str) -> Result<(), EnergyDesignAdapterError> {
    if !value.is_finite() || value < 0.0 {
        return Err(EnergyDesignAdapterError::new(format!(
            "{label} must be finite and non-negative"
        )));
    }
    Ok(())
}

fn numeric_claim(
    component: &CandidateComponent,
    parameter_id: &str,
    expected_unit: &str,
    required: bool,
) -> Result<Option<f64>, EnergyDesignAdapterError> {
    let matches: Vec<_> = component
        .parameter_claims
        .iter()
        .filter(|claim| claim.parameter_id == parameter_id)
        .collect();
    if matches.len() > 1 {
        return Err(EnergyDesignAdapterError::new(format!(
            "component {} has duplicate parameter claim {parameter_id}",
            component.component_id
        )));
    }
    let Some(claim) = matches.first() else {
        if required {
            return Err(EnergyDesignAdapterError::new(format!(
                "component {} lacks explicit parameter claim {parameter_id}",
                component.component_id
            )));
        }
        return Ok(None);
    };
    if claim.unit != expected_unit {
        return Err(EnergyDesignAdapterError::new(format!(
            "parameter {parameter_id} on {} has unit {}, expected {expected_unit}",
            component.component_id, claim.unit
        )));
    }
    if claim.evidence_class != "analyst_assumption" || claim.source_ref.is_some() {
        return Err(EnergyDesignAdapterError::new(format!(
            "v0.1 adapter accepts only explicitly assumption-classed synthetic parameters; {}.{parameter_id} violates that boundary",
            component.component_id
        )));
    }
    let value = claim.value.as_f64().ok_or_else(|| {
        EnergyDesignAdapterError::new(format!(
            "parameter {parameter_id} on {} must have a numeric value",
            component.component_id
        ))
    })?;
    finite_nonnegative(value, parameter_id)?;
    Ok(Some(value))
}

fn quantize_kwh(
    value_kwh: f64,
    quantum_kwh: f64,
    label: &str,
) -> Result<u16, EnergyDesignAdapterError> {
    finite_nonnegative(value_kwh, label)?;
    let raw_quanta = value_kwh / quantum_kwh;
    if !raw_quanta.is_finite() || raw_quanta < 0.0 {
        return Err(EnergyDesignAdapterError::new(format!(
            "{label} cannot be represented using the declared energy quantum"
        )));
    }
    let rounded = raw_quanta.round();
    let reconstructed = rounded * quantum_kwh;
    let tolerance = ENERGY_ABSOLUTE_TOLERANCE_KWH.max(quantum_kwh.abs() * 1.0e-9);
    if (value_kwh - reconstructed).abs() > tolerance {
        return Err(EnergyDesignAdapterError::new(format!(
            "{label}={value_kwh} kWh is not an integer multiple of the declared {} kWh quantum",
            quantum_kwh
        )));
    }
    if rounded > f64::from(u16::MAX) {
        return Err(EnergyDesignAdapterError::new(format!(
            "{label} exceeds the u16 dispatch range"
        )));
    }
    Ok(rounded as u16)
}

fn rational_efficiency(value: f64, label: &str) -> Result<(u16, u16), EnergyDesignAdapterError> {
    if !value.is_finite() || value <= 0.0 || value > 1.0 {
        return Err(EnergyDesignAdapterError::new(format!(
            "{label} must lie in (0, 1]"
        )));
    }
    // Find the smallest denominator representable by the integer dispatch model
    // that reproduces the declared decimal within an explicit tolerance.
    for denominator in 1_u32..=u32::from(u16::MAX) {
        let numerator = (value * f64::from(denominator)).round();
        if numerator < 1.0 || numerator > f64::from(denominator) {
            continue;
        }
        let represented = numerator / f64::from(denominator);
        if (represented - value).abs() <= EFFICIENCY_ABSOLUTE_TOLERANCE {
            let numerator = numerator as u16;
            let denominator = denominator as u16;
            let gcd = gcd_u16(numerator, denominator);
            return Ok((numerator / gcd, denominator / gcd));
        }
    }
    Err(EnergyDesignAdapterError::new(format!(
        "{label}={value} cannot be represented by u16 rational efficiency within tolerance"
    )))
}

fn gcd_u16(mut a: u16, mut b: u16) -> u16 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.max(1)
}

fn claim_by_id<'a>(
    component: &'a CandidateComponent,
    parameter_id: &str,
) -> Option<&'a CandidateParameterClaim> {
    component
        .parameter_claims
        .iter()
        .find(|claim| claim.parameter_id == parameter_id)
}

fn validate_config(config: &DispatchMappingConfig) -> Result<(), EnergyDesignAdapterError> {
    if config.schema_version != MAPPING_SCHEMA_VERSION {
        return Err(EnergyDesignAdapterError::new(format!(
            "unsupported mapping schema version {}",
            config.schema_version
        )));
    }
    if !config.energy_quantum_kwh.is_finite() || config.energy_quantum_kwh <= 0.0 {
        return Err(EnergyDesignAdapterError::new(
            "energy quantum must be finite and greater than zero",
        ));
    }
    if config.max_candidate_expansions == 0 || config.max_candidate_expansions > 1_000_000 {
        return Err(EnergyDesignAdapterError::new(
            "max_candidate_expansions must be in 1..=1,000,000",
        ));
    }
    if config.factor_basis.trim().is_empty() {
        return Err(EnergyDesignAdapterError::new(
            "grid cost/carbon factors require an explicit non-empty basis",
        ));
    }
    if !config.synthetic {
        return Err(EnergyDesignAdapterError::new(
            "v0.1 mapping config must be explicitly synthetic; field-factor mappings are not yet qualified",
        ));
    }
    Ok(())
}

fn source_descriptions(
    candidate: &CandidateDocument,
) -> Result<(BTreeMap<String, SourceDescription>, Option<BatteryDescription>), EnergyDesignAdapterError> {
    let mut sources = BTreeMap::new();
    let mut photovoltaic_count = 0;
    let mut grid_count = 0;
    let mut battery = None;

    for component in &candidate.components {
        match component.kind.as_str() {
            "source" => {
                let pv_claim = claim_by_id(component, "interval_available_energy_cap_kwh");
                let grid_claim = claim_by_id(component, "interval_import_energy_cap_kwh");
                let source = match (pv_claim, grid_claim) {
                    (Some(pv), None) => {
                        if pv.unit != "kWh" {
                            return Err(EnergyDesignAdapterError::new(format!(
                                "PV source {} must express its interval cap in kWh",
                                component.component_id
                            )));
                        }
                        photovoltaic_count += 1;
                        SourceDescription {
                            kind: SourceKind::Photovoltaic,
                            energy_cap_kwh: numeric_claim(
                                component,
                                "interval_available_energy_cap_kwh",
                                "kWh",
                                true,
                            )?.expect("required PV cap"),
                            power_cap_kw: numeric_claim(
                                component,
                                "rated_power_kw",
                                "kW",
                                true,
                            )?,
                        }
                    }
                    (None, Some(grid)) => {
                        if grid.unit != "kWh" {
                            return Err(EnergyDesignAdapterError::new(format!(
                                "grid source {} must express its interval cap in kWh",
                                component.component_id
                            )));
                        }
                        grid_count += 1;
                        SourceDescription {
                            kind: SourceKind::GridImport,
                            energy_cap_kwh: numeric_claim(
                                component,
                                "interval_import_energy_cap_kwh",
                                "kWh",
                                true,
                            )?.expect("required grid cap"),
                            power_cap_kw: numeric_claim(
                                component,
                                "grid_import_power_cap_kw",
                                "kW",
                                true,
                            )?,
                        }
                    }
                    _ => {
                        return Err(EnergyDesignAdapterError::new(format!(
                            "source {} lacks one unambiguous PV or grid-import claim; other source types are unsupported in v0.1",
                            component.component_id
                        )));
                    }
                };
                for port in &component.ports {
                    if port.carrier != "electricity"
                        || port.unit != "kWh"
                        || port.direction != "output"
                        || port.temperature_grade_c.is_some()
                    {
                        return Err(EnergyDesignAdapterError::new(format!(
                            "source {} has a port outside the supported electricity/kWh boundary",
                            component.component_id
                        )));
                    }
                }
                sources.insert(component.component_id.clone(), source);
            }
            "storage" => {
                if battery.is_some() {
                    return Err(EnergyDesignAdapterError::new(
                        "multiple storage components are unsupported in v0.1",
                    ));
                }
                let required_claims = [
                    ("capacity_kwh", "kWh"),
                    ("initial_soc_kwh", "kWh"),
                    ("terminal_reserve_kwh", "kWh"),
                    ("max_charge_power_kw", "kW"),
                    ("max_discharge_power_kw", "kW"),
                    ("charge_efficiency", "unitless"),
                    ("discharge_efficiency", "unitless"),
                ];
                for (id, unit) in required_claims {
                    numeric_claim(component, id, unit, true)?;
                }
                for port in &component.ports {
                    if port.carrier != "electricity"
                        || port.unit != "kWh"
                        || port.temperature_grade_c.is_some()
                        || !matches!(port.direction.as_str(), "input" | "output" | "bidirectional")
                    {
                        return Err(EnergyDesignAdapterError::new(format!(
                            "storage {} has a port outside the supported electricity/kWh boundary",
                            component.component_id
                        )));
                    }
                }
                let capacity_kwh = numeric_claim(component, "capacity_kwh", "kWh", true)?.unwrap();
                let initial_soc_kwh = numeric_claim(component, "initial_soc_kwh", "kWh", true)?.unwrap();
                let terminal_reserve_kwh = numeric_claim(component, "terminal_reserve_kwh", "kWh", true)?.unwrap();
                let max_charge_power_kw = numeric_claim(component, "max_charge_power_kw", "kW", true)?.unwrap();
                let max_discharge_power_kw = numeric_claim(component, "max_discharge_power_kw", "kW", true)?.unwrap();
                let charge_efficiency = numeric_claim(component, "charge_efficiency", "unitless", true)?.unwrap();
                let discharge_efficiency = numeric_claim(component, "discharge_efficiency", "unitless", true)?.unwrap();
                if initial_soc_kwh > capacity_kwh || terminal_reserve_kwh > capacity_kwh {
                    return Err(EnergyDesignAdapterError::new(format!(
                        "storage {} initial SOC or terminal reserve exceeds capacity",
                        component.component_id
                    )));
                }
                if claim_by_id(component, "charge_efficiency").is_none()
                    || claim_by_id(component, "discharge_efficiency").is_none()
                {
                    return Err(EnergyDesignAdapterError::new(format!(
                        "storage {} lacks explicit efficiencies",
                        component.component_id
                    )));
                }
                battery = Some(BatteryDescription {
                    capacity_kwh,
                    initial_soc_kwh,
                    terminal_reserve_kwh,
                    max_charge_power_kw,
                    max_discharge_power_kw,
                    charge_efficiency,
                    discharge_efficiency,
                });
            }
            "sink" => {
                for port in &component.ports {
                    if port.carrier != "electricity"
                        || port.unit != "kWh"
                        || port.direction != "input"
                        || port.temperature_grade_c.is_some()
                    {
                        return Err(EnergyDesignAdapterError::new(format!(
                            "sink {} has a port outside the supported electricity/kWh boundary",
                            component.component_id
                        )));
                    }
                }
            }
            other => {
                return Err(EnergyDesignAdapterError::new(format!(
                    "component kind {other} is unsupported by the bounded electrical adapter"
                )));
            }
        }

        for claim in &component.parameter_claims {
            if claim.evidence_class != "analyst_assumption" || claim.source_ref.is_some() {
                return Err(EnergyDesignAdapterError::new(format!(
                    "v0.1 only maps synthetic assumption-classed parameter claims; {}.{} has a different evidence class",
                    component.component_id, claim.parameter_id
                )));
            }
        }
    }

    if photovoltaic_count > 1 || grid_count > 1 {
        return Err(EnergyDesignAdapterError::new(
            "v0.1 supports at most one PV source and one grid-import source",
        ));
    }
    Ok((sources, battery))
}

fn validate_candidate_topology(candidate: &CandidateDocument) -> Result<(), EnergyDesignAdapterError> {
    if candidate.schema_version != CANDIDATE_SCHEMA_VERSION {
        return Err(EnergyDesignAdapterError::new(format!(
            "unsupported candidate schema version {}",
            candidate.schema_version
        )));
    }
    if candidate.candidate_id.trim().is_empty() {
        return Err(EnergyDesignAdapterError::new("candidate ID is empty"));
    }
    if !candidate.synthetic || candidate.evidence_class != "analyst_assumption" {
        return Err(EnergyDesignAdapterError::new(
            "v0.1 compiles synthetic analyst-assumption candidates only",
        ));
    }
    if candidate.components.is_empty() || candidate.services.is_empty() {
        return Err(EnergyDesignAdapterError::new(
            "candidate must have components and service contracts",
        ));
    }
    if candidate.scenarios.len() != 1 {
        return Err(EnergyDesignAdapterError::new(
            "v0.1 maps exactly one scenario; scenario-tree and multi-scenario semantics are unsupported",
        ));
    }
    if candidate.horizon.interval_seconds == 0
        || candidate.horizon.end_unix_seconds <= candidate.horizon.start_unix_seconds
    {
        return Err(EnergyDesignAdapterError::new("candidate horizon is invalid"));
    }

    let component_kinds: BTreeMap<&str, &str> = candidate
        .components
        .iter()
        .map(|component| (component.component_id.as_str(), component.kind.as_str()))
        .collect();
    if component_kinds.len() != candidate.components.len() {
        return Err(EnergyDesignAdapterError::new("duplicate component IDs"));
    }

    let mut port_map = BTreeMap::<(String, String), &CandidatePort>::new();
    for component in &candidate.components {
        for port in &component.ports {
            if port_map
                .insert((component.component_id.clone(), port.port_id.clone()), port)
                .is_some()
            {
                return Err(EnergyDesignAdapterError::new(format!(
                    "duplicate port {}.{}",
                    component.component_id, port.port_id
                )));
            }
        }
    }
    let mut connection_ids = BTreeSet::new();
    for connection in &candidate.connections {
        if !connection_ids.insert(connection.connection_id.as_str()) {
            return Err(EnergyDesignAdapterError::new(format!(
                "duplicate connection ID {}",
                connection.connection_id
            )));
        }
        if connection.carrier != "electricity" {
            return Err(EnergyDesignAdapterError::new(format!(
                "connection {} is not electrical",
                connection.connection_id
            )));
        }
        let source = port_map.get(&(
            connection.from_component.clone(),
            connection.from_port.clone(),
        ));
        let target = port_map.get(&(
            connection.to_component.clone(),
            connection.to_port.clone(),
        ));
        let (Some(source), Some(target)) = (source, target) else {
            return Err(EnergyDesignAdapterError::new(format!(
                "connection {} references an unknown port",
                connection.connection_id
            )));
        };
        if !matches!(source.direction.as_str(), "output" | "bidirectional")
            || !matches!(target.direction.as_str(), "input" | "bidirectional")
        {
            return Err(EnergyDesignAdapterError::new(format!(
                "connection {} has an incompatible port direction",
                connection.connection_id
            )));
        }
        if source.carrier != target.carrier
            || source.carrier != connection.carrier
            || source.unit != target.unit
            || source.unit != "kWh"
            || source.temperature_grade_c != target.temperature_grade_c
        {
            return Err(EnergyDesignAdapterError::new(format!(
                "connection {} has incompatible carrier, unit, or temperature grade",
                connection.connection_id
            )));
        }
    }

    let mut service_ids = BTreeSet::new();
    for service in &candidate.services {
        if !service_ids.insert(service.service_id.as_str()) {
            return Err(EnergyDesignAdapterError::new(format!(
                "duplicate service ID {}",
                service.service_id
            )));
        }
        if service.service_kind != "electricity"
            || service.unit != "kWh"
            || service.minimum_delivery_temperature_c.is_some()
        {
            return Err(EnergyDesignAdapterError::new(format!(
                "service {} is outside the supported electrical kWh boundary",
                service.service_id
            )));
        }
        if component_kinds.get(service.component_id.as_str()) != Some(&"sink") {
            return Err(EnergyDesignAdapterError::new(format!(
                "service {} must terminate at a declared sink",
                service.service_id
            )));
        }
    }

    let scenario = &candidate.scenarios[0];
    if scenario.evidence_class != "analyst_assumption" {
        return Err(EnergyDesignAdapterError::new(
            "v0.1 only maps analyst-assumption scenarios",
        ));
    }
    if scenario.intervals.is_empty() {
        return Err(EnergyDesignAdapterError::new("scenario has no intervals"));
    }
    let mut expected_start = candidate.horizon.start_unix_seconds;
    let interval_seconds = candidate.horizon.interval_seconds;
    for interval in &scenario.intervals {
        if interval.duration_seconds != interval_seconds {
            return Err(EnergyDesignAdapterError::new(
                "variable interval durations are unsupported because dispatch power limits are fixed per step",
            ));
        }
        if interval.start_unix_seconds != expected_start {
            return Err(EnergyDesignAdapterError::new(format!(
                "scenario chronology gap/overlap at interval {}",
                interval.interval_id
            )));
        }
        expected_start = interval
            .start_unix_seconds
            .checked_add(i64::try_from(interval.duration_seconds).map_err(|_| {
                EnergyDesignAdapterError::new("interval duration overflows i64")
            })?)
            .ok_or_else(|| EnergyDesignAdapterError::new("scenario chronology overflows i64"))?;

        let mut availability_ids = BTreeSet::new();
        for availability in &interval.resource_available_kwh {
            if !availability_ids.insert(availability.component_id.as_str()) {
                return Err(EnergyDesignAdapterError::new(format!(
                    "duplicate resource sample for {} in interval {}",
                    availability.component_id, interval.interval_id
                )));
            }
            if !component_kinds.contains_key(availability.component_id.as_str()) {
                return Err(EnergyDesignAdapterError::new(format!(
                    "unknown resource component {}",
                    availability.component_id
                )));
            }
            finite_nonnegative(
                availability.available_kwh,
                "resource availability",
            )?;
        }

        let mut request_ids = BTreeSet::new();
        for service_request in &interval.service_requested {
            if !request_ids.insert(service_request.service_id.as_str()) {
                return Err(EnergyDesignAdapterError::new(format!(
                    "duplicate service request {} in interval {}",
                    service_request.service_id, interval.interval_id
                )));
            }
            if !service_ids.contains(service_request.service_id.as_str()) {
                return Err(EnergyDesignAdapterError::new(format!(
                    "unknown requested service {}",
                    service_request.service_id
                )));
            }
            finite_nonnegative(service_request.requested_quantity, "requested service")?;
        }
        if request_ids != service_ids {
            return Err(EnergyDesignAdapterError::new(format!(
                "interval {} must contain exactly one request for every service",
                interval.interval_id
            )));
        }
    }
    if expected_start != candidate.horizon.end_unix_seconds {
        return Err(EnergyDesignAdapterError::new(
            "scenario does not cover the declared horizon exactly",
        ));
    }
    Ok(())
}

/// Compile a candidate document plus explicit synthetic mapping assumptions into a
/// bounded dispatch request. The candidate bytes and mapping settings are digest-bound.
/// Unsupported design features return an error; they are never silently dropped.
pub fn compile_energy_candidate(
    raw_candidate: &[u8],
    config: &DispatchMappingConfig,
) -> Result<EnergyDesignCompilation, EnergyDesignAdapterError> {
    validate_config(config)?;
    let candidate: CandidateDocument = serde_json::from_slice(raw_candidate)
        .map_err(|error| EnergyDesignAdapterError::new(format!("invalid candidate JSON: {error}")))?;
    validate_candidate_topology(&candidate)?;
    let (sources, battery_description) = source_descriptions(&candidate)?;

    let source_components: BTreeSet<&str> = sources.keys().map(String::as_str).collect();
    let seconds = candidate.horizon.interval_seconds;
    let interval_hours = seconds as f64 / 3600.0;
    if !interval_hours.is_finite() || interval_hours <= 0.0 {
        return Err(EnergyDesignAdapterError::new(
            "interval duration cannot be converted to hours",
        ));
    }

    let battery = if let Some(description) = battery_description {
        let capacity_quanta = quantize_kwh(
            description.capacity_kwh,
            config.energy_quantum_kwh,
            "battery capacity",
        )?;
        let initial_soc_quanta = quantize_kwh(
            description.initial_soc_kwh,
            config.energy_quantum_kwh,
            "battery initial SOC",
        )?;
        let terminal_reserve_quanta = quantize_kwh(
            description.terminal_reserve_kwh,
            config.energy_quantum_kwh,
            "battery terminal reserve",
        )?;
        let charge_limit_kwh = description.max_charge_power_kw * interval_hours;
        let discharge_limit_kwh = description.max_discharge_power_kw * interval_hours;
        let charge_input_limit_quanta = quantize_kwh(
            charge_limit_kwh,
            config.energy_quantum_kwh,
            "battery per-interval charge power limit",
        )?;
        let discharge_withdrawal_limit_quanta = quantize_kwh(
            discharge_limit_kwh,
            config.energy_quantum_kwh,
            "battery per-interval discharge power limit",
        )?;
        let (charge_numerator, charge_denominator) =
            rational_efficiency(description.charge_efficiency, "charge efficiency")?;
        let (discharge_numerator, discharge_denominator) =
            rational_efficiency(description.discharge_efficiency, "discharge efficiency")?;

        if initial_soc_quanta > capacity_quanta || terminal_reserve_quanta > capacity_quanta {
            return Err(EnergyDesignAdapterError::new(
                "quantized battery initial SOC or terminal reserve exceeds capacity",
            ));
        }
        DispatchBattery {
            capacity_quanta,
            initial_soc_quanta,
            charge_input_limit_quanta,
            discharge_withdrawal_limit_quanta,
            charge_efficiency_numerator: charge_numerator,
            charge_efficiency_denominator: charge_denominator,
            discharge_efficiency_numerator: discharge_numerator,
            discharge_efficiency_denominator: discharge_denominator,
            terminal_reserve_quanta,
        }
    } else {
        // An explicitly absent storage component means a zero-storage architecture;
        // this is not a default for an omitted battery capacity or power claim.
        DispatchBattery {
            capacity_quanta: 0,
            initial_soc_quanta: 0,
            charge_input_limit_quanta: 0,
            discharge_withdrawal_limit_quanta: 0,
            charge_efficiency_numerator: 1,
            charge_efficiency_denominator: 1,
            discharge_efficiency_numerator: 1,
            discharge_efficiency_denominator: 1,
            terminal_reserve_quanta: 0,
        }
    };

    let services_by_id: BTreeMap<&str, &CandidateService> = candidate
        .services
        .iter()
        .map(|service| (service.service_id.as_str(), service))
        .collect();

    let scenario = &candidate.scenarios[0];
    let mut intervals = Vec::with_capacity(scenario.intervals.len());
    for interval in &scenario.intervals {
        let resources: BTreeMap<&str, f64> = interval
            .resource_available_kwh
            .iter()
            .map(|resource| (resource.component_id.as_str(), resource.available_kwh))
            .collect();
        if resources.len() != interval.resource_available_kwh.len() {
            return Err(EnergyDesignAdapterError::new(format!(
                "interval {} has duplicate resource samples",
                interval.interval_id
            )));
        }
        if source_components.iter().any(|id| !resources.contains_key(id)) {
            return Err(EnergyDesignAdapterError::new(format!(
                "interval {} must declare availability for every recognized source",
                interval.interval_id
            )));
        }

        let mut pv_available_kwh = 0.0;
        let mut grid_available_kwh = 0.0;
        for (component_id, description) in &sources {
            let available = resources.get(component_id.as_str()).copied().ok_or_else(|| {
                EnergyDesignAdapterError::new(format!(
                    "interval {} lacks source availability for {}",
                    interval.interval_id, component_id
                ))
            })?;
            match description.kind {
                SourceKind::Photovoltaic => {
                    let power_cap = description.power_cap_kw.ok_or_else(|| {
                        EnergyDesignAdapterError::new(format!(
                            "PV source {component_id} lacks explicit rated_power_kw"
                        ))
                    })? * interval_hours;
                    pv_available_kwh = available
                        .min(description.energy_cap_kwh)
                        .min(power_cap);
                }
                SourceKind::GridImport => {
                    let power_cap = description.power_cap_kw.ok_or_else(|| {
                        EnergyDesignAdapterError::new(format!(
                            "grid source {component_id} lacks explicit grid_import_power_cap_kw"
                        ))
                    })? * interval_hours;
                    grid_available_kwh = available
                        .min(description.energy_cap_kwh)
                        .min(power_cap);
                }
            }
        }

        let pv_available_quanta = quantize_kwh(
            pv_available_kwh,
            config.energy_quantum_kwh,
            "PV availability",
        )?;
        let grid_import_limit_quanta = if interval.grid_outage {
            0
        } else {
            quantize_kwh(
                grid_available_kwh,
                config.energy_quantum_kwh,
                "grid import availability",
            )?
        };

        let mut essential_demand_quanta = 0_u32;
        let mut flexible_demand_quanta = 0_u32;
        for service_request in &interval.service_requested {
            let service = services_by_id
                .get(service_request.service_id.as_str())
                .ok_or_else(|| {
                    EnergyDesignAdapterError::new(format!(
                        "unknown service {}",
                        service_request.service_id
                    ))
                })?;
            let quantity = quantize_kwh(
                service_request.requested_quantity,
                config.energy_quantum_kwh,
                "requested electrical service",
            )?;
            if service.essential {
                essential_demand_quanta = essential_demand_quanta
                    .checked_add(u32::from(quantity))
                    .ok_or_else(|| EnergyDesignAdapterError::new("essential demand overflows"))?;
            } else {
                flexible_demand_quanta = flexible_demand_quanta
                    .checked_add(u32::from(quantity))
                    .ok_or_else(|| EnergyDesignAdapterError::new("flexible demand overflows"))?;
            }
        }
        if essential_demand_quanta > u32::from(u16::MAX)
            || flexible_demand_quanta > u32::from(u16::MAX)
        {
            return Err(EnergyDesignAdapterError::new(format!(
                "interval {} aggregate service demand exceeds dispatch range",
                interval.interval_id
            )));
        }

        intervals.push(DispatchIntervalInput {
            interval_id: interval.interval_id.clone(),
            start_unix_seconds: interval.start_unix_seconds,
            duration_seconds: interval.duration_seconds,
            grid_outage: interval.grid_outage,
            pv_available_quanta,
            grid_import_limit_quanta,
            essential_demand_quanta: essential_demand_quanta as u16,
            flexible_demand_quanta: flexible_demand_quanta as u16,
            grid_cost_units_per_quantum: config.grid_cost_units_per_quantum,
            grid_carbon_grams_per_quantum: config.grid_carbon_grams_per_quantum,
        });
    }

    let dispatch_request = EnergyDispatchRequest {
        schema_version: DISPATCH_SCHEMA_VERSION.to_owned(),
        case_id: format!("candidate:{}", candidate.candidate_id),
        synthetic: true,
        evidence_class: EvidenceClass::AnalystAssumption,
        evidence_ref: None,
        evidence_as_of: None,
        energy_quantum_kwh: config.energy_quantum_kwh,
        max_candidate_expansions: config.max_candidate_expansions,
        battery,
        intervals,
    };
    let encoded_request = serde_json::to_vec(&dispatch_request)
        .map_err(|error| EnergyDesignAdapterError::new(format!("cannot serialize dispatch request: {error}")))?;
    let encoded_config = serde_json::to_vec(config)
        .map_err(|error| EnergyDesignAdapterError::new(format!("cannot serialize mapping config: {error}")))?;

    Ok(EnergyDesignCompilation {
        receipt_schema_version: "1".to_owned(),
        mapping_schema_version: MAPPING_SCHEMA_VERSION.to_owned(),
        candidate_id: candidate.candidate_id,
        candidate_sha256: sha256_hex(raw_candidate),
        mapping_config_sha256: sha256_hex(&encoded_config),
        synthetic: true,
        supported_boundary: concat!(
            "synthetic, single-site, one chronological scenario; electrical kWh service; ",
            "at most one PV source, one grid source and one battery; no network power flow, ",
            "thermal/process conversion, degradation, uncertainty tree, or hardware behavior"
        )
        .to_owned(),
        claim_ceiling: concat!(
            "feasibility applies only to the compiled finite dispatch subproblem; it does not ",
            "prove feasibility of the full candidate graph, a physical site, cost/carbon truth, ",
            "grid stability, safety, or field performance"
        )
        .to_owned(),
        dispatch_request,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy_dispatch::{EnergyDispatchError, assess_energy_dispatch};

    const FIXTURE: &str =
        include_str!("../../schemas/examples/energy-design-pv-battery-grid.synthetic.json");

    fn config() -> DispatchMappingConfig {
        DispatchMappingConfig::synthetic_fixture()
    }

    #[test]
    fn maps_shared_candidate_contract_to_the_known_dispatch_problem() {
        let compiled =
            compile_energy_candidate(FIXTURE.as_bytes(), &config()).expect("candidate maps");
        assert_eq!(compiled.candidate_id, "synthetic-pv-battery-grid");
        assert!(compiled.synthetic);
        assert_eq!(compiled.candidate_sha256, sha256_hex(FIXTURE.as_bytes()));
        assert_eq!(compiled.dispatch_request.battery.capacity_quanta, 2);
        assert_eq!(compiled.dispatch_request.battery.initial_soc_quanta, 0);
        assert_eq!(compiled.dispatch_request.battery.terminal_reserve_quanta, 0);
        assert_eq!(compiled.dispatch_request.intervals.len(), 2);
        assert_eq!(compiled.dispatch_request.intervals[0].pv_available_quanta, 2);
        assert_eq!(compiled.dispatch_request.intervals[0].grid_import_limit_quanta, 2);
        assert_eq!(compiled.dispatch_request.intervals[0].essential_demand_quanta, 0);
        assert_eq!(compiled.dispatch_request.intervals[1].pv_available_quanta, 0);
        assert_eq!(compiled.dispatch_request.intervals[1].grid_import_limit_quanta, 0);
        assert_eq!(compiled.dispatch_request.intervals[1].essential_demand_quanta, 2);

        let dispatch =
            assess_energy_dispatch(&compiled.dispatch_request).expect("bounded oracle solves mapped case");
        assert!(dispatch.search_is_exact);
        assert!(dispatch.pareto_frontier_complete);
        assert_eq!(
            dispatch.pareto_alternatives[0].objective.essential_unserved_quanta,
            0
        );
    }

    #[test]
    fn rejects_unknown_component_kinds_instead_of_silently_dropping_them() {
        let mut candidate: Value = serde_json::from_str(FIXTURE).unwrap();
        candidate["components"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "component_id":"thermal-store",
                "kind":"storage_thermal",
                "name":"unsupported thermal store",
                "ports":[],
                "parameter_claims":[]
            }));
        let raw = serde_json::to_vec(&candidate).unwrap();
        let error = compile_energy_candidate(&raw, &config()).unwrap_err();
        assert!(error.reason.contains("unsupported"));
    }

    #[test]
    fn rejects_missing_explicit_terminal_reserve() {
        let mut candidate: Value = serde_json::from_str(FIXTURE).unwrap();
        let claims = candidate["components"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|component| component["component_id"] == "battery")
            .unwrap()["parameter_claims"]
            .as_array_mut()
            .unwrap();
        claims.retain(|claim| claim["parameter_id"] != "terminal_reserve_kwh");
        let raw = serde_json::to_vec(&candidate).unwrap();
        let error = compile_energy_candidate(&raw, &config()).unwrap_err();
        assert!(error.reason.contains("terminal_reserve_kwh"));
    }

    #[test]
    fn rejects_non_quantized_energy_instead_of_rounding_silently() {
        let mut candidate: Value = serde_json::from_str(FIXTURE).unwrap();
        candidate["scenarios"][0]["intervals"][0]["resource_available_kwh"][0]
            ["available_kwh"] = serde_json::json!(2.25);
        let raw = serde_json::to_vec(&candidate).unwrap();
        let error = compile_energy_candidate(&raw, &config()).unwrap_err();
        assert!(error.reason.contains("integer multiple"));
    }

    #[test]
    fn rejects_non_synthetic_or_non_assumption_candidates() {
        let mut candidate: Value = serde_json::from_str(FIXTURE).unwrap();
        candidate["synthetic"] = serde_json::json!(false);
        let raw = serde_json::to_vec(&candidate).unwrap();
        let error = compile_energy_candidate(&raw, &config()).unwrap_err();
        assert!(error.reason.contains("synthetic analyst-assumption"));

        let mut invalid_config = config();
        invalid_config.synthetic = false;
        assert!(compile_energy_candidate(FIXTURE.as_bytes(), &invalid_config).is_err());
    }

    #[test]
    fn rejects_unknown_source_type_and_missing_cost_carbon_basis() {
        let mut candidate: Value = serde_json::from_str(FIXTURE).unwrap();
        candidate["components"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|component| component["component_id"] == "pv")
            .unwrap()["parameter_claims"][0]["parameter_id"] =
            serde_json::json!("unknown_generation_technology");
        let raw = serde_json::to_vec(&candidate).unwrap();
        let error = compile_energy_candidate(&raw, &config()).unwrap_err();
        assert!(error.reason.contains("unambiguous PV or grid-import claim"));

        let mut invalid_config = config();
        invalid_config.factor_basis.clear();
        assert!(compile_energy_candidate(FIXTURE.as_bytes(), &invalid_config).is_err());
    }

    #[test]
    fn rejects_invalid_quantum_and_candidate_expansion_cap() {
        let mut invalid_config = config();
        invalid_config.energy_quantum_kwh = 0.0;
        assert!(compile_energy_candidate(FIXTURE.as_bytes(), &invalid_config).is_err());

        let mut invalid_config = config();
        invalid_config.max_candidate_expansions = 0;
        let error = compile_energy_candidate(FIXTURE.as_bytes(), &invalid_config).unwrap_err();
        assert!(error.reason.contains("max_candidate_expansions"));
    }
}
