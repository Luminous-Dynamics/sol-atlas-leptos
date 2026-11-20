//! Synthetic private-coverage partition evaluator.
//!
//! This module verifies only structural/set relationships in a custodian-side
//! coverage record. It does not authenticate receipts, verify COSE/VDS proofs,
//! prove source-owner completeness, establish trusted time, or authorize export.
//! A valid_partition_structure result is not cryptographic proof or permission.

use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageEvaluation {
    pub disposition: &'static str,
    pub reason_codes: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    None,
    Incomplete,
    Unknown,
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

fn lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_digest(value: &Value) -> bool {
    nonblank(value, "algorithm") == Some("sha-256")
        && nonblank(value, "value").is_some_and(lower_hex_64)
}

fn reference_set(value: &Value) -> Result<BTreeSet<String>, &'static str> {
    let Some(items) = value.as_array() else {
        return Err("reference_list_missing_or_invalid");
    };
    let mut refs = BTreeSet::new();
    for item in items {
        let Some(reference) = item.as_str().filter(|text| !text.trim().is_empty()) else {
            return Err("reference_list_contains_empty_item");
        };
        if !refs.insert(reference.to_owned()) {
            return Err("duplicate_reference");
        }
    }
    Ok(refs)
}

fn finalize(severity: Severity, reasons: BTreeSet<String>) -> CoverageEvaluation {
    let disposition = match severity {
        Severity::None => "valid_partition_structure",
        Severity::Incomplete => "incomplete",
        Severity::Unknown => "unknown",
        Severity::Deny => "deny",
    };
    CoverageEvaluation {
        disposition,
        reason_codes: reasons,
    }
}

fn check_public_view(
    view: Option<&Value>,
    frontier: &Value,
    query: &Value,
    package: &Value,
    severity: &mut Severity,
    reasons: &mut BTreeSet<String>,
) {
    let Some(view) = view else {
        raise(
            severity,
            reasons,
            Severity::Incomplete,
            "public_verifier_view_missing",
        );
        return;
    };
    let Some(object) = view.as_object() else {
        raise(
            severity,
            reasons,
            Severity::Deny,
            "public_verifier_view_not_object",
        );
        return;
    };

    const ALLOWED: &[&str] = &[
        "frontier_commitment",
        "query_commitment",
        "request_commitment",
        "package_commitment",
        "proof_ref",
        "non_inclusion_scope",
    ];
    if object.keys().any(|key| !ALLOWED.contains(&key.as_str())) {
        raise(
            severity,
            reasons,
            Severity::Deny,
            "public_verifier_view_over_discloses",
        );
    }
    if nonblank(view, "proof_ref").is_none() {
        raise(
            severity,
            reasons,
            Severity::Incomplete,
            "public_proof_reference_missing",
        );
    }

    if nonblank(view, "non_inclusion_scope") != Some("frontier_version_only") {
        raise(
            severity,
            reasons,
            Severity::Deny,
            "non_inclusion_scope_exceeds_committed_frontier",
        );
    }

    for (public_field, source) in [
        ("frontier_commitment", frontier.get("commitment")),
        ("query_commitment", query.get("query_commitment")),
        ("request_commitment", query.get("request_commitment")),
        ("package_commitment", package.get("package_commitment")),
    ] {
        if view.get(public_field) != source {
            raise(
                severity,
                reasons,
                Severity::Deny,
                format!("public_commitment_binding_mismatch:{public_field}"),
            );
        }
        if !view.get(public_field).is_some_and(sha256_digest) {
            raise(
                severity,
                reasons,
                Severity::Incomplete,
                format!("public_commitment_malformed:{public_field}"),
            );
        }
    }
}

