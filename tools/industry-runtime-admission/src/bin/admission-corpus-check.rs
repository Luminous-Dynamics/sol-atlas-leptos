use industry_assurance_validator::validate_profile;
use industry_runtime_admission::{evaluate_admission, independent_check};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const CASE_LEDGER_SCHEMA: &str = "luminous.ieac-runtime-admission-case-ledger.v0.1";

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest_bytes(bytes: &[u8]) -> String {
    hex_digest(&Sha256::digest(bytes))
}

fn digest_value(value: &Value) -> String {
    digest_bytes(&serde_json::to_vec(value).expect("JSON values serialize"))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn read_record(reader: &mut BufReader<File>, label: &str) -> Result<Option<(Vec<u8>, Value)>, String> {
    let mut line = Vec::new();
    let bytes_read = reader.read_until(b'\n', &mut line)
        .map_err(|error| format!("cannot read {label}: {error}"))?;
    if bytes_read == 0 {
        return Ok(None);
    }
    if !line.ends_with(b"\n") {
        return Err(format!("{label} is truncated: final JSONL record lacks LF"));
    }
    let payload = &line[..line.len() - 1];
    let value: Value = serde_json::from_slice(payload)
        .map_err(|error| format!("invalid JSON in {label}: {error}"))?;
    let canonical = serde_json::to_vec(&value)
        .map_err(|error| format!("cannot canonicalize {label}: {error}"))?;
    if canonical.as_slice() != payload {
        return Err(format!("{label} is not in canonical serde_json encoding"));
    }
    Ok(Some((line, value)))
}

fn string_array(value: &Value) -> Option<Vec<String>> {
    value.as_array()?.iter().map(|item| item.as_str().map(str::to_owned)).collect()
}

fn exact_reason_codes_match(report: &Value, expected: &Value) -> bool {
    let Some(mut actual) = string_array(&report["reason_codes"]) else {
        return false;
    };
    let Some(mut expected) = string_array(expected) else {
        return false;
    };
    actual.sort_unstable();
    expected.sort_unstable();
    actual == expected
}

fn require_string<'a>(value: &'a Value, pointer: &str, label: &str) -> Result<&'a str, String> {
    value.pointer(pointer).and_then(Value::as_str)
        .ok_or_else(|| format!("{label} is missing string field {pointer}"))
}

fn require_u64(value: &Value, pointer: &str, label: &str) -> Result<u64, String> {
    value.pointer(pointer).and_then(Value::as_u64)
        .ok_or_else(|| format!("{label} is missing unsigned integer field {pointer}"))
}

fn verify_case(
    record: &Value,
    case_index: usize,
    header_vertical: &str,
    expected_trial_count: usize,
    family_counts: &mut BTreeMap<String, usize>,
    case_ids: &mut BTreeSet<String>,
    counts: &mut Counts,
) -> Result<(), String> {
    if record["schema_version"].as_str() != Some(CASE_LEDGER_SCHEMA) {
        return Err(format!("case {case_index} has an unsupported schema version"));
    }
    if require_u64(record, "/case_index", "case record")? != case_index as u64 {
        return Err(format!("case index is not contiguous at record {case_index}"));
    }
    if require_string(record, "/vertical", "case record")? != header_vertical {
        return Err(format!("case {case_index} vertical differs from ledger header"));
    }
    let case_id = require_string(record, "/case_id", "case record")?.to_owned();
    if !case_ids.insert(case_id.clone()) {
        return Err(format!("duplicate case_id {case_id}"));
    }
    let family = require_string(record, "/failure_family", "case record")?.to_owned();
    let trial_index = require_u64(record, "/trial_index", "case record")? as usize;
    let next_expected_trial = family_counts.entry(family.clone()).or_insert(0);
    if trial_index != *next_expected_trial || trial_index >= expected_trial_count {
        return Err(format!(
            "non-contiguous trial index for {family}: expected {}, got {trial_index}",
            *next_expected_trial
        ));
    }
    *next_expected_trial += 1;

    let mut unsigned_record = record.clone();
    let record_object = unsigned_record.as_object_mut()
        .ok_or_else(|| format!("case {case_index} is not an object"))?;
    let recorded_case_digest = record_object.remove("case_sha256")
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or_else(|| format!("case {case_index} has no case_sha256"))?;
    if !is_sha256(&recorded_case_digest) || digest_value(&unsigned_record) != recorded_case_digest {
        return Err(format!("case {case_index} digest mismatch"));
    }

    let negative_control = record["negative_control"].as_bool()
        .ok_or_else(|| format!("case {case_index} negative_control is not boolean"))?;
    let profile = &record["inputs"]["profile"];
    let profile_pin = &record["inputs"]["profile_pin"];
    let request = &record["inputs"]["request"];
    if profile.is_null() || profile_pin.is_null() || request.is_null() {
        return Err(format!("case {case_index} is missing one or more exact input objects"));
    }

    let candidate_report = &record["observed"]["candidate_report"];
    if candidate_report.is_null() {
        return Err(format!("case {case_index} is missing candidate_report"));
    }
    let reconstructed = evaluate_admission(profile, profile_pin, request);
    if &reconstructed != candidate_report {
        return Err(format!("candidate report does not replay for case {case_index}"));
    }

    let checker_errors = independent_check(profile, profile_pin, request, candidate_report);
    let stored_checker_errors = &record["observed"]["checker_errors"];
    if stored_checker_errors != &serde_json::json!(checker_errors) {
        return Err(format!("checker errors do not replay for case {case_index}"));
    }
    if !checker_errors.is_empty() {
        counts.checker_disagreements += 1;
    }

    let static_profile_valid = validate_profile(profile).is_empty();
    if record["observed"]["static_profile_valid"].as_bool() != Some(static_profile_valid) {
        return Err(format!("static-profile baseline result does not replay for case {case_index}"));
    }

    let expected_admission = require_string(record, "/expected/admission", "case expectation")?;
    let actual_admission = require_string(candidate_report, "/admission", "candidate report")?;
    if record["observed"]["admission"].as_str() != Some(actual_admission) {
        return Err(format!("observed admission does not match candidate report for case {case_index}"));
    }
    if actual_admission != expected_admission {
        counts.expected_disposition_mismatches += 1;
    }
    if !exact_reason_codes_match(candidate_report, &record["expected"]["reason_codes"]) {
        counts.expected_reason_code_misses += 1;
    }

    if negative_control {
        counts.negative_trials += 1;
        if actual_admission.starts_with("admit_") {
            counts.false_admissions += 1;
        }
        if actual_admission == "abstain" {
            counts.negative_abstentions += 1;
        }
    } else {
        counts.positive_trials += 1;
        if actual_admission.starts_with("admit_") {
            counts.positive_admissions += 1;
        }
    }
    Ok(())
}

