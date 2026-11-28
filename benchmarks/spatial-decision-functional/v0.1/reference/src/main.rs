use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    process::ExitCode,
};

const EPSILON: f64 = 1e-9;

#[derive(Clone, Debug)]
struct Cell {
    id: String,
    measure: f64,
    observed: bool,
    lower: Option<f64>,
    upper: Option<f64>,
}

fn required<'a>(value: &'a Value, key: &str, path: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("{path}.{key} is required"))
}

fn as_object<'a>(value: &'a Value, path: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{path} must be an object"))
}

fn as_array<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{path} must be an array"))
}

fn as_string(value: &Value, path: &str) -> Result<String, String> {
    value
        .as_str()
        .filter(|item| !item.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("{path} must be a non-empty string"))
}

fn as_number(value: &Value, path: &str) -> Result<f64, String> {
    let number = value
        .as_f64()
        .ok_or_else(|| format!("{path} must be a finite number"))?;
    if !number.is_finite() {
        return Err(format!("{path} must be a finite number"));
    }
    Ok(number)
}

fn optional_number(value: &Value, key: &str, path: &str) -> Result<Option<f64>, String> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(item) => Ok(Some(as_number(item, &format!("{path}.{key}"))?)),
    }
}

fn require_unique(values: impl IntoIterator<Item = String>, path: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value.clone()) {
            return Err(format!("{path} contains duplicate ID {value}"));
        }
    }
    Ok(())
}

fn validate_digest(value: &str, path: &str) -> Result<(), String> {
    if value.len() != 64 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(format!("{path} must be a 64-character hexadecimal digest"));
    }
    Ok(())
}

fn parse_cells(support: &Value) -> Result<(Vec<Cell>, f64, f64), String> {
    let path = "case.support";
    let target_measure = as_number(
        required(support, "targetMeasure", path)?,
        "case.support.targetMeasure",
    )?;
    if target_measure < 0.0 {
        return Err("case.support.targetMeasure must be non-negative".into());
    }

    let raw_cells = as_array(required(support, "cells", path)?, "case.support.cells")?;
    let mut cells = Vec::with_capacity(raw_cells.len());
    let mut total_measure = 0.0;
    let mut known_measure = 0.0;
    let mut ids = Vec::with_capacity(raw_cells.len());

    for (index, raw) in raw_cells.iter().enumerate() {
        let cell_path = format!("case.support.cells[{index}]");
        let id = as_string(required(raw, "id", &cell_path)?, &format!("{cell_path}.id"))?;
        ids.push(id.clone());
        let measure = as_number(
            required(raw, "measure", &cell_path)?,
            &format!("{cell_path}.measure"),
        )?;
        if measure <= 0.0 {
            return Err(format!("{cell_path}.measure must be positive"));
        }

        let status = as_string(
            required(raw, "status", &cell_path)?,
            &format!("{cell_path}.status"),
        )?;
        let (observed, lower, upper) = match status.as_str() {
            "observed" => (true, None, None),
            "unknown" => {
                let lower = optional_number(raw, "lowerValue", &cell_path)?;
                let upper = optional_number(raw, "upperValue", &cell_path)?;
                if lower.is_some() != upper.is_some() {
                    return Err(format!(
                        "{cell_path} must declare both lowerValue and upperValue or neither"
                    ));
                }
                if let (Some(lo), Some(hi)) = (lower, upper) {
                    if lo > hi {
                        return Err(format!("{cell_path} lowerValue exceeds upperValue"));
                    }
                }
                (false, lower, upper)
            }
            other => {
                return Err(format!(
                    "{cell_path}.status has unsupported value {other}; expected observed or unknown"
                ));
            }
        };
        if observed {
            known_measure += measure;
        }
        total_measure += measure;
        cells.push(Cell {
            id,
            measure,
            observed,
            lower,
            upper,
        });
    }

    require_unique(ids, "case.support.cells")?;
    if (total_measure - target_measure).abs() > EPSILON * target_measure.max(1.0) {
        return Err(format!(
            "case.support cell measures sum to {total_measure}, not declared targetMeasure {target_measure}"
        ));
    }

    let known_fraction = if target_measure > 0.0 {
        known_measure / target_measure
    } else {
        0.0
    };
    Ok((cells, target_measure, known_fraction))
}

