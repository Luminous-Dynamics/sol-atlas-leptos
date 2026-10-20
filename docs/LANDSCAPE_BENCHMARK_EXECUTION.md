# Landscape native benchmark execution protocol

The executable is feature-gated so process/file access cannot accidentally become part of the deployed WASM editor.

## Run

~~~sh
cargo test --features landscape-bench --bin landscape-bench
cargo run --release --features landscape-bench --bin landscape-bench -- --out reports/landscape/native-kernel-v0.json
~~~

For a quick, low-cost smoke run:

~~~sh
cargo run --release --features landscape-bench --bin landscape-bench -- --counts 100,1000 --samples 2
~~~

The default run uses the feature counts and seed from `benchmarks/landscape/workloads-v0.json`, including 100,000 features. CLI overrides are recorded in the output, and duplicate/out-of-range counts are rejected.

## What is measured

Each raw sample separates synthetic scene generation, strict snapshot validation, normalized JSON serialization, and SHA-256 computation. The report records output byte size, SHA-256 fingerprint, raw nanosecond timings, p50/p95 summaries, sample count, seed, commit, OS/architecture, build profile, Rust version, and available CPU/memory metadata.

Set `LANDSCAPE_BENCH_MACHINE`, `LANDSCAPE_BENCH_CPU`, `LANDSCAPE_BENCH_RAM_BYTES`, and optionally `LANDSCAPE_BENCH_GPU` to supply labels when automatically collected metadata is missing or virtualized.

## Interpretation limits

This native benchmark does **not** measure browser frame times, pointer-event latency, visual quality, CAD/GIS interoperability, task completion, or real environmental benefits. The small CI smoke run verifies execution and schema-shaped output; it is not the reference-machine benchmark.

Before publishing a performance claim, preserve the raw report, exact commit SHA, machine/toolchain, profile, cache state, full sample count, and input manifest digest. Do not compare GitHub-hosted smoke timings with a local release baseline. A browser harness and matched task studies remain separate work.
