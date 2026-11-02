# Energy dispatch anytime engine v0.1

**Tracking issue:** [#417](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/417)  
**Implementation:** `sol-atlas-core/src/energy_dispatch.rs`  
**Base:** solver-compute-calibration v0.3 research branch

## API

The additive `assess_energy_dispatch_anytime(&EnergyDispatchRequest)` API preserves the current `assess_energy_dispatch` signature and behavior.

The legacy exact API remains all-or-nothing:
- exhaustive completion returns the exact complete Pareto frontier;
- candidate-expansion/frontier cap returns an error;
- no partial result is labelled exact.

The new API returns a typed status:

| Status | Meaning |
|---|---|
| `exact_complete` | Exhaustive search completed; complete Pareto frontier is present |
| `feasible_unproven` | At least one complete horizon trajectory exists, but search stopped at a declared cap |
| `proven_infeasible` | Exhaustive search completed without a feasible complete trajectory |
| `limit_without_candidate` | Search stopped at a declared cap before a complete trajectory existed |

Only candidate-expansion and Pareto-frontier cap errors are converted into bounded outcomes. Arithmetic overflow, malformed input and other internal errors remain errors.

## Incumbent policy

The search keeps an independent incumbent before Pareto insertion. Each complete schedule is scored using the existing objective vector and selected under a deterministic lexicographic policy:
1. essential unserved energy;
2. flexible unserved energy;
3. operating cost;
4. declared grid carbon;
5. curtailed PV;
6. descending terminal SOC.

Ties retain the first witness in the fixed ascending integer-action traversal. The policy is versioned as `lexicographic-objective-v0.1`.

The incumbent is always a complete horizon trajectory. It is never an incomplete interval path. For `feasible_unproven`, the report omits the Pareto frontier completely. It reports only the incumbent and the descriptive number of frontier entries observed before the cap. A frontier is published only after exhaustive completion.

## Work counters

The original exact search counter is retained for compatibility: `evaluated_action_expansions` is the attempted nested-loop enumeration-prefix visit count and includes the first rejected visit when a cap is exceeded.

The anytime report exposes:
- `attempted_enumeration_prefix_visits`: includes the first cap-denied attempt;
- `admitted_enumeration_prefix_visits`: only admitted work and never greater than the configured cap.

These are five-depth enumeration-prefix visits, not complete schedules, not complete dispatch action vectors, and not directly comparable to CP-SAT nodes or Gurobi WorkLimit values.

## Audit boundary

A `feasible_unproven` result is **not** an independently audited decision. The engine does not own the auditor and does not trust its own feasibility assertion as external evidence.

A caller may count an incumbent as decision-ready only after:
1. serialize the exact original request and incumbent schedule bytes;
2. invoke the separate standalone `energy-dispatch-schedule-auditor` process;
3. verify an accepted receipt bound to the exact request and schedule byte digests and exact auditor executable;
4. separately evaluate the pinned essential-service/reserve policy;
5. ensure the receipt and evidence remain current at the intended action boundary.

This API does not claim physical-model completeness, current telemetry truth, future viability, operator permission or authority to actuate.

## Qualification tests

The added unit tests verify:
- exact-complete parity with the legacy exact API and complete frontier;
- expansion cap before first candidate -> `limit_without_candidate`, no infeasibility claim;
- expansion cap after a complete candidate -> `feasible_unproven`, full-horizon incumbent, no published partial frontier;
- admitted expansion count never exceeds cap while attempted count explicitly records the denied visit;
- exhaustive no-schedule case -> `proven_infeasible`;
- invalid energy quantum remains an error rather than a partial success.

The targeted workflow checks the exact PR head with pinned Rust 1.96.0, formatting of the changed source module, locked core build and core library tests. The repository-wide formatter is intentionally not used by this targeted workflow because the #410 calibration base has unrelated existing formatting drift; the normal repository Check remains a separate gate and is reported honestly.

## Industry benchmark plan

OR-Tools CP-SAT already distinguishes FEASIBLE from OPTIMAL. Gurobi can expose incumbents and bounds under time/work limits. This API does not claim to invent anytime optimization.

Its narrower research objective is to report how many complete finite-model candidates survive independent domain auditing and a separate service-floor policy before a fixed deadline. Comparators must use identical request bytes, service floors, uncertainty knowledge, hardware cohorts and end-to-end timing boundaries.

Required metrics are:
- time to first complete candidate;
- time to first independently accepted audit receipt;
- time to first audited, service-floor-passing candidate;
- complete candidate count, expansion/work counters and terminal status;
- exact-reference component-wise regret on held-out enumerable cases;
- audit rejection, unavailable, stale/mismatched receipt and service-floor failure denominators;
- quality-vs-compute and audited-yield-vs-compute curves.

No scalar optimality gap is emitted for this multi-objective enumerator. No cross-solver expansion-count equivalence is assumed. A 25% median reduction in time-to-first-audited-candidate at matched quality is a promotion hypothesis, not an achieved result.

## Executable v0.1 implementation

The additive `assess_energy_dispatch_anytime` API is implemented in `sol-atlas-core/src/energy_dispatch.rs`. The existing `assess_energy_dispatch` function remains all-or-nothing and retains its exact output contract.

The new report uses:
- `ExactComplete` — complete frontier included;
- `FeasibleUnproven` — complete-horizon incumbent included, frontier omitted;
- `ProvenInfeasible` — only after exhaustive completion with no feasible schedule;
- `LimitWithoutCandidate` — declared cap reached before any complete schedule.

Only `CandidateExpansionCapExceeded` and `ParetoFrontierCapExceeded` are translated to bounded outcomes. Other errors propagate. The search stores the best complete incumbent before attempting Pareto insertion; the candidate is selected by a versioned lexicographic objective key and retained only when it improves the incumbent, avoiding an unnecessary schedule clone for every completed trajectory.

The report separates attempted enumeration-prefix visits (including the first cap-denied attempt) from admitted prefix visits (which never exceed the cap). The old exact API's successful work count is unchanged because a completed exact search has no denied visit.

### Additional interruption tests

The implementation includes tests for:
- exact-complete parity between the legacy exact result and the anytime complete frontier;
- cap before first candidate;
- cap after at least one complete candidate, with no partial frontier publication;
- exhaustive finite-model infeasibility;
- malformed-input error preservation;
- complete-incumbent preservation when Pareto-frontier insertion exceeds the frontier cap.

These tests establish finite algorithm/API semantics only. The incumbent remains unaudited until the separate schedule auditor accepts its exact request/schedule bytes, and service-floor qualification remains a distinct caller policy.