fn validate_support(support: &Value, kind: &str) -> Result<(), String> {
    let path = "case.support";
    as_string(required(support, "supportId", path)?, "case.support.supportId")?;
    as_string(
        required(support, "supportRevision", path)?,
        "case.support.supportRevision",
    )?;
    let digest = as_string(
        required(support, "geometryDigest", path)?,
        "case.support.geometryDigest",
    )?;
    validate_digest(&digest, "case.support.geometryDigest")?;
    as_string(
        required(support, "coordinateReference", path)?,
        "case.support.coordinateReference",
    )?;
    as_string(
        required(support, "verticalDatum", path)?,
        "case.support.verticalDatum",
    )?;
    as_string(
        required(support, "coordinateEpoch", path)?,
        "case.support.coordinateEpoch",
    )?;
    let measure_kind = as_string(
        required(support, "measureKind", path)?,
        "case.support.measureKind",
    )?;
    let _measure_units = as_string(
        required(support, "measureUnits", path)?,
        "case.support.measureUnits",
    )?;

    if kind == "connectivity" {
        if measure_kind != "graph" {
            return Err("connectivity support.measureKind must be graph".into());
        }
        let target_measure = as_number(
            required(support, "targetMeasure", path)?,
            "case.support.targetMeasure",
        )?;
        let cells = as_array(required(support, "cells", path)?, "case.support.cells")?;
        if target_measure != 0.0 || !cells.is_empty() {
            return Err("connectivity support must have targetMeasure 0 and no cells".into());
        }
    } else {
        let required_kind = match kind {
            "line_integral" => "line",
            "weighted_mean" | "weighted_total" | "thresholded_measure" => {
                if measure_kind == "graph" {
                    return Err(format!("{kind} cannot use graph support"));
                }
                measure_kind.as_str()
            }
            _ => return Err(format!("unsupported functional kind {kind}")),
        };
        if kind == "line_integral" && required_kind != "line" {
            return Err("line_integral requires measureKind line".into());
        }
        let (cells, target_measure, _) = parse_cells(support)?;
        if cells.is_empty() || target_measure <= 0.0 {
            return Err(format!("{kind} requires positive, non-empty support"));
        }
    }
    Ok(())
}

fn parse_weights(worlds: &[Value]) -> Result<Vec<(String, f64)>, String> {
    if worlds.is_empty() {
        return Err("case.worlds must not be empty".into());
    }
    let mut result = Vec::with_capacity(worlds.len());
    let mut ids = Vec::with_capacity(worlds.len());
    let mut total = 0.0;

    for (index, world) in worlds.iter().enumerate() {
        let path = format!("case.worlds[{index}]");
        let id = as_string(required(world, "worldId", &path)?, &format!("{path}.worldId"))?;
        let weight = as_number(
            required(world, "weight", &path)?,
            &format!("{path}.weight"),
        )?;
        if weight <= 0.0 {
            return Err(format!("{path}.weight must be positive"));
        }
        ids.push(id.clone());
        total += weight;
        result.push((id, weight));
    }
    require_unique(ids, "case.worlds")?;
    if (total - 1.0).abs() > EPSILON {
        return Err(format!("case world weights must sum to 1.0, observed {total}"));
    }
    Ok(result)
}

fn parse_world_values(
    world: &Value,
    cells: &[Cell],
    world_index: usize,
) -> Result<BTreeMap<String, f64>, String> {
    let path = format!("case.worlds[{world_index}]");
    let raw_values = as_object(
        required(world, "values", &path)?,
        &format!("{path}.values"),
    )?;
    let cells_by_id: BTreeMap<String, &Cell> = cells
        .iter()
        .map(|cell| (cell.id.clone(), cell))
        .collect();

    for key in raw_values.keys() {
        let cell = cells_by_id
            .get(key.as_str())
            .ok_or_else(|| format!("{path}.values contains unknown cell ID {key}"))?;
        if !cell.observed {
            return Err(format!("{path}.values must not fill unknown support cell {key}"));
        }
    }

    let mut values = BTreeMap::new();
    for cell in cells.iter().filter(|cell| cell.observed) {
        let raw = raw_values
            .get(&cell.id)
            .ok_or_else(|| format!("{path}.values is missing observed cell {}", cell.id))?;
        values.insert(
            cell.id.clone(),
            as_number(raw, &format!("{path}.values.{}", cell.id))?,
        );
    }
    Ok(values)
}

