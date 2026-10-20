# SA-ECONOMIC-PORTFOLIO-BASELINES-001
## Baseline ablations for staged, budgeted multi-objective development portfolios

**Status:** Independent analysis of a frozen synthetic fixture; not South African cost, ROI, service-impact or job estimates.  
**Parent benchmark:** [SA-ECONOMIC-PORTFOLIO-001](./SA-ECONOMIC-PORTFOLIO-001.md)  
**Source fixture:** [sa-economic-portfolio-v0.1.json](./fixtures/sa-economic-portfolio-v0.1.json)  
**Expected exact output:** [sa-economic-portfolio-v0.1.expected.json](./fixtures/sa-economic-portfolio-v0.1.expected.json)

## 1. Why this comparison exists

A correct exact oracle is necessary, but it does not show whether a portfolio method improves on ordinary planning shortcuts or whether complexity is justified. This document records a baseline/ablation experiment over the existing 8-project fixture so the next implementation work can be compared against explicit failure modes.

The fixture has 256 possible subsets, period-specific budgets and crew limits, prerequisites, a T+W complement, an A+F overlap adjustment, a protected essential-service floor, and nominal plus hot/dry/grid-stress scenarios. Both scenarios are unweighted stress cases. All costs/outcomes are arbitrary synthetic units.

The reproducible exact reference yields 22 feasible portfolios and a four-portfolio scenario-wise Pareto frontier. The existing Rust CLI is intended to reproduce that reference; the analysis below is a separate JavaScript enumeration of the committed input bytes, not a Rust test result.

## 2. Baselines and frozen methods

| ID | Method | Contract for this fixture |
|---|---|---|
| B0 | Incumbent workflow | Unavailable: no actual customer/operator process or authorized case logs are included in this fixture. Do not substitute a made-up incumbent. |
| B1 | Minimum-cost feasible portfolio | Enumerate the eligible set and return the feasible portfolio with minimum total period cost; tie-break by lower total crew, then canonical project ID. Cost is a selector only, not an extra objective axis in v0.1. |
| B2 | Weighted-sum decision policy | For each feasible portfolio, take the minimum value for each of the four maximize objectives across the two declared stress scenarios, normalize each objective across the complete 22-portfolio feasible set using min-max scaling, then score with one of five explicit weight profiles. This is a deliberately transparent scalarization ablation, not a universal social-welfare function. |
| B3 | Naive project-level Pareto prefilter | Before composing portfolios, remove any PASS-gated project robustly dominated on every objective in every scenario by another individual project, without considering prerequisites or interaction effects. Then compose remaining projects with the same dependency/resource/floor constraints. This is intentionally the problematic shortcut under test, not the recommended algorithm. |
| B4 | Exact subset oracle | Enumerate all 256 subsets; apply the declared gate, dependency/stage, budget/crew and every-scenario service-floor constraints; apply the two declared interaction rules exactly once; compute the complete feasible set and scenario-wise frontier. This is the correctness reference for the frozen micro-instance, not the scale-up candidate. |

The B2 normalization bounds across the 22 feasible portfolios are: economic value 3–14, essential-service points 6–11, paid-work-opportunity points 3–13, and ecological-quality points 3–14. These are fixture-specific values and have no real-world units.

Weight profiles (economic, essential service, paid-work opportunity, ecology) are purely illustrative: balanced = (0.25, 0.25, 0.25, 0.25); economic tilt = (0.55, 0.20, 0.15, 0.10); service tilt = (0.10, 0.60, 0.20, 0.10); paid-work tilt = (0.10, 0.15, 0.65, 0.10); ecology tilt = (0.10, 0.10, 0.10, 0.70). These weights are demonstration settings, not stakeholder preferences for South Africa.

## 3. Results

### B1 — Minimum cost

The minimum-cost rule selects **A+J**:
- Total cost: 4 synthetic cost units.
- Total crew: 2 synthetic crew units.
- Worst declared-scenario outcomes (economic, essential service, paid-work opportunity, ecology): (4, 6, 4, 5).

This portfolio passes the fixture's feasibility rules and essential-service floor, but that does not make it preferable under other objectives. On this synthetic corpus, the cost-only rule discards the trade-offs that motivated multi-objective comparison. Cost remains important; the lesson is not to ignore it, but to report it alongside outcomes rather than mistake the cheapest feasible option for a universal winner.

### B2 — Weighted-sum sensitivity

The weighted policy is applied after normalizing worst-case objective values over the entire feasible set. The selected portfolio changes with the declared weights:

| Profile | Selected portfolio | Worst-case objective vector (economic, service, paid-work, ecology) | Total synthetic cost |
|---|---|---:|---:|
| Balanced | I+J+T+W | (14, 11, 10, 7) | 11 |
| Economic tilt | I+J+T+W | (14, 11, 10, 7) | 11 |
| Service tilt | I+J+T+W | (14, 11, 10, 7) | 11 |
| Paid-work tilt | I+J+R+T | (12, 7, 13, 6) | 11 |
| Ecology tilt | F+J+T+W | (8, 11, 7, 14) | 10 |

