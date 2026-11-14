# EARTH-OBSERVABILITY-DATASET-ADMISSIBILITY-001
## Public GPR data candidates and a rights-aware ingestion plan v0.1

**Parent:** Sol-Atlas #771  
**Date:** 2026-11-14  
**Status:** literature and repository metadata reviewed; no dataset downloaded, parsed or ingested  
**Claim boundary:** Metadata review only. No measured benchmark result, model training or field qualification is claimed.

---

## 1. Highest-priority candidate: UT-Gold-D

**Dataset:** Ground Penetrating Radar dataset with ground-truth data of utility surveying activities  
**Creator:** Ramon ter Huurne  
**Repository:** 4TU.ResearchData  
**DOI:** 10.4121/96303227-5886-41c9-8607-70fdd2cfe7c1  
**Repository catalogue licence:** CC0 1.0 Universal  
**Access:** https://data.4tu.nl/datasets/96303227-5886-41c9-8607-70fdd2cfe7c1  
**University of Twente metadata record:** https://research.utwente.nl/en/datasets/ground-penetrating-radar-dataset-with-ground-truth-data-of-utilit/  
**Dataset paper:** ter Huurne, Olde Scholtenhuis and Dorée (2024), Data in Brief 54, 110329, DOI 10.1016/j.dib.2024.110329.

### Published contents

- 125 utility-surveying activities across 13 construction projects in the Netherlands.
- 959 radargrams.
- Raw radargrams in SEG-Y.
- Processed survey-line map images and trial-trench cross-section images.
- A site/application metadata CSV and a metadata codebook.
- Air-coupled 500 MHz GPR, GNSS RTK positioning and measuring-wheel encoder.
- Ground-truth observations collected through trial trenching.

The University of Twente dataset catalogue identifies CC0 1.0 Universal for the repository dataset. Preserve the exact dataset version, landing-page metadata and file hashes in the eventual source manifest. A dataset-level catalogue licence does not waive the need to inspect third-party material, site confidentiality and any sensitive infrastructure disclosure constraints.

### What it can and cannot establish

This is a strong first external physical-data candidate because it contains field radargrams and trench-based reference material.

It is not automatically a complete utility-network benchmark:

- The published collection is clustered in 13 construction projects, not 125 independent sites.
- The GPR equipment and antenna configuration are not a representative multi-vendor instrument panel.
- Trench truth is reported as images; structured coordinates and complete network topology must not be assumed.
- The paper notes that material/diameter details were not always available, including cases where trenches had already been closed for safety reasons.
- Some targets and survey outcomes may not have complete independent reference geometry.

Therefore, use this corpus first for event-level and selected asset-level retrospective work. Create an explicit truth-admissibility state per target: structured surveyed truth, image-interpreted trench truth, record-only support, conflicting truth, or insufficient truth. Only the first two may enter a physical scoring lane, with image-interpreted claims separately identified.

The 13 project clusters are the relevant starting unit for transfer splits and uncertainty calculations. Never randomly split adjacent radargrams or activities from one physical project across training and blind testing.

## 2. Complementary detector corpus: Moroccan GPR image dataset

**Dataset:** Intelligent recognition of subsurface utilities and voids  
**Mendeley DOI:** 10.17632/ww7fd9t325.1  
**Repository record:** https://data.mendeley.com/datasets/ww7fd9t325/1  
**Reported contents:** 2,239 radargram images, 200 MHz and 400 MHz antennas, urban surveys in Morocco.  
**Repository licence:** CC BY 4.0.

This is useful as a complementary detector/image benchmark. Its public record describes A-scan and B-scan image profiles. Do not treat it as a surveyed 3D asset-network truth set unless the annotations and reference geometry are independently confirmed.

The associated 2025 Data in Brief article is CC BY-NC. Distinguish the article licence from the Mendeley data-record licence and retain both references in the rights ledger.

## 3. Dataset roles

| Dataset | Best initial use | Not sufficient by itself for |
|---|---|---|
| UT-Gold-D | Field event detection, target-level retrospective evaluation, trench-reference comparisons | Multi-vendor transfer or complete network topology |
| Moroccan 2,239-image set | Detector pretraining and image-level benchmark replication | Georeferenced asset reconstruction and independent topology scoring |
| Sol-Atlas analytic fixtures | Contract, metric and negative-control tests | Physical sensor performance |
| Symtropy physics scenes | Controlled forward-model sensitivity and simulation studies | Field qualification without physical comparison |

The 2026 Gao & Hu review discusses the synthetic/field gap and event–utility mismatch. Its reported non-utility detections from cited case studies must not be copied into the Sol-Atlas baseline as a universal false-positive rate.

## 4. Ingestion protocol

1. Resolve the exact 4TU dataset version and freeze the repository metadata, licence record, file inventory and per-file cryptographic digests.
2. Download raw and processed data into a controlled, read-only research cache; preserve originals and all source metadata.
3. Parse SEG-Y headers and trace dimensions without silently normalizing acquisition fields. Record sample interval, trace count, survey direction, coordinate availability and missing metadata.
4. Verify whether GNSS/encoder route positions are trace-aligned, what CRS/datum they use, and how RTK and wheel-encoder uncertainty is represented.
5. Link trench images to survey activities only through documented identifiers. Human annotations must retain annotator, date, method, confidence, image region and review status.
6. Never derive a surveyed utility centerline from a trench image without an independently reviewed spatial-registration method and uncertainty.
7. Partition by entire construction project before any model fitting or tuning. Use leave-one-project-out evaluation for the 13 projects and report the small-cluster limitation.
8. Maintain a separate blind truth store. The model-development process must not see blind trench labels or derived target geometry.
9. Inspect precise location and critical-infrastructure disclosure risk before publishing any derived data. CC0 does not mean that Sol-Atlas should expose operationally sensitive utility coordinates.
10. Emit a data-admission receipt with source digest, licence evidence, file inventory, parser version, transform identities, truth tier, exclusions and split manifest.

## 5. Required ingestion outputs

- Dataset source manifest with version and per-file SHA-256.
- Rights/admissibility receipt with exact repository licence and third-party exclusions.
- SEG-Y parser report and unsupported-header inventory.
- Acquisition/coordinate manifest with CRS, datum, coordinate epoch and transform uncertainty.
- Target-level truth catalogue and trench-image annotation ledger.
- Project-level frozen split manifest.
- Data-quality report with missingness, unsupported attributes and selection bias.
- Explicit NOT_RUN markers for any unverified files, parser paths, transformations or physical assumptions.

## 6. Recommended first experiment

Start with a deliberately narrow experiment on UT-Gold-D:

**Can a detector and interpretation pipeline improve target-level utility association on entirely held-out construction projects, while preserving uncertainty and abstention?**

Baselines:

- manual expert reading under a fixed review-time budget;
- conventional image/signal processing with declared thresholds;
- a documented detector from the 2025 image corpus, provided its licence and label semantics permit the proposed use;
- the Sol-Atlas event-to-candidate and joint-hypothesis layers.

Primary results:

- event POD and false alarms, only where the reference truth supports them;
- asset-instance precision/recall, only where multiple observations map to independently documented targets;
- overmerge/oversplit and unsupported asset claims;
- cross-project calibration and selective risk;
- abstention and unobservable support;
- separate counts for structured, image-derived and incomplete truth.

Do not claim full network reconstruction until the data contain independently supported connected geometry and junction truth. This first pilot can still deliver an important result: an honest physical evaluation of when GPR events can and cannot be associated with utility targets.
