# RFC: Evidence-Driven Mineral Prospectivity for Sol-Atlas

- **Status:** Proposed research specification; not yet implemented
- **Date:** 2026-10-12
- **Initial region:** South Africa, with the Bushveld Complex PGE–Ni–Cu–Cr system as the primary benchmark
- **Companion boundaries:** Symthaea (hypothesis analysis), Mycelix-DeSci (provenance and attestations), Symtropy (only if a validated physical-simulation requirement exists)
- **Decision rule:** No prospectivity claim is promoted beyond the evidence stage it has actually passed.

## 1. Purpose

Build a reproducible mineral-prospectivity workflow that ranks places for further investigation by integrating geological, geophysical, geochemical, remote-sensing, and (where suitable) tectonic-history evidence.

The initial objective is **not** to announce a discovery or estimate a commercial reserve. It is to establish a repeatable baseline, test generalization to geographically separate areas, expose missing data and uncertainty, and measure whether the workflow can reduce the area requiring follow-up while retaining known reference targets.

### Research question

> Given the data available at a frozen cutoff date, does a provenance-aware, uncertainty-reporting prospectivity workflow improve the spatial efficiency of exploration targeting over transparent baselines on geographically independent validation regions?

### Non-goals for the first increment

- No automatic declaration that a mineral deposit exists.
- No reserve, grade, tonnage, depth, mineability, or economic-value claims from prospectivity scores alone.
- No claim that satellites can directly image deep ore bodies.
- No continent-scale deep-time simulation before a relevant model and input data have been validated.
- No automatic land access, sampling, drilling, permitting, or extraction decisions.
- No redistribution of source datasets until their licenses and relevant access terms have been checked.
- No changes to Sol-Atlas's existing energy-site layer or live behavior in this documentation-only RFC.

## 2. Research basis and starting baseline

The starting scientific reference is:

