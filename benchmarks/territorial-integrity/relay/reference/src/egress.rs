//! CTER disclosure-manifest structural qualification.
//!
//! This module checks internal manifest consistency only. It does not hash
//! payload bytes, verify RFC 8785 canonicalization, authenticate Mycelix
//! receipts, resolve source availability, qualify declassification, establish
//! trusted time, or authorize a release.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgressEvaluation {
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

fn string_set(value: &Value) -> Option<BTreeSet<String>> {
    let values = value.as_array()?;
    let mut out = BTreeSet::new();
    for item in values {
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

fn valid_data_classes(value: &Value) -> Option<BTreeSet<String>> {
    let classes = string_set(value)?;
    if classes.iter().all(|class| {
        matches!(
            class.as_str(),
            "public" | "community" | "restricted" | "culturally_protected"
        )
    }) {
        Some(classes)
    } else {
        None
    }
}

// Kahn's topological elimination avoids recursion on deep provenance chains.
fn has_cycle(graph: &BTreeMap<String, BTreeSet<String>>) -> bool {
    let mut indegree: BTreeMap<String, usize> = BTreeMap::new();
    for (node, children) in graph {
        indegree.entry(node.clone()).or_insert(0);
        for child in children {
            *indegree.entry(child.clone()).or_insert(0) += 1;
        }
    }

    let mut ready: Vec<String> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(node, _)| node.clone())
        .collect();
    let mut processed = 0usize;
    while let Some(node) = ready.pop() {
        processed += 1;
        if let Some(children) = graph.get(&node) {
            for child in children {
                if let Some(degree) = indegree.get_mut(child) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(child.clone());
                    }
                }
            }
        }
    }
    processed != indegree.len()
}

fn finalize(severity: Severity, reason_codes: BTreeSet<String>) -> EgressEvaluation {
    let disposition = match severity {
        Severity::None => "valid_structure",
        Severity::Incomplete => "incomplete",
        Severity::Unknown => "unknown",
        Severity::Review => "review",
        Severity::Deny => "deny",
    };
    EgressEvaluation {
        disposition,
        reason_codes,
    }
}

