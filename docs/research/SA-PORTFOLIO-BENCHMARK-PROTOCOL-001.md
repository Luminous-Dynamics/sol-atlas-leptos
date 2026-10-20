# SA-PORTFOLIO-BENCHMARK-PROTOCOL-001
## A reproducible comparison protocol for discrete, dependency-constrained multi-objective portfolios

**Status:** Protocol and harness changes implemented. Exact-head workflow passed on commit `b7148214db462ed52d5626f4da34e00812c0e4b7`; the warmed, order-alternated measurements are recorded in [SA-PORTFOLIO-SCALE-001 section 9](./SA-PORTFOLIO-SCALE-001.md). Independent CP-SAT/MILP result comparison remains outstanding.  
**Related implementations:** [Scale and frontier search](./SA-PORTFOLIO-SCALE-001.md), [baseline ablations](./SA-ECONOMIC-PORTFOLIO-BASELINES-001.md), [robust regret](./SA-ROBUST-REGRET-001.md).  
**Claim ceiling:** Synthetic solver evaluation only until real case data, decision rights and independent validation are approved.

## 1. Why benchmark design is part of the algorithm

A runtime number is meaningful only when the task, inputs, outputs, machine and measurement procedure are explicit. Exact enumeration returns every feasible portfolio plus the frontier; frontier search returns the frontier and an accounting certificate. Reporting their speeds together is useful, but must disclose that difference in returned work.

The first 21-repeat benchmark (recorded in the scale report) always measured exact enumeration before frontier search and did not record CPU/runner metadata or perform explicit warm-ups. Treat it as a preliminary baseline, not the final comparison. The methodology increment now adds two warm-up runs, alternates measured pair order, records OS / architecture / CPU / memory metadata where available, and records per-run samples and nearest-rank p50/p95/max. Results from different protocols must not be mixed as if they were one experiment.

COCO provides a useful example of disciplined optimizer assessment: fixed suites, public problem instances and data archives, objective-quality targets and runtime-to-target distributions. Its black-box continuous, mixed-integer and bi-objective testbeds are not, by themselves, a direct proxy for public-development portfolios with prerequisites, hard service floors, per-period budgets and complementarity. Adopt the discipline, not a false equivalence: https://coco-platform.org/data-archive/index.html, https://coco-platform.org/ppdata-archive/, https://arxiv.org/abs/1605.01746.

## 2. Benchmark tiers

| Tier | Instance type | Required reference | What it can prove |
|---|---|---|---|
| T0: contract correctness | Frozen 8- and 10-project known-answer corpora; permutations and negative controls | Exhaustive exact Rust oracle plus independently calculated manifest | 100% exact feasible-set/frontier agreement on finite instances; no constraint or accounting violations |
| T1: deterministic generated | 12 / 16 / 18 projects, 3 periods, mixed gates, dependencies/resources, floors and interaction rules | Exhaustive oracle on the same generated input | Search correctness beyond the hand-written example; full subset accounting |
| T2: adversarial pruning | 24-project case with 16 UNKNOWN-gated decoys, plus future hard cases | Frozen frontier reference; exact oracle on the 8-project core | Fails-closed behavior and proof-accounting under a deliberately easy prune case only |
| T3: random and structured hard cases | Many deterministic seeds and varied dependency graphs, coupling rules, constraint tightness and scenario sets | Exact oracle on sizes that fit; independent OR solver on larger cases | Distributional correctness and whether performance persists outside a friendly fixture |
| T4: external solver comparison | Matched integer model run through an independent MILP / CP-SAT or other suitable operations-research solver | Solver status and best bound / optimality gap; exact result on small cases | Comparative quality and scale under disclosed formulations, limits and hardware |
| T5: authorized field validation | Owner-approved, source-versioned real case studies and independently checked outcomes | Agreed historical/current outcome definitions, counterfactual and external review | External validity only for the population, domain and time horizon actually tested |

No T0–T2 result proves an industry/SOTA win. T4 requires that models represent the same feasible set, objective directions, interactions, constraints and evaluation budgets. T5 requires domain evidence and legitimate decision rights, not just software passes.

## 3. Correctness gates: fail closed

Before measuring speed, require:
- 100% equality with the known feasible set and complete outcome frontier on every exact-size fixture;
- zero returned unknown/blocked-gate projects, missing prerequisites, cost/crew cap violations or protected-floor violations;
- 100% accounting reconciliation of all (2^n) subset decisions as either leaves or subtrees pruned under a proved infeasibility condition;
- deterministic results under row permutations and repeated runs;
- exact input / reference-manifest SHA-256 digests in receipts;
- every benchmark result remains tied to the exact source commit, toolchain, build profile and workload manifest.

A missing/stale/queued check is not a pass. A reference that contains only a frontier validates frontier equality, not equality of the full feasible set. Large-instance search can only claim exactly what its accounting certificate and reference comparison justify.

## 4. Timing protocol

