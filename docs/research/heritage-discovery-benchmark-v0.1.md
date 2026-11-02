# Heritage Discovery Benchmark (HDB): global evaluation protocol v0.1

**Status:** proposed benchmark protocol; no benchmark results or partner-provided labels are claimed.  
**Parent workstream:** SOL-ATLAS-HERITAGE-001  
**Purpose:** evaluate archaeological candidate discovery and survey prioritisation across genuinely unseen sites, regions, sensors, and environmental conditions.

## 1. Benchmark thesis

The benchmark must test whether a system helps archaeologists find and validate useful candidates in landscapes it has not already seen.

A single random tile split is insufficient. Nearby tiles share geology, vegetation, acquisition conditions, architectural style, and often the same site. Random splitting can therefore reward memorisation of site-specific context instead of transfer to new places.

HDB measures a system as a discovery workflow:
1. screen a broad area;
2. rank candidate features or survey cells;
3. communicate uncertainty and alternative explanations;
4. use a limited human review budget;
5. select a safe and lawful next observation;
6. compare predictions with independent archaeological assessment.

HDB is proposed as a benchmark, not an existing community standard.

## 2. Independent evidence tiers

Every label must include how it was established. The benchmark must not treat all labels as equivalent.

| Tier | Label basis | Permitted evaluation role |
|---|---|---|
| A — field assessed | Documented field observation by qualified personnel, with location accuracy, method, date, and interpretation basis | Primary headline precision/recall and validation metrics |
| B — independently expert reviewed | Expert interpretation supported by high-quality remote-sensing or archival evidence, independently adjudicated | Secondary evaluation; reported separately from field-confirmed results |
| C — catalogue/reference | Existing inventory or published record whose location, positional error, and evidence completeness may vary | Retrieval and contextual tests; not silently upgraded to field truth |
| D — model-proposed | Machine-generated, not independently adjudicated | Training candidate only after review; never treated as ground truth |
| U — unknown / unsurveyed | No reliable assessment | Unlabelled, not negative |

A site record must preserve the evidence tier, reviewer roles, survey date/method, positional uncertainty, and any access restrictions. Conflicting interpretations remain visible.

## 3. Tasks

### HDB-1: fine-scale feature detection

Input: aligned high-resolution imagery, LiDAR-derived terrain products, or drone orthomosaics.  
Output: georeferenced points, lines, polygons, masks, or instances for one named feature family.

Examples: wall segments, stone-built enclosures, terraces, canals, roads, mounds, quarrying traces, or other regionally defined feature families.

Evaluate each feature family independently. A generic “archaeological” class is too broad to be scientifically useful.

### HDB-2: regional candidate ranking

Input: a landscape-scale area with broad-coverage imagery and terrain context.  
Output: ranked cells or candidate regions for expert review or higher-resolution acquisition.

Do not pretend a 10–30 m satellite product can delineate a small wall. The task is prioritisation, not feature-level geometry at impossible resolution.

### HDB-3: cross-region and cross-tradition generalisation

Test a model on sites excluded from training and, where data permit, on a region/biome or cultural landscape outside the training distribution.

Separate physical morphology transfer from archaeological interpretation transfer. A model might generalise at identifying linear stone alignments but have no authority to infer their age or cultural meaning in a new region.

### HDB-4: multi-sensor contribution and missing-modality robustness

Compare each modality alone and in combinations. Test missing, degraded, misaligned, seasonally mismatched, and conflicting sensors.

Report the incremental lift of each modality. More sensors are not better if they introduce correlated errors or misregistration.

### HDB-5: temporal monitoring

Input: dated image/survey sequences.  
Output: candidate disturbance, erosion, construction, looting, vegetation, exposure, or condition changes with temporal confidence.

Separate a genuine change from differences in sensor, orbit, season, resolution, illumination, or processing.

### HDB-6: active survey / value of information

Input: candidate landscape, uncertainty map, available survey methods, resource budget, access/permit constraints.  
Output: a ranked non-invasive survey plan.

Evaluate whether the recommended survey reduces uncertainty and increases validated candidate yield per unit cost or expert time compared with fixed-grid, random, and expert-prioritised survey baselines.

