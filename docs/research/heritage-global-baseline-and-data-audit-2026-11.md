# Sol-Atlas Heritage Intelligence: global baseline and data-access audit

**Research snapshot:** 2026-11-02  
**Parent:** SOL-ATLAS-HERITAGE-001 / draft PR #429  
**Purpose:** record the current competitive landscape and identify a fair first implementation path. This is a literature and source audit, not a reproduction report.

## 1. The competitive picture is now clearer

Archaeological remote sensing is no longer limited to GIS-based visual inspection and isolated classification experiments. There are at least four meaningful baseline families:

1. Specialist airborne LiDAR archaeological detectors.
2. Fine-resolution drone imagery models for stones and wall segments.
3. Terrain-variable models such as LightGBM for specific regional feature families.
4. Geospatial foundation models using multimodal satellite and elevation data under label scarcity.

These are complementary capabilities, not directly comparable leaderboards. Their input sensors, spatial resolution, feature ontology, geography, labels, validation design, and unit of evaluation differ. The correct question is not “which model has the highest F1?” but “which method delivers more independently validated useful candidates at the same expert-review and survey budget in the target domain?”

## 2. Important current baselines (checked 2026-11-02)

### 2.1 ADAF — airborne LiDAR / archaeological feature detector

Čož et al. (2026), *Automatic detection of archaeological features from airborne laser scanning data using deep learning: Development and evaluation of the ADAF tool*, Journal of Archaeological Science: Reports 71, 105733, DOI 10.1016/j.jasrep.2026.105733.

The article describes CNN-based detection from ALS data, with RVT-derived visualisations and AiTLAS model tooling. Training labels cover Irish barrows, ringforts and enclosures. The article reports 84% recall for known archaeological sites on an active infrastructure development corridor and 116 potential sites that had not been noticed during manual inspection. Those 116 are model-proposed candidates, not automatically confirmed discoveries.

The March 2026 LAC presentation “Barrows beyond borders: how far can an Irish model see?” reports an early transfer test in southern Herzegovina, a geomorphologically contrasting karst landscape: around half of known barrows were identified with few false positives, while dense clusters and rugged relief reduced detection accuracy. This is precisely the type of evidence HDB should measure systematically.

ADAF is a strong current ALS baseline. Its code repository declares Apache-2.0; its current README documents a Windows 64-bit / Python 3.9 / Conda-based workflow and approximately 5 GB of pretrained model weights. Separate software/data assets require their own licence check. We should wrap and reproduce the baseline rather than copy its stack into the Sol-Atlas core.

References:
- Article and performance statement: https://zenodo.org/records/20136757
- Tool repository: https://github.com/EarthObservation/adaf
- Cross-region transfer presentation: https://zenodo.org/records/20137748

### 2.2 ESA Φ-lab — archaeological geospatial foundation models

The ESA Φ-lab project page describes exploratory research conducted March–April 2026 using TerraMind and RemoteCLIP for ancient settlement-mound (tell) mapping in Central Iraq.

Input comprised Sentinel-2 RGB+NIR, Sentinel-1 VV+VH and Copernicus DEM (resampled to 10 m), with expert archaeological labels. The evaluation used frozen GFM encoders and a trainable UPerNet decoder, compared with U-Net and Vision Transformer baselines trained from scratch. Label subsets ranged from 100% to 5%. Early results on a mini-dataset covering approximately 20% of the total study area indicate benefits for TerraMind in low-label regimes and multimodal inputs.

The project page lists full-area scaling, object/tile-level metrics, an Iran out-of-distribution region, and TESSERA/AlphaEarth embeddings as future work. We must not report these planned tests as completed results.

This work raises the bar: HDB must include foundation-model baselines and label-efficiency curves. The stronger Sol-Atlas contribution should be external blind geography, calibrated uncertainty, candidate utility at fixed human budgets, modality missingness, source rights and active survey selection.

Reference: https://cin.philab.esa.int/databases/projects/geospatial-foundation-models-for-archaeological-prospection-tasks

### 2.3 Drone imagery — building stones and wall segments

Uziel, Zohar and Jaffe (2026), *Semi-automatic detection of building stones and wall segments of archaeological ruins*, Journal of Archaeological Science 185, 106430, DOI 10.1016/j.jas.2025.106430.

