//! Read-only adapter from CityMaker owner-envelope bindings to the
//! independently produced #500 source-payload verification report.
//!
//! This crate never opens payloads and never computes payload SHA-256. It
//! hashes only the exact raw verifier-report bytes supplied by the caller so
//! the report artifact itself can be pinned. Payload results are consumed
//! verbatim and cross-checked against the explicit owner-artifact descriptor
//! mapping. Scope, rights, provenance and engineering authority remain
//! separate axes and are never promoted by byte equality.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const CLAIM_CEILING: &str = "synthetic adapter semantics only; no source authenticity, rights authentication, scope approval, engineering correctness, physical acceptance, action authority or causal effect";

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn bool_value(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn expected_record_key(record_kind: &str, case_record_id: &str) -> Option<String> {
    let prefix = match record_kind {
        "evidence" => "evidence",
        "artifact" => "artifact",
        "engineering_receipt" => "engineering_receipt",
        "decision" => "decision",
        _ => return None,
    };
    if case_record_id.trim().is_empty() {
        return None;
    }
    Some(format!("{prefix}:{case_record_id}"))
}

fn verifier_disposition_state(disposition: &str) -> &'static str {
    match disposition {
        "verified_match" => "verified",
        "digest_mismatch" | "byte_length_mismatch" | "digest_and_byte_length_mismatch" => {
            "mismatch"
        }
        "declared_unavailable" | "object_not_found" | "resolver_rejected" => "unavailable",
        "resource_limit_reached" | "resolver_io_error" | "stream_read_error" => "indeterminate",
        "invalid_descriptor" => "unsupported",
        _ => "unsupported",
    }
}

fn identity_components(value: &Value) -> Vec<(String, String)> {
    let mut pairs = array(value, "native_identity_components")
        .iter()
        .filter_map(|component| {
            let name = text(component, "name");
            let component_value = text(component, "value");
            if name.is_empty() || component_value.is_empty() {
                None
            } else {
                Some((name.to_owned(), component_value.to_owned()))
            }
        })
        .collect::<Vec<_>>();
    pairs.sort();
    pairs
}

