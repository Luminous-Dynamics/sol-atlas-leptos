# SA-PORTFOLIO-COUNTEREXAMPLE-DISCOVERY-001

**Status:** First bounded Rust sweep implemented; exact-head qualification runs in CI. Synthetic-only v0.1 experiment.

## 1. Research question

Can a portfolio selected against a declared scenario set be falsified by a small, explicit perturbation to a protected-service constraint? The current sweep explores only a finite, caller-supplied list of integer service baselines. It is an executable seed for counterexample-guided hardening, not a proof that the real-world scenario universe is complete.

Related work frames robust multi-objective optimization around deep uncertainty, alternative scenarios, decision-maker exploration and adaptive reassessment:

- Shavazipour, Kwakkel & Miettinen (2021), *Multi-scenario multi-objective robust optimization under deep uncertainty: A posteriori approach*, Environmental Modelling & Software 144, 105134. https://doi.org/10.1016/j.envsoft.2021.105134
- “Let decision-makers direct the search for robust solutions: An interactive framework for multiobjective robust optimization under deep uncertainty,” Environmental Modelling & Software 183 (January 2025), 106233. https://doi.org/10.1016/j.envsoft.2024.106233
- Shavazipour & Stewart (2026), adaptive multi-stage, multi-scenario multi-objective robust decision-making, Omega 138, 103405. https://doi.org/10.1016/j.omega.2025.103405

For external benchmark practice, the COCO platform emphasizes frozen test suites and systematic, reproducible performance assessment: https://coco-platform.org/index.html and https://coco-platform.org/data-archive/. Its continuous black-box suites are **methodological inspiration**, not an appropriate drop-in workload for this discrete portfolio oracle. Any operations-research comparison must use equivalent integer constraints and objectives. OR-Tools explicitly distinguishes OPTIMAL, FEASIBLE, INFEASIBLE, MODEL_INVALID and UNKNOWN; our stop states follow the same important principle that a budget timeout must not be relabeled as proof: https://developers.google.com/optimization/cp/cp_solver.

These references motivate the test design. They do not establish that this implementation is novel or better than state of the art.

## 2. Paradigm shift

Shift from “solve once and assume the scenario list is adequate” to “search a declared uncertainty envelope, preserve discovered counterexamples, and report exactly what has and has not been covered.”

Counterexample guidance:

1. Start with a frozen input, a frozen scenario template and an explicit finite mutation domain.
2. Apply one deterministic mutation at a time; each mutated scenario has a stable ID.
3. Recompute exact feasibility/frontier with the existing evaluator. In this first version, a counterexample is any previously feasible portfolio removed by the new scenario's protected service floor.
4. Stop on the first counterexample, or exhaust every configured domain value.
5. Hash all inputs and the expected result; keep the attempted values and remaining search domain in the receipt.

The input, patch template, ordered values and budget fully specify the experiment. Domain ordering is intentional: it defines which mutation is attempted first. The current search varies only an integer service baseline; it does **not** mutate cost, crew caps, crew requirements, interaction topology or project prerequisites.

## 3. Executable contract

Rust API: `evaluate_counterexample_sweep(input, template_patch, config)`.

CLI:

```sh
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --counterexample-sweep \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  docs/research/fixtures/sa-portfolio-counterexample-sweep-v0.1-compound-template.json \
  docs/research/fixtures/sa-portfolio-counterexample-sweep-v0.1-compound-config.json \
  docs/research/fixtures/sa-portfolio-counterexample-sweep-v0.1-compound.expected.json
```

Status semantics are part of the contract:

- `COUNTEREXAMPLE_FOUND`: at least one earlier-feasible portfolio is excluded by the newly added scenario. The report lists the first counterexample and all remaining values; this is **not** domain exhaustion.
- `DOMAIN_EXHAUSTED_NO_COUNTEREXAMPLE`: every value in the declared finite domain was evaluated and none removed any previously feasible portfolio. This is a finite-domain result only, not a real-world robustness claim.
- `BUDGET_EXHAUSTED`: the configured evaluation budget ended before the domain was exhausted and no counterexample was found. Remaining values are listed. Never interpret this state as a pass.
- Invalid input, unsupported schemas, repeated domain values, negative baselines, missing patch rows or evaluator errors return an error rather than a passing receipt.

Every passing CLI invocation compares the complete report against a frozen manifest and emits SHA-256 digests of the input, template, config and expected output.

