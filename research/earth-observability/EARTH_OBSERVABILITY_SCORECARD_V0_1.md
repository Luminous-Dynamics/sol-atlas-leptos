# EARTH-OBSERVABILITY-SCORECARD-001
## Metrics, baselines, and SOTA comparison protocol v0.1

**Parent:** Sol-Atlas #761 — EARTH-OBSERVABILITY-QUAL-001  
**Status:** research protocol supplement; no benchmark results claimed  
**Date:** 2026-11-13

This scorecard makes the parent specification measurable. Scores remain task-specific: a point-cloud registration result, geological contact posterior, and groundwater time series are not interchangeable outputs.

## 1. Benchmark rules

1. Freeze target definition, spatial support, time horizon, evidence set, quality exclusions, baseline versions, hardware, compute/time budget, cost boundary and acceptance metric before evaluation.
2. Publish all eligible baselines and ablations, including simple conventional methods. An unavailable proprietary comparator is NOT_RUN, never assumed inferior.
3. Use spatially and temporally independent holdouts. Split by site, survey, borehole, campaign, geological domain, instrument/rig and time as relevant. Tile- or point-level random splits are invalid where they leak one physical acquisition across partitions.
4. Keep model-fitting observations, registration/control points, calibration samples and independent evaluation checkpoints disjoint.
5. Report distributions, worst-domain strata and failure cases, not only averages. Preserve abstentions and out-of-domain cases in the denominator.
6. Report raw-source, transformation, model, configuration, split and evaluation-code identities with each run receipt.
7. Synthetic, retrospective and prospective physical results are different evidence tiers and have separate tables and headlines.

## 2. Geometry and survey scorecard

| Task | Primary measures | Required stratification / cautions |
|---|---|---|
| Absolute point positioning | Horizontal RMSE/median/P95; vertical RMSE/median/P95; bias | Independent checkpoints, coordinate epoch, vertical datum, terrain/cover class, acquisition mode |
| Point-cloud registration | Independent checkpoint residuals; registration completeness; overlap; point-to-plane residual distribution | Internal ICP residual is not absolute accuracy; report degeneracy and weakly observed directions |
| Surface/volume reconstruction | Surface distance distribution; signed bias; area/volume error with uncertainty | Ground/non-ground class, vegetation, slope, density, occlusion, scan geometry |
| Repeat-survey change | Detection precision/recall; minimum detectable change; false-change area; change-volume error | Account for correlated registration drift and covariance; distinguish physical change from measurement noise |
| Control network adjustment | Closure, residual distribution, redundancy, covariance and rejected observations | Never remove outliers silently; report datum choice and under-constrained networks |

Do not infer a lidar accuracy quality level from point density alone. A density of at least 4 points/m² does not establish a USGS QL2 or any other accuracy class.

## 3. Subsurface reconstruction scorecard

Evaluate each target separately, with its own reference truth and support geometry.

| Target | Measures |
|---|---|
| Geological contact / interface | Signed-distance error; median/P95 boundary distance; contact depth MAE/RMSE/bias; 3D surface overlap where applicable |
| Lithology / material classes | Class-wise precision/recall; macro-F1; confusion-cost matrix; Brier score/log loss; calibration |
| Bedrock / layer depth | MAE/RMSE/bias; P50/P95 absolute error; 90/95% interval coverage and width |
| Fault / fracture geometry | Trace/plane position and orientation error; connectivity/topology consistency only where labelled reference supports it |
| Cavities / voids | Detection probability by size/depth/contrast; localization error; false positives per surveyed volume; blind-zone reporting |
| Groundwater | Water-level/pressure MAE and bias; probabilistic interval coverage; temporal forecast skill; site/time holdout |
| Utility / tunnel geometry | Geometry and attribute agreement against owner-authorized survey/as-built truth; access-control/disclosure tests |

For volumetric truth, include voxel/volume overlap only when voxelization and minimum feature size are fixed before testing. Do not let resolution choice manufacture a favourable score. For surface interfaces, use distance-based metrics alongside overlap measures; thin or displaced boundaries can have misleading volumetric IoU.

Mandatory baselines, when applicable:
- published geological map / qualified expert interpretation;
- borehole-only nearest-neighbour and linear interpolation;
- ordinary or indicator kriging with declared variogram and fitting procedure;
- single-modality geophysical inversion/interpretation;
- conventional implicit geological modelling;
- multi-modal fusion with explicit uncertainty;
- fusion ablations (remove each modality, remove uncertainty weighting, remove source-quality weighting);
- oracle with held-out truth (upper bound, never a deployable baseline).

## 4. Observability and detection scorecard

A sensor's geometric footprint is not its observability domain. For each modality and target, characterize detection probability as a function of relevant conditions.

