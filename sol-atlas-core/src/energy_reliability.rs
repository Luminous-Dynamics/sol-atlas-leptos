// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Chronological essential-service reliability and grid-outage trace accounting.
//!
//! This evaluator audits an already-declared service trace. It checks UTC epoch
//! interval continuity, demand/delivery arithmetic and service quality, then reports
//! outage-linked deficits and consecutive interruption durations. It is not a dispatch
//! optimizer, a physical simulator, a meter authenticator, or a proof that the supplied
//! trace corresponds to a real site.

use crate::energy_pathway::{EnergyForm, EvidenceClass};
use std::collections::{BTreeMap, BTreeSet};

/// Requirements and criticality for one service tracked throughout the trace.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceTraceDefinition {
    pub service_id: String,
    pub description: String,
    pub essential: bool,
    pub required_form: EnergyForm,
    /// Required minimum supply temperature for thermal-heat service, in degrees C.
    /// Must be absent for non-thermal service.
    #[serde(default)]
    pub minimum_temperature_c: Option<f64>,
}

/// One service's requested and quality-compatible delivered energy within an interval.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceTraceSample {
    pub service_id: String,
    pub requested_energy_kwh: f64,
    /// Quantity that satisfies both carrier and declared quality constraints.
    pub delivered_energy_kwh: f64,
    /// Actual declared supply temperature for thermal heat; absent for non-thermal service.
    #[serde(default)]
    pub delivered_temperature_c: Option<f64>,
}

/// A single contiguous UTC interval. Grid outage is a stress annotation, not proof
/// that a site lacked supply: behind-the-meter generation/storage may still serve loads.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceTraceInterval {
    pub interval_id: String,
    /// UTC Unix epoch seconds at the interval's start.
    pub start_unix_seconds: i64,
    pub duration_seconds: u64,
    pub grid_outage: bool,
    pub services: Vec<ServiceTraceSample>,
}

/// A fully ordered trace of requested versus delivered essential and nonessential service.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyReliabilityRequest {
    pub schema_version: String,
    pub trace_id: String,
    pub synthetic: bool,
    pub evidence_class: EvidenceClass,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub evidence_as_of: Option<String>,
    pub services: Vec<ServiceTraceDefinition>,
    pub intervals: Vec<ServiceTraceInterval>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnergyReliabilityError {
    EmptySchemaVersion,
    UnsupportedSchemaVersion { schema_version: String },
    EmptyTraceId,
    NoServices,
    NoIntervals,
    NoEssentialServices,
    MissingEvidenceReference,
    EmptyServiceId,
    DuplicateServiceId { service_id: String },
    EmptyServiceDescription { service_id: String },
    MissingServiceTemperature { service_id: String },
    UnexpectedServiceTemperature { service_id: String },
    InvalidServiceTemperature { service_id: String },
    EmptyIntervalId,
    DuplicateIntervalId { interval_id: String },
    InvalidIntervalDuration { interval_id: String },
    ChronologyGapOrOverlap {
        interval_id: String,
        expected_start_unix_seconds: i64,
        actual_start_unix_seconds: i64,
    },
    ChronologyOverflow { interval_id: String },
    DuplicateServiceSample { interval_id: String, service_id: String },
    UnknownServiceSample { interval_id: String, service_id: String },
    MissingServiceSample { interval_id: String, service_id: String },
    InvalidEnergyValue { interval_id: String, service_id: String, field: String },
    DeliveredExceedsDemand {
        interval_id: String,
        service_id: String,
        delivered_energy_kwh: f64,
        requested_energy_kwh: f64,
    },
    MissingDeliveryTemperature { interval_id: String, service_id: String },
    UnexpectedDeliveryTemperature { interval_id: String, service_id: String },
    InvalidDeliveryTemperature { interval_id: String, service_id: String },
    InsufficientTemperatureGrade {
        interval_id: String,
        service_id: String,
        required_temperature_c: f64,
        delivered_temperature_c: f64,
    },
    NonFiniteDerivedValue { context: String },
}

/// Per-interval service result preserving the original chronology and stress tag.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ServiceTraceSampleAssessment {
    pub service_id: String,
    pub essential: bool,
    pub required_form: EnergyForm,
    pub requested_energy_kwh: f64,
    pub delivered_energy_kwh: f64,
    pub unserved_energy_kwh: f64,
    pub delivered_temperature_c: Option<f64>,
    pub service_balance_residual_kwh: f64,
}

/// Summary for one service over the complete trace.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ServiceTraceSummary {
    pub service_id: String,
    pub description: String,
    pub essential: bool,
    pub required_form: EnergyForm,
    pub requested_energy_kwh: f64,
    pub delivered_energy_kwh: f64,
    pub unserved_energy_kwh: f64,
    pub delivery_fraction: Option<f64>,
    pub balance_residual_kwh: f64,
}

