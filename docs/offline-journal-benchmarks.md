# Offline Journal Protocol Benchmarks

**Status:** benchmark harness implemented; no performance results have been measured or claimed yet.

The CI workflow also runs a ten-sample smoke cohort on GitHub-hosted `ubuntu-latest` and uploads `offline-journal-bench-<commit SHA>` containing raw CSV and the build log. Treat this only as a noisy, shared-virtualized CI baseline; it is useful for detecting gross regressions and proving that the harness executes, not for a headline latency or industry comparison.

The standalone Rust benchmark measures the CPU-side cost of canonical payload hashing, constructing an immutable canonical-payload value, deriving the scoped request fingerprint, preparing a journal entry, and serializing/decoding/revalidating a snapshot. It emits per-sample CSV rows and p50/p95/p99 summary rows for four exact payload sizes. It does **not** measure IndexedDB commit latency, browser event-loop behavior, network timing, server-side idempotency, or end-to-end queue recovery.

## Run reproducibly

From the repository root:

```sh
{
  printf '# rustc_version=%s\n' "$(rustc --version)"
  printf '# source_revision=%s\n' "$(git rev-parse HEAD)"
  SOL_ATLAS_BENCH_SAMPLES=50 \
  SOL_ATLAS_BENCH_BUILD_SHA="$(git rev-parse HEAD)" \
    cargo bench -p sol-atlas-core --bench offline_journal_protocol
} > offline-journal-bench.csv 2> offline-journal-bench-build.log
```

Use the optimized `cargo bench` profile. The benchmark defaults to 50 measured samples per operation; the environment variable accepts 5–2,000. Every CSV sample includes payload bytes, sample index, iterations and nanoseconds per operation. Summary rows include nearest-rank p50/p95/p99 and operations/second computed from p50. Header comments record package version, operating system, architecture, sample count and the supplied source SHA.

The harness performs up to eight warm-up iterations per operation outside timed samples. The default sample count is enough to establish an initial view, not to establish a publication-grade performance claim. The CI artifact records timestamp, source revision, Rust compiler version, host kernel string, CPU model and runner class. It is still not a controlled benchmark environment. For comparative work, run multiple independent processes on the same named hardware under stable thermal/power conditions. Archive the full raw CSV, stderr/build log, exact commit SHA, Rust compiler version, CPU model, OS/kernel, power mode, and exact command. Do not trim out slow samples or merge results from different devices into one percentile distribution.

## Workloads

Each payload is valid JSON containing a string with exactly the target total byte length:

- 1 KiB;
- 32 KiB;
- 256 KiB;
- 1 MiB.

The harness measures:

| Operation | What is timed |
|---|---|
| `canonical_payload_sha256` | Strict JCS canonicalization plus SHA-256, returning digest hex |
| `canonical_payload_value` | Strict canonicalization plus retained bytes and digest |
| `scoped_request_fingerprint_sha256` | Payload validation/hash plus canonical immutable request fingerprint |
| `journal_entry_prepare` | Envelope and raw payload validation, canonicalization, digest binding and immutable entry creation |
| `snapshot_serialize_decode_recover` | JSON snapshot serialization, decoding, and schema/canonical-byte/digest/state recovery validation |

The harness targets approximately 64 KiB of input bytes per sample for each cohort (minimum one iteration). That means iteration counts differ by payload size; do not infer allocation counts or fine-grained tail behavior from one iteration. p95/p99 with 50 samples are coarse order-statistic summaries and should be reported with sample count.

## Compare to industry honestly

These timings can establish the Rust core's local CPU cost; by themselves they do not establish that Sol Atlas is faster than an offline-first database, browser IndexedDB, a mature idempotency service, or competing products. A valid competitive comparison must implement the same correctness semantics and report the whole-system boundary being measured.

Before claiming a SOTA result, add named reference devices and browser-engine runs for transaction commit/recovery plus an independently implemented authoritative server. Compare at least:

- end-to-end enqueue-to-durable-commit latency;
- durable pre-dispatch permit latency;
- cold recovery for 1k / 10k / 100k records;
- bounded pending-page retrieval at sizes 50 / 100 / 500;
- end-to-end retries and receipt recovery under response loss;
- concurrency 1 / 16 / 64 / 256 with p50/p95/p99 and throughput.

Every result must be accompanied by a run manifest: source SHA, dependency lock/revision, Rust/compiler version, browser/device/server/database versions, exact command, test seed, raw samples and failure count.

## Non-negotiable correctness gates

Performance data is admissible only when the corresponding correctness suite passes. The intended service-level target is zero duplicate logical effects across 100,000 randomized retries/response-loss/crash injections, zero payload/fingerprint mismatches accepted, zero dispatch permits after failed/indeterminate durable commits, and exact canonical-byte/fingerprint agreement across Rust and an independent service implementation. A performance improvement never compensates for one integrity failure.

## Next optimization loop

1. Capture a named-machine baseline from this harness.
2. Profile allocations and CPU hotspots without weakening the canonicalization profile.
3. Optimize only a proven hotspot (e.g., avoid re-canonicalizing a trusted private immutable byte buffer within one operation while still re-verifying at recovery/server boundaries).
4. Re-run identical raw benchmark cohorts and the exact conformance suite; publish before/after CSV plus the code SHA.
5. Add browser/server integration benchmarks before making end-to-end comparative claims.
