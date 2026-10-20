# Offline journal benchmark protocol v1

**Status:** reproducible native bundle-codec harness; no performance result is asserted by this document. CI establishes compilation/tests, not speed. Results must be generated on named hardware with raw samples retained.

## Run the native core harness

From repository root, run a three-size quick set or the full matrix:

```sh
GIT_SHA="$(git rev-parse HEAD)" \
HOST_LABEL="describe CPU / RAM / power mode / OS image" \
cargo run --release \
  --manifest-path sol-atlas-core/Cargo.toml \
  --example offline_journal_bundle_bench -- \
  --records 1000 --blob-bytes 1024 --samples 11 \
  > offline-journal-bundle-bench.json
```

```sh
bash scripts/bench-offline-journal-bundle.sh quick
bash scripts/bench-offline-journal-bundle.sh full /path/to/evidence-dir
```

The script writes one JSON report per cohort plus `environment.txt` and a `SHA256SUMS` manifest. It records the Git SHA, dirty-tree state, compiler/tool versions, OS, CPU and memory summary. Review the dirty-tree flag before comparing results, and do not publish a run from an uncommitted or mixed source tree as an exact release baseline.

The program writes a single JSON report to stdout and diagnostics to stderr. The report includes the exact input dimensions, Rust version, OS/architecture, caller-provided host label and Git SHA, bundle bytes / bytes per record, integrity digest, all raw sample durations, percentile summaries, and correctness-oracle results. A run whose correctness oracle fails exits unsuccessfully.

The deterministic workload varies operation IDs, aggregate IDs, expected revisions and idempotency keys; all entries share a fixed-size JSON string field so the bundle has a predictable payload-size cohort. Creation and validation are timed after one warmup. The raw samples are more important than the summary: archive the JSON report unchanged.

### Timed stages

- `create_validate_sort_and_digest`: validate records, normalize sort order, and calculate the unsigned manifest integrity digest.
- `canonical_serialize_and_validate`: validate and serialize as canonical JCS JSON.
- `parse_validate_and_recompute_digest`: parse the wire representation, validate all records and the outer integrity digest.
- `canonical_jcs_and_sha256_without_snapshot_validation`: serialize the typed unsigned manifest, canonicalize its bytes with JCS, and hash it, while omitting per-record semantic snapshot validation. Its digest must match the full codec for a valid bundle.
- `serde_json_unverified_roundtrip_baseline`: a deliberately *non-equivalent* informational baseline using plain Serde JSON encode/decode without canonical JCS encoding, semantic validation or integrity recomputation.

Both baselines are not competitor implementations for correctness and cannot replace the canonical contract. The JCS+SHA-256-only stage quantifies the cost of snapshot-validation gates; the bare Serde stage estimates the further cost of canonicalization and integrity checks. Since the input bundle has already been created and validated before either baseline is timed, neither represents safe ingestion of untrusted data. Compare full-contract paths when comparing systems.

The harness currently reports nearest-rank min, median, p95, p99 and max across measured samples. Small sample counts make p95/p99 noisy; use at least 31 runs for publishable tail statistics and increase repetitions for sub-millisecond samples. Compare like-for-like optimized builds, hardware, power settings, input corpus and schema versions.

## Test matrix and first hypotheses

Run a matrix, not a single cherry-picked size:

| Records | Payload blob per record | Samples | Purpose |
|---:|---:|---:|---|
| 100 | 1 KiB | 31 | sanity / small bundle overhead |
| 1,000 | 1 KiB | 31 | typical low-volume export |
| 10,000 | 1 KiB | 31 | mid-sized journal |
| 10,000 | 32 KiB | 31 | realistic larger operation payloads |
| 1,000 | 256 KiB | 31 | large-payload throughput without excessive fixture memory |
| 100,000 | 1 KiB | 11+ | scalability/stress cohort |
| 100 | 1 MiB | 11+ | maximum individual blob stress; record memory limits |

Large `records × payload` combinations grow memory rapidly and must only run on hosts that can safely retain both the records and encoded output. The 1 MiB payload case intentionally uses only 100 records to limit its default memory footprint. Report failed/out-of-memory runs; do not omit them without explanation.

The initial proposed target is **under 1 second to validate a 10,000-record / 1 KiB-blob bundle on a named modern desktop**. This is a hypothesis to measure, not a published result or a universal device guarantee. Report time and memory; consider streaming/incremental formats if fully materialized snapshots become the bottleneck.

## Browser persistence matrix

The native harness intentionally isolates the pure codec; it does not measure IndexedDB. A separate browser harness must measure at least Chromium, Firefox and WebKit on named versions, plus a representative low-resource field device, for:

- cold export from one readonly transaction at 1k/10k/100k records;
- staged import preflight and commit latency, including collision/abort scenarios;
- time to first visible staged record and total reconciliation time;
- peak heap, serialized bundle bytes, and bytes per record;
- quota exhaustion, tab suspension, origin eviction simulation where possible, and transaction abort behavior.

Keep correctness/fault tests separate from throughput tests. No performance aggregate can compensate for a single cross-owner accept, partial staging transaction, forged receipt promoted to success, malformed snapshot accepted, or duplicate authoritative service effect.

## Cross-industry SOTA comparison

Do not compare Sol Atlas's full JCS + snapshot-validation + digest-verification path to a plain JSON parser and call the result a win. Compare equivalent semantics against:
1. a plain typed-JSON baseline (included as clearly non-equivalent context);
2. a canonicalization + hash-only baseline;
3. a full validation baseline with schema/state/payload checks;
4. an IndexedDB implementation measured under the same browser, durability policy, corpus and read/write workload;
5. an independent implementation of the same canonical bundle schema and frozen fixtures.

The key potential advantage is not raw JSON throughput alone. It is a measurable package: stable portable bytes, fail-closed snapshot integrity, owner scope and namespace preservation, staged import without dispatch authority, conflict detection, and a reproducible cross-runtime corpus. This becomes a real differentiator only when third-party implementations pass the same tests and raw comparable measurements support the claim.

## Publication record

Every published result must include:
- exact Git SHA and clean/dirty state;
- Rust/compiler and dependency lock identity;
- CPU model, core count, RAM, OS/kernel, browser/version where relevant, power profile and virtualization;
- corpus dimensions and digest;
- raw sample JSON, benchmark command, sample count and warmup policy;
- correctness oracle and failure count;
- median/p95/p99 with method declared.

Until a measured report is committed, all target numbers remain proposed thresholds, not achieved outcomes.
