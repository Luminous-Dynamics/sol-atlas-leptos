use serde::{Deserialize, Serialize};
use serde_json_canonicalizer::to_vec as jcs_to_vec;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    process::ExitCode,
};

const TEMPLATE_SCHEMA: &str = "luminous.ieac-verification-capsule-build-template.v0.1";
const PIN_SCHEMA: &str = "luminous.ieac-verification-capsule-trust-pin.v0.1";
const MANIFEST_SCHEMA: &str = "luminous.ieac-verification-capsule.v0.1";
const RECEIPT_SCHEMA: &str = "luminous.ieac-verification-capsule-pack-receipt.v0.1";
const CLAIM_CEILING: &str = "replayable_integrity_of_included_signed_metadata_and_target_bytes_only";
const REQUIRED_TARGETS: [&str; 4] = [
    "channel_manifest",
    "checkpoint_state",
    "completeness_attestation",
    "availability_audit_report",
];
const MAX_SINGLE_FILE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildTemplate {
    schema_version: String,
    capsule_id: String,
    capsule_mode: String,
    replayability: String,
    created_at: String,
    bootstrap_root: RootFile,
    policy_file: PolicyFile,
    verifier_identity: VerifierIdentity,
    metadata_artifacts: Vec<MetadataTemplate>,
    targets: Vec<TargetTemplate>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RootFile {
    version: u64,
    staged_path: String,
    capsule_file_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyFile {
    staged_path: String,
    capsule_file_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VerifierIdentity {
    implementation: String,
    version: String,
    build_sha256: String,
    independent_verifier_required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataTemplate {
    role_name: String,
    version: u64,
    repository_path: String,
    staged_path: String,
    capsule_file_path: String,
    expires_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetTemplate {
    id: String,
    repository_path: String,
    length: u64,
    sha256: String,
    content_mode: String,
    #[serde(default)]
    staged_path: Option<String>,
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
    policy_scope_id: String,
    policy_kind: String,
    time_policy_sha256: String,
    time_verifier_build_sha256: String,
    time_verifier_trust_bundle_sha256: String,
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
struct FileEntry {
    path: String,
    length: u64,
    sha256: String,
    kind: String,
}

#[derive(Debug, Serialize)]
struct TrustBootstrap {
    repository_id: String,
    repository_origin: String,
    root_metadata_path: String,
    root_version: u64,
    root_sha256: String,
}

#[derive(Debug, Serialize)]
struct ReplayContext {
    evaluation_time: String,
    uncertainty_before_ms: u64,
    uncertainty_after_ms: u64,
    clock_id: String,
    time_policy_sha256: String,
    time_verifier_build_sha256: String,
    time_verifier_trust_bundle_sha256: String,
}

#[derive(Debug, Serialize)]
struct PolicyRef {
    policy_id: String,
    policy_sha256: String,
    policy_kind: String,
    scope_id: String,
}

#[derive(Debug, Serialize)]
struct MetadataArtifact {
    role_name: String,
    version: u64,
    repository_path: String,
    capsule_file_path: String,
    length: u64,
    sha256: String,
    expires_at: String,
}

#[derive(Debug, Serialize)]
struct Reference {
    scheme: String,
    locator: String,
}

#[derive(Debug, Serialize)]
struct TargetEntry {
    id: String,
    repository_path: String,
    length: u64,
    sha256: String,
    content_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    capsule_file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<Reference>,
}

#[derive(Debug, Serialize)]
struct CapsuleManifest {
    schema_version: &'static str,
    capsule_id: String,
    capsule_mode: String,
    replayability: String,
    canonicalization: &'static str,
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
    claim_ceiling: &'static str,
    action_authority: &'static str,
}

#[derive(Debug, Serialize)]
struct PackTargetResult {
    id: String,
    content_mode: String,
    length: u64,
    sha256: String,
    bytes_embedded: bool,
}

#[derive(Debug, Serialize)]
struct PackReceipt {
    schema_version: &'static str,
    status: &'static str,
    verification_performed: bool,
    capsule_id: String,
    capsule_path: String,
    capsule_mode: String,
    replayability: String,
    template_sha256: String,
    trust_pin_sha256: String,
    capsule_manifest_sha256: String,
    file_inventory_sha256: String,
    repository_id: String,
    root_sha256: String,
    policy_sha256: String,
    files: Vec<FileEntry>,
    targets: Vec<PackTargetResult>,
    claim_ceiling: &'static str,
    action_authority: &'static str,
}

struct OutputGuard {
    path: PathBuf,
    committed: bool,
}

impl Drop for OutputGuard {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_dir_all(&self.path);
        }
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
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn is_https_origin(value: &str) -> bool {
    let Some(authority) = value.strip_prefix("https://") else {
        return false;
    };
    !authority.is_empty()
        && !authority.contains('/')
        && !authority.contains('@')
        && !authority.contains('?')
        && !authority.contains('#')
        && authority.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b':' | b'[' | b']')
        })
}

fn read_regular_file(path: &Path, max_length: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "expected regular non-symlink file: {}",
            path.display()
        ));
    }
    if metadata.len() > max_length {
        return Err(format!("file exceeds configured limit: {}", path.display()));
    }
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if bytes.len() as u64 > max_length {
        return Err(format!("file grew beyond configured limit: {}", path.display()));
    }
    Ok(bytes)
}

