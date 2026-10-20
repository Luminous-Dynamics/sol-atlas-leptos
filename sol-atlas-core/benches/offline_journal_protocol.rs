// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Reproducible microbenchmarks for the CPU-side portion of the offline journal.
//!
//! Run with:
//!   SOL_ATLAS_BENCH_SAMPLES=50 cargo bench -p sol-atlas-core --bench offline_journal_protocol
//!
//! Output is CSV on stdout: every per-sample latency is retained, followed by
//! summary rows. The benchmark does not measure browser storage, IndexedDB commit
//! latency, network latency, or server idempotency. Those require separate target
//! browser/device and authoritative-service benchmarks.

use std::hint::black_box;
use std::time::Instant;

use sol_atlas_core::offline_idempotency_fingerprint::canonical_request_fingerprint_sha256;
use sol_atlas_core::offline_journal::{
    JournalEntry, JournalEntrySnapshotV1, OFFLINE_JOURNAL_SCHEMA_VERSION, OperationEnvelope,
    RequiredNullableOperationId,
};
use sol_atlas_core::offline_payload_digest::{CanonicalPayload, canonical_payload_digest_sha256};

const TENANT_SCOPE: &str = "bench-tenant";
const OPERATION_NAMESPACE: &str = "offline-journal.bench.apply";
const TARGET_PAYLOAD_BYTES: [usize; 4] = [1_024, 32_768, 262_144, 1_048_576];

fn main() {
    let samples = std::env::var("SOL_ATLAS_BENCH_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| (5..=2_000).contains(value))
        .unwrap_or(50);

    println!("# benchmark=offline_journal_protocol");
    println!("# crate_version={}", env!("CARGO_PKG_VERSION"));
    println!("# target_os={}", std::env::consts::OS);
    println!("# target_arch={}", std::env::consts::ARCH);
    println!(
        "# source_sha={}",
        option_env!("SOL_ATLAS_BENCH_BUILD_SHA").unwrap_or("not-set")
    );
    println!("# samples_per_operation={samples}");
    println!(
        "record_type,operation,payload_bytes,sample_index,iterations,ns_per_op,p50_ns,p95_ns,p99_ns,ops_per_second"
    );

    for target_bytes in TARGET_PAYLOAD_BYTES {
        let payload_json = payload_of_exact_size(target_bytes);
        assert_eq!(payload_json.len(), target_bytes, "payload fixture size");

        let payload_digest = canonical_payload_digest_sha256(&payload_json)
            .expect("synthetic benchmark JSON must be accepted by strict JCS");
        let envelope = OperationEnvelope {
            schema_version: OFFLINE_JOURNAL_SCHEMA_VERSION,
            operation_id: format!("bench-op-{target_bytes}"),
            aggregate_id: "bench-aggregate".to_owned(),
            expected_revision: 7,
            payload_digest_sha256: payload_digest,
            idempotency_key: format!("bench-idem-{target_bytes}"),
            supersedes_operation_id: RequiredNullableOperationId::Null(()),
        };
        let entry = JournalEntry::prepare(envelope.clone(), &payload_json)
            .expect("prepare benchmark entry");
        let snapshot: JournalEntrySnapshotV1 =
            entry.to_snapshot().expect("export benchmark snapshot");

        // Keep a roughly fixed amount of input bytes per sample to reduce the
        // runtime difference caused purely by the payload-size cohort.
        let iterations = (64 * 1024 / target_bytes).max(1);
        run_benchmark(
            "canonical_payload_sha256",
            target_bytes,
            samples,
            iterations,
            || {
                black_box(
                    canonical_payload_digest_sha256(black_box(&payload_json))
                        .expect("payload digest"),
                )
            },
        );
        run_benchmark(
            "canonical_payload_value",
            target_bytes,
            samples,
            iterations,
            || {
                black_box(
                    CanonicalPayload::from_json(black_box(&payload_json))
                        .expect("canonical payload"),
                )
            },
        );
        run_benchmark(
            "scoped_request_fingerprint_sha256",
            target_bytes,
            samples,
            iterations,
            || {
                black_box(
                    canonical_request_fingerprint_sha256(
                        TENANT_SCOPE,
                        OPERATION_NAMESPACE,
                        &envelope,
                        black_box(&payload_json),
                    )
                    .expect("request fingerprint"),
                )
            },
        );
        run_benchmark(
            "journal_entry_prepare",
            target_bytes,
            samples,
            iterations,
            || {
                black_box(
                    JournalEntry::prepare(envelope.clone(), black_box(&payload_json))
                        .expect("journal entry preparation"),
                )
            },
        );
        run_benchmark(
            "snapshot_serialize_decode_recover",
            target_bytes,
            samples,
            iterations,
            || {
                let encoded =
                    serde_json::to_vec(black_box(&snapshot)).expect("snapshot serialization");
                let decoded: JournalEntrySnapshotV1 =
                    serde_json::from_slice(black_box(&encoded)).expect("snapshot decoding");
                black_box(
                    JournalEntry::from_snapshot_for_operation(&envelope.operation_id, decoded)
                        .expect("validated snapshot recovery"),
                )
            },
        );
    }
}

fn payload_of_exact_size(target_bytes: usize) -> String {
    const OVERHEAD: usize = br#"{"data":""}"#.len();
    assert!(
        target_bytes > OVERHEAD,
        "target size must fit the JSON wrapper"
    );
    format!("{{\"data\":\"{}\"}}", "x".repeat(target_bytes - OVERHEAD))
}

fn run_benchmark<T, F>(
    operation: &str,
    payload_bytes: usize,
    samples: usize,
    iterations: usize,
    mut operation_fn: F,
) where
    F: FnMut() -> T,
{
    // Warm the code path and payload-size cohort outside timed samples.
    // Cap the warm-up so the largest payloads do not dominate the benchmark run.
    let warmup_iterations = iterations.min(8);
    for _ in 0..warmup_iterations {
        black_box(operation_fn());
    }

    let mut per_operation_ns = Vec::with_capacity(samples);
    for sample_index in 0..samples {
        let started = Instant::now();
        for _ in 0..iterations {
            black_box(operation_fn());
        }
        let ns_per_op = started.elapsed().as_nanos() as f64 / iterations as f64;
        per_operation_ns.push(ns_per_op);
        println!(
            "sample,{operation},{payload_bytes},{sample_index},{iterations},{ns_per_op:.2},,,,"
        );
    }

    per_operation_ns.sort_by(f64::total_cmp);
    let p50 = percentile(&per_operation_ns, 0.50);
    let p95 = percentile(&per_operation_ns, 0.95);
    let p99 = percentile(&per_operation_ns, 0.99);
    let ops_per_second = if p50 > 0.0 {
        1_000_000_000.0 / p50
    } else {
        0.0
    };
    println!(
        "summary,{operation},{payload_bytes},{samples},{iterations},,{p50:.2},{p95:.2},{p99:.2},{ops_per_second:.2}"
    );
}

fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    let index = ((fraction * sorted.len() as f64).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[index]
}
