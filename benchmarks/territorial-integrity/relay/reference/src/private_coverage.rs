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

fn data_class_set(value: &Value) -> Result<BTreeSet<String>, &'static str> {
    let classes = reference_set(value)?;
    if classes.is_empty() {
        return Err("data_class_set_empty");
    }
    if classes.iter().all(|class| {
        matches!(
            class.as_str(),
            "public" | "community" | "restricted" | "culturally_protected"
        )
    }) {
        Ok(classes)
    } else {
        Err("data_class_unknown")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryMatch {
    True,
    False,
    Unknown,
}

fn scalar(value: &Value) -> bool {
    value.is_string() || value.is_boolean() || value.is_number() || value.is_null()
}

// v0.1 deliberately supports one ASCII object-member segment only. It does
// not execute arbitrary JSONPath, SQL, Rego, user code, or extension functions.
fn query_member<'a>(
    input: Option<&'a Value>,
    path: &str,
) -> Result<Option<&'a Value>, &'static str> {
    let Some(member) = path.strip_prefix('/') else {
        return Err("predicate_path_must_be_single_member_pointer");
    };
    if member.is_empty()
        || member.contains('/')
        || member.contains('~')
        || !member
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        || !member.as_bytes()[0].is_ascii_alphabetic()
    {
        return Err("predicate_path_unsupported");
    }
    Ok(input.and_then(Value::as_object).and_then(|object| object.get(member)))
}

fn invert_query_match(value: QueryMatch) -> QueryMatch {
    match value {
        QueryMatch::True => QueryMatch::False,
        QueryMatch::False => QueryMatch::True,
        QueryMatch::Unknown => QueryMatch::Unknown,
    }
}

fn evaluate_query_predicate(
    predicate: Option<&Value>,
    input: Option<&Value>,
    depth: usize,
) -> Result<QueryMatch, &'static str> {
    if depth > 32 {
        return Err("predicate_depth_exceeded");
    }
    let Some(predicate) = predicate.and_then(Value::as_object) else {
        return Err("predicate_node_missing_or_not_object");
    };
    let Some(operation) = predicate.get("op").and_then(Value::as_str) else {
        return Err("predicate_operator_missing");
    };
    match operation {
        "all" | "any" => {
            let Some(args) = predicate.get("args").and_then(Value::as_array) else {
                return Err("predicate_args_missing_or_invalid");
            };
            if args.is_empty() || args.len() > 32 {
                return Err("predicate_args_count_out_of_range");
            }
            let mut saw_unknown = false;
            for argument in args {
                match evaluate_query_predicate(Some(argument), input, depth + 1)? {
                    QueryMatch::True if operation == "any" => return Ok(QueryMatch::True),
                    QueryMatch::False if operation == "all" => return Ok(QueryMatch::False),
                    QueryMatch::Unknown => saw_unknown = true,
                    _ => {}
                }
            }
            if saw_unknown {
                Ok(QueryMatch::Unknown)
            } else if operation == "all" {
                Ok(QueryMatch::True)
            } else {
                Ok(QueryMatch::False)
            }
        }
        "not" => {
            let Some(argument) = predicate.get("arg") else {
                return Err("predicate_not_arg_missing");
            };
            Ok(invert_query_match(evaluate_query_predicate(
                Some(argument),
                input,
                depth + 1,
            )?))
        }
        "eq" | "in" | "exists" => {
            let Some(path) = predicate.get("path").and_then(Value::as_str) else {
                return Err("predicate_path_missing");
            };
            let actual = query_member(input, path)?;
            if operation == "exists" {
                return Ok(match input.and_then(Value::as_object) {
                    None => QueryMatch::Unknown,
                    Some(_) if actual.is_some() => QueryMatch::True,
                    Some(_) => QueryMatch::False,
                });
            }
            let Some(actual) = actual else {
                return Ok(QueryMatch::Unknown);
            };
            if operation == "eq" {
                let Some(expected) = predicate.get("value") else {
                    return Err("predicate_eq_value_missing");
                };
                if !scalar(expected) {
                    return Err("predicate_eq_value_not_scalar");
                }
                Ok(if actual == expected {
                    QueryMatch::True
                } else {
                    QueryMatch::False
                })
            } else {
                let Some(values) = predicate.get("values").and_then(Value::as_array) else {
                    return Err("predicate_in_values_missing_or_invalid");
                };
                if values.is_empty() || values.len() > 64 || !values.iter().all(scalar) {
                    return Err("predicate_in_values_out_of_range_or_not_scalar");
                }
                Ok(if values.iter().any(|value| value == actual) {
                    QueryMatch::True
                } else {
                    QueryMatch::False
                })
            }
        }
        _ => Err("predicate_operator_unsupported"),
    }
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
    package: &Value,
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

    if receipt.get("package_commitment") != package.get("package_commitment") {
        raise(
            severity,
            reasons,
            Severity::Deny,
            "receipt_package_commitment_mismatch",
        );
    }

    for field in [
        "query_commitment",
        "request_commitment",
        "query_profile_ref",
        "principal_ref",
        "subject_ref",
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

    let query_classes = query
        .get("data_classes")
        .and_then(|value| data_class_set(value).ok());
    let receipt_classes = receipt
        .get("data_classes")
        .and_then(|value| data_class_set(value).ok());
    if query_classes != receipt_classes {
        raise(
            severity,
            reasons,
            Severity::Deny,
            "receipt_query_binding_mismatch:data_classes",
        );
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
        "query_profile_ref",
        "principal_ref",
        "subject_ref",
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
    if nonblank(query, "query_profile_ref") != Some("urn:luminous:cter:predicate-json-v0.1") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "query_profile_missing_or_unsupported",
        );
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

    match query.get("data_classes").map(data_class_set) {
        Some(Ok(_)) => {}
        Some(Err(reason)) => raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            format!("query_data_classes_invalid:{reason}"),
        ),
        None => raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "query_data_classes_missing",
        ),
    }

    let predicate_probe = Value::Object(serde_json::Map::new());
    let predicate_valid = match evaluate_query_predicate(
        query.get("predicate"),
        Some(&predicate_probe),
        0,
    ) {
        Ok(_) => true,
        Err(reason) => {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                format!("query_predicate_invalid:{reason}"),
            );
            false
        }
    };

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
        if predicate_valid {
            let query_result = match evaluate_query_predicate(
                query.get("predicate"),
                item.get("query_input"),
                0,
            ) {
                Ok(result) => result,
                Err(reason) => {
                    raise(
                        &mut severity,
                        &mut reasons,
                        Severity::Incomplete,
                        format!("query_predicate_invalid:{reason}"),
                    );
                    QueryMatch::Unknown
                }
            };
            let expected_scope = match query_result {
                QueryMatch::True => Some("in_scope"),
                QueryMatch::False => Some("out_of_scope"),
                QueryMatch::Unknown => Some("unknown"),
            };
            if scope != expected_scope {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Deny,
                    "partition_scope_does_not_match_query_predicate",
                );
            }
        }
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
        package,
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


