//! TUF client-under-test adapter for the official tuf-conformance suite.
//!
//! Implements the CLI protocol specified by:
//! https://github.com/theupdateframework/tuf-conformance/blob/main/CLIENT-CLI.md
//!
//! Adapted from sigstore-rust's crates/sigstore-tuf/examples/conformance_client.rs
//! (Apache-2.0). This adapter uses the same pinned sigstore-tuf library as IEAC.

use sha2::{Digest, Sha256, Sha512};
use sigstore_tuf::cache::MetadataStore;
use sigstore_tuf::client::{HttpRepository, Updater};
use sigstore_tuf::{Error, Result, TargetFile};
use std::path::{Component, Path, PathBuf};

/// Persist canonical, non-versioned trusted metadata filenames. The conformance
/// harness inspects this directory to determine what the client considers
/// trusted. Versioned roots and cached targets are not stored here.
struct ConformanceStore {
    dir: PathBuf,
}

impl MetadataStore for ConformanceStore {
    fn load(&self, name: &str) -> Result<Option<Vec<u8>>> {
        match std::fs::read(self.dir.join(name)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(Error::transport_with_source(
                format!("reading {name}"),
                error,
            )),
        }
    }

    fn store(&self, name: &str, bytes: &[u8]) -> Result<()> {
        // Do not persist cached target objects in METADATA_DIR.
        if name.contains('/') {
            return Ok(());
        }
        // Do not persist version-prefixed roots (for example 2.root.json).
        let stem = name.split('.').next().unwrap_or("");
        if !stem.is_empty() && stem.bytes().all(|byte| byte.is_ascii_digit()) {
            return Ok(());
        }
        std::fs::write(self.dir.join(name), bytes)
            .map_err(|error| Error::transport_with_source(format!("writing {name}"), error))
    }
}

#[derive(Default)]
struct Args {
    metadata_dir: Option<PathBuf>,
    metadata_url: Option<String>,
    target_names: Vec<String>,
    target_dir: Option<PathBuf>,
    target_base_url: Option<String>,
    positionals: Vec<String>,
}

fn parse_args() -> std::result::Result<Args, String> {
    let mut args = Args::default();
    let mut iter = std::env::args().skip(1);
    while let Some(token) = iter.next() {
        let take = |iter: &mut std::iter::Skip<std::env::Args>, option: &str| {
            iter.next()
                .ok_or_else(|| format!("missing value for {option}"))
        };
        match token.as_str() {
            "--metadata-dir" => args.metadata_dir = Some(PathBuf::from(take(&mut iter, &token)?)),
            "--metadata-url" => args.metadata_url = Some(take(&mut iter, &token)?),
            "--target-name" => args.target_names.push(take(&mut iter, &token)?),
            "--target-dir" => args.target_dir = Some(PathBuf::from(take(&mut iter, &token)?)),
            "--target-base-url" => args.target_base_url = Some(take(&mut iter, &token)?),
            "-v" | "--verbose" => {}
            other if other.starts_with('-') => return Err(format!("unknown flag {other}")),
            _ => args.positionals.push(token),
        }
    }
    Ok(args)
}

fn metadata_dir(args: &Args) -> Result<PathBuf> {
    args.metadata_dir
        .clone()
        .ok_or_else(|| Error::transport("--metadata-dir is required"))
}

/// init: seed the trusted metadata directory with the supplied root.
fn cmd_init(args: &Args) -> Result<()> {
    let directory = metadata_dir(args)?;
    let trusted_root_path = args
        .positionals
        .get(1)
        .ok_or_else(|| Error::transport("init requires a trusted root file"))?;
    let root_bytes = std::fs::read(trusted_root_path)
        .map_err(|error| Error::transport_with_source("reading trusted root", error))?;

    // The suite expects no repository request during init. Verifying the
    // out-of-band root before seeding is stricter than the minimum CLI protocol.
    sigstore_tuf::TrustedMetadataSet::from_root(&root_bytes)?;
    std::fs::create_dir_all(&directory)
        .map_err(|error| Error::transport_with_source("creating metadata directory", error))?;
    std::fs::write(directory.join("root.json"), &root_bytes)
        .map_err(|error| Error::transport_with_source("writing root.json", error))?;
    Ok(())
}