#[derive(Default)]
struct Counts {
    negative_trials: usize,
    false_admissions: usize,
    negative_abstentions: usize,
    positive_trials: usize,
    positive_admissions: usize,
    checker_disagreements: usize,
    expected_disposition_mismatches: usize,
    expected_reason_code_misses: usize,
}

fn verify(cases_path: &Path, report_path: &Path) -> Result<Counts, String> {
    let file = File::open(cases_path)
        .map_err(|error| format!("cannot open {}: {error}", cases_path.display()))?;
    let mut reader = BufReader::new(file);

    let (header_line, header) = read_record(&mut reader, "case-ledger header")?
        .ok_or("case ledger is empty")?;
    if header["record_type"].as_str() != Some("header")
        || header["schema_version"].as_str() != Some(CASE_LEDGER_SCHEMA)
    {
        return Err("case ledger header has an unsupported type or schema".to_owned());
    }
    let manifest_digest = require_string(&header, "/manifest_exact_bytes_sha256", "ledger header")?;
    if !is_sha256(manifest_digest) {
        return Err("ledger header manifest digest is malformed".to_owned());
    }
    let vertical = require_string(&header, "/vertical", "ledger header")?;
    let family_count = require_u64(&header, "/failure_family_count", "ledger header")? as usize;
    let trials_per_family = require_u64(&header, "/trials_per_failure_family", "ledger header")? as usize;
    if family_count == 0 || trials_per_family == 0 {
        return Err("ledger header declares an empty corpus".to_owned());
    }
    let expected_record_count = family_count.checked_mul(trials_per_family)
        .ok_or("ledger expected record count overflows usize")?;

    let mut prefix_hasher = Sha256::new();
    prefix_hasher.update(&header_line);
    let mut cases_hasher = Sha256::new();
    let mut case_ids = BTreeSet::new();
    let mut family_counts = BTreeMap::new();
    let mut counts = Counts::default();
    let mut case_index = 0usize;
    let footer;

    loop {
        let (line, record) = read_record(&mut reader, "case-ledger record")?
            .ok_or("case ledger ended without a footer; treat it as incomplete")?;
        match record["record_type"].as_str() {
            Some("case") => {
                if case_index >= expected_record_count {
                    return Err("case ledger contains more cases than preregistered".to_owned());
                }
                verify_case(
                    &record,
                    case_index,
                    vertical,
                    trials_per_family,
                    &mut family_counts,
                    &mut case_ids,
                    &mut counts,
                )?;
                prefix_hasher.update(&line);
                cases_hasher.update(&line);
                case_index += 1;
            }
            Some("footer") => {
                footer = record;
                break;
            }
            other => return Err(format!("unexpected ledger record_type {other:?}")),
        }
    }

    if case_index != expected_record_count {
        return Err(format!("ledger contains {case_index} cases; expected {expected_record_count}"));
    }
    if family_counts.len() != family_count
        || family_counts.values().any(|count| *count != trials_per_family)
    {
        return Err("failure-family strata are missing, duplicated or incomplete".to_owned());
    }
    if footer["schema_version"].as_str() != Some(CASE_LEDGER_SCHEMA)
        || footer["record_count"].as_u64() != Some(case_index as u64)
        || footer["case_records_sha256"].as_str() != Some(hex_digest(&cases_hasher.clone().finalize()).as_str())
        || footer["prefix_sha256"].as_str() != Some(hex_digest(&prefix_hasher.clone().finalize()).as_str())
    {
        return Err("case-ledger footer count or digest mismatch".to_owned());
    }
    if footer["digest_scope"].as_str() != Some("header_and_case_records_with_lf; footer excluded") {
        return Err("case-ledger footer digest scope is unsupported".to_owned());
    }
    if read_record(&mut reader, "data after case-ledger footer")?.is_some() {
        return Err("case ledger contains data after its footer".to_owned());
    }

    let report_bytes = std::fs::read(report_path)
        .map_err(|error| format!("cannot read {}: {error}", report_path.display()))?;
    let report: Value = serde_json::from_slice(&report_bytes)
        .map_err(|error| format!("benchmark report is invalid JSON: {error}"))?;
    if report["schema_version"].as_str() != Some("luminous.ieac-runtime-admission-benchmark-report.v0.1") {
        return Err("benchmark report schema version mismatch".to_owned());
    }
    if report["manifest_exact_bytes_sha256"].as_str() != Some(manifest_digest)
        || report["vertical"].as_str() != Some(vertical)
        || report["benchmark_id"] != header["benchmark_id"]
        || report["candidate_method_id"] != header["candidate_method_id"]
        || report["evaluator_version"] != header["evaluator_version"]
        || report["checker_version"] != header["checker_version"]
    {
        return Err("benchmark report identity does not match case-ledger header".to_owned());
    }
    if report["case_ledger"]["schema_version"].as_str() != Some(CASE_LEDGER_SCHEMA)
        || report["case_ledger"]["record_count"].as_u64() != Some(case_index as u64)
        || report["case_ledger"]["case_records_sha256"].as_str()
            != footer["case_records_sha256"].as_str()
        || report["case_ledger"]["prefix_sha256"].as_str() != footer["prefix_sha256"].as_str()
        || report["case_ledger"]["footer_record"].as_bool() != Some(true)
    {
        return Err("benchmark report does not bind the verified case ledger".to_owned());
    }

    let summary = &report["summary"];
    let summary_matches = summary["failure_family_count"].as_u64() == Some(family_count as u64)
        && summary["negative_trials"].as_u64() == Some(counts.negative_trials as u64)
        && summary["false_admissions"].as_u64() == Some(counts.false_admissions as u64)
        && summary["positive_trials"].as_u64() == Some(counts.positive_trials as u64)
        && summary["positive_admissions"].as_u64() == Some(counts.positive_admissions as u64)
        && summary["candidate_checker_disagreements"].as_u64() == Some(counts.checker_disagreements as u64)
        && summary["expected_disposition_mismatches"].as_u64()
            == Some(counts.expected_disposition_mismatches as u64)
        && summary["expected_reason_code_misses"].as_u64()
            == Some(counts.expected_reason_code_misses as u64);
    if !summary_matches {
        return Err("benchmark report aggregates do not reconstruct from the case ledger".to_owned());
    }

    if counts.false_admissions != 0
        || counts.checker_disagreements != 0
        || counts.expected_disposition_mismatches != 0
        || counts.expected_reason_code_misses != 0
    {
        return Err(format!(
            "case ledger replayed but acceptance failed: false_admissions={}, checker_disagreements={}, disposition_mismatches={}, reason_code_misses={}",
            counts.false_admissions,
            counts.checker_disagreements,
            counts.expected_disposition_mismatches,
            counts.expected_reason_code_misses
        ));
    }
    Ok(counts)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: industry-admission-corpus-check <cases.jsonl> <benchmark-report.json>");
        std::process::exit(2);
    }
    match verify(Path::new(&args[0]), Path::new(&args[1])) {
        Ok(counts) => println!(
            "verified {} exact case records; false admissions={}, checker disagreements={}, expected mismatches=0",
            counts.negative_trials + counts.positive_trials,
            counts.false_admissions,
            counts.checker_disagreements
        ),
        Err(error) => {
            eprintln!("case-ledger verification failed: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_code_matching_rejects_non_string_entries() {
        assert!(!exact_reason_codes_match(
            &serde_json::json!({"reason_codes": ["EXPECTED", 7]}),
            &serde_json::json!(["EXPECTED"])
        ));
    }

    #[test]
    fn reason_code_matching_is_order_independent_but_exact() {
        assert!(exact_reason_codes_match(
            &serde_json::json!({"reason_codes": ["B", "A"]}),
            &serde_json::json!(["A", "B"])
        ));
        assert!(!exact_reason_codes_match(
            &serde_json::json!({"reason_codes": ["A", "B", "EXTRA"]}),
            &serde_json::json!(["A", "B"])
        ));
    }
}