## 4. Dataset construction

### Global sampling design

Recruit or assemble independent site packs across:
- tropical and subtropical forest;
- temperate woodland and grassland;
- savanna and shrubland;
- arid and semi-arid desert;
- highland and mountain environments;
- riverine and wetland landscapes;
- coastal, estuarine, and maritime settings;
- urban, peri-urban, and heavily modified land;
- open and vegetated archaeological surfaces.

Include diverse feature materials and land-use histories: stone, earth, brick, fired material, timber traces, rock art, agricultural earthworks, mining, roads, ports, water systems, and submerged heritage where data are appropriate.

This is a sampling ambition, not a claim that global reference data currently exist for all environments.

### Site-pack requirements

For each site pack record:
- stable site and feature IDs (pseudonymous where required);
- geographic footprint and positional uncertainty;
- source sensor, platform, acquisition date, resolution, orbit/band/polarisation where applicable;
- original asset hashes and licence/consent/usage restrictions;
- source CRS, vertical datum, registration error, processing history;
- annotation instructions, class ontology and ontology version;
- evidence tier, reviewer roles, adjudication history;
- surveyed area and effort, visibility/condition, and known non-detections;
- access tier and publication rules;
- data and model cards, including known coverage gaps.

A site pack must not enter a public training release if its rights or governance state is unknown or incompatible with that release.

### Negative and unknown examples

Curate hard negatives from:
- geological joints, rock ledges, talus, natural terraces and erosional gullies;
- modern field walls, fences, roads, drainage, forestry, and agricultural earthworks;
- recent construction, quarrying, spoil heaps, and utility corridors;
- vegetation edges, shadows, water margins and sensor artefacts.

Do not label an unvisited or unrecorded area “negative.” Maintain three states: positive, assessed negative under a declared survey method, and unknown/unassessed.

## 5. Split protocol: no geographic leakage

Every result must state its split policy. Minimum split suite:

1. **Within-site spatial blocks:** contiguous blocks with buffers larger than the empirically estimated spatial dependence range; useful for interpolation within known landscapes.
2. **Held-out sites:** entire sites excluded from training and tuning.
3. **Held-out region:** entire region/administrative area or coherent archaeological landscape excluded from training.
4. **Leave-one-biome-out:** test environmental shift when enough independent site packs exist.
5. **Leave-one-sensor/source-out:** test sensor/platform and processing transfer.
6. **Temporal holdout:** train on earlier acquisitions, test on later observations.
7. **External blind set:** evaluation data held by an independent steward; labels remain hidden until model, code, thresholds, and candidate outputs are frozen.

For HDB-3, report the held-out region/biome results separately. Do not combine easy within-site and difficult cross-region results into a single headline score.

Any tiles, overlapping chips, augmentations, derived terrain rasters, or duplicate images from one site must remain in the same partition. Build the split manifest at site/landscape level before generating tiles.

## 6. Metrics

### Feature and candidate detection

For instance/segmentation tasks:
- precision, recall, F1 at predeclared IoU/coverage thresholds;
- average precision / PR curve, with class counts and thresholds;
- object/segment matching rules and minimum mapping unit;
- boundary quality and geolocation error.

For sparse regional search:
- recall at fixed review budgets (for example, top K candidates per square kilometre or per analyst-hour);
- precision among the top K reviewed candidates;
- false candidates per square kilometre;
- site-level recall and candidate yield by landscape;
- free-response ROC (FROC) where candidate count versus sensitivity is informative;
- area under the precision-recall curve, not ROC-AUC alone for extremely sparse positives.

Do not compare pixel accuracy from one study with object F1, area-under-PR, or candidate yield from another as though they are interchangeable.

### Calibration and abstention

Report:
- Brier score or another suitable proper scoring rule;
- reliability diagrams and expected calibration error with binning details;
- calibration method and calibration-set separation;
- selective risk versus coverage: how error changes as the system abstains from uncertain regions;
- out-of-distribution and open-set failure rates.

