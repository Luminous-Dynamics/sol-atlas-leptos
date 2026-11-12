use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

const SCHEMA_VERSION: &str = "0.1.0";
const BENCHMARK_ID: &str = "GMD-EVIDENCE-CAPABILITY-001";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Corpus {
    schema_version: String,
    benchmark_id: String,
    fixture_notice: String,
    synthetic_only: bool,
    claim_ceiling: String,
    evaluation_time_unix_seconds: u64,
    trust_store_artifact: ArtifactRef,
    claim: RequiredClaim,
    attestation: AttestationEnvelope,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustStore {
    schema_version: String,
    trust_store_id: String,
    issuers: Vec<TrustedIssuer>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TrustStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum IssuerRole {
    Laboratory,
    Utility,
    Manufacturer,
    ResearchGroup,
    FieldOperator,
    IndependentAuditor,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustedIssuer {
    issuer_id: String,
    trust_status: TrustStatus,
    allowed_roles: Vec<IssuerRole>,
    independence_assessed: bool,
    public_key_artifact: ArtifactRef,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CapabilityType {
    WaveformResolved,
    PhysicalRelayObserved,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum QualificationClass {
    SyntheticContract,
    SoftwareModel,
    PhysicalHardwareTest,
    FieldExercise,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequiredClaim {
    claim_id: String,
    claim_type: CapabilityType,
    subject_kind: String,
    subject_id: String,
    configuration_sha256: String,
    scope_sha256: String,
    accepted_qualification_classes: Vec<QualificationClass>,
    required_issuer_roles: Vec<IssuerRole>,
    require_independent: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttestationEnvelope {
    attestation_id: String,
    claim_id: String,
    payload_artifact: ArtifactRef,
    signature_artifact: ArtifactRef,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttestationPayload {
    schema_version: String,
    attestation_id: String,
    claim_id: String,
    issuer_id: String,
    issuer_role: IssuerRole,
    claim_type: CapabilityType,
    qualification_class: QualificationClass,
    subject: Subject,
    scope_sha256: String,
    evidence_artifact: ArtifactRef,
    method: String,
    protocol_revision: String,
    issued_at_unix_seconds: u64,
    expires_at_unix_seconds: u64,
    independent: bool,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Subject {
    subject_kind: String,
    subject_id: String,
    configuration_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactRef {
    path: String,
    sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CapabilityStatus {
    Satisfied,
    Unknown,
    Rejected,
}

#[derive(Debug, Serialize)]
struct Report {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    input_sha256: String,
    trust_store_sha256: String,
    synthetic_only: bool,
    claim_ceiling: String,
    evaluation_time_unix_seconds: u64,
    issuer_id: String,
    signature_verified: bool,
    issuer_trusted: bool,
    time_valid: bool,
    evidence_integrity_verified: bool,
    capability_status: CapabilityStatus,
    reason: String,
    limitations: Vec<&'static str>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn resolve_path(reference: &ArtifactRef, root: &Path) -> Result<PathBuf, String> {
    if !valid_sha256(&reference.sha256) || !safe_relative_path(&reference.path) {
        return Err(format!("invalid artifact reference {}", reference.path));
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("cannot resolve evidence root: {error}"))?;
    let canonical = canonical_root
        .join(&reference.path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve artifact {}: {error}", reference.path))?;
    if !canonical.starts_with(&canonical_root) || !canonical.is_file() {
        return Err(format!("artifact {} escapes evidence root or is not a file", reference.path));
    }
    Ok(canonical)
}

fn verify_artifact(reference: &ArtifactRef, root: &Path) -> Result<Vec<u8>, String> {
    let path = resolve_path(reference, root)?;
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read artifact {}: {error}", reference.path))?;
    let actual = sha256(&bytes);
    if actual != reference.sha256 {
        return Err(format!(
            "artifact digest mismatch for {}: expected {}, got {}",
            reference.path, reference.sha256, actual
        ));
    }
    Ok(bytes)
}

fn verify_ed25519(
    public_key: &ArtifactRef,
    payload: &ArtifactRef,
    signature: &ArtifactRef,
    root: &Path,
) -> Result<bool, String> {
    let key_path = resolve_path(public_key, root)?;
    let payload_path = resolve_path(payload, root)?;
    let signature_path = resolve_path(signature, root)?;
    let output = Command::new("openssl")
        .args(["pkeyutl", "-verify", "-pubin", "-inkey"])
        .arg(key_path)
        .args(["-keyform", "DER", "-rawin", "-in"])
        .arg(payload_path)
        .arg("-sigfile")
        .arg(signature_path)
        .output()
        .map_err(|error| format!("OpenSSL Ed25519 verifier unavailable: {error}"))?;
    Ok(output.status.success())
}

fn validate(corpus: &Corpus, trust: &TrustStore) -> Result<(), String> {
    if corpus.schema_version != SCHEMA_VERSION
        || corpus.benchmark_id != BENCHMARK_ID
        || trust.schema_version != SCHEMA_VERSION
    {
        return Err("unsupported schema or benchmark version".to_string());
    }
    if !corpus.synthetic_only
        || corpus.claim_ceiling != "E2"
        || !corpus.fixture_notice.to_ascii_uppercase().contains("ALL VALUES ARE SYNTHETIC")
    {
        return Err("v0.1 accepts synthetic E2 fixtures only".to_string());
    }
    if trust.trust_store_id.trim().is_empty() || trust.issuers.is_empty() {
        return Err("trust store must identify at least one issuer".to_string());
    }
    if corpus.claim.claim_id.trim().is_empty()
        || corpus.claim.subject_kind.trim().is_empty()
        || corpus.claim.subject_id.trim().is_empty()
        || !valid_sha256(&corpus.claim.configuration_sha256)
        || !valid_sha256(&corpus.claim.scope_sha256)
        || corpus.claim.accepted_qualification_classes.is_empty()
        || corpus.claim.required_issuer_roles.is_empty()
    {
        return Err("required claim is incomplete".to_string());
    }
    if corpus.attestation.claim_id != corpus.claim.claim_id
        || corpus.attestation.attestation_id.trim().is_empty()
    {
        return Err("attestation envelope does not identify the required claim".to_string());
    }
    let mut issuers = BTreeSet::new();
    for issuer in &trust.issuers {
        if issuer.issuer_id.trim().is_empty()
            || !issuers.insert(issuer.issuer_id.as_str())
            || issuer.allowed_roles.is_empty()
            || !valid_sha256(&issuer.public_key_artifact.sha256)
            || !safe_relative_path(&issuer.public_key_artifact.path)
        {
            return Err(format!("invalid or duplicate trusted issuer {}", issuer.issuer_id));
        }
    }
    Ok(())
}

fn assess(
    corpus: &Corpus,
    trust: &TrustStore,
    payload: &AttestationPayload,
    signature_verified: bool,
    issuer: &TrustedIssuer,
) -> (CapabilityStatus, String) {
    if !signature_verified {
        return (CapabilityStatus::Rejected, "Ed25519 signature verification failed".to_string());
    }
    if issuer.trust_status == TrustStatus::Revoked {
        return (CapabilityStatus::Rejected, "issuer is revoked".to_string());
    }
    if !issuer.allowed_roles.contains(&payload.issuer_role) {
        return (CapabilityStatus::Rejected, "issuer role is not trusted".to_string());
    }
    if payload.issued_at_unix_seconds > corpus.evaluation_time_unix_seconds {
        return (CapabilityStatus::Rejected, "attestation is not yet valid".to_string());
    }
    if corpus.evaluation_time_unix_seconds >= payload.expires_at_unix_seconds {
        return (CapabilityStatus::Rejected, "attestation is expired".to_string());
    }
    if payload.claim_type != corpus.claim.claim_type
        || payload.subject.subject_kind != corpus.claim.subject_kind
        || payload.subject.subject_id != corpus.claim.subject_id
        || payload.subject.configuration_sha256 != corpus.claim.configuration_sha256
        || payload.scope_sha256 != corpus.claim.scope_sha256
    {
        return (CapabilityStatus::Unknown, "claim subject, configuration or scope mismatch".to_string());
    }
    if !corpus.claim.accepted_qualification_classes.contains(&payload.qualification_class) {
        return (
            CapabilityStatus::Unknown,
            format!("qualification class {:?} is insufficient", payload.qualification_class),
        );
    }
    if !corpus.claim.required_issuer_roles.contains(&payload.issuer_role) {
        return (CapabilityStatus::Unknown, "issuer role does not meet claim requirement".to_string());
    }
    if corpus.claim.require_independent && (!payload.independent || !issuer.independence_assessed) {
        return (CapabilityStatus::Unknown, "independent evidence is required but not established".to_string());
    }
    let _ = trust;
    (
        CapabilityStatus::Satisfied,
        "signature, trust, validity, evidence and exact claim scope verified".to_string(),
    )
}

fn verify(
    corpus: &Corpus,
    trust: &TrustStore,
    input_bytes: &[u8],
    trust_bytes: &[u8],
    root: &Path,
) -> Result<Report, String> {
    validate(corpus, trust)?;
    let pinned_trust_bytes = verify_artifact(&corpus.trust_store_artifact, root)?;
    if sha256(&pinned_trust_bytes) != sha256(trust_bytes) {
        return Err("trust store bytes do not match parsed trust-store input".to_string());
    }
    let payload_bytes = verify_artifact(&corpus.attestation.payload_artifact, root)?;
    verify_artifact(&corpus.attestation.signature_artifact, root)?;
    let payload: AttestationPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|error| format!("signed payload is not valid typed JSON: {error}"))?;
    if payload.schema_version != SCHEMA_VERSION
        || payload.attestation_id != corpus.attestation.attestation_id
        || payload.claim_id != corpus.attestation.claim_id
        || payload.issuer_id.trim().is_empty()
        || payload.method.trim().is_empty()
        || payload.protocol_revision.trim().is_empty()
        || payload.limitations.is_empty()
        || payload.expires_at_unix_seconds <= payload.issued_at_unix_seconds
        || !valid_sha256(&payload.subject.configuration_sha256)
        || !valid_sha256(&payload.scope_sha256)
    {
        return Err("signed payload metadata is incomplete or mismatched".to_string());
    }
    verify_artifact(&payload.evidence_artifact, root)?;
    let issuer = trust.issuers.iter()
        .find(|issuer| issuer.issuer_id == payload.issuer_id)
        .ok_or_else(|| "issuer is not present in trust store".to_string())?;
    verify_artifact(&issuer.public_key_artifact, root)?;
    let signature_verified = verify_ed25519(
        &issuer.public_key_artifact,
        &corpus.attestation.payload_artifact,
        &corpus.attestation.signature_artifact,
        root,
    )?;
    let (capability_status, reason) = assess(
        corpus,
        trust,
        &payload,
        signature_verified,
        issuer,
    );
    Ok(Report {
        status: "valid_synthetic_attestation_contract",
        benchmark_id: corpus.benchmark_id.clone(),
        schema_version: corpus.schema_version.clone(),
        input_sha256: sha256(input_bytes),
        trust_store_sha256: sha256(trust_bytes),
        synthetic_only: true,
        claim_ceiling: corpus.claim_ceiling.clone(),
        evaluation_time_unix_seconds: corpus.evaluation_time_unix_seconds,
        issuer_id: payload.issuer_id,
        signature_verified,
        issuer_trusted: issuer.trust_status == TrustStatus::Active
            && issuer.allowed_roles.contains(&payload.issuer_role),
        time_valid: payload.issued_at_unix_seconds <= corpus.evaluation_time_unix_seconds
            && corpus.evaluation_time_unix_seconds < payload.expires_at_unix_seconds,
        evidence_integrity_verified: true,
        capability_status,
        reason,
        limitations: vec![
            "E2 synthetic fixture only; no physical relay, CT, power interface or field capability is established.",
            "Ed25519 verifies issuer-key control over exact payload bytes; it does not prove the real-world identity or competence of the issuer.",
            "Trust roots must be pinned out-of-band in production; the fixture's trust-store digest is illustrative.",
            "A valid signature does not prove the scientific truth of the measurement or physical model.",
            "Capability satisfaction also requires exact subject, configuration, scope, role, qualification class and independence matching.",
            "No operational control, switching, dispatch or protection-setting authority is created.",
        ],
    })
}

fn run(path: &Path) -> Result<Report, String> {
    let input = fs::read(path)
        .map_err(|error| format!("cannot read corpus {}: {error}", path.display()))?;
    let corpus: Corpus = serde_json::from_slice(&input)
        .map_err(|error| format!("invalid corpus JSON: {error}"))?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let trust_bytes = verify_artifact(&corpus.trust_store_artifact, root)?;
    let trust: TrustStore = serde_json::from_slice(&trust_bytes)
        .map_err(|error| format!("invalid trust store JSON: {error}"))?;
    verify(&corpus, &trust, &input, &trust_bytes, root)
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let Some(path) = args.next() else {
        eprintln!("usage: {} <synthetic-attestation-corpus.json>", PathBuf::from(program).display());
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("accepts exactly one corpus path");
        return ExitCode::from(2);
    }
    match run(Path::new(&path)).and_then(|report| {
        serde_json::to_string_pretty(&report).map_err(|error| format!("cannot encode report: {error}"))
    }) {
        Ok(report) => { println!("{report}"); ExitCode::SUCCESS }
        Err(error) => { eprintln!("GMD evidence attestation verification failed: {error}"); ExitCode::from(1) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture() -> (Corpus, TrustStore, Vec<u8>, Vec<u8>, PathBuf) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/gmd-ibr-protection/fixtures/gmd_evidence_capability_corpus_v0_1.json");
        let input = fs::read(&path).expect("fixture corpus");
        let corpus: Corpus = serde_json::from_slice(&input).expect("typed corpus");
        let root = path.parent().unwrap().to_path_buf();
        let trust_bytes = verify_artifact(&corpus.trust_store_artifact, &root).expect("trust store bytes");
        let trust: TrustStore = serde_json::from_slice(&trust_bytes).expect("typed trust store");
        (corpus, trust, input, trust_bytes, root)
    }

    #[test]
    fn valid_ed25519_attestation_satisfies_exact_synthetic_claim() {
        let (corpus, trust, input, trust_bytes, root) = fixture();
        let report = verify(&corpus, &trust, &input, &trust_bytes, &root).expect("valid fixture");
        assert!(report.signature_verified);
        assert!(report.issuer_trusted);
        assert_eq!(report.capability_status, CapabilityStatus::Satisfied);
    }

    #[test]
    fn synthetic_contract_cannot_satisfy_physical_relay_requirement() {
        let (mut corpus, trust, input, trust_bytes, root) = fixture();
        corpus.claim.claim_type = CapabilityType::PhysicalRelayObserved;
        corpus.claim.subject_kind = "protective_relay".to_string();
        corpus.claim.subject_id = "relay-001".to_string();
        corpus.claim.configuration_sha256 = "3".repeat(64);
        corpus.claim.scope_sha256 = "4".repeat(64);
        corpus.claim.accepted_qualification_classes = vec![QualificationClass::PhysicalHardwareTest];
        let report = verify(&corpus, &trust, &input, &trust_bytes, &root).expect("valid contract");
        assert_eq!(report.capability_status, CapabilityStatus::Unknown);
    }

    #[test]
    fn expired_attestation_is_rejected() {
        let (mut corpus, trust, input, trust_bytes, root) = fixture();
        corpus.evaluation_time_unix_seconds = 1810000000;
        let report = verify(&corpus, &trust, &input, &trust_bytes, &root).expect("valid contract");
        assert_eq!(report.capability_status, CapabilityStatus::Rejected);
        assert!(report.reason.contains("expired"));
    }

    #[test]
    fn revoked_issuer_is_rejected() {
        let (corpus, mut trust, input, trust_bytes, root) = fixture();
        trust.issuers[0].trust_status = TrustStatus::Revoked;
        let report = verify(&corpus, &trust, &input, &trust_bytes, &root).expect("revocation decision");
        assert_eq!(report.capability_status, CapabilityStatus::Rejected);
        assert!(report.reason.contains("revoked"));
    }

    #[test]
    fn configuration_or_scope_mismatch_is_unknown() {
        let (mut corpus, trust, input, trust_bytes, root) = fixture();
        corpus.claim.scope_sha256 = "9".repeat(64);
        let report = verify(&corpus, &trust, &input, &trust_bytes, &root).expect("valid contract");
        assert_eq!(report.capability_status, CapabilityStatus::Unknown);
    }

    #[test]
    fn issuer_independence_assessment_is_required() {
        let (corpus, mut trust, input, trust_bytes, root) = fixture();
        trust.issuers[0].independence_assessed = false;
        let report = verify(&corpus, &trust, &input, &trust_bytes, &root).expect("valid contract");
        assert_eq!(report.capability_status, CapabilityStatus::Unknown);
    }

    #[test]
    fn trust_store_digest_mismatch_fails_closed() {
        let (mut corpus, trust, input, trust_bytes, root) = fixture();
        corpus.trust_store_artifact.sha256 = "0".repeat(64);
        let error = verify(&corpus, &trust, &input, &trust_bytes, &root).unwrap_err();
        assert!(error.contains("digest mismatch"));
    }
}
