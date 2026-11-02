# Sol Atlas exact-dispatch profile runner v0.1

A source-bound execution profiler for the actual `sol-atlas-core::energy_dispatch::assess_energy_dispatch` implementation. This runner does not call a substitute solver: it times the existing bounded exhaustive integer enumerator and records its native work counters.

## What the engine is

The current v0.1 kernel recursively enumerates finite integer dispatch actions over chronological intervals, enforces its transition/bus-balance/reserve rules, and retains a non-dominated lexicographic Pareto frontier. It is **not** an LP, MILP, AC optimal-power-flow solver or continuous-physics simulator.

The engine exposes:
- evaluated action expansions;
- feasible complete schedules;
- retained Pareto frontier size;
- an exact/complete status for the declared finite quantized model.

The profiler preserves those fields rather than inventing continuous/binary variable counts, scalar objectives, dual bounds, a scalar optimality gap, a random seed or an unobserved first-feasible timestamp. Those trace fields stay `NA` where they are not defined or not instrumented.

## Trace and provenance contract

The 34-column `energy-solver-compute-trace-v0.2` row binds:
- exact request bytes SHA-256;
- scenario-panel SHA-256;
- dispatch source, pathway source, core library, core manifest, runner manifest, workspace manifest and lockfile fingerprint;
- stable mesh-policy digest and exact per-request mesh-instance digest;
- exact runner executable SHA-256;
- solver-configuration digest, including expansion/frontier limits and deterministic action-order semantics;
- host profile digest (OS, architecture, CPU model, logical CPU count and physical memory);
- interval count, horizon, measured kernel wall-clock nanoseconds, process peak resident memory;
- exact search counters and terminal disposition.

The host fingerprint is descriptive, not remotely attested. The binary digest binds the profiler executable. The model digest binds the exact core source and workspace dependency/manifest material used by the profiler.

`elapsed_ns` measures only the call to `assess_energy_dispatch`; request parsing, hashing, schedule serialization and independent auditing are outside that interval. It is a kernel runtime, not end-to-end decision latency.

## Independent schedule acceptance

For each successful exact assessment, the runner selects the first alternative in the engine's deterministic lexicographic Pareto ordering, maps it to the public domain-level schedule DTO, hashes the exact serialized bytes and publishes that schedule.

The workflow then invokes the standalone `energy-dispatch-schedule-auditor` executable as a separate process. Its receipt must:
- bind the exact same request hash;
- bind the exact selected schedule hash;
- report `accepted`;
- contain only passing findings;
- include auditor executable and receipt hashes.

A successful search without an accepted independent receipt is not counted as an audited decision. The workflow runs 30 exact-search samples and requires 30 accepted receipts.

## Current measured panel

The first frozen request is `schemas/examples/energy-dispatch-pv-outage.synthetic.json`: a two-interval PV-charge / grid-outage micro-case. It is explicitly synthetic. Runtime and memory are actually measured on the hosted runner; the data/model case is synthetic. The scenario class must not be mistaken for fabricated timings.

The repeated campaign:
- builds the exact profiler, standalone schedule auditor and trace aggregator once;
- executes the same frozen request 30 times in one source/model/mesh/solver-binary/configuration/hardware cohort;
- emits 30 unique sample IDs and a 34-column trace;
- serializes the first lexicographic Pareto alternative as the domain-level schedule DTO;
- audits all 30 selected schedules in a separate process and requires 30 accepted receipts bound to their exact request/schedule hashes;
- computes nearest-rank p50 and p95 for completed search latency;
- withholds p99 because the required 1,000 completed samples are not available;
- reports the sample denominators and full runner/auditor/aggregator provenance.

The trace has `measurement_class=synthetic` because the frozen request is synthetic; the `elapsed_ns` values are still measured with `Instant` around the real kernel call. The 30 repetitions estimate within-case timing variability on the exact hosted runner, not variation across scenarios or machines.

This is a **within-case repeatability measurement**, not a broad dispatch performance result. It does not yet establish how runtime scales with horizon, uncertainty, asset count, constraint density or network size. The first 30-row cohort will be uploaded as a CI artifact; use its exact source commit, panel digest, executable hashes and host fingerprint when interpreting it.

## Benchmark progression

### Tier 1 — exact finite-model correctness and cost

Expand the frozen panel through H=2, H=4 and H=8+ interval variants; PV and load ramps; outage timing shifts; reserve constraints; derated battery limits; infeasible cases; and expansion/frontier cap controls. For each run report:
- exact search status and cap exhaustion rate;
- action expansions, complete feasible schedules, Pareto frontier size;
- p50/p95/p99 kernel time (with separate sample eligibility counts);
- peak RSS, memory per expansion and expansions/second;
- independently audited schedule acceptance rate;
- essential/flexible shortfall, grid import, curtailment, terminal reserve and the full Pareto objective vector.

Do not extrapolate multi-day performance from a two-interval sample. Grow the horizon in a controlled ladder and preserve the complete scenario manifests.