fn compare_receipt_projection(
    input: &Value,
    frontier: &Value,
    query: &Value,
    severity: &mut Severity,
    reasons: &mut BTreeSet<String>,
) {
    let Some(receipt) = input.get("receipt_binding") else {
        raise(
            severity,
            reasons,
            Severity::Incomplete,
            "receipt_binding_missing",
        );
        return;
    };

    if nonblank(receipt, "receipt_ref") != nonblank(frontier, "receipt_ref") {
        raise(
            severity,
            reasons,
            Severity::Unknown,
            "receipt_reference_binding_unresolved",
        );
    }
    if nonblank(receipt, "frontier_ref") != nonblank(frontier, "frontier_ref")
        || nonblank(receipt, "frontier_version") != nonblank(frontier, "version")
        || receipt.get("frontier_commitment") != frontier.get("commitment")
    {
        raise(
            severity,
            reasons,
            Severity::Unknown,
            "receipt_frontier_version_or_commitment_mismatch",
        );
    }

    for field in [
        "query_commitment",
        "request_commitment",
        "principal_ref",
        "operation",
        "purpose_ref",
        "recipient_ref",
        "resource_ref",
        "jurisdiction_profile_ref",
        "project_snapshot_sha256",
    ] {
        if receipt.get(field) != query.get(field) {
            raise(
                severity,
                reasons,
                Severity::Deny,
                format!("receipt_query_binding_mismatch:{field}"),
            );
        }
    }

    if !receipt
        .get("query_commitment")
        .is_some_and(sha256_digest)
        || !receipt
            .get("request_commitment")
            .is_some_and(sha256_digest)
    {
        raise(
            severity,
            reasons,
            Severity::Incomplete,
            "receipt_query_or_request_commitment_malformed",
        );
    }
}

