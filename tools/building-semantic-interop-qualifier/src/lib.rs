//! Synthetic-only semantic mapping admission oracle.
//! It does not parse IFC, Brick, ASHRAE 223, Haystack/XETO or real building graphs.

#![forbid(unsafe_code)]

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fmt;

pub const PROFILE_JSON: &str = include_str!(
    "../../../docs/research/building-semantic-interop/BUILDING_SEMANTIC_PROFILE_V0_1.json"
);
pub const SCHEMA_JSON: &str = include_str!(
    "../../../docs/research/building-semantic-interop/BUILDING_SEMANTIC_PROFILE_V0_1.schema.json"
);
pub const CORPUS_JSON: &str = include_str!(
    "../../../docs/research/building-semantic-interop/BUILDING_SEMANTIC_CORPUS_V0_1.json"
);

const PROFILE_ID: &str = "CITY-BUILDING-SEMANTIC-PROFILE-V0.1";
const CORPUS_ID: &str = "CITY-BUILDING-SEMANTIC-CORPUS-V0.1";
const PROFILE_ROOT_FIELDS: &[&str] = &[
    "schema_version", "profile_id", "classification", "claim_ceiling",
    "mapping_relations", "standards", "policy",
];
const STANDARD_FIELDS: &[&str] = &["key", "family", "release_id", "maturity", "status"];
const MAPPING_FIELDS: &[&str] = &[
    "source", "target", "assertion", "semantics", "evidence",
];
const SOURCE_TARGET_FIELDS: &[&str] = &[
    "standard_key", "release_id", "term_id", "subject_id", "revision", "revision_state",
];
const ASSERTION_FIELDS: &[&str] = &[
    "relation", "exact_equivalence_claim", "subject_relationship", "declared_losses",
];
const SEMANTIC_FIELDS: &[&str] = &[
    "source_quantity_kind", "target_quantity_kind", "source_unit", "target_unit",
    "conversion_rule_id", "conversion_state", "time_basis_preserved",
    "evidence_class_preserved",
];
const EVIDENCE_FIELDS: &[&str] = &[
    "rule_id", "rule_revision", "rule_digest", "evidence_ref",
    "source_rights_state", "target_rights_state",
];
const CASE_FIELDS: &[&str] = &[
    "case_id", "description", "mapping", "expected_disposition",
];

const EXPECTED_STANDARDS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "ifc-4.3",
        "IFC",
        "IFC4.3-ISO16739-1:2024",
        "stable",
        "PINNED_FOR_FIXTURE",
    ),
    (
        "brick-1.4.4",
        "Brick",
        "1.4.4",
        "stable",
        "PINNED_FOR_FIXTURE",
    ),
    (
        "brick-1.5.0-rc1",
        "Brick",
        "1.5.0-rc1",
        "prerelease",
        "SEPARATE_PRERELEASE_LANE",
    ),
    (
        "ashrae-223",
        "ASHRAE Standard 223",
        "VERSION_NOT_PINNED",
        "unresolved",
        "BLOCKED_UNTIL_EXACT_ARTIFACT_PINNED",
    ),
    (
        "haystack-xeto",
        "Project Haystack/XETO",
        "VERSION_NOT_PINNED",
        "unresolved",
        "BLOCKED_UNTIL_EXACT_ARTIFACT_PINNED",
    ),
];

const EXPECTED_CASES: &[(&str, &str)] = &[
    ("SM-001", "ExactEquivalent"),
    ("SM-002", "AcceptedWithDeclaredLoss"),
    ("SM-003", "NotComparable"),
    ("SM-004", "NotComparable"),
    ("SM-005", "NotComparable"),
    ("SM-006", "Rejected"),
    ("SM-007", "Rejected"),
    ("SM-008", "MoreEvidenceRequired"),
    ("SM-009", "MoreEvidenceRequired"),
    ("SM-010", "MoreEvidenceRequired"),
    ("SM-011", "MoreEvidenceRequired"),
    ("SM-012", "Rejected"),
    ("SM-013", "MoreEvidenceRequired"),
    ("SM-014", "MoreEvidenceRequired"),
    ("SM-015", "MoreEvidenceRequired"),
    ("SM-016", "MoreEvidenceRequired"),
    ("SM-017", "NotComparable"),
    ("SM-018", "NotComparable"),
    ("SM-019", "MoreEvidenceRequired"),
    ("SM-020", "AcceptedWithDeclaredLoss"),
    ("SM-021", "AcceptedWithDeclaredLoss"),
    ("SM-022", "AcceptedWithDeclaredLoss"),
    ("SM-023", "AcceptedWithDeclaredLoss"),
    ("SM-024", "MoreEvidenceRequired"),
    ("SM-025", "NotComparable"),
    ("SM-026", "ExactEquivalent"),
    ("SM-027", "MoreEvidenceRequired"),
    ("SM-028", "MoreEvidenceRequired"),
    ("SM-029", "MoreEvidenceRequired"),
    ("SM-030", "Rejected"),
    ("SM-031", "NotComparable"),
    ("SM-032", "AcceptedWithDeclaredLoss"),
    ("SM-033", "NotComparable"),
    ("SM-034", "Rejected"),
    ("SM-035", "MoreEvidenceRequired"),
    ("SM-036", "MoreEvidenceRequired"),
    ("SM-037", "Rejected"),
    ("SM-038", "MoreEvidenceRequired"),
    ("SM-039", "MoreEvidenceRequired"),
    ("SM-040", "MoreEvidenceRequired"),
];

