# Sol-Atlas Heritage Intelligence: global research and engineering design

**Status:** research/design proposal, v0.1. This document does not claim a live archaeological discovery, partner agreement, data licence, field result, or deployed capability.  
**Workstream:** SOL-ATLAS-HERITAGE-001  
**Repository baseline inspected:** main at 9b946202f21670d20e4e1f191c333e7034d38453 (2026-10-31).  
**Primary product:** Sol-Atlas Heritage Intelligence  
**Scope:** globally reusable architecture with regional, community-governed interpretation profiles.

## 1. Mission and product boundary

Build a non-invasive, evidence-led system for discovering, documenting, reconstructing, and helping protect archaeological and historical landscapes. The system should help researchers move from broad regional screening to reviewable candidates, targeted non-invasive survey, and qualified human assessment.

The unit of research is not only an isolated ruin. It can be a structure, feature, route, field system, water network, extraction landscape, settlement, coastal landscape, or a set of relationships between these things.

The system produces research candidates and evidence packages, not automatic archaeological declarations. An image classifier cannot independently establish age, cultural attribution, significance, ownership, or legal status. Those claims require suitable historical, archaeological, community, field, and legal evidence.

### Non-goals

- No automated excavation, physical disturbance, or instructions to enter land without permission.
- No public release of precise sensitive-site coordinates by default.
- No claim that SAR sees through all vegetation or that any satellite sensor sees through solid earth.
- No universal architecture or period taxonomy imposed on every society.
- No prediction marketed as a confirmed discovery.
- No scraping, training, or redistribution of data where licences, community permissions, or applicable law do not allow it.
- No replacement for archaeologists, heritage authorities, descendant communities, Indigenous custodians, landholders, or local knowledge holders.

## 2. Paradigm shifts

### 2.1 From object classification to evidence-led open-world discovery

A conventional detector asks whether an image patch resembles a known class. Heritage Intelligence should ask:

1. What observable pattern is present?
2. Which sensors independently support or contradict it?
3. Which non-archaeological explanations remain plausible?
4. What can be inferred from the evidence, and what cannot?
5. What additional observation would most efficiently distinguish the hypotheses?

A candidate should retain the raw observations, source assets, transformations, model assessments, alternative hypotheses, reviewer decisions, and unresolved questions. Evidence that contradicts a hypothesis is first-class data, not something to discard.

### 2.2 From a single winning model to a portfolio of specialists

A stone-wall detector, a terrain classifier, a SAR anomaly model, a thermal model, and a historical-map alignment tool solve different problems. The platform should combine their evidence after measuring their reliability in the relevant region, biome, sensor, and spatial scale.

Use:
- supervised instance/semantic segmentation for documented feature families;
- object detection where discrete objects and boundaries are meaningful;
- geomorphometric and tabular models as transparent baselines;
- self-supervised and geospatial foundation-model embeddings for transfer and low-label settings;
- anomaly detection to preserve unusual patterns that do not fit known labels;
- temporal change detection for destruction, disturbance, exposure, vegetation change, and landscape evolution;
- explicit human review and field feedback for adjudication.

No model family is presumed superior before a split-matched benchmark.

### 2.3 From global uniformity to shared geometry plus local interpretation

Create a small global physical-observation vocabulary (for example, linear stone alignment, enclosure-like geometry, terrace edge, mound, canal-like feature, burn/thermal anomaly). Keep cultural attribution, local typologies, language, chronology, naming, significance, and interpretive relationships extensible and versioned.

A shared schema should make data exchange possible; it must not force different societies into a single architectural or chronological taxonomy. Preserve original terminology and its source alongside reviewed mappings to shared concepts.

### 2.4 From passive acquisition to active survey design

The platform should estimate which next survey will produce the most useful information per unit of field effort and cost. It can compare:
- another optical observation in a different season;
- Sentinel-1 ascending versus descending observations;
- higher-resolution commercial or airborne imagery where lawfully available;
- drone photogrammetry or LiDAR;
- ground-based magnetometry, resistivity, GPR, or field walking when authorised and led by qualified researchers.

