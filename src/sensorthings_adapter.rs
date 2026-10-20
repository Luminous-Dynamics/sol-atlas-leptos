// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Read-only adapter for expanded OGC SensorThings API 1.1 Observation entities.
//!
//! This module performs no network requests and never copies FeatureOfInterest
//! coordinates. The caller supplies a digest of the exact source bytes and
//! privacy-safe site/project references. Profile v0 accepts finite numeric
//! results and RFC3339 times at millisecond precision; unsupported input fails
//! closed. SensorThings Tasking and actuator APIs are out of scope.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::landscape_observation::{
    BATCH_SCHEMA, BATCH_SCHEMA_URI, validate_batch,
};

const PROFILE_ID: &str = "urn:luminous:adapter:ogc-sensorthings-sensing-1.1-landscape-v0";
const OBSERVATION_SCHEMA: &str = "luminous.landscape-observation.v0";
const OBSERVATION_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-observation-v0.schema.json";

#[derive(Debug, Clone)]
pub struct SensorThingsAdapterContext {
    /// Absolute URI for the exact observation resource or pinned source response.
    pub source_uri: String,
    /// Pinned source/provider version, e.g. SensorThings-1.1.
    pub source_version: String,
    /// SHA-256 of the exact raw response bytes, computed by the caller.
    pub payload_sha256: String,
    /// Privacy-preserving instrument reference from a separate registry.
    pub instrument_ref_sha256: String,
    /// Optional privacy-preserving observer reference.
    pub observer_ref_sha256: Option<String>,
    /// Optional project snapshot digest; feature_id requires this to be present.
    pub project_snapshot_sha256: Option<String>,
    pub feature_id: Option<String>,
    /// Opaque site reference supplied by an authority-aware caller.
    pub site_ref_sha256: String,
    pub site_precision_class: String,
}

