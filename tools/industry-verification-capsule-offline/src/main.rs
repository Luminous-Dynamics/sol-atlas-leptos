use serde::{Deserialize, Serialize};
use serde_json_canonicalizer::to_vec as jcs_to_vec;
use sha2::{Digest, Sha256};
use sigstore_tuf::{
    cache::MemoryStore,
    client::Updater,
    transport::{FetchFuture, Repository, UpdaterConfig},
    Error as TufError,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    process::ExitCode,
};

const MANIFEST_FILE: &str = "capsule.json";
const MANIFEST_SCHEMA: &str = "luminous.ieac-verification-capsule.v0.1";
const PIN_SCHEMA: &str = "luminous.ieac-verification-capsule-trust-pin.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-verification-capsule-audit-report.v0.1";
const REQUIRED_TARGETS: [&str; 4] = [
    "channel_manifest",
    "checkpoint_state",
    "completeness_attestation",
    "availability_audit_report",
];
const MAX_METADATA_BYTES: u64 = 5 * 1024 * 1024;
const MAX_SINGLE_FILE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileEntry {
    path: String,
    length: u64,
    sha256: String,
    kind: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    capsule_id: String,
    capsule_mode: String,
    replayability: String,
    canonicalization: String,
    created_at: String,
    trust_bootstrap: TrustBootstrap,
    replay_context: ReplayContext,
    policy: PolicyRef,
    policy_file_path: String,
    verifier_identity: VerifierIdentity,
    metadata_artifacts: Vec<MetadataArtifact>,
    targets: Vec<TargetEntry>,
    file_inventory: Vec<FileEntry>,
    file_inventory_sha256: String,
    claim_ceiling: String,
    action_authority: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustBootstrap {
    repository_id: String,
    repository_origin: String,
    root_metadata_path: String,
    root_version: u64,
    root_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayContext {
    evaluation_time: String,
    uncertainty_before_ms: u64,
    uncertainty_after_ms: u64,
    clock_id: String,
    time_policy_sha256: String,
    time_verifier_build_sha256: String,
    time_verifier_trust_bundle_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyRef {
    policy_id: String,
    policy_sha256: String,
    policy_kind: String,
    scope_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifierIdentity {
    implementation: String,
    version: String,
    build_sha256: String,
    independent_verifier_required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataArtifact {
    role_name: String,
    version: u64,
    repository_path: String,
    capsule_file_path: String,
    length: u64,
    sha256: String,
    expires_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetEntry {
    id: String,
    repository_path: String,
    length: u64,
    sha256: String,
    content_mode: String,
    #[serde(default)]
    capsule_file_path: Option<String>,
    #[serde(default)]
    reference: Option<Reference>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    scheme: String,
    locator: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustPin {
    schema_version: String,
    pin_id: String,
    repository_id: String,
    repository_origin: String,
    root_sha256: String,
    policy_id: String,
    policy_sha256: String,
    evaluation_time: String,
    uncertainty_before_ms: u64,
    uncertainty_after_ms: u64,
    clock_id: String,
    targets: Vec<PinTarget>,
    max_capsule_bytes: u64,
    max_file_count: usize,
    allowed_capsule_modes: Vec<String>,
    action_authority: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PinTarget {
    id: String,
    repository_path: String,
}

#[derive(Debug, Serialize)]
struct TargetResult {
    id: String,
    repository_path: String,
    metadata_authorization: &'static str,
    target_bytes: &'static str,
    expected_length: u64,
    expected_sha256: String,
    content_mode: String,
}

#[derive(Debug, Serialize)]
struct AuditReport {
    schema_version: &'static str,
    status: &'static str,
    capsule_id: String,
    capsule_mode: String,
    replayability: &'static str,
    capsule_manifest_sha256: String,
    file_inventory_sha256: String,
    trust_pin_sha256: String,
    policy_sha256: String,
    repository_id: String,
    repository_origin: String,
    bootstrap_root_sha256: String,
    bootstrap_root_version: u64,
    latest_root_version: u64,
    timestamp_version: u64,
    snapshot_version: u64,
    top_level_targets_version: u64,
    evaluation_time: String,
    tufs_evaluation_time_upper_bound: String,
    tuf_implementation: &'static str,
    metadata_verification: &'static str,
    targets: Vec<TargetResult>,
    unsigned_report: bool,
    claim_ceiling: &'static str,
    action_authority: &'static str,
}

#[derive(Debug, Clone)]
enum TargetSource {
    Embedded { capsule_path: String },
    Referenced { sha256: String },
    Withheld,
}

struct CapsuleRepository {
    capsule_root: PathBuf,
    cas_root: Option<PathBuf>,
    metadata_paths: BTreeMap<String, String>,
    target_sources: BTreeMap<String, TargetSource>,
}

impl CapsuleRepository {
    fn read_limited(root: &Path, rel: &str, max_length: u64) -> sigstore_tuf::Result<Option<Vec<u8>>> {
        let rel_path = Path::new(rel);
        if !safe_relative_path(rel) {
            return Err(TufError::transport(format!("unsafe capsule path: {rel}")));
        }
        let canonical_root = fs::canonicalize(root)
            .map_err(|error| TufError::transport_with_source("capsule root unavailable", error))?;
        let mut cursor = canonical_root.clone();
        for component in rel_path.components() {
            match component {
                Component::Normal(part) => cursor.push(part),
                _ => return Err(TufError::transport(format!("unsafe capsule path component: {rel}"))),
            }
            let metadata = match fs::symlink_metadata(&cursor) {
                Ok(value) => value,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(TufError::transport_with_source("cannot inspect capsule path", error)),
            };
            if metadata.file_type().is_symlink() {
                return Err(TufError::transport(format!("symlink not permitted in capsule: {rel}")));
            }
        }
        let canonical_file = fs::canonicalize(&cursor)
            .map_err(|error| TufError::transport_with_source("cannot resolve capsule file", error))?;
        if !canonical_file.starts_with(&canonical_root) {
            return Err(TufError::transport(format!("capsule path escapes root: {rel}")));
        }
        let metadata = fs::metadata(&canonical_file)
            .map_err(|error| TufError::transport_with_source("cannot stat capsule file", error))?;
        if !metadata.is_file() {
            return Err(TufError::transport(format!("capsule path is not a regular file: {rel}")));
        }
        if metadata.len() > max_length {
            return Err(TufError::transport(format!(
                "capsule file {rel} exceeds transport limit {max_length}"
            )));
        }
        let bytes = fs::read(&canonical_file)
            .map_err(|error| TufError::transport_with_source("cannot read capsule file", error))?;
        if bytes.len() as u64 > max_length {
            return Err(TufError::transport(format!(
                "capsule file {rel} grew beyond transport limit {max_length}"
            )));
        }
        Ok(Some(bytes))
    }

    fn target_bytes(&self, path: &str, max_length: u64) -> sigstore_tuf::Result<Option<Vec<u8>>> {
        let Some(source) = self.target_sources.get(path) else {
            return Ok(None);
        };
        match source {
            TargetSource::Embedded { capsule_path } => {
                Self::read_limited(&self.capsule_root, capsule_path, max_length)
            }
            TargetSource::Referenced { sha256 } => {
                let Some(root) = &self.cas_root else {
                    return Ok(None);
                };
                Self::read_limited(root, sha256, max_length)
            }
            TargetSource::Withheld => Ok(None),
        }
    }
}

impl Repository for CapsuleRepository {
    fn fetch_metadata<'a>(&'a self, name: &'a str, max_length: u64) -> FetchFuture<'a> {
        Box::pin(async move {
            let Some(path) = self.metadata_paths.get(name) else {
                return Ok(None);
            };
            Self::read_limited(&self.capsule_root, path, max_length)
        })
    }

    fn fetch_target<'a>(&'a self, path: &'a str, max_length: u64) -> FetchFuture<'a> {
        Box::pin(async move { self.target_bytes(path, max_length) })
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\\')
        && !value.contains('?')
        && !value.contains('#')
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-' | b'~')
        })
        && value.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn validate_metadata_name(name: &str) -> bool {
    safe_relative_path(name)
        && !name.contains('/')
        && name.ends_with(".json")
        && name.len() <= 256
}

fn read_regular_file(path: &Path, max_length: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("expected regular non-symlink file: {}", path.display()));
    }
    if metadata.len() > max_length {
        return Err(format!("file exceeds configured limit: {}", path.display()));
    }
    let bytes = fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if bytes.len() as u64 > max_length {
        return Err(format!("file grew beyond configured limit: {}", path.display()));
    }
    Ok(bytes)
}

fn read_capsule_file(root: &Path, rel: &str, max_length: u64) -> Result<Vec<u8>, String> {
    if !safe_relative_path(rel) {
        return Err(format!("unsafe capsule-relative path: {rel}"));
    }
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| format!("cannot canonicalize capsule root: {error}"))?;
    let mut cursor = canonical_root.clone();
    for component in Path::new(rel).components() {
        let Component::Normal(part) = component else {
            return Err(format!("unsafe capsule path component: {rel}"));
        };
        cursor.push(part);
        let metadata = fs::symlink_metadata(&cursor)
            .map_err(|error| format!("cannot inspect capsule path {rel}: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("capsule symlink is forbidden: {rel}"));
        }
    }
    let canonical = fs::canonicalize(&cursor)
        .map_err(|error| format!("cannot resolve capsule path {rel}: {error}"))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!("capsule path escapes root: {rel}"));
    }
    read_regular_file(&canonical, max_length)
}

fn walk_payload(root: &Path, current: &Path, out: &mut BTreeSet<String>) -> Result<(), String> {
    for entry in fs::read_dir(current).map_err(|error| format!("cannot read capsule directory: {error}"))? {
        let entry = entry.map_err(|error| format!("cannot enumerate capsule directory: {error}"))?;
        let kind = entry.file_type().map_err(|error| format!("cannot inspect capsule entry: {error}"))?;
        if kind.is_symlink() {
            return Err(format!("capsule symlink is forbidden: {}", entry.path().display()));
        }
        if kind.is_dir() {
            walk_payload(root, &entry.path(), out)?;
        } else if kind.is_file() {
            let rel = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| "capsule entry escaped root".to_string())?
                .to_str()
                .ok_or_else(|| "capsule file path is not UTF-8".to_string())?
                .replace('\\', "/");
            if rel != MANIFEST_FILE {
                if !safe_relative_path(&rel) || !out.insert(rel.clone()) {
                    return Err(format!("invalid or duplicate capsule path: {rel}"));
                }
            }
        } else {
            return Err(format!("unsupported capsule filesystem object: {}", entry.path().display()));
        }
    }
    Ok(())
}

fn exact_target_ids(values: &[String]) -> bool {
    let expected: BTreeSet<&str> = REQUIRED_TARGETS.into_iter().collect();
    let actual: BTreeSet<&str> = values.iter().map(String::as_str).collect();
    actual.len() == values.len() && actual == expected
}

fn validate_mode(manifest: &Manifest, pin: &TrustPin) -> Result<(), String> {
    if !pin.allowed_capsule_modes.iter().any(|mode| mode == &manifest.capsule_mode) {
        return Err("capsule mode is not allowed by the out-of-band trust pin".to_string());
    }
    let modes: Vec<&str> = manifest.targets.iter().map(|target| target.content_mode.as_str()).collect();
    match manifest.capsule_mode.as_str() {
        "self_contained" => {
            if manifest.replayability != "fully_replayable" || modes.iter().any(|mode| *mode != "embedded") {
                return Err("self_contained capsule must embed all target bytes and declare fully_replayable".to_string());
            }
        }
        "metadata_complete_target_referenced" => {
            if manifest.replayability != "conditionally_replayable"
                || modes.iter().any(|mode| *mode == "withheld")
            {
                return Err("metadata-complete target-referenced mode requires embedded/referenced targets and conditional replayability".to_string());
            }
        }
        "commitment_only" => {
            if manifest.replayability != "commitment_only" || modes.iter().any(|mode| *mode != "withheld") {
                return Err("commitment_only mode must withhold all target bytes".to_string());
            }
        }
        "mixed" => {
            if modes.iter().all(|mode| *mode == "embedded") && manifest.replayability != "fully_replayable" {
                return Err("mixed capsule with all targets embedded has incorrect replayability label".to_string());
            }
            if modes.iter().any(|mode| *mode == "withheld") && manifest.replayability != "commitment_only" {
                return Err("mixed capsule with withheld targets cannot claim stronger than commitment_only".to_string());
            }
            if modes.iter().any(|mode| *mode == "referenced")
                && !modes.iter().any(|mode| *mode == "withheld")
                && manifest.replayability != "conditionally_replayable"
            {
                return Err("mixed capsule with references must declare conditional replayability".to_string());
            }
        }
        _ => return Err("unsupported capsule mode".to_string()),
    }
    Ok(())
}

fn validate_pin(manifest: &Manifest, pin: &TrustPin) -> Result<(), String> {
    if pin.schema_version != PIN_SCHEMA || pin.action_authority != "none" {
        return Err("invalid trust-pin schema or action-authority boundary".to_string());
    }
    if manifest.schema_version != MANIFEST_SCHEMA
        || manifest.action_authority != "none"
        || manifest.claim_ceiling != "replayable_integrity_of_included_signed_metadata_and_target_bytes_only"
        || manifest.canonicalization != "RFC8785_JCS_FOR_CAPSULE_MANIFEST_ONLY"
    {
        return Err("invalid capsule schema, claim ceiling or canonicalization profile".to_string());
    }
    if manifest.trust_bootstrap.repository_id != pin.repository_id
        || manifest.trust_bootstrap.repository_origin != pin.repository_origin
        || manifest.trust_bootstrap.root_sha256 != pin.root_sha256
        || manifest.policy.policy_id != pin.policy_id
        || manifest.policy.policy_sha256 != pin.policy_sha256
        || manifest.replay_context.evaluation_time != pin.evaluation_time
        || manifest.replay_context.uncertainty_before_ms != pin.uncertainty_before_ms
        || manifest.replay_context.uncertainty_after_ms != pin.uncertainty_after_ms
        || manifest.replay_context.clock_id != pin.clock_id
    {
        return Err("capsule trust bootstrap, policy or replay context differs from out-of-band pin".to_string());
    }
    if manifest.targets.len() != REQUIRED_TARGETS.len() || pin.targets.len() != REQUIRED_TARGETS.len() {
        return Err("capsule and trust pin must each enumerate exactly four IEAC targets".to_string());
    }
    let target_ids: Vec<String> = manifest.targets.iter().map(|target| target.id.clone()).collect();
    let pin_ids: Vec<String> = pin.targets.iter().map(|target| target.id.clone()).collect();
    if !exact_target_ids(&target_ids) || !exact_target_ids(&pin_ids) {
        return Err("capsule and trust pin target IDs must contain exactly one of each required target".to_string());
    }
    let mut manifest_by_id = BTreeMap::new();
    for target in &manifest.targets {
        if manifest_by_id.insert(target.id.as_str(), target).is_some() {
            return Err(format!("duplicate target ID {}", target.id));
        }
    }
    for target in &pin.targets {
        let actual = manifest_by_id
            .get(target.id.as_str())
            .ok_or_else(|| format!("trust pin target {} missing from capsule", target.id))?;
        if actual.repository_path != target.repository_path {
            return Err(format!("target path for {} differs from operator pin", target.id));
        }
    }
    let unique_paths: BTreeSet<&str> = manifest.targets.iter().map(|target| target.repository_path.as_str()).collect();
    if unique_paths.len() != manifest.targets.len() {
        return Err("IEAC target repository paths must be unique".to_string());
    }
    validate_mode(manifest, pin)
}

fn validate_file_inventory(root: &Path, manifest: &Manifest, pin: &TrustPin) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if manifest.file_inventory.len() > pin.max_file_count {
        return Err("capsule file count exceeds operator maximum".to_string());
    }
    if manifest.file_inventory.len() > 4096 || pin.max_capsule_bytes == 0 {
        return Err("invalid capsule size/count pin".to_string());
    }
    let canonical = jcs_to_vec(&manifest.file_inventory)
        .map_err(|error| format!("cannot JCS-canonicalize file inventory: {error}"))?;
    let inventory_sha = sha256(&canonical);
    if inventory_sha != manifest.file_inventory_sha256 {
        return Err("file inventory JCS digest mismatch".to_string());
    }

    let mut previous: Option<&str> = None;
    let mut listed = BTreeMap::new();
    let mut total_bytes = 0u64;
    for entry in &manifest.file_inventory {
        if !safe_relative_path(&entry.path) || !is_sha256(&entry.sha256) {
            return Err(format!("unsafe file inventory entry: {}", entry.path));
        }
        if previous.is_some_and(|prior| prior >= entry.path.as_str()) {
            return Err("file inventory paths must be strictly sorted and unique".to_string());
        }
        previous = Some(&entry.path);
        if total_bytes > pin.max_capsule_bytes.saturating_sub(entry.length) {
            return Err("capsule payload bytes exceed operator maximum".to_string());
        }
        total_bytes += entry.length;
        let bytes = read_capsule_file(root, &entry.path, MAX_SINGLE_FILE_BYTES.min(pin.max_capsule_bytes))?;
        if bytes.len() as u64 != entry.length || sha256(&bytes) != entry.sha256 {
            return Err(format!("capsule file inventory mismatch: {}", entry.path));
        }
        listed.insert(entry.path.clone(), bytes);
    }

    let mut actual_files = BTreeSet::new();
    walk_payload(root, root, &mut actual_files)?;
    let listed_files: BTreeSet<String> = listed.keys().cloned().collect();
    if actual_files != listed_files {
        return Err("capsule contains unlisted files or inventory lists absent files".to_string());
    }
    Ok(listed)
}

fn validate_manifest_files(
    root: &Path,
    manifest: &Manifest,
    pin: &TrustPin,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let root_entry = manifest
        .file_inventory
        .iter()
        .find(|entry| entry.path == manifest.trust_bootstrap.root_metadata_path && entry.kind == "root")
        .ok_or_else(|| "pinned bootstrap root is absent from root file inventory".to_string())?;
    if root_entry.sha256 != pin.root_sha256 || manifest.trust_bootstrap.root_sha256 != pin.root_sha256 {
        return Err("capsule bootstrap root digest differs from operator trust pin".to_string());
    }
    let root_bytes = files
        .get(&manifest.trust_bootstrap.root_metadata_path)
        .ok_or_else(|| "bootstrap root bytes missing from capsule".to_string())?
        .clone();
    if sha256(&root_bytes) != pin.root_sha256 {
        return Err("bootstrap root bytes do not match the out-of-band SHA-256 pin".to_string());
    }
    if root_entry.length != root_bytes.len() as u64 {
        return Err("bootstrap root byte length mismatch".to_string());
    }

    let policy_entry = manifest
        .file_inventory
        .iter()
        .find(|entry| entry.path == manifest.policy_file_path && entry.kind == "policy")
        .ok_or_else(|| "policy file is absent from capsule inventory".to_string())?;
    if policy_entry.sha256 != pin.policy_sha256 || policy_entry.sha256 != manifest.policy.policy_sha256 {
        return Err("policy file digest differs from operator trust pin".to_string());
    }
    if !files.contains_key(&manifest.policy_file_path) {
        return Err("policy file bytes are missing".to_string());
    }

    let mut names = BTreeSet::new();
    let mut roles = BTreeMap::<&str, usize>::new();
    for metadata in &manifest.metadata_artifacts {
        if !validate_metadata_name(&metadata.repository_path)
            || !safe_relative_path(&metadata.capsule_file_path)
            || !is_sha256(&metadata.sha256)
        {
            return Err(format!("invalid metadata artifact mapping for {}", metadata.repository_path));
        }
        if !names.insert(metadata.repository_path.as_str()) {
            return Err(format!("duplicate TUF metadata repository path {}", metadata.repository_path));
        }
        let bytes = files
            .get(&metadata.capsule_file_path)
            .ok_or_else(|| format!("metadata artifact bytes missing: {}", metadata.capsule_file_path))?;
        let entry = manifest
            .file_inventory
            .iter()
            .find(|item| item.path == metadata.capsule_file_path)
            .ok_or_else(|| format!("metadata artifact absent from inventory: {}", metadata.capsule_file_path))?;
        if entry.length != metadata.length
            || entry.sha256 != metadata.sha256
            || bytes.len() as u64 != metadata.length
            || sha256(bytes) != metadata.sha256
        {
            return Err(format!("metadata artifact byte commitment mismatch: {}", metadata.repository_path));
        }
        *roles.entry(metadata.role_name.as_str()).or_default() += 1;
    }
    for required in ["timestamp", "snapshot", "targets"] {
        if roles.get(required).copied().unwrap_or(0) != 1 {
            return Err(format!("capsule metadata must contain exactly one {required} role"));
        }
    }

    let mut target_paths = BTreeSet::new();
    for target in &manifest.targets {
        if !target_paths.insert(target.repository_path.as_str())
            || !safe_relative_path(&target.repository_path)
            || !is_sha256(&target.sha256)
        {
            return Err(format!("invalid or duplicate target commitment: {}", target.repository_path));
        }
        match target.content_mode.as_str() {
            "embedded" => {
                let path = target.capsule_file_path.as_deref()
                    .ok_or_else(|| format!("embedded target {} lacks capsule_file_path", target.id))?;
                let bytes = files.get(path)
                    .ok_or_else(|| format!("embedded target bytes absent: {}", target.id))?;
                let entry = manifest.file_inventory.iter().find(|item| item.path == path)
                    .ok_or_else(|| format!("embedded target {} missing inventory entry", target.id))?;
                if entry.kind != "target_bytes" || bytes.len() as u64 != target.length
                    || entry.length != target.length || entry.sha256 != target.sha256
                    || sha256(bytes) != target.sha256
                {
                    return Err(format!("embedded target commitment mismatch: {}", target.id));
                }
            }
            "referenced" => {
                if target.capsule_file_path.is_some() {
                    return Err(format!("referenced target {} must not embed capsule_file_path", target.id));
                }
                let reference = target.reference.as_ref()
                    .ok_or_else(|| format!("referenced target {} lacks CAS locator", target.id))?;
                if reference.scheme != "cas" || reference.locator != format!("cas://sha256/{}", target.sha256) {
                    return Err(format!("referenced target {} CAS locator does not match digest", target.id));
                }
            }
            "withheld" => {
                if target.capsule_file_path.is_some() || target.reference.is_some() {
                    return Err(format!("withheld target {} unexpectedly includes bytes or locator", target.id));
                }
            }
            _ => return Err(format!("unsupported content mode for {}", target.id)),
        }
    }
    Ok(root_bytes)
}

fn conservative_eval_time(context: &ReplayContext) -> Result<jiff::Timestamp, String> {
    let base: jiff::Timestamp = context.evaluation_time.parse()
        .map_err(|error| format!("invalid evaluation_time: {error}"))?;
    let upper_seconds = context.uncertainty_after_ms.saturating_add(999) / 1000;
    let upper = base.as_second().checked_add(upper_seconds as i64)
        .ok_or_else(|| "evaluation-time upper bound overflow".to_string())?;
    jiff::Timestamp::from_second(upper)
        .map_err(|error| format!("cannot represent conservative TUF evaluation time: {error}"))
}

fn metadata_claim(artifact: &MetadataArtifact, version: u64, expires_at: &str) -> Result<(), String> {
    if artifact.version != version || artifact.expires_at != expires_at {
        return Err(format!(
            "capsule metadata claim for {} differs from verified TUF state",
            artifact.role_name
        ));
    }
    Ok(())
}

fn build_repository(root: &Path, cas_root: Option<PathBuf>, manifest: &Manifest) -> Result<CapsuleRepository, String> {
    let mut metadata_paths = BTreeMap::new();
    for item in &manifest.metadata_artifacts {
        metadata_paths.insert(item.repository_path.clone(), item.capsule_file_path.clone());
    }
    let mut target_sources = BTreeMap::new();
    for target in &manifest.targets {
        let source = match target.content_mode.as_str() {
            "embedded" => TargetSource::Embedded {
                capsule_path: target.capsule_file_path.clone()
                    .ok_or_else(|| format!("embedded target {} missing path", target.id))?,
            },
            "referenced" => TargetSource::Referenced { sha256: target.sha256.clone() },
            "withheld" => TargetSource::Withheld,
            _ => return Err(format!("unsupported content mode {}", target.content_mode)),
        };
        target_sources.insert(target.repository_path.clone(), source);
    }
    Ok(CapsuleRepository {
        capsule_root: fs::canonicalize(root).map_err(|error| format!("cannot canonicalize capsule root: {error}"))?,
        cas_root,
        metadata_paths,
        target_sources,
    })
}

async fn audit_capsule(
    capsule_root: &Path,
    trust_pin_path: &Path,
    cas_root: Option<PathBuf>,
) -> Result<AuditReport, String> {
    let pin_bytes = read_regular_file(trust_pin_path, 1024 * 1024)?;
    let pin: TrustPin = serde_json::from_slice(&pin_bytes)
        .map_err(|error| format!("invalid out-of-band trust pin JSON: {error}"))?;
    if pin.schema_version != PIN_SCHEMA || pin.action_authority != "none" {
        return Err("unsupported trust pin schema or action-authority boundary".to_string());
    }

    let manifest_path = capsule_root.join(MANIFEST_FILE);
    let manifest_bytes = read_regular_file(&manifest_path, 8 * 1024 * 1024)?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("invalid capsule manifest JSON: {error}"))?;
    if manifest.capsule_id.trim().is_empty()
        || manifest.metadata_artifacts.len() > 256
        || manifest.targets.len() != 4
    {
        return Err("capsule manifest has invalid identity or cardinality".to_string());
    }
    validate_pin(&manifest, &pin)?;
    if manifest.file_inventory.len() > pin.max_file_count {
        return Err("capsule file inventory exceeds operator file-count limit".to_string());
    }
    let files = validate_file_inventory(capsule_root, &manifest, &pin)?;
    let root_bytes = validate_manifest_files(capsule_root, &manifest, &pin, &files)?;
    let eval_upper = conservative_eval_time(&manifest.replay_context)?;

    let repository = build_repository(capsule_root, cas_root.clone(), &manifest)?;
    let mut config = UpdaterConfig::default();
    config.target_max_length = MAX_SINGLE_FILE_BYTES.min(pin.max_capsule_bytes);
    config.root_max_length = MAX_METADATA_BYTES;
    config.timestamp_max_length = MAX_METADATA_BYTES;
    config.snapshot_max_length = MAX_METADATA_BYTES;
    config.targets_max_length = MAX_METADATA_BYTES;

    // No HTTP feature is enabled in this package. The only repository passed
    // to Updater is the capsule-backed transport above.
    let mut updater = Updater::new(repository, &root_bytes)
        .map_err(|error| format!("pinned TUF root signature verification failed: {error}"))?
        .with_config(config)
        .with_store(MemoryStore::new());
    updater.refresh(eval_upper).await
        .map_err(|error| format!("offline TUF metadata verification failed: {error}"))?;

    let mut target_results = Vec::with_capacity(4);
    for target in &manifest.targets {
        let info = updater.get_target_info(&target.repository_path, eval_upper).await
            .map_err(|error| format!("TUF delegation resolution failed for {}: {error}", target.id))?
            .ok_or_else(|| format!("no verified TUF role authorizes target {}", target.repository_path))?;
        let verified_sha = info.hashes.get("sha256")
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| format!("verified TUF target {} has no SHA-256 hash", target.id))?;
        if info.length != target.length || verified_sha != target.sha256 {
            return Err(format!("capsule target claim differs from verified TUF metadata: {}", target.id));
        }

        let available = match target.content_mode.as_str() {
            "embedded" => true,
            "referenced" => {
                let Some(root) = &cas_root else { false };
                let reference = target.reference.as_ref().ok_or_else(|| "missing CAS reference".to_string())?;
                let digest = reference.locator.strip_prefix("cas://sha256/")
                    .ok_or_else(|| "invalid CAS reference URI".to_string())?;
                match CapsuleRepository::read_limited(root, digest, target.length) {
                    Ok(Some(bytes)) if bytes.len() as u64 == target.length && sha256(&bytes) == target.sha256 => true,
                    Ok(None) => false,
                    Ok(Some(_)) => return Err(format!("CAS target bytes mismatch: {}", target.id)),
                    Err(error) => return Err(format!("CAS target read failed: {error}")),
                }
            }
            "withheld" => false,
            _ => return Err(format!("unsupported target content mode: {}", target.content_mode)),
        };

        let bytes_status = if available {
            let bytes = updater.download_target(&info, &target.repository_path).await
                .map_err(|error| format!("offline target byte verification failed for {}: {error}", target.id))?;
            if bytes.len() as u64 != target.length || sha256(&bytes) != target.sha256 {
                return Err(format!("replayed target bytes differ from capsule commitment: {}", target.id));
            }
            "verified"
        } else {
            "not_replayable"
        };
        target_results.push(TargetResult {
            id: target.id.clone(),
            repository_path: target.repository_path.clone(),
            metadata_authorization: "verified",
            target_bytes: bytes_status,
            expected_length: target.length,
            expected_sha256: target.sha256.clone(),
            content_mode: target.content_mode.clone(),
        });
    }

    // The claims in the manifest are compared with parsed, verified library
    // state. TUF signature/expiry/version checks themselves are performed only
    // by sigstore-tuf, using TUF canonical JSON rather than JCS.
    let timestamp = updater.trusted().timestamp()
        .ok_or_else(|| "verified TUF timestamp metadata missing after refresh".to_string())?;
    let snapshot = updater.trusted().snapshot()
        .ok_or_else(|| "verified TUF snapshot metadata missing after refresh".to_string())?;
    for artifact in &manifest.metadata_artifacts {
        match artifact.role_name.as_str() {
            "timestamp" => metadata_claim(artifact, timestamp.version, &timestamp.expires.to_string())?,
            "snapshot" => metadata_claim(artifact, snapshot.version, &snapshot.expires.to_string())?,
            "targets" => {
                let targets = updater.trusted().targets_role("targets")
                    .ok_or_else(|| "verified top-level targets metadata missing".to_string())?;
                metadata_claim(artifact, targets.version, &targets.expires.to_string())?;
            }
            "root" => {
                // Each sequential root update is signature-checked by Updater.
                // The current version is reported below; individual historical
                // root claims are not used as a trust input.
            }
            role => {
                if let Some(targets) = updater.trusted().targets_role(role) {
                    metadata_claim(artifact, targets.version, &targets.expires.to_string())?;
                }
            }
        }
    }

    let verified_count = target_results.iter().filter(|target| target.target_bytes == "verified").count();
    let withheld_count = target_results.len() - verified_count;
    let (status, replayability) = if withheld_count == 0 {
        ("fully_replayed", "fully_replayable")
    } else if verified_count == 0 && target_results.iter().all(|target| target.content_mode == "withheld") {
        ("metadata_verified_commitment_only", "commitment_only")
    } else {
        ("metadata_verified_target_replay_incomplete", "conditionally_replayable")
    };
    if manifest.replayability == "fully_replayable" && withheld_count != 0 {
        return Err("capsule claimed fully replayable but one or more target bytes were unavailable".to_string());
    }

    Ok(AuditReport {
        schema_version: REPORT_SCHEMA,
        status,
        capsule_id: manifest.capsule_id,
        capsule_mode: manifest.capsule_mode,
        replayability,
        capsule_manifest_sha256: sha256(&manifest_bytes),
        file_inventory_sha256: manifest.file_inventory_sha256,
        trust_pin_sha256: sha256(&pin_bytes),
        policy_sha256: manifest.policy.policy_sha256,
        repository_id: manifest.trust_bootstrap.repository_id,
        repository_origin: manifest.trust_bootstrap.repository_origin,
        bootstrap_root_sha256: pin.root_sha256,
        bootstrap_root_version: manifest.trust_bootstrap.root_version,
        latest_root_version: updater.trusted().root().version,
        timestamp_version: timestamp.version,
        snapshot_version: snapshot.version,
        top_level_targets_version: updater.trusted().targets_role("targets").map(|targets| targets.version).unwrap_or_default(),
        evaluation_time: manifest.replay_context.evaluation_time,
        tufs_evaluation_time_upper_bound: eval_upper.to_string(),
        tuf_implementation: "sigstore-tuf/0.15.1",
        metadata_verification: "verified_in_process_offline_from_pinned_root",
        targets: target_results,
        unsigned_report: true,
        claim_ceiling: "included_signed_tuf_metadata_and_replayed_target_byte_integrity_only",
        action_authority: "none",
    })
}

