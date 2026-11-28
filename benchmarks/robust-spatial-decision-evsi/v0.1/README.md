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

The benchmark includes radii 0, 0.1, 0.3, 0.5, 0.7 and 1.0. The two boundary cases are tested: radius 0 equals nominal expected weighting; radius 1 equals maximin over the finite scenario set.

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
