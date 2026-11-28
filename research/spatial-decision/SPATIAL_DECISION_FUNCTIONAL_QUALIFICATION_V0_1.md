# Spatial Decision Functional Qualification v0.1

**Status:** research specification; no spatial functional, physical model or field result is newly qualified  
**Issue:** [#990 — SPATIAL-DECISION-FUNCTIONAL-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/990)  
**Scope:** spatial aggregation, threshold, path and topology functionals over qualified joint spatial evidence

## 1. Research question

Can Sol-Atlas turn a qualified spatial field, joint predictive distribution or model ensemble into a trustworthy uncertainty statement about the **actual spatial quantity or topology decision** being considered?

Pointwise predictions are not enough. A decision often concerns a parcel, field, catchment, corridor, volume, connected service region or critical network path. The uncertainty of that functional depends on:
- the exact physical support and integration measure;
- joint spatial and temporal dependence;
- the geometry and measurement uncertainty of the support itself;
- missing, occluded, out-of-domain and rights-withheld regions;
- model-form or topology alternatives;
- and the decision's threshold, units and time horizon.

The new benchmark must test the whole decision functional, not only the accuracy of its component pixels or points.

## 2. Paradigm shift: from pixel quality to decision-functional assurance

A conventional map workflow often looks like:

`predict at points/pixels -> render -> aggregate point estimates -> decide`

The proposed workflow is:

`declare estimand + support -> bind qualified joint spatial evidence -> propagate dependence/model uncertainty -> evaluate functional distribution -> assess decision and regret`

The benchmark explicitly separates:
1. **Pointwise prediction qualification:** whether predictions at a point or cell are reliable.
2. **Simultaneous spatial coverage:** whether a spatial field over a region is covered by a band/set with its stated guarantee.
3. **Functional qualification:** whether the distribution of an integral, mean, thresholded extent, path or topology property is calibrated for the declared decision support.
4. **Decision qualification:** whether the remaining functional uncertainty could change a consequential action.

A pointwise 95% interval is not a 95% simultaneous spatial band. Neither, by itself, establishes calibrated uncertainty for a nonlinear functional such as area above threshold or probability of network connectivity.

## 3. Mathematical contract

### 3.1 Linear functionals

For a spatial field (Z(s)), support (A), and declared weight/density (w(s)), define:

[
T_A = int_A w(s)Z(s),dmu(s)
]

The measure (dmu), weight units and physical support must be explicit. Examples include area, volume, length, mass or time-weighted exposure.

For a mean, the denominator is the measure of the **target support**, not automatically the observed-data footprint:

[
ar Z_A = rac{int_A w(s)Z(s),dmu(s)}
{int_A w(s),dmu(s)}
]

For a discretized field (z) and weights (a), if the upstream provider supplies the joint covariance matrix (Sigma), then:

[
operatorname{Var}(a^	op z)=a^	opSigma a
]

Equivalently, under a covariance function (C(s,u)), the variance of a normalized regional mean contains the double integral:

[
operatorname{Var}(ar Z_A)=
rac{1}{|A|^2}
int_Aint_A C(s,u),dmu(s),dmu(u)
]

Ignoring off-diagonal terms is an explicit independence approximation, not a harmless default. It must fail the corresponding correlated controls.

### 3.2 Nonlinear functionals

Examples include:

**Thresholded extent**
[
E_	au(A)=int_A mathbf{1}{Z(s)>	au},dmu(s)
]

**Route exposure**
[
R_Gamma=int_Gamma q(s,t),ds
]
with an explicit route geometry (Gamma), temporal support, exposure quantity (q), and units.

**Connectivity**
[
P(G(Z)	ext{ contains a qualifying path})
]
where (G) is an explicitly versioned graph construction/threshold operator. Marginal edge probabilities alone do not define this probability when edges are dependent.

**Change functional**
[
Delta_A=T_A(Z_{t_1})-T_A(Z_{t_0})
]
which must preserve baseline/follow-up cross-time covariance and any mismatch in spatial support.

For nonlinear functionals, use qualified joint predictive draws, a validated exact method or declared structural bounds from the provider. Do not manufacture a functional distribution by applying a nonlinear operator to pointwise means or independent marginal intervals.

### 3.3 Unknown and withheld support

Do not silently drop unknown cells and renormalize the remaining observed area as if it represented the full target region.

The contract must distinguish:
- functional over the full requested support;
- functional over the observed/qualified sub-support;
- lower/upper bounds for the full support, only where defensible physical bounds exist;
- unresolved functional because missing/withheld support prevents a supported result.

The output must carry both the numerator and the declared denominator/support measure, with units and coverage. A missing region does not become zero, safe, below-threshold or disconnected by default.

## 4. Cross-repository owners — compose, do not duplicate

- **Sol-Atlas #986 / PR #986:** pointwise predictive calibration, spatial interval coverage, worst-site/sensor reporting and metrology receipts.
- **Sol-Atlas #988 / PR #989:** spatial applicability, support, frame, time, campaign and calibration-scope binding for likelihood outputs.
- **Sol-Atlas #771 / PR #775:** sensor observability, POD/PFA and decision-grade investigation.
- **Sol-Atlas #583:** spatially integrated detection likelihood; do not create another target-detection/coverage integrator.
- **Sol-Atlas #845:** physical-change invariance under sensor, processing, sampling and estimand-support perturbations.
- **Sol-Atlas #922:** planetary/subsurface frame, geometry, uncertainty, evidence lineage and topology projection.
- **Sol-Atlas #534/#543:** spatially independent splits, independent units and effective-information accounting.
- **Sol-Atlas #552 / HDB-18:** independent metrology and covariance-aware aggregation in the heritage morphology vertical. Reuse its owner-native evidence and domain-specific methods; do not fork its metrology system.
- **Mycelix #5632/#5648:** covariance-aware likelihood, model discrepancy, posterior-SBC and robust Bayesian-design providers.
- **Symthaea #13904:** dynamic full-covariance state-space and covariance-model uncertainty.
- **Symtropy:** physical field/network and forward-model outputs remain owned by their relevant domain verticals.

Sol-Atlas owns:
- spatial functional identity and version;
- support geometry/mesh/graph, CRS, datum, epoch and integration measure;
- weights, units, boundary/missingness semantics and support conversion;
- exact provider-receipt binding and independent adapter checks;
- spatially stratified functional scorecards and decision-facing evidence projection.

It does not estimate a new covariance model, posterior, geostatistical field, physical solver or network simulator.

## 5. Functional families

### 5.1 Area/volume integral and mean

Examples: field/catchment average, biomass/carbon stock, water storage, pollutant load, earthwork quantity and parcel-scale material totals.

Qualification:
- source-point to block-support conversion;
- exact area/volume weights and unit conversions;
- uncertainty of both field and support boundary where relevant;
- spatial covariance contribution to aggregate variance;
- validation against independent bulk/composite measurements where possible.

### 5.2 Thresholded extent and excursion sets

Examples: area above a contaminant threshold, flood extent, heat exposure, subsidence above tolerance, or region where a material property falls outside specification.

Qualification:
- probability distribution of total exceedance area/volume;
- uncertainty in boundary and connected-component structure;
- threshold sensitivity frontier;
- false-criticality and missed-criticality rates;
- simultaneous/contour-set guarantees where the provider supports them.

### 5.3 Line and corridor functionals

Examples: exposure along a route, utility corridor, survey transect, railway embankment or pipeline alignment.

Qualification:
- physical line measure and sampling/integration weights;
- along-track temporal/spatial correlation;
- gaps, occlusion and changing observation cadence;
- worst-segment and total exposure outcomes;
- no interpolation across unknown support unless an owner-qualified method permits it.

### 5.4 Topology and connectivity

Examples: probability that a service network retains a path, a cave passage is connected, a utility crossing exists, or a flood/critical habitat region is connected.

Qualification:
- exact versioned graph-generation rule and geometry-to-edge mapping;
- dependence between uncertain edges, geometry and observations;
- probability of a qualifying path/component/cut under joint worlds;
- unresolved crossings distinct from confirmed junctions;
- structural model alternatives preserved instead of averaged into a fictional fractional connection.

### 5.5 Space-time change functionals

Examples: total stock change, wetland extent change, flood-duration area, asset movement, service-access opportunity and corridor degradation/recovery.

Qualification:
- matching support, frame, time basis and revision;
- baseline/follow-up error covariance;
- actual usable observation windows and interval censoring;
- support change and minimum mapping unit;
- missingness/rights changes across time.

### 5.6 Distributional and worst-unit functionals

Examples: upper quantiles, lower-tail service, worst-site, worst-segment or declared cohort results.

Qualification:
- explicit population/region and resampling unit;
- tail definition and denominator;
- no concealment of a critical site or protected cohort by a regional mean;
- report each material unit's disposition and uncertainty.

## 6. Matched uncertainty-propagation baselines

All arms receive the same support, source data, model revision, weights, thresholds, time window, calibration evidence and budget.

1. **Point-estimate plug-in:** aggregate the predicted mean field only. A baseline for central-value error, not functional uncertainty.
2. **Independence approximation:** propagate only marginal variances. Include as a deliberately limited comparator and require it to fail known-correlation controls.
3. **Analytic covariance reference:** exact (a^	opSigma a) and closed-form cases where available.
4. **Provider-native joint posterior/predictive propagation:** apply the functional to complete joint draws from a qualified upstream provider.
5. **Spatial block/cluster bootstrap:** resample the correct independent unit; do not resample pixels as independent trials.
6. **Geostatistical block/conditional simulation:** use only where the geostatistical assumptions and support are appropriate and declared.
7. **Robust structural ensemble:** for alternative topology/segmentation/physical models, retain the separate model worlds and report model sensitivity or robust bounds. Deterministic variant counts are not posterior probabilities without separate calibration.

Sol-Atlas performs the functional adapter and independent checks; it does not reimplement these numerical methods.

## 7. Frozen analytical and adversarial corpus

The initial finite corpus must include:

1. Independent equal-variance cells: aggregate variance matches a hand-derived reference.
2. Equicorrelated cells: identical pointwise marginals but different covariance produce different aggregate variances.
3. Shared spatial bias: pointwise RMSE remains unchanged while the regional-mean uncertainty is much larger than the independence estimate.
4. Thresholded extent: identical marginal exceedance probabilities but different spatial clustering produce different area/component distributions.
5. Connectivity: networks with equal marginal edge probabilities but different joint/common-cause dependence produce different path probabilities.
6. Unknown/no-data region: never silently becomes zero, safe, non-exceeding or disconnected.
7. Partial footprint and occlusion: target support and observed support denominators remain explicit.
8. Polygon boundary sliver, raster alignment and CRS/area-unit changes.
9. Resolution change with one functional that remains invariant under a declared operator and another that changes estimand.
10. Overlapping regions/tiles: no duplicate contribution to one total.
11. Three-dimensional volume with variable cell size and vertical datum.
12. Line integral with correlated along-track sensor drift.
13. Baseline/follow-up under positive, zero and negative cross-time error correlation.
14. Few independent sites with many pixels: uncertainty reflects the site-level independent unit.
15. Geometry/boundary/registration uncertainty changes area or volume and is propagated or explicitly excluded.
16. Two segmentation/model families disagree on a critical connection; alternatives remain explicit.
17. Rights-withheld region influences a public aggregate; the existing rights/taint owner determines disclosure eligibility.
18. Positive control: a permitted, informative observation reduces functional uncertainty and changes a declared decision.

Each case must expose evaluator-only truth and expected outputs separately from candidate-visible evidence. Include valid positive cases; an always-abstaining or always-withheld implementation cannot pass.

## 8. Metrics and qualification targets

Report by functional, site/campaign, support, regime and decision unit:
- functional estimate, units, denominator and support identity;
- full functional predictive distribution or interval;
- nominal 90%/95% coverage and interval score where applicable;
- CRPS/log score or other suitable proper scores for predictive distributions;
- bias/error for means and totals;
- threshold-exceedance Brier/log score and reliability;
- connectivity/path/component calibration on finite exact cases;
- aggregate uncertainty width and sensitivity to covariance assumptions;
- worst-site/segment/region coverage;
- unknown/withheld/out-of-domain support and its contribution to the unresolved functional;
- coverage/sharpness versus selective abstention;
- decision reversals under support/model/threshold perturbations;
- realized decision regret and acquisition effort where functional uncertainty drives investigation;
- runtime, memory and replay cost.

Hard gates:
- 100% agreement with independent analytical/exhaustive oracles on the frozen finite corpus.
- 0 implicit independence assumptions where the fixture contains spatial, temporal or common-cause dependence.
- 0 silent area, volume, CRS, datum, epoch, support or time-window transformations.
- 0 promotion of unknown/withheld/no-data support to zero or safe.
- 100% explicit denominator and coverage accounting.
- 100% deterministic replay for pinned deterministic cases.
- Pointwise calibration must not be presented as functional calibration.
- No aggregate score compensates for failure at a critical site, protected cohort, rights gate or mandatory-safety constraint.

Stretch targets (hypotheses, not results):
- calibrated functional intervals on whole-site/whole-campaign holdouts, stratified by the actual decision unit;
- measurable reduction in functional error or decision regret against the strongest owner-native matched baseline at equal resources;
- at least 20% lower out-of-sample regret at matched eligible effort, or at least 20% lower acquisition effort at matched regret.

Set tolerances, sample sizes and effect thresholds from independent baseline variance and preregistered precision/power analysis. The 20% target is an internal research hypothesis, not a published benchmark result or achieved target.

## 9. Research baselines and current landscape

These methods already address parts of the problem. Sol-Atlas should reproduce or consume them, not claim to invent block-support uncertainty or spatial aggregation:

- **Wadoux et al. (2023), uncertainty of spatial averages and totals of natural-resource maps**, DOI [10.1111/2041-210X.14106](https://doi.org/10.1111/2041-210X.14106). Describes block kriging and a scalable spatial-aggregation alternative for propagating uncertainty of spatial averages/totals; stresses that map-error spatial autocorrelation materially affects aggregate uncertainty. It also recommends validation at block support, ideally independent composite/bulk measurements where feasible.
- **Wadoux et al. (2026), digital soil mapping for soil-carbon MRV**, DOI [10.1038/s44264-026-00125-0](https://doi.org/10.1038/s44264-026-00125-0). Demonstrates a complete point-to-field/project aggregation and uncertainty workflow, including residual spatial correlation and project-level uncertainty. Its reported values are study-specific and are not South African acceptance targets.
- **Coherent Disaggregation and Uncertainty Quantification for Spatially Misaligned Data (2026)**, DOI [10.1002/env.70078](https://doi.org/10.1002/env.70078). Provides Bayesian approaches to point/raster/polygon aggregation and uncertainty propagation across misaligned supports; compare exact assumptions and alternatives rather than treating one approach as universally correct.
- **Localized Spatial Conformal Prediction (ICML 2026)**, [PMLR 306](https://proceedings.mlr.press/v306/jiang26at.html), and GeoSIMCP (2026), DOI [10.1080/24694452.2026.2639721](https://doi.org/10.1080/24694452.2026.2639721), are pointwise/localized spatial uncertainty comparators. They do not automatically calibrate an areal total, thresholded extent or connectivity functional.
- The R package [excursions](https://www.r-inla.org/), by Bolin & Lindgren, calculates excursion sets, contour credible regions and simultaneous bands for supported Gaussian-process/INLA/Monte-Carlo inputs. It is a relevant baseline for supported excursion/contour tasks, not a universal solution for all spatial functionals.
- A 2026 *Environmental Modelling & Software* diagnostic framework argues that aggregate probabilistic scores can hide spatiotemporal variation in uncertainty and offers map/polar diagnostics for interval width, exceedance and horizon drift: DOI [10.1016/j.envsoft.2026.107052](https://doi.org/10.1016/j.envsoft.2026.107052).

For the competitive comparison, include product-native workflows from ArcGIS Pro/Geostatistical Analyst, Leapfrog Works/Central, FME and Bentley iTwin only when the exact product/version/module can be configured and run. Do not infer absence of a capability from an incomplete public feature list. OGC exchange conformance is not functional semantic fidelity.

## 10. Industry matched experiment

Use the same source bytes, support, functional definition, model revision, weights, decision thresholds, resource envelope and independent reference:

- **B0 — specialist-native:** the incumbent's native workflow for the selected functional.
- **B1 — specialist + documented manual integration:** identical native outputs with a fully documented human handoff.
- **B2 — Sol-Atlas functional adapter:** same upstream outputs, with exact support/units/missingness and provider-receipt binding.
- **B3 — joint-uncertainty augmentation:** where eligible upstream joint distributions/covariance receipts exist, compute and evaluate functional distributions and robust bounds.
- **B4 — independent oracle:** hand-calculable or separately implemented exact references for finite cases; blind independent reference measurements for physical tasks.

Record exact software versions, options, operator expertise, tuning/data-access restrictions, hardware/compute, time, exclusions and failed/unsupported runs. Report NOT_RUN where a product or license cannot be accessed; do not convert unavailable into inferior.

## 11. Execution ladder and claim ceiling

- **F0:** mathematical contract and hand-computable support/functional cases.
- **F1:** independent Rust reference for bounded analytical/grid/network cases.
- **F2:** upstream joint predictive samples/ensembles with exact spatial support binding.
- **F3:** retrospective whole-site/whole-campaign validation against qualified reference measurements.
- **F4:** prospective, permissioned field campaign and decision tournament.

A lower tier cannot be promoted by wording or dashboard display. This work does not establish field accuracy, engineering safety, land access, rights, physical connectivity or authority to alter infrastructure.

## 12. Explicit non-claims

This is a research/benchmark contract. It does not:
- implement a new geostatistical, covariance, Bayesian or topology engine;
- establish a new physical measurement or reference survey;
- prove general uncertainty calibration from synthetic tests;
- authorize field sampling, excavation, utility marking or engineering approval;
- establish SOTA superiority before matched, independent, exact-head results exist.