fn read_staged_file(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    if !safe_relative_path(relative) {
        return Err(format!("unsafe staging path: {relative}"));
    }
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| format!("cannot canonicalize staging root: {error}"))?;
    let mut cursor = canonical_root.clone();
    for component in Path::new(relative).components() {
        let Component::Normal(part) = component else {
            return Err(format!("unsafe staging path component: {relative}"));
        };
        cursor.push(part);
        let metadata = fs::symlink_metadata(&cursor)
            .map_err(|error| format!("cannot inspect staged path {relative}: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("staging symlink is forbidden: {relative}"));
        }
    }
    let canonical = fs::canonicalize(&cursor)
        .map_err(|error| format!("cannot resolve staged file {relative}: {error}"))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!("staged file escapes staging root: {relative}"));
    }
    read_regular_file(&canonical, MAX_SINGLE_FILE_BYTES)
}

fn walk_staging(root: &Path, current: &Path, out: &mut BTreeSet<String>) -> Result<(), String> {
    for entry in
        fs::read_dir(current).map_err(|error| format!("cannot read staging directory: {error}"))?
    {
        let entry = entry.map_err(|error| format!("cannot enumerate staging directory: {error}"))?;
        let kind = entry
            .file_type()
            .map_err(|error| format!("cannot inspect staging entry: {error}"))?;
        if kind.is_symlink() {
            return Err(format!(
                "staging symlink is forbidden: {}",
                entry.path().display()
            ));
        }
        if kind.is_dir() {
            walk_staging(root, &entry.path(), out)?;
        } else if kind.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| "staging path escaped root".to_string())?
                .to_str()
                .ok_or_else(|| "staging path is not UTF-8".to_string())?
                .replace('\\', "/");
            if !safe_relative_path(&relative) || !out.insert(relative.clone()) {
                return Err(format!("unsafe or duplicate staging path: {relative}"));
            }
        } else {
            return Err(format!(
                "unsupported staging filesystem object: {}",
                entry.path().display()
            ));
        }
    }
    Ok(())
}

fn add_payload(
    payload: &mut BTreeMap<String, (Vec<u8>, String)>,
    capsule_path: &str,
    bytes: Vec<u8>,
    kind: &str,
) -> Result<(), String> {
    if !safe_relative_path(capsule_path) {
        return Err(format!("unsafe capsule output path: {capsule_path}"));
    }
    if payload
        .insert(capsule_path.to_string(), (bytes, kind.to_string()))
        .is_some()
    {
        return Err(format!("duplicate capsule output path: {capsule_path}"));
    }
    Ok(())
}

fn add_unique_staging_path(used: &mut BTreeSet<String>, path: String) -> Result<(), String> {
    if !safe_relative_path(&path) || !used.insert(path.clone()) {
        return Err(format!("unsafe or reused staging source path: {path}"));
    }
    Ok(())
}

