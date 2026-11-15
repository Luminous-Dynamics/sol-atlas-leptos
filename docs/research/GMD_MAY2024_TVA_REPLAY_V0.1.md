# May 2024 GIC Four-Site Replay — Benchmark Record v0.1

**Status:** Recomputed from pinned public files in the authoring session; not yet independently rerun from a committed tool.  
**Evidence tier:** E1 data-pipeline known-answer reproduction.  
**Date:** 2026-11-10.

## 1. Why this benchmark exists

The May 2024 Gannon storm has a rare combination of measured GIC, operator-computed GIC and reference-model outputs. The peer-reviewed article by Wilkerson et al. (first published 19 May 2026) reports results at four TVA substations and is unusually useful for an exact data-ingestion and metric-calculation check.

This replay covers Bull Run, Montgomery, Union and Widows Creek. It reproduces both:
- TVA operator-calculated neutral GIC at one-minute cadence; and
- the GMU reference-model median GIC at corresponding substations.

It does **not** rerun either physical model. It verifies data pairing, temporal aggregation, sign/polarity treatment and the published benchmark arithmetic.

## 2. Canonical paper version: do not use the old preprint table

Canonical publication:

Wilkerson et al., *GIC-Related Observations During the May 2024 Geomagnetic Storm in the United States*, **Space Weather**, Volume 24, Issue 5, e2025SW004758; first published 19 May 2026. DOI: https://doi.org/10.1029/2025SW004758

The October 2025 preprint and the final 2026 article do not report identical reference-model table values. This replay matches the **final publisher article**. Do not use the 2025 preprint table as the canonical known-answer target. Pin the exact article revision/citation when publishing a result.

Publisher's final Table 1 reports:

| Site | TVA r² | TVA PE | Reference r² | Reference PE |
|---|---:|---:|---:|---:|
| Bull Run | 0.80 | 0.50* | 0.48 | 0.48 |
| Montgomery | 0.86 | 0.43* | 0.15 | 0.15* |
| Union | 0.67 | 0.65* | 0.57 | 0.50* |
| Widows Creek | 0.67 | 0.63* | 0.28 | 0.23* |

An asterisk denotes the paper's sensor-orientation correction when the raw measured/modelled correlation is negative.

## 3. Frozen public source snapshot

- Repository: https://github.com/lucywilkerson/SWERVE-2024-05-10
- Pinned commit: `089617756920bcc9df621f7f2299ba0644320cc3`
- Dataset DOI: https://doi.org/10.5281/zenodo.20090223
- Repository license: MIT is declared in the repository and Zenodo metadata. Embedded third-party source datasets retain their provider terms; no raw data is copied into this repository by this benchmark record.
- Repository tree at the pinned commit is not truncated. Only the selected files below are needed for this four-site replay; do not clone or redistribute the multi-gigabyte repository wholesale.

### Measured 1-second TVA GIC files

| Site | Path | Git blob SHA |
|---|---|---|
| Bull Run | `data_original/tva/gic/GIC-measured/gic-bullrun_20240510.csv` | `edfcc1e6614a3b7477661f00e9e0f08cac98ae1a` |
| Montgomery | `data_original/tva/gic/GIC-measured/gic-montgomery_20240510.csv` | `d88cff5d1c3a9fc75bca17a0c1dbd6c1e35b5a1e` |
| Union | `data_original/tva/gic/GIC-measured/gic-union_20240510.csv` | `2dfd89020012c3d834b2da9f89a772f2cb08387a` |
| Widows Creek | `data_original/tva/gic/GIC-measured/gic-widowscreek2_20240510.csv` | `da7f8c0ac7fe99421d1e82acee0afccfee6001dc` |

### TVA operator-calculated one-minute files

For each site, use the files for 10, 11 and 12 May 2024. The `time` field is seconds from midnight; `GICXFNeutralAmps` is the calculated series. Exact file blob SHAs are pinned in `gmd_may2024_four_site_replay_v0_1.json`.

### GMU reference-model files

| Site | File | Substation ID | Nearest measurement | Separation |
|---|---|---:|---|---:|
| Bull Run | `data_original/gmu/tva/site_108137138.csv` | 108137138 | Bull Run | 0.0630 km |
| Montgomery | `data_original/gmu/tva/site_708931360.csv` | 708931360 | Montgomery | 0.0368 km |
| Union | `data_original/gmu/tva/site_98699542.csv` | 98699542 | Union | 0.1087 km |
| Widows Creek | `data_original/gmu/tva/site_14133615.csv` | 14133615 | Widows Creek 2 | 0.0537 km |

The model output is `Sim GIC (Median)`. The reference-model file also includes a nearest-device measurement column, but the reproduction protocol below uses the full 1-second measured TVA series for the observation average. The one-minute nearest-device sample must not be substituted for the paper's non-overlapping 60-sample mean.

## 4. Exact evaluation protocol

- Interval: `2024-05-10T15:00:00Z` through `2024-05-12T06:00:00Z`, inclusive.
- Interpret the TVA source timestamps as UTC, consistent with the paper's declared evaluation interval and its published comparison.
- Form non-overlapping UTC minute bins from the raw one-second measured series.
- Require exactly 60 measurement samples in every scored minute. No interpolation, forward-fill, imputation or partial-minute averaging.
- Pair predictions only to the identical UTC minute. No time shifting or nearest-neighbour temporal matching.
- For TVA files, combine all three daily model files. Convert the per-day seconds offset to the corresponding UTC timestamp.
- For reference-model files, parse the explicit timestamp and use `Sim GIC (Median)`.
- Required pair count: 2,341 one-minute pairs per site per model. All eight comparisons have this count and all 2,341 observed bins contain 60 measured samples.
- Metrics:
  - Pearson correlation: (r)
  - Coefficient of determination: (r^2)
  - Prediction efficiency: (PE = 1 - SSE/SST), where (SST = \sum_t (y_t - \bar y)^2)
  - Additional diagnostics: RMSE and MAE after the declared sensor-orientation correction.
