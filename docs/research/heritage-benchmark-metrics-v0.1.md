# HDB-20: Executable Stone and Wall Benchmark Metrics v0.1

## Purpose

HDB-20 turns the published stone-instance and wall-segment tasks into a deterministic evaluator with explicit metric semantics and source-pinned receipts. It is a benchmark implementation, not a detector or an archaeological interpretation engine.

## Implemented metric policy

### Stone instances

- Raster dimensions and zero-based row-major pixel indices are explicit.
- Every mask must be non-empty, in bounds, strictly increasing and duplicate-free.
- Predictions are ranked by descending confidence, with prediction ID as the deterministic tie-break.
- Each prediction is greedily assigned to the unmatched reference with highest IoU at or above the declared threshold. Equal-IoU reference ties resolve by reference ID. A reference can be matched once.
- Precision, recall and F1 are computed at the declared confidence operating point.
- AP uses all confidence-ranked predictions, the same IoU threshold and an all-points interpolated precision envelope. This policy is pinned here; it must not be called paper-equivalent until the released authors' code and matching rules have been inspected and reproduced.
- The size-rule identifier records upstream eligibility filtering. This evaluator does not infer centimetres or apply physical-size cutoffs to pixel masks.

### Wall segments

- Pixel precision, recall and F1 use the union of eligible reference wall pixels and confidence-filtered predicted wall pixels.
- Segment coverage is intersection with the opposite-side union divided by the segment's own pixel count.
- Multiple predicted fragments may jointly cover one reference wall segment. This is an explicit union-coverage policy.
- Only reference segments marked eligible enter metric denominators. Ambiguous, not-assessed and excluded segments remain visible in the report and are not silently converted into negatives.
- Coverage thresholds are explicit, unique and strictly descending. The conventional comparison points are 0.7, 0.6 and 0.5.

### Undefined denominators

An empty denominator is reported as absent in JSON, never as a perfect score. When reference positives exist but no predictions are selected, recall and F1 are zero and precision is undefined. When no eligible reference exists, recall, F1 and AP are undefined.

## Provenance and resource limits

Each evaluation binds the DOI or source identifier, dataset, model artifact, model build, predictions, reference labels, HDB-19 split manifest, evaluator build, metric protocol, rights receipt and independent evaluation-unit identity. SHA-256 fields are syntax-checked; this module does not read or authenticate the corresponding bytes or receipt.

The evaluator rejects duplicate IDs, non-finite/out-of-range confidences, invalid thresholds, malformed masks and oversized object/mask collections. Canonical sorted masks make the arithmetic deterministic. The current v0.1 implementation operates on in-memory raster masks; streaming and packed-bit representations are later performance work.

## Published baseline

Uziel, Zohar and Jaffe, *Semi-automatic detection of building stones and wall segments of archaeological ruins*, Journal of Archaeological Science 185 (January 2026), article 106430, DOI [10.1016/j.jas.2025.106430](https://doi.org/10.1016/j.jas.2025.106430). The paper reports stone precision/recall/F1 of 0.87/0.79/0.83, 0.86/0.85/0.86 and 0.86/0.87/0.86 at the 15 × 15, 20 × 20 and 25 × 25 cm² minimum-size settings. Its wall table reports one pixel precision/recall/F1 triple (0.80/0.78/0.79, in the row labelled coverage threshold 0.7). It separately reports covered-ground-truth / covered-prediction segment rates of 0.82/0.83 at 0.7, 0.87/0.87 at 0.6 and 0.91/0.89 at 0.5. The paper does not publish additional pixel F1 values in the 0.6 and 0.5 rows; 0.91 is a covered-ground-truth segment rate, not pixel F1. The authors released their source data and code through [Zenodo record 15673668](https://doi.org/10.5281/zenodo.15673668).

These are reference results, not results produced by this evaluator. Exact reproduction requires inspection of the released labels, tile-generation and split manifests, object matching, wall-coverage implementation, post-processing, model weights and thresholds. A numerically similar result under a different matching policy is not a faithful reproduction.

## External controlled-geometry opportunities

The [EPFL EESD open-data catalogue](https://www.epfl.ch/labs/eesd/data_sets/) provides geometrical digital twins of three laboratory-built rubble-stone masonry walls, including individual-stone geometry and positions, point clouds and surface meshes. The [Masonry Microstructure Database](https://www.epfl.ch/schools/enac/masonry-microstructure-database/) describes 36 walls, including 12 real and 24 virtual examples.

These are valuable controlled-geometry and algorithm-validation supplements for stone extraction, registration and shape measurements. They are not archaeological-site ground truth and must not be mixed into the nine-site archaeological test set or used to claim historical generalisation. Dataset-specific access and reuse conditions must be checked before ingestion.

## Industry comparison

ArcGIS Pro already provides object-detection accuracy tooling with precision, recall, F1 and AP, as well as COCO mAP and precision-recall outputs. OpenDroneMap already generates photogrammetric products such as orthophotos, point clouds, textured meshes and elevation models.

Sol-Atlas should interoperate with these mature systems rather than rebuild photogrammetry or a general-purpose GIS. Its differentiator is to bind evaluation outputs to rights-aware inputs, explicit split estimands, stable physical-feature identities, independent measurement certificates, uncertainty and archaeological claim boundaries. HDB-20 supplies the task-specific metric semantics; HDB-18 and HDB-19 supply metrology and split qualification.

## Competitive targets — preregistration candidates, not results

| Task | Published reference | Stretch target |
|---|---:|---:|
| Stone F1, 15 × 15 cm² | 0.83 | ≥0.90 |
| Stone F1, 20 × 20 cm² | 0.86 | ≥0.92 |
| Stone F1, 25 × 25 cm² | 0.86 | ≥0.92 |
| Wall pixel P/R/F1 (paper-reported setting) | 0.80 / 0.78 / 0.79 | F1 ≥0.84 |
| Reference segments covered ≥0.7 | 0.82 | ≥0.88 |
| Reference segments covered ≥0.6 | 0.87 | ≥0.92 |
| Reference segments covered ≥0.5 | 0.91 | ≥0.95 |
| Predicted segments covered ≥0.7 | 0.83 | ≥0.88 |
| Predicted segments covered ≥0.6 | 0.87 | ≥0.92 |
| Predicted segments covered ≥0.5 | 0.89 | ≥0.93 |

Here, "segments covered ≥t" means the fraction of segments whose overlap coverage reaches t; it is not pixel precision, recall or F1. Do not invent pixel F1 values for the paper's blank 0.6/0.5 cells. Every target must be reported with macro-by-site and worst-site results, cluster-aware uncertainty, confidence calibration, selective risk/coverage, annotation disagreement and review burden. No pooled metric can substitute for a held-out-site result.

## Claim boundary

This module computes metrics over supplied masks. It does not ingest imagery, run a detector, authenticate dataset bytes or rights, verify reference-label custody, prove annotation correctness, estimate physical measurement accuracy, establish site transfer, or establish any archaeological conclusion. A successful synthetic test is software qualification only.

The next data milestone is to freeze the rights-cleared Zenodo package, reproduce its original procedure and metric definitions, and then run the same evaluator under HDB-19's spatially independent and leave-one-site-out estimands.
