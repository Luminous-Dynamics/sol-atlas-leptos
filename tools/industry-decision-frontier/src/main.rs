use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const SNAPSHOT_SCHEMA: &str = "luminous.ieac-decision-frontier-snapshot.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-decision-frontier-report.v0.1";

fn deserialize_present_option<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema_version: String,
    snapshot_id: String,
    decision: DecisionRef,
    decision_cutoff: TimeBound,
    evidence: Vec<EvidenceItem>,
    claim_ceiling: String,
    action_authority: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionRef {
    decision_case_id: String,
    decision_event_sha256: String,
    subject_pseudonym: String,
    relying_party_id: String,
    evidence_channel_id: String,
    profile_sha256: String,
    policy_sha256: String,
    evaluator_build_sha256: String,
    time_verifier_build_sha256: String,
    time_verification_policy_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TimeBound {
    time_utc: String,
    uncertainty_before_seconds: u64,
    uncertainty_after_seconds: u64,
    clock_id: String,
    basis: String,
    verification_status: String,
    #[serde(deserialize_with = "deserialize_present_option")]
    proof_sha256: Option<String>,
    #[serde(deserialize_with = "deserialize_present_option")]
    verifier_build_sha256: Option<String>,
    #[serde(deserialize_with = "deserialize_present_option")]
    verification_policy_sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceItem {
    evidence_id: String,
    evidence_kind: String,
    source_id: String,
    artifact_sha256: String,
    #[serde(deserialize_with = "deserialize_present_option")]
    valid_time: Option<ValidTime>,
    #[serde(deserialize_with = "deserialize_present_option")]
    observed_at: Option<TimeBound>,
    #[serde(deserialize_with = "deserialize_present_option")]
    source_recorded_at: Option<TimeBound>,
    #[serde(deserialize_with = "deserialize_present_option")]
    availability_evidence: Option<FirstAvailability>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidTime {
    #[serde(deserialize_with = "deserialize_present_option")]
    valid_from: Option<String>,
    #[serde(deserialize_with = "deserialize_present_option")]
    valid_until: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FirstAvailability {
    relying_party_id: String,
    evidence_channel_id: String,
    proof_kind: String,
    artifact_sha256: String,
    time: TimeBound,
    #[serde(deserialize_with = "deserialize_present_option")]
    coverage: Option<CoverageProof>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageProof {
    relying_party_id: String,
    evidence_channel_id: String,
    coverage_from: TimeBound,
    complete_through: TimeBound,
    coverage_proof_sha256: String,
    completeness: String,
    verification_status: String,
    #[serde(deserialize_with = "deserialize_present_option")]
    verifier_build_sha256: Option<String>,
    #[serde(deserialize_with = "deserialize_present_option")]
    verification_policy_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct TimeInterval {
    earliest_epoch_seconds: i64,
    latest_epoch_seconds: i64,
}

#[derive(Debug, Serialize)]
struct Classification {
    evidence_id: String,
    artifact_sha256: String,
    as_known_status: String,
    reason_code: String,
    observed_before_cutoff_by_source_clock: Option<bool>,
    availability_interval: Option<TimeInterval>,
}

#[derive(Debug, Serialize)]
struct Summary {
    evidence_count: usize,
    candidate_as_known_count: usize,
    excluded_after_cutoff: usize,
    unknown: usize,
    history_completeness: String,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: String,
    snapshot_id: String,
    input_sha256: String,
    decision_case_id: String,
    decision_cutoff_interval: Option<TimeInterval>,
    classifications: Vec<Classification>,
    candidate_as_known_evidence_ids: Vec<String>,
    later_available_evidence_ids: Vec<String>,
    unresolved_evidence_ids: Vec<String>,
    summary: Summary,
    claim_ceiling: String,
    action_authority: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionStatus {
    Included,
    ExcludedAfterCutoff,
    Unknown,
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_time_bound(bound: &TimeBound, path: &str) -> Result<(), String> {
    parse_utc_seconds(&bound.time_utc)
        .map_err(|error| format!("{path}.time_utc: {error}"))?;

    if bound.clock_id.trim().is_empty() {
        return Err(format!("{path}.clock_id must not be empty"));
    }

    if bound.uncertainty_before_seconds > 86_400
        || bound.uncertainty_after_seconds > 86_400
    {
        return Err(format!(
            "{path} uncertainty exceeds the one-day v0.1 bound"
        ));
    }

    let optional_digests = [
        ("proof_sha256", bound.proof_sha256.as_deref()),
        ("verifier_build_sha256", bound.verifier_build_sha256.as_deref()),
        (
            "verification_policy_sha256",
            bound.verification_policy_sha256.as_deref(),
        ),
    ];

    for (field, value) in optional_digests {
        if let Some(value) = value {
            if !is_sha256(value) {
                return Err(format!("{path}.{field} must be 64 lowercase hex characters"));
            }
        }
    }

    if bound.verification_status == "verified" {
        for (field, value) in optional_digests {
            if value.is_none() {
                return Err(format!(
                    "{path}.{field} is required when verification_status is verified"
                ));
            }
        }
    }

    if !matches!(
        bound.verification_status.as_str(),
        "verified" | "failed" | "unknown" | "not_checked" | "disputed"
    ) {
        return Err(format!("{path}.verification_status is unsupported"));
    }

    Ok(())
}

fn validate_snapshot(snapshot: &Snapshot) -> Result<(), String> {
    if snapshot.schema_version != SNAPSHOT_SCHEMA {
        return Err("unsupported snapshot schema_version".to_string());
    }
    if snapshot.snapshot_id.trim().is_empty()
        || snapshot.decision.decision_case_id.trim().is_empty()
        || snapshot.decision.subject_pseudonym.trim().is_empty()
        || snapshot.decision.relying_party_id.trim().is_empty()
    {
        return Err("snapshot and decision identities must not be empty".to_string());
    }
    for (name, digest) in [
        ("decision_event_sha256", &snapshot.decision.decision_event_sha256),
        ("profile_sha256", &snapshot.decision.profile_sha256),
        ("policy_sha256", &snapshot.decision.policy_sha256),
        ("evaluator_build_sha256", &snapshot.decision.evaluator_build_sha256),
        ("time_verifier_build_sha256", &snapshot.decision.time_verifier_build_sha256),
        ("time_verification_policy_sha256", &snapshot.decision.time_verification_policy_sha256),
    ] {
        if !is_sha256(digest) {
            return Err(format!("decision.{name} must be 64 lowercase hex characters"));
        }
    }
    if snapshot.claim_ceiling != "historical_information_boundary_only" {
        return Err("unsupported claim_ceiling".to_string());
    }
    if snapshot.action_authority != "none" {
        return Err("action_authority must be none".to_string());
    }
    if snapshot.evidence.is_empty() || snapshot.evidence.len() > 10_000 {
        return Err("evidence count must be between 1 and 10000".to_string());
    }

    validate_time_bound(&snapshot.decision_cutoff, "decision_cutoff")?;
    validate_verifier_pin(&snapshot.decision_cutoff, &snapshot.decision, "decision_cutoff")?;

    let mut ids = BTreeSet::new();
    for (index, item) in snapshot.evidence.iter().enumerate() {
        let path = format!("evidence[{index}]");
        if item.evidence_id.trim().is_empty() || item.source_id.trim().is_empty() {
            return Err(format!("{path} identity fields must not be empty"));
        }
        if !matches!(
            item.evidence_kind.as_str(),
            "source_measurement" | "source_correction" | "model_output" |
            "outcome_observation" | "decision_input" | "other"
        ) {
            return Err(format!("{path}.evidence_kind is unsupported"));
        }
        if !ids.insert(item.evidence_id.as_str()) {
            return Err(format!("duplicate evidence_id: {}", item.evidence_id));
        }
        if !is_sha256(&item.artifact_sha256) {
            return Err(format!("{path}.artifact_sha256 is invalid"));
        }

        if let Some(valid_time) = &item.valid_time {
            let valid_from = valid_time.valid_from.as_deref()
                .map(parse_utc_seconds)
                .transpose()
                .map_err(|error| format!("{path}.valid_time.valid_from: {error}"))?;
            let valid_until = valid_time.valid_until.as_deref()
                .map(parse_utc_seconds)
                .transpose()
                .map_err(|error| format!("{path}.valid_time.valid_until: {error}"))?;
            if matches!((valid_from, valid_until), (Some(start), Some(end)) if start > end) {
                return Err(format!("{path}.valid_time interval is reversed"));
            }
        }
        if let Some(observed) = &item.observed_at {
            validate_time_bound(observed, &format!("{path}.observed_at"))?;
            validate_verifier_pin(observed, &snapshot.decision, &format!("{path}.observed_at"))?;
        }
        if let Some(recorded) = &item.source_recorded_at {
            validate_time_bound(recorded, &format!("{path}.source_recorded_at"))?;
            validate_verifier_pin(recorded, &snapshot.decision, &format!("{path}.source_recorded_at"))?;
        }
        if let Some(availability) = &item.availability_evidence {
            if availability.relying_party_id.trim().is_empty()
                || availability.evidence_channel_id.trim().is_empty()
            {
                return Err(format!("{path}.availability_evidence identity fields are empty"));
            }
            if availability.artifact_sha256 != item.artifact_sha256 {
                return Err(format!(
                    "{path}.availability_evidence.artifact_sha256 does not bind the evidence artifact"
                ));
            }
            validate_time_bound(
                &availability.time,
                &format!("{path}.availability_evidence.time"),
            )?;
            validate_verifier_pin(
                &availability.time,
                &snapshot.decision,
                &format!("{path}.availability_evidence.time"),
            )?;
            if let Some(coverage) = &availability.coverage {
                validate_coverage_proof(
                    coverage,
                    &snapshot.decision,
                    &format!("{path}.availability_evidence.coverage"),
                )?;
            }
            if availability.proof_kind != "complete_first_availability_ledger"
                && availability.coverage.is_some()
            {
                return Err(format!(
                    "{path}.availability_evidence.coverage requires complete_first_availability_ledger"
                ));
            }
        }
    }
    Ok(())
}

fn validate_coverage_proof(
    coverage: &CoverageProof,
    decision: &DecisionRef,
    path: &str,
) -> Result<(), String> {
    validate_time_bound(&coverage.coverage_from, &format!("{path}.coverage_from"))?;
    validate_time_bound(
        &coverage.complete_through,
        &format!("{path}.complete_through"),
    )?;
    validate_verifier_pin(
        &coverage.coverage_from,
        decision,
        &format!("{path}.coverage_from"),
    )?;
    validate_verifier_pin(
        &coverage.complete_through,
        decision,
        &format!("{path}.complete_through"),
    )?;
    let from = interval(&coverage.coverage_from)?;
    let through = interval(&coverage.complete_through)?;
    if from.latest_epoch_seconds > through.earliest_epoch_seconds {
        return Err(format!("{path} coverage interval is reversed or indeterminate"));
    }
    if !is_sha256(&coverage.coverage_proof_sha256) {
        return Err(format!("{path}.coverage_proof_sha256 is invalid"));
    }
    if !matches!(coverage.completeness.as_str(), "complete" | "partial" | "unknown") {
        return Err(format!("{path}.completeness is unsupported"));
    }
    if !matches!(
        coverage.verification_status.as_str(),
        "verified" | "failed" | "unknown" | "not_checked" | "disputed"
    ) {
        return Err(format!("{path}.verification_status is unsupported"));
    }
    for (field, value) in [
        ("verifier_build_sha256", coverage.verifier_build_sha256.as_deref()),
        ("verification_policy_sha256", coverage.verification_policy_sha256.as_deref()),
    ] {
        if let Some(value) = value {
            if !is_sha256(value) {
                return Err(format!("{path}.{field} is invalid"));
            }
        }
    }
    if coverage.verification_status == "failed" {
        return Err(format!("{path} coverage proof verification failed"));
    }
    if coverage.verification_status == "verified"
        && (coverage.verifier_build_sha256.as_deref()
            != Some(decision.time_verifier_build_sha256.as_str())
            || coverage.verification_policy_sha256.as_deref()
                != Some(decision.time_verification_policy_sha256.as_str()))
    {
        return Err(format!("{path} uses an unpinned coverage verifier build or policy"));
    }
    Ok(())
}

fn validate_verifier_pin(
    bound: &TimeBound,
    decision: &DecisionRef,
    path: &str,
) -> Result<(), String> {
    if bound.verification_status == "verified"
        && (bound.verifier_build_sha256.as_deref()
            != Some(decision.time_verifier_build_sha256.as_str())
            || bound.verification_policy_sha256.as_deref()
                != Some(decision.time_verification_policy_sha256.as_str()))
    {
        return Err(format!("{path} uses an unpinned time verifier build or policy"));
    }
    Ok(())
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

// v0.1 deliberately accepts normalized UTC RFC 3339 timestamps at whole-second
// precision only. Offsets, fractional seconds and leap seconds are rejected
// rather than guessed or lexicographically compared.
fn parse_utc_seconds(value: &str) -> Result<i64, String> {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return Err("expected YYYY-MM-DDTHH:MM:SSZ (UTC, whole seconds)".to_string());
    }

    let parse_part = |start: usize, end: usize| -> Result<i64, String> {
        let part = &bytes[start..end];
        if !part.iter().all(|byte| byte.is_ascii_digit()) {
            return Err("timestamp contains a non-digit component".to_string());
        }
        let part = std::str::from_utf8(part)
            .map_err(|_| "timestamp component is not ASCII".to_string())?;
        part.parse::<i64>()
            .map_err(|_| "timestamp component is out of range".to_string())
    };

    let year = parse_part(0, 4)?;
    let month = parse_part(5, 7)?;
    let day = parse_part(8, 10)?;
    let hour = parse_part(11, 13)?;
    let minute = parse_part(14, 16)?;
    let second = parse_part(17, 19)?;

    if year < 1970
        || !(1..=12).contains(&month)
        || day < 1
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err("timestamp is not a supported Gregorian UTC instant".to_string());
    }

    let adjusted_year = year - if month <= 2 { 1 } else { 0 };
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era =
        year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146_097 + day_of_era - 719_468;

    Ok(days_since_epoch * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn interval(bound: &TimeBound) -> Result<TimeInterval, String> {
    let center = parse_utc_seconds(&bound.time_utc)?;
    let before = i64::try_from(bound.uncertainty_before_seconds)
        .map_err(|_| "uncertainty_before_seconds is out of range")?;
    let after = i64::try_from(bound.uncertainty_after_seconds)
        .map_err(|_| "uncertainty_after_seconds is out of range")?;
    Ok(TimeInterval {
        earliest_epoch_seconds: center - before,
        latest_epoch_seconds: center + after,
    })
}

fn availability_is_relying_party_access(proof_kind: &str, basis: &str) -> bool {
    matches!(
        (proof_kind, basis),
        ("relying_party_ingest_receipt", "authenticated_ingest_log")
            | ("authenticated_access_log", "authenticated_ingest_log")
            | ("sealed_cache_manifest", "trusted_clock_attestation")
            | ("complete_first_availability_ledger", "authenticated_ingest_log")
    )
}

fn classify(
    snapshot: &Snapshot,
    item: &EvidenceItem,
    cutoff: Option<&TimeInterval>,
) -> Result<(DecisionStatus, String, Option<TimeInterval>), String> {
    let Some(cutoff) = cutoff else {
        return Ok((
            DecisionStatus::Unknown,
            "cutoff_time_unverified".to_string(),
            None,
        ));
    };

    let Some(availability) = &item.availability_evidence else {
        return Ok((
            DecisionStatus::Unknown,
            "missing_availability_evidence".to_string(),
            None,
        ));
    };

    if availability.time.verification_status == "failed" {
        return Err(format!(
            "availability proof verification failed for {}",
            item.evidence_id
        ));
    }

    if availability.relying_party_id != snapshot.decision.relying_party_id
        || availability.evidence_channel_id != snapshot.decision.evidence_channel_id {
        return Ok((
            DecisionStatus::Unknown,
            "availability_scope_mismatch".to_string(),
            None,
        ));
    }

    if !availability_is_relying_party_access(
        &availability.proof_kind,
        &availability.time.basis,
    ) {
        let reason = match availability.proof_kind.as_str() {
            "scitt_receipt" | "rfc3161_timestamp_token" => {
                "proof_establishes_existence_not_relying_party_access"
            }
            "source_assertion" => "source_assertion_does_not_prove_availability",
            _ => "unsupported_availability_proof_kind",
        };
        return Ok((
            DecisionStatus::Unknown,
            reason.to_string(),
            None,
        ));
    }

    match availability.time.verification_status.as_str() {
        "failed" => {
            return Err(format!(
                "availability proof verification failed for {}",
                item.evidence_id
            ));
        }
        "verified" => {}
        _ => {
            return Ok((
                DecisionStatus::Unknown,
                "availability_time_not_independently_verified".to_string(),
                None,
            ));
        }
    }

    let available = interval(&availability.time)?;
    let status;
    let reason;
    if available.latest_epoch_seconds <= cutoff.earliest_epoch_seconds {
        status = DecisionStatus::Included;
        reason = "verified_available_before_cutoff";
    } else if available.earliest_epoch_seconds > cutoff.latest_epoch_seconds {
        if availability.proof_kind != "complete_first_availability_ledger" {
            status = DecisionStatus::Unknown;
            reason = "post_cutoff_access_does_not_prove_prior_absence";
        } else if let Some(coverage) = &availability.coverage {
            if coverage.verification_status == "failed" {
                return Err(format!(
                    "availability coverage verification failed for {}",
                    item.evidence_id
                ));
            }
            let coverage_from = interval(&coverage.coverage_from)?;
            let coverage_through = interval(&coverage.complete_through)?;
            if coverage.relying_party_id != snapshot.decision.relying_party_id
                || coverage.evidence_channel_id != snapshot.decision.evidence_channel_id
            {
                status = DecisionStatus::Unknown;
                reason = "availability_coverage_scope_mismatch";
            } else if coverage.verification_status != "verified"
                || coverage.completeness != "complete"
                || coverage.coverage_from.verification_status != "verified"
                || coverage.complete_through.verification_status != "verified"
                || coverage.coverage_from.basis != "authenticated_ingest_log"
                || coverage.complete_through.basis != "authenticated_ingest_log"
            {
                status = DecisionStatus::Unknown;
                reason = "availability_coverage_not_complete_and_verified";
            } else if coverage_from.latest_epoch_seconds > cutoff.earliest_epoch_seconds
                || coverage_through.earliest_epoch_seconds < available.latest_epoch_seconds
            {
                status = DecisionStatus::Unknown;
                reason = "availability_coverage_does_not_span_cutoff_to_first_access";
            } else {
                status = DecisionStatus::ExcludedAfterCutoff;
                reason = "complete_channel_history_proves_first_availability_after_cutoff";
            }
        } else {
            status = DecisionStatus::Unknown;
            reason = "availability_coverage_missing";
        }
    } else {
        status = DecisionStatus::Unknown;
        reason = "availability_and_cutoff_intervals_overlap";
    }

    Ok((status, reason.to_string(), Some(available)))
}

fn audit(snapshot: &Snapshot, raw: &[u8]) -> Result<Report, String> {
    validate_snapshot(snapshot)?;

    if snapshot.decision_cutoff.verification_status == "failed" {
        return Err("decision cutoff trust verification failed".to_string());
    }
    let cutoff_trusted = snapshot.decision_cutoff.basis == "trusted_clock_attestation"
        && snapshot.decision_cutoff.verification_status == "verified";
    let cutoff_interval = if cutoff_trusted {
        Some(interval(&snapshot.decision_cutoff)?)
    } else {
        None
    };

    let mut classifications = Vec::with_capacity(snapshot.evidence.len());
    let mut candidate_as_known = Vec::new();
    let mut later = Vec::new();
    let mut unresolved = Vec::new();
    let mut included = 0usize;
    let mut excluded = 0usize;
    let mut unknown = 0usize;

    for item in &snapshot.evidence {
        let (status, reason, availability_interval) =
            classify(snapshot, item, cutoff_interval.as_ref())?;

        let observed_before = item
            .observed_at
            .as_ref()
            .and_then(|observed| {
                let observed_epoch = parse_utc_seconds(&observed.time_utc).ok()?;
                let cutoff = cutoff_interval.as_ref()?;
                Some(observed_epoch < cutoff.earliest_epoch_seconds)
            });

        match status {
            DecisionStatus::Included => {
                included += 1;
                candidate_as_known.push(item.evidence_id.clone());
            }
            DecisionStatus::ExcludedAfterCutoff => {
                excluded += 1;
                later.push(item.evidence_id.clone());
            }
            DecisionStatus::Unknown => {
                unknown += 1;
                unresolved.push(item.evidence_id.clone());
            }
        }

        classifications.push(Classification {
            evidence_id: item.evidence_id.clone(),
            artifact_sha256: item.artifact_sha256.clone(),
            as_known_status: match status {
                DecisionStatus::Included => "candidate_included".to_string(),
                DecisionStatus::ExcludedAfterCutoff => "excluded_after_cutoff".to_string(),
                DecisionStatus::Unknown => "unknown".to_string(),
            },
            reason_code: reason,
            observed_before_cutoff_by_source_clock: observed_before,
            availability_interval,
        });
    }

    let input_sha256 = format!("{:x}", Sha256::digest(raw));
    Ok(Report {
        schema_version: REPORT_SCHEMA.to_string(),
        snapshot_id: snapshot.snapshot_id.clone(),
        input_sha256,
        decision_case_id: snapshot.decision.decision_case_id.clone(),
        decision_cutoff_interval: cutoff_interval,
        classifications,
        candidate_as_known_evidence_ids: candidate_as_known,
        later_available_evidence_ids: later,
        unresolved_evidence_ids: unresolved,
        summary: Summary {
            evidence_count: snapshot.evidence.len(),
            candidate_as_known_count: included,
            excluded_after_cutoff: excluded,
            unknown,
            history_completeness: "not_established".to_string(),
        },
        claim_ceiling: "historical_information_boundary_only".to_string(),
        action_authority: "none".to_string(),
    })
}

fn run() -> Result<u8, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        return Err(format!(
            "usage: {} <snapshot.json> <report.json>",
            args.first().map(String::as_str).unwrap_or("industry-decision-frontier-audit")
        ));
    }

    let raw = fs::read(&args[1])
        .map_err(|error| format!("cannot read snapshot {}: {error}", args[1]))?;
    let snapshot: Snapshot = serde_json::from_slice(&raw)
        .map_err(|error| format!("invalid strict snapshot JSON: {error}"))?;
    let report = audit(&snapshot, &raw)?;
    let has_unknown = report.summary.unknown > 0;
    let serialized = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("cannot serialize report: {error}"))?;
    fs::write(&args[2], serialized)
        .map_err(|error| format!("cannot write report {}: {error}", args[2]))?;

    Ok(if has_unknown { 3 } else { 0 })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("industry-decision-frontier-audit: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(ch: char) -> String {
        std::iter::repeat_n(ch, 64).collect()
    }

    fn time(
        time_utc: &str,
        before: u64,
        after: u64,
        basis: &str,
        status: &str,
    ) -> TimeBound {
        TimeBound {
            time_utc: time_utc.to_string(),
            uncertainty_before_seconds: before,
            uncertainty_after_seconds: after,
            clock_id: "clock:test".to_string(),
            basis: basis.to_string(),
            verification_status: status.to_string(),
            proof_sha256: if status == "verified" { Some(digest('a')) } else { None },
            verifier_build_sha256: if status == "verified" { Some(digest('b')) } else { None },
            verification_policy_sha256: if status == "verified" { Some(digest('c')) } else { None },
        }
    }

    fn evidence(
        id: &str,
        availability_evidence: Option<FirstAvailability>,
        observed_at: Option<TimeBound>,
    ) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            evidence_kind: "measurement".to_string(),
            source_id: "source:test".to_string(),
            artifact_sha256: digest('d'),
            valid_time: None,
            observed_at,
            source_recorded_at: None,
            availability_evidence,
        }
    }

    fn available(
        proof_kind: &str,
        relying_party_id: &str,
        time: TimeBound,
    ) -> FirstAvailability {
        FirstAvailability {
            relying_party_id: relying_party_id.to_string(),
            evidence_channel_id: "channel:ieac-ingest-registry".to_string(),
            proof_kind: proof_kind.to_string(),
            artifact_sha256: digest('d'),
            time,
            coverage: None,
        }
    }

    fn snapshot_with(items: Vec<EvidenceItem>) -> Snapshot {
        Snapshot {
            schema_version: SNAPSHOT_SCHEMA.to_string(),
            snapshot_id: "snapshot:test-001".to_string(),
            decision: DecisionRef {
                decision_case_id: "case:test-001".to_string(),
                decision_event_sha256: digest('1'),
                subject_pseudonym: "subject:test".to_string(),
                relying_party_id: "rp:test".to_string(),
                evidence_channel_id: "channel:ieac-ingest-registry".to_string(),
                profile_sha256: digest('2'),
                policy_sha256: digest('3'),
                evaluator_build_sha256: digest('4'),
                time_verifier_build_sha256: digest('b'),
                time_verification_policy_sha256: digest('c'),
            },
            decision_cutoff: time(
                "2026-11-02T09:10:00Z",
                30,
                30,
                "trusted_clock_attestation",
                "verified",
            ),
            evidence: items,
            claim_ceiling: "historical_information_boundary_only".to_string(),
            action_authority: "none".to_string(),
        }
    }

    #[test]
    fn parses_normalized_utc_and_rejects_offsets_fractional_seconds_and_leap_seconds() {
        assert!(parse_utc_seconds("2026-11-02T09:10:00Z").is_ok());
        assert!(parse_utc_seconds("2026-11-02T11:10:00+02:00").is_err());
        assert!(parse_utc_seconds("2026-11-02T09:10:00.500Z").is_err());
        assert!(parse_utc_seconds("2026-11-02T09:10:60Z").is_err());
    }

    #[test]
    fn validates_gregorian_calendar_dates() {
        assert!(parse_utc_seconds("2024-02-29T00:00:00Z").is_ok());
        assert!(parse_utc_seconds("2025-02-29T00:00:00Z").is_err());
        assert!(parse_utc_seconds("2026-13-01T00:00:00Z").is_err());
    }

    #[test]
    fn includes_artifact_when_accessibility_interval_is_before_cutoff_interval() {
        let item = evidence(
            "before",
            Some(available(
                "relying_party_ingest_receipt",
                "rp:test",
                time(
                    "2026-11-02T09:08:00Z",
                    10,
                    10,
                    "authenticated_ingest_log",
                    "verified",
                ),
            )),
            Some(time("2026-11-02T09:00:00Z", 0, 0, "source_assertion", "not_checked")),
        );
        let snapshot = snapshot_with(vec![item]);
        let raw = br#"{}"#;
        let report = audit(&snapshot, raw).expect("audit should succeed");
        assert_eq!(report.summary.candidate_as_known_count, 1);
        assert_eq!(report.summary.unknown, 0);
    }

    #[test]
    fn excludes_late_evidence_even_when_source_says_it_was_observed_earlier() {
        let item = evidence(
            "late-observation",
            Some(available(
                "authenticated_access_log",
                "rp:test",
                time(
                    "2026-11-02T09:16:00Z",
                    5,
                    5,
                    "authenticated_ingest_log",
                    "verified",
                ),
            )),
            Some(time("2026-11-02T08:50:00Z", 0, 0, "source_assertion", "not_checked")),
        );
        let snapshot = snapshot_with(vec![item]);
        let report = audit(&snapshot, br#"{}"#).expect("audit should succeed");
        assert_eq!(report.summary.unknown, 1);
        assert_eq!(
            report.classifications[0].reason_code,
            "post_cutoff_access_does_not_prove_prior_absence"
        );
    }

    #[test]
    fn complete_first_availability_ledger_can_prove_late_arrival() {
        let mut availability = available(
            "complete_first_availability_ledger",
            "rp:test",
            time(
                "2026-11-02T09:25:00Z",
                5,
                5,
                "authenticated_ingest_log",
                "verified",
            ),
        );
        availability.coverage = Some(CoverageProof {
            relying_party_id: "rp:test".to_string(),
            evidence_channel_id: "channel:ieac-ingest-registry".to_string(),
            coverage_from: time(
                "2026-11-02T08:00:00Z", 0, 0, "authenticated_ingest_log", "verified"
            ),
            complete_through: time(
                "2026-11-02T09:30:00Z", 0, 0, "authenticated_ingest_log", "verified"
            ),
            coverage_proof_sha256: digest('e'),
            completeness: "complete".to_string(),
            verification_status: "verified".to_string(),
            verifier_build_sha256: Some(digest('b')),
            verification_policy_sha256: Some(digest('c')),
        });
        let item = evidence("complete-late", Some(availability), None);
        let report = audit(&snapshot_with(vec![item]), br#"{}"#)
            .expect("audit should succeed");
        assert_eq!(report.summary.excluded_after_cutoff, 1);
        assert_eq!(
            report.classifications[0].reason_code,
            "complete_channel_history_proves_first_availability_after_cutoff"
        );
    }

    #[test]
    fn incomplete_first_availability_coverage_cannot_exclude_as_late() {
        let mut availability = available(
            "complete_first_availability_ledger",
            "rp:test",
            time(
                "2026-11-02T09:25:00Z",
                5,
                5,
                "authenticated_ingest_log",
                "verified",
            ),
        );
        availability.coverage = Some(CoverageProof {
            relying_party_id: "rp:test".to_string(),
            evidence_channel_id: "channel:ieac-ingest-registry".to_string(),
            coverage_from: time(
                "2026-11-02T09:09:45Z", 0, 0, "authenticated_ingest_log", "verified"
            ),
            complete_through: time(
                "2026-11-02T09:30:00Z", 0, 0, "authenticated_ingest_log", "verified"
            ),
            coverage_proof_sha256: digest('e'),
            completeness: "complete".to_string(),
            verification_status: "verified".to_string(),
            verifier_build_sha256: Some(digest('b')),
            verification_policy_sha256: Some(digest('c')),
        });
        let item = evidence("coverage-starts-too-late", Some(availability), None);
        let report = audit(&snapshot_with(vec![item]), br#"{}"#)
            .expect("audit should succeed");
        assert_eq!(report.summary.unknown, 1);
        assert_eq!(
            report.classifications[0].reason_code,
            "availability_coverage_does_not_span_cutoff_to_first_access"
        );
    }

    #[test]
    fn scitt_and_rfc3161_proofs_do_not_imply_relying_party_access() {
        for (proof_kind, basis) in [
            ("scitt_receipt", "scitt_receipt"),
            ("rfc3161_timestamp_token", "rfc3161_timestamp_token"),
            ("source_assertion", "source_assertion"),
        ] {
            let item = evidence(
                proof_kind,
                Some(available(
                    proof_kind,
                    "rp:test",
                    time("2026-11-02T09:00:00Z", 0, 0, basis, "verified"),
                )),
                None,
            );
            let report = audit(&snapshot_with(vec![item]), br#"{}"#)
                .expect("audit should succeed");
            assert_eq!(report.summary.unknown, 1, "{proof_kind}");
        }
    }

    #[test]
    fn overlapping_clock_intervals_are_unknown() {
        let item = evidence(
            "overlap",
            Some(available(
                "relying_party_ingest_receipt",
                "rp:test",
                time(
                    "2026-11-02T09:10:00Z",
                    90,
                    90,
                    "authenticated_ingest_log",
                    "verified",
                ),
            )),
            None,
        );
        let report = audit(&snapshot_with(vec![item]), br#"{}"#)
            .expect("audit should succeed");
        assert_eq!(report.summary.unknown, 1);
        assert_eq!(
            report.classifications[0].reason_code,
            "availability_and_cutoff_intervals_overlap"
        );
    }

    #[test]
    fn missing_availability_and_scope_mismatch_are_unknown() {
        let missing = evidence("missing", None, None);
        let wrong_scope = evidence(
            "wrong-scope",
            Some(available(
                "relying_party_ingest_receipt",
                "rp:other",
                time(
                    "2026-11-02T09:00:00Z",
                    0,
                    0,
                    "authenticated_ingest_log",
                    "verified",
                ),
            )),
            None,
        );
        let report = audit(&snapshot_with(vec![missing, wrong_scope]), br#"{}"#)
            .expect("audit should succeed");
        assert_eq!(report.summary.unknown, 2);
    }

    #[test]
    fn failed_availability_verification_is_invalid() {
        let item = evidence(
            "invalid-proof",
            Some(available(
                "relying_party_ingest_receipt",
                "rp:test",
                time(
                    "2026-11-02T09:00:00Z",
                    0,
                    0,
                    "authenticated_ingest_log",
                    "failed",
                ),
            )),
            None,
        );
        let result = audit(&snapshot_with(vec![item]), br#"{}"#);
        assert!(result.is_err());
    }

    #[test]
    fn artifact_digest_must_match_the_availability_receipt_binding() {
        let mut first = available(
            "relying_party_ingest_receipt",
            "rp:test",
            time(
                "2026-11-02T09:00:00Z",
                0,
                0,
                "authenticated_ingest_log",
                "verified",
            ),
        );
        first.artifact_sha256 = digest('e');
        let item = evidence("substitution", Some(first), None);
        let result = audit(&snapshot_with(vec![item]), br#"{}"#);
        assert!(result.is_err());
    }

    #[test]
    fn verified_time_facts_must_use_the_pinned_verifier_and_policy() {
        let mut item = evidence(
            "wrong-verifier",
            Some(available(
                "relying_party_ingest_receipt",
                "rp:test",
                time(
                    "2026-11-02T09:00:00Z",
                    0,
                    0,
                    "authenticated_ingest_log",
                    "verified",
                ),
            )),
            None,
        );
        item.availability_evidence.as_mut().unwrap().time.verifier_build_sha256 = Some(digest('9'));
        let result = audit(&snapshot_with(vec![item]), br#"{}"#);
        assert!(result.is_err());
    }

    #[test]
    fn reversed_source_valid_time_interval_is_rejected() {
        let mut item = evidence("reversed-valid-time", None, None);
        item.valid_time = Some(ValidTime {
            valid_from: Some("2026-11-02T10:00:00Z".to_string()),
            valid_until: Some("2026-11-02T09:00:00Z".to_string()),
        });
        let result = audit(&snapshot_with(vec![item]), br#"{}"#);
        assert!(result.is_err());
    }

    #[test]
    fn required_nullable_fields_must_be_present_even_when_null() {
        let raw = br#"{
          "time_utc":"2026-11-02T09:10:00Z",
          "uncertainty_before_seconds":0,
          "uncertainty_after_seconds":0,
          "clock_id":"clock:test",
          "basis":"source_assertion",
          "verification_status":"not_checked",
          "verifier_build_sha256":null,
          "verification_policy_sha256":null
        }"#;
        assert!(serde_json::from_slice::<TimeBound>(raw).is_err());
    }

    #[test]
    fn duplicate_json_struct_fields_are_rejected_by_serde() {
        let raw = br#"{"schema_version":"one","schema_version":"two"}"#;
        let result = serde_json::from_slice::<Snapshot>(raw);
        assert!(result.is_err());
    }

    #[test]
    fn untrusted_cutoff_makes_every_item_unknown() {
        let mut snapshot = snapshot_with(vec![evidence("evidence", None, None)]);
        snapshot.decision_cutoff.verification_status = "not_checked".to_string();
        snapshot.decision_cutoff.proof_sha256 = None;
        snapshot.decision_cutoff.verifier_build_sha256 = None;
        snapshot.decision_cutoff.verification_policy_sha256 = None;
        let report = audit(&snapshot, br#"{}"#).expect("audit should succeed");
        assert_eq!(report.summary.unknown, 1);
        assert_eq!(report.classifications[0].reason_code, "cutoff_time_unverified");
    }

    #[test]
    fn duplicate_evidence_ids_are_rejected() {
        let first = evidence("same", None, None);
        let second = evidence("same", None, None);
        let result = audit(&snapshot_with(vec![first, second]), br#"{}"#);
        assert!(result.is_err());
    }
}
