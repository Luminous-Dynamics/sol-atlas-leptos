# Energy solver compute calibration v0.2

A dependency-free Rust validator and cohort aggregator for exact solver-run traces. This is the instrumentation layer needed to calibrate temporal-mesh interval budgets against measured solver work.

## Why this exists

The mesh allocator (#408) currently counts intervals. Interval count is a reproducible structural proxy, not solver latency. A serious comparison needs to record real process time, time-to-first-feasible, termination status, model size, memory, solver binary, hardware and exact input identities.

This crate does not invent or infer solver timings. The checked-in CSV is a **synthetic parser fixture**, not a measured benchmark. Real measurements must be produced by the downstream optimizer runner and passed through this validator.

## Trace identity

Every row binds:
- scenario panel, exact scenario request, solver model, temporal-mesh policy, exact mesh instance and solver-configuration SHA-256;
- solver name/version and exact executable SHA-256;
- hardware profile fingerprint SHA-256;
- horizon and observed model-size counts;
- sample ID, seed, status and timing milestones.

Records with the same cohort ID are rejected if measurement class, scenario panel, model formulation, mesh policy, solver binary/version, solver configuration, hardware fingerprint or horizon differ. Exact mesh-instance digests and request digests vary by sample, but are retained on every row. Duplicate sample IDs are rejected. The aggregator never silently pools across machines or solver builds.

The CSV grammar is fixed-column and deliberately excludes quoted/free-form fields. Identifiers must use a comma-free ASCII alphabet; digest fields must be exactly 64 lowercase hexadecimal characters.

## Outcome semantics

Optimal, feasible, infeasible, timed-out, expansion-limit-reached, cancelled and error outcomes remain separate denominators. Completed-latency percentiles use only optimal/feasible runs. The observed-attempt elapsed percentile includes timeout termination times and is explicitly labelled right-censored; it must not be interpreted as eventual solve completion latency.

- p50 is emitted only with at least 5 eligible samples.
- p95 is emitted only with at least 20 eligible samples.
- p99 is withheld until at least 1,000 eligible completed samples.
- Nearest-rank quantiles are used; no interpolation hides the discrete sample count.

The 34-column schema records elapsed wall time, optional time-to-first-feasible and time-to-optimal, optional scalar objective/best-bound/gap, peak memory, optional timeout limit, explicit mathematical-program size where available, and exact-search action expansions / feasible complete schedules / Pareto-frontier size when exposed. Missing metrics are `NA`, not zero. Exact multi-objective enumeration may certify exhaustive optimality from complete-search counters without inventing a scalar objective or first-hit time.

## Test panel

The crate has **24 unit tests** covering:
- exact digest and comma-free identifier validation;
- timing/status semantics for optimal, feasible, infeasible, timeout and incumbent-bearing records;
- exact cohort isolation across mesh policy, solver binary, solver configuration, hardware fingerprint and measurement class;
- exhaustive exact-search certificates without fabricated scalar objectives or MILP variable counts;
- expansion-cap exhaustion as a distinct `limit_reached` outcome;
- incomplete exact-search counter triples are rejected;
- exhaustive exact-search certificates without fabricated scalar objectives or MILP variable counts;
- expansion-cap exhaustion as a distinct `limit_reached` outcome;
- incomplete exact-search counter triples are rejected;
- different exact mesh instances may vary inside one stable mesh-policy cohort;
- a timeout with an incumbent remains a timeout for completed latency and a valid first-feasible observation;
- completed, first-feasible and memory percentiles expose separate sample denominators;
- duplicate sample suppression;
- nearest-rank percentile semantics;
- p50/p95/p99 sample-count gates;
- separate timeout/infeasible denominators from completed-solve latency;
- fixed-header/field-count parsing and an end-to-end parse of the checked-in 30-row fixture.

## Frozen fixture and exact-head workflow

The checked-in 30-row fixture is tagged `synthetic`: 18 optimal, 4 feasible, 4 timed out and 4 infeasible. It uses the v0.2 34-column schema, including stable mesh-policy, per-sample mesh-instance and solver-configuration fingerprints. The optional exact-search counters and expansion-limit status are `NA`/zero in this generic fixture; the actual engine-runner will exercise those fields. CI validates the entire strict parser → type validator → cohort guard → summary pipeline and records the fixture digest, aggregator-binary digest, source commit, Rust/Cargo versions and runner profile. Its numbers demonstrate the output schema only; they are not used as a performance claim.

The next real measurement campaign should run the same frozen scenario panel over:
- 60/30/15-minute mesh candidates and each selected horizon;
- fixed solver version/build, exact solver tolerances and exact model formulation;
- a pinned machine profile or explicitly stratified hardware cohorts;
- at least 20 completed attempts for p95, and at least 1,000 completed attempts for p99;
- repeated scenario seeds, with timeouts and infeasibility preserved in the denominator.

Report p50/p95/p99 for completed latency, time-to-first-feasible, memory and observed attempt elapsed, alongside timeout/limit-reached/infeasible/error rates, model-size ranges, exact action expansions, feasible complete schedules and Pareto-frontier size. Do not convert timeout time into a completed solve time.

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

## Anytime and audit trace schema v0.3 (additive prototype)

The existing v0.2 CSV parser and fixture remain unchanged. The new `src/trace_v0_3.rs` module introduces a separately named v0.3 record and strict parser for the anytime energy-dispatch contract.

The v0.3 record separates:
- search disposition: `exact_complete`, `feasible_unproven`, `proven_infeasible`, `limit_without_candidate`, `timed_out`, `cancelled`, `error`;
- complete candidate availability and exact schedule digest;
- independent audit disposition, receipt digest, echoed request/schedule digests and auditor binary digest;
- service-floor disposition and policy digest, with its own receipt digest, evaluator executable digest and an explicit link back to the exact schedule-audit receipt;
- search-prefix visits, complete schedules, complete-frontier status, first-candidate timing and first accepted-audit timing.

Validation rejects impossible combinations. For example, a `feasible_unproven` result cannot claim a complete Pareto frontier; an accepted receipt must bind the exact request and candidate digest; and a passed service-floor status must carry its own receipt/evaluator digests and reference the exact preceding schedule-audit receipt. Audit acceptance without a passed service-floor policy is not a decision-ready result.

The v0.3 aggregator reports search outcomes, candidate yield, audit acceptance/rejection/unavailability, service-floor outcomes and decision-ready yield as separate counts. It does not interpret a candidate as optimal merely because the independent schedule auditor accepts it.

This module is a typed parser/validator/summary prototype. The existing profile runner still emits v0.2 and has not yet been migrated to emit v0.3. The migration must preserve old v0.2 parsing and add a new exact-head runner/auditor integration rather than coercing unproven candidates into the old `feasible` or `limit_reached` statuses.
