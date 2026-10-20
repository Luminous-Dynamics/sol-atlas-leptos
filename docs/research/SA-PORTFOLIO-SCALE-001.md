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
4. **24-project hostile case:** start with the v0.1 case under generalized v0.2 semantics, add 16 UNKNOWN-gated decoys, and require exact frontier equality with the eight-project reference. The full universe contains 16,777,216 subsets; the candidate should account for that universe by visited leaves plus proven-pruned subtrees instead of enumerating each subset. Runtime and node counts remain measurements from the pinned run, not assumed outcomes.
5. **Safety mutations:** strict protected floor, a P1 budget reduction, crew-only caps, missing / invalid prerequisite, duplicate IDs, scenario permutation and input-row permutation must preserve explicit failure or canonical result semantics.
6. **Baseline ablations:** keep the tested minimum-cost, declared weighted-sum and naive-project-Pareto baselines as comparison controls, not recommendations.

## 6. How this compares to industry / research practice

- **World Bank Infrastructure Prioritization Framework:** keep financial-economic and social/environmental criteria visible; constraints, weighting and sensitivity need transparent review: https://ppp.worldbank.org/library/infrastructure-prioritization-framework-ipf
- **COCO benchmarking platform:** reuse benchmark discipline for fixed instance suites, non-dominated solution quality and runtime-to-target. COCO's numeric black-box suites do not directly represent this discrete, dependency/resource-constrained civic portfolio problem: https://coco-platform.org/data-archive/index.html and https://arxiv.org/abs/1605.01746
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

## 8. Current validation state

The exact-head audit and baseline-ablation branches have successful hosted Rust workflows. This generalized v0.2/search branch is a new increment; its own formatting, Clippy, test and receipt results must be checked by exact head. The independent fixture-preparation enumeration confirms the expected 1,024-subset manifest, but it is not a Rust compile/test and does not verify the new branch-and-bound implementation.

**Claim ceiling:** this is a synthetic contract and exact-search engineering exercise. It does not forecast GDP, jobs, real cost recovery, causal service improvement, or the optimal allocation of South African public resources.
