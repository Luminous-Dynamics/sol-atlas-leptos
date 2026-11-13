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

## 6A. First synthetic harness smoke-test (not a field benchmark)

The first pinned baseline run uses a two-transect abstract fixture, 6 sparse training borehole contacts, 22 noisy scalar interface-proxy records, 11 blind truth points and a 5-position unobserved blind zone. See the generated receipt at `research/earth-observability/baselines/synthetic-profile-baseline-v0.1.json`.

| Synthetic method | Coverage | MAE (m) | RMSE (m) | Bias (m) | P95 absolute error (m) | Abstentions |
|---|---:|---:|---:|---:|---:|---:|
| Sparse training-transect borehole piecewise-linear transfer | 100% (11/11) | 1.251818 | 1.253196 | -1.251818 | 1.345000 | 0 |
| Blind-transect scalar proxy-only | 54.55% (6/11) | 0.045000 | 0.053697 | 0.001667 | 0.100000 | 5 |

This intentionally creates a transfer shift between the two abstract transects. The borehole-only model appears to cover all target positions but carries systematic error on the shifted site. The local proxy has lower error where it exists but correctly abstains in five unobserved positions. This is a software-harness demonstration of the coverage/error/abstention trade-off—not evidence that one physical sensor modality is more accurate, not a published SOTA comparison, and not a geological field result. The proxy values are scalar synthetic interface estimates, not GPR/ERT/seismic waveforms or a forward simulation.

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

## 7A. Rights-aware federation scorecard

Treat discovery, read, processing, derivative creation, redistribution, retention and publication as distinct operations. Compare:
- owner-local execution;
- a centralized copy only where the exact source grant permits copying;
- an authorized local reference implementation.

| Measure | Definition | Required negative controls |
|---|---|---|
| Exact asset/subject grant binding | fraction of operations bound to a verified grant for the exact source, asset, requesting subject and purpose | grant for source A or asset A used on source/asset B; one worker's grant reused by another |
| Purpose/action separation | correct enforcement of READ / PROCESS / CREATE_DERIVATIVE / REDISTRIBUTE_SOURCE / REDISTRIBUTE_DERIVATIVE | read-only used to process; source-copy grant used to publish a derivative; derivative grant used to redistribute raw source |
| Grant validity | enforcement of not-before, expiry, conditions and revocation | expired grant, unmet conditions, revocation with stale cache |
| Authorization coverage | authorized and executed requested sources / all requested sources | inaccessible or unknown sources silently counted as covered |
| Reproducibility | output digest and exact source, adapter, code, model, config, policy and query receipt | replay drift or missing receipt identity |
| Numerical parity | difference from same computation on authorized local reference | results vary beyond preregistered numerical tolerance |
| Data egress / disclosure | restricted-source marker leakage into network payloads, logs, caches, artifacts and outputs | restricted coordinates or attributes appear outside owner boundary |
| Revocation latency | time from verified revocation to denial of new use and invalidation of served derivatives | processing or publication succeeds after revocation acknowledgement |
| Owner burden / latency / cost | custodian review time, compute cost, data movement, query latency and retry rate | speed claimed by dropping inaccessible sources or reducing review |

Hard acceptance: zero unauthorized read/process/derive/source-redistribute/derivative-redistribute actions in the frozen negative-control corpus; zero silent promotion of UNKNOWN rights to permission; 100% of executed results bind the exact source, asset, requesting subject, purpose and source version. This scorecard does not prove privacy or legal compliance. Query metadata, timing, model updates and derived-output leakage require separate threat-modelled tests.

For source-local operation, the source owner retains raw data and runs a bounded adapter. Sol-Atlas receives only a permitted result plus evidence receipt. Mycelix verifies issuer/grant evidence and revocation; the local predicate only checks declared scope and must not be represented as signature verification or licence interpretation.

## 8. Relevant 2026 research frontier and what is still missing

The frontier already contains important individual components:

