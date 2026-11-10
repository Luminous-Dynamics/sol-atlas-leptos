# GMD Forecast Benchmark and SOTA Comparison Protocol v0.1

**Date:** 2026-11-10  
**Status:** Research protocol; proposed gates are not achieved results.  
**Claim ceiling:** E0 until independent baseline reproduction and frozen event/site holdouts pass.

## 1. Why this benchmark exists

The field now has at least three distinct kinds of evidence that must not be collapsed:

1. **Short-horizon GIC forecasting:** e.g. the 2026 MM-CNN-MHA result at one 220-kV substation in Fujian, forecasting 60 minutes ahead.
2. **Event reconstruction and model comparison:** e.g. the May 2024 US GIC study with measured current, operator-calculated current and a reference model at multiple sites.
3. **Operational consequence and mitigation:** e.g. the UK SWIMMR operational multi-infrastructure programme and the New Zealand Transpower Gannon-storm mitigation exercise.

These are complementary, not interchangeable. A model that reproduces a historical trace has not demonstrated future forecast skill. A model with low pointwise RMSE has not necessarily detected a rare peak. A lower predicted GIC at one transformer does not prove the network or essential services became safer.

The benchmark must therefore evaluate a complete, evidence-bearing chain while keeping each model and claim separately qualified:

`solar-wind / geomagnetic forecast
 -> local magnetic field
 -> local geoelectric field
 -> induced voltage on exact conductor paths
 -> topology-conditioned GIC
 -> transformer-specific response
 -> network and essential-service consequence
 -> owner-approved mitigation options`

## 2. Current industry and research comparators

### 2.1 MM-CNN-MHA: 60-minute GIC forecast, Fujian

Zhang et al., *Forecasting Geomagnetically Induced Currents From Solar Wind and Geoelectric Field Data Using Deep Learning*, Space Weather, first published 26 September 2026. DOI: https://doi.org/10.1029/2025SW004900.

The reported setup:
- One 220-kV Huangmeishan substation in Fujian, China.
- One-minute cadence, 120-minute historical context, next-60-minute forecast.
- Solar-wind, geomagnetic and measured geoelectric-field inputs.
- Direct GIC prediction and an E-field-mediated physical calculation were compared.
- The paper reports 226,641 valid samples, a chronological 80/10/10 time-series split, and event evaluations including March and December 2015.
- Direct GIC model time-series test RMSE is reported as 0.27 A. In the 60-minute event comparison, direct-GIC RMSE is approximately 0.28 A, persistence also approximately 0.28 A, and the predicted-E-field / physical-model route approximately 0.26 A.

Important limitations:
- Empirical GIC coefficients were calibrated using March 2015; the authors explicitly state that the March comparison is not fully independent. December 2015 is the independent event check.
- The observed March 2015 peak of 3.28 A was forecast as approximately 0.6 A by the direct-GIC model and 1.03 A by the E-field-mediated model. Sparse high-amplitude training examples contributed to peak underprediction.
- This is a strong matched-site reference result, not a universal SOTA RMSE threshold. Do not compare 0.27 A against a different station, sensor, horizon, current definition or transformer configuration.

Reproducibility leads:
- Paper DOI: https://doi.org/10.1029/2025SW004900
- Software repository (MIT licence declared): https://github.com/zhangxuejianll/predicting-GIC-package
- Software DOI: https://doi.org/10.5281/zenodo.20381320
- Dataset DOI: https://doi.org/10.5281/zenodo.20380712
- Additional Huangmeishan 2015/2016 observations: https://doi.org/10.5281/zenodo.3714269

The GitHub package is small and includes an RAR archive; pin the exact commit and archive digest and inspect the actual release package before claiming a full reproduction. The retrieved Zenodo metadata for the additional 2015/2016 measurements did not establish a clear reuse licence. Do not redistribute or train commercial models on those data until their terms are confirmed.

### 2.2 May 2024 US GIC measurements and published baselines

Wilkerson et al., *GIC-Related Observations During the May 2024 Geomagnetic Storm in the United States*, Space Weather (2026), DOI https://doi.org/10.1029/2025SW004758.

The public companion repository contains:
- 1-second measured TVA GIC at Bull Run, Montgomery, Union and Widows Creek;
- 1-minute TVA operator-computed GIC at the same sites;
- GMU reference-model median GIC;
- additional NERC measured sites and magnetometer products.

