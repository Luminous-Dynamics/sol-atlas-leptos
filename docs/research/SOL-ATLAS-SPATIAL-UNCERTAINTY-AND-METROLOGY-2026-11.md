# Sol-Atlas: spatial uncertainty, calibration and metrology research update

**Date:** 2026-11-28  
**Status:** research plan + synthetic reference implementation; no field qualification claim.

## 1. Research direction

Sol-Atlas should treat uncertainty as an observable, auditable measurement object rather than a confidence decoration attached to geometry. Every estimate should separate:

- measurement uncertainty from acquisition and reference metrology;
- predictive uncertainty from the model;
- spatial distribution shift and local dependence;
- epistemic uncertainty from unobserved materials, sites or methods;
- decision uncertainty about whether a proposed action remains justified.

The system should report not only average performance but where and under what conditions its uncertainty claims fail.

## 2. What current research says

### Spatially aware conformal prediction is now a serious baseline

GeoConformal Prediction (GeoCP), published online in July 2025, adapts conformal prediction to spatially structured data, including spatial regression and interpolation. In its reported housing-price example it achieved 93.67% coverage, while the tested bootstrap approach reached at most 81% after 2,000 runs. These are results for those datasets and experimental conditions, not universal guarantees and not a direct benchmark for Sol-Atlas heritage geometry. The important lesson is that spatial dependence and local structure need to be addressed explicitly rather than applying ordinary exchangeable calibration to geographically clustered data ([Lou, Luo & Meng, 2025](https://doi.org/10.1080/24694452.2025.2516091)).

A 2026 article on nonstationary spatial processes further describes spatial weighting in conformal prediction to better account for local variation and exchangeability assumptions ([Luo, 2026](https://doi.org/10.1080/24694452.2026.2639721)). Sol-Atlas should therefore compare at least:
- ordinary split conformal;
- spatially weighted / GeoConformal Prediction;
- bootstrap intervals;
- geostatistical kriging variance where appropriate;
- calibrated ensembles or Bayesian predictive distributions.

Do not simply compare coverage percentages. Compare coverage against interval width, local coverage by spatial block/site, worst-region coverage and computational cost. Keep the calibration data spatially separated from the evaluation sites.

### ECE is not a sufficient calibration objective

Expected Calibration Error is a useful reliability-diagram summary, but its value depends on binning and sample size; fixed-width ECE can be discontinuous and can obscure calibration/accuracy interactions. The 2026 review of classifier probability-calibration metrics discusses these limitations and alternative metrics ([Comprehensive Review of Classifier Probability Calibration Metrics, 2026](https://journals.sagepub.com/doi/full/10.1177/24518492261436231)).

Sol-Atlas should report:
- Brier score and log loss as proper scoring rules;
- reliability diagrams with counts and confidence intervals;
- multiple ECE variants (equal-width, equal-mass, adaptive or debiased);
- calibration slope/intercept where applicable;
- selective risk versus coverage;
- stratified and worst-group calibration.

ECE must remain diagnostic and must not be optimized in isolation.

### Heritage geometry needs metrological traceability

A September 2026 heritage digital-twin measurement study grounds geometric uncertainty in the Guide to the Expression of Uncertainty in Measurement (GUM), ISO 25178-600:2019 and EN 15757:2010. It combines structured-light scanning, close-range photogrammetry and smartphone LiDAR/SLAM and reports sub-millimetre inter-system agreement in its own test setting, as well as expanded uncertainty estimates. Those figures are study-specific, not global sensor specifications or transferable acceptance limits ([Moyano et al., Measurement, 2026](https://doi.org/10.1016/j.measurement.2026.122271)).

Our physical qualification should therefore preserve a measurement traceability chain: reference instrument and calibration state, acquisition geometry, environmental conditions, alignment/transformation operations, covariance or uncertainty budget, coverage factor, reference artifacts and exact datasets.

### Heritage benchmarking still needs consistent reporting

A 2026 Scan-to-HBIM review reports inconsistent use of RMSE, IoU and level-of-detail metrics and argues for more transparent benchmarking that includes semantic and cultural-value concerns ([Waqar, Automation in Construction, 2026](https://doi.org/10.1016/j.autcon.2026.106848)). Sol-Atlas should not reduce geometric fidelity, uncertainty, semantic integrity and conservation value to one score.

## 3. Benchmark design

The benchmark introduced in `benchmarks/spatial-uncertainty-calibration/v0.1` is a small synthetic scoring oracle. It implements Brier score, log loss, fixed-bin ECE, selective risk, nominal interval coverage, interval width, mean absolute error, and worst-site / worst-sensor coverage. It also rejects train/evaluation site overlap.

It intentionally does not claim that a 6-sample fixture demonstrates statistical coverage. The 95% interval result in that fixture is an expected failure signal, not a real model result.

### Required comparisons on real sites

| Track | Baselines | Main metrics |
|---|---|---|
| Classification calibration | Uncalibrated, temperature scaling, isotonic, ensemble, Bayesian | Brier, log loss, reliability diagrams, calibration slope/intercept |
| Spatial interval calibration | Split CP, GeoCP / spatially weighted CP, bootstrap, kriging variance | Coverage, width, worst-block coverage, local coverage |
| Metrology | TLS / total station / GNSS qualified reference, independent check points | Bias, RMSE, repeatability, expanded uncertainty, interval coverage |
| Selective prediction | Fixed confidence threshold, risk-controlling prediction sets, human review | Risk-coverage curve, abstention fraction, decision loss |
| Distribution shift | Whole-site holdouts, leave-one-sensor-out, material/season/occlusion holdouts | Worst-stratum coverage, calibration drift, interval width increase |

### Data-splitting rules

1. Split by entire site, campaign and acquisition session. Never place scans from the same physical site in training and test.
2. Where spatial autocorrelation exists, use spatial blocks or buffered holdouts to prevent near-duplicate neighboring samples leaking across partitions.
3. Calibrate only on a dedicated calibration set; never choose conformal quantiles on the evaluation set.
4. Keep a sealed site holdout and a separate independent reference-measurement process.
5. Bootstrap by site/campaign, not by individual points, when estimating uncertainty in benchmark metrics.
6. Publish per-site and per-sensor distributions, worst strata, and coverage/width tradeoffs.

## 4. Initial target gates (proposed)

- Whole-site train/evaluation overlap: exactly zero.
- Every calibration metric is traceable to frozen predictions, reference measurements, configuration and dataset digests.
- Report proper scoring rules and calibration plots; ECE never appears alone.
- Empirical coverage is evaluated with confidence intervals against nominal coverage on independent sites.
- Interval width and mean error accompany coverage.
- Worst-sensor and worst-site coverage is mandatory.
- A system cannot pass by producing arbitrarily wide intervals: set maximum useful width per task before evaluation.
- Any uncalibrated or out-of-domain prediction must trigger an abstention or explicit downgraded evidence state rather than a false precision guarantee.

These are qualification rules, not claims that existing Sol-Atlas models meet them.

## 5. Next engineering increments

1. Add site-cluster bootstrap confidence intervals to the reference scorer.
2. Add equal-mass ECE, adaptive ECE and calibration slope/intercept, retaining raw reliability bins.
3. Implement split conformal and spatially weighted conformal baselines with a fixed calibration/evaluation split.
4. Add coverage-vs-width and risk-vs-coverage curves.
5. Add material, sensor, environment, scale and occlusion strata with worst-group reporting.
6. Integrate actual reference-survey records, GUM-style uncertainty budgets and sensor calibration receipts.
7. Couple interval and calibration quality to the EVSI investigation planner so high uncertainty is not automatically interpreted as a valuable survey target.

## References

- Lou, X., Luo, P. & Meng, L. (2025). [GeoConformal Prediction: A Model-Agnostic Framework for Measuring the Uncertainty of Spatial Prediction](https://doi.org/10.1080/24694452.2025.2516091). *Annals of the American Association of Geographers*.
- Luo, P. (2026). [Quantifying Uncertainty in Spatial Prediction for Nonstationary Spatial Processes](https://doi.org/10.1080/24694452.2026.2639721).
- [A Comprehensive Review of Classifier Probability Calibration Metrics](https://journals.sagepub.com/doi/full/10.1177/24518492261436231) (2026).
- Moyano, J. et al. (2026). [Metrological framework for digital twin-based measurement systems in cultural heritage](https://doi.org/10.1016/j.measurement.2026.122271). *Measurement*, 285, 122271.
- Waqar, A. (2026). [Performance benchmarking of scan-to-HBIM workflows for cultural heritage](https://doi.org/10.1016/j.autcon.2026.106848). *Automation in Construction*, 184, 106848.
