# Fixed-Reference Seasonal Robust Deviation (SRD) v0.1

## Purpose and claim boundary

SRD is a transparent temporal baseline for site-level feature monitoring. It compares each evaluation month with a frozen historical reference built from the same calendar phase. It is designed to separate predictable seasonal variation from unexpected feature changes before complex embedding or self-supervised pipelines are evaluated.

SRD identifies statistical deviation in a declared feature. It does not decide whether an archaeological feature exists, whether a disturbance is harmful, or what caused it. A robust z-score is not a probability.

## Method

For evaluation month t, collect valid observations in the fixed reference interval whose circular month-of-year distance from t is at most W. Require at least Y distinct reference years. If that requirement fails, emit InsufficientReference and no score.

Compute the reference median m and median absolute deviation MAD. Robust scale is max(1.4826 × MAD, scale_floor). The signed residual is x_t − m and robust score is (x_t − m) / robust_scale. Alert is a boolean comparison against an explicit absolute robust-score threshold. Threshold policy must be referenced and selected before the held-out test.

Reference end must be strictly earlier than evaluation start. The evaluator ignores all observations outside the frozen reference interval when calculating median/MAD, even if later observations are present in the supplied series.

Missing evaluation months and null feature values are Unobserved. They are never imputed as zero. The output materializes every month in the evaluation interval so gaps are auditable.

## Optional persistent shift: two-sided CUSUM

Positive and negative CUSUM are accumulated separately from signed SRD scores: C+ = max(0, C+ + z − k), C− = max(0, C− − z − k). Alert is not allowed until the segment contains the configured minimum number of valid scored months. A missing/unscored month gap strictly greater than reset_after_missing_months resets both sums and the valid-count. SRD and CUSUM outputs remain separate.

## Feature and acquisition contract

Every series must identify the feature, units, channel semantics, feature-definition reference, source reference and source-asset digest. This prevents arbitrary channel naming (e.g. treating an RGB validity mask as NIR) and keeps the input contract reproducible. Acquisition/phenophase metadata should be carried by the upstream observation asset and included in the run manifest.

## Required negative controls

1. Deterministic seasonal cycle with no injected event: no alert once reference coverage is sufficient.
2. Injected shift: signed deviation should be recovered with known direction and magnitude.
3. Missing months and null values: Unobserved, not zero.
4. Insufficient distinct reference years: abstention, not a synthetic negative.
5. Evaluation-window sensor/scale jump: score may alert but must not be labelled archaeological threat without a separate cause assessment.
6. Future observations in the input series: never influence the fixed reference statistics.
7. Persistent weak shift: CUSUM should accumulate according to the declared drift/minimum/gap policy.
8. Missing gap above reset limit: CUSUM resets before resuming.

## Benchmark comparison

Compare SRD and optional CUSUM against WATCH TED, SSCD and weakly supervised monthly scores on identical source assets, month availability, site splits and event labels. Include seasonal-naive, per-month median, harmonic regression where supported, and original WATCH scoring. Keep task units matched: event-month Recall@K is not pixel F1 or cause-classification F1.

Report robust-score distributions, alert burden per 100 observed site-months, event recall at fixed analyst review budget, early-warning lead, detection delay, abstention rate, coverage by reference years, calibration only after separate calibration fit, and worst-region performance. Stratify by biome, season/phenophase, sensor, cloud/no-data and observation cadence.

## Provisional research targets

- On seasonality-only negative controls, fewer than 1 false alert per 100 observed site-months after a sufficient reference period.
- On preregistered persistent-shift injections, median detection delay no worse than the strongest matched WATCH baseline at equal false-alert burden.
- On held-out sites with less than the minimum reference coverage, 100% of outputs abstain rather than impute.
- Do not claim improvement over WATCH until same-data, same-split, same-review-budget results and site-level uncertainty intervals are published.

These are targets, not current benchmark results.