/// Audited interval including service sums, outage status and essential-deficit status.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ServiceTraceIntervalAssessment {
    pub interval_id: String,
    pub start_unix_seconds: i64,
    pub duration_seconds: u64,
    pub grid_outage: bool,
    pub requested_energy_kwh: f64,
    pub delivered_energy_kwh: f64,
    pub unserved_energy_kwh: f64,
    pub essential_requested_energy_kwh: f64,
    pub essential_delivered_energy_kwh: f64,
    pub essential_unserved_energy_kwh: f64,
    pub essential_deficit: bool,
    pub services: Vec<ServiceTraceSampleAssessment>,
}

/// Whole-trace reliability metrics. Durations refer to contiguous supplied intervals;
/// unserved service is not energy generated and does not participate in a first-law ledger.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyReliabilityAssessment {
    pub assessment_schema_version: String,
    pub trace_id: String,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub evidence_class: EvidenceClass,
    pub evidence_ref: Option<String>,
    pub evidence_as_of: Option<String>,
    pub interval_count: usize,
    pub total_duration_seconds: u64,
    pub grid_outage_interval_count: usize,
    pub grid_outage_duration_seconds: u64,
    pub essential_deficit_interval_count: usize,
    pub essential_deficit_duration_seconds: u64,
    pub longest_consecutive_essential_deficit_seconds: u64,
    pub outage_essential_deficit_interval_count: usize,
    pub outage_essential_deficit_duration_seconds: u64,
    pub longest_consecutive_outage_essential_deficit_seconds: u64,
    pub essential_intervals_fully_met_count: usize,
    pub total_requested_service_kwh: f64,
    pub total_delivered_service_kwh: f64,
    pub total_unserved_service_kwh: f64,
    pub total_delivery_fraction: Option<f64>,
    pub essential_requested_service_kwh: f64,
    pub essential_delivered_service_kwh: f64,
    pub essential_unserved_service_kwh: f64,
    pub essential_delivery_fraction: Option<f64>,
    pub outage_essential_requested_service_kwh: f64,
    pub outage_essential_delivered_service_kwh: f64,
    pub outage_essential_unserved_service_kwh: f64,
    /// None when there is no essential demand during any flagged outage interval.
    pub outage_essential_delivery_fraction: Option<f64>,
    pub total_service_balance_residual_kwh: f64,
    pub essential_service_balance_residual_kwh: f64,
    pub services: Vec<ServiceTraceSummary>,
    pub intervals: Vec<ServiceTraceIntervalAssessment>,
}

const CELSIUS_TO_KELVIN: f64 = 273.15;
const ABSOLUTE_TOLERANCE_KWH: f64 = 1.0e-9;
const ABSOLUTE_TEMPERATURE_TOLERANCE_C: f64 = 1.0e-9;
const RELATIVE_TOLERANCE: f64 = 1.0e-12;

fn finite_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn tolerance(lhs: f64, rhs: f64) -> f64 {
    ABSOLUTE_TOLERANCE_KWH + RELATIVE_TOLERANCE * lhs.abs().max(rhs.abs())
}

fn close(lhs: f64, rhs: f64) -> bool {
    (lhs - rhs).abs() <= tolerance(lhs, rhs)
}

fn close_temperature_c(lhs: f64, rhs: f64) -> bool {
    (lhs - rhs).abs()
        <= ABSOLUTE_TEMPERATURE_TOLERANCE_C
            + RELATIVE_TOLERANCE * lhs.abs().max(rhs.abs())
}

fn add_finite(total: &mut f64, amount: f64, context: &str) -> Result<(), EnergyReliabilityError> {
    *total += amount;
    if !total.is_finite() {
        return Err(EnergyReliabilityError::NonFiniteDerivedValue {
            context: context.to_owned(),
        });
    }
    Ok(())
}

fn fraction(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator <= 0.0 {
        None
    } else {
        Some((numerator / denominator).clamp(0.0, 1.0))
    }
}