fn evaluate_artifact(
    artifact: &Value,
    scope_binding: &Value,
    report: &Value,
    report_binding: &Value,
    report_valid: bool,
) -> Value {
    let artifact_id = text(artifact, "artifact_id");
    let descriptor = artifact.get("descriptor").unwrap_or(&Value::Null);
    let descriptor_binding = artifact.get("descriptor_binding").unwrap_or(&Value::Null);
    let binding = array(scope_binding, "artifact_bindings")
        .iter()
        .find(|entry| text(entry, "artifact_id") == artifact_id);
    let binding_state = binding
        .map(|entry| text(entry, "binding_state"))
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let raw_owner_disposition = binding
        .map(|entry| text(entry, "raw_owner_disposition"))
        .filter(|value| !value.is_empty())
        .unwrap_or("Missing");
    let base_flag = binding.and_then(|entry| bool_value(entry, "payload_bytes_verified"));
    let mut reasons = BTreeSet::<String>::new();
    let mut scope_reasons = BTreeSet::<String>::new();

    if binding.is_none() {
        scope_reasons.insert("SCOPE_BINDING_ARTIFACT_MISSING".to_owned());
    }
    let owner_scope_digest_matches =
        text(artifact, "scope_digest") == text(scope_binding, "scope_digest");
    if !owner_scope_digest_matches {
        scope_reasons.insert("OWNER_SCOPE_DIGEST_MISMATCH".to_owned());
    }
    if base_flag == Some(true) {
        reasons.insert("SCOPE_BINDER_PREPROMOTED_PAYLOAD_BYTES".to_owned());
    }

    if bool_value(artifact, "payload_required") != Some(true) {
        return json!({
            "artifact_id": artifact_id,
            "scope_binding_state": binding_state,
            "raw_owner_disposition": raw_owner_disposition,
            "base_payload_bytes_verified": base_flag.unwrap_or(false),
            "payload_verification_state": "not_required",
            "payload_bytes_verified": false,
            "raw_byte_equality_verified": false,
            "byte_length_comparison_state": "not_required",
            "verifier_disposition": null,
            "record_key": null,
            "report_reference": text(report_binding, "report_reference"),
            "report_sha256": text(report_binding, "declared_report_sha256"),
            "reason_codes": [],
            "scope_reason_codes": scope_reasons.into_iter().collect::<Vec<_>>(),
            "owner_rights_state": "not_applicable",
            "rights_use_declaration_state": "not_applicable",
            "rights_authentication": "not_performed",
            "action_authority": "none"
        });
    }

    if binding.is_none() || base_flag == Some(true) || !report_valid {
        if !report_valid {
            reasons.insert("REPORT_INTEGRITY_OR_STRUCTURE_UNVERIFIED".to_owned());
        }
        return json!({
            "artifact_id": artifact_id,
            "scope_binding_state": binding_state,
            "owner_scope_digest_matches": owner_scope_digest_matches,
            "raw_owner_disposition": raw_owner_disposition,
            "base_payload_bytes_verified": base_flag.unwrap_or(false),
            "payload_verification_state": "unsupported",
            "payload_bytes_verified": false,
            "raw_byte_equality_verified": false,
            "byte_length_comparison_state": "not_assessed",
            "verifier_disposition": null,
            "record_key": text(descriptor_binding, "record_key"),
            "report_reference": text(report_binding, "report_reference"),
            "report_sha256": text(report_binding, "declared_report_sha256"),
            "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
            "scope_reason_codes": scope_reasons.into_iter().collect::<Vec<_>>(),
            "owner_rights_state": text(descriptor.get("rights").unwrap_or(&Value::Null), "state"),
            "rights_use_declaration_state": "unknown_or_not_authenticated",
            "rights_authentication": "not_performed",
            "action_authority": "none"
        });
    }

    let record_kind = text(descriptor, "record_kind");
    let case_record_id = text(descriptor, "case_record_id");
    let declared_record_key = text(descriptor_binding, "record_key");
    let computed_record_key = expected_record_key(record_kind, case_record_id);
    if computed_record_key.as_deref() != Some(declared_record_key) {
        reasons.insert("DESCRIPTOR_RECORD_KEY_MAPPING_MISMATCH".to_owned());
    }

    let expected_case_id = text(descriptor_binding, "case_id");
    let expected_case_digest = text(descriptor_binding, "case_representation_sha256");
    let expected_catalog_id = text(descriptor_binding, "catalog_id");
    let expected_catalog_digest = text(descriptor_binding, "catalog_representation_sha256");
    if expected_case_id != text(report, "case_id")
        || expected_case_digest != text(report, "case_representation_sha256")
        || expected_catalog_id != text(report, "catalog_id")
        || expected_catalog_digest != text(report, "catalog_representation_sha256")
    {
        reasons.insert("DESCRIPTOR_CATALOG_IDENTITY_MISMATCH".to_owned());
    }

    let payload = descriptor.get("source_payload").unwrap_or(&Value::Null);
    let descriptor_owner_namespace = text(descriptor, "owner_namespace");
    if descriptor_owner_namespace.is_empty()
        || descriptor_owner_namespace != text(descriptor_binding, "owner_namespace")
    {
        reasons.insert("DESCRIPTOR_OWNER_NAMESPACE_MISMATCH".to_owned());
    }
    if text(descriptor_binding, "owner_receipt_id") != text(artifact, "receipt_id") {
        reasons.insert("OWNER_RECEIPT_BINDING_MISMATCH".to_owned());
    }
    if text(descriptor_binding, "source_revision") != text(artifact, "source_revision") {
        reasons.insert("OWNER_SOURCE_REVISION_BINDING_MISMATCH".to_owned());
    }
    if text(descriptor_binding, "subject_id") != text(artifact, "subject_id") {
        reasons.insert("OWNER_SUBJECT_BINDING_MISMATCH".to_owned());
    }
    if text(descriptor_binding, "schema_artifact_digest")
        != text(artifact, "schema_artifact_digest")
    {
        reasons.insert("OWNER_SCHEMA_ARTIFACT_BINDING_MISMATCH".to_owned());
    }
    if text(descriptor_binding, "schema_id") != text(payload, "schema_id")
        || text(descriptor_binding, "schema_version") != text(payload, "schema_version")
    {
        reasons.insert("SOURCE_PAYLOAD_SCHEMA_IDENTITY_MISMATCH".to_owned());
    }
    if text(descriptor_binding, "owner_contract_id") != text(artifact, "contract_id")
        || text(descriptor_binding, "owner_contract_version") != text(artifact, "contract_version")
    {
        reasons.insert("OWNER_CONTRACT_IDENTITY_MISMATCH".to_owned());
    }
    let schema_digest_algorithm = text(artifact, "schema_digest_algorithm");
    let schema_artifact_digest = text(artifact, "schema_artifact_digest");
    if !matches!(schema_digest_algorithm, "git_blob_sha1" | "sha256")
        || (schema_digest_algorithm == "git_blob_sha1" && !lower_hex(schema_artifact_digest, 40))
        || (schema_digest_algorithm == "sha256" && !lower_hex(schema_artifact_digest, 64))
    {
        reasons.insert("OWNER_SCHEMA_PIN_INCOMPLETE_OR_INVALID".to_owned());
    }
    if schema_digest_algorithm == "sha256"
        && !text(payload, "schema_sha256").is_empty()
        && text(payload, "schema_sha256") != schema_artifact_digest
    {
        reasons.insert("OWNER_SCHEMA_DIGEST_MISMATCH".to_owned());
    }
    if text(artifact, "owner_repo").is_empty()
        || text(artifact, "contract_id").is_empty()
        || text(artifact, "contract_version").is_empty()
        || text(artifact, "receipt_id").is_empty()
        || text(artifact, "subject_id").is_empty()
        || !lower_hex(text(artifact, "subject_digest"), 64)
        || !lower_hex(text(artifact, "source_revision"), 40)
    {
        reasons.insert("OWNER_ARTIFACT_ENVELOPE_INCOMPLETE".to_owned());
    }
    if !lower_hex(text(artifact, "scope_digest"), 64) {
        reasons.insert("OWNER_SCOPE_DIGEST_INVALID".to_owned());
    }

    let payload_digest = text(payload, "sha256");
    let expected_sha = if payload.get("sha256").is_some_and(Value::is_string) {
        Some(payload_digest)
    } else {
        None
    };
    let expected_length = payload.get("byte_length").and_then(Value::as_u64);
    let digest_basis = text(payload, "digest_basis");
    let object_ref = text(payload, "payload_object_ref");
    if !matches!(
        digest_basis,
        "locally_retrieved_and_hashed"
            | "source_owner_attested"
            | "external_reference"
            | "unavailable"
    ) {
        reasons.insert("PAYLOAD_DIGEST_BASIS_UNSUPPORTED".to_owned());
    }
    if expected_sha.is_some_and(|digest| !lower_hex(digest, 64)) {
        reasons.insert("OWNER_PAYLOAD_DIGEST_INVALID".to_owned());
    }
    if digest_basis == "unavailable"
        && (expected_sha.is_some()
            || expected_length.is_some()
            || !object_ref.is_empty()
            || bool_value(payload, "bytes_retrieved") == Some(true))
    {
        reasons.insert("UNAVAILABLE_DESCRIPTOR_PAYLOAD_INCONSISTENT".to_owned());
    }
    if digest_basis == "source_owner_attested"
        && (bool_value(payload, "bytes_retrieved") == Some(true)
            || text(
                descriptor.get("attestation").unwrap_or(&Value::Null),
                "statement_ref",
            )
            .is_empty())
    {
        reasons.insert("SOURCE_OWNER_ATTESTATION_DESCRIPTOR_INCOMPLETE".to_owned());
    }
    if digest_basis == "external_reference"
        && (bool_value(payload, "bytes_retrieved") == Some(true)
            || expected_sha.is_none()
            || object_ref.is_empty())
    {
        reasons.insert("EXTERNAL_REFERENCE_DESCRIPTOR_INCOMPLETE".to_owned());
    }
    if digest_basis != "unavailable" && expected_sha.is_none() {
        reasons.insert("OWNER_PAYLOAD_DIGEST_MISSING".to_owned());
    }
    if digest_basis == "locally_retrieved_and_hashed"
        && (bool_value(payload, "bytes_retrieved") != Some(true)
            || expected_length.is_none()
            || object_ref.is_empty())
    {
        reasons.insert("LOCAL_PAYLOAD_DESCRIPTOR_INCOMPLETE".to_owned());
    }
    if matches!(
        digest_basis,
        "external_reference" | "locally_retrieved_and_hashed"
    ) && expected_sha.is_some()
        && object_ref != format!("cas:sha256:{}", expected_sha.unwrap_or(""))
    {
        reasons.insert("PAYLOAD_OBJECT_REFERENCE_DIGEST_MISMATCH".to_owned());
    }
    let owner_content_digest = text(artifact, "content_digest");
    if let Some(digest) = expected_sha
        && !owner_content_digest.is_empty()
        && owner_content_digest != digest
    {
        reasons.insert("OWNER_PAYLOAD_DIGEST_MISMATCH".to_owned());
    }

    let rights = descriptor.get("rights").unwrap_or(&Value::Null);
    let rights_state = text(rights, "state");
    let required_use = text(scope_binding, "required_use");
    let permitted_uses = array(rights, "permitted_uses");
    if rights_state == "verified"
        && (permitted_uses.is_empty() || text(rights, "rights_evidence_ref").is_empty())
    {
        reasons.insert("VERIFIED_RIGHTS_DESCRIPTOR_INCOMPLETE".to_owned());
    }
    if rights_state == "unknown" && !permitted_uses.is_empty() {
        reasons.insert("UNKNOWN_RIGHTS_HAS_PERMITTED_USES".to_owned());
    }
    if !matches!(
        rights_state,
        "verified" | "restricted" | "unknown" | "not_applicable"
    ) {
        reasons.insert("RIGHTS_STATE_UNSUPPORTED".to_owned());
    }
    if payload.get("schema_id").is_some_and(Value::is_string)
        && !lower_hex(text(payload, "schema_sha256"), 64)
    {
        reasons.insert("PAYLOAD_SCHEMA_SHA256_INVALID".to_owned());
    }
    let rights_use_declaration_state = match rights_state {
        "verified"
            if !required_use.is_empty()
                && permitted_uses
                    .iter()
                    .any(|value| value.as_str() == Some(required_use)) =>
        {
            "owner_declares_permitted"
        }
        "restricted" => "owner_declares_restricted",
        "not_applicable" => "not_applicable",
        _ => "unknown_or_not_authenticated",
    };

    let catalog_validation = report.get("catalog_validation").unwrap_or(&Value::Null);
    let record_key = declared_record_key;
    let identity_digest = text(
        catalog_validation
            .get("identity_sha256_by_case_record")
            .unwrap_or(&Value::Null),
        record_key,
    );
    let mapped_namespace = text(
        catalog_validation
            .get("native_identity_namespaces")
            .unwrap_or(&Value::Null),
        record_key,
    );
    let mapped_components = catalog_validation
        .get("native_identity_components")
        .and_then(|value| value.get(record_key))
        .unwrap_or(&Value::Null);
    let descriptor_components = identity_components(descriptor);
    let mapped_component_pairs = identity_components(&json!({
        "native_identity_components": mapped_components
    }));
    if !lower_hex(identity_digest, 64)
        || mapped_namespace != descriptor_owner_namespace
        || mapped_component_pairs != descriptor_components
    {
        reasons.insert("CATALOG_NATIVE_IDENTITY_BINDING_MISMATCH".to_owned());
    }

    let report_results = array(report, "payload_results");
    let matched_results = report_results
        .iter()
        .filter(|result| text(result, "record_key") == declared_record_key)
        .collect::<Vec<_>>();
    if matched_results.len() != 1 {
        reasons.insert(if matched_results.is_empty() {
            "VERIFIER_RECORD_RESULT_MISSING".to_owned()
        } else {
            "VERIFIER_RECORD_RESULT_DUPLICATED".to_owned()
        });
    }
    let result = matched_results.first().copied();
    let verifier_disposition = result
        .map(|record| text(record, "disposition"))
        .unwrap_or("not_reported");
    let mut state = "indeterminate";
    let mut byte_length_comparison_state = "not_assessed";
    let result_descriptor_matches = result.is_some_and(|record| {
        text(record, "record_key") == declared_record_key
            && record.get("expected_sha256").and_then(Value::as_str) == expected_sha
            && record.get("object_ref").and_then(Value::as_str)
                == if object_ref.is_empty() {
                    None
                } else {
                    Some(object_ref)
                }
            && record.get("expected_byte_length").and_then(Value::as_u64) == expected_length
            && text(record, "digest_basis") == digest_basis
    });
    if result_descriptor_matches {
        if let Some(record) = result {
            let observed_length = record.get("observed_byte_length").and_then(Value::as_u64);
            let bytes_hashed = record.get("bytes_hashed").and_then(Value::as_u64);
            byte_length_comparison_state =
                match (expected_length, bool_value(record, "byte_length_matches")) {
                    (Some(expected), Some(true))
                        if observed_length == Some(expected) && bytes_hashed == Some(expected) =>
                    {
                        "matched"
                    }
                    (Some(_), Some(false)) | (Some(_), Some(true)) => "mismatch",
                    (Some(_), None) => "not_verified",
                    (None, _) => "not_declared",
                };
        }
    }
    if !reasons.is_empty() {
        state = if reasons.iter().any(|reason| {
            reason.contains("MISMATCH")
                || reason.contains("DUPLICATED")
                || reason.contains("CONFLICT")
        }) {
            "mismatch"
        } else if reasons.contains("VERIFIER_RECORD_RESULT_MISSING")
            && text(report, "disposition") == "invalid_input"
        {
            reasons.insert("VERIFIER_REPORT_INPUT_INVALID".to_owned());
            "unsupported"
        } else if reasons.contains("VERIFIER_RECORD_RESULT_MISSING") {
            "indeterminate"
        } else {
            "unsupported"
        };
    } else if let Some(record) = result {
        let result_object_ref = record.get("object_ref").and_then(Value::as_str);
        let result_expected_length = record.get("expected_byte_length").and_then(Value::as_u64);
        if record.get("expected_sha256").and_then(Value::as_str) != expected_sha
            || result_object_ref
                != if object_ref.is_empty() {
                    None
                } else {
                    Some(object_ref)
                }
            || result_expected_length != expected_length
            || text(record, "digest_basis") != digest_basis
        {
            reasons.insert("VERIFIER_RESULT_DESCRIPTOR_MISMATCH".to_owned());
            state = "mismatch";
        } else {
            state = verifier_disposition_state(verifier_disposition);
            if verifier_disposition == "verified_match" {
                let observed_digest = record.get("observed_sha256").and_then(Value::as_str);
                let observed_length = record.get("observed_byte_length").and_then(Value::as_u64);
                let bytes_hashed = record.get("bytes_hashed").and_then(Value::as_u64);
                let resolver_opened = bool_value(record, "resolver_open_attempted") == Some(true);
                let digest_match = bool_value(record, "digest_matches") == Some(true)
                    && expected_sha.is_some()
                    && observed_digest == expected_sha;
                let length_ok = match expected_length {
                    Some(length) => {
                        byte_length_comparison_state = "matched";
                        bool_value(record, "byte_length_matches") == Some(true)
                            && observed_length == Some(length)
                            && bytes_hashed == Some(length)
                    }
                    None => {
                        byte_length_comparison_state = "not_declared";
                        record
                            .get("byte_length_matches")
                            .is_some_and(Value::is_null)
                            && observed_length.is_some()
                            && bytes_hashed == observed_length
                    }
                };
                if !resolver_opened || !digest_match || !length_ok {
                    reasons.insert("VERIFIED_RESULT_PREDICATE_INCOMPLETE".to_owned());
                    state = "mismatch";
                }
            } else if verifier_disposition == "declared_unavailable" {
                state = "unavailable";
                byte_length_comparison_state = "not_available";
            } else {
                byte_length_comparison_state =
                    match (expected_length, bool_value(record, "byte_length_matches")) {
                        (Some(_), Some(true)) => "matched",
                        (Some(_), Some(false)) => "mismatch",
                        (Some(_), None) => "not_verified",
                        (None, _) => "not_declared",
                    };
            }
        }
    }

    let observed_digest = result
        .and_then(|record| record.get("observed_sha256").and_then(Value::as_str))
        .unwrap_or("");
    let observed_length =
        result.and_then(|record| record.get("observed_byte_length").and_then(Value::as_u64));
    let raw_byte_equality_verified = verifier_disposition == "verified_match"
        && result.is_some_and(|record| {
            expected_sha.is_some()
                && text(record, "record_key") == declared_record_key
                && text(record, "digest_basis") == digest_basis
                && record.get("object_ref").and_then(Value::as_str)
                    == if object_ref.is_empty() {
                        None
                    } else {
                        Some(object_ref)
                    }
                && record.get("expected_sha256").and_then(Value::as_str) == expected_sha
                && record
                    .get("observed_sha256")
                    .and_then(Value::as_str)
                    .is_some_and(|observed| Some(observed) == expected_sha)
                && bool_value(record, "resolver_open_attempted") == Some(true)
                && bool_value(record, "digest_matches") == Some(true)
                && record.get("bytes_hashed").and_then(Value::as_u64)
                    == record.get("observed_byte_length").and_then(Value::as_u64)
        });
    json!({
        "artifact_id": artifact_id,
        "scope_binding_state": binding_state,
        "owner_scope_digest_matches": owner_scope_digest_matches,
        "raw_owner_disposition": raw_owner_disposition,
        "base_payload_bytes_verified": base_flag.unwrap_or(false),
        "payload_verification_state": state,
        "raw_byte_equality_verified": raw_byte_equality_verified,
        "payload_bytes_verified": state == "verified",
        "byte_length_comparison_state": byte_length_comparison_state,
        "verifier_disposition": verifier_disposition,
        "record_key": declared_record_key,
        "object_ref": if object_ref.is_empty() { Value::Null } else { json!(object_ref) },
        "digest_basis": digest_basis,
        "digest_algorithm": "sha256",
        "expected_sha256": expected_sha,
        "observed_sha256": observed_digest,
        "expected_byte_length": expected_length,
        "observed_byte_length": observed_length,
        "report_reference": text(report_binding, "report_reference"),
        "report_sha256": text(report_binding, "declared_report_sha256"),
        "owner_rights_state": rights_state,
        "rights_use_declaration_state": rights_use_declaration_state,
        "schema_digest_algorithm": text(artifact, "schema_digest_algorithm"),
        "schema_artifact_digest": text(artifact, "schema_artifact_digest"),
        "catalog_identity_sha256": identity_digest,
        "rights_authentication": "not_performed",
        "action_authority": "none",
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "scope_reason_codes": scope_reasons.into_iter().collect::<Vec<_>>()
    })
}

