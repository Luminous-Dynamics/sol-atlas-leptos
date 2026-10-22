// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Typed, acyclic energy-port accounting for multi-output conversion systems.
//!
//! Every node has a declared energy boundary and a first-law balance. Output energy
//! may be split across downstream edges, but routed portions cannot exceed the source
//! port; any unrouted remainder has one explicit terminal disposition. Internal edges
//! are not counted again as external supply or final service. Cycles are rejected:
//! temporal carry-over belongs in the stateful storage contract.

use crate::energy_pathway::{EnergyForm, EvidenceClass};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Explicit treatment of an output port's energy that is not routed downstream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyPortDisposition {
    UsefulService,
    Rejected,
    UnconvertedResource,
    Curtailed,
}

/// One node input port. External energy and inbound edge flows sum to energy_kwh.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyInputPort {
    pub port_id: String,
    pub form: EnergyForm,
    /// Total energy accepted by the node at this port, in kWh.
    pub energy_kwh: f64,
    /// Portion supplied across the graph boundary rather than by another node.
    pub external_energy_kwh: f64,
    /// Unique identity for this external supply stream. Re-used streams must be
    /// represented once and split through ordinary graph edges, never duplicated.
    #[serde(default)]
    pub external_source_id: Option<String>,
    /// Required for thermal heat; prohibited for other forms in v0.1.
    #[serde(default)]
    pub temperature_c: Option<f64>,
}

/// One node output port. Its remaining energy after downstream routing is explicit.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyOutputPort {
    pub port_id: String,
    pub form: EnergyForm,
    /// Full output-stream quantity, including portions that may be routed to other nodes.
    pub energy_kwh: f64,
    /// Required for thermal heat; prohibited for other forms in v0.1.
    #[serde(default)]
    pub temperature_c: Option<f64>,
    /// Terminal category assigned only to the portion not routed via an edge.
    pub remainder_disposition: EnergyPortDisposition,
}

/// Converter/service node with explicit system boundary and evidence ceiling.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyPortNode {
    pub node_id: String,
    pub boundary_description: String,
    pub evidence_class: EvidenceClass,
    #[serde(default)]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub evidence_as_of: Option<String>,
    pub inputs: Vec<EnergyInputPort>,
    pub outputs: Vec<EnergyOutputPort>,
}

/// Internal transfer. One output may split across multiple edges; total edge flow
/// leaving that output must not exceed the declared output-port quantity.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyPortEdge {
    pub edge_id: String,
    pub from_node_id: String,
    pub from_port_id: String,
    pub to_node_id: String,
    pub to_port_id: String,
    pub energy_kwh: f64,
}

