// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Demand-matched terminal service accounting for an energy-port graph.
//!
//! A terminal output is not useful merely because a caller labels it "useful".
//! It must be bound to an explicit demand with a matching energy form and any
//! required temperature grade. This module reports delivered and unserved service
//! separately from the graph's first-law energy ledger. It is not a dispatch optimizer,
//! time-series reliability model, or proof that a load was physically metered.

use crate::energy_pathway::EnergyForm;
use crate::energy_ports::{
    EnergyPortDisposition, EnergyPortsAssessment, EnergyPortsError, EnergyPortsRequest,
    assess_energy_ports,
};
use std::collections::BTreeSet;

/// Link a terminal output remainder to the demand it serves.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceOutputRef {
    pub node_id: String,
    pub port_id: String,
}

/// Consumer service demand at the same declared time boundary as the port graph.
///
/// A missing source_output means no compatible delivery has been declared; the full
/// request is recorded as unserved. To satisfy two distinct demands from one carrier,
/// split the physical output into separate output ports. v0.1 intentionally allows at
/// most one demand to claim any terminal output port.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyServiceDemand {
    pub service_id: String,
    pub description: String,
    pub essential: bool,
    pub requested_energy_kwh: f64,
    pub required_form: EnergyForm,
    /// For thermal heat, minimum acceptable supply temperature in degrees Celsius.
    /// Must be absent for non-thermal service in v0.1.
    #[serde(default)]
    pub minimum_temperature_c: Option<f64>,
    /// If absent, the request remains wholly unserved; no implicit source is selected.
    #[serde(default)]
    pub source_output: Option<ServiceOutputRef>,
}

/// One graph plus demands to be satisfied by its terminal outputs.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyServiceScenarioRequest {
    pub schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub graph: EnergyPortsRequest,
    pub demands: Vec<EnergyServiceDemand>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnergyServiceError {
    EmptySchemaVersion,
    UnsupportedSchemaVersion { schema_version: String },
    EmptyScenarioId,
    ScenarioMetadataMismatch,
    NoDemands,
    EmptyServiceId,
    DuplicateServiceId { service_id: String },
    EmptyDescription { service_id: String },
    InvalidRequestedEnergy { service_id: String },
    MissingDemandTemperature { service_id: String },
    UnexpectedDemandTemperature { service_id: String },
    InvalidDemandTemperature { service_id: String },
    MissingSourceNode { service_id: String, node_id: String },
    MissingSourcePort {
        service_id: String,
        node_id: String,
        port_id: String,
    },
    SourcePortNotTerminalUsefulService { service_id: String },
    InvalidSourceOutputValue { service_id: String },
    InvalidSourceTemperature { service_id: String },
    DuplicateOutputClaim { node_id: String, port_id: String },
    ServiceFormMismatch { service_id: String },
    ServiceTemperatureInsufficient {
        service_id: String,
        required_temperature_c: f64,
        available_temperature_c: f64,
    },
    OutputExceedsDemand {
        service_id: String,
        available_kwh: f64,
        requested_kwh: f64,
    },
    MissingDemandForUsefulOutput { node_id: String, port_id: String },
    NonFiniteDerivedValue { context: String },
    GraphAssessmentFailed { error: EnergyPortsError },
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyServiceDemandAssessment {
    pub service_id: String,
    pub description: String,
    pub essential: bool,
    pub requested_energy_kwh: f64,
    pub delivered_energy_kwh: f64,
    pub unserved_energy_kwh: f64,
    pub required_form: EnergyForm,
    pub minimum_temperature_c: Option<f64>,
    pub source_node_id: Option<String>,
    pub source_port_id: Option<String>,
    pub delivered_temperature_c: Option<f64>,
    pub service_balance_residual_kwh: f64,
}

/// Service delivery ledger; unserved demand is a service deficit, not an energy stream.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyServiceAssessment {
    pub assessment_schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub total_requested_service_kwh: f64,
    pub total_delivered_service_kwh: f64,
    pub total_unserved_service_kwh: f64,
    pub essential_requested_service_kwh: f64,
    pub essential_delivered_service_kwh: f64,
    pub essential_unserved_service_kwh: f64,
    pub service_balance_residual_kwh: f64,
    pub demands: Vec<EnergyServiceDemandAssessment>,
}