The paper provides a drone-imagery workflow with georeferenced stone instances and wall-segment outputs across nine sites with differing vegetation, ground colour and material composition. The authors provide code and data via Zenodo DOI 10.5281/zenodo.15673668. This is the right fine-scale baseline for stone/wall extraction, but it must be compared using the same image resolution, annotations and matching rules, then tested on held-out sites.

Reference: https://doi.org/10.1016/j.jas.2025.106430

### 2.4 Kweneng — South African GeoAI / LightGBM

Siteleki (2026), *Automating Archaeological Discovery: Assessing Geospatial Artificial Intelligence (GeoAI) Tools for Stone Wall Identification in Kweneng, South Africa*, Archaeometry, DOI 10.1111/arcm.70143.

The study covers about 945 ha in northern Kweneng with 211 known stone-wall sites. LightGBM was particularly effective among the tested AutoML methods, and elevation was an important variable. The author frames this as a preliminary step and notes the need for broader evaluation, especially false positives such as natural geology.

It is an important local baseline; the data and model should not be assumed to transfer beyond the study landscape.

Reference: https://doi.org/10.1111/arcm.70143

### 2.5 Sentinel-1 directional SAR

Michenot et al. (2024), *Identification of Maya ruins covered by jungle using Sentinel-1*, Scientific Reports, DOI 10.1038/s41598-024-53068-2.

The paper uses different ascending/descending look directions to exploit scattering anisotropy and identify some candidate structures in specific Maya landscape settings. It explicitly notes limits: C-band is generally not ideal for forest penetration, rough canopy signatures reduce certainty, and small regular terrain features can create false alarms. It should be a targeted screening baseline, not a claim of general radar vision through forest or soil.

Reference: https://doi.org/10.1038/s41598-024-53068-2

## 3. Standards and dataset access are a first-order engineering concern

### Open Heritage 3D
The repository contains high-resolution terrestrial/aerial LiDAR and photogrammetry. It reports average per-site downloads around 25 GB. Dataset licences are contributor-selected Creative Commons variants; commercial-use permission differs by project. The download workflow requests user and organisational contact information. This is valuable data, but ingestion needs per-project rights review, contributor attribution, and storage planning.

### ARIADNE
The ARIADNE catalogue enables searching and browsing archaeological fieldwork, reports, research papers, datasets, sites and artefacts. Its catalogue is a discovery index over heterogeneous providers; do not assume a record grants access to raw data or permission for training.

### EAMENA / MarEA
EAMENA explicitly follows “as open as possible, as closed as necessary.” Its public access mode omits precise coordinates, map zoom and site-condition information; research access is reviewed. This is an existing example of the tiered access architecture we should implement, rather than assuming that all heritage data should be fully public.

### Global source coverage
Sentinel and Landsat supply broad-coverage EO inputs. High-resolution airborne LiDAR and archaeologist-labelled site packs remain distributed unevenly across national portals, project archives and institutional repositories. A world-scale interface must show actual coverage and unknown regions, not imply global completeness.

References:
- Open Heritage 3D FAQ: https://openheritage3d.org/faq
- Open Heritage 3D dataset catalogue: https://openheritage3d.org/data
- ARIADNE portal: https://portal.ariadne-infrastructure.eu/
- EAMENA access policy: https://eamena.org/open-access-policy
- STAC 1.1.0: https://www.ogc.org/standards/stac/
- CIDOC CRM / ISO 21127:2023: https://www.iso.org/standard/85100.html
- CARE Principles: https://www.gida-global.org/careprinciples

## 4. How Sol-Atlas can surpass individual tools without pretending to already have

