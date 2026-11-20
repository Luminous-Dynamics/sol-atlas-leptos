//! CTER consumer-side authority and consent composition reference.
//!
//! This module consumes host-resolved Mycelix AuthorizationReceipt-shaped
//! records. It does not authenticate signatures, resolve authorities, query
//! revocation, establish community representation, or create legal effect.
//! "eligible" means only that the consumer contract is structurally satisfied.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationEvaluation {
    pub disposition: &'static str,
    pub reason_codes: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    None,
    Incomplete,
    Unknown,
    Review,
    Deny,
}

fn raise(
    severity: &mut Severity,
    reasons: &mut BTreeSet<String>,
    level: Severity,
    code: impl Into<String>,
) {
    *severity = (*severity).max(level);
    reasons.insert(code.into());
}

fn nonblank<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)?
        .as_str()
        .filter(|text| !text.trim().is_empty())
}

fn string_set(value: &Value, field: &str) -> Option<BTreeSet<String>> {
    let array = value.get(field)?.as_array()?;
    let mut out = BTreeSet::new();
    for item in array {
        let text = item.as_str()?.trim();
        if text.is_empty() || !out.insert(text.to_owned()) {
            return None;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn utc_timestamp(value: &str) -> bool {
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
    if !bytes.iter().enumerate().all(|(index, byte)| {
        matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
    }) {
        return false;
    }
    let number = |start: usize, end: usize| -> Option<u32> {
        std::str::from_utf8(&bytes[start..end]).ok()?.parse().ok()
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0, 4), number(5, 7), number(8, 10),
        number(11, 13), number(14, 16), number(17, 19)
    ) else {
        return false;
    };
    if year == 0 || !(1..=12).contains(&month)
        || hour > 23 || minute > 59 || second > 60
    {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sensitive(classes: &BTreeSet<String>) -> bool {
    classes.contains("community") || classes.contains("culturally_protected")
}

fn required_domains(operation: &str, is_sensitive: bool) -> Option<BTreeSet<&'static str>> {
    let mut out = BTreeSet::new();
    match operation {
        "collect_measurement" => {
            out.insert("participant_consent");
            out.insert("site_access");
            if is_sensitive {
                out.insert("community_mandate");
            }
        }
        "publish_observation" => {
            out.insert("publication_consent");
            if is_sensitive {
                out.insert("community_disclosure");
            }
        }
        "export_evidence" => {
            out.insert("archive_export");
            if is_sensitive {
                out.insert("community_disclosure");
            }
        }
        _ => return None,
    }
    Some(out)
}

fn exact_scope(receipt: &Value, request: &Value) -> bool {
    for field in [
        "principal_ref",
        "subject_ref",
        "operation",
        "resource_ref",
        "project_snapshot_sha256",
        "purpose_ref",
        "recipient_ref",
        "jurisdiction_profile_ref",
    ] {
        if receipt.get(field) != request.get(field) {
            return false;
        }
    }
    string_set(receipt, "authorized_data_classes") == string_set(request, "data_classes")
}

fn evaluate_receipt(
    receipt: &Value,
    request: &Value,
    evaluated_at: &str,
    offline_mode: bool,
    severity: &mut Severity,
    reasons: &mut BTreeSet<String>,
) -> Option<String> {
    let _receipt_ref = nonblank(receipt, "receipt_ref")?.to_owned();

    if nonblank(receipt, "receipt_type") != Some("AuthorizationReceipt") {
        raise(
            severity,
            reasons,
            Severity::Deny,
            "receipt_type_laundering",
        );
    }
    if !exact_scope(receipt, request) {
        raise(
            severity,
            reasons,
            Severity::Deny,
            "decision_scope_mismatch",
        );
    }
    if nonblank(receipt, "policy_ref").is_none()
        || nonblank(receipt, "status_snapshot_ref").is_none()
        || string_set(receipt, "input_receipt_refs").is_none()
    {
        raise(
            severity,
            reasons,
            Severity::Incomplete,
            "decision_lineage_incomplete",
        );
    }

    if nonblank(receipt, "decision_maker_ref").is_none()
        || nonblank(receipt, "decision_authority_ref").is_none()
    {
        raise(
            severity,
            reasons,
            Severity::Incomplete,
            "decision_authority_lineage_missing",
        );
    }

    let domain = nonblank(receipt, "decision_domain")?.to_owned();
    if !matches!(
        domain.as_str(),
        "participant_consent"
            | "site_access"
            | "community_mandate"
            | "publication_consent"
            | "community_disclosure"
            | "archive_export"
    ) {
        raise(
            severity,
            reasons,
            Severity::Incomplete,
            "decision_domain_unknown",
        );
    }

    match nonblank(receipt, "disposition") {
        Some("allow") => {}
        Some("deny") => raise(
            severity,
            reasons,
            Severity::Deny,
            "upstream_decision_denied",
        ),
        Some("review") => raise(
            severity,
            reasons,
            Severity::Review,
            "upstream_decision_requires_review",
        ),
        Some("unknown") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "upstream_decision_unknown",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "decision_disposition_invalid",
        ),
    }

    match nonblank(receipt, "signature_status") {
        Some("verified") => {}
        Some("invalid") => raise(
            severity,
            reasons,
            Severity::Deny,
            "receipt_signature_invalid",
        ),
        Some("unavailable") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "receipt_signature_unavailable",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "receipt_signature_status_missing",
        ),
    }

    match nonblank(receipt, "issuer_trust_status") {
        Some("trusted") => {}
        Some("untrusted") => raise(
            severity,
            reasons,
            Severity::Deny,
            "receipt_issuer_untrusted",
        ),
        Some("unavailable") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "receipt_issuer_trust_unavailable",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "receipt_issuer_trust_status_missing",
        ),
    }

    match nonblank(receipt, "decision_authority_status") {
        Some("resolved") => {}
        Some("revoked") => raise(
            severity,
            reasons,
            Severity::Deny,
            "decision_maker_authority_revoked",
        ),
        Some("unresolved") => raise(
            severity,
            reasons,
            Severity::Review,
            "decision_maker_authority_unresolved",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "decision_authority_status_missing",
        ),
    }

    match nonblank(receipt, "revocation_status") {
        Some("clear") => {}
        Some("revoked") => raise(
            severity,
            reasons,
            Severity::Deny,
            "decision_revoked",
        ),
        Some("unknown") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "revocation_state_unknown",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "revocation_status_missing",
        ),
    }

    match nonblank(receipt, "currentness_status") {
        Some("current") => {}
        Some("stale") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "decision_currentness_stale",
        ),
        Some("unknown") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "decision_currentness_unknown",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "currentness_status_missing",
        ),
    }

    match (
        nonblank(receipt, "valid_from"),
        nonblank(receipt, "valid_until"),
    ) {
        (Some(start), Some(end))
            if utc_timestamp(start) && utc_timestamp(end) && start < end =>
        {
            if evaluated_at < start {
                raise(
                    severity,
                    reasons,
                    Severity::Deny,
                    "decision_not_yet_valid",
                );
            } else if evaluated_at >= end {
                raise(
                    severity,
                    reasons,
                    Severity::Deny,
                    "decision_expired",
                );
            }
        }
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "decision_validity_window_invalid",
        ),
    }

    match receipt.get("offline_lease_expires_at") {
        Some(Value::String(expiry)) if utc_timestamp(expiry) => {
            if offline_mode && evaluated_at >= expiry.as_str() {
                raise(
                    severity,
                    reasons,
                    Severity::Unknown,
                    "offline_authority_lease_expired",
                );
            }
        }
        Some(Value::Null) if !offline_mode => {}
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "offline_lease_reference_invalid",
        ),
    }

    Some(domain)
}

