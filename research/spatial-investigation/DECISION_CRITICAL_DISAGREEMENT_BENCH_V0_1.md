# Decision-Critical Spatial Disagreement Benchmark v0.1

**Status:** research specification; synthetic reference not yet implemented  
**Related issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/992

## Thesis

Spatial alternatives should be compared by the decisions they imply, not only by pixelwise distance or ensemble variance. Two models can agree on average error while disagreeing over a critical corridor, catchment boundary, excavation volume or intervention choice. Conversely, large pointwise differences may be irrelevant when all admissible models imply the same action.

The benchmark asks: **where does model disagreement change a decision, how costly could that be, and which admissible observation is most likely to resolve it?**

## Mathematical object

Let admissible spatial hypotheses be (H_1,\ldots,H_m), each defining a predictive distribution over a common qualified support (S). Let (a\in\mathcal A) be an action and (L(a,H_j;S,u)) a loss functional under stakeholder utility (u).

For each support element (s\in S), define the model-specific preferred action:

\[
a_j^*(s)=\arg\min_{a\in\mathcal A} \mathbb E[L(a,H_j;s,u)].
\]

The decision-disagreement support is:

\[
D=\{s\in S: \exists i,j,\ a_i^*(s)\ne a_j^*(s)\}.
\]

This is not the same as a region of high prediction variance. Report both, separately.

At aggregate level, compare model-specific action functionals and regret:
\[
R(a,H_j)=L(a,H_j;S,u)-\min_{a'}L(a',H_j;S,u).
\]

If model probabilities are defensible, calculate expected discrimination value for candidate observation (Y_q):
\[
\operatorname{EVSI}(q)=\min_a\mathbb E[L(a,H)]-
\mathbb E_{Y_q}\left[\min_a\mathbb E[L(a,H)\mid Y_q]\right].
\]
If probabilities are not defensible, use the robust policy envelope and regret frontier from #987 rather than inventing weights.

## Required output

- common-support and non-overlap report;
- alternative/hypothesis IDs and source/rights/derivation receipts;
- pointwise prediction disagreement (diagnostic only);
- action-disagreement geometry and support measure;
- per-model optimal action and decision-functional value;
- expected and worst-case regret;
- sensitivity to utility, costs, priors, likelihoods and boundary conditions;
- candidate discriminating observations with correlation-aware net EVSI;
- withheld/unknown support and worst-site/stakeholder breakdown;
- reproducible input/configuration/output digests.

## Benchmark matrix

| Track | Baseline | Primary metrics |
|---|---|---|
| Prediction disagreement | RMSE, MAE, ensemble variance | Difference surfaces; not a decision score |
| Action disagreement | Per-model independent action maps | Disagreement support / total support |
| Decision regret | Nominal single-model action | Mean, worst-case and stakeholder regret |
| Robustness | Maximin, minimax regret, TV-DRO, CVaR | Policy-switch frontier and regret |
| Investigation | Random, highest variance, highest entropy, nominal EVSI | Net EVSI, regret reduction per acquisition cost |
| Dependence | Independent likelihood approximation | False-discrimination rate under correlated observations |
| Calibration | Spatial conformal / provider-native posterior checks | Coverage and hypothesis calibration on independent sites |
| Governance | Unrestricted synthetic projection | Zero unauthorized disclosure; derived-field taint |

## Frozen challenge cases

1. Same predicted scalar field, different network topology and action.
2. Large prediction disagreement, identical optimal action.
3. A narrow critical corridor with low area but high consequence.
4. Competing boundaries with partial support overlap.
5. Correlated repeat sensor observation that adds almost no discrimination.
6. A low-cost archive observation that dominates a new survey under robust regret.
7. Utility/cost ambiguity that changes the preferred action.
8. Hidden upstream source that taints the public disagreement map.
9. Unknown support that must remain bounded rather than assumed safe.
10. Input/model ordering invariance.

## Baselines and controls

Compose rather than duplicate:
- #985 for exact joint-observation and sequential EVSI;
- #987 for robust policy criteria and ambiguity sensitivity;
- #990/#991 for support-aware spatial functionals;
- #986 for calibration and metrology;
- #988 for likelihood applicability and transport;
- #983 and Mycelix for rights, evidence references and derived-data lineage.

Do not add another posterior engine, covariance estimator, geostatistical solver, spatial topology engine or general-purpose provenance graph.

## Evaluation protocol

1. Freeze support, actions, utilities, alternatives and acquisition costs before running.
2. Split by site/campaign and seal independent reference truth.
3. Include adversarial model alternatives and synthetic known-answer cases.
4. Compare on identical support and matched acquisition budgets.
5. Report every site, stakeholder and critical stratum; never hide worst-case failures in a global mean.
6. Record all policy switches as priors, likelihoods, costs, utilities and support assumptions vary.
7. Keep synthetic, replay, field and independently replicated results in separate qualification tiers.

## Proposed target gates

- Exact agreement with synthetic action-disagreement and regret oracles.
- Zero false agreement in frozen topology/action negative controls.
- Zero false disagreement when all admissible alternatives imply the same action.
- Zero silent support extrapolation.
- Zero false independence in correlated-observation challenges.
- Zero unauthorized disclosures.
- At least 20% lower decision regret or acquisition effort at matched regret versus the strongest preregistered baseline, on independent sites. This is a stretch target, not an achieved result.

## Claim ceiling

This document is a research specification only. It does not implement a spatial comparison engine, establish model probabilities, prove causal completeness, validate physical measurements, or demonstrate superiority over industry systems. Those claims require executable fixtures, real adapter/provider integration, independent reference data and matched external baselines.
