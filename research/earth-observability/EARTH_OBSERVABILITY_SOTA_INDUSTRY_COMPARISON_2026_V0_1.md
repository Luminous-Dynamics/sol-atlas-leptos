# EARTH-OBSERVABILITY-SOTA-COMPARISON-2026
## Current research baselines, industry boundary and matched tournament design v0.1

**Parent:** Sol-Atlas #771  
**Date:** 2026-11-14  
**Status:** Literature and public vendor/standards documentation reviewed; no reproduction or head-to-head result claimed  
**Claim ceiling:** Published results are task-specific baselines. They are not directly comparable unless data, splits, labels, target support, thresholds, hardware and cost boundaries are matched.

---

## 1. Current published research baselines

### 1.1 Event detector baseline — YOLO vs diffusion, November 2026

Senousy et al., *YOLO vs. diffusion networks for underground pipe detection: a case study using ground penetrating radar data*, Scientific Reports, published 4 November 2026.

- Site: Methanex industrial petrochemical complex, Egypt.
- 160 annotated pipe instances.
- GSSI 200 MHz, 300 MHz and 800 MHz systems; profile spacings are reported in the article.
- 118 training samples and 20 independent evaluation samples.
- Five-fold cross-validation groups by parent GPR profile, avoiding splitting one survey profile across train/validation.
- YOLO-family mean cross-validation mAP50 values are approximately 0.77–0.80.
- YOLOv12n reports 0.94 mAP50 on the 20-image independent test set, recall 1.0 and latency 28.9 ms/image.
- DiffusionDet reports 0.508 mean cross-validation mAP50 and 145.34 ms/image.
- The authors disclose small dataset size, only five folds, training-protocol differences between architectures and a single-site study.
- The dataset is under an NDA and is not publicly shareable; code/model configurations/weights are linked from Zenodo DOI 10.5281/zenodo.21538836.

**Sol-Atlas interpretation:** this is a strong current event-box/latency baseline, not an asset reconstruction result. A 20-image test set and single industrial site do not establish transfer to another site, instrument, geology or survey practice. The 0.94 test mAP50 is a useful reproduction target, not proof that a system has mapped utilities correctly.

### 1.2 Lightweight detector baseline — Ghost-YOLOv8n, July 2026

Arvas & Çınar, *Detection of underground objects from GPR data using a lightweight YOLO-based approach*, Scientific Reports, published 5 July 2026.

- Starts from YOLOv8n and replaces selected Conv/C2f stages with GhostConv/C3Ghost, with a single high-resolution detection head for small radargram targets.
- Uses four Roboflow sources (GEO, GPR_data, AAAA Radargram Dataset, Deep Helder Fresh) individually and as a combined multi-source image set.
- Uses 70/15/15 train/validation/test partitions. The combined dataset is described as a diversity/generalization experiment, not a newly established physical benchmark.
- On the integrated dataset, the paper reports mAP@0.5 = 0.756, mAP@0.5:0.95 = 0.363 and 24.3 ms/frame. Relative to its standard YOLOv8n baseline, the reported gains are 5.73% in mAP@0.5 and 5.88% in F1.
- The model is reported faster than the compared RT-DETR configuration (24.3 versus 118.6 ms/frame).
- The authors explicitly note that detailed acquisition metadata—including antenna frequency, burial depth, soil composition, moisture and radar hardware—are not consistently available across the Roboflow source sets. Their future work identifies validation on newly acquired field GPR data as necessary.

**Sol-Atlas interpretation:** a strong candidate for reproducing the lightweight detector/latency frontier and a useful small-object ablation baseline. However, mixed-source image aggregation does not by itself establish a physically controlled held-out-site evaluation when acquisition metadata and source lineage are incomplete. This should be treated as an image-level detector track, not utility-asset topology or physical POD qualification.

### 1.3 Detector architecture comparator — YOLOv12 + denoising AE + CBAM, February 2026