#[cfg(test)]
mod predicate_semantics_tests {
    use super::{QueryMatch, evaluate_query_predicate};
    use serde_json::json;

    #[test]
    fn all_false_dominates_unknown() {
        let input = json!({});
        let predicate = json!({
            "op": "all",
            "args": [
                {"op": "eq", "path": "/missing", "value": "x"},
                {"op": "eq", "path": "/present", "value": "y"}
            ]
        });
        // Missing attributes are UNKNOWN; a known FALSE must dominate.
        let input = json!({"present": "n"});
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&input), 0),
            Ok(QueryMatch::False)
        );
    }

    #[test]
    fn all_propagates_unknown_when_no_child_is_false() {
        let input = json!({"dataset": "cadastre"});
        let predicate = json!({
            "op": "all",
            "args": [
                {"op": "eq", "path": "/dataset", "value": "cadastre"},
                {"op": "eq", "path": "/jurisdiction", "value": "ZA"}
            ]
        });
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&input), 0),
            Ok(QueryMatch::Unknown)
        );
    }

    #[test]
    fn any_true_dominates_unknown() {
        let input = json!({"jurisdiction": "ZA"});
        let predicate = json!({
            "op": "any",
            "args": [
                {"op": "eq", "path": "/dataset", "value": "cadastre"},
                {"op": "eq", "path": "/jurisdiction", "value": "ZA"}
            ]
        });
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&input), 0),
            Ok(QueryMatch::True)
        );
    }

    #[test]
    fn any_propagates_unknown_when_no_child_is_true() {
        let input = json!({"jurisdiction": "US"});
        let predicate = json!({
            "op": "any",
            "args": [
                {"op": "eq", "path": "/dataset", "value": "cadastre"},
                {"op": "eq", "path": "/jurisdiction", "value": "ZA"}
            ]
        });
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&input), 0),
            Ok(QueryMatch::Unknown)
        );
    }

    #[test]
    fn not_preserves_unknown() {
        let predicate = json!({
            "op": "not",
            "arg": {"op": "eq", "path": "/missing", "value": "x"}
        });
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&json!({})), 0),
            Ok(QueryMatch::Unknown)
        );
    }

    #[test]
    fn exists_distinguishes_unavailable_input_from_known_absence() {
        let predicate = json!({"op": "exists", "path": "/jurisdiction"});
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), None, 0),
            Ok(QueryMatch::Unknown)
        );
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&json!({})), 0),
            Ok(QueryMatch::False)
        );
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&json!({"jurisdiction": "ZA"})), 0),
            Ok(QueryMatch::True)
        );
    }

    #[test]
    fn in_uses_exact_scalar_value_equality() {
        let predicate = json!({
            "op": "in",
            "path": "/jurisdiction",
            "values": ["ZA", "BW", "LS"]
        });
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&json!({"jurisdiction": "ZA"})), 0),
            Ok(QueryMatch::True)
        );
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&json!({"jurisdiction": "US"})), 0),
            Ok(QueryMatch::False)
        );
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&json!({})), 0),
            Ok(QueryMatch::Unknown)
        );
    }

    #[test]
    fn unsupported_operators_fail_closed() {
        let predicate = json!({"op": "regex", "path": "/name", "value": ".*"});
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&json!({"name": "item"})), 0),
            Err("predicate_operator_unsupported")
        );
    }

    #[test]
    fn predicate_depth_is_bounded() {
        let mut predicate = json!({"op": "eq", "path": "/x", "value": "1"});
        for _ in 0..34 {
            predicate = json!({"op": "not", "arg": predicate});
        }
        assert_eq!(
            evaluate_query_predicate(Some(&predicate), Some(&json!({"x": "1"})), 0),
            Err("predicate_depth_exceeded")
        );
    }
}
