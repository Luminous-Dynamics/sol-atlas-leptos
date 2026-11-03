// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Runtime cross-object verifier for declared-scope dependency coverage.
//!
//! This verifier deliberately does not authenticate source-owner or reviewer
//! attestations. A successful internal audit is not sufficient to make
//! NoImpact decision-eligible; trust-root and scope-adequacy checks remain
//! separate gates.

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::process::ExitCode;

const MANIFEST_SCHEMA: &str = "luminous.built-environment-dependency-coverage.v0.1";
const VERIFIER_ID: &str = "built-environment-coverage-auditor";
const VERIFIER_VERSION: &str = env!("CARGO_PKG_VERSION");

const NOT_CHECKED: &[&str] = &[
    "source-owner identity, signatures, and authority-chain authenticity",
    "authenticity of source completeness proofs and source snapshot bytes",
    "reviewer identity, credentials, independence, and review-evidence authenticity",
    "whether the declared dependency profile is adequate for the real project, discipline, jurisdiction, or decision",
    "whether the source inventory omits an undiscovered source system",
    "gold dependency-inventory agreement and impact-traversal correctness",
    "engineering solver correctness, physical safety, legal sufficiency, or action authority",
];

#[derive(Debug)]
struct StrictValue(Value);

struct StrictVisitor;

impl<'de> Visitor<'de> for StrictVisitor {
    type Value = StrictValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique object member names")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let number = Number::from_f64(value)
            .ok_or_else(|| E::custom("non-finite JSON number is not permitted"))?;
        Ok(StrictValue(Value::Number(number)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        StrictValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut entries: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, value)) = entries.next_entry::<String, StrictValue>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object member: {key}"
                )));
            }
            values.insert(key, value.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize)]
struct Finding {
    severity: String,
    code: String,
    path: String,
    detail: String,
}

#[derive(Default)]
struct Audit {
    findings: BTreeSet<Finding>,
}

impl Audit {
    fn invalid(&mut self, code: &str, path: impl Into<String>, detail: impl Into<String>) {
        self.findings.insert(Finding {
            severity: "invalid".to_owned(),
            code: code.to_owned(),
            path: path.into(),
            detail: detail.into(),
        });
    }

    fn incomplete(&mut self, code: &str, path: impl Into<String>, detail: impl Into<String>) {
        self.findings.insert(Finding {
            severity: "incomplete".to_owned(),
            code: code.to_owned(),
            path: path.into(),
            detail: detail.into(),
        });
    }

    fn invalid_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.severity == "invalid")
            .count()
    }

    fn incomplete_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.severity == "incomplete")
            .count()
    }
}

fn get_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn get_array<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn get_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn get_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn is_sha256(value: Option<&str>) -> bool {
    value.is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn duplicate_strings(values: &[Value]) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for value in values {
        if let Some(text) = value.as_str() {
            if !seen.insert(text.to_owned()) {
                duplicates.insert(text.to_owned());
            }
        }
    }
    duplicates
}

fn strings(value: Option<&Vec<Value>>) -> Vec<String> {
    value
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn valid_timestamp(value: &str) -> bool {
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

    let number = |start: usize, end: usize| -> Option<u32> {
        std::str::from_utf8(&bytes[start..end]).ok()?.parse().ok()
    };
    let year = number(0, 4).unwrap_or(0);
    let month = number(5, 7).unwrap_or(0);
    let day = number(8, 10).unwrap_or(0);
    let hour = number(11, 13).unwrap_or(99);
    let minute = number(14, 16).unwrap_or(99);
    let second = number(17, 19).unwrap_or(99);

    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }

    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    day >= 1 && day <= days_in_month
}

fn check_timestamp(audit: &mut Audit, object: &Value, key: &str, path: &str) -> Option<String> {
    match get_str(object, key) {
        Some(value) if valid_timestamp(value) => Some(value.to_owned()),
        Some(_) => {
            audit.invalid(
                "NON_CANONICAL_TIMESTAMP",
                format!("{path}/{key}"),
                "timestamp must be a valid UTC timestamp at whole-second precision (YYYY-MM-DDTHH:MM:SSZ)",
            );
            None
        }
        None => {
            audit.invalid(
                "MISSING_TIMESTAMP",
                format!("{path}/{key}"),
                "required timestamp is missing or is not a string",
            );
            None
        }
    }
}

fn check_digest(audit: &mut Audit, object: &Value, key: &str, path: &str) {
    if !is_sha256(get_str(object, key)) {
        audit.invalid(
            "INVALID_SHA256",
            format!("{path}/{key}"),
            "expected a 64-character hexadecimal SHA-256 digest",
        );
    }
}

fn required_string<'a>(
    audit: &mut Audit,
    object: &'a Value,
    key: &str,
    path: &str,
) -> Option<&'a str> {
    match get_str(object, key) {
        Some(value) if !value.trim().is_empty() => Some(value),
        _ => {
            audit.invalid(
                "MISSING_REQUIRED_STRING",
                format!("{path}/{key}"),
                "required non-empty string is missing",
            );
            None
        }
    }
}