/// Validate structural consistency of one synthetic or host-resolved manifest.
///
/// valid_structure is not permission to disclose. Payload digest values are
/// shape-checked but are not recomputed because this function is not supplied
/// the exact source/package bytes. All resolver and cryptographic facts remain
/// the responsibility of their owning Mycelix/custodian host.
pub fn evaluate_disclosure_manifest(input: &Value) -> EgressEvaluation {
    let mut severity = Severity::None;
    let mut reasons = BTreeSet::new();

    if nonblank(input, "schema") != Some("luminous.cter.disclosure-manifest.v0.1") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "manifest_schema_missing_or_unsupported",
        );
    }
    if nonblank(input, "canonicalization_profile") != Some("rfc8785-jcs-v1") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "canonicalization_profile_missing_or_unsupported",
        );
    }
    for field in ["manifest_id", "coverage_record_ref", "prepared_at"] {
        if nonblank(input, field).is_none() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                format!("manifest_{field}_missing"),
            );
        }
    }
    if !input
        .get("manifest_commitment")
        .is_some_and(sha256_digest)
    {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "manifest_commitment_malformed",
        );
    }

    let Some(binding) = input.get("authorization_binding") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "authorization_binding_missing",
        );
        return finalize(severity, reasons);
    };
    for field in [
        "request_ref",
        "authorization_receipt_ref",
        "principal_ref",
        "subject_ref",
        "operation",
        "resource_ref",
        "evidence_frontier_ref",
        "purpose_ref",
        "recipient_ref",
        "jurisdiction_profile_ref",
    ] {
        if nonblank(binding, field).is_none() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                format!("authorization_{field}_missing"),
            );
        }
    }
    if !binding
        .get("request_commitment")
        .is_some_and(sha256_digest)
    {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "authorization_request_commitment_malformed",
        );
    }
    if !nonblank(binding, "project_snapshot_sha256").is_some_and(lower_hex_64) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "authorization_project_snapshot_malformed",
        );
    }
    let operation = nonblank(binding, "operation");
    if !matches!(
        operation,
        Some("collect_measurement" | "publish_observation" | "export_evidence")
    ) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "operation_outside_cter_disclosure_boundary",
        );
    }
    let authorized_classes = match binding.get("data_classes").and_then(valid_data_classes) {
        Some(classes) => classes,
        None => {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "authorization_data_classes_missing_or_invalid",
            );
            BTreeSet::new()
        }
    };

    let Some(destination) = input.get("destination_binding") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "destination_binding_missing",
        );
        return finalize(severity, reasons);
    };
    for field in [
        "recipient_ref",
        "destination_ref",
        "destination_key_epoch_ref",
        "channel_ref",
        "transport_profile_ref",
    ] {
        if nonblank(destination, field).is_none() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                format!("destination_{field}_missing"),
            );
        }
    }
    if nonblank(destination, "recipient_ref") != nonblank(binding, "recipient_ref") {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "recipient_binding_mismatch",
        );
    }

    let Some(package) = input.get("package") else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "package_missing",
        );
        return finalize(severity, reasons);
    };
    if nonblank(package, "package_ref").is_none()
        || nonblank(package, "media_type").is_none()
    {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "package_identity_or_media_type_missing",
        );
    }
    if !package
        .get("package_payload_digest")
        .is_some_and(sha256_digest)
    {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "package_payload_digest_malformed",
        );
    }

    let Some(outputs) = package.get("outputs").and_then(Value::as_array) else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "package_outputs_missing",
        );
        return finalize(severity, reasons);
    };
    let Some(transformations) = package.get("transformations").and_then(Value::as_array) else {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "package_transformations_missing",
        );
        return finalize(severity, reasons);
    };
    if outputs.is_empty() || transformations.is_empty() {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Incomplete,
            "package_outputs_or_transformations_empty",
        );
        return finalize(severity, reasons);
    }

    let mut output_classes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut output_ids = BTreeSet::new();
    let mut source_digests: BTreeMap<String, String> = BTreeMap::new();
    let mut source_classes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut output_digests: BTreeMap<String, String> = BTreeMap::new();
    let mut output_transform_refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut output_source_refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for output in outputs {
        let Some(object_ref) = nonblank(output, "object_ref") else {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "output_object_ref_missing",
            );
            continue;
        };
        if !output_ids.insert(object_ref.to_owned()) {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Deny,
                "output_object_identity_duplicated",
            );
        }
        if !output
            .get("content_digest")
            .is_some_and(sha256_digest)
        {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "output_content_digest_malformed",
            );
        }
        let output_digest = output
            .get("content_digest")
            .and_then(|item| nonblank(item, "value"))
            .unwrap_or("")
            .to_owned();
        output_digests.insert(object_ref.to_owned(), output_digest);
        if output.get("byte_length").and_then(Value::as_u64).is_none()
            || nonblank(output, "media_type").is_none()
        {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "output_length_or_media_type_invalid",
            );
        }

        let classes = match output.get("data_classes").and_then(valid_data_classes) {
            Some(classes) => classes,
            None => {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "output_data_classes_missing_or_invalid",
                );
                BTreeSet::new()
            }
        };
        if !authorized_classes.is_empty()
            && !classes.is_subset(&authorized_classes)
        {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Deny,
                "output_data_class_outside_authorized_scope",
            );
        }
        output_classes.insert(object_ref.to_owned(), classes);

        let Some(sources) = output.get("source_refs").and_then(Value::as_array) else {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "output_source_lineage_missing",
            );
            continue;
        };
        if sources.is_empty() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "output_source_lineage_empty",
            );
        }
        let mut source_ids = BTreeSet::new();
        for source in sources {
            let Some(source_ref) = nonblank(source, "object_ref") else {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "source_object_ref_missing",
                );
                continue;
            };
            if !source
                .get("content_digest")
                .is_some_and(sha256_digest)
            {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "source_content_digest_malformed",
                );
            }
            let classes = match source.get("data_classes").and_then(valid_data_classes) {
                Some(classes) => classes,
                None => {
                    raise(
                        &mut severity,
                        &mut reasons,
                        Severity::Incomplete,
                        "source_data_classes_missing_or_invalid",
                    );
                    BTreeSet::new()
                }
            };
            if let Some(existing) = source_classes.get(source_ref) {
                if existing != &classes {
                    raise(
                        &mut severity,
                        &mut reasons,
                        Severity::Deny,
                        "source_classification_identity_conflict",
                    );
                }
            } else {
                source_classes.insert(source_ref.to_owned(), classes);
            }
            let digest = source
                .get("content_digest")
                .and_then(|item| nonblank(item, "value"))
                .unwrap_or("")
                .to_owned();
            if let Some(existing) = source_digests.get(source_ref) {
                if existing != &digest {
                    raise(
                        &mut severity,
                        &mut reasons,
                        Severity::Deny,
                        "source_object_identity_conflict",
                    );
                }
            } else {
                source_digests.insert(source_ref.to_owned(), digest);
            }
            if !source_ids.insert(source_ref.to_owned()) {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Deny,
                    "source_reference_duplicated",
                );
            }
        }
        output_source_refs.insert(object_ref.to_owned(), source_ids);

        let Some(transform_refs) = output
            .get("transformation_refs")
            .and_then(string_set)
        else {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "output_transformation_refs_missing_or_invalid",
            );
            continue;
        };
        output_transform_refs.insert(object_ref.to_owned(), transform_refs);
    }

    let mut transform_by_ref: BTreeMap<String, &Value> = BTreeMap::new();
    for transform in transformations {
        let Some(transform_ref) = nonblank(transform, "transform_ref") else {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "transform_ref_missing",
            );
            continue;
        };
        if transform_by_ref.insert(transform_ref.to_owned(), transform).is_some() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Deny,
                "transform_identity_duplicated",
            );
        }
        if nonblank(transform, "profile_ref").is_none() {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "transform_profile_ref_missing",
            );
        }
        if !transform
            .get("version_commitment")
            .is_some_and(sha256_digest)
        {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "transform_version_commitment_malformed",
            );
        }
        if !matches!(
            nonblank(transform, "transform_kind"),
            Some(
                "identity_copy"
                    | "redaction"
                    | "aggregation"
                    | "generalization"
                    | "translation"
                    | "format_conversion"
                    | "derived_summary"
                    | "other"
            )
        ) {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "transform_kind_missing_or_unknown",
            );
        }
        if nonblank(transform, "transform_kind") == Some("identity_copy") {
            let inputs = transform.get("input_object_refs").and_then(string_set);
            let outputs = transform.get("output_object_refs").and_then(string_set);
            match (inputs, outputs) {
                (Some(inputs), Some(outputs)) if inputs.len() == 1 && outputs.len() == 1 => {
                    let input_ref = inputs.iter().next().expect("one identity-copy input");
                    let output_ref = outputs.iter().next().expect("one identity-copy output");
                    let input_digest = source_digests
                        .get(input_ref)
                        .or_else(|| output_digests.get(input_ref));
                    let output_digest = output_digests.get(output_ref);
                    if let (Some(input_digest), Some(output_digest)) = (input_digest, output_digest) {
                        if input_digest != output_digest {
                            raise(
                                &mut severity,
                                &mut reasons,
                                Severity::Deny,
                                "identity_copy_digest_mismatch",
                            );
                        }
                    } else {
                        raise(
                            &mut severity,
                            &mut reasons,
                            Severity::Incomplete,
                            "identity_copy_digest_unresolved",
                        );
                    }
                }
                _ => raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "identity_copy_cardinality_invalid",
                ),
            }
        }
    }

    // An intermediate output may legitimately be a source for a later
    // transform. Its object identity must nevertheless bind the same bytes.
    for (source_ref, source_digest) in &source_digests {
        if let Some(output_digest) = output_digests.get(source_ref) {
            if source_digest != output_digest {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Deny,
                    "source_output_content_conflict",
                );
            }
        }
    }

    let mut known_object_ids: BTreeSet<String> =
        source_digests.keys().cloned().collect();
    known_object_ids.extend(output_ids.iter().cloned());
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut referenced_transforms = BTreeSet::new();
    let mut transform_inputs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for (output_ref, refs) in &output_transform_refs {
        for transform_ref in refs {
            referenced_transforms.insert(transform_ref.clone());
            let Some(transform) = transform_by_ref.get(transform_ref) else {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "transformation_lineage_incomplete",
                );
                continue;
            };
            let inputs = transform
                .get("input_object_refs")
                .and_then(string_set);
            let produced = transform
                .get("output_object_refs")
                .and_then(string_set);
            let (Some(inputs), Some(produced)) = (inputs, produced) else {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "transform_input_or_output_refs_invalid",
                );
                continue;
            };
            if !produced.contains(output_ref) {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Deny,
                    "transform_does_not_produce_referenced_output",
                );
            }
            if inputs.iter().any(|input_ref| !known_object_ids.contains(input_ref)) {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "transform_input_object_unresolved",
                );
            }
            for input_ref in &inputs {
                if output_ids.contains(input_ref) {
                    graph
                        .entry(input_ref.clone())
                        .or_default()
                        .insert(output_ref.clone());
                }
            }
            transform_inputs
                .entry(output_ref.clone())
                .or_default()
                .extend(inputs);
        }
    }

    for (transform_ref, transform) in &transform_by_ref {
        if !referenced_transforms.contains(transform_ref) {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "orphan_transformation_unreferenced",
            );
        }
        let Some(produced) = transform
            .get("output_object_refs")
            .and_then(string_set)
        else {
            continue;
        };
        for output_ref in produced {
            if !output_ids.contains(&output_ref) {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "transform_output_object_unresolved",
                );
            } else if !output_transform_refs
                .get(&output_ref)
                .is_some_and(|refs| refs.contains(transform_ref))
            {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "output_omits_transform_reference",
                );
            }
        }
    }

    for (output_ref, source_refs) in &output_source_refs {
        let inputs = transform_inputs.get(output_ref);
        if inputs.is_none_or(|values| values != source_refs) {
            raise(
                &mut severity,
                &mut reasons,
                Severity::Incomplete,
                "transformation_input_source_set_mismatch",
            );
        }
        for source_ref in source_refs {
            if !source_digests.contains_key(source_ref)
                || !inputs.is_some_and(|values| values.contains(source_ref))
            {
                raise(
                    &mut severity,
                    &mut reasons,
                    Severity::Incomplete,
                    "declared_source_not_connected_to_output_transform",
                );
            }
        }

        let mut source_class_union = BTreeSet::new();
        for source_ref in source_refs {
            if let Some(classes) = source_classes.get(source_ref) {
                source_class_union.extend(classes.iter().cloned());
            }
        }
        if let Some(classes) = output_classes.get(output_ref) {
            if !source_class_union.is_empty() && &source_class_union != classes {
                let has_change_reference = output_transform_refs
                    .get(output_ref)
                    .is_some_and(|refs| {
                        refs.iter().any(|transform_ref| {
                            transform_by_ref
                                .get(transform_ref)
                                .is_some_and(|transform| {
                                    nonblank(transform, "classification_change_authority_ref").is_some()
                                })
                        })
                    });
                if has_change_reference {
                    raise(
                        &mut severity,
                        &mut reasons,
                        Severity::Unknown,
                        "classification_change_authority_resolution_unavailable",
                    );
                } else {
                    raise(
                        &mut severity,
                        &mut reasons,
                        Severity::Review,
                        "classification_change_authority_reference_missing",
                    );
                }
            }
        }
    }

    if has_cycle(&graph) {
        raise(
            &mut severity,
            &mut reasons,
            Severity::Deny,
            "transformation_graph_cycle",
        );
    }

    // Class labels are not a universal total order. v0.1 does not infer that a
    // transform is safe or that source/output classifications are equivalent;
    // a release host must resolve the exact class-change authority separately.
    finalize(severity, reasons)
}
