# Directional Spectral-Geometric Transfer (DSS) v0.1

**Task scope:** cropmark vegetation-proxy transfer only. The implementation consumes externally computed A-to-A and A-to-H spectral-cluster distances; it does not generate cropmark masks or infer archaeology.

## Published method reproduced in the Rust evaluator

The 2026 PNAS Nexus study builds monthly pooled subclusters for A pixels (vegetation above buried archaeological remains) and H pixels (other vegetation). For one ordered source region s -> target region t and one calendar month m, it defines:

- r_st(m) = d(A_s(m), A_t(m)) / d(A_s(m), H_t(m))
- DSS_st = mean of 1 / r_st(m) over the M months with the lowest r_st(m)

The published paper uses M=3 and computes the reverse direction separately. A high score indicates that source A spectra are relatively close to target A spectra and relatively far from target H spectra. This is a directional task-specific transfer affinity, not a symmetric geographic similarity.

Four distance families are retained separately: Euclidean, band-wise Wasserstein, Kolmogorov–Smirnov and Mahalanobis. The Rust evaluator calculates ratios and top-M aggregation over precomputed distances, and records all monthly ratios and selected months.

## Reproduction context and limitations

The source study uses daily Sentinel-2 Level-2A BOA data for 2015–2025, 12 bands on a common 10 m grid, scene cloud coverage <=20%, scene-level radiometric filtering, NDVI phenophase filtering and pixel-level vegetation masks. It describes three European cropmark sites. A-pixel labels were made once per site by visual interpretation of high-resolution imagery and literature rather than a new field survey.

The study reports high agreement between DSS-based and model-F1 directional rankings, including Spearman 0.94 at its main residual scale. That correlation is based on only six ordered site pairs (three sites), so it is encouraging but exploratory. Its supplementary modelling split uses a stratified 70/30 pixel split within site/month; Sol-Atlas's primary extension must use spatial blocks and independent site/landscape holdouts to avoid overestimating transfer.

References:
- Paper: https://doi.org/10.1093/pnasnexus/pgag336
- Supplementary methods and code/data availability: https://doi.org/10.5281/zenodo.19207808

## Input requirements

- One input describes exactly one ordered source -> target region pair.
- The task scope is fixed to cropmark_vegetation_proxy; other archaeological feature types require independent methods and benchmarks.
- Positive and background class semantics and the label-protocol reference are explicit.
- Feature-definition and distance-pipeline digests are mandatory.
- Distances are supplied per calendar-month subcluster and per metric. The same calendar month pooled across years is one observation; repeated January acquisitions must already be aggregated upstream.
- All distances must be finite and strictly positive. The published ratio is undefined or unbounded for zero distances; v0.1 rejects those cases instead of adding an undocumented epsilon.
- Every metric requires at least M shared calendar months.

## Association with actual transfer performance

The association evaluator takes one record per ordered held-out pair for a single metric and reports Pearson and Spearman correlations between DSS and target F1. At least three distinct directed pairs are required. Reverse directions remain separate. With only six pairs, correlations should be reported as exploratory with uncertainty and not treated as universal predictive power.

## Next extension

1. Reproduce the released three-site DSS table exactly, under documented artifact licences.
2. Expand to more landscapes and cropmark ecologies.
3. Use spatially blocked, buffered site/landscape partitions, not random pixels, for the primary transfer benchmark.
4. Measure whether DSS predicts target-region PR-AUC, recall at fixed review budget, calibration and worst-region degradation.
5. Compare DSS with geographic distance, biome/land-cover similarity, phenophase alignment and combined predictors.
6. Preserve directionality and show the full source-by-target transfer matrix.

## Claim ceiling

This module implements the published distance-ratio aggregation over caller-supplied distances. It does not verify source reflectance data, cropmark labels, archaeological remains, or independent field truth. A high DSS is not an archaeological discovery and cannot replace expert review.
## Region jackknife sensitivity

The association report additionally removes each landscape in turn, drops every directed pair touching that landscape, and recomputes Spearman correlation on the remaining pairs when at least three remain. It records the omitted landscape, remaining pair count and correlation; it then reports the minimum and maximum available jackknife correlations.

This is a **leave-one-landscape-out sensitivity diagnostic, not a confidence interval**. With only three sites/six directed pairs it may be undefined for every omission, which is the honest result. A larger spatially independent benchmark with at least five landscapes is needed before this diagnostic becomes informative. Directed pairs still share landscapes; do not treat them as independent replicates or use a naive pair-level standard error.