| Competitor class | Existing strength | Proposed Sol-Atlas differentiation | Required proof |
|---|---|---|---|
| ADAF / ALS tools | Feature-specific airborne LiDAR detector and retraining workflow | Multi-sensor source/evidence graph, global transfer testing, policy gates, review-budget ranking, active survey planning | Same ALS task reproduction plus independent region tests and field utility |
| Drone stone/wall detectors | Fine-resolution visible stone and wall geometry | Multi-scale context, terrain and regional screening, explicit uncertainty and cross-site transfer | Same dataset/split comparison and held-out-site test |
| Regional LightGBM / GeoAI | Efficient and interpretable region-specific features | Generalisation, calibrated ranking, hard-negative mining, sensor and region robustness | Reproduction and leave-one-region-out comparison |
| TerraMind/RemoteCLIP/EO foundation models | Strong pretrained representations and label-efficient adaptation | Archaeology-specific evidence fusion, uncertainty, task hierarchy, held-out archaeology, survey economics | Same input and split benchmark; no general EO score substitution |
| Commercial EO/GIS stacks | Potential access to high-resolution data, mature GIS workflow, enterprise integration | Open contracts and receipts, reproducible source manifests, model-independent adapters, community governance | Matched asset/task test; feature and price claims must be source-backed |
| Heritage management platforms | Inventory, record and conservation-management workflow | Connect survey evidence through candidate generation, review and survey planning while respecting existing catalogues | Interoperability and authorised data exchange in a real partner workflow |

There is no honest cross-task global SOTA number. The intended competitive wedge is a complete, measurable workflow:

**source-qualified observation → quality-aware processing → explicit evidence → alternative hypotheses → calibrated candidate ranking → human review → best next survey → independently verified outcome.**

## 5. Recommended first research sequence

1. Reproduce ADAF's published ALS task and its available code using a rights-checked release; record the exact Python/GDAL/runtime environment for the baseline.
2. Reproduce the drone stone/wall model on its published dataset only after confirming the dataset's specific licence and attribution.
3. Reproduce Kweneng LightGBM only if the underlying labels, data, preprocessing and reuse rights are available.
4. Reproduce the ESA tell-mapping setup only when the annotations and source records are available under confirmed conditions.
5. Build site-level splits before tiling. Preserve an entire site or landscape as the test domain.
6. Add a held-out-region transfer study; the reported Herzegovina ADAF transfer and ESA's planned Iran test are leads, not substitutes for an independently governed blind evaluation.
7. Compare at matched reviewer budget, false candidates per area, site-level recall, uncertainty calibration and field validation cost.
8. Only after reproduction and transfer, add the value-of-information survey planner.

## 6. Decision

Start implementation with source registry, rights, exact-byte receipts, STAC preservation, coverage metadata and frozen benchmark manifests. The research stack can consume external Python/model baselines behind reproducible adapters; production policy, provenance, domain contracts and orchestration stay Rust-first.

The first public benchmark should be small, honest and reproducible. The global benchmark grows through independently governed regional site packs. No global completeness, partner access, new discovery, or superiority claim is implied by this audit.

## 7. Additional competitive baselines and audit findings (2026-11-02)

### 7.1 Q2000 — broadscale Maya LiDAR detector

Britton et al., *Evaluating Broadscale Deep Learning for Maya Settlement Detection in G-LiHT Lidar*, Journal of Archaeological Method and Theory, first published 2025-11-19 (Volume 33, article 15, 2026), DOI 10.1007/s10816-025-09741-5.

Q2000 is an important broadscale ALS baseline: YOLOv3 on 1 m LiDAR-derived visualizations, trained from 2,000 high-confidence, visually assessed structures (1,000 each from the G-LiHT South and North2 regions) across about 35,584 km². The paper reports combined F1 0.89. The two macro-regions supplied both training examples and the subsequent external-validation review; test cells were previously unreviewed terrain within those same regions, not a wholly held-out third geography. The authors explicitly identify Chiapas, other Yucatán areas and out-of-Yucatán data as future transfer tests.

Important denominator caveat: the paper uses 100 m x 100 m validation cells, with 400 cells per region and curated 240-positive / 160-negative sampling. Its evaluation counts TP/FP per detection and FN/TN per cell. The resulting F1 is therefore a landscape-cell/detection hybrid and should not be compared directly with conventional object-level F1 or an unweighted population-prevalence estimate. The selected test cells are designed to test the detector, not to yield prevalence-representative confusion totals. This directly motivates HDB-7's explicit inclusion probabilities and design-weighted negative-space audit.

The paper reports that approximately 70% of research time was spent on human visual review. Sol-Atlas should measure review-hours, validated detections per hour, ambiguous-case yield and the cost of checking predicted-negative terrain alongside model metrics.