/// Convert one expanded SensorThings Observation to the offline native envelope.
///
/// This adapter is intentionally one-way and read-only. It does not infer
/// calibration, measurement uncertainty, consent, or publication permission.
/// The resulting observation remains unreviewed and private-only.
pub fn adapt_observation(
    input: &Value,
    context: &SensorThingsAdapterContext,
) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    validate_context(context, &mut errors);

    if !input.is_object() {
        return Err(vec![
            "$: SensorThings Observation must be a JSON object".to_string(),
        ]);
    }

    let observation_id = entity_id(input.get("@iot.id"), "$.@iot.id", &mut errors)
        .unwrap_or_default();

    let datastream = input.get("Datastream");
    if !datastream.is_some_and(Value::is_object) {
        errors.push(
            "$.Datastream: expanded Datastream object is required by profile v0".to_string(),
        );
    }
    let datastream = datastream.unwrap_or(&Value::Null);

    let observed_property = datastream.get("ObservedProperty");
    if !observed_property.is_some_and(Value::is_object) {
        errors.push(
            "$.Datastream.ObservedProperty: expanded ObservedProperty is required".to_string(),
        );
    }
    let observed_property = observed_property.unwrap_or(&Value::Null);

    let sensor = datastream.get("Sensor");
    if !sensor.is_some_and(Value::is_object) {
        errors.push("$.Datastream.Sensor: expanded Sensor is required".to_string());
    }
    let sensor = sensor.unwrap_or(&Value::Null);

    let sensor_id = entity_id(
        sensor.get("@iot.id"),
        "$.Datastream.Sensor.@iot.id",
        &mut errors,
    )
    .unwrap_or_default();

    let phenomenon_time = match input.get("phenomenonTime").and_then(Value::as_str) {
        Some(value) => match parse_phenomenon_time(value) {
            Ok(window) => Some(window),
            Err(error) => {
                errors.push(format!("$.phenomenonTime: {error}"));
                None
            }
        },
        None => {
            errors.push(
                "$.phenomenonTime: expected an RFC3339 instant or start/end interval".to_string(),
            );
            None
        }
    };
    let result_time = match input.get("resultTime").and_then(Value::as_str) {
        Some(value) => match parse_rfc3339_millis(value) {
            Ok(value) => Some(value),
            Err(error) => {
                errors.push(format!("$.resultTime: {error}"));
                None
            }
        },
        None => {
            errors.push("$.resultTime: expected an RFC3339 timestamp".to_string());
            None
        }
    };

    let numeric_result = match input.get("result").and_then(Value::as_f64) {
        Some(number) if number.is_finite() => Some(number),
        _ => {
            errors.push(
                "$.result: profile v0 maps only finite numeric results".to_string(),
            );
            None
        }
    };

    let property_id = required_uri(
        observed_property.get("definition"),
        "$.Datastream.ObservedProperty.definition",
        &mut errors,
    );
    let property_label = required_string(
        observed_property.get("name"),
        "$.Datastream.ObservedProperty.name",
        &mut errors,
    );
    if !datastream
        .get("unitOfMeasurement")
        .is_some_and(Value::is_object)
    {
        errors.push(
            "$.Datastream.unitOfMeasurement: expanded unit object is required".to_string(),
        );
    }
    let unit = required_string(
        datastream
            .get("unitOfMeasurement")
            .and_then(|value| value.get("symbol")),
        "$.Datastream.unitOfMeasurement.symbol",
        &mut errors,
    );
    let method_uri = required_uri(
        sensor.get("metadata"),
        "$.Datastream.Sensor.metadata",
        &mut errors,
    );

    if !errors.is_empty() {
        return Err(errors);
    }

    let (start_ms, end_ms) = phenomenon_time.expect("checked above");
    let result_time_ms = result_time.expect("checked above");
    let value = numeric_result.expect("checked above");
    let unit = unit.expect("checked above");
    // Context source_uri is the exact URI for this observation resource.
    let observation_uri = context.source_uri.clone();
    let native = json!({
        "schema": OBSERVATION_SCHEMA,
        "schema_uri": OBSERVATION_SCHEMA_URI,
        "observation_id": format!("obs-sensorthings-{observation_id}-v1"),
        "source_event_id": format!("sensorthings-observation-{observation_id}"),
        "project_ref": {
            "project_snapshot_sha256": context.project_snapshot_sha256,
            "feature_id": context.feature_id,
            "site_ref_sha256": context.site_ref_sha256
        },
        "phenomenon_time": {
            "start_unix_ms": start_ms,
            "end_unix_ms": end_ms,
            "clock_uncertainty_ms": 0
        },
        "result_time_unix_ms": result_time_ms,
        "observed_property": {
            "id": property_id.expect("checked above"),
            "label": property_label.expect("checked above"),
            "metric_id": null,
            "unit": unit
        },
        "result": {
            "kind": "quantity",
            "numeric_value": value,
            "unit": unit
        },
        "evidence_class": "sensor-observation",
        "source": {
            "kind": "sensor",
            "source_id": format!("sensor-{sensor_id}"),
            "source_uri": observation_uri,
            "source_version": context.source_version,
            "payload_sha256": context.payload_sha256,
            "method_uri": method_uri.expect("checked above"),
            "method_version": context.source_version,
            "observer_ref_sha256": context.observer_ref_sha256,
            "instrument_ref_sha256": context.instrument_ref_sha256
        },
        "quality": {
            "state": "unreviewed",
            "calibration_state": "unknown",
            "flags": ["adapter-mapped", "sensor-calibration-not-verified"],
            "uncertainty": {
                "kind": "unknown",
                "value": null,
                "lower_bound": null,
                "upper_bound": null,
                "coverage_probability": null,
                "unit": null,
                "method_uri": null,
                "reason_code": "source-uncertainty-not-provided"
            }
        },
        "location": {
            "kind": "opaque-site",
            "site_ref_sha256": context.site_ref_sha256,
            "precision_class": context.site_precision_class,
            "sensitivity": "restricted"
        },
        "privacy": {
            "classification": "private",
            "publication_policy": "private-only",
            "consent_record_ref_sha256": null,
            "policy_version": "landscape-privacy-v0",
            "consent_authority": "mycelix"
        },
        "transformations": [{
            "activity_uri": PROFILE_ID,
            "activity_version": "0",
            "input_sha256": context.payload_sha256,
            "output_sha256": context.payload_sha256,
            "performed_at_unix_ms": result_time_ms,
            "agent_ref_sha256": null
        }]
    });

    let mapped_payload_sha256 = sha256_json(&native).map_err(|error| {
        vec![format!("mapped observation could not be serialized for its digest: {error}")]
    })?;
    native["transformations"] = json!([{
        "activity_uri": PROFILE_ID,
        "activity_version": "0",
        "input_sha256": context.payload_sha256,
        "output_sha256": mapped_payload_sha256,
        "performed_at_unix_ms": result_time_ms,
        "agent_ref_sha256": null
    }]);

    let envelope = json!({
        "schema": BATCH_SCHEMA,
        "schema_uri": BATCH_SCHEMA_URI,
        "batch_id": "obs-batch-sensorthings-adapter-v0",
        "created_at_unix_ms": result_time_ms,
        "observations": [native]
    });
    validate_batch(&envelope).map_err(|batch_errors| {
        batch_errors
            .into_iter()
            .map(|error| format!("adapted record failed native validation: {error}"))
            .collect::<Vec<_>>()
    })?;

    Ok(envelope["observations"][0].clone())
}