fn evaluate_governance(
    context: Option<&Value>,
    evaluated_at: &str,
    offline_mode: bool,
    severity: &mut Severity,
    reasons: &mut BTreeSet<String>,
) {
    let Some(context) = context else {
        raise(
            severity,
            reasons,
            Severity::Unknown,
            "community_governance_context_missing",
        );
        return;
    };

    for field in [
        "governance_profile_ref",
        "decision_procedure_ref",
        "representation_assessment_ref",
        "appeal_route_ref",
    ] {
        if nonblank(context, field).is_none() {
            raise(
                severity,
                reasons,
                Severity::Incomplete,
                format!("governance_{field}_missing"),
            );
        }
    }

    match nonblank(context, "representation_status") {
        Some("resolved") => {}
        Some("contested") => raise(
            severity,
            reasons,
            Severity::Review,
            "community_representation_contested",
        ),
        Some("unknown") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "community_representation_unknown",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "community_representation_status_invalid",
        ),
    }

    match nonblank(context, "coverage_status") {
        Some("complete_within_declared_scope") => {}
        Some("known_gaps") => raise(
            severity,
            reasons,
            Severity::Review,
            "community_representation_has_gaps",
        ),
        Some("unknown") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "community_coverage_unknown",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "community_coverage_status_invalid",
        ),
    }

    match nonblank(context, "dissent_status") {
        Some("none") => {}
        Some("recorded") => {
            if nonblank(context, "dissent_register_ref").is_none() {
                raise(
                    severity,
                    reasons,
                    Severity::Incomplete,
                    "recorded_dissent_reference_missing",
                );
            }
            // Recorded dissent is preserved; it is not automatically a veto.
        }
        Some("material_unresolved") => raise(
            severity,
            reasons,
            Severity::Review,
            "material_dissent_unresolved",
        ),
        Some("unknown") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "dissent_state_unknown",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "dissent_status_invalid",
        ),
    }

    match nonblank(context, "challenge_status") {
        Some("none" | "resolved") => {}
        Some("open") => raise(
            severity,
            reasons,
            Severity::Review,
            "material_challenge_open",
        ),
        Some("unknown") => raise(
            severity,
            reasons,
            Severity::Unknown,
            "challenge_state_unknown",
        ),
        _ => raise(
            severity,
            reasons,
            Severity::Incomplete,
            "challenge_status_invalid",
        ),
    }

    if offline_mode {
        match nonblank(context, "governance_offline_lease_expires_at") {
            Some(expiry) if utc_timestamp(expiry) && evaluated_at < expiry => {}
            Some(expiry) if utc_timestamp(expiry) => raise(
                severity,
                reasons,
                Severity::Unknown,
                "governance_offline_lease_expired",
            ),
            _ => raise(
                severity,
                reasons,
                Severity::Unknown,
                "governance_offline_lease_missing_or_invalid",
            ),
        }
    }
}

