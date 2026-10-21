// Copyright (C) 2026 Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! End-to-end structural conformance for the draft landscape artifact chain.
//! This tool proves reference/linkage invariants for a synthetic fixture only.
//! It does not validate a physical landscape model or replace upstream validators.

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const REPORT_SCHEMA: &str = "luminous.landscape-chain-conformance-report.v0.1";
const SUITE_ID: &str = "LANDSCAPE-CHAIN-CONFORMANCE-V0.1";
const CLAIM_CEILING: &str = "SYNTHETIC_STRUCTURAL_CONFORMANCE_ONLY";
const DESIGN_SCHEMA: &str = "luminous.landscape-design.v0";
const DESIGN_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-design-v0.schema.json";
const EVAL_SCHEMA: &str = "luminous.landscape-evaluation-plan.v0";
const EVAL_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-evaluation-plan-v0.schema.json";
const OBS_BATCH_SCHEMA: &str = "luminous.landscape-observation-batch.v0";
const SCENARIO_SCHEMA: &str = "luminous.landscape-scenario-manifest.v0";
const STEWARDSHIP_SCHEMA: &str = "mycelix.landscape-stewardship-profile.v0";
const REVIEW_SCHEMA: &str = "symthaea.landscape-alternative-review.v0";
const LOCAL_FRAME: &str = "LOCAL_CARTESIAN_DRAWING_UNITS";

#[derive(Debug, Clone)]
pub struct ChainArtifacts {
    project_bytes: Vec<u8>,
    alternative_bytes: Vec<u8>,
    boundary_bytes: Vec<u8>,
    intervention_site_bytes: Vec<u8>,
    comparison_site_bytes: Vec<u8>,
    metric_registry_bytes: Vec<u8>,
    metric_observation_map_bytes: Vec<u8>,
    source_catalog_bytes: Vec<u8>,
    measurement_authorization_template_bytes: Vec<u8>,
    upstream_manifest_bytes: Vec<u8>,
    project: Value,
    alternative: Value,
    boundary: Value,
    intervention_site: Value,
    comparison_site: Value,
    metric_registry: Value,
    metric_observation_map: Value,
    source_catalog: Value,
    measurement_authorization: Value,
    upstream_manifest: Value,
    evaluation_plan: Value,
    observations: Value,
    scenario: Value,
    alternative_scenario: Value,
    stewardship: Value,
    review: Value,
    template_digests: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConformanceReport {
    pub schema_version: String,
    pub suite_id: String,
    pub conformance_status: String,
    pub data_origin: String,
    pub claim_ceiling: String,
    pub upstream_pinned_contract_count: usize,
    pub project_feature_count: usize,
    pub observation_count: usize,
    pub metric_definition_count: usize,
    pub metric_observation_mapping_count: usize,
    pub observations_matching_crosswalk_count: usize,
    pub eligible_supporting_observation_count: usize,
    pub unmapped_observation_count: usize,
    pub source_catalog_record_count: usize,
    pub source_catalog_status: String,
    pub measurement_consent_status: String,
    pub site_access_status: String,
    pub measurement_authorization_verification_status: String,
    pub measurement_operation_authorized_by_harness: bool,
    pub scenario_execution_status: String,
    pub paired_scenario_count: usize,
    pub paired_scenarios_share_boundary_conditions: bool,
    pub review_ranking_status: String,
    pub recommendation_status: String,
    pub passed_check_count: usize,
    pub checks: Vec<String>,
    pub negative_controls_attempted: usize,
    pub negative_controls_rejected: usize,
    pub negative_controls_unexpectedly_accepted: usize,
    pub artifact_sha256: BTreeMap<String, String>,
    pub template_input_sha256: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
struct NegativeControlSummary {
    attempted: usize,
    rejected: usize,
    unexpectedly_accepted: usize,
    cases: Vec<String>,
}

fn read_bytes(dir: &Path, name: &str) -> Result<Vec<u8>, String> {
    fs::read(dir.join(name)).map_err(|error| format!("could not read {name}: {error}"))
}

fn parse_json(bytes: &[u8], label: &str) -> Result<Value, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("{label}: invalid JSON: {error}"))
}

fn read_json(dir: &Path, name: &str) -> Result<(Vec<u8>, Value), String> {
    let bytes = read_bytes(dir, name)?;
    let value = parse_json(&bytes, name)?;
    Ok((bytes, value))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("could not serialize canonical JSON: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn canonical_sha256(value: &Value) -> Result<String, String> {
    Ok(sha256_hex(&canonical_json_bytes(value)?))
}

fn at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for component in path {
        current = current.get(*component)?;
    }
    Some(current)
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    at(value, path)?.as_str()
}

fn set_at(value: &mut Value, path: &[&str], new_value: Value) -> Result<(), String> {
    if path.is_empty() {
        return Err("cannot assign an empty JSON path".to_owned());
    }
    let mut current = value;
    for component in &path[..path.len() - 1] {
        current = current
            .get_mut(*component)
            .ok_or_else(|| format!("missing JSON path component: {component}"))?;
    }
    let object = current
        .as_object_mut()
        .ok_or_else(|| format!("JSON path parent is not an object: {}", path.join(".")))?;
    object.insert(path[path.len() - 1].to_owned(), new_value);
    Ok(())
}

fn set_str(value: &mut Value, path: &[&str], new_value: &str) -> Result<(), String> {
    set_at(value, path, json!(new_value))
}

fn check_eq(errors: &mut Vec<String>, value: &Value, path: &[&str], expected: &str, label: &str) {
    match string_at(value, path) {
        Some(actual) if actual == expected => {}
        Some(actual) => errors.push(format!("{label}: expected {expected:?}, found {actual:?}")),
        None => errors.push(format!("{label}: missing string field {}", path.join("."))),
    }
}

fn check_true(errors: &mut Vec<String>, value: &Value, path: &[&str], label: &str) {
    if at(value, path).and_then(Value::as_bool) != Some(true) {
        errors.push(format!("{label}: {} must be true", path.join(".")));
    }
}

fn check_false(errors: &mut Vec<String>, value: &Value, path: &[&str], label: &str) {
    if at(value, path).and_then(Value::as_bool) != Some(false) {
        errors.push(format!("{label}: {} must be false", path.join(".")));
    }
}

fn digest_is_well_formed(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
}

fn require_digest(errors: &mut Vec<String>, value: Option<&str>, label: &str) {
    if !value.is_some_and(digest_is_well_formed) {
        errors.push(format!(
            "{label}: expected a 64-character lowercase SHA-256 digest"
        ));
    }
}

fn feature_ids(project: &Value) -> Result<BTreeSet<String>, String> {
    let features = project
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| "project.features must be an array".to_owned())?;
    let mut ids = BTreeSet::new();
    for feature in features {
        let id = feature
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "each project feature must have a string id".to_owned())?;
        if !ids.insert(id.to_owned()) {
            return Err(format!("duplicate project feature id: {id}"));
        }
    }
    Ok(ids)
}

