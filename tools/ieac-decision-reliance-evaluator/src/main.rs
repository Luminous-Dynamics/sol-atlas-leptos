// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bitemporal decision-reliance receipt composer.
//!
//! This command reports decision-time knowledge, retrospective decision-time
//! validity, and present-day reliance as three separate temporal assessments.
//!
//! This package composes normalized outputs from existing assurance authorities.
//! It does not authenticate their receipts, traverse dependency graphs, verify
//! engineering calculations, adjudicate rights, or authorize action.

use serde::Deserialize;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

const INPUT_SCHEMA: &str = "luminous.ieac-decision-reliance-input.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-decision-reliance-report.v0.1";
const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_GATES: usize = 11;
const MAX_EVENTS: usize = 10_000;

const GATE_NAMES: &[&str] = &[
    "source_coverage",
    "native_identity",
    "payload_integrity",
    "source_authenticity",
    "rights",
    "lifecycle",
    "impact_oracle",
    "engineering",
    "decision_authority",
    "inspection",
    "accepted_work",
];

#[derive(Debug)]
enum StrictJson {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<StrictJson>),
    Object(BTreeMap<String, StrictJson>),
}

impl StrictJson {
    fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(value),
            Self::Number(value) => Value::Number(value),
            Self::String(value) => Value::String(value),
            Self::Array(values) => Value::Array(values.into_iter().map(Self::into_value).collect()),
            Self::Object(values) => {
                let mut object = Map::new();
                for (key, value) in values {
                    object.insert(key, value.into_value());
                }
                Value::Object(object)
            }
        }
    }
}

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrictVisitor;

        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictJson;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("JSON with unique object member names")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Null)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Null)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Number(Number::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::Number(Number::from(value)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Number::from_f64(value)
                    .map(StrictJson::Number)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(StrictJson::String(value))
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element::<StrictJson>()? {
                    values.push(value);
                }
                Ok(StrictJson::Array(values))
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = access.next_entry::<String, StrictJson>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!(
                            "duplicate JSON object member: {key}"
                        )));
                    }
                    values.insert(key, value);
                }
                Ok(StrictJson::Object(values))
            }
        }

        deserializer.deserialize_any(StrictVisitor)
    }
}

