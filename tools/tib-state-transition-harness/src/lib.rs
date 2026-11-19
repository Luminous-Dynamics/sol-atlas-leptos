// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Conservative host-disposition reference model for TIB-PROPERTY-STATE-TRANSITIONS-001.
//!
//! This module never authorizes a national-register or other authoritative mutation.
//! It is intentionally separate from territorial claim validation and legal status.

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    ReviewRequired,
    NotEvaluable,
    StructuralReject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Evaluation {
    pub outcome: Outcome,
    pub host_disposition: &'static str,
    pub authoritative_mutation: bool,
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn bool_at(value: &Value, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_bool()
}

fn scope_covers(scope: &str, requested_decision: &str) -> bool {
    let scope = scope.to_ascii_lowercase();
    if scope.contains("all operations") || scope.contains("unrestricted") {
        return true;
    }
    let requested_terms: Vec<&str> = requested_decision
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 4)
        .collect();
    requested_terms.iter().any(|term| scope.contains(term))
}

/// Evaluate a synthetic host transition conservatively.
///
/// Ordering matters:
/// - An unreconciled timed-out operation rejects a second retry.
/// - A timed-out or unavailable source is not evaluable.
/// - Missing, stale, revoked or wrong-scope authority requires review.
/// - Any active conflict or incomplete evidence frontier requires review.
/// - The default is review, never authoritative mutation.
pub fn evaluate_case(case: &Value) -> Evaluation {
    let retry_attempted = bool_at(case, &["external_operation", "retry_attempted"])
        .unwrap_or(false);
    let first_request_state =
        string_at(case, &["external_operation", "first_request_state"]);
    let first_reconciled = bool_at(
        case,
        &["external_operation", "first_outcome_reconciled"],
    )
    .unwrap_or(false);

    if retry_attempted && first_request_state == Some("timeout_unknown") && !first_reconciled {
        return Evaluation {
            outcome: Outcome::StructuralReject,
            host_disposition: "reject_retry_and_hold",
            authoritative_mutation: false,
        };
    }

    let request_state = string_at(case, &["external_operation", "request_state"]);
    let coverage = string_at(case, &["evidence_frontier", "coverage"]);
    if request_state == Some("timeout_unknown") || coverage == Some("unavailable") {
        return Evaluation {
            outcome: Outcome::NotEvaluable,
            host_disposition: "hold_for_reconciliation",
            authoritative_mutation: false,
        };
    }

    if request_state == Some("not_submitted") {
        return Evaluation {
            outcome: Outcome::ReviewRequired,
            host_disposition: "hold_for_reconciliation",
            authoritative_mutation: false,
        };
    }

    let issuer = string_at(case, &["authority_assessment", "issuer"]);
    let jurisdiction = string_at(case, &["authority_assessment", "jurisdiction"]);
    let scope = string_at(case, &["authority_assessment", "scope"]);
    let currentness = string_at(case, &["authority_assessment", "currentness"]);

    if !matches!(issuer, Some("reference_verified" | "authenticated"))
        || jurisdiction != Some("ZA")
        || !matches!(currentness, Some("current" | "current_within_frontier"))
    {
        return Evaluation {
            outcome: Outcome::ReviewRequired,
            host_disposition: "hold_for_reconciliation",
            authoritative_mutation: false,
        };
    }

    if let (Some(scope), Some(requested)) = (
        scope,
        string_at(case, &["authority_assessment", "requested_decision_type"]),
    ) {
        if !scope_covers(scope, requested) {
            return Evaluation {
                outcome: Outcome::ReviewRequired,
                host_disposition: "hold_for_reconciliation",
                authoritative_mutation: false,
            };
        }
    } else if scope.is_none_or(str::is_empty) {
        return Evaluation {
            outcome: Outcome::ReviewRequired,
            host_disposition: "hold_for_reconciliation",
            authoritative_mutation: false,
        };
    }

    let has_conflicts = case
        .get("conflicts")
        .and_then(Value::as_array)
        .is_none_or(|conflicts| !conflicts.is_empty());
    if has_conflicts || coverage != Some("complete_within_declared_scope") {
        return Evaluation {
            outcome: Outcome::ReviewRequired,
            host_disposition: "hold_for_reconciliation",
            authoritative_mutation: false,
        };
    }

    // This is deliberately not an approval rule. A new positive disposition
    // requires an explicit, separately reviewed protocol version.
    Evaluation {
        outcome: Outcome::ReviewRequired,
        host_disposition: "hold_for_reconciliation",
        authoritative_mutation: false,
    }
}
