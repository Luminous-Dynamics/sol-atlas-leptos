# SA-PORTFOLIO-SCENARIO-ROBUSTNESS-001
## Adversarial scenario-set sensitivity for auditable multi-objective portfolios

**Status:** Research protocol / implementation contract; executable perturbation harness and pinned outputs are not yet implemented.  
**Tracking issue:** [#254](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/254).  
**Builds on:** [portfolio scale/search](./SA-PORTFOLIO-SCALE-001.md), [robust regret](./SA-ROBUST-REGRET-001.md), [benchmark protocol](./SA-PORTFOLIO-BENCHMARK-PROTOCOL-001.md), and [independent solver baseline](./SA-PORTFOLIO-OR-SOLVER-BASELINE-001.md).  
**Claim ceiling:** Synthetic scenario-model sensitivity only. Passing this protocol cannot prove that real-world scenarios are complete, calibrate unknown probabilities, validate field outcomes, or establish SOTA.

## 1. The gap: robustness is conditional on the scenario set

The current portfolio pipeline correctly keeps stress cases unweighted and computes scenario-wise outcome and regret frontiers. But it has a boundary that matters: its regret and feasibility guarantees apply to the scenarios encoded in the input. A scenario that was omitted, a correlated shock that was never modeled, or a wrong interaction coefficient is invisible to the optimizer.

The central next question is therefore not only:

> How well does a portfolio perform across the declared scenarios?

It is also:

> How much does the candidate set, feasibility result, and regret story change when plausible parts of the scenario model are added, removed, or perturbed?

This is **scenario-set robustness**, distinct from scenario-wise robust regret. Robust regret is conditional on a fixed scenario universe; scenario-set robustness audits sensitivity to that universe. Neither method proves that the scenarios span reality.

This direction is consistent with established research on multi-scenario multi-objective robust decision-making. Shavazipour, Kwakkel and Miettinen discuss scenario discovery and robustness/optimality trade-offs under deep uncertainty (2021, *Environmental Modelling & Software*, DOI: [10.1016/j.envsoft.2021.105134](https://doi.org/10.1016/j.envsoft.2021.105134)). A 2025 interactive framework proposes scenario exploration and a benchmark for multi-objective decision-making under deep uncertainty (DOI: [10.1016/j.envsoft.2024.106233](https://doi.org/10.1016/j.envsoft.2024.106233)). A January 2026 paper proposes multi-stage, multi-scenario adaptive robust planning and moving-horizon reassessment (DOI: [10.1016/j.omega.2025.103405](https://doi.org/10.1016/j.omega.2025.103405)). These papers motivate benchmark questions; they do not establish that the Sol-Atlas implementation is novel or superior.

## 2. Keep the semantics separated

The benchmark must distinguish four things:

1. **Outcome uncertainty:** how a project's objective contributions change inside a given scenario.
2. **Constraint uncertainty:** how budgets, crew capacity, or service floors change.
3. **Model-form uncertainty:** whether a dependency, overlap or synergy rule was incorrectly specified.
4. **Scenario-set uncertainty:** whether an important scenario or combination of stressors was absent from the evaluated set.

Do not encode unknown scenario likelihoods as weights or probabilities. Do not mix a new scenario with the baseline scenario set and call its result a forecast. Each perturbation is a named, versioned counterfactual used to stress-test the decision model.

The first implementation is an **offline benchmark harness in Rust** that constructs deterministic input variants and compares them with independent expected outputs. Reuse the current input contract, exact evaluator, dependency-aware search, and robust-regret functions. Do not create a parallel production schema or require Python. Do not modify production recommendations to “win” a benchmark.

Adaptive recourse is a separate future capability. Re-solving a different portfolio for each fully known scenario is perfect hindsight, not an executable adaptive policy. A true adaptive model must define what is committed before observations, what can be observed and when, which decisions can still change afterward, the remaining resource constraints, and any costs or delays of changing course.

## 3. Required perturbation families

Every generated case binds the source input digest, generator version, perturbation ID, seed (where relevant), exact transformation parameters, resulting-input digest and expected-output digest.

### A. Leave-one-scenario-out

For every scenario (s) in a frozen baseline:

1. Remove (s) from the *selection model* while leaving all other input semantics unchanged.
2. Recompute the feasible set/frontier and the robust-regret view on the reduced universe.
3. Rescore every selected portfolio against the original full scenario universe.
4. Report separately: portfolios that fail an omitted scenario's hard floor; portfolio-ID frontier turnover; objective-vector frontier turnover; and objective-wise full-universe regret.

Do not hide a floor violation inside an objective-average score. A portfolio infeasible under a required original scenario is reported as infeasible, not merely “high regret.” The reduced-set frontier and full-set outcome frontier are different artifacts and must not be compared as if their memberships had the same semantics.

### B. Withheld scenario addition

Freeze a scenario that is excluded from the selection input but is defined in a separate, hashed challenge manifest. Once the candidate selection result is recorded, add the withheld scenario and reevaluate feasibility, outcome frontier and regret.

Start with hand-calculable cases where the new scenario is known to remove at least one formerly feasible portfolio, while a negative-control scenario changes no feasibility or objective comparisons. Expected results must be independently enumerated or calculated; the generator must not call the candidate evaluator to generate its own oracle.

### C. Correlated compound stress

One-factor-at-a-time tests are not sufficient. Include declared joint perturbations such as:

- elevated essential-service demand + constrained electricity availability + reduced crew capacity;
- cost escalation + prerequisite delay + reduced later-period budget;
- drought or heat stress + reduced service baseline + an affected complementarity rule;
- multiple interacting shocks whose combined effect differs from the sum of individual effects.

Keep each factor change and the combined transformation auditable. Avoid implying that one compound event is probabilistically likely unless supported by a separate calibrated model. A pairwise/interaction design is acceptable for the first bounded corpus; broader coverage can follow after the generator and mutation tests are reliable.

### D. Parameter and model-form mutation

Include monotonic, integer-valued variants for:

- raising selected project costs or crew requirements;
- tightening one period's cost/crew cap;
- changing scenario outcome coefficients by signed declared deltas;
- tightening/loosening a protected-service floor;
- removing, flipping or perturbing an interaction rule;
- removing or adding a dependency as an explicitly labeled model-form fault, not a valid input assumption.

A malformed-input test and a valid-but-wrong-model test are different. The former must fail closed; the latter should produce a valid result and demonstrate the effect of the incorrect assumption.

Every perturbation family needs a negative control. For example, increasing the cost of a project absent from every feasible portfolio should not change the feasible set; a scenario perturbation that is mathematically dominated and does not affect floors may leave a frontier unchanged. Negative controls ensure the test harness is not “detecting” change every time.

## 4. Metrics and exact definitions

Report separate metrics rather than one uninspectable robustness score.

### Hard safety and feasibility

- Non-PASS gate selections: exactly zero.
- Missing prerequisites: exactly zero.
- Resource and protected-floor violations in returned feasible portfolios: exactly zero under that input's declared constraints.
- For each stress-tested recommendation, separately report feasibility against the **frozen reference scenario universe**. A candidate that violates an omitted reference floor is a failure, not an acceptable trade-off.
- On exact-size cases, compare the entire feasible set and the complete Pareto frontier against a frozen exhaustive oracle. Frontier-only agreement is not full feasible-set equality.

### Scenario omission regret

For each candidate portfolio (p) chosen from a reduced scenario set, rescore it on every original reference scenario (s). Define objective-wise regret as:

[
r_{p,s,j} = I_{s,j}^{mathrm{reference}} - V_{p,s,j}
]

where the reference ideal (I_{s,j}^{mathrm{reference}}) is independently established from the complete feasible set for the full reference input, for scenario (s) and objective (j). Report the full vector and maximum across reference scenarios separately for every objective. If (p) violates a reference hard floor, record that infeasibility before or alongside its regret; regret cannot make an infeasible portfolio admissible.

Keep units, directions and scenario IDs explicit. Do not aggregate the four objectives into a scalar unless a distinct, pre-registered decision policy supplies explicit weights or tolerances and the result includes sensitivity analysis.

### Frontier stability

Report both:

- **Portfolio-ID Jaccard:** (|A cap B| / |A cup B|) for canonical portfolio-ID sets; define the empty/empty case as 1.
- **Objective-vector-set overlap:** exact set overlap of canonical scenario-by-objective vectors, independently of portfolio IDs.

Equal objective vectors do not necessarily mean equal resource use, prerequisites, geographic distribution, reversibility, delivery risk or public acceptability. Preserve portfolio IDs and resource metadata even when vectors duplicate. For a scenario deletion, compute stability on the common scenario projection separately from the full-vector comparison; vectors with different scenario dimensions cannot be compared directly.

### Sensitivity and robustness cost

- Number/share of reduced-set frontier portfolios that become infeasible under the full reference scenarios.
- Per-objective minimum, median, p95 and maximum scenario-omission regret across pinned cases (quantiles only over a clearly declared corpus; these are corpus statistics, not real-world probabilities).
- Frontier membership changes for each transformation family and severity, with all seeds retained.
- Per-objective reference-scenario degradation when using the full scenario-robust frontier versus a declared nominal-only or fixed-weight baseline.
- Runtime distributions and memory only after semantic equality has passed, using the existing warmed and order-alternated measurement protocol.

Averages never substitute for worst-case floor checks. Normalize metrics only using frozen reference ranges; do not let each method choose its own favorable normalization.

## 5. Metamorphic invariants

The Rust test harness should prove the following:

1. **Permutation invariance:** reordering project rows, interaction rows, or scenario rows must not change the canonical semantic result.
2. **Cost monotonicity:** increasing the cost of project (i) can remove feasible portfolios containing (i) but cannot make a previously infeasible portfolio feasible because of cost.
3. **Capacity monotonicity:** tightening a budget or crew cap cannot add feasible portfolios; relaxing it cannot remove a portfolio for that constraint alone.
4. **Scenario-floor monotonicity:** adding a scenario with an additional hard floor can remove feasible portfolios but cannot make an otherwise identical portfolio that violates the new floor feasible. Adding a scenario can change Pareto membership because it changes the objective vector; this is expected.
5. **Gate invariance:** any project whose gate is not exactly PASS must never occur in a returned portfolio, regardless of its objective contributions.
6. **Search accounting:** every subset must be accounted for as an evaluated leaf or as part of a proven-infeasible pruned subtree; no partial result may masquerade as complete.
7. **Independent expectations:** expected frontiers and mutation effects are frozen from hand calculation / separate exhaustive implementation or the independent solver in [#248](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/248), never generated by the function under test.
8. **Deterministic replay:** identical source bytes, generator version, seed and perturbation config produce identical outputs and digests.

Any violated invariant is a correctness failure, regardless of speed.

## 6. Benchmark ladder

| Tier | Case family | Required reference | Claim supported |
|---|---|---|---|
| S0 | 4–8 project hand-checkable case; scenario omission with known effect and a no-effect negative control | Independent manual calculation plus exhaustive enumeration | Specific transformation semantics are correct |
| S1 | Frozen v0.1/v0.2 corpus with each scenario held out in turn | Existing exact evaluator and frozen manifests, plus independently derived perturbed manifests | Within-corpus leave-one-out behavior |
| S2 | Deterministic generated small cases varying scenario count, dependency density, interaction arity and resource tightness | Exhaustive feasible-set/frontier oracle | Generated-case correctness, not field validity |
| S3 | Correlated compound stress and model-form mutation cases | Exhaustive oracle where tractable, then independent CP-SAT/MILP under [#248](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/248) | Resilience to defined perturbations and model misspecification |
| S4 | Larger generated suite and matched external solver | Exact inputs, solver statuses/bounds/gaps, complete receipts | Comparative algorithmic scale under matched semantics |
| S5 | Authorized, independently reviewed real cases | Source-versioned outcomes, domain-specific counterfactuals and review | External validity only for the specific population and horizon studied |

Do not publish a single aggregate rank across unlike case families. Stratify results by dependency density/depth, interaction density/arity, scenario count, constraint tightness and stress family. Freeze development/calibration cases separately from a final, held-out challenge suite; otherwise iteration can overfit the benchmark.

## 7. Comparisons that are fair—and comparisons that are not

Use these as explicitly different baselines:

- current full-declared-scenario exact/search plus robust regret;
- nominal-only selection, subsequently evaluated against the frozen full scenario universe;
- leave-one-scenario-out selection, similarly rescored;
- declared fixed-weight single-scenario or weighted-sum selection, as a policy baseline only;
- an independent CP-SAT/MILP model with the same gates, dependencies, interactions, budgets, floors and objective directions.

Do not compare the full frontier with a one-solution weighted-sum optimizer as if both solved the same task. Do not award a speed win to a method that skips validation, returns fewer output objects or evaluates fewer scenarios. No model that changes the feasible set semantics is an apples-to-apples baseline.

## 8. Execution plan

1. Add a tiny frozen baseline plus omitted-scenario and no-effect negative-control manifests.
2. Implement a deterministic **Rust** transformation layer that emits variant inputs and a manifest; keep perturbation generation separate from evaluator logic.
3. Add exhaustive exact expected-result generation only as an offline development aid; freeze and check in the resulting reference manifests, and include a second independent derivation for the tiny challenge.
4. Add leave-one-scenario-out and withheld-scenario receipts with source, input, mutation-config, reference and output digests.
5. Add metamorphic tests before adding larger random suites.
6. Compare the same tiny corpus with the independent solver under #248. Only then move to high-density correlated-stress cases and timing.

No CI pass or executable implementation is claimed by this protocol document. Exact-head checks are required after the harness exists.

## References

- Shavazipour, B., Kwakkel, J. H., & Miettinen, K. (2021). “Multi-scenario multi-objective robust optimization under deep uncertainty: A posteriori approach.” *Environmental Modelling & Software*, 144, 105134. https://doi.org/10.1016/j.envsoft.2021.105134
- “Let decision-makers direct the search for robust solutions: An interactive framework for multiobjective robust optimization under deep uncertainty.” *Environmental Modelling & Software*, 183 (January 2025), 106233. https://doi.org/10.1016/j.envsoft.2024.106233
- Shavazipour, B., Stewart, T. J. (2026). “A novel multi-stage multi-scenario multi-objective optimisation framework for adaptive robust decision-making under deep uncertainty.” *Omega*, 138, 103405. https://doi.org/10.1016/j.omega.2025.103405