Do not label a raw model logit, ranking score, anomaly score, or uncalibrated probability as a calibrated probability.

### Transfer and robustness

Report per-site and per-region results, macro averages, worst-region performance, and uncertainty intervals using site-level resampling rather than treating adjacent pixels as independent samples.

Test:
- held-out geography and cultural landscape;
- seasonal and temporal changes;
- vegetation/land-cover variation;
- sensor resolution and platform changes;
- missing modalities and alignment degradation;
- distribution shift and unknown feature families.

### Operational value

Measure:
- independently validated candidates per 100 km² and per archaeologist-hour;
- review minutes per accepted/rejected candidate;
- field survey hours and cost per independently validated candidate;
- reduction in area that needs intensive survey at a fixed target recall;
- useful discoveries and false leads under a fixed budget;
- value of information of model-directed versus random, grid and expert-directed survey plans.

Operational results must include the field confirmation method and its cost. A high model score is not a discovery.

### Governance and evidence integrity

Acceptance tests:
- 100% of candidates carry source, model/pipeline version, input/output digest, evidence tier and rights state;
- no restricted exact geometry appears in public exports, APIs, logs, screenshots or generated reports;
- unknown/denied usage rights stop or quarantine processing according to the source policy;
- every published claim links to supporting evidence and review status;
- reviewer corrections and disputed interpretations remain auditable;
- public exports never expose custodial knowledge or personal/community details without permission.

## 7. Baseline suite

| Baseline | Why include it | Required caveat |
|---|---|---|
| Expert visual interpretation | Establishes human review quality and effort | Record expertise, available context and time; inter-rater disagreement matters |
| Terrain derivatives + RF/LightGBM/XGBoost | Strong, explainable tabular baselines | Risk of relying on site-specific relief; must include hard negatives and held-out geography |
| Published drone stone/wall segmentation model | Strong local object/instance baseline | Reproduce original split and also add site-held-out evaluation |
| Sentinel-1 ascending/descending SAR | Tests radar contribution in suitable environments | Do not claim universal canopy penetration |
| Prithvi-EO-2.0 | Strong multitemporal Earth-observation pretrained representation | General EO performance does not establish archaeological accuracy |
| TerraMind | Multimodal EO model and modality reasoning baseline | Check model release, input requirements, licence, and actual reproducibility |
| Clay Foundation Model | Self-supervised EO embedding baseline | Evaluate its modalities and resolution compatibility |
| Late calibrated fusion | Transparent multisensor combination | Baseline before complex learned fusion |
| Open-set anomaly detector | Tests discovery beyond known labels | Anomaly is not archaeological probability |

### Literature anchors for reproduction

- Siteleki (2026), Kweneng stone-wall AutoML / LightGBM study: DOI 10.1111/arcm.70143.
- Uziel, Zohar and Jaffe (2026), drone imagery stone and wall detection; nine sites, 44 stone tiles / 5,563 shapes and 56 wall tiles / 673 shapes; Mask R-CNN ResNet-50; DOI 10.1016/j.jas.2025.106430; data/code at Zenodo DOI 10.5281/zenodo.15673668.
- Michenot et al. (2024), Sentinel-1 ascending/descending Maya-ruin study; DOI 10.1038/s41598-024-53068-2.
- Prithvi-EO-2.0 (2025); DOI 10.1109/TGRS.2025.3642610.
- TerraMind (ICCV 2025); DOI 10.1109/ICCV51701.2025.00693.

## 8. Initial performance targets (aspirational, not results)

These are proposed gates after a baseline pilot has frozen the exact task, split, review rubric and data coverage:

- **Detection utility:** at the same expert review budget, improve independently validated candidate yield by at least 20% relative to the strongest task-matched baseline, with a site-bootstrap 95% confidence interval whose lower bound is above zero.
- **Transfer:** demonstrate statistically supported improvement on at least two independently held-out regions without a severe regression in any reported region. Report every region, not only the aggregate.
- **Review burden:** reduce median review time per independently validated candidate by at least 20% without increasing false candidate burden at matched recall.
- **Calibration:** target ECE no worse than 0.05 on a separately held-out calibration/evaluation design, alongside Brier score and reliability plots; ECE alone is not an acceptance criterion.
- **Survey planning:** improve independently validated candidates per unit survey cost/time by at least 20% over predeclared random/grid/expert-prioritised baselines.
- **Data integrity:** 100% provenance and licence-state coverage; zero restricted-geometry disclosures in release tests.

