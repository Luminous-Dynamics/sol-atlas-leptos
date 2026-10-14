# Johannesburg / Highveld Food-Resilience Model

- **Profile:** agroclimate-johannesburg-highveld-v1, version 1.0.0
- **Status:** Design-only. No source raster or local field-observation artifact has been acquired by this profile.
- **Related work:** [Earth Systems Intelligence RFC and Baixo Limpopo pilot, PR #101](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/101); [Mycelix food-cooperative vertical slice, issue #5001](https://github.com/Luminous-Dynamics/mycelix/issues/5001); [Sol Atlas food-network layer, issue #105](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/105).
- **Machine-readable profile:** [johannesburg-highveld-food-resilience.json](johannesburg-highveld-food-resilience.json)
- **Schema:** [agroclimate-model-profile.schema.json](../schemas/agroclimate-model-profile.schema.json)

## Why Johannesburg first

Johannesburg/Highveld gives the project a concrete local problem: produce reliable, affordable food while handling seasonal water stress, heat, uneven water access, and variable production methods. The intended users include community gardens, small farms, growers using protected cultivation, and neighborhood food cooperatives.

GAEZ's source-defined scenario projection windows must retain their own model-time semantics; the 2021–2040 window is partly elapsed as of this profile's review date and is not a forward operational forecast.

The South African Government published an agricultural weather notice on 2 September 2026, summarizing the South African Weather Service (SAWS) Seasonal Climate Watch dated 31 August 2026. It says El Niño conditions are expected through the 2026/27 summer season and below-normal rainfall may affect many areas. This is a **risk signal motivating readiness work**, not a deterministic forecast for Johannesburg, a particular farm, or the current growing plot. The original SAWS bulletin, local spatial probabilities and validity dates must be acquired and reviewed before its output is used by a model.

Reference: https://www.gov.za/news/media-statements/agriculture-el-nino-weather-climate-02-sep-2026

## Do not build one black-box model per climate zone

The proposed architecture is a shared global evidence/model contract plus regional configurations and crop-/production-system-specific science packs. Climate archetypes are useful for sampling, evaluation and navigation, but they are labels—not the scientific model itself. Use continuous site and climate variables underneath them.

This lets the same software and evaluation rules work in the Highveld, Mediterranean climates, humid tropics, semi-arid regions and protected growing systems without claiming that one parameterization transfers everywhere.

The JSON Schema is a shared contract: it accepts other region/profile identifiers and source families. The Johannesburg-specific validator separately pins this pilot's six expected sources, local-source permissions, source-resolution caveats, and maturity claims. The reusable schema therefore does not imply that any one region's source set or crop parameters are globally applicable.

Shared capabilities should include:
- versioned source manifests and immutable artifact digests;
- explicit units, spatial/temporal support, uncertainty and missingness;
- provenance, source overlap, correction and supersession;
- train/validation/test splits grouped by site and season where feasible;
- fixed baselines, out-of-sample evaluation and separate maturity gates;
- reusable reports that preserve separate objectives rather than hiding water use, marketable yield, cost and access inside one score.

## Candidate data and models

These source families are candidates, not acquired evidence. Every row in the machine-readable profile remains not acquired until the exact product/artifact is selected, access and reuse terms are reviewed, and bytes are hashed.

The common rights field is a review state; exact licence details belong in a separate licence reference. A known dataset-family licence does not automatically grant redistribution rights for an unselected derivative or justify skipping product-specific metadata review.

### FAO GAEZ v5 — global screening, not farm truth

GAEZ v5 combines agroclimatic potential with soil/terrain constraints. Its documented products include suitability and attainable-yield information for more than 70 crops and land-use types, rainfed/irrigated conditions, input-management levels, historical reference periods and CMIP6-based future periods. Key outputs are approximately 30 arc-seconds (about 1 km). First inspect the crop/product catalogue, units, masks and exact model configuration; do not assume the desired crop/variety is included or infer a plot-specific yield from a regional suitability cell.

References:
- https://www.fao.org/gaez/en
- https://data.fao.org/catalog/dataset/gaez-v5-master-config/resource/31abf121-1644-4473-8c1b-bfec83112749

### ERA5-Land — historical weather backbone

ERA5-Land provides hourly land reanalysis from 1950 to present. Its native support is around 9 km while distributed data are available on a 0.1-degree grid. A distributed grid cell is not a new direct observation, and ERA5-Land is a model-based replay driven by ERA5 atmospheric variables. Use it for a consistent historical backbone and scenario preparation; compare it to suitable local station records and preserve source vintage and uncertainty.

Reference: https://cds.climate.copernicus.eu/datasets/reanalysis-era5-land

### CHIRPS v3 monthly — rainfall candidate with overlap checks

A 0.05-degree monthly Africa rainfall product is a reasonable candidate for broad seasonal rainfall context, but its pixels are not field rain gauges. CHIRPS blends satellite estimates and in-situ station observations. Before comparing with stations, inspect station-input lineage and do not report a contributing gauge as independent validation. Select an analysis window only after capturing and hashing the current official index and verifying exact month IDs, units, masks and release status. A browser view is not a frozen data artifact.

References:
- https://www.chc.ucsb.edu/data/chirps3
- https://data.chc.ucsb.edu/products/CHIRPS/v3.0/monthly/africa/tifs/

### SoilGrids 2.0 — regional prior, not a substitute for soil samples

SoilGrids provides global soil-property predictions at 250 m over six depth intervals, with prediction uncertainty represented by quantile layers. ISRIC specifically cautions against using this global product for farm-level conclusions. As checked on 14 October 2026, ISRIC also reports that the complementary REST API is paused. Do not build a runtime around that API. Review the documented alternative access path, preserve prediction intervals and exact data vintage, and use local soil sampling for field calibration.

References:
- https://docs.isric.org/globaldata/soilgrids/index.html
- https://docs.isric.org/globaldata/soilgrids/SoilGrids_faqs_01.html

### SAWS and local observations — required grounding

The original SAWS Seasonal Climate Watch bulletin and suitable historical station observations must be reviewed before any seasonal forecast is used. Historical data access, terms and available station density are not assumed here.

Calibration needs local, authorized evidence for:
- daily minimum/maximum temperature, rainfall and reference evapotranspiration (or its input weather variables);
- soil profile and water-holding parameters at the relevant field/site scale;
- water source, actual availability, storage, irrigation volume and schedule, interruptions and restrictions;
- crop species/variety, sowing/transplanting and harvest dates, cultivar parameters, management and inputs;
- measured harvest amount, marketable fraction, losses and the protocol that produced each observation.

Farm/household locations and grower data need permission and access controls. Do not publish exact private coordinates just because the model uses spatial data.

### Candidate crop-water model: FAO AquaCrop

FAO AquaCrop 7.3 is a candidate established reference model for suitable herbaceous crops under water-limited conditions. It needs daily weather variables including maximum/minimum temperature and rainfall, reference evapotranspiration, and crop/soil/management parameterization. It is not a universal crop solver and does not remove the need for local calibration and physical harvest observations.

The first implementation should compare a transparent water-balance baseline with AquaCrop where its crop scope and inputs fit. Do not start by inventing a new crop-yield model. The AquaCrop reference implementation may be invoked as an isolated reproducible research tool; Sol Atlas's Rust/WASM client should not embed a Python runtime. A future native implementation requires reference-vector parity and independent validation.

References:
- https://www.fao.org/aquacrop/overview/en
- https://www.fao.org/aquacrop/software/
- https://www.fao.org/4/i2800e/i2800e00.htm

## Evaluation plan

Keep model decisions staged:

1. **D0 — freeze the study boundary and decision.** Choose an authoritative, versioned region boundary. Johannesburg municipality and the broader Highveld/food catchment are not interchangeable.
2. **D1 — source readiness.** Confirm exact products, spatial/temporal support, time standards, units, rights, product vintage and exact file hashes. Check source overlap and missingness before selecting the comparison period.
3. **D2 — descriptive baseline.** Produce source coverage and a transparent water/climate baseline only after the input record is adequate. Do not call this a forecast.
4. **D3 — independent validation.** Use time-appropriate hindcasts, held-out sites/seasons and independent references. Split by farm/site or growing season where feasible to avoid leakage from near-duplicate records. Keep a simple climatology and interpretable water-balance baseline.
5. **D4 — expert review.** Ask local growers, agronomists, soil specialists and irrigation/water experts to review crop parameters, soil assumptions, management and practical constraints.
6. **D5 — decision-support candidate.** Only after previous gates pass, evaluate whether outputs are reliable enough for bounded advisory use. No automatic planting, irrigation, payment, cooling or robot command authority follows from this profile.

Report multiple outcomes independently: rainfall/ET0 error, water-balance residual, irrigation-demand error, measured harvest/marketable-yield error, stress-event calibration, water per marketable kilogram, cost and supply continuity. Preserve uncertainty intervals, sample sizes, denominators and missingness. Show failures by crop, site, cultivation type and season. A model that saves water but collapses marketable harvest—or raises yield but consumes unavailable water—is not a success just because one scalar metric improves.

## Integration boundaries

- **Sol Atlas:** spatial/time visualization and source/uncertainty layers. It is not the physical-inventory authority or climate solver.
- **Symthaea:** advisory synthesis, contradiction detection and hypotheses about the next useful measurement. Model confidence is not scientific authority.
- **Symtropy:** later deterministic scenario comparisons only where its tested physical abstractions fit; it is not a validated crop-growth or hydrology solver.
- **Mycelix:** provenance, product/lot records, offers, reservations, dispatch/receipt and recall lineage. Model forecasts remain forecasts and must never create stock or confirm food safety.
- **Food-network maps:** consume read-only, explicitly versioned projections. Do not duplicate farmer identity, inventory, provenance or cooperative economics.

## Current maturity and limits

The profile is design-only. The candidate study boundary is unverified. No source raster or local field observations have been acquired or hashed by this profile; no local baseline, forecast, crop-suitability result, yield estimate or operational recommendation has been produced. A passing schema/contract test would show that the research plan is structurally consistent, not that the agricultural model is scientifically accurate.
