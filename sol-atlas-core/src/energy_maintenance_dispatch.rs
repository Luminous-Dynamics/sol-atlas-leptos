// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Fail-closed coupling between one PV inverter maintenance trace and the
//! finite exact site-microgrid dispatch oracle.
//!
//! The request's PV availability series must represent exactly one explicitly
//! bound inverter. PlannedOutage and Unknown set that source's available PV to
//! zero. This is not a network topology model and cannot represent line or
//! transformer outages, multiple PV inverters, partial derating or AC power flow.
//! Intervals crossing maintenance-state boundaries are rejected, not averaged.
//! Any changed post-maintenance configuration requires a new dispatch profile.

use crate::energy_dispatch::{
    EnergyDispatchAssessment, EnergyDispatchError, EnergyDispatchRequest, assess_energy_dispatch,
};
use crate::maintenance_availability::{
    AvailabilityState, MaintenanceAvailabilityTraceV1, TraceValidationError,
    validate_trace_structure,
};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAINTENANCE_DISPATCH_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PvAssetDispatchBinding {
    /// The single PV inverter represented by all dispatch PV availability values.
    pub pv_asset_id: String,
    /// Exact configuration for which the PV production envelope was qualified.
    pub pv_configuration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchAssetAvailability {
    Available,
    PlannedOutage,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceDispatchIntervalBinding {
    pub dispatch_interval_id: String,
    pub start_unix_seconds: i64,
    pub end_unix_seconds: i64,
    pub asset_availability: DispatchAssetAvailability,
    pub configuration_digest: String,
    pub input_pv_available_quanta: u16,
    pub effective_pv_available_quanta: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceDispatchProjection {
    pub schema_version: u16,
    pub maintenance_trace_id: String,
    pub work_order_id: String,
    pub pv_asset_id: String,
    pub pv_configuration_digest: String,
    pub dispatch_case_id: String,
    pub interval_bindings: Vec<MaintenanceDispatchIntervalBinding>,
    pub adjusted_dispatch_request: EnergyDispatchRequest,
    pub claim_ceiling: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceDispatchAssessment {
    pub projection: MaintenanceDispatchProjection,
    pub dispatch_assessment: EnergyDispatchAssessment,
    pub claim_ceiling: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaintenanceDispatchError {
    InvalidMaintenanceTrace(TraceValidationError),
    EmptyPvAssetBinding,
    AssetIdentityMismatch,
    ConfigurationChangedRequiresRequalification,
    DispatchIntervalsOutsideMaintenanceStudy,
    DispatchIntervalCrossesMaintenanceBoundary { interval_id: String },
    DispatchIntervalOverflow { interval_id: String },
    DispatchRejected { detail: String },
}

impl fmt::Display for MaintenanceDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMaintenanceTrace(error) => {
                write!(formatter, "invalid maintenance trace: {error}")
            }
            Self::EmptyPvAssetBinding => formatter.write_str("PV asset binding is empty"),
            Self::AssetIdentityMismatch => {
                formatter.write_str("maintenance trace targets a different PV asset")
            }
            Self::ConfigurationChangedRequiresRequalification => formatter.write_str(
                "post-maintenance configuration differs from the dispatch profile",
            ),
            Self::DispatchIntervalsOutsideMaintenanceStudy => formatter.write_str(
                "dispatch intervals are outside the maintenance study window",
            ),
            Self::DispatchIntervalCrossesMaintenanceBoundary { interval_id } => write!(
                formatter,
                "dispatch interval crosses a maintenance availability boundary: {interval_id}"
            ),
            Self::DispatchIntervalOverflow { interval_id } => write!(
                formatter,
                "dispatch interval end overflows Unix seconds: {interval_id}"
            ),
            Self::DispatchRejected { detail } => {
                write!(formatter, "finite dispatch oracle rejected adjusted request: {detail}")
            }
        }
    }
}

impl std::error::Error for MaintenanceDispatchError {}

fn dispatch_state(state: &AvailabilityState) -> DispatchAssetAvailability {
    match state {
        AvailabilityState::Available => DispatchAssetAvailability::Available,
        AvailabilityState::PlannedOutage => DispatchAssetAvailability::PlannedOutage,
        AvailabilityState::Unknown => DispatchAssetAvailability::Unknown,
    }
}

/// Bind one asset's maintenance state to the PV availability series.
///
/// The function rejects another asset, configuration drift, a dispatch horizon
/// outside the trace, and intervals crossing availability boundaries. Unknown
/// and PlannedOutage both zero the bound PV source; their distinct states remain
/// visible in the returned interval binding.
pub fn project_pv_dispatch_with_maintenance(
    trace: &MaintenanceAvailabilityTraceV1,
    binding: &PvAssetDispatchBinding,
    dispatch: &EnergyDispatchRequest,
) -> Result<MaintenanceDispatchProjection, MaintenanceDispatchError> {
    validate_trace_structure(trace)
        .map_err(MaintenanceDispatchError::InvalidMaintenanceTrace)?;

    if binding.pv_asset_id.trim().is_empty()
        || binding.pv_configuration_digest.trim().is_empty()
    {
        return Err(MaintenanceDispatchError::EmptyPvAssetBinding);
    }
    if trace.target.asset_id != binding.pv_asset_id {
        return Err(MaintenanceDispatchError::AssetIdentityMismatch);
    }
    if trace.target.configuration_digest != binding.pv_configuration_digest
        || trace
            .intervals
            .iter()
            .any(|interval| interval.configuration_digest != binding.pv_configuration_digest)
    {
        return Err(MaintenanceDispatchError::ConfigurationChangedRequiresRequalification);
    }

    let mut adjusted = dispatch.clone();
    let mut interval_bindings = Vec::with_capacity(dispatch.intervals.len());

    for interval in &mut adjusted.intervals {
        let duration = i64::try_from(interval.duration_seconds).map_err(|_| {
            MaintenanceDispatchError::DispatchIntervalOverflow {
                interval_id: interval.interval_id.clone(),
            }
        })?;
        let end = interval.start_unix_seconds.checked_add(duration).ok_or_else(|| {
            MaintenanceDispatchError::DispatchIntervalOverflow {
                interval_id: interval.interval_id.clone(),
            }
        })?;

        if interval.start_unix_seconds < trace.study_window.start_unix_seconds
            || end > trace.study_window.end_unix_seconds
        {
            return Err(MaintenanceDispatchError::DispatchIntervalsOutsideMaintenanceStudy);
        }

        let availability = trace.intervals.iter().find(|candidate| {
            interval.start_unix_seconds >= candidate.start_unix_seconds
                && end <= candidate.end_unix_seconds
        });
        let availability = availability.ok_or_else(|| {
            MaintenanceDispatchError::DispatchIntervalCrossesMaintenanceBoundary {
                interval_id: interval.interval_id.clone(),
            }
        })?;

        let input_pv = interval.pv_available_quanta;
        let state = dispatch_state(&availability.state);
        let effective_pv = match state {
            DispatchAssetAvailability::Available => input_pv,
            DispatchAssetAvailability::PlannedOutage | DispatchAssetAvailability::Unknown => 0,
        };
        interval.pv_available_quanta = effective_pv;

        interval_bindings.push(MaintenanceDispatchIntervalBinding {
            dispatch_interval_id: interval.interval_id.clone(),
            start_unix_seconds: interval.start_unix_seconds,
            end_unix_seconds: end,
            asset_availability: state,
            configuration_digest: availability.configuration_digest.clone(),
            input_pv_available_quanta: input_pv,
            effective_pv_available_quanta: effective_pv,
        });
    }

    Ok(MaintenanceDispatchProjection {
        schema_version: MAINTENANCE_DISPATCH_SCHEMA_VERSION,
        maintenance_trace_id: trace.trace_id.clone(),
        work_order_id: trace.work_order_id.clone(),
        pv_asset_id: binding.pv_asset_id.clone(),
        pv_configuration_digest: binding.pv_configuration_digest.clone(),
        dispatch_case_id: dispatch.case_id.clone(),
        interval_bindings,
        adjusted_dispatch_request: adjusted,
        claim_ceiling: String::from(
            "single explicitly bound PV inverter derating in a finite site-microgrid dispatch model; not a network topology, line/transformer outage, AC power-flow, physical equipment, operational, or field qualification",
        ),
    })
}

/// Apply the maintenance projection and run the declared finite dispatch oracle.
///
/// Exactness remains bounded by the declared integer-quantized site model. The
/// composition adds no network-flow, asset-health or physical qualification.
pub fn assess_pv_dispatch_with_maintenance(
    trace: &MaintenanceAvailabilityTraceV1,
    binding: &PvAssetDispatchBinding,
    dispatch: &EnergyDispatchRequest,
) -> Result<MaintenanceDispatchAssessment, MaintenanceDispatchError> {
    let projection = project_pv_dispatch_with_maintenance(trace, binding, dispatch)?;
    let dispatch_assessment = assess_energy_dispatch(&projection.adjusted_dispatch_request)
        .map_err(|error: EnergyDispatchError| MaintenanceDispatchError::DispatchRejected {
            detail: format!("{error:?}"),
        })?;

    Ok(MaintenanceDispatchAssessment {
        projection,
        dispatch_assessment,
        claim_ceiling: String::from(
            "exact dispatch result for a single-PV-source finite site model after fail-closed application of one asset maintenance trace; no transmission/network topology or physical/field qualification is implied",
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy_dispatch::{DispatchBattery, DispatchIntervalInput};
    use crate::energy_pathway::EvidenceClass;
    use crate::maintenance_availability::{
        AssetIdentity, EvidenceRef, MaintenanceWorkOrder, ProjectionRequest, StudyWindow,
        project_maintenance_availability,
    };

    const CONFIG: &str = "pv-inverter-config-a";

    fn evidence(reference: &str, at: i64, config: &str) -> EvidenceRef {
        EvidenceRef {
            reference: reference.to_owned(),
            digest: format!("opaque:{reference}"),
            observed_at_unix_seconds: at,
            configuration_digest: config.to_owned(),
        }
    }

    fn trace_request() -> ProjectionRequest {
        ProjectionRequest {
            study_id: "synthetic-pv-maintenance-case".to_owned(),
            study_window: StudyWindow {
                start_unix_seconds: 0,
                end_unix_seconds: 10_800,
            },
            target: AssetIdentity {
                asset_id: "pv-inverter-1".to_owned(),
                configuration_digest: CONFIG.to_owned(),
            },
            baseline_available: evidence("pv-baseline", -1, CONFIG),
            work_order: MaintenanceWorkOrder {
                work_order_id: "pv-inverter-maintenance-1".to_owned(),
                target: AssetIdentity {
                    asset_id: "pv-inverter-1".to_owned(),
                    configuration_digest: CONFIG.to_owned(),
                },
                planned_start_unix_seconds: 3_600,
                planned_end_unix_seconds: 7_200,
                source_event: evidence("maintenance-created", 1, CONFIG),
                authorization: Some(evidence("maintenance-authorized", 2, CONFIG)),
                isolation: Some(evidence("inverter-isolated", 3_600, CONFIG)),
                completion: Some(evidence("maintenance-completed", 6_000, CONFIG)),
                post_work_configuration_digest: None,
                inspection: Some(evidence("inspection-accepted", 6_600, CONFIG)),
                return_to_service_authorization: Some(evidence("return-authorized", 7_200, CONFIG)),
            },
        }
    }

    fn dispatch_request() -> EnergyDispatchRequest {
        EnergyDispatchRequest {
            schema_version: "energy-dispatch-v0.1".to_owned(),
            case_id: "synthetic-pv-maintenance-dispatch".to_owned(),
            synthetic: true,
            evidence_class: EvidenceClass::AnalystAssumption,
            evidence_ref: None,
            evidence_as_of: None,
            energy_quantum_kwh: 1.0,
            max_candidate_expansions: 100_000,
            battery: DispatchBattery {
                capacity_quanta: 0,
                initial_soc_quanta: 0,
                charge_input_limit_quanta: 0,
                discharge_withdrawal_limit_quanta: 0,
                charge_efficiency_numerator: 1,
                charge_efficiency_denominator: 1,
                discharge_efficiency_numerator: 1,
                discharge_efficiency_denominator: 1,
                terminal_reserve_quanta: 0,
            },
            intervals: vec![
                DispatchIntervalInput {
                    interval_id: "pre-maintenance".to_owned(),
                    start_unix_seconds: 0,
                    duration_seconds: 3_600,
                    grid_outage: false,
                    pv_available_quanta: 2,
                    grid_import_limit_quanta: 0,
                    essential_demand_quanta: 1,
                    flexible_demand_quanta: 0,
                    grid_cost_units_per_quantum: 1,
                    grid_carbon_grams_per_quantum: 1,
                },
                DispatchIntervalInput {
                    interval_id: "during-maintenance".to_owned(),
                    start_unix_seconds: 3_600,
                    duration_seconds: 3_600,
                    grid_outage: false,
                    pv_available_quanta: 2,
                    grid_import_limit_quanta: 0,
                    essential_demand_quanta: 1,
                    flexible_demand_quanta: 0,
                    grid_cost_units_per_quantum: 1,
                    grid_carbon_grams_per_quantum: 1,
                },
                DispatchIntervalInput {
                    interval_id: "post-maintenance".to_owned(),
                    start_unix_seconds: 7_200,
                    duration_seconds: 3_600,
                    grid_outage: false,
                    pv_available_quanta: 2,
                    grid_import_limit_quanta: 0,
                    essential_demand_quanta: 1,
                    flexible_demand_quanta: 0,
                    grid_cost_units_per_quantum: 1,
                    grid_carbon_grams_per_quantum: 1,
                },
            ],
        }
    }

    fn binding() -> PvAssetDispatchBinding {
        PvAssetDispatchBinding {
            pv_asset_id: "pv-inverter-1".to_owned(),
            pv_configuration_digest: CONFIG.to_owned(),
        }
    }

    fn complete_trace() -> MaintenanceAvailabilityTraceV1 {
        project_maintenance_availability(&trace_request()).expect("valid maintenance trace")
    }

    #[test]
    fn planned_maintenance_removes_only_bound_pv_source_for_exact_interval() {
        let projection =
            project_pv_dispatch_with_maintenance(&complete_trace(), &binding(), &dispatch_request())
                .expect("aligned maintenance trace and dispatch case");

        assert_eq!(
            projection
                .adjusted_dispatch_request
                .intervals
                .iter()
                .map(|interval| interval.pv_available_quanta)
                .collect::<Vec<_>>(),
            vec![2, 0, 2]
        );
        assert_eq!(
            projection
                .interval_bindings
                .iter()
                .map(|interval| interval.asset_availability.clone())
                .collect::<Vec<_>>(),
            vec![
                DispatchAssetAvailability::Available,
                DispatchAssetAvailability::PlannedOutage,
                DispatchAssetAvailability::Available
            ]
        );
        assert!(
            projection
                .adjusted_dispatch_request
                .intervals
                .iter()
                .all(|interval| !interval.grid_outage)
        );
    }

    #[test]
    fn composed_exact_dispatch_exposes_maintenance_induced_service_shortfall() {
        let result = assess_pv_dispatch_with_maintenance(
            &complete_trace(),
            &binding(),
            &dispatch_request(),
        )
        .expect("finite dispatch problem is feasible");

        assert!(result.dispatch_assessment.search_is_exact);
        assert_eq!(
            result.dispatch_assessment.pareto_alternatives[0]
                .objective
                .essential_unserved_quanta,
            1
        );
        assert_eq!(
            result.dispatch_assessment.pareto_alternatives[0]
                .schedule
                .iter()
                .map(|step| step.essential_unserved_quanta)
                .collect::<Vec<_>>(),
            vec![0, 1, 0]
        );
        assert_eq!(
            result.projection.interval_bindings[1].asset_availability,
            DispatchAssetAvailability::PlannedOutage
        );
        assert!(!result.dispatch_assessment.pareto_alternatives[0].schedule[1].grid_outage);
    }

    #[test]
    fn unknown_authority_fails_closed_for_pv_supply() {
        let mut input = trace_request();
        input.work_order.authorization = None;
        input.work_order.isolation = None;
        input.work_order.completion = None;
        input.work_order.inspection = None;
        input.work_order.return_to_service_authorization = None;
        let trace = project_maintenance_availability(&input).expect("unknown is explicit");

        let projection =
            project_pv_dispatch_with_maintenance(&trace, &binding(), &dispatch_request())
                .expect("unknown availability can be conservatively mapped");
        assert_eq!(
            projection.adjusted_dispatch_request.intervals[1].pv_available_quanta,
            0
        );
        assert_eq!(
            projection.interval_bindings[1].asset_availability,
            DispatchAssetAvailability::Unknown
        );
    }

    #[test]
    fn interval_crossing_maintenance_boundary_is_rejected() {
        let trace = complete_trace();
        let mut dispatch = dispatch_request();
        dispatch.intervals[0].duration_seconds = 3_601;

        assert_eq!(
            project_pv_dispatch_with_maintenance(&trace, &binding(), &dispatch),
            Err(MaintenanceDispatchError::DispatchIntervalCrossesMaintenanceBoundary {
                interval_id: "pre-maintenance".to_owned()
            })
        );
    }

    #[test]
    fn replacement_configuration_requires_new_dispatch_profile() {
        let mut input = trace_request();
        let new_config = "pv-inverter-config-b";
        input.work_order.completion =
            Some(evidence("maintenance-completed-new-config", 6_000, new_config));
        input.work_order.post_work_configuration_digest = Some(new_config.to_owned());
        input.work_order.inspection = Some(evidence("inspection-new-config", 6_600, new_config));
        input.work_order.return_to_service_authorization =
            Some(evidence("return-new-config", 7_200, new_config));
        let trace = project_maintenance_availability(&input).expect("valid changed configuration");

        assert_eq!(
            project_pv_dispatch_with_maintenance(&trace, &binding(), &dispatch_request()),
            Err(MaintenanceDispatchError::ConfigurationChangedRequiresRequalification)
        );
    }

    #[test]
    fn wrong_asset_binding_is_rejected() {
        let mut wrong = binding();
        wrong.pv_asset_id = "pv-inverter-2".to_owned();

        assert_eq!(
            project_pv_dispatch_with_maintenance(&complete_trace(), &wrong, &dispatch_request()),
            Err(MaintenanceDispatchError::AssetIdentityMismatch)
        );
    }

    #[test]
    fn dispatch_window_outside_maintenance_horizon_is_rejected() {
        let trace = complete_trace();
        let mut dispatch = dispatch_request();
        dispatch.intervals[0].start_unix_seconds = -3_600;

        assert_eq!(
            project_pv_dispatch_with_maintenance(&trace, &binding(), &dispatch),
            Err(MaintenanceDispatchError::DispatchIntervalsOutsideMaintenanceStudy)
        );
    }

    #[test]
    fn empty_pv_binding_is_rejected() {
        let mut invalid = binding();
        invalid.pv_configuration_digest.clear();

        assert_eq!(
            project_pv_dispatch_with_maintenance(
                &complete_trace(),
                &invalid,
                &dispatch_request()
            ),
            Err(MaintenanceDispatchError::EmptyPvAssetBinding)
        );
    }
}