fn staging_source_paths(template: &BuildTemplate) -> Result<BTreeSet<String>, String> {
    let mut used = BTreeSet::new();
    add_unique_staging_path(&mut used, template.bootstrap_root.staged_path.clone())?;
    add_unique_staging_path(&mut used, template.policy_file.staged_path.clone())?;
    for item in &template.metadata_artifacts {
        add_unique_staging_path(&mut used, item.staged_path.clone())?;
    }
    for target in &template.targets {
        if let Some(path) = &target.staged_path {
            add_unique_staging_path(&mut used, path.clone())?;
        }
    }
    Ok(used)
}

fn check_exact_staging_set(
    staging_root: &Path,
    used: &BTreeSet<String>,
) -> Result<(), String> {
    let mut actual = BTreeSet::new();
    walk_staging(staging_root, staging_root, &mut actual)?;
    if &actual != used {
        let extra: Vec<&str> = actual.difference(used).map(String::as_str).collect();
        let missing: Vec<&str> = used.difference(&actual).map(String::as_str).collect();
        return Err(format!(
            "staging files do not match template; extra={extra:?}, missing={missing:?}"
        ));
    }
    Ok(())
}

fn exact_target_ids(ids: &[String]) -> bool {
    let expected: BTreeSet<&str> = REQUIRED_TARGETS.into_iter().collect();
    let actual: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    actual.len() == ids.len() && actual == expected
}

fn validate_mode(mode: &str, replayability: &str, targets: &[TargetTemplate]) -> Result<(), String> {
    let modes: Vec<&str> = targets.iter().map(|target| target.content_mode.as_str()).collect();
    match mode {
        "self_contained" => {
            if replayability != "fully_replayable" || modes.iter().any(|item| *item != "embedded") {
                return Err("self_contained requires four embedded targets and fully_replayable".to_string());
            }
        }
        "metadata_complete_target_referenced" => {
            if replayability != "conditionally_replayable"
                || modes.iter().any(|item| *item == "withheld")
            {
                return Err("metadata_complete_target_referenced requires embedded/referenced targets and conditional replayability".to_string());
            }
        }
        "commitment_only" => {
            if replayability != "commitment_only" || modes.iter().any(|item| *item != "withheld") {
                return Err("commitment_only requires four withheld targets and commitment_only replayability".to_string());
            }
        }
        "mixed" => {
            if modes.iter().all(|item| *item == "embedded") && replayability != "fully_replayable" {
                return Err("mixed/all-embedded mode must be fully_replayable".to_string());
            }
            if modes.iter().any(|item| *item == "withheld") && replayability != "commitment_only" {
                return Err("mixed mode with withheld targets must declare commitment_only replayability".to_string());
            }
            if modes.iter().any(|item| *item == "referenced")
                && !modes.iter().any(|item| *item == "withheld")
                && replayability != "conditionally_replayable"
            {
                return Err("mixed/referenced mode must declare conditional replayability".to_string());
            }
        }
        _ => return Err(format!("unsupported capsule mode: {mode}")),
    }
    Ok(())
}

fn metadata_artifacts_versions(items: &[MetadataTemplate]) -> Vec<u64> {
    items.iter()
        .filter(|item| item.role_name == "root")
        .map(|item| item.version)
        .collect()
}

