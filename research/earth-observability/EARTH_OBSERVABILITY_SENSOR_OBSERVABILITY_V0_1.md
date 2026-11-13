# EARTH-OBSERVABILITY-SENSOR-OBSERVABILITY-001
## Sensor observability envelope and blind detection scoring protocol v0.1

**Parent:** #771 — EARTH-OBSERVABILITY-QUAL-002  
**Status:** analytic synthetic runner implemented; physical sensor qualification NOT_RUN  
**Date:** 2026-11-13  
**Owner:** Sol-Atlas (contracts, evidence, scoring and receipts)  
**Supporting owner:** Symtropy (sensor physics, calibration and qualification)  
**Claim ceiling:** the current Rust contract and fixture validate software semantics and deterministic metric calculation only. The fixture is abstract, non-geographic and not based on GPR, ERT, seismic, EM, DAS or any other sensor physics.

---

## 1. Research shift: an envelope is a conditional capability claim

A survey footprint is not an observability claim. Deployment records where and when an instrument was used. An observability envelope says what target classes the system was qualified to detect, under what environmental and acquisition conditions, and with what demonstrated performance.

For a sensor configuration c, target class k, target descriptors x, environmental state e, and spatial/time support s, the long-term object is:

`POD(x,e,c,k,s) = P(detector_call=1 | target_present=1,x,e,c,k,s)`

This is not a single sensor rating. It is conditional on target size, depth, orientation, material contrast, host geology, moisture/conductivity, geometry, coupling, noise, acquisition settings, processing and calibration. The evidence graph should preserve the exact support and configuration to which a POD claim applies.

Detection, localization and interpretation are separate outcomes:

1. **Detection:** did the sensor produce a target indication?
2. **Localization:** how accurately was the indicated target positioned?
3. **Interpretation:** what does the evidence support about target identity, material and physical significance?

A strong detector can still have ambiguous interpretation. A correct interpretation can still be badly localized. These must not be compressed into one accuracy score.

## 2. Current executable Rust increment

The public branch adds:

- `sol-atlas-core/src/sensor_observability.rs`
  - `SensorObservabilityEnvelope` binds source evidence, sensor, configuration, calibration, target class, spatial support, validity interval, target-size/depth/contrast ranges, environmental covariate ranges and preregistered POD / false-alarm requirements.
  - `DetectionTrial` preserves target truth, detector call, target characteristics, environmental measurements, route length and localization error.
  - `validate_fixture` requires disjoint calibration and blind-test sites, acquisition IDs and target IDs. It rejects missing environmental covariates and invalid localization semantics rather than filling defaults.
  - `evaluate_blind_trials` excludes out-of-domain trials from in-domain POD and false-alarm denominators, and reports them separately.
  - It calculates descriptive POD, false alarms per surveyed route kilometre, localization median/P95 and site/acquisition cluster counts.
  - Cluster-aware confidence intervals remain an explicit `NOT_RUN` state; point estimates cannot qualify a physical sensor.
- `src/bin/sensor_observability_eval.rs`
  - offline JSON evaluator;
  - SHA-256 binding of exact fixture bytes;
  - deterministic receipt replay with `--check`;
  - explicit Rust typed-contract validation vs JSON Schema validation distinction;
  - physical qualification remains `NOT_RUN`.
- `research/earth-observability/fixtures/sensor-observability-analytic-v0.1.json`
  - abstract synthetic calibration and blind-test records;
  - separate in-domain and out-of-domain observations;
  - both positive targets and negative controls.
- `research/earth-observability/baselines/sensor-observability-analytic-baseline-v0.1.json`
  - pinned receipt checked by the workflow.
- `research/earth-observability/SENSOR_OBSERVABILITY_FIXTURE_V0_1.schema.json`
  - JSON Schema Draft 2020-12 structural interchange contract. The current CI uses the strict Rust typed contract; full JSON Schema validation remains NOT_RUN until an offline validator/version policy is added.

### Current analytic fixture result (not sensor evidence)

