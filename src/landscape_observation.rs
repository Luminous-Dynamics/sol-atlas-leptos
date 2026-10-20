// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Strict, offline-first landscape observation contracts.
//!
//! This module validates record shape and cross-field invariants. It does not
//! establish measurement accuracy, verify consent, certify methods, or prove
//! causal effects. Mycelix remains the consent/governance authority; external
//! sensor exchange should use an adapter to OGC SensorThings/STAplus rather
//! than duplicating those APIs.

use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub const OBSERVATION_SCHEMA: &str = "luminous.landscape-observation.v0";
pub const OBSERVATION_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-observation-v0.schema.json";
pub const BATCH_SCHEMA: &str = "luminous.landscape-observation-batch.v0";
pub const BATCH_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-observation-batch-v0.schema.json";
pub const CONSENT_AUTHORITY: &str = "mycelix";
pub const MAX_BATCH_OBSERVATIONS: usize = 10_000;

const BATCH_KEYS: &[&str] = &[
    "schema",
    "schema_uri",
    "batch_id",
    "created_at_unix_ms",
    "observations",
];
const OBSERVATION_KEYS: &[&str] = &[
    "schema",
    "schema_uri",
    "observation_id",
    "source_event_id",
    "project_ref",
    "phenomenon_time",
    "result_time_unix_ms",
    "observed_property",
    "result",
    "evidence_class",
    "source",
    "quality",
    "location",
    "privacy",
    "transformations",
];
const PROJECT_REF_KEYS: &[&str] = &[
    "project_snapshot_sha256",
    "feature_id",
    "site_ref_sha256",
];
const PHENOMENON_TIME_KEYS: &[&str] =
    &["start_unix_ms", "end_unix_ms", "clock_uncertainty_ms"];
