// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Independent importer for model-generated Humanity Atlas technology trees.
//!
//! The generic run envelope records provenance and claims; this module validates
//! the embedded model-specific graph against those claims and the exact source
//! artifact bytes. It does not load or execute the simulator, retrieve artifact
//! URIs, authenticate publishers, calibrate probabilities, or establish truth.

use super::{
    OutputClassification, OutputStatistic, OutputTimeBasis, RunStatus, ScenarioRunEnvelope,
};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;

const TREE_SCHEMA_VERSION: &str = "mycelix-multiworld-sim.tech-tree.v1";
const RUN_MANIFEST_SCHEMA_VERSION: &str = "humanity-atlas.run-manifest.v1";
const EXPECTED_CLASSIFICATION: &str = "simulated_scenario";
const EXPECTED_YEAR_ZERO: &str = "relative_to_scenario_start";
const SIMULATOR_SOURCE_PREFIX: &str = "mycelix-multiworld-sim/";
const EXPECTED_ADAPTER: &str = "mycelix-multiworld-sim-humanity-atlas-adapter";
const EXPECTED_JSON_POINTER: &str = "/technology_tree";
const SECTOR_COUNT: usize = 8;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Fully checked graph imported from the captured simulator artifact.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioTechnologyTree {
    pub schema_version: String,
    pub classification: String,
    pub source_model: String,
    /// Digest of model definitions; deliberately excludes run-specific outcomes.
    pub definition_sha256: String,
    pub definition_source: String,
    pub ticks_per_year: u32,
    pub year_zero_semantics: String,
    pub seed: u64,
    pub horizon_ticks: u32,
    pub horizon_relative_years: u32,
    /// Whether the simulator reported civilization survival at the run horizon.
    pub survived: bool,
    pub source_revision: Option<String>,
    /// SHA-256 of the exact complete simulator-output artifact bytes.
    pub source_artifact_sha256: String,
    pub milestones: Vec<ScenarioTechnologyMilestone>,
    pub dependencies: Vec<ScenarioTechnologyDependency>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioTechnologyMilestone {
    pub id: String,
    pub name: String,
    pub earliest_tick: u32,
    pub latest_tick: u32,
    pub earliest_relative_year: f64,
    pub latest_relative_year: f64,
    /// Conditional model parameter, not an empirically calibrated forecast.
    pub base_probability_per_tick: f64,
    pub sector_prerequisites: Vec<ScenarioSectorPrerequisite>,
    pub prerequisite_milestone_ids: Vec<String>,
    pub effects: ScenarioTechnologyEffects,
    pub achieved: bool,
    pub achieved_tick: Option<u32>,
    pub achieved_relative_year: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioSectorPrerequisite {
    pub sector_index: usize,
    pub minimum_level: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioTechnologyEffects {
    pub tech_level_boost: Vec<(usize, f64)>,
    pub power_multiplier: f64,
    pub propulsion_unlock: bool,
    pub resource_efficiency: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioTechnologyDependency {
    pub prerequisite_id: String,
    pub dependent_id: String,
    pub relation: ScenarioDependencyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioDependencyKind {
    /// This is an explicit prerequisite declared by the source simulator.
    ModelPrerequisite,
}

/// Verify envelope/source bindings and import the embedded technology tree.
///
/// Callers supply the exact artifact bytes they obtained. This function checks
/// those bytes against the envelope's SHA-256 before reading the model payload;
/// it never fetches `artifact_ref` and never treats the digest as authentication.
pub fn import_technology_tree(
    run: &ScenarioRunEnvelope,
    artifact_id: &str,
    source_artifact_bytes: &[u8],
) -> Result<ScenarioTechnologyTree, String> {
    run.validate()?;
    let artifact = run
        .artifacts
        .iter()
        .find(|item| item.artifact_id == artifact_id)
        .ok_or_else(|| format!("unknown artifact_id: {artifact_id}"))?;
    if artifact.json_pointer.as_deref() != Some(EXPECTED_JSON_POINTER) {
        return Err(format!(
            "artifact {artifact_id} must select the embedded {EXPECTED_JSON_POINTER} payload"
        ));
    }
    if artifact.role != "source_simulation_output" || artifact.media_type != "application/json" {
        return Err(format!("artifact {artifact_id} must be a JSON source_simulation_output"));
    }
    run.verify_artifact_bytes(artifact_id, source_artifact_bytes)?;

    let source_artifact_sha256 = sha256_hex(source_artifact_bytes);
    if run.run_id != format!("urn:sha256:{source_artifact_sha256}") {
        return Err("run_id does not bind the exact source artifact bytes".into());
    }

    let strict_value = parse_json_without_duplicate_keys(source_artifact_bytes)?;
    let source_value = strict_value_to_json(strict_value);
    let tree_value = source_value
        .get("technology_tree")
        .and_then(Value::as_object)
        .ok_or_else(|| "simulator output must include technology_tree object".to_string())?;
    let milestone_values = tree_value
        .get("milestones")
        .and_then(Value::as_array)
        .ok_or_else(|| "technology_tree.milestones must be an array".to_string())?;
    for (index, milestone) in milestone_values.iter().enumerate() {
        if !milestone.as_object().is_some_and(|fields| fields.contains_key("achieved_tick")) {
            return Err(format!("technology_tree.milestones[{index}] is missing required achieved_tick key"));
        }
    }
    let source: SimulationOutputWire = serde_json::from_value(source_value.clone())
        .map_err(|error| format!("invalid simulator output envelope: {error}"))?;

    // source_revision is a required nullable key in the v1 manifest. Option<T>
    // alone would otherwise accept a missing key as None.
    let manifest_value = source_value
        .get("run_manifest")
        .and_then(Value::as_object)
        .ok_or_else(|| "simulator output must include run_manifest object".to_string())?;
    if !manifest_value.contains_key("source_revision") {
        return Err("run manifest is missing required source_revision key".into());
    }

    validate_source_and_run_binding(run, &source)?;
    let tree = source.technology_tree;
    let dependencies = validate_tree(&tree, source.ticks)?;

    let ticks_per_year = tree.ticks_per_year;
    let milestones = tree
        .milestones
        .into_iter()
        .map(|milestone| ScenarioTechnologyMilestone {
            id: milestone.id,
            name: milestone.name,
            earliest_tick: milestone.earliest_tick,
            latest_tick: milestone.latest_tick,
            earliest_relative_year: milestone.earliest_tick as f64 / ticks_per_year as f64,
            latest_relative_year: milestone.latest_tick as f64 / ticks_per_year as f64,
            base_probability_per_tick: milestone.base_probability_per_tick,
            sector_prerequisites: milestone
                .sector_prerequisites
                .into_iter()
                .map(|item| ScenarioSectorPrerequisite {
                    sector_index: item.sector_index,
                    minimum_level: item.minimum_level,
                })
                .collect(),
            prerequisite_milestone_ids: milestone.prerequisite_milestone_ids,
            effects: ScenarioTechnologyEffects {
                tech_level_boost: milestone.effects.tech_level_boost,
                power_multiplier: milestone.effects.power_multiplier,
                propulsion_unlock: milestone.effects.propulsion_unlock,
                resource_efficiency: milestone.effects.resource_efficiency,
            },
            achieved: milestone.achieved,
            achieved_tick: milestone.achieved_tick,
            achieved_relative_year: milestone
                .achieved_tick
                .map(|tick| tick as f64 / ticks_per_year as f64),
        })
        .collect();

    Ok(ScenarioTechnologyTree {
        schema_version: tree.schema_version,
        classification: tree.classification,
        source_model: tree.source_model,
        definition_sha256: tree.definition_sha256,
        definition_source: tree.definition_source,
        ticks_per_year,
        year_zero_semantics: tree.year_zero_semantics,
        seed: source.seed,
        horizon_ticks: source.ticks,
        horizon_relative_years: source.years,
        survived: source.survived,
        source_revision: source.run_manifest.source_revision,
        source_artifact_sha256,
        milestones,
        dependencies,
    })
}

fn validate_source_and_run_binding(
    run: &ScenarioRunEnvelope,
    source: &SimulationOutputWire,
) -> Result<(), String> {
    let tree = &source.technology_tree;
    let manifest = &source.run_manifest;

    if manifest.schema_version != RUN_MANIFEST_SCHEMA_VERSION {
        return Err(format!("unsupported run manifest schema: {}", manifest.schema_version));
    }
    if manifest.source_revision.as_ref().is_some_and(|value| value.trim().is_empty()) {
        return Err("source_revision must be null or non-empty".into());
    }
    if manifest.seed != source.seed || manifest.ticks != source.ticks {
        return Err("run manifest seed/ticks disagree with the enclosing simulator output".into());
    }
    if source.seed != run.seed {
        return Err("generic run seed disagrees with source simulator output".into());
    }
    if run.replicates != 1 {
        return Err("this adapter binds one realized simulator run and requires replicates = 1".into());
    }
    if run.run_status != RunStatus::Completed {
        return Err("technology-tree import requires a completed simulator run".into());
    }
    if run.output_classification != OutputClassification::UnvalidatedSimulation {
        return Err("simulator tree artifact must remain classified unvalidated_simulation".into());
    }
    if run.deterministic.is_some() {
        return Err("the adapter has not independently established replay determinism".into());
    }
    if run.inputs.dataset_sha256.is_some() {
        return Err("the simulator manifest does not establish a standalone dataset digest".into());
    }

    let source_model = &tree.source_model;
    let package_version = source_model
        .strip_prefix(SIMULATOR_SOURCE_PREFIX)
        .filter(|value| {
            !value.trim().is_empty()
                && value.len() <= 120
                && !value.contains('/')
                && !value.chars().any(char::is_whitespace)
        })
        .ok_or_else(|| "source_model must include mycelix-multiworld-sim/<package version>".to_string())?;
    if &run.engine.engine_version != package_version {
        return Err("generic engine_version disagrees with source_model package version".into());
    }
    if run.engine.adapter != EXPECTED_ADAPTER {
        return Err("unexpected Humanity Atlas simulator adapter identity".into());
    }
    if run.engine.model_identity_kind.as_deref() != Some("technology_tree_definition") {
        return Err("generic model identity must be scoped to technology_tree_definition".into());
    }
    if run.engine.model_sha256 != tree.definition_sha256
        || manifest.effective_model_sha256 != tree.definition_sha256
    {
        return Err("technology-tree definition digest disagrees across source and run envelope".into());
    }
    if tree.definition_source != manifest.selected_definition_source {
        return Err("technology-tree definition source disagrees with run manifest".into());
    }
    if run.engine.definition_source.as_deref() != Some(tree.definition_source.as_str())
        || run.inputs.selected_definition_source.as_deref() != Some(tree.definition_source.as_str())
    {
        return Err("generic run definition source disagrees with effective source tree".into());
    }
    if run.engine.source_revision != manifest.source_revision {
        return Err("generic source_revision disagrees with simulator run manifest".into());
    }
    if run.inputs.scenario_identity_kind.as_deref() != Some("canonical_effective_scenario") {
        return Err("generic run must use canonical_effective_scenario identity".into());
    }

    let config = &source.resolved_config;
    let params = &source.resolved_params;
    if config.get("seed").and_then(Value::as_u64) != Some(source.seed) {
        return Err("top-level seed disagrees with resolved_config.seed".into());
    }
    if config.get("total_ticks").and_then(Value::as_u64) != Some(u64::from(source.ticks)) {
        return Err("top-level ticks disagree with resolved_config.total_ticks".into());
    }
    if stable_json_value_sha256(config)? != manifest.resolved_config_sha256
        || run.inputs.resolved_config_sha256.as_deref() != Some(manifest.resolved_config_sha256.as_str())
    {
        return Err("resolved_config digest disagrees across source manifest and generic envelope".into());
    }
    if stable_json_value_sha256(params)? != manifest.resolved_params_sha256
        || run.inputs.resolved_params_sha256.as_deref() != Some(manifest.resolved_params_sha256.as_str())
    {
        return Err("resolved_params digest disagrees across source manifest and generic envelope".into());
    }
    if !is_lowercase_sha256(&manifest.resolved_config_sha256)
        || !is_lowercase_sha256(&manifest.resolved_params_sha256)
    {
        return Err("run manifest input digests must be lowercase SHA-256".into());
    }
    if run.seed > MAX_SAFE_INTEGER || source.seed > MAX_SAFE_INTEGER {
        return Err("seed exceeds exact-safe integer range".into());
    }

    let expected_scenario = serde_json::json!({
        "schema_version": "humanity-atlas.effective-scenario.v1",
        "source_model": source_model,
        "selected_definition_source": tree.definition_source,
        "effective_model_sha256": tree.definition_sha256,
        "resolved_config_sha256": manifest.resolved_config_sha256,
        "resolved_params_sha256": manifest.resolved_params_sha256,
        "ticks_per_year": tree.ticks_per_year,
        "ticks": source.ticks,
    });
    let scenario_sha256 = stable_json_value_sha256(&expected_scenario)?;
    if run.inputs.scenario_sha256 != scenario_sha256
        || run.scenario_id != format!("mycelix-multiworld-sim:scenario:{scenario_sha256}")
    {
        return Err("scenario identity digest/ID is not bound to the declared effective inputs".into());
    }
    if source.years == 0 || source.years > 100_000 {
        return Err("source horizon years must be within 1..=100000".into());
    }
    if source.ticks_per_year_product()? != source.ticks {
        return Err("source horizon ticks must equal years * technology_tree.ticks_per_year".into());
    }

    validate_summary_metrics(run, source)?;
    Ok(())
}

fn validate_summary_metrics(
    run: &ScenarioRunEnvelope,
    source: &SimulationOutputWire,
) -> Result<(), String> {
    if source.final_population > MAX_SAFE_INTEGER
        || source.tech_milestones > MAX_SAFE_INTEGER
    {
        return Err("summary counts exceed exact-safe numeric range".into());
    }
    if !source.final_cvs.is_finite() {
        return Err("source final_cvs must be finite".into());
    }
    if run.outputs.len() != 3 {
        return Err("simulator adapter envelope must contain exactly three source-bound summary metrics".into());
    }
    let expected: [(&str, &str, f64); 3] = [
        ("final_cvs", "CVS", source.final_cvs),
        ("final_population", "persons", source.final_population as f64),
        ("tech_milestones_achieved", "milestones", source.tech_milestones as f64),
    ];
    for (metric_id, unit, value) in expected {
        let mut matching = run.outputs.iter().filter(|row| row.metric_id == metric_id);
        let row = matching.next().ok_or_else(|| format!("missing source summary metric {metric_id}"))?;
        if matching.next().is_some()
            || row.year != source.years as i32
            || row.time_basis != Some(OutputTimeBasis::ScenarioRelativeYear)
            || row.unit != unit
            || row.statistic != OutputStatistic::Value
            || row.sample_count != 1
            || row.value != value
        {
            return Err(format!("summary metric {metric_id} disagrees with source simulator output"));
        }
    }
    Ok(())
}

fn validate_tree(
    tree: &TechnologyTreeWire,
    horizon_ticks: u32,
) -> Result<Vec<ScenarioTechnologyDependency>, String> {
    if tree.schema_version != TREE_SCHEMA_VERSION {
        return Err(format!("unsupported technology tree schema: {}", tree.schema_version));
    }
    if tree.classification != EXPECTED_CLASSIFICATION {
        return Err(format!("technology tree must be classified {EXPECTED_CLASSIFICATION}"));
    }
    if tree.source_model.trim().is_empty() || tree.definition_source.trim().is_empty() {
        return Err("source_model and definition_source must be non-empty".into());
    }
    if !is_lowercase_sha256(&tree.definition_sha256) {
        return Err("definition_sha256 must be lowercase SHA-256".into());
    }
    if tree.ticks_per_year == 0 {
        return Err("ticks_per_year must be positive".into());
    }
    if tree.year_zero_semantics != EXPECTED_YEAR_ZERO {
        return Err("technology-tree times must be relative to scenario start".into());
    }
    if tree.milestones.is_empty() {
        return Err("technology tree must contain at least one milestone".into());
    }

    let mut by_id = HashMap::with_capacity(tree.milestones.len());
    for (index, milestone) in tree.milestones.iter().enumerate() {
        if milestone.id.trim().is_empty() || milestone.name.trim().is_empty() {
            return Err("milestone ID/name must be non-empty".into());
        }
        if milestone.id != technology_milestone_id(&milestone.name)
            || milestone.id == "mycelix-multiworld-sim:tech:"
        {
            return Err(format!("milestone {} has non-canonical name-bound ID", milestone.id));
        }
        if by_id.insert(milestone.id.as_str(), index).is_some() {
            return Err(format!("duplicate milestone ID {}", milestone.id));
        }
        if milestone.earliest_tick > milestone.latest_tick {
            return Err(format!("milestone {} has invalid time window", milestone.id));
        }
        if !milestone.base_probability_per_tick.is_finite()
            || !(0.0..=1.0).contains(&milestone.base_probability_per_tick)
        {
            return Err(format!("milestone {} has invalid per-tick parameter", milestone.id));
        }
        match (milestone.achieved, milestone.achieved_tick) {
            (true, Some(tick)) if tick >= milestone.earliest_tick && tick <= milestone.latest_tick && tick <= horizon_ticks => {}
            (false, None) => {}
            (true, Some(tick)) if tick > horizon_ticks => {
                return Err(format!("milestone {} is marked achieved after the run horizon", milestone.id));
            }
            (true, Some(_)) => return Err(format!("milestone {} achieved outside declared window", milestone.id)),
            (true, None) => return Err(format!("milestone {} achieved without achieved_tick", milestone.id)),
            (false, Some(_)) => return Err(format!("milestone {} has achieved_tick while achieved=false", milestone.id)),
        }

        let mut sector_ids = HashSet::new();
        for item in &milestone.sector_prerequisites {
            if item.sector_index >= SECTOR_COUNT {
                return Err(format!("milestone {} has out-of-range sector prerequisite", milestone.id));
            }
            if !item.minimum_level.is_finite() || item.minimum_level < 0.0 {
                return Err(format!("milestone {} has invalid sector prerequisite level", milestone.id));
            }
            if !sector_ids.insert(item.sector_index) {
                return Err(format!("milestone {} repeats sector prerequisite {}", milestone.id, item.sector_index));
            }
        }
        if !milestone.effects.power_multiplier.is_finite()
            || milestone.effects.power_multiplier < 0.0
            || !milestone.effects.resource_efficiency.is_finite()
            || milestone.effects.resource_efficiency < 0.0
        {
            return Err(format!("milestone {} has invalid effect multiplier", milestone.id));
        }
        let mut boost_sectors = HashSet::new();
        for (sector, boost) in &milestone.effects.tech_level_boost {
            if *sector >= SECTOR_COUNT || !boost.is_finite() {
                return Err(format!("milestone {} has invalid technology boost", milestone.id));
            }
            if !boost_sectors.insert(*sector) {
                return Err(format!("milestone {} repeats technology boost for sector {sector}", milestone.id));
            }
        }

        if milestone.prerequisite_milestone_ids.len() != milestone.prerequisite_milestone_names.len() {
            return Err(format!("milestone {} has mismatched prerequisite ID/name counts", milestone.id));
        }
        for (id, name) in milestone.prerequisite_milestone_ids.iter().zip(&milestone.prerequisite_milestone_names) {
            if id != &technology_milestone_id(name) {
                return Err(format!("milestone {} has prerequisite ID/name mismatch", milestone.id));
            }
        }
    }

    let recomputed = technology_tree_definition_sha256(tree)?;
    if recomputed != tree.definition_sha256 {
        return Err("technology-tree definition SHA-256 mismatch".into());
    }
    // Validate and convert the source-declared graph without coupling graph
    // semantics to any rendering or presentation implementation.
    validate_graph_and_build_dependencies(tree)
}

fn validate_graph_and_build_dependencies(
    tree: &TechnologyTreeWire,
) -> Result<Vec<ScenarioTechnologyDependency>, String> {
    let mut index_by_id = HashMap::with_capacity(tree.milestones.len());
    for (index, milestone) in tree.milestones.iter().enumerate() {
        index_by_id.insert(milestone.id.as_str(), index);
    }
    let mut dependencies = Vec::new();
    let mut indegree = vec![0usize; tree.milestones.len()];
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); tree.milestones.len()];
    for (dependent_index, milestone) in tree.milestones.iter().enumerate() {
        let mut local = HashSet::new();
        for prerequisite in &milestone.prerequisite_milestone_ids {
            if !local.insert(prerequisite.as_str()) {
                return Err(format!("milestone {} repeats prerequisite {prerequisite}", milestone.id));
            }
            let prerequisite_index = *index_by_id.get(prerequisite.as_str())
                .ok_or_else(|| format!("milestone {} references missing prerequisite {prerequisite}", milestone.id))?;
            if prerequisite_index == dependent_index {
                return Err(format!("milestone {} depends on itself", milestone.id));
            }
            dependencies.push(ScenarioTechnologyDependency {
                prerequisite_id: (*prerequisite).clone(),
                dependent_id: milestone.id.clone(),
                relation: ScenarioDependencyKind::ModelPrerequisite,
            });
            adjacency[prerequisite_index].push(dependent_index);
            indegree[dependent_index] += 1;
        }
    }
    let mut ready = VecDeque::new();
    for (index, degree) in indegree.iter().enumerate() {
        if *degree == 0 {
            ready.push_back(index);
        }
    }
    let mut visited = 0usize;
    while let Some(index) = ready.pop_front() {
        visited += 1;
        for dependent in &adjacency[index] {
            indegree[*dependent] -= 1;
            if indegree[*dependent] == 0 {
                ready.push_back(*dependent);
            }
        }
    }
    if visited != tree.milestones.len() {
        return Err("technology milestone prerequisite graph contains a cycle".into());
    }
    Ok(dependencies)
}

fn technology_milestone_id(name: &str) -> String {
    let mut slug = String::new();
    let mut pending_separator = false;
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() {
            if pending_separator && !slug.is_empty() {
                slug.push('-');
            }
            slug.push((byte as char).to_ascii_lowercase());
            pending_separator = false;
        } else {
            pending_separator = true;
        }
    }
    format!("mycelix-multiworld-sim:tech:{slug}")
}

fn technology_tree_definition_sha256(tree: &TechnologyTreeWire) -> Result<String, String> {
    let projection = TechnologyTreeDefinitionDigest {
        digest_version: "mycelix-multiworld-sim.tech-definition.v1",
        schema_version: &tree.schema_version,
        classification: &tree.classification,
        source_model: &tree.source_model,
        definition_source: &tree.definition_source,
        ticks_per_year: tree.ticks_per_year,
        year_zero_semantics: &tree.year_zero_semantics,
        milestones: tree.milestones.iter().map(|milestone| TechnologyMilestoneDefinitionDigest {
            id: &milestone.id,
            name: &milestone.name,
            earliest_tick: milestone.earliest_tick,
            latest_tick: milestone.latest_tick,
            base_probability_per_tick: milestone.base_probability_per_tick,
            sector_prerequisites: &milestone.sector_prerequisites,
            prerequisite_milestone_ids: &milestone.prerequisite_milestone_ids,
            prerequisite_milestone_names: &milestone.prerequisite_milestone_names,
            effects: &milestone.effects,
        }).collect(),
    };
    let bytes = serde_json::to_vec(&projection)
        .map_err(|error| format!("cannot serialize model-definition digest input: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn stable_json_value_sha256(value: &Value) -> Result<String, String> {
    fn sorted(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut entries: Vec<_> = map.into_iter().collect();
                entries.sort_by(|left, right| left.0.cmp(&right.0));
                let mut result = Map::new();
                for (key, child) in entries {
                    result.insert(key, sorted(child));
                }
                Value::Object(result)
            }
            Value::Array(items) => Value::Array(items.into_iter().map(sorted).collect()),
            other => other,
        }
    }
    let bytes = serde_json::to_vec(&sorted(value.clone()))
        .map_err(|error| format!("cannot serialize canonical JSON digest input: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug)]
enum StrictJson {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<StrictJson>),
    Object(BTreeMap<String, StrictJson>),
}

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonVisitor)
    }
}

struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = StrictJson;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where E: de::Error {
        Ok(StrictJson::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where E: de::Error {
        Ok(StrictJson::Null)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where E: de::Error {
        Ok(StrictJson::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where E: de::Error {
        Ok(StrictJson::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where E: de::Error {
        Ok(StrictJson::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where E: de::Error {
        serde_json::Number::from_f64(value)
            .map(StrictJson::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where E: de::Error {
        Ok(StrictJson::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where E: de::Error {
        Ok(StrictJson::String(value))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<StrictJson>()? {
            values.push(value);
        }
        Ok(StrictJson::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de> {
        let mut entries = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, StrictJson>()? {
            if entries.insert(key.clone(), value).is_some() {
                return Err(de::Error::custom(format!("duplicate JSON object key: {key}")));
            }
        }
        Ok(StrictJson::Object(entries))
    }
}

fn parse_json_without_duplicate_keys(bytes: &[u8]) -> Result<StrictJson, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictJson::deserialize(&mut deserializer)
        .map_err(|error| format!("invalid source JSON: {error}"))?;
    deserializer.end().map_err(|error| format!("invalid trailing JSON data: {error}"))?;
    Ok(value)
}

fn strict_json_to_json(value: StrictJson) -> Value {
    match value {
        StrictJson::Null => Value::Null,
        StrictJson::Bool(value) => Value::Bool(value),
        StrictJson::Number(value) => Value::Number(value),
        StrictJson::String(value) => Value::String(value),
        StrictJson::Array(items) => Value::Array(items.into_iter().map(strict_json_to_json).collect()),
        StrictJson::Object(items) => {
            let mut result = Map::new();
            for (key, value) in items {
                result.insert(key, strict_json_to_json(value));
            }
            Value::Object(result)
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SimulationOutputWire {
    seed: u64,
    years: u32,
    ticks: u32,
    survived: bool,
    final_cvs: f64,
    final_population: u64,
    tech_milestones: u64,
    resolved_config: Value,
    resolved_params: Value,
    technology_tree: TechnologyTreeWire,
    run_manifest: SimulationRunManifestWire,
}

impl SimulationOutputWire {
    fn ticks_per_year_product(&self) -> Result<u32, String> {
        self.years.checked_mul(self.technology_tree.ticks_per_year)
            .ok_or_else(|| "year * ticks_per_year overflows u32".to_string())
    }

}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SimulationRunManifestWire {
    schema_version: String,
    selected_definition_source: String,
    effective_model_sha256: String,
    resolved_config_sha256: String,
    resolved_params_sha256: String,
    seed: u64,
    ticks: u32,
    source_revision: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TechnologyTreeWire {
    schema_version: String,
    classification: String,
    source_model: String,
    definition_sha256: String,
    definition_source: String,
    ticks_per_year: u32,
    year_zero_semantics: String,
    milestones: Vec<TechnologyMilestoneWire>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TechnologyMilestoneWire {
    id: String,
    name: String,
    earliest_tick: u32,
    latest_tick: u32,
    base_probability_per_tick: f64,
    sector_prerequisites: Vec<SectorPrerequisiteWire>,
    prerequisite_milestone_ids: Vec<String>,
    prerequisite_milestone_names: Vec<String>,
    effects: TechnologyEffectsWire,
    achieved: bool,
    achieved_tick: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SectorPrerequisiteWire {
    sector_index: usize,
    minimum_level: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TechnologyEffectsWire {
    tech_level_boost: Vec<(usize, f64)>,
    power_multiplier: f64,
    propulsion_unlock: bool,
    resource_efficiency: f64,
}

#[derive(Serialize)]
struct TechnologyTreeDefinitionDigest<'a> {
    digest_version: &'static str,
    schema_version: &'a str,
    classification: &'a str,
    source_model: &'a str,
    definition_source: &'a str,
    ticks_per_year: u32,
    year_zero_semantics: &'a str,
    milestones: Vec<TechnologyMilestoneDefinitionDigest<'a>>,
}

#[derive(Serialize)]
struct TechnologyMilestoneDefinitionDigest<'a> {
    id: &'a str,
    name: &'a str,
    earliest_tick: u32,
    latest_tick: u32,
    base_probability_per_tick: f64,
    sector_prerequisites: &'a [SectorPrerequisiteWire],
    prerequisite_milestone_ids: &'a [String],
    prerequisite_milestone_names: &'a [String],
    effects: &'a TechnologyEffectsWire,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{
        EngineIdentity, MetricOutput, RunArtifactReference, RunInputDigests, ValidationRecord,
        ValidationStatus,
    };
    use serde_json::json;

    const FIXTURE_DEFINITION_SOURCE: &str = "in_code_default_tree";

    fn test_milestone(
        name: &str,
        prerequisites: Vec<(String, String)>,
        sector_boosts: Vec<(usize, f64)>,
    ) -> TechnologyMilestoneWire {
        TechnologyMilestoneWire {
            id: technology_milestone_id(name),
            name: name.into(),
            earliest_tick: 0,
            latest_tick: 120,
            base_probability_per_tick: 0.02,
            sector_prerequisites: vec![SectorPrerequisiteWire { sector_index: 0, minimum_level: 1.2 }],
            prerequisite_milestone_ids: prerequisites.iter().map(|(id, _)| id.clone()).collect(),
            prerequisite_milestone_names: prerequisites.iter().map(|(_, name)| name.clone()).collect(),
            effects: TechnologyEffectsWire {
                tech_level_boost: sector_boosts,
                power_multiplier: 1.0,
                propulsion_unlock: false,
                resource_efficiency: 1.0,
            },
            achieved: false,
            achieved_tick: None,
        }
    }

    fn fixture(
        milestones: Vec<TechnologyMilestoneWire>,
    ) -> (ScenarioRunEnvelope, Vec<u8>) {
        let mut tree = TechnologyTreeWire {
            schema_version: TREE_SCHEMA_VERSION.into(),
            classification: EXPECTED_CLASSIFICATION.into(),
            source_model: "mycelix-multiworld-sim/0.1.0".into(),
            definition_sha256: String::new(),
            definition_source: FIXTURE_DEFINITION_SOURCE.into(),
            ticks_per_year: 12,
            year_zero_semantics: EXPECTED_YEAR_ZERO.into(),
            milestones,
        };
        tree.definition_sha256 = technology_tree_definition_sha256(&tree).expect("tree digest");
        let model_sha = tree.definition_sha256.clone();
        let config = json!({"seed":42, "total_ticks":24, "policy":{"trust_weight":0.5}});
        let params = json!({"spoilage_food":0.03, "memory_half_life_ticks":12.0});
        let config_sha = stable_json_value_sha256(&config).expect("config digest");
        let params_sha = stable_json_value_sha256(&params).expect("param digest");
        let revision = Some("0123456789abcdef".to_string());
        let manifest = SimulationRunManifestWire {
            schema_version: RUN_MANIFEST_SCHEMA_VERSION.into(),
            selected_definition_source: FIXTURE_DEFINITION_SOURCE.into(),
            effective_model_sha256: model_sha.clone(),
            resolved_config_sha256: config_sha.clone(),
            resolved_params_sha256: params_sha.clone(),
            seed: 42,
            ticks: 24,
            source_revision: revision.clone(),
        };
        let source = json!({
            "seed":42,
            "years":2,
            "ticks":24,
            "survived":true,
            "final_cvs":0.73,
            "final_population":1234,
            "tech_milestones":0,
            "resolved_config":config,
            "resolved_params":params,
            "technology_tree":tree,
            "run_manifest":manifest
        });
        let bytes = serde_json::to_vec(&source).expect("source output JSON");
        let artifact_sha = sha256_hex(&bytes);
        let scenario = json!({
            "schema_version":"humanity-atlas.effective-scenario.v1",
            "source_model":"mycelix-multiworld-sim/0.1.0",
            "selected_definition_source":FIXTURE_DEFINITION_SOURCE,
            "effective_model_sha256":model_sha,
            "resolved_config_sha256":config_sha,
            "resolved_params_sha256":params_sha,
            "ticks_per_year":12,
            "ticks":24
        });
        let scenario_sha = stable_json_value_sha256(&scenario).expect("scenario digest");
        let run = ScenarioRunEnvelope {
            schema_version: super::super::RUN_SCHEMA_VERSION.into(),
            run_id: format!("urn:sha256:{artifact_sha}"),
            scenario_id: format!("mycelix-multiworld-sim:scenario:{scenario_sha}"),
            inputs: RunInputDigests {
                scenario_sha256: scenario_sha,
                dataset_sha256: None,
                scenario_identity_kind: Some("canonical_effective_scenario".into()),
                resolved_config_sha256: Some(config_sha),
                resolved_params_sha256: Some(params_sha),
                selected_definition_source: Some(FIXTURE_DEFINITION_SOURCE.into()),
            },
            engine: EngineIdentity {
                adapter: EXPECTED_ADAPTER.into(),
                engine_version: "0.1.0".into(),
                model_sha256: model_sha,
                model_identity_kind: Some("technology_tree_definition".into()),
                source_revision: revision,
                definition_source: Some(FIXTURE_DEFINITION_SOURCE.into()),
            },
            run_status: RunStatus::Completed,
            output_classification: OutputClassification::UnvalidatedSimulation,
            deterministic: None,
            seed: 42,
            replicates: 1,
            outputs: vec![
                MetricOutput {
                    metric_id:"final_cvs".into(), year:2, time_basis:Some(OutputTimeBasis::ScenarioRelativeYear),
                    unit:"CVS".into(), statistic:OutputStatistic::Value, value:0.73, sample_count:1,
                },
                MetricOutput {
                    metric_id:"final_population".into(), year:2, time_basis:Some(OutputTimeBasis::ScenarioRelativeYear),
                    unit:"persons".into(), statistic:OutputStatistic::Value, value:1234.0, sample_count:1,
                },
                MetricOutput {
                    metric_id:"tech_milestones_achieved".into(), year:2, time_basis:Some(OutputTimeBasis::ScenarioRelativeYear),
                    unit:"milestones".into(), statistic:OutputStatistic::Value, value:0.0, sample_count:1,
                },
            ],
            validation: ValidationRecord {
                status: ValidationStatus::NotRun,
                notes:vec!["structural adapter only".into()],
                evidence:vec![],
            },
            warnings:vec!["not calibrated".into()],
            artifacts:vec![RunArtifactReference {
                artifact_id:"source-simulator-output".into(),
                role:"source_simulation_output".into(),
                media_type:"application/json".into(),
                artifact_ref:format!("urn:sha256:{artifact_sha}"),
                artifact_sha256:artifact_sha,
                json_pointer:Some(EXPECTED_JSON_POINTER.into()),
            }],
        };
        (run, bytes)
    }

    #[test]
    fn checked_in_schema_matches_the_importer_contract_version() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../schemas/mycelix-multiworld-sim-tech-tree-v1.schema.json"
        )).expect("checked-in technology-tree schema must be valid JSON");
        assert_eq!(schema["$id"], "https://atlas.luminousdynamics.io/schemas/mycelix-multiworld-sim-tech-tree-v1.schema.json");
        assert_eq!(schema["properties"]["schema_version"]["const"], TREE_SCHEMA_VERSION);
        assert_eq!(schema["properties"]["classification"]["const"], EXPECTED_CLASSIFICATION);
        assert_eq!(schema["properties"]["year_zero_semantics"]["const"], EXPECTED_YEAR_ZERO);
        assert_eq!(schema["additionalProperties"], false);
        assert!(schema["$defs"]["milestone"]["required"]
            .as_array()
            .expect("milestone required fields")
            .iter()
            .any(|field| field == "achieved_tick"));
    }

    #[test]
    fn imports_and_binds_a_valid_synthetic_source_artifact() {
        let (run, raw) = fixture(vec![
            test_milestone("Fusion Demo", vec![], vec![(0,0.5)]),
            test_milestone("Fusion Grid Scale", vec![(technology_milestone_id("Fusion Demo"),"Fusion Demo".into())], vec![]),
        ]);
        let tree = import_technology_tree(&run, "source-simulator-output", &raw).expect("valid fixture");
        assert_eq!(tree.milestones.len(), 2);
        assert_eq!(tree.dependencies.len(), 1);
        assert_eq!(tree.dependencies[0].prerequisite_id, technology_milestone_id("Fusion Demo"));
        assert_eq!(tree.dependencies[0].dependent_id, technology_milestone_id("Fusion Grid Scale"));
        assert_eq!(tree.horizon_relative_years, 2);
        assert_eq!(tree.milestones[1].earliest_relative_year, 0.0);
        assert_eq!(tree.milestones[0].effects.tech_level_boost, vec![(0,0.5)]);
    }

    #[test]
    fn rejects_milestone_achievement_after_run_horizon() {
        let mut milestone = test_milestone("Fusion Demo", vec![], vec![]);
        milestone.achieved = true;
        milestone.achieved_tick = Some(36);
        let (run, raw) = fixture(vec![milestone]);
        assert!(import_technology_tree(&run, "source-simulator-output", &raw)
            .unwrap_err().contains("achieved after the run horizon"));
    }

    #[test]
    fn rejects_missing_required_achieved_tick_key() {
        let (run, raw) = fixture(vec![test_milestone("Fusion Demo", vec![], vec![])]);
        let raw_text = String::from_utf8(raw).expect("source fixture is UTF-8");
        let malformed = raw_text.replacen(",\"achieved_tick\":null", "", 1).into_bytes();
        let digest = sha256_hex(&malformed);
        let mut adjusted = run;
        adjusted.run_id = format!("urn:sha256:{digest}");
        adjusted.artifacts[0].artifact_sha256 = digest.clone();
        adjusted.artifacts[0].artifact_ref = format!("urn:sha256:{digest}");
        assert!(import_technology_tree(&adjusted, "source-simulator-output", &malformed)
            .unwrap_err().contains("missing required achieved_tick key"));
    }

    #[test]
    fn rejects_tampered_artifact_before_parsing() {
        let (run, raw) = fixture(vec![test_milestone("Fusion Demo", vec![], vec![])]);
        let mut changed = raw.clone();
        changed.push(b' ');
        assert!(import_technology_tree(&run, "source-simulator-output", &changed)
            .unwrap_err().contains("artifact digest mismatch"));
    }

    #[test]
    fn rejects_duplicate_json_keys_before_deserializing() {
        let (run, raw) = fixture(vec![test_milestone("Fusion Demo", vec![], vec![])]);
        let raw_text = String::from_utf8(raw.clone()).expect("source fixture is UTF-8");
        let duplicate = raw_text
            .replacen("\"seed\":42", "\"seed\":42,\"seed\":42", 1)
            .into_bytes();
        // Keep the generic artifact binding valid so the duplicate-key check is
        // the first failure rather than the outer digest check.
        let mut adjusted = run.clone();
        let digest = sha256_hex(&duplicate);
        adjusted.run_id = format!("urn:sha256:{digest}");
        adjusted.artifacts[0].artifact_sha256 = digest.clone();
        adjusted.artifacts[0].artifact_ref = format!("urn:sha256:{digest}");
        assert!(import_technology_tree(&adjusted, "source-simulator-output", &duplicate)
            .unwrap_err().contains("duplicate JSON object key"));
    }

    #[test]
    fn rejects_a_cyclic_prerequisite_graph_even_with_recomputed_digest() {
        let a = technology_milestone_id("A");
        let b = technology_milestone_id("B");
        let (run, raw) = fixture(vec![
            test_milestone("A", vec![(b.clone(),"B".into())], vec![]),
            test_milestone("B", vec![(a.clone(),"A".into())], vec![]),
        ]);
        assert!(import_technology_tree(&run, "source-simulator-output", &raw)
            .unwrap_err().contains("contains a cycle"));
    }

    #[test]
    fn rejects_scenario_id_not_bound_to_effective_inputs() {
        let (mut run, raw) = fixture(vec![test_milestone("Fusion Demo", vec![], vec![])]);
        run.scenario_id = "mycelix-multiworld-sim:scenario:unrelated-but-long-enough".into();
        assert!(import_technology_tree(&run, "source-simulator-output", &raw)
            .unwrap_err().contains("scenario identity digest/ID"));
    }

    #[test]
    fn rejects_summary_metric_not_bound_to_source_bytes() {
        let (mut run, raw) = fixture(vec![test_milestone("Fusion Demo", vec![], vec![])]);
        run.outputs[0].value = 0.74;
        assert!(import_technology_tree(&run, "source-simulator-output", &raw)
            .unwrap_err().contains("summary metric final_cvs disagrees"));
    }

    #[test]
    fn rejects_tree_hash_scope_mismatch() {
        let (mut run, raw) = fixture(vec![test_milestone("Fusion Demo", vec![], vec![])]);
        run.engine.model_sha256 = "f".repeat(64);
        assert!(import_technology_tree(&run, "source-simulator-output", &raw)
            .unwrap_err().contains("definition digest disagrees"));
    }
}