Do et al., *Ground Penetrating Radar Image Analysis for Underground Barrier Detection by Combining YOLOv12 with Channel-wise Attention and Denoising Auto-Encoder*, ACM RACS proceedings, published 4 February 2026, DOI 10.1145/3769002.3769961.

- Combines a denoising autoencoder to suppress clutter while preserving hyperbolic signatures with channel/spatial attention (CBAM) over a YOLOv12 detector.
- Evaluates real GPR datasets of gas pipelines and reports higher precision, recall and mAP@50 than baseline YOLOv12 in its abstract.
- The accessible publication abstract does not disclose the numerical metric values needed for a directly comparable scorecard.
- The official implementation repository describes a company-provided, private B-scan dataset with four pipeline classes (water, sewage, rain and gas), but says the paper experiment focuses on gas pipelines.

**Sol-Atlas interpretation:** this is a valuable architecture/reproduction comparator for denoising + attention + detector ablations. It is not presently an independently reproducible numeric SOTA baseline because the data are private and accessible headline metrics are not available in the abstract. Do not invent or infer scores. The correct experiment is to reproduce the AE/CBAM ablations on a permissioned matched GPR corpus and compare detection, calibration, latency and failure modes—not assume that preprocessing improvement transfers to field utility reconstruction.

Sources: ACM record https://doi.org/10.1145/3769002.3769961; authors' implementation https://github.com/NSLab-CUK/GPR-YOLOv12.

### 1.4 Probabilistic localization baseline — Bayesian forward model, January 2026

Zhang & Cai, *Uncertainty-aware localization and orientation estimation of underground pipelines from GPR data*, Advanced Engineering Informatics, January 2026, DOI 10.1016/j.aei.2025.104093.

The work develops a domain-informed 3D pipe forward model and Bayesian inference for position, depth, orientation and radius. It reports:
- mean travel-time RMSE 0.454 ns against high-fidelity simulations;
- average credible-interval coverage of 97.5% on simulated data;
- average coverage of 77.5% on field data for the reported VI+MCMC normal-likelihood configuration.

**Sol-Atlas interpretation:** the field/simulation coverage gap is a major warning about uncertainty under model mismatch. Sol-Atlas should compare posterior coverage and interval width on independent physical field data, stratified by site and acquisition domain. A 97.5% simulation coverage headline is not a physical qualification result.

### 1.5 Survey and processing optimization baseline — Leica DS2000, April 2026

Marjetič et al., *Multicriteria Statistical Optimization of GPR Survey and Processing for Underground Utility Mapping: Case Study of the Leica DS2000 System*, Remote Sensing 18(7), 1092, 5 April 2026, DOI 10.3390/rs18071092.

The study:
- evaluates 128 acquisition/processing configurations;
- uses three selected utility lines at 35 cm, 55 cm and 85 cm depth;
- records approximately 4.6 km, 1,680 B-scans and 72 C-scans;
- compares 250/700 MHz, two sampling configurations, 3/6/10 cm scan steps, two grid orientations and processing workflows;
- reports mean depth error about 0.9 cm, standard deviation about 3 cm and depth RMSE about 3.5 cm;
- ranks processing workflow, antenna frequency and orientation as important factors; processing and antenna choice were particularly influential;
- notes that results were obtained under dry soil conditions on relatively flat terrain and were focused on depth rather than positional accuracy.

**Sol-Atlas interpretation:** this is a useful operational baseline for survey-parameter optimization. It demonstrates that acquisition and processing should be selected jointly and conditionally, not fixed globally. It is not a broad transfer benchmark: three lines, one DS2000 system and favourable conditions do not characterize wet clay, rough terrain, multiple instruments or network topology.

### 1.6 Field data benchmark — UT-Gold-D, 2023/2024

ter Huurne, Olde Scholtenhuis & Dorée, *Ground penetrating radar at work: A realistic perspective on utility surveying in the Netherlands through a comprehensive ground-truth dataset*, Data in Brief 54, 110329 (2024), DOI 10.1016/j.dib.2024.110329. Dataset DOI 10.4121/96303227-5886-41c9-8607-70fdd2cfe7c1.

