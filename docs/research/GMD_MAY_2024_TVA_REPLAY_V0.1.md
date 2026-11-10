# May 2024 TVA GIC replay — preliminary reference reproduction v0.1

**Status:** Preliminary exploratory replay; not yet a packaged executable benchmark.  
**Date:** 2026-11-10  
**Claim ceiling:** Research reproducibility check only. This is not independent physical-model validation, operational qualification, or a new SOTA claim.

## 1. Why this replay matters

The open-access 2026 *Space Weather* article by Wilkerson et al. compares measured GIC against (a) TVA operator calculations at four substations and (b) a reference model across 47 sites for the May 2024 geomagnetic storm. It provides a concrete, recent, measured-data baseline rather than requiring us to invent a synthetic performance claim.

We independently recalculated the four-site Table 1 metrics from the public companion data. The resulting (r^2) and prediction-efficiency values reproduce the published rounded results at all four sites.

This first replay qualifies the data alignment and metric semantics. It does not improve on the TVA operator model, which remains the stronger baseline at these four sites.

## 2. Pinned source data

- Paper: Wilkerson et al., *GIC-Related Observations During the May 2024 Geomagnetic Storm in the United States*, **Space Weather**, first published 19 May 2026, DOI [10.1029/2025SW004758](https://doi.org/10.1029/2025SW004758).
- Companion data repository: [lucywilkerson/SWERVE-2024-05-10](https://github.com/lucywilkerson/SWERVE-2024-05-10).
- Repository commit used for this pass: `089617756920bcc9df621f7f2299ba0644320cc3` (pin this commit; do not follow moving `main` for a frozen benchmark).
- Repository/Zenodo metadata declares MIT; Zenodo DOI: [10.5281/zenodo.20090223](https://doi.org/10.5281/zenodo.20090223). Verify individual upstream data-provider conditions before redistribution or commercial use.
- The paper reports measured GIC at 47 sites, magnetometer data at 17 sites, TVA operator calculations at four sites, reference-model GIC at 47 sites, and global magnetospheric-model ΔB at 12 magnetometer sites.

### Input files

For each of the four matched TVA sites:

- Measured 1-second GIC:
  - `data_original/tva/gic/GIC-measured/gic-bullrun_20240510.csv`
  - `data_original/tva/gic/GIC-measured/gic-montgomery_20240510.csv`
  - `data_original/tva/gic/GIC-measured/gic-union_20240510.csv`
  - `data_original/tva/gic/GIC-measured/gic-widowscreek2_20240510.csv`
- TVA operator model, one minute, split into daily files:
  - `data_original/tva/gic/GIC-calculated/20240510_BullRunXfrmGIC.dat` and corresponding 11/12 May files;
  - `20240510_MontgomeryGIC.dat` and corresponding 11/12 May files;
  - `20240510_UnionGIC.dat` and corresponding 11/12 May files;
  - `20240510_WidowsCreek2GIC.dat` and corresponding 11/12 May files.
- Reference-model one-minute GIC:
  - Bull Run: `data_original/gmu/tva/site_108137138.csv`
  - Montgomery: `data_original/gmu/tva/site_708931360.csv`
  - Union: `data_original/gmu/tva/site_98699542.csv`
  - Widows Creek: `data_original/gmu/tva/site_14133615.csv`

The TVA measured and calculated data are separate from the GMU reference-model series. Do not substitute the reference model's nearest-sensor column for the actual TVA measured series in this four-site comparison.

## 3. Frozen evaluation protocol reproduced

- Evaluation window: 2024-05-10 15:00 UTC through 2024-05-12 06:00 UTC, inclusive.
- Measured series: arithmetic mean of each UTC-aligned, non-overlapping 60-second bin from the 1-second data.
- Required samples: exactly 60 measured values in each one-minute bin.
- Model series: one-minute model records joined by exact UTC minute.
- Matched minutes: 2,341 per site and model.
- Pearson correlation: raw signed correlation (r).
- (r^2): square of raw Pearson correlation, unaffected by sign.
- Prediction efficiency:
  [
  PE=1-\frac{\sum_t(y_t-\hat y_t)^2}{\sum_t(y_t-\bar y)^2}
  ]
- Sensor polarity: where raw (r) is negative, the paper multiplies measured GIC by (-1) for PE, assuming the measurement sensor orientation is opposite to the model convention. We applied that same evaluation convention to reproduce the paper. We did **not** change raw (r) or (r^2).

**Important future benchmark rule:** in a new benchmark, sensor polarity must come from frozen instrument metadata or an independently documented calibration record. Do not infer or select polarity by maximizing correlation on the evaluation set. The sign adjustment here is a reproduction of the published protocol, not a general license to optimize sign per test site.

## 4. Reproduced results

### 4.1 TVA operator model

| Site | Raw (r) | (r^2) reproduced | (r^2) published | PE after published polarity convention | PE published | Measured σ (A) | TVA σ (A) | Aligned RMSE (A) | Aligned MAE (A) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Bull Run | -0.8941 | 0.7994 | 0.80 | 0.4979 | 0.50* | 1.505 | 2.167 | 1.066 | 0.763 |
| Montgomery | -0.9294 | 0.8637 | 0.86 | 0.4298 | 0.43* | 3.704 | 1.003 | 2.797 | 1.475 |
| Union | -0.8178 | 0.6688 | 0.67 | 0.6515 | 0.65* | 3.752 | 3.548 | 2.215 | 1.547 |
| Widows Creek | -0.8190 | 0.6708 | 0.67 | 0.6315 | 0.63* | 1.942 | 1.212 | 1.179 | 0.825 |

### 4.2 Reference model

| Site | Raw (r) | (r^2) reproduced | (r^2) published | PE after published polarity convention | PE published | Reference σ (A) | Aligned RMSE (A) | Aligned MAE (A) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Bull Run | +0.6930 | 0.4802 | 0.48 | 0.4753 (no flip) | 0.48 | 1.146 | 1.090 | 0.711 |
| Montgomery | -0.3921 | 0.1537 | 0.15 | 0.1520 | 0.15* | 1.300 | 3.411 | 1.847 |
| Union | -0.7544 | 0.5691 | 0.57 | 0.4981 | 0.50* | 1.831 | 2.658 | 1.808 |
| Widows Creek | -0.5282 | 0.2790 | 0.28 | 0.2272 | 0.23* | 0.584 | 1.708 | 1.135 |

`*` denotes the published sign correction for the measured sensor orientation.

The reproduced values round to the published Table 1 values for all eight model/site comparisons. Small differences in unrounded values are expected from data precision and implementation details; the rounded table agreement is the present reproduction criterion.

## 5. Interpretation — what the replay actually establishes

1. The data paths, station mapping, UTC minute joins, one-minute aggregation and sign convention are consistent with the paper's Table 1.
2. The TVA operator model reproduces (r^2) of approximately 0.67–0.86 and polarity-adjusted PE of approximately 0.43–0.65 at these four stations during this one event.
3. The generic reference model is materially weaker at these four paired sites: (r^2) approximately 0.15–0.57 and PE approximately 0.15–0.50.
4. Correlation alone is insufficient. The sign can be reversed by instrument orientation; (r^2) discards sign; PE responds to magnitude error. Future comparisons also need absolute error, peak error, timing error, interval coverage and site-level worst cases.
5. The operator model is not uniformly superior on every error measure. For example, at Bull Run the reference model's MAE is lower in this replay, while its (r^2)/PE are slightly lower. Keep the complete metric vector rather than selecting only the metric that favours a model.
6. One event and four matched operator sites do not establish geographic generalization, Carrington-class performance or SOTA superiority.

## 6. Important limitation discovered: evaluation is not operational forecasting

This is an event reconstruction/reanalysis: the geomagnetic event has already happened and the published model outputs are compared with the same event measurements. It does not establish that a model can forecast CME arrival, local E-field or GIC with operational lead time.

Keep three distinct benchmark tracks:

- **Retrospective reconstruction:** estimate the historical E-field/GIC given observations from the event.
- **Nowcast:** estimate present state using only data available at that timestamp.
- **Forecast:** predict future state using only information available at issue time, at declared lead times (e.g. 15 min, 1 h, 6 h, 24 h, 48 h).

Each track must freeze its information cutoff. Future measurements, retrospective model products and post-event correction data must not leak into nowcast or forecast inputs.

## 7. Next benchmark increment: 43-site reference-model reanalysis

The paper reports reference-model performance across 43 sites, with (r) spanning 0.01–0.85 and PE spanning -36.77–0.59. This broad spread is more important than only the four TVA sites.

Next work:
1. Reproduce Appendix Table A1 at all 43 accepted sites from the pinned raw/processed data; document exactly which sites were omitted and why.
2. Verify station/asset pairing, polarity metadata, instrument direction, sample cadence, dropout filtering, gaps and zero/constant-current exclusions.
3. Report signed (r), (r^2), polarity-qualified PE, MAE, RMSE, bias, peak magnitude/timing error and observation/model standard deviation per site.
4. Report robust distribution summaries, worst-decile and region/site strata. Do not hide severe negative PE behind a mean.
5. Use leave-region/site-cluster-out tests if training/calibrating any spatial model. The 43 rows are not 43 independent geomagnetic events.
6. Freeze model configuration before evaluating holdout sites. Keep station clusters and shared geophysical inputs grouped to avoid correlated leakage.

## 8. Paradigm shifts and the actual opportunity

### 8.1 Treat sensor orientation/calibration as first-class evidence

Each GIC sensor record must declare the conductor/winding reference, measurement direction, polarity, calibration and revision. Ambiguous polarity is not silently fixed by a test-time correlation search; report signed metrics as unresolved and magnitude-only metrics as a separate, clearly labelled diagnostic track.

### 8.2 Predict local consequences, not a storm category

Fuse measured magnetic variation, geoelectric response, local conductivity/MT transfer functions, line path integrals, network topology and transformer design. A regional G-scale label or latitude is not an asset-level risk estimate.

### 8.3 Carry multiple sources and models without false independence

NOAA 1-D, NOAA EMTF 3-D, local MT transfer functions, operator models, magnetometer observations and global MHD outputs have different coverage and error structures. Record common source lineage and model dependence. Do not average correlated models as if they were independent witnesses.

### 8.4 Optimize essential-service continuity, not merely GIC fit

The end-state benchmark must test whether a mitigation plan keeps owner-defined electricity, water, telecom, hospital and fuel services above their minimum envelopes. A better GIC RMSE does not automatically imply a better contingency plan; a lower-cost plan does not automatically preserve essential service.

### 8.5 Build recoverability into the objective

For each scenario, quantify critical-service served energy/time, longest interruption, restoration-time distribution, spares/crew/fuel constraints, regional inequity, tail risk and resource feasibility. Test both single-failure and common-cause cascades.

## 9. Proposed performance targets — only after corpus freeze

These are proposed research gates, not achieved results or universal standards:

- Exact reproduction of Appendix A1 before training any model.
- Zero unexplained sign, unit, timestamp, site-mapping or omitted-record discrepancies.
- Holdout-site GIC MAE reduction of at least 10% against the strongest admissible reference baseline, with paired uncertainty intervals excluding zero.
- No predeclared high-consequence site stratum worsens by more than 10% in MAE or peak-error metrics.
- Nominal 90% uncertainty intervals achieve 85–95% empirical coverage on adequately sized, geographically held-out strata; report sample sizes and confidence bands.
- Positive PE on at least 90% of held-out sites, with worst-decile PE and worst-site failures fully disclosed. This is an aspirational gate; if unsupported, report the shortfall rather than relaxing the split.
- Peak GIC magnitude and timing error reported independently from average time-series error.
- Any claimed improvement must survive leave-site-cluster-out and leave-region-out tests; event-held-out evaluation requires additional measured storm events.

A single May 2024 event is insufficient for a general SOTA claim. The benchmark must expand to additional measured storms and regions before any universal superiority claim.

## 10. Source and comparison notes

- Wilkerson et al. (2026), DOI 10.1029/2025SW004758: https://doi.org/10.1029/2025SW004758
- Pinned companion dataset commit: https://github.com/lucywilkerson/SWERVE-2024-05-10/tree/089617756920bcc9df621f7f2299ba0644320cc3
- NOAA/SWPC geoelectric products: https://www.spaceweather.gov/products/geoelectric-field-models-1-minute
- USGS Carrington-class geoelectric mapping (2025): https://doi.org/10.1029/2025GL116835
- NERC TPL-007-4 benchmark requirements: https://www.nerc.com/globalassets/standards/reliability-standards/tpl/tpl-007-4.pdf

---

**Status discipline:** the numerical replay above is complete as an exploratory analysis. A packaged evaluator, a second independent reproduction, exact file-level digest receipt, full 43-site replication, cross-event generalization and physical-model improvement remain future work. No CI pass or operational validation is claimed.