fn feature_count(project: &Value) -> usize {
    project
        .get("features")
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

fn records<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Vec<Value>> {
    at(value, path)?.as_array()
}

fn template_digest_map(items: &[(&str, &[u8])]) -> BTreeMap<String, String> {
    items
        .iter()
        .map(|(name, bytes)| ((*name).to_owned(), sha256_hex(bytes)))
        .collect()
}

pub fn build_chain(dir: &Path) -> Result<ChainArtifacts, Vec<String>> {
    let mut construction_errors = Vec::new();
    let (project_bytes, project) = read_json(dir, "project.json").map_err(|error| vec![error])?;
    let (alternative_bytes, alternative) =
        read_json(dir, "alternative.json").map_err(|error| vec![error])?;
    let (boundary_bytes, boundary) =
        read_json(dir, "boundary-conditions.json").map_err(|error| vec![error])?;
    let (intervention_site_bytes, intervention_site) =
        read_json(dir, "site-intervention.json").map_err(|error| vec![error])?;
    let (comparison_site_bytes, comparison_site) =
        read_json(dir, "site-comparison.json").map_err(|error| vec![error])?;
    let (metric_registry_bytes, metric_registry) =
        read_json(dir, "metric-registry.json").map_err(|error| vec![error])?;
    let (metric_observation_map_bytes, metric_observation_map) =
        read_json(dir, "metric-observation-map.json").map_err(|error| vec![error])?;
    let (source_catalog_bytes, source_catalog) =
        read_json(dir, "source-catalog.json").map_err(|error| vec![error])?;
    let (measurement_authorization_template_bytes, mut measurement_authorization) =
        read_json(dir, "measurement-authorization-template.json").map_err(|error| vec![error])?;
    let (upstream_manifest_bytes, upstream_manifest) =
        read_json(dir, "upstream-contract-manifest.json").map_err(|error| vec![error])?;
    let (eval_template_bytes, mut evaluation_plan) =
        read_json(dir, "evaluation-plan-template.json").map_err(|error| vec![error])?;
    let (obs_template_bytes, mut observations) =
        read_json(dir, "observation-batch-template.json").map_err(|error| vec![error])?;
    let (scenario_template_bytes, mut scenario) =
        read_json(dir, "scenario-manifest-template.json").map_err(|error| vec![error])?;
    let (stewardship_template_bytes, mut stewardship) =
        read_json(dir, "stewardship-template.json").map_err(|error| vec![error])?;
    let (review_template_bytes, mut review) =
        read_json(dir, "review-template.json").map_err(|error| vec![error])?;

    let project_sha = sha256_hex(&project_bytes);
    let alternative_sha = sha256_hex(&alternative_bytes);
    let boundary_sha = sha256_hex(&boundary_bytes);
    let intervention_site_sha = sha256_hex(&intervention_site_bytes);
    let comparison_site_sha = sha256_hex(&comparison_site_bytes);
    let metric_registry_sha = sha256_hex(&metric_registry_bytes);
    let source_catalog_sha = sha256_hex(&source_catalog_bytes);

    let permission_metric = records(&metric_registry, &["metrics"])
        .and_then(|metrics| metrics.first())
        .ok_or_else(|| vec!["metric registry requires at least one metric for permission binding".to_owned()])?;
    let permission_metric_id = string_at(permission_metric, &["id"])
        .ok_or_else(|| vec!["permission metric requires an id".to_owned()])?
        .to_owned();
    let permission_metric_unit = string_at(permission_metric, &["unit"])
        .ok_or_else(|| vec!["permission metric requires a unit".to_owned()])?
        .to_owned();
    set_str(
        &mut measurement_authorization,
        &["project_snapshot_sha256"],
        &project_sha,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut measurement_authorization,
        &["site_ref_sha256"],
        &intervention_site_sha,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut measurement_authorization,
        &["metric_id"],
        &permission_metric_id,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut measurement_authorization,
        &["unit"],
        &permission_metric_unit,
    )
    .map_err(|error| vec![error])?;
    set_str(&mut measurement_authorization, &["consent_status"], "unknown")
        .map_err(|error| vec![error])?;
    set_at(
        &mut measurement_authorization,
        &["consent_evidence_sha256"],
        Value::Null,
    )
    .map_err(|error| vec![error])?;
    set_str(&mut measurement_authorization, &["site_access_status"], "unknown")
        .map_err(|error| vec![error])?;
    set_at(
        &mut measurement_authorization,
        &["site_access_evidence_sha256"],
        Value::Null,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut measurement_authorization,
        &["publication_consent_status"],
        "not-requested",
    )
    .map_err(|error| vec![error])?;
    set_at(
        &mut measurement_authorization,
        &["publication_consent_evidence_sha256"],
        Value::Null,
    )
    .map_err(|error| vec![error])?;
    set_str(&mut measurement_authorization, &["publication_policy"], "private")
        .map_err(|error| vec![error])?;
    set_str(&mut measurement_authorization, &["coordinate_disclosure"], "none")
        .map_err(|error| vec![error])?;
    set_str(&mut measurement_authorization, &["evidence_class"], "synthetic")
        .map_err(|error| vec![error])?;
    set_str(&mut measurement_authorization, &["verification_status"], "not-verified")
        .map_err(|error| vec![error])?;

    let intervention_arm = evaluation_plan
        .get_mut("arms")
        .and_then(Value::as_array_mut)
        .and_then(|arms| arms.get_mut(0))
        .ok_or_else(|| {
            vec!["evaluation-plan template requires at least one intervention arm".to_owned()]
        })?;
    intervention_arm["site_ref_sha256"] = json!(intervention_site_sha);
    let comparison_arm = evaluation_plan
        .get_mut("arms")
        .and_then(Value::as_array_mut)
        .and_then(|arms| arms.get_mut(1))
        .ok_or_else(|| {
            vec!["evaluation-plan template requires a second comparison arm".to_owned()]
        })?;
    comparison_arm["site_ref_sha256"] = json!(comparison_site_sha);
    set_str(
        &mut evaluation_plan,
        &["project_snapshot_sha256"],
        &project_sha,
    )
    .map_err(|error| vec![error])?;
    let scope = string_at(&project, &["scope"])
        .ok_or_else(|| vec!["project.scope is missing".to_owned()])?
        .to_owned();
    set_str(&mut evaluation_plan, &["project_scope"], &scope).map_err(|error| vec![error])?;
    set_str(
        &mut evaluation_plan,
        &["evaluation_id"],
        "eval-site-demo-v0",
    )
    .map_err(|error| vec![error])?;
    let evaluation_plan_sha = canonical_sha256(&evaluation_plan).map_err(|error| vec![error])?;

    let observation_rows = observations
        .get_mut("observations")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| vec!["observation-batch template requires observations[]".to_owned()])?;
    for observation in observation_rows {
        set_str(
            observation,
            &["project_ref", "project_snapshot_sha256"],
            &project_sha,
        )
        .map_err(|error| vec![error])?;
    }
    let geometry_status = string_at(&project, &["geometry_status"])
        .ok_or_else(|| vec!["project.geometry_status is missing".to_owned()])?
        .to_owned();
    let coordinate_frame_kind = string_at(&project, &["coordinate_frame", "kind"])
        .ok_or_else(|| vec!["project.coordinate_frame.kind is missing".to_owned()])?
        .to_owned();
    set_str(&mut scenario, &["design_snapshot", "sha256"], &project_sha)
        .map_err(|error| vec![error])?;
    set_str(&mut scenario, &["design_snapshot", "scope"], &scope).map_err(|error| vec![error])?;
    set_str(
        &mut scenario,
        &["design_snapshot", "geometry_status"],
        &geometry_status,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut scenario,
        &["design_snapshot", "coordinate_frame_kind"],
        &coordinate_frame_kind,
    )
    .map_err(|error| vec![error])?;
    set_at(
        &mut scenario,
        &["design_snapshot", "georeferenced"],
        project["coordinate_frame"]["georeferenced"].clone(),
    )
    .map_err(|error| vec![error])?;
    set_at(
        &mut scenario,
        &["design_snapshot", "feature_count"],
        json!(feature_count(&project)),
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut scenario,
        &["boundary_conditions_sha256"],
        &boundary_sha,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut scenario,
        &["evaluation_plan", "sha256"],
        &evaluation_plan_sha,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut scenario,
        &["evaluation_plan", "evaluation_id"],
        "eval-site-demo-v0",
    )
    .map_err(|error| vec![error])?;
    let scenario_sha = canonical_sha256(&scenario).map_err(|error| vec![error])?;
    let mut alternative_scenario = scenario.clone();
    set_str(
        &mut alternative_scenario,
        &["scenario_id"],
        "scenario-landscape-alternative-synthetic-v0",
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut alternative_scenario,
        &["design_snapshot", "sha256"],
        &alternative_sha,
    )
    .map_err(|error| vec![error])?;
    set_at(
        &mut alternative_scenario,
        &["design_snapshot", "feature_count"],
        json!(feature_count(&alternative)),
    )
    .map_err(|error| vec![error])?;
    let alternative_scenario_sha =
        canonical_sha256(&alternative_scenario).map_err(|error| vec![error])?;

    set_str(
        &mut stewardship,
        &["project_snapshot", "sha256"],
        &project_sha,
    )
    .map_err(|error| vec![error])?;
    set_str(&mut stewardship, &["project_snapshot", "scope"], &scope)
        .map_err(|error| vec![error])?;
    set_str(
        &mut stewardship,
        &["project_snapshot", "coordinate_frame_kind"],
        &coordinate_frame_kind,
    )
    .map_err(|error| vec![error])?;
    set_at(
        &mut stewardship,
        &["project_snapshot", "georeferenced"],
        project["coordinate_frame"]["georeferenced"].clone(),
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut stewardship,
        &["project_snapshot", "geometry_status"],
        &geometry_status,
    )
    .map_err(|error| vec![error])?;
    set_str(&mut stewardship, &["record_type"], "scenario-review").map_err(|error| vec![error])?;
    set_str(&mut stewardship, &["evidence_class"], "synthetic").map_err(|error| vec![error])?;
    set_str(
        &mut stewardship,
        &["actor_authority", "publication_consent"],
        "not-requested",
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut stewardship,
        &["access_policy", "visibility"],
        "private",
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut stewardship,
        &["access_policy", "coordinate_disclosure"],
        "none",
    )
    .map_err(|error| vec![error])?;
    set_at(
        &mut stewardship,
        &["access_policy", "sensitive_site_data_included"],
        json!(false),
    )
    .map_err(|error| vec![error])?;
    stewardship["scenario_reference"] = json!({
        "system": "symtropy",
        "manifest_sha256": scenario_sha,
        "model_id": scenario["model_id"],
        "model_version": scenario["model_version"],
        "purpose": scenario["purpose"],
        "evidence_class": "synthetic"
    });

    set_str(
        &mut review,
        &["inputs", "baseline_design", "sha256"],
        &project_sha,
    )
    .map_err(|error| vec![error])?;
    set_str(&mut review, &["inputs", "baseline_design", "scope"], &scope)
        .map_err(|error| vec![error])?;
    set_str(
        &mut review,
        &["inputs", "baseline_design", "coordinate_frame_kind"],
        &coordinate_frame_kind,
    )
    .map_err(|error| vec![error])?;
    set_at(
        &mut review,
        &["inputs", "baseline_design", "georeferenced"],
        project["coordinate_frame"]["georeferenced"].clone(),
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut review,
        &["inputs", "baseline_design", "geometry_status"],
        &geometry_status,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut review,
        &["inputs", "metric_registry_sha256"],
        &metric_registry_sha,
    )
    .map_err(|error| vec![error])?;
    set_str(
        &mut review,
        &["inputs", "source_catalog_sha256"],
        &source_catalog_sha,
    )
    .map_err(|error| vec![error])?;

    let alternatives = review
        .get_mut("alternatives")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| vec!["review template requires alternatives[]".to_owned()])?;
    if alternatives.len() < 2 {
        return Err(vec![
            "review template requires at least two alternatives".to_owned(),
        ]);
    }
    alternatives[0]["snapshot_sha256"] = json!(project_sha);
    alternatives[0]["scope"] = json!(scope);
    alternatives[0]["coordinate_frame_kind"] = json!(coordinate_frame_kind);
    alternatives[0]["geometry_status"] = json!(geometry_status);
    alternatives[0]["changed_feature_ids"] = json!([]);
    alternatives[0]["change_summary"] = json!("Baseline authored design snapshot; schematic only.");
    alternatives[1]["snapshot_sha256"] = json!(alternative_sha);
    alternatives[1]["scope"] = json!(scope);
    alternatives[1]["coordinate_frame_kind"] = json!(coordinate_frame_kind);
    alternatives[1]["geometry_status"] = json!(geometry_status);
    alternatives[1]["changed_feature_ids"] = json!(["feature-0004"]);
    alternatives[1]["change_summary"] =
        json!("Synthetic alternative moving the water symbol; not a hydraulic simulation.");

    let model_id = string_at(&scenario, &["model_id"])
        .ok_or_else(|| vec!["scenario.model_id is missing".to_owned()])?
        .to_owned();
    let model_version = string_at(&scenario, &["model_version"])
        .ok_or_else(|| vec!["scenario.model_version is missing".to_owned()])?
        .to_owned();
    let purpose = string_at(&scenario, &["purpose"])
        .ok_or_else(|| vec!["scenario.purpose is missing".to_owned()])?
        .to_owned();
    let baseline_review_alternative_id = review["alternatives"][0]["alternative_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    let alternative_review_alternative_id = review["alternatives"][1]["alternative_id"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    review["inputs"]["scenario_runs"] = json!([
        {
            "system": "symtropy",
            "manifest_sha256": scenario_sha,
            "purpose": purpose,
            "evidence_class": "synthetic",
            "status": "blocked",
            "model_id": model_id,
            "model_version": model_version,
            "pairing_context": {
                "alternative_id": baseline_review_alternative_id,
                "design_snapshot_sha256": project_sha,
                "boundary_conditions_sha256": boundary_sha,
                "evaluation_plan_sha256": evaluation_plan_sha,
                "metric_registry_sha256": metric_registry_sha,
                "seed": scenario["seed"],
                "scope": scope
            },
            "limitations": [
                "Baseline scenario is schema-qualified only; it has not run a physical landscape model.",
                "Design uses schematic local drawing units and is not georeferenced."
            ]
        },
        {
            "system": "symtropy",
            "manifest_sha256": alternative_scenario_sha,
            "purpose": purpose,
            "evidence_class": "synthetic",
            "status": "blocked",
            "model_id": model_id,
            "model_version": model_version,
            "pairing_context": {
                "alternative_id": alternative_review_alternative_id,
                "design_snapshot_sha256": alternative_sha,
                "boundary_conditions_sha256": boundary_sha,
                "evaluation_plan_sha256": evaluation_plan_sha,
                "metric_registry_sha256": metric_registry_sha,
                "seed": scenario["seed"],
                "scope": scope
            },
            "limitations": [
                "Alternative scenario is schema-qualified only; it has not run a physical landscape model.",
                "Design uses schematic local drawing units and is not georeferenced."
            ]
        }
    ]);

    let template_digests = template_digest_map(&[
        ("evaluation-plan-template.json", &eval_template_bytes),
        ("observation-batch-template.json", &obs_template_bytes),
        ("scenario-manifest-template.json", &scenario_template_bytes),
        ("stewardship-template.json", &stewardship_template_bytes),
        ("review-template.json", &review_template_bytes),
        ("metric-observation-map.json", &metric_observation_map_bytes),
        ("upstream-contract-manifest.json", &upstream_manifest_bytes),
    ]);

    // Construction-time checks catch unanticipated template changes before report generation.
    if project_sha == alternative_sha {
        construction_errors
            .push("baseline and alternative must be distinct exact byte sequences".to_owned());
    }
    if construction_errors.is_empty() {
        Ok(ChainArtifacts {
            project_bytes,
            alternative_bytes,
            boundary_bytes,
            intervention_site_bytes,
            comparison_site_bytes,
            metric_registry_bytes,
            metric_observation_map_bytes,
            source_catalog_bytes,
            upstream_manifest_bytes,
            project,
            alternative,
            boundary,
            intervention_site,
            comparison_site,
            metric_registry,
            metric_observation_map,
            source_catalog,
            upstream_manifest,
            evaluation_plan,
            observations,
            scenario,
            alternative_scenario,
            stewardship,
            review,
            template_digests,
        })
    } else {
        Err(construction_errors)
    }
}