Nwaila, G. T. et al. (2024), “Mineral Reconnaissance Through Scientific Consensus: First National Prospectivity Maps for PGE–Ni–Cu–Cr and Witwatersrand-type Au Deposits in South Africa,” *Natural Resources Research* 33, 2357–2384. DOI: [10.1007/s11053-024-10390-w](https://doi.org/10.1007/s11053-024-10390-w).

The paper reports a national evidence compilation with 38 covariates, a common spatial index, and a workflow ensemble that varied feature-space dimensionality, algorithms, selection metrics, and sampled negative labels. It reports potential target areas as hypotheses, not confirmed new deposits. Its supplementary material includes compiled data and model results; inspect the actual supplement, its metadata, and the reuse conditions before depending on or redistributing any individual asset.

A critical limitation documented by the paper is that comprehensive drilling data are sparse. Unlabeled cells were therefore used as statistical proxies for negative labels, with the variation in sampled negatives propagated through the ensemble. This is a useful starting baseline, but it is not evidence that every unlabeled cell is barren ground.

### First target family

Start with the South African PGE–Ni–Cu–Cr system associated with the Bushveld Complex. Treat Witwatersrand-type Au as a second, separately configured benchmark only after the first target-family pipeline is reproducible. Do not mix deposit types under one undifferentiated label.

### Baseline reproduction gate

Before claiming an improvement:

1. Inventory the paper's supplementary data, model products, input feature definitions, grid/indexing assumptions, and provenance.
2. Identify any unavailable, ambiguous, or non-redistributable inputs.
3. Reproduce as much of the published result as licensing and artifact availability permit.
4. Compare the reproduced map and metrics to the paper, and document every intentional deviation.
5. If exact reproduction is not possible, call it a **partial reproduction** and name the missing inputs; do not describe a merely similar map as a reproduction.

## Companion artifacts

- [Initial data inventory](MINERAL_PROSPECTIVITY_DATA_INVENTORY.md) records what source access has been verified, what remains unchecked, and the next M0 actions.
- [Dataset manifest JSON Schema](schemas/mineral-dataset-manifest.schema.json) defines the provenance/rights/quality contract for a source or derived layer.

These are desk-research artifacts only. They do not certify individual datasets as licensed, downloaded, validated, or ready for ingestion.

## 3. Data-source plan

Data access, license, version, coverage, and reuse conditions are separate properties. A dataset being viewable through a map portal does not imply it can be freely redistributed or bulk-downloaded.

| Layer | Initial source / entry point | Initial use | Guardrails |
|---|---|---|---|
| Geological map units, ages, structures | [South African Council for Geoscience portal](https://maps.geoscience.org.za/) | Lithology, stratigraphy, age, faults/contacts | Record product IDs, scales, version/date, attribution, service/download terms, and missing coverage |
| Published benchmark layers and labels | [2024 South African MPM paper and supplement](https://link.springer.com/article/10.1007/s11053-024-10390-w) | Reproduction benchmark | Audit supplement assets and conditions individually; do not silently substitute a newer dataset |
| Geophysical measurements / derived grids | CGS where authorized; [USGS Mineral Resources data](https://www.usgs.gov/programs/mineral-resources-program/data) and [Geology, Geophysics, and Geochemistry Science Center data](https://www.usgs.gov/centers/gggsc/data) for methods and relevant international comparisons | Magnetic, gravity, radiometric, seismic, or electromagnetic evidence where applicable | Distinguish measured data from derived transforms; preserve survey age, spacing, height, correction method, effective depth/resolution, and artifacts |
| Geochemistry and petrology | CGS resources; [EarthChem Portal](https://portal.earthchem.org/) where applicable | Element associations, lithochemistry, isotopic/age constraints | Preserve units, detection limits, analytical method, sample type, coordinates/uncertainty, and censoring |
| Multispectral time series | [Copernicus Data Space](https://dataspace.copernicus.eu/) and its [STAC catalog documentation](https://documentation.dataspace.copernicus.eu/APIs/STAC.html) | Surface alteration/mineral-proxy features, land-cover context, change/mask checks | Query catalog by AOI/time; record item IDs and asset checksums; mask clouds/shadows; evaluate seasonal effects |
| Imaging spectroscopy | [NASA EMIT mission/data resources](https://earth.jpl.nasa.gov/emit/) | Targeted surface mineral maps where footprint/coverage is suitable | Not a uniform global layer; reflect surface materials and do not imply direct subsurface detection |
| Tectonic reconstruction | [GPlates / PyGPlates documentation](https://www.gplates.org/docs/pygplates/) | Later-stage time-dependent tectonic context | Require valid reconstruction models, plate assignments and uncertainty; preserve model/time version; do not extrapolate unsupported locations |
| Deposit / occurrence reference data | CGS and documented scholarly compilations; USGS sources for international comparisons | Labeling and independent reference targets | Separate confirmed deposits, occurrences/prospects, drill intercepts, mines and resource statements; retain status/date/source and deduplicate with an auditable rule |

### Satellite-feature policy

Satellite features are an **incremental evidence layer**, not a mandatory prerequisite for the first reproduction. Sentinel-2's public multispectral coverage and NASA EMIT's imaging spectroscopy differ substantially in bands, spatial sampling, footprint, and acquisition conditions. Surface alteration indices are proxies, not mineral deposit labels.

Evaluate satellite features by an ablation experiment: baseline without the new feature layer, then the same evaluation with it. Admit a feature only if its incremental value survives spatially independent testing and is not explained by coverage artifacts, cloud/vegetation masks, or label leakage.

### Dataset manifest

Each imported layer must have a machine-readable manifest including:

- Stable layer ID, display name, source organization, canonical URL/product ID, citation, accession timestamp, and source publication/acquisition date.
- License/reuse state: **verified**, **restricted**, **unknown**, or **not permitted**; unknown is not treated as permission.
- File/media type, checksum algorithm and digest, byte size, coordinate reference system, units, spatial/temporal extent, nominal resolution/spacing, and no-data convention.
- Transformation history, software/version, parameter digest, upstream layer IDs, and resulting artifact digest.
- Coverage/missingness and known limitations, including survey boundaries, interpolation, temporal mismatch, detection thresholds, and processing artifacts.

Never place secrets, restricted raw data, private landholder records, or sensitive access coordinates in a public repository. Public manifests may describe withheld artifacts by digest and permitted metadata without publishing their contents.

## 4. Spatial index and geospatial correctness

Use an equal-area projected CRS for distance/area calculations within the South African pilot and retain original coordinates/CRSs in the provenance chain. A hierarchical grid such as H3 can be used as an indexing/join layer and for multi-resolution aggregation, but grid choice must not erase source resolution or create false precision.

For every feature, retain at least:

- Original source resolution/scale and effective resolution.
- Transformation/resampling method and output grid resolution.
- Valid-data mask and fraction of valid contributing source data.
- Distance/proximity calculation definition and units.
- Whether the value is observed, interpolated, reconstructed, or model-derived.

Do not infer fine-grained prospectivity from coarse gravity/seismic products by simply resampling them to a finer grid. Derived magnetic filters and lineament transformations may accentuate noise and survey seams; store the raw/source grid identity and parameters for each derivative.

## 5. Model design

Implement the simplest reproducible baseline first. Retain target-type configuration and evaluation code so separate mineral systems cannot accidentally share incompatible labels or priors.

### Model ladder

1. **Transparent baseline:** evidence-weighted overlay or logistic regression with declared feature handling and weighting.
2. **Tabular baselines:** at least one tree ensemble and one simpler statistical comparator, evaluated on identical folds and feature sets.
3. **Workflow ensemble:** vary reasonable algorithms, feature subsets/dimensionalities, negative/unlabeled sampling, and hyperparameters. Report both consensus and disagreement.
4. **Geological-prior experiment:** add deposit-system-specific relationships only when each relation is documented by a source or an explicit expert hypothesis that can be challenged.
5. **Novelty queue:** separately identify areas outside the training distribution or with strong model disagreement. Novelty is a request for investigation, not positive prospectivity evidence.

Symthaea can assist with hypothesis generation, contradiction search, and prioritization of follow-up tests. It cannot supply missing geological evidence by narrative or confidence alone. Keep model inference and external scientific validation as distinct roles.

### Uncertainty must remain factored

Do not compress all uncertainty into one opaque confidence value. At minimum report:

- Model/workflow disagreement.
- Label and negative-sampling uncertainty.
- Input-data coverage/quality and temporal mismatch.
- Extrapolation / distance from training distribution.
- Validation performance for the relevant target and spatial domain.
- Geological plausibility and unresolved alternative explanations.

An ensemble vote or workflow consensus is not a calibrated probability that an economic deposit exists.

## 6. Leakage-resistant validation

Random splits can overstate performance when deposits and neighboring grid cells share geology or when label footprints leak across folds. The primary generalization estimate must come from spatially separated groups.

### Required evaluation

- **Spatial holdout:** reserve whole districts/terranes or well-separated geological groups for testing. Document buffers and the reasoning used to avoid overlapping deposit footprints.
- **Grouped out-of-fold estimates:** keep samples associated with one deposit and its spatial neighborhood in the same fold.
- **Temporal backtest (where possible):** freeze source/feature availability to a stated historical cutoff and test on independently documented later outcomes. Do not use post-discovery mapping or labels as predictors.
- **Unlabeled treatment:** label cells only as “confirmed negative” when supported by sufficiently informative evidence (for example, a relevant adequately sampled/drilled interval under a documented criterion). Otherwise keep them unlabeled or mark them as weak negative proxies in explicitly named sensitivity runs.
- **Repeated negative samples:** run multiple negative/unlabeled selection schemes and publish their range of outcomes. Do not select the single scheme that gives the most attractive map.
- **Independent review:** a reviewer not responsible for producing the candidate map checks label construction, source provenance, folds, metric implementation, and claims.
- **Negative results:** keep a target/review ledger; failures remain visible and may reduce confidence in similar hypotheses.

### Report metrics that answer the spatial question

Report metrics by spatial fold and for aggregate results:

- Precision–recall area under the curve (PR-AUC), with class prevalence and fold composition.
- Recall / hit rate within the top 1%, 5%, and 10% of eligible **surveyed/valid area**.
- Spatial selectivity (fraction of eligible area nominated), with the exact denominator.
- Enrichment over base-rate and simple geological baselines.
- Calibration diagnostics only if probability calibration is explicitly fitted and evaluated on independent data.
- Stability across random seeds, spatial folds, negative-sampling schemes, and reasonable preprocessing choices.
- Metrics by coverage/quality stratum so the model cannot receive undue credit for simply predicting well-mapped districts.

AUROC or F1 alone is insufficient: neither communicates the exploration area required to recover a target, and metrics can be misleading when the unlabeled population contains undiscovered positives.

## 7. Active exploration / value of information

After the baseline and validation gates pass, add a planner that compares candidate **measurements**, not only candidate locations.

For a candidate action (for example, a new spectral acquisition, reprocessing an available magnetic survey, sampling, a geophysical traverse, or—only with separate approvals—drilling), estimate:

- The uncertainty or competing hypotheses it could discriminate.
- Expected information gain and the assumptions behind that estimate.
- Cost, time, logistics, data resolution, footprint, environmental disturbance and permission requirements.
- Whether the measurement is independent of existing evidence or merely repeats it.
- The decision threshold that the result could change.

The planner must permit “collect no new data” and “model is not useful here” as valid outcomes. An expected-information-gain estimate is a planning heuristic, not a guarantee of discovery.

## 8. Cross-system boundaries

| System | Responsibility in this research |
|---|---|
| **Sol-Atlas** | Atlas layers, candidate exploration areas, source/coverage visibility, uncertainty and dissent display, drill-down to manifests, export of a target packet |
| **Symthaea** | Generate/test explicit geological hypotheses; compare competing explanations; suggest the next discriminating observation; keep suggestions separate from authoritative evidence |
| **Mycelix / Mycelix-DeSci** | Bind claims to dataset/artifact digests, source citations, model run IDs, reviewer attestations, supersession history, access controls and permitted distribution |
| **Symtropy** | Not in the MVP. Consider later only for a specific, validated physical process or scenario that an appropriate geoscience solver cannot already support. Do not treat its current rigid-body engine as an ore-genesis simulator. |

Proposed portable **prospectivity result packet**:

- Target-system ID, region/grid-cell identifier and spatial reference.
- Model-run ID, exact source revision, configuration digest, seed, environment/lockfile digest.
- Dataset-manifest IDs and source artifact digests.
- Output score/rank, consensus, disagreement, valid-data coverage, extrapolation flag, and explicitly non-probabilistic interpretation unless calibrated.
- Contributing evidence and counter-evidence with provenance links.
- Validation fold/metric references and reviewer state.
- Follow-up recommendation, expected uncertainty reduction, cost-estimate source, permit/consent constraints and any withholding reason.
- State transitions and supersession links; no silent rewrite of prior results.

## 9. Sol-Atlas presentation requirements

A candidate map is not a single heatmap. The explorer should be able to toggle:

1. Geological context and known deposits/occurrences, with labels visibly differentiated by evidence class.
2. Each input layer, its valid-data mask, age, resolution and source.
3. Prospectivity output and ensemble consensus.
4. Model disagreement / dissent.
5. Input coverage and extrapolation.
6. Negative or failed validation outcomes.
7. Field-investigation actions and review status, when such information is lawfully publishable.

Every map legend and export must state the target-system type, model version, data cutoff, grid resolution, validation scope, and warning: **“Prospectivity hypothesis—not a confirmed deposit, resource, reserve, or extraction recommendation.”**

## 10. Milestones and acceptance gates

### M0 — Dataset and rights inventory

Deliverable: versioned source inventory and layer manifest schema.

Pass only when every proposed input has a known access route and a clear reuse state; unknown licensing is marked explicitly, and no restricted asset is committed.

### M1 — Baseline reproduction

Deliverable: script/notebook or reproducible pipeline to reconstruct the benchmark from declared inputs, including exact preprocessing and output artifacts.

Pass only when the reproduction status is honestly classified (exact / partial / blocked), differences from the published reference are recorded, and rerunning with the same frozen inputs/configuration yields matching outputs within the specified numerical tolerance.

### M2 — Independent spatial validation

Deliverable: versioned folds, leakage tests, baselines, uncertainty and spatial-efficiency report.

Pass only when spatial folds are independent by documented rules, all metrics state their denominators, label uncertainty is evaluated, and at least one independent review verifies the evaluation implementation. A high score alone is not a pass.

### M3 — Satellite incremental-value experiment

Deliverable: side-by-side baseline and satellite-augmented comparisons, with coverage masks and source manifests.

Pass only if any claimed performance improvement survives the same spatial validation and ablation protocol. If it does not improve performance, preserve that negative result and do not retain the layer merely for visual appeal.

### M4 — Sol-Atlas review surface

Deliverable: explicit display of inputs, hypotheses, dissent, missing coverage, evidence lineage and validation state.

Pass only when a reviewer can trace a candidate from map cell to source inputs and reproduce its output from the declared run packet, without interpreting the display as a confirmed discovery.

### M5 — Active measurement planner (deferred)

Deliverable: a limited, benchmarked decision experiment comparing information gain, cost, and constraints.

Pass only after M2 passes and the planner is evaluated against baselines such as highest-score-first and uncertainty-only selection. Do not issue operational survey or drilling instructions automatically.

## 11. Risks and controls

- **Exploration bias:** known mines and samples are not a random sample of the crust. Preserve an unknown class and report spatial coverage.
- **Label leakage / clustered labels:** group deposits and buffers into spatial folds; audit feature dates against the data cutoff.
- **Resolution illusion:** propagate true source scale and masks; never upsample and imply improved physical resolution.
- **Circular corroboration:** several products derived from the same raw survey do not count as independent evidence. Track shared ancestry in the feature/provenance graph.
- **False certainty from ensembles:** show consensus and dissent separately from calibration and data quality.
- **Model/data drift:** version source data and configurations; re-evaluate when material new surveys or geological reinterpretations arrive.
- **Licensing and sovereignty:** verify source terms before hosting, caching, or redistributing; respect land rights, Indigenous/local community rights, permits, confidentiality and environmental constraints.
- **Economic overclaim:** prospectivity is not grade, tonnage, metallurgy, recoverability, feasibility, or commodity-price prediction.
- **Unvalidated physics:** use GPlates/PyGPlates for tectonic reconstructions where applicable; defer geochemical/thermal simulations to domain-valid tools and verified input parameters.

## 12. Immediate implementation tasks

1. Create a source inventory for the 2024 paper and identify which supplementary assets and input layers can legally and practically be used.
2. Define the canonical dataset manifest and spatial-fold schema before processing any large rasters.
3. Build a baseline reproduction in a separate research pipeline; do not load large source data into the Leptos/WASM client.
4. Publish generated, license-compatible vector/raster tiles through a data-serving boundary; the Sol-Atlas client should visualize derived, attributable products, not act as a research compute engine.
5. Run M1 and M2 before implementing any active targeting or adding Symthaea-produced hypotheses to a published map.

## References

- Nwaila et al. (2024), South African consensus-based mineral prospectivity mapping: <https://doi.org/10.1007/s11053-024-10390-w>
- Council for Geoscience map/data portal: <https://maps.geoscience.org.za/>
- USGS Mineral Resources Program data: <https://www.usgs.gov/programs/mineral-resources-program/data>
- USGS Geology, Geophysics, and Geochemistry Science Center data: <https://www.usgs.gov/centers/gggsc/data>
- EarthChem Portal: <https://portal.earthchem.org/>
- Copernicus Data Space APIs: <https://documentation.dataspace.copernicus.eu/APIs.html>
- NASA EMIT: <https://earth.jpl.nasa.gov/emit/>
- GPlates / PyGPlates: <https://www.gplates.org/docs/pygplates/>
