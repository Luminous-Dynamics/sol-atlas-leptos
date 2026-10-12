# Mineral Prospectivity Pilot — Initial Data Inventory

**Inventory status:** First-pass desk research, 2026-10-12  
**Purpose:** Choose the lowest-risk path to M0/M1. This is not an assertion that data have been downloaded, licensed for redistribution, or quality-checked.

## Candidate sources and current disposition

| ID | Source / entry point | What is currently verified | Current reuse/access state | Next action |
|---|---|---|---|---|
| SA-MPM-2024 | [Nwaila et al. (2024), article and supplement](https://link.springer.com/article/10.1007/s11053-024-10390-w) | Open-access article describes a South African consensus-based PGE–Ni–Cu–Cr and Witwatersrand-type Au workflow, 38 covariates, and a supplementary PDF containing compiled data and results. | Article page states CC BY 4.0, but individual supplementary assets, third-party material and all upstream sources still require an asset-level reuse audit. The large supplement has not been ingested or validated in this work. | Inventory supplement files and feature labels; check source attribution, data schemas, exact output files, file size, and license notes before reproducibility claims. |
| SA-CGS | [Council for Geoscience Interactive Web Map](https://maps.geoscience.org.za/) | The portal lists categories including geology, geophysics, geochemistry and mineral resources; it describes digital data, PDF maps and services for use in other applications. | Public portal is confirmed. Bulk access, individual product terms, redistribution rights and programmatic endpoints for selected layers have not been verified. Do not infer an open license from portal visibility. | Identify exact map/service product IDs and scale; save product metadata and terms. Contact the CGS data contact if terms are unclear. |
| USGS-MR | [USGS Mineral Resources Program data](https://www.usgs.gov/programs/mineral-resources-program/data) and [GGGSC Data & Tools](https://www.usgs.gov/centers/gggsc/data) | USGS describes downloadable regional/global geology, mineral-resource, geochemical and geophysical data, including magnetic, gravity and radiometric products. | Useful open-data discovery entry point, mainly for method comparison or compatible global data. License and citation must be checked per selected release; US data must not be used as an assumed substitute for South African coverage. | Select only datasets with clear metadata and geographic relevance; capture release page, terms, units, resolution and processing metadata. |
| EARTHCHEM | [EarthChem Portal](https://portal.earthchem.org/) | Established entry point for geochemical and petrological data. | A portal is confirmed; suitability, data-level licensing, spatial coverage and analytical comparability for this pilot remain unverified. | Filter by target region, sample type and commodity-relevant chemistry; document detection limits, units, method and uncertainty. |
| CDSE-STAC | [Copernicus Data Space APIs](https://documentation.dataspace.copernicus.eu/APIs.html), [STAC API guide](https://documentation.dataspace.copernicus.eu/APIs/STAC.html) | Current documentation describes catalogue APIs and the STAC endpoint; Sentinel-2 data products are described as freely available to public, scientific and commercial users. | Dataset access model is documented. Specific item availability, quotas, processing lineage and exact product-specific attribution/terms need to be recorded per downloaded asset. | Query the chosen AOI and temporal window; persist STAC item IDs, product level, acquisition time, cloud metadata and asset checksums. |
| NASA-EMIT | [EMIT mission/data resources](https://earth.jpl.nasa.gov/emit/) | Imaging spectroscopy is a potential source for mapped surface mineral signatures where spatial and temporal coverage supports the study. | Not a uniform national/global data layer for every area. Specific products and their reuse metadata have not been fetched or evaluated here. | Check footprint, product level, quality flags, spectral mineral-class definitions and overlap with the pilot region before including any feature. |
| GPLATES | [GPlates / PyGPlates documentation](https://www.gplates.org/docs/pygplates/) | PyGPlates supports reconstructing geospatial features through geological time using compatible plate/reconstruction models. | Software capability and documentation verified; no specific South African reconstruction model has been selected or validated here. | Defer until a specific mineral-system hypothesis requires tectonic context and reconstruction model provenance/uncertainty is identified. |

## Findings that affect implementation

1. **The best starting point is the published local benchmark, not a brand-new global satellite scrape.** The 2024 study already has local target labels and compiled covariates, and its methods allow a direct reproduction/partial-reproduction question.
2. **The CGS portal is important but access must be product-specific.** It advertises data downloads and service integration, but that does not establish permission to mirror all layers. Store access state and terms in the manifest.
3. **Remote sensing is a later ablation, not a prerequisite.** Test its added value over geology/geophysics baselines with identical spatial folds and a valid-data mask.
4. **USGS data are not South African substitutes.** Use them for relevant global comparisons or method tests unless an individual dataset is genuinely appropriate to the target area.
5. **Deep-time reconstruction is deferred.** It should enter only if a defined mineral-system hypothesis can be supported by a compatible reconstruction model and data.
6. **No source files have been downloaded, verified, or added to this repository by this inventory.** This status must not be confused with a completed data pipeline.

## M0 completion checklist

- [ ] Supplementary artifacts listed with names, digests, and per-asset terms.
- [ ] Each selected CGS product has an identifier, scale, data vintage, access method and reuse decision.
- [ ] Every candidate layer has a completed manifest conforming to [the dataset manifest schema](schemas/mineral-dataset-manifest.schema.json).
- [ ] Missing coverage, invalid/no-data regions, units and coordinate reference systems are documented.
- [ ] A reviewer checks the provenance and reuse decision before data ingestion.
- [ ] No restricted dataset content is committed or published inadvertently.

## Source links

- Nwaila et al. (2024): <https://doi.org/10.1007/s11053-024-10390-w>
- Council for Geoscience portal: <https://maps.geoscience.org.za/>
- USGS Mineral Resources Program: <https://www.usgs.gov/programs/mineral-resources-program/data>
- USGS GGGSC data: <https://www.usgs.gov/centers/gggsc/data>
- EarthChem: <https://portal.earthchem.org/>
- Copernicus APIs: <https://documentation.dataspace.copernicus.eu/APIs.html>
- Copernicus STAC: <https://documentation.dataspace.copernicus.eu/APIs/STAC.html>
- NASA EMIT: <https://earth.jpl.nasa.gov/emit/>
- GPlates / PyGPlates: <https://www.gplates.org/docs/pygplates/>
