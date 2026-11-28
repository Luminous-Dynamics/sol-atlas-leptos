# Sol-Atlas spatial uncertainty and calibration benchmark v0.1

**Status:** synthetic scoring oracle; no field or SOTA performance claim.

## Why this benchmark exists

A low average geometric error does not establish that uncertainty estimates are trustworthy. Likewise, a high classification accuracy can hide severe overconfidence in a particular sensor group or site. Sol-Atlas needs to report calibration and failure strata separately from average accuracy.

This benchmark provides an executable reference for:

- Brier score and log loss (proper scoring rules for probabilistic predictions), with an exact forecast-category Murphy decomposition into reliability, resolution and outcome uncertainty;
- equal-width expected calibration error (ECE) with the reliability-bin counts and gaps retained. ECE is diagnostic only because it is binning- and sample-size-sensitive;
- selective risk versus coverage when low-confidence predictions are withheld;
- empirical coverage and width of nominal 95% spatial measurement intervals;
- mean absolute metrology error;
- worst sensor-group and worst-site interval coverage;
- traceability receipts binding each measurement to instrument/modality, calibration status, CRS/units, environment, uncertainty budget, reference method and processing lineage. The synthetic receipts explicitly state that no physical calibration or reference evidence was supplied;
- train/evaluation separation at whole-site granularity;
- deterministic site-cluster bootstrap percentile confidence intervals for Brier, log loss and spatial interval coverage.

## Synthetic oracle

The fixture intentionally includes one highly confident but wrong probability prediction and a photogrammetry group with weaker interval coverage than TLS. The reference tests assert:

| Metric | Synthetic value |
|---|---:|
| Brier score | 0.2150167 |
| Log loss | 0.9844226 |
| 5-bin ECE | 0.2483333 |
| Nominal 95% interval empirical coverage | 66.67% |
| Mean interval width | 0.566667 |
| Mean absolute error | 0.316667 |
| Worst sensor-group coverage | 33.33% |
| Worst site coverage | 50.00% |

These results are designed to demonstrate that aggregate metrics and worst-stratum performance can tell different stories. They are not estimates of Sol-Atlas or any real sensor system.

## Metrology receipt and physical-evidence boundary

Every scored metrology observation now names a receipt identifying its instrument modality and site. The receipt provides fields for calibration certificate status, coordinate reference system, acquisition setup, environment, GUM-style Type A/Type B uncertainty contributions and covariance terms, reference survey and processing lineage. All six receipts are **synthetic fixtures**: they explicitly contain no real calibration certificate, no physical reference survey and no artifact digests. They validate linkage and schema behavior only. A field dataset cannot be qualified until actual instrument certificates, reference-control observations, data digests, environmental logs and independently reviewed uncertainty budgets replace these placeholders.

## Interpreting the Brier score

Brier score is an overall proper forecast score, not a calibration-only measure: it combines reliability, resolution and outcome uncertainty. The reference scorer therefore also emits the exact finite-sample forecast-category Murphy decomposition `BS = reliability - resolution + uncertainty`, grouping identical forecast probabilities. Sparse forecast categories can make the components noisy, so retain category counts and do not substitute the decomposition for reliability diagrams or independent-site evaluation.

## Protocol recommendations

1. Split by whole site, survey campaign and acquisition session; do not let scans from one physical site leak across train and evaluation.
2. Keep a sealed evaluation set and independent reference metrology (TLS/total station/GNSS or another qualified measurement process).
3. Report Brier and log loss alongside reliability diagrams; ECE depends on binning and sample size and must never be the sole calibration metric. Add adaptive-bin / debiased ECE and confidence intervals for real evaluation.
4. Report interval coverage with interval width and uncertainty units/method. A nominal 95% interval with 95% coverage but unusably wide bounds is not sufficient.
5. Break out sensor, material, scale, occlusion, operator, environment and site strata; publish worst-group coverage and selective risk. Include spatially weighted conformal baselines where local dependence or nonstationarity is material.
6. Compare human / survey-metrology baseline, uncalibrated model, post-hoc calibrated model, ensemble or conformal method, and abstention policies on the same frozen holdouts.
7. Pre-register target coverage and statistical confidence bounds before the evaluation. Small samples such as this fixture are only for implementation checks.
8. Estimate uncertainty in the benchmark metrics by resampling whole sites/campaigns, not individual points. The small synthetic three-site bootstrap here only validates the implementation; three clusters are far too few for a strong empirical interval.

## Initial qualification targets (proposed, not measured)

- Empirical coverage statistically compatible with nominal coverage on independent sites, with confidence intervals and width reported.
- No sensor group silently omitted from aggregate reports.
- Calibration reporting contains Brier, log loss, reliability bins, ECE, selective risk/coverage, interval coverage/width, and worst-site/sensor breakdown.
- Zero train/evaluation site overlap.
- Every score traceable to frozen prediction rows, reference observations, configuration and dataset digests.

## Run

```sh
cargo test --manifest-path reference/Cargo.toml
cargo run --manifest-path reference/Cargo.toml -- fixtures/calibration-case.json
```
