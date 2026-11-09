//! Strict, offline reference evaluator for federated habitat evidence composition.
//!
//! The evaluator checks synthetic owner-reference and reliance mechanics only.
//! It does not authenticate source payload bytes or issuers, qualify engineering,
//! validate a physical article, or establish building/service performance.

use serde_json::{Value, json};
use std::collections::{BTreeSet, HashSet};

pub const CORPUS_ID: &str = "HIGHVELD-FEDERATED-TWIN-COMPOSITION-CORPUS-V0.1";
pub const CORPUS_SCHEMA_VERSION: &str = "0.1.0";
pub const CORPUS_CLASSIFICATION: &str = "synthetic_fixture_only";
pub const PROFILE_ID: &str = "HIGHVELD-FEDERATED-OWNER-PROFILES-V0.1";
pub const CLAIM_CEILING: &str = "synthetic_federated_composition_mechanics_only";

const ROOT_KEYS: &[&str] = &[
    "corpus_id",
    "schema_version",
    "classification",
    "purpose",
    "temporal_model",
    "composition_default",
    "claim_ceiling",
    "cases",
];
const CASE_KEYS: &[&str] = &["case_id", "name", "input", "expected"];
const INPUT_KEYS: &[&str] = &[
    "classification",
    "as_of",
    "decision",
    "required_owner_refs",
    "owner_inventory",
    "records",
    "identity_relations",
    "dependency_edges",
    "disclosure_request",
    "building_service_claim",
    "query_cuts",
];
const COMMON_RECORD_FIELDS: &[&str] = &[
    "record_id",
    "owner",
    "owner_native_id",
    "native_revision",
    "schema_id",
    "schema_version",
    "payload_ref",
    "payload_sha256",
    "payload_bytes_state",
    "issuer_state",
    "rights_state",
    "evidence_state",
    "scope_digest",
    "functional_unit",
    "valid_from",
    "valid_until",
    "observed_at",
    "recorded_at",
    "registered_at",
    "ingested_at",
    "source_timezone",
    "clock_id",
    "clock_uncertainty_ms",
    "verifier_id",
    "verifier_build",
    "claim_refs",
];
const EXPECTED_KEYS: &[&str] = &[
    "historical_reliance",
    "current_reliance",
    "automatic_identity_merges",
    "disclosure_state",
    "disclosed_fields",
    "reason_codes",
    "claim_ceiling",
    "retrospective_historical_assessment",
];

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn push_error(errors: &mut Vec<String>, path: &str, reason: &str) {
    errors.push(format!("{path}:{reason}"));
}

fn check_keys(
    value: &Value,
    path: &str,
    allowed: &[&str],
    required: &[&str],
    errors: &mut Vec<String>,
) {
    let Some(object) = value.as_object() else {
        push_error(errors, path, "OBJECT_REQUIRED");
        return;
    };
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            push_error(errors, path, &format!("UNKNOWN_FIELD:{key}"));
        }
    }
    for key in required {
        if !object.contains_key(*key) {
            push_error(errors, path, &format!("REQUIRED_FIELD_MISSING:{key}"));
        }
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn check_sha(value: &Value, key: &str, path: &str, errors: &mut Vec<String>) {
    let candidate = string(value, key);
    if !is_lower_hex(candidate, 64) {
        push_error(errors, path, &format!("INVALID_SHA256:{key}"));
    }
}

fn check_string_array(value: &Value, key: &str, path: &str, errors: &mut Vec<String>) {
    let Some(values) = value.get(key).and_then(Value::as_array) else {
        push_error(errors, path, &format!("STRING_ARRAY_REQUIRED:{key}"));
        return;
    };
    let mut seen = HashSet::new();
    for item in values {
        let Some(item) = item.as_str() else {
            push_error(errors, path, &format!("STRING_ARRAY_ITEM_REQUIRED:{key}"));
            continue;
        };
        if item.is_empty() || !seen.insert(item) {
            push_error(
                errors,
                path,
                &format!("EMPTY_OR_DUPLICATE_ARRAY_ITEM:{key}"),
            );
        }
    }
}

fn skip_json_whitespace(bytes: &[u8], cursor: &mut usize) {
    while bytes
        .get(*cursor)
        .is_some_and(|byte| matches!(*byte, 32 | 10 | 13 | 9))
    {
        *cursor += 1;
    }
}

fn scan_json_string_token<'a>(bytes: &'a [u8], cursor: &mut usize) -> Result<&'a str, String> {
    let start = *cursor;
    if bytes.get(*cursor).copied() != Some(34) {
        return Err("JSON_STRING_EXPECTED".to_owned());
    }
    *cursor += 1;
    while let Some(byte) = bytes.get(*cursor).copied() {
        match byte {
            34 => {
                *cursor += 1;
                return std::str::from_utf8(&bytes[start..*cursor])
                    .map_err(|_| "JSON_NOT_UTF8".to_owned());
            }
            92 => {
                *cursor += 1;
                if *cursor >= bytes.len() {
                    return Err("JSON_STRING_ESCAPE_TRUNCATED".to_owned());
                }
                *cursor += 1;
            }
            _ => *cursor += 1,
        }
    }
    Err("JSON_STRING_UNTERMINATED".to_owned())
}