/// Directed acyclic graph for one declared scenario and one time boundary.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyPortsRequest {
    pub schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub nodes: Vec<EnergyPortNode>,
    pub edges: Vec<EnergyPortEdge>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnergyPortsError {
    EmptySchemaVersion,
    UnsupportedSchemaVersion { schema_version: String },
    EmptyScenarioId,
    NoNodes,
    EmptyNodeId,
    DuplicateNodeId { node_id: String },
    EmptyBoundaryDescription { node_id: String },
    NodeWithoutPorts { node_id: String },
    EmptyPortId { node_id: String },
    DuplicateInputPortId { node_id: String, port_id: String },
    DuplicateOutputPortId { node_id: String, port_id: String },
    InvalidPortEnergy { node_id: String, port_id: String },
    InvalidExternalEnergy { node_id: String, port_id: String },
    MissingExternalSourceId { node_id: String, port_id: String },
    UnexpectedExternalSourceId { node_id: String, port_id: String },
    DuplicateExternalSourceId { source_id: String },
    MissingThermalTemperature { node_id: String, port_id: String },
    UnexpectedTemperature { node_id: String, port_id: String },
    InvalidAbsoluteTemperature { node_id: String, port_id: String },
    MissingEvidenceReference { node_id: String },
    EmptyEdgeId,
    DuplicateEdgeId { edge_id: String },
    InvalidEdgeEnergy { edge_id: String },
    MissingSourceNode { edge_id: String, node_id: String },
    MissingSourcePort { edge_id: String, node_id: String, port_id: String },
    MissingTargetNode { edge_id: String, node_id: String },
    MissingTargetPort { edge_id: String, node_id: String, port_id: String },
    GraphCycleDetected,
    EnergyFormMismatch { edge_id: String },
    TemperatureGradeMismatch { edge_id: String },
    OutputOverdrawn {
        node_id: String,
        port_id: String,
        routed_kwh: f64,
        available_kwh: f64,
    },
    InputRoutingMismatch {
        node_id: String,
        port_id: String,
        declared_kwh: f64,
        accounted_kwh: f64,
    },
    NodeEnergyBalanceMismatch {
        node_id: String,
        input_kwh: f64,
        output_kwh: f64,
    },
    GraphBalanceMismatch {
        external_input_kwh: f64,
        terminal_disposition_kwh: f64,
    },
    NonFiniteDerivedValue { context: String },
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyPortNodeAssessment {
    pub node_id: String,
    pub boundary_description: String,
    pub input_energy_kwh: f64,
    pub output_energy_kwh: f64,
    pub balance_residual_kwh: f64,
    pub evidence_class: EvidenceClass,
    pub evidence_ref: Option<String>,
    pub evidence_as_of: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyInputPortAssessment {
    pub node_id: String,
    pub port_id: String,
    pub form: EnergyForm,
    pub energy_kwh: f64,
    pub external_energy_kwh: f64,
    pub external_source_id: Option<String>,
    pub routed_energy_kwh: f64,
    pub temperature_c: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyPortEdgeAssessment {
    pub edge_id: String,
    pub from_node_id: String,
    pub from_port_id: String,
    pub to_node_id: String,
    pub to_port_id: String,
    pub energy_kwh: f64,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyOutputPortAssessment {
    pub node_id: String,
    pub port_id: String,
    pub form: EnergyForm,
    pub energy_kwh: f64,
    pub routed_energy_kwh: f64,
    pub terminal_remainder_kwh: f64,
    pub remainder_disposition: EnergyPortDisposition,
    pub temperature_c: Option<f64>,
}

/// Scenario-level totals. Routed transfers are diagnostics, not an extra energy source
/// or terminal service quantity. System closure is external input versus terminal remainders.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EnergyPortsAssessment {
    pub assessment_schema_version: String,
    pub scenario_id: String,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub external_input_energy_kwh: f64,
    pub routed_energy_kwh: f64,
    pub terminal_useful_service_kwh: f64,
    pub terminal_rejected_kwh: f64,
    pub terminal_unconverted_resource_kwh: f64,
    pub terminal_curtailed_kwh: f64,
    pub terminal_disposition_energy_kwh: f64,
    pub system_balance_residual_kwh: f64,
    pub nodes: Vec<EnergyPortNodeAssessment>,
    pub input_ports: Vec<EnergyInputPortAssessment>,
    pub edges: Vec<EnergyPortEdgeAssessment>,
    pub output_ports: Vec<EnergyOutputPortAssessment>,
}

const CELSIUS_TO_KELVIN: f64 = 273.15;
const ABSOLUTE_TOLERANCE_KWH: f64 = 1.0e-9;
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

fn checked_add(total: &mut f64, amount: f64, context: &str) -> Result<(), EnergyPortsError> {
    *total += amount;
    if !total.is_finite() {
        return Err(EnergyPortsError::NonFiniteDerivedValue {
            context: context.to_owned(),
        });
    }
    Ok(())
}

fn validate_temperature(
    node_id: &str,
    port_id: &str,
    form: EnergyForm,
    temperature_c: Option<f64>,
) -> Result<(), EnergyPortsError> {
    match (form, temperature_c) {
        (EnergyForm::ThermalHeat, None) => {
            Err(EnergyPortsError::MissingThermalTemperature {
                node_id: node_id.to_owned(),
                port_id: port_id.to_owned(),
            })
        }
        (EnergyForm::ThermalHeat, Some(celsius)) => {
            let kelvin = celsius + CELSIUS_TO_KELVIN;
            if !celsius.is_finite() || !kelvin.is_finite() || kelvin <= 0.0 {
                Err(EnergyPortsError::InvalidAbsoluteTemperature {
                    node_id: node_id.to_owned(),
                    port_id: port_id.to_owned(),
                })
            } else {
                Ok(())
            }
        }
        (_, Some(_)) => Err(EnergyPortsError::UnexpectedTemperature {
            node_id: node_id.to_owned(),
            port_id: port_id.to_owned(),
        }),
        (_, None) => Ok(()),
    }
}

/// Validate and assess a boundary-explicit multi-output graph.
///
/// Node balance is checked before portfolio aggregation. An edge is an internal
/// transfer: it satisfies part of one target input but does not become new external
/// energy or a terminal disposition. Each output's un-routed remainder is assigned
/// exactly one declared disposition. All cycles fail closed; model time/state belongs
/// in the separate storage trajectory contract.
pub fn assess_energy_ports(
    request: &EnergyPortsRequest,
) -> Result<EnergyPortsAssessment, EnergyPortsError> {
    if request.schema_version.trim().is_empty() {
        return Err(EnergyPortsError::EmptySchemaVersion);
    }
    if request.schema_version != "energy-ports-v0.1" {
        return Err(EnergyPortsError::UnsupportedSchemaVersion {
            schema_version: request.schema_version.clone(),
        });
    }
    if request.scenario_id.trim().is_empty() {
        return Err(EnergyPortsError::EmptyScenarioId);
    }
    if request.nodes.is_empty() {
        return Err(EnergyPortsError::NoNodes);
    }

    let mut node_indices = BTreeMap::<String, usize>::new();
    let mut input_ids = BTreeSet::<(String, String)>::new();
    let mut output_ids = BTreeSet::<(String, String)>::new();
    let mut external_source_ids = BTreeSet::<String>::new();

    for (index, node) in request.nodes.iter().enumerate() {
        if node.node_id.trim().is_empty() {
            return Err(EnergyPortsError::EmptyNodeId);
        }
        if node_indices.insert(node.node_id.clone(), index).is_some() {
            return Err(EnergyPortsError::DuplicateNodeId {
                node_id: node.node_id.clone(),
            });
        }
        if node.boundary_description.trim().is_empty() {
            return Err(EnergyPortsError::EmptyBoundaryDescription {
                node_id: node.node_id.clone(),
            });
        }
        if node.inputs.is_empty() || node.outputs.is_empty() {
            return Err(EnergyPortsError::NodeWithoutPorts {
                node_id: node.node_id.clone(),
            });
        }
        if node.evidence_class != EvidenceClass::AnalystAssumption
            && node
                .evidence_ref
                .as_deref()
                .is_none_or(|reference| reference.trim().is_empty())
        {
            return Err(EnergyPortsError::MissingEvidenceReference {
                node_id: node.node_id.clone(),
            });
        }

        let mut local_inputs = BTreeSet::new();
        for port in &node.inputs {
            if port.port_id.trim().is_empty() {
                return Err(EnergyPortsError::EmptyPortId {
                    node_id: node.node_id.clone(),
                });
            }
            if !local_inputs.insert(port.port_id.clone())
                || !input_ids.insert((node.node_id.clone(), port.port_id.clone()))
            {
                return Err(EnergyPortsError::DuplicateInputPortId {
                    node_id: node.node_id.clone(),
                    port_id: port.port_id.clone(),
                });
            }
            if !finite_nonnegative(port.energy_kwh) {
                return Err(EnergyPortsError::InvalidPortEnergy {
                    node_id: node.node_id.clone(),
                    port_id: port.port_id.clone(),
                });
            }
            let external_exceeds_declared = port.external_energy_kwh
                > port.energy_kwh
                    + tolerance(port.external_energy_kwh, port.energy_kwh);
            if !finite_nonnegative(port.external_energy_kwh) || external_exceeds_declared {
                return Err(EnergyPortsError::InvalidExternalEnergy {
                    node_id: node.node_id.clone(),
                    port_id: port.port_id.clone(),
                });
            }
            if port.external_energy_kwh > 0.0 {
                let Some(source_id) = port.external_source_id.as_deref() else {
                    return Err(EnergyPortsError::MissingExternalSourceId {
                        node_id: node.node_id.clone(),
                        port_id: port.port_id.clone(),
                    });
                };
                if source_id.trim().is_empty() {
                    return Err(EnergyPortsError::MissingExternalSourceId {
                        node_id: node.node_id.clone(),
                        port_id: port.port_id.clone(),
                    });
                }
                if !external_source_ids.insert(source_id.to_owned()) {
                    return Err(EnergyPortsError::DuplicateExternalSourceId {
                        source_id: source_id.to_owned(),
                    });
                }
            } else if port.external_source_id.is_some() {
                return Err(EnergyPortsError::UnexpectedExternalSourceId {
                    node_id: node.node_id.clone(),
                    port_id: port.port_id.clone(),
                });
            }
            validate_temperature(&node.node_id, &port.port_id, port.form, port.temperature_c)?;
        }

        let mut local_outputs = BTreeSet::new();
        for port in &node.outputs {
            if port.port_id.trim().is_empty() {
                return Err(EnergyPortsError::EmptyPortId {
                    node_id: node.node_id.clone(),
                });
            }
            if !local_outputs.insert(port.port_id.clone())
                || !output_ids.insert((node.node_id.clone(), port.port_id.clone()))
            {
                return Err(EnergyPortsError::DuplicateOutputPortId {
                    node_id: node.node_id.clone(),
                    port_id: port.port_id.clone(),
                });
            }
            if !finite_nonnegative(port.energy_kwh) {
                return Err(EnergyPortsError::InvalidPortEnergy {
                    node_id: node.node_id.clone(),
                    port_id: port.port_id.clone(),
                });
            }
            validate_temperature(&node.node_id, &port.port_id, port.form, port.temperature_c)?;
        }
    }

    let mut edge_ids = BTreeSet::new();
    let mut adjacency = vec![Vec::<usize>::new(); request.nodes.len()];
    let mut indegree = vec![0usize; request.nodes.len()];
    let mut resolved_edges = Vec::with_capacity(request.edges.len());
    for edge in &request.edges {
        if edge.edge_id.trim().is_empty() {
            return Err(EnergyPortsError::EmptyEdgeId);
        }
        if !edge_ids.insert(edge.edge_id.clone()) {
            return Err(EnergyPortsError::DuplicateEdgeId {
                edge_id: edge.edge_id.clone(),
            });
        }
        if !edge.energy_kwh.is_finite() || edge.energy_kwh <= 0.0 {
            return Err(EnergyPortsError::InvalidEdgeEnergy {
                edge_id: edge.edge_id.clone(),
            });
        }
        let from_index = *node_indices.get(&edge.from_node_id).ok_or_else(|| {
            EnergyPortsError::MissingSourceNode {
                edge_id: edge.edge_id.clone(),
                node_id: edge.from_node_id.clone(),
            }
        })?;
        let to_index = *node_indices.get(&edge.to_node_id).ok_or_else(|| {
            EnergyPortsError::MissingTargetNode {
                edge_id: edge.edge_id.clone(),
                node_id: edge.to_node_id.clone(),
            }
        })?;
        if !request.nodes[from_index]
            .outputs
            .iter()
            .any(|port| port.port_id == edge.from_port_id)
        {
            return Err(EnergyPortsError::MissingSourcePort {
                edge_id: edge.edge_id.clone(),
                node_id: edge.from_node_id.clone(),
                port_id: edge.from_port_id.clone(),
            });
        }
        if !request.nodes[to_index]
            .inputs
            .iter()
            .any(|port| port.port_id == edge.to_port_id)
        {
            return Err(EnergyPortsError::MissingTargetPort {
                edge_id: edge.edge_id.clone(),
                node_id: edge.to_node_id.clone(),
                port_id: edge.to_port_id.clone(),
            });
        }
        adjacency[from_index].push(to_index);
        indegree[to_index] += 1;
        resolved_edges.push((edge, from_index, to_index));
    }

    // Check a deterministic topological traversal before evaluating flow semantics.
    let mut ready = VecDeque::new();
    let mut node_order = (0..request.nodes.len()).collect::<Vec<_>>();
    node_order.sort_by(|left, right| {
        request.nodes[*left]
            .node_id
            .cmp(&request.nodes[*right].node_id)
    });
    for index in &node_order {
        if indegree[*index] == 0 {
            ready.push_back(*index);
        }
        adjacency[*index].sort_by(|left, right| {
            request.nodes[*left].node_id.cmp(&request.nodes[*right].node_id)
        });
    }
    let mut topologically_seen = 0usize;
    while let Some(index) = ready.pop_front() {
        topologically_seen += 1;
        for next in &adjacency[index] {
            indegree[*next] -= 1;
            if indegree[*next] == 0 {
                ready.push_back(*next);
            }
        }
    }
    if topologically_seen != request.nodes.len() {
        return Err(EnergyPortsError::GraphCycleDetected);
    }

    let mut routed_by_output = BTreeMap::<(String, String), f64>::new();
    let mut incoming_by_input = BTreeMap::<(String, String), f64>::new();
    let mut routed_energy_kwh = 0.0;
    for (edge, from_index, to_index) in &resolved_edges {
        let source = &request.nodes[*from_index];
        let target = &request.nodes[*to_index];
        let source_port = source
            .outputs
            .iter()
            .find(|port| port.port_id == edge.from_port_id)
            .unwrap();
        let target_port = target
            .inputs
            .iter()
            .find(|port| port.port_id == edge.to_port_id)
            .unwrap();
        if source_port.form != target_port.form {
            return Err(EnergyPortsError::EnergyFormMismatch {
                edge_id: edge.edge_id.clone(),
            });
        }
        if source_port.form == EnergyForm::ThermalHeat
            && !close(
                source_port.temperature_c.unwrap(),
                target_port.temperature_c.unwrap(),
            )
        {
            return Err(EnergyPortsError::TemperatureGradeMismatch {
                edge_id: edge.edge_id.clone(),
            });
        }
        checked_add(
            routed_by_output
                .entry((edge.from_node_id.clone(), edge.from_port_id.clone()))
                .or_default(),
            edge.energy_kwh,
            &format!("routed output {}:{}", edge.from_node_id, edge.from_port_id),
        )?;
        checked_add(
            incoming_by_input
                .entry((edge.to_node_id.clone(), edge.to_port_id.clone()))
                .or_default(),
            edge.energy_kwh,
            &format!("received input {}:{}", edge.to_node_id, edge.to_port_id),
        )?;
        checked_add(&mut routed_energy_kwh, edge.energy_kwh, "total routed energy")?;
    }

    let mut external_input_energy_kwh = 0.0;
    for node in &request.nodes {
        for input in &node.inputs {
            checked_add(
                &mut external_input_energy_kwh,
                input.external_energy_kwh,
                "total external input energy",
            )?;
        }
    }

    // Output overdrawn checks precede input matching so duplicated/over-allocated
    // co-products get a direct refusal reason.
    for node in &request.nodes {
        for output in &node.outputs {
            let routed = routed_by_output
                .get(&(node.node_id.clone(), output.port_id.clone()))
                .copied()
                .unwrap_or(0.0);
            if routed > output.energy_kwh + tolerance(routed, output.energy_kwh) {
                return Err(EnergyPortsError::OutputOverdrawn {
                    node_id: node.node_id.clone(),
                    port_id: output.port_id.clone(),
                    routed_kwh: routed,
                    available_kwh: output.energy_kwh,
                });
            }
        }
    }

    for node in &request.nodes {
        for input in &node.inputs {
            let received = incoming_by_input
                .get(&(node.node_id.clone(), input.port_id.clone()))
                .copied()
                .unwrap_or(0.0);
            let accounted = input.external_energy_kwh + received;
            if !accounted.is_finite() {
                return Err(EnergyPortsError::NonFiniteDerivedValue {
                    context: format!("input {}:{}", node.node_id, input.port_id),
                });
            }
            if !close(accounted, input.energy_kwh) {
                return Err(EnergyPortsError::InputRoutingMismatch {
                    node_id: node.node_id.clone(),
                    port_id: input.port_id.clone(),
                    declared_kwh: input.energy_kwh,
                    accounted_kwh: accounted,
                });
            }
        }
    }

    let mut node_assessments = Vec::with_capacity(request.nodes.len());
    for node in &request.nodes {
        let mut input_energy_kwh = 0.0;
        let mut output_energy_kwh = 0.0;
        for input in &node.inputs {
            checked_add(&mut input_energy_kwh, input.energy_kwh, "node input energy")?;
        }
        for output in &node.outputs {
            checked_add(&mut output_energy_kwh, output.energy_kwh, "node output energy")?;
        }
        let residual = input_energy_kwh - output_energy_kwh;
        if !residual.is_finite() {
            return Err(EnergyPortsError::NonFiniteDerivedValue {
                context: format!("node balance {}", node.node_id),
            });
        }
        if !close(input_energy_kwh, output_energy_kwh) {
            return Err(EnergyPortsError::NodeEnergyBalanceMismatch {
                node_id: node.node_id.clone(),
                input_kwh: input_energy_kwh,
                output_kwh: output_energy_kwh,
            });
        }
        node_assessments.push(EnergyPortNodeAssessment {
            node_id: node.node_id.clone(),
            boundary_description: node.boundary_description.clone(),
            input_energy_kwh,
            output_energy_kwh,
            balance_residual_kwh: residual,
            evidence_class: node.evidence_class,
            evidence_ref: node.evidence_ref.clone(),
            evidence_as_of: node.evidence_as_of.clone(),
        });
    }

    let mut input_port_assessments = Vec::new();
    for node in &request.nodes {
        for input in &node.inputs {
            let routed = incoming_by_input
                .get(&(node.node_id.clone(), input.port_id.clone()))
                .copied()
                .unwrap_or(0.0);
            input_port_assessments.push(EnergyInputPortAssessment {
                node_id: node.node_id.clone(),
                port_id: input.port_id.clone(),
                form: input.form,
                energy_kwh: input.energy_kwh,
                external_energy_kwh: input.external_energy_kwh,
                external_source_id: input.external_source_id.clone(),
                routed_energy_kwh: routed,
                temperature_c: input.temperature_c,
            });
        }
    }
    input_port_assessments.sort_by(|left, right| {
        (&left.node_id, &left.port_id).cmp(&(&right.node_id, &right.port_id))
    });

    let mut output_port_assessments = Vec::new();
    let mut terminal_useful_service_kwh = 0.0;
    let mut terminal_rejected_kwh = 0.0;
    let mut terminal_unconverted_resource_kwh = 0.0;
    let mut terminal_curtailed_kwh = 0.0;
    for node in &request.nodes {
        for output in &node.outputs {
            let routed = routed_by_output
                .get(&(node.node_id.clone(), output.port_id.clone()))
                .copied()
                .unwrap_or(0.0);
            let terminal_remainder_kwh = (output.energy_kwh - routed).max(0.0);
            let (total, context) = match output.remainder_disposition {
                EnergyPortDisposition::UsefulService => {
                    (&mut terminal_useful_service_kwh, "terminal useful service")
                }
                EnergyPortDisposition::Rejected => {
                    (&mut terminal_rejected_kwh, "terminal rejected energy")
                }
                EnergyPortDisposition::UnconvertedResource => (
                    &mut terminal_unconverted_resource_kwh,
                    "terminal unconverted resource",
                ),
                EnergyPortDisposition::Curtailed => {
                    (&mut terminal_curtailed_kwh, "terminal curtailed energy")
                }
            };
            checked_add(total, terminal_remainder_kwh, context)?;
            output_port_assessments.push(EnergyOutputPortAssessment {
                node_id: node.node_id.clone(),
                port_id: output.port_id.clone(),
                form: output.form,
                energy_kwh: output.energy_kwh,
                routed_energy_kwh: routed,
                terminal_remainder_kwh,
                remainder_disposition: output.remainder_disposition,
                temperature_c: output.temperature_c,
            });
        }
    }

    let terminal_disposition_energy_kwh = terminal_useful_service_kwh
        + terminal_rejected_kwh
        + terminal_unconverted_resource_kwh
        + terminal_curtailed_kwh;
    if !terminal_disposition_energy_kwh.is_finite() {
        return Err(EnergyPortsError::NonFiniteDerivedValue {
            context: "terminal disposition energy".to_owned(),
        });
    }
    let system_balance_residual_kwh =
        external_input_energy_kwh - terminal_disposition_energy_kwh;
    if !system_balance_residual_kwh.is_finite() {
        return Err(EnergyPortsError::NonFiniteDerivedValue {
            context: "system energy balance".to_owned(),
        });
    }
    if !close(external_input_energy_kwh, terminal_disposition_energy_kwh) {
        return Err(EnergyPortsError::GraphBalanceMismatch {
            external_input_kwh: external_input_energy_kwh,
            terminal_disposition_kwh: terminal_disposition_energy_kwh,
        });
    }

    node_assessments.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    output_port_assessments.sort_by(|left, right| {
        (&left.node_id, &left.port_id).cmp(&(&right.node_id, &right.port_id))
    });
    let mut edge_assessments = resolved_edges
        .iter()
        .map(|(edge, _, _)| EnergyPortEdgeAssessment {
            edge_id: edge.edge_id.clone(),
            from_node_id: edge.from_node_id.clone(),
            from_port_id: edge.from_port_id.clone(),
            to_node_id: edge.to_node_id.clone(),
            to_port_id: edge.to_port_id.clone(),
            energy_kwh: edge.energy_kwh,
        })
        .collect::<Vec<_>>();
    edge_assessments.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));

    Ok(EnergyPortsAssessment {
        assessment_schema_version: "1".to_owned(),
        scenario_id: request.scenario_id.clone(),
        synthetic: request.synthetic,
        claim_ceiling: concat!(
            "boundary-declared acyclic graph accounting; values are not authenticated metering ",
            "or proof of physical performance; routing and balances are scenario calculations",
        )
        .to_owned(),
        external_input_energy_kwh,
        routed_energy_kwh,
        terminal_useful_service_kwh,
        terminal_rejected_kwh,
        terminal_unconverted_resource_kwh,
        terminal_curtailed_kwh,
        terminal_disposition_energy_kwh,
        system_balance_residual_kwh,
        nodes: node_assessments,
        input_ports: input_port_assessments,
        edges: edge_assessments,
        output_ports: output_port_assessments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chp_fixture() -> EnergyPortsRequest {
        EnergyPortsRequest {
            schema_version: "energy-ports-v0.1".to_owned(),
            scenario_id: "synthetic-chp-with-heat-recovery".to_owned(),
            synthetic: true,
            nodes: vec![
                EnergyPortNode {
                    node_id: "chp".to_owned(),
                    boundary_description: concat!(
                        "fuel input to net electricity and thermal outputs"
                    )
                    .to_owned(),
                    evidence_class: EvidenceClass::AnalystAssumption,
                    evidence_ref: None,
                    evidence_as_of: None,
                    inputs: vec![EnergyInputPort {
                        port_id: "fuel".to_owned(),
                        form: EnergyForm::Chemical,
                        energy_kwh: 100.0,
                        external_energy_kwh: 100.0,
                        external_source_id: Some("fuel-batch-001".to_owned()),
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
                            port_id: "heat-coproduct".to_owned(),
                            form: EnergyForm::ThermalHeat,
                            energy_kwh: 45.0,
                            temperature_c: Some(80.0),
                            remainder_disposition: EnergyPortDisposition::Rejected,
                        },
                        EnergyOutputPort {
                            port_id: "exhaust".to_owned(),
                            form: EnergyForm::ThermalHeat,
                            energy_kwh: 20.0,
                            temperature_c: Some(150.0),
                            remainder_disposition: EnergyPortDisposition::Rejected,
                        },
                    ],
                },
                EnergyPortNode {
                    node_id: "process-load".to_owned(),
                    boundary_description: "heat delivery boundary for a process load".to_owned(),
                    evidence_class: EvidenceClass::AnalystAssumption,
                    evidence_ref: None,
                    evidence_as_of: None,
                    inputs: vec![EnergyInputPort {
                        port_id: "heat-in".to_owned(),
                        form: EnergyForm::ThermalHeat,
                        energy_kwh: 15.0,
                        external_energy_kwh: 0.0,
                        external_source_id: None,
                        temperature_c: Some(80.0),
                    }],
                    outputs: vec![
                        EnergyOutputPort {
                            port_id: "process-heat".to_owned(),
                            form: EnergyForm::ThermalHeat,
                            energy_kwh: 14.0,
                            temperature_c: Some(80.0),
                            remainder_disposition: EnergyPortDisposition::UsefulService,
                        },
                        EnergyOutputPort {
                            port_id: "process-rejection".to_owned(),
                            form: EnergyForm::ThermalHeat,
                            energy_kwh: 1.0,
                            temperature_c: Some(80.0),
                            remainder_disposition: EnergyPortDisposition::Rejected,
                        },
                    ],
                },
            ],
            edges: vec![EnergyPortEdge {
                edge_id: "recovered-heat-to-process".to_owned(),
                from_node_id: "chp".to_owned(),
                from_port_id: "heat-coproduct".to_owned(),
                to_node_id: "process-load".to_owned(),
                to_port_id: "heat-in".to_owned(),
                energy_kwh: 15.0,
            }],
        }
    }

    #[test]
    fn chp_coproduct_is_routed_once_and_global_energy_closes() {
        let assessment = assess_energy_ports(&chp_fixture()).unwrap();
        assert_eq!(assessment.external_input_energy_kwh, 100.0);
        assert_eq!(assessment.routed_energy_kwh, 15.0);
        assert_eq!(assessment.terminal_useful_service_kwh, 49.0);
        assert_eq!(assessment.terminal_rejected_kwh, 51.0);
        assert_eq!(assessment.terminal_unconverted_resource_kwh, 0.0);
        assert_eq!(assessment.terminal_curtailed_kwh, 0.0);
        assert_eq!(assessment.terminal_disposition_energy_kwh, 100.0);
        assert!(assessment.system_balance_residual_kwh.abs() < 1.0e-9);

        let heat = assessment.output_ports.iter()
            .find(|port| port.node_id == "chp" && port.port_id == "heat-coproduct")
            .unwrap();
        assert_eq!(heat.energy_kwh, 45.0);
        assert_eq!(heat.routed_energy_kwh, 15.0);
        assert_eq!(heat.terminal_remainder_kwh, 30.0);
        assert_eq!(heat.remainder_disposition, EnergyPortDisposition::Rejected);
    }

    #[test]
    fn output_cannot_be_double_counted_or_overrouted() {
        let mut request = chp_fixture();
        request.edges.push(EnergyPortEdge {
            edge_id: "duplicate-heat-claim".to_owned(),
            from_node_id: "chp".to_owned(),
            from_port_id: "heat-coproduct".to_owned(),
            to_node_id: "process-load".to_owned(),
            to_port_id: "heat-in".to_owned(),
            energy_kwh: 31.0,
        });
        let error = assess_energy_ports(&request).unwrap_err();
        assert!(matches!(error, EnergyPortsError::OutputOverdrawn { .. }));
    }

    #[test]
    fn input_must_be_filled_by_external_supply_plus_edges() {
        let mut request = chp_fixture();
        request.edges.clear();
        let error = assess_energy_ports(&request).unwrap_err();
        assert!(matches!(error, EnergyPortsError::InputRoutingMismatch { .. }));
    }

    #[test]
    fn node_first_law_balance_is_mandatory() {
        let mut request = chp_fixture();
        request.nodes[0].outputs[0].energy_kwh = 34.0;
        let error = assess_energy_ports(&request).unwrap_err();
        assert!(matches!(error, EnergyPortsError::NodeEnergyBalanceMismatch { .. }));
    }

    #[test]
    fn edge_must_preserve_energy_form() {
        let mut request = chp_fixture();
        request.nodes[1].inputs[0].form = EnergyForm::Electricity;
        request.nodes[1].inputs[0].temperature_c = None;
        let error = assess_energy_ports(&request).unwrap_err();
        assert!(matches!(error, EnergyPortsError::EnergyFormMismatch { .. }));
    }

    #[test]
    fn thermal_edge_must_preserve_declared_temperature_grade() {
        let mut request = chp_fixture();
        request.nodes[1].inputs[0].temperature_c = Some(70.0);
        let error = assess_energy_ports(&request).unwrap_err();
        assert!(matches!(error, EnergyPortsError::TemperatureGradeMismatch { .. }));
    }

    #[test]
    fn cycles_are_rejected_and_must_use_stateful_storage_semantics() {
        let make_node = |node_id: &str| EnergyPortNode {
            node_id: node_id.to_owned(),
            boundary_description: format!("cycle fixture node {node_id}"),
            evidence_class: EvidenceClass::AnalystAssumption,
            evidence_ref: None,
            evidence_as_of: None,
            inputs: vec![EnergyInputPort {
                port_id: "in".to_owned(),
                form: EnergyForm::Electricity,
                energy_kwh: 1.0,
                external_energy_kwh: 0.0,
                external_source_id: None,
                temperature_c: None,
            }],
            outputs: vec![EnergyOutputPort {
                port_id: "out".to_owned(),
                form: EnergyForm::Electricity,
                energy_kwh: 1.0,
                temperature_c: None,
                remainder_disposition: EnergyPortDisposition::Rejected,
            }],
        };
        let request = EnergyPortsRequest {
            schema_version: "energy-ports-v0.1".to_owned(),
            scenario_id: "synthetic-cycle".to_owned(),
            synthetic: true,
            nodes: vec![make_node("a"), make_node("b")],
            edges: vec![
                EnergyPortEdge {
                    edge_id: "a-to-b".to_owned(),
                    from_node_id: "a".to_owned(),
                    from_port_id: "out".to_owned(),
                    to_node_id: "b".to_owned(),
                    to_port_id: "in".to_owned(),
                    energy_kwh: 1.0,
                },
                EnergyPortEdge {
                    edge_id: "b-to-a".to_owned(),
                    from_node_id: "b".to_owned(),
                    from_port_id: "out".to_owned(),
                    to_node_id: "a".to_owned(),
                    to_port_id: "in".to_owned(),
                    energy_kwh: 1.0,
                },
            ],
        };
        assert_eq!(
            assess_energy_ports(&request),
            Err(EnergyPortsError::GraphCycleDetected)
        );
    }

    #[test]
    fn external_source_identity_cannot_be_counted_twice() {
        let mut request = chp_fixture();
        request.nodes[1].inputs[0].energy_kwh = 16.0;
        request.nodes[1].inputs[0].external_energy_kwh = 1.0;
        request.nodes[1].inputs[0].external_source_id = Some("fuel-batch-001".to_owned());
        assert!(matches!(
            assess_energy_ports(&request),
            Err(EnergyPortsError::DuplicateExternalSourceId { .. })
        ));
    }

    #[test]
    fn positive_external_supply_requires_an_identity() {
        let mut request = chp_fixture();
        request.nodes[0].inputs[0].external_source_id = None;
        assert!(matches!(
            assess_energy_ports(&request),
            Err(EnergyPortsError::MissingExternalSourceId { .. })
        ));
    }

    #[test]
    fn non_assumption_nodes_need_evidence_references() {
        let mut request = chp_fixture();
        request.nodes[0].evidence_class = EvidenceClass::PublishedModel;
        assert!(matches!(
            assess_energy_ports(&request),
            Err(EnergyPortsError::MissingEvidenceReference { .. })
        ));
    }

    #[test]
    fn thermal_ports_require_valid_temperature_metadata() {
        let mut request = chp_fixture();
        request.nodes[0].outputs[1].temperature_c = None;
        assert!(matches!(
            assess_energy_ports(&request),
            Err(EnergyPortsError::MissingThermalTemperature { .. })
        ));
        let mut request = chp_fixture();
        request.nodes[0].outputs[1].temperature_c = Some(-273.15);
        assert!(matches!(
            assess_energy_ports(&request),
            Err(EnergyPortsError::InvalidAbsoluteTemperature { .. })
        ));
    }
}
