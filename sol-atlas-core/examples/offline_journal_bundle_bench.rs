// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Reproducible microbenchmark for the portable offline-journal bundle codec.
//!
//! Example:
//! cargo run --release --manifest-path sol-atlas-core/Cargo.toml --example offline_journal_bundle_bench -- --records 10000 --blob-bytes 1024 --samples 9
//!
//! Output is machine-readable JSON with raw samples and percentile summaries.
//! Set GIT_SHA and HOST_LABEL externally when collecting results for comparison.

use std::{
    env,
    error::Error,
    process::Command,
    time::{Duration, Instant},
};

use serde::Serialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::{
    offline_journal::{
        JournalEntry, OFFLINE_JOURNAL_SCHEMA_VERSION, OperationEnvelope,
        RequiredNullableOperationId,
    },
    offline_journal_bundle::{OfflineJournalBundleRecordV1, OfflineJournalBundleV1},
    offline_payload_digest::{canonical_payload_digest_sha256, canonicalize_payload_json},
};

#[derive(Debug, Clone, Copy)]
struct Options {
    records: usize,
    blob_bytes: usize,
    samples: usize,
}

#[derive(Debug, Serialize)]
struct BenchmarkParameters {
    records: usize,
    blob_bytes_per_payload: usize,
    payload_json_bytes: usize,
    samples: usize,
    warmup_iterations: usize,
    owner_scope: String,
    operation_namespace: String,
}

#[derive(Debug, Serialize)]
struct Sample {
    sample_index: usize,
    stage: &'static str,
    elapsed_ns: u64,
}

#[derive(Serialize)]
struct UnsignedManifestBaseline<'a> {
    schema_version: u16,
    owner_scope: &'a str,
    records: &'a [OfflineJournalBundleRecordV1],
}

#[derive(Debug, Serialize)]
struct StageSummary {
    stage: &'static str,
    sample_count: usize,
    min_ns: u64,
    median_ns: u64,
    p95_ns: u64,
    p99_ns: u64,
    max_ns: u64,
    median_records_per_second: f64,
}

#[derive(Debug, Serialize)]
struct BenchmarkReport {
    schema_version: u16,
    benchmark: &'static str,
    git_sha: String,
    host_label: String,
    os: &'static str,
    architecture: &'static str,
    rustc_version: String,
    parameters: BenchmarkParameters,
    bundle_json_bytes: usize,
    bare_serde_json_bytes: usize,
    canonical_hash_only_manifest_bytes: usize,
    bytes_per_record: f64,
    bare_serde_json_bytes_per_record: f64,
    canonical_hash_only_bytes_per_record: f64,
    baseline_note: &'static str,
    canonical_hash_only_note: &'static str,
    bundle_integrity_sha256: String,
    stage_summaries: Vec<StageSummary>,
    raw_samples: Vec<Sample>,
    correctness: Correctness,
}