This is a value-of-information problem. The objective is not to maximise model confidence; it is to reduce decision-relevant uncertainty while minimising cost, risk, and disturbance.

### 2.5 From static ruins to historical landscape systems

Once evidence is adequate, Symtropy can model explicit, bounded questions: catchments, slope and least-cost movement, viewsheds, water access, agricultural suitability under stated paleoclimate assumptions, settlement connectivity, and infrastructure change through time.

Every modelled reconstruction must separate:
- observed evidence;
- interpolated geometry;
- inferred relationships;
- simulation assumptions;
- alternative scenarios.

A plausible-looking digital reconstruction is not proof that the reconstructed feature existed.

### 2.6 From open data alone to FAIR plus CARE stewardship

Use FAIR data practices for findability and interoperability, but pair them with CARE-oriented governance: Collective Benefit, Authority to Control, Responsibility, and Ethics. Public access is not automatically appropriate for all cultural knowledge or site coordinates. Communities and legitimate custodians must have meaningful authority over appropriate access, reuse, attribution, and benefit.

## 3. Global sensing strategy

The global pipeline must be useful where high-resolution LiDAR is absent. Use low-cost, broad-coverage sensors to prioritise landscapes; reserve expensive, high-resolution surveys for areas where they can resolve meaningful uncertainty.

| Modality | Useful observations | Proper role and limitations |
|---|---|---|
| Airborne or drone LiDAR | Ground/surface geometry, earthworks, walls, terraces, landscape microtopography | High-value local evidence. Coverage, point density, ground-return quality, licence, and processing quality vary. It does not directly image buried artefacts. |
| Sentinel-1 SAR | Backscatter, coherence/time-series response, geometry-dependent scattering | Broad screening and complementary observations. Not a universal vegetation-penetrating or subsurface instrument. Ascending/descending passes can add different viewing geometries. |
| Sentinel-2 and Landsat | Multispectral soil/vegetation/water patterns and temporal context | Global-scale triage; pixel size and vegetation, cloud, season, land use, and mixed pixels limit feature-level detection. |
| High-resolution optical/aerial | Building outlines, soil marks, crop marks, surface stone patterns | Powerful where coverage and lawful reuse exist. Availability and acquisition dates are uneven. |
| Hyperspectral | Material/surface spectral differences | Specialist targeted acquisition; signals need local ground truth and environmental controls. |
| Thermal | Surface temperature and thermal-inertia contrasts | Context-dependent; weather, moisture, time of day, materials, and vegetation can dominate. |
| DEM and derived terrain | Slope, curvature, local relief, openness, drainage, viewshed | Useful context and geomorphometric baseline. A coarse DEM cannot resolve small walls. |
| GEDI / ICESat-2 | Sampled/footprint-scale elevation and vegetation structure information | Regional context and selected observations; sampling is not a substitute for dense airborne LiDAR. |
| Historical aerial/maps | Earlier landscape state, vanished tracks, old field boundaries, urban growth | Critical temporal evidence, but georeferencing, scale, datum, map intent, and positional uncertainty must be recorded. |
| Ground survey | Direct observations, condition, materials, archaeological interpretation | Essential validation. Must be properly authorised, documented, safe, and carried out by qualified teams. |
| GPR, magnetometry, resistivity | Subsurface contrasts under suitable conditions | Specialist non-invasive follow-up, not universal detectors. Soil, moisture, geology, target, survey design, and operator expertise strongly affect results. |
| Photogrammetry | Visible-surface 3D geometry and orthomosaics | Detailed local recording; cannot reveal features hidden below ground or behind opaque cover. |

### Multi-modal evidence is conditional, not additive by assumption

A fusion model must know whether a modality was unavailable, cloud-obscured, low quality, misregistered, or truly observed with no signal. Missing data must never be represented as a negative observation.

Before fusion, record:
- source acquisition time and time span;
- sensor/platform, band/polarisation/orbit direction where relevant;
- CRS, vertical datum, georeferencing method and accuracy;
- ground sampling distance or point density and effective resolution;
- cloud, shadow, vegetation, moisture, incidence angle, scan angle, and other known quality flags;
- nodata and coverage masks;
- transformation history and alignment error.