fn build_updater(args: &Args) -> Result<Updater> {
    let directory = metadata_dir(args)?;
    let metadata_url = args
        .metadata_url
        .as_ref()
        .ok_or_else(|| Error::transport("--metadata-url is required"))?;
    let root_bytes = std::fs::read(directory.join("root.json"))
        .map_err(|error| Error::transport_with_source("no trusted root in metadata directory", error))?;

    let mut repository = HttpRepository::new(metadata_url)?;
    if let Some(target_url) = &args.target_base_url {
        repository = repository.with_targets_base(target_url)?;
    }
    Ok(Updater::new(repository, &root_bytes)?
        .with_store(ConformanceStore { dir: directory }))
}

/// refresh: run the full TUF refresh chain and persist canonical metadata.
async fn cmd_refresh(args: &Args) -> Result<()> {
    let mut updater = build_updater(args)?;
    updater.refresh(jiff::Timestamp::now()).await
}

/// download: refresh, resolve delegated targets with one updater instance,
/// verify their bytes and cache matching files in TARGET_DIR.
async fn cmd_download(args: &Args) -> Result<()> {
    if args.target_names.is_empty() {
        return Err(Error::transport("--target-name is required"));
    }
    let target_directory = args
        .target_dir
        .as_ref()
        .ok_or_else(|| Error::transport("--target-dir is required"))?;

    let mut updater = build_updater(args)?;
    let update_time = jiff::Timestamp::now();
    updater.refresh(update_time).await?;

    for target_name in &args.target_names {
        let info = updater
            .get_target_info(target_name, update_time)
            .await?
            .ok_or_else(|| Error::TargetNotFound(target_name.clone()))?;
        let output = safe_target_path(target_directory, target_name)?;

        if let Ok(existing) = std::fs::read(&output) {
            if target_bytes_match(&existing, &info) {
                continue;
            }
        }

        let bytes = updater.download_target(&info, target_name).await?;
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| Error::transport_with_source("creating target directory", error))?;
        }
        std::fs::write(&output, &bytes)
            .map_err(|error| Error::transport_with_source("writing verified target", error))?;
    }
    Ok(())
}

fn target_bytes_match(bytes: &[u8], info: &TargetFile) -> bool {
    if bytes.len() as u64 != info.length {
        return false;
    }
    !info.hashes.is_empty()
        && info.hashes.iter().all(|(algorithm, expected)| {
            let actual = match algorithm.as_str() {
                "sha256" => format!("{:x}", Sha256::digest(bytes)),
                "sha512" => format!("{:x}", Sha512::digest(bytes)),
                _ => return false,
            };
            actual.eq_ignore_ascii_case(expected)
        })
}

fn safe_target_path(target_directory: &Path, target_name: &str) -> Result<PathBuf> {
    if target_name.is_empty()
        || target_name.starts_with('/')
        || target_name.contains('\\')
        || target_name.contains('?')
        || target_name.contains('#')
    {
        return Err(Error::transport("unsafe TUF target path"));
    }
    let relative = Path::new(target_name);
    if target_name
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Error::transport("unsafe TUF target path component"));
    }
    Ok(target_directory.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_target_path_rejects_absolute_and_traversal_paths() {
        let directory = Path::new("/tmp/tuf-targets");
        for name in [
            "",
            "/etc/passwd",
            "../escape",
            "channels/../escape",
            "channels//manifest.json",
            "channels/./manifest.json",
            "channels\\manifest.json",
            "channels/manifest.json?download=1",
        ] {
            assert!(
                safe_target_path(directory, name).is_err(),
                "unsafe target path must fail: {name}"
            );
        }
        assert_eq!(
            safe_target_path(directory, "channels/plant-17/manifest.json").unwrap(),
            directory.join("channels/plant-17/manifest.json")
        );
    }
}

fn print_error_chain(error: &Error) -> String {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

#[tokio::main]
async fn main() {
    let args = match parse_args() {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("conformance_client: {error}");
            std::process::exit(1);
        }
    };

    let result = match args.positionals.first().map(String::as_str) {
        Some("init") => cmd_init(&args),
        Some("refresh") => cmd_refresh(&args).await,
        Some("download") => cmd_download(&args).await,
        other => Err(Error::transport(format!(
            "expected subcommand init|refresh|download, got {other:?}"
        ))),
    };

    if let Err(error) = result {
        eprintln!("conformance_client: {}", print_error_chain(&error));
        std::process::exit(1);
    }
}
