# GMD Adaptive Residual Calibration — May 2024 Exploratory Pilot v0.1

**Status:** exploratory prequential diagnostic; not a new physical GIC model.  
**Date:** 2026-11-10  
**Claim ceiling:** within-event temporal validation only. No independent event or regional generalisation claim.

## 1. Research question

Can a model's local magnitude bias be corrected online from recent, quality-qualified GIC observations without using future samples, hiding physical model error, or transferring calibration beyond its valid site/topology/sensor context?

This is deliberately separate from the upstream physical chain:
`solar wind / geomagnetic forcing -> Earth response -> E-field -> induced voltage -> network GIC`

The calibrator consumes a qualified model output and a local measured residual stream. It may improve an estimate for a local site; it cannot repair a wrong Earth model, missing network path, unknown transformer configuration or invalid sensor.

## 2. Public data and baseline

Use the pinned May 2024 TVA replay record:
- [GMD_MAY2024_TVA_REPLAY_V0.1.md](GMD_MAY2024_TVA_REPLAY_V0.1.md)
- Dataset: https://github.com/lucywilkerson/SWERVE-2024-05-10
- Repository snapshot: `089617756920bcc9df621f7f2299ba0644320cc3`
- Publisher article: Wilkerson et al. (2026), DOI https://doi.org/10.1029/2025SW004758

The publisher comparison window is 10 May 2024 15:00 UTC through 12 May 2024 06:00 UTC. Measured 1-second GIC is averaged into complete, non-overlapping 1-minute bins (60 source samples per scored minute) to match one-minute TVA/GMU predictions. Sensor polarity remains explicit; for this paper-comparable analysis the measured series is sign-normalised at sites whose raw correlation is negative, following the publisher's documented orientation correction.

## 3. Causal prequential protocol

For each site/model pair:

1. Preserve the unmodified source measurement and model prediction.
2. Use the paper's declared sensor-polarity convention as an explicit input. It is never inferred silently from a production data stream.
3. Use the preceding 180 matched one-minute samples (three hours) to fit an ordinary least-squares affine map:
   `y = a + b p`
   where `p` is the raw model output and `y` is the polarity-normalised measured GIC.
4. Apply a fixed 0.5 shrinkage blend toward identity:
   `p_cal = p + 0.5[(a + b p) - p]`
5. Produce the next 60 one-minute predictions, then advance the window and refit using only measurements that have become available after those predictions.
6. Score only the predictions made after the initial three-hour history. Each scored prediction is produced before its corresponding measured value is observed.

This is a chronological prequential test: observations from an earlier portion of the same storm may update later forecasts, but future observations do not enter the transform for an earlier forecast. It is not a leave-one-event-out test. The three-hour history, hourly refit and 0.5 shrinkage were selected after exploratory comparison of parameter alternatives; the resulting test is therefore post-hoc rather than pre-registered.

## 4. TVA operator-model results

The scored portion contains 2,161 one-minute forecasts per site.

| Site | Baseline MAE (A) | Calibrated MAE (A) | MAE change | Baseline RMSE (A) | Calibrated RMSE (A) | RMSE change |
|---|---:|---:|---:|---:|---:|---:|
| Bull Run | 0.8012 | 0.5656 | -29.4% | 1.1010 | 0.7943 | -27.9% |
| Montgomery | 1.5439 | 1.0163 | -34.2% | 2.8962 | 1.9545 | -32.5% |
| Union | 1.6257 | 1.5292 | -5.9% | 2.2936 | 2.2340 | -2.6% |
| Widows Creek | 0.8654 | 0.7930 | -8.4% | 1.2178 | 1.1278 | -7.4% |

A paired one-hour-block bootstrap used 3,000 deterministic resamples of 37 within-event test blocks per site (36 full one-hour blocks plus the final partial block). Calibration-minus-baseline intervals:

| Site | MAE delta 95% interval (A) | RMSE delta 95% interval (A) |
|---|---:|---:|
| Bull Run | [-0.282, -0.190] | [-0.395, -0.234] |
| Montgomery | [-0.709, -0.365] | [-1.191, -0.674] |
| Union | [-0.191, -0.006] | [-0.151, +0.022] |
| Widows Creek | [-0.124, -0.018] | [-0.174, -0.017] |

Interpretation: in this single event, the calibrator reduced MAE at all four sites. RMSE block-bootstrap intervals were below zero at Bull Run, Montgomery and Widows Creek, while Union's interval crossed zero. These resamples are dependent within one event and do not estimate cross-storm uncertainty. The result does not establish that calibration improves an independent future storm.

## 5. Transfer check on the GMU reference model

The same 3-hour / hourly-update / 0.5-shrinkage rule was applied to the GMU reference-model median GIC at the same four substations. Hyperparameters were not retuned for this model family.

| Site | Baseline MAE (A) | Calibrated MAE (A) | MAE change | Baseline RMSE (A) | Calibrated RMSE (A) | RMSE change |
|---|---:|---:|---:|---:|---:|---:|
| Bull Run | 0.7428 | 0.7454 | +0.3% | 1.1259 | 1.1386 | +1.1% |
| Montgomery | 1.9445 | 1.9996 | +2.8% | 3.5355 | 3.6135 | +2.2% |
| Union | 1.8818 | 1.7668 | -6.1% | 2.7426 | 2.5930 | -5.5% |
| Widows Creek | 1.1855 | 1.1549 | -2.6% | 1.7624 | 1.7132 | -2.8% |