The first pilot may show that one or more targets are unrealistic for a particular feature family. In that case, publish the baseline and revise only through a documented protocol update—not by tuning the test set or silently changing the metric.

## 9. Statistical and scientific controls

- Pre-register or version the primary task, primary metric, split manifest, thresholds, exclusions and stopping rule before evaluating the blind set.
- Compare systems on identical sites, assets, annotations, splits, and review budgets.
- Use site-level or landscape-level bootstrap intervals; do not use millions of correlated pixels to manufacture tiny uncertainty.
- Perform modality ablations and sensitivity tests.
- Report all attempted model configurations or declare the selection process.
- Keep the blind labels and test set under a separate steward where practical.
- Report model cards, dataset cards, annotation disagreements, and known failure modes.
- Avoid post-hoc claims that a model “discovered” features already included in training or public catalogues. Define novel/previously undocumented status through dated and independently checked reference sources.

## 10. Benchmark execution sequence

1. Establish rights-cleared data inventory and source registry; do not start with bulk downloads.
2. Select one initial feature family with independent labels and genuine hard negatives.
3. Freeze site-level splits before tiling or feature generation.
4. Reproduce a simple terrain baseline and the closest published model baseline.
5. Record per-region quality, calibration, failure modes and expert time.
6. Add foundation-model embeddings and modality fusion.
7. Run a hidden, independently stewarded regional test.
8. Design a small lawful, non-invasive survey to validate top-ranked candidates and a sample of model-rejected areas.
9. Publish positive and negative results with provenance, limitations, code and permitted data references.

The benchmark's primary claim is not that it finds more shapes. It is that, with the same available evidence and human review resources, it helps qualified researchers make better, safer and more cost-effective decisions about where to look next.


## 11. Baseline update — literature and project scan, 2026-11-02

The initial baseline list is incomplete without three recent archaeological-prospection efforts. Include these before claiming the benchmark represents the field.

### ADAF: airborne laser scanning detection (Ireland)

Čož et al. (2026), *Automatic detection of archaeological features from airborne laser scanning data using deep learning: Development and evaluation of the ADAF tool*, Journal of Archaeological Science: Reports 71, 105733, DOI 10.1016/j.jasrep.2026.105733.

- The published tool uses CNN-based detection from ALS derivatives, with Relief Visualization Toolbox (RVT) and AiTLAS.
- Training labels cover Irish barrows, ringforts and enclosures from an extensive archive of ALS data.
- In an active infrastructure development corridor test, the paper reports 84% recall for known archaeological sites and 116 potential new sites not noticed during manual inspection.
- The 116 outputs are potential candidates, not 116 independently proven discoveries.
- A March 2026 conference presentation reports preliminary transfer of an Irish-trained model to southern Herzegovina: about half of known barrows identified with few false positives, with dense clusters and rugged relief reducing accuracy.
- Public software is in the EarthObservation/adaf repository under Apache-2.0. The user-facing installation documentation is Windows 64-bit/Conda/Python-oriented and the README describes approximately 5 GB of pretrained weights. This makes it an important reproducibility/integration baseline, not a Rust architecture template.
- The article and repository are useful baselines for ALS feature detection. Do not imply the entire training archive or every model weight is automatically licensed for all re-use merely because the tool code is Apache-2.0.

References:
- Article and reported results: https://doi.org/10.1016/j.jasrep.2026.105733
- Zenodo article record: https://zenodo.org/records/20136757
- Tool source: https://github.com/EarthObservation/adaf
- Cross-region preliminary presentation: https://zenodo.org/records/20137748

### ESA Φ-lab: geospatial foundation models for archaeological prospection

