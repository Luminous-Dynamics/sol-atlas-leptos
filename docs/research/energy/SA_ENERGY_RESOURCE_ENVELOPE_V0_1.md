# South African Multi-Resource Energy Site Evidence Envelope v0.1

**Status:** research specification / screening contract  
**Date:** 2026-11-09  
**Owner:** Sol-Atlas spatial and evidence plane  
**Parent work:** [ENERGY-RESOURCE-SITE-001 #662](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/662)

## 1. Decision problem

A resource map is not a power-plant feasibility study.

For a spatially bounded candidate region, this work records what is known about the natural resource and the surrounding conditions needed to turn it into a constructible, connectable, maintainable energy service. It must preserve the difference between:

1. **Resource signal** — an observation or model indicates solar, wind, hydraulic, geothermal or marine resource.
2. **Technical envelope** — a named conversion technology could plausibly use that resource under declared operating conditions.
3. **Site eligibility** — land/sea use, water, ecology, geology, rights and environmental constraints have been screened.
4. **Deliverability** — the required grid, cable, access, port, water or fuel interfaces have their own evidence.
5. **Buildability and supportability** — construction, supply-chain, workforce, repair, spares and end-of-life routes have been assessed.
6. **Delivered service** — actual electricity, heat, cooling, water or industrial service is evaluated against time-dependent demand.

These are separate gates, not dimensions to hide inside a single site score. A site can have excellent wind resource and still be unsuitable due to seabed, cable, port, environmental, rights or grid constraints.

This specification does not implement a resource model, predict plant output, rank parcels or authorize a project.

## 2. Ownership and integration

| System | Owns | This profile supplies |
|---|---|---|
| Sol-Atlas #450 | Layer provenance, space/time scope, uncertainty and availability | Resource-specific source and evidence metadata |
| Sol-Atlas #293 | Generation and conversion portfolio evaluation | Source-qualified resource input and site constraints |
| Sol-Atlas #519 | Producer-side grid-connection evidence | Candidate site/POC references; never inferred export capacity |
| Sol-Atlas #116 | Spatial/economic resilience, network dependence and service floors | Site access, spatial constraints and evidence coverage |
| Sol-Atlas #308 / #312 / #332 / #352 | Exact dispatch, uncertainty and network-constrained operation | Admitted time-series and network boundary metadata |
| Symthaea #8496 | Candidate architecture generation and experiment selection | Evidence-bounded resource envelopes and unknowns |
| Symtropy #2721 | Dynamic twin execution, fault replay and trajectory validation | Versioned component/site parameters and boundary declarations |
| Mycelix | Provenance, approval/review records and economic coordination | Content identity and review receipts, not scientific truth |

Do not put a second GIS, dispatch optimizer, thermal solver, hydrology model or connection-rights database into this layer.

## 3. South African source discovery register

The following are **candidate sources**, not yet admitted source snapshots. Before ingestion, pin exact edition, artifact bytes, licence, date, spatial and temporal extent, units, CRS and known limitations.

| Resource | Source / evidence lead | Appropriate initial use | Important limitation |
|---|---|---|---|
| Solar | [Global Solar Atlas: South Africa](https://globalsolaratlas.info/download/south-africa); [data outputs](https://dev.globalsolaratlas.info/support/data-outputs) | GHI, DNI, diffuse irradiation, tilted irradiation, optimum tilt and PVOUT screening | Country-download layers are described as last updated in 2019 (data period through 2018), while world GIS layers are described as updated in 2026 (period through 2025). Do not silently join them as one release. Global grid cells are not parcel-level truth. |
| Wind | [Global Wind Atlas via IEA Africa GIS catalogue](https://www.iea.org/data-and-statistics/data-tools/africa-gis-catalogue-for-energy-planning) | Preliminary onshore/offshore wind screening and GIS discovery | Pin exact edition, model chain, hub height, spatial resolution, offshore coverage and uncertainty. Mean wind speed alone is not annual energy. |
| Hydropower | [South African Department of Electricity and Energy — Hydro-Power](https://www.dee.gov.za/hydro-power/) | Regional discovery, including historic small-hydro leads in Eastern Cape and KwaZulu-Natal | The department's cited national baseline is from 2002. It is not current site-level streamflow, ecological-flow, water-right or head evidence. |
| Geothermal | [Council for Geoscience — Geothermal Energy Potential of South Africa](https://www.geoscience.org.za/project/geothermal-energy-potential-of-south-africa-project/) | Low-enthalpy exploration and magnetotelluric target context near known hot springs | Current work indicates exploration of possible hot-water reservoirs and fluid pathways, not demonstrated power-plant resources. |
| Offshore wind | [World Bank — A Strategic Framework for Offshore Wind Development in South Africa](https://documents.worldbank.org/en/publication/documents-reports/documentdetail/099091026151019349) | Sector, policy, port, transmission and institutional context | Document dated 2026-09-30 and disclosed 2026-11-06; it is not a site-specific wind-yield certification or connection allocation. |
| Ocean current | [CSIR Agulhas Current study](https://researchspace.csir.co.za/bitstreams/fcffd13e-9db3-4b60-a306-74c34ed989a7/download) | A source-bounded marine-current research case; compare remote/predictive products with ADCP observations | The study reports 1.34 m/s and 1.59 m/s mean current velocities at 30 m depth at two studied locations. These values cannot be extrapolated coast-wide or translated directly into commercial turbine output. |
| Waves | [Copernicus Marine](https://marine.copernicus.eu/) analysis, reanalysis and forecast products | Wave height, period, direction and spectral-resource screening | Record product ID, hindcast/forecast status, temporal coverage, grid resolution, temporal step and validation source. |
| Cross-resource discovery | [IRENA Global Atlas](https://www.irena.org/Energy-Transition/Project-Facilitation/Renewable-potential-assessment/Global-Atlas) | Dataset discovery across solar, wind, bioenergy, geothermal, hydropower and marine energy | It aggregates many institutions and datasets; there is no single common truth surface or common uncertainty model. |
| Cross-resource discovery | [IEA Africa GIS Catalogue](https://www.iea.org/data-and-statistics/data-tools/africa-gis-catalogue-for-energy-planning) | Find regional renewable layers and metadata | Catalogued sources have different editions, resolutions and reference periods; catalogue presence is not validation. |

South African supplementary sources to investigate include NTCSA/Eskom connection and transmission data; DWS hydrology and water authorisations; DFFE marine spatial planning and environmental layers; SANBI protected-area and biodiversity products; SAWS weather observations; CSIR and university field studies; and Council for Geoscience subsurface data. Each dataset must pass source admission individually.

### Initial geographic research profiles

These are **research profiles**, not selected or approved project sites:

- **Eastern Cape / KwaZulu-Natal:** screen small hydro against modern flow-duration, head, environmental-flow, water-right and access evidence. The historical regional signal only determines where to look.
- **Known hot-spring / low-enthalpy contexts:** evaluate direct-use heat and heat-pump service first. Do not upgrade a low-temperature heat opportunity into an electricity resource without measured temperature, depth, flow and reservoir evidence.
- **South African offshore wind:** combine source-qualified wind and metocean layers with seabed, bathymetry, cable corridors, ports, maintenance access and #519 grid evidence.
- **Agulhas Current:** keep the CSIR's sampled locations and measurement depths intact. Test representativeness and temporal persistence before considering energy extraction.
- **High solar-resource regions:** pair resource evidence with ambient temperature, soiling/dust, water availability, land, grid and demand. PVOUT screening is not a net-delivered-power guarantee.

## 4. Resource-specific minimum variables

### Solar PV and CSP

- GHI, DNI and diffuse irradiance; plane-of-array conversion method and orientation.
- Time series or representative profiles with temporal coverage, timezone and missingness.
- Ambient temperature, wind/cooling context, soiling/dust, shading and terrain horizons where evidenced.
- PV module / inverter power curve and losses must be separately sourced; CSP requires DNI and thermal-cycle / storage boundary.
- Report gross DC, net AC, auxiliaries, curtailment and useful service separately.

### Onshore and offshore wind

- Hub-height wind-speed distribution, direction, air density and time profile.
- Power-density calculation, turbine power curve and reference air density.
- Wind shear, turbulence, extreme gusts, wakes, terrain complexity and icing/corrosion where applicable.
- Offshore: significant wave height, period/direction, currents, water depth, seabed/geotechnical evidence, cable route, port/vessel access and weather windows.
- Mean speed or theoretical resource density must not be reported as bankable annual generation.

### Hydropower and pumped hydro

- River flow series, flow-duration curve, seasonal drought/flood conditions and measurement station representativeness.
- Gross/net head, penstock / conveyance loss, turbine operating range and environmental-flow requirement.
- Sediment, debris, fish/ecological passage, flood safety, water quality and water entitlement.
- Separate run-of-river generation, existing-dam retrofit, water-network energy recovery, new reservoir and pumped-storage cases.
- Pumped hydro is storage: report pumping input, generation output, round-trip efficiency, reservoir constraints and ecological / water impacts. Do not count stored water as a new primary energy resource.

### Geothermal

- Temperature versus depth, gradient / heat-flow observations, borehole logs and measurement provenance.
- Geology, permeability / fracture connectivity, fluid chemistry, pressure, sustainable flow and reinjection constraints where measured.
- Separate direct-use thermal service, heat-pump service and electricity-generation potential.
- Do not infer deep reservoir temperature or deliverable flow from a surface hot spring alone.
- Drilling, casing, cement, high-temperature materials and well-life assumptions are independent engineering and cost inputs.

### Marine energy

- Wave: significant wave height, energy period, direction, spectral distribution and temporal persistence.
- Current: velocity profile by depth, direction, turbulence, shear, extremes, persistence and ADCP calibration.
- Tidal stream and tidal range: separate current resource from tidal elevation/range and local hydrodynamics.
- OTEC: surface/deep temperature differential, depth-to-resource, intake/pipeline and pumping burden.
- Offshore buildability: bathymetry, seabed geology, corrosion/biofouling, mooring, export cable, ports, vessel access, maintenance windows, fisheries, navigation and marine habitats.
- Do not assume that coastal proximity or high current speed implies feasible extraction.

## 5. Site constraints and buildability

A resource profile must retain independent evidence for:

- Land/sea tenure, land-use designation, community and customary rights, heritage and public consultation.
- Protected areas, biodiversity, migratory routes, fisheries, navigation, noise, sediment, water quality and ecological flows.
- Water abstraction, consumptive use, cooling, reinjection, treatment and seasonal availability.
- Geotechnical conditions, seismicity, slope, flood, erosion, bathymetry and seabed conditions.
- Grid voltage, transmission route, substation/feeder, requested import/export, connection status and curtailment terms. Delegate adjudication to #519.
- Road, bridge, rail, port, crane, heavy-lift, vessel, laydown and construction access.
- Weather / metocean construction windows and safe access for inspection and repair.
- Regional manufacturing capacity, material availability, supplier lead times, workforce, training, spare parts, local repair and end-of-life recycling.
- Lifecycle cost and embodied materials with currency-year, location and uncertainty.

A technically possible design may still be unbuildable or non-maintainable in a particular region. Treat this as an engineering objective, not merely a project-management note.

## 6. Proposed interchange contract

A future `EnergyResourceEvidenceV1` record should bind at minimum:

- stable opaque feature / site ID and geometry digest;
- CRS, geometry type, spatial coverage, resolution and any resampling;
- resource family, physical variable, unit, height/depth/reference frame;
- source ID, publisher, exact release/version, retrieval timestamp, valid period and publication date;
- raw source digest where locally stored, transformations and derived-feature lineage;
- evidence class: measurement, validated reanalysis/model, unvalidated model, compiled secondary source, scenario assumption, vendor/developer assertion or unknown;
- uncertainty, quality flags, temporal completeness, station representativeness and known bias;
- licence, attribution and redistribution permission;
- availability: `available`, `partial_coverage`, `stale`, `withheld`, `unavailable`, `conflicting`, `needs_ground_measurement` or `unknown`.

A digest provides content identity only. It does not authenticate the publisher or prove that a value is true.

Suggested readiness states are `screening_only`, `needs_evidence`, `constraints_unresolved`, `connection_unresolved`, `pre_feasibility_reviewable` and `not_eligible_under_declared_constraints`. These describe evidence status, not a project's formal approval.

## 7. Benchmark protocol

### Data ingestion and provenance

- 100% of admitted artifacts retain exact source/release, licence, CRS, units, spatial and temporal scope, retrieval time, evidence class and availability.
- 0 missing / unavailable / outside-coverage values silently become numeric zero.
- 0 hidden unit conversions, unlabelled interpolation or mixed-release joins.
- Re-import pinned golden source snapshots and independently compare metadata and selected raster/series summaries.
- Record exact transformations and output digests; identical inputs and configuration must produce identical manifests.

### Physical resource validation

For each resource family compare model/reanalysis values to independent and co-located evidence where available: weather stations, wind masts, buoys, ADCPs, stream gauges, boreholes or reservoir measurements.

Report sample size, coverage, bias, MAE, RMSE, P50/P90 absolute error, seasonal/regime stratification and uncertainty-interval coverage. Use spatial and temporal holdouts. Freeze acceptance tolerances by variable, application and measurement quality **before** evaluating the held-out set; there is no universal threshold shared by solar, wind, flow, geothermal and marine currents.

### Negative controls

The contract verifier must reject or return indeterminate for wrong CRS, invalid units, stale source, absent coverage, unsupported resampling, missing licence, impossible flow/head, missing environmental-flow evidence, unresolved rights, unknown seabed/geology, nonrepresentative station, expired source, missing connection evidence or firm-capacity claims unsupported by operator evidence.

Target zero false acceptance on a finite, frozen hostile test panel. This is a contract-verifier target, not zero real-world project risk.

### Industry comparison

- Compare resource discovery / metadata coverage against IRENA Global Atlas, Global Solar Atlas, Global Wind Atlas and IEA Africa GIS Catalogue.
- Compare energy-system design and dispatch only against matched REopt, SAM, PyPSA, GenX or other reference formulations.
- Compare engineering cost only with matching technology, scale, site, currency year, scope, capacity factor, storage, interconnection and financing assumptions.
- Never compare a raster resource-potential value directly with project LCOE, delivered cost or firm capacity.
- Preserve the difference between a resource-screening tool, optimization model, digital twin, field measurement and bankable engineering study.

## 8. Delivery sequence

1. **Source register:** pin candidate dataset IDs, releases, licences, CRS, resolutions, reference periods and access status.
2. **Golden sample pack:** admit one small artifact or published table for solar, wind, small hydro, low-enthalpy geothermal and marine energy; where raw artifacts are unavailable, record a source blocker rather than fabricate values.
3. **Schema and importer:** define strict typed Rust metadata and source-state handling in `sol-atlas-core`; renderer remains a consumer.
4. **Independent intake audit:** verify metadata, units, coordinate transforms, time alignment and selected derived values against a second parser / hand-calculated fixture.
5. **Spatial constraint crosswalk:** bind rights, environment, water, access, network and construction evidence with explicit unknown states.
6. **Downstream hand-off:** create evidence-bounded profiles for #293, #519, Symthaea #8496 and Symtropy #2721.
7. **Only after screening:** select a specific candidate site for independent ground measurement, pre-feasibility and qualified engineering review.

## 9. Claim ceiling

This v0.1 is a research specification. It does not establish that any named site is suitable, permitted, interconnected, bankable, safe or ready to construct. It does not forecast plant output without validated time-series and technology-specific conversion models. No spatial score substitutes for water rights, environmental authorisation, community consent, grid connection, geotechnical investigation, metocean measurement or licensed engineering approval.

All source-derived claims remain at the evidence level of their underlying source. No synthetic or modelled value may be presented as an observation.


## 10. Source-register corrections and admission cautions (checked 2026-11-09)

The machine-readable discovery register is `SA_ENERGY_RESOURCE_SOURCE_REGISTER_V0_1.json`, validated against `schemas/energy-resource-source-registry-v0.1.schema.json`. Its entries are deliberately not admitted datasets: all have `raw_snapshot_ingested=false` and `snapshot_digest=null`.

Important source-status corrections:

- **Global Wind Atlas:** use GWA 4.0 as the current global screening candidate. DTU's 2025-06-18 announcement describes a new microscale modelling step at 250 m across country boundaries and offshore EEZs. That is a model-grid description, not a claim of 250 m measurement accuracy. Pin the exact downloaded object, model metadata, release and validation coverage. The older IEA Africa GIS catalogue entry may still carry older release metadata; do not let the catalogue listing overwrite the direct GWA 4.0 version record.
- **WASA:** the CSAG/UCT time-series portal notice dated 2026-09-01 says several hard-drive failures corrupted the portal's 30-year WRF time series and regeneration was underway. The notice says WASA3 reports remain available. Until the portal is rechecked and the restored files are downloaded and hashed, the time-series product is `temporarily_unavailable`. A portal status notice is not a substitute dataset.
- **Agulhas Current:** Meyer, Braby, Krug and Backeberg's ResearchSpace record dates the publication to 2017-04. Its two-site ADCP/model comparison is a historical reference case, not a 2026 campaign or coast-wide current atlas. Preserve the two sampled locations, depths and acquisition windows separately if the full record is admitted.
- **Copernicus waves:** `GLOBAL_MULTIYEAR_WAV_001_032` (WAVERYS) is the historical wave reanalysis candidate at 0.2 degrees with hourly/monthly products and the product page reporting coverage through 2026-08-31. `GLOBAL_ANALYSISFORECAST_WAV_001_027` is a separate 0.083-degree hourly analysis/forecast product. Forecast lead times must never be mixed into historical climate statistics.
- **Copernicus ocean physics:** `GLOBAL_ANALYSISFORECAST_PHY_001_024` can provide coarse global velocity profiles, but its grid and 50-level representation require matched ADCP validation and do not establish turbine-depth currents at a particular site.
- **Solar Atlas:** the South Africa country/region package last updated 2019 (data through 2018) is not the same data release as the global GIS layers last updated 2026 (data through 2025). Keep separate source IDs, file manifests, periods, licences and hashes.
- **Hydro / geothermal:** the government small-hydro lead cites a 2002 baseline; the CGS page describes exploration context. Neither currently supplies an admitted modern site-level flow-duration, head, borehole temperature-depth or sustainable reservoir-flow dataset.

No resource source should be promoted to `observed_measurement` solely because a report contains a number. The epistemic class belongs to the underlying evidence object: a peer-reviewed paper may report field observations, but the raw measurement series, methods, location, timebase and uncertainty still need to be recovered before those observations become a reusable site-level dataset.

## 11. Immediate source-ingress work order

1. Recheck WASA portal status on the day of any attempted acquisition; record the response and time.
2. Download one small, licensed Global Wind Atlas 4.0 South African tile and one matching GSA global GIS tile. Preserve original bytes and metadata without reprojecting or resampling.
3. Obtain a DWS gauge time series with station metadata, flow measurement quality and data-use terms for a candidate hydro catchment; do not select a site solely from the old 2002 regional lead.
4. Request CGS access to the exact geothermal datasets (borehole, geochemical and magnetotelluric observations) and record withheld/unavailable separately.
5. Acquire one Copernicus Marine wave and current/physics subset with the Quality Information Document; pair it with available buoy / ADCP observations and do not treat reanalysis as ground truth.
6. Only after source admission, run independent parser checks, held-out calibration and spatial/temporal representativeness tests.


## 12. WASA product separation and wind-model validation references (checked 2026-11-09)
The WASA time-series portal outage must not be generalized to all WASA products.

The separate CSIR/DTU WASAData page lists a **High-Resolution Wind Resource Map for South Africa 2020**, published March 2021, as a downloadable raster product. It covers South Africa's land area/nine provinces and describes TIF/ASC files at 0.0025-degree WGS84 grid spacing with mean wind speed, mean power density, air density, Weibull A/k and WAsP RIX at 20, 50, 100 and 150 m AGL. Its wind-climatological input is the 3.3-km VNWA WRF 3.8.1 with ERA5 boundary conditions over 1990–2019; air-density input is ERA5 2010–2019; terrain/land-cover inputs are SRTM+ and ESACCI 2015. The page also lists 285,390 South African WAsP generalized wind climate (LIB) files and an index/coordinate CSV.

These are distinct products from the 30-year point time-series extraction portal described in the 2026-09-01 CSAG outage notice. The portal is unavailable pending regeneration; the WASA3 raster/GWC products are listed separately and must be source-admitted individually.

Important WASA3 limitations:
- The resource map was designed for GIS-based strategic environmental assessment and planning.
- The download page states that accuracy depends on VNWA, WAsP microscale modelling and topographic inputs.
- In complex terrain where RIX > 5%, WAsP IBZ may significantly overestimate resource.
- Results near/above built-up areas and forests are less reliable.
- On-site measurements are strongly recommended for actual wind-farm design and detailed site assessment.
- The page notes VNWA validation against 19 WASA measurement masts. That does not mean every raster cell or proposed turbine location has been measured.

The **GWA4 validation cohort** is also not a South Africa validation campaign. The official validation page lists 52 sites in Bangladesh, Ethiopia, Maldives, Pakistan, Papua New Guinea, Vietnam and Zambia; South Africa is absent from that listed cohort. GWA4 over South Africa therefore remains modelled screening pending local validation.

A further external benchmark is the 2026-01-14 Nature Communications paper *Towards high resolution, validated and open global wind power assessments*, which describes ETHOS.RESKit. The paper uses GWA4 + ERA5; it reports calibration on 213 meteorological mast locations in 25 countries with over 8 million filtered hourly observations and states South African mast observations occur in its wider calibration geography. Its separate turbine/farm validation dataset covers more than 8 million hourly observations from 152 turbine/farm sites, but the raw power data are restricted by third-party agreements. The published hourly capacity-factor comparison reports 5.6% relative mean error, Pearson r=0.844, RMSE 0.196 and Perkins skill score 0.90. The authors also warn that local single-site accuracy remains limited by coarse weather-data representation and terrain/diurnal/seasonal effects. They state that the proprietary wind-farm inventory is not redistributable.

**South African holdout rule:** resolve exact WASA mast IDs, observation periods and heights against the ETHOS.RESKit 213-mast calibration cohort before using any such mast in a blind evaluation. A mast included in calibration cannot serve as an independent holdout. Do not imply ETHOS.RESKit has South African turbine/farm output validation merely because it used South African weather masts.

Sources:
- WASA3 South Africa resource map and downloads: https://wasadata.csir.co.za/wasa1/WASAData?pemail=nimo%40risoe.dtu.dk&pscreen=resource2021
- CSAG 30-year time-series portal availability notice: https://www.csag.uct.ac.za/wind-atlas-for-south-africa-time-series-data/
- GWA4 validation: https://globalwindatlas.info/about/validation
- ETHOS.RESKit 2026 paper: https://pmc.ncbi.nlm.nih.gov/articles/PMC12804690/
- ETHOS.RESKit archive: https://doi.org/10.5281/zenodo.17668776
- Current RESKit tag v0.5.0: https://github.com/FZJ-IEK3-VSA/RESKit/releases/tag/v0.5.0


## 13. South African hydropower: use existing water infrastructure before new dams
The primary local reference is the **South African Hydropower Atlas**, WRC Report TT 916/23 (June 2023), plus the WRC SAHA ArcGIS E-Tool. The report explicitly covers conduit hydropower, hydrokinetic rivers/canals, run-of-river, storage dams, pumped storage, WTW, WWTW, weirs and transfer schemes. It notes 284 municipalities and multiple utilities/mines with gravity-fed water distribution systems as potential conduit-hydro owners.

This is a much stronger foundation than the older 2002 national baseline, but it remains a discovery and preliminary assessment:
- Conduit potential uses **pressure head and flow**; data availability particularly limited evaluation of conduit opportunities.
- WWTW/WTW assessment methods are described as underexplored and initial criteria rely on assumptions.
- Hydrokinetic South African case studies were scarce, and the report notes potential variance between estimated and actual flows.
- The atlas's pumped-storage potential figure of 31,000 MW assumes 12 hours/day and overlapping candidate sites; it states actual potential is significantly lower. Geological, hydrological and environmental investigations were not completed for every identified pair.
- The report says the assessment criteria should be strengthened through additional site feasibility studies.

The source registry separately records the WRC report and the WRC ArcGIS catalogue entry. Do not assume that a report download grants rights to every underlying dataset or ArcGIS service.

### First-order architecture hypotheses

1. **Pressure-reducing station recovery:** compare a bypass turbine / PAT with the PRV baseline. The water operator's pressure setpoints, minimum delivery pressure and outage response remain hard constraints.
2. **Water-treatment works (WTW):** recover available elevation head in an inlet/conveyance route only where the hydraulic boundary and plant operations support it.
3. **Wastewater-treatment works (WWTW):** inspect actual inlet/outfall head and flow; include treatment performance, debris/solids, water quality and discharge compliance. Do not presume a constant outflow or that a turbine is suitable at every works.
4. **Irrigation canals, rivers and weirs:** compare low-head diversion/chute turbines and hydrokinetic devices separately. Hydrokinetic energy depends strongly on velocity, swept area and device-specific hydrodynamics; velocity alone is not a power prediction.
5. **Existing dam releases:** retrofit turbines into authorised water-supply, irrigation, flood-control or environmental releases without assuming extra flow or modified timing.
6. **Pumped storage:** separate storage from generation and quantify pumping input, generating output, round-trip efficiency, reservoir active volume, head, evaporation/seepage, environmental conditions and grid services.

### Service-preserving design objective

Maximise **net useful energy recovered while maintaining or improving water service and environmental outcomes**. A design that reduces water delivery, violates pressure requirements, degrades treatment, compromises an ecological release or weakens system resilience is infeasible regardless of its energy revenue.

Report two trajectories independently:
- water: delivered volume, pressure at critical nodes, treatment/effluent compliance, service availability, ecological releases and recovery from faults;
- energy: gross generation, auxiliary/parasitic load, net output, temporal profile, self-consumption, export, curtailment and recovered energy per cubic metre.

The DWS January 2025 Water and Sanitation Sector Climate Response Strategy notes existing South African PAT examples and PRV-replacement energy-recovery opportunities. The 2026 WDN hydropower review describes PATs as a low-cost standardised option but reports a broad 40–75% small PAT/micro-hydro efficiency range versus >90% for suitable larger turbines. Those are literature ranges, not specific device or site performance guarantees. A PAT must be selected from a manufacturer/test efficiency map at the actual head/flow duty profile; do not apply one constant peak efficiency to all operating conditions.

### WRC Atlas potential table: do not overclaim

The 2023 report lists, among other categories, conduit potential >83 MW; WWTW 0.73–3.7 MW; WTW 0.67–3.3 MW; weirs 10.6–50.3 MW; hydrokinetic >16 MW; storage schemes 1,102 MW; and pumped storage 31,000 MW. These are reported preliminary atlas figures, not confirmed available plant capacity. For run-of-river it reports 760–882 kW/km with the national total still being determined. The report explicitly says figures will be updated as source data mature and small-hydro data access remains a challenge.

Never sum overlapping resources, convert kW/km to site MW without a reach-specific model, or compare the pumped-storage headline to conventional generation capacity.

Sources:
- WRC Hydropower Atlas TT 916/23: https://wrcwebsite.azurewebsites.net/wp-content/uploads/mdocs/TT%20916%20final%20web.pdf
- WRC SAHA E-Tool: https://search.wrc.org.za/reports/ada2ec32-f2c3-4703-af7f-d632d9965a90
- DWS 2025 Climate Change Response Strategy: https://www.dws.gov.za/documents/Other/Water%20and%20Sanitation%20Sector/2025/NCC_2025_Final_01H_4%20%282%29.pdf
- 2026 WDN hydropower review: https://doi.org/10.3390/en19020320
- IEC 2026 pumped-storage trend report: https://webstore.iec.ch/en/publication/115300


## 14. Hydraulic-machine standards and South African water-use gate

For a conduit/low-head hydro study, the applicable acceptance/commissioning references must be selected by machine size and contract boundary:

- **IEC 60041:1991** — field acceptance tests for hydraulic turbines, storage pumps and pump-turbines, including contractual guarantee verification, measurement and reporting.
- **IEC 60193:2019** — hydraulic-machine model acceptance tests; its principal scope is machines above 5 MW unit power or 3 m reference diameter, with use on smaller units by agreement.
- **IEC 61362:2024** — specification and performance information for turbine governing systems, including speed, power, water level, opening/flow control and emergency shutdown. It is not an acceptance-test standard.
- **IEC 60545:2021** — commissioning and operation guidelines for hydraulic turbines, pump-turbines and storage pumps.
- **IEC 62006:2010** — small-hydroelectric installation acceptance tests, including safety, trial operation, reliability and performance classes A/B/C. IEC 62006 Edition 2 is a development project as of 2026-11-09; it is not yet a published replacement.

South African water-use authorization is a separate gate. The National Water Act 36 of 1998 defines water use broadly and generally requires authorization unless an applicable Schedule 1 use, existing lawful use, general authorization or other exemption applies. The Reserve (basic human needs and ecological requirements) is a core constraint. The owner/operator must establish the exact authorization path with DWS or the relevant Catchment Management Agency; this research document is not legal advice and cannot decide a licence class.

Sources:
- IEC 60041: https://webstore.iec.ch/
- IEC 60193:2019: https://webstore.iec.ch/en/publication/60951
- IEC 61362:2024: https://webstore.iec.ch/en/publication/72909
- IEC 60545:2021: https://webstore.iec.ch/en/publication/64109
- IEC 62006 second-edition project status: https://iss.rs/en/project/show/iec%3Aproj%3A138148
- National Water Act: https://www.gov.za/documents/national-water-act
- DWS Water Use Authorization Process: https://www.dws.gov.za/ewulaas/WUA.aspx

No model result or SAHA candidate substitutes for a project-specific authorization, environmental-flow requirement, owner agreement or field acceptance test.


## 15. WDN pressure, leakage and PAT control: contemporary benchmark architecture (checked 2026-11-09)

The WDN research branch is now source-backed by five distinct South African and international references:
- SAHA Wastewater Treatment Works layer (ArcGIS item 15265200a008406a951e40b7fb6d976f), a facility locator only; it has no measured outlet head/flow or current operational evidence.
- UKZN's 2021 Cornubia/Pat Marshal optimisation dissertation; modelled PRV/PAT placement and leakage reduction, not field-metered generation.
- The 2022 peer-reviewed Cornubia PAT study; its five-PAT configuration reports 166.31 kW candidate output and calculated 45.59 kL/month (18.16%) leakage reduction. It used EPANET 2.2 PRV proxies and post-processed PAT power using empirical BEP/flow/head assumptions, with 12-hour/day and 335-day/year operating assumptions. These are model outputs, not metered plant results.
- The 2025 Jowitt network study compares one-to-five PAT layouts under DDA and PDA on a 24-hour extended-period simulation; reported energy differs between demand-driven and pressure-driven cases. PRVs performed better for leakage reduction in the paper's tested configurations, so pressure, delivered demand, leakage and generation must remain separate outcome fields.
- The April 2026 multi-node PAT RTC study combines a Kalman filter for state/disturbance estimation, tracking MPC for multiple pressure outputs and an actuator-settings optimiser. Its evidence is closed-loop simulation on an unsteady pressure-driven model, not South African field deployment.

### EPANET version policy

Use **OWA-EPANET 2.3.5** (tag commit `444725fbf16ab28b0470ef0ed281dc4849d49583`, released 2026-02-20) as the current open network engine. EPANET 2.3 introduced:
- FAVAD pressure-dependent pipe leakage;
- Positional Control Valve (PCV);
- independent pressure-unit conventions;
- Flow Balance reporting;
- other Toolkit and controls improvements.

Version 2.3.5 fixes Generic-curve flow-unit conversion and mixed-unit handling. For a faithful replay of the 2022 Cornubia article, separately pin **OWA-EPANET 2.2** (tag commit `4d8d82ddc260fce216af9321fc3d9a4646ac6827`). A 2.3.5 result is a modern re-analysis, not an exact reproduction of a 2.2 paper result.

EPANET is a network hydraulic solver, not an electrical PAT model. PAT behavior must not be represented by a PRV proxy and then reported as verified PAT power. A calibrated equipment-specific hydraulic characteristic can enter the network solution only if it reproduces the declared PAT head/flow boundary. The resulting operating point must be passed to Symtropy's evidence-bound PAT/generator map for net electrical output. This co-simulation remains a proposed benchmark architecture until validated against full manufacturer maps and physical test data.

### What should distinguish Luminous

The next frontier is a *water-first, pressure/leakage-aware energy control system*:
1. Estimate node pressures, demand and disturbances with explicit sensor quality/uncertainty.
2. Forecast demand and uncertainty across the network horizon.
3. Keep minimum service pressure, delivered volume, treatment quality and ecological/water-use constraints as hard gates.
4. Coordinate PRVs and PATs globally rather than maximizing each PAT's local power.
5. Model pressure-dependent leakage and report water saved independently of electricity recovered.
6. Use actuator penalties and equipment maps to prevent chattering, excessive cycling and out-of-domain operation.
7. Include safe fallback and explicit indeterminate states when sensing, model fit or solver convergence is inadequate.

The current programme issue for this work is Symtropy #4397. Its candidate controller is a proposed robust/service-constrained MPC benchmark, not a current field controller or claimed winner.


## 15. PAT evidence ladder: laboratory curves, full-scale operation and South African transfer

The source register now preserves several distinct types of PAT evidence. They must not be treated as interchangeable:

**Component laboratory curves**
- Pugliese et al. (2016) experimentally characterized two pumps operating in turbine mode and compared results with prediction relationships. The paper is particularly relevant because it identifies limited reverse-mode manufacturer information and the scarcity of experimental PAT curves as an unresolved issue.
- Manservigi et al. (2025) compare hydraulic and electrical regulation using experimental characteristic curves for 16 turbomachines, applied to a real Northern Italian WDN serving approximately 5,000 users. The study reports 44.1% recovery of available network hydraulic energy for its selected PAT and case-specific LCOE/capital costs. Its experimental curves are component evidence; the WDN energy and financial results are a model/case evaluation, not 16 field installations.
- Parra et al. (2018) test an intelligent PAT/PRV pressure-management arrangement under variable discharges. The abstract reports critical-point pressure maintenance, up to 2.3 kW at PAT full load, maximum total net system efficiency of 40%, and up to 16% water savings over the tested conventional PRV baseline. These are experimental results for a German case, not South African network performance.

**Full-scale field operation**
- Muhammetoglu et al. (2018) report approximately five months of online monitoring of a full-scale PAT installed in parallel with a PRV in the Antalya WDN. The abstract gives 130–300 m³/h inflows, 0.7–8.4 kWh reported energy values for approximately 100 kPa pressure reduction, and 60% average efficiency. The abstract does not state the time denominator for those kWh values. We must not convert them to kW, daily energy or annual yield without the full article. The reported payback is an initial case calculation, not a South African commercial forecast. The related 2017 ASCE paper describes the same installation, not an independent second plant.

**South African transfer baseline**
- Cornubia 2022 is EPANET 2.2-based and uses PRV proxies for the PAT branches; the five-PAT 166.31 kW / 45.59 kL/month result is modelled rather than metered output.
- The UKZN 2021 dissertation has overlapping Cornubia geography. It is not an independent replication until network input and placement assumptions are cross-walked.
- The 2025 Jowitt study is a different network and shows that DDA/PDA and PAT selection based on max/mean/minimum discharge change results; its results are simulations, not field observations.
- The 2026 multi-node PAT controller combines Kalman filtering, multi-output tracking MPC and actuator-setting optimization in closed-loop simulation on a pressure-driven unsteady-flow network; it is a current control-method comparator, not yet South African field evidence.

### Luminous benchmark strategy

Our target should be a reproducible evidence chain across all four levels:
1. Reproduce a laboratory PAT head-flow-efficiency map within its test uncertainty.
2. Reproduce a full-scale PAT operating trajectory using the actual data boundary and sampling interval.
3. Reproduce network-scale water balance, pressure, leakage and PAT-energy results against an exact EPANET baseline.
4. Compare fixed controls, pressure-management-only, static PAT, hydraulic regulation, electrical regulation, Kalman/MPC and robust service-constrained MPC under matched demand, sensor, map and compute budgets.

A “better than SOTA” result must show matched improvements in net kWh, leakage, pressure compliance, delivered demand, actuator effort, lifecycle cost and uncertainty—not one attractive energy number. For independent South African transfer, hold out a complete zone and period never used for map fitting, placement tuning or controller calibration.

References:
- Antalya full-scale operation: https://doi.org/10.2166/ws.2017.087
- Parra PAT/PRV experimental control: https://doi.org/10.2166/ws.2017.154
- Manservigi 16 experimental PAT curves: https://doi.org/10.1017/wat.2025.10004
- Pugliese two-PAT test-rig characterization: https://doi.org/10.1016/j.renene.2016.06.051
- Cornubia 2022: https://doi.org/10.3390/w14071150
- Jowitt 2025: https://doi.org/10.1016/j.egyr.2025.09.034
- Multi-node PAT RTC/MPC 2026: https://doi.org/10.1016/j.conengprac.2026.106777


## 16. South African field evidence: Queenswood, Pierre van Ryneveld and Leliefontein

Three local field records are now separate in the source register. This is a meaningful upgrade from comparing South African simulations with overseas field evidence alone.

### Queenswood Reservoir, Tshwane — WRC 2010 PAT pilot

WRC KV 238/10 documents a low-budget PAT pilot at Queenswood. Table i contains five time-stamped readings. It reports measured output of approximately **2.32–4.52 kW**, turbine-branch flows of **27.55–32.49 L/s**, and head drops of **25.04–30.94 m**. The report describes these as initial observations from a pilot that was not optimized; the reverse-mode characteristics were manufacturer-estimated rather than a full, independently characterized PAT map.

The 2014 Tshwane paper also describes Queenswood, but it is the **same installation** and must not be counted as an independent replication. The WRC table heading refers to measured output energy, while its column is watts; retain the actual power units. Five points do not establish a duty cycle, availability, seasonal performance or annual yield.

### Pierre van Ryneveld Reservoir — Tshwane 2011 conduit-hydro installation

The 2014 paper separately documents a cross-flow turbine and synchronous generator at Pierre van Ryneveld Reservoir, launched on 29 November 2011 for local lighting, alarm and communications loads. It describes three field experiments and gives approximately **15 kW maximum capacity depending on flow and head**. This is not a measured annual average or a current operating-status claim.

Importantly, the paper records practical controller and hydraulic issues: generator frequency regulation, sudden load rejection, PRV hunting due to slow response, variable water supply/head, and manual operation. These should become explicit negative-control and fault-injection cases for modern PAT control, not be hidden as implementation details.

### Leliefontein, Drakenstein — bidirectional municipal PAT station

The 2021 IMESA paper describes the Leliefontein station as operating since June 2018. It is a bidirectional pump/PAT arrangement with reversible flow/rotation, active-front-end drives, actuated valves, grid protection, bidirectional metering and SCADA. This is especially valuable because it couples pumping and generation instead of treating the site as a one-way generator.

The paper's **320 MWh/year** and **9.9 GWh over 2018–2041** values are estimates, not an admitted, audited meter export. A separate statement gives cumulative figures of **271,236 MWh generated** and **53,577 MWh consumed**, which are internally inconsistent with the paper's own annual/lifetime figures. We preserve that contradiction as unresolved; do not normalize, use as MWh or infer efficiency until the authors/operator confirm the units and original register data.

### What we should request from operators

1. Queenswood: the original five-point test log, instrument calibration, exact sample durations, PAT/bypass valve state, turbine speed and pressure/flow meter locations.
2. Pierre van Ryneveld: three field experiment logs, generator voltage/frequency/power, flow/head, load-rejection traces, PRV response and current operating/maintenance status.
3. Leliefontein: bidirectional meter exports and SCADA tags with timestamps, import/export register semantics, meter calibration, pump/turbine mode flags, flow/pressure, bypass/valve states, and the raw values behind the conflicting cumulative totals.

No telemetry should be copied into the public repository without operator permission and an explicit data-rights review. The source register records the papers as published field evidence but keeps all original snapshots absent and all claims below the raw-data admission gate.

Sources:
- WRC KV 238/10: https://www.wrc.org.za/wp-content/uploads/mdocs/KV%20238-10.pdf
- Tshwane conduit-hydropower case studies: https://www.scielo.org.za/scielo.php?pid=S1021-20192014000300001&script=sci_arttext
- Leliefontein PAT station, IMESA 2021: https://researchspace.csir.co.za/bitstream/handle/10204/12279/Steyn_2021.pdf