fn registry_metric_ids(registry: &Value) -> BTreeSet<&str> {
    records(registry, &["metrics"])
        .into_iter()
        .flatten()
        .filter_map(|metric| string_at(metric, &["id"]))
        .collect()
}

fn observation_crosswalk_match_count(chain: &ChainArtifacts) -> usize {
    let mappings = records(&chain.metric_observation_map, &["mappings"]);
    let observations = records(&chain.observations, &["observations"]);
    let Some(mappings) = mappings else { return 0 };
    let Some(observations) = observations else {
        return 0;
    };
    observations
        .iter()
        .filter(|observation| {
            let Some(property_id) = string_at(observation, &["observed_property", "id"]) else {
                return false;
            };
            mappings.iter().any(|mapping| {
                records(mapping, &["possible_observed_properties"]).is_some_and(|properties| {
                    properties
                        .iter()
                        .any(|property| string_at(property, &["id"]) == Some(property_id))
                })
            })
        })
        .count()
}

fn eligible_supporting_observation_count(chain: &ChainArtifacts) -> usize {
    let mappings = records(&chain.metric_observation_map, &["mappings"]);
    let observations = records(&chain.observations, &["observations"]);
    let Some(mappings) = mappings else { return 0 };
    let Some(observations) = observations else {
        return 0;
    };
    observations
        .iter()
        .filter(|observation| {
            let property_id = string_at(observation, &["observed_property", "id"]);
            let evidence_class = string_at(observation, &["evidence_class"]);
            let Some(property_id) = property_id else {
                return false;
            };
            let Some(evidence_class) = evidence_class else {
                return false;
            };
            if matches!(evidence_class, "synthetic-test" | "synthetic-fixture") {
                return false;
            }
            mappings.iter().any(|mapping| {
                records(mapping, &["possible_observed_properties"]).is_some_and(|properties| {
                    properties.iter().any(|property| {
                        string_at(property, &["id"]) == Some(property_id)
                            && records(property, &["minimum_evidence_classes"]).is_some_and(
                                |classes| {
                                    classes
                                        .iter()
                                        .any(|class| class.as_str() == Some(evidence_class))
                                },
                            )
                            && string_at(observation, &["observed_property", "unit"])
                                == string_at(property, &["unit"])
                            && string_at(observation, &["result", "kind"]).is_some_and(|kind| {
                                records(property, &["result_kinds"]).is_some_and(|kinds| {
                                    kinds.iter().any(|expected| expected.as_str() == Some(kind))
                                })
                            })
                    })
                })
            })
        })
        .count()
}

fn unmapped_observation_count(chain: &ChainArtifacts) -> usize {
    records(&chain.observations, &["observations"]).map_or(0, |observations| {
        observations
            .len()
            .saturating_sub(observation_crosswalk_match_count(chain))
    })
}

fn validate_observation_crosswalk(chain: &ChainArtifacts) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(mappings) = records(&chain.metric_observation_map, &["mappings"]) else {
        return vec!["metric-observation crosswalk requires mappings[]".to_owned()];
    };
    let Some(observations) = records(&chain.observations, &["observations"]) else {
        return vec!["observation batch requires observations[]".to_owned()];
    };

    for observation in observations {
        let observation_id = string_at(observation, &["observation_id"]).unwrap_or("<unknown>");
        let Some(property_id) = string_at(observation, &["observed_property", "id"]) else {
            errors.push(format!("{observation_id}: observed property ID is missing"));
            continue;
        };
        for mapping in mappings {
            let metric_id = string_at(mapping, &["metric_id"]).unwrap_or("<unknown-metric>");
            let Some(properties) = records(mapping, &["possible_observed_properties"]) else {
                continue;
            };
            for property in properties {
                if string_at(property, &["id"]) != Some(property_id) {
                    continue;
                }

                let declared_unit = string_at(property, &["unit"]);
                let observed_unit = string_at(observation, &["observed_property", "unit"]);
                let result_unit = string_at(observation, &["result", "unit"]);
                if observed_unit != declared_unit || result_unit != declared_unit {
                    errors.push(format!(
                        "{observation_id}: units do not match crosswalk property {property_id} for metric {metric_id}"
                    ));
                }

                let result_kind = string_at(observation, &["result", "kind"]);
                let kind_allowed = result_kind.is_some_and(|kind| {
                    records(property, &["result_kinds"]).is_some_and(|kinds| {
                        kinds.iter().any(|expected| expected.as_str() == Some(kind))
                    })
                });
                if !kind_allowed {
                    errors.push(format!(
                        "{observation_id}: result kind does not match crosswalk property {property_id} for metric {metric_id}"
                    ));
                }

                if let Some(observation_metric_id) =
                    string_at(observation, &["observed_property", "metric_id"])
                {
                    if observation_metric_id != metric_id {
                        errors.push(format!(
                            "{observation_id}: explicit metric ID conflicts with crosswalk mapping {metric_id}"
                        ));
                    }
                }
            }
        }
    }
    errors.sort();
    errors
}

