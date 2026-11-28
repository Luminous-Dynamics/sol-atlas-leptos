# Sol-Atlas robust spatial decision-EVSI benchmark v0.1

**Status:** synthetic reference oracle; no field performance or SOTA advantage claim.

## Research question

A point-estimate EVSI planner can be mathematically correct and still be brittle when prior probabilities, sensor likelihoods, acquisition costs or conservation utilities are uncertain. A planning system should expose which assumptions drive its chosen observation and whether the choice survives plausible scenario shifts.

This benchmark compares three different decision criteria over the same frozen scenario set:

- **Nominal expected net EVSI:** maximize the scenario-weighted average.
- **Maximin net EVSI:** maximize the worst-case net EVSI across scenarios.
- **Minimax regret:** minimize the largest gap to the best feasible portfolio in any scenario.
- **Total-variation DRO:** maximize worst-case expected net EVSI over all reweightings of the nominal scenario probabilities within a declared total-variation radius. Radius 0 recovers nominal expected EVSI; radius 1 spans the full probability simplex and recovers maximin.

The benchmark always includes the no-observation option. Survey effort must fit the same hard budget in every scenario; survey costs and information quality may vary by scenario.

## Synthetic counterexample

There are two hypotheses, two actions, two candidate surveys and three scenarios with weights 0.7 / 0.1 / 0.2.

| Policy | Selected survey | Weighted mean net EVSI | Worst-case net EVSI | Maximum regret |
|---|---|---:|---:|---:|
| Nominal expected value | Multispectral material scan | 19.6 | -1.0 | 29.0 |
| Maximin | Archive cross-check | 9.0 | 3.0 | 15.0 |
| Minimax regret | Archive cross-check | 9.0 | 3.0 | 15.0 |

The multispectral survey is much more valuable under ordinary conditions, but its likelihood deteriorates sharply in the sensor-shift scenario. The archive survey has lower nominal value but remains useful in every scenario. Nominal optimization and robust criteria therefore choose different observations.


### Total-variation robustness curve

| TV radius | Selected policy | Worst-case expected net EVSI |
|---:|---|---:|
| 0.0 | Multispectral material scan | 19.6 |
| 0.1 | Multispectral material scan | 17.2 |
| 0.3 | Multispectral material scan | 12.4 |
| 0.5 | Multispectral material scan | 7.6 |
| 0.7 | Archive cross-check | 3.5 |
| 1.0 | Archive cross-check | 3.0 |

The policy changes near a TV radius of 0.6632 for this fixture. This is a sensitivity result for the declared three-scenario problem, not a universal robustness threshold.

### Lower-tail CVaR risk frontier

| Tail mass τ | Selected survey | Lower-tail CVaR net EVSI |
|---:|---|---:|
| 0.1 | Archive cross-check | 3.0 |
| 0.2 | Multispectral material scan | 8.5 |
| 0.5 | Multispectral material scan | 16.2 |
| 1.0 | Multispectral material scan | 19.6 |

The policy changes between tail mass τ=0.1 and τ=0.2 in this fixture. The tail mass is a risk preference, not a learned physical parameter; it should be pre-registered with decision-makers.


This is intentionally constructed. It is not an empirical claim about multispectral scanning or archival research.

## Protocol

1. Define scenarios before evaluating the planner. Record why each prior, likelihood, cost and utility range is plausible.
2. Include prior shift, sensor drift, material/occlusion changes, acquisition-cost changes and stakeholder utility sensitivity.
3. Include a no-observation action and hard budget/permission/safety constraints.
4. Compare nominal expected value, maximin net EVSI, minimax regret and human expert plans.
5. Publish scenario-specific net EVSI and decision changes rather than only a scalar robust score.
6. Use independent data to estimate plausible likelihood ranges; do not invent uncertainty sets after seeing the preferred policy.
7. Report sensitivity to scenario weights and include a worst-case scenario analysis. Robustness is only as credible as the scenario set.

## Finite-support DRO extension

The reference implementation uses a finite set of named scenarios and a total-variation ball around their nominal weights. For each candidate portfolio, it moves probability mass from the scenarios with the highest net EVSI to the lowest-scoring scenario, bounded by the declared TV radius. This exactly solves the finite-support weight-ambiguity subproblem; it does **not** construct a Wasserstein ball over raw sensor measurements, continuous covariates or unknown likelihood functions. Its guarantees apply only to the declared scenario support and radius.

The benchmark includes TV radii 0, 0.1, 0.3, 0.5, 0.7 and 1.0, plus lower-tail probability masses τ=0.1, 0.2, 0.5 and 1.0. The two boundary cases are tested: radius 0 equals nominal expected weighting; radius 1 equals maximin over the finite scenario set. The optimum worst-case value is tested as non-increasing with larger radii.

## Interpreting lower-tail CVaR

For a fixed portfolio, the lower-tail reward statistic sorts scenario net EVSI from lowest to highest and averages the worst τ probability mass, splitting a scenario's probability mass at the tail boundary when needed. In conventional loss-CVaR notation this is equivalent to `-CVaR_{1-τ}(-R)`, where `R` is net EVSI. The policy maximizes this lower-tail reward. This produces a risk-preference frontier between the worst case and the full nominal expectation. The tail-mass parameter τ must be chosen and pre-registered with decision-makers; it is not a property discovered by the sensor model.

## Limits

- Two binary hypotheses and a finite action set.
- Independent binary survey outcomes within each scenario.
- Exhaustive portfolio enumeration for at most eight surveys.
- Scenario probabilities are discrete weights; the TV ambiguity set only reweights the supplied finite scenario support and does not prove robustness to omitted scenarios or continuous distribution shift.
- No spatially explicit acquisition route, travel cost surface, correlated sensor model or adaptive sequential policy here; those are qualified in separate benchmarks.
- Synthetic utilities and likelihoods are not operational estimates.

## Run

```sh
cargo test --manifest-path reference/Cargo.toml
cargo run --manifest-path reference/Cargo.toml -- fixtures/robust-decision-case.json
```


## Spatial uncertainty baselines update (2026-11)

The spatial calibration comparison should include **Localized Spatial Conformal Prediction (LSCP)**, presented at ICML 2026, alongside ordinary split conformal, GeoCP, GeoSIMCP, bootstrap and provider-native predictive samples. LSCP combines local quantile regression with conformal calibration; its stated guarantees depend on spatial exchangeability and, for asymptotic conditional coverage, stationarity and spatial mixing. Do not transfer those guarantees to arbitrary spatial shifts.

A 2026 GeoSIMCP study explicitly combines geographic distance and feature-space similarity to address nearby but process-dissimilar locations. This reinforces a key challenge case for robust EVSI: geographic proximity alone must not imply likelihood transportability.

Benchmark policy comparison must keep three axes separate:
- probability-model calibration and applicability;
- ambiguity-set robustness and policy regret;
- observation value under the decision utility.

A robust policy score cannot repair a misspecified likelihood or an unsupported ambiguity set. Record the provenance and empirical basis for each uncertainty radius, likelihood shift and acquisition-cost range.

References:
- Jiang & Xie (2026), [Spatial Conformal Inference through Localized Quantile Regression](https://proceedings.mlr.press/v306/jiang26at.html), ICML/PMLR 306.
- Luo (2026), [Quantifying Uncertainty in Spatial Prediction for Nonstationary Spatial Processes](https://doi.org/10.1080/24694452.2026.2639721), GeoSIMCP.
- [OGC JSON-FG 1.0.0](https://www.ogc.org/standards/json-fg/), released 2026-05-21.
