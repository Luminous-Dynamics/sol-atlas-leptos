// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Validation boundary for the native landscape-design v0 snapshot.
//!
//! This validates document structure and the current schematic-symbol contract.
//! It does not validate surveyed geometry, geospatial reference systems, ecology,
//! hydrology, engineering compliance, or predicted landscape outcomes.

use serde_json::Value;
use std::collections::HashSet;

pub const SNAPSHOT_SCHEMA: &str = "luminous.landscape-design.v0";
pub const SNAPSHOT_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-design-v0.schema.json";
pub const FRAME_KIND: &str = "LOCAL_CARTESIAN_DRAWING_UNITS";
pub const GEOMETRY_STATUS: &str = "schematic-design-intent";
pub const SYNTHETIC_GEOMETRY_STATUS: &str = "synthetic-benchmark";
pub const DESIGN_INTENT_STATUS: &str = "design-intent";
pub const USER_AUTHORED_SOURCE: &str = "user-authored-schematic";
pub const SYNTHETIC_SOURCE: &str = "synthetic-benchmark";
pub const GEOMETRY_REPRESENTATION: &str = "schematic-symbol-anchor-v0";
pub const MAX_SNAPSHOT_FEATURES: usize = 100_000;

const ROOT_KEYS: &[&str] = &[
    "schema",
    "schema_uri",
    "project_name",
    "scope",
    "coordinate_frame",
    "geometry_status",
    "features",
];
const FRAME_KEYS: &[&str] = &["kind", "origin", "x_axis", "y_axis", "georeferenced"];
const FEATURE_KEYS: &[&str] = &[
    "id",
    "kind",
    "label",
    "x_du",
    "y_du",
    "status",
    "source",
    "geometry_representation",
];

/// Create a deterministic synthetic scene using the LCG algorithm pinned in
/// benchmarks/landscape/workloads-v0.json. The output is benchmark data, not a
/// user-authored proposal or a geospatial/physical model.
pub fn synthetic_benchmark_snapshot(feature_count: usize, seed: u32) -> Result<Value, String> {
    const KINDS: [(&str, &str); 5] = [
        ("tree", "Canopy tree"),
        ("planting-bed", "Planting bed"),
        ("water", "Water feature"),
        ("path", "Path"),
        ("built-element", "Built element"),
    ];

    if feature_count > MAX_SNAPSHOT_FEATURES {
        return Err(format!(
            "benchmark feature count {feature_count} exceeds the v0 limit {MAX_SNAPSHOT_FEATURES}"
        ));
    }

    let mut state = seed;
    let mut next_u32 = || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        state
    };

    let features = (0..feature_count)
        .map(|index| {
            let kind_index = (next_u32() % KINDS.len() as u32) as usize;
            let x_du = next_u32() % 1001;
            let y_du = next_u32() % 651;
            let (kind, label) = KINDS[kind_index];

            serde_json::json!({
                "id": format!("feature-{:04}", index + 1),
                "kind": kind,
                "label": label,
                "x_du": x_du,
                "y_du": y_du,
                "status": SYNTHETIC_GEOMETRY_STATUS,
                "source": SYNTHETIC_SOURCE,
                "geometry_representation": GEOMETRY_REPRESENTATION
            })
        })
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "schema": SNAPSHOT_SCHEMA,
        "schema_uri": SNAPSHOT_SCHEMA_URI,
        "project_name": format!("Synthetic benchmark ({feature_count} features)"),
        "scope": "site-parcel",
        "coordinate_frame": {
            "kind": FRAME_KIND,
            "origin": "canvas top-left",
            "x_axis": "right",
            "y_axis": "down",
            "georeferenced": false
        },
        "geometry_status": SYNTHETIC_GEOMETRY_STATUS,
        "features": features
    }))
}

