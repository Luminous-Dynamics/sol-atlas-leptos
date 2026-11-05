# HDB-20: Cluster-Level Confidence Intervals and Spatial Effective Information v0.1

## Why this exists

A million pixels in one archaeological site do not constitute a million independent transfer trials. HDB-20 binds each reported metric to the highest independent unit that matches the intended generalisation claim (site, region, buffered spatial block, sensor family or acquisition epoch).

## Implemented v0.1

- A typed input binds evaluation-run, HDB-19 split-manifest, metric-definition and correlation-model SHA-256 digests.
- Each unique independent unit contributes exactly one metric value and a descriptive support count.
- The point estimator is the equal-weight macro mean across independent units. Support count is not silently converted into a weight.
- A correlation matrix is validated for dimensions, symmetry, unit diagonal, finite nonnegative correlations in [0,1], and positive semidefiniteness.
- The equal-variance effective-unit diagnostic is computed as:

`K_eff = K² / ΣᵢΣⱼ Rᵢⱼ`

where K is the number of independent evaluation units and R is the supplied correlation matrix of unit-level metric estimation errors.
- A deterministic SplitMix64 generator produces a percentile cluster bootstrap with the supplied seed and replicate count.
- Confidence intervals are emitted only when the effective-unit count meets the preregistered minimum and maximum observed inter-unit correlation is within the declared tolerance. Otherwise the report contains a typed withholding status and no interval endpoints.

## Interpretation and limitations

The effective-unit formula assumes equal marginal variances and that R describes correlation among unit-level metric estimation errors, not raw pixel correlation. A correlation matrix is not inferred from distance alone; the input binds an external correlation model digest and evidence reference.

The bootstrap is a conditional percentile bootstrap over the supplied independent units. It is not a universal finite-sample coverage guarantee. The validator cannot authenticate the correlation evidence, establish unit independence, estimate a variogram from source raster residuals, or prove the supplied metric values were computed correctly.

A report with `correlation_exceeds_tolerance` or `insufficient_effective_units` is still a useful audit result; it explicitly refuses to emit an unjustified interval.

## Research basis

- Young (2026), *How Many Independent Samples Does a Satellite Image Contain? Generalization Bounds for Spatially Dependent Data*, IEEE TGRS 64, published 1 October 2026: https://doi.org/10.1109/TGRS.2026.3739158.
- Comer et al. (2022), *Sampling methods for archaeological predictive modeling: Spatial autocorrelation and model performance*: https://doi.org/10.1016/j.jasrep.2022.103824.
- Tabassum & De Brabanter (2026), *Pointwise confidence intervals of local linear regression with correlated errors using block bootstrap*: https://doi.org/10.1016/j.spasta.2026.101003.
- Cameron & Miller (2026), *Inference for Regression with Clustered or Spatially Correlated Data I: Framework and Clustering*: https://doi.org/10.3386/w35800.
- Orengo et al. (2026), *Theory and practice of artificial intelligence in archaeology*: https://doi.org/10.1016/j.jas.2026.106571.

## Next qualification

- Run coverage studies with independent, equicorrelated, spatially decaying and anisotropic dependence.
- Compare percentile and BCa intervals; keep BCa out of the v0.1 implementation.
- Evaluate paired model differences at the same held-out sites with a cluster bootstrap.
- Verify the effective-unit calculation against a high-precision reference and independent geostatistical software.
- Add paired model-comparison coverage studies under different correlation scenarios.
- Bind actual model outputs from the HDB-B1 baseline before making any SOTA claim.
