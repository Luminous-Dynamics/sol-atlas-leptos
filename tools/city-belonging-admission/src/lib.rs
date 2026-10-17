use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const REPORT_VERSION: &str = "city-belonging-admission-report/0.1";
const CLAIM_CEILING: &str = "synthetic admission-policy fixtures only; not HSDS conformance, service truth, safety, availability, or outcome evidence";

const HSDS_SELECTED_CONTRACT: &str = include_str!("../../../docs/research/fixtures/hsds-3.1-selected-contract-v0.1.json");
const LOCAL_ADMISSION_PROFILE: &str = include_str!("../../../docs/research/fixtures/city-belonging-admission-profile-v0.1.json");
const HSDS_SCHEMA_SOURCES: [(&str, &str); 9] = [
    ("organization", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/organization.json")),
    ("service", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/service.json")),
    ("location", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/location.json")),
    ("service_at_location", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/service_at_location.json")),
    ("schedule", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/schedule.json")),
    ("address", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/address.json")),
    ("service_area", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/service_area.json")),
    ("service_capacity", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/service_capacity.json")),
    ("unit", include_str!("../../../docs/research/fixtures/upstream/hsds-3.1/schema/unit.json")),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Disposition {
    CandidateOnly,
    ReviewRequired,
    ProviderConfirmed,
    StaleOrDisputed,
    ClosedOrWithdrawn,
}

impl Disposition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CandidateOnly => "CandidateOnly",
            Self::ReviewRequired => "ReviewRequired",
            Self::ProviderConfirmed => "ProviderConfirmed",
            Self::StaleOrDisputed => "StaleOrDisputed",
            Self::ClosedOrWithdrawn => "ClosedOrWithdrawn",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decision {
    pub disposition: Disposition,
    pub surface_as_current: bool,
    pub show_map_pin: bool,
    pub may_claim_free: bool,
    pub may_claim_accessible: bool,
    pub may_claim_live_capacity: bool,
}

pub fn verify_files(fixture_path: &str, manifest_path: &str) -> Result<Value, String> {
    let fixture_bytes = std::fs::read(fixture_path)
        .map_err(|e| format!("could not read fixture {}: {}", fixture_path, e))?;
    let manifest = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("could not read manifest {}: {}", manifest_path, e))?;
    verify_bytes(&fixture_bytes, &manifest)
}

pub fn verify_bytes(fixture_bytes: &[u8], manifest_text: &str) -> Result<Value, String> {
    let manifest: Value = serde_json::from_str(manifest_text)
        .map_err(|e| format!("manifest is not valid JSON: {}", e))?;
    if required_str(&manifest, "hash_algorithm", "manifest")? != "SHA-256" {
        return Err("manifest.hash_algorithm must be SHA-256".to_owned());
    }
    let expected_hash = required_str(&manifest, "fixture_sha256", "manifest")?;
    let actual_hash = sha256_hex(fixture_bytes);
    if expected_hash != actual_hash {
        return Err(format!(
            "fixture SHA-256 mismatch: manifest={}, actual={}",
            expected_hash, actual_hash
        ));
    }
    if !fixture_bytes.ends_with(b"\n") {
        return Err("fixture must end with a newline".to_owned());
    }

    validate_pinned_schema_contract(&manifest)?;

    validate_local_profile_manifest(&manifest)?;

    let as_of = required_str(&manifest, "policy_as_of_date", "manifest")?;
    if days_from_date(as_of).is_none() {
        return Err("manifest.policy_as_of_date must be a valid YYYY-MM-DD date".to_owned());
    }
    let max_age_days = manifest.get("provider_confirmation_max_age_days").and_then(Value::as_u64)
        .ok_or_else(|| "manifest.provider_confirmation_max_age_days must be an unsigned integer".to_owned())?;
    let declared_count = manifest.get("record_count").and_then(Value::as_u64)
        .ok_or_else(|| "manifest.record_count must be an unsigned integer".to_owned())? as usize;
    let mut seen = BTreeSet::new();
    let mut observed_counts = BTreeMap::<String, usize>::new();
    let mut receipts = Vec::new();

    for (index, raw) in fixture_bytes.split(|byte| *byte == b'\n').enumerate() {
        if raw.is_empty() {
            if index == fixture_bytes.split(|byte| *byte == b'\n').count() - 1 {
                continue;
            }
            return Err(format!("blank JSONL record at line {}", index + 1));
        }
        let record: Value = serde_json::from_slice(raw)
            .map_err(|e| format!("line {} is not valid JSON: {}", index + 1, e))?;
        let fixture_id = required_str(&record, "fixture_id", &format!("line {}", index + 1))?;
        if !seen.insert(fixture_id.to_owned()) {
            return Err(format!("duplicate fixture_id {}", fixture_id));
        }
        validate_hsds_resources(&record, &format!("line {} {}", index + 1, fixture_id))?;
        validate_admission_evidence(&record, &format!("line {} {}", index + 1, fixture_id))?;
        let hsds_expected = record.pointer("/expected/hsds_selected_contract_valid")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("{}.expected.hsds_selected_contract_valid must be true", fixture_id))?;
        if !hsds_expected {
            return Err(format!("{} expected HSDS selected-resource contract to be valid", fixture_id));
        }
        let expected = record.get("expected")
            .ok_or_else(|| format!("{} missing expected contract", fixture_id))?;
        let expected_disposition = required_str(expected, "disposition", fixture_id)?;
        let decision = evaluate_record(&record, as_of, max_age_days)?;
        let matches = decision.disposition.as_str() == expected_disposition;
        if !matches {
            return Err(format!(
                "{} disposition mismatch: expected={}, actual={}",
                fixture_id, expected_disposition, decision.disposition.as_str()
            ));
        }

        for (key, actual) in [
            ("surface_as_current", decision.surface_as_current),
            ("show_map_pin", decision.show_map_pin),
            ("may_claim_free", decision.may_claim_free),
            ("may_claim_accessible", decision.may_claim_accessible),
            ("may_claim_live_capacity", decision.may_claim_live_capacity),
        ] {
            let declared = expected.get(key).and_then(Value::as_bool)
                .ok_or_else(|| format!("{}.expected.{} must be a boolean", fixture_id, key))?;
            if declared != actual {
                return Err(format!(
                    "{} safety expectation mismatch for {}: expected={}, actual={}",
                    fixture_id, key, declared, actual
                ));
            }
        }

        *observed_counts.entry(decision.disposition.as_str().to_owned()).or_default() += 1;
        receipts.push(json!({
            "fixture_id": fixture_id,
            "expected_disposition": expected_disposition,
            "actual_disposition": decision.disposition.as_str(),
            "policy_contract_match": true,
            "receipt_scope": "SYNTHETIC_POLICY_FIXTURE_ONLY",
            "surface_as_current": decision.surface_as_current,
            "show_map_pin": decision.show_map_pin,
            "may_claim_free": decision.may_claim_free,
            "may_claim_accessible": decision.may_claim_accessible,
            "may_claim_live_capacity": decision.may_claim_live_capacity
        }));
    }

    if receipts.is_empty() {
        return Err("fixture contains no records".to_owned());
    }
    if receipts.len() != declared_count {
        return Err(format!(
            "manifest record_count mismatch: manifest={}, observed={}",
            declared_count, receipts.len()
        ));
    }
    let declared_dispositions = manifest.get("disposition_counts")
        .and_then(Value::as_object)
        .ok_or_else(|| "manifest.disposition_counts must be an object".to_owned())?;
    if declared_dispositions.len() != observed_counts.len() {
        return Err("manifest disposition_counts keys do not match observed dispositions".to_owned());
    }
    for (name, count) in &observed_counts {
        let declared = declared_dispositions.get(name).and_then(Value::as_u64)
            .ok_or_else(|| format!("manifest missing disposition count {}", name))? as usize;
        if declared != *count {
            return Err(format!(
                "manifest disposition count mismatch for {}: manifest={}, observed={}",
                name, declared, count
            ));
        }
    }

    Ok(json!({
        "report_version": REPORT_VERSION,
        "status": "SYNTHETIC_POLICY_FIXTURE_MATCH",
        "claim_ceiling": CLAIM_CEILING,
        "fixture_id": manifest.get("fixture_id").and_then(Value::as_str).unwrap_or("unknown"),
        "fixture_sha256": actual_hash,
        "policy_as_of_date": as_of,
        "provider_confirmation_max_age_days": max_age_days,
        "record_count": receipts.len(),
        "disposition_counts": observed_counts,
        "all_cases_match_expected_policy": true,
        "real_world_service_truth": "NOT_ASSESSED",
        "pinned_hsds_selected_resource_contract": "MATCH",
        "local_admission_profile_sha256": sha256_hex(LOCAL_ADMISSION_PROFILE.as_bytes()),
        "local_admission_profile_version": "city-belonging-admission-profile/0.1",
        "canonical_hsds_full_conformance": "NOT_ASSERTED",
        "human_efficacy_or_social_connection_outcomes": "NOT_ASSESSED",
        "case_receipts": receipts
    }))
}


fn validate_pinned_schema_contract(fixture_manifest: &Value) -> Result<(), String> {
    let declared_pin_hash = fixture_manifest
        .pointer("/schema_baseline/schema_pin_manifest_sha256_utf8")
        .and_then(Value::as_str)
        .ok_or_else(|| "manifest missing pinned HSDS contract-manifest SHA-256".to_owned())?;
    let actual_pin_hash = sha256_hex(HSDS_SELECTED_CONTRACT.as_bytes());
    if declared_pin_hash != actual_pin_hash {
        return Err(format!(
            "pinned HSDS contract-manifest SHA-256 mismatch: manifest={}, actual={}",
            declared_pin_hash, actual_pin_hash
        ));
    }
    let pin: Value = serde_json::from_str(HSDS_SELECTED_CONTRACT)
        .map_err(|e| format!("pinned HSDS contract manifest is malformed: {}", e))?;
    if required_str(&pin, "manifest_version", "HSDS pin")? != "hsds-3.1-selected-contract/0.1" {
        return Err("unsupported pinned HSDS contract version".to_owned());
    }
    if required_str(&pin, "upstream_ref", "HSDS pin")? != "3.1" {
        return Err("pinned HSDS source ref must be 3.1".to_owned());
    }
    let entries = pin.get("schema_files").and_then(Value::as_array)
        .ok_or_else(|| "HSDS pin schema_files must be an array".to_owned())?;
    if entries.len() != HSDS_SCHEMA_SOURCES.len() {
        return Err("HSDS pin schema list does not match vendored source count".to_owned());
    }

    for (resource, source_text) in HSDS_SCHEMA_SOURCES {
        let entry = entries.iter().find(|entry| entry.get("resource").and_then(Value::as_str) == Some(resource))
            .ok_or_else(|| format!("HSDS pin has no contract for resource {}", resource))?;
        let expected_hash = required_str(entry, "sha256_utf8", resource)?;
        let actual_hash = sha256_hex(source_text.as_bytes());
        if expected_hash != actual_hash {
            return Err(format!("vendored HSDS {} schema SHA-256 mismatch", resource));
        }
        let expected_bytes = entry.get("utf8_byte_length").and_then(Value::as_u64)
            .ok_or_else(|| format!("HSDS pin {} utf8_byte_length missing", resource))? as usize;
        if expected_bytes != source_text.as_bytes().len() {
            return Err(format!("vendored HSDS {} schema byte length mismatch", resource));
        }
        let upstream: Value = serde_json::from_str(source_text)
            .map_err(|e| format!("vendored HSDS {} schema is malformed: {}", resource, e))?;
        let properties = upstream.get("properties").and_then(Value::as_object)
            .ok_or_else(|| format!("upstream HSDS {} schema properties missing", resource))?;
        let actual_fields: BTreeSet<String> = properties.keys().cloned().collect();
        let pinned_fields = string_set(entry.get("allowed_fields"), &format!("{}.allowed_fields", resource))?;
        if actual_fields != pinned_fields {
            return Err(format!("pinned HSDS {} allowed-field contract drift", resource));
        }
        for (schema_key, pin_key) in [("required", "required_fields"), ("tabular_required", "tabular_required_fields")] {
            let actual = upstream.get(schema_key)
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned).collect::<BTreeSet<_>>())
                .unwrap_or_default();
            let expected = string_set(entry.get(pin_key), &format!("{}.{}", resource, pin_key))?;
            if actual != expected {
                return Err(format!("pinned HSDS {} {} contract drift", resource, pin_key));
            }
        }
        for (property_key, pin_key) in [("type", "field_types"), ("format", "field_formats"), ("enum", "field_enums")] {
            let mut actual = serde_json::Map::new();
            for (field_name, property) in properties {
                if let Some(value) = property.get(property_key) {
                    let include = match property_key {
                        "type" | "format" => value.as_str().is_some(),
                        "enum" => value.as_array().is_some(),
                        _ => false,
                    };
                    if include { actual.insert(field_name.clone(), value.clone()); }
                }
            }
            let expected = entry.get(pin_key)
                .ok_or_else(|| format!("HSDS pin {}.{} missing", resource, pin_key))?;
            if &Value::Object(actual) != expected {
                return Err(format!("pinned HSDS {} {} contract drift", resource, pin_key));
            }
        }
    }
    Ok(())
}

fn string_set(value: Option<&Value>, where_: &str) -> Result<BTreeSet<String>, String> {
    let array = value.and_then(Value::as_array)
        .ok_or_else(|| format!("{} must be a string array", where_))?;
    array.iter().map(|item| item.as_str().map(str::to_owned)
        .ok_or_else(|| format!("{} entries must be strings", where_))).collect()
}


fn validate_local_profile_manifest(fixture_manifest: &Value) -> Result<(), String> {
    let declared_hash = fixture_manifest.pointer("/local_profile/sha256_utf8")
        .and_then(Value::as_str)
        .ok_or_else(|| "manifest missing local admission profile SHA-256".to_owned())?;
    let actual_hash = sha256_hex(LOCAL_ADMISSION_PROFILE.as_bytes());
    if declared_hash != actual_hash {
        return Err(format!("local admission profile SHA-256 mismatch: manifest={}, actual={}", declared_hash, actual_hash));
    }
    let declared_path = required_str(
        fixture_manifest.get("local_profile").ok_or_else(|| "manifest.local_profile missing".to_owned())?,
        "path",
        "manifest.local_profile",
    )?;
    if declared_path != "docs/research/fixtures/city-belonging-admission-profile-v0.1.json" {
        return Err("manifest local admission profile path is unexpected".to_owned());
    }
    let profile: Value = serde_json::from_str(LOCAL_ADMISSION_PROFILE)
        .map_err(|e| format!("local admission profile is malformed: {}", e))?;
    if required_str(&profile, "profile_version", "local admission profile")? != "city-belonging-admission-profile/0.1" {
        return Err("unsupported local admission profile version".to_owned());
    }
    Ok(())
}

fn validate_admission_evidence(record: &Value, where_: &str) -> Result<(), String> {
    let profile: Value = serde_json::from_str(LOCAL_ADMISSION_PROFILE)
        .map_err(|e| format!("local admission profile is malformed: {}", e))?;
    let contract = profile.get("evidence_contract")
        .ok_or_else(|| "local admission profile missing evidence_contract".to_owned())?;
    let evidence = record.get("admission_evidence")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{} admission_evidence must be an object", where_))?;
    let required = string_set(contract.get("required_fields"), "local profile required_fields")?;
    for key in &required {
        if !evidence.contains_key(key) {
            return Err(format!("{} admission_evidence missing required field {}", where_, key));
        }
    }
    for field in ["source_id", "source_uri", "upstream_record_id", "retrieved_at"] {
        let value = required_map_str(evidence, field, where_)?;
        if field == "source_uri" && !value.contains("://") {
            return Err(format!("{} source_uri must be URI-like", where_));
        }
        if field == "retrieved_at" && !valid_timestamp(value) {
            return Err(format!("{} retrieved_at must be a valid timestamp", where_));
        }
    }

    let source_kind = required_map_str(evidence, "source_kind", where_)?;
    if !string_set(contract.get("source_kind_enum"), "local profile source_kind_enum")?.contains(source_kind) {
        return Err(format!("{} unknown source_kind {}", where_, source_kind));
    }
    let permission = required_map_str(evidence, "permission_disposition", where_)?;
    if !string_set(contract.get("permission_disposition_enum"), "local profile permission_disposition_enum")?.contains(permission) {
        return Err(format!("{} unknown permission_disposition {}", where_, permission));
    }

    let verified_at = evidence.get("provider_verified_at").expect("required field checked above");
    if !verified_at.is_null() {
        let timestamp = verified_at.as_str()
            .ok_or_else(|| format!("{} provider_verified_at must be a timestamp string or null", where_))?;
        if !valid_timestamp(timestamp) {
            return Err(format!("{} provider_verified_at must be a valid timestamp", where_));
        }
    }

    let profile_states = contract.pointer("/field_state_enums").and_then(Value::as_object)
        .ok_or_else(|| "local profile field_state_enums must be an object".to_owned())?;
    let states = evidence.get("field_states").and_then(Value::as_object)
        .ok_or_else(|| format!("{} field_states must be an object", where_))?;
    for (field, observed) in states {
        let allowed = profile_states.get(field).and_then(Value::as_array)
            .ok_or_else(|| format!("{} undeclared field-state dimension {}", where_, field))?;
        let observed = observed.as_str().ok_or_else(|| format!("{} field state {} must be a string", where_, field))?;
        if !allowed.iter().any(|value| value.as_str() == Some(observed)) {
            return Err(format!("{} invalid field state {}={}", where_, field, observed));
        }
    }

    let pin: Value = serde_json::from_str(HSDS_SELECTED_CONTRACT)
        .map_err(|e| format!("HSDS contract manifest is malformed: {}", e))?;
    let verified_fields = evidence.get("provider_verified_fields").and_then(Value::as_array)
        .ok_or_else(|| format!("{} provider_verified_fields must be an array", where_))?;
    let mut verified_seen = BTreeSet::new();
    let hsds_rows = record.get("hsds_resources").and_then(Value::as_object)
        .ok_or_else(|| format!("{} missing hsds_resources", where_))?;
    for item in verified_fields {
        let path = item.as_str().ok_or_else(|| format!("{} provider_verified_fields entries must be strings", where_))?;
        if !verified_seen.insert(path.to_owned()) {
            return Err(format!("{} duplicate provider_verified_field {}", where_, path));
        }
        let (resource, field) = path.split_once('.')
            .ok_or_else(|| format!("{} provider_verified_field must be resource.field: {}", where_, path))?;
        let schema = pin.get("schema_files").and_then(Value::as_array)
            .and_then(|schemas| schemas.iter().find(|schema| schema.get("resource").and_then(Value::as_str) == Some(resource)))
            .ok_or_else(|| format!("{} provider_verified_field has unknown resource {}", where_, resource))?;
        let allowed = string_set(schema.get("allowed_fields"), &format!("{}.allowed_fields", resource))?;
        if !allowed.contains(field) {
            return Err(format!("{} provider_verified_field {} is not a field in the pinned schema", where_, path));
        }
        let row = hsds_rows.get(resource).and_then(Value::as_object)
            .ok_or_else(|| format!("{} provider_verified_field {} has no resource row", where_, path))?;
        if !row.contains_key(field) {
            return Err(format!("{} provider_verified_field {} is not present in the resource row", where_, path));
        }
    }

    let checks = evidence.get("contact_checks").and_then(Value::as_array)
        .ok_or_else(|| format!("{} contact_checks must be an array", where_))?;
    let allowed_contact_fields = string_set(contract.pointer("/contact_check/allowed_fields"), "local profile contact allowed_fields")?;
    let allowed_contact_kinds = string_set(contract.pointer("/contact_check/kind_enum"), "local profile contact kind_enum")?;
    let allowed_contact_results = string_set(contract.pointer("/contact_check/result_enum"), "local profile contact result_enum")?;
    for (index, check) in checks.iter().enumerate() {
        let object = check.as_object().ok_or_else(|| format!("{} contact check {} must be an object", where_, index))?;
        for key in object.keys() {
            if !allowed_contact_fields.contains(key) {
                return Err(format!("{} contact check {} has undeclared field {}", where_, index, key));
            }
        }
        for key in &allowed_contact_fields {
            if !object.contains_key(key) {
                return Err(format!("{} contact check {} missing field {}", where_, index, key));
            }
        }
        let kind = required_str(check, "kind", where_)?;
        let result = required_str(check, "result", where_)?;
        let _value = required_str(check, "value", where_)?;
        if !allowed_contact_kinds.contains(kind) {
            return Err(format!("{} unknown contact-check kind {}", where_, kind));
        }
        if !allowed_contact_results.contains(result) {
            return Err(format!("{} unknown contact-check result {}", where_, result));
        }
    }

    if let Some(conflicts) = evidence.get("conflicts") {
        let list = conflicts.as_array().ok_or_else(|| format!("{} conflicts must be an array", where_))?;
        for (index, conflict) in list.iter().enumerate() {
            for key in ["field", "claim_a", "claim_b"] {
                let _ = required_str(conflict, key, &format!("{} conflict {}", where_, index))?;
            }
        }
    }
    if let Some(correction) = evidence.get("user_correction") {
        for key in ["field", "proposed_value", "review_state"] {
            let _ = required_str(correction, key, &format!("{} user_correction", where_))?;
        }
        let allowed = string_set(contract.pointer("/optional_evidence_fields/user_correction/review_state_enum"), "local profile correction state enum")?;
        let state = required_str(correction, "review_state", where_)?;
        if !allowed.contains(state) {
            return Err(format!("{} unknown user correction review state {}", where_, state));
        }
    }
    if let Some(capacity) = evidence.get("capacity_claim") {
        for key in ["units", "available"] {
            if !capacity.get(key).map(Value::is_number).unwrap_or(false) {
                return Err(format!("{} capacity_claim.{} must be numeric", where_, key));
            }
        }
        let observed_at = required_str(capacity, "observed_at", where_)?;
        if !valid_timestamp(observed_at) {
            return Err(format!("{} capacity_claim.observed_at must be a timestamp", where_));
        }
        let authority = required_str(capacity, "authority", where_)?;
        let allowed = string_set(contract.pointer("/optional_evidence_fields/capacity_claim/authority_enum"), "local profile capacity authority enum")?;
        if !allowed.contains(authority) {
            return Err(format!("{} unknown capacity authority {}", where_, authority));
        }
    }
    Ok(())
}

fn valid_timestamp(value: &str) -> bool {
    value.len() >= 20
        && value.as_bytes().get(10) == Some(&b'T')
        && value.get(..10).map(days_from_date).flatten().is_some()
        && (value.ends_with('Z') || value.rfind('+').map(|at| at > 10).unwrap_or(false) || value.rfind('-').map(|at| at > 10).unwrap_or(false))
}

fn validate_hsds_resources(record: &Value, where_: &str) -> Result<(), String> {
    let pin: Value = serde_json::from_str(HSDS_SELECTED_CONTRACT)
        .map_err(|e| format!("HSDS contract manifest is malformed: {}", e))?;
    let resources = record.get("hsds_resources").and_then(Value::as_object)
        .ok_or_else(|| format!("{} must contain hsds_resources object", where_))?;
    for required in ["organization", "service", "location", "service_at_location"] {
        if !resources.contains_key(required) {
            return Err(format!("{} missing HSDS resource row {}", where_, required));
        }
    }
    let known: BTreeSet<&str> = ["organization", "service", "location", "service_at_location", "schedule", "address", "service_area", "service_capacity", "unit"].into_iter().collect();
    for key in resources.keys() {
        if !known.contains(key.as_str()) {
            return Err(format!("{} contains undeclared HSDS resource {}", where_, key));
        }
        let row = resources.get(key).expect("resource key was just enumerated");
        validate_hsds_resource_row(&pin, key, row, where_)?;
    }

    let org_id = required_str(resources.get("organization").unwrap(), "id", where_)?;
    let service = resources.get("service").unwrap();
    let service_id = required_str(service, "id", where_)?;
    if required_str(service, "organization_id", where_)? != org_id {
        return Err(format!("{} service.organization_id does not link to organization.id", where_));
    }
    let location = resources.get("location").unwrap();
    let location_id = required_str(location, "id", where_)?;
    if required_str(location, "organization_id", where_)? != org_id {
        return Err(format!("{} location.organization_id does not link to organization.id", where_));
    }
    let service_at_location = resources.get("service_at_location").unwrap();
    if required_str(service_at_location, "service_id", where_)? != service_id {
        return Err(format!("{} service_at_location.service_id does not link to service.id", where_));
    }
    if required_str(service_at_location, "location_id", where_)? != location_id {
        return Err(format!("{} service_at_location.location_id does not link to location.id", where_));
    }
    if let Some(row) = resources.get("schedule") {
        if row.get("service_id").and_then(Value::as_str).map(|id| id != service_id).unwrap_or(false) {
            return Err(format!("{} schedule.service_id does not link to service.id", where_));
        }
    }
    if let Some(row) = resources.get("address") {
        if row.get("location_id").and_then(Value::as_str).map(|id| id != location_id).unwrap_or(false) {
            return Err(format!("{} address.location_id does not link to location.id", where_));
        }
        if location.get("location_type").and_then(Value::as_str) == Some("virtual") {
            return Err(format!("{} virtual location must not carry a physical address row", where_));
        }
    }
    if let Some(row) = resources.get("service_area") {
        if row.get("service_id").and_then(Value::as_str).map(|id| id != service_id).unwrap_or(false) {
            return Err(format!("{} service_area.service_id does not link to service.id", where_));
        }
    }
    if let Some(row) = resources.get("service_capacity") {
        if row.get("service_id").and_then(Value::as_str).map(|id| id != service_id).unwrap_or(false) {
            return Err(format!("{} service_capacity.service_id does not link to service.id", where_));
        }
        let unit = resources.get("unit")
            .ok_or_else(|| format!("{} service_capacity row requires a linked unit row", where_))?;
        if row.get("unit_id").and_then(Value::as_str) != unit.get("id").and_then(Value::as_str) {
            return Err(format!("{} service_capacity.unit_id does not link to unit.id", where_));
        }
    }
    Ok(())
}

fn validate_hsds_resource_row(pin: &Value, resource: &str, row: &Value, where_: &str) -> Result<(), String> {
    let entries = pin.get("schema_files").and_then(Value::as_array)
        .ok_or_else(|| "HSDS pin schema_files must be an array".to_owned())?;
    let schema = entries.iter().find(|entry| entry.get("resource").and_then(Value::as_str) == Some(resource))
        .ok_or_else(|| format!("no pinned HSDS resource schema for {}", resource))?;
    let object = row.as_object().ok_or_else(|| format!("{}.{} row must be an object", where_, resource))?;
    let allowed = string_set(schema.get("allowed_fields"), &format!("{}.allowed_fields", resource))?;
    let mut required = string_set(schema.get("required_fields"), &format!("{}.required_fields", resource))?;
    required.extend(string_set(schema.get("tabular_required_fields"), &format!("{}.tabular_required_fields", resource))?);
    for key in object.keys() {
        if !allowed.contains(key) {
            return Err(format!("{}.{} has field not in pinned HSDS 3.1 contract: {}", where_, resource, key));
        }
    }
    for key in &required {
        let value = object.get(key).ok_or_else(|| format!("{}.{} missing required field {}", where_, resource, key))?;
        if value.is_null() {
            return Err(format!("{}.{} required field {} is null", where_, resource, key));
        }
        if let Some(text) = value.as_str() {
            if text.trim().is_empty() {
                return Err(format!("{}.{} required string {} is blank", where_, resource, key));
            }
        }
    }
    let types = schema.get("field_types").and_then(Value::as_object)
        .ok_or_else(|| format!("HSDS pin {} field_types missing", resource))?;
    let formats = schema.get("field_formats").and_then(Value::as_object)
        .ok_or_else(|| format!("HSDS pin {} field_formats missing", resource))?;
    let enums = schema.get("field_enums").and_then(Value::as_object)
        .ok_or_else(|| format!("HSDS pin {} field_enums missing", resource))?;
    for (key, value) in object {
        if value.is_null() {
            return Err(format!("{}.{} field {} must be omitted rather than null", where_, resource, key));
        }
        if let Some(field_type) = types.get(key).and_then(Value::as_str) {
            let valid = match field_type {
                "string" => value.is_string(),
                "number" => value.is_number(),
                "array" => value.is_array(),
                "object" => value.is_object(),
                _ => true,
            };
            if !valid {
                return Err(format!("{}.{} field {} violates pinned type {}", where_, resource, key, field_type));
            }
        }
        if let Some(values) = enums.get(key).and_then(Value::as_array) {
            let observed = value.as_str().ok_or_else(|| format!("{}.{} enum field {} must be a string", where_, resource, key))?;
            if !values.iter().any(|candidate| candidate.as_str() == Some(observed)) {
                return Err(format!("{}.{} field {} violates pinned enum: {}", where_, resource, key, observed));
            }
        }
        if let Some(format) = formats.get(key).and_then(Value::as_str) {
            let observed = value.as_str().ok_or_else(|| format!("{}.{} formatted field {} must be a string", where_, resource, key))?;
            let valid = match format {
                "uuid" => valid_uuid(observed),
                "uri" => observed.contains("://"),
                "email" => observed.contains('@') && !observed.starts_with('@') && !observed.ends_with('@'),
                "date" => days_from_date(observed).is_some(),
                "date-time" => observed.len() >= 20 && observed.as_bytes().get(10) == Some(&b'T') && days_from_date(&observed[..10]).is_some() && (observed.ends_with('Z') || observed.rfind('+').map(|at| at > 10).unwrap_or(false) || observed.rfind('-').map(|at| at > 10).unwrap_or(false)),
                _ => true,
            };
            if !valid {
                return Err(format!("{}.{} field {} violates pinned format {}", where_, resource, key, format));
            }
        }
    }
    Ok(())
}

fn valid_uuid(value: &str) -> bool {
    if value.len() != 36 { return false; }
    for (index, byte) in value.bytes().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if byte != b'-' { return false; }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

pub fn evaluate_record(record: &Value, as_of: &str, max_age_days: u64) -> Result<Decision, String> {
    let evidence = record.get("admission_evidence")
        .ok_or_else(|| "record missing admission_evidence".to_owned())?;
    let fields = evidence.get("field_states")
        .and_then(Value::as_object)
        .ok_or_else(|| "admission_evidence.field_states must be an object".to_owned())?;
    let permission = required_str(evidence, "permission_disposition", "admission_evidence")?;
    let source_kind = required_str(evidence, "source_kind", "admission_evidence")?;
    let verified_at = evidence.get("provider_verified_at");
    let verified_fields = evidence.get("provider_verified_fields")
        .and_then(Value::as_array)
        .ok_or_else(|| "provider_verified_fields must be an array".to_owned())?;
    let record_status = record.pointer("/hsds_resources/service/status").and_then(Value::as_str).unwrap_or("unknown");

    // Publication rights are a prerequisite and deliberately outrank provider confirmation.
    let disposition = if permission != "approved_for_fixture_only" {
        Disposition::CandidateOnly
    } else if has_material_conflict(evidence) || has_failed_contact(evidence) {
        Disposition::ReviewRequired
    } else if record.get("admission_evidence").and_then(|e| e.get("user_correction")).is_some() {
        Disposition::ReviewRequired
    } else if matches!(record_status, "closed" | "temporarily closed" | "inactive" | "defunct") || field(fields, "operational_status") == Some("provider_confirmed_closed") {
        Disposition::ClosedOrWithdrawn
    } else if field(fields, "event_date") == Some("past")
        || field(fields, "capacity") == Some("stale")
        || schedule_expired(record, as_of)
        || verification_is_stale(evidence, as_of, max_age_days)?
    {
        Disposition::StaleOrDisputed
    } else if source_kind == "provider_supplied"
        && verified_at.map(|v| !v.is_null()).unwrap_or(false)
        && !verified_fields.is_empty()
        && verified_fields.iter().any(|v| v.as_str() == Some("service.status"))
        && field(fields, "operational_status") == Some("provider_confirmed")
        && record_status == "active"
    {
        Disposition::ProviderConfirmed
    } else {
        Disposition::CandidateOnly
    };

    let surface_as_current = disposition == Disposition::ProviderConfirmed;
    let location_type = record.pointer("/hsds_resources/location/location_type").and_then(Value::as_str);
    let has_verified_coordinates = location_type == Some("physical")
        && record.pointer("/hsds_resources/location/latitude").and_then(Value::as_f64).is_some()
        && record.pointer("/hsds_resources/location/longitude").and_then(Value::as_f64).is_some()
        && field(fields, "coordinates") == Some("provider_confirmed");
    let show_map_pin = surface_as_current && has_verified_coordinates;
    let may_claim_free = surface_as_current && field(fields, "cost") == Some("provider_confirmed_free");
    let may_claim_accessible = surface_as_current && field(fields, "accessibility") == Some("provider_confirmed_accessible");
    let may_claim_live_capacity = surface_as_current
        && field(fields, "capacity") == Some("provider_confirmed_live")
        && evidence.pointer("/capacity_claim/authority").and_then(Value::as_str) == Some("provider_live_source");

    Ok(Decision {
        disposition,
        surface_as_current,
        show_map_pin,
        may_claim_free,
        may_claim_accessible,
        may_claim_live_capacity,
    })
}

fn has_material_conflict(evidence: &Value) -> bool {
    evidence.get("conflicts").and_then(Value::as_array).map(|v| !v.is_empty()).unwrap_or(false)
}

fn has_failed_contact(evidence: &Value) -> bool {
    evidence.get("contact_checks").and_then(Value::as_array).map(|checks| {
        checks.iter().any(|check| check.get("result").and_then(Value::as_str) == Some("failed"))
    }).unwrap_or(false)
}

fn schedule_expired(record: &Value, as_of: &str) -> bool {
    record.pointer("/hsds_resources/schedule/valid_to")
        .and_then(Value::as_str)
        .map(|valid_to| valid_to < as_of)
        .unwrap_or(false)
}

fn field<'a>(fields: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a str> {
    fields.get(key).and_then(Value::as_str)
}

fn required_map_str<'a>(value: &'a serde_json::Map<String, Value>, key: &str, where_: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("{}.{} must be a non-empty string", where_, key))
}

fn required_str<'a>(value: &'a Value, key: &str, where_: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("{}.{} must be a non-empty string", where_, key))
}

fn valid_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && value.as_bytes().iter().enumerate().all(|(i, b)| matches!(i, 4 | 7) || b.is_ascii_digit())
}