/// Combined graph energy ledger and service-satisfaction ledger.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyServiceScenarioAssessment {
    pub assessment_schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub graph: EnergyPortsAssessment,
    pub services: EnergyServiceAssessment,
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

fn validate_demand_temperature(
    demand: &EnergyServiceDemand,
) -> Result<(), EnergyServiceError> {
    match (demand.required_form, demand.minimum_temperature_c) {
        (EnergyForm::ThermalHeat, None) => Err(EnergyServiceError::MissingDemandTemperature {
            service_id: demand.service_id.clone(),
        }),
        (EnergyForm::ThermalHeat, Some(celsius)) => {
            let kelvin = celsius + CELSIUS_TO_KELVIN;
            if !celsius.is_finite() || !kelvin.is_finite() || kelvin <= 0.0 {
                Err(EnergyServiceError::InvalidDemandTemperature {
                    service_id: demand.service_id.clone(),
                })
            } else {
                Ok(())
            }
        }
        (_, Some(_)) => Err(EnergyServiceError::UnexpectedDemandTemperature {
            service_id: demand.service_id.clone(),
        }),
        (_, None) => Ok(()),
    }
}

/// Evaluate explicit service demands against an already assessed port graph.
pub fn assess_energy_services(
    scenario_id: &str,
    synthetic: bool,
    demands: &[EnergyServiceDemand],
    graph: &EnergyPortsAssessment,
) -> Result<EnergyServiceAssessment, EnergyServiceError> {
    if scenario_id.trim().is_empty() {
        return Err(EnergyServiceError::EmptyScenarioId);
    }
    if demands.is_empty() {
        return Err(EnergyServiceError::NoDemands);
    }
    if graph.scenario_id != scenario_id || graph.synthetic != synthetic {
        return Err(EnergyServiceError::ScenarioMetadataMismatch);
    }

    let mut service_ids = BTreeSet::new();
    let mut claimed_outputs = BTreeSet::<(String, String)>::new();
    let mut demand_assessments = Vec::with_capacity(demands.len());
    let mut total_requested_service_kwh = 0.0;
    let mut total_delivered_service_kwh = 0.0;
    let mut total_unserved_service_kwh = 0.0;
    let mut essential_requested_service_kwh = 0.0;
    let mut essential_delivered_service_kwh = 0.0;
    let mut essential_unserved_service_kwh = 0.0;

    for demand in demands {
        if demand.service_id.trim().is_empty() {
            return Err(EnergyServiceError::EmptyServiceId);
        }
        if !service_ids.insert(demand.service_id.clone()) {
            return Err(EnergyServiceError::DuplicateServiceId {
                service_id: demand.service_id.clone(),
            });
        }
        if demand.description.trim().is_empty() {
            return Err(EnergyServiceError::EmptyDescription {
                service_id: demand.service_id.clone(),
            });
        }
        if !finite_nonnegative(demand.requested_energy_kwh) {
            return Err(EnergyServiceError::InvalidRequestedEnergy {
                service_id: demand.service_id.clone(),
            });
        }
        validate_demand_temperature(demand)?;

        let mut delivered_energy_kwh = 0.0;
        let mut delivered_temperature_c = None;
        let (source_node_id, source_port_id) = if let Some(source) = &demand.source_output {
            let port = graph.output_ports.iter().find(|port| {
                port.node_id == source.node_id && port.port_id == source.port_id
            }).ok_or_else(|| {
                if graph.nodes.iter().any(|node| node.node_id == source.node_id) {
                    EnergyServiceError::MissingSourcePort {
                        service_id: demand.service_id.clone(),
                        node_id: source.node_id.clone(),
                        port_id: source.port_id.clone(),
                    }
                } else {
                    EnergyServiceError::MissingSourceNode {
                        service_id: demand.service_id.clone(),
                        node_id: source.node_id.clone(),
                    }
                }
            })?;
            if port.remainder_disposition != EnergyPortDisposition::UsefulService {
                return Err(EnergyServiceError::SourcePortNotTerminalUsefulService {
                    service_id: demand.service_id.clone(),
                });
            }
            if !claimed_outputs.insert((source.node_id.clone(), source.port_id.clone())) {
                return Err(EnergyServiceError::DuplicateOutputClaim {
                    node_id: source.node_id.clone(),
                    port_id: source.port_id.clone(),
                });
            }
            if port.form != demand.required_form {
                return Err(EnergyServiceError::ServiceFormMismatch {
                    service_id: demand.service_id.clone(),
                });
            }
            if !finite_nonnegative(port.terminal_remainder_kwh) {
                return Err(EnergyServiceError::InvalidSourceOutputValue {
                    service_id: demand.service_id.clone(),
                });
            }
            if port.form == EnergyForm::ThermalHeat {
                let Some(available_temperature_c) = port.temperature_c else {
                    return Err(EnergyServiceError::InvalidSourceTemperature {
                        service_id: demand.service_id.clone(),
                    });
                };
                let kelvin = available_temperature_c + CELSIUS_TO_KELVIN;
                let Some(required_temperature_c) = demand.minimum_temperature_c else {
                    return Err(EnergyServiceError::MissingDemandTemperature {
                        service_id: demand.service_id.clone(),
                    });
                };
                if !available_temperature_c.is_finite() || !kelvin.is_finite() || kelvin <= 0.0 {
                    return Err(EnergyServiceError::InvalidSourceTemperature {
                        service_id: demand.service_id.clone(),
                    });
                }
                if available_temperature_c + ABSOLUTE_TEMPERATURE_TOLERANCE_C
                    < required_temperature_c
                {
                    return Err(EnergyServiceError::ServiceTemperatureInsufficient {
                        service_id: demand.service_id.clone(),
                        required_temperature_c,
                        available_temperature_c,
                    });
                }
                delivered_temperature_c = Some(available_temperature_c);
            }

            delivered_energy_kwh = port.terminal_remainder_kwh;
            if delivered_energy_kwh > demand.requested_energy_kwh
                + tolerance(delivered_energy_kwh, demand.requested_energy_kwh)
            {
                return Err(EnergyServiceError::OutputExceedsDemand {
                    service_id: demand.service_id.clone(),
                    available_kwh: delivered_energy_kwh,
                    requested_kwh: demand.requested_energy_kwh,
                });
            }
            (Some(source.node_id.clone()), Some(source.port_id.clone()))
        } else {
            (None, None)
        };

        let unserved_energy_kwh = (demand.requested_energy_kwh - delivered_energy_kwh).max(0.0);
        let service_balance_residual_kwh =
            demand.requested_energy_kwh - delivered_energy_kwh - unserved_energy_kwh;
        if !delivered_energy_kwh.is_finite()
            || !unserved_energy_kwh.is_finite()
            || !service_balance_residual_kwh.is_finite()
        {
            return Err(EnergyServiceError::NonFiniteDerivedValue {
                context: format!("service {}", demand.service_id),
            });
        }

        for (total, value, context) in [
            (
                &mut total_requested_service_kwh,
                demand.requested_energy_kwh,
                "total requested service",
            ),
            (
                &mut total_delivered_service_kwh,
                delivered_energy_kwh,
                "total delivered service",
            ),
            (
                &mut total_unserved_service_kwh,
                unserved_energy_kwh,
                "total unserved service",
            ),
        ] {
            *total += value;
            if !total.is_finite() {
                return Err(EnergyServiceError::NonFiniteDerivedValue {
                    context: context.to_owned(),
                });
            }
        }

        if demand.essential {
            for (total, value, context) in [
                (
                    &mut essential_requested_service_kwh,
                    demand.requested_energy_kwh,
                    "essential requested service",
                ),
                (
                    &mut essential_delivered_service_kwh,
                    delivered_energy_kwh,
                    "essential delivered service",
                ),
                (
                    &mut essential_unserved_service_kwh,
                    unserved_energy_kwh,
                    "essential unserved service",
                ),
            ] {
                *total += value;
                if !total.is_finite() {
                    return Err(EnergyServiceError::NonFiniteDerivedValue {
                        context: context.to_owned(),
                    });
                }
            }
        }

        demand_assessments.push(EnergyServiceDemandAssessment {
            service_id: demand.service_id.clone(),
            description: demand.description.clone(),
            essential: demand.essential,
            requested_energy_kwh: demand.requested_energy_kwh,
            delivered_energy_kwh,
            unserved_energy_kwh,
            required_form: demand.required_form,
            minimum_temperature_c: demand.minimum_temperature_c,
            source_node_id,
            source_port_id,
            delivered_temperature_c,
            service_balance_residual_kwh,
        });
    }

    for port in &graph.output_ports {
        if port.remainder_disposition == EnergyPortDisposition::UsefulService
            && !claimed_outputs.contains(&(port.node_id.clone(), port.port_id.clone()))
        {
            return Err(EnergyServiceError::MissingDemandForUsefulOutput {
                node_id: port.node_id.clone(),
                port_id: port.port_id.clone(),
            });
        }
    }

    demand_assessments.sort_by(|left, right| left.service_id.cmp(&right.service_id));
    let service_balance_residual_kwh =
        total_requested_service_kwh - total_delivered_service_kwh - total_unserved_service_kwh;
    if !service_balance_residual_kwh.is_finite() || !close(
        total_requested_service_kwh,
        total_delivered_service_kwh + total_unserved_service_kwh,
    ) {
        return Err(EnergyServiceError::NonFiniteDerivedValue {
            context: "aggregate service balance".to_owned(),
        });
    }

    Ok(EnergyServiceAssessment {
        assessment_schema_version: "1".to_owned(),
        scenario_id: scenario_id.to_owned(),
        synthetic,
        claim_ceiling: concat!(
            "calculated service delivery against declared one-boundary demand and quality; ",
            "not metered proof of service, chronological reliability, or optimization",
        )
        .to_owned(),
        total_requested_service_kwh,
        total_delivered_service_kwh,
        total_unserved_service_kwh,
        essential_requested_service_kwh,
        essential_delivered_service_kwh,
        essential_unserved_service_kwh,
        service_balance_residual_kwh,
        demands: demand_assessments,
    })
}

