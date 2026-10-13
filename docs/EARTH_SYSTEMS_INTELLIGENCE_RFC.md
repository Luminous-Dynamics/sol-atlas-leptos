# Earth Systems Intelligence: A Multi-Domain Research Framework

- **Status:** Research proposal and domain registry; no domain is certified operational by this document.
- **Date:** 2026-10-13
- **Repository:** Sol-Atlas
- **Related mineral work:** [Mineral prospectivity PR #95](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/95)
- **Machine-readable inventory:** [Earth systems domain catalog](earth-systems-domain-catalog.json)
- **Schema:** [Domain catalog JSON Schema](schemas/earth-systems-domain-catalog.schema.json)

## 1. Thesis: minerals are the first benchmark, not the product boundary

Sol-Atlas can become a spatial/temporal evidence platform for Earth systems and, later, planetary surfaces. Mineral prospectivity is an excellent first benchmark because it forces the program to address heterogeneous evidence, sparse positives, biased labels, spatial resolution mismatch, uncertainty, and expensive follow-up observations. Those capabilities generalize. Mineral-specific models, geological priors, and validation labels do not.

The goal is not a universal AI that predicts anything from a globe. It is a **shared evidence and provenance substrate with separate domain-specific scientific models**, explicit observability limits, and independently testable outputs.

The initial registry names nine research domains:

1. Mineral systems and geological resources.
2. Water security, drought and water-storage change.
3. Agricultural productivity and land-water management.
4. Ecosystems, restoration and carbon.
5. Wildfire, flood, rainfall extremes and drought monitoring.
6. Renewable energy and low-carbon infrastructure screening.
7. Infrastructure resilience and essential-service access.
8. Circular materials, secondary resources and industrial waste.
9. Planetary surfaces and comparative geology.

The registry is intentionally heterogeneous. These domains need different kinds of truth data, units, scales, scientific models, review processes and decision authorities. They must not be merged into a single "planetary score."

## 2. What the external data ecosystem makes feasible

The availability of public source systems means a useful prototype does not require a new satellite constellation or a global proprietary archive. Current official entry points include:

- Copernicus Data Space provides catalog, OData, STAC, download and processing APIs for Earth observation products: https://documentation.dataspace.copernicus.eu/APIs.html
- NASA GRACE/GRACE-FO assimilated drought products provide regional water-storage, shallow-groundwater and soil-moisture indicators, but their grid and modeling assumptions are too coarse to identify an individual depleted well: https://earth.gsfc.nasa.gov/hydro/data/groundwater-and-soil-moisture-conditions-grace-data-assimilation
- FAO WaPOR provides open-access remote-sensing-derived water/land productivity layers for Africa and the Near East at different product levels and resolutions: https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data/
- ESA WorldCover supplies land-cover products and annual composites; its 2020 v100 and 2021 v200 map products use different algorithm versions, so simple pixelwise differencing is not clean evidence of land-cover change: https://esa-worldcover.org/en/data-access
- NASA FIRMS provides active-fire detections through APIs and map services. Sensor, version, latency and product lifecycle need to be versioned; its API page currently announces a Suomi NPP product transition effective 1 November 2026: https://firms.modaps.eosdis.nasa.gov/web-services/
- Copernicus Emergency Management Service distributes drought, flood and emergency mapping products with geographic and event-specific coverage: https://emergency.copernicus.eu/data/
- NASA's Planetary Data System preserves planetary mission, ground-based and laboratory data; USGS Astrogeology supplies planetary mapping/GIS and analysis-ready data resources: https://pds.nasa.gov/ and https://www.usgs.gov/special-topics/planetary-geologic-mapping

These are candidate source families, not proof that every product, layer or downstream artifact is currently available under unrestricted redistribution terms. Every imported artifact requires a versioned inventory, exact source/product ID, access method, terms review, checksum, coordinate/measurement metadata, temporal support and source lineage.

## 3. Architecture: one evidence fabric, many science packs

### Sol-Atlas — the spatial/temporal interface

Sol-Atlas should present domain-specific layers, time slices, source resolution, valid-data masks, provenance, uncertainty, disagreement and validation status. Large rasters and model outputs should be produced in an offline/reproducible research pipeline and published through a data-serving boundary; they should not be downloaded or modeled inside the Leptos/WASM client.

### Symthaea — hypothesis and next-measurement assistant

Symthaea can compare explicit hypotheses, detect contradictions between evidence, identify out-of-distribution regions, and suggest the next observation that may discriminate between plausible explanations. A generated hypothesis remains a hypothesis. Symthaea must not invent missing input evidence or promote its own confidence to scientific authority.

### Mycelix / Mycelix-DeSci — provenance, access and review

Record dataset/artifact digests, transformations, model/configuration IDs, reviewer attestations, access and consent constraints, and supersession history. Cryptographic integrity proves which artifact was used; it does not prove that its scientific interpretation is correct. Community consensus or a trust score is not a substitute for domain expertise or field evidence.

### Symtropy — domain-appropriate scenarios only

Symtropy may coordinate deterministic scenario experiments when its existing capabilities fit a specific problem. It is not currently a validated hydrology, climate, tectonic, geochemical, ecosystem or ore-genesis solver. Use established domain solvers or statistical models for those calculations; add a Symtropy bridge only after a meaningful model has been validated.

## 4. Common evidence contract

Every domain pack must define, before its model is run:

- The scientific question and target/quantity, including units and what outcome is being estimated.
- Observation vs derived product vs model estimate vs expert hypothesis.
- Source/product IDs, checksums, citation, licensing/reuse state and per-layer lineage.
- Spatial reference, measurement/nominal/effective resolution, footprint, coordinate uncertainty, time range, product vintage and no-data semantics.
- Valid-data masks, missingness, known selection bias, product limitations and source-version changes.
- Domain-specific baseline, holdout strategy, metrics, uncertainty method, independent-review requirement and known failure cases.
- The decision the output can inform, who has authority to make it, and what human/community/permit/environmental constraints must apply.

Missing evidence stays missing. A coarse pixel remains coarse after rasterization to a finer grid. Multiple derived layers from the same upstream sensor or product do not count as independent corroboration. Model disagreement, out-of-distribution status and input coverage must be shown separately from predictive uncertainty.

## 5. High-value pilot sequence

### P0 — Mineral prospectivity benchmark

Continue the South African PGE–Ni–Cu–Cr / Witwatersrand-type Au reproduction in PR #95. Reproduce the published baseline as faithfully as the actual supplement permits, then separately test generalization using district/deposit-grouped spatial holdouts. Do not call modeled undiscovered-resource tracts independent labels.

### P1 — Baixo Limpopo irrigation water–food productivity

The first water–food experiment is now scoped to the FAO WaPOR Level III Baixo Limpopo area in Mozambique, rather than a whole-basin or continent-wide claim. The dedicated [pilot protocol](pilots/BAIXO_LIMPOPO_WATER_PRODUCTIVITY.md) and [machine-readable profile](pilots/baixo-limpopo-water-productivity.json) define a source-readiness report first, followed by a seasonal baseline only if exact product layers, dates, quality masks, and common support justify it.

Candidate inputs are WaPOR v3 product layers selected only after the live catalogue and product metadata have been frozen, plus CHIRPS v3 monthly Africa-subset GeoTIFFs from the official final monthly archive. Monthly is the first-pass cadence because CHIRPS v3's daily products are derived by partitioning pentadal totals with ERA5 or IMERG. The read-only inventory tool snapshots the official monthly index and checks month completeness before a later, separate acquisition step. CHIRPS v2 production is scheduled to end after December 2026; its blended station inputs must be audited before any purported gauge-based independent validation. WaPOR remains product-level rights-review-required until the exact layer's metadata and reuse terms are checked.

This pilot asks where source-supported indicators change and whether their seasonal co-variation merits further measurement. It does not estimate farm yield, prove causal irrigation effects, identify individual wells or authorize water allocation. GRACE is excluded from the first pass because its regional grid is not an irrigation-scheme or well-scale observation.

### P1 — Hazard observation and impact review

Use fire detections, rainfall and available flood/drought products to create a latency-aware observations layer. Distinguish event time, sensing time and product-publication time. A detected hotspot is not a fire perimeter; a map of observed rainfall is not a flood forecast. Emergency response authority stays with the relevant public/emergency-service organizations.

### P2 — Ecosystem and restoration monitoring

Combine land-cover products with GEDI vegetation structure/biomass-related products, forest-change sources and local inventory/field evidence. Use product-version-aware change detection. Do not claim that vegetation greenness is biodiversity, that biomass equals total ecosystem carbon, or that an alert establishes illegality or land ownership.

### P2 — Renewable-resource and infrastructure pre-feasibility

Combine solar/wind resource data with grid capacity, settlement/demand, transport, water, ecology, land rights and permitting information. Resource maps alone cannot establish feasibility or social acceptance. Keep objective trade-offs explicit and make alternative weightings visible.

### P3 — Circular material systems

Start from documented mines, processing plants, tailings inventories, waste records, assays and material-flow data. Remote sensing can identify candidate landforms or changes but cannot determine material composition, contamination, recoverability or safety on its own.

### P3 — Planetary surface science

A separate long-term track can use NASA PDS and USGS planetary mapping for lunar/Martian geology, topography and calibrated remote sensing. Planetary bodies require their own coordinate reference systems, projections, calibration models and validation. Earth-trained geology models must not be transferred without testing.

## 6. A decision system, not a heatmap factory

Each research cycle should choose among measurement actions, not only locations. Candidate actions can include obtaining a new satellite observation, acquiring a better source layer, collecting a field sample, checking a borehole or gauge record, running a survey, or choosing to collect no more data.

Measure the expected value of information using explicit assumptions:
- Which competing hypotheses would the observation distinguish?
- Is it genuinely independent of existing inputs or derived from the same source?
- What is its expected resolution, coverage, latency and uncertainty?
- What are its cost, access permissions, environmental impact and community implications?
- Would its possible outcomes change a decision?

Only implement an active planner after the associated domain passes baseline and independent-validation gates. No automatic drilling, extraction, land-access, aid-allocation or emergency-control instructions.

## 7. Domain gates and maturity levels

The catalog uses these conceptual gates:

- **D0 — Inventory:** research question, source list, licensing/access review and observability limits.
- **D1 — Data intake:** frozen source IDs/digests, metadata, CRS/units, valid-data mask, lineage and schema checks.
- **D2 — Baseline:** reproduce or define a transparent domain-specific baseline with documented deviations.
- **D3 — Independent validation:** spatial/time/grouped holdouts matched to the scientific target, leakage tests, appropriate metrics and calibration/uncertainty checks.
- **D4 — Independent review:** review labels, source provenance, assumptions, validation implementation, failures and claims.
- **D5 — Decision support:** traceable user-facing outputs, known limitations, update policy and accountability for actions.

A domain in the JSON registry is a **candidate research direction**, not an implemented feature. Only domains that pass D0–D5 may be described as operational, and validation in one domain does not certify another.

## 8. Cross-domain questions without false unification

Some decisions genuinely need multiple evidence families: drought may affect land productivity and ecosystem stress; renewable siting may interact with water use, land cover and service access; floods may interact with settlement exposure and infrastructure access. These should be modeled as transparent decision graphs with source lineage and uncertainty on each edge.

Do not imply causation from correlation. Keep each domain's units, validation and uncertainty intact before combining results. Make value judgements and multi-objective trade-offs explicit rather than hiding them in one learned scalar.

## 9. Catalog policy and implementation

The machine-readable [domain catalog](earth-systems-domain-catalog.json) lists target questions, priorities, readiness, candidate data sources and their limitations. [Its schema](schemas/earth-systems-domain-catalog.schema.json) defines the interface; the standard-library [validator](../scripts/earth-systems/validate_catalog.py), [tests](../tests/test_earth_systems_catalog.py), [usage guide](../scripts/earth-systems/README.md), and [CI workflow](../.github/workflows/earth-systems-catalog.yml) check catalog integrity.

Catalog checks are structural safeguards only. They do not fetch datasets, adjudicate licensing, certify scientific validity or establish that any domain is implemented.

## 10. Immediate implementation plan

1. Land and maintain the catalog/schema/validator with CI.
2. Continue PR #95 for mineral-specific ingestion and benchmark work; do not blend mineral labels into other domains.
3. Use [the WaPOR catalogue inventory](../scripts/earth-systems/inspect_wapor_catalog.py) and [CHIRPS monthly index planner](../scripts/earth-systems/plan_chirps_monthly.py) to snapshot public metadata first, then freeze the exact WaPOR layer and CHIRPS month window, artifact bytes, release metadata and rights before analysis. These tools do not download raster files.
4. Build a small, reproducible baseline/coverage report before any advanced model.
5. Test against independent ground observations and report limits; if the required local reference data cannot be obtained, keep the pilot at data-readiness status.
6. Add Sol-Atlas presentation only after the output has a validated evidence packet.

## References

- Copernicus Data Space APIs: https://documentation.dataspace.copernicus.eu/APIs.html
- NASA GRACE/GRACE-FO groundwater and soil-moisture indicators: https://earth.gsfc.nasa.gov/hydro/data/groundwater-and-soil-moisture-conditions-grace-data-assimilation
- FAO WaPOR data: https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data/
- CHIRPS v3 rainfall data and citation: https://www.chc.ucsb.edu/data/chirps3
- ESA WorldCover data and license: https://esa-worldcover.org/en/data-access
- NASA FIRMS web services and API: https://firms.modaps.eosdis.nasa.gov/web-services/
- Copernicus Emergency Management Service data: https://emergency.copernicus.eu/data/
- NASA GPM data directory: https://gpm.nasa.gov/data/directory
- NASA GEDI data: https://www.earthdata.nasa.gov/data/instruments/gedi
- Global Forest Watch Data API: https://data-api.globalforestwatch.org/docs
- Global Solar Atlas: https://globalsolaratlas.info/
- OpenStreetMap copyright and ODbL: https://www.openstreetmap.org/copyright
- Global Human Settlement Layer: https://human-settlement.emergency.copernicus.eu/
- NASA Planetary Data System: https://pds.nasa.gov/
- USGS Planetary Geologic Mapping: https://www.usgs.gov/special-topics/planetary-geologic-mapping