/// Return every structural violation found in a native v0 project snapshot.
///
/// An invalid or unsupported snapshot must never be treated as an empty, valid
/// project. Callers should display the returned errors and stop export/import.
pub fn validate_snapshot(value: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let Some(root) = value.as_object() else {
        return Err(vec!["$: expected a JSON object".to_string()]);
    };

    validate_keys(root.keys().map(String::as_str), ROOT_KEYS, "$", &mut errors);
    require_string_eq(value, "schema", SNAPSHOT_SCHEMA, "$", &mut errors);
    require_string_eq(value, "schema_uri", SNAPSHOT_SCHEMA_URI, "$", &mut errors);
    let geometry_status = value.get("geometry_status").and_then(Value::as_str);
    match geometry_status {
        Some(GEOMETRY_STATUS | SYNTHETIC_GEOMETRY_STATUS) => {}
        Some(_) => errors.push(format!(
            "$.geometry_status: expected {GEOMETRY_STATUS:?} or {SYNTHETIC_GEOMETRY_STATUS:?}"
        )),
        None => errors.push("$.geometry_status: expected a string".to_string()),
    }

    match value.get("project_name").and_then(Value::as_str) {
        Some(name) if !name.trim().is_empty() && name.chars().count() <= 200 => {}
        Some(_) => errors.push(
            "$.project_name: expected a non-empty name of at most 200 Unicode scalar values"
                .to_string(),
        ),
        None => errors.push("$.project_name: expected a string".to_string()),
    }

    match value.get("scope").and_then(Value::as_str) {
        Some("site-parcel" | "district" | "watershed" | "bioregion") => {}
        Some(_) => errors.push(
            "$.scope: unsupported scope (expected site-parcel, district, watershed, or bioregion)"
                .to_string(),
        ),
        None => errors.push("$.scope: expected a string".to_string()),
    }

    if let Some(frame) = value.get("coordinate_frame") {
        if let Some(object) = frame.as_object() {
            validate_keys(
                object.keys().map(String::as_str),
                FRAME_KEYS,
                "$.coordinate_frame",
                &mut errors,
            );
            require_string_eq(frame, "kind", FRAME_KIND, "$.coordinate_frame", &mut errors);
            require_string_eq(
                frame,
                "origin",
                "canvas top-left",
                "$.coordinate_frame",
                &mut errors,
            );
            require_string_eq(frame, "x_axis", "right", "$.coordinate_frame", &mut errors);
            require_string_eq(frame, "y_axis", "down", "$.coordinate_frame", &mut errors);
            if frame.get("georeferenced") != Some(&Value::Bool(false)) {
                errors.push(
                    "$.coordinate_frame.georeferenced: v0 schematic snapshots must be false"
                        .to_string(),
                );
            }
        } else {
            errors.push("$.coordinate_frame: expected an object".to_string());
        }
    } else {
        errors.push("$.coordinate_frame: missing required object".to_string());
    }

    match value.get("features").and_then(Value::as_array) {
        Some(features) if features.len() > MAX_SNAPSHOT_FEATURES => {
            errors.push(format!(
                "$.features: feature count {} exceeds the v0 limit {MAX_SNAPSHOT_FEATURES}",
                features.len()
            ));
        }
        Some(features) => {
            let mut ids = HashSet::with_capacity(features.len());
            for (index, feature) in features.iter().enumerate() {
                validate_feature(
                    feature,
                    index,
                    geometry_status.unwrap_or(""),
                    &mut ids,
                    &mut errors,
                );
            }
        }
        None => errors.push("$.features: expected an array".to_string()),
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_feature(
    feature: &Value,
    index: usize,
    document_status: &str,
    ids: &mut HashSet<String>,
    errors: &mut Vec<String>,
) {
    let path = format!("$.features[{index}]");
    let Some(object) = feature.as_object() else {
        errors.push(format!("{path}: expected an object"));
        return;
    };
    validate_keys(
        object.keys().map(String::as_str),
        FEATURE_KEYS,
        &path,
        errors,
    );

    let id = feature.get("id").and_then(Value::as_str);
    match id {
        Some(id) if valid_feature_id(id) => {
            if !ids.insert(id.to_string()) {
                errors.push(format!("{path}.id: duplicate feature ID {id}"));
            }
        }
        Some(_) => errors.push(format!(
            "{path}.id: expected feature-NNNN with at least four decimal digits"
        )),
        None => errors.push(format!("{path}.id: expected a string")),
    }

    let kind = feature.get("kind").and_then(Value::as_str);
    let expected_label = match kind {
        Some("tree") => Some("Canopy tree"),
        Some("planting-bed") => Some("Planting bed"),
        Some("water") => Some("Water feature"),
        Some("path") => Some("Path"),
        Some("built-element") => Some("Built element"),
        Some(_) => {
            errors.push(format!("{path}.kind: unsupported feature kind"));
            None
        }
        None => {
            errors.push(format!("{path}.kind: expected a string"));
            None
        }
    };

    if let Some(expected) = expected_label {
        if feature.get("label").and_then(Value::as_str) != Some(expected) {
            errors.push(format!(
                "{path}.label: label must match kind {kind:?} ({expected:?})"
            ));
        }
    }

    validate_integer_range(feature, "x_du", 0, 1000, &path, errors);
    validate_integer_range(feature, "y_du", 0, 650, &path, errors);
    let feature_status = feature.get("status").and_then(Value::as_str);
    let feature_source = feature.get("source").and_then(Value::as_str);
    let provenance_matches_document = match document_status {
        GEOMETRY_STATUS => {
            feature_status == Some(DESIGN_INTENT_STATUS)
                && feature_source == Some(USER_AUTHORED_SOURCE)
        }
        SYNTHETIC_GEOMETRY_STATUS => {
            feature_status == Some(SYNTHETIC_GEOMETRY_STATUS)
                && feature_source == Some(SYNTHETIC_SOURCE)
        }
        _ => false,
    };
    if !provenance_matches_document {
        errors.push(format!(
            "{path}: feature status/source must agree with document geometry_status {document_status:?}"
        ));
    }
    require_string_eq(
        feature,
        "geometry_representation",
        GEOMETRY_REPRESENTATION,
        &path,
        errors,
    );
}

fn valid_feature_id(id: &str) -> bool {
    let Some(digits) = id.strip_prefix("feature-") else {
        return false;
    };
    digits.len() >= 4
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && digits.parse::<u32>().is_ok_and(|value| value < u32::MAX)
}

fn validate_integer_range(
    object: &Value,
    key: &str,
    minimum: u64,
    maximum: u64,
    path: &str,
    errors: &mut Vec<String>,
) {
    match object.get(key).and_then(Value::as_u64) {
        Some(value) if (minimum..=maximum).contains(&value) => {}
        Some(_) => errors.push(format!(
            "{path}.{key}: expected an integer in {minimum}..={maximum}"
        )),
        None => errors.push(format!("{path}.{key}: expected a non-negative integer")),
    }
}

fn require_string_eq(
    object: &Value,
    key: &str,
    expected: &str,
    path: &str,
    errors: &mut Vec<String>,
) {
    if object.get(key).and_then(Value::as_str) != Some(expected) {
        errors.push(format!("{path}.{key}: expected {expected:?}"));
    }
}

fn validate_keys<'a>(
    actual: impl Iterator<Item = &'a str>,
    required_and_allowed: &[&str],
    path: &str,
    errors: &mut Vec<String>,
) {
    let actual: HashSet<&str> = actual.collect();
    for key in required_and_allowed {
        if !actual.contains(key) {
            errors.push(format!("{path}.{key}: missing required field"));
        }
    }
    for key in actual {
        if !required_and_allowed.contains(&key) {
            errors.push(format!("{path}.{key}: unknown field"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        serde_json::from_str(include_str!("../fixtures/landscape/seed-landscape-v0.json"))
            .expect("committed landscape fixture must be valid JSON")
    }

    #[test]
    fn schema_file_contract_matches_rust_constants() {
        let schema: Value =
            serde_json::from_str(include_str!("../schemas/landscape-design-v0.schema.json"))
                .expect("committed snapshot schema must be valid JSON");

        assert_eq!(schema["$id"], SNAPSHOT_SCHEMA_URI);
        assert_eq!(schema["properties"]["schema"]["const"], SNAPSHOT_SCHEMA);
        assert_eq!(
            schema["properties"]["schema_uri"]["const"],
            SNAPSHOT_SCHEMA_URI
        );
        assert_eq!(
            schema["properties"]["coordinate_frame"]["properties"]["kind"]["const"],
            FRAME_KIND
        );
        let geometry_statuses = schema["properties"]["geometry_status"]["enum"]
            .as_array()
            .unwrap();
        assert!(
            geometry_statuses
                .iter()
                .any(|value| value.as_str() == Some(GEOMETRY_STATUS))
        );
        assert!(
            geometry_statuses
                .iter()
                .any(|value| value.as_str() == Some(SYNTHETIC_GEOMETRY_STATUS))
        );
        let feature_properties = &schema["properties"]["features"]["items"]["properties"];
        assert!(
            feature_properties["status"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some(DESIGN_INTENT_STATUS))
        );
        assert!(
            feature_properties["status"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some(SYNTHETIC_GEOMETRY_STATUS))
        );
        assert!(
            feature_properties["source"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some(USER_AUTHORED_SOURCE))
        );
        assert!(
            feature_properties["source"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some(SYNTHETIC_SOURCE))
        );
        assert_eq!(
            schema["properties"]["features"]["items"]["properties"]["geometry_representation"]["const"],
            GEOMETRY_REPRESENTATION
        );
        for required in ROOT_KEYS {
            assert!(
                schema["required"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|entry| entry.as_str() == Some(*required))
            );
        }
    }

    #[test]
    fn committed_v0_fixture_passes_document_validator() {
        assert_eq!(validate_snapshot(&fixture()), Ok(()));
    }

    #[test]
    fn synthetic_benchmark_generation_is_deterministic_and_valid() {
        let first = synthetic_benchmark_snapshot(100, 0x4c41_4e44).unwrap();
        let repeat = synthetic_benchmark_snapshot(100, 0x4c41_4e44).unwrap();
        let other_seed = synthetic_benchmark_snapshot(100, 0x4c41_4e45).unwrap();

        assert_eq!(first, repeat);
        assert_ne!(first, other_seed);
        assert_eq!(first["geometry_status"], SYNTHETIC_GEOMETRY_STATUS);
        assert_eq!(first["features"].as_array().unwrap().len(), 100);
        assert_eq!(validate_snapshot(&first), Ok(()));
        assert_eq!(first["features"][0]["kind"], "built-element");
        assert_eq!(first["features"][0]["x_du"], 600);
        assert_eq!(first["features"][0]["y_du"], 29);
        assert_eq!(first["features"][1]["kind"], "planting-bed");
        assert_eq!(first["features"][1]["x_du"], 994);
        assert_eq!(first["features"][1]["y_du"], 500);
    }

    #[test]
    fn synthetic_benchmark_generator_enforces_size_ceiling() {
        assert!(synthetic_benchmark_snapshot(100_001, 1).is_err());
    }

    #[test]
    fn rejects_snapshot_over_v0_feature_limit() {
        let mut snapshot = fixture();
        snapshot["features"] = Value::Array(vec![Value::Null; MAX_SNAPSHOT_FEATURES + 1]);
        let errors = validate_snapshot(&snapshot).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("exceeds the v0 limit"))
        );
    }

    #[test]
    fn rejects_mixed_provenance_in_synthetic_document() {
        let mut snapshot = synthetic_benchmark_snapshot(2, 1).unwrap();
        snapshot["features"][1]["source"] = Value::String(USER_AUTHORED_SOURCE.to_string());

        let errors = validate_snapshot(&snapshot).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("status/source must agree"))
        );
    }

    #[test]
    fn rejects_georeference_claim_in_local_drawing_frame() {
        let mut snapshot = fixture();
        snapshot["coordinate_frame"]["georeferenced"] = Value::Bool(true);

        let errors = validate_snapshot(&snapshot).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("georeferenced")));
    }

    #[test]
    fn rejects_duplicate_feature_ids() {
        let mut snapshot = fixture();
        let first = snapshot["features"][0]["id"].clone();
        snapshot["features"][1]["id"] = first;

        let errors = validate_snapshot(&snapshot).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("duplicate feature ID"))
        );
    }

    #[test]
    fn rejects_kind_label_mismatch() {
        let mut snapshot = fixture();
        snapshot["features"][0]["label"] = Value::String("Water feature".to_string());

        let errors = validate_snapshot(&snapshot).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("label must match kind"))
        );
    }

    #[test]
    fn rejects_out_of_canvas_coordinates_and_unknown_fields() {
        let mut snapshot = fixture();
        snapshot["features"][0]["x_du"] = json!(1001);
        snapshot["features"][0]["surprise"] = json!("not in schema");

        let errors = validate_snapshot(&snapshot).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("x_du")));
        assert!(errors.iter().any(|error| error.contains("unknown field")));
    }

    #[test]
    fn rejects_blank_project_names_and_unknown_scopes() {
        let mut snapshot = fixture();
        snapshot["project_name"] = json!("   ");
        snapshot["scope"] = json!("planetary");

        let errors = validate_snapshot(&snapshot).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("project_name")));
        assert!(errors.iter().any(|error| error.contains("scope")));
    }

    #[test]
    fn rejects_non_object_roots_and_non_array_features() {
        assert!(validate_snapshot(&json!(null)).is_err());
        let mut snapshot = fixture();
        snapshot["features"] = json!({});
        assert!(validate_snapshot(&snapshot).is_err());
    }

    #[test]
    fn rejects_invalid_feature_ids() {
        for id in ["plant-0001", "feature-999999999999", "feature-4294967295"] {
            let mut snapshot = fixture();
            snapshot["features"][0]["id"] = json!(id);
            let errors = validate_snapshot(&snapshot).unwrap_err();
            assert!(errors.iter().any(|error| error.contains("feature-NNNN")));
        }
    }

    #[test]
    fn south_africa_source_catalog_has_versioned_provenance_and_no_ingested_layers() {
        let catalog: Value = serde_json::from_str(include_str!(
            "../data-sources/landscape/south-africa-johannesburg-v0.json"
        ))
        .expect("South African source catalog must be valid JSON");
        let schema: Value = serde_json::from_str(include_str!(
            "../schemas/landscape-source-catalog-v0.schema.json"
        ))
        .expect("source-catalog schema must be valid JSON");

        assert_eq!(catalog["schema"], "luminous.landscape-source-catalog.v0");
        assert_eq!(catalog["schema_uri"], schema["$id"]);
        assert_eq!(catalog["catalog_status"], "metadata-only-not-ingested");
        let ngi = catalog["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["id"] == "ngi.cdngi-topo-orthophoto")
            .expect("CD:NGI should be catalogued as a candidate source");
        assert_eq!(ngi["ingestion_state"], "not-ingested");
        assert_eq!(ngi["license"]["status"], "review-required");
        assert_eq!(
            catalog["pilot"]["name"],
            "Johannesburg / Highveld, Gauteng, South Africa"
        );

        let records = catalog["records"]
            .as_array()
            .expect("source records must be an array");
        assert_eq!(records.len(), 7);
        let mut ids = HashSet::with_capacity(records.len());
        for record in records {
            let id = record["id"].as_str().expect("source ID must be a string");
            assert!(ids.insert(id), "duplicate source ID: {id}");
            for field in ["publisher", "service_uri", "metadata_uri", "attribution"] {
                assert!(
                    record[field]
                        .as_str()
                        .is_some_and(|value| !value.trim().is_empty()),
                    "source {id} has no {field}"
                );
            }
            assert!(
                record["license"]["status"].as_str().is_some(),
                "source {id} has no explicit license state"
            );
            assert_eq!(record["ingestion_state"], "not-ingested");
            assert!(
                record["validation_gates"]
                    .as_array()
                    .is_some_and(|gates| !gates.is_empty()),
                "source {id} has no validation gates"
            );
        }
        assert_eq!(
            schema["properties"]["catalog_status"]["enum"][0],
            "metadata-only-not-ingested"
        );
        assert_eq!(
            schema["properties"]["records"]["items"]["additionalProperties"],
            false
        );
    }

    #[test]
    fn performance_metric_registry_is_traceable_and_definition_only() {
        let registry: Value =
            serde_json::from_str(include_str!("../metrics/landscape/registry-v0.json"))
                .expect("performance metric registry must be valid JSON");
        let schema: Value = serde_json::from_str(include_str!(
            "../schemas/landscape-performance-registry-v0.schema.json"
        ))
        .expect("performance registry schema must be valid JSON");

        assert_eq!(
            registry["schema"],
            "luminous.landscape-performance-registry.v0"
        );
        assert_eq!(registry["schema_uri"], schema["$id"]);
        assert_eq!(registry["status"], "specification-only-no-field-values");
        assert_eq!(
            schema["properties"]["metrics"]["items"]["additionalProperties"],
            false
        );

        let metrics = registry["metrics"]
            .as_array()
            .expect("metric definitions must be an array");
        assert_eq!(metrics.len(), 8);

        let catalog: Value = serde_json::from_str(include_str!(
            "../data-sources/landscape/south-africa-johannesburg-v0.json"
        ))
        .expect("landscape source catalog must be valid JSON");
        let source_records = catalog["records"]
            .as_array()
            .expect("source records must be an array");
        let source_ids: HashSet<&str> = source_records
            .iter()
            .map(|record| record["id"].as_str().expect("source ID must be a string"))
            .collect();
        assert_eq!(
            source_ids.len(),
            source_records.len(),
            "source IDs must be unique"
        );

        let mut ids = HashSet::with_capacity(metrics.len());
        let mut families = HashSet::with_capacity(7);
        for metric in metrics {
            let id = metric["id"].as_str().expect("metric ID must be a string");
            assert!(ids.insert(id), "duplicate metric ID: {id}");
            families.insert(
                metric["family"]
                    .as_str()
                    .expect("metric family must be a string"),
            );
            for field in ["label", "unit", "semantic_type", "formula", "target_policy"] {
                assert!(
                    metric[field]
                        .as_str()
                        .is_some_and(|value| !value.trim().is_empty()),
                    "metric {id} has no {field}"
                );
            }
            for field in [
                "required_inputs",
                "failure_modes",
                "uncertainty_requirements",
            ] {
                assert!(
                    metric[field]
                        .as_array()
                        .is_some_and(|values| !values.is_empty()),
                    "metric {id} has no {field}"
                );
            }
            assert!(
                matches!(
                    metric["v0_status"].as_str(),
                    Some("not-computable" | "requires-myclix-stewardship-contract")
                ),
                "metric {id} must be explicitly unavailable until qualified"
            );
            assert!(
                metric.as_object().is_some_and(|object| {
                    !object.contains_key("value") && !object.contains_key("measured_value")
                }),
                "metric {id} must not embed an unqualified result value"
            );

            let candidate_sources = metric["candidate_sources"]
                .as_array()
                .expect("candidate_sources must be an array");
            let mut metric_source_ids = HashSet::with_capacity(candidate_sources.len());
            for candidate in candidate_sources {
                let source_id = candidate["source_id"]
                    .as_str()
                    .expect("candidate source ID must be a string");
                assert!(
                    source_ids.contains(source_id),
                    "metric {id} references unknown source {source_id}"
                );
                assert!(
                    metric_source_ids.insert(source_id),
                    "metric {id} repeats source {source_id}"
                );
                assert!(
                    candidate["role"]
                        .as_str()
                        .is_some_and(|value| !value.trim().is_empty()),
                    "metric {id} candidate {source_id} must explain its role"
                );
                assert!(
                    candidate["limitation"]
                        .as_str()
                        .is_some_and(|value| !value.trim().is_empty()),
                    "metric {id} candidate {source_id} must state limitations"
                );
                let catalog_record = source_records
                    .iter()
                    .find(|record| record["id"] == source_id)
                    .expect("candidate source reference must match a catalog record");
                assert_eq!(
                    catalog_record["ingestion_state"], "not-ingested",
                    "candidate source {source_id} must not be promoted to available data by a metric link"
                );
            }
            assert!(
                metric["unprovided_inputs"]
                    .as_array()
                    .is_some_and(|values| !values.is_empty()),
                "metric {id} must explicitly list evidence gaps"
            );
        }
        assert_eq!(families.len(), 7);
    }

    #[test]
    fn benchmark_manifest_matches_generator_limits_without_claiming_results() {
        let manifest: Value =
            serde_json::from_str(include_str!("../benchmarks/landscape/workloads-v0.json"))
                .expect("landscape benchmark manifest must be valid JSON");

        assert_eq!(
            manifest["manifest_version"],
            "landscape-editor-benchmark-v0"
        );
        assert_eq!(manifest["status"], "protocol-only-no-measurements");
        assert_eq!(
            manifest["generator"]["implementation"],
            "src/landscape.rs::synthetic_benchmark_snapshot"
        );
        assert_eq!(manifest["coordinate_frame"], FRAME_KIND);

        let sizes = manifest["workload_sizes"]
            .as_array()
            .expect("workload sizes must be an array");
        let counts: Vec<u64> = sizes
            .iter()
            .map(|size| size["feature_count"].as_u64().unwrap())
            .collect();
        assert_eq!(counts, [100, 1_000, 10_000, MAX_SNAPSHOT_FEATURES as u64]);
        assert_eq!(manifest["generator"]["initial_seed_hex"], "0x4c414e44");
        assert_eq!(
            manifest["generator"]["geometry_warning"],
            "These generated features are synthetic drawing units; no geographic or physical meaning."
        );
    }
}
