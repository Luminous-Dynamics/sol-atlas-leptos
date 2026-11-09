//! Synthetic reference oracle for CityMaker's time-aware state synchronizer.
//!
//! The evaluator is deliberately read-only. It replays owner events at an
//! explicit valid-time, record-time and ingestion-time cut, preserves owner
//! sequence semantics, and never authenticates an opaque physical payload or
//! infers a real-world state from this synthetic corpus.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

const TIME_BASIS: &str = "UTC_UNIX_SECONDS";

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn visible(event: &Value, as_of: &Value) -> bool {
    let (Some(_valid_cut), Some(recorded_cut), Some(ingested_cut), Some(recorded), Some(ingested)) = (
        number(as_of, "valid_at"),
        number(as_of, "recorded_at"),
        number(as_of, "ingested_at"),
        number(event, "recorded_at"),
        number(event, "ingested_at"),
    ) else {
        return false;
    };
    recorded <= recorded_cut && ingested <= ingested_cut
}

fn visible_stage(stage: &Value, as_of: &Value) -> bool {
    let (Some(valid_cut), Some(occurred_at)) =
        (number(as_of, "valid_at"), number(stage, "occurred_at"))
    else {
        return false;
    };
    valid_cut >= occurred_at && visible(stage, as_of)
}

fn push_reason(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn entity_result(
    owner_repo: &str,
    subject_id: &str,
    state: &str,
    selected_event_id: Option<&str>,
    owner_state: Option<&str>,
    reasons: BTreeSet<String>,
    history: Vec<String>,
) -> Value {
    json!({
        "owner_repo": owner_repo,
        "subject_id": subject_id,
        "state": state,
        "selected_event_id": selected_event_id,
        "owner_state": owner_state,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "historical_event_ids": history
    })
}

fn policy_for<'a>(input: &'a Value, event: &Value) -> Option<&'a Value> {
    input
        .get("freshness_policies")
        .and_then(Value::as_array)?
        .iter()
        .find(|policy| {
            text(policy, "policy_id") == text(event, "freshness_policy_id")
                && text(policy, "owner_repo") == text(event, "owner_repo")
                && text(policy, "contract_id") == text(event, "contract_id")
                && text(policy, "contract_version") == text(event, "contract_version")
        })
}

fn owner_health<'a>(input: &'a Value, owner_repo: &str) -> Option<&'a str> {
    input
        .get("owner_health")
        .and_then(Value::as_array)?
        .iter()
        .find(|item| text(item, "owner_repo") == owner_repo)
        .and_then(|item| item.get("state"))
        .and_then(Value::as_str)
}

fn event_relation_target(event: &Value) -> &str {
    text(event, "target_event_id")
}

