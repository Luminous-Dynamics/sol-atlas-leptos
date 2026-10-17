# Probability-Free Robust Regret for Investigation Planning

Status: implementation proposal v0.1. The Rust kernel is authored in `sol-atlas-core::robust_regret`; exact-head hosted validation is still required.

## Paradigm shift: investigate to reduce regret, not merely to improve a score

A site-ranking pipeline commonly asks which candidate has the highest estimated score, then treats missing measurements as a data-cleaning problem. That can hide the more important decision: **which next observation could change the decision enough to justify its cost?** Bayesian expected value of sample information (EVSI) is useful when priors and observation likelihoods are defensible. It can be fragile when those probabilities are not defensible yet.

This increment adds a complementary *probability-free* view. It treats each enumerated scenario as plausible, uses lower/upper loss intervals, and compares the minimax-regret bound before and after a proposed investigation partitions scenarios into possible outcomes. It invents no scenario probabilities. This is not a replacement for EVSI; the disagreement between Bayesian EVSI and robust regret is itself a useful warning signal.

## Contract and calculation

Input contains a finite set of decisions, a finite set of plausible scenarios, a complete matrix of non-negative loss intervals, and candidate investigation actions. Each action provides an explicit outcome label for every scenario and a cost in the same declared units as the losses.

For decision \(d\) in scenario \(s\), the conservative regret bound is:

\[
R^+(d,s)=\max\left(0,\;L^+_{d,s}-\min_j L^-_{j,s}\right)
\]

The baseline decision minimizes its worst-case regret bound across all scenarios. After an investigation, scenarios are grouped by predicted observation; the system chooses a minimax-regret decision independently within each outcome group. The maximum regret bound across those groups is the post-investigation bound.

- **Regret-bound reduction** = baseline bound − post-investigation bound.
- **Net regret-bound reduction** = regret-bound reduction − declared investigation cost.
- The output is **worthwhile**, **break-even**, or **not worthwhile** under this robust criterion.

This is a conservative comparison, not an expected-value or guaranteed causal effect. Interval endpoints must be meaningful and commensurable; dependence among loss intervals is not modeled. Cost and loss units must match. If no defensible common unit exists, keep costs external and do not interpret net reduction as a utility.

## Fail-closed validation

The kernel rejects empty decision/scenario sets, unknown IDs, duplicate loss cells, missing cells in the decision × scenario matrix, inverted/negative/non-finite intervals, duplicate actions, missing or empty observation outcomes, and negative/non-finite costs. BTree-backed identities and strict comparisons give deterministic tie-breaking by lexical ID. Actions are ranked by net regret-bound reduction with a stable action-ID tie-break.

No probabilities are inferred from scenario counts. Duplicating a scenario is therefore not a probability update; callers must model scenario identity intentionally.

## Synthetic reference cases

The pinned JSONL fixture has four cases:

1. Perfectly discriminating observation: bound 80 → 0, cost 10, net reduction 70.
2. Non-discriminating inspection: bound stays 80, cost 5, net reduction −5.
3. Interval losses with targeted observation: bound 80 → 10, cost 20, net reduction 50.
4. Zero-cost but uninformative observation: bound stays 80; break-even by definition.

All IDs, scenarios, losses, and costs are synthetic arithmetic examples. They are not estimates for a real parcel, hazard, water system, community or project.

## Benchmark targets

Promotion targets for the first production-quality release:

| Dimension | Target | Why it matters |
|---|---:|---|
| Reference fixtures | 100% exact expected-output agreement | Keep hand-checkable semantics stable |
| Invalid-input controls | 100% rejected for declared malformed cases | Never silently complete an incomplete matrix |
| Observation refinement | 0 cases where a finer partition raises pre-cost regret bound | Information must not worsen the decision option set |
| Determinism | Byte-identical serialized report for repeated identical input | Receipts and independent review need repeatability |
| Permutation invariance | Same result after reordering input records | Input ordering must not change recommendations |
| Independent oracle | 100% agreement on at least 100 small enumerated problems | Catch shared implementation assumptions |
| Scale | Report median/p95 for 10,000 decision-scenario cells and 1,000 actions | Establish a real baseline before setting an SLA |
| Evidence discipline | 0 probability or currency claims inferred from labels alone | Avoid fabricated precision |

Before using the ranking operationally, add property-based generation or exhaustive tiny-state enumeration, a second independent reference implementation, and a deterministic scale benchmark. Report runtime, allocation/memory, fixture manifest hash and toolchain version. Do not optimize for a latency SLA before measuring representative matrices.

## How this fits the Built Environment Foundry

- The evidence-first suitability kernel in [PR #129](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/129) keeps hard constraints ahead of preference scores.
- The Bayesian EVSI prototype in [PR #133](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/133) handles cases where priors/likelihoods are supportable.
- Coverage-aware overlay receipts in [PR #140](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/140) preserve the difference between absent and not observed.
- The robust regret result can prioritize *which unresolved question to measure next*, but it does not grant permission to bypass hard safety, environmental, legal, land-rights or community-consent gates.

A useful next integration would map open evidence gaps to explicit investigation actions, then compare the Bayesian EVSI ranking with the robust-regret ranking. Disagreement should trigger assumption review rather than be averaged into a single score.

## Claim ceiling

This kernel evaluates a finite declared scenario/loss model. It does not discover scenarios, estimate their probabilities, verify source evidence, validate an investigation's diagnostic accuracy, predict a real site outcome, conduct a legal/environmental review, or certify a decision. Synthetic benchmark success is necessary but not sufficient for operational use.