const PROPERTY_KEYS: &[&str] = &["id", "label", "metric_id", "unit"];
const SOURCE_KEYS: &[&str] = &[
    "kind",
    "source_id",
    "source_uri",
    "source_version",
    "payload_sha256",
    "method_uri",
    "method_version",
    "observer_ref_sha256",
    "instrument_ref_sha256",
];
const QUALITY_KEYS: &[&str] = &["state", "calibration_state", "flags", "uncertainty"];
const UNCERTAINTY_KEYS: &[&str] = &[
    "kind",
    "value",
    "lower_bound",
    "upper_bound",
    "coverage_probability",
    "unit",
    "method_uri",
    "reason_code",
];
const PRIVACY_KEYS: &[&str] = &[
    "classification",
    "publication_policy",
    "consent_record_ref_sha256",
    "policy_version",
    "consent_authority",
];
const TRANSFORMATION_KEYS: &[&str] = &[
    "activity_uri",
    "activity_version",
    "input_sha256",
    "output_sha256",
    "performed_at_unix_ms",
    "agent_ref_sha256",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchValidation {
    /// Number of records in the input, including repeated byte-equivalent records.
    pub input_records: usize,
    /// Number of unique observation IDs after idempotent duplicate recognition.
    pub unique_observations: usize,
    /// Count of extra, byte-equivalent records with an already-seen observation ID.
    pub identical_duplicate_records: usize,
    /// Stable, sorted IDs that occurred more than once with exactly equal content.
    pub duplicate_observation_ids: Vec<String>,
    /// Count of unique records explicitly classified as synthetic.
    pub synthetic_observations: usize,
}

/// Validate an offline batch and remove only records proven to be exact
/// duplicates. Original first-seen order is preserved. Conflicting content for
/// a previously seen ID is rejected by `validate_batch`, never last-write-wins.
pub fn normalize_batch(value: &Value) -> Result<(Value, BatchValidation), Vec<String>> {
    let summary = validate_batch(value)?;
    let mut normalized = value.clone();
    let Some(observations) = normalized
        .get_mut("observations")
        .and_then(Value::as_array_mut)
    else {
        // validate_batch already guards this invariant; retain a typed failure
        // if a future schema evolution breaks that assumption.
        return Err(vec![
            "$.observations: validated batch lost its observation array".to_string(),
        ]);
    };
    let mut seen = HashSet::with_capacity(summary.unique_observations);
    observations.retain(|observation| {
        observation
            .get("observation_id")
            .and_then(Value::as_str)
            .is_some_and(|id| seen.insert(id.to_string()))
    });
    Ok((normalized, summary))
}

/// Validate an offline batch. Repeated identical records are reported as
/// idempotent duplicates; different content under one observation ID is a
/// hard conflict and invalidates the whole batch.
pub fn validate_batch(value: &Value) -> Result<BatchValidation, Vec<String>> {
    let mut errors = Vec::new();
    if !check_exact_keys(value, BATCH_KEYS, "$", &mut errors) {
        return Err(errors);
    }

    require_string_eq(value, "schema", BATCH_SCHEMA, "$", &mut errors);
    require_string_eq(value, "schema_uri", BATCH_SCHEMA_URI, "$", &mut errors);
    validate_identifier(
        value.get("batch_id"),
        "obs-batch-",
        "$.batch_id",
        128,
        &mut errors,
    );
    let created_at = required_u64(value, "created_at_unix_ms", "$", &mut errors);

    let Some(observations) = value.get("observations").and_then(Value::as_array) else {
        errors.push("$.observations: expected an array".to_string());
        return Err(errors);
    };
    if observations.is_empty() {
        errors.push("$.observations: expected at least one observation".to_string());
        return Err(errors);
    }
    if observations.len() > MAX_BATCH_OBSERVATIONS {
        errors.push(format!(
            "$.observations: batch size {} exceeds limit {MAX_BATCH_OBSERVATIONS}",
            observations.len()
        ));
        return Err(errors);
    }

    let mut seen: HashMap<String, Value> = HashMap::new();
    let mut duplicate_ids = HashSet::new();
    let mut duplicate_records = 0usize;
    let mut synthetic_ids = HashSet::new();

    for (index, observation) in observations.iter().enumerate() {
        let path = format!("$.observations[{index}]");
        validate_observation(observation, &path, &mut errors);

        if let Some(result_time) = observation.get("result_time_unix_ms").and_then(Value::as_u64) {
            if created_at.is_some_and(|created| created < result_time) {
                errors.push(format!(
                    "{path}.result_time_unix_ms: batch creation time predates the observation result"
                ));
            }
        }

        let Some(id) = observation
            .get("observation_id")
            .and_then(Value::as_str)
        else {
            continue;
        };

        if observation.get("evidence_class").and_then(Value::as_str) == Some("synthetic-test") {
            synthetic_ids.insert(id.to_string());
        }

        match seen.get(id) {
            Some(previous) if previous == observation => {
                duplicate_records += 1;
                duplicate_ids.insert(id.to_string());
            }
            Some(_) => errors.push(format!(
                "{path}.observation_id: conflicting content for previously seen observation ID {id:?}"
            )),
            None => {
                seen.insert(id.to_string(), observation.clone());
            }
        }
    }

    if errors.is_empty() {
        let mut duplicate_observation_ids: Vec<String> = duplicate_ids.into_iter().collect();
        duplicate_observation_ids.sort();
        Ok(BatchValidation {
            input_records: observations.len(),
            unique_observations: seen.len(),
            identical_duplicate_records: duplicate_records,
            duplicate_observation_ids,
            synthetic_observations: synthetic_ids.len(),
        })
    } else {
        Err(errors)
    }
}

fn validate_observation(value: &Value, path: &str, errors: &mut Vec<String>) {
    if !check_exact_keys(value, OBSERVATION_KEYS, path, errors) {
        return;
    }

    require_string_eq(value, "schema", OBSERVATION_SCHEMA, path, errors);
    require_string_eq(value, "schema_uri", OBSERVATION_SCHEMA_URI, path, errors);
    validate_identifier(
        value.get("observation_id"),
        "obs-",
        &format!("{path}.observation_id"),
        128,
        errors,
    );
    validate_token(
        value.get("source_event_id"),
        &format!("{path}.source_event_id"),
        256,
        errors,
    );

    validate_project_ref(value.get("project_ref").unwrap(), &format!("{path}.project_ref"), errors);

    let event_end = validate_phenomenon_time(
        value.get("phenomenon_time").unwrap(),
        &format!("{path}.phenomenon_time"),
        errors,
    );
    let result_time = required_u64(value, "result_time_unix_ms", path, errors);
    if let (Some(end), Some(result)) = (event_end, result_time) {
        if result < end {
            errors.push(format!(
                "{path}.result_time_unix_ms: result time must not precede the phenomenon end"
            ));
        }
    }

    let property = value.get("observed_property").unwrap();
    validate_property(property, &format!("{path}.observed_property"), errors);
    let result = value.get("result").unwrap();
    let (result_kind, result_unit) =
        validate_result(result, property, &format!("{path}.result"), errors);

    let evidence_class = required_text(value, "evidence_class", path, errors);
    let source = value.get("source").unwrap();
    let source_kind = validate_source(source, &format!("{path}.source"), errors);
    let quality = value.get("quality").unwrap();
    let calibration_state = validate_quality(
        quality,
        result_kind,
        result_unit,
        &format!("{path}.quality"),
        errors,
    );

    if let (Some(evidence), Some(source_kind)) = (evidence_class, source_kind) {
        let expected = match evidence {
            "field-observation" => Some("human-protocol"),
            "sensor-observation" => Some("sensor"),
            "remote-sensing-classification" => Some("remote-sensing-provider"),
            "lab-analysis" => Some("laboratory"),
            "community-report" => Some("community"),
            "synthetic-test" => Some("synthetic-fixture"),
            "imported-observation" => Some("external-adapter"),
            _ => {
                errors.push(format!(
                    "{path}.evidence_class: unsupported evidence class {evidence:?}"
                ));
                None
            }
        };
        if expected != Some(source_kind) {
            errors.push(format!(
                "{path}: evidence_class {evidence:?} must pair with source.kind {:?}",
                expected.unwrap_or("a supported source kind")
            ));
        }
        if evidence == "sensor-observation"
            && source.get("instrument_ref_sha256") == Some(&Value::Null)
        {
            errors.push(format!(
                "{path}.source.instrument_ref_sha256: sensor observations require a non-identity instrument reference"
            ));
        }
        if evidence == "sensor-observation"
            && quality.get("state").and_then(Value::as_str) == Some("accepted")
            && calibration_state != Some("current")
        {
            errors.push(format!(
                "{path}.quality: accepted sensor observations require current calibration"
            ));
        }
    }

    let location_sensitivity =
        validate_location(value.get("location").unwrap(), &format!("{path}.location"), errors);
    validate_privacy(
        value.get("privacy").unwrap(),
        evidence_class,
        location_sensitivity,
        &format!("{path}.privacy"),
        errors,
    );

    let source_digest = source.get("payload_sha256").and_then(Value::as_str);
    validate_transformations(
        value.get("transformations").unwrap(),
        source_digest,
        result_time,
        &format!("{path}.transformations"),
        errors,
    );
}

fn validate_project_ref(value: &Value, path: &str, errors: &mut Vec<String>) {
    if !check_exact_keys(value, PROJECT_REF_KEYS, path, errors) {
        return;
    }
    let project_digest = optional_digest(value, "project_snapshot_sha256", path, errors);
    let site_digest = optional_digest(value, "site_ref_sha256", path, errors);
    if project_digest.is_none() && site_digest.is_none() {
        errors.push(format!(
            "{path}: a project snapshot digest or opaque site-reference digest is required"
        ));
    }
    match value.get("feature_id") {
        Some(Value::Null) => {}
        Some(Value::String(id)) if valid_feature_id(id) => {
            if project_digest.is_none() {
                errors.push(format!(
                    "{path}.feature_id: a feature reference requires project_snapshot_sha256"
                ));
            }
        }
        _ => errors.push(format!(
            "{path}.feature_id: expected null or a valid feature-NNNN identifier"
        )),
    }
}

fn validate_phenomenon_time(value: &Value, path: &str, errors: &mut Vec<String>) -> Option<u64> {
    if !check_exact_keys(value, PHENOMENON_TIME_KEYS, path, errors) {
        return None;
    }
    let start = required_u64(value, "start_unix_ms", path, errors);
    let end = required_u64(value, "end_unix_ms", path, errors);
    required_u64(value, "clock_uncertainty_ms", path, errors);
    if let (Some(start), Some(end)) = (start, end) {
        if start > end {
            errors.push(format!("{path}: start time must be <= end time"));
        }
    }
    end
}

fn validate_property(value: &Value, path: &str, errors: &mut Vec<String>) {
    if !check_exact_keys(value, PROPERTY_KEYS, path, errors) {
        return;
    }
    require_uri(value, "id", path, errors);
    validate_bounded_text(value, "label", 200, path, errors);
    match value.get("metric_id") {
        Some(Value::Null) => {}
        Some(Value::String(id)) if valid_metric_id(id) => {}
        _ => errors.push(format!("{path}.metric_id: expected null or a stable metric identifier")),
    }
    match value.get("unit") {
        Some(Value::Null) => {}
        Some(Value::String(unit)) if !unit.trim().is_empty() && unit.len() <= 128 => {}
        _ => errors.push(format!("{path}.unit: expected null or a non-empty unit")),
    }
}

fn validate_result<'a>(
    value: &'a Value,
    property: &Value,
    path: &str,
    errors: &mut Vec<String>,
) -> (Option<&'a str>, Option<&'a str>) {
    let Some(kind) = value.get("kind").and_then(Value::as_str) else {
        errors.push(format!("{path}.kind: expected a string"));
        return (None, None);
    };
    let property_unit = property.get("unit").and_then(Value::as_str);

    match kind {
        "quantity" => {
            if !check_exact_keys(value, &["kind", "numeric_value", "unit"], path, errors) {
                return (Some(kind), None);
            }
            match value.get("numeric_value").and_then(Value::as_f64) {
                Some(number) if number.is_finite() => {}
                _ => errors.push(format!("{path}.numeric_value: expected a finite JSON number")),
            }
            let unit = required_text(value, "unit", path, errors);
            if let (Some(unit), Some(property_unit)) = (unit, property_unit) {
                if unit != property_unit {
                    errors.push(format!(
                        "{path}.unit: result unit {unit:?} does not match observed_property.unit {property_unit:?}"
                    ));
                }
            } else {
                errors.push(format!(
                    "{path}.unit: quantity results require the observed property to declare the same unit"
                ));
            }
            (Some(kind), unit)
        }
        "category" => {
            if !check_exact_keys(value, &["kind", "category_value", "vocabulary_uri"], path, errors) {
                return (Some(kind), None);
            }
            validate_bounded_text(value, "category_value", 256, path, errors);
            require_uri(value, "vocabulary_uri", path, errors);
            if property_unit.is_some() {
                errors.push(format!(
                    "{path}: category results must not declare a physical unit"
                ));
            }
            (Some(kind), None)
        }
        "boolean" => {
            if !check_exact_keys(value, &["kind", "boolean_value"], path, errors) {
                return (Some(kind), None);
            }
            if !matches!(value.get("boolean_value"), Some(Value::Bool(_))) {
                errors.push(format!("{path}.boolean_value: expected a boolean"));
            }
            if property_unit.is_some() {
                errors.push(format!(
                    "{path}: boolean results must not declare a physical unit"
                ));
            }
            (Some(kind), None)
        }
        _ => {
            errors.push(format!("{path}.kind: unsupported result kind {kind:?}"));
            (Some(kind), None)
        }
    }
}

