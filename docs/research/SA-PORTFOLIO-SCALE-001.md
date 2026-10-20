# SA-PORTFOLIO-SCALE-001
## Generalized synthetic portfolio contract and exact frontier-search qualification

**Status:** New v0.2 fixture and Rust search implementation; exact-head qualification pending.  
**Parent:** [Issue #230](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/230).  
**v0.1 regression:** [SA-ECONOMIC-PORTFOLIO-001](./SA-ECONOMIC-PORTFOLIO-001.md).  
**Claim boundary:** Synthetic optimizer engineering only; no real South African cost, impact, job, ROI or investment recommendation.

## 1. Problem being solved

The v0.1 oracle is an immutable, exact eight-project reference case. The number 20 in its exact-subset cap is a safety limit—not evidence of a general 20-project problem format. Scaling needs a more general input contract and an alternative to evaluating every subset.

This slice introduces two separate roles:

- **Exact oracle:** exhaustive enumeration for small generated instances, which provides a truth set for testing.
- **Candidate frontier search:** dependency-aware depth-first search that prunes a branch only when every completion below it is provably infeasible. It retains the full scenario-by-objective vector and returns the Pareto frontier rather than choosing a societal winner.

The candidate remains exact rather than approximate, but has exponential worst-case complexity. A good synthetic fixture is not a scaling benchmark; matched large-instance evaluation remains a separate gate.

## 2. V0.2 input contract

Schema: [sa-economic-portfolio-v0.2.schema.json](./schemas/sa-economic-portfolio-v0.2.schema.json).  
Ten-project fixture: [sa-economic-portfolio-v0.2-10-project.json](./fixtures/sa-economic-portfolio-v0.2-10-project.json).  
Frozen exact expected result: [sa-economic-portfolio-v0.2-10-project.expected.json](./fixtures/sa-economic-portfolio-v0.2-10-project.expected.json).  
Twenty-four-project hostile fixture: [sa-economic-portfolio-v0.2-24-project.json](./fixtures/sa-economic-portfolio-v0.2-24-project.json).  
Adversarial frontier reference: [sa-economic-portfolio-v0.2-24-project.search-expected.json](./fixtures/sa-economic-portfolio-v0.2-24-project.search-expected.json).

V0.2 differs from v0.1 by accepting non-empty unique project IDs and scenario IDs and any declared positive Pn periods, while retaining:
- four typed, separately maximized synthetic objective dimensions;
- explicit project gates, dependencies and stage order;
- a cost cap and crew cap for each declared period;
- exact per-project and per-interaction scenario coverage;
- explicit versioned interaction rules; and
- essential-service floors required to hold in every declared scenario.

All stress scenarios are unweighted. Unknown gates cannot be compensated by high apparent benefit. The contract rejects malformed / missing prerequisites, unknown periods, duplicate IDs, inconsistent scenario keys, invalid resource values, unsupported objective/floor semantics and over-cap exact evaluation. V0.1 keeps its named eight-project corpus guard so the old known-answer case remains immutable.

## 3. Independent exact result for the ten-project fixture

A separate exhaustive evaluator enumerated all 1,024 subsets before the Rust implementation was added to the fixture.

| Measure | Frozen result |
|---|---:|
| Candidate subsets | 1,024 |
| Feasible portfolios | 82 |
| Rejected subsets | 942 |
| Non-PASS gate primary rejections | 512 |
| Missing-prerequisite primary rejections | 192 |
| Period-budget primary rejections | 200 |
| Crew-cap primary rejections | 0 |
| Protected-service-floor primary rejections | 38 |
| Scenario-wise Pareto frontier | 8 |

The five rejection categories are mutually exclusive **primary** reasons under the fixed precedence gate → missing prerequisite → period budget → crew cap → protected floor. They sum to 942; they do not count every simultaneously violated constraint. Project/outcome values are arbitrary synthetic units.

The exact frontier is:
- P00+P01+P03+P05+P08
- P00+P01+P03+P05+P09
- P00+P01+P05+P06+P08
- P00+P01+P05+P06+P09
- P00+P03+P04+P05+P08
- P00+P03+P04+P05+P09
- P02+P04+P05+P06+P08
- P02+P04+P05+P06+P09

These IDs identify synthetic test projects only. They are not a South African development portfolio proposal.

## 4. Candidate search mechanics

The current Rust library's **evaluate_frontier_search(&Input)** sorts projects by declared period rank, stage order and stable ID, then recursively explores exclusion and inclusion decisions. A project may be included only if its prerequisites were already selected. Infeasible include branches are pruned on:
- a mandatory gate other than PASS;
- a missing prerequisite;
- exceeding that period's cost cap; or
- exceeding that period's crew cap.

These conditions are monotone under adding more projects, because costs / crew requirements are non-negative and prerequisite satisfaction cannot be repaired by adding another project when a dependency was already excluded earlier in topological order. The search still evaluates surviving leaves through the common subset evaluator, which applies all declared interaction rules and every-scenario service floors. It does **not** prune by objective value or by individual-project Pareto scores; doing so can lose non-dominated bundles when projects are enablers or interact.

The archive updates the non-dominated portfolio set using the same scenario-wise, componentwise dominance predicate as the exact oracle. An output is considered complete only when **evaluated leaves + all subtrees proven infeasible = 2^n** and exact-head tests show the frontier matches exhaustive enumeration on small inputs.

Counters emitted by the library:
- candidate subsets represented by the complete problem;
- visited search nodes and evaluated leaves;
- feasible portfolios and protected-floor rejection leaves;
- pruned completion counts for gate, prerequisites, period budget and crew capacity; and
- Pareto comparisons.

These counts help explain algorithm behavior, but do not alone prove speed. The 63-project maximum follows from a u64 subset mask; it is **not** a promise that 63-project worst-case instances are computationally practical.

## 5. Qualification suite

The new tests are intended to qualify the following layers:

1. **V0.1 exact regression:** preserve 256 subsets, 22 feasible portfolios, the pinned four-member frontier and the existing negative controls.
2. **V0.2 exact oracle:** compare all 1,024 subsets to the pinned manifest (82 feasible, eight frontier portfolios and the five rejection counters above).
3. **Cross-check:** the branch-and-bound candidate must return structurally equal portfolio DTOs / resource totals / objective vectors as exhaustive enumeration for both frozen instances.
4. **24-project hostile case:** the frozen v0.2 fixture starts with the v0.1 case, adds 16 UNKNOWN-gated decoys and requires exact frontier equality with the eight-project reference. The full universe contains 16,777,216 subsets; the candidate must account for the universe by visited leaves plus proven-pruned subtrees instead of enumerating each subset. It is an intentionally prune-friendly negative control, not a representative random case or a scaling win by itself.
5. **Deterministic generated 12 / 16 / 18-project cases:** unit tests produce instances with three stress scenarios, three periods, nonnegative period resources, mixed gates, service floors and interaction adjustments. For each instance, compare exhaustive feasible/frontier results to the candidate search and reconcile all subsets. Their exact-head status and measured duration must be recorded; no runtime claim is implied by their presence in the source.
5. **Safety mutations:** strict protected floor, a P1 budget reduction, crew-only caps, missing / invalid prerequisite, duplicate IDs, scenario permutation and input-row permutation must preserve explicit failure or canonical result semantics.
6. **Baseline ablations:** keep the tested minimum-cost, declared weighted-sum and naive-project-Pareto baselines as comparison controls, not recommendations.

## 6. How this compares to industry / research practice

- **World Bank Infrastructure Prioritization Framework:** keep financial-economic and social/environmental criteria visible; constraints, weighting and sensitivity need transparent review: https://ppp.worldbank.org/library/infrastructure-prioritization-framework-ipf
- **COCO benchmarking platform:** reuse benchmark discipline for fixed instance suites, non-dominated solution quality and runtime-to-target. COCO's numeric black-box suites do not directly represent this discrete, dependency/resource-constrained civic portfolio problem: https://coco-platform.org/data-archive/index.html and https://arxiv.org/abs/1605.01746
- **Google OR-Tools CP-SAT:** use an independent operations-research formulation as a comparator on matched feasibility constraints and objective directions; report the solver version, parameters, wall-time limit, status and optimality bound/gap rather than comparing to a differently posed problem: https://developers.google.com/optimization/cp
- **NIST AI RMF Playbook:** use Govern / Map / Measure / Manage as a governance structure for the decision-support layer, not as a compliance claim from passing software tests: https://www.nist.gov/itl/ai-risk-management-framework/nist-ai-rmf-playbook
- **Independent operations-research baseline:** for generated 12–18 project cases, compare against a separately formulated MILP / CP-SAT or suitable solver on identical constraints, scenario set, objective directions and wall-time budget. Verify exactness against enumeration on smaller instances, and report solver version, settings and optimality gap.

## 7. Required run receipt and publication gates

For each pinned workload, record commit SHA, toolchain/compiler, runner/CPU, fixture/generator revision, input digest, project / scenario / period counts, total subsets, visited nodes, pruned completion counts by reason, visited leaves, feasible portfolio count, frontier IDs and digest, p50/p95/max latency, peak RSS, output size, and any timeout/error.

Pass criteria:
- 100% exact feasible-set and frontier agreement on every frozen small oracle instance;
- 0 returned portfolio with an unknown/blocked gate, missing prerequisite, period cap violation, crew cap violation or protected-floor failure;
- 100% accounting reconciliation between candidate subsets and evaluated/pruned subsets;
- deterministic canonical result under project, scenario, objective, budget and interaction row permutations;
- no claim of completed search if any branch remains unevaluated or unaccounted;
- matched OR-solver comparison before claiming scale or superiority;
- real data only after source rights, owner, baseline, independent verification and decision-rights gates.

## 8. Initial hosted smoke benchmark and validation

Exact-head hosted run for this benchmark-suite PR: **commit `185c26e56cea43cc4e677666c03bf348d55ebbe2`**, dedicated workflow `SA economic portfolio oracle` = **success**. Rust test suite: **19 passed, 0 failed**; `cargo fmt --check`, Clippy with `-D warnings`, both exact receipts, both search receipts, and the release benchmark commands completed successfully. The separate repository `Check` workflow was still in progress when this record was written, so no claim is made about that check's final status here.

The benchmark artifact records these 21-repeat release-build measurements on the current `ubuntu-latest` runner with Rust toolchain 1.96.0:

| Experiment | Metric | Exact subset evaluator | Dependency-aware frontier search |
|---|---|---:|---:|
| V0.2 ten-project fixture | p50 algorithm time | 434 µs | 139 µs |
| V0.2 ten-project fixture | p95 algorithm time | 468 µs | 150 µs |
| V0.2 ten-project fixture | minimum / maximum | 406 / 483 µs | 127 / 158 µs |

**Measurement-protocol caveat:** these numbers are from the initial pre-warm-up, fixed-order comparison. They are preserved as a historical smoke baseline; the benchmark-methodology follow-up adds two warm-ups per algorithm, alternates paired order, and records host metadata. Future readings produced under the improved protocol must be reported separately, not silently appended to or directly mixed with these initial samples. See [SA-PORTFOLIO-BENCHMARK-PROTOCOL-001](./SA-PORTFOLIO-BENCHMARK-PROTOCOL-001.md).\n\nThis is a single-host **smoke benchmark**, not an apples-to-apples performance proof: the exhaustive evaluator returns the complete set of 82 feasible portfolios as well as the 8-member frontier; the candidate search returns the frontier and proof-of-accounting counters. The 3.1x median ratio describes only these two recorded implementations and this small synthetic input. It is not evidence of beating a commercial system, an independent MILP / CP-SAT formulation, or SOTA.

The V0.2 ten-project search receipt reports 1,024 subsets, 305 visited nodes, 120 evaluated leaves, 82 feasible portfolios and the same 8-member frontier as exact enumeration. The 904 pruned completions partition into 448 gate, 192 missing-prerequisite and 264 budget prunes; 120 leaves + 904 prunes = 1,024.

The adversarial 24-project case has a full universe of 16,777,216 subsets. Its 21-repeat search receipt reports:
- p50 / p95 / max algorithm time: **67 / 85 / 128 µs**;
- 745 search nodes and 40 evaluated leaves;
- 9,961,432 gate-pruned, 3,145,728 prerequisite-pruned, 3,670,016 budget-pruned and 0 crew-pruned completions;
- 22 feasible leaf portfolios, 4 frontier portfolios and exact equality with the supplied frozen frontier reference.

The partition reconciles: 40 evaluated leaves + 16,777,176 pruned completions = 16,777,216. This adversarial fixture deliberately adds sixteen `UNKNOWN`-gated decoys that are immediately provable infeasible; it demonstrates accounting and negative-control behavior, not representative large-instance scaling.



## 9. Warmed, order-alternated benchmark (methodology PR #247)

The measurement protocol was then hardened and run on exact source head `b7148214db462ed52d5626f4da34e00812c0e4b7`, workflow [SA economic portfolio oracle run #39489050764](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/39489050764). The dedicated workflow passed formatting, Clippy with warnings denied, the full Rust test suite, exact/search receipts and both benchmark commands. The separate repository `Check` run was still in progress at the time of this record.

Environment recorded in both benchmark receipts: GitHub Actions Linux/X64 runner; AMD EPYC 7763 64-Core Processor; 4 logical CPUs visible to the process; 16,766,414,848 bytes reported total memory; release build. Process high-water RSS was 3,522,560 bytes for the ten-project comparison process and 2,945,024 bytes for the 24-project search process. These are per-process high-water readings from this run, not peak RSS distributions across repeated isolated processes or evidence of production capacity.

The new protocol ran **2 unmeasured warm-ups per algorithm** and alternated measurement order over **21 paired repetitions**:

| Experiment | Measure | Exact subset evaluation | Frontier search |
|---|---|---:|---:|
| 10 projects | p50 algorithm latency | 449 µs | 151 µs |
| 10 projects | p95 algorithm latency | 465 µs | 156 µs |
| 10 projects | min / max | 428 / 477 µs | 146 / 157 µs |
| 24-project adversarial fixture | p50 / p95 / max search latency | — | 81 / 113 / 128 µs |

The warmed ten-project median ratio is about **2.98x** for this exact pair of code paths on this runner. The exact evaluator also returns all 82 feasible portfolios, whereas the search returns the 8-member frontier and proof-of-accounting counters; therefore this remains a benchmark of two defined outputs, not a claim that equivalent full-output tasks differ by 2.98x or that the method beats SOTA.

On the 10-project input the frontier search still visited 305 nodes, evaluated 120 leaves and pruned 904 completions: 448 gate, 192 prerequisite, 264 budget; 120 + 904 = 1,024. All 21 exact/search pairs matched the pinned outcome frontier and exact known-answer manifest.

On the adversarial 24-project input the candidate still represented all 16,777,216 subsets with 745 visited nodes and 40 evaluated leaves. The 16,777,176 pruned completions were 9,961,432 gate, 3,145,728 prerequisite and 3,670,016 budget prunes. Its four-member frontier matched the supplied reference every time. This fixture remains deliberately easy to prune; the timing is not a representative hard-instance claim.

The initial un-warmed 434 µs / 139 µs ten-project numbers in section 8 are retained as historical baseline only. They must not be blended with the hardened-protocol samples above. The next performance threshold should wait for the independent, matched solver model tracked by [issue #248](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/248) and a wider structured/generated case suite.

**Claim ceiling:** this is a synthetic contract and exact-search engineering exercise. It does not forecast GDP, jobs, real cost recovery, causal service improvement, or the optimal allocation of South African public resources.
