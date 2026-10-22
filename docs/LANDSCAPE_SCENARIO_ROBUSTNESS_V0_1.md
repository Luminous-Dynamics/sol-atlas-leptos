# Landscape Scenario Robustness Benchmark v0.1

## Purpose

Most landscape comparisons report a score for one assumed future. This benchmark makes the scenario set explicit and asks two different robustness questions:

1. **Maximin utility:** Which alternative has the highest score in its worst declared scenario?
2. **Minimax regret:** Which alternative limits its largest shortfall relative to the best alternative in each scenario?

Those policies need not select the same design. The tool reports both and deliberately does not fabricate one universal ranking. Selection uses raw validated utility values; rounding is only for report presentation, so display precision cannot manufacture a tie and let the alternative ID decide the winner.

## Method contract

Each alternative has one normalized `utility_points` outcome in `[0, 100]` for every declared scenario. A utility-method specification is pinned by SHA-256 and checked against its exact bytes. The upstream project must define how its physical, ecological, accessibility, maintenance and cost measurements were transformed into utility points. This benchmark does not infer value weights, convert unlike units, or choose a scoring preference.

Scenarios carry **no probabilities**. The arithmetic average over the listed scenarios is explicitly named an *unweighted descriptive mean*, not expected utility. A scenario panel is an assumption set, not a climate forecast. Separate non-compensatory critical constraints encode requirements such as a water-service floor or accessible-route continuity. Exactly one pass/fail outcome is required for every scenario × alternative × constraint tuple. A high utility score cannot offset any failed hard constraint. Raw maximin and minimax-regret winners remain visible for analysis, but gated winners are computed only among alternatives that satisfy every critical constraint in every declared scenario and meet the worst-case utility threshold; if none qualify, both gated winners are null. The evaluator parses and validates the pinned utility-method document itself: it requires the declared 0–100 utility scale, higher-is-better direction, explicit normalization, fail-closed missing-data policy, no scenario probability weights, and the v0.1 claim ceiling. Exact hash identity and semantic conformance are separate gates. Missing outcome pairs, duplicate IDs, invalid utility ranges, synthetic-to-observed relabelling, undeclared method fields, and method-digest mismatch fail closed.

## Frozen synthetic example

Four scenarios are included: typical growing season, intense rainfall, compound heat/drought, and maintenance-capacity reduction. The fixture uses a six-by-six style test scale (0–100 utility points); its values are synthetic and not derived from field or model data.

| Alternative | Worst-case utility | Unweighted mean | Maximum regret | Hard-constraint violations | Deployable under v0.1 gate? |
|---|---:|---:|---:|---:|---|
| Drought-resilient planting (`strategy-a`) | 70 | 76.25 | 10 | 0 | Yes |
| High baseline performance (`strategy-b`) | 40 | 59.25 | 35 | 3 | No |
| Balanced resilient planting (`strategy-c`) | 72 | 73.75 | 17 | 1 | No |

**Two gates are deliberately separated.** On score alone, maximin selects `strategy-c` (worst-case 72) and minimax regret selects `strategy-a` (maximum regret 10). But `strategy-c` violates the accessible-route continuity constraint under maintenance shock, so it is not a deployable recommendation. After hard constraints and the utility threshold are applied, both feasible decision rules select `strategy-a`. The high-utility unconstrained winner is not allowed to buy its way out of a critical requirement.

A second diagnostic removes each scenario once and recomputes both decision rules without reweighting the remaining cases. For this panel, maximin preserves its full-set winner in 3/4 omissions (75%), while minimax regret preserves its full-set winner in 4/4 (100%); omitting the extreme-rain case switches maximin from strategy-c to strategy-a. This is sensitivity to the declared scenario panel, not statistical confidence or evidence that one future is more likely. The CLI report binds exact fixture and utility-method bytes. Unit tests verify hand-calculated metrics, both raw and gated decision rules, complete hard-constraint matrix checks, the no-feasible-alternative null outcome, leave-one-out outcomes, raw-score selection when displayed values round to a tie, deterministic serialization, missing and duplicate scenario/alternative/constraint tuples, out-of-range scores, observed-data relabelling, utility-method digest tampering, and rejection of digest-correct but semantically contradictory methods (such as a different score scale or probability-weighting rule).

## Benchmarks to pursue

- **Correctness:** 100% known-answer agreement; zero missing/duplicate pairs accepted; zero synthetic fixture relabelling accepted; exact digest binding; deterministic replay.
- **Robustness:** no critical constraint breach or threshold failure reported as a gated recommendation; 100% exact matrix completeness; if no alternative passes all hard constraints, return no recommendation rather than select the least-bad violation.
- **Sensitivity:** v0.1 measures winner stability under leave-one-scenario-out deletion. On larger held-out scenario panels, add family-level omission and bounded perturbation tests; report policy instability rather than hiding it in one aggregate. Leave-one-out fractions are descriptive diagnostics, not probabilities of correctness.
- **Generalization:** build preregistered panels from held-out sites, seasons and independently sourced climate/maintenance conditions. Split by site and time, not randomly by rows.
- **Calibration:** only assign scenario probabilities after there is a defensible calibration protocol; then evaluate expected utility separately from this unweighted robustness contract.
- **Decision-usefulness:** include lifecycle cost, water demand, canopy establishment/survival, thermal exposure, stormwater performance, biodiversity, maintenance workload, accessibility and distributional equity where relevant, with compatible, documented utility transformations.

## Industry posture and next step

The Landscape Architecture Foundation's Landscape Performance Series supports project-specific, defensible environmental, social and economic metrics and provides over 200 case-study briefs and a guide covering more than 100 metrics across 33 benefit categories: https://www.lafoundation.org/what-we-do/research/landscape-performance and https://www.landscapeperformance.org/guide-to-evaluate-performance.

A 2026 systematic review of urban green-space scenario analysis reviewed 157 publications and identified gaps in systematic evaluation and stakeholder engagement: https://pmc.ncbi.nlm.nih.gov/articles/PMC13293698/. That supports treating the scenario set, evaluation pathway and stakeholder-defined constraints as first-class artifacts. It does not validate this synthetic example as a landscape predictor.

The benchmark is a small, reproducible risk-analysis primitive. It does **not** claim to outperform landscape architecture software, hydrology/urban-climate models or ecological methods on measured outcomes. A defensible SOTA claim needs independent real outcomes and a preregistered comparison against each task-appropriate reference.

## Run

```sh
cargo test --locked --manifest-path tools/landscape-scenario-robustness/Cargo.toml
cargo run --locked --manifest-path tools/landscape-scenario-robustness/Cargo.toml
```

The default output is `PASS_SYNTHETIC_SCENARIO_ROBUSTNESS_ONLY`. It establishes contract conformance only; it is not a field-performance result.