fn validate_pin_and_template(
    template: &BuildTemplate,
    pin: &TrustPin,
) -> Result<(), String> {
    if template.schema_version != TEMPLATE_SCHEMA
        || pin.schema_version != PIN_SCHEMA
        || pin.action_authority != "none"
        || template.capsule_id.trim().is_empty()
        || template.bootstrap_root.version == 0
        || template.verifier_identity.implementation != "sigstore-tuf"
        || template.verifier_identity.version != "0.15.1"
        || !template.verifier_identity.independent_verifier_required
        || !is_sha256(&template.verifier_identity.build_sha256)
    {
        return Err("unsupported template/pin or invalid verifier identity".to_string());
    }
    if !is_https_origin(&pin.repository_origin)
        || !is_sha256(&pin.root_sha256)
        || !is_sha256(&pin.policy_sha256)
        || !is_sha256(&pin.time_policy_sha256)
        || !is_sha256(&pin.time_verifier_build_sha256)
        || !is_sha256(&pin.time_verifier_trust_bundle_sha256)
        || pin.max_capsule_bytes == 0
        || pin.max_capsule_bytes > 1_073_741_824
        || pin.max_file_count < 6
        || pin.max_file_count > 4096
        || pin.allowed_capsule_modes.is_empty()
        || pin.pin_id.trim().is_empty()
        || pin.repository_id.trim().is_empty()
        || pin.policy_id.trim().is_empty()
        || pin.policy_scope_id.trim().is_empty()
        || pin.clock_id.trim().is_empty()
        || pin.uncertainty_before_ms > 86_400_000
        || pin.uncertainty_after_ms > 86_400_000
        || !pin.allowed_capsule_modes.iter().any(|mode| mode == &template.capsule_mode)
    {
        return Err("invalid out-of-band trust pin".to_string());
    }
    if template.bootstrap_root.staged_path.trim().is_empty()
        || template.policy_file.staged_path.trim().is_empty()
        || !safe_relative_path(&template.bootstrap_root.staged_path)
        || !safe_relative_path(&template.bootstrap_root.capsule_file_path)
        || !safe_relative_path(&template.policy_file.staged_path)
        || !safe_relative_path(&template.policy_file.capsule_file_path)
    {
        return Err("root or policy source/destination path is unsafe".to_string());
    }
    if template.metadata_artifacts.len() > 256 || template.targets.len() != 4 {
        return Err("template metadata/target cardinality exceeds limits".to_string());
    }
    let ids: Vec<String> = template.targets.iter().map(|target| target.id.clone()).collect();
    let pin_ids: Vec<String> = pin.targets.iter().map(|target| target.id.clone()).collect();
    if !exact_target_ids(&ids) || !exact_target_ids(&pin_ids) {
        return Err("template and trust pin must enumerate exactly one of each required target".to_string());
    }
    let pin_by_id: BTreeMap<&str, &str> = pin.targets
        .iter()
        .map(|target| (target.id.as_str(), target.repository_path.as_str()))
        .collect();
    let unique_paths: BTreeSet<&str> = template.targets.iter().map(|target| target.repository_path.as_str()).collect();
    if unique_paths.len() != 4 {
        return Err("template target repository paths must be unique".to_string());
    }
    for target in &template.targets {
        if pin_by_id.get(target.id.as_str()).copied() != Some(target.repository_path.as_str()) {
            return Err(format!("target {} path differs from out-of-band pin", target.id));
        }
        if target.length == 0 || target.length > MAX_SINGLE_FILE_BYTES || !is_sha256(&target.sha256) {
            return Err(format!("target {} has invalid TUF length/hash claim", target.id));
        }
        match target.content_mode.as_str() {
            "embedded" if target.staged_path.is_some() => {}
            "referenced" if target.staged_path.is_none() => {}
            "withheld" if target.staged_path.is_none() => {}
            _ => return Err(format!("target {} source path does not match content mode", target.id)),
        }
        if let Some(path) = &target.staged_path {
            if !safe_relative_path(path) {
                return Err(format!("unsafe staged target path: {path}"));
            }
        }
    }
    validate_mode(&template.capsule_mode, &template.replayability, &template.targets)?;

    let mut metadata_repo_paths = BTreeSet::new();
    let mut metadata_capsule_paths = BTreeSet::new();
    let mut metadata_roles = BTreeSet::new();
    for item in &template.metadata_artifacts {
        if item.version == 0
            || item.role_name.trim().is_empty()
            || !safe_relative_path(&item.repository_path)
            || item.repository_path.contains('/')
            || !safe_relative_path(&item.staged_path)
            || !safe_relative_path(&item.capsule_file_path)
            || !metadata_repo_paths.insert(item.repository_path.as_str())
            || !metadata_capsule_paths.insert(item.capsule_file_path.as_str())
            || (item.role_name != "root" && !metadata_roles.insert(item.role_name.as_str()))
        {
            return Err(format!("invalid or duplicate metadata template entry: {}", item.role_name));
        }
    }
    for required in ["timestamp", "snapshot", "targets"] {
        if !metadata_roles.contains(required) {
            return Err(format!("template lacks required metadata role {required}"));
        }
    }
    let mut root_versions: Vec<u64> = metadata_artifacts_versions(&template.metadata_artifacts);
    root_versions.sort_unstable();
    let mut next_root = template.bootstrap_root.version.saturating_add(1);
    for version in root_versions {
        if version != next_root {
            return Err("root-history versions must form a contiguous N+1 chain".to_string());
        }
        next_root = next_root.saturating_add(1);
    }
    Ok(())
}