- **Probabilistic landslide stratigraphic modelling by uncertainty fusion of surface-wave inversion and borehole data** (Dengyi et al., Engineering Geology, online 7 October 2026, article 109128). The method retains an inversion ensemble, derives layer-specific coefficients of variation, and uses inverse-CV reliability weights to fuse surface-wave and borehole evidence into class-support and entropy maps. Under the paper's adopted configuration, it reports about 26% lower locally propagated false-positive area than uniform weighting. This is a bounded numerical/Enziping case result, not a general accuracy improvement. Crucial validation caveat: the two boreholes used in fusion cannot simultaneously serve as independent field truth; the authors also note that cross-source dependence is not explicitly modelled. Supporting data/code are available from the corresponding author on reasonable request, not as a public benchmark deposit. DOI: https://doi.org/10.1016/j.enggeo.2026.109128.
- **U-shaped multi-scale feature fusion network for three-dimensional geological modelling and uncertainty assessment** (Zhang et al., Engineering Geology, 21 May 2026, article 108727). The hybrid CNN/Transformer three-branch encoder integrates borehole and geophysical data, delineates five stratigraphic units and identifies sparsely distributed karst cavities in a field case. Monte Carlo dropout uncertainty concentrates near interfaces and cavity boundaries. The paper reports 74.68% accuracy for subsurface stratigraphic spatial extrapolation against validation boreholes. Treat that as a study-specific reference point—not a universal SOTA threshold. Reproduce the target labels, train/validation geography, borehole independence, class balance and uncertainty outputs before comparing. DOI: https://doi.org/10.1016/j.enggeo.2026.108727.
- **A hybrid recurrence–wavelet transform approach for geochemical classification of rocks along boreholes** (Anvari et al., Engineering Geology, 4 June 2026, article 108746). This is an adjacent borehole-analytics baseline for multivariate assay/sensor sequences, not the probabilistic landslide-fusion paper. Keep geochemical domain classification separate from 3D contact/stratigraphy reconstruction. DOI: https://doi.org/10.1016/j.enggeo.2026.108746.
- **Optimizing Geotechnical Site Characterization: A Value of Information Approach Using Coupled Spatial Random Fields and Bayesian Networks** (Fartosy et al., 2 June 2026). The paper reports up to 143% economic-efficiency improvement under its stated case-study assumptions. That is a case-specific result, not a transferable industry benchmark. Reproduce the loss model, cost assumptions, spatial correlation, sample constraints and baseline before comparing planners. DOI: https://doi.org/10.3311/PPci.43308.
- **Bayesian continual learning and dynamic uncertainty quantification for TBM geological prediction in non-stationary environments** (International Journal of Rock Mechanics and Mining Sciences, September 2026, article 106610). The study uses ring-level operational data from a real twin-line Shenzhen intercity tunnel. It reports static known-state accuracy of 84.76%, a cross-line transfer result of 45.40%, and higher sequential/transfer performance with replay-based updates (77.2% sequential, 79.9% cross-line in the reported experiments); EWC is reported at 82.1% for newly emerging geological information states. Importantly, its labels are operational proxy states, not strict lithological ground truth. Compare sequential adaptation, forgetting, transfer, unknown-state uncertainty and prospective utility—not accuracy alone. DOI: https://doi.org/10.1016/j.ijrmms.2026.106610.
- **Uncertainty-aware geological prediction in TBM tunneling: A probabilistic Bayesian framework with exploratory multi-source label construction** (Pan et al., Tunnelling and Underground Space Technology, April 2026). It introduces a probabilistic CNN for next-ring geological class prediction and decomposes epistemic/aleatoric uncertainty. Its exploratory labels are built from operational and geological information; this remains a prediction comparator rather than an independent geological truth source. DOI: https://doi.org/10.1016/j.tust.2025.107424.
- **Uncertainty-aware prediction and decision support for disc-cutter consumption in hard-rock TBM tunnelling** (Zhao et al., Transportation Geotechnics, June 2026, article 102039). It uses ring-scale prediction, conformal uncertainty intervals and cost-sensitive maintenance thresholds. This is a downstream machinery/maintenance comparator: evaluate calibration and decision utility separately from geological reconstruction. DOI: https://doi.org/10.1016/j.trgeo.2026.102039.

Therefore, the novelty claim must not be “we combine data,” “we use Bayesian uncertainty,” “we create entropy maps,” “we choose survey locations,” or “we update TBM models online.” Those are established research directions. A particularly important open benchmark gap is independent truth: some leading studies use boreholes as fusion inputs and then have no independent borehole validation, while TBM operational labels may represent machine–ground interaction proxies rather than lithology.

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
| Restricted / federated subsurface data | Owner-hosted portals, licensed data rooms and source-local analysis workflows | Execute only within exact owner grants; return reproducible bounded evidence receipts without assuming source-copy or derivative redistribution rights |

A fair SOTA table must record product/version, license/access, exact functions exercised, data contract, task and metric, hardware/runtime, domain, validation population, and independent result. Product pages are capability statements, not proof of accuracy or SOTA.

## 10. Proposed research gates

**Cross-paper reproduction targets (proposed hypotheses, not claims):**
- For a compatible stratigraphic extrapolation task, reproduce a clearly defined UFFN-style learned-fusion reference near its reported 74.68% study-specific validation accuracy before claiming any improvement. The numerical target is not portable unless labels, scale, geology, split, class balance and independent borehole truth are matched.
- For TBM transfer, reproduce the static-vs-cross-line degradation and uncertainty response on a compatible ring-level dataset before comparing continual-learning methods. The 84.76% known-state / 45.40% direct-transfer pair from one twin-line project is a transfer stress-test reference, not an industry-wide benchmark.
- For survey planning, compare fixed-spacing, documented expert design, cost-only, greedy uncertainty reduction and EVSI/ENBS under matched cost and authority constraints; report decision regret and field effort, not only entropy reduction.
- For all probabilistic outputs, require nominal 90% and 95% coverage with interval width/CRPS and subgroup calibration by site, geology, depth and modality. A pooled pass cannot hide a failed domain.
- Treat the first 20% decision-loss or survey-cost improvement as a stretch hypothesis to test after baseline variance is measured; require no degradation in hard safety, legal, rights, or coverage constraints.

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