fn validate_functional_units(functional: &Value, support: &Value, kind: &str) -> Result<(), String> {
    let path = "case.functional";
    let output_units = as_string(
        required(functional, "outputUnits", path)?,
        "case.functional.outputUnits",
    )?;
    let measure_units = as_string(
        required(support, "measureUnits", "case.support")?,
        "case.support.measureUnits",
    )?;

    match kind {
        "weighted_mean" => {
            let value_units = as_string(
                required(functional, "valueUnits", path)?,
                "case.functional.valueUnits",
            )?;
            if output_units != value_units {
                return Err("weighted_mean outputUnits must equal valueUnits".into());
            }
        }
        "weighted_total" | "line_integral" => {
            as_string(
                required(functional, "valueUnits", path)?,
                "case.functional.valueUnits",
            )?;
        }
        "thresholded_measure" => {
            as_string(
                required(functional, "valueUnits", path)?,
                "case.functional.valueUnits",
            )?;
            let threshold_units = as_string(
                required(functional, "thresholdUnits", path)?,
                "case.functional.thresholdUnits",
            )?;
            if threshold_units != as_string(
                required(functional, "valueUnits", path)?,
                "case.functional.valueUnits",
            )? {
                return Err("thresholdUnits must equal valueUnits".into());
            }
            if output_units != measure_units {
                return Err("thresholded_measure outputUnits must equal support.measureUnits".into());
            }
        }
        "connectivity" => {
            if output_units != "probability" {
                return Err("connectivity outputUnits must be probability".into());
            }
        }
        _ => return Err(format!("unsupported functional kind {kind}")),
    }
    Ok(())
}