fn read_trust_pin(path: &Path) -> Result<(TrustPin, Vec<u8>), String> {
    let bytes = read_regular_file(path, 1024 * 1024)?;
    let pin = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid out-of-band trust pin JSON: {error}"))?;
    Ok((pin, bytes))
}

fn read_template(path: &Path) -> Result<(BuildTemplate, Vec<u8>), String> {
    let bytes = read_regular_file(path, 2 * 1024 * 1024)?;
    let template = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid capsule build template JSON: {error}"))?;
    Ok((template, bytes))
}

fn build_manifest(
    template: BuildTemplate,
    pin: &TrustPin,
    payload: &mut BTreeMap<String, (Vec<u8>, String)>,
    staging_root: &Path,
    cas_root: Option<&Path>,
) -> Result<CapsuleManifest, String> {
    let root_bytes = read_staged_file(staging_root, &template.bootstrap_root.staged_path)?;
    let root_sha = sha256(&root_bytes);
    if root_sha != pin.root_sha256 {
        return Err("staged bootstrap root differs from out-of-band root digest".to_string());
    }
    add_payload(
        payload,
        &template.bootstrap_root.capsule_file_path,
        root_bytes,
        "root",
    )?;

    let policy_bytes = read_staged_file(staging_root, &template.policy_file.staged_path)?;
    let policy_sha = sha256(&policy_bytes);
    if policy_sha != pin.policy_sha256 {
        return Err("staged policy bytes differ from out-of-band policy digest".to_string());
    }
    add_payload(
        payload,
        &template.policy_file.capsule_file_path,
        policy_bytes,
        "policy",
    )?;

    let mut metadata = Vec::with_capacity(template.metadata_artifacts.len());
    for item in &template.metadata_artifacts {
        let bytes = read_staged_file(staging_root, &item.staged_path)?;
        let digest = sha256(&bytes);
        let expected_filename = match item.role_name.as_str() {
            "root" => format!("{}.root.json", item.version),
            "timestamp" => "timestamp.json".to_string(),
            "snapshot" => "snapshot.json".to_string(),
            role => format!("{role}.json"),
        };
        let versioned_filename = if item.role_name == "root" {
            expected_filename.clone()
        } else {
            format!("{}.{}", item.version, expected_filename)
        };
        if item.repository_path != expected_filename && item.repository_path != versioned_filename {
            return Err(format!("metadata repository filename does not match role/version: {}", item.repository_path));
        }
        let kind = match item.role_name.as_str() {
            "root" => "root_history",
            "timestamp" => "timestamp",
            "snapshot" => "snapshot",
            _ => "targets_role",
        };
        add_payload(
            payload,
            &item.capsule_file_path,
            bytes.clone(),
            kind,
        )?;
        metadata.push(MetadataArtifact {
            role_name: item.role_name.clone(),
            version: item.version,
            repository_path: item.repository_path.clone(),
            capsule_file_path: item.capsule_file_path.clone(),
            length: bytes.len() as u64,
            sha256: digest,
            expires_at: item.expires_at.clone(),
        });
    }
    metadata.sort_by(|left, right| {
        left.role_name
            .cmp(&right.role_name)
            .then(left.version.cmp(&right.version))
            .then(left.repository_path.cmp(&right.repository_path))
    });

    let mut targets = Vec::with_capacity(4);
    for target in &template.targets {
        let (capsule_file_path, reference) = match target.content_mode.as_str() {
            "embedded" => {
                let staged = target.staged_path.as_ref().ok_or_else(|| "embedded target missing staging path".to_string())?;
                let bytes = read_staged_file(staging_root, staged)?;
                if bytes.len() as u64 != target.length || sha256(&bytes) != target.sha256 {
                    return Err(format!("embedded target bytes differ from signed TUF target commitment: {}", target.id));
                }
                let capsule_path = format!("targets/{}", target.repository_path);
                add_payload(payload, &capsule_path, bytes.clone(), "target_bytes")?;
                (Some(capsule_path), None)
            }
            "referenced" => {
                let digest = &target.sha256;
                let root = cas_root.ok_or_else(|| format!("referenced target {} requires local CAS root", target.id))?;
                let bytes = read_staged_file(root, digest)?;
                if bytes.len() as u64 != target.length || sha256(&bytes) != target.sha256 {
                    return Err(format!("local CAS object differs from TUF target commitment: {}", target.id));
                }
                (
                    None,
                    Some(Reference {
                        scheme: "cas".to_string(),
                        locator: format!("cas://sha256/{digest}"),
                    }),
                )
            }
            "withheld" => (None, None),
            _ => return Err(format!("unsupported target content mode: {}", target.content_mode)),
        };
        targets.push(TargetEntry {
            id: target.id.clone(),
            repository_path: target.repository_path.clone(),
            length: target.length,
            sha256: target.sha256.clone(),
            content_mode: target.content_mode.clone(),
            capsule_file_path,
            reference,
        });
    }
    targets.sort_by(|left, right| left.id.cmp(&right.id));

    let mut inventory = Vec::with_capacity(payload.len());
    for (path, (bytes, kind)) in payload.iter() {
        inventory.push(FileEntry {
            path: path.clone(),
            length: bytes.len() as u64,
            sha256: sha256(bytes),
            kind: kind.clone(),
        });
    }
    inventory.sort_by(|left, right| left.path.cmp(&right.path));
    if inventory.len() as u64 > pin.max_file_count as u64 {
        return Err("capsule file count exceeds out-of-band trust pin".to_string());
    }
    let mut total_bytes = 0u64;
    for entry in &inventory {
        if entry.length > pin.max_capsule_bytes
            || total_bytes > pin.max_capsule_bytes.saturating_sub(entry.length)
        {
            return Err("capsule payload exceeds out-of-band byte limit".to_string());
        }
        total_bytes += entry.length;
    }
    let inventory_sha = sha256(
        &jcs_to_vec(&inventory)
            .map_err(|error| format!("cannot canonicalize capsule inventory: {error}"))?,
    );

    let mode_allowed = pin
        .allowed_capsule_modes
        .iter()
        .any(|mode| mode == &template.capsule_mode);
    if !mode_allowed {
        return Err("capsule mode is not allowed by out-of-band trust pin".to_string());
    }
    Ok(CapsuleManifest {
        schema_version: MANIFEST_SCHEMA,
        capsule_id: template.capsule_id,
        capsule_mode: template.capsule_mode,
        replayability: template.replayability,
        canonicalization: "RFC8785_JCS_FOR_CAPSULE_MANIFEST_ONLY",
        created_at: template.created_at,
        trust_bootstrap: TrustBootstrap {
            repository_id: pin.repository_id.clone(),
            repository_origin: pin.repository_origin.clone(),
            root_metadata_path: template.bootstrap_root.capsule_file_path,
            root_version: template.bootstrap_root.version,
            root_sha256: root_sha,
        },
        replay_context: ReplayContext {
            evaluation_time: pin.evaluation_time.clone(),
            uncertainty_before_ms: pin.uncertainty_before_ms,
            uncertainty_after_ms: pin.uncertainty_after_ms,
            clock_id: pin.clock_id.clone(),
            time_policy_sha256: pin.time_policy_sha256.clone(),
            time_verifier_build_sha256: pin.time_verifier_build_sha256.clone(),
            time_verifier_trust_bundle_sha256: pin.time_verifier_trust_bundle_sha256.clone(),
        },
        policy: PolicyRef {
            policy_id: pin.policy_id.clone(),
            policy_sha256: policy_sha,
            policy_kind: pin.policy_kind.clone(),
            scope_id: pin.policy_scope_id.clone(),
        },
        policy_file_path: template.policy_file.capsule_file_path,
        verifier_identity: template.verifier_identity,
        metadata_artifacts: metadata,
        targets,
        file_inventory: inventory,
        file_inventory_sha256: inventory_sha,
        claim_ceiling: CLAIM_CEILING,
        action_authority: "none",
    })
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create parent directory {}: {error}", parent.display()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("refusing overwrite or cannot create {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("cannot sync {}: {error}", path.display()))?;
    Ok(())
}