/// SHA-256 over serde_json's deterministic Value serialization in this
/// pinned native contract. This is not RFC 8785/JCS; consumers must use this
/// declared digest profile rather than assuming cross-language canonical JSON.
fn sha256_json(value: &Value) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

fn validate_context(context: &SensorThingsAdapterContext, errors: &mut Vec<String>) {
    if !valid_uri(&context.source_uri) {
        errors.push("context.source_uri: expected absolute HTTP(S) or URN URI".to_string());
    }
    if context.source_version.trim().is_empty() || context.source_version.len() > 128 {
        errors.push("context.source_version: expected a pinned source version".to_string());
    }
    if !valid_sha256(&context.payload_sha256) {
        errors.push("context.payload_sha256: expected lowercase SHA-256 hex".to_string());
    }
    if !valid_sha256(&context.instrument_ref_sha256) {
        errors.push("context.instrument_ref_sha256: expected lowercase SHA-256 hex".to_string());
    }
    if context.observer_ref_sha256.as_deref().is_some_and(|value| !valid_sha256(value)) {
        errors.push("context.observer_ref_sha256: expected null or lowercase SHA-256 hex".to_string());
    }
    if context
        .project_snapshot_sha256
        .as_deref()
        .is_some_and(|value| !valid_sha256(value))
    {
        errors.push("context.project_snapshot_sha256: expected null or lowercase SHA-256 hex".to_string());
    }
    if context.project_snapshot_sha256.is_none() && context.feature_id.is_some() {
        errors.push("context.feature_id requires project_snapshot_sha256".to_string());
    }
    if context.feature_id.as_deref().is_some_and(|value| !valid_feature_id(value)) {
        errors.push("context.feature_id: invalid feature ID".to_string());
    }
    if !valid_sha256(&context.site_ref_sha256) {
        errors.push("context.site_ref_sha256: expected lowercase SHA-256 hex".to_string());
    }
    if !matches!(
        context.site_precision_class.as_str(),
        "site" | "parcel" | "neighbourhood" | "catchment" | "region" | "redacted"
    ) {
        errors.push("context.site_precision_class: unsupported precision class".to_string());
    }
}

fn entity_id(value: Option<&Value>, path: &str, errors: &mut Vec<String>) -> Option<String> {
    let id = match value {
        Some(Value::String(id)) if valid_identifier_token(id, 100) => Some(id.clone()),
        Some(Value::Number(number)) => number.as_u64().map(|value| value.to_string()),
        _ => None,
    };
    if id.is_none() {
        errors.push(format!(
            "{path}: expected a non-negative integer or stable opaque ID"
        ));
    }
    id
}

fn parse_phenomenon_time(value: &str) -> Result<(u64, u64), String> {
    if let Some((start, end)) = value.split_once('/') {
        if end.contains('/') {
            return Err("interval must contain exactly one slash separator".to_string());
        }
        let start_ms = parse_rfc3339_millis(start)?;
        let end_ms = parse_rfc3339_millis(end)?;
        if start_ms > end_ms {
            return Err("interval start must be <= interval end".to_string());
        }
        Ok((start_ms, end_ms))
    } else {
        let instant = parse_rfc3339_millis(value)?;
        Ok((instant, instant))
    }
}