pub fn validate_chain(chain: &ChainArtifacts) -> Vec<String> {
    let mut errors = Vec::new();

    let parsed_project = parse_json(&chain.project_bytes, "project.json");
    match parsed_project {
        Ok(parsed) if parsed == chain.project => {}
        Ok(_) => {
            errors.push("project snapshot object differs from its exact source bytes".to_owned())
        }
        Err(error) => errors.push(error),
    }
    let parsed_alternative = parse_json(&chain.alternative_bytes, "alternative.json");
    match parsed_alternative {
        Ok(parsed) if parsed == chain.alternative => {}
        Ok(_) => errors
            .push("alternative snapshot object differs from its exact source bytes".to_owned()),
        Err(error) => errors.push(error),
    }
    let parsed_boundary = parse_json(&chain.boundary_bytes, "boundary-conditions.json");
    match parsed_boundary {
        Ok(parsed) if parsed == chain.boundary => {}
        Ok(_) => {
            errors.push("boundary conditions object differs from its exact source bytes".to_owned())
        }
        Err(error) => errors.push(error),
    }
    let parsed_sites = [
        parse_json(&chain.intervention_site_bytes, "site-intervention.json"),
        parse_json(&chain.comparison_site_bytes, "site-comparison.json"),
    ];
    if !matches!(&parsed_sites[0], Ok(parsed) if *parsed == chain.intervention_site) {
        errors.push("intervention site object differs from its exact source bytes".to_owned());
    }
    if !matches!(&parsed_sites[1], Ok(parsed) if *parsed == chain.comparison_site) {
        errors.push("comparison site object differs from its exact source bytes".to_owned());
    }
    let parsed_metric_registry = parse_json(&chain.metric_registry_bytes, "metric-registry.json");
    if !matches!(parsed_metric_registry, Ok(parsed) if parsed == chain.metric_registry) {
        errors.push("metric registry object differs from its exact source bytes".to_owned());
    }
    let parsed_metric_observation_map = parse_json(
        &chain.metric_observation_map_bytes,
        "metric-observation-map.json",
    );
    if !matches!(parsed_metric_observation_map, Ok(parsed) if parsed == chain.metric_observation_map)
    {
        errors.push(
            "metric-observation crosswalk object differs from its exact source bytes".to_owned(),
        );
    }
    let parsed_source_catalog = parse_json(&chain.source_catalog_bytes, "source-catalog.json");
    if !matches!(parsed_source_catalog, Ok(parsed) if parsed == chain.source_catalog) {
        errors.push("source catalog object differs from its exact source bytes".to_owned());
    }

    let project_sha = sha256_hex(&chain.project_bytes);
    let alternative_sha = sha256_hex(&chain.alternative_bytes);
    let boundary_sha = sha256_hex(&chain.boundary_bytes);
    let intervention_site_sha = sha256_hex(&chain.intervention_site_bytes);
    let comparison_site_sha = sha256_hex(&chain.comparison_site_bytes);
    let metric_registry_sha = sha256_hex(&chain.metric_registry_bytes);
    let metric_observation_map_sha = sha256_hex(&chain.metric_observation_map_bytes);
    let source_catalog_sha = sha256_hex(&chain.source_catalog_bytes);
    let evaluation_plan_sha = match canonical_sha256(&chain.evaluation_plan) {
        Ok(digest) => digest,
        Err(error) => {
            errors.push(error);
            String::new()
        }
    };
    let observations_sha = match canonical_sha256(&chain.observations) {
        Ok(digest) => digest,
        Err(error) => {
            errors.push(error);
            String::new()
        }
    };
    let scenario_sha = match canonical_sha256(&chain.scenario) {
        Ok(digest) => digest,
        Err(error) => {
            errors.push(error);
            String::new()
        }
    };
    let alternative_scenario_sha = match canonical_sha256(&chain.alternative_scenario) {
        Ok(digest) => digest,
        Err(error) => {
            errors.push(error);
            String::new()
        }
    };
    let _stewardship_sha = match canonical_sha256(&chain.stewardship) {
        Ok(digest) => digest,
        Err(error) => {
            errors.push(error);
            String::new()
        }
    };
    let _review_sha = match canonical_sha256(&chain.review) {
        Ok(digest) => digest,
        Err(error) => {
            errors.push(error);
            String::new()
        }
    };

    check_eq(
        &mut errors,
        &chain.project,
        &["schema"],
        DESIGN_SCHEMA,
        "project",
    );
    check_eq(
        &mut errors,
        &chain.project,
        &["schema_uri"],
        DESIGN_SCHEMA_URI,
        "project",
    );
    check_eq(
        &mut errors,
        &chain.project,
        &["coordinate_frame", "kind"],
        LOCAL_FRAME,
        "project",
    );
    check_false(
        &mut errors,
        &chain.project,
        &["coordinate_frame", "georeferenced"],
        "project",
    );
    check_eq(
        &mut errors,
        &chain.alternative,
        &["schema"],
        DESIGN_SCHEMA,
        "alternative",
    );
    check_eq(
        &mut errors,
        &chain.boundary,
        &["schema"],
        "luminous.landscape-boundary-conditions.v0.1",
        "boundary conditions",
    );
    check_eq(
        &mut errors,
        &chain.boundary,
        &["data_origin"],
        "synthetic_fixture",
        "boundary conditions",
    );

    let project_features = match feature_ids(&chain.project) {
        Ok(ids) => ids,
        Err(error) => {
            errors.push(error);
            BTreeSet::new()
        }
    };
    let alternative_features = match feature_ids(&chain.alternative) {
        Ok(ids) => ids,
        Err(error) => {
            errors.push(error);
            BTreeSet::new()
        }
    };
    if project_features.is_empty() {
        errors.push(
            "project snapshot must contain at least one uniquely identified feature".to_owned(),
        );
    }
    if project_features == alternative_features && project_sha == alternative_sha {
        errors.push("alternative must have a distinct snapshot identity".to_owned());
    }

    check_eq(
        &mut errors,
        &chain.intervention_site,
        &["schema"],
        "luminous.landscape-site-reference.v0.1",
        "intervention site",
    );
    check_eq(
        &mut errors,
        &chain.comparison_site,
        &["schema"],
        "luminous.landscape-site-reference.v0.1",
        "comparison site",
    );
    for (label, site) in [
        ("intervention site", &chain.intervention_site),
        ("comparison site", &chain.comparison_site),
    ] {
        check_eq(
            &mut errors,
            site,
            &["data_origin"],
            "synthetic_fixture",
            label,
        );
        check_eq(&mut errors, site, &["scope"], "site-parcel", label);
        check_eq(
            &mut errors,
            site,
            &["coordinate_frame", "kind"],
            LOCAL_FRAME,
            label,
        );
        check_false(
            &mut errors,
            site,
            &["coordinate_frame", "georeferenced"],
            label,
        );
    }

    check_eq(
        &mut errors,
        &chain.evaluation_plan,
        &["schema"],
        EVAL_SCHEMA,
        "evaluation plan",
    );
    check_eq(
        &mut errors,
        &chain.evaluation_plan,
        &["schema_uri"],
        EVAL_SCHEMA_URI,
        "evaluation plan",
    );
    check_eq(
        &mut errors,
        &chain.evaluation_plan,
        &["project_snapshot_sha256"],
        &project_sha,
        "evaluation plan project reference",
    );
    if string_at(&chain.evaluation_plan, &["project_scope"])
        != string_at(&chain.project, &["scope"])
    {
        errors.push(
            "evaluation plan scope does not match the referenced project snapshot".to_owned(),
        );
    }
    let arms = records(&chain.evaluation_plan, &["arms"]);
    match arms {
        Some(arms) if arms.len() == 2 => {
            if string_at(&arms[0], &["site_ref_sha256"]) != Some(intervention_site_sha.as_str()) {
                errors.push(
                    "evaluation intervention arm does not reference exact intervention-site bytes"
                        .to_owned(),
                );
            }
            if string_at(&arms[1], &["site_ref_sha256"]) != Some(comparison_site_sha.as_str()) {
                errors.push(
                    "evaluation comparison arm does not reference exact comparison-site bytes"
                        .to_owned(),
                );
            }
        }
        _ => errors.push("evaluation plan must contain exactly two pinned site arms".to_owned()),
    }

    check_eq(
        &mut errors,
        &chain.observations,
        &["schema"],
        OBS_BATCH_SCHEMA,
        "observation batch",
    );
    let observations = records(&chain.observations, &["observations"]);
    let mut observation_count = 0usize;
    match observations {
        Some(rows) => {
            observation_count = rows.len();
            for row in rows {
                check_eq(
                    &mut errors,
                    row,
                    &["project_ref", "project_snapshot_sha256"],
                    &project_sha,
                    "observation project reference",
                );
                if let Some(feature_id) = string_at(row, &["project_ref", "feature_id"]) {
                    if !project_features.contains(feature_id) {
                        errors.push(format!(
                            "observation {} references missing project feature {feature_id}",
                            string_at(row, &["observation_id"]).unwrap_or("<unknown>")
                        ));
                    }
                } else {
                    errors.push("observation is missing project_ref.feature_id".to_owned());
                }
                check_eq(
                    &mut errors,
                    row,
                    &["evidence_class"],
                    "synthetic-test",
                    "observation evidence class",
                );
                check_eq(
                    &mut errors,
                    row,
                    &["privacy", "classification"],
                    "synthetic",
                    "observation privacy class",
                );
                require_digest(
                    &mut errors,
                    string_at(row, &["source", "payload_sha256"]),
                    "observation source payload reference",
                );
            }
        }
        None => errors.push("observation batch requires observations[]".to_owned()),
    }
    if observation_count == 0 {
        errors.push("observation batch must contain at least one observation".to_owned());
    }

    check_eq(
        &mut errors,
        &chain.metric_observation_map,
        &["schema"],
        "luminous.landscape-metric-observation-map.v0",
        "metric-observation crosswalk",
    );
    check_eq(
        &mut errors,
        &chain.metric_observation_map,
        &["schema_uri"],
        "https://luminousdynamics.io/schemas/landscape-metric-observation-map-v0.schema.json",
        "metric-observation crosswalk",
    );
    check_eq(
        &mut errors,
        &chain.metric_observation_map,
        &["registry_schema_uri"],
        "https://luminousdynamics.io/schemas/landscape-performance-registry-v0.schema.json",
        "metric-observation crosswalk registry schema",
    );
    check_eq(
        &mut errors,
        &chain.metric_observation_map,
        &["registry_version"],
        "landscape-performance-registry-v0",
        "metric-observation crosswalk registry version",
    );
    if let Some(mappings) = records(&chain.metric_observation_map, &["mappings"]) {
        let registry_ids = registry_metric_ids(&chain.metric_registry);
        let mut mapped_metric_ids = BTreeSet::new();
        for mapping in mappings {
            if let Some(metric_id) = string_at(mapping, &["metric_id"]) {
                if !registry_ids.contains(metric_id) {
                    errors.push(format!("crosswalk references metric {metric_id} absent from the pinned metric registry"));
                }
                if !mapped_metric_ids.insert(metric_id) {
                    errors.push(format!(
                        "crosswalk contains duplicate metric mapping: {metric_id}"
                    ));
                }
            } else {
                errors.push("crosswalk mapping is missing metric_id".to_owned());
            }
        }
    } else {
        errors.push("metric-observation crosswalk requires mappings[]".to_owned());
    }
    errors.extend(validate_observation_crosswalk(chain));
    let crosswalk_matches = observation_crosswalk_match_count(chain);
    let crosswalk_eligible = eligible_supporting_observation_count(chain);
    if crosswalk_eligible != 0
        && chain.observations["observations"]
            .as_array()
            .is_some_and(|rows| {
                rows.iter().any(|row| {
                    matches!(
                        string_at(row, &["evidence_class"]),
                        Some("synthetic-test" | "synthetic-fixture")
                    )
                })
            })
    {
        errors.push(
            "synthetic observation unexpectedly counted as eligible metric support".to_owned(),
        );
    }
    if crosswalk_matches + unmapped_observation_count(chain) != observation_count {
        errors.push("crosswalk match census does not equal total observation count".to_owned());
    }

    check_eq(
        &mut errors,
        &chain.scenario,
        &["schema"],
        SCENARIO_SCHEMA,
        "scenario manifest",
    );
    check_eq(
        &mut errors,
        &chain.scenario,
        &["design_snapshot", "sha256"],
        &project_sha,
        "scenario snapshot reference",
    );
    check_eq(
        &mut errors,
        &chain.scenario,
        &["design_snapshot", "schema"],
        DESIGN_SCHEMA,
        "scenario snapshot schema",
    );
    check_eq(
        &mut errors,
        &chain.scenario,
        &["design_snapshot", "schema_uri"],
        DESIGN_SCHEMA_URI,
        "scenario snapshot schema URI",
    );
    if string_at(&chain.scenario, &["design_snapshot", "scope"])
        != string_at(&chain.project, &["scope"])
        || string_at(&chain.scenario, &["design_snapshot", "geometry_status"])
            != string_at(&chain.project, &["geometry_status"])
        || string_at(
            &chain.scenario,
            &["design_snapshot", "coordinate_frame_kind"],
        ) != Some(LOCAL_FRAME)
        || at(&chain.scenario, &["design_snapshot", "georeferenced"])
            != at(&chain.project, &["coordinate_frame", "georeferenced"])
        || at(&chain.scenario, &["design_snapshot", "feature_count"]).and_then(Value::as_u64)
            != Some(feature_count(&chain.project) as u64)
    {
        errors.push("scenario snapshot metadata diverges from exact project snapshot".to_owned());
    }
    check_eq(
        &mut errors,
        &chain.scenario,
        &["boundary_conditions_sha256"],
        &boundary_sha,
        "scenario boundary-condition reference",
    );
    check_eq(
        &mut errors,
        &chain.scenario,
        &["evaluation_plan", "sha256"],
        &evaluation_plan_sha,
        "scenario evaluation-plan reference",
    );
    if !matches!(
        string_at(&chain.scenario, &["purpose"]),
        Some("schema-qualification" | "synthetic-benchmark")
    ) {
        errors.push(
            "scenario purpose exceeds the v0 synthetic/schema-qualification claim ceiling"
                .to_owned(),
        );
    }
    require_digest(
        &mut errors,
        string_at(&chain.scenario, &["boundary_conditions_sha256"]),
        "scenario boundary-condition reference",
    );

    check_eq(
        &mut errors,
        &chain.alternative_scenario,
        &["schema"],
        SCENARIO_SCHEMA,
        "alternative scenario manifest",
    );
    check_eq(
        &mut errors,
        &chain.alternative_scenario,
        &["scenario_id"],
        "scenario-landscape-alternative-synthetic-v0",
        "alternative scenario identity",
    );
    check_eq(
        &mut errors,
        &chain.alternative_scenario,
        &["design_snapshot", "sha256"],
        &alternative_sha,
        "alternative scenario snapshot reference",
    );
    check_eq(
        &mut errors,
        &chain.alternative_scenario,
        &["design_snapshot", "schema"],
        DESIGN_SCHEMA,
        "alternative scenario snapshot schema",
    );
    check_eq(
        &mut errors,
        &chain.alternative_scenario,
        &["design_snapshot", "schema_uri"],
        DESIGN_SCHEMA_URI,
        "alternative scenario snapshot schema URI",
    );
    if string_at(&chain.alternative_scenario, &["design_snapshot", "scope"])
        != string_at(&chain.alternative, &["scope"])
        || string_at(
            &chain.alternative_scenario,
            &["design_snapshot", "geometry_status"],
        ) != string_at(&chain.alternative, &["geometry_status"])
        || string_at(
            &chain.alternative_scenario,
            &["design_snapshot", "coordinate_frame_kind"],
        ) != string_at(
            &chain.scenario,
            &["design_snapshot", "coordinate_frame_kind"],
        )
        || at(
            &chain.alternative_scenario,
            &["design_snapshot", "georeferenced"],
        ) != at(&chain.alternative, &["coordinate_frame", "georeferenced"])
        || at(
            &chain.alternative_scenario,
            &["design_snapshot", "feature_count"],
        )
        .and_then(Value::as_u64)
            != Some(feature_count(&chain.alternative) as u64)
    {
        errors.push(
            "alternative scenario metadata diverges from exact alternative snapshot".to_owned(),
        );
    }
    check_eq(
        &mut errors,
        &chain.alternative_scenario,
        &["boundary_conditions_sha256"],
        &boundary_sha,
        "alternative scenario boundary-condition reference",
    );
    check_eq(
        &mut errors,
        &chain.alternative_scenario,
        &["evaluation_plan", "sha256"],
        &evaluation_plan_sha,
        "alternative scenario evaluation-plan reference",
    );
    check_eq(
        &mut errors,
        &chain.alternative_scenario,
        &["evaluation_plan", "evaluation_id"],
        "eval-site-demo-v0",
        "alternative scenario evaluation identity",
    );
    if at(&chain.alternative_scenario, &["seed"]) != at(&chain.scenario, &["seed"])
        || string_at(&chain.alternative_scenario, &["model_id"])
            != string_at(&chain.scenario, &["model_id"])
        || string_at(&chain.alternative_scenario, &["model_version"])
            != string_at(&chain.scenario, &["model_version"])
        || string_at(&chain.alternative_scenario, &["purpose"])
            != string_at(&chain.scenario, &["purpose"])
        || string_at(&chain.alternative_scenario, &["boundary_conditions_sha256"])
            != string_at(&chain.scenario, &["boundary_conditions_sha256"])
        || string_at(&chain.alternative_scenario, &["evaluation_plan", "sha256"])
            != string_at(&chain.scenario, &["evaluation_plan", "sha256"])
        || string_at(&chain.alternative_scenario, &["scenario_id"])
            == string_at(&chain.scenario, &["scenario_id"])
    {
        errors.push(
            "paired scenarios must use distinct IDs but identical model/version/seed, forcing, and evaluation plan"
                .to_owned(),
        );
    }

    check_eq(
        &mut errors,
        &chain.stewardship,
        &["schema"],
        STEWARDSHIP_SCHEMA,
        "Mycelix stewardship profile",
    );
    check_eq(
        &mut errors,
        &chain.stewardship,
        &["project_snapshot", "sha256"],
        &project_sha,
        "Mycelix project reference",
    );
    if string_at(&chain.stewardship, &["project_snapshot", "scope"])
        != string_at(&chain.project, &["scope"])
        || string_at(
            &chain.stewardship,
            &["project_snapshot", "coordinate_frame_kind"],
        ) != Some(LOCAL_FRAME)
        || at(&chain.stewardship, &["project_snapshot", "georeferenced"])
            != at(&chain.project, &["coordinate_frame", "georeferenced"])
        || string_at(&chain.stewardship, &["project_snapshot", "geometry_status"])
            != string_at(&chain.project, &["geometry_status"])
    {
        errors.push("Mycelix project metadata diverges from the pinned design snapshot".to_owned());
    }
    check_eq(
        &mut errors,
        &chain.stewardship,
        &["record_type"],
        "scenario-review",
        "Mycelix record type",
    );
    check_eq(
        &mut errors,
        &chain.stewardship,
        &["evidence_class"],
        "synthetic",
        "Mycelix evidence class",
    );
    check_eq(
        &mut errors,
        &chain.stewardship,
        &["scenario_reference", "manifest_sha256"],
        &scenario_sha,
        "Mycelix scenario reference",
    );
    check_eq(
        &mut errors,
        &chain.stewardship,
        &["scenario_reference", "evidence_class"],
        "synthetic",
        "Mycelix scenario evidence class",
    );
    check_eq(
        &mut errors,
        &chain.stewardship,
        &["access_policy", "visibility"],
        "private",
        "Mycelix synthetic project visibility",
    );
    check_eq(
        &mut errors,
        &chain.stewardship,
        &["access_policy", "coordinate_disclosure"],
        "none",
        "Mycelix coordinate disclosure",
    );
    check_false(
        &mut errors,
        &chain.stewardship,
        &["access_policy", "sensitive_site_data_included"],
        "Mycelix privacy policy",
    );
    check_eq(
        &mut errors,
        &chain.stewardship,
        &["actor_authority", "publication_consent"],
        "not-requested",
        "Mycelix publication-consent state",
    );

    check_eq(
        &mut errors,
        &chain.review,
        &["schema"],
        REVIEW_SCHEMA,
        "Symthaea review",
    );
    check_eq(
        &mut errors,
        &chain.review,
        &["inputs", "baseline_design", "sha256"],
        &project_sha,
        "Symthaea baseline reference",
    );
    check_eq(
        &mut errors,
        &chain.review,
        &["inputs", "metric_registry_sha256"],
        &metric_registry_sha,
        "Symthaea metric-registry reference",
    );
    check_eq(
        &mut errors,
        &chain.review,
        &["inputs", "source_catalog_sha256"],
        &source_catalog_sha,
        "Symthaea source-catalog reference",
    );
    check_eq(
        &mut errors,
        &chain.review,
        &["inputs", "source_catalog_status"],
        "metadata-only-not-ingested",
        "Symthaea source-catalog readiness",
    );
    check_eq(
        &mut errors,
        &chain.source_catalog,
        &["catalog_status"],
        "metadata-only-not-ingested",
        "Sol-Atlas source-catalog readiness",
    );
    let review_alternatives = records(&chain.review, &["alternatives"]);
    match review_alternatives {
        Some(alternatives) if alternatives.len() >= 2 => {
            let digests: BTreeSet<&str> = alternatives
                .iter()
                .filter_map(|alternative| string_at(alternative, &["snapshot_sha256"]))
                .collect();
            if !digests.contains(project_sha.as_str())
                || !digests.contains(alternative_sha.as_str())
            {
                errors.push(
                    "Symthaea review alternatives do not bind both exact design snapshots"
                        .to_owned(),
                );
            }
            for alternative in alternatives {
                if string_at(alternative, &["scope"]) != string_at(&chain.project, &["scope"])
                    || string_at(alternative, &["coordinate_frame_kind"]) != Some(LOCAL_FRAME)
                    || string_at(alternative, &["geometry_status"])
                        != Some("schematic-design-intent")
                {
                    errors.push(
                        "Symthaea alternative scope/frame/geometry metadata is inconsistent"
                            .to_owned(),
                    );
                    break;
                }
            }
        }
        _ => errors.push("Symthaea review must contain at least two alternatives".to_owned()),
    }
    let catalog_ids: BTreeSet<&str> = records(&chain.source_catalog, &["records"])
        .into_iter()
        .flatten()
        .filter_map(|record| string_at(record, &["id"]))
        .collect();
    match records(&chain.review, &["candidate_sources"]) {
        Some(candidates) => {
            for candidate in candidates {
                if let Some(source_id) = string_at(candidate, &["source_id"]) {
                    if !catalog_ids.contains(source_id) {
                        errors.push(format!(
                            "Symthaea review references unknown source catalog ID {source_id}"
                        ));
                    }
                } else {
                    errors.push("Symthaea candidate source is missing source_id".to_owned());
                }
            }
        }
        None => errors.push("Symthaea review requires candidate_sources[]".to_owned()),
    }
    if records(&chain.review, &["metric_assessments"]).is_none_or(|metrics| {
        metrics.is_empty()
            || metrics
                .iter()
                .any(|metric| string_at(metric, &["status"]) != Some("blocked"))
    }) {
        errors.push("all consequence-bearing metrics must remain blocked until real qualified inputs/models exist".to_owned());
    }
    let scenario_runs = records(&chain.review, &["inputs", "scenario_runs"]);
    match scenario_runs {
        Some(runs) if runs.len() == 2 => {
            let actual_manifests: BTreeSet<&str> = runs
                .iter()
                .filter_map(|run| string_at(run, &["manifest_sha256"]))
                .collect();
            let expected_manifests: BTreeSet<&str> =
                [scenario_sha.as_str(), alternative_scenario_sha.as_str()]
                    .into_iter()
                    .collect();
            if actual_manifests != expected_manifests {
                errors.push(
                    "Symthaea review must pin both the exact baseline and alternative scenario manifests"
                        .to_owned(),
                );
            }
            for run in runs {
                check_eq(
                    &mut errors,
                    run,
                    &["status"],
                    "blocked",
                    "Symthaea scenario qualification status",
                );
                check_eq(
                    &mut errors,
                    run,
                    &["evidence_class"],
                    "synthetic",
                    "Symthaea scenario evidence class",
                );
                if string_at(run, &["model_id"]) != string_at(&chain.scenario, &["model_id"])
                    || string_at(run, &["model_version"])
                        != string_at(&chain.scenario, &["model_version"])
                    || string_at(run, &["purpose"]) != string_at(&chain.scenario, &["purpose"])
                {
                    errors.push(
                        "paired Symthaea scenario records disagree with the shared model/version/purpose"
                            .to_owned(),
                    );
                }

                let is_baseline_scenario =
                    string_at(run, &["manifest_sha256"]) == Some(scenario_sha.as_str());
                let alternative_index = if is_baseline_scenario { 0 } else { 1 };
                let expected_alternative_id = string_at(
                    &chain.review["alternatives"][alternative_index],
                    &["alternative_id"],
                )
                .unwrap_or("");
                let expected_design_sha = if is_baseline_scenario {
                    project_sha.as_str()
                } else {
                    alternative_sha.as_str()
                };
                match run.get("pairing_context") {
                    Some(context) if context.is_object() => {
                        check_eq(
                            &mut errors,
                            context,
                            &["alternative_id"],
                            expected_alternative_id,
                            "Symthaea paired-run alternative identity",
                        );
                        check_eq(
                            &mut errors,
                            context,
                            &["design_snapshot_sha256"],
                            expected_design_sha,
                            "Symthaea paired-run design snapshot",
                        );
                        check_eq(
                            &mut errors,
                            context,
                            &["boundary_conditions_sha256"],
                            &boundary_sha,
                            "Symthaea paired-run boundary conditions",
                        );
                        check_eq(
                            &mut errors,
                            context,
                            &["evaluation_plan_sha256"],
                            &evaluation_plan_sha,
                            "Symthaea paired-run evaluation plan",
                        );
                        check_eq(
                            &mut errors,
                            context,
                            &["metric_registry_sha256"],
                            &metric_registry_sha,
                            "Symthaea paired-run metric registry",
                        );
                        check_eq(
                            &mut errors,
                            context,
                            &["scope"],
                            string_at(&chain.project, &["scope"]).unwrap_or(""),
                            "Symthaea paired-run scope",
                        );
                        if at(context, &["seed"]) != at(&chain.scenario, &["seed"]) {
                            errors.push(
                                "Symthaea paired-run seed differs from the pinned scenario seed"
                                    .to_owned(),
                            );
                        }
                    }
                    _ => errors.push(
                        "Symthaea scenario run requires a complete pairing_context".to_owned(),
                    ),
                }
                if at(run, &["result_sha256"]).is_some() {
                    errors.push(
                        "blocked scenario run must not claim a qualified result digest".to_owned(),
                    );
                }
            }
        }
        _ => errors.push(
            "Symthaea review must retain exactly two blocked, pinned scenario records".to_owned(),
        ),
    }
    check_eq(
        &mut errors,
        &chain.review,
        &["comparison", "ranking_status"],
        "no-defensible-ranking",
        "Symthaea ranking status",
    );
    check_eq(
        &mut errors,
        &chain.review,
        &["recommendation", "status"],
        "no-ranking",
        "Symthaea recommendation status",
    );
    check_true(
        &mut errors,
        &chain.review,
        &["authority", "read_only"],
        "Symthaea authority",
    );
    check_false(
        &mut errors,
        &chain.review,
        &["authority", "mutates_project"],
        "Symthaea authority",
    );
    check_false(
        &mut errors,
        &chain.review,
        &["authority", "actuation_authority"],
        "Symthaea authority",
    );

    if string_at(&chain.metric_registry, &["status"]) != Some("specification-only-no-field-values")
    {
        errors.push(
            "metric registry must remain definitions-only with no fabricated field values"
                .to_owned(),
        );
    }
    let upstream_contracts = records(&chain.upstream_manifest, &["source_contracts"]);
    match upstream_contracts {
        Some(rows) if rows.len() >= 6 => {
            for row in rows {
                let commit = string_at(row, &["commit_sha"]);
                if !commit.is_some_and(|sha| {
                    sha.len() == 40 && sha.chars().all(|ch| ch.is_ascii_hexdigit())
                }) {
                    errors.push(
                        "upstream source manifest contains an unpinned draft commit".to_owned(),
                    );
                }
            }
        }
        _ => errors.push(
            "upstream source manifest must pin at least six upstream contract revisions".to_owned(),
        ),
    }

    // These variables intentionally force all evidence roots to have well-formed exact hashes.
    if !digest_is_well_formed(&metric_observation_map_sha) {
        errors.push("metric-observation crosswalk: invalid generated digest".to_owned());
    }

    for (label, digest) in [
        ("project snapshot", project_sha),
        ("alternative snapshot", alternative_sha),
        ("boundary conditions", boundary_sha),
        ("intervention site", intervention_site_sha),
        ("comparison site", comparison_site_sha),
        ("metric registry", metric_registry_sha),
        ("source catalog", source_catalog_sha),
        ("evaluation plan", evaluation_plan_sha),
        ("observation batch", observations_sha),
        ("scenario manifest", scenario_sha),
        ("alternative scenario manifest", alternative_scenario_sha),
    ] {
        if !digest_is_well_formed(&digest) {
            errors.push(format!("{label}: invalid generated digest"));
        }
    }

    errors.sort();
    errors
}

