# EARTH-OBSERVABILITY-DETECTION-TO-DECISION-001
## A calibrated bridge from sensor observability to spatial EVSI v0.1

**Parent:** #771 — EARTH-OBSERVABILITY-QUAL-002  
**Related existing work:** [EVSI kernel PR #133](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/133), [spatial sampling roadmap #134](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/134)  
**Status:** integration protocol; adapter and physical likelihood qualification NOT_IMPLEMENTED  
**Date:** 2026-11-13  
**Owners:** Sol-Atlas (sensor/evidence contract), Symtropy (physical likelihood and sensor calibration), Symthaea (decision analysis and investigation planning), Mycelix (authority, source rights and custody)  
**Claim ceiling:** this document defines an interface and statistical preconditions. It does not create a validated detection likelihood or authorize a survey.

---

## 1. Do not build a second EVSI engine

Sol-Atlas already has an open discrete expected-value-of-information implementation in PR #133. It accepts:
- mutually exclusive states and prior probabilities;
- candidate decisions and a complete decision/state loss table;
- investigation outcomes and outcome likelihoods by state;
- investigation cost in the same declared utility unit.

Issue #134 already defines the next spatial extension: where, how deep, how many and in what sequence to investigate, conditional on the engineering decision and failure/serviceability mechanism.

The Earth-Observability work should contribute the missing observation-model layer. It should **not** fork or duplicate the EVSI arithmetic.

Proposed flow:

**qualified sensor envelope → target-conditional measurement likelihood → spatial investigation action → existing EVSI kernel → equal-budget decision tournament**

Sol-Atlas retains evidence, spatial support, source identity and benchmark receipts. Symtropy qualifies the measurement physics, calibration and likelihood. Symthaea proposes candidate actions and interprets decision-value trade-offs. Mycelix authenticates authority, rights, custody and review. The existing site-investigation module owns discrete EVSI arithmetic; #134 owns spatial/sequential sampling research.

## 2. Binary detection likelihoods: the minimum bridge

Consider a bounded search support and a target class k. Let the latent state be target present (T=1) or absent (T=0), and the observed sensor call be Hit or NoHit.

A qualified envelope can provide the target-conditional detection probability:

`P(Hit | T=1, x,e,c,k,s) = POD(x,e,c,k,s)`

But EVSI also requires the probability of a false call when the target is absent:

`P(Hit | T=0, e,c,k,s) = PFA(e,c,k,s)`

The two must remain distinct. POD cannot stand in for both.

The four outcome likelihoods are then:

| Latent state | Hit | NoHit |
|---|---:|---:|
| Target present | POD | 1 − POD |
| Target absent | PFA | 1 − PFA |

Only after both rows are qualified can the adapter emit a valid `P(outcome | state)` table to the discrete EVSI kernel.

### 2.1 False alarms per route-kilometre are not automatically PFA

The current v0.1 scorer reports false alarms per surveyed route-km:

`lambda_FA = false_calls / negative_control_route_km`

That is an operational nuisance-burden rate. It is not automatically a per-target-opportunity false-positive probability.

One possible conversion is:

`PFA(L) = 1 - exp(-lambda_FA * L)`

but only if false calls are adequately modelled as a conditionally homogeneous Poisson process over the searched length L. This assumption can fail when clutter, buried infrastructure density, site heterogeneity, processing thresholds, operator behavior or repeated spatial artefacts create clustering/overdispersion. Until this process model is tested, keep false alarms/km as a separate metric and do not convert it into PFA.

The adapter must bind the searched support length/area/time, not reuse a false-alarm probability from a differently sized search window.

### 2.2 Prevalence is decision-relevant

Even a high POD and low PFA can yield a poor positive predictive value when the target is rare. For prior target prevalence pi:

`P(T=1 | Hit) = POD * pi / (POD*pi + PFA*(1-pi))`

The prior prevalence is a site/target-specific evidence claim, not a default constant. Where prevalence is unknown or unvalidated, the planner should report sensitivity over a defensible range rather than invent a precise prior.

## 3. Multi-outcome observations

A binary hit/miss model is only the first layer. Real field decisions often need at least:

- no indication within a qualified envelope;
- indication detected, localization within tolerance;
- indication detected, localization outside tolerance;
- ambiguous or multiple indication;
- acquisition invalid / sensor fault;
- out-of-domain environmental condition;
- unobserved / not acquired.

Do not treat every detector output as a definitive geological interpretation. Detection and localization can be qualified separately. A missed target, a bad location and a wrong material interpretation have different consequences and may lead to different next actions.

The discrete EVSI model can accept richer outcome sets, but each likelihood must be defined by target and environment strata and calibrated with independent truth. An `Unknown` or unqualified likelihood must not be filled with an arbitrary neutral probability.

## 4. Correlated sensors and common-cause errors

A tempting shortcut is to multiply independent likelihoods from GPR, ERT, seismic and borehole data. This can be badly wrong.

Multiple modalities may share environmental causes:
- moisture and conductivity can affect both GPR and ERT responses;
- geology and acquisition geometry can affect different seismic products;
- the same coordinate transform, control network or registration process can bias multiple surveys;
- common processing thresholds or training labels can couple multiple detector outputs.

The adapter must not assume conditional independence by default.

Preferred progression:
1. fit and qualify modality-specific likelihoods first;
2. model measured common covariates and shared calibration uncertainty;
3. test residual dependence / conditional independence on independent data;
4. use joint likelihoods or a hierarchical latent common-cause model where dependence is material;
5. preserve separate evidence claims when joint calibration is unavailable.

A fusion model that multiplies unvalidated likelihoods can become more confident while being less correct. The benchmark must include a common-cause negative control where two biased modalities agree with each other but disagree with independent truth.

## 5. Investigation action contract

The action selected by a planner is not merely a modality name. It should bind:

- action ID and spatial support / candidate location;
- target class and engineering decision mechanism;
- sensor, configuration, calibration and procedure;
- acquisition geometry, spacing, depth, duration and searched area/route/time;
- expected likelihood-model and envelope IDs;
- expected monetary cost, crew-hours and elapsed time;
- access, permit, method, data-rights and custodian eligibility;
- disturbance and environmental/cultural constraints;
- source of cost assumptions and likelihood calibration;
- declared model version and validity domain.

The action's likelihood must refer to the exact target, environment, support and configuration. A GPR survey with a different antenna frequency or soil condition cannot silently reuse the same POD curve.

## 6. Hard feasibility before expected value

Legal access, landowner permission, method authorization, required permits, source/data rights, professional review, environmental restrictions and non-compensable safety constraints are hard gates.

The planner must:
1. classify candidate actions and retain an explicit rejection reason;
2. remove ineligible actions before economic ranking;
3. call EVSI only for eligible actions with qualified outcome models;
4. preserve mandatory investigations even if EVSI is low or negative;
5. never monetize non-compensable duties to make them appear optional.

A high information-gain or EVSI estimate cannot create legal authority or a land-access right.

## 7. Equal-budget tournament interface

For each frozen case, the tournament should receive a single admissible action catalogue and compare:

A. fixed-grid/fixed-spacing plan;  
B. documented expert plan;  
C. random eligible sampling;  
D. cost-only selection;  
E. entropy/information-gain greedy selection;  
F. existing EVSI/ENBS kernel with spatial candidate actions;  
G. mechanism-specific sequential EVSI planning;  
H. oracle using withheld truth, upper bound only.

Freeze monetary budget, crew-hours, duration, access and method constraints, disturbance limits, target truth, prior model, likelihood model, loss table and scoring code before the blind run.

Report:
- post-investigation expected loss and realized loss against withheld truth;
- regret versus the best eligible strategy in hindsight;
- EVSI and ENBS predicted versus realized value;
- investigation cost, crew-hours, delay and disturbance;
- modality/site/action counts and uncovered support;
- unauthorized/ineligible action recommendations (required zero);
- recommendation sensitivity to priors, costs, likelihoods and consequence models.

A plan cannot win by acquiring more data or relaxing the authority constraints. Publish a Pareto frontier when objectives cannot be converted into a defensible common utility unit.

## 8. Required adversarial cases

| Case | Failure to expose | Required behavior |
|---|---|---|
| EVSI-POD-NO-PFA-001 | POD known, PFA unknown | no EVSI likelihood table; remain unqualified |
| EVSI-RARE-TARGET-001 | rare target prevalence makes PPV low | expose prevalence sensitivity; do not imply high certainty from POD alone |
| EVSI-POISSON-INVALID-001 | clustered false calls violate Poisson conversion | reject/disable lambda-to-PFA conversion |
| EVSI-COMMON-CAUSE-001 | correlated sensors share moisture/registration bias | do not multiply likelihoods under untested independence |
| EVSI-OUTCOME-AMBIGUOUS-001 | detector indication has uncertain localization/class | use distinct outcomes or abstain |
| EVSI-ACCESS-DENIED-001 | high-value action lacks land access/permit | reject before economic selection |
| EVSI-NECESSARY-SAFETY-001 | mandatory investigation has negative net economic value | retain hard obligation; economic model cannot waive |
| EVSI-RANK-REVERSAL-001 | prior or loss perturbation reverses action ranking | report rank instability |
| EVSI-EQUAL-BUDGET-001 | planner spends more to improve apparent quality | enforce frozen equal budget and report Pareto |
| EVSI-TRUTH-LEAKAGE-001 | calibration targets also appear in blind truth | invalidate evaluation |

## 9. Research and benchmark references

- ASTM E2862-23, *Standard Practice for Probability of Detection Analysis for Hit/Miss Data*. It covers hit/miss POD analysis, validation and interpretation. Its scope assumes clearly defined discontinuities, known/documented sizes and a controlled, well-designed and unbiased examination. DOI: https://doi.org/10.1520/E2862-23
- ASTM E3023-21, *Standard Practice for Probability of Detection Analysis for `a-hat` Versus `a` Data*. It applies where the initial inspection response is measurable and continuous and the inspection system is calibrated. DOI: https://doi.org/10.1520/E3023-21
- MIL-HDBK-1823A, *Nondestructive Evaluation System Reliability Assessment*, 7 Apr 2009; DLA ASSIST lists Notice 2 validation dated 13 Nov 2018. It is guidance for NDE capability assessment, not a geotechnical requirement. Official status/history: https://quicksearch.dla.mil/qsDocDetails.aspx?ident_number=206122
- Fartosy et al. (2026), *Optimizing Geotechnical Site Characterization: A Value of Information Approach Using Coupled Spatial Random Fields and Bayesian Networks*. DOI: https://doi.org/10.3311/PPci.43308
- Dengyi et al. (2026), *Probabilistic landslide stratigraphic modelling by uncertainty fusion of surface wave inversion and borehole data*. DOI: https://doi.org/10.1016/j.enggeo.2026.109128

The NDE standards give us mature POD methodology. They do not automatically validate geophysical target classes, spatial clustering, environmental transfer or geology-specific truth. Those are the research extension.

## 10. Implementation sequence and ownership

**Sol-Atlas**
- own `SensorObservabilityEnvelope`, evidence provenance, support/time binding, trial splits, evaluation receipts and the conversion contract;
- keep route/area/time false-alarm burden distinct from per-target PFA until a model justifies conversion;
- adapt the existing discrete EVSI kernel rather than implement a second one.

**Symtropy**
- own physical forward models, sensor calibration, modality-specific POD/PFA and localization likelihoods;
- test environmental and common-cause dependence;
- own the physics-simulation-to-physical-validation bridge.

**Symthaea**
- own candidate action ranking, mechanism-specific decision models, plan explanation, sensitivity analysis and sequential selection;
- never invent prior probabilities, outcome likelihoods or authority.

**Mycelix**
- own exact data rights, authority evidence, custody, reviewer identity, revocation and audit receipts.

**Existing work**
- PR #133 is the discrete EVSI arithmetic base;
- issue #134 owns spatial investigation and adaptive sampling;
- PR #775 implements the initial sensor envelope and analytic POD scoring interface;
- PR #138 provides a relevant immutable evidence/dependency and revalidation pattern for downstream decisions.

This keeps the architecture compositional and avoids competing owners for the same decision kernel.