/// Check one private partition against a named frontier and exact package source leaves.
///
/// A valid_partition_structure result is only structural agreement. It does
/// not prove that the frontier is complete in the world or that a receipt is
/// authentic, fresh or legally authoritative.
pub fn evaluate_private_coverage(input: &Value) -> CoverageEvaluation {
    let mut severity = Severity::None;
    let mut reasons = BTreeSet::new();

    if nonblank(input, "schema") != Some("luminous.cter.private-coverage.v0.1") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "coverage_schema_missing_or_unsupported",
        );
    }
    if nonblank(input, "coverage_ref").is_none() {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "coverage_reference_missing",
        );
    }

    let Some(frontier) = input.get("frontier") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "frontier_missing",
        );
        return finalize(severity, reasons);
    };
    for field in ["frontier_ref", "version", "source_owner_ref", "receipt_ref"] {
        if nonblank(frontier, field).is_none() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                format!("frontier_{field}_missing"),
            );
        }
    }
    if !frontier.get("commitment").is_some_and(sha256_digest) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "frontier_commitment_malformed",
        );
    }
    let frontier_refs = match frontier.get("object_refs").map(reference_set) {
        Some(Ok(refs)) if !refs.is_empty() => refs,
        Some(Ok(_)) => {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "frontier_object_set_empty",
            );
            BTreeSet::new()
        }
        Some(Err(reason)) => {
            raise(&mut severity, &mut reasons, Severity::Deny, reason);
            BTreeSet::new()
        }
        None => {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "frontier_object_set_missing",
            );
            BTreeSet::new()
        }
    };

    let Some(query) = input.get("query_binding") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "query_binding_missing",
        );
        return finalize(severity, reasons);
    };
    for field in [
        "query_ref",
        "principal_ref",
        "operation",
        "purpose_ref",
        "recipient_ref",
        "resource_ref",
        "jurisdiction_profile_ref",
    ] {
        if nonblank(query, field).is_none() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                format!("query_{field}_missing"),
            );
        }
    }
    if !matches!(
        nonblank(query, "operation"),
        Some("export_evidence" | "publish_observation")
    ) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "coverage_operation_outside_supported_scope",
        );
    }
    for field in ["query_commitment", "request_commitment"] {
        if !query.get(field).is_some_and(sha256_digest) {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                format!("query_{field}_malformed"),
            );
        }
    }
    if !nonblank(query, "project_snapshot_sha256").is_some_and(lower_hex_64) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "query_project_snapshot_malformed",
        );
    }

    let Some(partition) = input.get("private_partition").and_then(Value::as_array) else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "private_partition_missing",
        );
        return finalize(severity, reasons);
    };
    let mut partition_refs = BTreeSet::new();
    let mut included_refs = BTreeSet::new();

    for item in partition {
        let Some(object_ref) = nonblank(item, "object_ref") else {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "partition_object_reference_missing",
            );
            continue;
        };
        if !partition_refs.insert(object_ref.to_owned()) {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Deny,
                "partition_object_disposition_duplicated",
            );
        }

        let scope = nonblank(item, "scope_state");
        let disposition = nonblank(item, "disposition");
        match (scope, disposition) {
            (Some("in_scope"), Some("included")) => {
                included_refs.insert(object_ref.to_owned());
            }
            (Some("in_scope"), Some("withheld" | "unavailable" | "unresolved")) => {}
            (Some("out_of_scope"), Some("excluded")) => {}
            (Some("unknown"), Some("unresolved")) => {}
            (_, Some("absent")) => raise(
                &mut severity,
                &mut reasons,
                Severity::Deny,
                "partition_absence_state_forbidden",
            ),
            (None, _) | (_, None) => raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "partition_scope_or_disposition_missing",
            ),
            _ => raise(
                &mut severity,
                &mut reasons,
                Severity::Deny,
                "partition_scope_disposition_conflict",
            ),
        }
    }

    if partition_refs != frontier_refs {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "partition_frontier_object_set_mismatch",
        );
    }

    let Some(package) = input.get("package_binding") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "package_binding_missing",
        );
        return finalize(severity, reasons);
    };
    if !package.get("package_commitment").is_some_and(sha256_digest) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "package_commitment_malformed",
        );
    }
    let package_leaves = match package
        .get("terminal_source_leaf_refs")
        .map(reference_set)
    {
        Some(Ok(refs)) => refs,
        Some(Err(reason)) => {
            raise(&mut severity, &mut reasons, Severity::Deny, reason);
            BTreeSet::new()
        }
        None => {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "package_terminal_source_leaves_missing",
            );
            BTreeSet::new()
        }
    };
    if package_leaves != included_refs {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "package_included_source_set_mismatch",
        );
    }
    if !package_leaves.is_subset(&frontier_refs) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "package_contains_phantom_frontier_source",
        );
    }

    compare_receipt_projection(
        input,
        frontier,
        query,
        &mut severity,
        &mut reasons,
    );

    match nonblank(
        input.get("receipt_assessment").unwrap_or(&Value::Null),
        "resolution_status",
    ) {
        Some("current") => {}
        Some("stale") => raise(
            &mut severity,
            &mut reasons,
            Severity::Unknown,
            "vds_receipt_stale",
        ),
        Some("unavailable") => raise(
            &mut severity,
            &mut reasons,
            Severity::Unknown,
            "vds_receipt_unavailable",
        ),
        Some("unknown") | None => raise(
            &mut severity,
            &mut reasons,
            Severity::Unknown,
            "vds_receipt_currentness_unknown",
        ),
        Some("revoked") => raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "vds_receipt_revoked",
        ),
        _ => raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "vds_receipt_assessment_invalid",
        ),
    }
    let assessment = input.get("receipt_assessment").unwrap_or(&Value::Null);
    if nonblank(assessment, "assessment_ref").is_none()
        || nonblank(assessment, "status_snapshot_ref").is_none()
    {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "vds_receipt_assessment_lineage_missing",
        );
    }

    check_public_view(
        input.get("public_verifier_view"),
        frontier,
        query,
        package,
        &mut severity,
        &mut reasons,
    );

    finalize(severity, reasons)
}
