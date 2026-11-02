# ENERGY-ANYTIME-AUDIT-001 — Independently audited incumbents under bounded search

**Status:** research specification, v0.1  
**Tracking issue:** [#417](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/417)  
**Stack:** [solver compute calibration #410](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/410)  
**Evidence class:** design proposal; no new executable behavior or benchmark result is claimed here

## 1. Decision

Add an assurance-aware anytime search mode to the existing finite energy-dispatch enumerator.

The mode should preserve a fully materialized, complete-horizon candidate when one has been found, even if a search expansion or wall-clock budget prevents exhaustive completion. A separately compiled schedule auditor must verify the exact request and candidate bytes before the candidate can count as independently audited.

The result must keep three assertions independent:

1. **Feasibility:** a complete trajectory satisfies the predicates checked by the domain-level schedule auditor.
2. **Completeness:** the declared finite search space was exhaustively explored, or search ended before that point.
3. **Optimality:** a candidate/frontier is optimal only when the declared model and completed search establish that claim.

This is a composition of established optimization semantics, not a claim to invent anytime optimization. OR-Tools CP-SAT distinguishes FEASIBLE from OPTIMAL. Gurobi documents incumbent/bound output under time or work limits and exposes a deterministic WorkLimit for controlled stopping. The proposed distinction is the independently executed, byte-bound domain schedule audit and its explicit authority boundary.

References:
- OR-Tools CP-SAT status semantics: https://developers.google.com/optimization/cp/cp_solver
- Gurobi measurement under time limits: https://support.gurobi.com/hc/en-us/articles/37638046123921-What-to-measure-during-benchmarking
- Gurobi deterministic work limits and parameter guidance: https://docs.gurobi.com/projects/optimizer/en/current/concepts/parameters/guidelines.html

## 2. Existing components and boundaries

Reuse, do not duplicate:

| Component | Existing role |
|---|---|
| Exact finite dispatch oracle (#309) | Exhaustive integer-quantum search and complete Pareto frontier for its declared finite model |
| Independent complete-set reference (#321) | Test-only differential reference on enumerable tiny cases |
| Domain schedule auditor (#351) | Separate executable checking exact request/schedule bytes and domain-level trajectory predicates |
| Replan policy bake-off (#403) | Online decision rules and evaluator-only execution truth |
| Temporal mesh allocator (#408) | Safety-boundary preservation and compute-budgeted mesh selection |
| Trace validator/aggregator (#410) | Versioned source-bound trace parsing, cohort isolation and outcome denominators |
| Profile runner | Measures the actual `assess_energy_dispatch` kernel and emits a schedule for external audit |

The current profiler README states that the first campaign repeats one two-interval synthetic request 30 times. It is a measured runtime on a synthetic workload, not a broad workload-scaling or industry comparison. The runner also measures only the kernel call; request parsing, serialization and independent audit are outside its current `elapsed_ns`.

The current exact API is intentionally all-or-nothing: expansion-cap exhaustion returns an error and does not publish a candidate. Preserve that API and its exactness contract. Introduce an additive anytime API; do not change what the current exact result means.

## 3. Result contract

The result should be a typed enum or an equivalently unambiguous tagged schema:

| Disposition | Required meaning |
|---|---|
| `exact_complete` | Declared search completed. A published Pareto frontier is complete for the declared finite model. |
| `feasible_unproven` | At least one complete trajectory was found; search did not establish completeness or optimality. |
| `proven_infeasible` | Exhaustive search completed and found no feasible trajectory. |
| `limit_without_candidate` | A declared budget/deadline stopped search before a complete trajectory was found. |
| `input_rejected` | Input failed schema/semantic admission before search. |
| `search_error` | Unexpected execution failure; never reinterpret as infeasibility. |

### 3.1 Incumbent rules

For `feasible_unproven`:

- Preserve only a fully materialized trajectory containing exactly one dispatch record for every required interval, in request order.
- Select deterministically using a separately versioned candidate-selection policy. Pin its exact bytes/digest and document objective ordering and tie-breaks.
- A candidate found in a partial traversal is an incumbent, not an optimum.
- The set of candidates seen before a cap is not a complete feasible set or Pareto frontier. Do not publish `pareto_frontier_complete=true`, an exact search flag or an unqualified frontier count.
- Do not fabricate scalar objective values, dual bounds or MIP gaps for a multi-objective enumerator.
- Retain expansion and frontier limits, stop reason and counters. A cap-exhausted search cannot become `proven_infeasible`.
- A no-candidate budget exhaustion remains distinct from exhaustive infeasibility.

Candidate selection should initially be a transparent deterministic lexicographic policy, not a learned ranking. A later policy may compare alternate traversal orderings, but its selection policy digest and training/tuning split must be explicit.

### 3.2 Separate service-floor disposition

Mathematical feasibility and service adequacy are distinct.

The schedule auditor can accept a correctly reconciled trajectory that deliberately leaves some load unserved if that is allowed by the request model. Therefore the evaluator must separately apply a declared service-floor policy to essential service, interruption duration, terminal reserve and any other protected quantity.

A candidate is **decision-ready in this benchmark** only when:
1. its exact request/schedule pair has an accepted independent audit receipt; and
2. the separately pinned service-floor policy passes.

This status is still not actuation authority.

## 4. Independent audit envelope

Invoke the existing standalone schedule auditor as a separate process. The audit receipt must identify:

- request-byte SHA-256;
- candidate schedule-byte SHA-256;
- exact auditor executable SHA-256;
- auditor protocol/schema version;
- accepted/rejected disposition;
- complete predicate findings;
- receipt digest and, where available, exact command/build provenance.

An accepted receipt for another request, another schedule serialization, another auditor binary, or another schema version cannot be substituted.

Missing, stale, duplicate, mismatched, rejected or unavailable receipts do not count as audited. Search code must not call auditor internals, and the auditor must not depend on the search crate, objective evaluator, frontier helper or candidate-selection policy.

The auditor's scope remains exactly its declared domain predicates. It does not authenticate input sources, prove physical model suitability, prove that telemetry is current, prove search optimality, certify field safety or authorize dispatch. Operator approval, hardware interlocks and runtime control authorization remain outside this benchmark.

## 5. Benchmark protocol

### 5.1 Workload ladder

Start with synthetic, deterministic, hand-checkable requests. Scale only as exact full-search references and runtime permit.

| Dimension | Initial levels |
|---|---|
| Horizon | H=2, H=4, H=8; extend only after bounded runtime is measured |
| Generation | PV surplus, PV ramp, low/high generation |
| Demand | Essential-only, essential plus flexible demand, demand ramps |
| Grid | Connected, early outage, late outage, prolonged outage |
| Storage | Initial SOC variants, reserve variants, power/energy derating, rational non-unit efficiencies |
| Feasibility | Feasible, exhaustive infeasible, expansion-limit before candidate, expansion-limit after candidate |
| Audit | Positive, schedule mutation, request substitution, duplicate receipt, missing receipt, unavailable process, wrong executable |

Freeze case bytes, scenario-panel digest, seed, expected disposition and exact full-search result before evaluating candidate search orderings. Keep held-out case families/seeds; do not tune the incumbent ordering on the final evaluation set.

### 5.2 Budget ladder

Use both algorithmic and real-world budgets, but never conflate them:

- **Expansion budget:** deterministic limit on enumerated action expansions.
- **Wall-clock deadline:** external process deadline; machine/scheduler sensitive.
- **CPU time:** report only when directly measured with a documented clock/source.
- **Memory budget:** enforce and report the actual configured process/container limit.

For tiny exact cases, first complete an unrestricted bounded reference run to record total expansions. Re-run the same bytes with predeclared fractions of that count (for example 1%, 5%, 10%, 25%, 50% and 100%), retaining absolute expansion counts. These fractions are benchmark analysis aids, not online information available to the controller.

### 5.3 Baselines

1. Existing all-or-nothing exact API.
2. New anytime API with deterministic incumbent preservation.
3. Transparent deterministic greedy baseline.
4. Independent CP-SAT/MILP formulation only where model-translation parity and objective/constraint equality have been demonstrated.
5. Full exact enumeration as the complete-set reference for small instances.

Same request bytes, model scope, service floors, scenario knowledge, hardware cohort and declared compute budget are mandatory for a matched comparison. Report preprocessing, solve, serialization, audit and end-to-end time separately.

### 5.4 Metrics

Preserve every request in the denominator. Report:

- complete candidates found, no-candidate stops, exact completions, exhaustive infeasibility, unexpected errors;
- time to first complete candidate;
- time to first accepted independent audit receipt;
- end-to-end time to an audit-accepted, service-floor-passing decision;
- search expansions, complete schedules, frontier high-water mark (not a completeness claim), stop reason, peak RSS, and CPU/energy only when measured;
- audit acceptance, rejection and unavailable counts, with exact request/schedule/auditor identities;
- essential/flexible unserved energy, interruption duration, reserve, grid import, PV curtailment, cost, carbon and the full objective vector;
- component-wise deltas/regret against the complete exact reference, retaining signs and units;
- audit-accepted service-floor-passing candidates per unit of compute;
- quality-versus-budget and audited-yield-versus-budget curves.

Do not collapse incomparable objectives into an arbitrary scalar. A weighted or lexicographic selection policy is a named, versioned policy and must be reported separately from the underlying objective vector.

Wall time is not energy consumption. Report compute energy only with a named, available meter and a source-bound measurement. Otherwise emit `not_measured`.

## 6. Promotion gates and research targets

### Mandatory correctness gates

- 100% of emitted incumbents are complete horizon trajectories.
- 100% of candidate bytes are bound to the exact original request bytes.
- Zero false auditor accepts across all planted negative controls; positive controls pass.
- 100% full-search agreement with the independent exact reference on the enumerable panel.
- No partial search can claim exhaustive infeasibility, a complete frontier or proven optimality.
- No candidate without a matching accepted receipt and passing service-floor policy counts as decision-ready.
- Deterministic replay holds for pinned inputs/configurations wherever the implementation declares deterministic behavior.
- Duplicate/missing/mismatched/stale receipts, schedule mutation, request substitution and executable substitution fail closed.

### Performance hypotheses (not achieved results)

- At a matched deadline, the anytime path should increase the fraction of feasible cases with an accepted, service-floor-passing decision relative to the all-or-nothing exact path.
- A provisional promotion target is at least 25% lower median time-to-first-audited-candidate at equal model and quality constraints. Validate this target on held-out cases; do not report it as achieved unless measured.
- Publish component-wise quality-versus-compute curves and all failed/late cases. There is no universal scalar winner.

## 7. Industry comparison

| Reference | Fair comparison | Boundary |
|---|---|---|
| OR-Tools CP-SAT | FEASIBLE versus OPTIMAL outcomes and bounded-time candidate quality | Same model, constraints, objective and time/work budgets required |
| Gurobi | Incumbent, bound, gap and time/work-limit behavior | Work units are not hardware-independent seconds; record version and hardware |
| REopt v3 / REopt.jl | Matched distributed-energy sizing, dispatch, economics and resilience cases | Broader planning scope; do not compare unlike model boundaries |
| PyPSA | Matched temporal/network dispatch and rolling-horizon cases | Introduce matching network/commitment constraints before broader claims |
| SAC / learning-enhanced stochastic MPC research | Controller quality under identical uncertainty and degradation scenarios | Controller benchmarks do not substitute for independent schedule verification |

Current reference documentation:
- REopt v3: https://developer.nlr.gov/docs/energy-optimization/reopt/v3/
- PyPSA rolling horizon: https://docs.pypsa.org/latest/api/networks/optimize/
- PyPSA stochastic optimization: https://docs.pypsa.org/latest/user-guide/optimization/stochastic/
- IEEE Access SAC microgrid study (6 Oct 2026): https://doi.org/10.1109/ACCESS.2026.3740101
- Applied Energy learning-enhanced stochastic MPC (1 Nov 2026): https://doi.org/10.1016/j.apenergy.2026.128321

These references define comparison targets, not evidence that Sol-Atlas is presently superior.

## 8. Implementation sequence

1. Preserve the current exact API and trace v0.2 semantics.
2. Add a pure typed anytime outcome/result model and known-answer unit tests.
3. Refactor or extend the traversal to retain a best complete candidate without exposing mutable/partial interval paths.
4. Keep complete-frontier publication behind the exhaustive completion path only.
5. Add a separate candidate serializer and a separate audit-process integration. Verify exact request/schedule/executable bindings.
6. Add a versioned anytime report and trace schema; keep missing measurements explicit.
7. Add a small benchmark panel with exact reference runs and a mutation/receipt-negative panel.
8. Expand to larger horizons and matched CP-SAT/MILP cases only after the first slice is qualified.

## 9. Claim ceiling

This is an engineering and benchmark proposal. A successful candidate audit establishes only the auditor's declared trajectory predicates for those exact bytes. It does not establish source truth, physical completeness, forecast validity, field reliability, battery lifetime, network feasibility, regulatory compliance, operational authorization or SOTA superiority. Synthetic fixtures are conformance and hypothesis tests, not field evidence.
