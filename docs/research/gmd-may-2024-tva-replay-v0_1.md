# May 2024 TVA GIC Replay — Reproduction Note v0.1

**Status:** Reproduced published comparison; no new physical model claimed.  
**Evidence level:** E2 reference-differential reproduction of a published owner-model comparison.  
**Date:** 2026-11-10  
**Purpose:** Establish an exact, pinned first replay fixture for the GMD resilience programme.

## 1. Finding

We independently recomputed the four-site TVA measured-versus-calculated GIC comparison published by Wilkerson et al. (2026), using the companion public dataset. The computed rounded values reproduce the paper's Table 1 at all four sites for standard deviation, coefficient of determination and prediction efficiency.

This is a useful baseline and ingestion check. It is **not** a new model, not evidence of improvement over state of the art, not a Carrington forecast and not operational grid qualification.

## 2. Frozen source

- Repository: https://github.com/lucywilkerson/SWERVE-2024-05-10
- Immutable source commit used: `089617756920bcc9df621f7f2299ba0644320cc3` (2026-06-05)
- Dataset DOI: https://doi.org/10.5281/zenodo.20090223
- Repository declares MIT licensing. Individual upstream source-owner conditions and attribution still apply; do not assume that an MIT repository license supersedes separate NERC/TVA source conditions.
- Paper: Wilkerson et al. (2026), *GIC-Related Observations During the May 2024 Geomagnetic Storm in the United States*, Space Weather 24, e2025SW004758, https://doi.org/10.1029/2025SW004758
- Paper's full text: https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2025SW004758

## 3. Frozen comparison protocol

- Sites: Bull Run, Montgomery, Union and Widows Creek.
- Event: 10–12 May 2024 Gannon storm.
- Evaluation window: 2024-05-10 15:00 UTC through 2024-05-12 06:00 UTC, inclusive.
- Measured TVA GIC cadence: 1 second.
- TVA calculated GIC cadence: 1 minute.
- Aggregation: non-overlapping arithmetic mean of the 60 one-second measurements in each UTC minute, aligned to the model minute. All four sites yielded 60 measured samples for every matched minute.
- Matched samples: 2,341 one-minute pairs per site.
- Polarity: the raw measured/model correlations are negative at all four sites. The paper states that where (r) is negative, prediction efficiency is computed after multiplying measured GIC by (-1), reflecting sensor orientation opposite to the model convention. The table below reports (a) raw-sign correlation and (b) the equivalent polarity-normalized correlation and prediction efficiency. In this independent calculation, the equivalent operation was implemented as (GIC_{p}mapsto -GIC_{p}); this gives the same squared-error prediction efficiency as (GIC_{m}mapsto -GIC_{m}), while preserving the raw measured series for provenance.
- Metrics:
  - (r = corr(GIC_m,GIC_p))
  - (r^2) coefficient of determination
  - (PE = 1-rac{sum_t(GIC_m(t)-GIC_p(t))^2}{sum_t(GIC_m(t)-\overline{GIC_m})^2})
  - MAE and RMSE are added as direct magnitude-error metrics.
  - Standard deviations are population standard deviations of the 2,341 paired minute values; rounding is consistent with the paper's displayed values.

## 4. Results

| Site | Matched min | Measured σ (A) | TVA model σ (A) | Raw (r) | Polarity-normalized (r) | (r^2) | (PE) | Normalized MAE (A) | Normalized RMSE (A) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Bull Run | 2,341 | 1.505 | 2.167 | -0.8941 | 0.8941 | 0.79936 | 0.49795 | 0.763 | 1.066 |
| Montgomery | 2,341 | 3.704 | 1.003 | -0.9294 | 0.9294 | 0.86372 | 0.42977 | 1.475 | 2.797 |
| Union | 2,341 | 3.752 | 3.548 | -0.8178 | 0.8178 | 0.66882 | 0.65151 | 1.547 | 2.215 |
| Widows Creek | 2,341 | 1.942 | 1.212 | -0.8190 | 0.8190 | 0.67084 | 0.63147 | 0.825 | 1.179 |

