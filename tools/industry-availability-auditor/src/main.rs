use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::process::ExitCode;

const REPORT_SCHEMA: &str = "luminous.ieac-availability-audit-report.v0.1";

fn get<'a>(value: &'a Value, path: &str) -> Result<&'a Value, String> {
    let mut current = value;
    for part in path.split('.') {
        current = current
            .get(part)
            .ok_or_else(|| format!("missing required field: {path}"))?;
    }
    Ok(current)
}

fn string<'a>(value: &'a Value, path: &str) -> Result<&'a str, String> {
    get(value, path)?
        .as_str()
        .ok_or_else(|| format!("field is not a string: {path}"))
}

fn integer(value: &Value, path: &str) -> Result<u64, String> {
    get(value, path)?
        .as_u64()
        .ok_or_else(|| format!("field is not a non-negative integer: {path}"))
}

fn boolean(value: &Value, path: &str) -> Result<bool, String> {
    get(value, path)?
        .as_bool()
        .ok_or_else(|| format!("field is not a boolean: {path}"))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

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
        return Err("expected normalized UTC YYYY-MM-DDTHH:MM:SSZ".to_string());
    }

    let parse_part = |start: usize, end: usize| -> Result<i64, String> {
        let part = &bytes[start..end];
        if !part.iter().all(|byte| byte.is_ascii_digit()) {
            return Err("timestamp contains a non-digit component".to_string());
        }
        let part = std::str::from_utf8(part)
            .map_err(|_| "timestamp contains a non-ASCII component".to_string())?;
        part.parse::<i64>()
            .map_err(|_| "timestamp component is out of range".to_string())
    };

    let year = parse_part(0, 4)?;
    let month = parse_part(5, 7)?;
    let day = parse_part(8, 10)?;
    let hour = parse_part(11, 13)?;
    let minute = parse_part(14, 16)?;
    let second = parse_part(17, 19)?;

    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year < 1970
        || day < 1
        || day > month_days
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Interval {
    earliest: i64,
    latest: i64,
}

fn time_interval(value: &Value, path: &str) -> Result<Interval, String> {
    let time = string(value, &format!("{path}.time_utc"))?;
    let center = parse_utc_seconds(time)
        .map_err(|error| format!("{path}.time_utc: {error}"))?;
    let before = i64::try_from(integer(
        value,
        &format!("{path}.uncertainty_before_seconds"),
    )?)
    .map_err(|_| format!("{path} uncertainty is out of range"))?;
    let after = i64::try_from(integer(
        value,
        &format!("{path}.uncertainty_after_seconds"),
    )?)
    .map_err(|_| format!("{path} uncertainty is out of range"))?;
    if before > 86_400 || after > 86_400 {
        return Err(format!("{path} uncertainty exceeds one day"));
    }
    Ok(Interval {
        earliest: center - before,
        latest: center + after,
    })
}

fn require_digest(value: &Value, path: &str) -> Result<(), String> {
    let raw = string(value, path)?;
    if !is_sha256(raw) {
        return Err(format!("{path} is not a lowercase SHA-256 digest"));
    }
    Ok(())
}

fn verified_time(
    root: &Value,
    path: &str,
    verifier_build: &str,
    verification_policy: &str,
) -> Result<Interval, String> {
    if string(root, &format!("{path}.verification_status"))? != "verified" {
        return Err(format!("{path} is not independently verified"));
    }
    require_digest(root, &format!("{path}.proof_sha256"))?;
    if string(root, &format!("{path}.verifier_build_sha256"))? != verifier_build {
        return Err(format!("{path} verifier build does not match the attestation pin"));
    }
    if string(root, &format!("{path}.verification_policy_sha256"))? != verification_policy {
        return Err(format!("{path} verification policy does not match the attestation pin"));
    }
    time_interval(root, path)
}

