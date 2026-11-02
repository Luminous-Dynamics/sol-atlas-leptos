use serde::{Deserialize, Serialize};
use sigstore_tuf::{
    cache::FileStore,
    client::{HttpRepository, Updater},
    transport::UpdaterConfig,
};
use std::{
    collections::BTreeSet,
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

const REQUEST_SCHEMA: &str = "luminous.ieac-tuf-inprocess-request.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-tuf-inprocess-report.v0.1";
const TUF_CLIENT_VERSION: &str = "sigstore-tuf/0.15.1";
const REQUIRED_TARGET_IDS: [&str; 4] = [
    "channel_manifest",
    "checkpoint_state",
    "completeness_attestation",
    "availability_audit_report",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerificationRequest {
    schema_version: String,
    repository_id: String,
    repository_url: String,
    bootstrap_root_path: PathBuf,
    metadata_cache_path: PathBuf,
    max_target_bytes: u64,
    target_bindings: Vec<TargetBinding>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetBinding {
    id: String,
    path: String,
    #[serde(default)]
    expected_sha256: Option<String>,
    #[serde(default)]
    expected_length: Option<u64>,
}

#[derive(Debug, Serialize)]
struct VerifiedTarget {
    id: String,
    path: String,
    length: u64,
    sha256: String,
    cached_bytes_used: bool,
    verification: &'static str,
}

#[derive(Debug, Serialize)]
struct VerificationReport {
    schema_version: &'static str,
    status: &'static str,
    verification_method: &'static str,
    tuf_client: &'static str,
    repository_id: String,
    repository_url: String,
    bootstrap_root_path: String,
    bootstrap_root_version: u64,
    timestamp_version: u64,
    snapshot_version: u64,
    top_level_targets_version: u64,
    update_started_at: String,
    targets: Vec<VerifiedTarget>,
    metadata_cache: &'static str,
    persistent_high_water: &'static str,
    claim_ceiling: &'static str,
    action_authority: &'static str,
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_target_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains('?')
        || path.contains('#')
        || path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || !path.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-' | b'~')
        })
    {
        return Err(format!("unsafe or non-normalized repository-relative TUF target path: {path:?}"));
    }
    Ok(())
}

fn validate_request(request: &VerificationRequest) -> Result<(), String> {
    if request.schema_version != REQUEST_SCHEMA {
        return Err("unsupported IEAC TUF request schema".to_string());
    }
    if request.repository_id.trim().is_empty() {
        return Err("repository_id must not be empty".to_string());
    }
    if request.max_target_bytes == 0 || request.max_target_bytes > 64 * 1024 * 1024 {
        return Err("max_target_bytes must be between 1 and 67108864".to_string());
    }

    let url = sigstore_tuf::reqwest::Url::parse(&request.repository_url)
        .map_err(|error| format!("invalid repository_url: {error}"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("repository_url must be HTTPS without userinfo, query, or fragment".to_string());
    }

    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for target in &request.target_bindings {
        if !ids.insert(target.id.as_str()) {
            return Err(format!("duplicate target id: {}", target.id));
        }
        if !paths.insert(target.path.as_str()) {
            return Err(format!("duplicate TUF target path: {}", target.path));
        }
        validate_target_path(&target.path)?;
        if let Some(expected) = &target.expected_sha256 {
            if !is_sha256(expected) {
                return Err(format!("{} expected_sha256 must be lowercase SHA-256", target.id));
            }
        }
    }
    let expected: BTreeSet<&str> = REQUIRED_TARGET_IDS.into_iter().collect();
    if ids != expected {
        return Err(format!(
            "target_bindings must contain exactly these four ids: {}",
            REQUIRED_TARGET_IDS.join(", ")
        ));
    }
    Ok(())
}

fn read_request(path: &Path) -> Result<VerificationRequest, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read request {}: {error}", path.display()))?;
    // Struct deserialization rejects duplicate known fields and unknown fields.
    let request: VerificationRequest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid request JSON: {error}"))?;
    validate_request(&request)?;
    Ok(request)
}

fn write_new_report(path: &Path, report: &VerificationReport) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("cannot serialize report: {error}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "refusing to overwrite or cannot create report {}: {error}",
                path.display()
            )
        })?;
    file.write_all(&bytes)
        .map_err(|error| format!("cannot write report {}: {error}", path.display()))?;
    file.write_all(b"\n")
        .map_err(|error| format!("cannot finish report {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("cannot sync report {}: {error}", path.display()))?;
    Ok(())
}

async fn verify_repository(request: VerificationRequest) -> Result<VerificationReport, String> {
    let root_metadata = fs::metadata(&request.bootstrap_root_path).map_err(|error| {
        format!(
            "cannot inspect out-of-band bootstrap root {}: {error}",
            request.bootstrap_root_path.display()
        )
    })?;
    if !root_metadata.is_file() || root_metadata.len() > 512_000 {
        return Err("bootstrap root must be a regular file no larger than 512000 bytes".to_string());
    }
    let root_bytes = fs::read(&request.bootstrap_root_path).map_err(|error| {
        format!(
            "cannot read out-of-band bootstrap root {}: {error}",
            request.bootstrap_root_path.display()
        )
    })?;

    // A caller-configured timeout prevents a hostile or stalled mirror from
    // blocking the TUF refresh indefinitely.
    let http = sigstore_tuf::reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(45))
        .redirect(sigstore_tuf::reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| format!("cannot create bounded HTTPS client: {error}"))?;
    let repository = HttpRepository::new(&request.repository_url)
        .map_err(|error| format!("cannot create TUF repository transport: {error}"))?
        .with_http_client(http);

    // One fixed update-start time is used for refresh and every target
    // authorization check, matching the TUF client workflow.
    let update_started_at = jiff::Timestamp::now();
    let mut updater_config = UpdaterConfig::default();
    updater_config.target_max_length = request.max_target_bytes;
    let mut updater = Updater::new(repository, &root_bytes)
        .map_err(|error| format!("pinned TUF bootstrap root failed verification: {error}"))?
        .with_config(updater_config)
        .with_store(FileStore::new(request.metadata_cache_path.clone()));

    updater
        .refresh(update_started_at)
        .await
        .map_err(|error| format!("TUF metadata refresh/verification failed: {error}"))?;

    let root_version = updater.trusted().root().version;
    let timestamp_version = updater
        .trusted()
        .timestamp()
        .ok_or_else(|| "TUF refresh completed without trusted timestamp metadata".to_string())?
        .version;
    let snapshot_version = updater
        .trusted()
        .snapshot()
        .ok_or_else(|| "TUF refresh completed without trusted snapshot metadata".to_string())?
        .version;
    let targets_version = updater
        .trusted()
        .targets_role("targets")
        .ok_or_else(|| "TUF refresh completed without top-level targets metadata".to_string())?
        .version;

    let mut verified_targets = Vec::with_capacity(REQUIRED_TARGET_IDS.len());
    for binding in &request.target_bindings {
        let info = updater
            .get_target_info(&binding.path, update_started_at)
            .await
            .map_err(|error| {
                format!(
                    "TUF could not resolve target {} at {}: {error}",
                    binding.id, binding.path
                )
            })?
            .ok_or_else(|| {
                format!(
                    "no trusted top-level or delegated TUF role authorizes {}",
                    binding.path
                )
            })?;

        if info.length > request.max_target_bytes {
            return Err(format!(
                "target {} length {} exceeds policy maximum {}",
                binding.id, info.length, request.max_target_bytes
            ));
        }
        if let Some(expected_length) = binding.expected_length {
            if info.length != expected_length {
                return Err(format!(
                    "target {} length differs from operator pin: metadata={}, pinned={}",
                    binding.id, info.length, expected_length
                ));
            }
        }
        let target_sha256 = info
            .hashes
            .get("sha256")
            .ok_or_else(|| format!("target {} has no SHA-256 hash in TUF metadata", binding.id))?
            .to_ascii_lowercase();
        if !is_sha256(&target_sha256) {
            return Err(format!("target {} has malformed TUF SHA-256 metadata", binding.id));
        }
        if let Some(expected_sha256) = &binding.expected_sha256 {
            if &target_sha256 != expected_sha256 {
                return Err(format!(
                    "target {} SHA-256 differs from operator pin",
                    binding.id
                ));
            }
        }

        // find_cached_target verifies cached bytes against TUF length/hash;
        // download_target verifies fresh bytes against the same authenticated
        // TargetFile. No self-asserted verifier report is consumed.
        let (bytes, cached) = match updater.find_cached_target(&info, &binding.path) {
            Some(bytes) => (bytes, true),
            None => (
                updater
                    .download_target(&info, &binding.path)
                    .await
                    .map_err(|error| {
                        format!("TUF target download/integrity check failed for {}: {error}", binding.id)
                    })?,
                false,
            ),
        };
        if bytes.len() as u64 != info.length {
            return Err(format!(
                "TUF library returned an unexpected byte length for {}",
                binding.id
            ));
        }

        verified_targets.push(VerifiedTarget {
            id: binding.id.clone(),
            path: binding.path.clone(),
            length: info.length,
            sha256: target_sha256,
            cached_bytes_used: cached,
            verification: "verified_by_sigstore_tuf_target_length_and_hash",
        });
    }

    Ok(VerificationReport {
        schema_version: REPORT_SCHEMA,
        status: "cryptographically_verified",
        verification_method: "in_process_sigstore_tuf_updater",
        tuf_client: TUF_CLIENT_VERSION,
        repository_id: request.repository_id,
        repository_url: request.repository_url,
        bootstrap_root_path: request.bootstrap_root_path.display().to_string(),
        bootstrap_root_version: root_version,
        timestamp_version,
        snapshot_version,
        top_level_targets_version: targets_version,
        update_started_at: update_started_at.to_string(),
        targets: verified_targets,
        metadata_cache: "sigstore_tuf_file_store_reverified_from_pinned_bootstrap_root",
        persistent_high_water: "not_managed_by_this_binary",
        claim_ceiling: "tuf_metadata_signatures_thresholds_expiry_rollback_delegation_and_target_integrity_only",
        action_authority: "none",
    })
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: industry-tuf-inprocess-verify <request.json> <new-report.json>".to_string(),
        );
    }
    let request_path = Path::new(&args[0]);
    let report_path = Path::new(&args[1]);
    if report_path == request_path {
        return Err("report destination must not be the request input path".to_string());
    }
    let request = read_request(request_path)?;
    let report = verify_repository(request).await?;
    write_new_report(report_path, &report)?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("industry-tuf-inprocess-verify: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigstore_tuf::{
        cache::{FileStore, StoreRepository},
        transport::{FetchFuture, Repository},
        Error, Updater,
    };
    use std::path::{Path as FsPath, PathBuf};

    struct FixtureRepository {
        metadata_dir: PathBuf,
        targets_dir: PathBuf,
        corrupt_target: Option<String>,
    }

    impl FixtureRepository {
        fn read(dir: &FsPath, name: &str, max_length: u64) -> sigstore_tuf::Result<Option<Vec<u8>>> {
            match fs::read(dir.join(name)) {
                Ok(bytes) if bytes.len() as u64 > max_length => Err(Error::transport(format!(
                    "{name} exceeds transport limit {max_length}"
                ))),
                Ok(bytes) => Ok(Some(bytes)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(Error::transport_with_source("fixture read failed", error)),
            }
        }
    }

    impl Repository for FixtureRepository {
        fn fetch_metadata<'a>(&'a self, name: &'a str, max_length: u64) -> FetchFuture<'a> {
            let result = Self::read(&self.metadata_dir, name, max_length);
            Box::pin(async move { result })
        }

        fn fetch_target<'a>(&'a self, path: &'a str, max_length: u64) -> FetchFuture<'a> {
            let mut result = Self::read(&self.targets_dir, path, max_length);
            if self.corrupt_target.as_deref() == Some(path) {
                result = result.map(|bytes| {
                    bytes.map(|mut value| {
                        if let Some(first) = value.first_mut() {
                            *first ^= 0x01;
                        }
                        value
                    })
                });
            }
            Box::pin(async move { result })
        }
    }

    fn fixture_root() -> Vec<u8> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/tuf-reference-impl/metadata/1.root.json");
        fs::read(path).expect("reference implementation root fixture")
    }

    fn fixture_repo(corrupt_target: Option<&str>) -> FixtureRepository {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tuf-reference-impl");
        FixtureRepository {
            metadata_dir: base.join("metadata"),
            targets_dir: base.join("targets"),
            corrupt_target: corrupt_target.map(str::to_string),
        }
    }

    fn fixture_time() -> jiff::Timestamp {
        "2026-06-01T00:00:00Z".parse().expect("fixed fixture time")
    }

    fn valid_request() -> VerificationRequest {
        VerificationRequest {
            schema_version: REQUEST_SCHEMA.to_string(),
            repository_id: "test-repository".to_string(),
            repository_url: "https://tuf.example.invalid".to_string(),
            bootstrap_root_path: PathBuf::from("/etc/ieac/root.json"),
            metadata_cache_path: PathBuf::from("/var/lib/ieac/tuf"),
            max_target_bytes: 1024 * 1024,
            target_bindings: vec![
                TargetBinding {
                    id: "channel_manifest".to_string(),
                    path: "channels/plant-17/manifest.json".to_string(),
                    expected_sha256: None,
                    expected_length: None,
                },
                TargetBinding {
                    id: "checkpoint_state".to_string(),
                    path: "channels/plant-17/checkpoint.json".to_string(),
                    expected_sha256: None,
                    expected_length: None,
                },
                TargetBinding {
                    id: "completeness_attestation".to_string(),
                    path: "channels/plant-17/completeness-attestation.json".to_string(),
                    expected_sha256: None,
                    expected_length: None,
                },
                TargetBinding {
                    id: "availability_audit_report".to_string(),
                    path: "channels/plant-17/completeness-report.json".to_string(),
                    expected_sha256: None,
                    expected_length: None,
                },
            ],
        }
    }

    #[test]
    fn request_requires_https_without_redirect_or_authority_overrides() {
        for url in [
            "http://tuf.example.invalid",
            "https://user@tuf.example.invalid",
            "https://tuf.example.invalid?mirror=other",
            "https://tuf.example.invalid#fragment",
            "file:///etc/passwd",
        ] {
            let mut request = valid_request();
            request.repository_url = url.to_string();
            assert!(validate_request(&request).is_err(), "must reject {url}");
        }
        assert!(validate_request(&valid_request()).is_ok());
    }

    #[test]
    fn request_requires_exact_target_id_set_and_unique_paths() {
        let mut request = valid_request();
        request.target_bindings.pop();
        assert!(validate_request(&request).is_err());

        let mut request = valid_request();
        request.target_bindings[3].path = request.target_bindings[2].path.clone();
        assert!(validate_request(&request).is_err());

        let mut request = valid_request();
        request.target_bindings[3].id = request.target_bindings[2].id.clone();
        assert!(validate_request(&request).is_err());
    }

    #[tokio::test]
    async fn verifies_python_tuf_repository_and_delegated_target() {
        let root = fixture_root();
        let mut updater = Updater::new(fixture_repo(None), &root)
            .expect("python-tuf root must self-verify");
        updater.refresh(fixture_time()).await.expect("full TUF refresh");
        assert_eq!(updater.trusted().root().version, 1);
        assert_eq!(updater.trusted().timestamp().unwrap().version, 1);
        assert_eq!(updater.trusted().snapshot().unwrap().version, 1);

        let file1 = updater.get_target("file1.txt", fixture_time()).await.unwrap();
        assert_eq!(file1, b"This is an example target file.");

        assert!(
            updater.trusted().targets_role("targets").unwrap().target("file3.txt").is_none(),
            "file3 must be reached through delegation, not top-level targets"
        );
        let file3 = updater.get_target("file3.txt", fixture_time()).await.unwrap();
        assert_eq!(file3, b"This is role1's target file.");
    }

    #[tokio::test]
    async fn rejects_target_bytes_changed_after_authenticated_metadata() {
        let root = fixture_root();
        let mut updater = Updater::new(fixture_repo(Some("file3.txt")), &root)
            .expect("python-tuf root must self-verify");
        updater.refresh(fixture_time()).await.expect("full TUF refresh");
        let result = updater.get_target("file3.txt", fixture_time()).await;
        assert!(result.is_err(), "tampered target bytes must fail TUF integrity verification");
    }

    #[tokio::test]
    async fn offline_store_reverifies_cached_metadata_and_target() {
        let root = fixture_root();
        let temp = tempfile::tempdir().expect("temporary metadata store");
        let store = FileStore::new(temp.path().to_path_buf());
        let mut online = Updater::new(fixture_repo(None), &root)
            .expect("python-tuf root must self-verify")
            .with_store(store.clone());
        online.refresh(fixture_time()).await.expect("online fixture refresh");
        let downloaded = online.get_target("file3.txt", fixture_time()).await.unwrap();
        assert_eq!(downloaded, b"This is role1's target file.");
        drop(online);

        let offline_store = FileStore::new(temp.path().to_path_buf());
        let mut offline = Updater::new(StoreRepository::new(offline_store.clone()), &root)
            .expect("bootstrap root is pinned")
            .with_store(offline_store);
        offline.refresh(fixture_time()).await.expect("offline cached refresh");
        let cached = offline.get_target("file3.txt", fixture_time()).await.unwrap();
        assert_eq!(cached, b"This is role1's target file.");
    }

    #[test]
    fn malformed_roots_fail_bootstrap_closed() {
        let negative = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/negative");
        for file in [
            "no-root-json-signatures.root.json",
            "invalid-root-json-signature.root.json",
            "duplicate-sigs.root.json",
        ] {
            let bytes = fs::read(negative.join(file)).expect("negative root fixture");
            let error = sigstore_tuf::TrustedMetadataSet::from_root(&bytes)
                .expect_err("malformed root must fail closed");
            assert!(
                matches!(error, Error::ThresholdNotMet { .. } | Error::DuplicateSignature { .. }),
                "{file}: unexpected verification error {error:?}"
            );
        }
    }

    #[test]
    fn target_path_rejects_encoded_traversal_and_non_ascii_paths() {
        for path in [
            "../escape.json",
            "channels/%2e%2e/escape.json",
            "channels/%2fetc/passwd",
            "channels//checkpoint.json",
            "channels/./checkpoint.json",
            "channels\\\\checkpoint.json",
            "channels/plant 17/checkpoint.json",
        ] {
            assert!(
                validate_target_path(path).is_err(),
                "unsafe target path must be rejected: {path}"
            );
        }
        assert!(validate_target_path("channels/plant-17/checkpoint.json").is_ok());
    }

    #[test]
    fn request_deserialization_rejects_duplicate_known_fields() {
        let duplicate = br#"{
            "schema_version":"luminous.ieac-tuf-inprocess-request.v0.1",
            "schema_version":"luminous.ieac-tuf-inprocess-request.v0.1"
        }"#;
        assert!(serde_json::from_slice::<VerificationRequest>(duplicate).is_err());
    }
}