The ESA Φ-lab/Cultural Heritage Technology project page documents an exploratory March–April 2026 evaluation of TerraMind and RemoteCLIP for settlement-mound (tell) mapping in Central Iraq.

- Seven-band input: Sentinel-2 RGB+NIR, Sentinel-1 VV+VH and Copernicus DEM resampled to 10 m.
- Expert annotations and comparisons with U-Net and Vision Transformer trained from scratch.
- TerraMind and RemoteCLIP used as frozen encoders with a trainable UPerNet decoder.
- Label-scarcity experiments range from 100% to 5% of training labels.
- Preliminary mini-dataset results covering approximately 20% of the total study area indicate TerraMind advantages in low-label regimes and improved results from combined SAR, optical and elevation inputs.
- The same project page lists full-area scaling, object/tile metrics, an Iran out-of-distribution test, and exploration of TESSERA/AlphaEarth embeddings as future work. These are not yet completed validations on that project page.

This is a highly relevant multimodal-GFM baseline. Our HDB contribution should be a rigorous independent evaluation suite around it: frozen geographic holdouts, object-level and site-level utility, calibrated uncertainty, explicit unknowns, rights/provenance, and survey value—not the claim that applying a foundation model to archaeology is itself novel.

Reference: https://cin.philab.esa.int/databases/projects/geospatial-foundation-models-for-archaeological-prospection-tasks

### Drone imagery: stone instances and wall segments

Uziel, Zohar and Jaffe (2026), *Semi-automatic detection of building stones and wall segments of archaeological ruins*, Journal of Archaeological Science 185, 106430, DOI 10.1016/j.jas.2025.106430.

This is a task-specific baseline for high-resolution drone orthomosaics and stone/wall segmentation across nine sites with differing vegetation, ground colour and material composition. The article reports strong results for the studied cases and provides a reproducible code/data bundle at Zenodo DOI 10.5281/zenodo.15673668. Keep its feature definition, ground resolution, annotations, matching rules and split design aligned when comparing results. Do not compare its pixel-wall F1 to regional candidate ranking or ALS object-level recall as a single leaderboard score.

Reference: https://doi.org/10.1016/j.jas.2025.106430

### Kweneng: terrain derivatives and LightGBM

Siteleki (2026), DOI 10.1111/arcm.70143, is a regionally important South African baseline. It uses high-resolution LiDAR/context data in Kweneng North and assesses AutoML for stone-wall identification; LightGBM and elevation are highlighted. The studied northern area is approximately 945 ha with 211 known stone-wall sites. The study itself motivates wider testing and stronger assessment of false positives. Treat this as a local task baseline, not independent proof of transfer to other southern African regions.

Reference: https://doi.org/10.1111/arcm.70143

### Sentinel-1 SAR: two viewing geometries

Michenot et al. (2024), DOI 10.1038/s41598-024-53068-2, uses Sentinel-1 ascending/descending observations to exploit directional response and identify some structures under jungle canopy. The paper describes limitations: C-band is generally not ideal for forest penetration, rough canopy signatures can obscure structures, and small regular terrain features can cause false alarms. Treat this as a targeted regional-screening feature and a hard-negative generator, not a universal subsurface or canopy-penetration capability.

Reference: https://doi.org/10.1038/s41598-024-53068-2

## 12. Competitive positioning and what would constitute an actual advance