/// Scan original bytes before serde_json creates maps; compare decoded key spellings.
fn scan_json_value(bytes: &[u8], cursor: &mut usize) -> Result<(), String> {
    skip_json_whitespace(bytes, cursor);
    match bytes.get(*cursor).copied() {
        Some(123) => {
            *cursor += 1;
            skip_json_whitespace(bytes, cursor);
            if bytes.get(*cursor).copied() == Some(125) {
                *cursor += 1;
                return Ok(());
            }
            let mut keys = HashSet::new();
            loop {
                skip_json_whitespace(bytes, cursor);
                let token = scan_json_string_token(bytes, cursor)?;
                let key: String = serde_json::from_str(token)
                    .map_err(|_| "JSON_OBJECT_KEY_INVALID".to_owned())?;
                if !keys.insert(key.clone()) {
                    return Err(format!("JSON_DUPLICATE_MEMBER_KEY:{key}"));
                }
                skip_json_whitespace(bytes, cursor);
                if bytes.get(*cursor).copied() != Some(58) {
                    return Err("JSON_OBJECT_COLON_EXPECTED".to_owned());
                }
                *cursor += 1;
                scan_json_value(bytes, cursor)?;
                skip_json_whitespace(bytes, cursor);
                match bytes.get(*cursor).copied() {
                    Some(44) => *cursor += 1,
                    Some(125) => {
                        *cursor += 1;
                        return Ok(());
                    }
                    _ => return Err("JSON_OBJECT_SEPARATOR_EXPECTED".to_owned()),
                }
            }
        }
        Some(91) => {
            *cursor += 1;
            skip_json_whitespace(bytes, cursor);
            if bytes.get(*cursor).copied() == Some(93) {
                *cursor += 1;
                return Ok(());
            }
            loop {
                scan_json_value(bytes, cursor)?;
                skip_json_whitespace(bytes, cursor);
                match bytes.get(*cursor).copied() {
                    Some(44) => *cursor += 1,
                    Some(93) => {
                        *cursor += 1;
                        return Ok(());
                    }
                    _ => return Err("JSON_ARRAY_SEPARATOR_EXPECTED".to_owned()),
                }
            }
        }
        Some(34) => {
            let token = scan_json_string_token(bytes, cursor)?;
            serde_json::from_str::<String>(token)
                .map(|_| ())
                .map_err(|_| "JSON_STRING_INVALID".to_owned())
        }
        Some(_) => {
            let start = *cursor;
            while bytes
                .get(*cursor)
                .is_some_and(|byte| !matches!(*byte, 32 | 10 | 13 | 9 | 44 | 93 | 125))
            {
                *cursor += 1;
            }
            if *cursor == start {
                Err("JSON_VALUE_EXPECTED".to_owned())
            } else {
                Ok(())
            }
        }
        None => Err("JSON_VALUE_EXPECTED".to_owned()),
    }
}

/// Reject duplicate decoded keys, malformed UTF-8, invalid JSON and trailing bytes.
pub fn parse_unique_json_bytes(bytes: &[u8]) -> Result<Value, String> {
    let input = std::str::from_utf8(bytes).map_err(|_| "JSON_NOT_UTF8".to_owned())?;
    let mut cursor = 0;
    scan_json_value(bytes, &mut cursor)?;
    skip_json_whitespace(bytes, &mut cursor);
    if cursor != bytes.len() {
        return Err("JSON_TRAILING_CONTENT".to_owned());
    }
    serde_json::from_str(input).map_err(|error| format!("JSON_PARSE_FAILED:{error}"))
}

fn profile_for<'a>(profiles: &'a Value, record: &Value) -> Option<&'a Value> {
    array(profiles, "profiles").iter().find(|profile| {
        string(profile, "owner") == string(record, "owner")
            && string(profile, "schema_id") == string(record, "schema_id")
    })
}

fn expected_owner_fields(owner: &str) -> Option<&'static [&'static str]> {
    match owner {
        "design" => Some(&["display_name", "design_revision"]),
        "geometry" => Some(&[
            "display_name",
            "geometry_digest",
            "geometry_verification",
            "coordinate_frame",
            "units",
            "ifc_schema",
            "ids_status",
            "adapter_state",
        ]),
        "solver" => Some(&[
            "solver_id",
            "solver_version",
            "solver_input_geometry_digest",
            "convergence_state",
            "validation_scope",
            "claim_ceiling",
            "input_bom_revision",
        ]),
        "product" => Some(&["product_id", "bom_revision", "approval_scope"]),
        "provenance" => Some(&[
            "lot_id",
            "lot_state",
            "custody_state",
            "inspection_state",
            "event_kind",
            "target_record_id",
        ]),
        "workcell" => Some(&[
            "work_package_id",
            "cell_id",
            "accepted_output_state",
            "metrology_state",
            "as_built_geometry_digest",
        ]),
        _ => None,
    }
}