fn negative_control_results(chain: &ChainArtifacts) -> NegativeControlSummary {
    let mut cases: Vec<(String, ChainArtifacts)> = Vec::new();

    let mut stale_observation = chain.clone();
    stale_observation.observations["observations"][0]["project_ref"]["project_snapshot_sha256"] =
        json!("f".repeat(64));
    cases.push((
        "stale-observation-project-digest".to_owned(),
        stale_observation,
    ));

    let mut wrong_scope = chain.clone();
    wrong_scope.evaluation_plan["project_scope"] = json!("watershed");
    cases.push(("evaluation-scope-mismatch".to_owned(), wrong_scope));

    let mut missing_feature = chain.clone();
    missing_feature.observations["observations"][0]["project_ref"]["feature_id"] =
        json!("feature-does-not-exist");
    cases.push(("unknown-observation-feature".to_owned(), missing_feature));

    let mut relabelled = chain.clone();
    relabelled.observations["observations"][0]["evidence_class"] = json!("observed");
    cases.push(("synthetic-to-observed-relabel".to_owned(), relabelled));

    let mut privacy_leak = chain.clone();
    privacy_leak.stewardship["access_policy"]["visibility"] = json!("public");
    privacy_leak.stewardship["access_policy"]["coordinate_disclosure"] = json!("exact");
    privacy_leak.stewardship["access_policy"]["sensitive_site_data_included"] = json!(true);
    cases.push(("public-exact-sensitive-location".to_owned(), privacy_leak));

    let mut scenario_disagreement = chain.clone();
    scenario_disagreement.review["inputs"]["scenario_runs"][0]["manifest_sha256"] =
        json!("e".repeat(64));
    cases.push((
        "scenario-review-digest-conflict".to_owned(),
        scenario_disagreement,
    ));

    let mut paired_review_context_mismatch = chain.clone();
    paired_review_context_mismatch.review["inputs"]["scenario_runs"][1]["pairing_context"]["seed"] =
        json!(8);
    cases.push((
        "paired-review-seed-mismatch".to_owned(),
        paired_review_context_mismatch,
    ));

    let mut paired_forcing_mismatch = chain.clone();
    paired_forcing_mismatch.alternative_scenario["boundary_conditions_sha256"] =
        json!("d".repeat(64));
    cases.push((
        "paired-scenario-forcing-mismatch".to_owned(),
        paired_forcing_mismatch,
    ));

    let mut crosswalk_unit_mismatch = chain.clone();
    crosswalk_unit_mismatch.observations["observations"][1]["observed_property"]["unit"] =
        json!("mm");
    cases.push((
        "observation-crosswalk-unit-mismatch".to_owned(),
        crosswalk_unit_mismatch,
    ));

    let mut changed_snapshot_bytes = chain.clone();
    if let Some(position) = changed_snapshot_bytes
        .project_bytes
        .windows(b"Canopy tree".len())
        .position(|window| window == b"Canopy tree")
    {
        changed_snapshot_bytes.project_bytes[position] = b'c';
    }
    cases.push((
        "one-byte-project-snapshot-change".to_owned(),
        changed_snapshot_bytes,
    ));

    let mut result = NegativeControlSummary {
        attempted: cases.len(),
        rejected: 0,
        unexpectedly_accepted: 0,
        cases: Vec::new(),
    };
    for (name, changed) in cases {
        result.cases.push(name);
        if validate_chain(&changed).is_empty() {
            result.unexpectedly_accepted += 1;
        } else {
            result.rejected += 1;
        }
    }
    result
}

