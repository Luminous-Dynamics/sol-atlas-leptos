# Energy solver compute calibration v0.1

A dependency-free Rust validator and cohort aggregator for exact solver-run traces. This is the instrumentation layer needed to calibrate temporal-mesh interval budgets against measured solver work.

## Why this exists

The mesh allocator (#408) currently counts intervals. Interval count is a reproducible structural proxy, not solver latency. A serious comparison needs to record real process time, time-to-first-feasible, termination status, model size, memory, solver binary, hardware and exact input identities.

This crate does not invent or infer solver timings. The checked-in CSV is a **synthetic parser fixture**, not a measured benchmark. Real measurements must be produced by the downstream optimizer runner and passed through this validator.

## Trace identity

Every row binds:
- scenario panel, scenario request, solver model and temporal mesh SHA-256;
- solver name/version and exact executable SHA-256;
- hardware profile fingerprint SHA-256;
- horizon and observed model-size counts;
- sample ID, seed, status and timing milestones.

Records with the same cohort ID are rejected if measurement class, scenario panel, model, mesh, solver binary/version, hardware fingerprint or horizon differ. Duplicate sample IDs are rejected. The aggregator never silently pools across machines or solver builds.

The CSV grammar is fixed-column and deliberately excludes quoted/free-form fields. Identifiers must use a comma-free ASCII alphabet; digest fields must be exactly 64 lowercase hexadecimal characters.

## Outcome semantics

Optimal, feasible, infeasible, timed-out, cancelled and error outcomes remain separate denominators. Completed-latency percentiles use only optimal/feasible runs. The observed-attempt elapsed percentile includes timeout termination times and is explicitly labelled right-censored; it must not be interpreted as eventual solve completion latency.

- p50 is emitted only with at least 5 eligible samples.
- p95 is emitted only with at least 20 eligible samples.
- p99 is withheld until at least 1,000 eligible completed samples.
- Nearest-rank quantiles are used; no interpolation hides the discrete sample count.

The current schema records elapsed wall time, time-to-first-feasible, time-to-optimal, objective/best-bound/gap, peak memory and timeout limit. It also retains intervals, binary variables and constraints so cost-vs-size behavior can be stratified.

## Test panel

The crate has **18 unit tests** covering:
- exact digest and comma-free identifier validation;
- timing/status semantics for optimal, feasible, infeasible, timeout and incumbent-bearing records;
- exact cohort isolation across mesh, solver binary, hardware fingerprint and measurement class;
- duplicate sample suppression;
- nearest-rank percentile semantics;
- p50/p95/p99 sample-count gates;
- separate timeout/infeasible denominators from completed-solve latency;
- fixed-header/field-count parsing and an end-to-end parse of the checked-in 30-row fixture.

## Frozen fixture and exact-head workflow

The checked-in 30-row fixture is tagged `synthetic`: 18 optimal, 4 feasible, 4 timed out and 4 infeasible. CI validates the entire strict parser → type validator → cohort guard → summary pipeline and records the fixture digest, aggregator-binary digest, source commit, Rust/Cargo versions and runner profile. Its numbers demonstrate the output schema only; they are not used as a performance claim.

The next real measurement campaign should run the same frozen scenario panel over:
- 60/30/15-minute mesh candidates and each selected horizon;
- fixed solver version/build, exact solver tolerances and exact model formulation;
- a pinned machine profile or explicitly stratified hardware cohorts;
- at least 20 completed attempts for p95, and at least 1,000 completed attempts for p99;
- repeated scenario seeds, with timeouts and infeasibility preserved in the denominator.

Report p50/p95/p99 for completed latency, time-to-first-feasible, memory and observed attempt elapsed, alongside timeout/infeasible/error rates and model-size ranges. Do not convert timeout time into a completed solve time.

## Run

```sh
cargo fmt --manifest-path tools/energy-solver-compute-calibration/Cargo.toml --check
cargo check --locked --all-targets --manifest-path tools/energy-solver-compute-calibration/Cargo.toml
cargo test --locked --all-targets --manifest-path tools/energy-solver-compute-calibration/Cargo.toml
cargo clippy --locked --all-targets --manifest-path tools/energy-solver-compute-calibration/Cargo.toml -- -D warnings
cargo run --release --locked --manifest-path tools/energy-solver-compute-calibration/Cargo.toml -- tools/energy-solver-compute-calibration/fixtures/synthetic-traces.csv
```

## Claim ceiling

This is trace integrity and aggregation machinery. It does not run an optimizer, measure real solver performance by itself, guarantee that a hardware fingerprint is truthful, prove model correctness, or establish SOTA superiority. Every measured trace still needs an independent source/model/schedule identity chain. The first real output should be published only after an instrumented solver runner emits actual executable and host fingerprints.