/// Restricted RFC3339 parser. Requires a timezone and supports at most
/// millisecond precision so higher precision is rejected instead of truncated.
fn parse_rfc3339_millis(value: &str) -> Result<u64, String> {
    let bytes = value.as_bytes();
    if !value.is_ascii()
        || bytes.len() < 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || !matches!(bytes.get(10), Some(b'T' | b't'))
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return Err("expected RFC3339 date-time with explicit timezone".to_string());
    }

    let year = parse_digits(bytes, 0, 4)?;
    let month = parse_digits(bytes, 5, 2)?;
    let day = parse_digits(bytes, 8, 2)?;
    let hour = parse_digits(bytes, 11, 2)?;
    let minute = parse_digits(bytes, 14, 2)?;
    let second = parse_digits(bytes, 17, 2)?;
    if year == 0 || !(1..=12).contains(&month) {
        return Err("invalid calendar year or month".to_string());
    }
    if hour > 23 || minute > 59 || second > 59 {
        return Err("invalid clock time; leap seconds are not supported in v0".to_string());
    }
    if day == 0 || day > days_in_month(year, month) {
        return Err("invalid day for calendar month".to_string());
    }

    let mut index = 19usize;
    let mut fractional_ms = 0u64;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        let digits = index - fraction_start;
        if !(1..=3).contains(&digits) {
            return Err("fractional seconds must contain one to three digits".to_string());
        }
        let fraction = u64::from(parse_digits(bytes, fraction_start, digits)?);
        fractional_ms = match digits {
            1 => fraction * 100,
            2 => fraction * 10,
            3 => fraction,
            _ => unreachable!(),
        };
    }

    let offset_seconds = if index + 1 == bytes.len()
        && matches!(bytes[index], b'Z' | b'z')
    {
        0i64
    } else if index + 6 == bytes.len()
        && matches!(bytes[index], b'+' | b'-')
        && bytes[index + 3] == b':'
    {
        let offset_hour = parse_digits(bytes, index + 1, 2)?;
        let offset_minute = parse_digits(bytes, index + 4, 2)?;
        if offset_hour > 23 || offset_minute > 59 {
            return Err("invalid timezone offset".to_string());
        }
        let magnitude = i64::from(offset_hour * 3600 + offset_minute * 60);
        if bytes[index] == b'+' { magnitude } else { -magnitude }
    } else {
        return Err("timezone must be Z or numeric plus/minus HH:MM".to_string());
    };

    let days = days_from_civil(i64::from(year), month, day);
    let local_seconds = days * 86_400
        + i64::from(hour) * 3_600
        + i64::from(minute) * 60
        + i64::from(second);
    let utc_seconds = local_seconds - offset_seconds;
    if utc_seconds < 0 {
        return Err("pre-Unix-epoch timestamps are unsupported".to_string());
    }
    u64::try_from(utc_seconds)
        .ok()
        .and_then(|seconds| seconds.checked_mul(1_000))
        .and_then(|millis| millis.checked_add(fractional_ms))
        .ok_or_else(|| "timestamp exceeds supported range".to_string())
}

fn parse_digits(bytes: &[u8], start: usize, count: usize) -> Result<u32, String> {
    let Some(slice) = bytes.get(start..start + count) else {
        return Err("truncated timestamp component".to_string());
    };
    if !slice.iter().all(u8::is_ascii_digit) {
        return Err("timestamp component must contain decimal digits".to_string());
    }
    Ok(slice
        .iter()
        .fold(0u32, |value, digit| value * 10 + u32::from(*digit - b'0')))
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5
        + i64::from(day)
        - 1;
    let day_of_era =
        year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn required_string<'a>(
    value: Option<&'a Value>,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    match value.and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => Some(value),
        _ => {
            errors.push(format!("{path}: expected a non-empty string"));
            None
        }
    }
}