#[derive(Debug)]
struct UniqueJson(Value);

impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueJsonVisitor)
    }
}

struct UniqueJsonVisitor;

impl<'de> Visitor<'de> for UniqueJsonVisitor {
    type Value = UniqueJson;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with no duplicate object member names")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UniqueJson(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UniqueJson(Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UniqueJson(Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| E::custom("non-finite JSON number"))?;
        Ok(UniqueJson(Value::Number(number)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UniqueJson(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UniqueJson(Value::String(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UniqueJson(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UniqueJson(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UniqueJson::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueJson>()? {
            values.push(value.0);
        }
        Ok(UniqueJson(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        let mut seen = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object member {key:?}"
                )));
            }
            let value = map.next_value::<UniqueJson>()?;
            values.insert(key, value.0);
        }
        Ok(UniqueJson(Value::Object(values)))
    }
}

pub fn parse_unique_json(source: &str) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_str(source);
    let value = UniqueJson::deserialize(&mut deserializer)
        .map_err(|error| format!("STRICT_JSON_REJECTED: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("STRICT_JSON_TRAILING_DATA: {error}"))?;
    Ok(value.0)
}

fn exact_keys(value: &Value, fields: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == fields.len() && fields.iter().all(|field| object.contains_key(*field))
    })
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn boolean(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn string_array(value: &Value, key: &str) -> Option<&[Value]> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
}

fn standard_status<'a>(profile: &'a Value, key: &str, release: &str) -> Option<&'a str> {
    let standards = profile.get("standards")?.as_array()?;
    standards
        .iter()
        .find(|entry| text(entry, "key") == key && text(entry, "release_id") == release)
        .map(|entry| text(entry, "maturity"))
}

fn validate_profile(profile: &Value) -> Result<(), String> {
    if !exact_keys(profile, PROFILE_ROOT_FIELDS) {
        return Err("PROFILE_ROOT_FIELDS_INVALID".to_owned());
    }
    if text(profile, "schema_version") != "sol-atlas.building-semantic-profile/0.1"
        || text(profile, "profile_id") != PROFILE_ID
        || text(profile, "classification") != "synthetic_mapping_contract_only"
        || text(profile, "claim_ceiling") != "synthetic_mapping_admission_and_semantic_loss_checks_only"
    {
        return Err("PROFILE_IDENTITY_OR_CLAIM_CEILING_INVALID".to_owned());
    }

    let standards = profile["standards"].as_array().ok_or("STANDARDS_MISSING")?;
    if standards.len() != EXPECTED_STANDARDS.len() {
        return Err("STANDARD_COUNT_INVALID".to_owned());
    }
    for (standard, expected) in standards.iter().zip(EXPECTED_STANDARDS.iter()) {
        if !exact_keys(standard, STANDARD_FIELDS)
            || text(standard, "key") != expected.0
            || text(standard, "family") != expected.1
            || text(standard, "release_id") != expected.2
            || text(standard, "maturity") != expected.3
            || text(standard, "status") != expected.4
        {
            return Err(format!("STANDARD_PIN_DRIFT:{}", expected.0));
        }
    }

    let relations = string_array(profile, "mapping_relations").ok_or("MAPPING_RELATIONS_MISSING")?;
    let expected_relations = [
        "exact", "narrower", "broader", "related", "composed", "derived", "unresolved",
    ];
    if relations.len() != expected_relations.len()
        || !relations
            .iter()
            .zip(expected_relations)
            .all(|(value, expected)| value.as_str() == Some(expected))
    {
        return Err("MAPPING_RELATION_VOCABULARY_DRIFT".to_owned());
    }

    let policy = profile.get("policy").ok_or("POLICY_MISSING")?;
    let policy_fields = [
        "exact_equivalence_requires_same_subject",
        "exact_equivalence_requires_same_quantity_kind",
        "exact_equivalence_requires_same_unit",
        "exact_equivalence_requires_preserved_time_basis",
        "exact_equivalence_requires_preserved_evidence_class",
        "prerelease_mappings_admitted_as_stable",
        "unknown_standard_release_admitted",
        "unknown_rights_admitted",
        "automatic_label_based_equivalence_allowed",
        "new_core_ontology_allowed",
    ];
    if !exact_keys(policy, &policy_fields) {
        return Err("POLICY_FIELDS_INVALID".to_owned());
    }
    for key in [
        "exact_equivalence_requires_same_subject",
        "exact_equivalence_requires_same_quantity_kind",
        "exact_equivalence_requires_same_unit",
        "exact_equivalence_requires_preserved_time_basis",
        "exact_equivalence_requires_preserved_evidence_class",
    ] {
        if boolean(policy, key) != Some(true) {
            return Err(format!("POLICY_BOUNDARY_DRIFT:{key}"));
        }
    }
    for key in [
        "prerelease_mappings_admitted_as_stable",
        "unknown_standard_release_admitted",
        "unknown_rights_admitted",
        "automatic_label_based_equivalence_allowed",
        "new_core_ontology_allowed",
    ] {
        if boolean(policy, key) != Some(false) {
            return Err(format!("POLICY_BOUNDARY_DRIFT:{key}"));
        }
    }
    Ok(())
}

fn validate_schema(schema: &Value) -> Result<(), String> {
    if text(schema, "type") != "object"
        || schema["additionalProperties"] != false
        || !exact_keys(&schema["properties"], PROFILE_ROOT_FIELDS)
    {
        return Err("SCHEMA_ROOT_CONTRACT_INVALID".to_owned());
    }
    if !string_array(schema, "required").is_some_and(|items| {
        items.len() == PROFILE_ROOT_FIELDS.len()
            && items.iter().zip(PROFILE_ROOT_FIELDS).all(|(item, field)| {
                item.as_str() == Some(*field)
            })
    }) {
        return Err("SCHEMA_REQUIRED_FIELDS_INVALID".to_owned());
    }
    if schema["$schema"] != "https://json-schema.org/draft/2020-12/schema"
        || schema["title"] != "Sol-Atlas Building Semantic Mapping Profile v0.1"
        || schema["properties"]["schema_version"]["const"]
            != "sol-atlas.building-semantic-profile/0.1"
        || schema["properties"]["profile_id"]["const"] != PROFILE_ID
        || schema["properties"]["classification"]["const"]
            != "synthetic_mapping_contract_only"
        || schema["properties"]["claim_ceiling"]["const"]
            != "synthetic_mapping_admission_and_semantic_loss_checks_only"
    {
        return Err("SCHEMA_IDENTITY_CONSTRAINT_DRIFT".to_owned());
    }

    let relations = &schema["properties"]["mapping_relations"];
    let expected_relations = [
        "exact", "narrower", "broader", "related", "composed", "derived", "unresolved",
    ];
    if text(relations, "type") != "array"
        || relations["minItems"] != expected_relations.len()
        || relations["maxItems"] != expected_relations.len()
        || !string_array(&relations["items"], "enum").is_some_and(|items| {
            items.len() == expected_relations.len()
                && items.iter().zip(expected_relations).all(|(item, expected)| {
                    item.as_str() == Some(expected)
                })
        })
    {
        return Err("SCHEMA_RELATION_VOCABULARY_INVALID".to_owned());
    }

    let policy = &schema["properties"]["policy"];
    if text(policy, "type") != "object"
        || policy["additionalProperties"] != false
        || !exact_keys(&policy["properties"], &[
            "exact_equivalence_requires_same_subject",
            "exact_equivalence_requires_same_quantity_kind",
            "exact_equivalence_requires_same_unit",
            "exact_equivalence_requires_preserved_time_basis",
            "exact_equivalence_requires_preserved_evidence_class",
            "prerelease_mappings_admitted_as_stable",
            "unknown_standard_release_admitted",
            "unknown_rights_admitted",
            "automatic_label_based_equivalence_allowed",
            "new_core_ontology_allowed",
        ])
    {
        return Err("SCHEMA_POLICY_CONTRACT_INVALID".to_owned());
    }

    let standard = &schema["$defs"]["standard"];
    if text(standard, "type") != "object"
        || standard["additionalProperties"] != false
        || !exact_keys(&standard["properties"], STANDARD_FIELDS)
    {
        return Err("SCHEMA_STANDARD_CONTRACT_INVALID".to_owned());
    }
    let standards = &schema["properties"]["standards"];
    if text(standards, "type") != "array"
        || standards["minItems"] != EXPECTED_STANDARDS.len()
        || standards["maxItems"] != EXPECTED_STANDARDS.len()
        || standards["items"]["$ref"] != "#/$defs/standard"
    {
        return Err("SCHEMA_STANDARDS_ARRAY_INVALID".to_owned());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    ExactEquivalent,
    AcceptedWithDeclaredLoss,
    MoreEvidenceRequired,
    NotComparable,
    Rejected,
}

impl Disposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExactEquivalent => "ExactEquivalent",
            Self::AcceptedWithDeclaredLoss => "AcceptedWithDeclaredLoss",
            Self::MoreEvidenceRequired => "MoreEvidenceRequired",
            Self::NotComparable => "NotComparable",
            Self::Rejected => "Rejected",
        }
    }
}