- 125 surveying activities across 13 construction projects.
- 959 radargrams.
- Raw SEG-Y, survey-line maps, trench cross-section images and project/application metadata.
- Air-coupled 500 MHz GPR with GNSS RTK and measuring-wheel encoder.
- Trial-trench-derived reference evidence.
- CC0 1.0 Universal is listed in the University of Twente dataset catalogue.

The paper identifies important limitations: several SEG-Y files lack georeferencing because GNSS obstruction led to wheel-encoder-only positioning; rough terrain sometimes prevented consistent wheel contact and compromised data throughput/quality.

**Sol-Atlas interpretation:** UT-Gold-D is a promising retrospective field benchmark, but its 13 project clusters—not 125 activities or 959 radargrams—are the transfer split units. It supports event/selected asset association where trench truth is reviewable. It does not automatically provide complete structured 3D geometry or connected network topology. Positionless radargrams may still enter a signal-level event track, but not spatial localization scoring.

---

## 2. Task-specific SOTA: what we should measure

| Track | Strong current comparator | Sol-Atlas target |
|---|---|---|
| Event detection | YOLOv12n 0.94 mAP50 on 20-image single-site test; Ghost-YOLOv8n 0.756 mAP@0.5 and 24.3 ms/frame on a mixed Roboflow image set | Reproduce both on their own splits where rights permit; then evaluate on physically controlled independent projects/sites and target strata |
| Localization | Bayesian physics-informed GPR inversion | Match or exceed field interval calibration while reducing interval width and controlling depth/position bias |
| Survey optimization | 128-configuration DS2000 study and TOPSIS workflow | Equal-budget, multi-domain action selection using explicit decision loss and EVSI |
| Sensor qualification | POD/PFA and hit/miss methods | Target-conditional curves with independent site-cluster uncertainty and validated operating envelope |
| Asset identity | Not measured by event-box mAP50 | B-cubed/pairwise clustering, asset precision/recall, overmerge/oversplit and false candidate assets |
| Network topology | Not established by event boxes or trench images alone | Node/edge/junction metrics only with independently surveyed connected geometry |
| Decision value | Generally not reported in detector papers | At least 20% lower out-of-sample expected decision loss at the same permitted investigation budget, or equivalent decision quality at 20% lower cost |

### Non-comparability rule

Do not compare YOLO mAP50, localization interval coverage, PAS quality levels, and asset-network B-cubed F1 as if they were one ranking. These are different tasks and evidence claims.

For every tournament, freeze:
- data version and rights scope;
- target and spatial-support definitions;
- truth-admission tier;
- site/project split;
- preprocessing and parameter selection boundary;
- matching tolerances;
- hardware, compute and inference-time boundary;
- investigation-cost and access boundary;
- proper scores, decision loss and failure costs.

A proprietary product without an admissible matched dataset is `NOT_RUN`, not an assumed loss.

---

## 3. Industry and standards: complement, do not replace

| Incumbent / standard | Established capability | Sol-Atlas role |
|---|---|---|
| PAS 128:2022 | Survey Types D/C/B/A and quality levels QL-D, QL-C, QL-B4/B3/B2/B1 and QL-A | Preserve method, support and evidence; map claim-local outputs into the correct vocabulary without self-certifying |
| ASCE/UESI/CI 38-22 | Standard Guideline for Investigating and Documenting Existing Utilities | Support professional investigation planning and traceability; do not replace professional judgment or survey process |
| ASCE/UESI/CI 75-22 | Recording and exchanging utility infrastructure data, including geometry and feature attributes for 3D digital twins | Build interoperable export after owner review |
| OGC MUDDI Part 1 v1.0 | Conceptual core for integrating utility networks, underground space, events and other subsurface features | MUDDI-aligned logical model and traceable evidence links; no conformance claim until abstract tests pass |
| ArcGIS Utility Network | Authoritative owner network topology, rules, validation, dirty areas, tracing and versioning | Supply reviewed candidate claims and evidence; never issue authoritative asset edits automatically |
| Bentley OpenGround | Geotechnical factual-data and AGS/DIGGS exchange management | Exchange qualified borehole/geotechnical evidence |
| Seequent Leapfrog Works | Implicit geological modelling and subsurface visualization | Exchange geometry/interpretation while preserving conflicting hypotheses and uncertainty |