pub fn validate_profiles(profiles: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    check_keys(
        profiles,
        "owner_profiles",
        &[
            "profile_id",
            "schema_version",
            "common_required",
            "profiles",
        ],
        &[
            "profile_id",
            "schema_version",
            "common_required",
            "profiles",
        ],
        &mut errors,
    );
    if string(profiles, "profile_id") != PROFILE_ID || string(profiles, "schema_version") != "0.1.0"
    {
        push_error(
            &mut errors,
            "owner_profiles",
            "PROFILE_ID_OR_VERSION_UNSUPPORTED",
        );
    }
    check_string_array(profiles, "common_required", "owner_profiles", &mut errors);
    let declared_common = array(profiles, "common_required")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    if declared_common.as_slice() != COMMON_RECORD_FIELDS {
        push_error(
            &mut errors,
            "owner_profiles",
            "COMMON_RECORD_ENVELOPE_DIFFERS_FROM_PINNED_CONTRACT",
        );
    }
    let mut identities = HashSet::new();
    for (index, profile) in array(profiles, "profiles").iter().enumerate() {
        let path = format!("owner_profiles.profiles[{index}]");
        check_keys(
            profile,
            &path,
            &["owner", "schema_id", "schema_version", "additional_allowed"],
            &["owner", "schema_id", "schema_version", "additional_allowed"],
            &mut errors,
        );
        for key in ["owner", "schema_id", "schema_version"] {
            if string(profile, key).is_empty() {
                push_error(&mut errors, &path, &format!("EMPTY_PROFILE_FIELD:{key}"));
            }
        }
        if string(profile, "schema_version") != "1.0.0"
            || !matches!(
                (string(profile, "owner"), string(profile, "schema_id")),
                ("design", "design.contract")
                    | ("geometry", "geometry.contract")
                    | ("solver", "solver.contract")
                    | ("product", "product.contract")
                    | ("provenance", "provenance.contract")
                    | ("workcell", "workcell.contract")
            )
        {
            push_error(&mut errors, &path, "OWNER_SCHEMA_PROFILE_UNSUPPORTED");
        }
        if !identities.insert((
            string(profile, "owner").to_owned(),
            string(profile, "schema_id").to_owned(),
            string(profile, "schema_version").to_owned(),
        )) {
            push_error(&mut errors, &path, "DUPLICATE_OWNER_SCHEMA_PROFILE");
        }
        check_string_array(profile, "additional_allowed", &path, &mut errors);
        let actual_fields = array(profile, "additional_allowed")
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        if expected_owner_fields(string(profile, "owner")) != Some(actual_fields.as_slice()) {
            push_error(
                &mut errors,
                &path,
                "OWNER_FIELD_ALLOWLIST_DIFFERS_FROM_PINNED_PROFILE",
            );
        }
        for field in array(profile, "additional_allowed")
            .iter()
            .filter_map(Value::as_str)
        {
            if COMMON_RECORD_FIELDS.contains(&field) {
                push_error(
                    &mut errors,
                    &path,
                    &format!("OWNER_FIELD_REPEATS_COMMON_FIELD:{field}"),
                );
            }
        }
    }
    if array(profiles, "profiles").len() != 6 {
        push_error(&mut errors, "owner_profiles", "UNEXPECTED_PROFILE_COUNT");
    }
    errors.sort();
    errors
}

fn validate_record(record: &Value, index: usize, profiles: &Value, errors: &mut Vec<String>) {
    let path = format!("input.records[{index}]");
    let Some(profile) = profile_for(profiles, record) else {
        push_error(errors, &path, "OWNER_SCHEMA_PROFILE_UNREGISTERED");
        return;
    };
    let mut allowed = COMMON_RECORD_FIELDS
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    for field in array(profile, "additional_allowed")
        .iter()
        .filter_map(Value::as_str)
    {
        allowed.push(field.to_owned());
    }
    let allowed_refs = allowed.iter().map(String::as_str).collect::<Vec<_>>();
    check_keys(record, &path, &allowed_refs, COMMON_RECORD_FIELDS, errors);
    for field in [
        "record_id",
        "owner",
        "owner_native_id",
        "native_revision",
        "schema_id",
        "schema_version",
        "payload_ref",
        "payload_bytes_state",
        "issuer_state",
        "rights_state",
        "evidence_state",
        "functional_unit",
        "source_timezone",
        "clock_id",
        "verifier_id",
    ] {
        if string(record, field).is_empty() {
            push_error(errors, &path, &format!("EMPTY_RECORD_FIELD:{field}"));
        }
    }
    check_sha(record, "payload_sha256", &path, errors);
    check_sha(record, "scope_digest", &path, errors);
    let verifier_build = string(record, "verifier_build");
    if !verifier_build
        .strip_prefix("sha256:")
        .is_some_and(|value| is_lower_hex(value, 64))
    {
        push_error(errors, &path, "INVALID_SHA256:verifier_build");
    }
    for field in [
        "valid_from",
        "observed_at",
        "recorded_at",
        "registered_at",
        "ingested_at",
    ] {
        if number(record, field).is_none() {
            push_error(
                errors,
                &path,
                &format!("UNSIGNED_TIMESTAMP_REQUIRED:{field}"),
            );
        }
    }
    if !record
        .get("valid_until")
        .is_some_and(|value| value.is_null() || value.as_u64().is_some())
    {
        push_error(errors, &path, "INVALID_VALID_UNTIL");
    }
    if record
        .get("clock_uncertainty_ms")
        .and_then(Value::as_u64)
        .is_none()
    {
        push_error(errors, &path, "INVALID_CLOCK_UNCERTAINTY");
    }
    check_string_array(record, "claim_refs", &path, errors);
}

fn validate_time_cut(value: &Value, path: &str, errors: &mut Vec<String>) {
    for key in ["valid_at", "recorded_at", "registered_at", "ingested_at"] {
        if number(value, key).is_none() {
            push_error(
                errors,
                path,
                &format!("UNSIGNED_TIME_CUTOFF_REQUIRED:{key}"),
            );
        }
    }
    if string(value, "time_basis") != "UTC_UNIX_SECONDS" {
        push_error(errors, path, "TIME_BASIS_UNSUPPORTED");
    }
}

