# Civic utility telemetry: Johannesburg reference pilot

**Status:** research/design proposal. This document does not claim a municipal deployment, data-sharing agreement, completed integration, or permission to connect to operational technology (OT).

## Objective

Build a read-only, evidence-preserving coordination layer that can help electricity and water operators detect cross-utility risks, evaluate maintenance priorities, and explain recommendations. Start in Johannesburg because water delivery depends on electrical pumping and because Johannesburg Water has publicly described an expanding instrumentation programme. Design the data contract for other cities from the outset.

The initial product is not an autonomous SCADA replacement. It is a replayable telemetry and digital-twin pipeline with explicit uncertainty, source lineage, freshness checks, and independent evaluation.

## What public evidence tells us

- Johannesburg Water's 23 July 2026 release describes smart hydraulic flow controls at outlets of 28 high-consumption reservoirs and towers, a smart pressure controller programme, and 225 acoustic noise loggers already installed. The same release said another 125 loggers were planned by October 2026 and that an additional 306 smart controllers were being installed toward a September 2026 target. **As of this document's research date, those target dates have passed; actual commissioning status must be confirmed with Johannesburg Water rather than inferred from the announcement.** Source: [Johannesburg Water, “accelerates smart technology rollout to reduce water losses” (23 July 2026)](https://www.johannesburgwater.co.za/blog/2026/07/23/johannesburg-water-accelerates-smart-technology-rollout-to-reduce-water-losses/).
- Johannesburg Water's 2024/25 annual report identifies non-revenue water and infrastructure maintenance as material risks and lists telemetry/remote monitoring and alternative power among operational risk treatments. Source: [Johannesburg Water Annual Report 2024/25](https://www.johannesburgwater.co.za/wp-content/uploads/2026/05/Annual-Report-2425.pdf).
- Johannesburg Water has also procured support for SCADA, systems, data warehousing, reporting, and associated network systems. That is a strong reason to integrate through approved read-only exports and existing vendor-supported interfaces, not to duplicate or bypass the utility's control infrastructure. Source: [Johannesburg Water awarded tenders](https://scm.johannesburgwater.co.za/supply-chain/tenders/awarded-tenders/) (entry JWOPS048/24).
- The City of Johannesburg's Corporate Geo-Informatics directorate publishes a Spatial Data Inventory containing data layers, custodians, contact details, formats and use limitations. Its 2024 inventory lists City Power layers including low-, medium- and high-voltage network infrastructure, substations and metering points. This gives us a concrete, permission-aware starting point for the GIS asset layer; it is not evidence that operational telemetry or restricted network files are openly downloadable. Sources: [City GIS portal and inventory](https://eservices.joburg.org.za/sites/cgismaps/Pages/default.aspx), [Spatial Data Inventory 2024](https://eservices.joburg.org.za/sites/cgismaps/SiteAssets/Pages/default/Spatial%20Data%20Inventory%202024.pdf), [GIS data request process](https://eservices.joburg.org.za/corporate-geo-informatics/overview).
- A South African study has assessed Sentinel-2 and Landsat 8 for monitoring Vaal Dam, a major water resource supplying Johannesburg and surrounding areas. Source: [Assessing high-resolution satellite monitoring of Vaal Dam](https://doi.org/10.1016/j.rsase.2021.100521).
- The Copernicus Data Space Ecosystem exposes a STAC 1.1 catalogue; its `sentinel-2-l2a` collection provides Level-2A bottom-of-atmosphere surface-reflectance products and associated scene metadata. Source: [CDSE STAC product catalogue](https://documentation.dataspace.copernicus.eu/APIs/STAC.html), [Sentinel-2 data collection](https://dataspace.copernicus.eu/data-collections/copernicus-sentinel-missions/sentinel-2).
- USGS publishes the LandsatLook STAC API and identifies the Landsat Collection 2 Level-2 surface-reflectance collection as `landsat-c2l2-sr`. It is a provider collection and may contain Landsat 8 and Landsat 9 records; scene identifiers/platform metadata must be checked before labeling a record Landsat 8. Sources: [USGS STAC documentation](https://www.usgs.gov/landsat-missions/spatiotemporal-asset-catalog-stac), [LandsatLook STAC API](https://landsatlook.usgs.gov/stac-server/api.html).
- USGS specifies that Landsat Collection 2 surface-reflectance digital numbers use a scale factor of 0.0000275 and additive offset of -0.2; DN 0 is fill. Correct scaling and QA_PIXEL/QA_RADSAT masking are prerequisites to index analysis. Sources: [USGS scale-factor FAQ](https://www.usgs.gov/faqs/how-do-i-use-a-scale-factor-landsat-level-2-science-products), [Landsat Collection 2 QA bands](https://www.usgs.gov/landsat-missions/landsat-collection-2-quality-assessment-bands).
- Sentinel-2 Level-2A Scene Classification Layer has classes for no-data, saturated/defective, dark-area pixels, cloud shadow, vegetation, non-vegetated surface, water, unclassified, medium/high-probability cloud, thin cirrus, and snow/ice. The exact product baseline and data-mask still need to be retained with each scene. Source: [Copernicus Sentinel-2 Products Specification Document, Level-2 classification map](https://sentinels.copernicus.eu/documents/d/sentinel/s2-pdgs-cs-di-psd-v15-0).
- NASA GIBS exposes pre-rendered visualizations as web-map tiles. It is useful for the Sol-Atlas browse/display layer, but tile URL construction is not a substitute for retrieving analysis-ready bands and applying QA masks to produce a measured surface-water estimate. Source: [NASA GIBS overview](https://nasa-gibs.github.io/gibs-api-docs/).
- Sentinel-1 synthetic-aperture radar supports flood-extent assessment in cloud and darkness; its acquisition timing and resolution still limit which decisions it can support. Source: [European Space Agency: Sentinel-1 emergency response](https://www.esa.int/Applications/Observing_the_Earth/Copernicus/Sentinel-1/Emergency_response).
- South Africa's National Geo-spatial Information services include a national Water Areas layer useful for inventory and spatial context. Treat it as a reference dataset, not a live telemetry feed. Source: [NGI National Datasets — Water Areas](https://nspdr.dlrrd.gov.za/server/rest/services/NationalDatasets/Hydrology/MapServer/3).
- The Department of Water and Sanitation reported the Integrated Vaal River System at 97.1% combined storage and Vaal Dam at 95.4% on 6 October 2026. That weekly report is useful as a dated external comparison point, not as an instantaneous operational sensor. Source: [DWS, Vaal Dam reservoir levels (6 October 2026)](https://www.gov.za/news/media-statements/water-and-sanitation-vaal-dam-reservoir-levels-06-oct-2026).
- Sentinel Hub within the Copernicus Data Space Ecosystem exposes catalog, processing and statistical interfaces for satellite imagery, with free use within published quotas; requests require versioned product details and any necessary authentication. This is a concrete public-data ingestion route for a prototype. Sources: [Copernicus Data Space Sentinel Hub](https://dataspace.copernicus.eu/ecosystem/services/sentinel-hub), [Sentinel Hub API overview](https://documentation.dataspace.copernicus.eu/APIs/SentinelHub/Overview.html), [API beginner's guide](https://documentation.dataspace.copernicus.eu/APIs/SentinelHub/UserGuides/BeginnersGuide.html).

## Three separate data planes

| Plane | Examples | Proper use | Not a substitute for |
|---|---|---|---|
| **In-situ operational telemetry** | SCADA read-only exports, pump status, pressure, flow, reservoir level, power draw, feeder/outage events, acoustic leak events | Minute-to-minute state estimation and incident correlation, subject to timestamp, calibration, network and sensor-quality checks | Independent field verification or engineering judgment |
| **Satellite remote sensing** | Optical imagery, radar, thermal observations, satellite-derived rainfall/soil-moisture products | Reservoir/surface-water extent, inundation, selected vegetation/land-cover changes, environmental context, and longer-horizon risk indicators | Pipe pressure, flow-meter values, drinking-water potability, breaker state, or sub-minute protection |
| **Satellite communications** | Existing site sensor data relayed over a satellite link, with store-and-forward delivery | Connectivity for remote/unreliable-network sites where justified by a cost, latency, coverage and security assessment | Direct physical observation; the transport path is not the observation source |

The Rust observation contract in `sol-atlas-core/src/utility_telemetry.rs` explicitly separates observation source from transport. It validates metric/unit compatibility, timestamps, source version, derived-data lineage and quality metadata. Satellite-derived values can be represented as observations only when explicitly classified as derived estimates; this schema does not itself prove a product or sensor reading is authentic.

### Avoid duplicated satellite providers

The canonical provider-specific STAC discovery implementation belongs in **Mycelix Earth**, where the existing evidence model and `EarthProvider` interface already live. [Draft PR #3776](https://github.com/Luminous-Dynamics/luminous-dynamics/pull/3776) adds bounded read-only catalogue search for Sentinel-2 Level-2A and Landsat 8 Collection 2 Level-2, plus explicit synthetic-fixture labeling. Its scene search has not yet been exercised against a live catalogue; the exact-head `test-stac` CI run failed, and its job logs were unavailable through the current integration, so the cause remains unknown. Raster download/QA/index processing remains incomplete. Do not treat this provider as passing validation until the failure is triaged and the exact current head passes.

Sol-Atlas should not maintain a competing STAC parser. This PR instead keeps the responsibilities distinct:
- `sol-atlas-core/src/gibs.rs` provides configured NASA GIBS visualization tile URL construction, now wired into the core crate with validated XYZ coordinates in `tiles.rs`.
- `sol-atlas-core/src/utility_telemetry.rs` defines the common read-only observation/evidence contract.
- `sol-atlas-core/src/surface_water.rs` implements deterministic NDWI/MNDWI calculation and area accounting over already aligned reflectance samples whose product-specific QA mask has already been applied. It withholds estimates below the declared usable-pixel coverage threshold and can convert a qualified result into a provenance-bearing derived observation. The reported area is **classified area within valid pixels only**: the implementation never extrapolates masked/cloudy/invalid regions, and partial coverage is recorded in both a quality flag and processing metadata.
- `sol-atlas-core/src/raster_quality.rs` contains pure, versioned QA/scaling helpers: Sentinel-2 SCL/data-mask classification, Landsat 8/9 Collection 2 QA_PIXEL and QA_RADSAT classification for the bands used by NDWI/MNDWI, and the Landsat Collection 2 surface-reflectance scale/offset. These helpers have unit tests, but they do not yet read raster files or establish that an input product is correctly identified.
- The end-to-end raster pipeline is still outstanding: authenticated/allowlisted asset retrieval, exact input hashing, COG/raster reads, Sentinel product-baseline-specific reflectance quantification/offset handling, band alignment/resampling, reservoir polygon clipping, threshold calibration, cross-sensor evaluation and independent evidence review.

### Quality-mask contract (implemented pure functions; no raster IO yet)

- **Sentinel-2 L2A:** use `classify_sentinel2_l2a_scl` with the pixel's SCL value and actual data-mask bit. No-data, defective, dark-area, unclassified, shadow, cloud/cirrus and snow classes are masked conservatively. SCL water pixels remain eligible for the index; SCL class 6 is not counted as independent validation.
- **Landsat 8 Collection 2:** use `classify_landsat8_c2_pixel` with both QA_PIXEL and QA_RADSAT. The chosen index determines which band saturation bits are required (Band 3+5 for NDWI; Band 3+6 for MNDWI). Water bit 7 is deliberately retained; cloud, dilated cloud, cirrus, shadow, snow and terrain-occlusion conditions are excluded. The default policy excludes medium/reserved and high confidence conditions; a high-only policy exists but requires documented comparative justification.
- **Landsat scaling:** `scale_landsat_c2_surface_reflectance` applies `DN * 0.0000275 - 0.2` and returns no value for fill DN 0. It does not substitute for QA checks or silently clamp negative scaled values.
- Both helpers have policy-version constants. The caller must record the policy/product baseline in the derived observation, align all bands and QA rasters to the same documented target grid, then supply only valid samples to the area estimator. No production raster is currently downloaded or processed by these helpers.

## Proposed system boundary

    Approved read-only exports + public satellite products + field inspections
                                    |
                                    v
            Canonical observation contract and quality validation
                                    |
                 +------------------+------------------+
                 |                  |                  |
                 v                  v                  v
           Mycelix evidence     Symtropy model     Sol-Atlas view
           & governance         & replay           & spatial context
                 |                  |
                 +--------+---------+
                          v
             Symthaea hypotheses / risk prioritization
                          |
                          v
           Human-reviewed recommendations and work tickets

This is a target integration shape, not a claim that these integrations already exist. The first adapter should output normalized observations only. Do not add an SCADA write path, actuator credentials, or a direct route from Symthaea to PLCs, relays, breakers, pumps, or valves.

### Responsibility separation

- **Sol-Atlas:** spatial infrastructure context, observation provenance, map layers and visual comparison of observed conditions versus explicitly labelled estimates/scenarios.
- **Mycelix:** signed/attributable receipts, authorized stewardship, governance approvals, maintenance evidence and an auditable trail of what was known and when. Do not put raw high-resolution SCADA values or sensitive asset topology into a public DHT by default.
- **Symtropy:** calibrated, uncertainty-aware hydraulic/electrical scenario models, deterministic replay where supported, and fault-injection experiments. Model fitness must be measured against independent observations.
- **Symthaea:** anomaly hypotheses, cross-domain correlation, forecast/risk ranking, and readable reasons with uncertainty. It proposes; it does not grant itself authority.
- **Satiate/satellite tooling:** no repository or integration named “Satiate” was located during this research pass. The current design therefore treats satellite-derived products and satellite communication as separate, replaceable adapters. If “Satiate” is a distinct system name, its actual interface and semantics remain unverified.

## First pilot: two linked workstreams

### Public-data prototype (can start before a municipal partnership)

Use Vaal Dam as the first repeatable optical Earth-observation case. Search Sentinel-2 Level-2A through Copernicus Data Space STAC and Landsat 8 Collection 2 Level-2 surface reflectance through USGS LandsatLook STAC. Preserve complete item/product metadata, asset IDs, acquisition time, processing/version identifiers and cloud/QA status. The USGS Landsat collection can return Landsat 8 and 9 items, so filter by platform/item ID and never infer mission from the collection ID alone.

Use an agreed reservoir/catchment polygon—not a rough bounding box—for clip and area estimation. Apply product-specific scale/offset to raw raster integers; use Sentinel-2 scene classification/cloud QA and Landsat QA_PIXEL masks; reproject/resample the required bands to a documented common grid; then compute NDWI and/or MNDWI with a threshold calibrated on independently labelled dates/shoreline conditions. Sentinel-2 SWIR band B11 is 20 m while its green band B03 is 10 m, so MNDWI must document a deliberate target-grid strategy. Landsat Collection 2 surface reflectance is 30 m. Do not compare a 10 m estimate and a 30 m estimate as if they had identical spatial support.

Compare the derived, quality-qualified surface-water area series with dated DWS reservoir-level reports as a consistency check, not ground truth: lake surface area and water level are related by local bathymetry and shoreline shape, not by a fixed one-to-one function. Keep gaps when scenes are unusable; do not interpolate missing acquisitions into observations.

This phase tests data discovery, processing reproducibility, acquisition-time handling, quality flags, geospatial alignment, lineage and Sol-Atlas visualization without requiring access to restricted utility data.

### Operational shadow pilot (requires a utility agreement)

Do not select a neighbourhood by guesswork. Ask Johannesburg Water and City Power to identify a candidate water-pressure zone with its upstream pumping/electricity dependencies, an accountable operational owner, approved data access, adequate instrumentation, and useful incident history. Include Rand Water/upstream-system dependencies only where the data agreement and scope permit it. Start with read-only historical exports, then shadow mode; do not connect to control systems for actuation.

### Minimum data contract

Each observation records:

- a stable observation ID and subject ID/type;
- domain and metric, with an explicit canonical unit/value shape;
- source observation/acquisition time and ingestion receipt time;
- observation source separately from data transport;
- source/product/firmware version and optional SHA-256 evidence digest;
- quality flags, optional confidence and uncertainty, and optional coordinates;
- upstream observation IDs when a value is model-derived.

A valid structure is not proof that the reading is authentic, calibrated, or physically true. The digest checker validates only hash syntax; the pipeline must separately verify captured bytes/signatures when a trustworthy source supports that. Satellite footprints and classifications need acquisition/product identifiers, spatial resolution, cloud/quality flags where applicable, and an explicit asset/zone association.

### Sequence and evidence gates

1. **Access and asset map.** Agree on owner, legal basis, permitted fields, retention, security classification, source clocks, calibration metadata, unit mappings and a named incident reviewer. Keep sensitive network topology private.
2. **Read-only ingestion.** Ingest small historical exports and public imagery/products first. Reject malformed units/timestamps and quarantine unknown metric mappings. If a source observation timestamp is later than its receipt time, reject it unless the record carries `clock_skew_suspected`; preserve the reported source time and classify it as `FutureObserved` until source-clock integrity is established. Preserve source data and a digest before any transformation.
3. **Baseline and replay.** Build an incident corpus from verified outages, pump trips, pressure drops, leak alarms, planned maintenance and normal periods. Mark synthetic scenarios separately. Do not count a generated scenario as a real incident.
4. **Shadow mode.** Compare recommendations with existing operator actions and independently adjudicated outcomes without actuation. Measure false alarms, missed incidents, lead time, calibration, data staleness and operational usefulness.
5. **Bounded automation only after a safety case.** Any future setpoint or switching automation requires an explicitly authorized control boundary, equipment-specific hazard analysis, deterministic interlocks, independent state checks, fail-safe fallback, manual override, tested recovery, change management and qualified engineering sign-off. This proposal does not authorize that phase.

## Evaluation measures (baseline first; no promised gains)

| Measure | Definition / guard against gaming |
|---|---|
| Fault precision and recall | Evaluate by fault class against labelled incidents; report false alarms and missed events, not one aggregate accuracy |
| Warning lead time | Use the time of first actionable warning before the independently recorded incident; no retrospective timestamp leakage |
| Sensor / source health | Missingness, clock drift, calibration expiry, stale-data rate and source coverage by zone |
| Water balance / non-revenue water | Use accepted zone boundaries and meter/bulk-flow reconciliation; distinguish measured loss from estimation uncertainty |
| Pumping energy intensity | kWh per verified volume delivered, with quality and service-level constraints |
| Restoration and repair | Incident-to-triage, dispatch, arrival, repair and service-restoration intervals, stratified by incident class |
| Equity and service outcomes | Disaggregate service interruption duration and recovery by area/service population while applying privacy protections |
| Evidence completeness | Percentage of recommendations traceable to exact input IDs, source/product version, model version, uncertainty, reviewer, and outcome |
| Safety invariants | Zero unauthorized actuation; explicit adversarial, data-corruption, stale-input, replay, comms-loss and failover tests |

Agree the metric definitions, baseline window, exclusion rules, and thresholds with the utility before evaluation.

## Satellite-specific guardrails

- Use radar for cloud/night-capable flood and inundation context; use optical products for suitable cloud-free surface-water and vegetation observations; use thermal/metrology products only for measures supported by the specific product.
- Track acquisition time, processing time, pixel size, georegistration uncertainty, masks/quality bands, revisit interval and missing acquisitions.
- Do not infer potable-water quality from satellite imagery. Use accredited sampling/lab data and approved in-situ probes.
- Do not infer a small underground pipe leak directly from a coarse satellite pixel. Satellite indicators may prioritize investigation but a field crew, acoustic logger, pressure/flow evidence or other validated measurement must corroborate the fault.
- Treat a satellite communications outage as a telemetry freshness problem, not evidence that the physical asset failed. Store-and-forward records must remain visibly late.

## Security and governance baseline

Keep the telemetry gateway separate from OT networks and use vendor-supported, least-privilege read-only integration initially. Separate recommendation identities from operator and actuation identities. Validate authorization, freshness, asset scope, configuration version, and approval state at any future boundary. Redact or aggregate sensitive location data in public views.

Use [NIST SP 800-82 Rev. 3: Guide to Operational Technology Security](https://csrc.nist.gov/pubs/sp/800/82/r3/final) and the [IEC 62443 series](https://www.iec.ch/blog/understanding-iec-62443) as engineering references, not claims of certification. For South African deployment planning, legal review should include the [Water Services Act 108 of 1997](https://www.gov.za/documents/water-services-act) and the [Municipal Systems Act 32 of 2000](https://www.gov.za/documents/local-government-municipal-systems-act), as amended, alongside current electricity-sector and privacy requirements.

## Immediate implementation work

1. Review the Rust observation contract and unit tests added alongside this document.
2. Add fixtures from a documented source with licensing, timestamps and expected units; keep synthetic fixtures visibly labelled.
3. Build strict parsers/adapters one data source at a time, with source-specific age and quality policy.
4. Add deterministic replay plus an independent evaluator before optimizing any model.
5. Only after the ingestion/evidence path is reproducible, add a Sol-Atlas layer for observations versus modelled risk.

**Acceptance criterion for this stage:** the data contract compiles and its tests pass; invalid or epistemically mismatched measurements fail validation; observation time is preserved through delayed delivery; no command/actuation interface is added. No deployment or field-performance claim is made by this design document.