Sol-Atlas has provisionally reproduced the final publisher table from a pinned companion-repository commit and exact raw files. See [GMD_MAY2024_TVA_REPLAY_V0.1.md](GMD_MAY2024_TVA_REPLAY_V0.1.md), [machine-readable results](results/gmd_may2024_four_site_replay_v0_1.json), and [replay issue #695](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/695).

That replay is **data-pipeline / metric reproduction**, not independent physical-model validation. It provides useful matched baselines and realistic amplitude-bias diagnostics.

### 2.3 UK SWIMMR: operational multi-infrastructure capability

Beggan et al. (2025), DOI https://doi.org/10.1029/2025SW004364, describe nine operationalised models for space-weather impacts on the British high-voltage electricity network, high-pressure gas pipelines and railway network. The programme also supported new variometer observations and magnetotelluric measurements at 53 British sites.

This is an operational-system comparator. It is not simply a neural model score. Luminous should aim to connect the same categories of physical and infrastructure evidence while making provenance, uncertainty, rights, model validity and offline coordination explicit.

### 2.4 Transformer response and duration

- The May 2024 New Zealand Halfway Bush study reports transformer GIC episodes above 50 A and a maximum of 113 A, alongside harmonic and reactive-power observations. DOI https://doi.org/10.1029/2024SW004235.
- A 2026 New Zealand study reports up to approximately 20 A sustained for roughly 90 minutes during a moderate storm. DOI https://doi.org/10.1029/2026SW005032.
- These measurements demonstrate that peak-only forecasting is insufficient. The thresholds are specific to the observed transformers and operating conditions; they are not universal damage thresholds.

### 2.5 Operational mitigation as a comparator

The New Zealand Transpower Gannon-event paper documents an operator-approved, targeted line-disconnection mitigation strategy used during the May 2024 G5 storm. The retrospective compares observed operation with a modelled no-mitigation counterfactual. See Mac Manus et al. (2025), DOI https://doi.org/10.1029/2025SW004388, and the requirements in [Sol-Atlas GMD-MITIGATION-PLANNING-001 #703](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/703).

This establishes a much more meaningful downstream challenge than merely minimising one asset's current: reduce network-wide asset stress without compromising supply, protection, voltage security or essential services.

## 3. Benchmark task separation

Never combine the following tasks into a single score.

| Task | Question | Valid evidence | Not established |
|---|---|---|---|
| Nowcast / event reconstruction | Can the system reproduce a known event from contemporaneous observations? | Matched sensors, event traces and published baselines | Future forecast skill |
| 15 / 30 / 60-minute forecasting | Can it predict future GIC / E-field using only information available at issue time? | Chronological event-held-out forecasts | Transformer safety |
| Spatial transfer | Does local risk remain calibrated at unseen sites or under sparse station coverage? | Geographic and station-held-out events | National coverage from one region |
| Transformer response | Does predicted winding GIC correctly predict saturation, harmonics, reactive power and thermal evolution? | Unit-specific measurements / OEM parameters | Generic thresholds for all transformers |
| Mitigation value | Does a plan reduce system-wide risk while preserving essential services? | Owner-reviewed grid, service and exercise outcomes | Autonomous authority to switch equipment |

## 4. Frozen forecasting protocol

### 4.1 Common task definition

Primary benchmark:
- Native source cadence: 1 minute where available.
- Historical context: 120 minutes.
- Forecast horizons: 15, 30 and 60 minutes.
- Optional 180-minute horizon only where upstream observation cadence, L1 data latency and event count justify it.
- Separate signed-current forecasts from absolute-magnitude forecasts. Signed current requires explicit sensor orientation and transformer/winding convention.
- Separate direct-GIC models from field-first hybrid models.

The MM-CNN-MHA comparison is admissible only when its time horizon, site, sensor definition, inputs, feature latency and train/test event design are matched.

### 4.2 Leakage-resistant splits

1. Partition by event, site and region **before** creating input/target windows, normalising, imputing, augmenting or fitting any feature transforms.
2. Purge at least the full lookback plus forecast horizon around all split boundaries.
3. Keep all overlapping windows and augmented derivatives from an event in the same partition.
4. Use rolling-origin evaluation, leave-one-event-out, leave-one-site-out and leave-one-region/network-out.
5. Freeze a final blind event/site/region holdout; do not use it for threshold selection, calibration, feature engineering, conductivity fitting, hyperparameter search or model selection.
6. An event used to fit empirical physical coefficients is not an independent event holdout, even if its ML windows are nominally separated.
7. Report event count, site count, missingness, effective independent storm blocks and provenance for each split. Minute rows are not IID samples.

The March 2015 calibration/evaluation overlap acknowledged by the MM-CNN-MHA paper is a required negative control for benchmark design: a result cannot be labelled independent if calibration or model coefficients consumed the holdout event.

### 4.3 Dataset/event portfolio

| Family | Role | Qualification status |
|---|---|---|
| Huangmeishan 2015 / 2016 | Reproduce published one-site 60-minute forecast and event transfer | Public DOI references; licence and exact feature compatibility must be checked |
| US/Tennessee Valley May 2024 | Multi-site measured/operator/reference replay | Four-site data-pipeline reproduction provisionally recorded; not independent event forecast validation |
| South African Grassridge / Matimba / Botrivier | Southern-hemisphere and regional transfer challenge | Published history exists; raw data rights, sensor calibration, polarity, cadence and network revisions remain owner/data-audit obligations |
| New Zealand | Transformer-response, long-duration and mitigation consequence validation | Published observations and event study; owner-controlled raw data require permission |
| Synthetic networks / E-field cases | Analytical oracle and negative controls | Known truth only within declared synthetic assumptions |

### 4.4 Input-time integrity

At every forecast issue time, record which values were actually available, not merely the physical timestamp of the data:
- L1 solar-wind observations: spacecraft-to-Earth propagation and packet latency;
- official model forecast issue time and revision;
- magnetometer timestamp, arrival time, sample cadence and calibration state;
- E-field observation/model validity and local transfer-function coverage;
- network topology/transformer parameter revision known at the issue time;
- missing/stale data and clock uncertainty.

A model cannot consume future-available observations during retrospective replay. Future information leakage can occur through delayed ingestion, source revision, retrospective quality flags, backfilled data and post-event calibration.

## 5. Mandatory baseline tournament

All learned and physical baselines must use the same prediction issue time, forecast horizon, available inputs, target definition and evaluation split.

1. Persistence / last-value hold.
2. Training-only climatology or event-conditioned seasonal baseline, where sample counts permit.
3. Linear autoregression, ridge or elastic-net baseline.
4. Gradient-boosted trees (pinned implementation, features and hyperparameters).
5. Published MM-CNN-MHA, reproduced from pinned source, weights/config and licensed inputs where possible.
6. LSTM/GRU with matched context and training budget.
7. Direct GIC prediction from solar-wind / geomagnetic / local observations.
8. E-field forecast followed by the same qualified physical network-GIC solver.
9. Physics-only calculation driven by contemporaneous measured E-field. This is an information-rich reference, **not** a deployable future forecast if the future E-field was not available at issue time.
10. Source-native operational/utility forecast or current documented operator procedure, when an owner permits access.

No SOTA claim from an unmatched architecture or incomparable metric. Report training compute/data, model parameter count, input availability, horizon, event count, inference latency and energy/cost if relevant.

## 6. Metrics

### 6.1 Continuous forecast quality
- MAE, RMSE, signed mean error/bias, nRMSE, prediction efficiency and Pearson correlation.
- Report raw and explicitly polarity-normalised versions; never infer a sign flip silently from the test set.
- Peak magnitude error, peak timing error, rise-rate error and duration error.

### 6.2 Probabilistic forecast quality
- CRPS, Brier score for predeclared threshold exceedance, weighted interval score.
- 50 / 80 / 90 / 95% interval coverage and interval width.
- Calibration by event, site, horizon, amplitude regime and season.
- Use event-block bootstrap or other dependence-aware uncertainty. Do not bootstrap individual minutes as IID.
- For conformal or residual calibration, fit only on earlier events / earlier chronological segments and track regime shift.

### 6.3 Event and tail performance
- POD, POFD, HSS, CSI, precision, recall, F1 and false alarms per day.
- Warning lead time and missed-exceedance duration.
- Peak-current amplitude/timing error.
- Duration above owner-defined, asset-specific operating envelopes.
- Separate sustained moderate episodes from short peaks.
- Thresholds must be predeclared by asset owners / test protocol, not tuned after seeing holdout labels.

### 6.4 Transfer and robustness
- Leave-one-event/site/region-out performance degradation.
- Worst-event and worst-site error, not only pooled averages.
- OOD detection, abstention coverage and selective risk.
- Sensor outage, stale/missing packets, clock drift, polarity change, relocation, clipping, calibration drift and station/source-distance sensitivity.
- Ground-conductivity / MT support gaps, topology changes, unknown grounding, transformer parameter mismatch and source-model revisions.
- Source correlation / common-cause dependence; repeated measurements from the same station do not create independent evidence.

### 6.5 Physical and service consequences
Only score where corresponding specialist model and measurements exist:
- E-field vector error and direction;
- induced-voltage and GIC peak / waveform error;
- effective winding GIC;
- harmonic amplitude/phase and reactive-power response;
- transformer thermal trajectory and model residual;
- essential energy not served, critical-service interruption duration and restoration time;
- mitigation feasibility, tail service risk and distributional impact.

A lower GIC RMSE cannot compensate for worse calibration, worse tail detection, or an unsafe network/service outcome.

## 7. Proposed acceptance targets

These are project gates, not demonstrated results or universal field standards.

| Gate | Proposed target | Qualification condition |
|---|---|---|
| Baseline reproducibility | Reproduce published MM-CNN-MHA / persistence values within published rounding tolerance | Exact source commit, weights/data version, feature and evaluation protocol pinned |
| Leakage | Zero future-information leakage, cross-split windows, source/label leakage | Independent split audit and mutation controls |
| Forecast improvement | At least 5% lower aggregate MAE or CRPS than strongest admissible baseline | Frozen event/site holdout, paired event-block 95% CI excludes zero |
| Tail safety | Non-inferior or better POD/HSS at predeclared high-amplitude thresholds | False-alarm rate also reported and bounded |
| Calibration | Nominal 90% intervals achieve initial 85–95% empirical coverage | Adequate held-out event/site sample counts; worst-stratum coverage published |
| Regression control | No major event/site stratum >10% relative MAE regression without documented risk trade-off | Strata and exception rule pre-registered |
| Robustness | Missing/stale/out-of-domain inputs cause abstention or conservative widening | Zero false low-risk disposition in negative controls |
| Operational relevance | Improvements survive physical, transformer and service-level consequence metrics | Specialist models and owner data independently qualified |

If event counts are too small, report the benchmark as inconclusive. Do not weaken the split or thresholds to force a pass.

## 8. The Luminous research hypothesis

The opportunity is not to out-CNN a single-site time series. Test whether a **physics-conditioned, multi-fidelity, evidence-gated forecasting and planning architecture** can improve spatial transfer, tail coverage and consequence-aware decisions.

- Symtropy owns Earth-response / E-field / line-integral / quasi-DC GIC and transformer-specific specialist physics.
- Sol-Atlas owns spatial asset topology, MT and station footprints, sensor/site provenance, dependency graph, data coverage and service criticality.
- Symthaea owns probabilistic assimilation, source disagreement, calibration validity, OOD/abstention, tail-risk reasoning and counterfactual comparison.
- Mycelix owns federated identity, offline incident evidence exchange, partition-safe reconciliation and auditable human approval records.

Use a model tournament rather than an assumed winner. The hybrid approach should beat simple baselines on held-out events and transfer tasks before being embedded in Symthaea. Any calibration state expires or is requalified when sensors, station locations, polarity, MT transfer functions, topology, transformer parameters, model revision or rights change.

## 9. South African path

South Africa is a promising region-transfer challenge, not a presumed-safe region. The existing literature documents Grassridge/Hermanus GIC modelling, Matimba records and Botrivier under-line magnetic measurements; it does not by itself establish a complete open national transformer-neutral GIC dataset.

SANSA SANDIMS provides an important source inventory route, but academic access and commercial product rights differ. Sol-Atlas must confirm source-level access, commercial rights, station calibration, raw cadence, sensor orientation, and whether direct GIC rather than magnetic proxy is available. Utility topology, transformer design, neutral CTs and operating records require explicit owner permission.

Start with historical-data rights and metadata audit before a sensor deployment or live data pipeline. See [GMD South African Data Access Matrix](GMD_SOUTH_AFRICA_DATA_ACCESS_MATRIX_V0.1.md).

## 10. Immediate implementation sequence

1. Pin the MM-CNN-MHA software repository commit and archive digest; establish a source-compatible environment without copying or redistributing raw data.
2. Confirm exact public data version, licence and feature alignment; reproduce the published baseline before adding Luminous models.
3. Freeze a versioned event/site split manifest, purge windows and negative-control IDs.
4. Extend the May 2024 replay tool (#699) only as a data-ingestion/metrics oracle; do not use it as the final forecast test event if used for calibration.
5. Implement simple, strong statistical and tree baselines before new HDC/CfC model work.
6. Compare direct GIC with E-field-first physics using matched issue-time inputs.
7. Add uncertainty intervals, high-amplitude and sustained-current event metrics, and abstention/OOD behaviour.
8. Add South African regional transfer only when raw data, rights and topology/asset revisions are qualified.
9. Connect validated forecasts to transformer response (#4528) and operator-approved mitigation planning (#703) only after those specialist layers pass.

## 11. Claim ceiling

This protocol does not establish that a Luminous model outperforms MM-CNN-MHA, NOAA, SANSA, SWIMMR or a utility model. A passing forecast benchmark does not establish transformer thermal safety, grid stability, NERC compliance, South African system risk or operational authority. The first credible claim is narrowly scoped to the event/site/horizon/model regime demonstrated by reproducible held-out evidence.