fn verification_is_stale(evidence: &Value, as_of: &str, max_age_days: u64) -> Result<bool, String> {
    let Some(value) = evidence.get("provider_verified_at") else {
        return Ok(false);
    };
    if value.is_null() {
        return Ok(false);
    }
    let timestamp = value.as_str()
        .ok_or_else(|| "provider_verified_at must be a timestamp string or null".to_owned())?;
    let verified_date = timestamp.get(..10)
        .ok_or_else(|| "provider_verified_at must start with YYYY-MM-DD".to_owned())?;
    let verified_day = days_from_date(verified_date)
        .ok_or_else(|| "provider_verified_at contains an invalid calendar date".to_owned())?;
    let policy_day = days_from_date(as_of)
        .ok_or_else(|| "policy_as_of_date contains an invalid calendar date".to_owned())?;
    let age = policy_day - verified_day;
    if age < 0 {
        return Err("provider_verified_at is after policy_as_of_date".to_owned());
    }
    Ok(age > max_age_days as i64)
}

// Proleptic Gregorian ordinal; used only for deterministic age comparison in the policy checker.
fn days_from_date(value: &str) -> Option<i64> {
    if !valid_date(value) {
        return None;
    }
    let year: i64 = value.get(0..4)?.parse().ok()?;
    let month: i64 = value.get(5..7)?.parse().ok()?;
    let day: i64 = value.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&month) || day < 1 {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if day > month_days {
        return None;
    }
    let adjusted_year = if month <= 2 { year - 1 } else { year };
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../docs/research/fixtures/city-belonging-admission-v0.1.jsonl");
    const MANIFEST: &str = include_str!("../../../docs/research/fixtures/city-belonging-admission-v0.1.manifest.json");

    #[test]
    fn golden_fixture_matches_every_expected_disposition_and_safety_flag() {
        let report = verify_bytes(FIXTURE.as_bytes(), MANIFEST).unwrap();
        assert_eq!(report["status"], "SYNTHETIC_POLICY_FIXTURE_MATCH");
        assert_eq!(report["record_count"], 11);
        assert_eq!(report["all_cases_match_expected_policy"], true);
        assert_eq!(report["pinned_hsds_selected_resource_contract"], "MATCH");
        assert_eq!(report["canonical_hsds_full_conformance"], "NOT_ASSERTED");
        assert_eq!(report["disposition_counts"]["ProviderConfirmed"], 3);
        assert_eq!(report["disposition_counts"]["ReviewRequired"], 3);
        assert_eq!(report["disposition_counts"]["StaleOrDisputed"], 3);
        assert_eq!(report["disposition_counts"]["CandidateOnly"], 1);
        assert_eq!(report["disposition_counts"]["ClosedOrWithdrawn"], 1);
    }



    #[test]
    fn local_admission_profile_hash_matches_fixture_manifest() {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        validate_local_profile_manifest(&manifest).unwrap();
    }

    #[test]
    fn provider_verified_paths_must_exist_in_the_pinned_resource_row() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["admission_evidence"]["provider_verified_fields"][2] = json!("schedule.weekday");
        assert!(validate_admission_evidence(&record, "test CBF-001").unwrap_err().contains("not a field in the pinned schema"));
    }

    #[test]
    fn unknown_field_state_values_fail_closed() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["admission_evidence"]["field_states"]["cost"] = json!("free");
        assert!(validate_admission_evidence(&record, "test CBF-001").unwrap_err().contains("invalid field state"));
    }

    #[test]
    fn duplicate_verified_field_paths_fail_closed() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["admission_evidence"]["provider_verified_fields"][1] = json!("service.name");
        assert!(validate_admission_evidence(&record, "test CBF-001").unwrap_err().contains("duplicate provider_verified_field"));
    }

    #[test]
    fn pinned_hsds_schema_bytes_match_derived_contract_and_fixture_manifest() {
        let fixture_manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        validate_pinned_schema_contract(&fixture_manifest).unwrap();
    }

    #[test]
    fn rejects_non_hsds_location_type_enum() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["hsds_resources"]["location"]["location_type"] = json!("service_area");
        assert!(validate_hsds_resources(&record, "test CBF-001").unwrap_err().contains("pinned enum"));
    }

    #[test]
    fn rejects_non_hsds_service_status_enum() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["hsds_resources"]["service"]["status"] = json!("closed");
        assert!(validate_hsds_resources(&record, "test CBF-001").unwrap_err().contains("pinned enum"));
    }

    #[test]
    fn rejects_orphaned_service_location_relationship() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["hsds_resources"]["service_at_location"]["location_id"] = json!("ffffffff-ffff-4fff-8fff-ffffffffffff");
        assert!(validate_hsds_resources(&record, "test CBF-001").unwrap_err().contains("does not link to location.id"));
    }

    #[test]
    fn rejects_missing_capacity_unit_link() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().nth(6).unwrap()).unwrap();
        record["hsds_resources"]["service_capacity"].as_object_mut().unwrap().remove("unit_id");
        assert!(validate_hsds_resources(&record, "test CBF-007").unwrap_err().contains("missing required field unit_id"));
    }

    #[test]
    fn rejects_capacity_unit_relationship_mismatch() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().nth(6).unwrap()).unwrap();
        record["hsds_resources"]["service_capacity"]["unit_id"] = json!("ffffffff-ffff-4fff-8fff-ffffffffffff");
        assert!(validate_hsds_resources(&record, "test CBF-007").unwrap_err().contains("unit_id does not link to unit.id"));
    }

    #[test]
    fn rejects_missing_tabular_required_service_organization() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["hsds_resources"]["service"].as_object_mut().unwrap().remove("organization_id");
        assert!(validate_hsds_resources(&record, "test CBF-001").unwrap_err().contains("missing required field organization_id"));
    }

    #[test]
    fn refuses_changed_fixture_bytes_before_interpretation() {
        let mut changed = FIXTURE.as_bytes().to_vec();
        changed[0] ^= 1;
        assert!(verify_bytes(&changed, MANIFEST).unwrap_err().contains("SHA-256 mismatch"));
    }

    #[test]
    fn missing_final_newline_fails_closed_even_with_matching_digest() {
        let fixture = FIXTURE.trim_end().as_bytes();
        let mut manifest: Value = serde_json::from_str(MANIFEST).unwrap();
        manifest["fixture_sha256"] = json!(sha256_hex(fixture));
        let manifest = serde_json::to_string(&manifest).unwrap();
        assert!(verify_bytes(fixture, &manifest).unwrap_err().contains("must end with a newline"));
    }

    #[test]
    fn an_expired_event_cannot_be_current() {
        let record: Value = serde_json::from_str(FIXTURE.lines().nth(1).unwrap()).unwrap();
        let decision = evaluate_record(&record, "2026-10-17", 30).unwrap();
        assert_eq!(decision.disposition, Disposition::StaleOrDisputed);
        assert!(!decision.surface_as_current);
    }

    #[test]
    fn unresolved_reuse_permission_blocks_publication() {
        let record: Value = serde_json::from_str(FIXTURE.lines().nth(8).unwrap()).unwrap();
        let decision = evaluate_record(&record, "2026-10-17", 30).unwrap();
        assert_eq!(decision.disposition, Disposition::CandidateOnly);
        assert!(!decision.surface_as_current);
    }

    #[test]
    fn a_failed_contact_path_requires_review() {
        let record: Value = serde_json::from_str(FIXTURE.lines().nth(2).unwrap()).unwrap();
        let decision = evaluate_record(&record, "2026-10-17", 30).unwrap();
        assert_eq!(decision.disposition, Disposition::ReviewRequired);
        assert!(!decision.surface_as_current);
    }

    #[test]
    fn an_old_provider_confirmation_is_not_current() {
        let record: Value = serde_json::from_str(FIXTURE.lines().nth(10).unwrap()).unwrap();
        let decision = evaluate_record(&record, "2026-10-17", 30).unwrap();
        assert_eq!(decision.disposition, Disposition::StaleOrDisputed);
        assert!(!decision.surface_as_current);
    }

    #[test]
    fn provider_confirmation_at_freshness_boundary_remains_eligible() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["admission_evidence"]["provider_verified_at"] = json!("2026-09-17T00:00:00Z");
        let decision = evaluate_record(&record, "2026-10-17", 30).unwrap();
        assert_eq!(decision.disposition, Disposition::ProviderConfirmed);
    }

    #[test]
    fn future_provider_timestamp_fails_closed() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        record["admission_evidence"]["provider_verified_at"] = json!("2026-10-18T00:00:00Z");
        assert!(evaluate_record(&record, "2026-10-17", 30).unwrap_err().contains("after policy_as_of_date"));
    }

    #[test]
    fn unknown_cost_accessibility_and_capacity_are_never_promoted() {
        let record: Value = serde_json::from_str(FIXTURE.lines().next().unwrap()).unwrap();
        let decision = evaluate_record(&record, "2026-10-17", 30).unwrap();
        assert!(!decision.may_claim_free);
        assert!(!decision.may_claim_accessible);
        assert!(!decision.may_claim_live_capacity);
    }

    #[test]
    fn virtual_service_area_has_no_fabricated_pin() {
        let record: Value = serde_json::from_str(FIXTURE.lines().nth(9).unwrap()).unwrap();
        let decision = evaluate_record(&record, "2026-10-17", 30).unwrap();
        assert_eq!(decision.disposition, Disposition::ProviderConfirmed);
        assert!(!decision.show_map_pin);
    }

    #[test]
    fn service_area_never_becomes_a_street_pin_even_if_coordinates_exist() {
        let mut record: Value = serde_json::from_str(FIXTURE.lines().nth(9).unwrap()).unwrap();
        record["hsds_resources"]["location"]["latitude"] = json!(-26.2);
        record["hsds_resources"]["location"]["longitude"] = json!(28.0);
        record["admission_evidence"]["field_states"]["coordinates"] = json!("provider_confirmed");
        let decision = evaluate_record(&record, "2026-10-17", 30).unwrap();
        assert!(!decision.show_map_pin);
    }

    #[test]
    fn identical_bytes_produce_identical_report() {
        let first = serde_json::to_vec(&verify_bytes(FIXTURE.as_bytes(), MANIFEST).unwrap()).unwrap();
        let second = serde_json::to_vec(&verify_bytes(FIXTURE.as_bytes(), MANIFEST).unwrap()).unwrap();
        assert_eq!(first, second);
    }
}
