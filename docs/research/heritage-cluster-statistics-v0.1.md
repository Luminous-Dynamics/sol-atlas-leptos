# HDB-20: Cluster-Level Confidence Intervals and Spatial Effective Information v0.1

## Purpose

HDB-19 declares the evaluation estimand and independent split units. This module makes uncertainty reporting use those same units rather than pretending pixels, chips or stones inside one site are independent transfer trials.

## v0.1 estimand

The input contains one metric value for each independent unit (site, region, spatial block, sensor family or acquisition epoch). The report estimand is the **equally weighted macro-unit mean**. \`support_count\` is retained for audit but does not weight the mean. A micro-average is a separate estimand and must not be silently substituted.

The correlation matrix describes the correlation of the **unit-level metric estimation errors**, not raw-pixel similarity. It is externally fitted or justified and bound to:
- exact HDB-19 split-manifest digest;
- evaluation-run and metric-definition digests;
- unit-level values, identities and support counts;
- correlation-matrix digest;
- correlation-model digest, method, evidence reference and receipt;
- seed, replicate count and interval configuration.

The Rust module validates digest shape and evidence-reference presence only. It does not authenticate bytes, signatures, rights or custody.

## Effective-unit diagnostic

For \(K\) equally weighted independent-unit estimates with common marginal variance and a supplied correlation matrix \(R\):

\[
K_{\mathrm{eff}} = \frac{K^2}{\sum_i\sum_j R_{ij}}
\]

The implementation requires the equal-marginal-variance assumption to be declared. The matrix must be square, finite, symmetric, have unit diagonal, contain nonnegative correlations in \([0,1]\), and be positive semidefinite. A tolerant LDLᵀ factorisation accepts valid singular positive-semidefinite matrices.

This diagnostic is conditional on the supplied covariance assumptions. It is not a substitute for fitting and checking residual variograms/correlograms, nor does geometric distance alone establish the entries of \(R\).

## Cluster bootstrap

- Deterministic SplitMix64 seeded resampling; the same ordered input and seed replay exactly.
- Resample \(K\) independent units with replacement for each replicate.
- Compute the macro-unit mean for each replicate.
- Use the all-observation percentile interval with linear interpolation between order statistics.
- Report the bootstrap standard error, exact seed, requested/completed replicates, unit count, effective-unit count, matrix sum and maximum off-diagonal correlation.
- At least 1,000 replicates are required; v0.1 caps requests at 100,000 and independent units at 512.
- The interval is withheld with a typed status when \(K_{\mathrm{eff}}\) is below the preregistered minimum (which must be at least 2), or when the maximum off-diagonal correlation exceeds the declared tolerance. A withheld interval is represented as null, never as a zero-width or perfect interval.

The percentile interval is conditional on the supplied unit-level metric values and correlation model. It is not a universal coverage guarantee, especially with few independent units. The nine-site drone benchmark cannot support pixel-level precision intervals by treating thousands of image chips as independent. Report per-site results and worst-site performance alongside any macro mean; preregister a higher minimum effective-unit threshold for any claim that requires robust regional generalisation.

## Test coverage and limits

Synthetic tests cover independent and correlated units, equal weighting despite unequal support counts, deterministic replay, PSD and singular PSD matrices, asymmetric/non-PSD/negative/non-finite correlation matrices, bad digests, duplicate IDs, zero support, invalid replicate counts and confidence levels, and interval-withholding gates.

No Monte Carlo interval-coverage study has been run by this implementation. The HDB-20 issue's 90% and 95% empirical coverage targets remain a separate qualification milestone. BCa intervals, paired model-difference inference, hierarchical/multiway cluster bootstrap, unequal-variance effective sample size, spatial covariance estimation, nonstationarity and directional variograms remain out of scope.

## Industry and research comparison

For raster classifiers, per-pixel confidence intervals that ignore spatial dependence overstate independent evidence. The appropriate comparator is the deployment estimand:
- random-tile reproduction for comparison with a paper's original protocol;
- source-footprint-independent blocks;
- buffered spatial holdout;
- leave-one-site-out;
- cross-region, cross-sensor and cross-acquisition-epoch transfer.

These are different questions and must be reported separately. Sol-Atlas should report macro-site performance, worst-site performance, effective independent units, calibration/abstention, interval width, validated findings per expert-hour and false-lead burden. No single pooled F1 or confidence interval substitutes for field validation.

## Claim boundary

This module validates and computes from supplied independent-unit results and a supplied correlation matrix. It does not prove that units are truly independent, estimate correlation from raw residuals, authenticate the evidence receipt, validate archaeological labels, or establish field accuracy or archaeological interpretation.
