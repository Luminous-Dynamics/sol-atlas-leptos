use industry_runtime_admission::{evaluate_admission, independent_check};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs;

fn digest_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sorted_reason_codes(value: &Value) -> Option<Vec<String>> {
    let codes = value.as_array()?;
    let mut result = codes
        .iter()
        .map(|code| code.as_str().map(str::to_owned))
        .collect::<Option<Vec<_>>>()?;
    result.sort_unstable();
    Some(result)
}

fn verify_ledger(bytes: &[u8]) -> Result<Value, String> {
    if !bytes.ends_with(b"\n") {
        return Err("case ledger is truncated: final newline/footer missing".to_owned());
    }
    let lines = bytes.split(|byte| *byte == b'\n').collect::<Vec<_>>();
    if lines.len() < 3 || !lines.last().is_some_and(|line| line.is_empty()) {
        return Err("case ledger must contain a header and footer".to_owned());
    }
    let records = &lines[..lines.len() - 1];
    let header: Value = serde_json::from_slice(records[0])
        .map_err(|error| format!("invalid case-ledger header: {error}"))?;
    if header["record_type"].as_str() != Some("header")
        || header["schema_version"].as_str()
            != Some("luminous.ieac-runtime-admission-case-ledger.v0.1")
    {
        return Err("unsupported case-ledger header or schema version".to_owned());
    }

    let footer_index = records.len() - 1;
    let footer: Value = serde_json::from_slice(records[footer_index])
        .map_err(|error| format!("invalid case-ledger footer: {error}"))?;
    if footer["record_type"].as_str() != Some("footer")
        || footer["schema_version"].as_str()
            != Some("luminous.ieac-runtime-admission-case-ledger.v0.1")
    {
        return Err("case-ledger footer is absent or has an unsupported schema version".to_owned());
    }
    let prefix_len = records[..footer_index]
        .iter()
        .map(|line| line.len() + 1)
        .sum::<usize>();
    let actual_ledger_digest = digest_bytes(&bytes[..prefix_len]);
    if footer["preceding_records_sha256"].as_str()
        != Some(actual_ledger_digest.as_str())
    {
        return Err("case-ledger content digest mismatch".to_owned());
    }

    let expected_case_count = footer["case_records"]
        .as_u64()
        .ok_or("case-ledger footer.case_records must be an unsigned integer")?
        as usize;
    let mut seen_case_ids = HashSet::new();
    let mut report_mismatches = 0usize;
    let mut input_digest_mismatches = 0usize;
    let mut checker_mismatches = 0usize;
    let mut expectation_mismatches = 0usize;
    let mut tamper_mismatches = 0usize;
    let mut deterministic_mismatches = 0usize;
    let mut case_count = 0usize;

    for (offset, line) in records[1..footer_index].iter().enumerate() {
        let record: Value = serde_json::from_slice(line)
            .map_err(|error| format!("invalid case record at index {offset}: {error}"))?;
        if record["record_type"].as_str() != Some("case") {
            return Err(format!("unexpected record type at case index {offset}"));
        }
        let case_index = record["case_index"]
            .as_u64()
            .ok_or_else(|| format!("case {offset} is missing case_index"))?
            as usize;
        if case_index != offset {
            return Err(format!(
                "non-contiguous case index: expected {offset}, got {case_index}"
            ));
        }
        let case_id = record["case_id"]
            .as_str()
            .ok_or_else(|| format!("case {offset} is missing case_id"))?;
        if !seen_case_ids.insert(case_id.to_owned()) {
            return Err(format!("duplicate case_id {case_id}"));
        }

        let inputs = &record["inputs"];
        let input_bytes = serde_json::to_vec(inputs)
            .map_err(|error| format!("cannot serialize replay inputs: {error}"))?;
        let actual_input_digest = digest_bytes(&input_bytes);
        if record["inputs_sha256"].as_str() != Some(actual_input_digest.as_str()) {
            input_digest_mismatches += 1;
        }
        let profile = &inputs["profile"];
        let pin = &inputs["profile_pin"];
        let request = &inputs["request"];
        if profile.is_null() || pin.is_null() || request.is_null() {
            return Err(format!(
                "case {case_id} lacks profile, profile pin or request"
            ));
        }

        let replayed_report = evaluate_admission(profile, pin, request);
        if replayed_report != record["candidate_report"] {
            report_mismatches += 1;
        }
        let checker_errors = independent_check(profile, pin, request, &replayed_report);
        if json!(checker_errors) != record["checker_errors"] {
            checker_mismatches += 1;
        }

        let actual_admission = replayed_report["admission"]
            .as_str()
            .unwrap_or("invalid");
        let expected_admission = record["expected_admission"]
            .as_str()
            .unwrap_or("invalid");
        let expected_reasons = sorted_reason_codes(&record["expected_reason_codes"]);
        let actual_reasons = sorted_reason_codes(&replayed_report["reason_codes"]);
        if actual_admission != expected_admission || expected_reasons != actual_reasons {
            expectation_mismatches += 1;
        }

        let mut tampered_report = replayed_report.clone();
        let admitted = actual_admission.starts_with("admit_");
        tampered_report["admission"] =
            json!(if admitted { "reject" } else { "admit_descriptive" });
        let tamper_detected =
            !independent_check(profile, pin, request, &tampered_report).is_empty();
        if record["tamper_detected"].as_bool() != Some(tamper_detected) {
            tamper_mismatches += 1;
        }

        let replay_again = evaluate_admission(profile, pin, request);
        let deterministic = replay_again == replayed_report;
        if record["replay_checked"].as_bool() == Some(true)
            && record["replay_deterministic"].as_bool() != Some(deterministic)
        {
            deterministic_mismatches += 1;
        }
        case_count += 1;
    }

    if case_count != expected_case_count {
        return Err(format!(
            "footer case count mismatch: expected {expected_case_count}, parsed {case_count}"
        ));
    }
    let verified = input_digest_mismatches == 0
        && report_mismatches == 0
        && checker_mismatches == 0
        && expectation_mismatches == 0
        && tamper_mismatches == 0
        && deterministic_mismatches == 0;
    Ok(json!({
        "schema_version": "luminous.ieac-runtime-admission-case-replay-report.v0.1",
        "verified": verified,
        "case_records": case_count,
        "case_ledger_sha256": actual_ledger_digest,
        "input_digest_mismatches": input_digest_mismatches,
        "report_mismatches": report_mismatches,
        "checker_mismatches": checker_mismatches,
        "expectation_mismatches": expectation_mismatches,
        "tamper_mismatches": tamper_mismatches,
        "deterministic_mismatches": deterministic_mismatches,
        "claim_ceiling": "Deterministic replay verifies consistency with this evaluator/checker build and ledger; it is not an independent source-truth audit."
    }))
}

