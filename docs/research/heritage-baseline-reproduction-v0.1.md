# Heritage baseline reproduction plan and source audit v0.1

**Research snapshot:** 2026-11-02  
**Parent:** HERITAGE-BASELINE-001 / issue #451  
**Status:** source-code and release audit only. No dataset archives downloaded, no model training executed, no performance reproduced.

## 1. Pinned upstream snapshots

The public upstream branches inspected for this audit were pinned to these commit IDs:

| Project | Upstream commit | Role |
|---|---|---|
| Microsoft HERITAGE | \`46d895bf2c983f6fc2269a1268922bf3ba247e73\` | Dataset documentation, release manifests and companion analysis metadata |
| Microsoft WATCH | \`ec6e1539673da46437c276c3ce42534982d2faff\` | Temporal change detection |
| Microsoft looted_site_detection | \`a80bdf1aab2a3976053bfe8b574942fd3eac8f9a\` | Site-state looting classifier |
| EarthObservation ADAF | \`a44acdd678773666458ea50f544b5d2292bcef51\` | Airborne LiDAR feature detector |

A baseline run must pin immutable upstream commits, not just a moving branch name. Re-check each upstream repository before starting a new reproduction because a source revision change may alter preprocessing, labels, feature extraction or evaluation.

## 2. HERITAGE release audit

The current public HERITAGE README describes:

- 1,943 Afghanistan sites: 898 looted and 1,045 preserved.
- 39 additional imagery-only sites from 15 other countries, making 1,982 sites across 16 countries.
- 212,776 monthly image chips across 113 month positions.
- Afghanistan observations from January 2016 through May 2025 in the release manifest, with up to 108 monthly images per site; the change-month annotation campaign covers 2016–2021.
- 118 of 898 looted Afghanistan sites have confirmed change-month labels. The remaining 780 looted sites have an unknown event month (\`looted_month = -1\`), not an event-negative label.
- The 2022–2025 images are available for monitoring, but are not month-localised by the described annotation campaign.
- One primary annotator produced labels, with qualitative expert review and case-by-case use of field records when available; the release reports no formal inter-annotator agreement statistic.
- Coordinates and polygons are withheld in the public release.

The dataset record is CC BY 4.0 (verify the release-level licence and its attribution each time). The GitHub repository contains code/docs under its own software licence. Do not infer the dataset licence from the repository's LICENSE file.

Data source:
- https://github.com/microsoft/HERITAGE
- Dataset rights record: https://datasetcatalog.nlm.nih.gov/dataset?q=0003365987

### 2.1 A 117 vs 118 annotation-count discrepancy

The HERITAGE README says 118 confirmed change-month sites. Earlier WATCH paper/results summaries reported 117 usable event-month labels in an experiment. The reproduction protocol must capture:

- the exact data-release manifest and ground-truth CSV digest;
- number of release-level event-month labels;
- any excluded/filtered sites and exclusion reasons;
- number of labels in each declared split;
- number of events evaluated after filtering.

Until those records are reproduced, do not silently replace 118 with 117 or vice versa. The difference may arise from split filtering, imagery availability or the exact experimental sample, but that explanation must be evidenced by the actual pipeline.

### 2.2 Public channel semantics are not WATCH channel semantics

The HERITAGE public PNG release documents four channels as RGB plus a per-pixel data-validity mask. The WATCH pipeline is documented around Planet monthly GeoTIFFs with RGB+NIR inputs; its code includes NIR-based indices such as NDVI, NDWI and BSI and a 4-channel (B,G,R,NIR) image conversion utility.

These tensors are not equivalent. The validity mask is not NIR. Passing it into an NIR slot creates meaningless spectral indices and is not a valid WATCH reproduction.

A release-specific channel contract must be inspected and recorded before loading:
- band/channel index;
- semantic identity (red, green, blue, NIR, data-validity mask, site geometry mask, etc.);
- units and scale;
- nodata convention;
- original order and any explicit reordering;
- transformation code digest;
- target model input contract.

The WATCH repository also contains separate code paths for embeddings (which may use RGB preprocessing) and handcrafted four-band features. The current source comments deserve an explicit audit of array ordering: the 4-band RGB converter describes B,G,R,NIR, while the handcrafted feature module documents assumptions about R,G,B,NIR. Do not resolve this by guessing. Record actual source GeoTIFF band metadata and test known channel values through the full preprocessing path.

A strict reproduction of a NIR-dependent WATCH baseline is **blocked** if only RGB+validity-mask PNGs are available. A RGB-only or three-band adapted model is a new baseline and must be named as such.

### 2.3 Missing data, radiometric shift and seasonal effects

The HERITAGE README reports:
- monthly acquisition gaps where no Planet mosaics were available; missing files are not filled;
- radiometric trends across time, including decreases in average RGB values from 2016 to 2019 followed by stabilisation;
- seasonal variation;
- Afghanistan sample no-data fractions averaging 4.7%, ranging up to 35.1%;
- irregular global footprints with no-data/padding fractions reaching 66% for some sites.

These are not cosmetic metadata. They affect the input distribution and alert probabilities. Baseline runs must preserve validity masks, missing months, no-data fractions and fixed preprocessing, and should separately stress-test temporal normalization and seasonality.

## 3. WATCH baseline audit

Pinned upstream: \`ec6e1539673da46437c276c3ce42534982d2faff\`.

WATCH describes three distinct method families:
- **TED:** training-free temporal embedding distance;
- **SSCD:** self-supervised reconstruction/forecasting/novelty;
- **WS:** weakly supervised temporal event localisation.

It expects monthly site images and optional site masks, with ground-truth fields \`site_name\`, \`split\`, \`looted\`, and \`looted_month\`. Its maintained pipeline references four-band GeoTIFFs and NIR-dependent handcrafted features. The repository code is MIT-licensed; the imagery and checkpoints are not automatically included or licensed by that code grant.

Important fair-comparison rules:
1. Reproduce each algorithm separately; TED, SSCD and WS do not have identical supervision or inference semantics.
2. Preserve the exact site split, months, mask and preprocessing.
3. Match the published event-month and temporal-margin definitions, including top-K and directional early/late margins.
4. Report exact-month and ±1/±3/±6-month results separately.
5. Preserve missing months; do not fill them as zero-change months.
6. Record the dataset event-label count before and after split/filtering.
7. Where NIR-dependent source bands are unavailable, mark strict reproduction blocked and report RGB-only adaptation separately.
8. The global 39-site subset is image-only: it cannot support quantitative binary-looting or event-month performance without independently authorised external labels.

Reference:
- https://github.com/microsoft/WATCH
- https://arxiv.org/abs/2605.08160

## 4. Site-state looting classifier audit

Pinned upstream: \`a80bdf1aab2a3976053bfe8b574942fd3eac8f9a\`.

The public repository has a dynamic feature-based split function that splits unique \`site_name\` values with stratification. Its CNN image split function likewise constructs a site-ID split and checks train/validation/test overlap. This is a valid site-disjoint starting point for the site-state task; it is not a geographic external holdout.

The README and training implementation distinguish selected-year experiments from all-year sampling:
- Using a selected year (the documented examples use 2023) gives a final-state image classification setting.
- The CNN path with no year filter samples among multiple monthly images while applying one final looted/preserved label. The code explicitly warns that a site looted mid-period may have pre-looting images carrying the final looted label. This creates temporal label noise; do not combine this experiment with the selected-year reproduction.
- The public CNN loader opens the PNG and converts it to RGB, which discards an alpha/data-validity channel. Site boundary masks are a separate input path. This must be recorded as the baseline's actual preprocessing, not described as validity-mask-aware.

The HERITAGE README baseline table reports:
- handcrafted logistic regression: F1 0.705, AUC 0.768;
- SatCLIP logistic regression: F1 0.685, AUC 0.779;
- SatMAE random forest: F1 0.635, AUC 0.741;
- ResNet-18: F1 0.807, AUC 0.859;
- EfficientNet-B0: F1 0.862, AUC 0.927;
- EfficientNet-B1: F1 0.936, AUC 0.975.

The February 2026 paper summary cited earlier reports a distinct selected setup and different best-F1 figure (0.926). These are not to be averaged or treated as a contradiction until the exact fold assignment, label table, image year, model checkpoint, threshold, augmentation, and reporting method are aligned. The run manifest should identify the source table and exact recipe for each value.

Reference:
- https://github.com/microsoft/looted_site_detection
- https://github.com/microsoft/HERITAGE

## 5. ADAF airborne-LiDAR baseline audit

Pinned upstream: \`a44acdd678773666458ea50f544b5d2292bcef51\`.

The published ADAF tool combines relief visualisations from airborne laser scanning with HRNet segmentation and Faster R-CNN object detection, targeting Irish enclosures, ringforts and barrows. Its paper reports 84% recall for known sites along a 197 km² infrastructure corridor and 116 potential candidates not noticed by manual inspection. Those 116 are candidates, not independently verified archaeological discoveries.

License separation is mandatory:
- source code is Apache-2.0;
- pretrained weight archive is a separate artifact, with its own licence/attribution/share-alike conditions;
- training/validation ALS source datasets are separate rights objects, and some additional data is available from the corresponding author on request.

A Fossett Laboratory catalog lists ADAF weights under CC-BY-SA-4.0; confirm the exact Zenodo archive licence and obligations before downloading, fine-tuning or redistributing any weight-derived model. Do not infer weight licensing from the software repository's Apache-2.0 code licence.

Reference:
- https://github.com/EarthObservation/adaf
- https://doi.org/10.1016/j.jasrep.2026.105733
- https://zenodo.org/records/20136757
- https://fossettlab.org/models/adaf/

## 6. Reproduction order

1. Hash and verify HERITAGE's public manifests and archive files; preserve its per-subset rights and source provenance. Do not attempt coordinate reconstruction.
2. Reproduce the selected-year site-state classifier on Afghanistan using the fixed site-disjoint split and exact model/augmentation recipe. Report confusion matrix, F1, precision/recall, PR-AUC, ROC-AUC and site-level confidence intervals.
3. Reconcile the README's 0.936 EfficientNet-B1 result with the paper's distinct 0.926 result; preserve both recipes if they differ.
4. Reproduce WATCH TED/SSCD/WS independently. Exact reproduction is blocked unless a channel-compatible input product (including NIR where required) and masks/checkpoints under permitted terms are available.
5. Reconcile the 117 vs 118 event-month count at dataset and split level.
6. Reproduce ADAF only on a rights-cleared ALS dataset and with the separate weight licence documented.
7. Run geographic external evaluation only where independent labels and authority exist. The 39 global HERITAGE image-only sites are exploratory, not a global labelled test set.
8. Publish adapted RGB-only variants separately from strict reproduction runs.

## 7. Claim ceiling

A baseline run manifest with a valid provenance, rights, channel, split and output digest establishes traceability. It does not establish that a dataset annotation is correct, a licence has been interpreted legally, a model is globally transferable, or any archaeological hypothesis is true.