fn aggregate_state(artifacts: &[Value]) -> &'static str {
    let states = artifacts
        .iter()
        .map(|artifact| text(artifact, "payload_verification_state"))
        .collect::<BTreeSet<_>>();
    for state in ["mismatch", "unsupported", "indeterminate", "unavailable"] {
        if states.contains(state) {
            return state;
        }
    }
    if states
        .iter()
        .all(|state| *state == "verified" || *state == "not_required")
    {
        "verified"
    } else {
        "unsupported"
    }
}

/// Evaluate one CityMaker payload-adapter fixture using input only. Expected
/// known-answer values are never consulted.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let scope = input.get("scope_binding").unwrap_or(&Value::Null);
    let report_binding = input.get("report_binding").unwrap_or(&Value::Null);
    let mut global_reasons = BTreeSet::<String>::new();
    let mut report_reasons = BTreeSet::<String>::new();

    if !lower_hex(text(scope, "scope_digest"), 64) || text(scope, "binding_disposition").is_empty()
    {
        global_reasons.insert("SCOPE_BINDING_CONTEXT_INCOMPLETE".to_owned());
    }
    if array(input, "owner_artifacts").is_empty() {
        global_reasons.insert("OWNER_ARTIFACTS_MISSING".to_owned());
    }

    let raw_report = text(report_binding, "raw_report_json");
    let declared_report_sha = text(report_binding, "declared_report_sha256");
    let report_reference = text(report_binding, "report_reference");
    if raw_report.is_empty() || report_reference.is_empty() || !lower_hex(declared_report_sha, 64) {
        report_reasons.insert("VERIFIER_REPORT_REFERENCE_OR_DIGEST_MISSING".to_owned());
    }
    let computed_report_sha = sha256_hex(raw_report.as_bytes());
    if !declared_report_sha.is_empty() && declared_report_sha != computed_report_sha {
        report_reasons.insert("VERIFIER_REPORT_RAW_BYTE_DIGEST_MISMATCH".to_owned());
    }
    if report_reference.starts_with("cas:sha256:")
        && report_reference != format!("cas:sha256:{computed_report_sha}")
    {
        report_reasons.insert("VERIFIER_REPORT_OBJECT_REFERENCE_MISMATCH".to_owned());
    }

    let report: Value = match serde_json::from_str(raw_report) {
        Ok(value) => value,
        Err(_) => {
            report_reasons.insert("VERIFIER_REPORT_JSON_INVALID".to_owned());
            Value::Null
        }
    };
    if !report.is_null() {
        let catalog_validation = report.get("catalog_validation").unwrap_or(&Value::Null);
        if catalog_validation.is_null()
            || text(catalog_validation, "case_id") != text(&report, "case_id")
            || text(catalog_validation, "catalog_id") != text(&report, "catalog_id")
            || text(catalog_validation, "catalog_representation_sha256")
                != text(&report, "catalog_representation_sha256")
            || catalog_validation
                .get("descriptor_count")
                .and_then(Value::as_u64)
                != report.get("descriptor_count").and_then(Value::as_u64)
        {
            report_reasons.insert("VERIFIER_CATALOG_VALIDATION_BINDING_MISMATCH".to_owned());
        }
        if !matches!(
            text(catalog_validation, "disposition"),
            "internally_consistent_for_declared_case"
                | "incomplete_or_indeterminate"
                | "invalid_input"
        ) {
            report_reasons.insert("VERIFIER_CATALOG_VALIDATION_DISPOSITION_UNSUPPORTED".to_owned());
        }
        if text(catalog_validation, "action_authority") != "none"
            || text(catalog_validation, "rights_authentication") != "not_performed"
            || text(catalog_validation, "source_attestation_authentication") != "not_performed"
        {
            report_reasons.insert("VERIFIER_CATALOG_VALIDATION_CLAIM_CEILING_MISMATCH".to_owned());
        }
        if text(&report, "case_id") != text(report_binding, "expected_case_id")
            || text(&report, "case_representation_sha256")
                != text(report_binding, "expected_case_representation_sha256")
            || text(&report, "catalog_id") != text(report_binding, "expected_catalog_id")
            || text(&report, "catalog_representation_sha256")
                != text(report_binding, "expected_catalog_representation_sha256")
        {
            report_reasons.insert("VERIFIER_REPORT_CATALOG_BINDING_MISMATCH".to_owned());
        }
        if text(&report, "action_authority") != "none"
            || text(&report, "rights_authentication") != "not_performed"
            || text(&report, "source_attestation_authentication") != "not_performed"
        {
            report_reasons.insert("VERIFIER_REPORT_CLAIM_CEILING_MISMATCH".to_owned());
        }
        if text(&report, "verifier_build_sha256").is_empty()
            || !lower_hex(text(&report, "verifier_build_sha256"), 64)
        {
            report_reasons.insert("VERIFIER_BUILD_IDENTITY_INVALID".to_owned());
        }
        if !matches!(
            text(&report, "disposition"),
            "all_available_payloads_verified" | "incomplete_or_indeterminate" | "invalid_input"
        ) {
            report_reasons.insert("VERIFIER_REPORT_DISPOSITION_UNSUPPORTED".to_owned());
        }
        let descriptor_count = report.get("descriptor_count").and_then(Value::as_u64);
        let payload_count = report.get("payload_count").and_then(Value::as_u64);
        let not_attempted = report.get("not_attempted_count").and_then(Value::as_u64);
        let results = array(&report, "payload_results");
        if descriptor_count.is_none()
            || payload_count.is_none()
            || not_attempted.is_none()
            || descriptor_count != payload_count.zip(not_attempted).map(|(a, b)| a + b)
            || payload_count != Some(results.len() as u64)
        {
            report_reasons.insert("VERIFIER_REPORT_DENOMINATOR_INCONSISTENT".to_owned());
        }
        let mut keys = BTreeSet::new();
        for result in results {
            let key = text(result, "record_key");
            if key.is_empty() || !keys.insert(key.to_owned()) {
                report_reasons.insert("VERIFIER_REPORT_RECORD_KEY_DUPLICATED".to_owned());
            }
        }
        let disposition_count = |wanted: &[&str]| {
            results
                .iter()
                .filter(|row| wanted.contains(&text(row, "disposition")))
                .count() as u64
        };
        for (field, expected_count) in [
            (
                "verified_payload_count",
                disposition_count(&["verified_match"]),
            ),
            (
                "unavailable_payload_count",
                disposition_count(&["declared_unavailable", "object_not_found"]),
            ),
            (
                "resource_limited_payload_count",
                disposition_count(&["resource_limit_reached"]),
            ),
            (
                "resolver_error_count",
                disposition_count(&["resolver_rejected", "resolver_io_error"]),
            ),
            (
                "stream_error_count",
                disposition_count(&["stream_read_error"]),
            ),
            (
                "invalid_payload_count",
                disposition_count(&[
                    "digest_mismatch",
                    "byte_length_mismatch",
                    "digest_and_byte_length_mismatch",
                    "invalid_descriptor",
                ]),
            ),
        ] {
            if report.get(field).and_then(Value::as_u64) != Some(expected_count) {
                report_reasons.insert("VERIFIER_REPORT_RESULT_COUNTS_INCONSISTENT".to_owned());
            }
        }
        let hashed_total = results
            .iter()
            .filter_map(|row| row.get("bytes_hashed").and_then(Value::as_u64))
            .fold(0_u64, u64::saturating_add);
        if report.get("total_bytes_hashed").and_then(Value::as_u64) != Some(hashed_total) {
            report_reasons.insert("VERIFIER_REPORT_BYTE_DENOMINATOR_INCONSISTENT".to_owned());
        }
    }

    let report_state = if report_reasons.contains("VERIFIER_REPORT_RAW_BYTE_DIGEST_MISMATCH")
        || report_reasons.contains("VERIFIER_REPORT_OBJECT_REFERENCE_MISMATCH")
        || report_reasons.contains("VERIFIER_REPORT_CATALOG_BINDING_MISMATCH")
    {
        "mismatch"
    } else if report_reasons.is_empty() {
        "verified"
    } else {
        "unsupported"
    };
    let report_valid = report_state == "verified";

    let mut artifact_outputs = array(input, "owner_artifacts")
        .iter()
        .map(|artifact| evaluate_artifact(artifact, scope, &report, report_binding, report_valid))
        .collect::<Vec<_>>();
    artifact_outputs
        .sort_by(|left, right| text(left, "artifact_id").cmp(text(right, "artifact_id")));

    // Reusing one exact object reference under conflicting digests is never
    // last-write-wins. Mark every affected mapping rather than choosing one.
    let mut object_digests = BTreeMap::<String, BTreeSet<String>>::new();
    for output in &artifact_outputs {
        let object_ref = text(output, "object_ref");
        let digest = text(output, "expected_sha256");
        if !object_ref.is_empty() && !digest.is_empty() {
            object_digests
                .entry(object_ref.to_owned())
                .or_default()
                .insert(digest.to_owned());
        }
    }
    for output in &mut artifact_outputs {
        let object_ref = text(output, "object_ref").to_owned();
        if object_digests
            .get(&object_ref)
            .is_some_and(|digests| digests.len() > 1)
        {
            output["payload_verification_state"] = json!("mismatch");
            output["payload_bytes_verified"] = json!(false);
            output["raw_byte_equality_verified"] = json!(false);
            let mut reasons = output["reason_codes"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            reasons.push(json!("DUPLICATE_OBJECT_REF_CONFLICTING_DIGESTS"));
            reasons.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
            reasons.dedup();
            output["reason_codes"] = json!(reasons);
        }
    }

    let payload_state = aggregate_state(&artifact_outputs);
    json!({
        "case_id": case_id,
        "report_integrity_state": report_state,
        "report_disposition_state": match text(&report, "disposition") {
            "all_available_payloads_verified" => "complete",
            "incomplete_or_indeterminate" => "incomplete",
            "invalid_input" => "invalid_input",
            _ => "unsupported"
        },
        "report_reference": report_reference,
        "report_sha256": computed_report_sha,
        "report_disposition": text(&report, "disposition"),
        "catalog_validation_disposition": text(report.get("catalog_validation").unwrap_or(&Value::Null), "disposition"),
        "catalog_missing_descriptor_count": report.get("catalog_validation").and_then(|value| value.get("missing_descriptor_count")).and_then(Value::as_u64),
        "catalog_extra_descriptor_count": report.get("catalog_validation").and_then(|value| value.get("extra_descriptor_count")).and_then(Value::as_u64),
        "descriptor_catalog_id": text(&report, "catalog_id"),
        "descriptor_catalog_representation_sha256": text(&report, "catalog_representation_sha256"),
        "verifier_build_sha256": text(&report, "verifier_build_sha256"),
        "scope_binding_disposition": text(scope, "binding_disposition"),
        "scope_digest": text(scope, "scope_digest"),
        "payload_verification_state": payload_state,
        "artifact_payloads": artifact_outputs,
        "reason_codes": global_reasons
            .into_iter()
            .chain(report_reasons)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>(),
        "action_authority": "none",
        "owner_result_promotion": "none",
        "causal_effect_established": false,
        "claim_ceiling": CLAIM_CEILING
    })
}