fn evaluate_numeric_case(case: &Value, kind: &str) -> Result<Value, String> {
    let support = required(case, "support", "case")?;
    validate_support(support, kind)?;
    validate_functional_units(required(case, "functional", "case")?, support, kind)?;
    let (cells, target_measure, known_fraction) = parse_cells(support)?;
    let worlds = as_array(required(case, "worlds", "case")?, "case.worlds")?;
    let weights = parse_weights(worlds)?;
    let functional = required(case, "functional", "case")?;

    let threshold = if kind == "thresholded_measure" {
        Some(as_number(
            required(functional, "threshold", "case.functional")?,
            "case.functional.threshold",
        )?)
    } else {
        None
    };

    let mut world_results = Vec::with_capacity(worlds.len());
    let mut exact_values = Vec::with_capacity(worlds.len());
    let mut bounded_lowers = Vec::with_capacity(worlds.len());
    let mut bounded_uppers = Vec::with_capacity(worlds.len());
    let mut all_exact = true;
    let mut all_bounded = true;

    for (index, world) in worlds.iter().enumerate() {
        let values = parse_world_values(world, &cells, index)?;
        let world_id = weights[index].0.clone();
        let weight = weights[index].1;
        let mut known_contribution = 0.0;
        let mut lower = 0.0;
        let mut upper = 0.0;
        let mut bounds_available = true;

        for cell in &cells {
            if cell.observed {
                let value = *values
                    .get(&cell.id)
                    .ok_or_else(|| format!("missing value for observed cell {}", cell.id))?;
                let contribution = match threshold {
                    Some(limit) => {
                        if value > limit {
                            cell.measure
                        } else {
                            0.0
                        }
                    }
                    None => value * cell.measure,
                };
                known_contribution += contribution;
                lower += contribution;
                upper += contribution;
            } else if let Some(limit) = threshold {
                match (cell.lower, cell.upper) {
                    (Some(lo), Some(hi)) => {
                        if lo > limit {
                            lower += cell.measure;
                            upper += cell.measure;
                        } else if hi > limit {
                            upper += cell.measure;
                        }
                    }
                    _ => bounds_available = false,
                }
            } else {
                match (cell.lower, cell.upper) {
                    (Some(lo), Some(hi)) => {
                        lower += lo * cell.measure;
                        upper += hi * cell.measure;
                    }
                    _ => bounds_available = false,
                }
            }
        }

        if kind == "weighted_mean" {
            lower /= target_measure;
            upper /= target_measure;
            known_contribution /= target_measure;
        }

        if bounds_available {
            if (upper - lower).abs() <= EPSILON * upper.abs().max(lower.abs()).max(1.0) {
                exact_values.push((lower, weight));
                bounded_lowers.push((lower, weight));
                bounded_uppers.push((upper, weight));
                world_results.push(json!({
                    "worldId": world_id,
                    "weight": weight,
                    "status": "EXACT",
                    "value": lower
                }));
            } else {
                all_exact = false;
                bounded_lowers.push((lower, weight));
                bounded_uppers.push((upper, weight));
                world_results.push(json!({
                    "worldId": world_id,
                    "weight": weight,
                    "status": "BOUNDED_UNKNOWN",
                    "lowerBound": lower,
                    "upperBound": upper,
                    "knownContribution": known_contribution,
                    "knownSupportFraction": known_fraction
                }));
            }
        } else {
            all_exact = false;
            all_bounded = false;
            world_results.push(json!({
                "worldId": world_id,
                "weight": weight,
                "status": "UNKNOWN",
                "knownContribution": known_contribution,
                "knownSupportFraction": known_fraction
            }));
        }
    }

    let case_id = as_string(required(case, "caseId", "case")?, "case.caseId")?;
    let support_id = as_string(
        required(support, "supportId", "case.support")?,
        "case.support.supportId",
    )?;

    let (status, summary) = if all_exact {
        let mean = weighted_mean(&exact_values);
        let variance = exact_values
            .iter()
            .map(|(value, weight)| weight * (value - mean).powi(2))
            .sum::<f64>();
        let mut summary = json!({
            "expectedValue": mean,
            "varianceAcrossSyntheticWorlds": variance,
            "quantile05": weighted_quantile(&exact_values, 0.05),
            "quantile50": weighted_quantile(&exact_values, 0.50),
            "quantile95": weighted_quantile(&exact_values, 0.95)
        });
        if let Some(decision_threshold) = optional_number(functional, "decisionThreshold", "case.functional")? {
            let probability = exact_values
                .iter()
                .filter(|(value, _)| *value > decision_threshold)
                .map(|(_, weight)| *weight)
                .sum::<f64>();
            summary["probabilityAboveDecisionThreshold"] = json!(probability);
            summary["decisionThreshold"] = json!(decision_threshold);
        }
        ("QUALIFIED", summary)
    } else if all_bounded {
        let lower_mean = weighted_mean(&bounded_lowers);
        let upper_mean = weighted_mean(&bounded_uppers);
        (
            "BOUNDED_UNKNOWN",
            json!({
                "weightedMeanLowerBound": lower_mean,
                "weightedMeanUpperBound": upper_mean,
                "minimumWorldLowerBound": bounded_lowers.iter().map(|(value, _)| *value).fold(f64::INFINITY, f64::min),
                "maximumWorldUpperBound": bounded_uppers.iter().map(|(value, _)| *value).fold(f64::NEG_INFINITY, f64::max)
            }),
        )
    } else {
        ("UNKNOWN", json!({}))
    };

    Ok(json!({
        "caseId": case_id,
        "functionalKind": kind,
        "supportId": support_id,
        "status": status,
        "targetSupportMeasure": target_measure,
        "knownSupportFraction": known_fraction,
        "worldResults": world_results,
        "summary": summary
    }))
}

fn weighted_mean(values: &[(f64, f64)]) -> f64 {
    values.iter().map(|(value, weight)| value * weight).sum()
}

fn weighted_quantile(values: &[(f64, f64)], quantile: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut cumulative = 0.0;
    for (value, weight) in &sorted {
        cumulative += weight;
        if cumulative + EPSILON >= quantile {
            return *value;
        }
    }
    sorted.last().map(|(value, _)| *value).unwrap_or(f64::NAN)
}

fn graph_reachable(
    nodes: &BTreeSet<String>,
    edges: &BTreeMap<String, (String, String)>,
    selected_edge_ids: &BTreeSet<String>,
    directed: bool,
    source: &str,
    target: &str,
) -> bool {
    if source == target {
        return true;
    }
    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for edge_id in selected_edge_ids {
        if let Some((from, to)) = edges.get(edge_id) {
            adjacency
                .entry(from.clone())
                .or_default()
                .push(to.clone());
            if !directed {
                adjacency
                    .entry(to.clone())
                    .or_default()
                    .push(from.clone());
            }
        }
    }

    let mut visited = BTreeSet::new();
    let mut pending = vec![source.to_owned()];
    while let Some(node) = pending.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }
        if node == target {
            return true;
        }
        if let Some(neighbours) = adjacency.get(&node) {
            for neighbour in neighbours {
                if nodes.contains(neighbour) && !visited.contains(neighbour) {
                    pending.push(neighbour.clone());
                }
            }
        }
    }
    false
}

