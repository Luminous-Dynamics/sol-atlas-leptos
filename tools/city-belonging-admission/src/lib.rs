use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const REPORT_VERSION: &str = "city-belonging-admission-report/0.1";
const CLAIM_CEILING: &str = "synthetic admission-policy fixtures only; not HSDS conformance, service truth, safety, availability, or outcome evidence";

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
        "hsds_schema_conformance": "NOT_ASSESSED",
        "human_efficacy_or_social_connection_outcomes": "NOT_ASSESSED",
        "case_receipts": receipts
    }))
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
    let record_status = record.pointer("/hsds_fragment/service/status").and_then(Value::as_str).unwrap_or("unknown");

    // Publication rights are a prerequisite and deliberately outrank provider confirmation.
    let disposition = if permission != "approved_for_fixture_only" {
        Disposition::CandidateOnly
    } else if has_material_conflict(evidence) || has_failed_contact(evidence) {
        Disposition::ReviewRequired
    } else if record.get("admission_evidence").and_then(|e| e.get("user_correction")).is_some() {
        Disposition::ReviewRequired
    } else if record_status == "closed" || field(fields, "operational_status") == Some("provider_confirmed_closed") {
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
    let location_type = record.pointer("/hsds_fragment/location/location_type").and_then(Value::as_str);
    let has_verified_coordinates = location_type == Some("physical")
        && record.pointer("/hsds_fragment/location/latitude").and_then(Value::as_f64).is_some()
        && record.pointer("/hsds_fragment/location/longitude").and_then(Value::as_f64).is_some()
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
    record.pointer("/hsds_fragment/schedule/valid_to")
        .and_then(Value::as_str)
        .map(|valid_to| valid_to < as_of)
        .unwrap_or(false)
}

fn field<'a>(fields: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a str> {
    fields.get(key).and_then(Value::as_str)
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
        assert_eq!(report["disposition_counts"]["ProviderConfirmed"], 3);
        assert_eq!(report["disposition_counts"]["ReviewRequired"], 3);
        assert_eq!(report["disposition_counts"]["StaleOrDisputed"], 3);
        assert_eq!(report["disposition_counts"]["CandidateOnly"], 1);
        assert_eq!(report["disposition_counts"]["ClosedOrWithdrawn"], 1);
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
        record["hsds_fragment"]["location"]["latitude"] = json!(-26.2);
        record["hsds_fragment"]["location"]["longitude"] = json!(28.0);
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