fn mapping_shape_valid(mapping: &Value) -> bool {
    exact_keys(mapping, MAPPING_FIELDS)
        && exact_keys(&mapping["source"], SOURCE_TARGET_FIELDS)
        && exact_keys(&mapping["target"], SOURCE_TARGET_FIELDS)
        && exact_keys(&mapping["assertion"], ASSERTION_FIELDS)
        && exact_keys(&mapping["semantics"], SEMANTIC_FIELDS)
        && exact_keys(&mapping["evidence"], EVIDENCE_FIELDS)
        && boolean(&mapping["assertion"], "exact_equivalence_claim").is_some()
        && boolean(&mapping["semantics"], "time_basis_preserved").is_some()
        && boolean(&mapping["semantics"], "evidence_class_preserved").is_some()
        && string_array(&mapping["assertion"], "declared_losses").is_some()
}

pub fn classify_mapping(profile: &Value, mapping: &Value) -> Disposition {
    if !mapping_shape_valid(mapping) {
        return Disposition::MoreEvidenceRequired;
    }

    let source = &mapping["source"];
    let target = &mapping["target"];
    let assertion = &mapping["assertion"];
    let semantics = &mapping["semantics"];
    let evidence = &mapping["evidence"];

    for side in [source, target] {
        let key = text(side, "standard_key");
        let release = text(side, "release_id");
        if standard_status(profile, key, release) != Some("stable") {
            return Disposition::MoreEvidenceRequired;
        }
        if text(side, "term_id").is_empty()
            || text(side, "subject_id").is_empty()
            || text(side, "revision").is_empty()
            || text(side, "revision_state") != "current"
        {
            return Disposition::MoreEvidenceRequired;
        }
    }

    let source_rights = text(evidence, "source_rights_state");
    let target_rights = text(evidence, "target_rights_state");
    if source_rights == "revoked" || target_rights == "revoked" {
        return Disposition::Rejected;
    }
    if source_rights != "permitted" || target_rights != "permitted" {
        return Disposition::MoreEvidenceRequired;
    }
    if text(evidence, "rule_id").is_empty()
        || text(evidence, "rule_revision").is_empty()
        || text(evidence, "rule_digest").is_empty()
        || text(evidence, "evidence_ref").is_empty()
    {
        return Disposition::MoreEvidenceRequired;
    }

    let relation = text(assertion, "relation");
    let exact_claim = boolean(assertion, "exact_equivalence_claim") == Some(true);
    let relationship = text(assertion, "subject_relationship");
    let losses = string_array(assertion, "declared_losses").unwrap_or(&[]);
    let source_quantity_kind = text(semantics, "source_quantity_kind");
    let target_quantity_kind = text(semantics, "target_quantity_kind");
    let source_unit = text(semantics, "source_unit");
    let target_unit = text(semantics, "target_unit");
    let time_preserved = boolean(semantics, "time_basis_preserved") == Some(true);
    let evidence_preserved = boolean(semantics, "evidence_class_preserved") == Some(true);

    if relation == "unresolved" {
        return Disposition::MoreEvidenceRequired;
    }
    if !["exact", "narrower", "broader", "related", "composed", "derived"].contains(&relation) {
        return Disposition::Rejected;
    }
    if source_quantity_kind.is_empty()
        || target_quantity_kind.is_empty()
        || source_unit.is_empty()
        || target_unit.is_empty()
    {
        return Disposition::MoreEvidenceRequired;
    }

    if exact_claim {
        if relationship == "unknown" || relationship.is_empty() {
            return Disposition::NotComparable;
        }
        if relation != "exact" || relationship != "same_subject" || !losses.is_empty() {
            return Disposition::Rejected;
        }
        if text(source, "subject_id") != text(target, "subject_id") {
            return Disposition::NotComparable;
        }
        if source_quantity_kind != target_quantity_kind
            || source_unit != target_unit
            || !time_preserved
            || !evidence_preserved
        {
            return Disposition::NotComparable;
        }
        if text(semantics, "conversion_state") != "none"
            || !text(semantics, "conversion_rule_id").is_empty()
        {
            return Disposition::NotComparable;
        }
        return Disposition::ExactEquivalent;
    }

    if relation == "exact" {
        return Disposition::MoreEvidenceRequired;
    }
    if relationship == "unknown" || relationship.is_empty() {
        return Disposition::NotComparable;
    }
    if losses.is_empty() || losses.iter().any(|loss| {
        loss.as_str().is_none_or(str::is_empty)
    }) {
        return Disposition::MoreEvidenceRequired;
    }
    if source_unit != target_unit {
        if text(semantics, "conversion_state") != "verified"
            || text(semantics, "conversion_rule_id").is_empty()
        {
            return Disposition::NotComparable;
        }
    } else if text(semantics, "conversion_state") != "none"
        && text(semantics, "conversion_state") != "verified"
    {
        return Disposition::MoreEvidenceRequired;
    }

    Disposition::AcceptedWithDeclaredLoss
}