fn evaluate_subject(input: &Value, required: &Value) -> Value {
    let owner_repo = text(required, "owner_repo");
    let subject_id = text(required, "subject_id");
    let expected_subject_digest = text(required, "subject_digest");
    let expected_scope_digest = text(required, "scope_digest");
    let expected_contract = text(required, "contract_id");
    let expected_version = text(required, "contract_version");
    let as_of = input.get("as_of").unwrap_or(&Value::Null);
    let events = input
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let relevant: Vec<Value> = events
        .into_iter()
        .filter(|event| {
            text(event, "owner_repo") == owner_repo && text(event, "subject_id") == subject_id
        })
        .filter(|event| visible(event, as_of))
        .collect();

    if relevant.is_empty() {
        let mut reasons = BTreeSet::new();
        push_reason(&mut reasons, "NO_VISIBLE_OWNER_EVENT");
        return entity_result(
            owner_repo,
            subject_id,
            "Unknown",
            None,
            None,
            reasons,
            Vec::new(),
        );
    }

    let mut reasons = BTreeSet::new();
    let mut history_events = relevant.clone();
    history_events.sort_by_key(|event| {
        (
            number(event, "owner_sequence").unwrap_or(0),
            text(event, "event_id").to_owned(),
        )
    });
    let history = history_events
        .iter()
        .map(|event| text(event, "event_id").to_owned())
        .collect::<Vec<_>>();

    let mut event_ids = BTreeMap::<String, String>::new();
    let mut receipt_ids = BTreeSet::<String>::new();
    for event in &relevant {
        let event_id = text(event, "event_id");
        let content_digest = text(event, "content_sha256");
        if event_id.is_empty() {
            push_reason(&mut reasons, "EVENT_ID_MISSING");
        } else if let Some(previous_digest) =
            event_ids.insert(event_id.to_owned(), content_digest.to_owned())
        {
            if previous_digest != content_digest {
                push_reason(&mut reasons, "EVENT_ID_CONTENT_CONFLICT");
            } else {
                push_reason(&mut reasons, "DUPLICATE_EVENT_ID");
            }
        }
        let receipt_id = text(event, "receipt_id");
        if receipt_id.is_empty() || !receipt_ids.insert(receipt_id.to_owned()) {
            push_reason(&mut reasons, "DUPLICATE_OR_MISSING_RECEIPT_ID");
        }
        if text(event, "contract_id") != expected_contract
            || text(event, "contract_version") != expected_version
        {
            push_reason(&mut reasons, "OWNER_CONTRACT_VERSION_MISMATCH");
        }
        if text(event, "subject_digest") != expected_subject_digest {
            push_reason(&mut reasons, "SUBJECT_IDENTITY_MISMATCH");
        }
        if text(event, "scope_digest") != expected_scope_digest {
            push_reason(&mut reasons, "SCOPE_DIGEST_MISMATCH");
        }
        if !is_sha256(text(event, "schema_sha256"))
            || !is_sha256(text(event, "content_sha256"))
            || !is_sha256(text(event, "subject_digest"))
            || !is_sha256(text(event, "scope_digest"))
        {
            push_reason(&mut reasons, "EVENT_DIGEST_INVALID");
        }
        if text(event, "time_basis").is_empty() || text(event, "timezone").is_empty() {
            push_reason(&mut reasons, "TIME_BASIS_MISSING");
        } else if text(event, "time_basis") != TIME_BASIS {
            push_reason(&mut reasons, "TIME_BASIS_UNSUPPORTED");
        }
        if number(event, "valid_from").is_none()
            || number(event, "recorded_at").is_none()
            || number(event, "ingested_at").is_none()
            || number(event, "owner_sequence").is_none()
        {
            push_reason(&mut reasons, "REQUIRED_TIME_OR_SEQUENCE_MISSING");
        }
    }

    if reasons.contains("SCOPE_DIGEST_MISMATCH") {
        return entity_result(
            owner_repo,
            subject_id,
            "NotComparable",
            None,
            None,
            reasons,
            history,
        );
    }
    if reasons.contains("OWNER_CONTRACT_VERSION_MISMATCH")
        || reasons.contains("SUBJECT_IDENTITY_MISMATCH")
        || reasons.contains("EVENT_DIGEST_INVALID")
        || reasons.contains("EVENT_ID_MISSING")
        || reasons.contains("EVENT_ID_CONTENT_CONFLICT")
        || reasons.contains("DUPLICATE_EVENT_ID")
        || reasons.contains("DUPLICATE_OR_MISSING_RECEIPT_ID")
        || reasons.contains("TIME_BASIS_UNSUPPORTED")
    {
        return entity_result(
            owner_repo,
            subject_id,
            "Unsupported",
            None,
            None,
            reasons,
            history,
        );
    }
    if reasons.contains("TIME_BASIS_MISSING")
        || reasons.contains("REQUIRED_TIME_OR_SEQUENCE_MISSING")
    {
        return entity_result(
            owner_repo, subject_id, "Unknown", None, None, reasons, history,
        );
    }

    let mut sequences = BTreeSet::new();
    if relevant
        .iter()
        .any(|event| !sequences.insert(number(event, "owner_sequence").unwrap_or(0)))
    {
        push_reason(&mut reasons, "OWNER_SEQUENCE_CONFLICT");
        return entity_result(
            owner_repo,
            subject_id,
            "Unsupported",
            None,
            None,
            reasons,
            history,
        );
    }

    let as_of_valid = number(as_of, "valid_at").unwrap_or(0);
    let candidates: Vec<&Value> = relevant
        .iter()
        .filter(|event| {
            matches!(
                text(event, "event_kind"),
                "assertion" | "supersede" | "correction"
            ) && number(event, "valid_from").is_some_and(|from| from <= as_of_valid)
        })
        .collect();

    let Some(selected) = candidates
        .into_iter()
        .max_by_key(|event| number(event, "owner_sequence").unwrap_or(0))
    else {
        push_reason(&mut reasons, "NO_ASSERTION_VALID_AT_AS_OF_TIME");
        return entity_result(
            owner_repo, subject_id, "Unknown", None, None, reasons, history,
        );
    };

    for event in &relevant {
        if matches!(
            text(event, "event_kind"),
            "supersede" | "correction" | "revoke" | "dispute"
        ) {
            let target = event_relation_target(event);
            let target_exists = relevant.iter().any(|candidate| {
                text(candidate, "event_id") == target
                    && number(candidate, "owner_sequence").unwrap_or(0)
                        < number(event, "owner_sequence").unwrap_or(0)
            });
            if target.is_empty() || !target_exists {
                push_reason(&mut reasons, "OWNER_RELATION_TARGET_INVALID");
            }
        }
    }
    if reasons.contains("OWNER_RELATION_TARGET_INVALID") {
        return entity_result(
            owner_repo,
            subject_id,
            "Unsupported",
            Some(text(selected, "event_id")),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }

    let selected_id = text(selected, "event_id");
    let latest_relation = relevant
        .iter()
        .filter(|event| {
            matches!(text(event, "event_kind"), "revoke" | "dispute")
                && event_relation_target(event) == selected_id
                && number(event, "valid_from").is_some_and(|from| from <= as_of_valid)
        })
        .max_by_key(|event| number(event, "owner_sequence").unwrap_or(0));

    if let Some(relation) = latest_relation {
        let state = if text(relation, "event_kind") == "revoke" {
            push_reason(&mut reasons, "OWNER_REVOKED");
            "Revoked"
        } else {
            push_reason(&mut reasons, "OWNER_DISPUTE_OPEN");
            "Disputed"
        };
        return entity_result(
            owner_repo,
            subject_id,
            state,
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }

    if text(selected, "evidence_state") != "admitted" {
        push_reason(&mut reasons, "OWNER_EVIDENCE_NOT_ADMITTED");
        return entity_result(
            owner_repo,
            subject_id,
            "Unknown",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }

    let Some(policy) = policy_for(input, selected) else {
        push_reason(&mut reasons, "FRESHNESS_POLICY_MISSING");
        return entity_result(
            owner_repo,
            subject_id,
            "Unknown",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    };

    let expected_refresh = number(policy, "expected_refresh_seconds");
    let max_age = number(policy, "max_observation_age_seconds");
    let max_record_delay = number(policy, "max_record_delay_seconds");
    let max_ingest_lag = number(policy, "max_ingest_lag_seconds");
    let max_clock_uncertainty = number(policy, "max_clock_uncertainty_ms");
    if expected_refresh.unwrap_or(0) == 0
        || max_age.is_none()
        || max_record_delay.is_none()
        || max_ingest_lag.is_none()
        || max_clock_uncertainty.is_none()
    {
        push_reason(&mut reasons, "FRESHNESS_POLICY_INCOMPLETE");
        return entity_result(
            owner_repo,
            subject_id,
            "Unknown",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }

    match owner_health(input, owner_repo) {
        Some("available") => {}
        Some("partial_outage") => {
            push_reason(&mut reasons, "SOURCE_PARTIAL_OUTAGE");
            return entity_result(
                owner_repo,
                subject_id,
                "Stale",
                Some(selected_id),
                Some(text(selected, "owner_state")),
                reasons,
                history,
            );
        }
        _ => {
            push_reason(&mut reasons, "OWNER_HEALTH_UNKNOWN");
            return entity_result(
                owner_repo,
                subject_id,
                "Unknown",
                Some(selected_id),
                Some(text(selected, "owner_state")),
                reasons,
                history,
            );
        }
    }

    if number(selected, "clock_uncertainty_ms").is_none() {
        push_reason(&mut reasons, "CLOCK_UNCERTAINTY_MISSING");
        return entity_result(
            owner_repo,
            subject_id,
            "Unknown",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }
    if number(selected, "clock_uncertainty_ms").unwrap_or(u64::MAX)
        > max_clock_uncertainty.unwrap_or(0)
    {
        push_reason(&mut reasons, "CLOCK_UNCERTAINTY_EXCEEDS_POLICY");
        return entity_result(
            owner_repo,
            subject_id,
            "Unknown",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }

    let valid_until = selected.get("valid_until").and_then(Value::as_u64);
    if valid_until.is_some_and(|until| as_of_valid >= until) {
        push_reason(&mut reasons, "VALID_INTERVAL_EXPIRED");
        return entity_result(
            owner_repo,
            subject_id,
            "Stale",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }

    let Some(observed_at) = number(selected, "observed_at") else {
        push_reason(&mut reasons, "OBSERVATION_TIME_MISSING");
        return entity_result(
            owner_repo,
            subject_id,
            "Unknown",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    };
    let recorded_at = number(selected, "recorded_at").unwrap_or(0);
    let ingested_at = number(selected, "ingested_at").unwrap_or(0);
    if recorded_at < observed_at || ingested_at < recorded_at {
        push_reason(&mut reasons, "SOURCE_TIME_ORDER_INVALID");
        return entity_result(
            owner_repo,
            subject_id,
            "Unsupported",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }
    if as_of_valid < observed_at {
        push_reason(&mut reasons, "OBSERVATION_AFTER_AS_OF_TIME");
        return entity_result(
            owner_repo,
            subject_id,
            "Unknown",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }
    let observation_age = as_of_valid - observed_at;
    if observation_age > max_age.unwrap_or(0) {
        push_reason(&mut reasons, "OBSERVATION_AGE_EXCEEDED");
        return entity_result(
            owner_repo,
            subject_id,
            "Stale",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }
    if recorded_at - observed_at > max_record_delay.unwrap_or(0) {
        push_reason(&mut reasons, "SOURCE_RECORD_DELAY_EXCEEDED");
        return entity_result(
            owner_repo,
            subject_id,
            "Delayed",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }
    if ingested_at - recorded_at > max_ingest_lag.unwrap_or(0) {
        push_reason(&mut reasons, "INGESTION_DELAY_EXCEEDED");
        return entity_result(
            owner_repo,
            subject_id,
            "Delayed",
            Some(selected_id),
            Some(text(selected, "owner_state")),
            reasons,
            history,
        );
    }

    entity_result(
        owner_repo,
        subject_id,
        "Current",
        Some(selected_id),
        Some(text(selected, "owner_state")),
        reasons,
        history,
    )
}

fn evaluate_intervention(input: &Value, intervention_id: &str, stages: &[Value]) -> Value {
    let as_of = input.get("as_of").unwrap_or(&Value::Null);
    let visible_stages: Vec<&Value> = stages
        .iter()
        .filter(|stage| {
            text(stage, "intervention_id") == intervention_id && visible_stage(stage, as_of)
        })
        .collect();

    let mut reasons = BTreeSet::new();
    if visible_stages.is_empty() {
        push_reason(&mut reasons, "NO_VISIBLE_INTERVENTION_STAGE");
        return json!({
            "intervention_id": intervention_id,
            "state": "Unknown",
            "workflow_complete": false,
            "causal_effect_established": false,
            "reason_codes": reasons.into_iter().collect::<Vec<_>>()
        });
    }

    let scope_digests: BTreeSet<&str> = visible_stages
        .iter()
        .map(|stage| text(stage, "scope_digest"))
        .collect();
    let horizons: BTreeSet<&str> = visible_stages
        .iter()
        .map(|stage| text(stage, "horizon_id"))
        .collect();
    if scope_digests.len() != 1 || horizons.len() != 1 {
        push_reason(&mut reasons, "INTERVENTION_SCOPE_OR_HORIZON_MISMATCH");
        return json!({
            "intervention_id": intervention_id,
            "state": "NotComparable",
            "workflow_complete": false,
            "causal_effect_established": false,
            "reason_codes": reasons.into_iter().collect::<Vec<_>>()
        });
    }

    let mut seen_event_ids = BTreeSet::<String>::new();
    for stage in &visible_stages {
        let event_id = text(stage, "event_id");
        if event_id.is_empty() || !seen_event_ids.insert(event_id.to_owned()) {
            push_reason(&mut reasons, "INTERVENTION_EVENT_ID_MISSING_OR_DUPLICATE");
            return json!({
                "intervention_id": intervention_id,
                "state": "Unsupported",
                "workflow_complete": false,
                "causal_effect_established": false,
                "reason_codes": reasons.into_iter().collect::<Vec<_>>()
            });
        }
        if text(stage, "owner_repo").is_empty() || number(stage, "occurred_at").is_none() {
            push_reason(&mut reasons, "INTERVENTION_EVENT_PROVENANCE_INCOMPLETE");
            return json!({
                "intervention_id": intervention_id,
                "state": "Unknown",
                "workflow_complete": false,
                "causal_effect_established": false,
                "reason_codes": reasons.into_iter().collect::<Vec<_>>()
            });
        }
    }

    let mut by_stage = BTreeMap::<String, &Value>::new();
    for stage in &visible_stages {
        let key = text(stage, "stage").to_owned();
        if by_stage.insert(key, stage).is_some() {
            push_reason(&mut reasons, "DUPLICATE_INTERVENTION_STAGE");
            return json!({
                "intervention_id": intervention_id,
                "state": "Unsupported",
                "workflow_complete": false,
                "causal_effect_established": false,
                "reason_codes": reasons.into_iter().collect::<Vec<_>>()
            });
        }
    }

    let ordered = ["proposal", "approval", "execution", "inspection", "outcome"];
    let mut previous_id: Option<&str> = None;
    for (index, stage_name) in ordered.iter().enumerate() {
        if let Some(stage) = by_stage.get(*stage_name) {
            let event_id = text(stage, "event_id");
            let predecessor = stage.get("predecessor_event_id").and_then(Value::as_str);
            let predecessor_value = predecessor.unwrap_or("");
            let required_predecessor_present =
                index == 0 || by_stage.contains_key(ordered[index - 1]);
            let predecessor_is_valid = if index == 0 {
                predecessor_value.is_empty()
            } else {
                required_predecessor_present
                    && previous_id.is_some_and(|previous| predecessor_value == previous)
            };
            if !predecessor_is_valid {
                push_reason(
                    &mut reasons,
                    if index > 0 && !required_predecessor_present {
                        "INTERVENTION_PREDECESSOR_MISSING"
                    } else {
                        "INTERVENTION_PREDECESSOR_MISMATCH"
                    },
                );
                return json!({
                    "intervention_id": intervention_id,
                    "state": "Unsupported",
                    "workflow_complete": false,
                    "causal_effect_established": false,
                    "reason_codes": reasons.into_iter().collect::<Vec<_>>()
                });
            }
            let occurred_at = number(stage, "occurred_at");
            let recorded_at = number(stage, "recorded_at");
            let ingested_at = number(stage, "ingested_at");
            if !matches!((occurred_at, recorded_at, ingested_at), (Some(o), Some(r), Some(i)) if o <= r && r <= i)
            {
                push_reason(&mut reasons, "INTERVENTION_TIME_ORDER_INVALID");
                return json!({
                    "intervention_id": intervention_id,
                    "state": "Unsupported",
                    "workflow_complete": false,
                    "causal_effect_established": false,
                    "reason_codes": reasons.into_iter().collect::<Vec<_>>()
                });
            }
            previous_id = Some(event_id);
        }
    }

    let state = if by_stage.contains_key("outcome") {
        "OutcomeObserved"
    } else if by_stage.contains_key("inspection")
        && text(by_stage["inspection"], "disposition") == "passed"
    {
        "CommissionedOutcomeUnobserved"
    } else if by_stage.contains_key("inspection") {
        "InspectionNotAccepted"
    } else if by_stage.contains_key("execution") {
        "ExecutedNotInspected"
    } else if by_stage.contains_key("approval") {
        "ApprovedNotExecuted"
    } else if by_stage.contains_key("proposal") {
        "Proposed"
    } else {
        "Unknown"
    };

    let workflow_complete = by_stage.contains_key("outcome")
        && ordered.iter().all(|name| by_stage.contains_key(*name))
        && reasons.is_empty();

    json!({
        "intervention_id": intervention_id,
        "state": state,
        "workflow_complete": workflow_complete,
        "causal_effect_established": false,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>()
    })
}

/// Evaluate a case using only its input. The expected known-answer object is
/// never consulted by this function.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let required = input
        .get("required_subjects")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut entities = required
        .iter()
        .map(|subject| evaluate_subject(input, subject))
        .collect::<Vec<_>>();
    entities.sort_by(|left, right| {
        text(left, "owner_repo")
            .cmp(text(right, "owner_repo"))
            .then_with(|| text(left, "subject_id").cmp(text(right, "subject_id")))
    });

    let stages = input
        .get("intervention_stages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let intervention_ids: BTreeSet<String> = stages
        .iter()
        .map(|stage| text(stage, "intervention_id").to_owned())
        .filter(|id| !id.is_empty())
        .collect();
    let interventions = intervention_ids
        .iter()
        .map(|id| evaluate_intervention(input, id, &stages))
        .collect::<Vec<_>>();

    let events = input
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut identity_links = Vec::new();
    let required_subjects = input
        .get("required_subjects")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for event in events.iter().filter(|event| {
        text(event, "event_kind") == "identity_successor"
            && visible(event, input.get("as_of").unwrap_or(&Value::Null))
    }) {
        let owner_repo = text(event, "owner_repo");
        let target_subject_id = text(event, "target_subject_id");
        let target_subject_digest = text(event, "target_subject_digest");
        let registered_target = required_subjects.iter().find(|subject| {
            text(subject, "owner_repo") == owner_repo
                && text(subject, "subject_id") == target_subject_id
        });
        let (status, reason_codes) = if text(event, "evidence_state") != "admitted"
            || target_subject_id.is_empty()
            || !is_sha256(text(event, "content_sha256"))
            || !is_sha256(target_subject_digest)
        {
            ("Unsupported", vec!["IDENTITY_SUCCESSOR_EVIDENCE_INVALID"])
        } else if let Some(target) = registered_target {
            if text(target, "subject_digest") == target_subject_digest {
                ("OwnerDeclared", Vec::new())
            } else {
                ("Unsupported", vec!["IDENTITY_TARGET_DIGEST_MISMATCH"])
            }
        } else {
            ("Unknown", vec!["IDENTITY_TARGET_NOT_REGISTERED"])
        };
        identity_links.push(json!({
            "owner_repo": owner_repo,
            "from_subject_id": text(event, "subject_id"),
            "to_subject_id": target_subject_id,
            "status": status,
            "automatic_merge": false,
            "reason_codes": reason_codes
        }));
    }
    identity_links.sort_by(|left, right| {
        text(left, "owner_repo")
            .cmp(text(right, "owner_repo"))
            .then_with(|| text(left, "from_subject_id").cmp(text(right, "from_subject_id")))
    });

    json!({
        "case_id": case_id,
        "entities": entities,
        "interventions": interventions,
        "identity_links": identity_links,
        "claim_ceiling": "synthetic_temporal_state_and_feedback_semantics_only"
    })
}

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(actual), Value::Object(expected)) => {
            actual.len() == expected.len()
                && actual.iter().all(|(key, value)| {
                    expected
                        .get(key)
                        .is_some_and(|expected_value| matches_expected(value, expected_value))
                })
        }
        (Value::Array(actual), Value::Array(expected)) => {
            actual.len() == expected.len()
                && actual
                    .iter()
                    .zip(expected)
                    .all(|(a, e)| matches_expected(a, e))
        }
        _ => actual == expected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hash(ch: char) -> String {
        std::iter::repeat_n(ch, 64).collect()
    }

    fn policy() -> Value {
        json!({
            "policy_id":"P-1",
            "owner_repo":"owner/a",
            "contract_id":"asset-state",
            "contract_version":"1.0",
            "expected_refresh_seconds":60,
            "max_observation_age_seconds":300,
            "max_record_delay_seconds":120,
            "max_ingest_lag_seconds":120,
            "max_clock_uncertainty_ms":100
        })
    }

    fn event(id: &str, sequence: u64, observed: u64, recorded: u64, ingested: u64) -> Value {
        json!({
            "event_id":id,
            "receipt_id":format!("R-{id}"),
            "owner_repo":"owner/a",
            "contract_id":"asset-state",
            "contract_version":"1.0",
            "source_revision":"0123456789012345678901234567890123456789",
            "schema_sha256":hash('a'),
            "content_sha256":hash('b'),
            "subject_id":"asset-1",
            "subject_digest":hash('c'),
            "scope_digest":hash('d'),
            "owner_sequence":sequence,
            "event_kind":"assertion",
            "target_event_id":null,
            "target_subject_id":null,
            "owner_state":"Available",
            "evidence_state":"admitted",
            "valid_from":0,
            "valid_until":null,
            "observed_at":observed,
            "recorded_at":recorded,
            "ingested_at":ingested,
            "time_basis":TIME_BASIS,
            "timezone":"UTC",
            "clock_id":"clock-1",
            "clock_uncertainty_ms":5,
            "freshness_policy_id":"P-1"
        })
    }

    fn input(events: Vec<Value>) -> Value {
        json!({
            "as_of":{"valid_at":1000,"recorded_at":1100,"ingested_at":1200},
            "freshness_policies":[policy()],
            "owner_health":[{"owner_repo":"owner/a","state":"available"}],
            "required_subjects":[{
                "owner_repo":"owner/a",
                "subject_id":"asset-1",
                "subject_digest":hash('c'),
                "scope_digest":hash('d'),
                "contract_id":"asset-state",
                "contract_version":"1.0"
            }],
            "events":events,
            "intervention_stages":[],
            "identity_links":[]
        })
    }

    #[test]
    fn current_state_requires_owner_policy_and_admitted_observation() {
        let data = input(vec![event("E-1", 1, 950, 960, 970)]);
        let result = evaluate_subject(&data, &data["required_subjects"][0]);
        assert_eq!(result["state"], "Current");
    }

    #[test]
    fn canonical_utc_timestamp_preserves_south_african_source_timezone() {
        let mut observation = event("E-1", 1, 950, 960, 970);
        observation["timezone"] = json!("Africa/Johannesburg");
        let data = input(vec![observation]);
        let result = evaluate_subject(&data, &data["required_subjects"][0]);
        assert_eq!(result["state"], "Current");
    }

    #[test]
    fn owner_sequence_not_publication_time_selects_state() {
        let mut older_sequence = event("E-1", 1, 950, 990, 1000);
        let newer_sequence = event("E-2", 2, 940, 960, 970);
        older_sequence["owner_state"] = json!("Old");
        let data = input(vec![older_sequence, newer_sequence]);
        let result = evaluate_subject(&data, &data["required_subjects"][0]);
        assert_eq!(result["selected_event_id"], "E-2");
    }

    #[test]
    fn revoked_owner_event_cannot_remain_current() {
        let assertion = event("E-1", 1, 950, 960, 970);
        let mut revoke = event("E-2", 2, 980, 990, 1000);
        revoke["event_kind"] = json!("revoke");
        revoke["target_event_id"] = json!("E-1");
        let data = input(vec![assertion, revoke]);
        let result = evaluate_subject(&data, &data["required_subjects"][0]);
        assert_eq!(result["state"], "Revoked");
    }

    #[test]
    fn missing_freshness_policy_never_emits_current() {
        let mut data = input(vec![event("E-1", 1, 950, 960, 970)]);
        data["freshness_policies"] = json!([]);
        let result = evaluate_subject(&data, &data["required_subjects"][0]);
        assert_eq!(result["state"], "Unknown");
    }

    #[test]
    fn scope_mismatch_is_not_comparable() {
        let mut e = event("E-1", 1, 950, 960, 970);
        e["scope_digest"] = json!(hash('e'));
        let data = input(vec![e]);
        let result = evaluate_subject(&data, &data["required_subjects"][0]);
        assert_eq!(result["state"], "NotComparable");
    }

    #[test]
    fn partial_outage_preserves_last_known_state_as_stale() {
        let mut data = input(vec![event("E-1", 1, 950, 960, 970)]);
        data["owner_health"][0]["state"] = json!("partial_outage");
        let result = evaluate_subject(&data, &data["required_subjects"][0]);
        assert_eq!(result["state"], "Stale");
    }

    #[test]
    fn duplicate_event_ids_are_unsupported() {
        let data = input(vec![
            event("E-1", 1, 950, 960, 970),
            event("E-1", 2, 951, 961, 971),
        ]);
        let case = json!({"case_id":"duplicate","input":data});
        let result = evaluate_case(&case);
        assert_eq!(result["entities"][0]["state"], "Unsupported");
    }

    #[test]
    fn intervention_without_execution_is_not_complete() {
        let proposal = json!({
            "event_id":"P","intervention_id":"I-1","stage":"proposal",
            "predecessor_event_id":null,"owner_repo":"owner/a",
            "scope_digest":hash('d'),"horizon_id":"H-1",
            "occurred_at":890,"recorded_at":900,"ingested_at":910,"disposition":"proposed"
        });
        let mut approval = proposal.clone();
        approval["event_id"] = json!("A");
        approval["stage"] = json!("approval");
        approval["predecessor_event_id"] = json!("P");
        approval["recorded_at"] = json!(920);
        approval["ingested_at"] = json!(930);
        let data = json!({
            "as_of":{"valid_at":1000,"recorded_at":1100,"ingested_at":1200},
            "intervention_stages":[proposal,approval]
        });
        let result = evaluate_intervention(
            &data,
            "I-1",
            data["intervention_stages"].as_array().unwrap(),
        );
        assert_eq!(result["state"], "ApprovedNotExecuted");
        assert_eq!(result["workflow_complete"], false);
    }

    #[test]
    fn missing_as_of_cutoffs_never_admit_an_event_as_current() {
        let mut data = input(vec![event("E-1", 1, 950, 960, 970)]);
        data["as_of"].as_object_mut().unwrap().remove("recorded_at");
        let result = evaluate_subject(&data, &data["required_subjects"][0]);
        assert_eq!(result["state"], "Unknown");
        assert_eq!(result["reason_codes"][0], "NO_VISIBLE_OWNER_EVENT");
    }

    #[test]
    fn duplicate_intervention_event_ids_are_unsupported() {
        let stages = vec![
            json!({"event_id":"S-1","intervention_id":"I-1","stage":"proposal",
                "predecessor_event_id":null,"owner_repo":"owner/a","scope_digest":hash('d'),
                "horizon_id":"H-1","occurred_at":890,"recorded_at":900,"ingested_at":910,
                "disposition":"proposed"}),
            json!({"event_id":"S-1","intervention_id":"I-1","stage":"approval",
                "predecessor_event_id":"S-1","owner_repo":"owner/a","scope_digest":hash('d'),
                "horizon_id":"H-1","occurred_at":891,"recorded_at":901,"ingested_at":911,
                "disposition":"approved"}),
        ];
        let data = json!({"as_of":{"valid_at":1000,"recorded_at":1100,"ingested_at":1200}});
        let result = evaluate_intervention(&data, "I-1", &stages);
        assert_eq!(result["state"], "Unsupported");
        assert_eq!(
            result["reason_codes"][0],
            "INTERVENTION_EVENT_ID_MISSING_OR_DUPLICATE"
        );
    }

    #[test]
    fn intervention_stage_not_yet_valid_is_not_visible() {
        let stages = vec![json!({
            "event_id":"S-1","intervention_id":"I-1","stage":"proposal",
            "predecessor_event_id":null,"owner_repo":"owner/a","scope_digest":hash('d'),
            "horizon_id":"H-1","occurred_at":1100,"recorded_at":900,"ingested_at":910,
            "disposition":"proposed"
        })];
        let data = json!({"as_of":{"valid_at":1000,"recorded_at":1100,"ingested_at":1200}});
        let result = evaluate_intervention(&data, "I-1", &stages);
        assert_eq!(result["state"], "Unknown");
        assert_eq!(result["reason_codes"][0], "NO_VISIBLE_INTERVENTION_STAGE");
    }

    #[test]
    fn approval_without_a_proposal_is_unsupported() {
        let stages = vec![json!({
            "event_id":"A","intervention_id":"I-1","stage":"approval",
            "predecessor_event_id":null,"owner_repo":"owner/a","scope_digest":hash('d'),
            "horizon_id":"H-1","occurred_at":890,"recorded_at":900,"ingested_at":910,
            "disposition":"approved"
        })];
        let data = json!({"as_of":{"valid_at":1000,"recorded_at":1100,"ingested_at":1200}});
        let result = evaluate_intervention(&data, "I-1", &stages);
        assert_eq!(result["state"], "Unsupported");
        assert_eq!(
            result["reason_codes"][0],
            "INTERVENTION_PREDECESSOR_MISSING"
        );
    }

    #[test]
    fn execution_cannot_skip_the_approval_stage() {
        let stages = vec![
            json!({
                "event_id":"P","intervention_id":"I-1","stage":"proposal",
                "predecessor_event_id":null,"owner_repo":"owner/a","scope_digest":hash('d'),
                "horizon_id":"H-1","occurred_at":890,"recorded_at":900,"ingested_at":910,
                "disposition":"proposed"
            }),
            json!({
                "event_id":"X","intervention_id":"I-1","stage":"execution",
                "predecessor_event_id":"P","owner_repo":"owner/a","scope_digest":hash('d'),
                "horizon_id":"H-1","occurred_at":920,"recorded_at":930,"ingested_at":940,
                "disposition":"executed"
            }),
        ];
        let data = json!({"as_of":{"valid_at":1000,"recorded_at":1100,"ingested_at":1200}});
        let result = evaluate_intervention(&data, "I-1", &stages);
        assert_eq!(result["state"], "Unsupported");
        assert_eq!(
            result["reason_codes"][0],
            "INTERVENTION_PREDECESSOR_MISSING"
        );
    }

    #[test]
    fn intervention_with_invalid_time_order_is_unsupported() {
        let stages = vec![json!({
            "event_id":"P","intervention_id":"I-1","stage":"proposal",
            "predecessor_event_id":null,"owner_repo":"owner/a","scope_digest":hash('d'),
            "horizon_id":"H-1","occurred_at":950,"recorded_at":900,"ingested_at":910,
            "disposition":"proposed"
        })];
        let data = json!({"as_of":{"valid_at":1000,"recorded_at":1100,"ingested_at":1200}});
        let result = evaluate_intervention(&data, "I-1", &stages);
        assert_eq!(result["state"], "Unsupported");
        assert_eq!(result["reason_codes"][0], "INTERVENTION_TIME_ORDER_INVALID");
    }

    #[test]
    fn complete_feedback_chain_does_not_claim_causality() {
        let names = ["proposal", "approval", "execution", "inspection", "outcome"];
        let mut stages = Vec::new();
        let mut previous = Value::Null;
        for (index, name) in names.iter().enumerate() {
            let id = format!("S-{index}");
            stages.push(json!({
                "event_id":id,"intervention_id":"I-1","stage":name,
                "predecessor_event_id":previous,"owner_repo":"owner/a",
                "scope_digest":hash('d'),"horizon_id":"H-1",
                "occurred_at":890+index as u64,
                "recorded_at":900+index as u64,
                "ingested_at":910+index as u64,
                "disposition":if *name=="inspection" {"passed"} else {"recorded"}
            }));
            previous = json!(id);
        }
        let data = json!({"as_of":{"valid_at":1000,"recorded_at":1100,"ingested_at":1200},"intervention_stages":stages});
        let result = evaluate_intervention(
            &data,
            "I-1",
            data["intervention_stages"].as_array().unwrap(),
        );
        assert_eq!(result["state"], "OutcomeObserved");
        assert_eq!(result["workflow_complete"], true);
        assert_eq!(result["causal_effect_established"], false);
    }
}
