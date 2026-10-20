// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Reproducible native-document benchmark for the landscape schema.
//! It measures synthetic generation, strict validation, normalized JSON
//! serialization, and SHA-256—not browser rendering or environmental outcomes.

#[path = "../landscape.rs"]
mod landscape;

use landscape::{
    FRAME_KIND, MAX_SNAPSHOT_FEATURES, synthetic_benchmark_snapshot, validate_snapshot,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const WORKLOAD_MANIFEST: &str = include_str!("../../benchmarks/landscape/workloads-v0.json");
const REPORT_SCHEMA: &str = "luminous.landscape-benchmark-report-v0";
const REPORT_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/landscape-benchmark-report-v0.schema.json";
const DEFAULT_SAMPLES: usize = 20;
const MAX_SAMPLES: usize = 1_000;

#[derive(Debug)]
struct Config {
    feature_counts: Vec<usize>,
    samples: usize,
    seed: u32,
    output: Option<PathBuf>,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return;
    }

    if let Err(error) = parse_config(&args).and_then(run) {
        eprintln!("landscape-bench: {error}");
        std::process::exit(2);
    }
}

fn print_usage() {
    println!(
        "Landscape native document-kernel benchmark\n\
         Usage: cargo run --release --features landscape-bench --bin landscape-bench -- [OPTIONS]\n\
         Options:\n\
           --counts N,N,...   Feature counts (default: checked-in workload manifest)\n\
           --samples N        Samples per count, 1..={MAX_SAMPLES} (default: {DEFAULT_SAMPLES})\n\
           --seed N|0xHEX     Synthetic-generator seed (default: manifest seed)\n\
           --out PATH         Write JSON report to PATH instead of stdout\n\
           --help             Show this help\n\n\
         Measures generation, validation, serialization and SHA-256 only.\n\
         Browser frame time and physical/ecological outcomes are not measured."
    );
}

fn manifest_defaults() -> Result<(String, Vec<usize>, u32), String> {
    let manifest: Value = serde_json::from_str(WORKLOAD_MANIFEST)
        .map_err(|error| format!("invalid workload manifest: {error}"))?;
    if manifest["status"].as_str() != Some("protocol-only-no-measurements") {
        return Err("workload manifest status is unsupported".to_string());
    }
    if manifest["coordinate_frame"].as_str() != Some(FRAME_KIND) {
        return Err("workload coordinate frame does not match Rust validator".to_string());
    }
    if manifest["generator"]["implementation"].as_str()
        != Some("src/landscape.rs::synthetic_benchmark_snapshot")
    {
        return Err("workload manifest points to a different synthetic generator".to_string());
    }

    let version = manifest["manifest_version"]
        .as_str()
        .ok_or_else(|| "manifest_version is missing".to_string())?
        .to_string();
    let sizes = manifest["workload_sizes"]
        .as_array()
        .ok_or_else(|| "workload_sizes must be an array".to_string())?;
    let mut counts = Vec::with_capacity(sizes.len());
    for workload in sizes {
        let count = workload["feature_count"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| "feature_count must be a non-negative integer".to_string())?;
        if count == 0 || count > MAX_SNAPSHOT_FEATURES {
            return Err(format!(
                "feature_count {count} is outside 1..={MAX_SNAPSHOT_FEATURES}"
            ));
        }
        counts.push(count);
    }
    if counts.is_empty() {
        return Err("manifest has no workload sizes".to_string());
    }

    let seed_hex = manifest["generator"]["initial_seed_hex"]
        .as_str()
        .ok_or_else(|| "generator initial_seed_hex is missing".to_string())?;
    Ok((version, counts, parse_seed(seed_hex)?))
}

fn parse_seed(value: &str) -> Result<u32, String> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).map_err(|error| format!("invalid hexadecimal seed: {error}"))
    } else {
        value
            .parse::<u32>()
            .map_err(|error| format!("invalid decimal seed: {error}"))
    }
}

