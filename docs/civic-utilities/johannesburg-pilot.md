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

The Rust observation contract in sol-atlas-core/src/utility_telemetry.rs explicitly separates source from transport. It rejects satellite-source records that claim metrics outside the supported remote-sensing set; satellite products require a version identifier. Delayed data retains separate observed and received timestamps.

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

Use Vaal Dam as the first repeatable satellite-observation case. Retrieve versioned Sentinel-2 optical and/or Sentinel-1 radar acquisitions from the Copernicus Data Space Ecosystem, preserve acquisition/product metadata, and compare a derived surface-water extent time series against dated DWS reservoir-level reports. Use a stable study-area polygon and fixed cloud/quality rules; show gaps when acquisitions are unusable instead of interpolating them away. The output is an evaluated surface-water indicator and evidence bundle—not a claim that imagery measures reservoir elevation or that this alone improves Johannesburg household supply.

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
2. **Read-only ingestion.** Ingest small historical exports and public imagery/products first. Reject malformed units/timestamps; quarantine unknown metric mappings. Preserve source data and a digest before any transformation.
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
