use industry_runtime_admission::{evaluate_admission, independent_check};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

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

fn verify_reader<R: BufRead>(mut reader: R) -> Result<Value, String> {
    let mut line = Vec::new();
    let mut hasher = Sha256::new();
    let mut header_seen = false;
    let mut footer_seen = false;
    let mut footer_case_count = None;
    let mut ledger_digest = None;
    let mut seen_case_ids = HashSet::new();
    let mut report_mismatches = 0usize;
    let mut input_digest_mismatches = 0usize;
    let mut checker_mismatches = 0usize;
    let mut expectation_mismatches = 0usize;
    let mut tamper_mismatches = 0usize;
    let mut deterministic_mismatches = 0usize;
    let mut case_count = 0usize;

    loop {
        line.clear();
        let bytes_read = reader
            .read_until(b'\n', &mut line)
            .map_err(|error| format!("cannot read case ledger: {error}"))?;
        if bytes_read == 0 {
            break;
        }
        if !line.ends_with(b"\n") {
            return Err("case ledger is truncated: final newline/footer missing".to_owned());
        }

        let record: Value = serde_json::from_slice(&line[..line.len() - 1])
            .map_err(|error| format!("invalid case-ledger record: {error}"))?;
        let record_type = record["record_type"].as_str().unwrap_or("");

        if !header_seen {
            if record_type != "header"
                || record["schema_version"].as_str()
                    != Some("luminous.ieac-runtime-admission-case-ledger.v0.1")
            {
                return Err("unsupported case-ledger header or schema version".to_owned());
            }
            hasher.update(&line);
            header_seen = true;
            continue;
        }

        if record_type == "footer" {
            if record["schema_version"].as_str()
                != Some("luminous.ieac-runtime-admission-case-ledger.v0.1")
            {
                return Err("case-ledger footer has an unsupported schema version".to_owned());
            }
            let actual_digest = hasher
                .clone()
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            if record["preceding_records_sha256"].as_str() != Some(actual_digest.as_str()) {
                return Err("case-ledger content digest mismatch".to_owned());
            }
            footer_case_count = Some(
                record["case_records"]
                    .as_u64()
                    .ok_or("case-ledger footer.case_records must be an unsigned integer")?
                    as usize,
            );
            ledger_digest = Some(actual_digest);
            footer_seen = true;
            let mut trailing = Vec::new();
            if reader
                .read_until(b'\n', &mut trailing)
                .map_err(|error| format!("cannot check case-ledger trailing data: {error}"))?
                != 0
            {
                return Err("unexpected content after case-ledger footer".to_owned());
            }
            break;
        }

        if record_type != "case" {
            return Err(format!("unexpected case-ledger record type {record_type:?}"));
        }
        hasher.update(&line);

        let offset = case_count;
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
        let expected_disposition_match = actual_admission == expected_admission;
        let expected_reason_match = expected_reasons == actual_reasons;
        if record["expected_disposition_match"].as_bool()
            != Some(expected_disposition_match)
            || record["expected_reason_codes_match"].as_bool() != Some(expected_reason_match)
        {
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

    if !header_seen || !footer_seen {
        return Err("case ledger is incomplete: header or footer missing".to_owned());
    }
    let expected_case_count =
        footer_case_count.ok_or("case-ledger footer count is missing")?;
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
        "case_ledger_preceding_records_sha256": ledger_digest,
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
    let file = File::open(&path).map_err(|error| format!("cannot open {path}: {error}"))?;
    verify_reader(BufReader::new(file))
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
        let report = verify_reader(Cursor::new(empty_ledger())).unwrap();
        assert_eq!(report["verified"], true);
        assert_eq!(report["case_records"], 0);
    }

    #[test]
    fn modified_ledger_bytes_fail_digest_validation() {
        let mut bytes = empty_ledger();
        let header_end = bytes.iter().position(|byte| *byte == b'\n').unwrap();
        bytes.insert(header_end, b' ');
        assert!(verify_reader(Cursor::new(bytes))
            .unwrap_err()
            .contains("digest mismatch"));
    }

    #[test]
    fn truncated_ledger_is_rejected() {
        let mut bytes = empty_ledger();
        bytes.pop();
        assert!(verify_reader(Cursor::new(bytes))
            .unwrap_err()
            .contains("truncated"));
    }
}
