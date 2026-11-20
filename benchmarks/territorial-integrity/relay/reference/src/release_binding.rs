//! Cross-bind the CTER egress manifest and private coverage record.
//!
//! This composes two structural evaluators. It does not authenticate the
//! Mycelix receipt, verify a VDS proof, hash supplied payload bytes, resolve
//! consent/authority/currentness, or authorize a transmission.

use crate::egress::{
    evaluate_disclosure_manifest, terminal_source_leaf_refs,
};
use crate::private_coverage::evaluate_private_coverage;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseBindingEvaluation {
    pub disposition: &'static str,
    pub reason_codes: BTreeSet<String>,
    pub derived_terminal_source_leaf_refs: BTreeSet<String>,
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

fn reference_set(value: Option<&Value>) -> Option<BTreeSet<String>> {
    let values = value?.as_array()?;
    let mut refs = BTreeSet::new();
    for item in values {
        let reference = item.as_str()?.trim();
        if reference.is_empty() || !refs.insert(reference.to_owned()) {
            return None;
        }
    }
    Some(refs)
}

fn data_class_set(value: Option<&Value>) -> Option<BTreeSet<String>> {
    reference_set(value)
}

fn map_egress_disposition(value: &str) -> Severity {
    match value {
        "valid_structure" => Severity::None,
        "incomplete" => Severity::Incomplete,
        "unknown" => Severity::Unknown,
        "review" => Severity::Review,
        "deny" => Severity::Deny,
        _ => Severity::Incomplete,
    }
}

fn map_coverage_disposition(value: &str) -> Severity {
    match value {
        "valid_partition_structure" => Severity::None,
        "incomplete" => Severity::Incomplete,
        "unknown" => Severity::Unknown,
        "deny" => Severity::Deny,
        _ => Severity::Incomplete,
    }
}

fn finalize(
    severity: Severity,
    reason_codes: BTreeSet<String>,
    derived_terminal_source_leaf_refs: BTreeSet<String>,
) -> ReleaseBindingEvaluation {
    let disposition = match severity {
        Severity::None => "valid_composed_structure",
        Severity::Incomplete => "incomplete",
        Severity::Unknown => "unknown",
        Severity::Review => "review",
        Severity::Deny => "deny",
    };
    ReleaseBindingEvaluation {
        disposition,
        reason_codes,
        derived_terminal_source_leaf_refs,
    }
}

/// Prove structural agreement between one egress manifest and one private
/// coverage record. This is not a signature, VDS, authorization or transport
/// verifier. It ensures that the two independently valid records describe
/// the same exact use and package, and that coverage leaves are derived from
/// the egress manifest's actual transformation graph.
pub fn evaluate_composed_release_binding(
    manifest: &Value,
    coverage: &Value,
) -> ReleaseBindingEvaluation {
    let egress = evaluate_disclosure_manifest(manifest);
    let partition = evaluate_private_coverage(coverage);
    let mut severity = map_egress_disposition(egress.disposition)
        .max(map_coverage_disposition(partition.disposition));
    let mut reasons = BTreeSet::new();
    let mut derived_leaves = BTreeSet::new();

    for reason in egress.reason_codes {
        reasons.insert(format!("egress:{reason}"));
    }
    for reason in partition.reason_codes {
        reasons.insert(format!("coverage:{reason}"));
    }

    if severity != Severity::None {
        return finalize(severity, reasons, derived_leaves);
    }

    let Some(coverage_ref) = nonblank(coverage, "coverage_ref") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "coverage_reference_missing",
        );
        return finalize(severity, reasons, derived_leaves);
    };
    if nonblank(manifest, "coverage_record_ref") != Some(coverage_ref) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "coverage_record_reference_mismatch",
        );
    }

    let Some(package) = manifest.get("package") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "egress_package_missing",
        );
        return finalize(severity, reasons, derived_leaves);
    };
    let Some(coverage_package) = coverage.get("package_binding") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "coverage_package_binding_missing",
        );
        return finalize(severity, reasons, derived_leaves);
    };
    if package.get("package_payload_digest") != coverage_package.get("package_commitment") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "package_payload_commitment_mismatch",
        );
    }

    let Some(authority) = manifest.get("authorization_binding") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "egress_authorization_binding_missing",
        );
        return finalize(severity, reasons, derived_leaves);
    };
    let Some(query) = coverage.get("query_binding") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "coverage_query_binding_missing",
        );
        return finalize(severity, reasons, derived_leaves);
    };
    let Some(frontier) = coverage.get("frontier") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "coverage_frontier_missing",
        );
        return finalize(severity, reasons, derived_leaves);
    };

    for (manifest_field, coverage_field) in [
        ("request_commitment", "request_commitment"),
        ("principal_ref", "principal_ref"),
        ("subject_ref", "subject_ref"),
        ("operation", "operation"),
        ("purpose_ref", "purpose_ref"),
        ("recipient_ref", "recipient_ref"),
        ("resource_ref", "resource_ref"),
        ("jurisdiction_profile_ref", "jurisdiction_profile_ref"),
        ("project_snapshot_sha256", "project_snapshot_sha256"),
    ] {
        if authority.get(manifest_field) != query.get(coverage_field) {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Deny,
                format!("authorization_query_binding_mismatch:{manifest_field}"),
            );
        }
    }
    if nonblank(authority, "evidence_frontier_ref") != nonblank(frontier, "frontier_ref") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "authorization_frontier_reference_mismatch",
        );
    }

    if nonblank(authority, "coverage_query_ref") != nonblank(query, "query_ref") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "authorization_coverage_query_reference_mismatch",
        );
    }
    if authority.get("coverage_query_commitment") != query.get("query_commitment") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "authorization_coverage_query_commitment_mismatch",
        );
    }
    if authority.get("coverage_query_profile_ref") != query.get("query_profile_ref") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "authorization_coverage_query_profile_mismatch",
        );
    }

    let authorized_classes = data_class_set(authority.get("data_classes"));
    let coverage_classes = data_class_set(query.get("data_classes"));
    match (authorized_classes, coverage_classes) {
        (Some(left), Some(right)) if left == right => {}
        (Some(_), Some(_)) => raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "authorization_data_class_set_mismatch",
        ),
        _ => raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "authorization_or_coverage_data_classes_invalid",
        ),
    }

    match terminal_source_leaf_refs(manifest) {
        Ok(leaves) => {
            derived_leaves = leaves;
            let asserted_leaves = reference_set(
                coverage_package.get("terminal_source_leaf_refs"),
            );
            if asserted_leaves.as_ref() != Some(&derived_leaves) {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Deny,
                    "derived_terminal_source_leaf_set_mismatch",
                );
            }
        }
        Err(reason) => raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            format!("terminal_source_leaf_derivation_failed:{reason}"),
        ),
    }

    finalize(severity, reasons, derived_leaves)
}


/// Strict raw-JSON entry point for composed releases. Both records pass
/// through the same duplicate-member and I-JSON numeric boundary before any
/// structural evaluator consumes them.
pub fn evaluate_composed_release_binding_json(
    manifest_json: &str,
    coverage_json: &str,
) -> Result<ReleaseBindingEvaluation, serde_json::Error> {
    let manifest = crate::private_coverage::parse_unique_json_value(manifest_json)?;
    let coverage = crate::private_coverage::parse_unique_json_value(coverage_json)?;
    Ok(evaluate_composed_release_binding(&manifest, &coverage))
}