fn write_report(path: &Path, report: &AuditReport) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("cannot serialize audit report: {error}"))?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|error| format!("refusing to overwrite or cannot create {}: {error}", path.display()))?;
    file.write_all(&bytes).map_err(|error| format!("cannot write audit report: {error}"))?;
    file.write_all(b"\n").map_err(|error| format!("cannot finish audit report: {error}"))?;
    file.sync_all().map_err(|error| format!("cannot sync audit report: {error}"))?;
    Ok(())
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 3 || args.len() > 4 {
        return Err(
            "usage: industry-verification-capsule-audit <capsule-dir> <out-of-band-trust-pin.json> <new-report.json> [local-cas-dir]".to_string()
        );
    }
    let capsule_root = Path::new(&args[0]);
    let trust_pin = Path::new(&args[1]);
    let report_path = Path::new(&args[2]);
    let cas_root = args.get(3).map(PathBuf::from);
    if report_path.starts_with(capsule_root) {
        return Err("audit report destination must be outside the untrusted capsule directory".to_string());
    }
    let report = audit_capsule(capsule_root, trust_pin, cas_root).await?;
    write_report(report_path, &report)?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("industry-verification-capsule-audit: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigstore_tuf::TrustedMetadataSet;

    fn fixture_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../industry-tuf-inprocess-verifier/tests/fixtures")
    }

    #[test]
    fn inventory_jcs_is_order_sensitive_until_paths_are_sorted() {
        let mut one = vec![
            FileEntry { path: "b.json".into(), length: 2, sha256: "b".repeat(64), kind: "other".into() },
            FileEntry { path: "a.json".into(), length: 1, sha256: "a".repeat(64), kind: "other".into() },
        ];
        let mut two = one.iter().cloned().collect::<Vec<_>>();
        one.sort_by(|left, right| left.path.cmp(&right.path));
        two.sort_by(|left, right| left.path.cmp(&right.path));
        let one_digest = sha256(&jcs_to_vec(&one).unwrap());
        let two_digest = sha256(&jcs_to_vec(&two).unwrap());
        assert_eq!(one_digest, two_digest);
    }

    #[test]
    fn pinned_reference_root_is_cryptographically_valid() {
        let root = fs::read(fixture_root().join("tuf-reference-impl/metadata/1.root.json")).unwrap();
        TrustedMetadataSet::from_root(&root).expect("python-tuf reference root verifies");
    }

    #[test]
    fn malformed_reference_roots_fail_closed() {
        let negative = fixture_root().join("negative");
        for name in [
            "no-root-json-signatures.root.json",
            "invalid-root-json-signature.root.json",
            "duplicate-sigs.root.json",
        ] {
            let bytes = fs::read(negative.join(name)).unwrap();
            assert!(
                TrustedMetadataSet::from_root(&bytes).is_err(),
                "must reject {name}"
            );
        }
    }

    #[test]
    fn unsafe_relative_paths_are_rejected() {
        for value in [
            "../escape.json",
            "/etc/passwd",
            "channels/%2e%2e/escape",
            "channels//manifest.json",
            "channels/./manifest.json",
            "channels\\manifest.json",
            "channels/plant 17/manifest.json",
        ] {
            assert!(!safe_relative_path(value), "must reject {value}");
        }
        assert!(safe_relative_path("channels/plant-17/manifest.json"));
    }
}