pub fn run_conformance(dir: &Path) -> Result<ConformanceReport, Vec<String>> {
    let chain = build_chain(dir)?;
    let errors = validate_chain(&chain);
    if !errors.is_empty() {
        return Err(errors);
    }

    let negative_controls = negative_control_results(&chain);
    if negative_controls.unexpectedly_accepted != 0 {
        return Err(vec![format!(
            "{} of {} authored negative controls were unexpectedly accepted",
            negative_controls.unexpectedly_accepted, negative_controls.attempted
        )]);
    }

    let mut artifact_sha256 = BTreeMap::new();
    artifact_sha256.insert(
        "project_snapshot".to_owned(),
        sha256_hex(&chain.project_bytes),
    );
    artifact_sha256.insert(
        "alternative_snapshot".to_owned(),
        sha256_hex(&chain.alternative_bytes),
    );
    artifact_sha256.insert(
        "intervention_site_reference".to_owned(),
        sha256_hex(&chain.intervention_site_bytes),
    );
    artifact_sha256.insert(
        "comparison_site_reference".to_owned(),
        sha256_hex(&chain.comparison_site_bytes),
    );
    artifact_sha256.insert(
        "boundary_conditions".to_owned(),
        sha256_hex(&chain.boundary_bytes),
    );
    artifact_sha256.insert(
        "metric_registry".to_owned(),
        sha256_hex(&chain.metric_registry_bytes),
    );
    artifact_sha256.insert(
        "metric_observation_map".to_owned(),
        sha256_hex(&chain.metric_observation_map_bytes),
    );
    artifact_sha256.insert(
        "source_catalog".to_owned(),
        sha256_hex(&chain.source_catalog_bytes),
    );
    artifact_sha256.insert(
        "evaluation_plan".to_owned(),
        canonical_sha256(&chain.evaluation_plan).map_err(|error| vec![error])?,
    );
    artifact_sha256.insert(
        "observation_batch".to_owned(),
        canonical_sha256(&chain.observations).map_err(|error| vec![error])?,
    );
    artifact_sha256.insert(
        "scenario_manifest".to_owned(),
        canonical_sha256(&chain.scenario).map_err(|error| vec![error])?,
    );
    artifact_sha256.insert(
        "alternative_scenario_manifest".to_owned(),
        canonical_sha256(&chain.alternative_scenario).map_err(|error| vec![error])?,
    );
    artifact_sha256.insert(
        "stewardship_record".to_owned(),
        canonical_sha256(&chain.stewardship).map_err(|error| vec![error])?,
    );
    artifact_sha256.insert(
        "alternative_review".to_owned(),
        canonical_sha256(&chain.review).map_err(|error| vec![error])?,
    );
    artifact_sha256.insert(
        "upstream_contract_manifest".to_owned(),
        sha256_hex(&chain.upstream_manifest_bytes),
    );

    let mut template_input_sha256 = chain.template_digests.clone();
    template_input_sha256.insert(
        "metric-registry.json".to_owned(),
        sha256_hex(&chain.metric_registry_bytes),
    );
    template_input_sha256.insert(
        "source-catalog.json".to_owned(),
        sha256_hex(&chain.source_catalog_bytes),
    );

    let checks = vec![
        "exact design and alternative snapshot bytes are separately digested".to_owned(),
        "evaluation plan binds project snapshot, scope, and exact intervention/comparison site references".to_owned(),
        "every observation binds the project digest and an existing feature identifier".to_owned(),
        "observation evidence remains synthetic and cannot be relabelled as observed".to_owned(),
        "scenario manifest binds design, boundary conditions, and exact normalized evaluation-plan bytes".to_owned(),
        "baseline and alternative scenarios use the same model/version/seed, forcing bytes, and evaluation plan".to_owned(),
        "Mycelix stewardship binds the same project and baseline scenario, with private and non-sensitive defaults".to_owned(),
        "Symthaea review binds both alternatives, metric registry, source catalog, and the paired scenario contexts".to_owned(),
        "unqualified source catalog and metrics keep consequence-bearing assessments blocked".to_owned(),
        "observation properties are checked against the declared metric crosswalk and evidence class".to_owned(),
        "review correctly abstains from ranking while scenario execution remains blocked".to_owned(),
        "all upstream contract inputs cite immutable draft commit SHAs".to_owned(),
        "observation crosswalk metric IDs resolve against the pinned registry, while synthetic observations remain ineligible".to_owned(),
        "ten adversarial cross-reference/privacy/crosswalk/paired-context/claim controls are rejected".to_owned(),
        "report emits exact SHA-256 digests for every artifact in the synthetic chain".to_owned(),
    ];

    Ok(ConformanceReport {
        schema_version: REPORT_SCHEMA.to_owned(),
        suite_id: SUITE_ID.to_owned(),
        conformance_status: "PASS_SYNTHETIC_CONTRACT_CONFORMANCE_ONLY".to_owned(),
        data_origin: "synthetic_fixture".to_owned(),
        claim_ceiling: CLAIM_CEILING.to_owned(),
        upstream_pinned_contract_count: records(&chain.upstream_manifest, &["source_contracts"])
            .map_or(0, Vec::len),
        project_feature_count: feature_count(&chain.project),
        observation_count: records(&chain.observations, &["observations"]).map_or(0, Vec::len),
        metric_definition_count: records(&chain.metric_registry, &["metrics"]).map_or(0, Vec::len),
        metric_observation_mapping_count: records(&chain.metric_observation_map, &["mappings"])
            .map_or(0, Vec::len),
        observations_matching_crosswalk_count: observation_crosswalk_match_count(&chain),
        eligible_supporting_observation_count: eligible_supporting_observation_count(&chain),
        unmapped_observation_count: unmapped_observation_count(&chain),
        source_catalog_record_count: records(&chain.source_catalog, &["records"])
            .map_or(0, Vec::len),
        source_catalog_status: string_at(&chain.source_catalog, &["catalog_status"])
            .unwrap_or("unknown")
            .to_owned(),
        scenario_execution_status: "BLOCKED_NOT_EXECUTED".to_owned(),
        paired_scenario_count: records(&chain.review, &["inputs", "scenario_runs"])
            .map_or(0, Vec::len),
        paired_scenarios_share_boundary_conditions: string_at(
            &chain.scenario,
            &["boundary_conditions_sha256"],
        ) == string_at(
            &chain.alternative_scenario,
            &["boundary_conditions_sha256"],
        ),
        review_ranking_status: string_at(&chain.review, &["comparison", "ranking_status"])
            .unwrap_or("unknown")
            .to_owned(),
        recommendation_status: string_at(&chain.review, &["recommendation", "status"])
            .unwrap_or("unknown")
            .to_owned(),
        passed_check_count: checks.len(),
        checks,
        negative_controls_attempted: negative_controls.attempted,
        negative_controls_rejected: negative_controls.rejected,
        negative_controls_unexpectedly_accepted: negative_controls.unexpectedly_accepted,
        artifact_sha256,
        template_input_sha256,
    })
}

