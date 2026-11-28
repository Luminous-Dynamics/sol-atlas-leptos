# Spatial Likelihood Qualification v0.1

**Status:** research specification; not an implementation or physical qualification  
**Issue:** [#988 — SPATIAL-LIKELIHOOD-QUAL-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/988)  
**Scope:** campaign-derived observation likelihoods and their uncertainty as inputs to existing spatial investigation / EVSI owners.

## 1. Research question

Given finite, heterogeneous calibration campaigns, how do we estimate a defensible predictive distribution for the next spatial observation—including parameter uncertainty, cross-sensor dependence, site/instrument effects and model discrepancy—and determine when it is sufficiently qualified to influence a decision?

The benchmark fills the integration gap between:

- [#771 — Earth observability](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/771): sensor observability envelopes, POD/PFA, calibration validity, blind truth, clustered detection scoring and sensor-to-EVSI bridge;
- [#785 — physical forward-model validation](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/785): analytical, numerical, inter-code and physical qualification;
- [#845 — physical-change invariance](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/845): measurement-system, processing, support and sampling perturbations;
- [#986 — spatial uncertainty and calibration](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/986): predictive metrics, spatial intervals and metrology receipt template;
- [#985 — correlated and sequential EVSI](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/985): exact joint likelihood and adaptive stopping;
- [#987 — robust spatial decision-EVSI](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/987): maximin, regret, finite-support TV-DRO and lower-tail CVaR;
- [#134 — spatial sampling design](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/134): location/depth/sequence-dependent investigation plans.

The aim is to produce calibrated, versioned observation-model evidence for those existing owners—not to create another sensor registry, forward solver, Bayesian inference library, conformal implementation or campaign planner.

## 2. Paradigm shift: from fixed likelihoods to qualified predictive distributions

The current narrow pipeline is:

`state hypothesis -> fixed supplied likelihood -> EVSI`

The target pipeline is:

`calibration evidence -> posterior predictive joint likelihood + discrepancy/shift envelope -> decision-conditioned EVSI -> observe / calibrate / cross-check / stop / abstain`

Keep six uncertainty sources distinct:

1. **Physical-state uncertainty:** what is true about the target or site.
2. **Measurement noise:** irreducible variability conditional on a fixed measurement process.
3. **Likelihood-parameter uncertainty:** finite-data uncertainty in POD, PFA, bias, variance, covariance and related parameters.
4. **Model discrepancy:** systematic mismatch between the observation model and physical observations.
5. **Distribution shift:** changed geography, sensor, configuration, time, environment, source or processing.
6. **Structural uncertainty:** alternative likelihood families or omitted mechanisms.

Do not collapse them into one confidence value. A good score on one axis cannot compensate for an unqualified domain, invalid source rights or missing physical reference.

### 2.1 Posterior-predictive contract

For physical hypothesis (H), acquisition action (a), context (z), likelihood parameters (	heta) and model family (M), the admitted calibration corpus (D_c) should support a predictive distribution of the form:

[
p(ymid H,a,z,D_c)
=
sum_M int p(ymid H,a,z,	heta,M)
,p(	heta,Mmid D_c),d	heta
]

This is a joint predictive distribution when several observations are acquired. It is not permission to multiply marginal likelihoods. Shared calibration, site, operator, mount, clock, power, processing or environmental causes must be represented or explicitly left unresolved.

For a new observation, the posterior update must preserve the joint belief over physical hypotheses and uncertain likelihood parameters. Computing EVSI with an uncertain likelihood silently frozen at its posterior mean is an invalid plug-in shortcut unless the approximation is independently justified.

### 2.2 Two kinds of acquisition value

A candidate acquisition can have two distinct values:

- **Immediate decision value:** can its result change the current decision?
- **Calibration value:** can a reference/calibration observation improve future likelihood estimates and reduce regret over a preregistered set of future investigations?

Eligible action classes should include target observation, calibration/reference measurement, independent modality cross-check, repeat observation, and stop/decide/abstain. Calibration value must be scoped to a declared deployment horizon or future decision set; it must not be an unlimited speculative benefit.

Sol-Atlas supplies qualified likelihood evidence and action descriptors. Symthaea owns policy selection and campaign orchestration.

## 3. Ownership boundaries

| Owner | Responsibility |
|---|---|
| Sol-Atlas | Spatial support; target/sensor/configuration identity; campaign manifests; likelihood qualification and transport diagnostics; adapter to existing EVSI contracts |
| Symtropy | Modality-specific physical models, forward simulation, instrument response, physical experiments |
| Symthaea | Experiment/campaign policy, multi-step decision reasoning, policy evaluation |
| Mycelix | Rights, actor/purpose authority, source custody, independence/dependency receipts and disclosure controls |
| Independent partners | Reference truth, access, metrology review and field-study approval |

Do not create a parallel sensor registry, source-authority service, inference engine or EVSI planner. Keep modality-specific physics with the relevant owner.

## 4. Calibration evidence and model identity

Every likelihood qualification package must bind:

- target class/hypothesis and outcome alphabet;
- sensor/instrument identity, configuration, firmware and processing revision;
- physical support, region, depth, temporal window, coordinate frame, CRS, units and spatial transform;
- relevant environmental/acquisition covariates;
- calibration/reference method, validity interval, instrument state and uncertainty budget;
- independent truth/reference identity and its uncertainty;
- site, campaign, operator, instrument and acquisition-cluster identities;
- raw and derived artifact digests where rights permit;
- train/calibration/validation/test role and split-manifest digest;
- model family, parameter posterior/estimate, prior, fitting procedure, diagnostics and software revision;
- declared likelihood dependencies and common-cause/fault-domain assumptions;
- supported domain, exclusions, unresolved discrepancy and claim ceiling.

A calibration receipt does not prove that all relevant site, instrument or environment conditions are represented. Missing or stale fields remain explicit and block unsupported promotion.

## 5. Matched likelihood baselines

### 5.1 Binary detection

Compare, on identical campaign splits:

1. Fixed supplied likelihood / oracle-known sensor model (upper-bound reference only).
2. Plug-in empirical or maximum-likelihood POD/PFA.
3. Hierarchical logistic or generalized mixed-effects model with site/instrument/campaign effects.
4. Bayesian hierarchical posterior-predictive likelihood, integrating parameter uncertainty.
5. Calibrated post-hoc baseline (for example isotonic calibration), fitted only on an authorized calibration split.
6. Empirical joint outcome table when the number of independent campaigns supports it.

### 5.2 Multivariate/continuous observations

Add covariance-aware and copula/factor models only when their assumptions fit the observation support and data. Compare full joint predictions against a diagonal/independence ablation. Preserve modality-specific outcome representations; do not force unlike measurements into one shared likelihood.

### 5.3 Spatial uncertainty baselines

Where the task predicts spatial fields or intervals, compare ordinary conformal, GeoCP, GeoSIMCP and Localized Spatial Conformal Prediction (LSCP) on their stated assumptions. These are uncertainty/coverage comparators; they do not automatically qualify sensor POD/PFA or physical calibration.

## 6. Model checking and uncertainty qualification

Report separately:

- proper predictive scores: log score and Brier for binary/categorical outcomes; CRPS and suitable multivariate scores (energy / variogram) for continuous outcomes;
- calibration intercept/slope and reliability curves; ECE is secondary, not a stand-alone gate;
- POD and false-alarm estimates with site/campaign/instrument-cluster uncertainty;
- posterior-predictive checks for residual structure, spatial/temporal autocorrelation, heavy tails and dependence;
- prior SBC for broad model/algorithm checks and posterior SBC conditional on the admitted calibration data;
- identifiability and posterior sensitivity to priors/model-family choice;
- discrepancy diagnostics separate from measurement-noise and parameter uncertainty;
- localized coverage and sharpness by site, sensor, environmental regime, target class and support.

Aggregate calibration is insufficient if a material site, sensor, environment or target stratum fails. Use abstention, a wider supported envelope or a new calibration acquisition rather than silently carrying over nominal accuracy.

## 7. Data-derived ambiguity sets for robust EVSI

Feed #987 with ambiguity sets grounded in calibration evidence:

- posterior credible regions or posterior-predictive model mixtures;
- cluster-bootstrap confidence regions respecting site/campaign/instrument dependence;
- total-variation or moment sets over empirical scenario support;
- Wasserstein or other distributional sets only when the ground metric, scaling, support and radius are defensible from observations;
- alternative-model/discrepancy sets for structural uncertainty.

Keep uncertainty within a model, uncertainty over parameters, ambiguity over the generating distribution and omitted-scenario risk distinct. Increasing an ambiguity radius does not protect against an omitted mechanism. Publish construction, calibration evidence, support, finite-sample limitations and sensitivity frontiers.

## 8. Split and leakage protocol

Random point-level splits are insufficient. Freeze and report independent:

- leave-site-out;
- leave-campaign-out;
- leave-instrument/configuration-out;
- forward temporal holdout;
- source/provider or processing-version holdout;
- environment/regime holdout;
- spatial-block holdout for prediction tasks.

Where deployment targets both a new site and a new instrument, hold out both together. Keep repeated measurements and derived products from one acquisition/campaign on one side. Calibration labels, held-out truth, fault schedules and scenario identifiers remain evaluator-only.

## 9. Frozen adversarial corpus

At minimum, include:

1. Perfectly known likelihood with exact posterior-predictive EVSI.
2. Sparse calibration data causing plug-in overconfidence.
3. Site-specific sensitivity/intercept shift.
4. Instrument replacement with changed detection and false-alarm behavior.
5. Time-varying drift and calibration expiry.
6. Environmental regime transition despite geographic proximity.
7. Sensors sharing power, mount, operator, timing, calibration or processing.
8. Cross-modality observations with correlated residuals.
9. An independence model with incorrect marginals or joint probabilities.
10. High-information observation that cannot change the action.
11. Calibration action with low immediate value but measurable future value.
12. Target observation that also updates a shared likelihood parameter.
13. Label error or uncertain reference truth.
14. Selection-biased calibration data omitting hard-to-observe targets.
15. Below-detection-limit, no-coverage, occluded and out-of-domain outcomes.
16. Shifted model whose predictive checks correctly force abstention.
17. Ambiguity set that is too narrow and one that is unjustifiably broad.
18. Rights-expired calibration record used for a new purpose.
19. Stale calibration receipt that remains syntactically valid.
20. No-observation control where a costly calibration is correctly rejected.

Include positive controls: an informative eligible measurement must be selected when it changes a declared decision and has positive qualified net value. An always-abstaining or always-buying policy cannot pass.

## 10. Metrics and qualification targets

Report separate results for:

- marginal and joint predictive scores;
- calibration and posterior-SBC diagnostics;
- POD/false-alarm estimates and cluster-level uncertainty;
- conditional/worst-stratum coverage and interval width;
- model-family/prior sensitivity;
- likelihood transport degradation by held-out shift;
- policy-switch frequency and sensitivity frontier;
- realized decision regret, regret distribution and lower-tail outcomes;
- net EVSI after travel, crew, processing, delay and opportunity costs;
- immediate decision value versus future calibration value;
- acquisition effort at matched realized regret;
- abstention frequency/reasons;
- runtime, memory and replay cost.

Hard gates:

- 100% rejection of invalid probability normalization and incompatible outcome supports.
- 0 false independence claims in fixtures containing shared dependence.
- 0 calibration/validation/test leakage across declared grouping keys.
- 0 in-domain promotion of unsupported sensor/site/configuration regimes.
- 100% deterministic known-answer agreement for bounded exact-reference cases.
- No aggregate score conceals a failed safety, rights or mandatory-evidence gate.

Stretch target: at least 20% lower out-of-sample expected regret at equal eligible acquisition budget, or at least 20% lower acquisition effort at matched regret, against the strongest qualified baseline. This is a hypothesis, not an achieved result. Freeze effect size and sample size after independent baseline variance and power/precision analysis.

## 11. Research comparators

Reproduce methods on their declared tasks and assumptions. Paper metrics are not universal thresholds.

- Lou, Luo & Meng (2025), GeoCP, DOI: [10.1080/24694452.2025.2516091](https://doi.org/10.1080/24694452.2025.2516091).
- Luo (2026), GeoSIMCP / nonstationary spatial uncertainty, DOI: [10.1080/24694452.2026.2639721](https://doi.org/10.1080/24694452.2026.2639721).
- Jiang & Xie (ICML 2026), Localized Spatial Conformal Prediction, [PMLR 306](https://proceedings.mlr.press/v306/jiang26at.html).
- Doerksen & Kerner (2026), [EarthShift](https://earthshift.github.io/) (reuse #771's integration; this is EO OOD evaluation, not sensor-POD evaluation).
- Li & Parlikad (2026), Value-Based Active Sensing, DOI: [10.1109/TSMC.2026.3745855](https://doi.org/10.1109/TSMC.2026.3745855).
- Säilynoja et al. (2026), Posterior SBC, DOI: [10.1007/s11222-026-10825-9](https://doi.org/10.1007/s11222-026-10825-9).

## 12. Execution ladder and claim ceiling

| Tier | Qualification | Claim ceiling |
|---|---|---|
| L0 | Contract/schema and independent hand-computable cases | Structural only |
| L1 | Exact finite categorical/binary models and posterior-predictive integration | Mathematical/reference correctness |
| L2 | Synthetic site/instrument/campaign hierarchies, common causes and controlled shifts | Synthetic benchmark correctness |
| L3 | Published/public-data reproduction with frozen rights and splits | Reproduction on named dataset |
| L4 | Retrospective physical campaigns with independent reference truth | Bounded retrospective validation |
| L5 | Prospective, permissioned multi-site campaign | Bounded prospective field qualification |

Each tier receives its own receipt. A lower tier is never promoted to a higher tier by wording, dashboards or a nominal benchmark score.

This specification does not establish calibrated likelihoods, qualified sensors, independent instruments, policy superiority, data availability/rights or authority to conduct surveys, excavations or utility marking. Physical validation requires site permission, competent operators, qualified reference truth and independent review.