/// Evaluate the graph and service claims from one combined scenario request.
pub fn assess_energy_service_scenario(
    request: &EnergyServiceScenarioRequest,
) -> Result<EnergyServiceScenarioAssessment, EnergyServiceError> {
    if request.schema_version.trim().is_empty() {
        return Err(EnergyServiceError::EmptySchemaVersion);
    }
    if request.schema_version != "energy-service-v0.1" {
        return Err(EnergyServiceError::UnsupportedSchemaVersion {
            schema_version: request.schema_version.clone(),
        });
    }
    if request.scenario_id.trim().is_empty() {
        return Err(EnergyServiceError::EmptyScenarioId);
    }
    if request.graph.scenario_id != request.scenario_id
        || request.graph.synthetic != request.synthetic
    {
        return Err(EnergyServiceError::ScenarioMetadataMismatch);
    }
    let graph = assess_energy_ports(&request.graph)
        .map_err(|error| EnergyServiceError::GraphAssessmentFailed { error })?;
    let services = assess_energy_services(
        &request.scenario_id,
        request.synthetic,
        &request.demands,
        &graph,
    )?;
    Ok(EnergyServiceScenarioAssessment {
        assessment_schema_version: "1".to_owned(),
        scenario_id: request.scenario_id.clone(),
        synthetic: request.synthetic,
        claim_ceiling: concat!(
            "calculated graph conservation and declared service matching; not authenticated ",
            "metering, physical performance, chronology, or optimizer output",
        )
        .to_owned(),
        graph,
        services,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy_ports::{
        EnergyInputPort, EnergyOutputPort, EnergyPortDisposition, EnergyPortEdge, EnergyPortNode,
    };
    use crate::energy_pathway::EvidenceClass;

    fn fixture() -> EnergyServiceScenarioRequest {
        EnergyServiceScenarioRequest {
            schema_version: "energy-service-v0.1".to_owned(),
            scenario_id: "synthetic-chp-service-matching".to_owned(),
            synthetic: true,
            graph: EnergyPortsRequest {
                schema_version: "energy-ports-v0.1".to_owned(),
                scenario_id: "synthetic-chp-service-matching".to_owned(),
                synthetic: true,
                nodes: vec![
                    EnergyPortNode {
                        node_id: "chp".to_owned(),
                        boundary_description: "synthetic fuel to electricity and heat".to_owned(),
                        evidence_class: EvidenceClass::AnalystAssumption,
                        evidence_ref: None,
                        evidence_as_of: None,
                        inputs: vec![EnergyInputPort {
                            port_id: "fuel".to_owned(),
                            form: EnergyForm::Chemical,
                            energy_kwh: 100.0,
                            external_energy_kwh: 100.0,
                            external_source_id: Some("fuel-batch-1".to_owned()),
                            temperature_c: None,
                        }],
                        outputs: vec![
                            EnergyOutputPort {
                                port_id: "electricity".to_owned(),
                                form: EnergyForm::Electricity,
                                energy_kwh: 35.0,
                                temperature_c: None,
                                remainder_disposition: EnergyPortDisposition::UsefulService,
                            },
                            EnergyOutputPort {
                                port_id: "heat".to_owned(),
                                form: EnergyForm::ThermalHeat,
                                energy_kwh: 14.0,
                                temperature_c: Some(80.0),
                                remainder_disposition: EnergyPortDisposition::UsefulService,
                            },
                            EnergyOutputPort {
                                port_id: "exhaust".to_owned(),
                                form: EnergyForm::ThermalHeat,
                                energy_kwh: 51.0,
                                temperature_c: Some(150.0),
                                remainder_disposition: EnergyPortDisposition::Rejected,
                            },
                        ],
                    },
                ],
                edges: vec![],
            },
            demands: vec![
                EnergyServiceDemand {
                    service_id: "electric-service".to_owned(),
                    description: "site electricity load".to_owned(),
                    essential: true,
                    requested_energy_kwh: 40.0,
                    required_form: EnergyForm::Electricity,
                    minimum_temperature_c: None,
                    source_output: Some(ServiceOutputRef {
                        node_id: "chp".to_owned(),
                        port_id: "electricity".to_owned(),
                    }),
                },
                EnergyServiceDemand {
                    service_id: "process-heat-service".to_owned(),
                    description: "process heat at useful grade".to_owned(),
                    essential: true,
                    requested_energy_kwh: 20.0,
                    required_form: EnergyForm::ThermalHeat,
                    minimum_temperature_c: Some(75.0),
                    source_output: Some(ServiceOutputRef {
                        node_id: "chp".to_owned(),
                        port_id: "heat".to_owned(),
                    }),
                },
            ],
        }
    }

    #[test]
    fn partial_service_is_reported_without_changing_energy_balance() {
        let assessment = assess_energy_service_scenario(&fixture()).unwrap();
        assert_eq!(assessment.graph.external_input_energy_kwh, 100.0);
        assert_eq!(assessment.graph.terminal_disposition_energy_kwh, 100.0);
        assert_eq!(assessment.services.total_requested_service_kwh, 60.0);
        assert_eq!(assessment.services.total_delivered_service_kwh, 49.0);
        assert_eq!(assessment.services.total_unserved_service_kwh, 11.0);
        assert_eq!(assessment.services.essential_unserved_service_kwh, 11.0);
        let electricity = assessment.services.demands.iter()
            .find(|demand| demand.service_id == "electric-service").unwrap();
        assert_eq!(electricity.delivered_energy_kwh, 35.0);
        assert_eq!(electricity.unserved_energy_kwh, 5.0);
        let heat = assessment.services.demands.iter()
            .find(|demand| demand.service_id == "process-heat-service").unwrap();
        assert_eq!(heat.delivered_energy_kwh, 14.0);
        assert_eq!(heat.unserved_energy_kwh, 6.0);
        assert!(assessment.services.service_balance_residual_kwh.abs() < 1.0e-9);
    }

    #[test]
    fn thermal_service_uses_temperature_grade_not_energy_quantity_alone() {
        let mut request = fixture();
        request.demands[1].minimum_temperature_c = Some(85.0);
        let result = assess_energy_service_scenario(&request);
        assert!(matches!(result, Err(EnergyServiceError::ServiceTemperatureInsufficient { .. })));
    }

    #[test]
    fn malformed_assessed_thermal_source_fails_closed_without_panicking() {
        let request = fixture();
        let mut graph = assess_energy_ports(&request.graph).unwrap();
        let heat = graph.output_ports.iter_mut()
            .find(|port| port.node_id == "chp" && port.port_id == "heat")
            .unwrap();
        heat.temperature_c = None;
        assert!(matches!(
            assess_energy_services(
                &request.scenario_id,
                request.synthetic,
                &request.demands,
                &graph,
            ),
            Err(EnergyServiceError::InvalidSourceTemperature { .. })
        ));
    }

    #[test]
    fn service_delivery_cannot_exceed_requested_quantity() {
        let mut request = fixture();
        request.demands[0].requested_energy_kwh = 20.0;
        assert!(matches!(
            assess_energy_service_scenario(&request),
            Err(EnergyServiceError::OutputExceedsDemand { .. })
        ));
    }

    #[test]
    fn useful_output_without_demand_link_fails_closed() {
        let mut request = fixture();
        request.demands[0].source_output = None;
        assert!(matches!(
            assess_energy_service_scenario(&request),
            Err(EnergyServiceError::MissingDemandForUsefulOutput { .. })
        ));
    }

    #[test]
    fn demand_without_a_source_is_reported_fully_unserved() {
        let mut request = fixture();
        request.demands[1].source_output = None;
        // A useful output may not remain unlinked. Mark the heat stream rejected
        // in this negative control to isolate the independent demand shortfall.
        request.graph.nodes[0].outputs[1].remainder_disposition = EnergyPortDisposition::Rejected;
        let assessment = assess_energy_service_scenario(&request).unwrap();
        let heat = assessment.services.demands.iter()
            .find(|demand| demand.service_id == "process-heat-service").unwrap();
        assert_eq!(heat.delivered_energy_kwh, 0.0);
        assert_eq!(heat.unserved_energy_kwh, 20.0);
    }

    #[test]
    fn one_output_cannot_be_claimed_for_two_services() {
        let mut request = fixture();
        request.demands.push(EnergyServiceDemand {
            service_id: "duplicate".to_owned(),
            description: "attempt to re-use same output".to_owned(),
            essential: false,
            requested_energy_kwh: 40.0,
            required_form: EnergyForm::Electricity,
            minimum_temperature_c: None,
            source_output: Some(ServiceOutputRef {
                node_id: "chp".to_owned(),
                port_id: "electricity".to_owned(),
            }),
        });
        assert!(matches!(
            assess_energy_service_scenario(&request),
            Err(EnergyServiceError::DuplicateOutputClaim { .. })
        ));
    }

    #[test]
    fn carrier_mismatch_is_refused() {
        let mut request = fixture();
        request.demands[0].required_form = EnergyForm::Mechanical;
        assert!(matches!(
            assess_energy_service_scenario(&request),
            Err(EnergyServiceError::ServiceFormMismatch { .. })
        ));
    }

    #[test]
    fn missing_and_invalid_thermal_demand_quality_fail_closed() {
        let mut request = fixture();
        request.demands[1].minimum_temperature_c = None;
        assert!(matches!(
            assess_energy_service_scenario(&request),
            Err(EnergyServiceError::MissingDemandTemperature { .. })
        ));

        let mut request = fixture();
        request.demands[1].minimum_temperature_c = Some(-273.15);
        assert!(matches!(
            assess_energy_service_scenario(&request),
            Err(EnergyServiceError::InvalidDemandTemperature { .. })
        ));
    }

    #[test]
    fn duplicate_service_ids_are_rejected() {
        let mut request = fixture();
        request.demands[1].service_id = request.demands[0].service_id.clone();
        assert!(matches!(
            assess_energy_service_scenario(&request),
            Err(EnergyServiceError::DuplicateServiceId { .. })
        ));
    }
}
