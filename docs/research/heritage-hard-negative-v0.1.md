# HDB-B9 — operational hard-negative challenge scorecard v0.1

## Scope

This evaluator is the first executable scoring contract for HDB-B9 (#559). It does not run a detector, match polygons, authenticate an archaeological interpretation or create reference labels. It consumes frozen, independently adjudicated counts and evidence receipts for a model operating at one declared confidence threshold.

The independent evaluation unit is inherited from HDB-20's evaluation provenance contract (site, region or source-paper test partition). Every report keeps unit-level results and identifies the weakest unit.

## New research baseline: DINO-CV

Huang et al. (2026), *Science of Remote Sensing* 13, 100372, present DINO-CV: self-supervised cross-view pretraining over high-resolution LiDAR DEM derivatives (multi-directional hillshade and visualization for archaeological topography). The Budj Bim study reports 68.6% mIoU on its test areas and 63.8% mIoU when fine-tuned with only 10% labelled data.

This is a new, task-matched baseline for topography-first wall segmentation and label efficiency. Reproduce it on its exact data, partition, resolution, class semantics and metric before making any comparison. Its reported mIoU is not directly comparable to Kweneng's geodesic-area categorical LightGBM log-loss or JAS stone-instance F1. The study concerns Budj Bim in Australia, not Southern Africa; cross-region transfer remains unproven.

## Input contract

Each input unit binds:
- existing HDB-20 EvaluationProvenance (dataset, model, prediction, reference, split, evaluator, metric-protocol and rights digests);
- site, region and acquisition identities;
- the actually surveyed and adequately assessed area in km², with an area receipt;
- completeness receipts for all candidate adjudication and reference coverage within that area;
- independently recorded expert review hours and a time receipt;
- confirmed constructed-wall reference count, TP and FN counts;
- hard-negative lookalike counts by declared class (eroded ledge, bedrock scarp, geological lineament, gully/erosion channel, terrace edge, road cut/track, modern retaining wall, modern kraal/enclosure, fence/boundary, other confirmed lookalike);
- background false positives, unresolved predictions, and explicit ambiguous / not observable / not assessed reference counts.

Incomplete candidate adjudication or reference coverage fails closed. Unsurveyed, inaccessible, rights-withheld and not-observable area cannot be included in the false-positive-per-km² denominator. Ambiguous and unresolved cases are reported separately; they are not silently counted as true or false positives.

## Metrics

At the frozen threshold the report provides:
- micro TP / FP / FN, resolved precision, recall and F1;
- macro precision / recall / F1 across independent units, with the number of units contributing to each;
- worst-unit recall and the complete per-unit scorecard;
- confirmed false positives per adequately assessed km² plus a conservative upper bound that treats every unresolved prediction as a false positive;
- verified findings per expert-review hour;
- false leads per expert-review hour;
- candidate resolution fraction and unresolved candidate count;
- per-lookalike class reference count and false-positive candidate count.

Precision is conditional on resolved predictions. The unresolved-prediction count and candidate-resolution fraction are always shown beside it. False-positive density and false-lead productivity are reported both as confirmed-only values and conservative upper bounds that count every unresolved candidate as a false positive. A model cannot improve apparent precision or hide unresolved burden by silently dropping difficult detections.

The per-class false-positive-candidates/reference-object ratio is a burden ratio, not a probability; multiple predictions can hit one negative object, so values above one are possible.

## Statistical constraints

No confidence interval is generated. Macro metrics are descriptive across the supplied independent units. HDB-20's site/region/acquisition-level bootstrap and spatial effective-information analysis are required before population-level performance claims. Never bootstrap individual pixels, chips or stones as independent sites.

The evaluator requires a common model artifact/build, split manifest, evaluator build, metric protocol and evaluation-unit kind across units. The confidence threshold must be accompanied by a receipt documenting that it was selected on validation data and frozen before unsealing the challenge partition.

## Benchmark targets and comparisons

- DINO-CV exact reproduction: recover 68.6% test mIoU and the 63.8% 10%-label result within preregistered tolerance on the exact published Budj Bim setup; failure is a replication result, not a model score to hide.
- Stretch: achieve ≥75% mIoU on the same task/split with matched label budget, and ≥68.6% with ≤10% labelled data. These targets are hypotheses, not achieved results.
- Cross-region test: separately evaluate leave-one-region-out and Southern-African transfer. Do not compare it to the Budj Bim score as though it were the same partition.
- HDB-B9 operational target: at a partner-approved recall floor, improve verified findings per expert-hour by ≥2× against a human-only workflow while keeping false positives per adequately assessed km² below a preregistered ceiling and reporting worst-unit recall.
- Do not set the false-positive ceiling before archaeological partners define the cost of a false lead, the reference sample size and the negative-area coverage needed for a useful upper confidence bound.

## Limitations

Synthetic fixtures only. No archaeological dataset, trained model, source LiDAR, field site or reference label was ingested or scored by this module. It is an operational scorecard contract, not evidence of detector performance or generalisation.