| Metric | Result | Meaning |
|---|---:|---|
| Blind in-domain trials | 14 | Excludes one out-of-domain observation |
| Out-of-domain trials | 1 | Conductive condition exceeds the declared envelope |
| Positive targets | 8 | Independent in-domain positive target instances |
| True positives | 7 | Detected in-domain targets |
| False negatives | 1 | Missed in-domain target |
| Point POD | 87.5% | 7/8; descriptive point estimate only |
| Negative controls | 6 | In-domain no-target route sections |
| False positives | 1 | One indication where no target was present |
| Surveyed negative-route distance | 1.5 km | Denominator for false alarms/km |
| False alarms | 0.667/km | 1 / 1.5 km |
| Localization median / P95 | 0.25 m / 0.28 m | Seven detected positive targets only |
| Site / acquisition clusters | 4 / 4 | Cluster counts, not confidence intervals |

The synthetic envelope sets a target POD requirement of 0.90 and maximum false alarms of 0.5/km. The analytic point estimates miss both requirements. The receipt therefore records `point_estimates_meet_declared_requirements=false`, `NOT_QUALIFIED_CLUSTERED_INTERVALS_NOT_RUN`, and physical qualification `NOT_RUN`.

The scorer also preserves site-level results rather than hiding them in one pooled number:

| Blind site | POD | False alarms/km |
|---|---:|---:|
| blind-site-a | 100% (2/2) | 2.0 |
| blind-site-b | 100% (2/2) | 0.0 |
| blind-site-c | 100% (2/2) | 0.0 |
| blind-site-d | 50% (1/2) | 0.0 |

Four site clusters are only a software fixture. The site rows are deliberately small and unstable; they demonstrate the need for stratification, not site-level performance confidence. The pooled POD is 87.5%, but that value alone conceals the blind-site-d miss and blind-site-a's nuisance indication burden.

This is the intended behavior: a harness should preserve failures and claim limits, not optimise for a green headline. The fixture does not establish any GPR performance and must not be quoted as a field POD or localization result.

## 3. Statistical method and current limitation

### 3.1 Reuse established POD methods

The correct baseline is not to invent a new POD formula. Relevant established NDE practice includes:

- **ASTM E2862-23**, hit/miss probability-of-detection analysis. It defines POD(a) for a defined discontinuity size and a controlled, documented examination procedure. Its scope assumes known specimen size/distribution and an unbiased, controlled examination. It allows justified multi-predictor models.
- **ASTM E3023-21**, continuous `a-hat` versus `a` POD analysis, where the system response is measurable and continuous and the inspection system is calibrated.
- **MIL-HDBK-1823A**, Nondestructive Evaluation System Reliability Assessment, with validation notices including Notice 2 (2018). It describes planning, specimen design, data acquisition, POD curves and confidence bounds for NDE systems.

These methods are an excellent statistical foundation, but geophysical field surveys introduce spatial clustering, site heterogeneity, uncertain ground truth, acquisition overlap, correlated noise, moisture/weather and environmental domain shift. The method must be adapted without pretending that NDE specimen assumptions automatically hold for field geophysics.

### 3.2 Next statistical increment

The v0.1 scorer now includes an optional deterministic **site-cluster percentile bootstrap** for POD and false alarms/km. It resamples entire independent site blocks rather than treating individual traces as iid. The CLI's pinned analytic receipt sets a preregistered pilot gate of 10 independent site clusters for each rate; the current fixture has four for POD and four for false alarms, so its receipt returns `INSUFFICIENT_SITE_CLUSTERS_POD_4_FAR_4_REQUIRED_10_NOT_RUN` and no confidence intervals.

The value 10 is a transparent, provisional run configuration for this software benchmark, not a statistical standard or a recommended universal sample size. A physical experiment must derive its cluster count through power/precision simulation based on target prevalence, intra-site correlation, acceptable confidence-bound width and the intended deployment population. A small synthetic test exercises deterministic resampling when the configured minimum is met, but does not validate inference for a field campaign.

The next statistical increment must extend beyond the current site-block percentile bootstrap:

- binomial hit/miss POD curves when the detector output is binary;
- continuous-response `a-hat`/signal-response models where the instrument provides a measurable response;
- target-size/depth/contrast/environment covariates with monotonicity or physically justified constraints where appropriate;
- multi-level site/campaign/target/acquisition correlation models where data structure requires them;
- lower confidence bounds for POD and upper confidence bounds for false-alarm burden;
- separate calibration, model-selection and blind-test data;
- model checking, residual diagnostics and sensitivity to the detection threshold.

A pooled iid binomial interval is not acceptable if multiple test records come from the same site, target or acquisition. The current CLI refuses to produce intervals when its preregistered site-cluster threshold is unmet.

Do not choose a universal POD floor for all Earth targets. The acceptable lower bound and false-alarm ceiling depend on consequence, target type and the cost of misses vs nuisance indications. Freeze those values before the blind test.

### 3.3 Experimental design

Each physical POD study needs:
- a precise target definition and reference truth with uncertainty;
- representative target size, depth, orientation, material and contrast distributions;
- environmental strata representative of the intended deployment domain;
- exact instrument, firmware, antenna/frequency/transmitter, gain, sampling, speed, geometry and coupling;
- calibration and processing identities;
- blind independent target labels and locked analysis plan;
- enough independent target/site/acquisition clusters to attain predeclared confidence-bound precision;
- negative controls distributed over the surveyed area/route/time to estimate false-alarm burden;
- a plan for missing, inaccessible, damaged or ambiguous reference targets;
- an external site/campaign for transfer assessment.

Use power / precision simulation before fixing sample size. The 8 positives and 6 negative controls in the current fixture are software smoke-test counts, not a defensible physical POD study.

## 4. Observability state semantics

A no-call is not one state. Sol-Atlas should represent:

| State | Meaning | Evidence of absence? |
|---|---|---|
| Detected | Detector indication exists; identity can remain ambiguous | No |
| No detection within qualified envelope | A target was inside the qualified domain and detector made no call | Only bounded, target-specific evidence |
| Below detection limit | Target lies outside demonstrated size/depth/contrast sensitivity | No |
| Unobserved | No admissible acquisition covers the target support | No |
| Out of domain | Environment/configuration is outside the validated envelope | No |
| Unqualified | Required calibration, metadata, source or instrument evidence is missing | No |
| Conflicted | Admissible observations or interpretations disagree | No |

“No detection within qualified envelope” is not itself proof of absence. It is evidence only for the tested target class, size/depth range, environmental condition, spatial support, acquisition configuration and detection confidence. A universal “no utility here” claim cannot be derived from a GPR no-call.

## 5. Benchmark measures and target gates

| Track | Measure | Proposed gate |
|---|---|---|
| POD | target-specific POD curve and lower confidence bound | preregister by consequence class |
| False alarm | false calls per route-km / area / observation-hour | preregister burden ceiling |
| Localization | median/P95 error and spatial uncertainty coverage | compare with downstream decision tolerance |
| Detection limit | target size/depth/contrast threshold at specified POD | report threshold with confidence bound |
| Calibration | calibration by site, geology, moisture, depth and sensor setting | no pooled-only calibration claim |
| Domain transfer | leave-site, campaign and instrument/configuration out | abstain or widen uncertainty under shift |
| Efficiency | acquisition cost, crew-hours, elapsed time and false-alarm follow-up burden | equal-budget comparisons |
| Integrity | invalid absence promotions and split leakage | zero negative-control promotions; zero test leakage |

Do not compare GPR, ERT, seismic, EM, DAS and LiDAR as one sensor leaderboard. Compare methods only on the same target, truth support, environment, cost and decision. A modality can be superior for one target and unsuitable for another.

## 6. Standards and interoperability