ArcGIS' dirty-area model is a useful operational precedent: edits are not part of the validated network topology until validation is completed; traces through unvalidated dirty areas can yield unexpected results. Sol-Atlas should similarly invalidate dependent claims after changes to source data, transforms, calibration or geometry and require revalidation.

PAS 128's client guide explains that Type B detection includes both EML and GPR at minimum for a full utility mapping survey, with other techniques where appropriate; Type A verification is visual identification at an exposed utility or chamber. Sol-Atlas should therefore avoid treating GPR-only non-detection as a survey-level proof of absence.

---

## 4. What a paradigm shift looks like

### 4.1 Evidence-qualified reconstruction, not a better map

Represent a set of possible utility networks, not a single maximum-score drawing. Keep residual/unknown mass, alternative asset partitions, conflicting owner records, spatial uncertainty and local observability gaps.

### 4.2 Active investigation, not passive processing

Plan the next survey action by expected reduction in engineering decision loss. Entropy reduction is not enough: an observation can be informative but irrelevant to the decision. Compare expert, fixed-grid, random eligible, entropy-greedy and EVSI/expected-net-benefit policies at equal permitted cost.

### 4.3 Negative evidence as a bounded claim

A no-hit is useful only when the sensor covered the target support and the target lies inside a qualified POD envelope. Use both hit and no-hit likelihoods with explicit prevalence. Do not label unobserved or out-of-domain regions as absent.

### 4.4 Segment-local and claim-local quality

A route may be QL-D in one section, QL-B in another and physically verified in a third. A model confidence score cannot overwrite a method/quality classification. Geometry, attributes, topology and ownership have separate evidence requirements.

### 4.5 Physics discrepancy as a first-class model

Treat simulated-to-field mismatch as an estimand. Retain forward-model version, material parameters, solver version, mesh/time-step convergence evidence, antenna response, acquisition configuration and model discrepancy. Do not infer physical performance from synthetic accuracy.

---

## 5. Recommended execution order

1. **Reproduce the 2026 detection baseline** on its exact independent test split if the data are legally and technically available. Preserve profile-level split and publish mAP50, mAP50-95, precision, recall, latency and failure cases. Do not treat the 20-image test as broad transfer.
2. **Use UT-Gold-D for a blind field track** with 13 project-level holdouts and truth tiers. Separately score event detection, event-to-target association and geospatial localization.
3. **Build an uncertainty tournament** using posterior coverage and width, Brier/log score, calibration by domain and selective risk/coverage; include simulation and field results as separate tracks.
4. **Reproduce the DS2000 acquisition/processing tournament** with a larger factorial design and target/domain strata; add cost and information gain, not only depth RMSE.
5. **Build a physics-validation ladder** with analytical references, convergence studies, inter-code comparisons, backend parity, regression and physical field comparisons.
6. **Run equal-budget survey-selection experiments** and measure expected/realized decision loss.

## 6. Stretch targets and claim discipline

- Reproduce the 2026 YOLO baseline before claiming detector superiority; then target robust cross-project transfer, not just mAP50.
- Target nominal 90%/95% field interval coverage with materially narrower intervals than conservative baselines, reported per project/domain. If coverage fails, the system must expand uncertainty or abstain.
- Target lower false-absence risk while maintaining useful coverage.
- Target fewer asset overmerges, oversplits and false junctions than the matched clustering baselines.
- Retain the 20% equal-budget decision-loss/equivalent-cost stretch goal as a preregistered hypothesis.

Every headline must identify whether it is synthetic analytic, physics simulation, retrospective physical, prospective physical or independently accepted engineering evidence. No SOTA claim is earned by comparing different datasets, splits, metrics or claim types.