fn required_uri<'a>(
    value: Option<&'a Value>,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    match required_string(value, path, errors) {
        Some(uri) if valid_uri(uri) => Some(uri),
        Some(_) => {
            errors.push(format!("{path}: expected an absolute HTTP(S) or URN URI"));
            None
        }
        None => None,
    }
}

fn valid_uri(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://") || value.starts_with("urn:")
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_feature_id(value: &str) -> bool {
    value.strip_prefix("feature-").is_some_and(|digits| {
        digits.len() >= 4
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && digits.parse::<u64>().is_ok()
    })
}

fn valid_identifier_token(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn context() -> SensorThingsAdapterContext {
        SensorThingsAdapterContext {
            source_uri: "https://sensor.example.invalid/v1.1/Observations(71)".to_string(),
            source_version: "SensorThings-1.1".to_string(),
            payload_sha256: "b".repeat(64),
            instrument_ref_sha256: "c".repeat(64),
            observer_ref_sha256: None,
            project_snapshot_sha256: Some("a".repeat(64)),
            feature_id: Some("feature-0002".to_string()),
            site_ref_sha256: "d".repeat(64),
            site_precision_class: "parcel".to_string(),
        }
    }

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../fixtures/landscape/sensorthings-observation-v1-1.json"
        ))
        .expect("SensorThings fixture must be valid JSON")
    }

    #[test]
    fn adapts_numeric_expanded_sensorthings_observation_privately() {
        let adapted = adapt_observation(&fixture(), &context()).unwrap();
        assert_eq!(adapted["schema"], "luminous.landscape-observation.v0");
        assert_eq!(adapted["evidence_class"], "sensor-observation");
        assert_eq!(adapted["result"]["numeric_value"], 31.7);
        assert_eq!(adapted["result"]["unit"], "degC");
        assert_eq!(adapted["privacy"]["publication_policy"], "private-only");
        assert_eq!(adapted["quality"]["state"], "unreviewed");
        assert_eq!(adapted["quality"]["calibration_state"], "unknown");
        assert_eq!(adapted["location"]["kind"], "opaque-site");
        assert_eq!(adapted["source"]["source_uri"], context().source_uri);
        assert_eq!(adapted["transformations"][0]["input_sha256"], context().payload_sha256);
        assert_eq!(adapted["transformations"][0]["activity_uri"], PROFILE_ID);
        assert_ne!(
            adapted["transformations"][0]["output_sha256"],
            context().payload_sha256
        );
        assert!(adapted["location"].get("coordinates").is_none());
    }

    #[test]
    fn rfc3339_offset_and_milliseconds_normalize_to_utc() {
        assert_eq!(parse_rfc3339_millis("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(parse_rfc3339_millis("1970-01-01T01:00:00+01:00").unwrap(), 0);
        assert_eq!(parse_rfc3339_millis("1970-01-01T00:00:00.250Z").unwrap(), 250);
        assert_eq!(parse_rfc3339_millis("1970-01-01T02:30:00+02:30").unwrap(), 0);
    }

    #[test]
    fn rejects_unrepresentable_submillisecond_timestamp_precision() {
        assert!(parse_rfc3339_millis("2026-10-20T12:00:00.0001Z").is_err());
    }

    #[test]
    fn rejects_invalid_calendar_date_and_result_type() {
        assert!(parse_rfc3339_millis("2025-02-29T12:00:00Z").is_err());
        let mut input = fixture();
        input["result"] = json!("31.7");
        assert!(adapt_observation(&input, &context()).is_err());
    }

    #[test]
    fn rejects_incomplete_unit_or_sensor_metadata() {
        let mut input = fixture();
        input["Datastream"]["unitOfMeasurement"]["symbol"] = Value::Null;
        assert!(adapt_observation(&input, &context()).is_err());

        let mut input = fixture();
        input["Datastream"]["Sensor"]["metadata"] = json!("not-a-uri");
        assert!(adapt_observation(&input, &context()).is_err());
    }

    #[test]
    fn missing_raw_payload_digest_fails_closed() {
        let mut ctx = context();
        ctx.payload_sha256 = "unknown".to_string();
        assert!(adapt_observation(&fixture(), &ctx).is_err());
    }
}
