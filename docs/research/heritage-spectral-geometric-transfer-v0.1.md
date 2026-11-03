# Directional Spectral-Geometric Transfer (DSS) v0.1

## Research basis

Gravanis and Agapiou, PNAS Nexus, published 10 October 2026, report cross-location transfer of Sentinel-2 cropmark vegetation proxy signals across three European landscapes. Their result associates transferability with spectral-geometric similarity and finds transfer to be directional; NDVI phenophase alignment is used. The study supports a cropmark-specific proxy-transfer experiment, not universal archaeological detection. DOI: https://doi.org/10.1093/pnasnexus/pgag336.

The paper's exploratory scope is only three landscapes and six directed site pairs. Its spectral labels are visually interpreted cropmark-proxy pixels, not complete field-verified archaeological inventories. The paper is CC BY-NC; that license does not authorize commercial reuse. Inspect the supplementary data and each artifact's license separately before using data or code.

## Directional Strength Score

For ordered source site 1 and target site 2, and each shared phenophase month m:

- d_AA(m): distance between source archaeological-proxy A and target archaeological-proxy A spectral subclusters.
- d_AH(m): distance between source A and target vegetated-control H subclusters.
- r_12(m) = d_AA(m) / d_AH(m).
- Select the M shared months with the smallest positive r_12 values.
- DSS_12 = mean(1 / r_12(m)) across those selected months.

Lower r and higher reciprocal affinity indicate stronger relative spectral similarity. Reverse direction 2 -> 1 is a separate run with its own distances. Never symmetrize the pair or infer that A->B equals B->A.

## Input contract

Each ordered pair records source and target site IDs, distance metric, source/target split-manifest digests, feature-transform digest, input channel semantics, phenophase filter reference, top-M and minimum shared-month count. Every month records source A sample count, target A sample count, target H control count, A-A distance and A-H distance.

Strictly positive finite distances are required. Zero or non-finite distances fail closed because the ratio/reciprocal is undefined or unbounded. Shared months must be unique and strictly increasing. The selected top-M months and every monthly ratio/reciprocal are retained in the report.

## Split protocol

The original study's pixel-random split is available only as an explicitly labelled within-site interpolation reproduction. Primary external-generalization results require held-out-site or held-out-region evaluation. Spatially blocked splits are useful leakage controls but do not by themselves establish a held-out site. Tiling, crop extraction and augmentation must occur after site/region partitioning.

## DSS validity across many landscapes

The validation evaluator compares DSS with an observed held-out transfer outcome (F1, PR-AUC or recall at a fixed review budget) using Spearman rank correlation. It requires at least six directed pairs across four unique sites and reports leave-one-site-out sensitivity.

Directed pairs share site nodes and are dyadically dependent. v0.1 intentionally emits no naive pair-level confidence interval. A later benchmark release should use a site-level dyadic/multiway bootstrap or a larger independent-site design before inferential claims.

## Baselines and improvements over the paper

Compare DSS against geographic distance, biome/land-cover similarity, phenophase-alignment similarity, spectral-distribution similarity, terrain/geomorphology similarity and a combined transfer predictor. For predictive transfer, compare RBF SVM, constrained XGBoost, LightGBM, remote-sensing foundation embeddings and calibrated late fusion under identical held-out site/region splits.

Do not compare a cropmark-proxy DSS against wall segmentation, looting detection or airborne LiDAR object detection. Extend the protocol to another archaeological proxy only after defining its own A/H classes, ecological meaning, annotation protocol and independent labels.

## Provisional targets

- Reproduce the published three-landscape directional ranking before attempting to improve it.
- Expand to at least 10 independent landscapes across more than one cropmark ecology before claiming global transfer.
- Target at least +10 absolute percentage points in worst-target-region recall at the same analyst review budget versus geographic-nearest transfer, without worse worst-region false-positive burden or calibration.
- Report every ordered direction, leave-one-site-out stability, per-target PR-AUC, recall@budget, calibration, OOD abstention and sample counts.

These are research targets, not Sol-Atlas results.