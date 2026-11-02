#![forbid(unsafe_code)]
//! Manifest-driven CLI for the independent non-anticipativity receipt auditor.

use energy_dispatch_schedule_auditor::nonanticipativity_v0_1::{
    evaluate_nonanticipativity_bytes, NonAnticipativityDispositionV1,
    NonAnticipativityInformationSetBytes, NonAnticipativityScenarioBytes, MAX_FORECAST_BUNDLE_BYTES,
    MAX_SCENARIOS,
};
use energy_dispatch_schedule_auditor::{parse_unique_json, MAX_REQUEST_BYTES, MAX_SCHEDULE_BYTES};
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const MANIFEST_SCHEMA: &str = "energy-dispatch-nonanticipativity-manifest-v0.2";
const MAX_MANIFEST_BYTES: usize = 1_048_576;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InformationSetManifestV1 {
    forecast_as_of_unix_seconds: i64,
    forecast_bundle_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioManifestV1 {
    scenario_id: String,
    request_path: String,
    schedule_path: String,
    information_sets_by_interval: Vec<InformationSetManifestV1>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NonAnticipativityManifestV1 {
    schema_version: String,
    scenario_set_id: String,
    scenarios: Vec<ScenarioManifestV1>,
}

struct LoadedInformationSet {
    forecast_as_of_unix_seconds: i64,
    forecast_bundle_bytes: Vec<u8>,
}

struct LoadedScenario {
    scenario_id: String,
    request_bytes: Vec<u8>,
    schedule_bytes: Vec<u8>,
    information_sets_by_interval: Vec<LoadedInformationSet>,
}

fn read_bounded(path: &Path, limit: usize, label: &str) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|error| format!("open {label}: {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("inspect opened {label}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{label} must be a regular file"));
    }
    if metadata.len() > limit as u64 {
        return Err(format!("{label} exceeds {limit} byte limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read {label}: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeded {limit} byte limit while reading"));
    }
    Ok(bytes)
}

fn resolve_under_root(root: &Path, supplied: &str, label: &str) -> Result<PathBuf, String> {
    let relative = Path::new(supplied);
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("{label} must be a non-empty path relative to the manifest directory"));
    }
    let candidate = root.join(relative);
    let canonical = candidate
        .canonicalize()
        .map_err(|error| format!("resolve {label} beneath manifest directory: {error}"))?;
    if !canonical.starts_with(root) {
        return Err(format!("{label} resolves outside the manifest directory"));
    }
    Ok(canonical)
}

fn load_manifest(
    path: &Path,
) -> Result<(NonAnticipativityManifestV1, Vec<LoadedScenario>), String> {
    let canonical_manifest_path = path
        .canonicalize()
        .map_err(|error| format!("resolve non-anticipativity manifest: {error}"))?;
    let manifest_bytes = read_bounded(
        &canonical_manifest_path,
        MAX_MANIFEST_BYTES,
        "non-anticipativity manifest",
    )?;
    let manifest_value = parse_unique_json(&manifest_bytes)
        .map_err(|error| format!("strict manifest JSON parse: {error}"))?;
    let manifest: NonAnticipativityManifestV1 = serde_json::from_value(manifest_value)
        .map_err(|error| format!("non-anticipativity manifest schema: {error}"))?;
    if manifest.schema_version != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported non-anticipativity manifest schema {}",
            manifest.schema_version
        ));
    }
    if manifest.scenario_set_id.trim().is_empty()
        || manifest.scenario_set_id.len() > 128
        || !manifest
            .scenario_set_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "-_.:".contains(ch))
    {
        return Err("scenario_set_id must be a safe non-empty identifier".to_owned());
    }
    if !(2..=MAX_SCENARIOS).contains(&manifest.scenarios.len()) {
        return Err(format!(
            "scenario count must be between 2 and {MAX_SCENARIOS}"
        ));
    }

    let manifest_dir = canonical_manifest_path
        .parent()
        .ok_or_else(|| "canonical manifest path has no parent directory".to_owned())?
        .to_path_buf();

    let mut loaded = Vec::with_capacity(manifest.scenarios.len());
    for scenario in &manifest.scenarios {
        let request_path =
            resolve_under_root(&manifest_dir, &scenario.request_path, "request_path")?;
        let schedule_path =
            resolve_under_root(&manifest_dir, &scenario.schedule_path, "schedule_path")?;
        let mut information_sets_by_interval = Vec::with_capacity(
            scenario.information_sets_by_interval.len(),
        );
        for information_set in &scenario.information_sets_by_interval {
            let forecast_bundle_path = resolve_under_root(
                &manifest_dir,
                &information_set.forecast_bundle_path,
                "forecast_bundle_path",
            )?;
            information_sets_by_interval.push(LoadedInformationSet {
                forecast_as_of_unix_seconds: information_set.forecast_as_of_unix_seconds,
                forecast_bundle_bytes: read_bounded(
                    &forecast_bundle_path,
                    MAX_FORECAST_BUNDLE_BYTES,
                    "forecast bundle",
                )?,
            });
        }
        loaded.push(LoadedScenario {
            scenario_id: scenario.scenario_id.clone(),
            request_bytes: read_bounded(
                &request_path,
                MAX_REQUEST_BYTES,
                "scenario source request",
            )?,
            schedule_bytes: read_bounded(
                &schedule_path,
                MAX_SCHEDULE_BYTES,
                "scenario dispatch schedule",
            )?,
            information_sets_by_interval,
        });
    }

    Ok((manifest, loaded))
}