The SAR jungle study using Sentinel-1 ascending and descending observations is a useful targeted baseline, not proof of a general ability to see through canopy. The paper itself frames the method as candidate identification in a particular Maya-ruin setting.

## 4. Data sources and global coverage model

Adopt a federated source registry. A source is not approved merely because it is technically downloadable.

| Source family | Example entry points | Intended use and access caveat |
|---|---|---|
| Global optical | Copernicus Sentinel-2, Landsat | Broad regional screening and multi-temporal context. Observe product versions, cloud masks, resolution, and licence terms. |
| Global SAR | Copernicus Sentinel-1 | Backscatter/time-series and orbit geometry. Preserve product level, orbit direction, polarisation, incidence angle and preprocessing lineage. |
| Global elevation | Copernicus DEM, SRTM/NASADEM, ALOS products | Regional terrain context. Record native resolution, vertical reference, acquisition/processing lineage, and known artefacts. |
| Spaceborne laser | NASA GEDI, ICESat-2 | Sparse sampled structure/elevation context, not dense wall-level mapping. |
| Airborne and community LiDAR | OpenTopography, national mapping agencies, research repositories | High-resolution validation and local screening. Coverage, registration, download limits, licences and access tiers differ. |
| Cultural 3D data | Open Heritage 3D | Point clouds, aerial/terrestrial LiDAR, photogrammetry and related metadata. Data access and reuse must be checked per collection. |
| Archaeological data catalogues | ARIADNE, national archaeology data services, institutional repositories | Existing site, excavation, survey and documentation records. Searchability does not grant rights to bulk export or train on all records. |
| Threat / regional heritage records | EAMENA and analogous national or regional systems | Historical and satellite-derived site documentation. Sensitive coordinates and condition records can have tiered access. |
| Local records | Museums, archives, cadastral/land records where lawful, heritage authorities, community archives | Essential regional context. Never infer permission from public visibility. |
| Field data | Archaeologist-approved survey records, community-contributed observations | Ground truth, correction and local interpretation subject to consent, attribution, safety, and governance. |

### Coverage strategy

Create a coverage atlas, not a list of supposedly complete countries. For every region and modality, store:
- actual footprint and time range;
- resolution and quality;
- licence and permitted uses;
- access path and rate constraints;
- whether labels exist and their evidence tier;
- known gaps and survey-effort bias.

A missing archaeological record is not evidence that a place has no archaeology. The model must distinguish surveyed-and-not-observed, not-surveyed, inaccessible, obscured, and unknown.

The platform should work across continents while allowing a local research partner to contribute a site pack without uploading sensitive raw data to a central service. Where lawful and technically feasible, train/evaluate with federated or permission-bounded data custody.

## 5. Data and interoperability contracts

Prefer standards and widely used formats over bespoke storage.

| Data class | Preferred exchange/storage | Purpose |
|---|---|---|
| Raster assets | GeoTIFF / Cloud Optimized GeoTIFF (COG) | Georeferenced imagery and terrain layers with efficient windowed access. |
| LiDAR point clouds | LAS/LAZ; COPC where supported by the source and processing chain | Preserve point-level observations and enable spatially selective access. Never silently discard classifications or provenance. |
| Vector features | GeoParquet for analytical exchange; GeoPackage for portable field/offline workflows | Candidate geometries, annotations, survey tracks and derived layers. |
| Asset catalogue | STAC Item/Catalog/API | Search and link imagery, LiDAR, DEM, vectors, labels and model outputs using space/time/asset metadata. |
| Candidate/evidence records | Versioned JSON schema and typed Rust structures | Stable application contract with explicit lifecycle and access policy. |
| Cultural heritage semantics | CIDOC CRM mappings and local vocabularies | Link events, places, actors, objects, observations and interpretations without flattening their distinctions. |
| Provenance | W3C PROV-O-aligned provenance graph plus content hashes | Preserve which source and transformation produced each claim and output. |
| Geospatial services | OGC API family where appropriate; STAC for discovery | Interoperable service boundaries. |