fn validate_source<'a>(
    value: &'a Value,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    if !check_exact_keys(value, SOURCE_KEYS, path, errors) {
        return None;
    }
    let kind = required_text(value, "kind", path, errors);
    if !matches!(
        kind,
        Some(
            "human-protocol"
                | "sensor"
                | "remote-sensing-provider"
                | "laboratory"
                | "community"
                | "synthetic-fixture"
                | "external-adapter"
        )
    ) {
        errors.push(format!("{path}.kind: unsupported source kind"));
    }
    validate_token(value.get("source_id"), &format!("{path}.source_id"), 256, errors);
    require_uri(value, "source_uri", path, errors);
    validate_bounded_text(value, "source_version", 128, path, errors);
    require_digest(value, "payload_sha256", path, errors);
    require_uri(value, "method_uri", path, errors);
    validate_bounded_text(value, "method_version", 128, path, errors);
    optional_digest(value, "observer_ref_sha256", path, errors);
    optional_digest(value, "instrument_ref_sha256", path, errors);

    if kind == Some("human-protocol") && value.get("observer_ref_sha256") == Some(&Value::Null) {
        errors.push(format!(
            "{path}.observer_ref_sha256: human field observations require a privacy-preserving observer reference"
        ));
    }
    if kind == Some("community") && value.get("observer_ref_sha256") == Some(&Value::Null) {
        errors.push(format!(
            "{path}.observer_ref_sha256: community reports require a privacy-preserving contributor reference"
        ));
    }

    kind
}

