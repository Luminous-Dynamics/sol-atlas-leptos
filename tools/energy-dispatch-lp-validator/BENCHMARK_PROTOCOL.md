# Energy Dispatch Assurance and Industry Benchmark Protocol v0.1

Status: benchmark design for issue #343. All target numbers below are acceptance goals for explicitly frozen finite panels, not measured performance or a claim of superiority over a commercial or open-source planner.

## 1. Benchmarking principle

The object being qualified is not just an optimizer. It is a chain:

1. source request and unit/time-base interpretation;
2. compiled integer model;
3. optimizer output and optimality evidence;
4. independently checked dispatch trajectory;
5. provenance receipt that binds the exact request, model, binaries, result, certificate and checker.

Report each stage separately. A valid MIP certificate establishes a property of the encoded model; it does not prove that the source-to-model translation is correct, that the model captures all site physics, or that a dispatch is safe to execute.

## 2. Assurance tiers and target criteria

### Tier A — finite semantic oracle

- H=2 fixtures: enumerate every bounded assignment and require exact agreement between LP feasibility and the independent source-request schedule evaluator.
- H=2 objective: require exact equality between LP objective and independently recomputed essential-unserved quanta for every assignment.
- H=3 fixture: retain the fixed-seed 50,000-assignment sample as sampled coverage; never describe it as exhaustive.
- Future H=3/H=4 exhaustive enumeration should be attempted only when the complete assignment count is tractable and recorded before execution.
- Target: zero false-feasible and zero false-infeasible decisions on each published exhaustive panel.

### Tier B — mutation resistance

Freeze a mutation manifest with deterministic seed, mutation ID, target family, expected rejection and output digest. At minimum cover:
- SOC carry-forward and interval-index errors;
- outage bit shifts and nonzero grid import in outage;
- simultaneous charge/discharge;
- floor efficiency changed to round/ceil;
- PV partition, load partition and bus-balance corruption;
- missing, duplicated, reordered or truncated intervals;
- interval-duration/time-base and power/energy unit changes;
- terminal reserve and import/export bound mutations;
- receipt/request/LP/certificate/schedule cross-association tampering.

Target: 100% detection of the finite published mutation panel, with the intended predicate recorded. A crash, parse failure or timeout is not credited as semantic mutation detection unless malformed input is the mutation itself.

### Tier C — proof and evidence integrity

For every evidence edge, run a negative control that substitutes or corrupts the source request, translator receipt, LP bytes, semantic-model digest, solver binary/version, solver output, certificate, VIPR checker binary, checker log, schedule, evaluator binary or result digest.

Target: 100% rejection of the frozen association-tamper panel. Record whether a digest was merely present or actively checked; provenance fields must not turn unverified claims into verified facts.

### Tier D — operational scenario realism

On held-out time-series cases, report:
- essential energy not served (kWh);
- essential served fraction;
- longest continuous essential-load interruption;
- outage coverage duration;
- flexible-load energy shed;
- minimum and terminal state of charge;
- post-outage recovery time;
- grid import/export, PV curtailment, operating cost and emissions;
- time to first feasible solution, time to proven optimum, objective gap, peak memory;
- translation validation time, proof generation time, VIPR check time and independent schedule check time.

Do not infer real-site reliability from the synthetic H=2/H=3 fixtures. Use separately sourced and versioned weather, load, tariff, outage and equipment data for operational conclusions.

## 3. Comparison matrix

| Reference | Appropriate comparison | Inappropriate claim |
|---|---|---|
| NREL REopt API v3 / REopt.jl | DER sizing, PV/storage/grid mix, resilience and techno-economic outcomes where horizon, tariff, service constraints and degradation assumptions can be aligned | Treating a synthetic two-interval integer dispatch fixture as a complete REopt facility-planning comparison |
| PGLib-OPF | Network-level AC optimal power flow and related OPF formulations when Sol-Atlas models the same network equations and case | Using PGLib-OPF as a direct benchmark for behind-the-meter battery dispatch with no network OPF |
| SCIP 10.1.0 + VIPR | Exact integer-model solving and independently checked optimality certificates for the same encoded LP/MIP | Claiming a valid certificate proves the source request was translated correctly or the physical model is complete |
| HiGHS or another pinned MIP solver | Solver speed/gap/solution comparison on identical exported models and stopping criteria | Comparing different model formulations, tolerances or objective weights as if the solver alone caused the difference |

External references:
- REopt API: https://developer.nlr.gov/docs/energy-optimization/reopt/
- PGLib-OPF: https://github.com/power-grid-lib/pglib-opf
- SCIP: https://www.scipopt.org/
- VIPR: https://github.com/scipopt/vipr

These are complementary baselines, not interchangeable benchmarks. Publish a capability matrix when exact model equivalence is impossible.

## 4. Fair experimental protocol

For each compared workload, freeze and publish:
- dataset/source revision and SHA-256;
- horizon, interval duration, chronology and time zone;
- power and energy units, sign convention and rounding rules;
- objective coefficients and objective sense;
- every constraint, reserve policy and service-priority definition;
- tariff, degradation, export limits, outage trace and weather/load trace;
- solver/version/binary digest and all relevant solver parameters;
- CPU model, core count, memory, OS, compiler/toolchain and lockfile digest;
- thread count, warm/cold state, time limit, gap limit and feasibility tolerances.

Use the same input and model for solver comparisons. Run at least 30 independent repetitions for noisy performance cases, publish raw per-run results and report median plus p50/p95/p99 where sample size supports it. Separate:
- source parsing/normalization;
- model construction;
- solver search;
- proof logging;
- independent proof checking;
- LP translation validation;
- schedule feasibility validation;
- full end-to-end latency.

Do not add checker cost to one solver and omit it from another. Report both solver-only and assurance-inclusive totals.

## 5. Performance objectives

Performance work begins only after the correctness tiers are frozen. Optimize against the measured bottleneck:
- reduce model size without changing the feasible set;
- strengthen valid bounds/cuts only when independently validated;
- warm-start repeated rolling-horizon solves while retaining exact request/receipt identity;
- reuse parsed immutable inputs and compiled policy/model structures;
- parallelize independent scenario evaluation, not order-sensitive state transitions;
- cache evidence only by complete cryptographic subject identity.

Initial engineering objective: for larger workloads, aim for independent validation and receipt overhead below 10% of end-to-end solve time at p95. This is a measured optimization goal, not a gate that can weaken correctness. Report the actual overhead even when it exceeds the target.

## 6. Publication format

Every benchmark publication should include:
- a machine-readable manifest with schema/version, repository commit, source/data digests and claim ceiling;
- raw per-run JSONL or CSV;
- summary distributions and exact test/mutation counts;
- binary, solver, checker and compiler provenance;
- proof/certificate and independent-check receipts where applicable;
- known limitations and all skipped/indeterminate cases.

A missing run, queued runner or timed-out check is not a pass and must remain visible as not qualified.

## 7. Claim ceiling

A successful Tier A-C result establishes properties only for the declared finite fixtures, mutation panel and exact evidence graph. Tier D supports operational conclusions only for the frozen datasets and modeled assumptions. None establishes physical completeness, real-world safety certification, grid-interconnection approval, field reliability, or authorization to control equipment.