pub fn validate_case(case: &Value, profiles: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    check_keys(case, "case", CASE_KEYS, CASE_KEYS, &mut errors);

    let input = case.get("input").unwrap_or(&Value::Null);
    check_keys(input, "input", INPUT_KEYS, INPUT_KEYS, &mut errors);
    for key in [
        "owner_inventory",
        "records",
        "identity_relations",
        "dependency_edges",
    ] {
        if !input.get(key).is_some_and(Value::is_array) {
            push_error(&mut errors, "input", &format!("ARRAY_REQUIRED:{key}"));
        }
    }

    let as_of_keys = [
        "valid_at",
        "recorded_at",
        "registered_at",
        "ingested_at",
        "time_basis",
    ];
    let decision_keys = [
        "decision_id",
        "decided_at",
        "purpose",
        "subject_id",
        "scope_digest",
        "functional_unit",
        "claim_id",
        "claim_type",
    ];
    check_keys(
        input.get("as_of").unwrap_or(&Value::Null),
        "input.as_of",
        &as_of_keys,
        &as_of_keys,
        &mut errors,
    );
    check_keys(
        input.get("decision").unwrap_or(&Value::Null),
        "input.decision",
        &decision_keys,
        &decision_keys,
        &mut errors,
    );

    for cut_name in ["as_known_at_decision", "as_known_now"] {
        let cut = input
            .get("query_cuts")
            .unwrap_or(&Value::Null)
            .get(cut_name)
            .unwrap_or(&Value::Null);
        check_keys(
            cut,
            &format!("input.query_cuts.{cut_name}"),
            &as_of_keys,
            &as_of_keys,
            &mut errors,
        );
        validate_time_cut(cut, &format!("input.query_cuts.{cut_name}"), &mut errors);
    }
    validate_time_cut(
        input.get("as_of").unwrap_or(&Value::Null),
        "input.as_of",
        &mut errors,
    );
    check_keys(
        input.get("query_cuts").unwrap_or(&Value::Null),
        "input.query_cuts",
        &["as_known_at_decision", "as_known_now"],
        &["as_known_at_decision", "as_known_now"],
        &mut errors,
    );
    check_keys(
        input.get("disclosure_request").unwrap_or(&Value::Null),
        "input.disclosure_request",
        &[
            "actor_role",
            "purpose",
            "requested_fields",
            "authorized_fields",
            "policy_state",
        ],
        &[
            "actor_role",
            "purpose",
            "requested_fields",
            "authorized_fields",
            "policy_state",
        ],
        &mut errors,
    );
    check_keys(
        input.get("building_service_claim").unwrap_or(&Value::Null),
        "input.building_service_claim",
        &[
            "requested",
            "required_owner_refs",
            "received_owner_refs",
            "commissioning_state",
            "service_state",
        ],
        &["requested", "required_owner_refs"],
        &mut errors,
    );
    let mut inventory_owners = HashSet::new();
    for (index, owner) in array(input, "owner_inventory").iter().enumerate() {
        check_keys(
            owner,
            &format!("input.owner_inventory[{index}]"),
            &["owner", "inventory_state", "snapshot_revision"],
            &["owner", "inventory_state", "snapshot_revision"],
            &mut errors,
        );
        let owner_name = string(owner, "owner");
        if owner_name.is_empty() || !inventory_owners.insert(owner_name.to_owned()) {
            push_error(
                &mut errors,
                &format!("input.owner_inventory[{index}]"),
                "EMPTY_OR_DUPLICATE_OWNER_INVENTORY",
            );
        }
    }
    let mut record_versions = HashSet::new();
    for (index, record) in array(input, "records").iter().enumerate() {
        validate_record(record, index, profiles, &mut errors);
        let key = (
            string(record, "owner").to_owned(),
            string(record, "owner_native_id").to_owned(),
            string(record, "record_id").to_owned(),
            string(record, "native_revision").to_owned(),
        );
        if !record_versions.insert(key) {
            push_error(
                &mut errors,
                &format!("input.records[{index}]"),
                "DUPLICATE_NATIVE_RECORD_REVISION",
            );
        }
    }
    for (index, relation) in array(input, "identity_relations").iter().enumerate() {
        check_keys(
            relation,
            &format!("input.identity_relations[{index}]"),
            &[
                "owner",
                "relation",
                "from_native_id",
                "to_native_id",
                "authority_state",
                "evidence_state",
                "automatic_merge",
            ],
            &[
                "owner",
                "relation",
                "from_native_id",
                "to_native_id",
                "authority_state",
                "evidence_state",
                "automatic_merge",
            ],
            &mut errors,
        );
        if relation
            .get("automatic_merge")
            .and_then(Value::as_bool)
            .is_none()
        {
            push_error(
                &mut errors,
                &format!("input.identity_relations[{index}]"),
                "AUTOMATIC_MERGE_BOOLEAN_REQUIRED",
            );
        }
        for key in [
            "owner",
            "relation",
            "from_native_id",
            "to_native_id",
            "authority_state",
            "evidence_state",
        ] {
            if string(relation, key).is_empty() {
                push_error(
                    &mut errors,
                    &format!("input.identity_relations[{index}]"),
                    &format!("IDENTITY_RELATION_FIELD_EMPTY:{key}"),
                );
            }
        }
    }
    let mut edge_set = HashSet::new();
    for (index, edge) in array(input, "dependency_edges").iter().enumerate() {
        check_keys(
            edge,
            &format!("input.dependency_edges[{index}]"),
            &["from", "to", "relation"],
            &["from", "to", "relation"],
            &mut errors,
        );
        let edge_key = (
            string(edge, "from").to_owned(),
            string(edge, "to").to_owned(),
            string(edge, "relation").to_owned(),
        );
        if !edge_set.insert(edge_key) {
            push_error(
                &mut errors,
                &format!("input.dependency_edges[{index}]"),
                "DUPLICATE_DEPENDENCY_EDGE",
            );
        }
        if string(edge, "from").is_empty()
            || string(edge, "to").is_empty()
            || string(edge, "relation").is_empty()
        {
            push_error(
                &mut errors,
                &format!("input.dependency_edges[{index}]"),
                "DEPENDENCY_EDGE_FIELD_EMPTY",
            );
        }
        // A missing endpoint is an evidence gap, not malformed input. Mandatory
        // owner coverage below decides whether that gap blocks reliance.
    }

    check_string_array(input, "required_owner_refs", "input", &mut errors);
    check_string_array(
        input.get("disclosure_request").unwrap_or(&Value::Null),
        "requested_fields",
        "input.disclosure_request",
        &mut errors,
    );
    check_string_array(
        input.get("disclosure_request").unwrap_or(&Value::Null),
        "authorized_fields",
        "input.disclosure_request",
        &mut errors,
    );
    check_string_array(
        input.get("building_service_claim").unwrap_or(&Value::Null),
        "required_owner_refs",
        "input.building_service_claim",
        &mut errors,
    );
    if input
        .get("building_service_claim")
        .and_then(|v| v.get("received_owner_refs"))
        .is_some()
    {
        check_string_array(
            input.get("building_service_claim").unwrap_or(&Value::Null),
            "received_owner_refs",
            "input.building_service_claim",
            &mut errors,
        );
    }

    let expected = case.get("expected").unwrap_or(&Value::Null);
    check_keys(
        expected,
        "expected",
        EXPECTED_KEYS,
        EXPECTED_KEYS,
        &mut errors,
    );
    check_string_array(expected, "disclosed_fields", "expected", &mut errors);
    check_string_array(expected, "reason_codes", "expected", &mut errors);
    if string(input, "classification") != CORPUS_CLASSIFICATION {
        push_error(&mut errors, "input", "CLASSIFICATION_UNSUPPORTED");
    }
    let decision = input.get("decision").unwrap_or(&Value::Null);
    for key in [
        "decision_id",
        "purpose",
        "subject_id",
        "functional_unit",
        "claim_id",
        "claim_type",
    ] {
        if string(decision, key).is_empty() {
            push_error(
                &mut errors,
                "input.decision",
                &format!("EMPTY_DECISION_FIELD:{key}"),
            );
        }
    }
    if number(decision, "decided_at").is_none() {
        push_error(&mut errors, "input.decision", "DECISION_TIME_REQUIRED");
    }
    check_sha(decision, "scope_digest", "input.decision", &mut errors);
    errors.sort();
    errors
}