fn audit_manifest(manifest: &Value, input_sha256: &str) -> Value {
    let mut audit = Audit::default();
    let declared_disposition = get_str(manifest, "disposition").unwrap_or("missing");
    let evaluated_at = check_timestamp(&mut audit, manifest, "evaluated_at", "manifest");

    if get_str(manifest, "schema_version") != Some(MANIFEST_SCHEMA) {
        audit.invalid(
            "UNSUPPORTED_SCHEMA_VERSION",
            "manifest/schema_version",
            "runtime verifier accepts only luminous.built-environment-dependency-coverage.v0.1",
        );
    }
    if get_str(manifest, "claim_ceiling") != Some("coverage_only") {
        audit.invalid(
            "CLAIM_CEILING_ESCALATION",
            "manifest/claim_ceiling",
            "coverage manifests must retain the coverage_only claim ceiling",
        );
    }
    if get_str(manifest, "action_authority") != Some("none") {
        audit.invalid(
            "ACTION_AUTHORITY_ESCALATION",
            "manifest/action_authority",
            "coverage verification grants no action or actuation authority",
        );
    }
    if !matches!(
        declared_disposition,
        "complete_for_declared_scope" | "incomplete_or_indeterminate" | "invalid"
    ) {
        audit.invalid(
            "INVALID_DECLARED_DISPOSITION",
            "manifest/disposition",
            "unknown coverage disposition",
        );
    } else if declared_disposition == "incomplete_or_indeterminate" {
        audit.incomplete(
            "DECLARED_INCOMPLETE",
            "manifest/disposition",
            "runtime verification cannot promote a manifest declared incomplete",
        );
    } else if declared_disposition == "invalid" {
        audit.invalid(
            "DECLARED_INVALID",
            "manifest/disposition",
            "manifest declares itself invalid",
        );
    }

    let profile = manifest.get("coverage_profile").unwrap_or(&Value::Null);
    let required_values = get_array(profile, "required_dependency_families");
    let excluded_values = get_array(profile, "excluded_dependency_families");
    let required = strings(required_values);
    let excluded = strings(excluded_values);

    if let Some(values) = required_values {
        for duplicate in duplicate_strings(values) {
            audit.invalid(
                "DUPLICATE_REQUIRED_FAMILY",
                "manifest/coverage_profile/required_dependency_families",
                format!("dependency family {duplicate} appears more than once"),
            );
        }
    } else {
        audit.invalid(
            "MISSING_REQUIRED_FAMILIES",
            "manifest/coverage_profile/required_dependency_families",
            "required dependency family list is missing",
        );
    }
    if let Some(values) = excluded_values {
        for duplicate in duplicate_strings(values) {
            audit.invalid(
                "DUPLICATE_EXCLUDED_FAMILY",
                "manifest/coverage_profile/excluded_dependency_families",
                format!("dependency family {duplicate} appears more than once"),
            );
        }
    }
    let required_set = required.iter().cloned().collect::<BTreeSet<_>>();
    let excluded_set = excluded.iter().cloned().collect::<BTreeSet<_>>();
    for family in required_set.intersection(&excluded_set) {
        audit.invalid(
            "REQUIRED_EXCLUDED_FAMILY_OVERLAP",
            "manifest/coverage_profile",
            format!("dependency family {family} is both required and excluded"),
        );
    }
    check_digest(
        &mut audit,
        profile,
        "applicability_basis_sha256",
        "manifest/coverage_profile",
    );
    required_string(&mut audit, profile, "profile_id", "manifest/coverage_profile");
    required_string(
        &mut audit,
        profile,
        "profile_version",
        "manifest/coverage_profile",
    );

    let source_rows = get_array(manifest, "source_inventory");
    let mut sources: BTreeMap<String, &Value> = BTreeMap::new();
    if let Some(source_rows) = source_rows {
        for source in source_rows {
            let source_id = required_string(
                &mut audit,
                source,
                "source_id",
                "manifest/source_inventory",
            );
            let Some(source_id) = source_id else {
                continue;
            };
            if sources.insert(source_id.to_owned(), source).is_some() {
                audit.invalid(
                    "DUPLICATE_SOURCE_ID",
                    format!("manifest/source_inventory/{source_id}"),
                    "source IDs must be unique within a manifest",
                );
                continue;
            }

            required_string(
                &mut audit,
                source,
                "owner_namespace",
                &format!("manifest/source_inventory/{source_id}"),
            );
            required_string(
                &mut audit,
                source,
                "native_snapshot_id",
                &format!("manifest/source_inventory/{source_id}"),
            );
            required_string(
                &mut audit,
                source,
                "schema_id",
                &format!("manifest/source_inventory/{source_id}"),
            );
            required_string(
                &mut audit,
                source,
                "schema_version",
                &format!("manifest/source_inventory/{source_id}"),
            );
            check_digest(
                &mut audit,
                source,
                "snapshot_sha256",
                &format!("manifest/source_inventory/{source_id}"),
            );
            check_digest(
                &mut audit,
                source,
                "schema_sha256",
                &format!("manifest/source_inventory/{source_id}"),
            );

            let captured = check_timestamp(
                &mut audit,
                source,
                "captured_at",
                &format!("manifest/source_inventory/{source_id}"),
            );
            let retrieved = check_timestamp(
                &mut audit,
                source,
                "retrieved_at",
                &format!("manifest/source_inventory/{source_id}"),
            );
            if let (Some(captured), Some(retrieved)) = (&captured, &retrieved) {
                if captured > retrieved {
                    audit.invalid(
                        "SOURCE_TIME_REVERSAL",
                        format!("manifest/source_inventory/{source_id}"),
                        "source retrieval precedes its captured snapshot",
                    );
                }
                if let Some(evaluated_at) = &evaluated_at {
                    if retrieved > evaluated_at {
                        audit.incomplete(
                            "SOURCE_RETRIEVED_AFTER_EVALUATION",
                            format!("manifest/source_inventory/{source_id}"),
                            "source retrieval time is later than the manifest evaluation time",
                        );
                    }
                }
            }

            match get_str(source, "access_state") {
                Some("available") => {}
                Some(state) => audit.incomplete(
                    "SOURCE_NOT_AVAILABLE",
                    format!("manifest/source_inventory/{source_id}/access_state"),
                    format!("source access state is {state}, not available"),
                ),
                None => audit.invalid(
                    "MISSING_SOURCE_ACCESS_STATE",
                    format!("manifest/source_inventory/{source_id}/access_state"),
                    "source access state is missing",
                ),
            }

            let permitted = strings(get_array(source, "permitted_uses"));
            match get_str(source, "rights_state") {
                Some("verified") | Some("not_applicable") => {
                    let intended_use = get_str(
                        manifest.get("case_scope").unwrap_or(&Value::Null),
                        "intended_use",
                    );
                    let required_use = match intended_use {
                        Some("engineering_change_impact") => Some("impact_analysis"),
                        Some("design_review") => Some("design_review"),
                        Some("construction_release_review") => {
                            Some("construction_release_review")
                        }
                        Some("maintenance_review") => Some("maintenance_review"),
                        Some("other_declared_review") => None,
                        _ => None,
                    };
                    if required_use.is_some_and(|required_use| !permitted.iter().any(|p| p == required_use)) {
                        audit.incomplete(
                            "SOURCE_USE_NOT_PERMITTED",
                            format!("manifest/source_inventory/{source_id}/permitted_uses"),
                            "source rights do not declare the intended case-scope use",
                        );
                    }
                }
                Some("unknown") => {
                    if !permitted.is_empty() {
                        audit.invalid(
                            "UNKNOWN_RIGHTS_WITH_PERMITTED_USE",
                            format!("manifest/source_inventory/{source_id}"),
                            "unknown source rights cannot carry permitted uses",
                        );
                    }
                    audit.incomplete(
                        "SOURCE_RIGHTS_UNKNOWN",
                        format!("manifest/source_inventory/{source_id}/rights_state"),
                        "unknown rights cannot support a complete coverage conclusion",
                    );
                }
                Some(state) => audit.incomplete(
                    "SOURCE_RIGHTS_NOT_CLEAR",
                    format!("manifest/source_inventory/{source_id}/rights_state"),
                    format!("source rights state is {state}"),
                ),
                None => audit.invalid(
                    "MISSING_SOURCE_RIGHTS_STATE",
                    format!("manifest/source_inventory/{source_id}/rights_state"),
                    "source rights state is missing",
                ),
            }

            let extraction = source.get("extraction").unwrap_or(&Value::Null);
            required_string(
                &mut audit,
                extraction,
                "adapter_id",
                &format!("manifest/source_inventory/{source_id}/extraction"),
            );
            required_string(
                &mut audit,
                extraction,
                "adapter_version",
                &format!("manifest/source_inventory/{source_id}/extraction"),
            );
            check_digest(
                &mut audit,
                extraction,
                "adapter_build_sha256",
                &format!("manifest/source_inventory/{source_id}/extraction"),
            );
            check_digest(
                &mut audit,
                extraction,
                "configuration_sha256",
                &format!("manifest/source_inventory/{source_id}/extraction"),
            );
            check_digest(
                &mut audit,
                extraction,
                "query_sha256",
                &format!("manifest/source_inventory/{source_id}/extraction"),
            );

            let completeness = source.get("completeness").unwrap_or(&Value::Null);
            required_string(
                &mut audit,
                completeness,
                "method",
                &format!("manifest/source_inventory/{source_id}/completeness"),
            );
            required_string(
                &mut audit,
                completeness,
                "boundary",
                &format!("manifest/source_inventory/{source_id}/completeness"),
            );
            check_digest(
                &mut audit,
                completeness,
                "proof_sha256",
                &format!("manifest/source_inventory/{source_id}/completeness"),
            );
            if get_bool(completeness, "pagination_complete") != Some(true) {
                audit.incomplete(
                    "SOURCE_PAGINATION_INCOMPLETE",
                    format!("manifest/source_inventory/{source_id}/completeness"),
                    "source pagination is not proven complete",
                );
            }
            let returned = get_u64(completeness, "returned_record_count");
            if returned.is_none() {
                audit.invalid(
                    "MISSING_RETURNED_RECORD_COUNT",
                    format!("manifest/source_inventory/{source_id}/completeness"),
                    "returned record count is missing or invalid",
                );
            }
            if let Some(total) = completeness.get("total_record_count") {
                if !total.is_null() {
                    match (total.as_u64(), returned) {
                        (Some(total), Some(returned)) if total == returned => {}
                        (Some(_), Some(_)) => audit.incomplete(
                            "SOURCE_RECORD_COUNT_MISMATCH",
                            format!("manifest/source_inventory/{source_id}/completeness"),
                            "source total record count differs from returned record count",
                        ),
                        _ => audit.invalid(
                            "INVALID_SOURCE_RECORD_COUNT",
                            format!("manifest/source_inventory/{source_id}/completeness"),
                            "source record counts must be non-negative integers or a null total",
                        ),
                    }
                }
            }
        }
    } else {
        audit.invalid(
            "MISSING_SOURCE_INVENTORY",
            "manifest/source_inventory",
            "source inventory array is missing",
        );
    }

    let relationship_rows = get_array(manifest, "relationship_coverage");
    let mut families: BTreeMap<String, &Value> = BTreeMap::new();
    if let Some(relationship_rows) = relationship_rows {
        for row in relationship_rows {
            let family = required_string(
                &mut audit,
                row,
                "family",
                "manifest/relationship_coverage",
            );
            let Some(family) = family else {
                continue;
            };
            if families.insert(family.to_owned(), row).is_some() {
                audit.invalid(
                    "DUPLICATE_RELATIONSHIP_FAMILY",
                    format!("manifest/relationship_coverage/{family}"),
                    "each dependency family must appear exactly once",
                );
                continue;
            }

            let required_flag = get_bool(row, "required");
            if required_set.contains(family) && required_flag != Some(true) {
                audit.invalid(
                    "REQUIRED_FAMILY_FLAG_MISMATCH",
                    format!("manifest/relationship_coverage/{family}"),
                    "a family listed as required by the profile must be marked required",
                );
            }
            if !required_set.contains(family) && required_flag == Some(true) {
                audit.invalid(
                    "UNDECLARED_REQUIRED_FAMILY",
                    format!("manifest/relationship_coverage/{family}"),
                    "relationship row is required but absent from the profile's required family list",
                );
            }
            if excluded_set.contains(family) && required_flag == Some(true) {
                audit.invalid(
                    "EXCLUDED_FAMILY_MARKED_REQUIRED",
                    format!("manifest/relationship_coverage/{family}"),
                    "an excluded dependency family cannot be required",
                );
            }

            match get_str(row, "status") {
                Some("complete") => {}
                Some(status) => {
                    if required_flag == Some(true) {
                        audit.incomplete(
                            "REQUIRED_FAMILY_INCOMPLETE",
                            format!("manifest/relationship_coverage/{family}/status"),
                            format!("required dependency family status is {status}"),
                        );
                    }
                }
                None => audit.invalid(
                    "MISSING_FAMILY_STATUS",
                    format!("manifest/relationship_coverage/{family}/status"),
                    "relationship family status is missing",
                ),
            }

            let source_ids = get_array(row, "source_ids");
            if let Some(source_ids) = source_ids {
                for duplicate in duplicate_strings(source_ids) {
                    audit.invalid(
                        "DUPLICATE_FAMILY_SOURCE",
                        format!("manifest/relationship_coverage/{family}/source_ids"),
                        format!("source {duplicate} is listed more than once"),
                    );
                }
                for source_id in strings(Some(source_ids)) {
                    if !sources.contains_key(&source_id) {
                        audit.invalid(
                            "UNKNOWN_FAMILY_SOURCE",
                            format!("manifest/relationship_coverage/{family}/source_ids"),
                            format!("relationship family references unknown source {source_id}"),
                        );
                    }
                }
            } else {
                audit.invalid(
                    "MISSING_FAMILY_SOURCES",
                    format!("manifest/relationship_coverage/{family}/source_ids"),
                    "relationship family source list is missing",
                );
            }

            let expected = row.get("expected_relationship_count").and_then(Value::as_u64);
            let observed = get_u64(row, "observed_relationship_count");
            if observed.is_none() {
                audit.invalid(
                    "MISSING_OBSERVED_RELATIONSHIP_COUNT",
                    format!("manifest/relationship_coverage/{family}"),
                    "observed relationship count is missing or invalid",
                );
            }
            if expected.is_none() {
                audit.incomplete(
                    "EXPECTED_RELATIONSHIP_COUNT_UNKNOWN",
                    format!("manifest/relationship_coverage/{family}"),
                    "expected relationship count is unknown; exact family reconciliation is unavailable",
                );
            } else if expected != observed {
                audit.incomplete(
                    "FAMILY_RELATIONSHIP_COUNT_MISMATCH",
                    format!("manifest/relationship_coverage/{family}"),
                    "expected and observed relationship counts do not agree",
                );
            }

            let unresolved = get_u64(row, "unresolved_count");
            let unsupported = get_u64(row, "unsupported_count");
            if unresolved.is_none() || unsupported.is_none() {
                audit.invalid(
                    "MISSING_FAMILY_EXCEPTION_COUNTS",
                    format!("manifest/relationship_coverage/{family}"),
                    "unresolved and unsupported counts must be non-negative integers",
                );
            }
            if get_str(row, "status") == Some("complete")
                && (unresolved.unwrap_or(1) > 0 || unsupported.unwrap_or(1) > 0)
            {
                audit.invalid(
                    "COMPLETE_FAMILY_HAS_EXCEPTIONS",
                    format!("manifest/relationship_coverage/{family}"),
                    "a complete relationship family cannot contain unresolved or unsupported relationships",
                );
            }
            check_digest(
                &mut audit,
                row,
                "mapping_rule_sha256",
                &format!("manifest/relationship_coverage/{family}"),
            );
            check_digest(
                &mut audit,
                row,
                "coverage_evidence_sha256",
                &format!("manifest/relationship_coverage/{family}"),
            );
        }
    } else {
        audit.invalid(
            "MISSING_RELATIONSHIP_COVERAGE",
            "manifest/relationship_coverage",
            "relationship coverage array is missing",
        );
    }

    for family in &required_set {
        if !families.contains_key(family) {
            audit.incomplete(
                "REQUIRED_FAMILY_NOT_REPRESENTED",
                format!("manifest/relationship_coverage/{family}"),
                "profile requires this family but no coverage row exists",
            );
        }
    }

    let edge_rows = get_array(manifest, "edge_provenance");
    let mut edge_ids = BTreeSet::new();
    let mut source_relationships = BTreeSet::new();
    let mut observed_by_family: BTreeMap<String, u64> = BTreeMap::new();

    if let Some(edge_rows) = edge_rows {
        for edge in edge_rows {
            let edge_id = required_string(
                &mut audit,
                edge,
                "edge_id",
                "manifest/edge_provenance",
            );
            let family = required_string(
                &mut audit,
                edge,
                "family",
                "manifest/edge_provenance",
            );
            let source_id = required_string(
                &mut audit,
                edge,
                "source_id",
                "manifest/edge_provenance",
            );
            let source_relationship_id = required_string(
                &mut audit,
                edge,
                "source_relationship_id",
                "manifest/edge_provenance",
            );
            let from_key = required_string(
                &mut audit,
                edge,
                "from_native_key",
                "manifest/edge_provenance",
            );
            let to_key = required_string(
                &mut audit,
                edge,
                "to_native_key",
                "manifest/edge_provenance",
            );

            if let Some(edge_id) = edge_id {
                if !edge_ids.insert(edge_id.to_owned()) {
                    audit.invalid(
                        "DUPLICATE_EDGE_ID",
                        format!("manifest/edge_provenance/{edge_id}"),
                        "edge IDs must be unique",
                    );
                }
            }
            if let (Some(source_id), Some(relationship_id)) =
                (source_id, source_relationship_id)
            {
                if !source_relationships.insert((source_id.to_owned(), relationship_id.to_owned())) {
                    audit.invalid(
                        "DUPLICATE_SOURCE_RELATIONSHIP",
                        "manifest/edge_provenance",
                        format!("source relationship {source_id}/{relationship_id} is represented more than once"),
                    );
                }
            }

            if let Some(family) = family {
                *observed_by_family.entry(family.to_owned()).or_default() += 1;
                let family_row = families.get(family).copied();
                let Some(family_row) = family_row else {
                    audit.invalid(
                        "EDGE_WITHOUT_FAMILY_COVERAGE",
                        format!("manifest/edge_provenance/{}", edge_id.unwrap_or("unknown")),
                        format!("edge family {family} has no relationship coverage row"),
                    );
                    continue;
                };

                if let Some(source_id) = source_id {
                    let family_sources = strings(get_array(family_row, "source_ids"));
                    if !family_sources.iter().any(|candidate| candidate == source_id) {
                        audit.invalid(
                            "EDGE_SOURCE_NOT_IN_FAMILY",
                            format!("manifest/edge_provenance/{}", edge_id.unwrap_or("unknown")),
                            format!("source {source_id} is not declared for family {family}"),
                        );
                    }
                    if let Some(source) = sources.get(source_id).copied() {
                        if get_str(edge, "source_snapshot_sha256")
                            != get_str(source, "snapshot_sha256")
                        {
                            audit.invalid(
                                "EDGE_SNAPSHOT_BINDING_MISMATCH",
                                format!("manifest/edge_provenance/{}", edge_id.unwrap_or("unknown")),
                                "edge source snapshot digest does not match the inventoried source snapshot",
                            );
                        }
                        if get_str(edge, "extraction_build_sha256")
                            != get_str(
                                source.get("extraction").unwrap_or(&Value::Null),
                                "adapter_build_sha256",
                            )
                        {
                            audit.invalid(
                                "EDGE_EXTRACTOR_BINDING_MISMATCH",
                                format!("manifest/edge_provenance/{}", edge_id.unwrap_or("unknown")),
                                "edge extractor build does not match the inventoried adapter build",
                            );
                        }
                    }
                }

                if get_str(edge, "mapping_rule_sha256")
                    != get_str(family_row, "mapping_rule_sha256")
                {
                    audit.invalid(
                        "EDGE_MAPPING_RULE_MISMATCH",
                        format!("manifest/edge_provenance/{}", edge_id.unwrap_or("unknown")),
                        "edge mapping rule does not match the family mapping rule",
                    );
                }

                if from_key.is_none() || to_key.is_none() {
                    audit.invalid(
                        "MISSING_EDGE_ENDPOINT",
                        format!("manifest/edge_provenance/{}", edge_id.unwrap_or("unknown")),
                        "edge native endpoints must be non-empty",
                    );
                }

                match get_str(edge, "mapping_disposition") {
                    Some("exact_native_relationship") | Some("deterministic_mapping") => {}
                    Some(disposition) => audit.incomplete(
                        "EDGE_MAPPING_NOT_DECISION_GRADE",
                        format!("manifest/edge_provenance/{}", edge_id.unwrap_or("unknown")),
                        format!("mapping disposition {disposition} is not an exact or deterministic relationship"),
                    ),
                    None => audit.invalid(
                        "MISSING_EDGE_MAPPING_DISPOSITION",
                        format!("manifest/edge_provenance/{}", edge_id.unwrap_or("unknown")),
                        "edge mapping disposition is missing",
                    ),
                }
            }

            check_digest(
                &mut audit,
                edge,
                "source_snapshot_sha256",
                &format!(
                    "manifest/edge_provenance/{}",
                    edge_id.unwrap_or("unknown")
                ),
            );
            check_digest(
                &mut audit,
                edge,
                "extraction_build_sha256",
                &format!(
                    "manifest/edge_provenance/{}",
                    edge_id.unwrap_or("unknown")
                ),
            );
            check_digest(
                &mut audit,
                edge,
                "mapping_rule_sha256",
                &format!(
                    "manifest/edge_provenance/{}",
                    edge_id.unwrap_or("unknown")
                ),
            );
        }
    } else {
        audit.invalid(
            "MISSING_EDGE_PROVENANCE",
            "manifest/edge_provenance",
            "edge provenance array is missing",
        );
    }

    for (family, row) in &families {
        let observed_edges = observed_by_family.get(family).copied().unwrap_or(0);
        let declared_observed = get_u64(row, "observed_relationship_count");
        if declared_observed != Some(observed_edges) {
            audit.incomplete(
                "EDGE_PROVENANCE_COUNT_MISMATCH",
                format!("manifest/relationship_coverage/{family}"),
                format!(
                    "family declares {} observed relationships but {} source-bound edge records exist",
                    declared_observed.map_or_else(|| "unknown".to_owned(), |count| count.to_string()),
                    observed_edges
                ),
            );
        }
    }
    for family in observed_by_family.keys() {
        if !families.contains_key(family) {
            audit.invalid(
                "UNDECLARED_EDGE_FAMILY",
                "manifest/edge_provenance",
                format!("edge provenance contains undeclared family {family}"),
            );
        }
    }

    let absence_rows = get_array(manifest, "absence_assertions");
    let mut absence_ids = BTreeSet::new();
    if let Some(absence_rows) = absence_rows {
        for assertion in absence_rows {
            let assertion_id = required_string(
                &mut audit,
                assertion,
                "assertion_id",
                "manifest/absence_assertions",
            );
            let Some(assertion_id) = assertion_id else {
                continue;
            };
            if !absence_ids.insert(assertion_id.to_owned()) {
                audit.invalid(
                    "DUPLICATE_ABSENCE_ASSERTION_ID",
                    format!("manifest/absence_assertions/{assertion_id}"),
                    "absence assertion IDs must be unique",
                );
                continue;
            }

            let source_id = required_string(
                &mut audit,
                assertion,
                "source_id",
                &format!("manifest/absence_assertions/{assertion_id}"),
            );
            let snapshot = get_str(assertion, "source_snapshot_sha256");
            check_digest(
                &mut audit,
                assertion,
                "source_snapshot_sha256",
                &format!("manifest/absence_assertions/{assertion_id}"),
            );
            check_digest(
                &mut audit,
                assertion,
                "query_sha256",
                &format!("manifest/absence_assertions/{assertion_id}"),
            );

            if let Some(source_id) = source_id {
                match sources.get(source_id).copied() {
                    Some(source) if snapshot == get_str(source, "snapshot_sha256") => {}
                    Some(_) => audit.invalid(
                        "ABSENCE_SNAPSHOT_BINDING_MISMATCH",
                        format!("manifest/absence_assertions/{assertion_id}"),
                        "absence assertion is not bound to the inventoried source snapshot",
                    ),
                    None => audit.invalid(
                        "ABSENCE_UNKNOWN_SOURCE",
                        format!("manifest/absence_assertions/{assertion_id}"),
                        format!("absence assertion references unknown source {source_id}"),
                    ),
                }
            }

            let checked = check_timestamp(
                &mut audit,
                assertion,
                "checked_at",
                &format!("manifest/absence_assertions/{assertion_id}"),
            );
            let valid_until = check_timestamp(
                &mut audit,
                assertion,
                "valid_until",
                &format!("manifest/absence_assertions/{assertion_id}"),
            );
            if let (Some(checked), Some(valid_until)) = (&checked, &valid_until) {
                if checked > valid_until {
                    audit.invalid(
                        "ABSENCE_VALIDITY_TIME_REVERSAL",
                        format!("manifest/absence_assertions/{assertion_id}"),
                        "absence assertion expires before it was checked",
                    );
                }
                if let Some(source_id) = source_id {
                    if let Some(source) = sources.get(source_id) {
                        if get_str(source, "captured_at").is_some_and(|captured| captured > checked) {
                            audit.invalid(
                                "ABSENCE_CHECK_PRECEDES_SOURCE_SNAPSHOT",
                                format!("manifest/absence_assertions/{assertion_id}"),
                                "absence query time precedes the captured source snapshot it claims to query",
                            );
                        }
                    }
                }
                if let Some(evaluated_at) = &evaluated_at {
                    if checked > evaluated_at {
                        audit.invalid(
                            "ABSENCE_CHECK_AFTER_EVALUATION",
                            format!("manifest/absence_assertions/{assertion_id}"),
                            "absence query was checked after the manifest evaluation time",
                        );
                    }
                    if valid_until < evaluated_at {
                        audit.incomplete(
                            "ABSENCE_ASSERTION_EXPIRED",
                            format!("manifest/absence_assertions/{assertion_id}"),
                            "absence assertion expired before the manifest evaluation time",
                        );
                    }
                }
            }

            let result_count = get_u64(assertion, "result_count");
            if result_count.is_none() {
                audit.invalid(
                    "INVALID_ABSENCE_RESULT_COUNT",
                    format!("manifest/absence_assertions/{assertion_id}"),
                    "absence result count must be a non-negative integer",
                );
            }
            if result_count == Some(0) {
                if get_bool(assertion, "enumeration_complete") != Some(true)
                    || get_bool(assertion, "pagination_complete") != Some(true)
                    || !is_sha256(get_str(assertion, "source_completeness_proof_sha256"))
                {
                    audit.incomplete(
                        "UNPROVEN_ABSENCE",
                        format!("manifest/absence_assertions/{assertion_id}"),
                        "zero results require complete bounded enumeration, complete pagination, and a source completeness proof digest",
                    );
                }
            }
        }
    } else {
        audit.invalid(
            "MISSING_ABSENCE_ASSERTIONS",
            "manifest/absence_assertions",
            "absence assertions array is missing",
        );
    }

    let completeness_basis = manifest.get("completeness_basis").unwrap_or(&Value::Null);
    required_string(
        &mut audit,
        completeness_basis,
        "method",
        "manifest/completeness_basis",
    );
    required_string(
        &mut audit,
        completeness_basis,
        "boundary_statement",
        "manifest/completeness_basis",
    );
    check_digest(
        &mut audit,
        completeness_basis,
        "source_inventory_sha256",
        "manifest/completeness_basis",
    );
    check_digest(
        &mut audit,
        completeness_basis,
        "dependency_profile_sha256",
        "manifest/completeness_basis",
    );
    if get_array(completeness_basis, "limitations")
        .is_none_or(|limitations| limitations.is_empty())
    {
        audit.invalid(
            "MISSING_COMPLETENESS_LIMITATIONS",
            "manifest/completeness_basis/limitations",
            "completeness basis must state at least one limitation",
        );
    }

    let review = manifest.get("independent_review").unwrap_or(&Value::Null);
    required_string(
        &mut audit,
        review,
        "reviewer_id",
        "manifest/independent_review",
    );
    check_digest(
        &mut audit,
        review,
        "review_scope_sha256",
        "manifest/independent_review",
    );
    check_digest(
        &mut audit,
        review,
        "review_evidence_sha256",
        "manifest/independent_review",
    );
    let reviewed_at = if get_str(review, "status") == Some("accepted_for_declared_scope") {
        check_timestamp(
            &mut audit,
            review,
            "reviewed_at",
            "manifest/independent_review",
        )
    } else if review.get("reviewed_at").is_some_and(Value::is_null) {
        None
    } else {
        check_timestamp(
            &mut audit,
            review,
            "reviewed_at",
            "manifest/independent_review",
        )
    };
    if get_str(review, "status") != Some("accepted_for_declared_scope") {
        audit.incomplete(
            "INDEPENDENT_REVIEW_NOT_ACCEPTED",
            "manifest/independent_review/status",
            "coverage has not been accepted by an independent reviewer for the declared scope",
        );
    }
    if let (Some(reviewed_at), Some(evaluated_at)) = (&reviewed_at, &evaluated_at) {
        if reviewed_at > evaluated_at {
            audit.incomplete(
                "REVIEW_AFTER_EVALUATION",
                "manifest/independent_review/reviewed_at",
                "review record is later than the coverage evaluation",
            );
        }
    }

    let invalid_count = audit.invalid_count();
    let incomplete_count = audit.incomplete_count();
    let verified_disposition = if invalid_count > 0 || declared_disposition == "invalid" {
        "invalid"
    } else if incomplete_count > 0 || declared_disposition == "incomplete_or_indeterminate" {
        "incomplete_or_indeterminate"
    } else {
        "complete_for_declared_scope"
    };

    let findings = audit
        .findings
        .into_iter()
        .map(|finding| {
            json!({
                "severity": finding.severity,
                "code": finding.code,
                "path": finding.path,
                "detail": finding.detail
            })
        })
        .collect::<Vec<_>>();

    json!({
        "verifier": {
            "id": VERIFIER_ID,
            "version": VERIFIER_VERSION
        },
        "input_sha256": input_sha256,
        "declared_disposition": declared_disposition,
        "verified_disposition": verified_disposition,
        "no_impact_eligible": false,
        "reliance_gate": "blocked_unverified_external_attestations_and_profile_adequacy",
        "summary": {
            "invalid_findings": invalid_count,
            "incomplete_findings": incomplete_count,
            "source_count": sources.len(),
            "required_family_count": required_set.len(),
            "relationship_family_count": families.len(),
            "edge_count": edge_rows.map_or(0, Vec::len),
            "absence_assertion_count": absence_rows.map_or(0, Vec::len)
        },
        "findings": findings,
        "not_checked": NOT_CHECKED
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn verify_bytes(bytes: &[u8]) -> Result<Value, String> {
    let parsed: StrictValue =
        serde_json::from_slice(bytes).map_err(|error| format!("strict JSON parse failed: {error}"))?;
    let root = parsed.0;
    let manifest = root.get("coverage_manifest").unwrap_or(&root);
    Ok(audit_manifest(manifest, &sha256_hex(bytes)))
}

fn main() -> ExitCode {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("usage: built-environment-coverage-audit <manifest-or-fixture.json>");
        return ExitCode::from(64);
    };

    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("unable to read {}: {error}", path.to_string_lossy());
            return ExitCode::from(66);
        }
    };
    let report = match verify_bytes(&bytes) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(65);
        }
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_owned())
    );

    match get_str(&report, "verified_disposition") {
        Some("complete_for_declared_scope") => ExitCode::SUCCESS,
        Some("incomplete_or_indeterminate") => ExitCode::from(2),
        _ => ExitCode::from(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        let parsed: StrictValue = serde_json::from_str(include_str!(
            "../../../schemas/examples/built-environment-dependency-coverage-complete-synthetic-v0.1.json"
        ))
        .expect("frozen fixture must be strict JSON");
        parsed.0
    }

    fn manifest_mut(document: &mut Value) -> &mut Value {
        document
            .get_mut("coverage_manifest")
            .expect("fixture wrapper contains coverage_manifest")
    }

    fn report(document: &Value) -> Value {
        let manifest = document.get("coverage_manifest").unwrap();
        audit_manifest(manifest, &"a".repeat(64))
    }

    fn findings(report: &Value) -> &Vec<Value> {
        report.get("findings").and_then(Value::as_array).unwrap()
    }

    #[test]
    fn frozen_fixture_is_internally_complete_but_never_grants_no_impact_reliance() {
        let result = report(&fixture());
        assert_eq!(
            get_str(&result, "verified_disposition"),
            Some("complete_for_declared_scope")
        );
        assert_eq!(result["no_impact_eligible"], false);
        assert_eq!(
            get_str(&result, "reliance_gate"),
            Some("blocked_unverified_external_attestations_and_profile_adequacy")
        );
        assert_eq!(result["summary"]["edge_count"], 10);
        assert_eq!(result["summary"]["required_family_count"], 6);
    }

    #[test]
    fn omitted_mandatory_edge_with_reconciled_observed_count_is_incomplete() {
        let mut document = fixture();
        let manifest = manifest_mut(&mut document);
        manifest["edge_provenance"].as_array_mut().unwrap().remove(0);
        manifest["relationship_coverage"][0]["observed_relationship_count"] = json!(1);

        let result = report(&document);
        assert_eq!(
            get_str(&result, "verified_disposition"),
            Some("incomplete_or_indeterminate")
        );
        assert!(
            findings(&result)
                .iter()
                .any(|finding| finding["code"] == "FAMILY_RELATIONSHIP_COUNT_MISMATCH")
        );
        assert_eq!(result["no_impact_eligible"], false);
    }

    #[test]
    fn edge_cannot_be_rebound_to_another_source_snapshot() {
        let mut document = fixture();
        manifest_mut(&mut document)["edge_provenance"][0]["source_snapshot_sha256"] =
            json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let result = report(&document);
        assert_eq!(get_str(&result, "verified_disposition"), Some("invalid"));
        assert!(
            findings(&result)
                .iter()
                .any(|finding| finding["code"] == "EDGE_SNAPSHOT_BINDING_MISMATCH")
        );
    }

    #[test]
    fn unavailable_source_never_supports_complete_coverage() {
        let mut document = fixture();
        manifest_mut(&mut document)["source_inventory"][0]["access_state"] =
            json!("unavailable");
        let result = report(&document);
        assert_eq!(
            get_str(&result, "verified_disposition"),
            Some("incomplete_or_indeterminate")
        );
    }

    #[test]
    fn unknown_rights_cannot_be_promoted_by_permitted_uses() {
        let mut document = fixture();
        manifest_mut(&mut document)["source_inventory"][0]["rights_state"] = json!("unknown");
        let result = report(&document);
        assert_eq!(get_str(&result, "verified_disposition"), Some("invalid"));
        assert!(
            findings(&result)
                .iter()
                .any(|finding| finding["code"] == "UNKNOWN_RIGHTS_WITH_PERMITTED_USE")
        );
    }

    #[test]
    fn expired_absence_assertion_downgrades_the_manifest() {
        let mut document = fixture();
        manifest_mut(&mut document)["evaluated_at"] = json!("2026-11-03T13:00:00Z");
        let result = report(&document);
        assert_eq!(
            get_str(&result, "verified_disposition"),
            Some("incomplete_or_indeterminate")
        );
        assert!(
            findings(&result)
                .iter()
                .any(|finding| finding["code"] == "ABSENCE_ASSERTION_EXPIRED")
        );
    }

    #[test]
    fn incomplete_declaration_is_never_promoted() {
        let mut document = fixture();
        manifest_mut(&mut document)["disposition"] = json!("incomplete_or_indeterminate");
        let result = report(&document);
        assert_eq!(
            get_str(&result, "verified_disposition"),
            Some("incomplete_or_indeterminate")
        );
    }

    #[test]
    fn duplicate_json_object_members_are_rejected_before_audit() {
        let error = verify_bytes(br#"{"coverage_manifest":{"disposition":"complete_for_declared_scope","disposition":"invalid"}}"#)
            .expect_err("duplicate member must not be silently overwritten");
        assert!(error.contains("duplicate JSON object member"));
    }

    #[test]
    fn input_vector_permutations_do_not_change_semantic_findings() {
        let original = fixture();
        let mut reordered = fixture();
        let manifest = manifest_mut(&mut reordered);
        manifest["source_inventory"].as_array_mut().unwrap().reverse();
        manifest["relationship_coverage"].as_array_mut().unwrap().reverse();
        manifest["edge_provenance"].as_array_mut().unwrap().reverse();

        let original_report = report(&original);
        let reordered_report = report(&reordered);
        assert_eq!(findings(&original_report), findings(&reordered_report));
        assert_eq!(
            original_report["verified_disposition"],
            reordered_report["verified_disposition"]
        );
    }

    #[test]
    fn required_family_absence_is_incomplete_not_no_impact() {
        let mut document = fixture();
        manifest_mut(&mut document)["relationship_coverage"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["family"] != "solver_inputs");
        let result = report(&document);
        assert_eq!(
            get_str(&result, "verified_disposition"),
            Some("incomplete_or_indeterminate")
        );
        assert!(
            findings(&result)
                .iter()
                .any(|finding| finding["code"] == "REQUIRED_FAMILY_NOT_REPRESENTED")
        );
    }
}