| Existing capability | Strong point | Sol-Atlas should add | Evidence required before claiming an advantage |
|---|---|---|---|
| ADAF / RVT / AiTLAS | Operational ALS feature detector, specialist archaeological classes, accessible workflow and retraining | Sensor-flexible source registry, global rights/provenance, cross-domain evaluation, candidate evidence graph and active survey planning | Reproduce on the same ALS tasks, then demonstrate held-out-region utility and lower review/field cost |
| Drone stone/wall segmentation | Fine-grained stone instances and wall geometry from high-resolution imagery | Fuse local geometry with regional context and non-drone imagery; assess transfer beyond sampled landscapes | Same-image/same-label comparison, plus independent held-out sites |
| Kweneng LightGBM | Interpretable terrain-variable baseline on local stone-wall landscape | Stronger hard negatives, independent regions, confidence calibration, survey-budget measure | Same-region reproduction first; separate South African holdout second |
| ESA Φ-lab TerraMind/RemoteCLIP exploration | Multimodal EO foundation representations under label scarcity | Complete geographic OOD tests, uncertainty/abstention, site-level statistics and active survey economics | Replicate the stated setup where possible, then compare on a blind region with identical splits |
| Commercial high-resolution imagery/GIS | Potentially valuable imagery coverage, mature GIS and enterprise delivery | Open evidence contracts, reproducibility, source rights and interoperable outputs across providers | Compare on identical purchased/permitted assets; do not assume commercial coverage or pricing |
| Heritage inventories / Arches-style systems | Domain records, human workflows, governance and conservation management | Connect survey evidence, multi-sensor candidates, model lineage, benchmark and 3D reconstruction | Demonstrate standards compatibility and partner-authorised exchange, not just a new map UI |

There is no defensible single global “archaeological AI SOTA” score across these systems: they address different objects, sensors, resolutions, geographies, labels, and outcomes. HDB should publish task-specific scorecards and a workflow-level result at a fixed review budget.

### The intended competitive wedge

The differentiator is not merely “we use more sensors” or “we use a larger foundation model.” It is a reproducible chain:

**licensed observation → quality-aware processing → explicit evidence → competing explanations → calibrated prioritisation → human adjudication → best next survey → independently recorded outcome.**

A claim of superiority requires evidence for the complete chain, especially better candidate yield per expert-hour and per field-survey cost on independent geographies.

## 13. Global data access realities

The source inventory is not a globally complete archaeological training set.

- **Open Heritage 3D** publishes high-resolution terrestrial/aerial LiDAR and photogrammetry data with contributor-selected Creative Commons terms. The organisation says a typical site download is around 25 GB, and commercial use varies by project. Site-level licensing, access and reuse must be checked before every ingest or training use.
- **ARIADNE** provides a large searchable archaeology catalogue. Catalogue discoverability and metadata aggregation do not mean all underlying files are downloadable, harmonised, globally representative, or licensed for model training.
- **EAMENA/MarEA** is an important model for tiered heritage data access. Its current policy explicitly follows “as open as possible, as closed as necessary”; the public mode omits coordinates and site-condition data, while research access is reviewed. This is a production precedent for protecting sensitive site data.
- **Copernicus and Landsat** provide broad-coverage EO inputs, but their resolutions and observables determine the task: regional candidate screening is reasonable; identifying small walls from a coarse pixel is not.
- **OpenTopography and national LiDAR portals** provide valuable high-resolution ALS where available, but coverage, terms, format, and access differ by country and collection.

The coverage atlas must measure actual coverage and survey effort. Never render gaps as absence of archaeological material. A non-surveyed area remains unknown.

References:
- Open Heritage 3D FAQ and per-project licensing: https://openheritage3d.org/faq
- Open Heritage 3D dataset catalogue: https://openheritage3d.org/data
- ARIADNE portal: https://portal.ariadne-infrastructure.eu/
- EAMENA access policy: https://eamena.org/open-access-policy
- OpenTopography catalogue: https://portal.opentopography.org/datasets

## 14. First benchmark recommendation

Start with a **two-stage reproducibility challenge**, not a global model training campaign.

**Stage A — reproduce existing specialist detectors**
- Reproduce ADAF on the documented Irish ALS task using its permitted code, weights and data.
- Reproduce the drone stone/wall model on its published Zenodo data bundle, after confirming the dataset's precise licence and required attribution.
- Reproduce the Kweneng LightGBM baseline only when its exact source data, preprocessing and label access are authorised.
- Reproduce the ESA multimodal tell experiment only if its annotation/data access and use conditions are confirmed.

**Stage B — held-out transfer**
- Test the Irish ALS detector on a separately governed geomorphologically contrasting ALS dataset, using the publicly reported southern Herzegovina transfer as an external comparison lead.
- Test the drone stone/wall detector on a site that contributed no training tiles, with resolution and annotation protocol matched.
- Test multimodal tell mapping on a distinct, held-out landscape (the ESA page names Iran as planned OOD work; do not represent that test as already complete).
- Test the South African Kweneng model on a different southern African landscape, not adjacent Kweneng tiles.
- For every transfer, report the false-candidate burden, abstention, calibration and per-site results.