pub fn validate_corpus(profile: &Value, corpus: &Value) -> Result<usize, String> {
    if !exact_keys(corpus, &[
        "schema_version", "corpus_id", "classification", "profile_id",
        "claim_ceiling", "cases",
    ])
        || text(corpus, "schema_version") != "sol-atlas.building-semantic-corpus/0.1"
        || text(corpus, "corpus_id") != CORPUS_ID
        || text(corpus, "classification") != "synthetic_semantic_mapping_fixtures_only"
        || text(corpus, "profile_id") != PROFILE_ID
        || text(corpus, "claim_ceiling") != "synthetic_mapping_admission_and_semantic_loss_checks_only"
    {
        return Err("CORPUS_IDENTITY_OR_ROOT_FIELDS_INVALID".to_owned());
    }
    let cases = corpus["cases"].as_array().ok_or("CASES_MISSING")?;
    if cases.len() != EXPECTED_CASES.len() {
        return Err("CASE_COUNT_INVALID".to_owned());
    }

    for (case, (expected_id, expected_disposition)) in
        cases.iter().zip(EXPECTED_CASES.iter())
    {
        if !exact_keys(case, CASE_FIELDS)
            || text(case, "case_id") != *expected_id
            || text(case, "description").len() < 20
            || text(case, "expected_disposition") != *expected_disposition
        {
            return Err(format!("CASE_ID_OR_EXPECTED_DISPOSITION_DRIFT:{expected_id}"));
        }
        let actual = classify_mapping(profile, &case["mapping"]).as_str();
        if actual != *expected_disposition {
            return Err(format!(
                "{expected_id}:EXPECTED_{expected_disposition},GOT_{actual}"
            ));
        }
    }
    Ok(cases.len())
}

