// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Strict native contract for preregistered landscape intervention evaluation.
//!
//! The plan pins an authored project digest, study design, time windows, arms,
//! methods, units, sampling minimums and privacy policy. Passing this validator
//! is structural qualification only: it does not prove field performance,
//! scientific validity, causal identification, consent, or compliance.

use serde_json::{Map, Value};
use std::collections::HashSet;

pub const PLAN_SCHEMA: &str = "luminous.landscape-evaluation-plan.v0";
pub const PLAN_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-evaluation-plan-v0.schema.json";
pub const INTERPRETATION_LIMIT: &str = "validation-does-not-prove-impact-or-causality";

const ROOT_KEYS: &[&str] = &[
    "schema",
    "schema_uri",
    "evaluation_id",
    "project_snapshot_sha256",
    "created_at_unix_s",
    "project_scope",
    "plan_status",
    "study_design",
    "analysis_intent",
    "time_windows",
    "arms",
    "outcomes",
    "causal_analysis_plan",
    "privacy",
    "interpretation_limit",
];
const WINDOW_GROUP_KEYS: &[&str] = &["baseline", "follow_up"];
const WINDOW_KEYS: &[&str] = &["start_unix_s", "end_unix_s"];
const ARM_KEYS: &[&str] = &["id", "role", "site_ref_sha256", "allocation_method"];
const OUTCOME_KEYS: &[&str] = &[
    "metric_id",
    "unit",
    "method_id",
    "method_uri",
    "sampling_design",
    "minimum_observations_per_arm_window",
    "uncertainty_method",
    "desired_direction",
    "target_band",
];
const CAUSAL_KEYS: &[&str] = &[
    "method",
    "pre_registered_at_unix_s",
    "preregistration_record_sha256",
    "confounders",
    "sensitivity_checks",
    "independent_review_required",
];
const PRIVACY_KEYS: &[&str] = &[
    "location_sensitivity",
    "publication_policy",
    "consent_status",
    "consent_record_ref",
    "aggregation_policy",
];