References: https://doi.org/10.1007/s10816-025-09741-5

### 7.2 Satellite looting detection — localized PlanetScope signatures

Tadesse et al., *Satellite-Based Detection of Looted Archaeological Sites Using Machine Learning*, Proceedings of the IEEE/CVF Winter Conference on Applications of Computer Vision Workshops (WACVW 2026), DOI 10.1109/wacvw68408.2026.00093; arXiv 2602.19608.

The study uses 4.7 m PlanetScope monthly mosaics and a curated set of 1,943 Afghan archaeological sites (898 looted, 1,045 preserved) with multi-year imagery for 2016–2023. An ImageNet-pretrained CNN with spatial masking reports F1 0.926; the strongest compared traditional setup (SatCLIP-V + Random Forest + mean temporal aggregation) reports F1 0.710. The paper suggests looting signatures can be highly localized, and the spatial mask materially affects performance.

The public repository documents static five-fold CNN splits, dynamic stratified splits, seed = 42 + fold index, and recommends year 2023 to align imagery with final-state labels. It also warns through its workflow design that temporal label alignment matters: looting is cumulative, scars may fade, and mixing earlier images with final-state labels can introduce temporal label noise. The README's small sample chips illustrate inputs but are not the full dataset; data files are gitignored. Code is MIT-licensed, but full imagery, labels and model-weight permissions must be checked independently. Before calling a split geographically external, audit the split utilities for site-level disjointness; stratification alone does not establish held-out-site transfer.

This is a strong looting-task baseline, not a general archaeological detector. Sol-Atlas should reproduce it on the exact 2023 label-aligned setting, add temporally ordered event evaluation, and independently test another country or region before making transfer claims.

References:
- Paper: https://doi.org/10.1109/wacvw68408.2026.00093
- arXiv: https://arxiv.org/abs/2602.19608
- Code and split workflow: https://github.com/microsoft/looted_site_detection

### 7.3 S.A.D.A. — accessible remote-sensing analysis workflow

Ciccone et al., *Bridging the gap between contemporary remote sensing and archaeology: The S.A.D.A. platform for accessible remote-sensing image analysis*, Journal of Cultural Heritage 81 (September–October 2026), DOI 10.1016/j.culher.2026.06.026.

S.A.D.A. is a relevant workflow/UI baseline: an open-source GUI unifies remote-sensing indices, PCA, detectors, clustering and Local Indicators of Spatial Association, with guided human-in-the-loop analysis and GIS-ready exports. It also uses guardrails to hide incompatible options. Sol-Atlas should not position itself as merely another GUI. Its differentiator must be traceable evidence, source-specific rights, alternative causal hypotheses, global transfer tests, independent negative audits and decision-support for the next safe survey.

### 7.4 Archaeological CV best-practice guidance

Berganzo-Besga, Orengo and Lumbreras, *Best practices for the application of computer vision-based machine learning in archaeology*, Journal of Archaeological Science 191, article 106577 (July 2026), DOI 10.1016/j.jas.2026.106577. The paper groups best practices across sources, preprocessing, algorithms, training, post-treatment, validation and ethical/communication responsibilities. These are now treated as benchmark gates and required metadata, not optional documentation.

Related framework: Orengo, Berganzo-Besga and Lumbreras, *Theory and practice of artificial intelligence in archaeology*, Journal of Archaeological Science 190, article 106571 (June 2026), DOI 10.1016/j.jas.2026.106571. The authors stress that models learn labelled proxies and sampling choices, not archaeological objects directly, and that transferability and explainability require archaeological accountability.

## 8. Updated competitive interpretation

The field now has at least four strong but task-specific numerical anchors: Q2000 F1 0.89 for a broadscale Maya LiDAR workflow; ADAF recall 0.84 on known sites in an Irish corridor; PlanetScope looting F1 0.926 on an Afghanistan site dataset; and the PNAS Nexus cropmark transfer study's directional DSS across three European sites. These figures are not directly rankable because their targets, sensors, spatial units, splits and evidence tiers differ.