For each workload and algorithm:
1. Pin a specific commit, benchmark input digest, manifest digest, toolchain version and build profile. Use an optimized release build for algorithm latency, but keep formatting, lint and correctness tests separate and mandatory.
2. Run two warm-up executions per algorithm. Check their outputs / counters are stable and their exact output agrees with the expected manifest before collecting timings.
3. Use paired comparisons over the same input. Alternate algorithm order between measured pairs to reduce fixed order/cache bias. Do not subtract validation time from only one algorithm; state whether I/O, parsing, validation and output serialization are inside or outside the timer.
4. Retain all per-run latency samples and report minimum, median (p50), p95, maximum and the sample count; do not report only the best run. Increase repetitions for noisier machines. Use multiple independent runs / runner classes before inferring hardware-independent trends.
5. Record OS, architecture, runner identity where available, CPU model, logical processor count, available memory, process high-water RSS where available, compiler/toolchain and whether the build is optimized. Report missing fields honestly.
6. Separate cold-start latency, steady-state algorithm latency and full end-to-end command latency; use each only for the question it represents.

The CLI in `tools/sa-economic-portfolio-oracle` now adds warm-ups, alternating pair order and host metadata to the benchmark receipts. It remains a smoke benchmark until varied workload families and an independent solver are included.

## 5. Quality and scaling metrics

### Exact-size instances
Report:
- feasible-set count and exact frontier match;
- hard-constraint/gate failure counts;
- frontier recall and precision (both must be 1.0 against exact output);
- exact objective/resource vector agreement;
- number of visited nodes, evaluated leaves and pruned completions by reason.

### Larger instances
Report:
- feasible coverage and number of mutually non-dominated alternatives returned;
- best bound / proven optimality gap from the independent solver, with status such as optimal, feasible-but-not-proven, infeasible, model-invalid or unknown;
- p50/p95/max wall-clock runtime, peak RSS, memory/time limit and exit/status semantics;
- frontier-quality indicators such as hypervolume only when objective direction, normalization and reference point are fixed in advance and published;
- scenario-wise outcome gaps, robust-regret vectors, outcome- and regret-frontier sizes, and instability under one-at-a-time scenario / outcome perturbations.

COCO's bi-objective benchmark methodology illustrates fixed-target runtime measurement against declared solution-quality targets and hypervolume-based quality indicators. For this application, adapt that approach to discrete cases without concealing hard constraint failures: https://numbbo.github.io/coco-doc/bbob-biobj/perf-assessment/ and https://arxiv.org/abs/1605.01746.

### Evidence quality
Report exact inputs and schema versions, source-data owners / licenses / dates for non-synthetic cases, evaluation code hashes, calibration / validation status and the claim ceiling. Never translate synthetic points into dollars, jobs, service delivery or social benefit.

## 6. Independent OR-solver baseline specification

The follow-on CP-SAT / MILP comparison must encode the same model:
- binary decision (x_i) for each project;
- (x_i=0) for evidence gates that are not PASS;
- (x_i le x_j) for each prerequisite (j) of project (i);
- period-specific (sum_i cost_i x_i le B_p) and (sum_i crew_i x_i le C_p);
- one binary interaction variable (y_r) per declared interaction rule, constrained as the AND of all participant (x_i) variables;
- each protected service floor applies separately in every declared stress scenario, including interaction adjustments;
- each objective/scenario has a separate objective run unless the solver is explicitly performing a multiobjective method with a published definition.

Use the independent solver first on tiny cases and compare all feasible/frontier outcomes to the Rust oracle. For larger cases, present each objective ideal, solver status and best bound; use a declared frontier enumeration/epsilon-constraint method or an appropriately configured multiobjective solver. Do not say two solvers agree because they return one similar-looking portfolio.

Google OR-Tools documents CP-SAT as an integer constraint solver and defines explicit statuses (OPTIMAL, FEASIBLE, INFEASIBLE, MODEL_INVALID, UNKNOWN); record the actual status and bound/gap rather than treating any incumbent as an optimum: https://developers.google.com/optimization/cp/cp_solver.

## 7. Decision and governance gates

- Preserve the complete outcome frontier and separate robust-regret frontier. No default scalar “national winner”.
- Keep stress cases unweighted unless a future schema contains calibrated probability semantics and validation.
- Predeclare any criterion weights/tolerances and report sensitivity to them. The World Bank Infrastructure Prioritization Framework likewise emphasizes multidimensional outcomes, explicit criteria/weighting/sensitivity, and transparent review: https://ppp.worldbank.org/library/infrastructure-prioritization-framework-ipf.
- Treat a decision-support system as governance-relevant software. NIST AI RMF offers Govern / Map / Measure / Manage functions to organize trust, risk and accountability work; it is voluntary guidance, not something a passing CI workflow establishes: https://www.nist.gov/itl/ai-risk-management-framework/nist-ai-rmf-playbook.
- Promote real data only with permissions, source provenance, an authorized decision owner, community/statutory review and an independent evaluator.

## 8. What would justify a SOTA claim?

A credible claim would need: (1) a public and reproducible instance suite representative of the task; (2) matched independent baselines and disclosed parameters; (3) comparable outputs and solver statuses; (4) multi-run / multi-instance distributions with host metadata; (5) exact agreement on all small cases and no constraint violations; (6) quality/runtime trade-offs across diverse larger cases; and (7) an externally reviewable replication package. Until then, the strongest defensible statement is limited to the exact cases measured and the recorded execution environment.

**Bottom line:** the distinctive contribution should be auditable portfolio composition under hard floors, dependencies, resource bottlenecks and multi-scenario regret—not a claim that one microbenchmark ratio beats commercial or research state of the art.