| Measure | Why it matters |
|---|---|
| Probability of detection (POD) / detection curve | Makes detection likelihood conditional on size, depth, contrast, material, moisture, geometry and environment |
| False alarms per area-time or route-time | Avoids hiding operational alert burden behind sample-level accuracy |
| Localization P50/P95 and uncertainty coverage | Measures whether a detected feature is located well enough for the intended follow-up |
| Observable fraction and blind-zone fraction | Exposes ineffective coverage caused by coupling, attenuation, occlusion, conductivity or acquisition geometry |
| Minimum detectable feature / change | Defines what non-detection can and cannot mean |
| Detection latency and uptime | Relevant to time-critical monitoring |
| Abstention and out-of-domain rate | Prevents unsupported confidence outside the validated envelope |

A negative result is evidence of absence only if the target was inside an independently qualified observation domain and validated detection probability meets the preregistered requirement. Otherwise report UNOBSERVED, BELOW_DETECTION_LIMIT or OUT_OF_DOMAIN.

Do not merge GPR, ERT, EM, seismic, microgravity, borehole logs and DAS into a single modality leaderboard. Their physics and detection targets differ. Compare fusion only on the same target cases and reference truth.

## 5. Uncertainty scorecard

For continuous predictions:
- empirical coverage of nominal 50%, 80%, 90% and 95% intervals;
- interval width / weighted interval score;
- CRPS or pinball loss where distributional forecasts are available;
- bias and sharpness, reported together;
- calibration by site, geology, depth, modality, target class and distance from observations.

For categorical predictions:
- Brier score and log loss;
- reliability curves and class-conditional calibration;
- confusion-cost matrix at any operational threshold;
- selective risk / coverage curve under abstention.

For spatial predictions:
- uncertainty coverage on independently withheld boundaries/checkpoints;
- spatial residual correlation and variogram diagnostics where appropriate;
- calibration of uncertainty growth with distance from observations;
- separate aleatoric, epistemic, measurement and transform contributions where identifiable.

Expected calibration error (ECE) must not be the only calibration metric. Calibration can be hidden by binning choices, class imbalance and pooling heterogeneous domains. A nominal 95% interval that covers 95% only after pooling all sites but under-covers a particular geology or modality is not adequately calibrated for that subgroup.

## 6. Adaptive investigation scorecard

Compare candidate planners at the same declared field budget and eligibility constraints:

A. fixed-spacing / fixed-grid plan;  
B. expert-designed plan, if documented;  
C. random or cost-only plan;  
D. uncertainty-reduction greedy plan;  
E. mechanism-/decision-aware EVSI or expected net benefit of sampling plan;  
F. oracle selected with withheld truth (upper bound only).

Report accepted decision quality / expected loss after the survey; expected value of sample information and net benefit after survey cost; decision regret relative to the best eligible strategy in hindsight; uncertainty reduction by target and spatial domain; cost, time, access burden and disturbance as separate dimensions; number and type of samples, boreholes, survey hours and coverage; recommendation stability under prior, covariance, likelihood, cost and consequence perturbations; and fraction of recommendations ineligible or abstained with reason.

For prior information D and candidate survey A with outcome Y, define value of sample information relative to decision loss L as:

EVSI(A | D) = min_d E[L(d, θ) | D] - E_Y[min_d E[L(d, θ) | D, Y]]

Then define net benefit of sampling:

ENBS(A | D) = EVSI(A | D) - C(A)

where C(A) is the declared compatible cost/loss burden. Do not monetize non-compensable safety, legal, land, ecological or cultural obligations. Those remain eligibility constraints. When delay, access or disturbance cannot be defensibly converted to money, report them separately or use an explicitly declared constrained/multiobjective comparison.

## 7. Cross-domain transfer and adversarial robustness

Minimum holdouts:
- leave-one-site-out;
- leave-one-geological-domain-out;
- leave-one-instrument or sensor-configuration-out;
- leave-one-survey campaign/time period out;
- prospective blind site, when permission and qualified ground truth are available.

Perturbations:
- remove a modality; bias or drift one sensor; degrade calibration; shift coordinate frame/datum; change sampling density; simulate occlusion/attenuation; alter seasonal moisture; add correlated registration error; remove or stale source records; inject conflicting interpretations; move beyond trained depth/geology/domain.

Report both performance degradation and whether the system correctly expands uncertainty or abstains. A model that keeps its confidence while failing under shift is a critical failure even when mean accuracy remains acceptable.

## 8. Relevant 2026 research frontier and what is still missing

The frontier already contains important individual components:

- Probabilistic landslide stratigraphic modelling combining boreholes with surface-wave inversion ensembles and variability-weighted fusion, with entropy maps for targeted follow-up investigation. The reported approximately 26% false-positive-area reduction is from one bounded case/configuration, not a universal field-performance claim. DOI: https://doi.org/10.1016/j.enggeo.2026.108746
- Spatial random fields, Bayesian networks and expected value of information for geotechnical investigation planning. A reported “up to 143%” economic-efficiency improvement is specific to the paper's assumptions and case studies; it is not a transferable industry benchmark. DOI: https://doi.org/10.3311/PPci.43308
- Multi-source 3D geological modelling and uncertainty assessment, including borehole/geophysical fusion and karst application. DOI: https://doi.org/10.1016/j.enggeo.2026.108659
- Bayesian continual learning and uncertainty-aware TBM geological prediction in real tunnel twin-line operations. This establishes a moving industrial-research frontier, not an independent cross-vendor benchmark. Paper: https://www.sciencedirect.com/science/article/abs/pii/S0886779826305746

Therefore, the novelty claim must not be “we combine data,” “we use Bayesian uncertainty,” “we create entropy maps,” or “we choose survey locations.” Those are established research directions.

The differentiating hypothesis is a reusable, open, vendor-neutral qualification layer that:
1. preserves source and physical observation semantics across standards and vendors;
2. exposes where every sensing process is and is not capable of detecting the target;
3. keeps correlated uncertainty and conflicting interpretations alive through downstream decisions;
4. selects additional evidence based on decision loss, not generic information gain alone;
5. performs region/site/instrument-held-out and prospective evaluation;
6. revalidates only affected downstream claims when evidence or the physical environment changes.

## 9. Industry baseline comparison

| Capability | Established industry baseline | Sol-Atlas target position |
|---|---|---|
| Geotechnical data management | Bentley OpenGround: borehole, test, laboratory and monitoring records/workflows | Preserve exchange lineage and evidence state across tools; do not rebuild the data management product |
| 3D geological modelling | Seequent Leapfrog Works and specialist geology platforms | Compare their outputs where data/licence permit; add independent calibration, cross-source disagreement and repeatable evidence receipts |
| GIS and spatial data | Mature GIS and spatial databases | Use established geometry/CRS engines and OGC APIs; concentrate on evidence, uncertainty and claims |
| Surveying/point-cloud | Specialist survey instruments and registration/adjustment packages | Retain exact frame/transform/calibration and independent checkpoint quality; do not claim instrument superiority |
| Geophysics / inversion | Modality-specific specialist processors | Treat as reference engines; compare same raw observations and same held-out truth |
| TBM digital twin / monitoring | OEM and project-specific machine telemetry, physics and prediction systems | Integrate qualified telemetry with actual geology/as-built results and uncertainty; do not claim replacement without matched field trial |

A fair SOTA table must record product/version, license/access, exact functions exercised, data contract, task and metric, hardware/runtime, domain, validation population, and independent result. Product pages are capability statements, not proof of accuracy or SOTA.

## 10. Proposed research gates

**G0 — Protocol:** preregister a target, use case, baselines, split, primary metrics, failure costs and claim ceiling.  
**G1 — Contract:** pass every seeded schema/frame/rights/lineage/negative-control case.  
**G2 — Reproduction:** reproduce at least one relevant published method or specialist baseline within a predeclared tolerance.  
**G3 — Matched comparison:** demonstrate better calibration, decision loss or survey cost at equal constraints on independent holdout.  
**G4 — Transfer:** retain performance/appropriate abstention on site, geology, instrument and time holdouts.  
**G5 — Prospective:** blind field evaluation with independent control and competent review.  
**G6 — Operations:** demonstrate monitoring, drift, stale evidence, revalidation and recovery under field conditions.

A research hypothesis of 20% improvement in expected decision loss or survey cost is a candidate stretch goal only after G2 establishes baseline variance. No numerical superiority claim is permitted before G3. No engineering safety or excavation authorization follows automatically from G5 or G6; those require the applicable discipline-specific authority and evidence.

## 11. References and product descriptions

- OGC MUDDI: https://docs.ogc.org/is/23-024/23-024.html
- OGC GeoSciML: https://www.ogc.org/standards/geosciml/
- OGC GroundWaterML 2: https://www.ogc.org/standards/gwml2/
- OGC API - Connected Systems: https://ogcapi.ogc.org/connectedsystems/
- OGC API - EDR 1.2: https://docs.ogc.org/is/19-086r9/19-086r9.html
- USGS lidar quality levels: https://www.usgs.gov/3d-elevation-program/topographic-data-quality-levels-qls
- Bentley OpenGround: https://www.bentley.com/products/openground/
- Seequent Leapfrog Works: https://www.seequent.com/products-solutions/leapfrog-works/