fn evaluate_connectivity_case(case: &Value) -> Result<Value, String> {
    let support = required(case, "support", "case")?;
    validate_support(support, "connectivity")?;
    let functional = required(case, "functional", "case")?;
    validate_functional_units(functional, support, "connectivity")?;
    let graph = required(case, "graph", "case")?;
    let graph_path = "case.graph";

    let raw_nodes = as_array(required(graph, "nodes", graph_path)?, "case.graph.nodes")?;
    let nodes_vec = raw_nodes
        .iter()
        .enumerate()
        .map(|(index, value)| as_string(value, &format!("case.graph.nodes[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    require_unique(nodes_vec.clone(), "case.graph.nodes")?;
    let nodes: BTreeSet<String> = nodes_vec.into_iter().collect();
    if nodes.is_empty() {
        return Err("case.graph.nodes must not be empty".into());
    }

    let raw_edges = as_array(required(graph, "edges", graph_path)?, "case.graph.edges")?;
    let mut edges = BTreeMap::new();
    for (index, edge) in raw_edges.iter().enumerate() {
        let path = format!("case.graph.edges[{index}]");
        let id = as_string(required(edge, "id", &path)?, &format!("{path}.id"))?;
        let from = as_string(required(edge, "from", &path)?, &format!("{path}.from"))?;
        let to = as_string(required(edge, "to", &path)?, &format!("{path}.to"))?;
        if !nodes.contains(&from) || !nodes.contains(&to) {
            return Err(format!("{path} references an unknown node"));
        }
        if edges.insert(id.clone(), (from, to)).is_some() {
            return Err(format!("case.graph.edges contains duplicate ID {id}"));
        }
    }

    let directed = required(graph, "directed", graph_path)?
        .as_bool()
        .ok_or_else(|| "case.graph.directed must be a boolean".to_string())?;
    let source = as_string(
        required(graph, "sourceNode", graph_path)?,
        "case.graph.sourceNode",
    )?;
    let target = as_string(
        required(graph, "targetNode", graph_path)?,
        "case.graph.targetNode",
    )?;
    if !nodes.contains(&source) || !nodes.contains(&target) {
        return Err("case.graph sourceNode/targetNode must exist in nodes".into());
    }

    let worlds = as_array(required(case, "worlds", "case")?, "case.worlds")?;
    let weights = parse_weights(worlds)?;
    let mut results = Vec::with_capacity(worlds.len());
    let mut probability_lower = 0.0;
    let mut probability_upper = 0.0;
    let mut all_exact = true;

    for (index, world) in worlds.iter().enumerate() {
        let path = format!("case.worlds[{index}]");
        let active = parse_edge_ids(world, "activeEdgeIds", &path, &edges)?;
        let unknown = parse_edge_ids(world, "unknownEdgeIds", &path, &edges)?;
        if active.iter().any(|edge_id| unknown.contains(edge_id)) {
            return Err(format!("{path} activeEdgeIds overlaps unknownEdgeIds"));
        }

        let lower = graph_reachable(&nodes, &edges, &active, directed, &source, &target);
        let mut optimistic = active.clone();
        optimistic.extend(unknown.iter().cloned());
        let upper = graph_reachable(&nodes, &edges, &optimistic, directed, &source, &target);
        if lower != upper {
            all_exact = false;
        }

        let (world_id, weight) = &weights[index];
        if lower {
            probability_lower += weight;
        }
        if upper {
            probability_upper += weight;
        }
        results.push(json!({
            "worldId": world_id,
            "weight": weight,
            "connectivityLowerBound": lower,
            "connectivityUpperBound": upper
        }));
    }

    let mut summary = json!({
        "probabilityConnectedLowerBound": probability_lower,
        "probabilityConnectedUpperBound": probability_upper
    });
    if all_exact {
        summary["probabilityConnected"] = json!(probability_lower);
    }

    Ok(json!({
        "caseId": as_string(required(case, "caseId", "case")?, "case.caseId")?,
        "functionalKind": "connectivity",
        "supportId": as_string(required(support, "supportId", "case.support")?, "case.support.supportId")?,
        "status": if all_exact { "QUALIFIED" } else { "BOUNDED_UNKNOWN" },
        "sourceNode": source,
        "targetNode": target,
        "worldResults": results,
        "summary": summary
    }))
}

fn parse_edge_ids(
    world: &Value,
    key: &str,
    path: &str,
    edges: &BTreeMap<String, (String, String)>,
) -> Result<BTreeSet<String>, String> {
    let values = as_array(
        required(world, key, path)?,
        &format!("{path}.{key}"),
    )?;
    let ids = values
        .iter()
        .enumerate()
        .map(|(index, value)| as_string(value, &format!("{path}.{key}[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    require_unique(ids.clone(), &format!("{path}.{key}"))?;
    for id in &ids {
        if !edges.contains_key(id) {
            return Err(format!("{path}.{key} references unknown edge {id}"));
        }
    }
    Ok(ids.into_iter().collect())
}

fn evaluate_case(case: &Value) -> Result<Value, String> {
    let functional = required(case, "functional", "case")?;
    let kind = as_string(
        required(functional, "kind", "case.functional")?,
        "case.functional.kind",
    )?;
    match kind.as_str() {
        "connectivity" => evaluate_connectivity_case(case),
        "weighted_mean" | "weighted_total" | "line_integral" | "thresholded_measure" => {
            evaluate_numeric_case(case, &kind)
        }
        other => Err(format!("unsupported functional kind {other}")),
    }
}

fn evaluate_document(document: &Value) -> Result<Value, String> {
    as_object(document, "document")?;
    let schema_version = as_string(
        required(document, "schemaVersion", "document")?,
        "document.schemaVersion",
    )?;
    if schema_version != "0.1" {
        return Err(format!("unsupported schemaVersion {schema_version}"));
    }
    let fixture_status = as_string(
        required(document, "fixtureStatus", "document")?,
        "document.fixtureStatus",
    )?;
    if fixture_status != "synthetic" {
        return Err("this exact reference only accepts synthetic fixtures".into());
    }
    let weight_semantics = as_string(
        required(document, "worldWeightSemantics", "document")?,
        "document.worldWeightSemantics",
    )?;
    if weight_semantics != "exact_probability" {
        return Err("v0.1 reference requires explicit exact_probability world weights".into());
    }

    let cases = as_array(required(document, "cases", "document")?, "document.cases")?;
    if cases.is_empty() {
        return Err("document.cases must not be empty".into());
    }
    let mut case_ids = Vec::with_capacity(cases.len());
    let mut results = Vec::with_capacity(cases.len());
    for (index, case) in cases.iter().enumerate() {
        let id = as_string(
            required(case, "caseId", &format!("document.cases[{index}]"))?,
            &format!("document.cases[{index}].caseId"),
        )?;
        case_ids.push(id);
        results.push(evaluate_case(case)?);
    }
    require_unique(case_ids, "document.cases")?;

    Ok(json!({
        "schemaVersion": "0.1",
        "fixtureStatus": "synthetic",
        "worldWeightSemantics": weight_semantics,
        "caseCount": results.len(),
        "results": results
    }))
}

fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| "usage: spatial-decision-functional <fixture.json>".to_string())?;
    let input = fs::read_to_string(&path)
        .map_err(|error| format!("could not read {path}: {error}"))?;
    let document: Value = serde_json::from_str(&input)
        .map_err(|error| format!("invalid JSON in {path}: {error}"))?;
    let report = evaluate_document(&document)?;
    let rendered = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("could not serialize report: {error}"))?;
    println!("{rendered}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("spatial-decision-functional: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../fixtures/spatial-functional-cases-v0.1.json");

    fn fixture() -> Value {
        serde_json::from_str(FIXTURE).expect("fixture JSON must parse")
    }

    fn case_result<'a>(report: &'a Value, case_id: &str) -> &'a Value {
        report["results"]
            .as_array()
            .expect("results must be an array")
            .iter()
            .find(|result| result["caseId"] == case_id)
            .expect("expected case result")
    }

    #[test]
    fn weighted_mean_uses_full_target_support_and_reports_bounds() {
        let report = evaluate_document(&fixture()).expect("fixture must evaluate");
        let result = case_result(&report, "mean-with-unknown-support");
        assert_eq!(result["status"], "BOUNDED_UNKNOWN");
        assert!((result["knownSupportFraction"].as_f64().unwrap() - 0.5).abs() < EPSILON);
        assert!((result["summary"]["weightedMeanLowerBound"].as_f64().unwrap() - 1.5).abs() < EPSILON);
        assert!((result["summary"]["weightedMeanUpperBound"].as_f64().unwrap() - 6.5).abs() < EPSILON);
    }

    #[test]
    fn thresholded_extent_keeps_unknown_area_as_an_interval() {
        let report = evaluate_document(&fixture()).expect("fixture must evaluate");
        let result = case_result(&report, "thresholded-area-with-unknown-cell");
        assert_eq!(result["status"], "BOUNDED_UNKNOWN");
        assert!((result["summary"]["weightedMeanLowerBound"].as_f64().unwrap() - 2.5).abs() < EPSILON);
        assert!((result["summary"]["weightedMeanUpperBound"].as_f64().unwrap() - 3.5).abs() < EPSILON);
    }

    #[test]
    fn line_integral_is_weighted_by_declared_length() {
        let report = evaluate_document(&fixture()).expect("fixture must evaluate");
        let result = case_result(&report, "corridor-line-integral");
        assert_eq!(result["status"], "QUALIFIED");
        assert!((result["summary"]["expectedValue"].as_f64().unwrap() - 5.0).abs() < EPSILON);
    }

    #[test]
    fn weighted_total_multiplies_density_by_volume_measure() {
        let report = evaluate_document(&fixture()).expect("fixture must evaluate");
        let result = case_result(&report, "weighted-stock-total");
        assert_eq!(result["status"], "QUALIFIED");
        assert!((result["summary"]["expectedValue"].as_f64().unwrap() - 8.0).abs() < EPSILON);
    }

    #[test]
    fn unknown_support_without_bounds_never_becomes_zero() {
        let mut document = fixture();
        document["cases"][0]["support"]["cells"][1].as_object_mut().unwrap().remove("lowerValue");
        document["cases"][0]["support"]["cells"][1].as_object_mut().unwrap().remove("upperValue");
        let report = evaluate_document(&document).expect("unknown support remains a valid input");
        let result = case_result(&report, "mean-with-unknown-support");
        assert_eq!(result["status"], "UNKNOWN");
        assert!(result["summary"].get("expectedValue").is_none());
        assert!(result["worldResults"][0].get("knownContribution").is_some());
    }

    #[test]
    fn connectivity_probability_uses_joint_worlds_not_marginal_edge_products() {
        let report = evaluate_document(&fixture()).expect("fixture must evaluate");
        let result = case_result(&report, "joint-connectivity-worlds");
        assert_eq!(result["status"], "QUALIFIED");
        assert!((result["summary"]["probabilityConnected"].as_f64().unwrap() - 0.4).abs() < EPSILON);
    }

    #[test]
    fn unknown_network_edge_produces_lower_and_upper_connectivity() {
        let report = evaluate_document(&fixture()).expect("fixture must evaluate");
        let result = case_result(&report, "connectivity-with-unknown-edge");
        assert_eq!(result["status"], "BOUNDED_UNKNOWN");
        assert!((result["summary"]["probabilityConnectedLowerBound"].as_f64().unwrap() - 0.5).abs() < EPSILON);
        assert!((result["summary"]["probabilityConnectedUpperBound"].as_f64().unwrap() - 1.0).abs() < EPSILON);
        assert!(result["summary"].get("probabilityConnected").is_none());
    }

    #[test]
    fn rejects_world_weights_that_do_not_normalize() {
        let mut document = fixture();
        document["cases"][0]["worlds"][0]["weight"] = json!(0.1);
        assert!(evaluate_document(&document).is_err());
    }

    #[test]
    fn rejects_target_measure_mismatch() {
        let mut document = fixture();
        document["cases"][0]["support"]["targetMeasure"] = json!(99.0);
        assert!(evaluate_document(&document).is_err());
    }

    #[test]
    fn rejects_unknown_cell_values_smuggled_into_worlds() {
        let mut document = fixture();
        document["cases"][1]["worlds"][0]["values"]["cell-c"] = json!(3.0);
        assert!(evaluate_document(&document).is_err());
    }
}
