# SA-WORKFORCE-RECOVERY-001 — Stability-aware plan recovery under disruption v0.1

**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/287  
**Builds on:** portfolio oracle #285 / PR #286, outcome/additionality #276 / PR #277 and evidence-chain conformance #282 / PR #283.  
**Status:** deterministic synthetic benchmark; no claim about real worker availability or service impact.

## Paradigm shift: the nominally optimal plan is not the resilient plan

A portfolio that is feasible at time t may become infeasible when a tool goes offline, a funded action loses authorization, shared supervision tightens, a demand floor rises or the chosen bundle becomes unavailable. Full re-optimization can recover service but churn previously communicated work; freezing the original plan preserves stability while potentially failing essential service. The correct comparison makes both costs explicit.

This benchmark evaluates each immutable baseline plan under a declared disruption, then exhaustively enumerates the feasible recovery alternatives. It returns a Pareto frontier across service benefit, resource use, cash, staff effort, delay, worker burden, added bundles and removed bundles. Hard requirements remain hard: uncertainty cannot become permission, and plan stability cannot justify violating an essential-service floor.

## What each scenario contains

- A baseline set of selected bundle IDs.
- A post-disruption candidate catalog with explicit availability, eligibility, prerequisite, funding, authorization and work-quality state.
- Post-disruption resource caps and essential-service floors.
- A frozen baseline assessment reference.
- An exact recovery frontier reference (or explicit infeasibility code) for small instances.

A recovery bundle is selectable only when it is available, permitted, prerequisite-satisfied, funded, authorized and viable. One bundle per case is permitted and declared dependencies must be included. Resource capacities and any worker-burden cap apply to the full recovery portfolio.

Plan additions and removals are set differences from the frozen baseline IDs—not changes in row ordering. An unavailable or no-longer-authorized baseline bundle cannot deliver benefit merely because it remains on the plan.

## Research basis and industry comparison

Mattia, Rossi & Smriglio's June 2026 *Omega* paper, *Robust shift scheduling with discretionary rest breaks*, uses a two-stage robust model and treats employee discretion as a source of uncertainty while preserving service levels. This benchmark transfers the evaluation insight, not their algorithm or results: https://doi.org/10.1016/j.omega.2025.103500

Böðvarsdóttir & Stidsen's review of 52 multi-objective personnel-rostering papers describes competing employer, employee and customer objectives and the common over-reliance on weighted-sum scalarization: https://link.springer.com/article/10.1007/s10951-025-00845-0

Commercial workforce schedulers already provide constrained assignment/rescheduling capabilities. A fair comparator must receive the same baseline, disruption events, permitted candidates, resource caps, essential-service floors and outcome definitions. The complement proposed here is an auditable evaluation of stability, recovery feasibility and essential-service preservation—not an unsubstantiated claim of beating a vendor.

## Metrics

- Baseline floor/constraint status under each disruption, including explicit unavailable/unauthorized IDs.
- Recourse feasibility and exact Pareto frontier completeness for tiny cases.
- Essential floor violations in emitted recourse portfolios (target zero).
- Resource and worker-burden violations (target zero).
- Added bundle IDs and removed bundle IDs as independent counts.
- Recovery cost vector: resources, cash, staff effort, delay and worker burden.
- Recovery regret against the predeclared exact scenario frontier; report component-wise regret or a declared scalarization, never an undisclosed composite.
- Sensitivity to tool/resource loss, availability changes, funding/authorization changes, demand-floor changes and disruption combinations.
- Oracle subset count, runtime and hard cut-off status.
- Determinism under row order and IDs; monotonic sanity checks for added capacities and removal of a forbidden candidate.

## Frozen baselines

- R0: frozen baseline, no recourse.
- R1: complete exact re-optimization on each small disruption scenario.
- R2 (future): limited-change recourse that minimizes bundle additions/removals subject to all hard constraints.
- R3 (future): pre-positioned contingency portfolio selected before the disruption is known.

The v0.1 Rust verifier establishes R0 assessment and R1 exact recovery-frontier conformance. It does not yet optimize a single cross-disruption policy or claim that a strategy is robust to unknown real events.

## Acceptance gates

- Independent hand enumerations and implementation agree on all frozen tiny-case recovery frontiers.
- Every emitted recovery plan meets all essential floors; infeasible cases emit no plan and name blockers.
- Zero forbidden, unavailable, unknown, unfunded, unauthorized, prerequisite-failing or nonviable bundles selected.
- Zero resource-capacity/worker-burden violations.
- Plan additions/removals are computed from set differences.
- Same pinned inputs under row permutation produce identical canonical receipts.
- Fixture/schema digests and the synthetic-only claim ceiling appear in the receipt.

## Claim ceiling

The candidate catalogs and disruptions are synthetic test conditions, not estimates of real absence rates or service resilience. A passing fixture does not establish production robustness, job creation, operational impact or SOTA superiority. Real evaluation requires owner-authorized disruption records, matched comparator behavior and independently verified outcomes.
