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

### Scale (report; do not fabricate target performance)
Benchmark generated cases at 8 / 12 / 16 / 18 projects on a named, pinned runner. Report p50/p95/max, peak memory, subset count, explored/pruned counts, feasible portfolio count and frontier size. The exhaustive oracle should have a fixed maximum problem size and reject anything above the cap. Set a speedup target only after obtaining a reproducible measured baseline.

### Decision-quality / value
Report frontier preservation, selected-portfolios sensitivity to criterion changes, hard-floor robustness and exact optimality gap where an independent solver can produce one. If hypervolume is used for approximate frontiers, disclose objective normalization and reference point and keep it secondary to feasibility/oracle correctness. Do not use an arbitrary weighted sum as a universal societal objective.

### Real-world promotion
Before real project data are admitted, secure permissions and source vintages, define ownership and decision rights, validate each domain model separately, define outcomes and counterfactual method, review privacy/equity/environmental risk and name an independent evaluator. Real cost and benefit estimates must remain separate by actor and must not double-count household, provider and public-sector value.

## 7. Current implementation and next gates

The initial reference implementation is now authored in `tools/sa-economic-portfolio-oracle/` as a standalone Rust crate, with no UI or physical-model dependency. It:
- validates the frozen v0.1 input semantics and rejects over-cap inputs rather than truncating;
- enumerates every subset and applies evidence, dependency/stage, period-budget, crew-cap and every-scenario service-floor constraints;
- applies only the versioned T+W complement and A+F overlap adjustment;
- returns all feasible portfolios and the complete scenario-wise frontier in canonical order;
- emits a deterministic JSON receipt that includes the exact input-byte SHA-256; and
- has unit tests for the 22-feasible/four-frontier known answer, row-order invariance, UNKNOWN-gate exclusion, unknown prerequisites, over-cap rejection, and interaction accounting.

The authored Rust tests have **not** been represented as passing until the dedicated exact-head workflow completes. A separate evaluator used during fixture preparation parsed the committed JSON, checked the input against its schema, exhaustively enumerated all 256 subsets, and matched the frozen 22-portfolios / four-frontier manifest. This independent fixture check does not compile or execute the Rust crate.

Next gates:
1. Require a successful exact-head [SA economic portfolio oracle workflow](../../../.github/workflows/sa-economic-portfolio-oracle.yml); queued, missing or stale checks are not passes.
2. Fix any compiler, formatting or test failures on a new exact head and rerun the workflow.
3. Add mutation tests for budget overruns, each hard floor, interaction-rule revisions and corrupted expected manifests.
4. Compare a transparent greedy, predeclared weighted-sum and individual-project-Pareto-then-budget baseline on the same corpus; freeze more cases before reporting scale.
5. Add generated synthetic instances and compare larger cases against an independently configured MILP / CP-SAT / appropriate OR solver with matching constraints and budgets.
6. Connect real Sol-Atlas service evidence only when source permissions, source-time semantics and domain-adapter gates pass.

## 8. Industry / research anchors

- World Bank, [Infrastructure Prioritization Framework](https://ppp.worldbank.org/library/infrastructure-prioritization-framework-ipf): financial-economic and social-environmental dimensions, budget constraints, transparent criteria/weights and sensitivity, with third-party review.
- OECD, [Evidence-informed infrastructure decision-making, Government at a Glance 2025](https://www.oecd.org/en/publications/government-at-a-glance-2025_0efd0bcd-en/full-report/evidence-informed-infrastructure-decision-making_ac3e2c57.html): cost-benefit analysis is common; multi-criteria methods help preserve long-term and non-monetized dimensions.
- OECD / ITF, [Strategic Infrastructure Planning](https://www.oecd.org/en/publications/strategic-infrastructure-planning_4142787d-en.html): long asset lives, uncertainty and need for integrated planning.
- [Robust multiobjective portfolio optimization using scenario-based minimax regret](https://www.sciencedirect.com/science/article/pii/S0377221717302631): a method family for examining regret under future scenarios; it is not a ready-made SOTA comparison for this synthetic public-development case.

**Claim ceiling:** the benchmark is a synthetic portfolio oracle. It does not forecast GDP/jobs, authorize spending, establish causal impact, decide legitimate public priorities, or replace professional/community/statutory review.