fn run() -> Result<Value, String> {
    let mut args = env::args().skip(1);
    let mut path = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--cases" => {
                if path.is_some() {
                    return Err("--cases may be supplied only once".to_owned());
                }
                path = Some(args.next().ok_or("missing value after --cases")?);
            }
            _ => return Err(format!("unknown argument {argument}")),
        }
    }
    let path = path.ok_or("required argument: --cases <case-ledger.jsonl>")?;
    let bytes = fs::read(&path).map_err(|error| format!("cannot read {path}: {error}"))?;
    verify_ledger(&bytes)
}

fn main() {
    match run() {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("replay report serializes")
            );
            if report["verified"].as_bool() != Some(true) {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            eprintln!("usage: industry-admission-replay --cases <case-ledger.jsonl>");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ledger() -> Vec<u8> {
        let header = serde_json::to_vec(&json!({
            "record_type": "header",
            "schema_version": "luminous.ieac-runtime-admission-case-ledger.v0.1"
        }))
        .unwrap();
        let mut bytes = header;
        bytes.push(b'\n');
        let digest = digest_bytes(&bytes);
        let footer = serde_json::to_vec(&json!({
            "record_type": "footer",
            "schema_version": "luminous.ieac-runtime-admission-case-ledger.v0.1",
            "case_records": 0,
            "preceding_records_sha256": digest
        }))
        .unwrap();
        bytes.extend_from_slice(&footer);
        bytes.push(b'\n');
        bytes
    }

    #[test]
    fn valid_header_and_footer_are_accepted() {
        let report = verify_ledger(&empty_ledger()).unwrap();
        assert_eq!(report["verified"], true);
        assert_eq!(report["case_records"], 0);
    }

    #[test]
    fn modified_ledger_bytes_fail_digest_validation() {
        let mut bytes = empty_ledger();
        bytes[2] ^= 1;
        assert!(verify_ledger(&bytes).unwrap_err().contains("digest mismatch"));
    }

    #[test]
    fn truncated_ledger_is_rejected() {
        let mut bytes = empty_ledger();
        bytes.pop();
        assert!(verify_ledger(&bytes).unwrap_err().contains("truncated"));
    }
}
