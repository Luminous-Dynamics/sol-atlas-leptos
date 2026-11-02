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