The initial release can reproduce public benchmarks without claiming global coverage. The global benchmark only becomes credible after independently governed site packs from multiple geographic/cultural regions and environmental settings are available.


## 15. Executable survey-planning and split-guard contract (2026-11-02)

The first Rust research implementation adds two core capabilities:

### Explicit expected-information-gain planner

Given a prior distribution over competing explanations H and a proposed observation action A, the planner computes:

\[
\mathrm{EIG}(A)=\mathcal{H}(H)-\sum_o P(o\mid A)\mathcal{H}(H\mid o,A)
\]

- It accepts explicit hypothesis priors and action-specific outcome likelihoods.
- It requires probabilities to be finite, complete, and normalized within a declared tolerance.
- It rejects missing or extra hypothesis keys, duplicate IDs, invalid probabilities, and empty likelihood provenance.
- It reports EIG in nats, expected posterior entropy, cost, and EIG/cost as separate fields. It does not hide trade-offs inside an arbitrary weighted score.
- It rejects actions where data rights, required land access, method permit, or custodial review are not confirmed, and where cost or declared impact exceed the caller's constraints.
- It deterministically ranks eligible actions and retains ineligible actions with the first rejection reason.

The model does not fabricate priors, costs, likelihoods, permits, or field permissions. It consumes probability models and declarations supplied by a research team. EIG is a decision-support measure, not an archaeological truth score.

### Benchmark split guard

The core validator distinguishes two different evaluation designs:

- **Held-out site/region/sensor/time:** train, validation and test site IDs must be unique and disjoint.
- **Within-site spatial blocks:** the same site may contribute spatially separated blocks to multiple partitions, but every block must have a stable ID and an externally computed nearest cross-partition separation. The declared buffer must meet or exceed the estimated spatial-dependence distance.

For spatial-block runs, the receipt records the external geometry-audit reference and explicitly states geometry_distances_recomputed_by_core = false. This is intentional: the Rust core checks declared audit evidence and threshold consistency; a pinned geospatial adapter must compute and preserve the actual geometry-distance results. Site-ID disjointness alone cannot prove the absence of duplicate imagery, overlapping campaigns, or derived-tile leakage.

These are executable contract and negative-control tests over synthetic data. They are not benchmark scores or evidence that a real-world survey has improved discovery yield.


## 16. Expanded split contracts: region, biome, sensor, time and blind holdout

The executable split contract now distinguishes:

- **Held-out region:** sample-level assignments carry region IDs; all test samples must belong to the declared held-out region; train/validation samples must not; a site may not cross partitions.
- **Leave-one-biome-out:** same rule for a declared biome.
- **Leave-one-sensor-out:** sample assignments carry sensor IDs; all test samples must belong to the held-out sensor, while train/validation samples must not. This protocol may intentionally compare sensors at sites that occur in more than one partition; that design must be explicit in the preregistration and should not be mistaken for geographic generalization.
- **Temporal holdout:** acquisition intervals in train/validation must end on or before the cutoff; test intervals must start strictly after it. The same site may appear before and after the cutoff by design.
- **External blind:** a separately controlled evaluation reference is mandatory, in addition to site-disjoint train/validation/test IDs.
- **Within-site spatial blocks:** block assignments and an external CRS-aware distance audit remain mandatory, with the buffer at least as large as the estimated spatial-dependence range.

The validator does not compute geometry or confirm the authenticity of an external blind custodian. It checks supplied declarations, exact sample/site IDs, timestamps, groups and digest inputs. The split-validation receipt records its scope.

This reduces obvious site/region/time leakage. It does not prove that duplicated imagery, near-duplicate raster chips, overlapping survey campaigns, derivatives, labels copied between catalogues, or leaked model pretraining data are absent. Those require a separate asset-level and lineage audit before any blind evaluation is trusted.