Do not force every dataset into one projected CRS. Keep source CRS and vertical datum; select an analysis CRS appropriate to the local task; record all transformations and their error. Split global indexing from local metric calculations. Handle antimeridian, polar regions, dateline-crossing geometry, vertical datums, and large-area geodesic calculations explicitly.

## 6. Proposed architecture

    GLOBAL / REGIONAL SOURCE REGISTRY
    STAC | national portals | OpenTopography | archives | approved field data
                                  |
                                  v
                         RIGHTS + COVERAGE GATE
          licence | consent | access tier | jurisdiction | freshness
                                  |
                                  v
                        IMMUTABLE SOURCE ASSETS
              content hash | source ID | time | CRS | quality
                                  |
                                  v
                         PREPROCESSING GRAPH
       GDAL / PDAL / specialist GIS tools behind versioned adapters
       terrain derivatives | SAR | optical | point clouds | alignment
                                  |
                                  v
                         FEATURE EVIDENCE STORE
            observations | masks | uncertainty | negative evidence
                                  |
                                  v
                      SPECIALIST MODEL ENSEMBLE
        LightGBM | segmentation | object detection | anomaly detection
           Prithvi / TerraMind / other geospatial encoders as baselines
                                  |
                                  v
                       EVIDENCE FUSION / RANKING
        calibrated scores | alternatives | open-set uncertainty | gaps
                                  |
                                  v
                        HUMAN REVIEW WORKBENCH
         archaeologist | local expert | community / custodial steward
                                  |
                       authorised feedback
                                  v
                    SURVEY VALUE-OF-INFORMATION
             recommend next non-invasive observation or field visit
                                  |
                                  v
                    RESEARCH / RECONSTRUCTION OUTPUTS
           Sol-Atlas | Symthaea | Symtropy | Mycelix provenance/governance

### Component responsibilities

**Sol-Atlas**
- Own the spatial catalogue, candidate map, time layers, evidence overlays, and survey planning interface.
- Display uncertainty, provenance, sensor quality and access state as first-class layers.
- Keep sensitive geometry off public rendering paths. Public generalisation must happen server-side or in a controlled publication pipeline, not solely in the browser.

**Symthaea**
- Generate candidate explanations from evidence bundles; identify contradictions and alternative hypotheses; help choose follow-up observations.
- Never silently turn a language-model narrative into an evidence record. Every factual proposition must reference its source observation or be marked as a hypothesis.
- Learn regional feature representations only with governed, properly licensed data and independent holdouts.

**Symtropy**
- Reconstruct terrain/settlement scenarios and test bounded questions such as catchments, route accessibility, visibility, water access, and landscape change.
- Label simulated or inferred outputs separately from measured geometry. Provide parameter and assumption sensitivity.

**Mycelix**
- Preserve contributor and source provenance, rights declarations, review lineage, access decisions, attribution, and governed correction history.
- A ledger or reputation mechanism is not proof that an archaeological claim is true. Authority and evidentiary weight remain domain- and context-specific.

**Human research partners**
- Decide archaeological interpretation, field methods, permission requirements, significance, and publication readiness.
- Descendant/Indigenous communities and relevant custodians participate as knowledge holders and decision-makers, not merely as data providers.

### Rust-first implementation boundary

Keep the existing Leptos/WebGL application as the user-facing viewer. Do not make the browser responsible for bulk point-cloud processing or model training.

Start with a pure-Rust domain crate/module for:
- source-asset and candidate records;
- rights and access-policy evaluation;
- candidate lifecycle transitions;
- evidence graph references and integrity checks;
- geospatial metadata/CRS declarations;
- benchmark manifests, split manifests, and score aggregation;
- deterministic job configuration and provenance receipts.

Use mature geospatial engines through pinned, isolated adapters rather than reimplementing complex raster and point-cloud algorithms prematurely. GDAL and PDAL are appropriate initial processing backends; keep commands, versions, container/image digests, stdout/stderr, input hashes, output hashes and exit codes in every run receipt.