### Tier 2 — decision quality under changing information

Implement true receding-horizon operation: solve from the current forecast, commit only the first interval, ingest the next observation and forecast, then re-optimize. Compare against:
- a clairvoyant full-horizon optimum as an offline lower bound, never as an online controller;
- deterministic receding-horizon dispatch;
- robust uncertainty-set dispatch;
- scenario/stochastic dispatch;
- an explicitly specified MPC baseline.

Use frozen unbiased, biased, heavy-tailed, missing-telemetry, outage-start-shift, PV/load-correlation-shift and equipment-derating cases. Report regret, essential unserved energy, outage coverage, longest interruption, recovery time, flexible shed, terminal reserve, costs, emissions and tail risk. Preserve infeasible and timed-out cases in the denominators.

### Tier 3 — physical realism and matched industry comparisons

| Reference | What it is a fair comparator for | What must not be claimed |
|---|---|---|
| [NREL REopt API v3 / REopt.jl](https://developer.nlr.gov/docs/energy-optimization/reopt/) | Distributed-energy technology sizing, hourly operations, economics and resilience where the same loads, tariffs, technologies and objectives can be matched | Do not claim parity from a tiny fixed integer microgrid fixture; REopt solves a different and broader planning problem |
| [PyPSA unit commitment](https://github.com/PyPSA/PyPSA) | Temporal optimization methods once generator/link commitment, start-up/shut-down and ramping constraints are represented | Do not call the current discrete battery kernel a MILP or compare its two-interval micro-case with system-scale unit commitment |
| [IEEE PES PGLib-UC](https://github.com/power-grid-lib/pglib-uc) | Reproducible temporal MILP/commitment stress cases and optimizer methodology after translating the matching mathematical model | Thermal unit-commitment cases are not battery-equivalent scenarios |
| [IEEE PES PGLib-OPF](https://github.com/power-grid-lib/pglib-opf) | AC optimal power-flow methods after network equations, voltage limits and relevant grid physics are implemented | Not a valid current benchmark for a behind-the-meter energy-balance enumerator without network flows |
| [BOPTEST](https://ibpsa.github.io/project1-boptest/) | Closed-loop building/control-policy comparison with standardized test cases and KPIs | BOPTEST is a control benchmarking environment, not a direct standalone electricity dispatch optimizer |

REopt documents PV, wind, conventional generation, storage, dispatchable loads, cost and resilience outputs. PyPSA's unit-commitment formulation introduces binary commitment variables and start-up/shut-down/ramping constraints. PGLib is curated by the IEEE PES Task Force on Benchmarks for Validation of Emerging Power System Algorithms. BOPTEST supplies a standard test-case/API/KPI environment for evaluating control algorithms. These are distinct benchmark families and should remain distinct in reports.

## Targets and claim gates

Targets for the expanded matched benchmark, not results already achieved:

- **Correctness:** zero accepted schedules that violate any frozen source-request predicate; positive controls accepted; all negative mutations rejected.
- **Search completeness:** exact finite-panel optimum/frontier on tiny cases; every budget/frontier cap hit is explicit and never labelled optimal.
- **Decision utility:** minimize essential unserved energy and tail outage risk before cost/emissions trade-offs; publish the full Pareto set.
- **Operational cost:** p50/p95 latency and peak memory stratified by exact case/model/host/solver cohort; p99 only with at least 1,000 completed samples.
- **Robustness:** hold-out uncertainty cases and forecast shifts; report regret and infeasibility rather than hiding failures.
- **Comparative claim:** beat a named baseline only on a preregistered matched case set, same physical model, objective priorities, forecasts, hardware and solver budget. Report confidence intervals and all failed runs.

The near-term competitive differentiator is **audited decision yield per unit compute**: the fraction of requests that produce a source-bound, independently accepted schedule before a stated compute deadline, alongside service resilience, regret and resource cost. It is not raw search speed alone.

## Run locally

```sh
cargo fmt --package energy-dispatch-profile-runner --check
cargo check --locked --manifest-path tools/energy-dispatch-profile-runner/Cargo.toml --bin energy-dispatch-profile
cargo test --locked --manifest-path tools/energy-dispatch-profile-runner/Cargo.toml --bin energy-dispatch-profile
cargo clippy --no-deps --locked --manifest-path tools/energy-dispatch-profile-runner/Cargo.toml --bin energy-dispatch-profile -- -D warnings
```

The exact 30-sample campaign and independent audit commands are kept in `.github/workflows/energy-dispatch-profiled-runner.yml`.

## Claim ceiling

This runner measures execution of the actual current bounded exact enumerator on a declared request and uses a separate executable to audit the selected trajectory. The current first panel is a single two-interval synthetic case; its 30 repeats are not an external baseline comparison or evidence of SOTA superiority. It does not establish grid safety, real-world optimality, network feasibility, battery life, forecast accuracy, field reliability, regulatory compliance, or SOTA superiority. Those require additional physics, data, operating constraints and matched baselines.