/// Audit a chronological service trace without optimizing or inventing missing supply.
///
/// Intervals must be strictly contiguous: any timestamp gap or overlap must be
/// represented as an explicit interval, usually with zero delivery. Thus outage hours,
/// interruption durations and deficit runs cannot silently skip missing chronology.
pub fn assess_energy_reliability(
    request: &EnergyReliabilityRequest,
) -> Result<EnergyReliabilityAssessment, EnergyReliabilityError> {
    if request.schema_version.trim().is_empty() {
        return Err(EnergyReliabilityError::EmptySchemaVersion);
    }
    if request.schema_version != "energy-reliability-v0.1" {
        return Err(EnergyReliabilityError::UnsupportedSchemaVersion {
            schema_version: request.schema_version.clone(),
        });
    }
    if request.trace_id.trim().is_empty() {
        return Err(EnergyReliabilityError::EmptyTraceId);
    }
    if request.services.is_empty() {
        return Err(EnergyReliabilityError::NoServices);
    }
    if request.intervals.is_empty() {
        return Err(EnergyReliabilityError::NoIntervals);
    }
    if !request.services.iter().any(|service| service.essential) {
        return Err(EnergyReliabilityError::NoEssentialServices);
    }
    if request.evidence_class != EvidenceClass::AnalystAssumption
        && request
            .evidence_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
    {
        return Err(EnergyReliabilityError::MissingEvidenceReference);
    }

    let mut service_by_id = BTreeMap::<String, &ServiceTraceDefinition>::new();
    for service in &request.services {
        if service.service_id.trim().is_empty() {
            return Err(EnergyReliabilityError::EmptyServiceId);
        }
        if service_by_id.insert(service.service_id.clone(), service).is_some() {
            return Err(EnergyReliabilityError::DuplicateServiceId {
                service_id: service.service_id.clone(),
            });
        }
        if service.description.trim().is_empty() {
            return Err(EnergyReliabilityError::EmptyServiceDescription {
                service_id: service.service_id.clone(),
            });
        }
        match (service.required_form, service.minimum_temperature_c) {
            (EnergyForm::ThermalHeat, None) => {
                return Err(EnergyReliabilityError::MissingServiceTemperature {
                    service_id: service.service_id.clone(),
                });
            }
            (EnergyForm::ThermalHeat, Some(celsius)) => {
                let kelvin = celsius + CELSIUS_TO_KELVIN;
                if !celsius.is_finite() || !kelvin.is_finite() || kelvin <= 0.0 {
                    return Err(EnergyReliabilityError::InvalidServiceTemperature {
                        service_id: service.service_id.clone(),
                    });
                }
            }
            (_, Some(_)) => {
                return Err(EnergyReliabilityError::UnexpectedServiceTemperature {
                    service_id: service.service_id.clone(),
                });
            }
            (_, None) => {}
        }
    }

    let mut interval_ids = BTreeSet::new();
    let mut expected_start = None::<i64>;
    let mut total_duration_seconds = 0_u64;
    let mut grid_outage_duration_seconds = 0_u64;
    for interval in &request.intervals {
        if interval.interval_id.trim().is_empty() {
            return Err(EnergyReliabilityError::EmptyIntervalId);
        }
        if !interval_ids.insert(interval.interval_id.as_str()) {
            return Err(EnergyReliabilityError::DuplicateIntervalId {
                interval_id: interval.interval_id.clone(),
            });
        }
        if interval.duration_seconds == 0 {
            return Err(EnergyReliabilityError::InvalidIntervalDuration {
                interval_id: interval.interval_id.clone(),
            });
        }
        if let Some(expected) = expected_start {
            if interval.start_unix_seconds != expected {
                return Err(EnergyReliabilityError::ChronologyGapOrOverlap {
                    interval_id: interval.interval_id.clone(),
                    expected_start_unix_seconds: expected,
                    actual_start_unix_seconds: interval.start_unix_seconds,
                });
            }
        }
        expected_start = Some(
            interval
                .start_unix_seconds
                .checked_add(i64::try_from(interval.duration_seconds).map_err(|_| {
                    EnergyReliabilityError::ChronologyOverflow {
                        interval_id: interval.interval_id.clone(),
                    }
                })?)
                .ok_or_else(|| EnergyReliabilityError::ChronologyOverflow {
                    interval_id: interval.interval_id.clone(),
                })?,
        );
        total_duration_seconds = total_duration_seconds
            .checked_add(interval.duration_seconds)
            .ok_or_else(|| EnergyReliabilityError::ChronologyOverflow {
                interval_id: interval.interval_id.clone(),
            })?;
        if interval.grid_outage {
            grid_outage_duration_seconds = grid_outage_duration_seconds
                .checked_add(interval.duration_seconds)
                .ok_or_else(|| EnergyReliabilityError::ChronologyOverflow {
                    interval_id: interval.interval_id.clone(),
                })?;
        }

        let mut sample_ids = BTreeSet::new();
        for sample in &interval.services {
            if !service_by_id.contains_key(&sample.service_id) {
                return Err(EnergyReliabilityError::UnknownServiceSample {
                    interval_id: interval.interval_id.clone(),
                    service_id: sample.service_id.clone(),
                });
            }
            if !sample_ids.insert(sample.service_id.as_str()) {
                return Err(EnergyReliabilityError::DuplicateServiceSample {
                    interval_id: interval.interval_id.clone(),
                    service_id: sample.service_id.clone(),
                });
            }
        }
        for service_id in service_by_id.keys() {
            if !sample_ids.contains(service_id.as_str()) {
                return Err(EnergyReliabilityError::MissingServiceSample {
                    interval_id: interval.interval_id.clone(),
                    service_id: service_id.clone(),
                });
            }
        }
    }

    let mut summary_by_id = BTreeMap::<String, ServiceTraceSummary>::new();
    for service in &request.services {
        summary_by_id.insert(
            service.service_id.clone(),
            ServiceTraceSummary {
                service_id: service.service_id.clone(),
                description: service.description.clone(),
                essential: service.essential,
                required_form: service.required_form,
                requested_energy_kwh: 0.0,
                delivered_energy_kwh: 0.0,
                unserved_energy_kwh: 0.0,
                delivery_fraction: None,
                balance_residual_kwh: 0.0,
            },
        );
    }

    let mut intervals_out = Vec::with_capacity(request.intervals.len());
    let mut total_requested_service_kwh = 0.0;
    let mut total_delivered_service_kwh = 0.0;
    let mut total_unserved_service_kwh = 0.0;
    let mut essential_requested_service_kwh = 0.0;
    let mut essential_delivered_service_kwh = 0.0;
    let mut essential_unserved_service_kwh = 0.0;
    let mut outage_essential_requested_service_kwh = 0.0;
    let mut outage_essential_delivered_service_kwh = 0.0;
    let mut outage_essential_unserved_service_kwh = 0.0;
    let mut essential_deficit_interval_count = 0_usize;
    let mut essential_deficit_duration_seconds = 0_u64;
    let mut longest_consecutive_essential_deficit_seconds = 0_u64;
    let mut current_essential_deficit_run_seconds = 0_u64;
    let mut outage_essential_deficit_interval_count = 0_usize;
    let mut outage_essential_deficit_duration_seconds = 0_u64;
    let mut longest_consecutive_outage_essential_deficit_seconds = 0_u64;
    let mut current_outage_essential_deficit_run_seconds = 0_u64;
    let mut essential_intervals_fully_met_count = 0_usize;

    for interval in &request.intervals {
        let samples_by_id = interval
            .services
            .iter()
            .map(|sample| (sample.service_id.as_str(), sample))
            .collect::<BTreeMap<_, _>>();
        let mut samples_out = Vec::with_capacity(interval.services.len());
        let mut interval_requested = 0.0;
        let mut interval_delivered = 0.0;
        let mut interval_unserved = 0.0;
        let mut interval_essential_requested = 0.0;
        let mut interval_essential_delivered = 0.0;
        let mut interval_essential_unserved = 0.0;

        for service in &request.services {
            let sample = samples_by_id
                .get(service.service_id.as_str())
                .copied()
                .expect("validated interval has exactly one sample for every service");
            if !finite_nonnegative(sample.requested_energy_kwh) {
                return Err(EnergyReliabilityError::InvalidEnergyValue {
                    interval_id: interval.interval_id.clone(),
                    service_id: service.service_id.clone(),
                    field: "requested_energy_kwh".to_owned(),
                });
            }
            if !finite_nonnegative(sample.delivered_energy_kwh) {
                return Err(EnergyReliabilityError::InvalidEnergyValue {
                    interval_id: interval.interval_id.clone(),
                    service_id: service.service_id.clone(),
                    field: "delivered_energy_kwh".to_owned(),
                });
            }
            // These are declared interval observations, not solver-derived values:
            // any delivery above the request is an overclaim, including tiny values
            // against zero demand. Numerical tolerance is reserved for computed sums.
            if sample.delivered_energy_kwh > sample.requested_energy_kwh {
                return Err(EnergyReliabilityError::DeliveredExceedsDemand {
                    interval_id: interval.interval_id.clone(),
                    service_id: service.service_id.clone(),
                    delivered_energy_kwh: sample.delivered_energy_kwh,
                    requested_energy_kwh: sample.requested_energy_kwh,
                });
            }

            match (service.required_form, sample.delivered_temperature_c) {
                (EnergyForm::ThermalHeat, Some(celsius)) => {
                    let kelvin = celsius + CELSIUS_TO_KELVIN;
                    if !celsius.is_finite() || !kelvin.is_finite() || kelvin <= 0.0 {
                        return Err(EnergyReliabilityError::InvalidDeliveryTemperature {
                            interval_id: interval.interval_id.clone(),
                            service_id: service.service_id.clone(),
                        });
                    }
                    if sample.delivered_energy_kwh > 0.0
                        && celsius + ABSOLUTE_TEMPERATURE_TOLERANCE_C
                            < service.minimum_temperature_c.unwrap()
                    {
                        return Err(EnergyReliabilityError::InsufficientTemperatureGrade {
                            interval_id: interval.interval_id.clone(),
                            service_id: service.service_id.clone(),
                            required_temperature_c: service.minimum_temperature_c.unwrap(),
                            delivered_temperature_c: celsius,
                        });
                    }
                }
                (EnergyForm::ThermalHeat, None)
                    if sample.delivered_energy_kwh > 0.0 =>
                {
                    return Err(EnergyReliabilityError::MissingDeliveryTemperature {
                        interval_id: interval.interval_id.clone(),
                        service_id: service.service_id.clone(),
                    });
                }
                (EnergyForm::ThermalHeat, None) => {}
                (_, Some(_)) => {
                    return Err(EnergyReliabilityError::UnexpectedDeliveryTemperature {
                        interval_id: interval.interval_id.clone(),
                        service_id: service.service_id.clone(),
                    });
                }
                (_, None) => {}
            }

            let unserved = (sample.requested_energy_kwh - sample.delivered_energy_kwh).max(0.0);
            let residual = sample.requested_energy_kwh
                - sample.delivered_energy_kwh
                - unserved;
            if !unserved.is_finite() || !residual.is_finite() {
                return Err(EnergyReliabilityError::NonFiniteDerivedValue {
                    context: format!(
                        "service {} interval {}",
                        service.service_id, interval.interval_id
                    ),
                });
            }
            for (total, value, label) in [
                (&mut interval_requested, sample.requested_energy_kwh, "interval requested"),
                (&mut interval_delivered, sample.delivered_energy_kwh, "interval delivered"),
                (&mut interval_unserved, unserved, "interval unserved"),
            ] {
                add_finite(total, value, label)?;
            }

            let summary = summary_by_id.get_mut(&service.service_id).unwrap();
            for (total, value, label) in [
                (
                    &mut summary.requested_energy_kwh,
                    sample.requested_energy_kwh,
                    "service requested",
                ),
                (
                    &mut summary.delivered_energy_kwh,
                    sample.delivered_energy_kwh,
                    "service delivered",
                ),
                (&mut summary.unserved_energy_kwh, unserved, "service unserved"),
            ] {
                add_finite(total, value, label)?;
            }
            summary.balance_residual_kwh += residual;

            if service.essential {
                for (total, value, label) in [
                    (
                        &mut interval_essential_requested,
                        sample.requested_energy_kwh,
                        "interval essential requested",
                    ),
                    (
                        &mut interval_essential_delivered,
                        sample.delivered_energy_kwh,
                        "interval essential delivered",
                    ),
                    (
                        &mut interval_essential_unserved,
                        unserved,
                        "interval essential unserved",
                    ),
                    (
                        &mut essential_requested_service_kwh,
                        sample.requested_energy_kwh,
                        "essential requested",
                    ),
                    (
                        &mut essential_delivered_service_kwh,
                        sample.delivered_energy_kwh,
                        "essential delivered",
                    ),
                    (
                        &mut essential_unserved_service_kwh,
                        unserved,
                        "essential unserved",
                    ),
                ] {
                    add_finite(total, value, label)?;
                }
                if interval.grid_outage {
                    for (total, value, label) in [
                        (
                            &mut outage_essential_requested_service_kwh,
                            sample.requested_energy_kwh,
                            "outage essential requested",
                        ),
                        (
                            &mut outage_essential_delivered_service_kwh,
                            sample.delivered_energy_kwh,
                            "outage essential delivered",
                        ),
                        (
                            &mut outage_essential_unserved_service_kwh,
                            unserved,
                            "outage essential unserved",
                        ),
                    ] {
                        add_finite(total, value, label)?;
                    }
                }
            }

            samples_out.push(ServiceTraceSampleAssessment {
                service_id: service.service_id.clone(),
                essential: service.essential,
                required_form: service.required_form,
                requested_energy_kwh: sample.requested_energy_kwh,
                delivered_energy_kwh: sample.delivered_energy_kwh,
                unserved_energy_kwh: unserved,
                delivered_temperature_c: sample.delivered_temperature_c,
                service_balance_residual_kwh: residual,
            });
        }

        // Unserved energy is computed from declared sample values, so any positive
        // essential shortfall is a service deficit even when smaller than balance tolerance.
        let essential_deficit = interval_essential_unserved > 0.0;
        if essential_deficit {
            essential_deficit_interval_count += 1;
            essential_deficit_duration_seconds = essential_deficit_duration_seconds
                .checked_add(interval.duration_seconds)
                .ok_or_else(|| EnergyReliabilityError::ChronologyOverflow {
                    interval_id: interval.interval_id.clone(),
                })?;
            current_essential_deficit_run_seconds = current_essential_deficit_run_seconds
                .checked_add(interval.duration_seconds)
                .ok_or_else(|| EnergyReliabilityError::ChronologyOverflow {
                    interval_id: interval.interval_id.clone(),
                })?;
            longest_consecutive_essential_deficit_seconds =
                longest_consecutive_essential_deficit_seconds
                    .max(current_essential_deficit_run_seconds);
        } else {
            current_essential_deficit_run_seconds = 0;
        }

        let outage_essential_deficit = interval.grid_outage && essential_deficit;
        if outage_essential_deficit {
            outage_essential_deficit_interval_count += 1;
            outage_essential_deficit_duration_seconds = outage_essential_deficit_duration_seconds
                .checked_add(interval.duration_seconds)
                .ok_or_else(|| EnergyReliabilityError::ChronologyOverflow {
                    interval_id: interval.interval_id.clone(),
                })?;
            current_outage_essential_deficit_run_seconds =
                current_outage_essential_deficit_run_seconds
                    .checked_add(interval.duration_seconds)
                    .ok_or_else(|| EnergyReliabilityError::ChronologyOverflow {
                        interval_id: interval.interval_id.clone(),
                    })?;
            longest_consecutive_outage_essential_deficit_seconds =
                longest_consecutive_outage_essential_deficit_seconds
                    .max(current_outage_essential_deficit_run_seconds);
        } else {
            current_outage_essential_deficit_run_seconds = 0;
        }
        if !essential_deficit {
            essential_intervals_fully_met_count += 1;
        }

        for value in [
            interval_requested,
            interval_delivered,
            interval_unserved,
            interval_essential_requested,
            interval_essential_delivered,
            interval_essential_unserved,
        ] {
            if !value.is_finite() {
                return Err(EnergyReliabilityError::NonFiniteDerivedValue {
                    context: format!("interval totals {}", interval.interval_id),
                });
            }
        }

        samples_out.sort_by(|left, right| left.service_id.cmp(&right.service_id));
        intervals_out.push(ServiceTraceIntervalAssessment {
            interval_id: interval.interval_id.clone(),
            start_unix_seconds: interval.start_unix_seconds,
            duration_seconds: interval.duration_seconds,
            grid_outage: interval.grid_outage,
            requested_energy_kwh: interval_requested,
            delivered_energy_kwh: interval_delivered,
            unserved_energy_kwh: interval_unserved,
            essential_requested_energy_kwh: interval_essential_requested,
            essential_delivered_energy_kwh: interval_essential_delivered,
            essential_unserved_energy_kwh: interval_essential_unserved,
            essential_deficit,
            services: samples_out,
        });
    }

    for (total, value, label) in [
        (
            &mut total_requested_service_kwh,
            summary_by_id
                .values()
                .map(|summary| summary.requested_energy_kwh)
                .sum::<f64>(),
            "all requested",
        ),
        (
            &mut total_delivered_service_kwh,
            summary_by_id
                .values()
                .map(|summary| summary.delivered_energy_kwh)
                .sum::<f64>(),
            "all delivered",
        ),
        (
            &mut total_unserved_service_kwh,
            summary_by_id
                .values()
                .map(|summary| summary.unserved_energy_kwh)
                .sum::<f64>(),
            "all unserved",
        ),
    ] {
        *total = value;
        if !total.is_finite() {
            return Err(EnergyReliabilityError::NonFiniteDerivedValue {
                context: label.to_owned(),
            });
        }
    }

    for summary in summary_by_id.values_mut() {
        if !summary.balance_residual_kwh.is_finite() {
            return Err(EnergyReliabilityError::NonFiniteDerivedValue {
                context: format!("service residual {}", summary.service_id),
            });
        }
        summary.delivery_fraction =
            fraction(summary.delivered_energy_kwh, summary.requested_energy_kwh);
    }

    let essential_service_balance_residual_kwh = essential_requested_service_kwh
        - essential_delivered_service_kwh
        - essential_unserved_service_kwh;
    let total_service_balance_residual_kwh =
        total_requested_service_kwh - total_delivered_service_kwh - total_unserved_service_kwh;
    if !essential_service_balance_residual_kwh.is_finite()
        || !total_service_balance_residual_kwh.is_finite()
        || !close(
            total_requested_service_kwh,
            total_delivered_service_kwh + total_unserved_service_kwh,
        )
        || !close(
            essential_requested_service_kwh,
            essential_delivered_service_kwh + essential_unserved_service_kwh,
        )
    {
        return Err(EnergyReliabilityError::NonFiniteDerivedValue {
            context: "aggregate service balance".to_owned(),
        });
    }

    let mut services = summary_by_id.into_values().collect::<Vec<_>>();
    services.sort_by(|left, right| left.service_id.cmp(&right.service_id));
    let outage_essential_delivery_fraction = fraction(
        outage_essential_delivered_service_kwh,
        outage_essential_requested_service_kwh,
    );

    Ok(EnergyReliabilityAssessment {
        assessment_schema_version: "1".to_owned(),
        trace_id: request.trace_id.clone(),
        synthetic: request.synthetic,
        claim_ceiling: concat!(
            "chronological arithmetic audit of a declared service trace; evidence metadata ",
            "is not independently authenticated; grid-outage tags are scenario observations, ",
            "not proof of site-wide loss or survival",
        )
        .to_owned(),
        evidence_class: request.evidence_class,
        evidence_ref: request.evidence_ref.clone(),
        evidence_as_of: request.evidence_as_of.clone(),
        interval_count: request.intervals.len(),
        total_duration_seconds,
        grid_outage_interval_count: request.intervals.iter().filter(|i| i.grid_outage).count(),
        grid_outage_duration_seconds,
        essential_deficit_interval_count,
        essential_deficit_duration_seconds,
        longest_consecutive_essential_deficit_seconds,
        outage_essential_deficit_interval_count,
        outage_essential_deficit_duration_seconds,
        longest_consecutive_outage_essential_deficit_seconds,
        essential_intervals_fully_met_count,
        total_requested_service_kwh,
        total_delivered_service_kwh,
        total_unserved_service_kwh,
        total_delivery_fraction: fraction(
            total_delivered_service_kwh,
            total_requested_service_kwh,
        ),
        essential_requested_service_kwh,
        essential_delivered_service_kwh,
        essential_unserved_service_kwh,
        essential_delivery_fraction: fraction(
            essential_delivered_service_kwh,
            essential_requested_service_kwh,
        ),
        outage_essential_requested_service_kwh,
        outage_essential_delivered_service_kwh,
        outage_essential_unserved_service_kwh,
        outage_essential_delivery_fraction,
        total_service_balance_residual_kwh,
        essential_service_balance_residual_kwh,
        services,
        intervals: intervals_out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> EnergyReliabilityRequest {
        EnergyReliabilityRequest {
            schema_version: "energy-reliability-v0.1".to_owned(),
            trace_id: "synthetic-four-hour-outage-trace".to_owned(),
            synthetic: true,
            evidence_class: EvidenceClass::AnalystAssumption,
            evidence_ref: None,
            evidence_as_of: None,
            services: vec![ServiceTraceDefinition {
                service_id: "essential-electricity".to_owned(),
                description: "essential electrical load".to_owned(),
                essential: true,
                required_form: EnergyForm::Electricity,
                minimum_temperature_c: None,
            }],
            intervals: vec![
                interval("normal-1", 1_800_000_000, false, 10.0, 10.0),
                interval("outage-1", 1_800_003_600, true, 10.0, 8.0),
                interval("outage-2", 1_800_007_200, true, 10.0, 0.0),
                interval("normal-2", 1_800_010_800, false, 10.0, 10.0),
            ],
        }
    }

    fn interval(
        id: &str,
        start_unix_seconds: i64,
        grid_outage: bool,
        requested_energy_kwh: f64,
        delivered_energy_kwh: f64,
    ) -> ServiceTraceInterval {
        ServiceTraceInterval {
            interval_id: id.to_owned(),
            start_unix_seconds,
            duration_seconds: 3600,
            grid_outage,
            services: vec![ServiceTraceSample {
                service_id: "essential-electricity".to_owned(),
                requested_energy_kwh,
                delivered_energy_kwh,
                delivered_temperature_c: None,
            }],
        }
    }

    #[test]
    fn synthetic_outage_trace_has_exact_known_answer() {
        let assessment = assess_energy_reliability(&fixture()).unwrap();
        assert_eq!(assessment.interval_count, 4);
        assert_eq!(assessment.total_duration_seconds, 14_400);
        assert_eq!(assessment.grid_outage_interval_count, 2);
        assert_eq!(assessment.grid_outage_duration_seconds, 7_200);
        assert_eq!(assessment.total_requested_service_kwh, 40.0);
        assert_eq!(assessment.total_delivered_service_kwh, 28.0);
        assert_eq!(assessment.total_unserved_service_kwh, 12.0);
        assert_eq!(assessment.essential_unserved_service_kwh, 12.0);
        assert_eq!(assessment.outage_essential_requested_service_kwh, 20.0);
        assert_eq!(assessment.outage_essential_delivered_service_kwh, 8.0);
        assert_eq!(assessment.outage_essential_unserved_service_kwh, 12.0);
        assert!((assessment.outage_essential_delivery_fraction.unwrap() - 0.4).abs() < 1.0e-12);
        assert_eq!(assessment.essential_deficit_interval_count, 2);
        assert_eq!(assessment.essential_deficit_duration_seconds, 7_200);
        assert_eq!(assessment.longest_consecutive_essential_deficit_seconds, 7_200);
        assert_eq!(assessment.outage_essential_deficit_interval_count, 2);
        assert_eq!(assessment.outage_essential_deficit_duration_seconds, 7_200);
        assert_eq!(assessment.longest_consecutive_outage_essential_deficit_seconds, 7_200);
        assert_eq!(assessment.essential_intervals_fully_met_count, 2);
        assert!(assessment.total_service_balance_residual_kwh.abs() < 1.0e-9);
        assert!(assessment.essential_service_balance_residual_kwh.abs() < 1.0e-9);
    }

    #[test]
    fn timeline_gap_and_overlap_fail_closed() {
        for offset in [-60, 60] {
            let mut request = fixture();
            request.intervals[1].start_unix_seconds += offset;
            assert!(matches!(
                assess_energy_reliability(&request),
                Err(EnergyReliabilityError::ChronologyGapOrOverlap { .. })
            ));
        }
    }

    #[test]
    fn delivered_service_cannot_exceed_demand() {
        let mut request = fixture();
        request.intervals[1].services[0].delivered_energy_kwh = 11.0;
        assert!(matches!(
            assess_energy_reliability(&request),
            Err(EnergyReliabilityError::DeliveredExceedsDemand { .. })
        ));
    }

    #[test]
    fn any_positive_delivery_above_zero_demand_is_rejected() {
        let mut request = fixture();
        request.intervals[0].services[0].requested_energy_kwh = 0.0;
        request.intervals[0].services[0].delivered_energy_kwh = 1.0e-12;
        assert!(matches!(
            assess_energy_reliability(&request),
            Err(EnergyReliabilityError::DeliveredExceedsDemand { .. })
        ));
    }

    #[test]
    fn positive_micro_shortfall_still_counts_as_a_service_deficit() {
        let mut request = fixture();
        request.intervals[0].services[0].requested_energy_kwh = 1.0e-10;
        request.intervals[0].services[0].delivered_energy_kwh = 0.0;
        request.intervals[1].services[0].delivered_energy_kwh = 10.0;
        request.intervals[2].services[0].delivered_energy_kwh = 10.0;
        let assessment = assess_energy_reliability(&request).unwrap();
        assert_eq!(assessment.essential_deficit_interval_count, 1);
        assert_eq!(assessment.essential_deficit_duration_seconds, 3_600);
        assert_eq!(assessment.longest_consecutive_essential_deficit_seconds, 3_600);
    }

    #[test]
    fn missing_or_duplicate_service_sample_fails_closed() {
        let mut missing = fixture();
        missing.intervals[0].services.clear();
        assert!(matches!(
            assess_energy_reliability(&missing),
            Err(EnergyReliabilityError::MissingServiceSample { .. })
        ));

        let mut duplicate = fixture();
        duplicate.intervals[0].services.push(duplicate.intervals[0].services[0].clone());
        assert!(matches!(
            assess_energy_reliability(&duplicate),
            Err(EnergyReliabilityError::DuplicateServiceSample { .. })
        ));
    }

    #[test]
    fn outage_tag_does_not_imply_service_failure() {
        let mut request = fixture();
        request.intervals[1].services[0].delivered_energy_kwh = 10.0;
        request.intervals[2].services[0].delivered_energy_kwh = 10.0;
        let assessment = assess_energy_reliability(&request).unwrap();
        assert_eq!(assessment.grid_outage_duration_seconds, 7_200);
        assert_eq!(assessment.outage_essential_unserved_service_kwh, 0.0);
        assert_eq!(assessment.longest_consecutive_outage_essential_deficit_seconds, 0);
        assert_eq!(assessment.outage_essential_delivery_fraction, Some(1.0));
    }

    #[test]
    fn zero_essential_demand_during_outage_has_undefined_fraction() {
        let mut request = fixture();
        for interval in &mut request.intervals {
            if interval.grid_outage {
                interval.services[0].requested_energy_kwh = 0.0;
                interval.services[0].delivered_energy_kwh = 0.0;
            }
        }
        let assessment = assess_energy_reliability(&request).unwrap();
        assert_eq!(assessment.outage_essential_requested_service_kwh, 0.0);
        assert_eq!(assessment.outage_essential_delivery_fraction, None);
    }

    #[test]
    fn thermal_delivery_below_grade_fails_closed() {
        let mut request = fixture();
        request.services[0].required_form = EnergyForm::ThermalHeat;
        request.services[0].minimum_temperature_c = Some(80.0);
        for interval in &mut request.intervals {
            interval.services[0].delivered_temperature_c = Some(70.0);
        }
        assert!(matches!(
            assess_energy_reliability(&request),
            Err(EnergyReliabilityError::InsufficientTemperatureGrade { .. })
        ));
    }

    #[test]
    fn non_assumption_trace_requires_source_reference() {
        let mut request = fixture();
        request.evidence_class = EvidenceClass::OperatingPlant;
        assert!(matches!(
            assess_energy_reliability(&request),
            Err(EnergyReliabilityError::MissingEvidenceReference)
        ));
        request.evidence_ref = Some("metered-trace-001".to_owned());
        assert!(assess_energy_reliability(&request).is_ok());
    }

    #[test]
    fn duplicate_interval_and_service_ids_fail_closed() {
        let mut request = fixture();
        request.intervals[1].interval_id = request.intervals[0].interval_id.clone();
        assert!(matches!(
            assess_energy_reliability(&request),
            Err(EnergyReliabilityError::DuplicateIntervalId { .. })
        ));

        let mut request = fixture();
        request.services.push(request.services[0].clone());
        assert!(matches!(
            assess_energy_reliability(&request),
            Err(EnergyReliabilityError::DuplicateServiceId { .. })
        ));
    }

    #[test]
    fn essential_deficit_runs_reset_after_service_recovers() {
        let mut request = fixture();
        request.intervals[3].services[0].delivered_energy_kwh = 9.0;
        let assessment = assess_energy_reliability(&request).unwrap();
        assert_eq!(assessment.essential_deficit_interval_count, 3);
        assert_eq!(assessment.essential_deficit_duration_seconds, 10_800);
        assert_eq!(assessment.longest_consecutive_essential_deficit_seconds, 7_200);
        assert_eq!(assessment.longest_consecutive_outage_essential_deficit_seconds, 7_200);
    }
}