#[derive(Debug, Serialize)]
struct Correctness {
    roundtrip_equal: bool,
    record_count_preserved: bool,
    owner_scope_preserved: bool,
    integrity_digest_stable: bool,
    canonical_bytes_stable: bool,
    bare_serde_json_roundtrip_equal: bool,
    canonical_hash_only_matches_bundle_digest: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("offline_journal_bundle_bench: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let options = parse_options()?;
    let owner_scope = "benchmark-tenant";
    let operation_namespace = "benchmark.operation";
    let payload_json = format!(r#"{{"blob":"{}"}}"#, "a".repeat(options.blob_bytes));
    let payload_digest = canonical_payload_digest_sha256(&payload_json)?;
    let records = make_records(
        options.records,
        operation_namespace,
        &payload_json,
        &payload_digest,
    )?;

    // Warm code paths, allocator and JCS caches before capturing samples.
    let warmup = OfflineJournalBundleV1::create(owner_scope, records.clone())?;
    let warmup_json = warmup.to_json()?;
    let _ = OfflineJournalBundleV1::from_json(&warmup_json)?;

    let mut raw_samples = Vec::with_capacity(options.samples * 5);
    let mut bundle_bytes = 0usize;
    let mut bare_serde_json_bytes = 0usize;
    let mut canonical_hash_only_manifest_bytes = 0usize;
    let mut bundle_digest = String::new();
    let mut reference_bundle_json: Option<String> = None;
    let mut correctness = Correctness {
        roundtrip_equal: true,
        record_count_preserved: true,
        owner_scope_preserved: true,
        integrity_digest_stable: true,
        canonical_bytes_stable: true,
        bare_serde_json_roundtrip_equal: true,
        canonical_hash_only_matches_bundle_digest: true,
    };

    for sample_index in 0..options.samples {
        let input_records = records.clone();
        let start = Instant::now();
        let bundle = OfflineJournalBundleV1::create(owner_scope, input_records)?;
        let elapsed = elapsed_ns(start.elapsed());
        raw_samples.push(Sample {
            sample_index,
            stage: "create_validate_sort_and_digest",
            elapsed_ns: elapsed,
        });

        let start = Instant::now();
        let json = bundle.to_json()?;
        let elapsed = elapsed_ns(start.elapsed());
        raw_samples.push(Sample {
            sample_index,
            stage: "canonical_serialize_and_validate",
            elapsed_ns: elapsed,
        });

        let start = Instant::now();
        let parsed = OfflineJournalBundleV1::from_json(&json)?;
        let elapsed = elapsed_ns(start.elapsed());
        raw_samples.push(Sample {
            sample_index,
            stage: "parse_validate_and_recompute_digest",
            elapsed_ns: elapsed,
        });

        // Informational, deliberately non-equivalent baseline: canonical JCS
        // encode + SHA-256, but without revalidating every snapshot's semantics.
        // The input bundle was already created/validated above; this timed stage
        // quantifies serialization/hash cost separately from semantic validation.
        let start = Instant::now();
        let (canonical_hash_only_digest, canonical_hash_only_bytes) =
            canonical_hash_only_manifest_digest(&bundle)?;
        let elapsed = elapsed_ns(start.elapsed());
        raw_samples.push(Sample {
            sample_index,
            stage: "canonical_jcs_and_sha256_without_snapshot_validation",
            elapsed_ns: elapsed,
        });

        // Informational, deliberately non-equivalent baseline: typed Serde JSON
        // roundtrip without JCS canonicalization, snapshot validation, or digest
        // recomputation. This quantifies overhead but is not a security substitute.
        let start = Instant::now();
        let bare_json = serde_json::to_string(&bundle)?;
        let bare_parsed: OfflineJournalBundleV1 = serde_json::from_str(&bare_json)?;
        let elapsed = elapsed_ns(start.elapsed());
        raw_samples.push(Sample {
            sample_index,
            stage: "serde_json_unverified_roundtrip_baseline",
            elapsed_ns: elapsed,
        });

        correctness.bare_serde_json_roundtrip_equal &= bare_parsed == bundle;
        correctness.canonical_hash_only_matches_bundle_digest &=
            canonical_hash_only_digest == bundle.integrity_sha256;
        correctness.roundtrip_equal &= parsed == bundle;
        correctness.record_count_preserved &= parsed.records.len() == options.records;
        correctness.owner_scope_preserved &= parsed.owner_scope == owner_scope;
        correctness.integrity_digest_stable &= parsed.integrity_sha256 == bundle.integrity_sha256;
        if let Some(reference_json) = reference_bundle_json.as_ref() {
            correctness.canonical_bytes_stable &= reference_json == &json;
        } else {
            reference_bundle_json = Some(json.clone());
        }
        bundle_bytes = json.len();
        bare_serde_json_bytes = bare_json.len();
        canonical_hash_only_manifest_bytes = canonical_hash_only_bytes;
        bundle_digest = bundle.integrity_sha256;
    }

    if !(correctness.roundtrip_equal
        && correctness.record_count_preserved
        && correctness.owner_scope_preserved
        && correctness.integrity_digest_stable
        && correctness.canonical_bytes_stable
        && correctness.bare_serde_json_roundtrip_equal
        && correctness.canonical_hash_only_matches_bundle_digest)
    {
        return Err("benchmark correctness oracle failed".into());
    }

    let stage_summaries = summarize(&raw_samples, options.records);
    let report = BenchmarkReport {
        schema_version: 1,
        benchmark: "sol-atlas-offline-journal-bundle-v1",
        git_sha: env::var("GIT_SHA").unwrap_or_else(|_| "unset".to_owned()),
        host_label: env::var("HOST_LABEL").unwrap_or_else(|_| "unset".to_owned()),
        os: env::consts::OS,
        architecture: env::consts::ARCH,
        rustc_version: rustc_version(),
        parameters: BenchmarkParameters {
            records: options.records,
            blob_bytes_per_payload: options.blob_bytes,
            payload_json_bytes: payload_json.len(),
            samples: options.samples,
            warmup_iterations: 1,
            owner_scope: owner_scope.to_owned(),
            operation_namespace: operation_namespace.to_owned(),
        },
        bundle_json_bytes: bundle_bytes,
        bare_serde_json_bytes,
        canonical_hash_only_manifest_bytes,
        bytes_per_record: bundle_bytes as f64 / options.records as f64,
        bare_serde_json_bytes_per_record: bare_serde_json_bytes as f64 / options.records as f64,
        canonical_hash_only_bytes_per_record: canonical_hash_only_manifest_bytes as f64
            / options.records as f64,
        baseline_note: "Bare Serde roundtrip omits canonicalization, semantic snapshot validation, and integrity digest verification; informational only, not an equivalent security baseline.",
        canonical_hash_only_note: "JCS+SHA-256 only over typed serialized manifest; skips per-record semantic snapshot validation, and is therefore not equivalent to full contract validation.",
        bundle_integrity_sha256: bundle_digest,
        stage_summaries,
        raw_samples,
        correctness,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn parse_options() -> Result<Options, Box<dyn Error>> {
    let mut options = Options {
        records: 1_000,
        blob_bytes: 1_024,
        samples: 7,
    };
    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value after {flag}"))?;
        match flag.as_str() {
            "--records" => options.records = value.parse()?,
            "--blob-bytes" => options.blob_bytes = value.parse()?,
            "--samples" => options.samples = value.parse()?,
            _ => return Err(format!("unknown argument {flag}").into()),
        }
    }
    if options.records == 0 {
        return Err("--records must be greater than zero".into());
    }
    if options.samples == 0 {
        return Err("--samples must be greater than zero".into());
    }
    // Cap input sizes to catch accidental multi-gigabyte runs caused by typos.
    if options.records > 100_000 {
        return Err("--records is capped at 100000".into());
    }
    if options.blob_bytes > 1_048_576 {
        return Err("--blob-bytes is capped at 1048576".into());
    }
    if options.samples > 1_000 {
        return Err("--samples is capped at 1000".into());
    }
    Ok(options)
}

fn make_records(
    count: usize,
    operation_namespace: &str,
    payload_json: &str,
    payload_digest: &str,
) -> Result<Vec<OfflineJournalBundleRecordV1>, Box<dyn Error>> {
    let mut records = Vec::with_capacity(count);
    for index in 0..count {
        let operation_id = format!("op-{index:08}");
        let envelope = OperationEnvelope {
            schema_version: OFFLINE_JOURNAL_SCHEMA_VERSION,
            operation_id,
            aggregate_id: format!("aggregate-{index:08}"),
            expected_revision: index as u64,
            payload_digest_sha256: payload_digest.to_owned(),
            idempotency_key: format!("idem-{index:08}"),
            supersedes_operation_id: RequiredNullableOperationId::Null(()),
        };
        let entry = JournalEntry::prepare(envelope, payload_json)?;
        records.push(OfflineJournalBundleRecordV1 {
            operation_namespace: operation_namespace.to_owned(),
            snapshot: entry.to_snapshot()?,
        });
    }
    Ok(records)
}

fn canonical_hash_only_manifest_digest(
    bundle: &OfflineJournalBundleV1,
) -> Result<(String, usize), Box<dyn Error>> {
    let unsigned = UnsignedManifestBaseline {
        schema_version: bundle.schema_version,
        owner_scope: &bundle.owner_scope,
        records: &bundle.records,
    };
    let serialized = serde_json::to_string(&unsigned)?;
    let canonical = canonicalize_payload_json(&serialized)?;
    let digest = Sha256::digest(&canonical);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}")?;
    }
    Ok((hex, canonical.len()))
}

fn summarize(samples: &[Sample], record_count: usize) -> Vec<StageSummary> {
    let stages = [
        "create_validate_sort_and_digest",
        "canonical_serialize_and_validate",
        "parse_validate_and_recompute_digest",
        "canonical_jcs_and_sha256_without_snapshot_validation",
        "serde_json_unverified_roundtrip_baseline",
    ];
    stages
        .iter()
        .map(|stage| {
            let mut values = samples
                .iter()
                .filter(|sample| sample.stage == *stage)
                .map(|sample| sample.elapsed_ns)
                .collect::<Vec<_>>();
            values.sort_unstable();
            let median_ns = percentile(&values, 0.50);
            let seconds = (median_ns as f64 / 1_000_000_000.0).max(1e-9);
            StageSummary {
                stage,
                sample_count: values.len(),
                min_ns: values.first().copied().unwrap_or_default(),
                median_ns,
                p95_ns: percentile(&values, 0.95),
                p99_ns: percentile(&values, 0.99),
                max_ns: values.last().copied().unwrap_or_default(),
                median_records_per_second: record_count as f64 / seconds,
            }
        })
        .collect()
}

fn percentile(sorted: &[u64], percentile: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = ((percentile * sorted.len() as f64).ceil() as usize)
        .max(1)
        .min(sorted.len());
    sorted[rank - 1]
}

fn elapsed_ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u64::MAX as u128) as u64
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "unavailable".to_owned())
}