pub fn validate_corpus(corpus: &Value, profiles: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    check_keys(corpus, "corpus", ROOT_KEYS, ROOT_KEYS, &mut errors);
    if string(corpus, "corpus_id") != CORPUS_ID
        || string(corpus, "schema_version") != CORPUS_SCHEMA_VERSION
        || string(corpus, "classification") != CORPUS_CLASSIFICATION
        || string(corpus, "claim_ceiling") != CLAIM_CEILING
        || string(corpus, "composition_default") != "federated"
        || corpus.get("temporal_model")
            != Some(&json!([
                "valid_at",
                "recorded_at",
                "registered_at",
                "ingested_at"
            ]))
    {
        push_error(
            &mut errors,
            "corpus",
            "IDENTITY_VERSION_CLASSIFICATION_OR_CLAIM_CEILING_INVALID",
        );
    }
    if array(corpus, "cases").len() != 18 {
        push_error(&mut errors, "corpus", "UNEXPECTED_CASE_COUNT");
    }
    let mut case_ids = HashSet::new();
    for (index, case) in array(corpus, "cases").iter().enumerate() {
        let case_id = string(case, "case_id");
        if case_id.is_empty() || !case_ids.insert(case_id.to_owned()) {
            push_error(
                &mut errors,
                &format!("cases[{index}]"),
                "EMPTY_OR_DUPLICATE_CASE_ID",
            );
        }
        errors.extend(validate_case(case, profiles));
    }
    errors.sort();
    errors
}

#[derive(Default)]
struct Assessment {
    disposition: String,
    reasons: BTreeSet<String>,
}

fn cut_visible(record: &Value, cut: &Value) -> bool {
    let (
        Some(valid_cut),
        Some(recorded_cut),
        Some(registered_cut),
        Some(ingested_cut),
        Some(valid_from),
        Some(recorded_at),
        Some(registered_at),
        Some(ingested_at),
    ) = (
        number(cut, "valid_at"),
        number(cut, "recorded_at"),
        number(cut, "registered_at"),
        number(cut, "ingested_at"),
        number(record, "valid_from"),
        number(record, "recorded_at"),
        number(record, "registered_at"),
        number(record, "ingested_at"),
    )
    else {
        return false;
    };
    let valid_until = record.get("valid_until").and_then(Value::as_u64);
    valid_from <= valid_cut
        && valid_until.is_none_or(|until| valid_cut < until)
        && recorded_at <= recorded_cut
        && registered_at <= registered_cut
        && ingested_at <= ingested_cut
}