## 4. Frozen challenge results

Independent expected results were derived using a separate subset enumerator and then pinned. The dedicated Rust suite must reproduce these exact reports.

| Case | Baseline domain (ordered) | Evaluations | Result | Before → after feasible portfolios | Frontier before → after |
|---|---|---:|---|---:|---:|
| Compound service-stress challenge | 10, 9, 8, 7, 6, 5, 4, 3, 2 | 2 | `COUNTEREXAMPLE_FOUND` at 9 | 82 → 80 | 8 → 8 at counterexample (ID intersection 7, union 9) |
| No-effect duplicate-scenario control | 3 | 1 | `DOMAIN_EXHAUSTED_NO_COUNTEREXAMPLE` | 82 → 82 | 8 → 8 (ID intersection/union 8/8) |
| Unit-only budget control | 3, 4, 5 with budget 1 | 1 | `BUDGET_EXHAUSTED` | No discovery claim | Not a robustness result |

On the compound fixture, baseline 10 leaves the 82-member feasible set unchanged, though the frontier changes because the scenario also contributes objective outcomes. Baseline 9 excludes two formerly feasible portfolios through the protected service floor; seven configured values remain untested. This demonstrates a detectable boundary in a *synthetic coefficient fixture*, not a calibrated real-world threshold. The no-effect control repeats the `normal_reference` coefficients exactly, and the sweep is required to preserve the feasible set and all eight frontier IDs.

## 5. Benchmarks to beat

These are proposed acceptance gates, not already established industry records.

| Dimension | First gate | Stronger comparative gate |
|---|---|---|
| Correctness | Exact manifest equality for every pinned sweep and scenario-addition case | Independent CP-SAT/MILP agreement under equivalent semantics, then generated held-out instances |
| Safety | Zero unreported protected-floor failures; no infeasible portfolio leaks into the feasible set | Mutation/property tests over service floors, gate statuses, prerequisites, costs and crew caps |
| Counterexample recall | 100% discovery of pinned counterexamples in exhaustive tiny domains | Recall vs seeded random and one-factor sensitivity at identical evaluation budgets, with uncertainty intervals over frozen seeds |
| Negative controls | 100% invariance on exact duplicate-scenario cases | Mutation families with provably no effect on feasibility or objective-vector ordering |
| Stop semantics | Zero false `DOMAIN_EXHAUSTED` / robustness claims on budget-limited runs | Independent tests for empty, malformed, duplicate and partially evaluated domains |
| Replayability | Identical inputs/config/order produce identical reports and hashes except no runtime fields are present | Reproducible corpus registry, versioned generators, immutable challenge digests |
| Performance | Distributional latency and evaluated-candidate counts on the fixed small corpus | Matched solver comparisons on hard cases, p50/p95/max latency, peak RSS, time-to-first-counterexample and discovery-per-evaluation |

## 6. Fair industry comparison

Compare four distinct baselines on the **same declared perturbation values and evaluation budget**:

1. Static scenario-set evaluation (no extra search).
2. One-factor baseline sensitivity (sweep the same scalar values without stopping early).
3. Seeded random selection from the same finite domain.
4. Counterexample-directed ordered search, with the sweep order and stop rule frozen before runs.

Add an exhaustive finite-domain oracle for tiny grids and an independent CP-SAT/MILP feasibility model. Preserve the exact solver release, parameters, status, objective bounds/gaps where applicable, input digests, timing distributions and stopping conditions. Do not compare one returned portfolio to a full frontier, do not treat a feasible incumbent as proven optimal, and do not report a universal speedup from a prune-friendly synthetic case.

## 7. Next steps

- Generalize from service-baseline values to independently typed perturbation families: service-outcome shifts, project cost/crew requirements, period capacity shocks and interaction-model mutations.
- Add pairwise and bounded correlated mutation designs with explicit domains and exact known answers.
- Implement a tiny exhaustive mutation grid to prove the counterexample result or no-counterexample status **within that finite grid**.
- Complete the independent CP-SAT/MILP baseline tracked in [issue #248](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/248).
- Keep adaptive recourse as a separate model: only decisions genuinely delayed until after observation can adapt. Scenario-by-scenario hindsight reoptimization is not a valid recourse model.

Implementation tracks [issue #258](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/258). The scenario omission/addition foundations are in [PR #255](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/255).