Sol-Atlas should beat the relevant method on matched data and splits, then prove a stronger operational claim: higher independent candidate yield at the same review budget, a lower false-negative rate under a probability-sampled audit, calibrated abstention under geographic shift, and lower cost/time/disturbance to reach a justified decision.

Do not claim general SOTA from a cross-task comparison. The strongest defensible claims will be task-specific and external: held-out site, held-out region, held-out sensor, and independent blind audit, each with proper population sampling and site/landscape uncertainty.
## 9. Further baseline audit: broadscale LiDAR, looting monitoring and workflow platforms

### 9.1 Q2000 broadscale Maya LiDAR

Britton et al., *Evaluating Broadscale Deep Learning for Maya Settlement Detection in G-LiHT Lidar*, Journal of Archaeological Method and Theory, DOI 10.1007/s10816-025-09741-5.

Q2000 uses a YOLOv3 detector trained on 2,000 high-confidence, visually assessed Maya structures (1,000 each from the G-LiHT South and North2 regions) and 1 m LiDAR-derived visualizations across approximately 35,584 km². It reports combined F1 0.89. This is an important broadscale ALS result, but it is not a held-out-region transfer test: the model is trained from both macro-regions and evaluated on previously unreviewed cells within those same regions. The authors identify Chiapas and out-of-Yucatán regions as future tests.

Evaluation caveat: the paper samples 400 100 m x 100 m cells per region, curated to 240 positive and 160 negative cells. TP/FP are counted per detection, while FN/TN are counted per cell. This is a useful hybrid landscape-evaluation protocol, but it is not standard object-level F1 nor a prevalence-representative probability sample. The selected test cells are designed to test the detector, not to yield prevalence-representative confusion totals. This directly motivates HDB-7's inclusion probabilities and design-weighted negative-space audit.

The paper reports that approximately 70% of research time was spent on human visual review. Sol-Atlas should measure review-hours, validated detections per hour, ambiguous-case yield and the cost of checking predicted-negative terrain alongside model metrics.

Reference: https://doi.org/10.1007/s10816-025-09741-5

### 9.2 Afghanistan looting detection from PlanetScope

Tadesse et al., *Satellite-Based Detection of Looted Archaeological Sites Using Machine Learning*, WACVW 2026, DOI 10.1109/wacvw68408.2026.00093; arXiv 2602.19608.

The study uses 4.7 m PlanetScope monthly mosaics and a curated dataset of 1,943 Afghanistan sites (898 looted, 1,045 preserved) across 2016–2023. ImageNet-pretrained CNNs with spatial masking report F1 0.926, compared with 0.710 for the strongest traditional setup (SatCLIP-V + Random Forest + mean temporal aggregation). The results show that localized site-footprint information and image texture can matter more than broad geospatial foundation embeddings for this specific looting task.

The public repository documents fixed five-fold CNN splits, dynamic stratified splits, seed = 42 + fold index, and recommends year 2023 to align with final-state labels. A site-level/landscape-level holdout must be verified from the actual split code before describing any result as geographic transfer; stratification alone does not prove geographic independence. The repository's code is MIT-licensed, but complete imagery, labels and other third-party rights must be cleared separately. Its example chips are not the complete dataset.

References:
- Paper: https://doi.org/10.1109/wacvw68408.2026.00093
- Preprint: https://arxiv.org/abs/2602.19608
- Code and split workflow: https://github.com/microsoft/looted_site_detection

### 9.3 S.A.D.A. and archaeological CV best practices

S.A.D.A. (Ciccone et al., Journal of Cultural Heritage 81, 2026, DOI 10.1016/j.culher.2026.06.026) is a relevant accessible GUI workflow that unifies remote-sensing indices, PCA, detectors, clustering, LISA and GIS-ready exports. Sol-Atlas should interoperate with such tools and distinguish itself through rights-aware provenance, competing explanations, independent negative audits, global transfer and survey decision support—not by duplicating a desktop GIS GUI.

Berganzo-Besga, Orengo and Lumbreras, *Best practices for the application of computer vision-based machine learning in archaeology*, Journal of Archaeological Science 191, 106577 (July 2026), DOI 10.1016/j.jas.2026.106577. The paper groups best practices across sources, preprocessing, algorithms, training, post-treatment, validation, ethics and communication. These should be treated as benchmark gates and required metadata, not optional documentation.

