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