- Preserve the raw signed correlation. Where (r < 0), compute the corrected residual metrics using the paper's sensor-orientation assumption. In this implementation the prediction series was sign-inverted for the residual calculation; this is algebraically equivalent to multiplying the measurement series by (-1). Do not modify the stored source data.

## 5. Recomputed results

The values below were recomputed from the pinned source files with the protocol above.

### TVA operator model

| Site | Raw r | r² | PE after orientation correction | RMSE (A) | MAE (A) | Measured σ (A) | TVA σ (A) |
|---|---:|---:|---:|---:|---:|---:|---:|
| Bull Run | -0.89407 | 0.79936 | 0.49795 | 1.066 | 0.763 | 1.505 | 2.167 |
| Montgomery | -0.92936 | 0.86372 | 0.42977 | 2.797 | 1.475 | 3.704 | 1.003 |
| Union | -0.81781 | 0.66882 | 0.65151 | 2.215 | 1.547 | 3.752 | 3.548 |
| Widows Creek | -0.81905 | 0.67084 | 0.63147 | 1.179 | 0.825 | 1.942 | 1.212 |

### GMU reference model

| Site | Raw r | r² | PE after orientation correction | RMSE (A) | MAE (A) | Measured σ (A) | Reference σ (A) |
|---|---:|---:|---:|---:|---:|---:|---:|
| Bull Run | +0.69299 | 0.48024 | 0.47525 | 1.090 | 0.711 | 1.505 | 1.146 |
| Montgomery | -0.39208 | 0.15373 | 0.15203 | 3.411 | 1.847 | 3.704 | 1.300 |
| Union | -0.75440 | 0.56912 | 0.49808 | 2.658 | 1.808 | 3.752 | 1.831 |
| Widows Creek | -0.52821 | 0.27901 | 0.22717 | 1.708 | 1.135 | 1.942 | 0.584 |

The values round to the final publisher table in all four sites for both model families. The reference-model reproduction is particularly useful: it would not have been recovered by using the one-minute `site_1_gic` nearest-monitor sample instead of averaging the original one-second measurements.

## 6. What the comparison actually says

1. Correlation and amplitude skill are distinct. A high (r^2) does not guarantee accurate current magnitude.
2. TVA's operator model correlates strongly at the four sites but has site-dependent amplitude mismatch: the final paper notes over-prediction at Bull Run and under-prediction at Montgomery and Widows Creek. Union is closer in amplitude.
3. The reference model has lower (r^2) and lower PE at several sites. The model's declared limitations—no live power-system configuration and simplified network representation—matter.
4. RMSE/MAE here are event-window diagnostics after polarity correction. They are not damage thresholds, transformer hot-spot estimates, or an operational acceptance criterion.
5. This is one event, four locations and a reproduction of published baselines. It is not a geographically independent validation set and does not establish that a new Luminous model beats the state of the art.

## 7. Negative controls required in the executable replay

- Use one point sample per minute instead of the 60-sample mean.
- Shift model time by (pm 1) minute.
- Parse timestamps in a non-UTC timezone.
- Omit one sample from a minute; that minute must be rejected, not silently scored.
- Duplicate a sample or a model minute.
- Reverse polarity but fail to record the correction.
- Mix the reference model substation with the wrong nearest GIC monitor.
- Use only the 10 May prediction file for the full multi-day interval.
- Change daily seconds-offset handling or sort/order assumptions.
- Remove a required source file or change its pinned blob SHA.

The positive controls are equally important: all eight supported comparisons must reproduce the published rounded values. Rejecting all inputs is not a pass.

## 8. Next research step

Implement an offline, deterministic, dependency-light Rust replay tool. It should:
- consume locally supplied, pinned files;
- avoid runtime network access;
- validate row grammar, finite values, timestamps, duplicate/missing minutes and exact sample counts;
- emit a deterministic JSON receipt binding the repo commit, every file blob SHA, interval, aggregation, polarity decisions, metrics and build identity;
- be exercised by exact-head CI, but not wait on CI to continue developing the next analytical oracle.

After this replay is independently reproducible, proceed to the tiny-network induced-voltage / quasi-DC GIC analytic oracle in Symtropy. Do not use the 2024 event to fit and score a new predictor on the same observations.

A separate exploratory, past-only prequential residual-calibration diagnostic is documented in [GMD_ADAPTIVE_CALIBRATION_V0.1.md](GMD_ADAPTIVE_CALIBRATION_V0.1.md) and tracked by [Symthaea GMD-ONLINE-CALIBRATION-001 #11370](https://github.com/Luminous-Dynamics/symthaea/issues/11370). It uses earlier within-event observations only to update later predictions, but it remains single-event, post-hoc exploratory evidence; it does not constitute cross-event model validation or justify fitting and scoring a static model on the same rows. The South African historical source audit is tracked by [Sol-Atlas #697](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/697).

## 9. Claim ceiling

This record demonstrates a data-pipeline reproduction of a peer-reviewed benchmark table from pinned public inputs. It does **not** independently validate the TVA or reference physical models, a new Luminous physical solver, transformer thermal response, grid security, resilience, NERC compliance, South African applicability or operational safety.