fn parse_counts(value: &str) -> Result<Vec<usize>, String> {
    let mut counts = Vec::new();
    let mut seen = HashSet::new();
    for part in value.split(',') {
        let count = part
            .trim()
            .parse::<usize>()
            .map_err(|error| format!("invalid feature count {part:?}: {error}"))?;
        if count == 0 || count > MAX_SNAPSHOT_FEATURES {
            return Err(format!(
                "feature count {count} is outside 1..={MAX_SNAPSHOT_FEATURES}"
            ));
        }
        if !seen.insert(count) {
            return Err(format!("duplicate feature count: {count}"));
        }
        counts.push(count);
    }
    if counts.is_empty() {
        return Err("--counts must include at least one count".to_string());
    }
    Ok(counts)
}

fn parse_config(args: &[String]) -> Result<Config, String> {
    let (_, default_counts, default_seed) = manifest_defaults()?;
    let mut counts = None;
    let mut samples = DEFAULT_SAMPLES;
    let mut seed = default_seed;
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index].as_str();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--counts" => counts = Some(parse_counts(value)?),
            "--samples" => {
                samples = value
                    .parse::<usize>()
                    .map_err(|error| format!("invalid sample count {value:?}: {error}"))?;
                if !(1..=MAX_SAMPLES).contains(&samples) {
                    return Err(format!("--samples must be in 1..={MAX_SAMPLES}"));
                }
            }
            "--seed" => seed = parse_seed(value)?,
            "--out" => output = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown option {option:?}; use --help for usage")),
        }
        index += 2;
    }
    Ok(Config {
        feature_counts: counts.unwrap_or(default_counts),
        samples,
        seed,
        output,
    })
}

fn run(config: Config) -> Result<(), String> {
    let (manifest_version, _, _) = manifest_defaults()?;
    let mut results = Vec::with_capacity(config.feature_counts.len());
    for count in &config.feature_counts {
        results.push(benchmark_count(*count, config.samples, config.seed)?);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before UNIX epoch: {error}"))?
        .as_secs();
    let report = json!({
        "report_schema": REPORT_SCHEMA,
        "schema_uri": REPORT_SCHEMA_URI,
        "benchmark_kind": "native-landscape-document-kernel",
        "manifest_version": manifest_version,
        "created_at_unix_s": timestamp,
        "git_commit_sha": env::var("SOL_ATLAS_COMMIT_SHA")
            .ok()
            .or_else(|| command_output("git", &["rev-parse", "HEAD"])),
        "configuration": {
            "feature_counts": config.feature_counts,
            "samples_per_count": config.samples,
            "seed": config.seed,
            "coordinate_frame": FRAME_KIND
        },
        "environment": {
            "os": env::consts::OS,
            "architecture": env::consts::ARCH,
            "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
            "rustc_version": command_output("rustc", &["--version"]),
            "machine_label": env::var("LANDSCAPE_BENCH_MACHINE").ok(),
            "cpu_model": cpu_model(),
            "ram_bytes": total_memory_bytes(),
            "gpu_label": env::var("LANDSCAPE_BENCH_GPU").ok(),
            "gpu_used_by_this_benchmark": false
        },
        "results": results,
        "interpretation_limits": [
            "Synthetic local drawing units have no geographic or physical meaning.",
            "Timings cover snapshot generation, strict validation, JSON serialization and SHA-256 only.",
            "Browser interaction/frame time and rendering performance are not measured.",
            "The report is not evidence of hydrological, ecological, social or economic outcomes.",
            "No industry/SOTA claim follows without matched runs and task-level studies."
        ]
    });
    let rendered = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("could not serialize benchmark report: {error}"))?
        + "\n";

    match config.output {
        Some(path) => {
            if let Some(parent) = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
            }
            fs::write(&path, rendered)
                .map_err(|error| format!("could not write {}: {error}", path.display()))?;
            eprintln!("Wrote native kernel report to {}", path.display());
        }
        None => print!("{rendered}"),
    }
    Ok(())
}

