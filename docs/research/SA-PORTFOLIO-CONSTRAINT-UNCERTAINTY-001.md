# SA-PORTFOLIO-CONSTRAINT-UNCERTAINTY-001

**Status:** First Rust cost/crew cap sweep implemented; exact-head qualification is pending. Evidence class: synthetic, bounded sensitivity analysis.

## 1. Research question and paradigm shift

Do not treat cost and crew caps as immutable model truth. Treat a resource-capacity boundary as an explicit uncertainty dimension, then compute the exact **capacity-to-feasibility frontier**: which formerly feasible portfolios disappear as capacity tightens, and how does the Pareto frontier change?

This is distinct from scenario-outcome uncertainty. Adding a scenario changes objective outcomes and may add a per-scenario protected-service constraint; changing a resource cap changes the shared feasible region. The model should not conflate constraint feasibility with objective regret or assign probabilities to arbitrary cap values.

This is the next constraint-side component of [counterexample-guided portfolio hardening](SA-PORTFOLIO-COUNTEREXAMPLE-DISCOVERY-001.md), linked to [issue #262](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/262).

## 2. Executable contract

Rust API: `evaluate_capacity_sweep(input, config)`.

CLI:

```sh
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --capacity-sweep \
  docs/research/fixtures/sa-economic-portfolio-v0.2-resource-capacity.json \
  docs/research/fixtures/sa-portfolio-capacity-sweep-v0.1-p1-crew-config.json \
  docs/research/fixtures/sa-portfolio-capacity-sweep-v0.1-p1-crew.expected.json
```

The config is versioned and selects one period and resource dimension (`cost` or `crew`). It contains a strictly descending, finite, non-negative sequence of capacity values; the first value must equal the actual cap in the input, so every domain begins with a no-shock negative control. Each value is applied independently to a fresh copy of the unchanged input and evaluated using the same exact evaluator.

The full report includes the baseline capacity, ordered domain, every attempted capacity, before/after feasible counts, the sorted IDs of portfolios excluded by each attempt, frontier size and ID intersection/union, stop state, and all unattempted values. The CLI requires complete equality with an independently frozen expected report before emitting a receipt with input/config/expected SHA-256 digests.

Stop semantics reuse the typed `CounterexampleSweepStatus` contract:

- `COUNTEREXAMPLE_FOUND`: at least one baseline-feasible portfolio is excluded under a tighter capacity.
- `DOMAIN_EXHAUSTED_NO_COUNTEREXAMPLE`: every configured cap was checked and no portfolio was excluded. This is only possible on a domain where the selected resource cap never binds for baseline-feasible portfolios.
- `BUDGET_EXHAUSTED`: the evaluation budget stopped the search while lower caps remain. This is not a pass and not a robustness certificate.

The evaluator fails closed if capacity order is invalid, the domain has duplicates or negative values, the period is absent/ambiguous, or a tighter cap unexpectedly adds a feasible portfolio. Current search is exact within the small input cap; no partial frontier is represented as complete.

## 3. Why a separate resource fixture matters

The original ten-project fixture had highly coupled resource values/caps, causing the first unit decrement of cost and crew cap to exclude the same portfolios. That could conceal a dimension mix-up. To avoid a vacuous resource benchmark, the dedicated fixture changes the synthetic P1 cost cap to 6 and P02 cost to 3 while leaving crew requirements and all outcomes fixed. This intentionally makes the constraints distinguishable. It is a test fixture, not an empirical cost calibration.

Independent subset enumeration was used to derive the expected reports, after which the JSON reports were frozen in the repository.

| Exact cap sweep | Baseline feasible | First tighter cap | Feasible after | Excluded baseline-feasible portfolios | Frontier size after | Frontier ID intersection / union |
|---|---:|---:|---:|---:|---:|---:|
| P1 cost cap: 6 → 5 | 82 | 5 | 72 | 10 | 8 | 6 / 10 |
| P1 crew cap: 3 → 2 | 82 | 2 | 42 | 40 | 6 | 0 / 14 |

Each sweep evaluates the baseline cap first, observes no feasible-set or frontier change, then stops at the first tighter cap that removes a baseline-feasible portfolio. Two lower values remain unattempted in each configured four-value domain. The two dimensions produce deliberately different reports; identical outcome values do not mean resource constraints are interchangeable.

## 4. Metamorphic and acceptance tests

- Baseline-cap evaluation preserves the full feasible set and frontier IDs exactly.
- Tightening any cost or crew cap cannot add feasible portfolios; relaxing a cap cannot remove any baseline-feasible portfolio.
- The capacity fixture separates cost and crew effects; the pinned removal counts must differ.
- Removed portfolio IDs reconcile with the feasible-set count difference exactly.
- Invalid, duplicate, ascending or negative capacity values fail closed.
- A one-evaluation budget with untried lower values returns `BUDGET_EXHAUSTED`, preserving the unattempted suffix.
- Receipts must match frozen known-answer JSON and hash the exact input, config and expected bytes.

These properties prove internal consistency of the bounded synthetic contract, not truth of the model assumptions.

## 5. Benchmark design and industry comparison

The first gate is not an impressive median runtime; it is correct feasible-set transition semantics, complete exclusion IDs, reproducibility and honest stop states. Next benchmark against:

1. **Static baseline:** one exact evaluation at the original cap.
2. **Full sensitivity curve:** evaluate every configured cap without early stop.
3. **Early-stop search:** same domain, stop at first removed portfolio.
4. **Independent constraint solver:** matched CP-SAT/MILP formulation with the same gates, dependencies, interaction linearizations, budgets, floors and objectives; statuses/bounds/gaps are preserved. See [issue #248](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/248).

Use frozen suites spanning periods, resource dimensions, tight/loose caps, near-binding capacities, prerequisites, high interaction density and infeasible-floor cases. Report the number of candidate subsets, evaluations, excluded portfolios, frontier IDs and objective vectors, p50/p95/max latency over repeated runs, peak RSS, inputs/config digests, solver versions and exact status distributions.

For multi-solver optimization benchmarks, report per-instance completion and failures; use Dolan–Moré performance profiles only on equivalent workloads and comparable success metrics (Dolan & Moré, 2002: https://doi.org/10.1007/S101070100263). COCO is an example of systematic frozen benchmark infrastructure, although its continuous black-box suites are not a drop-in test set for this discrete portfolio problem (https://coco-platform.org/index.html). For frontier approximations, hypervolume / IGD+ / epsilon indicators can supplement exact frontier comparison only after normalization and reference-set definitions are frozen; exact small instances should still be checked by full frontier equality. See the review of multi-objective performance indicators (Audet et al., 2021: https://doi.org/10.1016/j.ejor.2020.11.016) and the study of anytime multi-objective combinatorial optimization (2021: https://doi.org/10.1016/j.ins.2021.02.074).

OR-Tools documents distinct `OPTIMAL`, `FEASIBLE`, `INFEASIBLE`, `MODEL_INVALID`, and `UNKNOWN` states; never relabel a timed-out/unknown solver result as optimal or a counterexample search that ran out of evaluations as robustness: https://developers.google.com/optimization/cp/cp_solver.

These are benchmarking recommendations and research context, not evidence that this code currently outperforms state of the art.

## 6. Remaining work

- Broaden to independent project cost and crew requirement shifts; capacities are only one constraint parameter.
- Evaluate objective-vector overlap separately from portfolio-ID turnover.
- Add paired multi-period/correlated capacity shocks only after single-dimension semantics are verified.
- Build the independent CP-SAT/MILP baseline and use generated hard instances; easy-to-prune fixtures must not dominate the performance claims.
- Field validation requires authorized, reviewed data and independent follow-up outcomes. No scenario or cap range should be assigned a probability without separate calibration.