fn write_receipt(path: &Path, receipt: &impl serde::Serialize) -> Result<(), String> {
    let output = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("serialize non-anticipativity receipt: {error}"))?;
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create receipt output directory: {error}"))?;
    }
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, output)
        .map_err(|error| format!("write temporary non-anticipativity receipt: {error}"))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("publish non-anticipativity receipt: {error}"))
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err(format!(
            "usage: {} <nonanticipativity-manifest.json> <receipt.json>",
            args.first()
                .map(|value| value.to_string_lossy())
                .unwrap_or_else(|| "energy-dispatch-nonanticipativity-auditor".into())
        ));
    }
    let manifest_path = PathBuf::from(&args[1]);
    let receipt_path = PathBuf::from(&args[2]);
    let (manifest, loaded) = load_manifest(&manifest_path)?;
    let information_sets: Vec<Vec<_>> = loaded
        .iter()
        .map(|scenario| {
            scenario
                .information_sets_by_interval
                .iter()
                .map(|information_set| NonAnticipativityInformationSetBytes {
                    forecast_as_of_unix_seconds: information_set.forecast_as_of_unix_seconds,
                    forecast_bundle_bytes: &information_set.forecast_bundle_bytes,
                })
                .collect()
        })
        .collect();
    let scenario_inputs: Vec<_> = loaded
        .iter()
        .zip(&information_sets)
        .map(|(scenario, information_sets_by_interval)| NonAnticipativityScenarioBytes {
            scenario_id: &scenario.scenario_id,
            request_bytes: &scenario.request_bytes,
            schedule_bytes: &scenario.schedule_bytes,
            information_sets_by_interval,
        })
        .collect();

    let receipt = evaluate_nonanticipativity_bytes(
        &manifest.scenario_set_id,
        &scenario_inputs,
    )
    .map_err(|error| format!("non-anticipativity evaluation failed closed: {error}"))?;
    write_receipt(&receipt_path, &receipt)?;

    println!("scenario_set_id={}", manifest.scenario_set_id);
    println!("scenario_count={}", receipt.scenario_count);
    println!("interval_count={}", receipt.interval_count);
    println!("shared_information_set_count={}", receipt.shared_information_set_count);
    println!(
        "checked_action_comparison_count={}",
        receipt.checked_action_comparison_count
    );
    println!("disposition={:?}", receipt.disposition);
    println!("receipt_sha256={}", receipt.receipt_sha256);
    println!(
        "failed_findings={}",
        receipt.findings.iter().filter(|finding| !finding.passed).count()
    );

    Ok(match receipt.disposition {
        NonAnticipativityDispositionV1::Accepted => ExitCode::SUCCESS,
        NonAnticipativityDispositionV1::Rejected => ExitCode::from(1),
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("non-anticipativity audit failed closed: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{read_bounded, resolve_under_root};
    use std::path::Path;

    #[test]
    fn bounded_reader_accepts_limit_and_rejects_one_byte_over() {
        let path = std::env::temp_dir().join(format!(
            "nonanticipativity-bounded-read-{}",
            std::process::id()
        ));
        std::fs::write(&path, b"12345").unwrap();
        assert_eq!(read_bounded(&path, 5, "fixture").unwrap(), b"12345");
        assert!(read_bounded(&path, 4, "fixture").is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn manifest_paths_cannot_traverse_above_the_root() {
        let root = Path::new("/tmp");
        assert!(resolve_under_root(root, "../outside.json", "fixture").is_err());
        assert!(resolve_under_root(root, "/etc/passwd", "fixture").is_err());
    }
}