fn same_string(a: &Value, a_path: &str, b: &Value, b_path: &str) -> Result<bool, String> {
    Ok(string(a, a_path)? == string(b, b_path)?)
}

fn complete_or_unknown(
    manifest: &Value,
    attestation: &Value,
    manifest_bytes: &[u8],
    attestation_bytes: &[u8],
) -> Result<Value, String> {
    let manifest_digest = digest(manifest_bytes);
    let attestation_digest = digest(attestation_bytes);
    if string(attestation, "schema_version")?
        != "luminous.ieac-availability-completeness-attestation.v0.1"
    {
        return Err("unsupported availability attestation schema version".to_string());
    }
    if string(manifest, "schema_version")?
        != "luminous.ieac-evidence-channel-manifest.v0.1"
    {
        return Err("unsupported evidence-channel manifest schema version".to_string());
    }
    if string(attestation, "channel_manifest_sha256")? != manifest_digest {
        return Err("attestation does not bind the exact channel-manifest bytes".to_string());
    }

    // Repeated channel identity is checked against the independently supplied manifest.
    for (att_field, manifest_field) in [
        ("channel_identity.relying_party_id", "channel.relying_party_id"),
        ("channel_identity.evidence_channel_id", "channel.evidence_channel_id"),
        ("channel_identity.log_service_id", "channel.log_service_id"),
        ("channel_identity.operator_id", "channel.operator_id"),
        ("channel_identity.ledger_id", "ledger.ledger_id"),
        ("channel_identity.sequence_namespace", "ledger.sequence_namespace"),
        ("channel_identity.sequence_epoch", "ledger.sequence_epoch"),
    ] {
        if !same_string(attestation, att_field, manifest, manifest_field)? {
            return Err(format!(
                "channel identity mismatch: {att_field} versus manifest.{manifest_field}"
            ));
        }
    }

    for (att_field, manifest_field) in [
        ("scope.tenant_scope", "channel.tenant_scope"),
        ("scope.subject_pseudonym", "channel.subject_scope"),
    ] {
        if !same_string(attestation, att_field, manifest, manifest_field)? {
            return Err(format!("scope mismatch: {att_field}"));
        }
    }

    let purpose = string(attestation, "scope.purpose")?;
    let purposes = get(manifest, "channel.purpose_scope")?
        .as_array()
        .ok_or_else(|| "manifest.channel.purpose_scope is not an array".to_string())?;
    if !purposes.iter().any(|item| item.as_str() == Some(purpose)) {
        return Err("attestation purpose is outside the channel manifest scope".to_string());
    }

    let artifact = string(attestation, "scope.artifact_sha256")?;
    if !is_sha256(artifact)
        || string(attestation, "first_access.artifact_sha256")? != artifact
    {
        return Err("first-access event is not bound to the attested artifact digest".to_string());
    }

    let channel_manifest = get(manifest, "capture_contract")?;
    let event_type = string(attestation, "first_access.event_type")?;
    let event_types = get(channel_manifest, "event_types")?
        .as_array()
        .ok_or_else(|| "capture_contract.event_types is not an array".to_string())?;
    if !event_types.iter().any(|item| item.as_str() == Some(event_type)) {
        return Err("first-access event type is not covered by the capture policy".to_string());
    }

    let verifier_build = string(attestation, "verifier.build_sha256")?;
    let verification_policy = string(attestation, "verifier.policy_sha256")?;
    let trust_bundle = string(attestation, "verifier.trust_bundle_sha256")?;
    for (path, value) in [
        ("verifier.build_sha256", verifier_build),
        ("verifier.policy_sha256", verification_policy),
        ("verifier.trust_bundle_sha256", trust_bundle),
    ] {
        if !is_sha256(value) {
            return Err(format!("{path} is not a lowercase SHA-256 digest"));
        }
    }
    if string(attestation, "verifier.status")? != "verified" {
        return Ok(report(
            "unknown",
            "outer verifier facts are not independently verified",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }
    if string(attestation, "decision_cutoff.basis")? != "trusted_clock_attestation" {
        return Ok(report(
            "unknown",
            "decision cutoff is not based on a trusted clock attestation",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }
    if string(manifest, "capture_contract.durability")? != "synchronous_durable"
        || string(manifest, "capture_contract.loss_handling")? != "fail_closed"
        || string(manifest, "capture_contract.policy_change_semantics")? != "append_only_versioned"
        || get(manifest, "capture_contract.instrumented_access_paths")?.as_array().is_none_or(Vec::is_empty)
    {
        return Ok(report(
            "unknown",
            "channel capture policy does not provide complete instrumented, durable, fail-closed capture",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    if string(attestation, "absence_basis")? != "complete_channel_signed_query" {
        return Ok(report(
            "unknown",
            "v0.1 supports only the separately signed complete-channel query path; VDS non-inclusion is not inferred",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }
    if !get(attestation, "absence_proof")?.is_null() {
        return Err("complete-channel query mode must not also claim a VDS non-inclusion proof".to_string());
    }

    // The query must bind the exact scope, not merely a query digest.
    for (query_field, att_field) in [
        ("predicate_binding.relying_party_id", "channel_identity.relying_party_id"),
        ("predicate_binding.evidence_channel_id", "channel_identity.evidence_channel_id"),
        ("predicate_binding.tenant_scope", "scope.tenant_scope"),
        ("predicate_binding.subject_pseudonym", "scope.subject_pseudonym"),
        ("predicate_binding.purpose", "scope.purpose"),
        ("predicate_binding.artifact_sha256", "scope.artifact_sha256"),
    ] {
        if !same_string(attestation, &format!("absence_query.{query_field}"), attestation, att_field)? {
            return Err(format!("absence-query predicate binding mismatch at {query_field}"));
        }
    }
    let predicate_events = get(attestation, "absence_query.predicate_binding.event_types")?
        .as_array()
        .ok_or_else(|| "absence-query event_types is not an array".to_string())?;
    if predicate_events != event_types {
        return Err("absence-query event taxonomy differs from the channel capture policy".to_string());
    }

    let query_verifier = string(attestation, "absence_query.verification_status")?;
    if query_verifier == "failed" {
        return Err("zero-match query verifier reports failure".to_string());
    }
    if query_verifier != "verified"
        || string(attestation, "absence_query.verifier_build_sha256")? != verifier_build
        || string(attestation, "absence_query.verification_policy_sha256")? != verification_policy
    {
        return Ok(report(
            "unknown",
            "zero-match query is not verified under the pinned verifier build and policy",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }
    for path in [
        "absence_query.query_sha256",
        "absence_query.result_sha256",
        "absence_query.signed_statement_sha256",
        "absence_query.signature_sha256",
        "absence_query.receipt_sha256",
    ] {
        require_digest(attestation, path)?;
    }
    let query_signer = string(attestation, "absence_query.signer_id")?;
    if query_signer.is_empty() {
        return Err("absence-query signer identity is empty".to_string());
    }
    let allowed_query_signers = get(
        manifest,
        "capture_contract.authorized_completeness_query_signers",
    )?
    .as_array()
    .ok_or_else(|| "authorized_completeness_query_signers is not an array".to_string())?;
    if !allowed_query_signers
        .iter()
        .any(|allowed| allowed.as_str() == Some(query_signer))
    {
        return Err("zero-match query signer is not authorized by the channel manifest".to_string());
    }
    if integer(attestation, "absence_query.matching_event_count")? != 0 {
        return Ok(report(
            "unknown",
            "the signed query reports one or more matching pre-boundary access events",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    if string(attestation, "continuity.verifier_status")? == "failed" {
        return Err("checkpoint consistency verification failed".to_string());
    }
    if string(attestation, "continuity.verifier_status")? != "verified"
        || string(attestation, "continuity.verifier_build_sha256")? != verifier_build
        || string(attestation, "continuity.verification_policy_sha256")? != verification_policy
    {
        return Ok(report(
            "unknown",
            "checkpoint consistency is not verified under the pinned verifier",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }
    if !matches!(
        string(attestation, "continuity.proof_type")?,
        "rfc9162_sha256_consistency" | "other_registered_vds_consistency"
    ) {
        return Ok(report(
            "unknown",
            "unsupported VDS consistency proof type",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }
    for path in [
        "continuity.start_to_boundary_receipt_sha256",
        "continuity.boundary_to_end_receipt_sha256",
        "first_access.event_sha256",
        "first_access.inclusion_receipt_sha256",
    ] {
        require_digest(attestation, path)?;
    }

    // A complete assertion requires capture policy and health to be closed, not best effort.
    if string(manifest, "capture_contract.durability")? != "synchronous_durable"
        || string(manifest, "capture_contract.loss_handling")? != "fail_closed"
        || string(manifest, "capture_contract.policy_change_semantics")? != "append_only_versioned"
        || !boolean(manifest, "ledger.monotonic_sequence")?
        || !matches!(
            string(manifest, "ledger.reset_semantics")?,
            "authenticated_epoch_transition" | "never_reset"
        )
    {
        return Ok(report(
            "unknown",
            "capture policy does not provide the durability, anti-loss and monotonicity guarantees required for completeness",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    for path in [
        "capture_contract.unmonitored_access_paths",
        "known_bypass_paths",
        "capture_health.known_bypass_paths",
        "capture_health.unmonitored_access_paths",
    ] {
        let values = get(manifest, path)
            .or_else(|_| get(attestation, path))?
            .as_array()
            .ok_or_else(|| format!("{path} is not an array"))?;
        if !values.is_empty() {
            return Ok(report(
                "unknown",
                "declared bypass or unmonitored access path prevents a complete channel-absence claim",
                &manifest_digest,
                &attestation_digest,
                attestation,
            ));
        }
    }

    for path in [
        "capture_health.dropped_events_count",
        "capture_health.sequence_gap_count",
        "capture_health.unresolved_outage_count",
        "capture_health.backfilled_events_count",
        "capture_health.policy_changes_count",
    ] {
        if integer(attestation, path)? != 0 {
            return Ok(report(
                "unknown",
                &format!("{path} is nonzero"),
                &manifest_digest,
                &attestation_digest,
                attestation,
            ));
        }
    }
    if boolean(attestation, "capture_health.retention_loss")?
        || string(attestation, "capture_health.clock_health")? != "healthy"
    {
        return Ok(report(
            "unknown",
            "retention loss or clock degradation prevents completeness",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    // All checkpoint identities must share one ledger/namespace/epoch.
    let checkpoint_paths = [
        "coverage_start_checkpoint",
        "decision_boundary_checkpoint",
        "coverage_end_checkpoint",
    ];
    for path in checkpoint_paths {
        for (field, expected) in [
            ("ledger_id", string(attestation, "channel_identity.ledger_id")?),
            (
                "sequence_namespace",
                string(attestation, "channel_identity.sequence_namespace")?,
            ),
            (
                "sequence_epoch",
                string(attestation, "channel_identity.sequence_epoch")?,
            ),
        ] {
            if string(attestation, &format!("{path}.{field}"))? != expected {
                return Err(format!("{path}.{field} does not match channel identity"));
            }
        }
        for field in ["root_sha256", "signed_bytes_sha256", "receipt_sha256"] {
            require_digest(attestation, &format!("{path}.{field}"))?;
        }
    }

    let start_sequence = integer(attestation, "coverage_start_checkpoint.sequence")?;
    let boundary_sequence = integer(attestation, "decision_boundary_checkpoint.sequence")?;
    let first_sequence = integer(attestation, "first_access.sequence")?;
    let end_sequence = integer(attestation, "coverage_end_checkpoint.sequence")?;
    if !(start_sequence < boundary_sequence
        && boundary_sequence.checked_add(1) == Some(first_sequence)
        && first_sequence <= end_sequence)
    {
        return Ok(report(
            "unknown",
            "checkpoint, zero-match query and first-access sequence order is incomplete",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    let start_tree = integer(attestation, "coverage_start_checkpoint.tree_size")?;
    let boundary_tree = integer(attestation, "decision_boundary_checkpoint.tree_size")?;
    let end_tree = integer(attestation, "coverage_end_checkpoint.tree_size")?;
    if !(start_tree < boundary_tree && boundary_tree < end_tree) {
        return Ok(report(
            "unknown",
            "checkpoint tree sizes do not strictly advance",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    if integer(attestation, "absence_query.sequence_from_exclusive")? != start_sequence
        || integer(attestation, "absence_query.sequence_through_inclusive")? != boundary_sequence
    {
        return Ok(report(
            "unknown",
            "zero-match query does not cover the exact start-to-decision-boundary sequence interval",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    // Pin all time facts to the same verifier and policy.
    let cutoff = verified_time(
        attestation,
        "decision_cutoff",
        verifier_build,
        verification_policy,
    )?;
    for (path, expected_basis) in [
        ("coverage_start_checkpoint.observed_at", "authenticated_ingest_log"),
        ("decision_boundary_checkpoint.observed_at", "authenticated_ingest_log"),
        ("coverage_end_checkpoint.observed_at", "authenticated_ingest_log"),
        ("first_access.observed_at", "authenticated_ingest_log"),
        ("coverage.coverage_from", "authenticated_ingest_log"),
        ("coverage.coverage_through", "authenticated_ingest_log"),
    ] {
        if string(attestation, &format!("{path}.basis"))? != expected_basis {
            return Ok(report(
                "unknown",
                &format!("{path} does not use the authenticated channel clock"),
                &manifest_digest,
                &attestation_digest,
                attestation,
            ));
        }
    }
    let coverage_start = verified_time(
        attestation,
        "coverage_start_checkpoint.observed_at",
        verifier_build,
        verification_policy,
    )?;
    let decision_boundary = verified_time(
        attestation,
        "decision_boundary_checkpoint.observed_at",
        verifier_build,
        verification_policy,
    )?;
    let first_access = verified_time(
        attestation,
        "first_access.observed_at",
        verifier_build,
        verification_policy,
    )?;
    let coverage_end = verified_time(
        attestation,
        "coverage_end_checkpoint.observed_at",
        verifier_build,
        verification_policy,
    )?;
    let coverage_from = verified_time(
        attestation,
        "coverage.coverage_from",
        verifier_build,
        verification_policy,
    )?;
    let coverage_through = verified_time(
        attestation,
        "coverage.coverage_through",
        verifier_build,
        verification_policy,
    )?;

    if coverage_start.latest > cutoff.earliest
        || decision_boundary.earliest <= cutoff.latest
        || first_access.earliest <= decision_boundary.latest
        || coverage_end.earliest < first_access.latest
        || coverage_from.latest > cutoff.earliest
        || coverage_through.earliest < first_access.latest
        || coverage_from.earliest > coverage_start.earliest
        || coverage_through.latest < coverage_end.latest
    {
        return Ok(report(
            "unknown",
            "time intervals do not prove coverage start <= cutoff < decision boundary < first access <= coverage end",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    if string(attestation, "coverage.completeness")? != "complete"
        || string(attestation, "absence_basis")? != "complete_channel_signed_query"
    {
        return Ok(report(
            "unknown",
            "attestation does not declare the supported complete-channel query mode",
            &manifest_digest,
            &attestation_digest,
            attestation,
        ));
    }

    Ok(report(
        "structurally_complete_within_declared_channel",
        "all structural, scope, sequence, time and pinned-verifier-fact checks passed; cryptographic verification remains external",
        &manifest_digest,
        &attestation_digest,
        attestation,
    ))
}

fn report(
    status: &str,
    reason: &str,
    manifest_digest: &str,
    attestation_digest: &str,
    attestation: &Value,
) -> Value {
    json!({
        "schema_version": REPORT_SCHEMA,
        "status": status,
        "reason": reason,
        "manifest_sha256": manifest_digest,
        "attestation_sha256": attestation_digest,
        "attestation_id": attestation.get("attestation_id"),
        "channel_identity": attestation.get("channel_identity"),
        "scope": attestation.get("scope"),
        "checkpoint_sequences": {
            "coverage_start": attestation.pointer("/coverage_start_checkpoint/sequence"),
            "decision_boundary": attestation.pointer("/decision_boundary_checkpoint/sequence"),
            "first_access": attestation.pointer("/first_access/sequence"),
            "coverage_end": attestation.pointer("/coverage_end_checkpoint/sequence")
        },
        "claim_ceiling": "absence_within_declared_access_channel_only",
        "action_authority": "none",
        "universal_non_access_claim": false
    })
}

fn evaluate(
    manifest_bytes: &[u8],
    attestation_bytes: &[u8],
) -> Result<Value, String> {
    let manifest: Value = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("invalid channel manifest JSON: {error}"))?;
    let attestation: Value = serde_json::from_slice(attestation_bytes)
        .map_err(|error| format!("invalid completeness attestation JSON: {error}"))?;
    complete_or_unknown(&manifest, &attestation, manifest_bytes, attestation_bytes)
}

fn run() -> Result<u8, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err(format!(
            "usage: {} <channel-manifest.json> <availability-attestation.json> <report.json>",
            args.first()
                .map(String::as_str)
                .unwrap_or("industry-availability-audit")
        ));
    }
    let manifest_bytes = fs::read(&args[1])
        .map_err(|error| format!("cannot read channel manifest {}: {error}", args[1]))?;
    let attestation_bytes = fs::read(&args[2])
        .map_err(|error| format!("cannot read availability attestation {}: {error}", args[2]))?;
    let report_value = evaluate(&manifest_bytes, &attestation_bytes)?;
    let status = string(&report_value, "status")?.to_string();
    let output = serde_json::to_vec_pretty(&report_value)
        .map_err(|error| format!("cannot serialize audit report: {error}"))?;
    fs::write(&args[3], output)
        .map_err(|error| format!("cannot write audit report {}: {error}", args[3]))?;
    Ok(if status == "structurally_complete_within_declared_channel" {
        0
    } else {
        3
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("industry-availability-audit: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus() -> (Vec<u8>, Value) {
        let manifest = include_str!(
            "../../../schemas/examples/ieac-evidence-channel-manifest-synthetic-v0.1.json"
        );
        let attestation = include_str!(
            "../../../schemas/examples/ieac-availability-completeness-attestation-synthetic-v0.1.json"
        );
        (
            manifest.as_bytes().to_vec(),
            serde_json::from_str(attestation).expect("synthetic attestation must parse"),
        )
    }

    fn evaluate_mutation(manifest: &[u8], attestation: &Value) -> Result<Value, String> {
        let bytes = serde_json::to_vec(attestation).expect("attestation serialization");
        evaluate(manifest, &bytes)
    }

    #[test]
    fn synthetic_complete_channel_is_structurally_accepted() {
        let (manifest, attestation) = corpus();
        let report = evaluate_mutation(
            &manifest,
            &attestation,
        )
        .expect("synthetic fixture should evaluate");
        assert_eq!(
            report["status"],
            "structurally_complete_within_declared_channel"
        );
        assert_eq!(report["claim_ceiling"], "absence_within_declared_access_channel_only");
        assert_eq!(report["action_authority"], "none");
        assert_eq!(report["universal_non_access_claim"], false);
    }

    #[test]
    fn sequence_gap_forces_unknown() {
        let (manifest, mut attestation) = corpus();
        attestation["capture_health"]["sequence_gap_count"] = json!(1);
        let report = evaluate_mutation(&manifest, &attestation).expect("unknown is a valid result");
        assert_eq!(report["status"], "unknown");
    }

    #[test]
    fn bypass_path_forces_unknown() {
        let (manifest, mut attestation) = corpus();
        attestation["capture_health"]["known_bypass_paths"] = json!(["local-cache"]);
        let report = evaluate_mutation(&manifest, &attestation).expect("unknown is a valid result");
        assert_eq!(report["status"], "unknown");
    }

    #[test]
    fn nonzero_query_result_forces_unknown() {
        let (manifest, mut attestation) = corpus();
        attestation["absence_query"]["matching_event_count"] = json!(1);
        let report = evaluate_mutation(&manifest, &attestation).expect("unknown is a valid result");
        assert_eq!(report["status"], "unknown");
    }

    #[test]
    fn query_range_substitution_forces_unknown() {
        let (manifest, mut attestation) = corpus();
        attestation["absence_query"]["sequence_through_inclusive"] = json!(99);
        let report = evaluate_mutation(&manifest, &attestation).expect("unknown is a valid result");
        assert_eq!(report["status"], "unknown");
    }

    #[test]
    fn access_before_decision_boundary_forces_unknown() {
        let (manifest, mut attestation) = corpus();
        attestation["first_access"]["sequence"] = json!(109);
        let report = evaluate_mutation(&manifest, &attestation).expect("unknown is a valid result");
        assert_eq!(report["status"], "unknown");
    }

    #[test]
    fn skipped_sequence_between_boundary_and_first_access_forces_unknown() {
        let (manifest, mut attestation) = corpus();
        attestation["first_access"]["sequence"] = json!(111);
        let report = evaluate_mutation(&manifest, &attestation).expect("unknown is a valid result");
        assert_eq!(report["status"], "unknown");
    }

    #[test]
    fn decision_boundary_not_after_cutoff_forces_unknown() {
        let (manifest, mut attestation) = corpus();
        attestation["decision_boundary_checkpoint"]["observed_at"]["time_utc"] =
            json!("2026-11-02T09:10:00Z");
        let report = evaluate_mutation(&manifest, &attestation).expect("unknown is a valid result");
        assert_eq!(report["status"], "unknown");
    }

    #[test]
    fn manifest_digest_substitution_is_invalid() {
        let (manifest, mut attestation) = corpus();
        attestation["channel_manifest_sha256"] = json!("0".repeat(64));
        let result = evaluate_mutation(&manifest, &attestation);
        assert!(result.is_err());
    }

    #[test]
    fn artifact_predicate_substitution_is_invalid() {
        let (manifest, mut attestation) = corpus();
        attestation["absence_query"]["predicate_binding"]["artifact_sha256"] =
            json!("0".repeat(64));
        let result = evaluate_mutation(&manifest, &attestation);
        assert!(result.is_err());
    }

    #[test]
    fn unsupported_non_inclusion_mode_is_unknown() {
        let (manifest, mut attestation) = corpus();
        attestation["absence_basis"] = json!("registered_vds_non_inclusion");
        attestation["absence_query"] = Value::Null;
        attestation["absence_proof"] = json!({
            "vds_id":"rfc9162-sha256",
            "proof_type":"non-inclusion",
            "proof_sha256":"1".repeat(64),
            "receipt_sha256":"2".repeat(64),
            "verifier_status":"not_supported",
            "verifier_build_sha256":null,
            "verification_policy_sha256":null
        });
        let report = evaluate_mutation(&manifest, &attestation).expect("unknown is a valid result");
        assert_eq!(report["status"], "unknown");
    }
}