The paper's displayed values are:
- Bull Run: measured/model σ 1.5/2.2 A, (r^2=0.80), (PE=0.50).
- Montgomery: 3.7/1.0 A, (r^2=0.86), (PE=0.43).
- Union: 3.8/3.5 A, (r^2=0.67), (PE=0.65).
- Widows Creek: 1.9/1.2 A, (r^2=0.67), (PE=0.63).

All rounded values agree with Table 1.

### Interpretation

Correlation alone would overstate model quality. At Bull Run, the TVA model's standard deviation exceeds the measured standard deviation (over-dispersion); at Montgomery and Widows Creek it is substantially smaller (under-dispersion); Union is closer in scale. Prediction efficiency penalizes these magnitude errors. A useful GMD benchmark must therefore report calibration and magnitude error, not just correlation or (r^2).

## 5. Raw data pins

All paths below are relative to the immutable source commit above. Git blob IDs are included so each fixture element can be independently checked.

### Measured 1-second TVA GIC

| Site | Path | Git blob SHA |
|---|---|---|
| Bull Run | `data_original/tva/gic/GIC-measured/gic-bullrun_20240510.csv` | `edfcc1e6614a3b7477661f00e9e0f08cac98ae1a` |
| Montgomery | `data_original/tva/gic/GIC-measured/gic-montgomery_20240510.csv` | `d88cff5d1c3a9fc75bca17a0c1dbd6c1e35b5a1e` |
| Union | `data_original/tva/gic/GIC-measured/gic-union_20240510.csv` | `2dfd89020012c3d834b2da9f89a772f2cb08387a` |
| Widows Creek | `data_original/tva/gic/GIC-measured/gic-widowscreek2_20240510.csv` | `da7f8c0ac7fe99421d1e82acee0afccfee6001dc` |

### TVA calculated 1-minute GIC

| Day | Bull Run | Montgomery | Union | Widows Creek |
|---|---|---|---|---|
| 10 May | `562f690f3e2849fa24e85b2bcf733c3e69832e30` | `84690f81b28978d61d6afd8b5b97060c3869aaa3` | `01bcc3035eb9805749922f6fd9120af54b79e409` | `6e2ef9e1d0d6aaf56f866e8fbfb778cd578b3902` |
| 11 May | `74d88e8e0ae4827c2664e853441a4ee342934c0c` | `75db46ce3635afe5916a2bdd3a1302b09c996190` | `4e129587c1d677182804e6a4c6a54eb838f1460d` | `05a671a0923ad9ee75d8e6eb98de16938ee15bca` |
| 12 May | `73a8cca21d844d42c944c3e366eb920c79457d92` | `3d20085f7945eafbc428b3495329de6ffc0596d4` | `6650d1cb6987da7526045fb9acfc57936dc0d2bd` | `c861343ae8cf02d5a534dea7aabfe5fac1cff597` |

Paths follow the pattern:
`data_original/tva/gic/GIC-calculated/YYYYMMDD_<site>GIC.dat`.
Bull Run uses the filename stem `BullRunXfrmGIC`; the others use `MontgomeryGIC`, `UnionGIC` and `WidowsCreek2GIC`.

## 6. What this establishes—and what it does not

**Established in this pass**
- The source files are accessible at a pinned public commit.
- The measured-to-model temporal alignment is reproducible over the paper's stated interval.
- The four-site Table 1 statistics reproduce to the published two-decimal precision.
- Raw polarity disagreement is retained and explicitly normalized according to the paper's documented convention.
- Sigma, (r^2) and prediction efficiency jointly expose amplitude-fit limitations.

**Not established**
- No new GIC/geomagnetic physical model was developed or scored.
- This is the same event and four owner-model sites used by the source paper; it is not a held-out event or independent utility validation.
- No transformer thermal/harmonic model is validated here.
- No claim is made about Carrington-class conditions, South African networks, or any operational grid.
- The authoring pass used a programmatic data replay; a versioned standalone Rust reproduction harness, full Draft 2020-12 schema validation, CI run and external review remain outstanding.