fn validate_quality<'a>(
    value: &'a Value,
    result_kind: Option<&str>,
    result_unit: Option<&str>,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    if !check_exact_keys(value, QUALITY_KEYS, path, errors) {
        return None;
    }
    let state = required_text(value, "state", path, errors);
    if !matches!(state, Some("unreviewed" | "accepted" | "flagged" | "rejected")) {
        errors.push(format!("{path}.state: unsupported quality state"));
    }
    let calibration = required_text(value, "calibration_state", path, errors);
    if !matches!(calibration, Some("current" | "stale" | "not-required" | "unknown")) {
        errors.push(format!("{path}.calibration_state: unsupported calibration state"));
    }

    let Some(flags) = value.get("flags").and_then(Value::as_array) else {
        errors.push(format!("{path}.flags: expected an array"));
        validate_uncertainty(
            value.get("uncertainty").unwrap(),
            result_kind,
            result_unit,
            &format!("{path}.uncertainty"),
            errors,
        );
        return calibration;
    };
    let mut seen = HashSet::new();
    for (index, flag) in flags.iter().enumerate() {
        match flag.as_str() {
            Some(flag) if valid_token(flag, 128) => {
                if !seen.insert(flag) {
                    errors.push(format!("{path}.flags[{index}]: duplicate quality flag"));
                }
            }
            _ => errors.push(format!(
                "{path}.flags[{index}]: expected a stable lowercase quality flag"
            )),
        }
    }
    validate_uncertainty(
        value.get("uncertainty").unwrap(),
        result_kind,
        result_unit,
        &format!("{path}.uncertainty"),
        errors,
    );
    calibration
}

fn validate_uncertainty(
    value: &Value,
    result_kind: Option<&str>,
    result_unit: Option<&str>,
    path: &str,
    errors: &mut Vec<String>,
) {
    if !check_exact_keys(value, UNCERTAINTY_KEYS, path, errors) {
        return;
    }
    let kind = required_text(value, "kind", path, errors);
    let number = |key: &str| -> Option<f64> {
        match value.get(key) {
            Some(Value::Null) => None,
            Some(value) => value.as_f64().filter(|number| number.is_finite()),
            None => None,
        }
    };
    let unit = value.get("unit").and_then(Value::as_str);
    let method_uri = value.get("method_uri").and_then(Value::as_str);
    let reason = value.get("reason_code").and_then(Value::as_str);
    let raw_fields = [
        ("value", number("value")),
        ("lower_bound", number("lower_bound")),
        ("upper_bound", number("upper_bound")),
        ("coverage_probability", number("coverage_probability")),
    ];

    for (key, raw) in [
        ("value", value.get("value")),
        ("lower_bound", value.get("lower_bound")),
        ("upper_bound", value.get("upper_bound")),
        ("coverage_probability", value.get("coverage_probability")),
    ] {
        if !matches!(raw, Some(Value::Null) | Some(Value::Number(_))) {
            errors.push(format!("{path}.{key}: expected a finite number or null"));
        }
        if let Some(Value::Number(_)) = raw {
            if number(key).is_none() {
                errors.push(format!("{path}.{key}: expected a finite number"));
            }
        }
    }

    match kind {
        Some("standard-uncertainty") => {
            if result_kind != Some("quantity") {
                errors.push(format!(
                    "{path}.kind: standard uncertainty requires a quantity result"
                ));
            }
            if !number("value").is_some_and(|value| value >= 0.0)
                || number("lower_bound").is_some()
                || number("upper_bound").is_some()
                || number("coverage_probability").is_some()
            {
                errors.push(format!(
                    "{path}: standard uncertainty requires value >= 0 and null interval fields"
                ));
            }
            if unit != result_unit || unit.is_none() {
                errors.push(format!("{path}.unit: uncertainty unit must match result unit"));
            }
            if method_uri.is_none_or(|uri| !valid_uri(uri)) {
                errors.push(format!("{path}.method_uri: a valid uncertainty method URI is required"));
            }
            if reason.is_some() {
                errors.push(format!("{path}.reason_code: must be null for quantified uncertainty"));
            }
        }
        Some("interval") => {
            if result_kind != Some("quantity") {
                errors.push(format!("{path}.kind: interval uncertainty requires a quantity result"));
            }
            if number("value").is_some()
                || !number("lower_bound").is_some()
                || !number("upper_bound").is_some()
                || number("coverage_probability").is_none_or(|coverage| coverage <= 0.0 || coverage > 1.0)
            {
                errors.push(format!(
                    "{path}: interval requires lower/upper bounds and coverage in (0, 1]"
                ));
            }
            if let (Some(lower), Some(upper)) = (number("lower_bound"), number("upper_bound")) {
                if lower > upper {
                    errors.push(format!("{path}: lower_bound must be <= upper_bound"));
                }
            }
            if unit != result_unit || unit.is_none() {
                errors.push(format!("{path}.unit: uncertainty unit must match result unit"));
            }
            if method_uri.is_none_or(|uri| !valid_uri(uri)) {
                errors.push(format!("{path}.method_uri: a valid uncertainty method URI is required"));
            }
            if reason.is_some() {
                errors.push(format!("{path}.reason_code: must be null for quantified uncertainty"));
            }
        }
        Some("qualitative") => {
            if raw_fields.iter().any(|(_, number)| number.is_some()) || unit.is_some() {
                errors.push(format!("{path}: qualitative uncertainty must not carry numeric fields or a unit"));
            }
            if reason.is_none_or(|reason| !valid_lowercase_code(reason, 128)) {
                errors.push(format!("{path}.reason_code: qualitative uncertainty requires a stable reason code"));
            }
            if method_uri.is_some_and(|uri| !valid_uri(uri)) {
                errors.push(format!("{path}.method_uri: invalid URI"));
            }
        }
        Some("unknown" | "not-applicable") => {
            if raw_fields.iter().any(|(_, number)| number.is_some())
                || unit.is_some()
                || method_uri.is_some()
            {
                errors.push(format!(
                    "{path}: unknown/not-applicable uncertainty must not carry numeric fields, units, or a method URI"
                ));
            }
            if reason.is_none_or(|reason| !valid_token(reason, 128)) {
                errors.push(format!(
                    "{path}.reason_code: unknown/not-applicable uncertainty requires a stable reason code"
                ));
            }
        }
        Some(_) => errors.push(format!("{path}.kind: unsupported uncertainty kind")),
        None => {}
    }

    if result_kind != Some("quantity")
        && matches!(kind, Some("standard-uncertainty" | "interval"))
    {
        errors.push(format!(
            "{path}: non-quantity results cannot carry numeric uncertainty intervals"
        ));
    }
}

