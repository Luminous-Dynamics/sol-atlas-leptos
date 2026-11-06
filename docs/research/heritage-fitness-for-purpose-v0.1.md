# HDB-B10 — task-specific fitness-for-purpose evaluation v0.1

## Objective

This module prevents a common evaluation error: treating good object geometry or detection scores as proof that an extracted archaeological dataset supports every downstream claim.

Each report is evaluated against exactly one preregistered intended use:

1. feature extraction / cataloguing;
2. site-level descriptor estimation;
3. between-site descriptor comparison.

A pass at one level never implies a pass at another. Spatial-pattern inference and survey-allocation utility remain separate future profiles.

## Research basis

McCoy & Moncrieff (2026), *Heritage* 9(8), 327, DOI 10.3390/heritage9080327, provide a useful empirical example. Their legacy-map extraction workflow reported matched-detection mIoU around 0.89–0.94 while mean F1 across sites was 0.528. Their task-specific Gini analysis showed that the quality threshold for a site-level estimate differed from the higher threshold needed for reliable between-site comparisons (about F1 > 0.80 in that terrace-map task).

Those thresholds are **not** universal targets for stone-wall detection. HDB-B10 adopts the method—evaluate the downstream use separately—not the numerical thresholds.

## Contract

A Fitness Profile binds:
- intended use and plain-language claim;
- independent unit kind (site, region or acquisition; never pixel, tile or stone);
- metric-definition digest, source dataset and prediction digests;
- split manifest, validation receipt, method build and rights receipt;
- task-specific gates selected before final evaluation.

The unit input preserves:
- TP/FP/FN counts and matched-detection IoU;
- reference and predicted downstream descriptor values;
- explicit NotAssessed status and reason.

NotAssessed units are not converted into negative labels or zero-valued measurements.

## Metric semantics

### Feature extraction

Reports macro precision, recall and F1 across independent units, pooled precision/recall/F1, and macro matched IoU. Geometry IoU is a separate gate; it cannot substitute for detection count quality. A site with no defined F1 remains visible and does not silently contribute zero or one.

### Site descriptor estimation

Reports descriptor MAE and signed bias across the independent units. The descriptor and units are explicitly named. A model may pass feature extraction while failing this gate.

### Between-site comparison

Requires at least three evaluated independent units and reports Spearman rank correlation between reference and predicted descriptors, alongside descriptor MAE and absolute bias. Ties use average ranks; constant/undefined rank vectors fail the rank gate.

## Statistical boundary

This module produces deterministic per-unit point estimates and threshold gates. It does **not** compute population confidence intervals. HDB-20 remains responsible for cluster-aware resampling, effective information and uncertainty intervals. HDB-23B membership-sensitivity envelopes remain a separate within-evidence uncertainty layer; they must not be collapsed into between-site sampling uncertainty.

## Acceptance controls

- High matched IoU with low F1 fails a feature-extraction profile.
- A feature-extraction pass does not imply a site-descriptor pass.
- Low descriptor MAE does not rescue reversed between-site rankings.
- NotAssessed units remain explicit and do not enter denominators as negatives.
- Reordering input units does not change the canonical report.
- Pixel/tile/stone are not valid independent-unit kinds.
- Ties use average ranks; constant rank vectors are undefined and fail closed.
- All reports bind data, split, metric, build, validation and rights provenance.

## Claim ceiling

The strongest claim is the profile's intended use, and only when all its gates pass. A failed profile yields `not_fit_for_declared_use`. No chronology, ethnicity, identity, social hierarchy or historical interpretation is inferred from these evaluation metrics.