fn push_reason(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn disposition(reasons: &BTreeSet<String>) -> String {
    if reasons.contains("FUNCTIONAL_UNIT_MISMATCH") || reasons.contains("SCOPE_MISMATCH") {
        "not_comparable".to_owned()
    } else if reasons.contains("ISSUER_AUTHORITY_INVALID")
        || reasons.contains("PURPOSE_RIGHTS_REVOKED")
        || reasons.contains("DISCLOSURE_POLICY_DENIED")
    {
        "not_authorized".to_owned()
    } else if reasons.contains("SOURCE_EVIDENCE_REVOKED") {
        "invalid_evidence".to_owned()
    } else if reasons.contains("DEPENDENCY_REVISION_CHANGED")
        || reasons.contains("SOLVER_INPUT_DIGEST_MISMATCH")
        || reasons.contains("LATE_BACKDATED_SOURCE_CORRECTION")
        || reasons.contains("POST_DECISION_SOURCE_CORRECTION")
    {
        "revalidation_required".to_owned()
    } else if !reasons.is_empty() {
        "insufficient_evidence".to_owned()
    } else {
        "reliable_for_declared_scope".to_owned()
    }
}

fn assess_at(input: &Value, cut: &Value, profiles: &Value, current: bool) -> Assessment {
    let mut assessment = Assessment::default();
    let decision = input.get("decision").unwrap_or(&Value::Null);
    let required = array(input, "required_owner_refs");
    let inventory = array(input, "owner_inventory");
    let records = array(input, "records");
    let current_cut = input
        .get("query_cuts")
        .and_then(|v| v.get("as_known_now"))
        .unwrap_or(cut);

    for owner_value in required {
        let owner = owner_value.as_str().unwrap_or("");
        let inventory_row = inventory.iter().find(|row| string(row, "owner") == owner);
        if inventory_row.is_none_or(|row| string(row, "inventory_state") != "complete") {
            push_reason(&mut assessment.reasons, "SOURCE_INVENTORY_INCOMPLETE");
        }

        let visible_base = records.iter().any(|record| {
            string(record, "owner") == owner
                && string(record, "event_kind").is_empty()
                && cut_visible(record, cut)
        });
        if !visible_base {
            let visible_later = records.iter().any(|record| {
                string(record, "owner") == owner
                    && string(record, "event_kind").is_empty()
                    && cut_visible(record, current_cut)
            });
            if !current && visible_later {
                push_reason(
                    &mut assessment.reasons,
                    "EVIDENCE_NOT_VISIBLE_AT_HISTORICAL_INGESTION_CUT",
                );
            } else {
                push_reason(&mut assessment.reasons, "MANDATORY_OWNER_EVIDENCE_MISSING");
            }
        }
    }

    for record in records
        .iter()
        .filter(|record| string(record, "event_kind").is_empty() && cut_visible(record, cut))
    {
        if string(record, "scope_digest") != string(decision, "scope_digest") {
            push_reason(&mut assessment.reasons, "SCOPE_MISMATCH");
        }
        if string(record, "functional_unit") != string(decision, "functional_unit") {
            push_reason(&mut assessment.reasons, "FUNCTIONAL_UNIT_MISMATCH");
        }
        if string(record, "payload_bytes_state") != "verified" {
            push_reason(&mut assessment.reasons, "PAYLOAD_BYTES_UNVERIFIED");
        }
        if string(record, "issuer_state") != "authorized" {
            push_reason(&mut assessment.reasons, "ISSUER_AUTHORITY_INVALID");
        }
        if string(record, "rights_state") != "permitted" {
            push_reason(&mut assessment.reasons, "PURPOSE_RIGHTS_REVOKED");
        }
        if string(record, "evidence_state") != "admitted" {
            push_reason(&mut assessment.reasons, "SOURCE_EVIDENCE_NOT_ADMITTED");
        }
        if let Some(profile) = profile_for(profiles, record) {
            if string(profile, "schema_version") != string(record, "schema_version")
                || string(record, "adapter_state") == "unsupported"
            {
                push_reason(&mut assessment.reasons, "SOURCE_SCHEMA_UNSUPPORTED");
            }
        } else {
            push_reason(&mut assessment.reasons, "SOURCE_SCHEMA_UNSUPPORTED");
        }
        if string(record, "geometry_verification") == "failed" {
            push_reason(&mut assessment.reasons, "GEOMETRY_VERIFICATION_FAILED");
        }
    }

    for event in records
        .iter()
        .filter(|record| !string(record, "event_kind").is_empty() && cut_visible(record, cut))
    {
        match string(event, "event_kind") {
            "revocation" => {
                let target = string(event, "target_record_id");
                if records.iter().any(|record| {
                    string(record, "record_id") == target
                        && string(record, "owner") == string(event, "owner")
                        && string(record, "event_kind").is_empty()
                }) {
                    push_reason(&mut assessment.reasons, "SOURCE_EVIDENCE_REVOKED");
                }
            }
            "correction" if current => {
                let decided_at = number(decision, "decided_at").unwrap_or(0);
                let valid_from = number(event, "valid_from").unwrap_or(u64::MAX);
                if valid_from <= decided_at {
                    push_reason(&mut assessment.reasons, "LATE_BACKDATED_SOURCE_CORRECTION");
                } else {
                    push_reason(&mut assessment.reasons, "POST_DECISION_SOURCE_CORRECTION");
                }
            }
            _ => {}
        }
    }

    if current {
        let geometry_digest = records
            .iter()
            .find(|record| {
                string(record, "owner") == "geometry"
                    && string(record, "event_kind").is_empty()
                    && cut_visible(record, cut)
            })
            .map(|record| string(record, "geometry_digest"))
            .unwrap_or("");
        let product_bom = records
            .iter()
            .find(|record| {
                string(record, "owner") == "product"
                    && string(record, "event_kind").is_empty()
                    && cut_visible(record, cut)
            })
            .map(|record| string(record, "bom_revision"))
            .unwrap_or("");
        let solver = records.iter().find(|record| {
            string(record, "owner") == "solver"
                && string(record, "event_kind").is_empty()
                && cut_visible(record, cut)
        });
        if let Some(solver) = solver {
            let solver_geometry = string(solver, "solver_input_geometry_digest");
            if !geometry_digest.is_empty()
                && !solver_geometry.is_empty()
                && solver_geometry != geometry_digest
            {
                push_reason(&mut assessment.reasons, "SOLVER_INPUT_DIGEST_MISMATCH");
            }
            let solver_bom = string(solver, "input_bom_revision");
            if !product_bom.is_empty() && !solver_bom.is_empty() && solver_bom != product_bom {
                push_reason(&mut assessment.reasons, "DEPENDENCY_REVISION_CHANGED");
            }
        }
    }

    for relation in array(input, "identity_relations") {
        if relation.get("automatic_merge").and_then(Value::as_bool) == Some(true) {
            push_reason(
                &mut assessment.reasons,
                "AUTOMATIC_IDENTITY_MERGE_PROHIBITED",
            );
        }
        if string(relation, "authority_state") != "authorized"
            || string(relation, "evidence_state") != "admitted"
        {
            push_reason(&mut assessment.reasons, "IDENTITY_RELATION_UNAUTHORIZED");
        }
    }

    let service = input.get("building_service_claim").unwrap_or(&Value::Null);
    if service.get("requested").and_then(Value::as_bool) == Some(true) {
        let received = array(service, "received_owner_refs");
        let commissioning_received = received
            .iter()
            .any(|v| v.as_str() == Some("building-commissioning"));
        let continuity_received = received
            .iter()
            .any(|v| v.as_str() == Some("continuity-owner"));
        if string(service, "commissioning_state") != "passed" || !commissioning_received {
            push_reason(
                &mut assessment.reasons,
                "BUILDING_COMMISSIONING_EVIDENCE_MISSING",
            );
        }
        if string(service, "service_state") != "adequate" || !continuity_received {
            push_reason(
                &mut assessment.reasons,
                "CONTINUITY_SERVICE_EVIDENCE_MISSING",
            );
        }
    }

    assessment.disposition = disposition(&assessment.reasons);
    assessment
}

fn disclosure_result(input: &Value) -> (String, Vec<String>) {
    let request = input.get("disclosure_request").unwrap_or(&Value::Null);
    let requested = array(request, "requested_fields");
    let authorized = array(request, "authorized_fields")
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let mut disclosed = requested
        .iter()
        .filter_map(Value::as_str)
        .filter(|field| authorized.contains(field))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    disclosed.sort();
    let fully_permitted =
        string(request, "policy_state") == "permitted" && disclosed.len() == requested.len();
    (
        if fully_permitted {
            "permitted"
        } else {
            "denied"
        }
        .to_owned(),
        disclosed,
    )
}

/// Evaluate owner-native evidence independently at historical and current query cuts.
/// The caller compares this result to the frozen expected answer; expected values are never inputs.
pub fn evaluate_case(input: &Value, profiles: &Value) -> Value {
    let cuts = input.get("query_cuts").unwrap_or(&Value::Null);
    let historical_cut = cuts.get("as_known_at_decision").unwrap_or(&Value::Null);
    let current_cut = cuts.get("as_known_now").unwrap_or(&Value::Null);
    let historical = assess_at(input, historical_cut, profiles, false);
    let current = assess_at(input, current_cut, profiles, true);

    let mut reasons = historical.reasons.clone();
    reasons.extend(current.reasons.iter().cloned());

    let decision_at =
        number(input.get("decision").unwrap_or(&Value::Null), "decided_at").unwrap_or(0);
    let backdated_correction = array(input, "records").iter().any(|record| {
        string(record, "event_kind") == "correction"
            && number(record, "valid_from").is_some_and(|valid_from| valid_from <= decision_at)
            && cut_visible(record, current_cut)
            && !cut_visible(record, historical_cut)
    });
    let (disclosure_state, disclosed_fields) = disclosure_result(input);
    let retrospective = if disclosure_state == "denied" {
        "not_authorized".to_owned()
    } else if backdated_correction {
        "historical_basis_incomplete".to_owned()
    } else {
        historical.disposition.clone()
    };

    if disclosure_state == "denied" {
        push_reason(&mut reasons, "DISCLOSURE_POLICY_DENIED");
    }
    let historical_disposition = if disclosure_state == "denied" {
        "not_authorized".to_owned()
    } else {
        historical.disposition
    };
    let current_disposition = if disclosure_state == "denied" {
        "not_authorized".to_owned()
    } else {
        current.disposition
    };

    let automatic_merges = array(input, "identity_relations")
        .iter()
        .filter(|relation| relation.get("automatic_merge").and_then(Value::as_bool) == Some(true))
        .count();

    json!({
        "historical_reliance": historical_disposition,
        "current_reliance": current_disposition,
        "automatic_identity_merges": automatic_merges,
        "disclosure_state": disclosure_state,
        "disclosed_fields": disclosed_fields,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "claim_ceiling": CLAIM_CEILING,
        "retrospective_historical_assessment": retrospective
    })
}

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    actual == expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const CORPUS: &str = include_str!(
        "../../../docs/research/highveld-keep/HIGHVELD_KEEP_FEDERATED_TWIN_COMPOSITION_CORPUS_V0_1.json"
    );
    const PROFILES: &str = include_str!(
        "../../../docs/research/highveld-keep/HIGHVELD_FEDERATED_COMPOSITION_OWNER_PROFILES_V0_1.json"
    );

    fn corpus() -> Value {
        parse_unique_json_bytes(CORPUS.as_bytes()).expect("frozen corpus JSON")
    }

    fn profiles() -> Value {
        let value = parse_unique_json_bytes(PROFILES.as_bytes()).expect("owner profile JSON");
        assert!(validate_profiles(&value).is_empty());
        value
    }

    fn case(id: &str) -> Value {
        corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| string(item, "case_id") == id)
            .expect("case exists")
            .clone()
    }

    #[test]
    fn all_frozen_known_answer_cases_match() {
        let profile = profiles();
        let data = corpus();
        assert_eq!(data["cases"].as_array().unwrap().len(), 18);
        for item in data["cases"].as_array().unwrap() {
            let errors = validate_case(item, &profile);
            assert!(errors.is_empty(), "{}: {errors:?}", string(item, "case_id"));
            let actual = evaluate_case(&item["input"], &profile);
            assert!(
                matches_expected(&actual, &item["expected"]),
                "{} actual={actual} expected={}",
                string(item, "case_id"),
                item["expected"]
            );
        }
    }

    #[test]
    fn duplicate_json_members_are_rejected_before_map_creation() {
        assert!(parse_unique_json_bytes(br#"{"claim":1,"claim":1}"#).is_err());
        assert!(parse_unique_json_bytes(br#"{"claim":1,"\u0063laim":2}"#).is_err());
    }

    #[test]
    fn malformed_utf8_and_trailing_content_are_rejected() {
        assert!(parse_unique_json_bytes(&[0xff, 0xfe]).is_err());
        assert!(parse_unique_json_bytes(br#"{"ok":true} trailing"#).is_err());
    }

    #[test]
    fn unknown_root_and_nested_record_fields_fail_closed() {
        let profile = profiles();
        let mut item = case("HFC-001");
        item["input"]["unreviewed"] = json!(true);
        assert!(!validate_case(&item, &profile).is_empty());

        let mut nested = case("HFC-001");
        nested["input"]["records"][0]["vendor_extension"] = json!("unknown");
        assert!(!validate_case(&nested, &profile).is_empty());
    }

    #[test]
    fn hfc_016_keeps_both_native_revisions_and_revocation_history() {
        let profile = profiles();
        let item = case("HFC-016");
        let versions = array(&item["input"], "records")
            .iter()
            .filter(|record| string(record, "record_id") == "R-LOT-REVOCATION")
            .count();
        assert_eq!(versions, 2);
        let actual = evaluate_case(&item["input"], &profile);
        assert_eq!(actual["historical_reliance"], "reliable_for_declared_scope");
        assert_eq!(actual["current_reliance"], "invalid_evidence");
    }

    #[test]
    fn ingestion_cut_prevents_future_knowledge_leakage() {
        let profile = profiles();
        let actual = evaluate_case(&case("HFC-012")["input"], &profile);
        assert_eq!(actual["historical_reliance"], "insufficient_evidence");
        assert_eq!(actual["current_reliance"], "reliable_for_declared_scope");
        assert_eq!(
            actual["reason_codes"][0],
            "EVIDENCE_NOT_VISIBLE_AT_HISTORICAL_INGESTION_CUT"
        );
    }

    #[test]
    fn backdated_correction_does_not_rewrite_as_known_history() {
        let profile = profiles();
        let actual = evaluate_case(&case("HFC-010")["input"], &profile);
        assert_eq!(actual["historical_reliance"], "reliable_for_declared_scope");
        assert_eq!(actual["current_reliance"], "revalidation_required");
        assert_eq!(
            actual["retrospective_historical_assessment"],
            "historical_basis_incomplete"
        );
    }

    #[test]
    fn distinct_native_geometry_ids_are_not_fuzzy_merged() {
        let profile = profiles();
        let actual = evaluate_case(&case("HFC-003")["input"], &profile);
        assert_eq!(actual["automatic_identity_merges"], 0);
    }

    #[test]
    fn selective_disclosure_returns_only_authorized_fields() {
        let profile = profiles();
        let actual = evaluate_case(&case("HFC-015")["input"], &profile);
        assert_eq!(actual["disclosure_state"], "denied");
        assert_eq!(actual["disclosed_fields"], json!(["asset_revision"]));
        assert_eq!(actual["current_reliance"], "not_authorized");
    }

    #[test]
    fn unknown_owner_schema_revision_is_not_interpreted_by_old_profile() {
        let profile = profiles();
        let actual = evaluate_case(&case("HFC-013")["input"], &profile);
        assert_eq!(actual["current_reliance"], "insufficient_evidence");
        assert_eq!(actual["reason_codes"], json!(["SOURCE_SCHEMA_UNSUPPORTED"]));
    }

    #[test]
    fn incomplete_owner_inventory_cannot_produce_reliable_reliance() {
        let profile = profiles();
        let actual = evaluate_case(&case("HFC-014")["input"], &profile);
        assert_eq!(actual["current_reliance"], "insufficient_evidence");
        assert_eq!(
            actual["reason_codes"],
            json!(["SOURCE_INVENTORY_INCOMPLETE"])
        );
    }

    #[test]
    fn accepted_panel_never_substitutes_for_building_service_evidence() {
        let profile = profiles();
        let actual = evaluate_case(&case("HFC-018")["input"], &profile);
        assert_eq!(actual["current_reliance"], "insufficient_evidence");
        assert_eq!(
            actual["reason_codes"],
            json!([
                "BUILDING_COMMISSIONING_EVIDENCE_MISSING",
                "CONTINUITY_SERVICE_EVIDENCE_MISSING"
            ])
        );
    }

    #[test]
    fn automatic_identity_merge_is_counted_and_rejected() {
        let profile = profiles();
        let mut item = case("HFC-004");
        item["input"]["identity_relations"][0]["automatic_merge"] = json!(true);
        let actual = evaluate_case(&item["input"], &profile);
        assert_eq!(actual["automatic_identity_merges"], 1);
        assert_eq!(actual["current_reliance"], "insufficient_evidence");
    }
}