This transfer check is important: the calibration is not a universal uplift. It slightly worsened the already weaker GMU reference output at Bull Run and Montgomery while improving Union and Widows Creek modestly. Model family, site and local network/measurement context matter. No pooled cross-site correction should be inferred from four sites.

## 6. State of the art: what is already achieved elsewhere

This work is not competing against a blank field.

- The UK SWIMMR programme has operationalised nine models for real-time estimates and forecasts covering the Great Britain high-voltage network, high-pressure gas pipelines and railway network; its programme added three variometer sites and undertook MT measurements at 53 sites. Paper: Beggan et al. (2025), DOI https://doi.org/10.1029/2025SW004364.
- The September 2026 Fujian MM-CNN-MHA study reports 60-minute-ahead forecasts at one 220-kV substation. Direct GIC prediction reported RMSE 0.27 A; predicted-E-field-to-physics GIC reported 0.26 A; persistence rounded to 0.28 A. The paper also reports that a 3.28 A measured peak was predicted as 0.6 A by direct GIC prediction and 1.03 A by the field-mediated model, with high-amplitude training samples scarce. DOI https://doi.org/10.1029/2025SW004900.
- A 2026 study of a moderate storm in New Zealand reports GIC up to about 20 A for roughly 90 minutes. It argues that sustained moderate current can create transformer saturation/harmonic risk even when peak-only or global-index thresholds look unremarkable. DOI https://doi.org/10.1029/2026SW005032.

The opportunity is therefore not to add an attention layer to a single-site GIC time series. It is to build reproducible, multi-event and cross-site evaluation with strong physical baselines, explicit sparse-tail handling, local calibration validity and service/asset consequences.

## 7. Paradigm shift: prediction as a calibrated, stateful evidence object

Represent an estimate as a tuple, not a naked scalar:

`(raw_physical_output,
 local_calibration_state,
 sensor_health,
 polarity_and_orientation,
 topology_revision,
 model_revision,
 residual_history,
 uncertainty,
 validity_domain,
 abstention_state,
 provenance)`

The calibrator may issue one of:
- `CalibratedWithinDeclaredEnvelope`;
- `UncalibratedInsufficientHistory`;
- `AbstainSensorOrPolarityUnknown`;
- `AbstainClockOrAlignmentInvalid`;
- `AbstainTopologyOrModelRevisionChanged`;
- `AbstainDriftOrOutOfDomain`.

It never rewrites raw measurements and never converts a corrected prediction into a physically validated state. A topology, grounding, transformer, sensor or model revision change invalidates the prior calibration state unless an explicit compatibility proof exists.

## 8. Required benchmarks and hard negatives

### Data and calibration correctness
- Exact timestamp alignment and cadence; complete minute sample-count checks.
- Correct and explicitly recorded sensor polarity/orientation; no automatic sign flip based on the evaluation window.
- Reproducible causal window updates; no current/future observation leakage.
- Calibration-parameter uncertainty and prediction intervals.
- Residual autocorrelation and block/event-clustered confidence, not IID minute bootstrap.

### Negative controls
- Wrong polarity or polarity changed mid-event.
- One-minute clock offset, time-zone error, drift and unknown timestamp source.
- Missing, duplicated, reordered, stale or clipped sensor values.
- Sensor replacement/relocation and calibration change.
- Model revision change, stale geoelectric transfer function, network topology or grounding/winding change.
- Nearest-station substitution, wrong station pairing, source-model correlation and out-of-region extrapolation.
- Drift, heavy-tailed residuals, rare peaks and sustained low-to-moderate current episodes.

### Metrics
- MAE, RMSE, signed mean error/bias, PE, correlation, peak magnitude error, peak timing error.
- Event detection POD, POFD, HSS, threshold F1/precision/recall, false alarm rate and lead time for predeclared GIC/E-field thresholds.
- Tail quantile loss, exceedance calibration and interval coverage by event class/site.
- Duration above asset-specific thresholds, cumulative exposure proxies and time-integrated absolute GIC; these remain diagnostic until connected to transformer-specific thermal/harmonic models.
- Drift detection delay, stale-calibration false-acceptance rate, abstention coverage and selective risk.
- Cross-event, cross-region and leave-one-site-out transfer; no random row splitting.

## 9. Pre-registered next experiment

1. Build a deterministic offline Rust replay tool that emits a receipt bound to source commit, file blob SHAs, aggregation, polarity metadata, split, calibrator version and output digest.
2. Freeze a single algorithm and all hyperparameters before opening a held-out event.
3. Evaluate on event-held-out data where paired measured GIC and predictions exist; then leave-one-site-out transfer. May 2024 is development evidence, not the independent final holdout.
4. Compare identity/no correction, fixed gain, static affine, rolling gain-only, rolling affine, robust regression, exponentially weighted/Kalman bias-gain, and hierarchical partial pooling.
5. Add uncertainty calibration (e.g. conformal residual intervals) only with prequential coverage checks and regime-shift detection.
6. Compare the same algorithm across TVA operator and GMU reference outputs, and later against SANSA/Eskom historical series when permissions and exact data are available.
7. Report performance separately for high-amplitude spikes and sustained moderate currents.
8. Shadow-only for any utility data; no operational control or automatic public alert.

## 10. Claim ceiling

The current pilot is an exploratory, within-event, four-site calibration diagnostic. It does not demonstrate generalisation to a new storm, utility, network, transformer or geography. It does not validate geoelectric fields, network topology, transformer thermal response, harmonic behaviour or grid safety. The 0.5 shrinkage and 3-hour lookback were chosen post-hoc, and the block-bootstrap intervals are not independent-event uncertainty estimates.