For model research, compare open implementations on the same inputs and split manifests. Do not force every foundation model into Rust training immediately. For deployment, export supported models to a stable portable inference format where feasible and test the actual production runtime against the research reference. Avoid a permanent Python production service; a bounded research-only adapter may be used to reproduce external baselines until a verified export exists.

Initial module boundary proposal:
- heritage::asset
- heritage::rights
- heritage::candidate
- heritage::evidence
- heritage::hypothesis
- heritage::review
- heritage::provenance
- heritage::benchmark
- heritage::survey_planning

Do not add a web map dependency or heavyweight GIS crate to sol-atlas-core until the typed contracts and test needs are clear.

## 7. Global benchmark and model strategy

The detailed protocol is in heritage-discovery-benchmark-v0.1.md. The core rule is that model performance must be measured on independent sites and regions, not merely adjacent image tiles.

### Baselines to reproduce

1. Expert archaeological interpretation of the same evidence, with reviewer time recorded.
2. Terrain derivatives plus Random Forest / LightGBM / XGBoost.
3. A published stone/wall instance segmentation baseline such as Mask R-CNN, where the original data, labels, weights, and licence permit reproduction.
4. Sentinel-1 ascending/descending SAR baseline for the relevant vegetation/structure context.
5. Strong general EO foundation encoders (for example Prithvi-EO-2.0, TerraMind, and Clay) using frozen embeddings/linear probes and task-specific fine-tuning where licences and model inputs permit.
6. Modality ablations and simple fusion (late calibrated fusion) before complex multimodal fusion.

An Earth-observation foundation model is a pretraining baseline, not an archaeology benchmark result. Its success on land cover, flood, or other EO tasks does not demonstrate archaeological capability.

### Why an ensemble can beat a single detector

A detector may find a line, but not tell whether it is an ancient wall, a modern fence, a geological joint, a road cut, or a terrace. A terrain model may explain its topographic context. SAR may add independent geometry-dependent evidence. Historical maps may show persistence. A local expert may know the land-use history. The candidate record preserves each contribution and the remaining uncertainty.

The model should rank evidence for review, not convert weak signals into a high-confidence archaeological label through repeated correlated inputs.

## 8. Heritage safeguarding and data governance

Access policy is a domain requirement, not an optional UI setting.

- Default exact candidate geometry to restricted or custodial access until a documented review permits broader release.
- Separate exact geometry from a public generalised geometry; omit coordinates entirely where even a generalised view creates risk.
- Keep access-control checks at API/storage boundaries and in export jobs, not only in frontend rendering.
- Record the authority, scope, expiry/review date, purpose, and legal/consent basis for access.
- Preserve source-specific licence and downstream derivative obligations.
- Provide correction, takedown, attribution, and community review routes.
- Do not publish vulnerable site coordinates, looting indicators, burial locations, sacred places, or restricted knowledge merely because a model found them.
- Do not use oral histories or Indigenous knowledge for training or inference beyond the agreed purpose.
- Make data deletion, retention, replication, and irreversible public release limitations explicit.
- Require land access, permits, safety plans and qualified supervision before field activity.

EAMENA's tiered-access model is a useful real-world design reference: its public access intentionally withholds precise coordinates and some condition information because precise site disclosure can increase looting risk.

## 9. Staged implementation and acceptance gates

### H001 — Research contract and data governance
Deliver:
- this architecture and the benchmark protocol;
- candidate JSON Schema and synthetic fixture;
- source registry template and access policy;
- explicit non-goals and lifecycle states.

Gate: schema validates; synthetic data only; unknown rights/access fail closed; no public exact-coordinate path.

### H002 — Reproducible geospatial ingestion
Deliver:
- STAC discovery and asset registration;
- COG and LAS/LAZ/COPC metadata ingestion;
- CRS, vertical datum, resolution, coverage, licence and hash checks;
- deterministic GDAL/PDAL job receipts.

Gate: fixtures for bad CRS, missing datum, checksum mismatch, nodata, invalid geometry, duplicate assets, stale metadata, and denied rights all fail or quarantine safely.

