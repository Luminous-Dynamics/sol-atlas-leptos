# Home Freedom robust-decision benchmark v0.1

**Study:** HOME-FREEDOM-ROBUST-DECISION-001  
**Parent:** [Home Freedom Compiler #800](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/800) / [compiler PR #801](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/801)  
**Maturity:** SYNTHETIC_BENCHMARK_CONTRACT  
**Execution:** NOT_SIMULATED

## Purpose

The existing 100-case corpus asks whether a candidate may proceed to further screening. It does not establish which candidate remains preferable when climate, cost, maintenance, material supply, service availability or resident priorities change.

This increment adds a decision-analysis layer **after hard-gate qualification and before any claim of buildability**. It consumes candidate outcome intervals from owner-qualified sources; it does not generate geometry, estimate costs, simulate building physics, set resident priorities or authorize a build.

## Paradigm shift

Move from a nominal-year / single-point winner to:

1. **Robust feasibility before preference:** a candidate is robust-feasible only when every required hard constraint is PASS in every mandatory scenario. Any FAIL blocks it; any UNKNOWN makes it conditional.
2. **Uncertainty as data:** every objective is an interval with an explicit unit and basis. An unknown interval is not zero and cannot participate in dominance.
3. **Pareto evidence, not a universal score:** compare candidates only where the outcome intervals prove a strict ordering. Preserve trade-offs and unresolved overlaps.
4. **Scenario dependence is visible:** report a Pareto frontier for each scenario and a more conservative robust frontier across all scenarios.
5. **No hidden scenario advantage:** every candidate is evaluated against the same declared scenario identities and exogenous trace identities.
6. **Human preference remains human:** the benchmark does not infer weights or collapse agency, safety, affordability, comfort, water, privacy and repairability into a single objective.

## Input contract

A benchmark instance freezes:

- one synthetic resident brief and design revision;
- at least two mandatory scenarios, each with an opaque exogenous-trace identity;
- the hard constraints that must hold in every scenario;
- at least two objective definitions, each with a fixed unit and direction;
- one outcome record per candidate per scenario;
- exactly one gate result per required constraint and one interval record per objective in every outcome record.

The v0.1 interval basis is only SYNTHETIC_BOUND or UNKNOWN. Later measured, modeled or statistical intervals must declare their own semantics; a confidence interval, prediction interval, measurement uncertainty and engineering bound are not interchangeable.

The trace identities in the synthetic fixture are labels, **not hashes or source-data receipts**. A future empirical run must bind the exact source bytes, revision, time basis, units, rights, location, processing, model and uncertainty before a trace is admitted.

## Decision semantics

### Feasibility disposition

- ROBUST_FEASIBLE: all required hard gates are PASS in every scenario.
- CONDITIONAL: no required hard gate is FAIL, but at least one required gate is UNKNOWN.
- BLOCKED: at least one required hard gate is FAIL.

Objective completeness is independent of feasibility. A robust-feasible candidate with any UNKNOWN objective interval is OBJECTIVES_INCOMPLETE and is excluded from dominance/frontier calculations. It is not downgraded to a physical failure, nor silently assigned a value.

### Interval-guaranteed dominance

For a minimization objective, A is provably no worse than B only when:

`upper(A) <= lower(B)`

For a maximization objective:

`lower(A) >= upper(B)`

A dominates B only when this holds for **every objective** in the declared scenario and is strict for at least one objective. Overlapping intervals imply UNRESOLVED_ORDER, not equality and not a tie-break invitation.

- A scenario frontier contains comparable robust-feasible candidates not interval-dominated in that scenario.
- A robust frontier contains comparable robust-feasible candidates not dominated by another single candidate in every mandatory scenario.
- A different candidate dominating B in each scenario is not combined into a fictitious single robust dominator.
- No universal scalar score, expected utility or probability is emitted.

The rule is intentionally conservative: it may leave many alternatives incomparable. That is preferable to manufacturing precision from overlapping uncertainty bounds.

### Conservative regret bounds

For a minimization objective, an outer bound on candidate i's regret in scenario s is:

`upper(i,s) - min_j lower(j,s)`

For a maximization objective:

`max_j upper(j,s) - lower(i,s)`

This bound assumes no known cross-candidate dependence and may be loose. v0.1 reports no regret scalar and does not aggregate different units. Joint uncertainty/covariance models require separately admitted evidence and an explicit model contract.

## Synthetic corpus and negative controls

The bundled fixture has six symbolic candidates across three synthetic scenarios:

- one robust-feasible courtyard option;
- one reference option interval-dominated by the courtyard option;
- one low-capital / higher-discomfort trade-off option;
- one option with an UNKNOWN hard gate;
- one option with a failed hard gate;
- one robust-feasible option with an UNKNOWN objective interval.

The fixture is deliberately artificial and carries no site, weather, quote, instrument, simulation or physical outcome data.

The Rust qualifier checks:
- unique and complete candidate/scenario/constraint/objective identities;
- identical scenario identities across alternatives;
- gate and objective coverage;
- finite, ordered intervals and exact unit/direction agreement;
- fail-closed UNKNOWN and FAIL semantics;
- interval-dominance and frontier construction;
- permutation invariance;
- negative controls for missing gates, duplicated identities and inverted intervals.

## Industry and SOTA comparison

Do not compare this decision layer to a full professional design workflow as if it replaces one.

| Comparator | Strong incumbent role | Matched comparison lane |
|---|---|---|
| Architect-led practice | contextual judgement, client negotiation, professional responsibility | same brief/site/evidence, review maturity, human hours, iterations and fee boundary |
| TestFit | site feasibility, configuration and development analysis | only the published/geographically supported site and feasibility tasks |
| Finch3D | rule-bound layout generation and plan constraints | same plan-generation task, brief, library, evidence and human intervention |
| Autodesk Forma | early site design and supported daylight workflows | site/daylight lane only, with current release and geographic entitlement pinned |
| EnergyPlus / OpenStudio | specialist building energy and supported thermal modelling | version-pinned physics cases and independent reference results |
| BOPTEST | repeatable building-control benchmarking | only supported emulator cases and equivalent signals/control authority |
| IFC / IDS and semantic tools | information exchange and requirements conformance | round-trip field preservation and semantic loss, not thermal performance |
| Local QS, trades and suppliers | cost, availability, labour, sequencing and construction reality | local dated quotations, labour/productivity evidence and actual revision-specific BOM |

The differentiator to test is **decision integrity across evidence owners**: no false robust-feasible result, uncertainty-aware Pareto options, revision-bound provenance, scenario transparency and resident-owned trade-offs. This is not yet a claim of superiority over any named incumbent.

## Proposed benchmark ladder and targets

| Stage | Evaluation | Proposed gate |
|---|---|---|
| P0 synthetic | frozen known-answer cases and hostile mutations | 100% exact disposition; zero false ROBUST_FEASIBLE; 100% positive-control preservation |
| P0 properties | interval endpoint enumeration and metamorphic tests | 100% agreement with independent interval-dominance oracle |
| P1 model | qualified EnergyPlus / other specialist cases, held-out scenarios | no out-of-domain robust pass; report error and uncertainty calibration by climate and archetype |
| P2 matched workflow | architect-led and tool baselines, fixed human/compute budgets | at least 20% lower median time to a decision-ready Pareto set as a stretch hypothesis, with no hard-gate or quality regression |
| P3 empirical | independent metrology and controlled unoccupied test cell | uncertainty intervals achieve their declared coverage; no claims beyond instrument/model applicability |
| P4 in-use | consented, resident-governed post-occupancy evidence | resident-specific outcomes, privacy and opt-out preserved; no average hides an individual hard-floor failure |

Additional reporting:
- false robust-feasible rate and one-sided confidence bound by failure family;
- objective interval coverage and width, calibration drift and out-of-domain abstention;
- robust-frontier size, scenario-frontier changes and pairwise interval reversals;
- conservative regret bounds by objective, unit and scenario;
- number of resident constraints preserved and number of unresolved trade-offs;
- p50/p95 time to first decision-ready option, human intervention count, compute, memory and energy;
- revision invalidation/requalification completeness;
- worst-stratum performance by household functional profile and scenario family.

A zero observed false-pass count is not proof of zero population risk. Publish sample sizes and confidence bounds; generated synthetic trials are not a substitute for independent physical or field evidence.

## Execution and promotion gates

1. P0: this synthetic comparator and exact-head software tests.
2. P1: bind source-qualified climate/service/material/cost inputs and specialist-model outputs without changing their owners.
3. P2: freeze matched incumbent tasks and budgets before running any superiority comparison.
4. P3: validate model uncertainty on held-out reference cases and a qualified unoccupied test cell.
5. P4: only separately consented, resident-governed in-use evaluation.

A scenario, model, design, BOM, site, authority or resident-brief revision invalidates every dependent comparison. Recompute rather than carry forward stale ranks.

## Claim ceiling

This artifact establishes only a synthetic decision-analysis contract and its software oracle. It does not establish a real house's feasibility, cost, comfort, energy or water performance; calibrate a model; qualify climate data, sensors or materials; compare a live incumbent; or prove SOTA superiority.