fn equivalent(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .zip(right.as_f64())
            .is_some_and(|(left, right)| left == right),
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len() && left.iter().zip(right).all(|(a, b)| equivalent(a, b))
        }
        (Value::Object(left), Value::Object(right)) => right.iter().all(|(key, expected_item)| {
            left.get(key)
                .is_some_and(|actual_item| equivalent(actual_item, expected_item))
        }),
        _ => actual == expected,
    }
}

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    equivalent(actual, expected)
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case, matches_expected, text};
    use serde_json::Value;
    use std::path::PathBuf;

    fn corpus() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../docs/research/city-owner-payload-adapter/CITY_OWNER_PAYLOAD_ADAPTER_CORPUS_V0_1.json",
        );
        let bytes = std::fs::read(path).expect("payload adapter corpus must exist");
        serde_json::from_slice(&bytes).expect("payload adapter corpus must parse")
    }

    #[test]
    fn corpus_contains_24_unique_cases() {
        let c = corpus();
        assert_eq!(c["corpus_id"], "CITY-OWNER-PAYLOAD-ADAPTER-CORPUS-V0.1");
        let cases = c["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 24);
        let ids = cases
            .iter()
            .map(|case| case["case_id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), cases.len());
    }

    #[test]
    fn adapter_matches_all_independent_known_answers() {
        for case in corpus()["cases"].as_array().unwrap() {
            let actual = evaluate_case(case);
            assert!(
                matches_expected(&actual, &case["expected"]),
                "{} mismatch\nactual: {}\nexpected: {}",
                text(case, "case_id"),
                serde_json::to_string_pretty(&actual).unwrap(),
                serde_json::to_string_pretty(&case["expected"]).unwrap()
            );
        }
    }

    #[test]
    fn raw_report_digest_mutation_is_rejected() {
        let mut case = corpus()["cases"][0].clone();
        let raw = case["input"]["report_binding"]["raw_report_json"]
            .as_str()
            .unwrap()
            .to_owned();
        case["input"]["report_binding"]["raw_report_json"] = Value::String(format!("{raw} "));
        let result = evaluate_case(&case);
        assert_eq!(result["report_integrity_state"], "mismatch");
        assert_eq!(result["payload_verification_state"], "unsupported");
    }

    #[test]
    fn descriptor_mapping_does_not_guess_record_keys() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["owner_artifacts"][0]["descriptor_binding"]["record_key"] =
            Value::String("artifact:some-other-record".into());
        let result = evaluate_case(&case);
        assert_eq!(
            result["artifact_payloads"][0]["payload_verification_state"],
            "mismatch"
        );
        assert_eq!(
            result["artifact_payloads"][0]["payload_bytes_verified"],
            false
        );
    }
    #[test]
    fn byte_equality_is_independent_of_declared_rights_state() {
        let result = evaluate_case(&corpus()["cases"][6]);
        let artifact = &result["artifact_payloads"][0];
        assert_eq!(artifact["payload_verification_state"], "verified");
        assert_eq!(artifact["payload_bytes_verified"], true);
        assert_eq!(artifact["raw_byte_equality_verified"], true);
        assert_eq!(artifact["owner_rights_state"], "restricted");
        assert_eq!(
            artifact["rights_use_declaration_state"],
            "owner_declares_restricted"
        );
        assert_eq!(artifact["rights_authentication"], "not_performed");
    }

    #[test]
    fn scope_and_owner_disposition_do_not_change_raw_byte_equality() {
        for index in [13, 14, 15, 16, 17, 23] {
            let result = evaluate_case(&corpus()["cases"][index]);
            let artifact = &result["artifact_payloads"][0];
            assert_eq!(artifact["raw_byte_equality_verified"], true);
            assert_eq!(artifact["payload_verification_state"], "verified");
        }
    }

    #[test]
    fn invalid_verifier_input_is_not_misreported_as_raw_report_corruption() {
        for index in [18, 19] {
            let result = evaluate_case(&corpus()["cases"][index]);
            assert_eq!(result["report_integrity_state"], "verified");
            assert_eq!(result["report_disposition_state"], "invalid_input");
            assert_eq!(result["payload_verification_state"], "unsupported");
        }
    }

    #[test]
    fn owner_envelope_digest_mismatch_does_not_erase_descriptor_byte_result() {
        let result = evaluate_case(&corpus()["cases"][20]);
        let artifact = &result["artifact_payloads"][0];
        assert_eq!(artifact["payload_verification_state"], "mismatch");
        assert_eq!(artifact["payload_bytes_verified"], false);
        assert_eq!(artifact["raw_byte_equality_verified"], true);
        assert_eq!(artifact["byte_length_comparison_state"], "matched");
        assert!(
            artifact["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "OWNER_PAYLOAD_DIGEST_MISMATCH")
        );
    }

    #[test]
    fn object_reference_mismatch_cannot_claim_raw_byte_equality_for_the_owner_mapping() {
        let result = evaluate_case(&corpus()["cases"][4]);
        let artifact = &result["artifact_payloads"][0];
        assert_eq!(artifact["payload_verification_state"], "mismatch");
        assert_eq!(artifact["raw_byte_equality_verified"], false);
        assert!(
            artifact["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "VERIFIER_RESULT_DESCRIPTOR_MISMATCH")
        );
    }
    #[test]
    fn owner_contract_and_payload_schema_are_separately_pinned() {
        let mut schema_case = corpus()["cases"][0].clone();
        schema_case["input"]["owner_artifacts"][0]["descriptor_binding"]["schema_id"] =
            Value::String("living-city-metabolism".into());
        let schema_result = evaluate_case(&schema_case);
        let schema_artifact = &schema_result["artifact_payloads"][0];
        assert_eq!(schema_artifact["payload_verification_state"], "mismatch");
        assert_eq!(schema_artifact["raw_byte_equality_verified"], true);
        assert!(
            schema_artifact["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "SOURCE_PAYLOAD_SCHEMA_IDENTITY_MISMATCH")
        );

        let mut contract_case = corpus()["cases"][0].clone();
        contract_case["input"]["owner_artifacts"][0]["descriptor_binding"]["owner_contract_id"] =
            Value::String("different-owner-contract".into());
        let contract_result = evaluate_case(&contract_case);
        let contract_artifact = &contract_result["artifact_payloads"][0];
        assert_eq!(contract_artifact["payload_verification_state"], "mismatch");
        assert_eq!(contract_artifact["raw_byte_equality_verified"], true);
        assert!(
            contract_artifact["reason_codes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason == "OWNER_CONTRACT_IDENTITY_MISMATCH")
        );
    }
}
