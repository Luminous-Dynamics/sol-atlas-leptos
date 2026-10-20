# Intertemporal Resource Allocation V0.1 — Exact Reference

**Status:** implementation candidate; exact-head hosted qualification required.  
**Parent:** Sol-Atlas resilience work in [PR #116](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/116) and [issue #153](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/153).  
**Claim ceiling:** exact bounded allocation under explicitly declared integer-quanta constraints; no physical network, forecast, probability, causal, or operational-authority claim.

## Paradigm shift: from snapshot feasibility to horizon-wide service

The existing shared-resource allocator optimizes one aligned snapshot. Storage introduces intertemporal coupling: withdrawing one unit now changes what remains for a later interval. A myopic allocation can appear feasible in every independent time step while violating a terminal reserve, or exhaust stock before an essential service's later demand arrives.

V0.1 therefore starts with a small **exhaustive integer reference optimizer**. It is not designed to beat an industrial LP/MILP optimizer on scale. It defines an executable contract that a future scalable solver must match on supported small cases. Exhaustiveness makes its objective and feasibility outcome exact over its declared finite discrete search space.

## Contract

- The time grid is explicit, non-empty, contiguous, and bounded to 8 intervals.
- Service demand, resource throughput, inflow, declared losses and stock use integer quanta per interval. Units name the meaning of one quantum.
- Each physical resource is declared once. Each usage binds one service and one resource through a positive rational coefficient. Coefficients are rounded upward per usage row so integer accounting cannot understate resource use.
- An inventory records opening stock, per-interval inflow and losses, maximum stock, minimum reserve and terminal reserve. Each interval checks:

  `opening + inflow = losses + withdrawals + spill + ending_stock`

  Spill is explicit when inventory after planned withdrawal exceeds maximum storage. Throughput is constrained separately from storage.
- All three policies are declared rather than smuggled into the evaluator: equal-coverage leximin, fixed priority, and essential-floor-first. They use deterministic tie-breaking and integer / exact rational comparisons.
- An unknown required quantity is never converted to zero. It produces `indeterminate_input`, no proposed allocation, and an explicit blocker list.
- A complete search above the caller or hard search budget produces `search_cap_exceeded` with no provisional allocation marked as optimal.
- A fully enumerated space without a feasible plan produces `infeasible`. A feasible optimum returns `optimal`, a complete inventory ledger and every per-service/per-interval floor result.
- Each request carries a comparison-pair identity, baseline/candidate role, scenario ID, named failure-bundle IDs, and assumption/evidence references. These identities do not imply failure probabilities or authenticate the references.

## Frozen synthetic case

`schemas/examples/intertemporal-tank-two-step.synthetic.json` has two intervals. A tank opens with 3 water quanta, has a per-interval withdrawal limit of 3, receives inflows `[0, 3]`, can store at most 4, and must retain at least 2 at the end. Essential service A requires 1 per interval; general service B requests 3 per interval. The exact objective prioritizes weighted essential-floor shortfall.

The expected optimal plan allocates A = `[1, 1]` and B = `[1, 1]`. Tank stock ends at 1 after interval 0 and 2 after interval 1. The final stock condition is binding. The complete assignment space contains 64 integer allocations; a separately authored arithmetic enumerator found **37 feasible plans** under all throughput, stock-balance, storage and terminal-reserve constraints. The Rust regression test freezes both counts. The budget is explicitly a cap on complete candidate plans examined—not a count of internal DFS nodes. This is an invented unit-test fixture, not a real water forecast.

Negative controls vary this case to prove that missing inflow remains unknown and exhausted search budgets emit no plan. Removing the later inflow must reduce service while still preserving the terminal reserve; it is not automatically infeasible because service floors are objective terms, not hard minimum-delivery constraints in V0.1. A separate impossible-loss input verifies the genuinely infeasible disposition.

## Reference evaluation and industry comparison

MIPLIB 2017 is a standard comparison corpus for mixed-integer optimization, with a selected 240-instance benchmark set and an independently checked feasibility protocol. It is a useful solver-runtime comparison reference, but its unrelated generic instances do not by themselves measure whether a lifeline-service allocation is physically meaningful. See [MIPLIB 2017](https://miplib.zib.de/) and its [official benchmark set](https://miplib.zib.de/set_benchmark.html).

For real water systems, compare any hydraulic claims separately against EPANET's extended-period simulations. EPANET explicitly models pipes, junctions, pumps, valves, reservoirs and tanks, and tracks flow, pressure, tank levels and water-quality processes over time. This allocation oracle does none of that physics. See the [U.S. EPA EPANET documentation](https://www.epa.gov/water-research/epanet) and [EPANET 2.2 technical overview](https://usepa.github.io/EPANET2.2/).

NIST's September 2026 lifeline-infrastructure review highlights interdependence between water, power, transport, communications and other services, and the lack of a single system-level resilience guideline that covers service objectives, recovery and cross-sector priorities. This supports an evaluation that keeps per-interval/community service floors and recovery-reserve constraints visible instead of optimizing one aggregate uptime score. See [NIST, *Advancing Resilience Across Lifeline Infrastructure Systems* (29 September 2026)](https://www.nist.gov/publications/advancing-resilience-across-lifeline-infrastructure-systems-need-consistent-guidance).

## Proposed gates — not yet scale or field results

| Gate | Acceptance target |
|---|---|
| Conservation | 100% of inventory result rows satisfy the integer balance identity exactly; all stock/throughput bounds hold |
| Unknown handling | 100% of frozen missing-input cases return indeterminate with no numeric allocation |
| Search bounds | 100% of over-budget cases return no provisional allocation |
| Exactness | All enumerated feasible plans obey every declared constraint; returned objective is not beaten by any plan in the same finite search space |
| Reproducibility | Identical input bytes and executable yield byte-identical receipts; order permutations of services/resources/usages do not change the canonical receipt |
| Snapshot parity | One-step equal-coverage case agrees with the current single-snapshot allocator on matched unit coefficients and demand/capacity inputs |
| Comparison discipline | Baseline and candidate use matching scenario/time grid and shared resource definitions; report service-floor wins/losses and reserve deficits by community and interval |
| Scale transition | Compare future LP/QP/MILP implementations against this exact oracle on the overlapping small-instance corpus before timing larger cases |

For a future scalable solver, report p50/p95/max latency, peak memory, solver/version and optimality gap, exact constraints, feasible/infeasible/indeterminate denominators, terminal-reserve violations, essential-floor shortfalls and community-level outcomes. Proposed workload tiers should be measured first; do not soften conservation or protected-floor constraints to hit a latency target.

## Qualification evidence

The dedicated workflow checks format, the focused module tests, and Clippy correctness/suspicious lints; it then runs the CLI repeatedly and checks optimal, indeterminate, infeasible and over-budget results. It validates both JSON Schemas and validates the frozen request plus every emitted receipt with an independent full Draft 2020-12 validator. A content-addressed manifest binds the checked-out source commit, fixture, schemas and receipts, and the workflow uploads these files as an artifact.

Exact-head run [#39396331427](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/39396331427) passed at commit `5c19bd5ca62880749977a488ca828ef411f17685`. After download, an independent offline review rechecked eight SHA-256 bindings, meta-validated both schemas, validated the fixture and all five receipts, and confirmed the expected dispositions and digests.

The broader repository `Check` workflow separately failed on `service_equity_json_fixtures_match_distributional_golden_output` (359 passed, 1 failed). All focused intertemporal tests passed; the unrelated golden-fixture failure is not attributed to the new module or reported as a pass.

## Nonclaims and next steps

This crate is an exact oracle only for tiny finite integer cases. It does not validate quantity quanta against an external unit system, authenticate evidence references, model pressure/head loss/pump curves, estimate outage probabilities, infer causal impact, or authorize resource operations. The lineage fields preserve declared evidence/assumption references; canonical evidence qualification stays with the existing evidence layer.

Next, implement a sparse horizon optimizer and compare it to this oracle with differential/property tests on randomized small instances. Then connect a qualified EPANET adapter and explicit electricity/backup-power model for domain-specific coupled cases. Physical pilot use requires calibrated inputs, engineering review, and separate operational authorization.
