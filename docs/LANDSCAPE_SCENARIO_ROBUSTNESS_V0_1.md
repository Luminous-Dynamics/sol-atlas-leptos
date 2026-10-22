# Landscape Scenario Robustness Benchmark v0.1

## Purpose

Most landscape comparisons report a score for one assumed future. This benchmark makes the scenario set explicit and asks two different robustness questions:

1. **Maximin utility:** Which alternative has the highest score in its worst declared scenario?
2. **Minimax regret:** Which alternative limits its largest shortfall relative to the best alternative in each scenario?

Those policies need not select the same design. The tool reports both and deliberately does not fabricate one universal ranking. Selection uses raw validated utility values; rounding is only for report presentation, so display precision cannot manufacture a tie and let the alternative ID decide the winner.

## Method contract

Each alternative has one normalized `utility_points` outcome in `[0, 100]` for every declared scenario. A utility-method specification is pinned by SHA-256 and checked against its exact bytes. The upstream project must define how its physical, ecological, accessibility, maintenance and cost measurements were transformed into utility points. This benchmark does not infer value weights, convert unlike units, or choose a scoring preference.

Scenarios carry **no probabilities**. The arithmetic average over the listed scenarios is explicitly named an *unweighted descriptive mean*, not expected utility. A scenario panel is an assumption set, not a climate forecast. Missing outcome pairs, duplicate IDs, invalid utility ranges, synthetic-to-observed relabelling and method-digest mismatch fail closed.

## Frozen synthetic example

Four scenarios are included: typical growing season, intense rainfall, compound heat/drought, and maintenance-capacity reduction. The fixture uses a six-by-six style test scale (0–100 utility points); its values are synthetic and not derived from field or model data.

| Alternative | Worst-case utility | Unweighted mean | Maximum regret | Qualifies at threshold 70? |
|---|---:|---:|---:|---|
| Drought-resilient planting (`strategy-a`) | 70 | 76.25 | 10 | Yes |
| High baseline performance (`strategy-b`) | 40 | 59.25 | 35 | No |
| Balanced resilient planting (`strategy-c`) | 72 | 73.75 | 17 | Yes |

**The important result is policy disagreement:** maximin selects `strategy-c` because its worst-case utility is 72, while minimax regret selects `strategy-a` because its maximum scenario regret is 10. Neither is a universal winner. They encode different risk preferences and remain contingent on the scenario set and the declared utility transformation.

A second diagnostic removes each scenario once and recomputes both decision rules without reweighting the remaining cases. For this panel, maximin preserves its full-set winner in 3/4 omissions (75%), while minimax regret preserves its full-set winner in 4/4 (100%); omitting the extreme-rain case switches maximin from strategy-c to strategy-a. This is sensitivity to the declared scenario panel, not statistical confidence or evidence that one future is more likely. The CLI report binds the exact fixture bytes and utility-method bytes. Unit tests verify the hand-calculated metrics, both decision rules, leave-one-out outcomes, raw-score selection when displayed values round to a tie, deterministic serialization, missing and duplicate scenario/alternative pairs, out-of-range scores, observed-data relabelling, and utility-method digest tampering.

## Benchmarks to pursue

- **Correctness:** 100% known-answer agreement; zero missing/duplicate pairs accepted; zero synthetic fixture relabelling accepted; exact digest binding; deterministic replay.
- **Robustness:** no scenario-policy breach reported as robustly qualified; zero missing scenario/alternative evaluations in a report marked pass.
- **Sensitivity:** on future fixtures, measure whether maximin/minimax-regret choices change when stress scenarios are added, removed, or perturbed. Report policy instability; do not hide it in one aggregate.
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