fn write_payload(output: &Path, payload: &BTreeMap<String, (Vec<u8>, String)>) -> Result<(), String> {
    for (relative, (bytes, _kind)) in payload {
        if !safe_relative_path(relative) {
            return Err(format!("unsafe payload destination: {relative}"));
        }
        let path = output.join(relative);
        let parent = path.parent().ok_or_else(|| format!("no parent for output file {relative}"))?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create capsule subdirectory: {error}"))?;
        write_new_file(&path, bytes)?;
    }
    Ok(())
}

fn write_pack_receipt(path: &Path, receipt: &PackReceipt) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("cannot serialize pack receipt: {error}"))?;
    write_new_file(path, &bytes)
}

fn pack(
    staging_root: &Path,
    template_path: &Path,
    trust_pin_path: &Path,
    output_dir: &Path,
    cas_root: Option<&Path>,
    receipt_path: &Path,
) -> Result<PackReceipt, String> {
    let staging_canonical = fs::canonicalize(staging_root)
        .map_err(|error| format!("cannot canonicalize staging root: {error}"))?;
    let output_parent = output_dir
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let output_parent = fs::canonicalize(output_parent)
        .map_err(|error| format!("cannot canonicalize output parent: {error}"))?;
    let output_final = output_parent.join(
        output_dir.file_name().ok_or_else(|| "output capsule path must name a directory".to_string())?,
    );
    if output_final == staging_canonical || output_final.starts_with(&staging_canonical) {
        return Err("output capsule directory must be outside staging root".to_string());
    }
    let receipt_parent = receipt_path.parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let receipt_parent = fs::canonicalize(receipt_parent)
        .map_err(|error| format!("cannot canonicalize receipt parent: {error}"))?;
    if receipt_parent.starts_with(&output_final) {
        return Err("pack receipt must be outside capsule directory".to_string());
    }
    if let Some(root) = cas_root {
        let cas_canonical = fs::canonicalize(root)
            .map_err(|error| format!("cannot canonicalize local CAS root: {error}"))?;
        if cas_canonical.starts_with(&output_final) {
            return Err("local CAS root must be outside output capsule directory".to_string());
        }
    }
    let (pin, pin_bytes) = read_trust_pin(trust_pin_path)?;
    let (template, template_bytes) = read_template(template_path)?;
    validate_pin_and_template(&template, &pin)?;
    let used_staging = staging_source_paths(&template)?;

    let mut payload = BTreeMap::new();
    let manifest = build_manifest(template, &pin, &mut payload, &staging_canonical, cas_root)?;
    check_exact_staging_set(&staging_canonical, &used_staging)?;

    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("cannot serialize capsule manifest: {error}"))?;
    let manifest_sha = sha256(&manifest_bytes);
    let manifest_file_length = manifest_bytes.len() as u64;
    let payload_bytes = payload.values().try_fold(0u64, |total, (bytes, _kind)| {
        total.checked_add(bytes.len() as u64).ok_or_else(|| "payload size overflow".to_string())
    })?;
    if payload_bytes > pin.max_capsule_bytes.saturating_sub(manifest_file_length) {
        return Err("capsule including manifest exceeds out-of-band byte limit".to_string());
    }
    if payload.len() + 1 > pin.max_file_count {
        return Err("capsule file count including manifest exceeds out-of-band limit".to_string());
    }

    if output_final.exists() {
        return Err(format!("capsule output already exists: {}", output_final.display()));
    }
    fs::create_dir(&output_final)
        .map_err(|error| format!("cannot reserve new capsule directory {}: {error}", output_final.display()))?;
    let mut guard = OutputGuard {
        path: output_final.clone(),
        committed: false,
    };
    write_payload(&output_final, &payload)?;
    write_new_file(&output_final.join("capsule.json"), &manifest_bytes)?;

    let receipt_files: Vec<FileEntry> = manifest.file_inventory.iter()
        .map(|entry| FileEntry {
            path: entry.path.clone(),
            length: entry.length,
            sha256: entry.sha256.clone(),
            kind: entry.kind.clone(),
        })
        .collect();
    let target_results = manifest.targets.iter().map(|target| PackTargetResult {
        id: target.id.clone(),
        content_mode: target.content_mode.clone(),
        length: target.length,
        sha256: target.sha256.clone(),
        bytes_embedded: target.content_mode == "embedded",
    }).collect();
    let receipt = PackReceipt {
        schema_version: RECEIPT_SCHEMA,
        status: "packed_unverified",
        verification_performed: false,
        capsule_id: manifest.capsule_id.clone(),
        capsule_path: output_final.display().to_string(),
        capsule_mode: manifest.capsule_mode.clone(),
        replayability: manifest.replayability.clone(),
        template_sha256: sha256(&template_bytes),
        trust_pin_sha256: sha256(&pin_bytes),
        capsule_manifest_sha256: manifest_sha,
        file_inventory_sha256: manifest.file_inventory_sha256.clone(),
        repository_id: pin.repository_id.clone(),
        root_sha256: pin.root_sha256.clone(),
        policy_sha256: pin.policy_sha256.clone(),
        files: receipt_files,
        targets: target_results,
        claim_ceiling: "input_bytes_packaged_and_committed_only",
        action_authority: "none",
    };
    write_pack_receipt(receipt_path, &receipt)?;
    guard.committed = true;
    Ok(receipt)
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 5 || args.len() > 6 {
        return Err(
            "usage: industry-verification-capsule-pack <staging-root> <template.json> <trust-pin.json> <new-capsule-dir> <new-pack-receipt.json> [local-cas-root]".to_string(),
        );
    }
    let staging_root = Path::new(&args[0]);
    let template_path = Path::new(&args[1]);
    let trust_pin_path = Path::new(&args[2]);
    let output_dir = Path::new(&args[3]);
    let receipt_path = Path::new(&args[4]);
    let cas_root = args.get(5).map(Path::new);
    let receipt_parent = receipt_path.parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let receipt_parent = fs::canonicalize(receipt_parent)
        .map_err(|error| format!("cannot canonicalize receipt parent: {error}"))?;
    let staging_canonical = fs::canonicalize(staging_root)
        .map_err(|error| format!("cannot canonicalize staging root: {error}"))?;
    if receipt_parent.starts_with(&staging_canonical) {
        return Err("pack receipt must be outside staging root".to_string());
    }
    let receipt = pack(
        staging_root,
        template_path,
        trust_pin_path,
        output_dir,
        cas_root,
        receipt_path,
    )?;
    let output = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("cannot render pack receipt: {error}"))?;
    println!("{output}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("industry-verification-capsule-pack: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_digest_is_stable_after_path_normalization() {
        let mut first = vec![
            FileEntry {
                path: "b.json".to_string(),
                length: 2,
                sha256: "b".repeat(64),
                kind: "other".to_string(),
            },
            FileEntry {
                path: "a.json".to_string(),
                length: 1,
                sha256: "a".repeat(64),
                kind: "other".to_string(),
            },
        ];
        let mut second = first.iter().map(|entry| FileEntry {
            path: entry.path.clone(),
            length: entry.length,
            sha256: entry.sha256.clone(),
            kind: entry.kind.clone(),
        }).collect::<Vec<_>>();
        second.reverse();
        first.sort_by(|left, right| left.path.cmp(&right.path));
        second.sort_by(|left, right| left.path.cmp(&right.path));
        assert_eq!(
            sha256(&jcs_to_vec(&first).unwrap()),
            sha256(&jcs_to_vec(&second).unwrap())
        );
    }

    #[test]
    fn unsafe_paths_are_rejected() {
        for path in ["../escape", "/absolute", "x//y", "x/./y", "x\\\\y", "x%2fy"] {
            assert!(!safe_relative_path(path), "must reject {path}");
        }
        assert!(safe_relative_path("metadata/2.root.json"));
    }

    #[test]
    fn output_guard_removes_partial_directories_on_failure() {
        let parent = env::temp_dir().join(format!(
            "ieac-capsule-builder-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir(&parent).unwrap();
        let child = parent.join("partial");
        fs::create_dir(&child).unwrap();
        {
            let _guard = OutputGuard {
                path: child.clone(),
                committed: false,
            };
        }
        assert!(!child.exists());
        fs::remove_dir_all(parent).unwrap();
    }
}