fn benchmark_count(feature_count: usize, samples: usize, seed: u32) -> Result<Value, String> {
    // Warm-up verifies the contract but is excluded from reported samples.
    let warmup = synthetic_benchmark_snapshot(feature_count, seed)
        .map_err(|error| format!("warm-up generation failed for {feature_count}: {error}"))?;
    validate_snapshot(&warmup)
        .map_err(|errors| format!("warm-up validation failed: {}", errors.join("; ")))?;
    let _ = serde_json::to_vec(&warmup)
        .map_err(|error| format!("warm-up serialization failed: {error}"))?;

    let mut raw_samples = Vec::with_capacity(samples);
    let mut expected_digest: Option<String> = None;
    let mut expected_bytes = None;
    for sample_index in 0..samples {
        let started = Instant::now();
        let snapshot = synthetic_benchmark_snapshot(feature_count, seed)
            .map_err(|error| format!("generation failed for {feature_count}: {error}"))?;
        let generation_ns = elapsed_ns(started);
        let actual_count = snapshot["features"].as_array().map(Vec::len).unwrap_or(0);
        if actual_count != feature_count {
            return Err(format!(
                "generator returned {actual_count} features; expected {feature_count}"
            ));
        }

        let started = Instant::now();
        validate_snapshot(&snapshot)
            .map_err(|errors| format!("validation failed: {}", errors.join("; ")))?;
        let validation_ns = elapsed_ns(started);

        let started = Instant::now();
        let serialized = serde_json::to_vec(&snapshot)
            .map_err(|error| format!("serialization failed: {error}"))?;
        let serialization_ns = elapsed_ns(started);
        let bytes = serialized.len();

        let started = Instant::now();
        let digest = format!("{:x}", Sha256::digest(&serialized));
        let sha256_ns = elapsed_ns(started);
        if expected_digest
            .as_ref()
            .is_some_and(|expected| expected != &digest)
            || expected_bytes.is_some_and(|expected| expected != bytes)
        {
            return Err(format!(
                "non-deterministic normalized snapshot at feature count {feature_count}"
            ));
        }
        expected_digest.get_or_insert_with(|| digest.clone());
        expected_bytes.get_or_insert(bytes);

        let total_kernel_ns = generation_ns
            .saturating_add(validation_ns)
            .saturating_add(serialization_ns)
            .saturating_add(sha256_ns);
        raw_samples.push(json!({
            "sample_index": sample_index,
            "generation_ns": generation_ns,
            "validation_ns": validation_ns,
            "serialization_ns": serialization_ns,
            "sha256_ns": sha256_ns,
            "total_kernel_ns": total_kernel_ns,
            "snapshot_bytes": bytes,
            "snapshot_sha256": digest
        }));
    }

    let scene_sha256 =
        expected_digest.ok_or_else(|| "benchmark produced no samples".to_string())?;
    let snapshot_bytes = expected_bytes.unwrap_or_default();
    Ok(json!({
        "feature_count": feature_count,
        "sample_count": samples,
        "snapshot_bytes": snapshot_bytes,
        "normalized_snapshot_sha256": scene_sha256,
        "latency_ns": {
            "generation": summarize(&raw_samples, "generation_ns"),
            "validation": summarize(&raw_samples, "validation_ns"),
            "serialization": summarize(&raw_samples, "serialization_ns"),
            "sha256": summarize(&raw_samples, "sha256_ns"),
            "total_kernel": summarize(&raw_samples, "total_kernel_ns")
        },
        "raw_samples": raw_samples
    }))
}

fn elapsed_ns(started: Instant) -> u64 {
    started.elapsed().as_nanos().min(u64::MAX as u128) as u64
}

fn summarize(samples: &[Value], field: &str) -> Value {
    let mut values: Vec<u64> = samples
        .iter()
        .filter_map(|sample| sample[field].as_u64())
        .collect();
    values.sort_unstable();
    json!({
        "min_ns": values.first().copied().unwrap_or_default(),
        "p50_ns": percentile_sorted(&values, 50),
        "p95_ns": percentile_sorted(&values, 95),
        "max_ns": values.last().copied().unwrap_or_default()
    })
}