### 9.4 Updated comparison rule

Q2000 F1 0.89, ADAF recall 0.84, PlanetScope looting F1 0.926 and cropmark DSS results are not one leaderboard. Their sensors, target classes, spatial evaluation units, sample construction, geography and evidence tiers differ. Sol-Atlas should reproduce each relevant baseline on matched inputs/splits, then claim improvement only on task-specific held-out sites/regions, calibrated risk/coverage, review-budget utility and independently audited false-negative burden.

## 10. Temporal change-monitoring baselines (2026-11-02)

### 10.1 Microsoft WATCH — month-level event localization

Microsoft WATCH: Wide-Area Archaeological Site Tracking for Change Detection (2026). Repository: https://github.com/microsoft/WATCH (MIT).

WATCH uses Planet Labs monthly mosaics (RGB+NIR, 4.7 m/pixel) from 2017–2024, 1 km² site patches and masks for 1,943 Afghanistan archaeological sites. It compares six geospatial foundation-model embedding backends plus handcrafted spectral/texture features across three pipelines: TED (unsupervised temporal embedding distance, no training), SSCD (self-supervised change detection trained once), and WS (weakly supervised temporal localization using month labels only up to a cutoff, default 2020-12). Each produces a per-site probability distribution over months. Evaluation includes Recall@K, symmetric and directional temporal margins, and additional generalization sites in Syria, Turkey, Pakistan and Egypt.

Important boundaries: the public repository does not ship the PlanetScope imagery or model checkpoints; MIT code licensing does not grant rights to those assets. The README summarizes cross-country generalization but does not provide a full per-country audit table in the repository description. Sol-Atlas must verify exact site-level split disjointness, temporal label cutoffs, image-month completeness and model-weight licenses before claiming a reproduction or transfer score. WATCH is the required HDB-9 primary baseline, not evidence that its methods transfer globally.

### 10.2 Phenology-aware UAV change detection — fine-scale conservation baseline

Liu, Qiu and Yang, *Construction of an intelligent interpretation and change detection system based on UAV remote sensing for landscape architectural heritage conservation*, Scientific Reports 16, 33942 (published 2026-07-25), DOI 10.1038/s41598-026-63132-8.