fn finalize(
    severity: Severity,
    reason_codes: BTreeSet<String>,
) -> AuthorizationEvaluation {
    let disposition = match severity {
        Severity::None => "eligible",
        Severity::Incomplete => "incomplete",
        Severity::Unknown => "unknown",
        Severity::Review => "review",
        Severity::Deny => "deny",
    };
    AuthorizationEvaluation {
        disposition,
        reason_codes,
    }
}

/// Evaluate one exact CTER request against already resolved upstream decisions.
///
/// Only AuthorizationReceipt can satisfy an action prerequisite. A production
/// host must verify the upstream Mycelix records and receipts before calling
/// this structural consumer-side evaluator.
pub fn evaluate_authority_use(input: &Value) -> AuthorizationEvaluation {
    let mut severity = Severity::None;
    let mut reasons = BTreeSet::new();
    let Some(request) = input.get("request") else {
        return finalize(
            Severity::Incomplete,
            BTreeSet::from(["request_missing".to_owned()]),
        );
    };

    for field in [
        "request_id",
        "principal_ref",
        "subject_ref",
        "operation",
        "resource_ref",
        "project_snapshot_sha256",
        "purpose_ref",
        "recipient_ref",
        "jurisdiction_profile_ref",
    ] {
        if nonblank(request, field).is_none() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                format!("request_{field}_missing"),
            );
        }
    }

    let Some(evaluated_at) = nonblank(request, "evaluated_at") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "request_evaluated_at_missing",
        );
        return finalize(severity, reasons);
    };
    if !utc_timestamp(evaluated_at) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "request_evaluated_at_invalid",
        );
    }

    if !nonblank(request, "project_snapshot_sha256")
        .is_some_and(is_sha256)
    {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "request_project_snapshot_sha256_invalid",
        );
    }

    let Some(classes) = string_set(request, "data_classes") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "request_data_classes_missing_or_invalid",
        );
        return finalize(severity, reasons);
    };
    if classes.iter().any(|class| {
        !matches!(
            class.as_str(),
            "public" | "community" | "restricted" | "culturally_protected"
        )
    }) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "request_data_class_unknown",
        );
    }

    let Some(offline_mode) = request.get("offline_mode").and_then(Value::as_bool) else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "request_offline_mode_missing",
        );
        return finalize(severity, reasons);
    };

    let operation = nonblank(request, "operation").unwrap_or("");
    if operation.is_empty() {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "request_operation_missing",
        );
        return finalize(severity, reasons);
    }
    if operation == "assert_legal_title" {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "legal_effect_outside_cter",
        );
        return finalize(severity, reasons);
    }

    let Some(required) = required_domains(operation, sensitive(&classes)) else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "operation_outside_cter_authority_boundary",
        );
        return finalize(severity, reasons);
    };

    let Some(decisions) = input.get("decisions").and_then(Value::as_array) else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Unknown,
            "upstream_decision_bundle_unavailable",
        );
        return finalize(severity, reasons);
    };

    let mut by_domain: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut receipt_contents: BTreeMap<String, String> = BTreeMap::new();
    for receipt in decisions {
        let Some(receipt_ref) = nonblank(receipt, "receipt_ref") else {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "decision_receipt_malformed",
            );
            continue;
        };
        let content = serde_json::to_string(receipt).unwrap_or_default();
        if let Some(previous) = receipt_contents.get(receipt_ref) {
            if previous != &content {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Deny,
                    "authorization_receipt_identity_conflict",
                );
            }
            // Exact duplicate delivery is idempotent; one receipt is one decision.
            continue;
        }
        receipt_contents.insert(receipt_ref.to_owned(), content);
        match evaluate_receipt(
            receipt,
            request,
            evaluated_at,
            offline_mode,
            &mut severity,
            &mut reasons,
        ) {
            Some(domain) => {
                by_domain.entry(domain).or_default().push(
                    nonblank(receipt, "receipt_ref")
                        .unwrap_or("")
                        .to_owned(),
                );
            }
            None => raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "decision_receipt_malformed",
            ),
        }
    }

    for domain in &required {
        match by_domain.get(*domain) {
            None => raise(
                &mut severity,
                &mut reasons,
                Severity::Unknown,
                format!("required_decision_missing:{domain}"),
            ),
            Some(refs)
                if refs.iter().map(String::as_str).collect::<BTreeSet<_>>().len() > 1 =>
            {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Review,
                    format!("multiple_decisions_for_domain:{domain}"),
                );
            }
            _ => {}
        }
    }

    for domain in by_domain.keys() {
        if !required.contains(domain.as_str()) {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Review,
                format!("unexpected_decision_domain:{domain}"),
            );
        }
    }

    if required.contains("community_mandate")
        || required.contains("community_disclosure")
    {
        evaluate_governance(
            input.get("governance_context"),
            evaluated_at,
            offline_mode,
            &mut severity,
            &mut reasons,
        );
    }

    finalize(severity, reasons)
}