fn validate_location<'a>(
    value: &'a Value,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    let kind = value.get("kind").and_then(Value::as_str);
    match kind {
        Some("geographic-wgs84") => {
            const KEYS: &[&str] = &[
                "kind",
                "crs",
                "longitude_latitude_deg",
                "precision_m",
                "sensitivity",
            ];
            if !check_exact_keys(value, KEYS, path, errors) {
                return None;
            }
            require_string_eq(value, "crs", "OGC:CRS84", path, errors);
            match value.get("longitude_latitude_deg").and_then(Value::as_array) {
                Some(coords) if coords.len() == 2 => {
                    let lon = coords[0].as_f64().filter(|v| v.is_finite());
                    let lat = coords[1].as_f64().filter(|v| v.is_finite());
                    if !lon.is_some_and(|v| (-180.0..=180.0).contains(&v)) {
                        errors.push(format!(
                            "{path}.longitude_latitude_deg[0]: expected longitude in [-180, 180]"
                        ));
                    }
                    if !lat.is_some_and(|v| (-90.0..=90.0).contains(&v)) {
                        errors.push(format!(
                            "{path}.longitude_latitude_deg[1]: expected latitude in [-90, 90]"
                        ));
                    }
                }
                _ => errors.push(format!(
                    "{path}.longitude_latitude_deg: expected [longitude, latitude]"
                )),
            }
            validate_positive_number(value, "precision_m", path, errors);
            validate_sensitivity(value, path, errors)
        }
        Some("project-local") => {
            const KEYS: &[&str] = &[
                "kind",
                "frame_id",
                "units",
                "coordinates_xy",
                "precision_units",
                "sensitivity",
            ];
            if !check_exact_keys(value, KEYS, path, errors) {
                return None;
            }
            validate_bounded_text(value, "frame_id", 512, path, errors);
            let units = required_text(value, "units", path, errors);
            if !matches!(units, Some("drawing-unit" | "m" | "ft" | "other-declared")) {
                errors.push(format!("{path}.units: unsupported coordinate unit declaration"));
            }
            match value.get("coordinates_xy").and_then(Value::as_array) {
                Some(coords)
                    if coords.len() == 2
                        && coords.iter().all(|coord| {
                            coord.as_f64().is_some_and(|number| number.is_finite())
                        }) => {}
                _ => errors.push(format!(
                    "{path}.coordinates_xy: expected two finite [x, y] coordinates"
                )),
            }
            validate_positive_number(value, "precision_units", path, errors);
            validate_sensitivity(value, path, errors)
        }
        Some("opaque-site") => {
            const KEYS: &[&str] = &["kind", "site_ref_sha256", "precision_class", "sensitivity"];
            if !check_exact_keys(value, KEYS, path, errors) {
                return None;
            }
            require_digest(value, "site_ref_sha256", path, errors);
            let precision = required_text(value, "precision_class", path, errors);
            if !matches!(
                precision,
                Some("site" | "parcel" | "neighbourhood" | "catchment" | "region" | "redacted")
            ) {
                errors.push(format!("{path}.precision_class: unsupported precision class"));
            }
            validate_sensitivity(value, path, errors)
        }
        Some(_) => {
            errors.push(format!("{path}.kind: unsupported location representation"));
            None
        }
        None => {
            errors.push(format!("{path}.kind: expected a string"));
            None
        }
    }
}