The paper reports a six-layer conservation workflow with multi-scale element segmentation and temporal Siamese change detection with phenology-aware pseudo-change suppression across three East Asian garden/landscape heritage settings. Its 1,346 manually delineated change polygons were reviewed by two conservators with a third adjudicator (reported Cohen's kappa 0.87). Across four temporal intervals it reports F1 89.31%, false alarm rate 6.42%, miss rate 8.17%, overall accuracy 94.85%; the cross-season subset reports FAR 7.94%. An 18-month deployment reports 184 verified true-positive events, mean lead time 27.3 days, and throughput 1.93 ha/h on a dual RTX 4090 workstation.

Transfer limits: three site types, UAV-scale acquisition and a three-year in-situ phenophase calendar. The paper itself identifies sample scarcity, shallow element ontology, topographic shadowing, residual seasonal phenology lag, and the need for additional LiDAR/thermal fusion. This is a fine-scale conservation comparator, not a regional satellite benchmark.

Reference: https://doi.org/10.1038/s41598-026-63132-8.

### 10.3 Temporal comparison rule

Do not compare WATCH monthly satellite Recall@K with UAV pixel F1 or false-alarm rate as one score. HDB-9 must reproduce WATCH's own event-month ranking protocol on matched imagery and splits, then add interval-censored likelihood, MAP-to-interval distance and pairwise chronology Brier. UAV baselines should be evaluated separately on comparable fine-scale geometry and phenology conditions.
## 11. Evidence-grounded cause attribution baseline and required comparison

The current local implementation separates multi-label cause scoring (`heritage_cause.rs`) from typed evidence auditing (`heritage_cause_evidence.rs`). This is intentionally not positioned as a new cause detector: it is an evidence-integrity and claim-accountability layer.

Taxonomy v0.1 contains 11 competing causes: archaeological excavation/looting, agriculture/cultivation, seasonal vegetation phenology, geomorphic erosion, hydrology/flood, fire/vegetation clearance, modern construction/infrastructure, quarry/mining, military conflict, sensor/processing artifact, and other known cause.

HDB-10 evaluates both predictive quality and evidentiary quality. The evidence bundle records support and contradiction separately, event-specific scope, raw-source digest, method and independence group, modality, evidence tier, rights receipt, reviewer/authority references and detectability assessment for non-detections.

Key adversarial baselines:
- Probability-only multi-label cause classifier with no typed evidence.
- Same-source duplicated evidence bundle (must not improve independent corroboration).
- Context-only evidence bundle (must not support event-specific cause claims).
- Model-derived-only support (must not count as independent ground evidence).
- Missing counter-evidence and non-detection sensitivity (must trigger unsupported/abstain behavior).
- Rights-pending and rights-denied evidence (must remain visible but ineligible).

Report cause Brier, per-class reliability, macro F1 and archaeological false claims per 100 covered samples alongside unsupported assertion rate, support/contradiction retention, independent-source counts, duplicate-source robustness and risk/coverage. Cause probabilities across categories are one-vs-rest and are not required to sum to one because more than one cause can coexist.

Do not call a cause confirmed because the model's probability crosses a threshold. The typed evidence audit checks the integrity of the declared bundle; human archaeologists, custodians and regulators retain causal adjudication and significance authority.
### HDB-10 implementation refinement: cross-polarity independence conflict

The current HCEA audit excludes an eligible source digest or acquisition group from *both* independent support and independent contradiction counts when that same digest/group appears on both polarities for the same cause. The records remain visible and the hypothesis remains contested. HDB-10 therefore measures `cross_polarity_conflict_count` in addition to duplicate-source robustness. A non-detection must be represented as contradictory evidence and include a detectability assessment.

## 12. 2026 updates: seasonal baselines and spectral-geometric transfer

### 12.1 PNAS Nexus directional cropmark transfer

Gravanis and Agapiou (PNAS Nexus, published 10 October 2026) report directional cross-site cropmark-proxy transfer across three European Sentinel-2 landscapes. Their central result is that spectral-geometric similarity is more explanatory than geographic proximity alone, with NDVI phenophase alignment. This is a cropmark vegetation proxy result—not universal archaeology transfer. The paper is CC BY-NC; any commercial reuse of article content, data or derivative materials requires separate permission review. DOI: https://doi.org/10.1093/pnasnexus/pgag336.

The Sol-Atlas implementation mirrors the directional DSS formula but adds stricter positive-distance and shared-month validation, feature/phenophase/digest records, held-out-site/region split qualification, and a multi-site validation evaluator with site leave-out sensitivity. It deliberately emits no pair-IID confidence interval because ordered source-target pairs reuse sites.

### 12.2 Fixed-reference seasonal change monitoring

The new SRD baseline compares evaluation observations against a frozen historical month-of-year reference using median/MAD, distinct-year minimums and explicit abstention. It complements rather than replaces WATCH TED/SSCD/WS. Optional CUSUM captures persistent shifts separately. Missing values and insufficient reference are never interpreted as no change.

### 12.3 Analyst-workflow baseline: S.A.D.A.

The September–October 2026 S.A.D.A. platform is a useful open-source usability comparator: it brings spectral indices, PCA, anomaly detectors, clustering and local spatial indicators into a guided GIS-ready workflow with guardrails, while keeping interpretation with the analyst. The authors explicitly state it is not an automatic site detector. Sol-Atlas should match its reproducible analyst-oriented workflow and add rights-aware provenance, spatial holdout evidence, uncertainty/abstention, evidence-balanced cause review and non-invasive survey planning; it should not claim S.A.D.A. is an automated detector. DOI: https://doi.org/10.1016/j.culher.2026.06.026.

### 12.4 Comparison rule

Compare WATCH monthly event localization, UAV conservation pixel/object change detection, PNAS cropmark-proxy transfer and S.A.D.A. analyst workflow on their own task units. Do not flatten recall@K, pixel F1, cropmark DSS correlation and analyst throughput into one score. Any improvement must use matching assets/labels, held-out geography and review budgets with per-region uncertainty.