This is a useful sensitivity demonstration: reasonable-looking declared weight changes select different non-dominated alternatives. It does not say which policy is best. A real use case must set and disclose normalization, stakeholder priorities, hard constraints and sensitivity before viewing outcomes; otherwise both weights and normalization can be tuned to obtain a preferred answer.

### B3 — Individual-project Pareto prefilter

The PASS-gated project-level comparison removes **T** (the telemetry foundation) and **A** (the wastewater assessment) as individually dominated under the naive prefilter. But T is a prerequisite for W and I; its standalone objective vector is not an adequate measure of enabling value. A participates in a non-dominated portfolio with F and other projects because interactions and the bundle's outcome balance matter.

After dropping T and A, only **3 of the original 22 feasible portfolios** survive the same portfolio constraints. The filtered feasible set is F+J, F+J+R, and J+R, with F+J+R as its sole frontier member. **None of the original four exact-frontier portfolios survives.**

This counterexample is the key result of the ablation. Individual-candidate Pareto analysis cannot safely prune prerequisite projects or projects with portfolio-level complementarity unless the pruning proof accounts for dependencies, joint effects and shared constraints. The project-level frontier remains useful for its intended local comparison; using it as a substitute for portfolio composition is the error.

### B4 — Exact reference

- Candidate subsets evaluated: 256.
- Feasible portfolios: 22.
- Infeasible subsets: 234.
- Scenario-wise Pareto frontier: 4 portfolios.
- Exact frontier IDs: A+F+J+R, F+J+T+W, I+J+R+T, I+J+T+W.

The subsequent rejection-accounting increment partitions the 234 rejected subsets into primary reasons under a frozen precedence: 128 non-PASS gate, 48 missing prerequisite, 40 period budget, 0 crew capacity, and 18 protected service-floor rejections. These are mutually exclusive primary dispositions, not every overlapping violation a subset may contain.

## 4. Interpretation / claim ceiling

These tests establish only that the stated baselines behave differently on one synthetic, eight-project micro-instance. They do not establish:
- superiority to a real government, municipal or corporate investment process;
- runtime or memory superiority over MILP / CP-SAT / another operational-research solver;
- robustness to realistic South African outcome distributions;
- a forecast for GDP, jobs, cash savings, water loss, or human welfare;
- an optimal real public portfolio or the legitimacy of any set of societal weights.

We must not claim that an optimizer “beats SOTA” from these results. The comparison demonstrates that three specific shortcuts discard information or expose preference sensitivity; larger, harder and externally validated comparisons are still required.

## 5. Research-grounded path to a credible industry comparison

- The World Bank's Infrastructure Prioritization Framework separates financial-economic and social-environmental dimensions, applies budget constraints, and recommends transparent criteria, weights, sensitivity analysis and third-party review. Our portfolio layer should preserve the multidimensional spirit while explicitly handling prerequisites, complementarity, staff limits, and uncertainty rather than reproducing a composite score: https://ppp.worldbank.org/library/infrastructure-prioritization-framework-ipf
- COCO publishes standardized benchmark suites, including 55 bi-objective functions, and assesses multi-objective optimizer performance using non-dominated solution quality (including hypervolume) and runtime-to-target. This provides useful benchmark discipline, but COCO's numerical black-box benchmark functions are not a direct substitute for discrete, dependency-constrained development portfolios: https://coco-platform.org/data-archive/index.html and https://arxiv.org/abs/1605.01746
- NIST's AI Risk Management Framework organizes risk practice around Govern, Map, Measure and Manage. Use it as a governance/measurement reference, not a claim of compliance based on having a benchmark: https://www.nist.gov/itl/ai-risk-management-framework/nist-ai-rmf-playbook

## 6. Next experimental gates

1. Preserve this fixture as an oracle regression; use it to test each baseline's expected behavior exactly.
2. Add more frozen small cases for overlapping benefits, multiple dependencies, cross-period resource bottlenecks, correlated scenarios, and weight/rank reversals; independently enumerate them.
3. On those cases, compare B1–B3 and the portfolio candidate with identical input/capacity constraints. Report feasibility rate, hard-gate violations, frontier recall, exact objective/constraint disagreement and preference sensitivity.
4. For larger generated instances, compare against an independent MILP / CP-SAT / suitable OR solver with the same time budget and formulation. On oracle-sized cases, report exact agreement and optimality gaps; on larger cases, report solver gaps, feasible coverage, runtime, peak RSS, portfolios explored/pruned, frontier cardinality and hypervolume only with a predeclared normalization/reference point.
5. Add scenario-based minimax regret and adaptive pilot → scale/defer decisions only after uncertainty semantics are explicit. Stress scenarios must not be silently treated as probabilities. Value-of-information should reuse the existing Sol-Atlas EVSI work, not be implemented again.
6. Promote real-data work only after the data owner, rights, source vintage, outcome definition, evaluator, comparison design, service floors, and community/public decision rights are explicit.

**Conclusion:** the immediate improvement is not a magical new score. It is correct portfolio composition, transparent trade-offs, auditability, and a fair baseline harness. Only then should a faster approximate search be asked to scale the portfolio decision.