fn validate_privacy(
    value: &Value,
    evidence_class: Option<&str>,
    location_sensitivity: Option<&str>,
    path: &str,
    errors: &mut Vec<String>,
) {
    if !check_exact_keys(value, PRIVACY_KEYS, path, errors) {
        return;
    }
    let classification = required_text(value, "classification", path, errors);
    let policy = required_text(value, "publication_policy", path, errors);
    let consent = optional_digest(value, "consent_record_ref_sha256", path, errors);
    validate_bounded_text(value, "policy_version", 128, path, errors);
    require_string_eq(value, "consent_authority", CONSENT_AUTHORITY, path, errors);

    if !matches!(classification, Some("public" | "private" | "sensitive" | "synthetic")) {
        errors.push(format!("{path}.classification: unsupported privacy classification"));
    }
    if !matches!(policy, Some("private-only" | "aggregate-only" | "public-approved")) {
        errors.push(format!("{path}.publication_policy: unsupported publication policy"));
    }
    if policy == Some("public-approved") {
        if classification != Some("public") {
            errors.push(format!(
                "{path}: raw public release requires classification public"
            ));
        }
        if consent.is_none() {
            errors.push(format!(
                "{path}.consent_record_ref_sha256: public-approved release requires a consent/authority record reference"
            ));
        }
        if location_sensitivity != Some("public") {
            errors.push(format!(
                "{path}: public-approved raw release requires public location sensitivity or a prior redaction transformation"
            ));
        }
    }
    if classification == Some("sensitive") && policy == Some("public-approved") {
        errors.push(format!("{path}: sensitive observations cannot be raw-public-approved"));
    }
    if evidence_class == Some("synthetic-test") && classification != Some("synthetic") {
        errors.push(format!(
            "{path}.classification: synthetic-test observations must remain classified synthetic"
        ));
    }
    if classification == Some("synthetic") && evidence_class != Some("synthetic-test") {
        errors.push(format!(
            "{path}.classification: synthetic classification requires evidence_class synthetic-test"
        ));
    }
}

fn validate_transformations(
    value: &Value,
    source_digest: Option<&str>,
    result_time: Option<u64>,
    path: &str,
    errors: &mut Vec<String>,
) {
    let Some(transformations) = value.as_array() else {
        errors.push(format!("{path}: expected an array"));
        return;
    };
    if transformations.len() > 1024 {
        errors.push(format!("{path}: too many transformations"));
        return;
    }

    let mut expected_input = source_digest;
    let mut previous_time = None;
    for (index, transformation) in transformations.iter().enumerate() {
        let step_path = format!("{path}[{index}]");
        if !check_exact_keys(transformation, TRANSFORMATION_KEYS, &step_path, errors) {
            continue;
        }
        require_uri(transformation, "activity_uri", &step_path, errors);
        validate_bounded_text(transformation, "activity_version", 128, &step_path, errors);
        let input_digest = required_digest(transformation, "input_sha256", &step_path, errors);
        let output_digest = required_digest(transformation, "output_sha256", &step_path, errors);
        let performed_at =
            required_u64(transformation, "performed_at_unix_ms", &step_path, errors);
        optional_digest(transformation, "agent_ref_sha256", &step_path, errors);

        if let (Some(expected), Some(actual)) = (expected_input, input_digest) {
            if expected != actual {
                errors.push(format!(
                    "{step_path}.input_sha256: transformation chain is discontinuous"
                ));
            }
        }
        if let (Some(previous), Some(current)) = (previous_time, performed_at) {
            if current < previous {
                errors.push(format!(
                    "{step_path}.performed_at_unix_ms: transformation time moved backwards"
                ));
            }
        }
        if let (Some(result), Some(current)) = (result_time, performed_at) {
            if current > result {
                errors.push(format!(
                    "{step_path}.performed_at_unix_ms: transformation occurs after result time"
                ));
            }
        }
        expected_input = output_digest;
        previous_time = performed_at;
    }
}

fn validate_sensitivity<'a>(
    value: &'a Value,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    let sensitivity = required_text(value, "sensitivity", path, errors);
    if !matches!(
        sensitivity,
        Some("public" | "coarse" | "restricted" | "sensitive")
    ) {
        errors.push(format!("{path}.sensitivity: unsupported location sensitivity"));
    }
    sensitivity
}

fn validate_positive_number(value: &Value, key: &str, path: &str, errors: &mut Vec<String>) {
    match value.get(key).and_then(Value::as_f64) {
        Some(number) if number.is_finite() && number > 0.0 => {}
        _ => errors.push(format!("{path}.{key}: expected a finite number > 0")),
    }
}

fn check_exact_keys(value: &Value, keys: &[&str], path: &str, errors: &mut Vec<String>) -> bool {
    let Some(object) = value.as_object() else {
        errors.push(format!("{path}: expected an object"));
        return false;
    };
    let mut exact = true;
    for key in keys {
        if !object.contains_key(*key) {
            errors.push(format!("{path}.{key}: missing required field"));
            exact = false;
        }
    }
    for key in object.keys() {
        if !keys.contains(&key.as_str()) {
            errors.push(format!("{path}.{key}: unknown field"));
            exact = false;
        }
    }
    exact
}

fn required_text<'a>(
    value: &'a Value,
    key: &str,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    match value.get(key).and_then(Value::as_str) {
        Some(text) if !text.trim().is_empty() => Some(text),
        _ => {
            errors.push(format!("{path}.{key}: expected a non-empty string"));
            None
        }
    }
}