/// Validate a plan and return all structural/policy-contract errors found.
pub fn validate_plan(value: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let Some(root) = value.as_object() else {
        return Err(vec!["$: expected a JSON object".to_string()]);
    };
    validate_keys(root, ROOT_KEYS, "$", &mut errors);
    require_string_eq(value, "schema", PLAN_SCHEMA, "$", &mut errors);
    require_string_eq(value, "schema_uri", PLAN_SCHEMA_URI, "$", &mut errors);
    require_string_eq(
        value,
        "interpretation_limit",
        INTERPRETATION_LIMIT,
        "$",
        &mut errors,
    );

    let evaluation_id = value.get("evaluation_id").and_then(Value::as_str);
    if !evaluation_id.is_some_and(valid_evaluation_id) {
        errors.push(
            "$.evaluation_id: expected eval- followed by 3..61 lowercase letters, digits, or hyphens"
                .to_string(),
        );
    }
    if !value
        .get("project_snapshot_sha256")
        .and_then(Value::as_str)
        .is_some_and(is_sha256)
    {
        errors.push("$.project_snapshot_sha256: expected 64 lowercase hex digits".to_string());
    }
    let created_at = nonnegative_integer(value, "created_at_unix_s", "$", &mut errors);
    let scope = value
        .get("project_scope")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(
        scope,
        "site-parcel" | "district" | "watershed" | "bioregion"
    ) {
        errors.push("$.project_scope: unsupported planning scope".to_string());
    }

    let plan_status = value
        .get("plan_status")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(
        plan_status,
        "draft" | "preregistered" | "approved" | "completed" | "archived"
    ) {
        errors.push("$.plan_status: unsupported plan status".to_string());
    }
    let study_design = value
        .get("study_design")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(
        study_design,
        "descriptive-monitoring"
            | "before-after"
            | "matched-comparison"
            | "before-after-control-impact"
    ) {
        errors.push("$.study_design: unsupported study design".to_string());
    }
    let analysis_intent = value
        .get("analysis_intent")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(
        analysis_intent,
        "descriptive" | "associational" | "causal-estimate"
    ) {
        errors.push("$.analysis_intent: unsupported analysis intent".to_string());
    }

    let windows = validate_time_windows(value.get("time_windows"), &mut errors);
    if let Some((baseline_end, follow_up_start)) = windows {
        if baseline_end > follow_up_start {
            errors.push(
                "$.time_windows: baseline and follow-up windows must not overlap (half-open intervals)"
                    .to_string(),
            );
        }
    }

    let (roles, arm_count) = validate_arms(value.get("arms"), &mut errors);
    if matches!(
        study_design,
        "matched-comparison" | "before-after-control-impact"
    ) {
        if !roles.contains("intervention") {
            errors.push(format!(
                "$.arms: {study_design} requires an intervention arm"
            ));
        }
        if !roles.contains("comparison") {
            errors.push(format!(
                "$.arms: {study_design} requires a comparison/control arm"
            ));
        }
        if arm_count < 2 {
            errors.push(format!(
                "$.arms: {study_design} requires at least two distinct arms"
            ));
        }
    }
    if study_design == "matched-comparison"
        && !value
            .get("arms")
            .and_then(Value::as_array)
            .is_some_and(|arms| {
                arms.iter().any(|arm| {
                    arm.get("role").and_then(Value::as_str) == Some("comparison")
                        && arm.get("allocation_method").and_then(Value::as_str) == Some("matched")
                })
            })
    {
        errors.push(
            "$.arms: matched-comparison requires a comparison arm with allocation_method=matched"
                .to_string(),
        );
    }
    if study_design == "before-after" && !roles.contains("intervention") {
        errors.push("$.arms: before-after design requires an intervention arm".to_string());
    }
    if study_design == "descriptive-monitoring" && analysis_intent != "descriptive" {
        errors.push(
            "$.analysis_intent: descriptive-monitoring design permits descriptive intent only"
                .to_string(),
        );
    }
    if study_design == "before-after" && analysis_intent == "causal-estimate" {
        errors.push(
            "$.analysis_intent: before-after without a comparison arm cannot declare causal-estimate intent"
                .to_string(),
        );
    }

    validate_outcomes(value.get("outcomes"), &mut errors);
    let causal_timestamp = validate_causal_plan(value.get("causal_analysis_plan"), &mut errors);
    if analysis_intent == "causal-estimate" {
        if !matches!(
            study_design,
            "matched-comparison" | "before-after-control-impact"
        ) {
            errors.push(
                "$.analysis_intent: causal-estimate requires matched-comparison or before-after-control-impact"
                    .to_string(),
            );
        }
        if !matches!(plan_status, "preregistered" | "approved") {
            errors.push(
                "$.plan_status: causal-estimate intent requires a preregistered or approved plan"
                    .to_string(),
            );
        }
        match causal_timestamp {
            Some(timestamp) => {
                if let Some(created_at) = created_at {
                    if timestamp > created_at {
                        errors.push(
                            "$.causal_analysis_plan.pre_registered_at_unix_s: preregistration timestamp cannot be after plan creation"
                                .to_string(),
                        );
                    }
                }
                if let Some((_, follow_up_start)) = windows {
                    if timestamp >= follow_up_start {
                        errors.push(
                            "$.causal_analysis_plan.pre_registered_at_unix_s: preregistration must occur before the follow-up window starts"
                                .to_string(),
                        );
                    }
                }
            }
            None => errors.push(
                "$.causal_analysis_plan: causal-estimate intent requires a complete preregistered analysis plan"
                    .to_string(),
            ),
        }
    } else if value
        .get("causal_analysis_plan")
        .is_some_and(|plan| !plan.is_null())
    {
        errors.push(
            "$.causal_analysis_plan: must be null unless analysis_intent is causal-estimate"
                .to_string(),
        );
    }

    validate_privacy(value.get("privacy"), &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_time_windows(value: Option<&Value>, errors: &mut Vec<String>) -> Option<(u64, u64)> {
    let Some(value) = value else {
        errors.push("$.time_windows: missing required object".to_string());
        return None;
    };
    let Some(group) = value.as_object() else {
        errors.push("$.time_windows: expected an object".to_string());
        return None;
    };
    validate_keys(group, WINDOW_GROUP_KEYS, "$.time_windows", errors);

    let baseline = validate_window(value.get("baseline"), "$.time_windows.baseline", errors);
    let follow_up = validate_window(value.get("follow_up"), "$.time_windows.follow_up", errors);
    match (baseline, follow_up) {
        (Some((_, baseline_end)), Some((follow_up_start, _))) => {
            Some((baseline_end, follow_up_start))
        }
        _ => None,
    }
}

fn validate_window(
    value: Option<&Value>,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<(u64, u64)> {
    let Some(value) = value else {
        errors.push(format!("{path}: missing required object"));
        return None;
    };
    let Some(object) = value.as_object() else {
        errors.push(format!("{path}: expected an object"));
        return None;
    };
    validate_keys(object, WINDOW_KEYS, path, errors);
    let start = nonnegative_integer(value, "start_unix_s", path, errors);
    let end = nonnegative_integer(value, "end_unix_s", path, errors);
    match (start, end) {
        (Some(start), Some(end)) if start < end => Some((start, end)),
        (Some(_), Some(_)) => {
            errors.push(format!("{path}: start_unix_s must be less than end_unix_s"));
            None
        }
        _ => None,
    }
}

fn validate_arms(value: Option<&Value>, errors: &mut Vec<String>) -> (HashSet<String>, usize) {
    let Some(value) = value else {
        errors.push("$.arms: missing required array".to_string());
        return (HashSet::new(), 0);
    };
    let Some(arms) = value.as_array() else {
        errors.push("$.arms: expected an array".to_string());
        return (HashSet::new(), 0);
    };
    if !(1..=12).contains(&arms.len()) {
        errors.push("$.arms: expected 1..=12 arms".to_string());
    }
    let mut ids = HashSet::new();
    let mut site_refs = HashSet::new();
    let mut roles = HashSet::new();

    for (index, arm) in arms.iter().enumerate() {
        let path = format!("$.arms[{index}]");
        let Some(object) = arm.as_object() else {
            errors.push(format!("{path}: expected an object"));
            continue;
        };
        validate_keys(object, ARM_KEYS, &path, errors);
        match arm.get("id").and_then(Value::as_str) {
            Some(id) if valid_arm_id(id) => {
                if !ids.insert(id.to_string()) {
                    errors.push(format!("{path}.id: duplicate arm ID {id}"));
                }
            }
            Some(_) => errors.push(format!("{path}.id: invalid arm ID")),
            None => errors.push(format!("{path}.id: expected a string")),
        }
        match arm.get("role").and_then(Value::as_str) {
            Some(role @ ("intervention" | "comparison" | "reference")) => {
                roles.insert(role.to_string());
            }
            Some(_) => errors.push(format!("{path}.role: unsupported arm role")),
            None => errors.push(format!("{path}.role: expected a string")),
        }
        match arm.get("site_ref_sha256").and_then(Value::as_str) {
            Some(digest) if is_sha256(digest) => {
                if !site_refs.insert(digest.to_string()) {
                    errors.push(format!(
                        "{path}.site_ref_sha256: duplicate site reference digest"
                    ));
                }
            }
            Some(_) => errors.push(format!(
                "{path}.site_ref_sha256: expected 64 lowercase hex digits"
            )),
            None => errors.push(format!("{path}.site_ref_sha256: expected a string")),
        }
        if !matches!(
            arm.get("allocation_method").and_then(Value::as_str),
            Some("randomized" | "matched" | "non-randomized" | "observational")
        ) {
            errors.push(format!(
                "{path}.allocation_method: unsupported allocation method"
            ));
        }
    }
    (roles, arms.len())
}

fn validate_outcomes(value: Option<&Value>, errors: &mut Vec<String>) {
    let Some(value) = value else {
        errors.push("$.outcomes: missing required array".to_string());
        return;
    };
    let Some(outcomes) = value.as_array() else {
        errors.push("$.outcomes: expected an array".to_string());
        return;
    };
    if outcomes.is_empty() || outcomes.len() > 64 {
        errors.push("$.outcomes: expected 1..=64 outcomes".to_string());
    }
    let mut metric_ids = HashSet::new();

    for (index, outcome) in outcomes.iter().enumerate() {
        let path = format!("$.outcomes[{index}]");
        let Some(object) = outcome.as_object() else {
            errors.push(format!("{path}: expected an object"));
            continue;
        };
        validate_keys(object, OUTCOME_KEYS, &path, errors);

        match outcome.get("metric_id").and_then(Value::as_str) {
            Some(id) if valid_metric_id(id) => {
                if !metric_ids.insert(id.to_string()) {
                    errors.push(format!(
                        "{path}.metric_id: duplicate outcome metric ID {id}"
                    ));
                }
            }
            Some(_) => errors.push(format!("{path}.metric_id: invalid metric ID")),
            None => errors.push(format!("{path}.metric_id: expected a string")),
        }
        require_nonempty_string(outcome, "unit", &path, 80, errors);
        match outcome.get("method_id").and_then(Value::as_str) {
            Some(method_id) if valid_method_id(method_id) => {}
            Some(_) => errors.push(format!("{path}.method_id: invalid method ID")),
            None => errors.push(format!("{path}.method_id: expected a string")),
        }
        match outcome.get("method_uri").and_then(Value::as_str) {
            Some(uri) if looks_like_uri(uri) => {}
            Some(_) => errors.push(format!("{path}.method_uri: expected an absolute URI")),
            None => errors.push(format!("{path}.method_uri: expected a string")),
        }
        if !matches!(
            outcome.get("sampling_design").and_then(Value::as_str),
            Some(
                "event-based"
                    | "daily"
                    | "weekly"
                    | "monthly"
                    | "seasonal"
                    | "continuous"
                    | "manual-campaign"
            )
        ) {
            errors.push(format!(
                "{path}.sampling_design: unsupported sampling design"
            ));
        }
        match outcome
            .get("minimum_observations_per_arm_window")
            .and_then(Value::as_u64)
        {
            Some(count) if (1..=1_000_000).contains(&count) => {}
            Some(_) => errors.push(format!(
                "{path}.minimum_observations_per_arm_window: expected 1..=1000000"
            )),
            None => errors.push(format!(
                "{path}.minimum_observations_per_arm_window: expected a positive integer"
            )),
        }
        match outcome.get("uncertainty_method").and_then(Value::as_str) {
            Some(method) if (8..=1000).contains(&method.chars().count()) => {}
            Some(_) => errors.push(format!(
                "{path}.uncertainty_method: expected 8..=1000 characters"
            )),
            None => errors.push(format!("{path}.uncertainty_method: expected a string")),
        }

        let direction = outcome
            .get("desired_direction")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !matches!(
            direction,
            "increase" | "decrease" | "maintain" | "target-band" | "non-directional"
        ) {
            errors.push(format!("{path}.desired_direction: unsupported direction"));
        }
        let band = outcome.get("target_band");
        if direction == "target-band" {
            match band.and_then(Value::as_array) {
                Some(values) if values.len() == 2 => {
                    match (values[0].as_f64(), values[1].as_f64()) {
                        (Some(minimum), Some(maximum)) if minimum <= maximum => {}
                        (Some(_), Some(_)) => errors.push(format!(
                            "{path}.target_band: lower bound must be less than or equal to upper bound"
                        )),
                        _ => errors.push(format!(
                            "{path}.target_band: both bounds must be finite numbers"
                        )),
                    }
                }
                _ => errors.push(format!(
                    "{path}.target_band: target-band direction requires exactly two numeric bounds"
                )),
            }
        } else if !matches!(band, Some(Value::Null)) {
            errors.push(format!(
                "{path}.target_band: must be null unless desired_direction is target-band"
            ));
        }
    }
}

fn validate_causal_plan(value: Option<&Value>, errors: &mut Vec<String>) -> Option<u64> {
    let Some(value) = value else {
        errors.push(
            "$.causal_analysis_plan: missing required field (use null when not applicable)"
                .to_string(),
        );
        return None;
    };
    if value.is_null() {
        return None;
    }
    let Some(object) = value.as_object() else {
        errors.push("$.causal_analysis_plan: expected an object or null".to_string());
        return None;
    };
    validate_keys(object, CAUSAL_KEYS, "$.causal_analysis_plan", errors);
    match value.get("method").and_then(Value::as_str) {
        Some(method) if (8..=1000).contains(&method.chars().count()) => {}
        Some(_) => {
            errors.push("$.causal_analysis_plan.method: expected 8..=1000 characters".to_string())
        }
        None => errors.push("$.causal_analysis_plan.method: expected a string".to_string()),
    }
    let timestamp = nonnegative_integer(
        value,
        "pre_registered_at_unix_s",
        "$.causal_analysis_plan",
        errors,
    );
    if !value
        .get("preregistration_record_sha256")
        .and_then(Value::as_str)
        .is_some_and(is_sha256)
    {
        errors.push(
            "$.causal_analysis_plan.preregistration_record_sha256: expected 64 lowercase hex digits"
                .to_string(),
        );
    }
    for field in ["confounders", "sensitivity_checks"] {
        match value.get(field).and_then(Value::as_array) {
            Some(items) if !items.is_empty() => {
                let mut seen = HashSet::new();
                for (index, item) in items.iter().enumerate() {
                    match item.as_str() {
                        Some(text) if text.chars().count() >= 3 && text.chars().count() <= 300 => {
                            if !seen.insert(text.to_string()) {
                                errors.push(format!(
                                    "$.causal_analysis_plan.{field}[{index}]: duplicate item"
                                ));
                            }
                        }
                        _ => errors.push(format!(
                            "$.causal_analysis_plan.{field}[{index}]: expected 3..=300 characters"
                        )),
                    }
                }
            }
            Some(_) => errors.push(format!(
                "$.causal_analysis_plan.{field}: at least one item is required"
            )),
            None => errors.push(format!("$.causal_analysis_plan.{field}: expected an array")),
        }
    }
    if value.get("independent_review_required") != Some(&Value::Bool(true)) {
        errors.push("$.causal_analysis_plan.independent_review_required: must be true".to_string());
    }
    timestamp
}

fn validate_privacy(value: Option<&Value>, errors: &mut Vec<String>) {
    let Some(value) = value else {
        errors.push("$.privacy: missing required object".to_string());
        return;
    };
    let Some(object) = value.as_object() else {
        errors.push("$.privacy: expected an object".to_string());
        return;
    };
    validate_keys(object, PRIVACY_KEYS, "$.privacy", errors);

    let sensitivity = value
        .get("location_sensitivity")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(sensitivity, "public" | "restricted" | "sensitive") {
        errors.push("$.privacy.location_sensitivity: unsupported classification".to_string());
    }
    let publication = value
        .get("publication_policy")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(publication, "private" | "aggregate-only" | "public") {
        errors.push("$.privacy.publication_policy: unsupported policy".to_string());
    }
    let consent = value
        .get("consent_status")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !matches!(
        consent,
        "pending" | "approved" | "not-required" | "withdrawn"
    ) {
        errors.push("$.privacy.consent_status: unsupported status".to_string());
    }
    let consent_reference_present = match value.get("consent_record_ref") {
        Some(Value::Null) => false,
        Some(Value::String(reference))
            if !reference.trim().is_empty() && reference.chars().count() <= 300 =>
        {
            true
        }
        Some(_) => {
            errors.push(
                "$.privacy.consent_record_ref: expected a non-empty string reference or null"
                    .to_string(),
            );
            false
        }
        None => {
            errors.push("$.privacy.consent_record_ref: missing required field".to_string());
            false
        }
    };
    if matches!(consent, "approved" | "withdrawn") && !consent_reference_present {
        errors.push(
            "$.privacy.consent_record_ref: approved or withdrawn consent status requires an authoritative record reference"
                .to_string(),
        );
    }
    let aggregation_policy = value
        .get("aggregation_policy")
        .and_then(Value::as_str)
        .unwrap_or("");
    if publication == "aggregate-only" && aggregation_policy.chars().count() < 8 {
        errors.push(
            "$.privacy.aggregation_policy: aggregate-only publication requires a documented policy"
                .to_string(),
        );
    }
    if publication == "public" {
        if sensitivity != "public" {
            errors.push(
                "$.privacy.publication_policy: public release requires public location sensitivity"
                    .to_string(),
            );
        }
        if !matches!(consent, "approved" | "not-required") {
            errors.push(
                "$.privacy.publication_policy: public release requires approved or not-required consent status"
                    .to_string(),
            );
        }
    }
    if consent == "withdrawn" && publication != "private" {
        errors.push(
            "$.privacy.publication_policy: withdrawn consent blocks public or aggregate publication"
                .to_string(),
        );
    }
}

fn validate_keys(
    actual: &Map<String, Value>,
    allowed: &[&str],
    path: &str,
    errors: &mut Vec<String>,
) {
    for key in allowed {
        if !actual.contains_key(*key) {
            errors.push(format!("{path}.{key}: missing required field"));
        }
    }
    for key in actual.keys() {
        if !allowed.contains(&key.as_str()) {
            errors.push(format!("{path}.{key}: unknown field"));
        }
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

fn require_nonempty_string(
    object: &Value,
    key: &str,
    path: &str,
    maximum_chars: usize,
    errors: &mut Vec<String>,
) {
    match object.get(key).and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() && value.chars().count() <= maximum_chars => {}
        Some(_) => errors.push(format!(
            "{path}.{key}: expected a non-empty string of at most {maximum_chars} characters"
        )),
        None => errors.push(format!("{path}.{key}: expected a string")),
    }
}

fn nonnegative_integer(
    object: &Value,
    key: &str,
    path: &str,
    errors: &mut Vec<String>,
) -> Option<u64> {
    match object.get(key).and_then(Value::as_u64) {
        Some(value) => Some(value),
        None => {
            errors.push(format!("{path}.{key}: expected a non-negative integer"));
            None
        }
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_evaluation_id(value: &str) -> bool {
    let Some(suffix) = value.strip_prefix("eval-") else {
        return false;
    };
    (3..=61).contains(&suffix.len())
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && suffix
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn valid_arm_id(value: &str) -> bool {
    let Some(suffix) = value.strip_prefix("arm-") else {
        return false;
    };
    !suffix.is_empty()
        && suffix.len() <= 48
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && suffix
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn valid_method_id(value: &str) -> bool {
    let Some(first) = value.bytes().next() else {
        return false;
    };
    (3..=80).contains(&value.len())
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || byte == b'.'
                || byte == b'_'
                || byte == b'-'
        })
}

fn valid_metric_id(value: &str) -> bool {
    let mut parts = value.split('.');
    let valid_part = |part: &str| {
        let mut bytes = part.bytes();
        bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
            && part.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
            })
    };
    let first = parts.next().is_some_and(valid_part);
    let second = parts.next().is_some_and(valid_part);
    first && second && parts.all(valid_part)
}

fn looks_like_uri(value: &str) -> bool {
    let Some((scheme, remainder)) = value.split_once(':') else {
        return false;
    };
    !scheme.is_empty()
        && scheme
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic())
        && scheme.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'-' || byte == b'.'
        })
        && !remainder.is_empty()
        && !value.chars().any(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../fixtures/landscape/evaluation-plan-v0.json"
        ))
        .expect("evaluation plan fixture must be valid JSON")
    }

    fn causal_plan() -> Value {
        json!({
            "method": "difference-in-differences with predeclared seasonal covariates",
            "pre_registered_at_unix_s": 1788000000_u64,
            "preregistration_record_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "confounders": ["seasonal rainfall", "land-use change", "upstream works"],
            "sensitivity_checks": ["leave-one-season-out", "alternate matching caliper"],
            "independent_review_required": true
        })
    }

    #[test]
    fn schema_and_fixture_contract_match_rust_constants() {
        let schema: Value = serde_json::from_str(include_str!(
            "../schemas/landscape-evaluation-plan-v0.schema.json"
        ))
        .expect("evaluation plan schema must be valid JSON");
        assert_eq!(schema["$id"], PLAN_SCHEMA_URI);
        assert_eq!(schema["properties"]["schema"]["const"], PLAN_SCHEMA);
        assert_eq!(schema["properties"]["schema_uri"]["const"], PLAN_SCHEMA_URI);
        assert_eq!(
            schema["properties"]["interpretation_limit"]["const"],
            INTERPRETATION_LIMIT
        );
        assert_eq!(validate_plan(&fixture()), Ok(()));
    }

    #[test]
    fn accepts_valid_associational_comparison_plan() {
        assert_eq!(validate_plan(&fixture()), Ok(()));
    }

    #[test]
    fn rejects_baseline_overlapping_follow_up() {
        let mut value = fixture();
        value["time_windows"]["follow_up"]["start_unix_s"] = json!(1782863999_u64);
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("baseline must end before follow-up"))
        );
    }

    #[test]
    fn rejects_matched_comparison_without_matching_declaration() {
        let mut value = fixture();
        value["study_design"] = json!("matched-comparison");
        value["arms"][1]["allocation_method"] = json!("non-randomized");
        let errors = validate_plan(&value).unwrap_err();
        assert!(errors.iter().any(|error| {
            error.contains("requires a comparison arm with allocation_method=matched")
        }));
    }

    #[test]
    fn rejects_before_after_plan_claiming_causal_estimation() {
        let mut value = fixture();
        value["study_design"] = json!("before-after");
        value["analysis_intent"] = json!("causal-estimate");
        value["plan_status"] = json!("preregistered");
        value["causal_analysis_plan"] = causal_plan();
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("before-after without a comparison arm"))
        );
        assert!(errors.iter().any(|error| {
            error.contains("requires matched-comparison or before-after-control-impact")
        }));
    }

    #[test]
    fn accepts_causal_intent_only_with_preregistered_comparison_design() {
        let mut value = fixture();
        value["analysis_intent"] = json!("causal-estimate");
        value["plan_status"] = json!("preregistered");
        value["causal_analysis_plan"] = causal_plan();
        assert_eq!(validate_plan(&value), Ok(()));
    }

    #[test]
    fn rejects_causal_estimate_without_a_preregistration_record() {
        let mut value = fixture();
        value["analysis_intent"] = json!("causal-estimate");
        value["plan_status"] = json!("draft");
        value["causal_analysis_plan"] = Value::Null;
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("preregistered or approved"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("requires a complete preregistered analysis plan"))
        );
    }

    #[test]
    fn rejects_retroactive_preregistration_timestamp() {
        let mut value = fixture();
        value["analysis_intent"] = json!("causal-estimate");
        value["plan_status"] = json!("approved");
        let mut causal = causal_plan();
        causal["pre_registered_at_unix_s"] = json!(1792454500_u64);
        value["causal_analysis_plan"] = causal;
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cannot be after plan creation"))
        );
    }

    #[test]
    fn rejects_preregistration_after_followup_started() {
        let mut value = fixture();
        value["analysis_intent"] = json!("causal-estimate");
        value["plan_status"] = json!("preregistered");
        let mut causal = causal_plan();
        causal["pre_registered_at_unix_s"] = json!(1789000000_u64);
        value["causal_analysis_plan"] = causal;

        let errors = validate_plan(&value).unwrap_err();
        assert!(errors.iter().any(|error| {
            error.contains("preregistration must occur before the follow-up window starts")
        }));
    }

    #[test]
    fn rejects_duplicate_arm_ids_and_site_references() {
        let mut value = fixture();
        value["arms"][1]["id"] = value["arms"][0]["id"].clone();
        value["arms"][1]["site_ref_sha256"] = value["arms"][0]["site_ref_sha256"].clone();
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("duplicate arm ID"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("duplicate site reference"))
        );
    }

    #[test]
    fn rejects_duplicate_outcome_metric_ids_and_missing_units() {
        let mut value = fixture();
        value["outcomes"][1]["metric_id"] = value["outcomes"][0]["metric_id"].clone();
        value["outcomes"][1]["unit"] = json!(" ");
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("duplicate outcome metric ID"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains(".unit: expected a non-empty string"))
        );
    }

    #[test]
    fn rejects_target_band_inversion() {
        let mut value = fixture();
        value["outcomes"][0]["desired_direction"] = json!("target-band");
        value["outcomes"][0]["target_band"] = json!([12.0, 4.0]);
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("lower bound must be less than or equal"))
        );
    }

    #[test]
    fn rejects_sensitive_publication_even_if_consent_is_approved() {
        let mut value = fixture();
        value["privacy"]["location_sensitivity"] = json!("sensitive");
        value["privacy"]["publication_policy"] = json!("public");
        value["privacy"]["consent_status"] = json!("approved");
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("requires public location sensitivity"))
        );
    }

    #[test]
    fn rejects_approved_consent_without_authority_record() {
        let mut value = fixture();
        value["privacy"]["consent_status"] = json!("approved");
        value["privacy"]["consent_record_ref"] = Value::Null;
        let errors = validate_plan(&value).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("requires an authoritative record reference"))
        );
    }

    #[test]
    fn accepts_public_release_with_explicit_consent_record() {
        let mut value = fixture();
        value["privacy"]["location_sensitivity"] = json!("public");
        value["privacy"]["publication_policy"] = json!("public");
        value["privacy"]["consent_status"] = json!("approved");
        value["privacy"]["consent_record_ref"] = json!("mycelix-consent-ref-opaque-001");
        assert_eq!(validate_plan(&value), Ok(()));
    }

    #[test]
    fn rejects_publication_when_consent_is_pending_or_withdrawn() {
        let mut value = fixture();
        value["privacy"]["location_sensitivity"] = json!("public");
        value["privacy"]["publication_policy"] = json!("public");
        value["privacy"]["consent_status"] = json!("pending");
        let pending_errors = validate_plan(&value).unwrap_err();
        assert!(
            pending_errors
                .iter()
                .any(|error| error.contains("requires approved or not-required consent"))
        );
        value["privacy"]["consent_status"] = json!("withdrawn");
        let withdrawn_errors = validate_plan(&value).unwrap_err();
        assert!(
            withdrawn_errors
                .iter()
                .any(|error| error.contains("withdrawn consent blocks"))
        );
    }

    #[test]
    fn rejects_unknown_fields_and_modified_interpretation_limit() {
        let mut value = fixture();
        value["unexpected"] = json!(true);
        value["interpretation_limit"] = json!("causal-proof");
        let errors = validate_plan(&value).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("unknown field")));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("validation-does-not-prove-impact-or-causality"))
        );
    }
}