pub fn default_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/landscape/chain-conformance-v0")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ChainArtifacts {
        build_chain(&default_fixture_dir()).expect("frozen chain fixture builds")
    }

    #[test]
    fn linked_fixture_passes_without_claiming_physical_performance() {
        let chain = fixture();
        assert!(validate_chain(&chain).is_empty());
        let report = run_conformance(&default_fixture_dir()).expect("report passes");
        assert_eq!(
            report.conformance_status,
            "PASS_SYNTHETIC_CONTRACT_CONFORMANCE_ONLY"
        );
        assert_eq!(report.data_origin, "synthetic_fixture");
        assert_eq!(report.claim_ceiling, CLAIM_CEILING);
        assert_eq!(report.upstream_pinned_contract_count, 10);
        assert_eq!(report.project_feature_count, 7);
        assert_eq!(report.observation_count, 5);
        assert_eq!(report.metric_observation_mapping_count, 8);
        assert_eq!(report.observations_matching_crosswalk_count, 1);
        assert_eq!(report.eligible_supporting_observation_count, 0);
        assert_eq!(report.unmapped_observation_count, 4);
        assert_eq!(report.negative_controls_attempted, 10);
        assert_eq!(report.negative_controls_rejected, 10);
        assert_eq!(report.negative_controls_unexpectedly_accepted, 0);
        assert_eq!(report.source_catalog_status, "metadata-only-not-ingested");
        assert_eq!(report.scenario_execution_status, "BLOCKED_NOT_EXECUTED");
        assert_eq!(report.paired_scenario_count, 2);
        assert!(report.paired_scenarios_share_boundary_conditions);
        assert_eq!(report.review_ranking_status, "no-defensible-ranking");
        assert_eq!(report.recommendation_status, "no-ranking");
        assert_eq!(report.passed_check_count, report.checks.len());
        assert_eq!(report.passed_check_count, 15);
        assert_eq!(report.artifact_sha256.len(), 15);
        assert_eq!(report.template_input_sha256.len(), 9);
        assert!(
            report
                .artifact_sha256
                .values()
                .all(|value| digest_is_well_formed(value))
        );
    }

    #[test]
    fn paired_scenario_forcing_mismatch_is_rejected() {
        let mut chain = fixture();
        chain.alternative_scenario["boundary_conditions_sha256"] = json!("d".repeat(64));
        assert!(!validate_chain(&chain).is_empty());
    }

    #[test]
    fn observation_crosswalk_unit_mismatch_is_rejected() {
        let mut chain = fixture();
        chain.observations["observations"][1]["observed_property"]["unit"] = json!("mm");
        assert!(
            validate_chain(&chain)
                .iter()
                .any(|error| error.contains("units do not match crosswalk property"))
        );
    }

    #[test]
    fn report_is_byte_deterministic_for_identical_fixture_inputs() {
        let first = run_conformance(&default_fixture_dir()).expect("first report");
        let second = run_conformance(&default_fixture_dir()).expect("second report");
        assert_eq!(
            serde_json::to_vec(&first).expect("serialize first"),
            serde_json::to_vec(&second).expect("serialize second")
        );
    }

    #[test]
    fn one_byte_change_to_snapshot_invalidates_the_chain() {
        let mut chain = fixture();
        let position = chain
            .project_bytes
            .windows(b"Canopy tree".len())
            .position(|window| window == b"Canopy tree")
            .expect("fixture includes known label");
        chain.project_bytes[position] = b'c';
        let errors = validate_chain(&chain);
        assert!(!errors.is_empty());
        assert!(
            errors
                .iter()
                .any(|error| error.contains("exact source bytes"))
        );
    }

    #[test]
    fn stale_observation_digest_is_rejected() {
        let mut chain = fixture();
        chain.observations["observations"][0]["project_ref"]["project_snapshot_sha256"] =
            json!("f".repeat(64));
        assert!(!validate_chain(&chain).is_empty());
    }

    #[test]
    fn scope_mismatch_is_rejected() {
        let mut chain = fixture();
        chain.evaluation_plan["project_scope"] = json!("watershed");
        assert!(!validate_chain(&chain).is_empty());
    }

    #[test]
    fn nonexistent_feature_reference_is_rejected() {
        let mut chain = fixture();
        chain.observations["observations"][0]["project_ref"]["feature_id"] =
            json!("feature-does-not-exist");
        assert!(!validate_chain(&chain).is_empty());
    }

    #[test]
    fn synthetic_observation_cannot_be_relabelled_as_observed() {
        let mut chain = fixture();
        chain.observations["observations"][0]["evidence_class"] = json!("observed");
        assert!(!validate_chain(&chain).is_empty());
    }

    #[test]
    fn public_exact_sensitive_location_is_rejected() {
        let mut chain = fixture();
        chain.stewardship["access_policy"]["visibility"] = json!("public");
        chain.stewardship["access_policy"]["coordinate_disclosure"] = json!("exact");
        chain.stewardship["access_policy"]["sensitive_site_data_included"] = json!(true);
        assert!(!validate_chain(&chain).is_empty());
    }

    #[test]
    fn conflicting_scenario_digest_is_rejected() {
        let mut chain = fixture();
        chain.review["inputs"]["scenario_runs"][0]["manifest_sha256"] = json!("e".repeat(64));
        assert!(!validate_chain(&chain).is_empty());
    }
}