## 7. Critical baseline extension

The 2026 paper provides a broader reference-model comparison: the reference model was evaluated at 43 GIC sites, with reported (r) from 0.01 to 0.85 and prediction efficiency from (-36.77) to 0.59. It also documents that 47 GIC sites remained after quality filtering and deduplication from the larger source pool. This wider corpus should become the next baseline tranche.

Required extension:
1. Reproduce Table A1 for all 43 reference-model sites and retain every site, including poor and negative-skill sites.
2. Preserve the source paper's visual quality-control exclusions as an auditable exclusion ledger; do not silently omit difficult stations.
3. Use event- and site-blocked uncertainty estimates, not IID minute-level bootstrap intervals.
4. Evaluate absolute magnitude error, signed bias, peak timing, event-threshold detection, interval calibration and spatial generalisation.
5. Separate retrospective reconstruction from genuine forecast evaluation. A model driven by measured storm-time magnetometer input is not a pre-event forecast.
6. Add event holdouts as new public storms/data become available; one storm cannot establish generalization.

## 8. How Luminous could improve on this baseline

The opportunity is not merely to beat the four-site TVA result. It is to address the limits exposed by the broader data:

- **Spatially local Earth response:** compare 1-D conductivity with site-specific 3-D MT transfer functions, retaining coverage gaps and uncertainty.
- **Topology- and orientation-aware induction:** calculate path-integrated E-field and network GIC using declared line geometry, line resistance, transformer winding/grounding and topology; keep unknown parameters visible.
- **Calibrated multi-fidelity prediction:** combine official, operator, reference and data-driven estimates only with source-dependence and model-discrepancy tracking.
- **Tail-aware service risk:** estimate transformer stress and downstream critical-service shortfalls only where validated specialist models and owner limits exist.
- **Evidence-preserving abstention:** missing station, stale MT transfer function, incomplete grid topology or unknown grounding must prevent a false low-risk result.
- **Federated continuity:** deliver signed, scoped observations and incident coordination over partitioned networks, with no new authority granted by stale data.

Proposed superiority gate (not yet achieved): on a pre-registered, event-held-out and site-held-out corpus, reduce GIC MAE by at least 10% against the strongest admissible matched baseline while maintaining or improving probability/interval calibration and without a greater than 10% regression in any predeclared major region or asset stratum. If sample size, event independence or baseline parity is insufficient, report inconclusive rather than declaring SOTA.

## 9. Next work-item links

- Sol-Atlas spatial GMD task: https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/692
- Symtropy physics GMD task: https://github.com/Luminous-Dynamics/symtropy/issues/4461
- Symthaea consequence-intelligence task: https://github.com/Luminous-Dynamics/symthaea/issues/11358
- Mycelix offline-coordination task: https://github.com/Luminous-Dynamics/mycelix/issues/6018
- Shared architecture PR: https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/693
- Parent multi-hazard programme: https://github.com/Luminous-Dynamics/symtropy/issues/2232

## 10. References

1. Wilkerson, L. A., Weigel, R. S., Thomas, D., Bor, D., Oughton, E. J., Gaunt, C. T., et al. (2026). GIC-related observations during the May 2024 geomagnetic storm in the United States. *Space Weather*, 24, e2025SW004758. https://doi.org/10.1029/2025SW004758
2. NOAA SWPC, Geoelectric Field Models (US-Canada 1-D and empirical EMTF 3-D), including explicit validation and geographic-coverage caveats: https://www.spaceweather.gov/products/geoelectric-field-models-1-minute
3. NOAA SWPC, 1-minute Empirical EMTF 3-D model: https://www.spaceweather.gov/products/geoelectric-field-1-minute-empirical-emtf-3d-model
4. NERC, TPL-007-4, Transmission System Planned Performance for Geomagnetic Disturbance Events: https://www.nerc.com/globalassets/standards/reliability-standards/tpl/tpl-007-4.pdf

---

**Reproducibility status:** data source and commit pinned; aggregate metrics reproduced; no standalone executable benchmark harness or CI receipt yet. Keep this distinction in all future summaries.
