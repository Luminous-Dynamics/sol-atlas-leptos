# SA-ECONOMIC-PORTFOLIO-001
## A bounded, robust, multi-objective portfolio benchmark for South African development choices

**Status:** Synthetic v0.1 benchmark contract; no real project costs, impact or investment advice.  
**Parent:** [Sol-Atlas issue #214](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/214)  
**Related portfolio assessment:** [SA-ECONOMIC-LEVERAGE-001 / PR #208](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/208)

## 1. The gap this fills

There is already work for comparing *individual candidates*: interval-aware Pareto frontier ([PR #132](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/132)), stakeholder-weight sensitivity ([PR #137](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/137)), expected value of information ([PR #133](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/133)), and a compatible interval-robust frontier effort in [Symtropy RDO-002](https://github.com/Luminous-Dynamics/symtropy/issues/1874).

The missing layer is **portfolio composition**. Real budgets, staff, dependencies and annual delivery capacity constrain combinations of projects, not isolated projects. Benefits may be complementary or overlap. A project that looks mediocre alone may unlock a valuable follow-on project; selecting both may consume the same scarce crew or double-count the same benefit. Scenario shocks can affect several projects at once.

This benchmark therefore tests selection among sets of staged projects while reusing existing Pareto, sensitivity, value-of-information and resource-allocation logic. It is not a replacement for those algorithms or for domain-specific physics/engineering.

## 2. Core decision protocol

1. Validate all identities, units, time periods, scenario coverage, prerequisites, evidence/gate status and budget/resource limits.
2. Reject portfolios containing any project with a mandatory gate other than `PASS`, any missing prerequisite, any temporal-order violation, any annual budget or crew-capacity excess, or any protected essential-service floor failure in a required scenario.
3. Apply only explicit, versioned interaction rules. A complementary gain is counted only where defined; overlapping benefits require a documented adjustment or are not jointly credited.
4. Evaluate each remaining portfolio on all declared objectives in every scenario. The v0.1 fixture treats scenarios as an unweighted set of stress cases, not as probabilities.
5. Preserve every candidate that is not dominated on the full scenario-by-objective vector. Candidate A dominates B only if A is no worse for every objective in every scenario, and is strictly better in at least one component. This v0.1 fixture uses exact synthetic points; later interval inputs must use the previously qualified conservative interval-dominance contract.
6. Apply a stakeholder choice or deployment policy only after the feasible frontier is visible. Make the selected policy, criterion version, weights/constraints and sensitivity explicit. There is no automatic universal winner.

The project set is small by design: exhaustive enumeration is the correctness oracle. Over-cap inputs must fail explicitly rather than silently return a partial frontier.

## 3. Frozen synthetic test case

Input: `sa-economic-portfolio-v0.1.json`. Expected summary: `sa-economic-portfolio-v0.1.expected.json`.

All monetary/cost and outcome numbers in the fixture are **arbitrary synthetic units**, not rand, job counts, engineering forecasts or South African estimates.

- Two periods (P1 and P2), annual budget caps of 5 and 6 cost units, and crew/workforce caps of 3 units in each period.
- Essential-service baseline of 3 synthetic points; protected floor of at least 6 points in every declared scenario.
- Two equally *unweighted* declared cases: `nominal` and `hot_dry_grid_stress`. They are stress cases, not calibrated probabilities.
- Eight project IDs: telemetry foundation (T), water repair (W), wastewater assessment (A), industrial continuity (I), local repair workcell (R), job-access analysis (J), flood/ecology mitigation (F), and a stale-evidence decoy (X).
- W requires T and is staged after T within P1; I requires T and occurs in P2.
- Pair T+W has an explicit complementary effect; A+F has an explicit overlap adjustment to the ecology objective.
- X has an `UNKNOWN` evidence gate and implausibly large synthetic rewards. It must never enter a feasible portfolio. It is a hostile negative control, not a candidate recommendation.

Objectives are maximized separately: `economic_value_u`, `essential_service_points`, `paid_work_opportunity_points` and `ecological_quality_points`. Despite the names, all units are synthetic.

### Independent oracle result

Exhaustive enumeration of all (2^8 = 256) candidate subsets, followed by evidence / dependency / capacity / service-floor filters, yields **22 feasible portfolios**. The known-answer feasible set is listed explicitly in the expected manifest.

The robust scenario-wise Pareto frontier contains exactly four portfolios:

| Portfolio IDs | P1 cost / crew | P2 cost / crew | Nominal objective vector (economic, service, paid-work, ecology) | Stress vector |
|---|---:|---:|---|---|
| I + J + R + T | 5 / 3 | 6 / 3 | (17, 7, 13, 6) | (12, 7, 13, 6) |
| I + J + T + W | 5 / 3 | 6 / 3 | (21, 12, 10, 7) | (14, 11, 10, 7) |
| A + F + J + R | 5 / 3 | 5 / 2 | (10, 9, 10, 13) | (7, 8, 10, 13) |
| F + J + T + W | 5 / 3 | 5 / 2 | (11, 12, 7, 14) | (8, 11, 7, 14) |

These values are **internal synthetic oracle answers only**. The four alternatives trade off economic value, service, paid-work-opportunity proxy and ecological index. The result is not a ranking, a selected national portfolio or an indication that one policy is socially preferable.

## 4. What the hostile case should catch

- **Gate leakage:** X has high apparent rewards but an `UNKNOWN` gate; it must be excluded.
- **Broken dependency:** W/I cannot be selected without T, and the prerequisite must be earlier than the dependent stage.
- **Budget/staff overload:** every period's combined costs and crew requirements must fit its limits.
- **Average hides harm:** require the essential-service floor in *every* declared case, not merely a nominal or averaged outcome.
- **Benefit double count:** the A+F ecology overlap adjustment must apply exactly once when both are chosen.
- **Missing synergy:** T+W receives only its explicit jointly declared benefit; do not silently infer synergies from textual descriptions.
- **Scenario averaging:** do not use scenario count as a probability and do not allow strong nominal performance to erase a stress-case failure.
- **False completeness:** return all feasible portfolios/frontier members or an explicit error; never truncate and report success.
- **Order dependence:** permutation of projects and scenario rows must preserve the canonical feasible set and frontier.
- **False evidence elevation:** a digest proves the input bytes used, not the publisher's authority or the truth of synthetic or source claims.

## 5. Baselines and how to beat them

| ID | Baseline | What it reveals |
|---|---|---|
| B0 | Documented incumbent project selection process, when available | Real comparison; otherwise explicitly unavailable |
| B1 | Greedy cost-only / benefit-cost heuristic | Whether a more complex method improves over transparent simple rules |
| B2 | Predeclared weighted-sum ranking | Exposes how results depend on subjective weights |
| B3 | Rank projects individually using Pareto analysis, then filter by budget | Exposes complementarity, dependency and shared-capacity mistakes |
| B4 | Exhaustive enumeration of every subset on the tiny fixture | Gold oracle for exact feasibility and frontier correctness |
| Candidate | Dependency-closed, multi-period, resource-constrained portfolio evaluation; scenario-wise Pareto frontier | Test the value of portfolio-level reasoning |

A larger-instance comparison should use an independently configured MILP / CP-SAT / appropriate OR solver with identical constraints, scenarios, objective directions and time budget, plus exact oracle cases at tractable sizes. Record tool versions, solver gap, run time, memory, portfolios explored/pruned, feasible count and frontier cardinality.

Do not claim “better than SOTA” from a synthetic fixture alone. It qualifies finite contract correctness. The candidate beats a named reference only for a matched task and corpus, with exact-head evidence and disclosed uncertainty.

## 6. Benchmarks and progression gates

### Correctness (v0.1)
- 100% exact agreement with the known feasible set (22 portfolios) and four-member frontier.
- 0 outputs containing X or any portfolio with a failed/unknown gate.
- 0 annual cost/crew-cap violations, missing prerequisite/stage violations or service-floor violations.
- 100% deterministic result under input permutation.
- 100% explicit interaction accounting and complete scenario evaluation.

### Scale (current limitation; no scale claim yet)
The current Rust oracle is a **frozen v0.1 fixture evaluator**: it requires the eight declared project IDs, the two named scenarios, and P1/P2 budget periods. `MAX_EXACT_PROJECTS = 20` is only an exhaustive-enumeration safety cap; it does not make the current v0.1 contract a general 12–20-project instance format. Do not report generated 12 / 16 / 18-project measurements through this fixture-specific contract.

The next scale-up is tracked in [SA-PORTFOLIO-SCALE-001 (#230)](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/230): freeze a versioned generalized v0.2 input contract and deterministic generator, preserve v0.1 as the immutable known-answer oracle, and add a candidate frontier search. Only after that gate should the benchmark compare generated sizes (initially 8 / 10 / 12 / 14 / 16 / 18, with exact cases only where feasible) on a named pinned runner. Report p50/p95/max, peak RSS, subsets/nodes explored and pruned, feasible count, frontier size, exact frontier agreement and constraint violations. Establish speed targets only after a reproducible baseline exists.

### Decision-quality / value
Report frontier preservation, selected-portfolios sensitivity to criterion changes, hard-floor robustness and exact optimality gap where an independent solver can produce one. If hypervolume is used for approximate frontiers, disclose objective normalization and reference point and keep it secondary to feasibility/oracle correctness. Do not use an arbitrary weighted sum as a universal societal objective.

### Real-world promotion
Before real project data are admitted, secure permissions and source vintages, define ownership and decision rights, validate each domain model separately, define outcomes and counterfactual method, review privacy/equity/environmental risk and name an independent evaluator. Real cost and benefit estimates must remain separate by actor and must not double-count household, provider and public-sector value.

## 7. Current implementation and next gates

The initial reference implementation is authored in `tools/sa-economic-portfolio-oracle/` as a standalone Rust crate, with no UI or physical-model dependency. It is intentionally fixture-specific at v0.1, **not yet a general portfolio-planning or scalable-search engine**. The generalized v0.2 contract and candidate search are tracked in [issue #230](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/230). It:
- validates the frozen v0.1 input semantics and rejects over-cap inputs rather than truncating;
- enumerates every subset and applies evidence, dependency/stage, period-budget, crew-cap and every-scenario service-floor constraints;
- applies only the versioned T+W complement and A+F overlap adjustment;
- returns all feasible portfolios and the complete scenario-wise frontier in canonical order;
- emits a deterministic JSON receipt that includes the exact input-byte SHA-256 and a primary rejection reason for every infeasible subset; and
- has unit tests for the 22-feasible/four-frontier known answer, row-order invariance, UNKNOWN-gate exclusion, unknown prerequisites, over-cap rejection, interaction accounting, budget and crew mutations, strict service-floor rejection, and corrupted known-answer manifests.

### Auditability increment

The receipt now partitions every rejected subset into exactly one **primary** reason using a frozen precedence: non-PASS gate → missing prerequisite → period budget exceeded → crew-capacity exceeded → protected service-floor violation. For the pinned corpus, 128 / 48 / 40 / 0 / 18 subsets fall into those categories, totaling all 234 rejected subsets; 22 are feasible. The reason totals reconcile to all 256 candidate subsets. These are mutually exclusive primary reasons, not counts of every constraint violation a subset might have.

Mutation tests also create a crew-limited case with non-binding cost budgets, a tightened P1 budget, and a floor above all candidate outcomes. They assert the appropriate rejection accounting and verify that returned portfolios remain within the changed constraints. A corrupted known-answer manifest must be rejected.

The exact-head Rust workflow for the original v0.1 and the baseline-ablation increment completed successfully: the base audit slice passed 11 tests; the baseline test slice passed 14 tests (formatting, Clippy with warnings denied, oracle tests and receipt generation). The added generalized v0.2 / frontier-search tests are new and remain unqualified until their own exact-head workflow completes. A separate JavaScript evaluator independently checked the v0.2 input/schema and produced the pinned 1,024-subset exact manifest; this does not compile or execute the new Rust search.

### Generalized v0.2 and exact frontier-search candidate

The next implementation slice is on [SA-PORTFOLIO-SCALE-001 (#230)](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/230). It keeps the v0.1 input and expected output immutable, while adding:
- a general v0.2 synthetic contract accepting unique project/scenario IDs and ordered positive `Pn` periods, with the same typed objective/floor/resource semantics;
- a generated ten-project / three-period fixture with 1,024 total subsets, an independently enumerated exact expected feasible set and frontier;
- `evaluate_frontier_search`, which uses dependency-aware recursive search and prunes only branches proven infeasible from gate, prerequisite, cost or crew constraints. It retains an exact multi-scenario Pareto frontier and checks that leaves plus pruned subtrees account for the full (2^n) subset universe;
- counters for visited nodes, evaluated leaves, feasible portfolios, protected-floor failures, pruned subsets by reason and frontier comparisons; and
- Rust regression tests for exact v0.1/v0.2 frontier equality plus a 24-project synthetic case with 16 UNKNOWN-gate decoys; these new tests remain pending exact-head qualification.

This search has exponential worst-case behavior; it is not yet a demonstrated SOTA or runtime win. The documented 63-project limit is a mask representability bound, not a practical throughput guarantee.

Next gates:
1. Run the exact-head workflow for the generalized input and frontier search; queued/missing/stale checks do not pass.
2. Fix any compile, formatting, Clippy or behavioral failures without weakening acceptance conditions.
3. Compare the exact-search and pruned-search frontiers on small frozen cases and ensure every subset is accounted for.
4. Benchmark larger generated instances against an independently configured MILP / CP-SAT or suitable operations-research solver with the same constraints and compute budget; report exactness, p50/p95/max latency, peak memory, nodes visited/pruned, feasible count, frontier cardinality and solver gaps.
5. Connect real Sol-Atlas service evidence only after permissions, domain-model gates, real owners and independent validation are in place.

## 8. Industry / research anchors

- World Bank, [Infrastructure Prioritization Framework](https://ppp.worldbank.org/library/infrastructure-prioritization-framework-ipf): financial-economic and social-environmental dimensions, budget constraints, transparent criteria/weights and sensitivity, with third-party review.
- OECD, [Evidence-informed infrastructure decision-making, Government at a Glance 2025](https://www.oecd.org/en/publications/government-at-a-glance-2025_0efd0bcd-en/full-report/evidence-informed-infrastructure-decision-making_ac3e2c57.html): cost-benefit analysis is common; multi-criteria methods help preserve long-term and non-monetized dimensions.
- OECD / ITF, [Strategic Infrastructure Planning](https://www.oecd.org/en/publications/strategic-infrastructure-planning_4142787d-en.html): long asset lives, uncertainty and need for integrated planning.
- [Robust multiobjective portfolio optimization using scenario-based minimax regret](https://www.sciencedirect.com/science/article/pii/S0377221717302631): a method family for examining regret under future scenarios; it is not a ready-made SOTA comparison for this synthetic public-development case.

**Claim ceiling:** the benchmark is a synthetic portfolio oracle. It does not forecast GDP/jobs, authorize spending, establish causal impact, decide legitimate public priorities, or replace professional/community/statutory review.