/// Nearest-rank percentile: rank = ceil(q / 100 * N), clamped to 1..=N.
fn percentile_sorted(sorted: &[u64], quantile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (quantile * sorted.len())
        .div_ceil(100)
        .clamp(1, sorted.len());
    sorted[rank - 1]
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn cpu_model() -> Option<String> {
    if let Ok(value) = env::var("LANDSCAPE_BENCH_CPU")
        && !value.trim().is_empty()
    {
        return Some(value.trim().to_string());
    }
    let info = fs::read_to_string("/proc/cpuinfo").ok()?;
    info.lines()
        .find(|line| {
            line.starts_with("model name\t:")
                || line.starts_with("Hardware\t:")
                || line.starts_with("Processor\t:")
        })
        .and_then(|line| {
            line.split_once(':')
                .map(|(_, value)| value.trim().to_string())
        })
        .filter(|value| !value.is_empty())
}

fn total_memory_bytes() -> Option<u64> {
    if let Ok(value) = env::var("LANDSCAPE_BENCH_RAM_BYTES") {
        return value.parse::<u64>().ok();
    }
    let info = fs::read_to_string("/proc/meminfo").ok()?;
    let kibibytes = info
        .lines()
        .find(|line| line.starts_with("MemTotal:"))?
        .split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()?;
    kibibytes.checked_mul(1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_report_schema_has_a_stable_identity() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../schemas/landscape-benchmark-report-v0.schema.json"
        ))
        .expect("benchmark report schema must be valid JSON");
        assert_eq!(schema["$id"], REPORT_SCHEMA_URI);
        assert_eq!(
            schema["properties"]["report_schema"]["const"],
            REPORT_SCHEMA
        );
        assert_eq!(
            schema["properties"]["benchmark_kind"]["const"],
            "native-landscape-document-kernel"
        );
    }

    #[test]
    fn defaults_match_the_checked_in_manifest() {
        let config = parse_config(&[]).unwrap();
        assert_eq!(config.feature_counts, [100, 1_000, 10_000, 100_000]);
        assert_eq!(config.seed, 0x4c41_4e44);
        assert_eq!(config.samples, DEFAULT_SAMPLES);
    }

    #[test]
    fn explicit_counts_are_bounded_unique_and_reproducible() {
        let config = parse_config(&[
            "--counts".to_string(),
            "100, 1000,10000".to_string(),
            "--samples".to_string(),
            "3".to_string(),
            "--seed".to_string(),
            "0x4c414e44".to_string(),
        ])
        .unwrap();
        assert_eq!(config.feature_counts, [100, 1_000, 10_000]);
        assert_eq!(config.samples, 3);
        assert_eq!(config.seed, 0x4c41_4e44);
        assert!(parse_counts("100,100").is_err());
        assert!(parse_counts("100001").is_err());
        assert!(parse_counts("0").is_err());
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        let values: Vec<u64> = (1..=20).collect();
        assert_eq!(percentile_sorted(&values, 50), 10);
        assert_eq!(percentile_sorted(&values, 95), 19);
        assert_eq!(percentile_sorted(&[], 95), 0);
    }

    #[test]
    fn sha256_uses_the_standard_known_answer() {
        assert_eq!(
            format!("{:x}", Sha256::digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn kernel_benchmark_reports_stable_synthetic_output() {
        let report = benchmark_count(100, 2, 0x4c41_4e44).unwrap();
        assert_eq!(report["feature_count"], 100);
        assert_eq!(report["sample_count"], 2);
        assert_eq!(
            report["raw_samples"][0]["snapshot_sha256"],
            report["raw_samples"][1]["snapshot_sha256"]
        );
        let sample = &report["raw_samples"][0];
        let measured_total = sample["generation_ns"].as_u64().unwrap()
            + sample["validation_ns"].as_u64().unwrap()
            + sample["serialization_ns"].as_u64().unwrap()
            + sample["sha256_ns"].as_u64().unwrap();
        assert_eq!(sample["total_kernel_ns"].as_u64().unwrap(), measured_total);
        assert!(report["snapshot_bytes"].as_u64().unwrap() > 0);
    }
}
