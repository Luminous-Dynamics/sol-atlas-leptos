# Sol-Atlas robust spatial investigation: scenario sensitivity and finite-support DRO

**Date:** 2026-11-28  
**Status:** synthetic benchmark and research direction; no field performance claim.

## Why nominal EVSI is not enough

Expected value of sample information is defined relative to a decision model: priors, likelihoods, action utilities and acquisition costs determine which observation is worth acquiring. That makes EVSI decision-centric, but it also makes it sensitive to the assumptions used to define the model.

Sol-Atlas should expose sensitivity to hypothesis priors and prevalence; sensor likelihoods, drift and dependence; survey acquisition cost, time and access; conservation, safety and stakeholder utilities; scenario probabilities; and omitted or out-of-distribution conditions. A single expected-value score must never conceal how much the preferred survey depends on uncertain assumptions.

## Robust decision criteria

The benchmark compares four related but distinct criteria:

1. **Nominal expected net EVSI:** optimize under one declared scenario-weight distribution.
2. **Maximin net EVSI:** choose the portfolio with the best worst-case net EVSI among the supplied scenarios.
3. **Minimax regret:** minimize the maximum gap between the chosen portfolio and the scenario-specific best portfolio.
4. **Finite-support total-variation DRO:** optimize the worst expected net EVSI over a set of scenario-weight distributions near the nominal weights.

The first three criteria operate on a finite scenario list. The fourth adds an ambiguity set over the probabilities assigned to those scenarios.

## Synthetic total-variation result

The reference solver defines an ambiguity set (P_r = \{q \in \Delta(S): TV(q,p) \le r\}), where (p) is the nominal scenario-weight vector, (q) is an adversarial reweighting, and (r) is a declared total-variation radius. Under this convention, total variation is half the L1 distance and equals the probability mass moved.

For a fixed candidate portfolio, the finite-support worst-case expectation is computed by moving probability mass from scenarios with the highest net EVSI to the lowest-scoring scenario, bounded by (r). The outer optimization chooses the feasible survey portfolio with the largest worst-case expected net EVSI.

| TV radius | Chosen survey | Worst-case expected net EVSI |
|---:|---|---:|
| 0.0 | Multispectral material scan | 19.6 |
| 0.1 | Multispectral material scan | 17.2 |
| 0.3 | Multispectral material scan | 12.4 |
| 0.5 | Multispectral material scan | 7.6 |
| 0.7 | Archive cross-check | 3.5 |
| 1.0 | Archive cross-check | 3.0 |

The recommendation changes near a TV radius of 0.6632 in this fixture. At radius zero, TV-DRO equals nominal expected net EVSI. At radius one, the ambiguity set covers the full simplex over the finite scenario support and the result equals maximin.

This is a constructed counterexample and sensitivity curve. It does not demonstrate that either survey is superior in the field.

## Connection to current research

The 2025 *Acta Numerica* survey by Kuhn, Shafiee and Wiesemann defines DRO around an ambiguity set of distributions consistent with structural or statistical information, with decisions optimized against the worst distribution in that set. It reviews the statistical and optimization foundations and connections to regularization and adversarial ML ([Kuhn et al., 2025](https://doi.org/10.1017/S0962492924000084)). <Cite refs={["turn470860search0"]}/>

A 2026 sensitivity study emphasizes that the practical value of DRO depends critically on ambiguity-set construction: it must be broad enough to contain plausible data-generating laws while remaining structured and computationally tractable ([Evaluating sensitivity of distributionally robust optimization models, 2026](https://www.sciencedirect.com/science/article/pii/S0305054826002571)). <Cite refs={["turn470860search1"]}/>

Minimax decision rules also have known drawbacks: a small scenario set can exert disproportionate influence, while regret-based criteria have their own independence and scenario-structure concerns. Structured scenarios and explicit use of available probabilistic information are preferable to blindly choosing the most conservative rule ([Anderson & Zachary, 2023](https://doi.org/10.1016/j.ejor.2023.05.030)). <Cite refs={["turn470860search9"]}/>

The Sol-Atlas finite-support TV benchmark is an exact first step, not a replacement for data-driven Wasserstein DRO, moment-based ambiguity sets or divergence-based ambiguity sets. Those approaches require empirical distributions, calibration samples or credible moment bounds and a more substantial optimization solver. The finite-scenario fixture instead makes the policy consequences of probability ambiguity testable in a transparent reference oracle.

## Next research programme

1. Derive scenario envelopes from repeated field campaigns, sensor calibration logs, site/material strata and documented cost variation.
2. Estimate uncertainty in priors and sensor likelihoods using whole-site/campaign holdouts.
3. Add a data-driven Wasserstein ambiguity set over joint observation likelihoods, not just scenario weights.
4. Compare total variation, Wasserstein, phi-divergence and moment-based ambiguity sets with finite-sample confidence guarantees.
5. Add distributionally robust sequential EVSI where both the next survey result and its generating distribution are uncertain.
6. Include multi-objective Pareto analysis across conservation risk, evidence gain, time, cost, privacy and community preferences.
7. Publish the decision frontier and sensitivity to the ambiguity radius; never choose a radius solely because it preserves one preferred survey.

## References

- Kuhn, D., Shafiee, S. & Wiesemann, W. (2025). [Distributionally Robust Optimization](https://doi.org/10.1017/S0962492924000084). *Acta Numerica*, 34, 579–804.
- [Evaluating sensitivity of distributionally robust optimization models](https://www.sciencedirect.com/science/article/pii/S0305054826002571) (2026).
- Anderson, E. & Zachary, S. (2023). [Minimax decision rules for planning under uncertainty: Drawbacks and remedies](https://doi.org/10.1016/j.ejor.2023.05.030). *European Journal of Operational Research*, 311(2), 789–800.
- Fontem, B. & Ji, R. (2026). [Distributionally robust optimization with generalized total variation ambiguity sets](https://doi.org/10.1016/j.ejor.2025.09.024). *European Journal of Operational Research*, 328(3), 894–911.