fn parse_strict_json(input: &str) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let parsed = StrictJson::deserialize(&mut deserializer)
        .map_err(|error| format!("strict JSON parse failed: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("trailing JSON data: {error}"))?;
    Ok(parsed.into_value())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn canonical_json_sha256(value: &Value) -> Result<String, String> {
    let bytes =
        serde_json::to_vec(value).map_err(|error| format!("JSON serialization failed: {error}"))?;
    Ok(sha256_hex(&bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

/// Accepts canonical UTC whole-second timestamps only. Lexicographic
/// comparison is then chronological and cannot be confused by local offsets.
fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }

    for (index, byte) in bytes.iter().enumerate() {
        if !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit() {
            return false;
        }
    }

    let part = |start: usize, end: usize| {
        std::str::from_utf8(&bytes[start..end])
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
    };
    let year = part(0, 4).unwrap_or(0);
    let month = part(5, 7).unwrap_or(0);
    let day = part(8, 10).unwrap_or(0);
    let hour = part(11, 13).unwrap_or(99);
    let minute = part(14, 16).unwrap_or(99);
    let second = part(17, 19).unwrap_or(99);

    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }

    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    day >= 1 && day <= max_day
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn non_empty_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
}

fn is_hash_at(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(Value::as_str)
        .is_some_and(valid_sha256)
}

fn push_error(errors: &mut Vec<String>, code: &str, detail: impl AsRef<str>) {
    errors.push(format!("{code}: {}", detail.as_ref()));
}

fn validate_decision_and_top_level(input: &Value, errors: &mut Vec<String>) -> Result<(), String> {
    if non_empty_string(input, "schema_version") != Some(INPUT_SCHEMA) {
        push_error(
            errors,
            "UNSUPPORTED_INPUT_SCHEMA",
            "schema_version is not v0.1",
        );
    }
    if non_empty_string(input, "assessment_id").is_none() {
        push_error(errors, "ASSESSMENT_ID_MISSING", "assessment_id is required");
    }

    let decision = input.get("decision").ok_or("decision object is missing")?;
    let decision_at =
        non_empty_string(decision, "decided_at").ok_or("decision.decided_at missing")?;
    let evaluated_at = non_empty_string(input, "evaluated_at").ok_or("evaluated_at missing")?;

    if non_empty_string(decision, "decision_id").is_none()
        || non_empty_string(decision, "purpose").is_none()
        || non_empty_string(decision, "target_node_id").is_none()
    {
        push_error(
            errors,
            "DECISION_IDENTITY_INCOMPLETE",
            "decision ID, target node and purpose are required",
        );
    }
    if !is_hash_at(decision, "target_artifact_sha256") || !is_hash_at(decision, "scope_sha256") {
        push_error(
            errors,
            "DECISION_DIGEST_INVALID",
            "decision artifact and scope digests must be lowercase SHA-256",
        );
    }
    if !valid_utc_timestamp(decision_at) || !valid_utc_timestamp(evaluated_at) {
        push_error(
            errors,
            "DECISION_TIME_INVALID",
            "decision and evaluation times must be canonical UTC whole-second timestamps",
        );
    } else if decision_at > evaluated_at {
        push_error(
            errors,
            "EVALUATION_PRECEDES_DECISION",
            "evaluated_at cannot precede decided_at",
        );
    }

    let required = input
        .get("required_gates")
        .and_then(Value::as_array)
        .ok_or("required_gates must be an array")?;
    if required.is_empty() || required.len() > MAX_GATES {
        push_error(
            errors,
            "REQUIRED_GATE_COUNT_INVALID",
            "required_gates must contain 1..=11 entries",
        );
    }
    let mut unique = BTreeSet::new();
    for gate in required {
        let Some(name) = gate.as_str() else {
            push_error(
                errors,
                "REQUIRED_GATE_NAME_INVALID",
                "required gate names must be strings",
            );
            continue;
        };
        if !GATE_NAMES.contains(&name) {
            push_error(errors, "UNKNOWN_REQUIRED_GATE", name);
        }
        if !unique.insert(name.to_owned()) {
            push_error(errors, "DUPLICATE_REQUIRED_GATE", name);
        }
    }

    let views = input.get("views").ok_or("views object is missing")?;
    for name in [
        "historical_knowledge",
        "retrospective_at_decision",
        "current_reliance",
    ] {
        if !views.get(name).is_some_and(Value::is_object) {
            push_error(errors, "REQUIRED_VIEW_MISSING", name);
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct GateRow {
    gate: String,
    status: String,
    receipt_sha256: Option<String>,
    verifier_build_sha256: Option<String>,
    scope_sha256: String,
    checked_at: String,
    knowledge_available_at: String,
    valid_until: Option<String>,
    reason_codes: Vec<String>,
}

impl GateRow {
    fn to_json(&self) -> Value {
        json!({
            "gate": self.gate,
            "status": self.status,
            "receipt_sha256": self.receipt_sha256,
            "verifier_build_sha256": self.verifier_build_sha256,
            "scope_sha256": self.scope_sha256,
            "checked_at": self.checked_at,
            "knowledge_available_at": self.knowledge_available_at,
            "valid_until": self.valid_until,
            "reason_codes": self.reason_codes
        })
    }
}

fn parse_gate_row(
    raw: &Value,
    scope_sha256: &str,
    knowledge_cutoff_at: &str,
    valid_time_as_of: &str,
    errors: &mut Vec<String>,
) -> Option<GateRow> {
    let gate = non_empty_string(raw, "gate")?.to_owned();
    let mut status = non_empty_string(raw, "status")?.to_owned();

    if !GATE_NAMES.contains(&gate.as_str()) {
        push_error(errors, "UNKNOWN_GATE", &gate);
    }
    if ![
        "qualified",
        "failed",
        "unknown",
        "revalidation_required",
        "quarantined",
        "not_applicable",
    ]
    .contains(&status.as_str())
    {
        push_error(errors, "UNKNOWN_GATE_STATUS", format!("{gate}: {status}"));
    }

    let row_scope = non_empty_string(raw, "scope_sha256")?.to_owned();
    if !valid_sha256(&row_scope) || row_scope != scope_sha256 {
        push_error(
            errors,
            "GATE_SCOPE_MISMATCH",
            format!("{gate} does not bind the decision scope"),
        );
    }

    let checked_at = non_empty_string(raw, "checked_at")?.to_owned();
    let knowledge_available_at = non_empty_string(raw, "knowledge_available_at")?.to_owned();
    if !valid_utc_timestamp(&checked_at) || !valid_utc_timestamp(&knowledge_available_at) {
        push_error(errors, "GATE_TIME_INVALID", &gate);
    } else {
        if checked_at.as_str() > knowledge_cutoff_at {
            push_error(
                errors,
                "FUTURE_INFORMATION_LEAK",
                format!("{gate} was checked after the view's knowledge cutoff"),
            );
        }
        if knowledge_available_at.as_str() > knowledge_cutoff_at {
            push_error(
                errors,
                "FUTURE_KNOWLEDGE_LEAK",
                format!("{gate} became available after the view's knowledge cutoff"),
            );
        }
        if knowledge_available_at > checked_at {
            push_error(
                errors,
                "KNOWLEDGE_AFTER_CHECK",
                format!("{gate} cannot be checked before its evidence became available"),
            );
        }
    }

    let valid_until = match raw.get("valid_until") {
        Some(Value::Null) | None => None,
        Some(Value::String(value)) if valid_utc_timestamp(value) => Some(value.clone()),
        _ => {
            push_error(errors, "GATE_VALID_UNTIL_INVALID", &gate);
            None
        }
    };
    if let Some(until) = valid_until.as_deref() {
        if until < checked_at.as_str() {
            push_error(
                errors,
                "GATE_EXPIRY_PRECEDES_CHECK",
                format!("{gate} expires before it was checked"),
            );
        }
        if until < valid_time_as_of && status == "qualified" {
            status = "revalidation_required".into();
        }
    }

    let receipt_sha256 = match raw.get("receipt_sha256") {
        Some(Value::String(value)) if valid_sha256(value) => Some(value.clone()),
        Some(Value::Null) | None => None,
        _ => {
            push_error(errors, "GATE_RECEIPT_DIGEST_INVALID", &gate);
            None
        }
    };
    let verifier_build_sha256 = match raw.get("verifier_build_sha256") {
        Some(Value::String(value)) if valid_sha256(value) => Some(value.clone()),
        Some(Value::Null) | None => None,
        _ => {
            push_error(errors, "GATE_VERIFIER_DIGEST_INVALID", &gate);
            None
        }
    };

    if matches!(
        status.as_str(),
        "qualified" | "failed" | "revalidation_required" | "quarantined"
    ) && (receipt_sha256.is_none() || verifier_build_sha256.is_none())
    {
        push_error(
            errors,
            "GATE_EVIDENCE_REFERENCE_MISSING",
            format!("{gate} status {status} requires receipt and verifier digests"),
        );
    }

    let reason_codes = raw
        .get("reason_codes")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(GateRow {
        gate,
        status,
        receipt_sha256,
        verifier_build_sha256,
        scope_sha256: row_scope,
        checked_at,
        knowledge_available_at,
        valid_until,
        reason_codes,
    })
}

#[derive(Clone, Debug)]
struct EventEffect {
    event_sha256: String,
    target_node_id: String,
    cause_class: String,
    effect: String,
    reason_code: String,
}

impl EventEffect {
    fn to_json(&self) -> Value {
        json!({
            "event_sha256": self.event_sha256,
            "target_node_id": self.target_node_id,
            "cause_class": self.cause_class,
            "effect": self.effect,
            "reason_code": self.reason_code
        })
    }
}

fn validate_temporal_events(
    input: &Value,
    evaluated_at: &str,
    errors: &mut Vec<String>,
) -> Result<Vec<Value>, String> {
    let events = input
        .get("temporal_events")
        .and_then(Value::as_array)
        .ok_or("temporal_events must be an array")?;
    if events.len() > MAX_EVENTS {
        push_error(
            errors,
            "EVENT_COUNT_EXCEEDED",
            "more than 10,000 temporal events",
        );
    }

    let mut seen = BTreeSet::new();
    for event in events {
        let event_sha = non_empty_string(event, "event_sha256").unwrap_or_default();
        let target_sha = non_empty_string(event, "target_artifact_sha256").unwrap_or_default();
        let impact_sha = non_empty_string(event, "impact_report_sha256").unwrap_or_default();
        let scope_sha = non_empty_string(event, "scope_sha256").unwrap_or_default();
        let target_node = non_empty_string(event, "target_node_id").unwrap_or_default();
        let cause = non_empty_string(event, "cause_class").unwrap_or_default();
        let recorded_at = non_empty_string(event, "recorded_at").unwrap_or_default();
        let authority = non_empty_string(event, "authority_disposition").unwrap_or_default();
        let confidence = non_empty_string(event, "effective_time_confidence").unwrap_or_default();
        let impact = non_empty_string(event, "impact_disposition").unwrap_or_default();

        if !valid_sha256(event_sha) || !valid_sha256(target_sha) || !valid_sha256(impact_sha) {
            push_error(
                errors,
                "EVENT_DIGEST_INVALID",
                "event, target, or impact report digest is invalid",
            );
        }
        if !valid_sha256(scope_sha) {
            push_error(
                errors,
                "EVENT_SCOPE_DIGEST_INVALID",
                "event scope digest is invalid",
            );
        }
        if target_node.is_empty() || cause.is_empty() {
            push_error(
                errors,
                "EVENT_IDENTITY_MISSING",
                "event target and cause class are required",
            );
        }
        if !seen.insert(event_sha.to_owned()) {
            push_error(errors, "DUPLICATE_TEMPORAL_EVENT", event_sha);
        }
        if !valid_utc_timestamp(recorded_at) || recorded_at > evaluated_at {
            push_error(errors, "EVENT_RECORDING_TIME_INVALID", event_sha);
        }

        let effective_at = match event.get("effective_at") {
            Some(Value::String(value)) if valid_utc_timestamp(value) => Some(value.as_str()),
            Some(Value::Null) | None => None,
            _ => {
                push_error(errors, "EVENT_EFFECTIVE_TIME_INVALID", event_sha);
                None
            }
        };
        let trusted_at = match event.get("trusted_at") {
            Some(Value::String(value)) if valid_utc_timestamp(value) => Some(value.as_str()),
            Some(Value::Null) | None => None,
            _ => {
                push_error(errors, "EVENT_TRUSTED_TIME_INVALID", event_sha);
                None
            }
        };

        if !["verified", "authority_asserted", "unknown", "disputed"].contains(&confidence) {
            push_error(errors, "EVENT_EFFECTIVE_CONFIDENCE_INVALID", event_sha);
        }
        if !["authenticated", "unauthenticated", "unknown", "revoked"].contains(&authority) {
            push_error(errors, "EVENT_AUTHORITY_DISPOSITION_INVALID", event_sha);
        }
        if ![
            "affected",
            "unaffected",
            "revalidation_required",
            "quarantined",
            "not_checked",
        ]
        .contains(&impact)
        {
            push_error(errors, "EVENT_IMPACT_DISPOSITION_INVALID", event_sha);
        }

        if confidence == "verified" && effective_at.is_none() {
            push_error(errors, "VERIFIED_EFFECTIVE_TIME_MISSING", event_sha);
        }
        if authority == "authenticated" && trusted_at.is_none() {
            push_error(
                errors,
                "AUTHENTICATED_EVENT_TRUSTED_TIME_MISSING",
                event_sha,
            );
        }
        if let Some(trusted) = trusted_at
            && trusted < recorded_at
        {
            push_error(errors, "TRUSTED_EVENT_TIME_REVERSAL", event_sha);
        }
    }
    Ok(events.clone())
}

fn event_effect_for_view(
    event: &Value,
    valid_time_as_of: &str,
    knowledge_cutoff_at: &str,
    decision_scope_sha256: &str,
) -> Option<EventEffect> {
    let event_sha256 = non_empty_string(event, "event_sha256")?.to_owned();
    let target_node_id = non_empty_string(event, "target_node_id")?.to_owned();
    let cause_class = non_empty_string(event, "cause_class")?.to_owned();
    let recorded_at = non_empty_string(event, "recorded_at")?;
    let scope_sha256 = non_empty_string(event, "scope_sha256")?;
    let authority = non_empty_string(event, "authority_disposition")?;
    let confidence = non_empty_string(event, "effective_time_confidence")?;
    let impact = non_empty_string(event, "impact_disposition")?;

    // Events not yet known at the view's system-time cutoff cannot affect that
    // historical knowledge view. They may still affect retrospective/current.
    if recorded_at > knowledge_cutoff_at || impact == "unaffected" {
        return None;
    }

    if scope_sha256 != decision_scope_sha256 {
        return Some(EventEffect {
            event_sha256,
            target_node_id,
            cause_class,
            effect: "quarantined".into(),
            reason_code: "EVENT_SCOPE_NOT_EQUIVALENT_TO_DECISION_SCOPE".into(),
        });
    }

    let effective_at = event.get("effective_at").and_then(Value::as_str);
    if authority != "authenticated" || confidence != "verified" || effective_at.is_none() {
        return Some(EventEffect {
            event_sha256,
            target_node_id,
            cause_class,
            effect: "quarantined".into(),
            reason_code: "EVENT_AUTHORITY_OR_EFFECTIVE_TIME_NOT_QUALIFIED".into(),
        });
    }

    if effective_at.is_some_and(|effective| effective > valid_time_as_of) {
        return None;
    }

    let (effect, reason_code) = match impact {
        "affected" => (
            "claim_invalidated",
            "VERIFIED_IN_SCOPE_EVENT_AFFECTS_DECISION_DEPENDENCY",
        ),
        "revalidation_required" => (
            "revalidation_required",
            "VERIFIED_EVENT_REQUIRES_DOWNSTREAM_REVALIDATION",
        ),
        "quarantined" => ("quarantined", "UPSTREAM_IMPACT_REPORT_QUARANTINED_EVENT"),
        _ => ("indeterminate", "UPSTREAM_IMPACT_NOT_CHECKED"),
    };

    Some(EventEffect {
        event_sha256,
        target_node_id,
        cause_class,
        effect: effect.into(),
        reason_code: reason_code.into(),
    })
}

#[derive(Clone, Debug)]
struct ViewResult {
    disposition: String,
    valid_time_as_of: String,
    knowledge_cutoff_at: String,
    gate_results: Vec<GateRow>,
    missing_required_gates: Vec<String>,
    qualified_count: usize,
    failed_count: usize,
    unknown_count: usize,
    revalidation_required_count: usize,
    quarantined_count: usize,
    event_effects: Vec<EventEffect>,
}

impl ViewResult {
    fn to_json(&self) -> Value {
        json!({
            "valid_time_as_of": self.valid_time_as_of,
            "knowledge_cutoff_at": self.knowledge_cutoff_at,
            "disposition": self.disposition,
            "gate_results": self.gate_results.iter().map(GateRow::to_json).collect::<Vec<_>>(),
            "qualified_count": self.qualified_count,
            "failed_count": self.failed_count,
            "unknown_count": self.unknown_count,
            "revalidation_required_count": self.revalidation_required_count,
            "quarantined_count": self.quarantined_count,
            "missing_required_gates": self.missing_required_gates,
            "event_effects": self.event_effects.iter().map(EventEffect::to_json).collect::<Vec<_>>()
        })
    }
}

fn assess_view(
    name: &str,
    raw_view: &Value,
    required_gates: &[String],
    decision_scope_sha256: &str,
    events: &[Value],
    errors: &mut Vec<String>,
) -> Result<ViewResult, String> {
    let valid_time_as_of = non_empty_string(raw_view, "valid_time_as_of")
        .ok_or_else(|| format!("{name}.valid_time_as_of missing"))?
        .to_owned();
    let knowledge_cutoff_at = non_empty_string(raw_view, "knowledge_cutoff_at")
        .ok_or_else(|| format!("{name}.knowledge_cutoff_at missing"))?
        .to_owned();
    if !valid_utc_timestamp(&valid_time_as_of) || !valid_utc_timestamp(&knowledge_cutoff_at) {
        push_error(errors, "VIEW_TIME_INVALID", name);
    }

    let raw_gates = raw_view
        .get("gate_results")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{name}.gate_results must be an array"))?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for raw in raw_gates {
        if let Some(row) = parse_gate_row(
            raw,
            decision_scope_sha256,
            &knowledge_cutoff_at,
            &valid_time_as_of,
            errors,
        ) {
            if !seen.insert(row.gate.clone()) {
                push_error(
                    errors,
                    "DUPLICATE_GATE_RESULT",
                    format!("{name}: {}", row.gate),
                );
            } else {
                rows.push(row);
            }
        } else {
            push_error(errors, "GATE_RESULT_MALFORMED", name);
        }
    }

    let mut missing = Vec::new();
    for required in required_gates {
        match rows.iter().find(|row| &row.gate == required) {
            None => missing.push(required.clone()),
            Some(row) if row.status == "not_applicable" => missing.push(required.clone()),
            _ => {}
        }
    }

    let mut qualified_count = 0usize;
    let mut failed_count = 0usize;
    let mut unknown_count = missing.len();
    let mut revalidation_required_count = 0usize;
    let mut quarantined_count = 0usize;
    for row in &rows {
        if !required_gates.contains(&row.gate) {
            continue;
        }
        match row.status.as_str() {
            "qualified" => qualified_count += 1,
            "failed" => failed_count += 1,
            "unknown" | "not_applicable" => unknown_count += 1,
            "revalidation_required" => revalidation_required_count += 1,
            "quarantined" => quarantined_count += 1,
            _ => unknown_count += 1,
        }
    }

    let event_effects = events
        .iter()
        .filter_map(|event| {
            event_effect_for_view(
                event,
                &valid_time_as_of,
                &knowledge_cutoff_at,
                decision_scope_sha256,
            )
        })
        .collect::<Vec<_>>();

    let event_quarantined = event_effects
        .iter()
        .any(|effect| effect.effect == "quarantined");
    let event_invalidated = event_effects
        .iter()
        .any(|effect| effect.effect == "claim_invalidated");
    let event_revalidation = event_effects
        .iter()
        .any(|effect| effect.effect == "revalidation_required");
    let event_indeterminate = event_effects
        .iter()
        .any(|effect| effect.effect == "indeterminate");

    let disposition = if event_quarantined || quarantined_count > 0 {
        "quarantined_pending_review"
    } else if event_invalidated || failed_count > 0 {
        "claim_invalidated"
    } else if event_revalidation || revalidation_required_count > 0 {
        "revalidation_required"
    } else if event_indeterminate || unknown_count > 0 {
        if name == "historical_knowledge" || name == "retrospective_at_decision" {
            "historical_basis_incomplete"
        } else {
            "indeterminate"
        }
    } else {
        "candidate_current_within_declared_scope"
    };

    Ok(ViewResult {
        disposition: disposition.to_owned(),
        valid_time_as_of,
        knowledge_cutoff_at,
        gate_results: rows,
        missing_required_gates: missing,
        qualified_count,
        failed_count,
        unknown_count,
        revalidation_required_count,
        quarantined_count,
        event_effects,
    })
}

fn obligation_for_gate(gate: &str) -> &'static str {
    match gate {
        "source_coverage" => "restore_coverage",
        "native_identity" | "payload_integrity" => "reverify_source",
        "source_authenticity" => "reverify_signature_and_trust",
        "rights" => "reverify_rights",
        "lifecycle" | "impact_oracle" => "manual_review",
        "engineering" => "recompute_engineering",
        "decision_authority" => "review_authority",
        "inspection" | "accepted_work" => "reinspect_physical_work",
        _ => "manual_review",
    }
}

fn derive_obligations(views: &[ViewResult], decision_node_id: &str) -> Vec<Value> {
    let mut unique = BTreeSet::new();
    let mut output = Vec::new();

    for view in views {
        for gate in &view.gate_results {
            if matches!(
                gate.status.as_str(),
                "failed" | "unknown" | "revalidation_required" | "quarantined"
            ) {
                let reason = gate.reason_codes.first().cloned().unwrap_or_else(|| {
                    format!(
                        "{}_{}",
                        gate.gate.to_uppercase(),
                        gate.status.to_uppercase()
                    )
                });
                let target = decision_node_id.to_owned();
                let obligation = obligation_for_gate(&gate.gate);
                if unique.insert((target.clone(), obligation.to_owned(), reason.clone())) {
                    output.push(json!({
                        "target_node_id": target,
                        "obligation_class": obligation,
                        "reason_code": reason
                    }));
                }
            }
        }
        for effect in &view.event_effects {
            if effect.effect != "unaffected" {
                let obligation = if effect.effect == "claim_invalidated" {
                    "manual_review"
                } else {
                    "resolve_dispute"
                };
                if unique.insert((
                    effect.target_node_id.clone(),
                    obligation.to_owned(),
                    effect.reason_code.clone(),
                )) {
                    output.push(json!({
                        "target_node_id": effect.target_node_id,
                        "obligation_class": obligation,
                        "reason_code": effect.reason_code
                    }));
                }
            }
        }
    }

    output.sort_by(|left, right| {
        (
            left["target_node_id"].as_str(),
            left["obligation_class"].as_str(),
            left["reason_code"].as_str(),
        )
            .cmp(&(
                right["target_node_id"].as_str(),
                right["obligation_class"].as_str(),
                right["reason_code"].as_str(),
            ))
    });
    output
}

fn evaluate(input: &Value) -> Result<Value, String> {
    let mut errors = Vec::new();
    validate_decision_and_top_level(input, &mut errors)?;
    let decision = input.get("decision").ok_or("decision object missing")?;
    let decision_at = non_empty_string(decision, "decided_at").ok_or("decision time missing")?;
    let evaluated_at = non_empty_string(input, "evaluated_at").ok_or("evaluation time missing")?;
    let scope_sha256 = non_empty_string(decision, "scope_sha256").ok_or("scope digest missing")?;
    let target_node_id =
        non_empty_string(decision, "target_node_id").ok_or("decision target missing")?;

    let required_gates = input
        .get("required_gates")
        .and_then(Value::as_array)
        .ok_or("required_gates missing")?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let events = validate_temporal_events(input, evaluated_at, &mut errors)?;
    let views = input.get("views").ok_or("views missing")?;

    let historical_raw = views
        .get("historical_knowledge")
        .ok_or("historical view missing")?;
    let retrospective_raw = views
        .get("retrospective_at_decision")
        .ok_or("retrospective view missing")?;
    let current_raw = views
        .get("current_reliance")
        .ok_or("current view missing")?;

    let expected_times = [
        ("historical_knowledge", decision_at, decision_at),
        ("retrospective_at_decision", decision_at, evaluated_at),
        ("current_reliance", evaluated_at, evaluated_at),
    ];
    for (name, expected_valid, expected_knowledge) in expected_times {
        let raw = views.get(name).ok_or_else(|| format!("{name} missing"))?;
        if string_at(raw, &["valid_time_as_of"]) != Some(expected_valid)
            || string_at(raw, &["knowledge_cutoff_at"]) != Some(expected_knowledge)
        {
            push_error(
                &mut errors,
                "VIEW_TIME_AXIS_MISMATCH",
                format!(
                    "{name} must bind valid-time {expected_valid} and knowledge-time {expected_knowledge}"
                ),
            );
        }
    }

    let mut results = Vec::new();
    for (name, raw) in [
        ("historical_knowledge", historical_raw),
        ("retrospective_at_decision", retrospective_raw),
        ("current_reliance", current_raw),
    ] {
        let result = assess_view(
            name,
            raw,
            &required_gates,
            scope_sha256,
            &events,
            &mut errors,
        )?;
        results.push(result);
    }

    let continuity_assessment = if !errors.is_empty() {
        "invalid_input"
    } else {
        let first = &results[0].disposition;
        let all_same = results.iter().all(|result| &result.disposition == first);
        if all_same && first == "candidate_current_within_declared_scope" {
            "views_consistent_candidate"
        } else if all_same {
            "views_consistent_non_candidate"
        } else {
            "temporal_views_diverge"
        }
    };

    let input_sha256 = canonical_json_sha256(input)?;
    let report_id = format!("decision-reliance-{}", &input_sha256[..24]);
    let obligations = derive_obligations(&results, target_node_id);

    let decision_output = json!({
        "decision_id": non_empty_string(decision, "decision_id").unwrap_or_default(),
        "target_node_id": target_node_id,
        "target_artifact_sha256": non_empty_string(decision, "target_artifact_sha256").unwrap_or_default(),
        "decided_at": decision_at,
        "scope_sha256": scope_sha256,
        "purpose": non_empty_string(decision, "purpose").unwrap_or_default()
    });

    let mut report = json!({
        "schema_version": REPORT_SCHEMA,
        "report_id": report_id,
        "input_sha256": input_sha256,
        "decision": decision_output,
        "evaluated_at": evaluated_at,
        "views": {
            "historical_knowledge": results[0].to_json(),
            "retrospective_at_decision": results[1].to_json(),
            "current_reliance": results[2].to_json()
        },
        "temporal_events": events,
        "historical_record_preserved": true,
        "input_authentication": "not_performed",
        "impact_recomputation": "not_performed",
        "action_authority": "none",
        "claim_ceiling": "upstream_receipt_composition_only",
        "not_checked": [
            "upstream receipt signatures and trust roots are not authenticated by this composer",
            "source payload bytes are not fetched or re-hashed by this composer",
            "dependency traversal and impact-oracle computation are not repeated",
            "engineering calculations and metrology are not recomputed",
            "rights evidence and legal permissions are not independently adjudicated",
            "professional approval, construction release, and operational actuation are not granted"
        ],
        "obligations": obligations,
        "continuity_assessment": continuity_assessment,
        "validation_errors": errors
    });
    let report_core_digest = canonical_json_sha256(&report)?;
    report["report_sha256"] = json!(report_core_digest);
    Ok(report)
}

fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.len() != 1 {
        return Err("usage: ieac-decision-reliance <input.json>".into());
    }
    let metadata =
        fs::metadata(&arguments[0]).map_err(|error| format!("cannot stat input file: {error}"))?;
    if metadata.len() > MAX_INPUT_BYTES {
        return Err(format!("input file exceeds {MAX_INPUT_BYTES} bytes"));
    }
    let content = fs::read_to_string(&arguments[0])
        .map_err(|error| format!("cannot read input file: {error}"))?;
    let input = parse_strict_json(&content)?;
    let report = evaluate(&input)?;
    let output = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("cannot serialize report: {error}"))?;
    println!("{output}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr(), "ieac-decision-reliance: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn gate(name: &str, status: &str, index: usize, checked: &str, available: &str) -> Value {
        json!({
            "gate": name,
            "status": status,
            "receipt_sha256": digest(char::from_digit((index % 10) as u32, 10).unwrap()),
            "verifier_build_sha256": digest('b'),
            "scope_sha256": digest('c'),
            "checked_at": checked,
            "knowledge_available_at": available,
            "valid_until": "2027-01-01T00:00:00Z",
            "reason_codes": []
        })
    }

    fn base_input() -> Value {
        let required = GATE_NAMES.to_vec();
        let decision_at = "2026-11-02T10:00:00Z";
        let evaluated_at = "2026-11-04T11:00:00Z";
        let historical = required
            .iter()
            .enumerate()
            .map(|(index, name)| gate(name, "qualified", index + 1, decision_at, decision_at))
            .collect::<Vec<_>>();
        let retrospective = required
            .iter()
            .enumerate()
            .map(|(index, name)| gate(name, "qualified", index + 1, evaluated_at, evaluated_at))
            .collect::<Vec<_>>();
        json!({
            "schema_version": INPUT_SCHEMA,
            "assessment_id": "assessment-test-001",
            "decision": {
                "decision_id": "decision-test-001",
                "target_node_id": format!("decision:sha256:{}", digest('d')),
                "target_artifact_sha256": digest('e'),
                "decided_at": decision_at,
                "scope_sha256": digest('c'),
                "purpose": "engineering_review"
            },
            "evaluated_at": evaluated_at,
            "required_gates": required,
            "views": {
                "historical_knowledge": {
                    "valid_time_as_of": decision_at,
                    "knowledge_cutoff_at": decision_at,
                    "gate_results": historical
                },
                "retrospective_at_decision": {
                    "valid_time_as_of": decision_at,
                    "knowledge_cutoff_at": evaluated_at,
                    "gate_results": retrospective
                },
                "current_reliance": {
                    "valid_time_as_of": evaluated_at,
                    "knowledge_cutoff_at": evaluated_at,
                    "gate_results": retrospective.clone()
                }
            },
            "temporal_events": [],
            "limitations": ["Synthetic test only."]
        })
    }

    fn result_disposition<'a>(report: &'a Value, view: &str) -> &'a str {
        report["views"][view]["disposition"].as_str().unwrap()
    }

    /// Deliberately simple test-only oracle. It scans required gates and
    /// temporal events directly and does not call production assessment helpers.
    fn reference_view_disposition(input: &Value, view_name: &str) -> String {
        let decision = &input["decision"];
        let scope = decision["scope_sha256"].as_str().unwrap();
        let view = &input["views"][view_name];
        let valid_time = view["valid_time_as_of"].as_str().unwrap();
        let knowledge_cutoff = view["knowledge_cutoff_at"].as_str().unwrap();
        let required = input["required_gates"].as_array().unwrap();
        let gates = view["gate_results"].as_array().unwrap();

        let mut missing_or_unknown = false;
        let mut failed = false;
        let mut revalidation = false;
        let mut quarantined = false;

        for required_gate in required {
            let name = required_gate.as_str().unwrap();
            let found = gates.iter().find(|row| row["gate"] == name);
            let Some(row) = found else {
                missing_or_unknown = true;
                continue;
            };
            let status = row["status"].as_str().unwrap();
            if status == "not_applicable" || status == "unknown" {
                missing_or_unknown = true;
            } else if status == "failed" {
                failed = true;
            } else if status == "quarantined" {
                quarantined = true;
            } else if status == "revalidation_required" {
                revalidation = true;
            }

            if status == "qualified" {
                if let Some(until) = row["valid_until"].as_str() {
                    if until < valid_time {
                        revalidation = true;
                    }
                }
            }
        }

        for event in input["temporal_events"].as_array().unwrap() {
            let recorded = event["recorded_at"].as_str().unwrap();
            if recorded > knowledge_cutoff || event["impact_disposition"] == "unaffected" {
                continue;
            }

            if event["scope_sha256"].as_str() != Some(scope) {
                quarantined = true;
                continue;
            }

            let authority_ok = event["authority_disposition"] == "authenticated";
            let time_ok = event["effective_time_confidence"] == "verified"
                && event["effective_at"].as_str().is_some();
            if !authority_ok || !time_ok {
                quarantined = true;
                continue;
            }

            let effective = event["effective_at"].as_str().unwrap();
            if effective > valid_time {
                continue;
            }

            match event["impact_disposition"].as_str().unwrap() {
                "affected" => failed = true,
                "revalidation_required" => revalidation = true,
                "quarantined" => quarantined = true,
                _ => missing_or_unknown = true,
            }
        }

        if quarantined {
            "quarantined_pending_review".into()
        } else if failed {
            "claim_invalidated".into()
        } else if revalidation {
            "revalidation_required".into()
        } else if missing_or_unknown {
            if view_name == "historical_knowledge" || view_name == "retrospective_at_decision" {
                "historical_basis_incomplete".into()
            } else {
                "indeterminate".into()
            }
        } else {
            "candidate_current_within_declared_scope".into()
        }
    }

    fn assert_matches_reference_oracle(input: &Value, report: &Value) {
        for view_name in [
            "historical_knowledge",
            "retrospective_at_decision",
            "current_reliance",
        ] {
            assert_eq!(
                result_disposition(report, view_name),
                reference_view_disposition(input, view_name),
                "production evaluator diverged from reference oracle for {view_name}"
            );
        }
    }

    #[test]
    fn complete_baseline_is_candidate_in_all_three_temporal_views() {
        let input = base_input();
        let report = evaluate(&input).unwrap();
        assert_matches_reference_oracle(&input, &report);
        for view in [
            "historical_knowledge",
            "retrospective_at_decision",
            "current_reliance",
        ] {
            assert_eq!(
                result_disposition(&report, view),
                "candidate_current_within_declared_scope"
            );
        }
        assert_eq!(
            report["continuity_assessment"],
            "views_consistent_candidate"
        );
        assert_eq!(report["action_authority"], "none");
        assert_eq!(report["input_authentication"], "not_performed");
        assert_eq!(report["impact_recomputation"], "not_performed");
    }

    #[test]
    fn late_discovered_verified_retroactive_correction_changes_retrospective_not_historical_knowledge()
     {
        let mut input = base_input();
        input["temporal_events"] = json!([{
            "event_sha256": digest('f'),
            "cause_class": "source_corrected",
            "target_node_id": format!("source_measurement:sha256:{}", digest('1')),
            "target_artifact_sha256": digest('2'),
            "effective_at": "2026-11-01T12:00:00Z",
            "recorded_at": "2026-11-04T10:45:00Z",
            "trusted_at": "2026-11-04T10:50:00Z",
            "effective_time_confidence": "verified",
            "authority_disposition": "authenticated",
            "impact_disposition": "affected",
            "impact_report_sha256": digest('3'),
            "scope_sha256": digest('c')
        }]);

        let report = evaluate(&input).unwrap();
        assert_matches_reference_oracle(&input, &report);
        assert_eq!(
            result_disposition(&report, "historical_knowledge"),
            "candidate_current_within_declared_scope"
        );
        assert_eq!(
            result_disposition(&report, "retrospective_at_decision"),
            "claim_invalidated"
        );
        assert_eq!(
            result_disposition(&report, "current_reliance"),
            "claim_invalidated"
        );
        assert_eq!(report["continuity_assessment"], "temporal_views_diverge");
        assert!(report["historical_record_preserved"].as_bool().unwrap());
    }

    #[test]
    fn prospective_change_does_not_rewrite_decision_time_but_changes_current_reliance() {
        let mut input = base_input();
        input["temporal_events"] = json!([{
            "event_sha256": digest('f'),
            "cause_class": "rights_revoked_or_expired",
            "target_node_id": format!("source_measurement:sha256:{}", digest('1')),
            "target_artifact_sha256": digest('2'),
            "effective_at": "2026-11-03T12:00:00Z",
            "recorded_at": "2026-11-04T10:45:00Z",
            "trusted_at": "2026-11-04T10:50:00Z",
            "effective_time_confidence": "verified",
            "authority_disposition": "authenticated",
            "impact_disposition": "affected",
            "impact_report_sha256": digest('3'),
            "scope_sha256": digest('c')
        }]);

        let report = evaluate(&input).unwrap();
        assert_matches_reference_oracle(&input, &report);
        assert_eq!(
            result_disposition(&report, "retrospective_at_decision"),
            "candidate_current_within_declared_scope"
        );
        assert_eq!(
            result_disposition(&report, "current_reliance"),
            "claim_invalidated"
        );
    }

    #[test]
    fn disputed_effective_time_quarantines_without_rewriting_historical_knowledge() {
        let mut input = base_input();
        input["temporal_events"] = json!([{
            "event_sha256": digest('f'),
            "cause_class": "evidence_disputed",
            "target_node_id": format!("source_measurement:sha256:{}", digest('1')),
            "target_artifact_sha256": digest('2'),
            "effective_at": null,
            "recorded_at": "2026-11-04T10:45:00Z",
            "trusted_at": null,
            "effective_time_confidence": "disputed",
            "authority_disposition": "unknown",
            "impact_disposition": "affected",
            "impact_report_sha256": digest('3'),
            "scope_sha256": digest('c')
        }]);

        let report = evaluate(&input).unwrap();
        assert_matches_reference_oracle(&input, &report);
        assert_eq!(
            result_disposition(&report, "historical_knowledge"),
            "candidate_current_within_declared_scope"
        );
        assert_eq!(
            result_disposition(&report, "retrospective_at_decision"),
            "quarantined_pending_review"
        );
        assert_eq!(
            result_disposition(&report, "current_reliance"),
            "quarantined_pending_review"
        );
    }

    #[test]
    fn missing_mandatory_coverage_gate_is_not_a_clean_no_impact() {
        let mut input = base_input();
        input["views"]["current_reliance"]["gate_results"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["gate"] != "source_coverage");

        let report = evaluate(&input).unwrap();
        assert_matches_reference_oracle(&input, &report);
        assert_eq!(
            result_disposition(&report, "current_reliance"),
            "indeterminate"
        );
        assert_eq!(
            report["views"]["current_reliance"]["missing_required_gates"][0],
            "source_coverage"
        );
    }

    #[test]
    fn expired_gate_receipt_requires_revalidation() {
        let mut input = base_input();
        let gate = input["views"]["current_reliance"]["gate_results"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["gate"] == "rights")
            .unwrap();
        gate["checked_at"] = json!("2026-11-02T12:00:00Z");
        gate["knowledge_available_at"] = json!("2026-11-02T12:00:00Z");
        gate["valid_until"] = json!("2026-11-03T00:00:00Z");

        let report = evaluate(&input).unwrap();
        assert_matches_reference_oracle(&input, &report);
        assert_eq!(
            result_disposition(&report, "current_reliance"),
            "revalidation_required"
        );
        assert_eq!(
            report["views"]["current_reliance"]["gate_results"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["gate"] == "rights")
                .unwrap()["status"],
            "revalidation_required"
        );
    }

    #[test]
    fn historical_view_rejects_future_knowledge_leak() {
        let mut input = base_input();
        let gate = input["views"]["historical_knowledge"]["gate_results"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["gate"] == "source_coverage")
            .unwrap();
        gate["knowledge_available_at"] = json!("2026-11-03T00:00:00Z");

        let report = evaluate(&input).unwrap();
        assert_eq!(report["continuity_assessment"], "invalid_input");
        assert!(
            report["validation_errors"]
                .as_array()
                .unwrap()
                .iter()
                .any(|error| error.as_str().unwrap().starts_with("FUTURE_KNOWLEDGE_LEAK"))
        );
    }

    #[test]
    fn gate_scope_mismatch_is_invalid_input_not_an_unknown_pass() {
        let mut input = base_input();
        let gate = input["views"]["current_reliance"]["gate_results"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["gate"] == "native_identity")
            .unwrap();
        gate["scope_sha256"] = json!(digest('9'));

        let report = evaluate(&input).unwrap();
        assert_eq!(report["continuity_assessment"], "invalid_input");
    }

    #[test]
    fn duplicate_json_members_are_rejected_before_assessment() {
        let input = r#"{"schema_version":"a","schema_version":"b"}"#;
        assert!(parse_strict_json(input).is_err());
    }

    #[test]
    fn identity_and_scope_receipts_must_never_be_compensated_by_other_gates() {
        let mut input = base_input();
        let rows = input["views"]["current_reliance"]["gate_results"]
            .as_array_mut()
            .unwrap();
        let identity = rows
            .iter_mut()
            .find(|row| row["gate"] == "native_identity")
            .unwrap();
        identity["status"] = json!("failed");
        identity["reason_codes"] = json!(["NATIVE_IDENTITY_CONFLICT"]);

        let report = evaluate(&input).unwrap();
        assert_eq!(
            result_disposition(&report, "current_reliance"),
            "claim_invalidated"
        );
        assert!(!report["obligations"].as_array().unwrap().is_empty());
    }
}
