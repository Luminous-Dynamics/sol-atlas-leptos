# SA-PORTFOLIO-OR-SOLVER-BASELINE-001
## Independent operations-research reference for constrained multi-objective portfolios

**Status:** Formulation and qualification plan; independent solver implementation is not yet complete.  
**Tracking issue:** [#248](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/248).  
**Benchmark protocol:** [SA-PORTFOLIO-BENCHMARK-PROTOCOL-001](./SA-PORTFOLIO-BENCHMARK-PROTOCOL-001.md).  
**Input contract:** [SA-PORTFOLIO-SCALE-001](./SA-PORTFOLIO-SCALE-001.md).  
**Claim ceiling:** This is a plan for a synthetic independent reference, not a solver result or SOTA claim.

## 1. Why an independent baseline is the next step

The Rust implementation now has:
- a small exhaustive reference evaluator;
- an exact dependency-aware search candidate;
- frozen v0.1 / v0.2 answer manifests;
- generated 12 / 16 / 18 project comparisons;
- a deliberately easy 24-project prune test;
- repeated benchmark receipts; and
- scenario-wise robust regret output.

These are necessary correctness gates, but both algorithms share validation, objective aggregation and dominance code. A bug in the shared contract could therefore affect both algorithms. A separate operations-research formulation has value precisely because it encodes the constraints through different solver machinery.

The comparator must be matched in formulation and task. An optimizer that returns one feasible portfolio cannot be compared with a search required to return every frontier member; a solver that ignores prerequisites or interaction rules is not a valid baseline.

## 2. Mathematical formulation

For each project i, define binary selection (x_i in {0,1}). For each declared scenario s and objective j, project outcome (v_{i,s,j}) is a known synthetic coefficient. Scenario (s)'s baseline is (b_{s,j}).

### Evidence and prerequisite constraints

A project that is not evidence-gated PASS cannot be selected:

[
x_i=0 quad 	ext{when gate}_i 
e mathrm{PASS}.
]

For every prerequisite j required by project i:

[
x_i le x_j.
]

This means selecting a dependent requires the complete prerequisite closure; prerequisites are not optional bonus points.

### Period resource constraints

For every period p, cost and crew are hard independent limits:

[
sum_{i: period_i=p} c_i x_i le B_p,qquad
sum_{i: period_i=p} h_i x_i le H_p.
]

All resource coefficients must be non-negative integers and match the declared synthetic units.

### Interaction rules

For each interaction rule r with participants (R_r), define binary (y_r) indicating that all participants are selected:

[
y_r le x_i quad orall iin R_r,qquad
y_r ge sum_{iin R_r}x_i - (|R_r|-1).
]

For each objective and scenario, add that rule's signed adjustment coefficient (d_{r,s,j},y_r) **exactly once** when all participants are selected. Synergy and overlap reductions are both represented as signed coefficients; the rule type and reason remain recorded.

### Scenario outcomes and service floors

For each scenario and objective:

[
V_{s,j}(x,y)=b_{s,j}+sum_i v_{i,s,j}x_i+sum_r d_{r,s,j}y_r.
]

The declared hard service floor must hold in every required scenario after all interactions:

[
V_{s,mathrm{service}}(x,y)ge F_s quad orall s.
]

Do not add probability weights or combine stress cases into an expected value unless a separate future schema explicitly supplies calibrated probability semantics and validation.

## 3. How to compare multi-objective results

The first independent checks should be the scenario-objective ideals. For each declared scenario and each of the four objectives, solve one separate maximization problem. Each independent optimum and bound should equal the exact oracle's corresponding ideal on frozen small instances.

To recover a frontier, use a documented approach such as an epsilon-constraint procedure or a genuinely multiobjective solver. A single weighted sum is only a declared policy baseline. It does not prove it has found the full frontier, and a zero-weighted or improperly scaled objective can disappear silently.

For small instances, compare:
- all feasible objective vectors;
- the exact outcome-frontier objective vectors and exact portfolio IDs where the output contract distinguishes them;
- the per-scenario ideal values;
- all hard-constraint outcomes; and
- robust-regret vectors derived from the scenario ideals.

Duplicate objective vectors require explicit handling. Two portfolios with equal scenario/objective vectors are mutually non-dominating under strict Pareto dominance even when their resource vectors differ; report vector equality and resource/ID distinctions rather than silently discarding one.

## 4. Solver status is part of the result

Record exact solver release, formulation revision, parameters, wall-time limit, CPU/runner metadata, input digest, model digest, output digest, status, objective value and best bound. Preserve distinct states:
- **OPTIMAL** — objective is proven optimal under the declared model;
- **FEASIBLE** — an incumbent exists, but optimality is not proven;
- **INFEASIBLE** — model proves no feasible solution;
- **MODEL_INVALID** — formulation is invalid;
- **UNKNOWN** — no conclusive result within the solver's limits.

The cited status terminology is documented by the [Google OR-Tools CP-SAT reference](https://developers.google.com/optimization/cp/cp_solver). Keep this reference independent of the Rust enumerator's output; it should read the same versioned input contract, not an already-calculated frontier as hidden side information.

## 5. Suite design

The benchmark suite must contain multiple families:
1. frozen v0.1 and v0.2 tiny cases where exact enumeration gives complete known answers;
2. deterministic generated 12 / 16 / 18 project cases, with nontrivial resource bottlenecks, all three scenarios, interaction rules and protected floors;
3. varied dependency density and depth, including shared prerequisites and different valid staging orders;
4. resource stress from loose to tight budgets and crew caps, including cases infeasible only through crew capacity;
5. complementarity and overlap rules, including negative adjustment values;
6. scenario permutations and one-scenario-at-a-time stress mutations;
7. hard cases without blocks of obviously unselectable UNKNOWN decoys; and
8. adversarial negatives for missing prerequisites, invalid stage order, duplicate IDs, missing scenario data, floor failures and arithmetic overflow.

Keep the 24-project UNKNOWN-decoy fixture, but label it as a negative-control test. It is not representative of a hard combinatorial optimization problem and should never be used alone to claim scalability.

## 6. Reportable benchmarks and targets

On all exact-size instances:
- 100% oracle/solver agreement on feasibility and scenario-objective ideal values;
- zero gate, prerequisite, resource or floor violations;
- zero false OPTIMAL labels and zero mismatched model/input digests;
- exact frontier-vector equality where the independent formulation enumerates the complete frontier.

On larger instances:
- solver optimality gaps and status rates at fixed time / memory budgets;
- p50/p95/max wall time on paired, warmed runs;
- peak RSS and reported CPU / memory limits;
- feasible rate and number of distinct frontier vectors found;
- frontier quality under a normalization/reference point declared *before* running any method;
- regret-envelope and ideal-gap summaries; and
- exact performance distributions across multiple input instances and seeds.

Do not pick an arbitrary “10x” or “beats SOTA” threshold first. Measure a baseline, define the meaningful decision-quality target, then compare speed to target with confidence across the instance suite. The [COCO performance-assessment approach](https://numbbo.github.io/coco-doc/bbob-biobj/perf-assessment/) is a useful example of target-based multiobjective measurement, but the portfolio domain needs its own discrete, dependency-constrained instances and independent solver definitions.

## 7. Implementation path

1. Choose a pinned solver version and a reproducible runtime path.
2. Build the v0.1 and v0.2 model from the contract in an independent implementation. Prefer C++ CP-SAT or another native integer solver behind a machine-readable stdin/stdout protocol if no reliable Rust API exists; keep the production and main benchmark tooling Rust-first and avoid introducing a production Python runtime solely for this comparator.
3. Verify each constraint against fixtures with hand-computed tiny cases before comparing outcome values.
4. Compare individual scenario-objective ideals to exact results.
5. Add complete frontier extraction on tiny cases; validate vectors, ties and resource outcomes.
6. Benchmark larger generated cases using the [benchmark protocol](./SA-PORTFOLIO-BENCHMARK-PROTOCOL-001.md), recording all environment, formulation, status and digest metadata.
7. Publish results with explicit limits. Real data require source rights, a named owner, independently validated outcomes and legitimate decision rights.

**Conclusion:** beating industry practice requires an independent formulation and a reproducible, multi-instance comparison—not a self-comparison between two algorithms that share the same evaluator and dominance implementation.