### H003 — Terrain and SAR baseline
Deliver:
- DTM/DSM inputs and terrain derivatives;
- Sentinel-1 acquisition metadata and orbit-aware preprocessing;
- Sentinel-2/Landsat temporal context;
- small, synthetic or permissively licensed examples.

Gate: no unrecorded reprojection/resampling; masks and missingness preserved; repeat runs produce identical logical outputs or documented nondeterminism.

### H004 — First feature family
Start with one feature family supported by a usable, rights-cleared labelled dataset. Stone walls/settlement enclosures are a candidate, not an assumed data entitlement. Reproduce published baselines before claiming improvement.

Gate: independent site split, hard negatives, calibrated scores, reviewable polygons, and a full provenance receipt.

### H005 — Global generalisation
Add independently governed regional packs. Use leave-one-region-out, leave-one-biome-out, leave-one-sensor-out, and temporal holdouts. Add low-label adaptation and open-set/anomaly tasks.

Gate: disclose where the model fails; do not average away weak regions.

### H006 — Human review and active survey design
Deliver an evidence review queue, correction feedback, uncertainty decomposition, and non-invasive next-survey ranking.

Gate: survey recommendation states what uncertainty it addresses, the expected information gain, data needed, cost assumptions, authorisation requirements, and potential impact.

### H007 — Landscape reconstruction and monitoring
Add explicit scenario models, change detection, and preservation monitoring only after local validation and governance.

Gate: observed, inferred, simulated and speculative layers remain distinguishable in every export.

## 10. Success criteria and honest claims

A credible global system should demonstrate:
- better candidate yield at a fixed archaeologist review budget than the strongest task-matched baseline;
- meaningful improvement on held-out regions and biomes, not only within training landscapes;
- lower survey cost per independently validated candidate;
- calibrated uncertainty and effective abstention under unfamiliar conditions;
- robust performance with missing or degraded modalities;
- complete provenance for every displayed candidate;
- zero unauthorised exposure of restricted coordinates in release tests;
- community- and custodian-approved use and publication conditions.

The project may claim “model-ranked candidate” after reproducible inference. It may claim “field-observed feature” after the appropriate survey. It may claim “archaeologically validated” only after qualified review with stated evidence and authority. It must not claim “newly discovered ancient city” on the strength of a model score.

## 11. Key literature and standards

- Siteleki, M. J. (2026), “Automating Archaeological Discovery: Assessing Geospatial Artificial Intelligence (GeoAI) Tools for Stone Wall Identification in Kweneng, South Africa,” Archaeometry. https://doi.org/10.1111/arcm.70143
- Uziel, E., Zohar, M. and Jaffe, Y. (2026), “Semi-automatic detection of building stones and wall segments of archaeological ruins,” Journal of Archaeological Science 185, 106430. https://doi.org/10.1016/j.jas.2025.106430
- Michenot et al. (2024), “Identification of Maya ruins covered by jungle using Sentinel-1,” Scientific Reports. https://doi.org/10.1038/s41598-024-53068-2
- Prithvi-EO-2.0 (2025), “A Versatile Multitemporal Foundation Model for Earth Observation Applications.” https://doi.org/10.1109/TGRS.2025.3642610
- TerraMind (ICCV 2025), “Large-Scale Generative Multimodality for Earth Observation.” https://doi.org/10.1109/ICCV51701.2025.00693
- OGC STAC Community Standard 1.1.0: https://www.ogc.org/standards/stac/
- OGC Cloud Optimized GeoTIFF: https://www.ogc.org/standards/ogc-cloud-optimized-geotiff/
- CIDOC CRM / ISO 21127:2023: https://www.iso.org/standard/85100.html
- Global Indigenous Data Alliance, CARE Principles: https://www.gida-global.org/careprinciples
- ARIADNE Portal: https://www.ariadne-research-infrastructure.eu/portal/
- Open Heritage 3D: https://openheritage3d.org/data
- OpenTopography: https://portal.opentopography.org/datasets
- EAMENA database and access policy: https://eamena.web.ox.ac.uk/database and https://eamena.org/open-access-policy

The programme begins as a research proposal and synthetic contract. No partner, dataset entitlement, field survey, benchmark score, or discovery is implied by this document.