fn validate_bounded_text(
    value: &Value,
    key: &str,
    maximum: usize,
    path: &str,
    errors: &mut Vec<String>,
) {
    match value.get(key).and_then(Value::as_str) {
        Some(text) if !text.trim().is_empty() && text.chars().count() <= maximum => {}
        _ => errors.push(format!(
            "{path}.{key}: expected a non-empty string of at most {maximum} characters"
        )),
    }
}

fn validate_token(value: Option<&Value>, path: &str, maximum: usize, errors: &mut Vec<String>) {
    match value.and_then(Value::as_str) {
        Some(token) if valid_token(token, maximum) => {}
        _ => errors.push(format!("{path}: expected an opaque stable identifier")),
    }
}

fn validate_identifier(
    value: Option<&Value>,
    prefix: &str,
    path: &str,
    maximum: usize,
    errors: &mut Vec<String>,
) {
    match value.and_then(Value::as_str) {
        Some(id) if valid_prefixed_id(id, prefix, maximum) => {}
        _ => errors.push(format!("{path}: expected a valid opaque identifier with prefix {prefix:?}")),
    }
}

fn valid_prefixed_id(value: &str, prefix: &str, maximum: usize) -> bool {
    value.len() <= maximum
        && value.strip_prefix(prefix).is_some_and(|suffix| {
            suffix.len() >= 8
                && suffix.as_bytes()[0].is_ascii_alphanumeric()
                && suffix.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
                })
        })
}

fn valid_feature_id(value: &str) -> bool {
    value.strip_prefix("feature-").is_some_and(|digits| {
        digits.len() >= 4
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && digits.parse::<u64>().is_ok()
    })
}

fn valid_metric_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.split('.').all(valid_token_segment)
}

fn valid_token_segment(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-')
        })
}

