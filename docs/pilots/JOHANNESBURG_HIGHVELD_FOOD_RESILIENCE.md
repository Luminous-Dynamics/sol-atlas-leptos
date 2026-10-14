# Johannesburg / Highveld Food-Resilience Model

- **Profile:** agroclimate-johannesburg-highveld-v1, version 1.0.0
- **Status:** Design-only. No source raster or local field-observation artifact has been acquired by this profile.
- **Related work:** [Earth Systems Intelligence RFC and Baixo Limpopo pilot, PR #101](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/101); [Mycelix food-cooperative vertical slice, issue #5001](https://github.com/Luminous-Dynamics/mycelix/issues/5001); [Sol Atlas food-network layer, issue #105](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/105).
- **Machine-readable profile:** [johannesburg-highveld-food-resilience.json](johannesburg-highveld-food-resilience.json)
- **Schema:** [agroclimate-model-profile.schema.json](../schemas/agroclimate-model-profile.schema.json)

## Why Johannesburg first

Johannesburg/Highveld gives the project a concrete local problem: produce reliable, affordable food while handling seasonal water stress, heat, uneven water access, and variable production methods. The intended users include community gardens, small farms, growers using protected cultivation, and neighborhood food cooperatives.

For the first boundary, use the **City of Johannesburg municipal area** as the local-production study extent. “Highveld” is the climate context, not a substitute polygon, and the regional food-supply catchment is a separate supplier/market network that can extend beyond municipal borders. This prevents an administrative boundary from being mistaken for a climate boundary or from excluding incoming food supply.

Two official boundary discovery leads are recorded in the machine-readable profile: the City of Johannesburg Corporate Geo-Informatics Spatial Data Inventory (labelled 2024) and the government-hosted Municipal Demarcation Board local-municipality feature service. The City inventory documents custodian, format, creation-date and usage-limit fields and says it is reviewed annually; the MDB service describes itself as a municipality polygon layer and reports WKID 4148. Neither has yet been frozen as the study geometry. Confirm the current authoritative boundary/effective date, native CRS semantics, usage terms and exact geometry bytes; then record its URI, SHA-256 and independent D0 review before analysis.
The MDB service metadata also reports `isDataVersioned=false` and no historical-moment query support, so the live service is not an immutable historical record. Its `Timestamp` field is a version clue to investigate, not automatically the legal boundary effective date. Archive the selected feature's exact response/export and retrieval time before the source changes. The EPSG registry defines WKID 4148 as Hartebeesthoek94 geographic with latitude/longitude axis order, while ArcGIS service metadata presents extents in x/y fields. Inspect actual exported coordinate tuples and validate known control locations; do not silently relabel the artifact as EPSG:4326 or assume axis order from the metadata alone.
There is also a real boundary-process history to account for. The MDB's January 2024 Section 21 publication listed DEM7008 as a re-determination involving Stretford Ext 1, Emfuleni, Sedibeng and Johannesburg; MDB Circular 2 of 2024 later records DEM7008 as **withdrawn under Section 21(5)**. The earlier proposal must not be treated as an effective inclusion in Johannesburg. A 2026 ward-delimitation Gazette is a separate electoral geography reference and is not a substitute for the municipal polygon. These records demonstrate why D0 must record the legal effective date/source version alongside the geometry digest.
A later MDB update dated 29 September 2025 says it would not redetermine municipal boundaries at that time for the deferred cases considered under Circulars 3 of 2025, while the 2026 ward-delimitation process continued. That is useful procedural context but is scoped to those deferred cases; it does not itself settle Johannesburg's current boundary. For this study, record the exact case history, legal effective date, source vintage and the archived geometry hash together.

References:
- MDB Circular 2 of 2024 (DEM7008 withdrawn): https://www.demarcation.org.za/wp-content/uploads/2024/03/Final-Circular-2-2024.pdf
- January 2024 Gauteng Gazette containing the Section 21 notice: https://www.demarcation.org.za/wp-content/uploads/2024/01/Gauteng_S21_009-10-1-2024.pdf
- Gauteng ward delimitation for 2026 local-government elections, published 17 December 2025: https://www.demarcation.org.za/wp-content/uploads/2025/12/473-17-12-2025-GautengDem.pdf

References:
- City of Johannesburg CGIS inventory and data-custodian guidance: https://eservices.joburg.org.za/sites/cgismaps/Pages/default.aspx
- Municipal Demarcation Board local-municipality service: https://nspdr.dlrrd.gov.za/server/rest/services/NationalDatasets/MunicipalDemarcationBoard/MapServer/3
- EPSG:4148 Hartebeesthoek94 CRS definition: https://epsg.io/4148
- MDB municipal-boundary process and redetermination overview: https://www.demarcation.org.za/municipal-boundaries/
- MDB update (29 September 2025) on deferred boundary-redetermination cases: https://www.demarcation.org.za/final-chance-to-shape-your-wards-ahead-of-the-2026-local-government-elections/

GAEZ's source-defined scenario projection windows must retain their own model-time semantics; the 2021–2040 window is partly elapsed as of this profile's review date and is not a forward operational forecast.

The South African Government published an agricultural weather notice on 2 September 2026, summarizing the South African Weather Service (SAWS) Seasonal Climate Watch dated 31 August 2026. It says El Niño conditions are expected through the 2026/27 summer season and below-normal rainfall may affect many areas. This is a **risk signal motivating readiness work**, not a deterministic forecast for Johannesburg, a particular farm, or the current growing plot. The original SAWS bulletin, local spatial probabilities and validity dates must be acquired and reviewed before its output is used by a model.

Reference: https://www.gov.za/news/media-statements/agriculture-el-nino-weather-climate-02-sep-2026

## Local constraints that a climate model cannot solve by itself

City policy and projects give several local priorities more concrete shape. The City's 2024 Food Resilience Policy discusses urban agriculture, hubs, market needs and supply from other productive areas. Its front matter calls it final and council-approved, but an internal notice says the PDF is not a policy instrument until a council resolution is passed; we should retrieve that resolution or seek City confirmation before treating the document as binding. Its 2025 urban-agriculture project report describes site infrastructure including water tanks, agricultural tunnels and cold rooms. These are useful design references, not an inventory of current available capacity at every garden.

The City also reported in 2026 that an eco-filter and water-quality monitoring project in the Upper Jukskei catchment addresses microbial contamination, excess nutrients, organic waste and heavy metals, with monitoring linked to irrigation at Victoria Yards. This is a specific catchment/project signal—not proof that every Johannesburg irrigation source is contaminated. It nevertheless argues for profiling each intended water source rather than assuming that visible water, a remote-sensing layer or a generic “available water” quantity is adequate.

Reference leads:
- City of Johannesburg Food Resilience Policy 2024: https://joburg.org.za/documents_/Documents/FOOD-RESILIENCE-FINAL-APPROVED-POLICY-2024.pdf
- City urban-agriculture infrastructure project: https://joburg.org.za/media_/Newsroom/Pages/2025-News-Articles/Joburg%E2%80%99s-new-urban-agriculture-project-to-boost-community-farming.aspx
- Upper Jukskei eco-filter and water-quality monitoring: https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/Joburg-launches-innovative-eco-filter-system-to-address-pollution-in-Jukskei-River.aspx
- National Department of Health food-control regulation index: https://www.health.gov.za/food-control-regulations/
- Department of Water and Sanitation, South African Water Quality Guidelines, Volume 4: Agricultural Use—Irrigation (second edition, 1996; verify current applicability): https://www.dws.gov.za/iwqs/wq_guide/edited/Pol_saWQguideFRESHIrrigationvol4.pdf

Accordingly, the profile separates six site-level assessments from climate suitability:

1. **Water quantity and quality:** source, lawful access, seasonal reliability, storage and interruptions; appropriate microbiological/chemical assessment for intended crops and irrigation contact; salinity/sodicity where relevant. Reused wastewater/greywater requires specific permissions and risk review.
2. **Soil condition and contaminants:** representative site sampling; soil fertility and physical parameters; history- and pathway-informed contaminant screening before edible production where warranted. SoilGrids is not contaminant clearance.
3. **Land access and permission:** authorized cultivation, duration, restrictions, accountable operator and relevant approvals.
4. **Postharvest, energy and transport:** actual storage conditions, power interruptions, capacity, handling, packing, delivery and measured loss. A cold-room marker is not a qualified cold chain.
5. **Market, nutrition and affordability:** aggregated demand/prices, local nutrition priorities, labour/input/water/energy/storage/distribution costs, spoilage and access. High yield per hectare alone is not the objective.
6. **Regional supply and contingency:** current suppliers, seasonality, markets/hubs, transport, delivered price, actual capacity and substitutes under disruption. Johannesburg resilience does not require producing all food within municipal boundaries; regional sourcing and local production should be compared as complementary options.

Each assessment begins as `not_assessed`. Moving to `assessed` requires an auditable evidence reference and reviewer identity/role. The decision-support gate must fail closed until all six assessments have passed their own review. Model output cannot declare water safe, soil uncontaminated, land available or food compliant.

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

## A locally grounded crop cohort—not an automatic recommendation

A 2020 City of Johannesburg programme statement records tomatoes, spinach, green pepper and cucumber among crops handled by small-scale-farmer cooperatives. I have encoded these four as a **candidate cohort only** so the first crop-selection discussion starts from a documented local production lead instead of an arbitrary global top-crop list.

References:
- https://joburg.org.za/media_/Pages/Media/Media%20Statements/2020%20Media%20Statements/December/Mayor-Makhubo-launches-a-R34-million-commercialization-and-mechanization-programme-for-small-scale-farmers.aspx
- The 2024 City Food Resilience Policy also emphasizes diverse foods, training, technology and localized food-system support: https://joburg.org.za/documents_/Documents/FOOD-RESILIENCE-FINAL-APPROVED-POLICY-2024.pdf

This is a dated signal about one programme, not evidence of the dominant 2026 crop mix or current consumer demand. Before selecting a crop for model evaluation, the pilot needs current growers/extension input, crop-specific temperature and frost sensitivity, seasonal calendar, model support, soil and water requirements, actual prices/demand and post-harvest fit. If AquaCrop does not support or cannot be calibrated for a candidate crop, it must not be forced into that role; choose an appropriate reference model or keep that crop outside the initial yield benchmark.

The City's current food-resilience pages also describe agri-resource centres, market intelligence, hubs, packing/grading/distribution and food-storage compliance needs. These are potential partners/data leads rather than confirmed live feeds:
- https://joburg.org.za/media_/Pages/Media/Marang%20Newsletter/Quarter%201/City-of-Johannesburg-food-resilience-projects.aspx
- Joburg Market's 10 June 2026 cold-room commissioning notice: https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/Joburg-Market-unveils-new-cold-rooms-to-boost-food-security.aspx

## Candidate data and models

These source families are candidates, not acquired evidence. Every row in the machine-readable profile remains not acquired until the exact product/artifact is selected, access and reuse terms are reviewed, and bytes are hashed.

The common rights field is a review state; exact licence details belong in a separate licence reference. A known dataset-family licence does not automatically grant redistribution rights for an unselected derivative or justify skipping product-specific metadata review.

### FAO GAEZ v5 — global screening, not farm truth

GAEZ v5 combines agroclimatic potential with soil/terrain constraints. Its documented products include suitability and attainable-yield information across crop/land-utilization types, rainfed/irrigated conditions, management levels, historical reference periods and CMIP6-based future periods. Resolution is product-specific: the master catalogue describes many layers at 5 arc-minutes (about 10 km at the equator), with selected suitability and attainable-yield maps at 30 arc-seconds (about 1 km). Do not describe the entire product family as 1 km. Freeze the exact map code, units, mask, model configuration and spatial support; never infer plot-specific yield from a regional screening cell.

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

## What counts as gate evidence

The gate status is not the evidence artifact. A `passed` gate must have at least one auditable evidence reference and an independent reviewer identity/role; a `not_started` gate must have neither. A `blocked` gate must record a specific reason.

- **D0:** source-authoritative study-boundary artifact, exact SHA-256, coordinate reference system, privacy/location-precision rule and boundary review.
- **D1:** source manifest with selected product IDs, byte digests, terms/access decisions, units, temporal/spatial support and upstream overlap.
- **D2:** reproducible baseline output, selected crop cohort, model/configuration digest and data coverage report.
- **D3:** independently held-out evaluation report, split manifest, metrics, uncertainty and source-overlap analysis.
- **D4:** dated local agronomist, soil/water specialist, grower or other relevant expert review artifact with scope and limitations.
- **D5:** reviewed decision-support authorization record naming the accountable operator, permitted use, confidence/uncertainty limits and stop conditions.

References use stable HTTPS/URN/SHA-256-style identifiers. Hashes identify bytes, not correctness; reviewers must be able to retrieve or inspect the underlying artifacts and verify their provenance. The current validator checks the structure and gate-order invariants only; it does not claim that a referenced URL resolves or that an artifact's content is scientifically valid.

## Evaluation plan

Keep model decisions staged:

1. **D0 — freeze the study boundary and decision.** Choose an authoritative, versioned region boundary. Johannesburg municipality and the broader Highveld/food catchment are not interchangeable.
2. **D1 — source readiness.** Confirm exact products, spatial/temporal support, time standards, units, rights, product vintage and exact file hashes. Check source overlap and missingness before selecting the comparison period.
3. **D2 — descriptive baseline.** Produce source coverage and a transparent water/climate baseline only after the input record is adequate. Do not call this a forecast.
4. **D3 — independent validation.** Use time-appropriate hindcasts, held-out sites/seasons and independent references. Split by farm/site or growing season where feasible to avoid leakage from near-duplicate records. Keep a simple climatology and interpretable water-balance baseline.
5. **D4 — expert review.** Ask local growers, agronomists, soil specialists and irrigation/water experts to review crop parameters, soil assumptions, management and practical constraints.
6. **D5 — decision-support candidate.** Only after previous gates pass, evaluate whether outputs are reliable enough for bounded advisory use. No automatic planting, irrigation, payment, cooling or robot command authority follows from this profile.

Report multiple outcomes independently: rainfall/ET0 error, water-balance residual, irrigation-demand error, measured harvest/marketable-yield error, stress-event calibration, water per marketable kilogram, cost and supply continuity. Preserve uncertainty intervals, sample sizes, denominators and missingness. Show failures by crop, site, cultivation type and season. A model that saves water but collapses marketable harvest—or raises yield but consumes unavailable water—is not a success just because one scalar metric improves.

## Interoperable source metadata and artifact rules

A global framework only transfers cleanly if source adapters preserve the original product semantics. Prefer the **OGC STAC 1.1.0** family for discovery where a provider offers it; use a documented adapter for catalogs that do not expose STAC. For authorized raster serving, consider **OGC Cloud Optimized GeoTIFF (COG 1.0)** where it fits the native product. For climate/forecast NetCDF, validate relevant **CF metadata conventions** such as units, standard names, calendars, time bounds, grid mapping, cell methods and missing-value definitions.

References:
- OGC STAC: https://www.ogc.org/standards/stac/
- OGC Cloud Optimized GeoTIFF: https://www.ogc.org/standards/ogc-cloud-optimized-geotiff/
- CF Metadata Conventions: https://cfconventions.org/cf-conventions/cf-conventions.html

These formats are interoperability conventions, not accuracy certificates. Do not force a format on an upstream product, silently resample to finer grids, infer missing unit metadata, discard time bounds/calendar semantics, or republish data without checking exact rights.

Every source record carries an explicit `artifacts` list. It is empty while the source is `not_acquired`; an acquired source must have exactly one primary artifact. The source-level primary digest must exactly match that artifact's digest. This avoids a common trap where an acquisition status and a 64-character hash string are recorded without any asset-level provenance.
The source record distinguishes `source_catalog_verified` (the official provider/product-family reference was reviewed) from `selected_product_identity_verified` (the exact data product acquired for this profile was identified and checked). The latter must remain false and `selected_product` null while acquisition is `not_acquired`. A general provider homepage review is not proof that an exact raster/time slice/configuration has been acquired.
Each source family also declares `required_for_baseline`. In this pilot, ERA5-Land and authorized local observations are required for the first locally grounded baseline; GAEZ, CHIRPS, SoilGrids and the SAWS seasonal bulletin are candidate/supporting families that may remain unacquired if the selected question and baseline do not need them. D1 only passes when every required source is acquired, catalog/identity/rights checks appropriate to that source pass, and D1 evidence cites both the primary asset URI and its matching SHA-256. Optional sources that are actually acquired still need internally consistent asset manifests. This avoids making a paused service or an unused data family a mandatory dependency.

Every acquired product should have a reproducible intake manifest containing:
- provider, exact product/version/map code, retrieval time and original source URI;
- a stable asset URI (never a signed URL or bearer token), media type, byte size, exact retrieval timestamp and lowercase SHA-256 for each original/derived asset;
- original byte SHA-256, terms/licence/attribution and a product-specific redistribution decision;
- native CRS, spatial/temporal support, units, scale/offset, nodata/mask, variable/band names and grid-cell interpretation;
- upstream input/source overlap and known corrections or vintage changes;
- every transformation, software version, configuration and output hash;
- transformation-chain continuity: each step references the preceding step's output digest, and the final step output digest equals the hashed derived asset;
- study-boundary digest, model/configuration digest, frozen evaluation split and review evidence.

Preserve the original artifact and derive normalized/analysis-ready assets as separate outputs. A digest guarantees identity of bytes, not correctness of the underlying measurement or scientific interpretation.
The validator can check manifest consistency without downloading private or licensed assets. It does not independently verify remote bytes; the actual intake process must hash the exact acquired bytes before setting acquisition to `acquired`. A syntactically valid digest field alone is not proof the asset was downloaded or verified.

## How to expand globally without multiplying unvalidated models

Use the same profile schema and evidence gates for every region. Add a new regional profile only when it represents a meaningful change in climate, crop/production system, water constraint or local decision context—not merely because a map can be divided into another climate label.

A candidate expansion sequence is:

1. **Johannesburg / Highveld:** local water-resilience and food-network use case. Freeze a real study boundary, select a small crop/production profile, and acquire authorized local weather, soil/water and harvest records.
2. **Baixo Limpopo:** retain the existing WaPOR/CHIRPS irrigation-area water-productivity study as a complementary pilot with its own question and evidence gates. Do not pretend its current source stack is a direct crop-yield model or an independent validation set for Johannesburg.
3. **Western Cape winter-rainfall region:** a potential later contrast for winter-wet/summer-dry seasonality, selected only after the South African data and local review path are confirmed.
4. **Additional global regions:** choose one or more tropical, semi-arid, Mediterranean, humid-subtropical or temperate profiles based on real partner/site data and source readiness. Do not promise coverage of every Köppen class upfront.

For model transfer, compare at least three distinct baselines: a simple transparent climatology/water-balance reference; the selected regional model after local calibration; and a shared model applied to a held-out region without local tuning. Then separately measure transfer performance after local calibration. Use leave-one-region/site/season-out tests where the evidence volume permits and report the performance drop and uncertainty. Climate labels may be used to stratify errors, but never replace the actual covariates or prove that a model transfers.

The global success criterion is not the number of climate profiles. It is repeated evidence that the same data contract, baselines and validation methods reveal where a model transfers, where it fails, and what local measurements reduce error.

## Integration boundaries

- **Sol Atlas:** spatial/time visualization and source/uncertainty layers. It is not the physical-inventory authority or climate solver.
- **Symthaea:** advisory synthesis, contradiction detection and hypotheses about the next useful measurement. Model confidence is not scientific authority.
- **Symtropy:** later deterministic scenario comparisons only where its tested physical abstractions fit; it is not a validated crop-growth or hydrology solver.
- **Mycelix:** provenance, product/lot records, offers, reservations, dispatch/receipt and recall lineage. Model forecasts remain forecasts and must never create stock or confirm food safety.
- **Food-network maps:** consume read-only, explicitly versioned projections. Do not duplicate farmer identity, inventory, provenance or cooperative economics.

## Current maturity and limits

The profile is design-only. The candidate study boundary is unverified. No source raster or local field observations have been acquired or hashed by this profile; no local baseline, forecast, crop-suitability result, yield estimate or operational recommendation has been produced. A passing schema/contract test would show that the research plan is structurally consistent, not that the agricultural model is scientifically accurate.