pub fn validate_tracked() -> Result<usize, String> {
    let profile = parse_unique_json(PROFILE_JSON)?;
    let schema = parse_unique_json(SCHEMA_JSON)?;
    let corpus = parse_unique_json(CORPUS_JSON)?;
    validate_profile(&profile)?;
    validate_schema(&schema)?;
    validate_corpus(&profile, &corpus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracked_profile_and_forty_known_answers_validate() {
        assert_eq!(
            validate_tracked().expect("tracked mapping corpus validates"),
            40
        );
    }

    #[test]
    fn duplicate_root_json_key_is_rejected() {
        let changed = PROFILE_JSON.replacen(
            "\"profile_id\":",
            "\"profile_id\":\"CITY-BUILDING-SEMANTIC-PROFILE-V0.1\",\"profile_id\":",
            1,
        );
        assert!(parse_unique_json(&changed).is_err());
    }

    #[test]
    fn duplicate_nested_json_key_is_rejected() {
        let changed = PROFILE_JSON.replacen(
            "\"release_id\": \"1.4.4\"",
            "\"release_id\": \"1.4.4\", \"release_id\": \"1.4.3\"",
            1,
        );
        assert!(parse_unique_json(&changed).is_err());
    }

    #[test]
    fn profile_cannot_promote_an_unpinned_standard_release() {
        let changed = PROFILE_JSON.replace(
            "\"release_id\": \"VERSION_NOT_PINNED\"",
            "\"release_id\": \"latest\"",
        );
        let profile = parse_unique_json(&changed).unwrap();
        assert!(validate_profile(&profile).is_err());
    }

    #[test]
    fn schema_cannot_open_standard_objects_to_extra_fields() {
        let mut schema = parse_unique_json(SCHEMA_JSON).unwrap();
        schema["$defs"]["standard"]["additionalProperties"] = Value::Bool(true);
        assert!(validate_schema(&schema).is_err());
    }

    #[test]
    fn exact_equivalence_cannot_cross_units() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mut mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][0]["mapping"].clone();
        mapping["semantics"]["target_unit"] = Value::String("[degF]".to_owned());
        assert_eq!(
            classify_mapping(&profile, &mapping),
            Disposition::NotComparable
        );
    }

    #[test]
    fn exact_equivalence_cannot_cross_quantity_kinds() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mut mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][0]["mapping"].clone();
        mapping["semantics"]["target_quantity_kind"] =
            Value::String("mass_concentration".to_owned());
        assert_eq!(
            classify_mapping(&profile, &mapping),
            Disposition::NotComparable
        );
    }

    #[test]
    fn related_mapping_cannot_claim_exact_equivalence() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mut mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][1]["mapping"].clone();
        mapping["assertion"]["exact_equivalence_claim"] = Value::Bool(true);
        assert_eq!(classify_mapping(&profile, &mapping), Disposition::Rejected);
    }

    #[test]
    fn prerelease_mapping_is_not_promoted_to_stable() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][9]["mapping"].clone();
        assert_eq!(
            classify_mapping(&profile, &mapping),
            Disposition::MoreEvidenceRequired
        );
    }

    #[test]
    fn revoked_mapping_rights_are_rejected() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mut mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][0]["mapping"].clone();
        mapping["evidence"]["source_rights_state"] = Value::String("revoked".to_owned());
        assert_eq!(classify_mapping(&profile, &mapping), Disposition::Rejected);
    }

    #[test]
    fn missing_rule_evidence_does_not_admit_a_mapping() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mut mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][0]["mapping"].clone();
        mapping["evidence"]["evidence_ref"] = Value::String(String::new());
        assert_eq!(
            classify_mapping(&profile, &mapping),
            Disposition::MoreEvidenceRequired
        );
    }

    #[test]
    fn exact_mapping_requires_same_subject_identity() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mut mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][0]["mapping"].clone();
        mapping["target"]["subject_id"] = Value::String("asset-replacement".to_owned());
        assert_eq!(classify_mapping(&profile, &mapping), Disposition::NotComparable);
    }

    #[test]
    fn unresolved_standard_release_abstains() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][7]["mapping"].clone();
        assert_eq!(
            classify_mapping(&profile, &mapping),
            Disposition::MoreEvidenceRequired
        );
    }

    #[test]
    fn verified_non_exact_unit_conversion_requires_declared_loss() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mut mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][31]["mapping"].clone();
        mapping["assertion"]["declared_losses"] = Value::Array(Vec::new());
        assert_eq!(
            classify_mapping(&profile, &mapping),
            Disposition::MoreEvidenceRequired
        );
    }

    #[test]
    fn unknown_relation_is_rejected() {
        let profile = parse_unique_json(PROFILE_JSON).unwrap();
        let mut mapping = parse_unique_json(CORPUS_JSON).unwrap()["cases"][0]["mapping"].clone();
        mapping["assertion"]["relation"] = Value::String("looksSimilar".to_owned());
        assert_eq!(classify_mapping(&profile, &mapping), Disposition::Rejected);
    }
}