fn valid_lowercase_code(value: &str, maximum: usize) -> bool {
    let Some(first) = value.as_bytes().first() else {
        return false;
    };
    !value.is_empty()
        && value.len() <= maximum
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_token(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
}

fn valid_uri(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://") || value.starts_with("urn:")
}

fn require_uri(value: &Value, key: &str, path: &str, errors: &mut Vec<String>) {
    match value.get(key).and_then(Value::as_str) {
        Some(uri) if valid_uri(uri) => {}
        _ => errors.push(format!("{path}.{key}: expected an absolute HTTP(S) or URN URI")),
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_digest(value: &Value, key: &str, path: &str, errors: &mut Vec<String>) {
    match value.get(key).and_then(Value::as_str) {
        Some(digest) if valid_sha256(digest) => {}
        _ => errors.push(format!("{path}.{key}: expected lowercase 64-character SHA-256 hex")),
    }
}

fn required_digest<'a>(
    value: &'a Value,
    key: &str,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    match value.get(key).and_then(Value::as_str) {
        Some(digest) if valid_sha256(digest) => Some(digest),
        _ => {
            errors.push(format!("{path}.{key}: expected lowercase 64-character SHA-256 hex"));
            None
        }
    }
}

fn optional_digest<'a>(
    value: &'a Value,
    key: &str,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    match value.get(key) {
        Some(Value::Null) => None,
        Some(Value::String(digest)) if valid_sha256(digest) => Some(digest),
        _ => {
            errors.push(format!("{path}.{key}: expected null or lowercase SHA-256 hex"));
            None
        }
    }
}

fn required_u64(value: &Value, key: &str, path: &str, errors: &mut Vec<String>) -> Option<u64> {
    match value.get(key).and_then(Value::as_u64) {
        Some(number) => Some(number),
        None => {
            errors.push(format!("{path}.{key}: expected a non-negative integer"));
            None
        }
    }
}

fn require_string_eq(
    value: &Value,
    key: &str,
    expected: &str,
    path: &str,
    errors: &mut Vec<String>,
) {
    if value.get(key).and_then(Value::as_str) != Some(expected) {
        errors.push(format!("{path}.{key}: expected {expected:?}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../fixtures/landscape/observation-batch-v0.json"
        ))
        .expect("committed observation fixture must be valid JSON")
    }

    fn only_observation_mut(value: &mut Value) -> &mut Value {
        &mut value["observations"][0]
    }

    #[test]
    fn committed_fixture_is_valid_and_explicitly_synthetic() {
        let report = validate_batch(&fixture()).unwrap();
        assert_eq!(report.input_records, 5);
        assert_eq!(report.unique_observations, 5);
        assert_eq!(report.identical_duplicate_records, 0);
        assert_eq!(report.synthetic_observations, 5);
    }

    #[test]
    fn schema_constants_match_committed_json_schemas() {
        let observation_schema: Value = serde_json::from_str(include_str!(
            "../schemas/landscape-observation-v0.schema.json"
        ))
        .unwrap();
        let batch_schema: Value = serde_json::from_str(include_str!(
            "../schemas/landscape-observation-batch-v0.schema.json"
        ))
        .unwrap();
        assert_eq!(observation_schema["$id"], OBSERVATION_SCHEMA_URI);
        assert_eq!(
            observation_schema["properties"]["schema"]["const"],
            OBSERVATION_SCHEMA
        );
        assert_eq!(batch_schema["$id"], BATCH_SCHEMA_URI);
        assert_eq!(batch_schema["properties"]["schema"]["const"], BATCH_SCHEMA);
        assert_eq!(
            batch_schema["properties"]["observations"]["items"]["$ref"],
            OBSERVATION_SCHEMA_URI
        );
        assert_eq!(observation_schema["properties"]["location"]["oneOf"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn exact_reimport_is_idempotent_and_reported() {
        let mut batch = fixture();
        let duplicate = batch["observations"][0].clone();
        batch["observations"].as_array_mut().unwrap().push(duplicate);

        let (normalized, report) = normalize_batch(&batch).unwrap();
        assert_eq!(report.input_records, 6);
        assert_eq!(report.unique_observations, 5);
        assert_eq!(report.identical_duplicate_records, 1);
        assert_eq!(
            report.duplicate_observation_ids,
            vec!["obs-synthetic-plant-survival-0001"]
        );
        assert_eq!(normalized, fixture());
        assert_eq!(normalized["observations"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn conflicting_content_under_same_observation_id_fails_closed() {
        let mut batch = fixture();
        let mut conflicting = batch["observations"][0].clone();
        conflicting["result"]["numeric_value"] = json!(18);
        batch["observations"].as_array_mut().unwrap().push(conflicting);

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("conflicting content")));
    }

    #[test]
    fn quantity_result_and_observed_property_units_must_match() {
        let mut batch = fixture();
        only_observation_mut(&mut batch)["result"]["unit"] = json!("m");

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("does not match observed_property.unit")));
    }

    #[test]
    fn rejects_geo_axis_and_range_ambiguity() {
        let mut batch = fixture();
        only_observation_mut(&mut batch)["location"] = json!({
            "kind":"geographic-wgs84",
            "crs":"EPSG:4326",
            "longitude_latitude_deg":[181.0, 92.0],
            "precision_m":10.0,
            "sensitivity":"restricted"
        });

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("OGC:CRS84")));
        assert!(errors.iter().any(|error| error.contains("longitude in")));
        assert!(errors.iter().any(|error| error.contains("latitude in")));
    }

    #[test]
    fn raw_publication_of_sensitive_or_coarse_locations_is_blocked() {
        let mut batch = fixture();
        let obs = only_observation_mut(&mut batch);
        obs["privacy"]["classification"] = json!("public");
        obs["privacy"]["publication_policy"] = json!("public-approved");
        obs["privacy"]["consent_record_ref_sha256"] = json!(format!("{}","d".repeat(64)));
        obs["location"]["sensitivity"] = json!("restricted");

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("public location sensitivity")));
    }

    #[test]
    fn synthetic_observations_cannot_be_relabelled_as_field_data() {
        let mut batch = fixture();
        let obs = only_observation_mut(&mut batch);
        obs["evidence_class"] = json!("field-observation");
        obs["source"]["kind"] = json!("human-protocol");
        // Deliberately keep classification="synthetic": changing only the
        // evidence label must not launder generated data into field evidence.

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("synthetic classification requires evidence_class")));
        assert!(errors.iter().any(|error| error.contains("evidence_class")));
    }

    #[test]
    fn accepted_sensor_records_require_current_calibration() {
        let mut batch = fixture();
        let obs = only_observation_mut(&mut batch);
        obs["evidence_class"] = json!("sensor-observation");
        obs["source"]["kind"] = json!("sensor");
        obs["source"]["instrument_ref_sha256"] = json!("d".repeat(64));
        obs["quality"]["state"] = json!("accepted");
        obs["quality"]["calibration_state"] = json!("stale");
        obs["privacy"]["classification"] = json!("private");

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("current calibration")));
    }

    #[test]
    fn result_time_must_not_precede_measurement_window() {
        let mut batch = fixture();
        only_observation_mut(&mut batch)["result_time_unix_ms"] = json!(1792450000000_u64);
        batch["created_at_unix_ms"] = json!(1792450000000_u64);

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("must not precede")));
    }

    #[test]
    fn transformation_chain_must_start_at_source_payload_and_be_monotonic() {
        let mut batch = fixture();
        only_observation_mut(&mut batch)["transformations"] = json!([
            {
                "activity_uri":"urn:luminous:transform:aggregate",
                "activity_version":"v1",
                "input_sha256":"d".repeat(64),
                "output_sha256":"e".repeat(64),
                "performed_at_unix_ms":1792460000000_u64,
                "agent_ref_sha256":null
            }
        ]);

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("transformation chain is discontinuous")));
    }

    #[test]
    fn inverted_uncertainty_interval_is_rejected() {
        let mut batch = fixture();
        only_observation_mut(&mut batch)["quality"]["uncertainty"] = json!({
            "kind":"interval",
            "value":null,
            "lower_bound":20.0,
            "upper_bound":10.0,
            "coverage_probability":0.95,
            "unit":"count",
            "method_uri":"urn:luminous:uncertainty:fixture-v0",
            "reason_code":null
        });

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("lower_bound must be <= upper_bound")));
    }

    #[test]
    fn feature_reference_requires_pinned_project_snapshot() {
        let mut batch = fixture();
        only_observation_mut(&mut batch)["project_ref"]["project_snapshot_sha256"] = Value::Null;

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("requires project_snapshot_sha256")));
    }

    #[test]
    fn missing_required_nested_object_fails_without_panicking() {
        let mut batch = fixture();
        only_observation_mut(&mut batch).as_object_mut().unwrap().remove("quality");

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("missing required field")));
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let mut batch = fixture();
        only_observation_mut(&mut batch)["surprise"] = json!("silently ignored?");

        let errors = validate_batch(&batch).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("unknown field")));
    }
}