- OGC SensorThings API 1.1: observation and sensor entities for the sensing side.
- OGC SensorML 3.0: sensor/process/component description, measurement and post-measurement transformation.
- OGC API – Connected Systems 1.0, Parts 1 and 2: static system/deployment metadata connected to dynamic observations and events.
- OGC API – EDR 1.2: spatiotemporal query for position, area, cube, trajectory, corridor, location and item; publication 8 September 2026.
- GeoSciML 4.1 / GWML2: geological and groundwater feature context.

SensorThings / Connected Systems / SensorML should carry interoperable sensor, deployment, procedure and observation information. The Sol-Atlas envelope is an evidence-qualified profile above those encodings, not a claim that the OGC standards themselves define a universal POD model. Full conformance is a separate test.

## 7. SOTA and industry comparison

### Statistical capability

The SOTA baseline for POD is not simple classification accuracy. ASTM E2862-23 and E3023-21 already cover hit/miss and continuous-response POD analysis. Sol-Atlas must use them as baselines, reproduce suitable methods and add environmental/spatial transfer validation.

The research contribution is the extension from controlled NDE specimens to a spatially indexed, target/environment/configuration-specific envelope, coupled to:
- independent spatial/site/campaign holdouts;
- explicit blind zones and non-detection states;
- spatially clustered uncertainty;
- rights-aware source admission;
- downstream engineering decision/regret;
- immutable revalidation when new field evidence arrives.

### Product landscape

**Bentley OpenGround** provides mature geotechnical data capture, factual borehole/CPT/laboratory data management, reporting, configurable data structures, AGS 3.1/4.x and DIGGS exchange, and REST API workflows. Integrate with factual data management rather than reimplementing it.

**Seequent Leapfrog Works** supports dynamic 3D modelling from boreholes, GIS, 2D grids/maps, geophysical data, and engineering/environmental integration. Treat it as the geological modelling comparator, not a platform to dismiss without matched testing.

**GIS, point-cloud and survey systems** already provide mature spatial geometry, point-cloud registration, raster/vector processing and visualization. Sol-Atlas should add evidence semantics, observability, metrology/uncertainty and revalidation across tools.

Our positioning is an interoperability-first evidence and decision layer. Benchmark an industry product only where a lawful licence and an identical data/workflow pack are available. An untested proprietary comparator remains `NOT_RUN`, not a presumed loss.

## 8. Next implementation tasks

**P0 — finish current execution**
- [x] Typed envelope, trial, split and metric contracts.
- [x] Calibration vs blind-test split isolation by target, site and acquisition.
- [x] Analytic fixture and pinned SHA-256 receipt.
- [x] Offline Rust scoring CLI.
- [x] Point POD, false alarms/km, localization summary and out-of-domain exclusion.
- [ ] Exact-head formatting, compilation and tests on the new child PR.
- [ ] Confirm the AGS4 parser correction and original PR #762's full native suite.

**P0 — statistical qualification**
- [ ] Add cluster-aware POD confidence bounds and false-alarm upper bounds.
- [ ] Implement hit/miss POD curve fitting and continuous-response `a-hat` baseline; validate against ASTM E2862/E3023 examples where lawful access permits.
- [ ] Stratify target size/depth/contrast and environment; test nonlinear and interaction effects.
- [ ] Add power/precision simulation to select physical sample counts.

**P1 — sensor physics and field datasets**
- [ ] Symtropy owns forward models for GPR, ERT, surface-wave/seismic and DAS; each includes model discrepancy and non-uniqueness controls.
- [ ] Validate simulation against independently measured physical data.
- [ ] Add environmental shifts: conductive wet clay, moisture, layering, clutter, coupling loss and geometry changes.
- [ ] Acquire permissioned physical trials with blind reference truth and competent survey/geophysics review.

**P1 — decision tournament**
- [ ] Connect sensor envelopes to candidate investigation actions.
- [ ] Compare fixed-grid, expert, random, cost-only, entropy-greedy and EVSI/ENBS methods at equal feasible budgets.
- [ ] Evaluate decision regret, false-alarm follow-up burden and the value of additional measurements.

The current work is an executable analytic harness and a typed envelope contract. It is not a trained POD curve, a physical forward model, a field-validated sensor or a qualified geological survey.
