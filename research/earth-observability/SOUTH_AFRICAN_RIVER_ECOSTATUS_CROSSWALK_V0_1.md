# South African River Eco-status (REMP) ↔ SER Reference Model Crosswalk v0.1

**Status:** framework interoperability and data-admission protocol; no Jukskei REMP sample has been admitted.  
**Issue:** [#860](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/860)  
**Composes with:** #856 dynamic reference ensembles; #845 PCIT; #853 IOEQ; #847/#852 Jukskei interventions and source admission.

## 1. Why this is a strategic improvement

Sol-Atlas should not import an international restoration framework and then assume it has the best locally relevant ecological measurements. South Africa already has a river-specific monitoring framework: the Department of Water and Sanitation's River Eco-status Monitoring Programme (REMP), part of NAEHMP. REMP evaluates river ecological condition from system drivers and biological responses and establishes relative reference conditions for defined sites or reaches.

The DWS National State of Water Report 2024 describes REMP tools including IHI, FRAI, MIRAI, VEGRAI and integrated Eco-status, with quarterly monitoring and annual technical reports. DWS river rehabilitation guidance additionally lists SASS, HAI, GAI and PAI. These provide a much better starting vocabulary for a South African river dossier than a generic satellite-derived "river health" score. [DWS REMP](https://www.dws.gov.za/iwqs/rhp/rhp_background.aspx); [National State of Water Report 2024](https://www.dws.gov.za/Projects/National%20State%20of%20Water%20Report/Documents/National%20State%20of%20Water%20Report%202024_Final%20%28002%29.pdf); [River Rehabilitation Guidelines, 2024 draft](https://www.dws.gov.za/wem/documents/RMG_Draft%20River%20Guidelines_16Apr2024_V3.6.pdf). 

## 2. Do not collapse REMP and SER

The frameworks answer related but different questions:

- **REMP:** what is the ecological condition of this river reach/site, based on driver and biological-response evidence?
- **SER:** how does this restoration project compare with a justified reference model across the six key ecosystem attributes?
- **PCIT:** can an apparent temporal difference be distinguished from changes in the observation/measurement system?
- **IOEQ:** is the claim an implementation output, operational performance, observed physical outcome, attributable intervention effect, or modelled projection?

An integrated REMP Eco-status is not SER's Five-star recovery assessment and is not a causal effect estimate. A river may improve in one REMP indicator while another worsens; retain indicator-level evidence and the six SER attributes separately.

## 3. REMP indices and their limits

| DWS index | Domain | Evidence contribution | Important limit |
|---|---|---|---|
| SASS | Macroinvertebrate biological response | Macroinvertebrate community scoring | Depends on sample season, habitat, taxonomic level and field protocol |
| MIRAI | Macroinvertebrate biological response | Macroinvertebrate response to drivers/habitat | Biological response is not a direct measure of all physical-chemical drivers |
| FRAI | Fish biological response | Fish community response | Detection, sampling effort, barriers and site suitability matter |
| VEGRAI | Riparian vegetation response | Riparian vegetation condition | Not equivalent to satellite greenness or canopy fraction |
| IHI | Habitat/driver | Habitat integrity and relevant pressures | Not a complete measure of species composition or ecosystem function |
| HAI | Hydrological driver | Flow-regime condition | Requires flow series, gauge/datum/rating-curve quality and temporal support |
| GAI | Geomorphological driver | Channel/geomorphological condition | Does not by itself measure every connectivity or sediment-exchange process |
| PAI | Physico-chemical driver | Water-quality conditions | Keep analyte, unit, method, detection limits, accreditation and flow context |
| Integrated Eco-status | Integrated summary | Framework-specific ecological-condition category | Not a substitute for the raw indicator set or a causal restoration score |

The machine-readable contract defines the full candidate mapping and source IDs in `SOUTH_AFRICAN_RIVER_ECOSTATUS_CROSSWALK_V0_1.json`.

## 4. Coverage against SER's six attributes

The crosswalk distinguishes direct indicator families from partial proxies and gaps:

| SER attribute | REMP coverage | Additional evidence still needed |
|---|---|---|
| Absence of threats | Partial via PAI, IHI and driver context | Invasive species, abstraction, pollution sources, barriers and direct disturbance inventories |
| Physical conditions | Strong via HAI, GAI, PAI and IHI | Site-specific substrate, bank and microhabitat observations where relevant |
| Species composition | Direct biological response families: SASS, MIRAI, FRAI, VEGRAI | Taxa outside chosen protocols, microbial/fungal communities, finer taxonomy |
| Structural diversity | Partial via IHI and VEGRAI | Vertical structure, woody age/size, deadwood and channel-unit complexity |
| Ecosystem function | Partial proxies through biota and drivers | Productivity, nutrient cycling, decomposition and material processing if in project goals |
| External exchanges | Partial via hydrology, geomorphology, habitat and fish response | Fish passage, lateral floodplain exchange, sediment/organic matter flux, corridor connectivity |

This mapping prevents a common failure: using a convenient available indicator as a substitute for an unmeasured ecological attribute.

## 5. Jukskei-specific discovery and admission

The National State of Water Report's national site counts do **not** establish a REMP station in the Jukskei catchment or at a SUNCASA reach. The next real-world step is a site-ID and data request, not an assumption that national programme data contain our study area.

For each candidate DWS site, obtain:
- Exact station/site identifier, reach and sub-quaternary catchment.
- Geometry, CRS, upstream/downstream position and any changed site boundaries.
- REMP index outputs and, where available, the raw indicator observations and sampling records.
- Reference-condition narrative/version, reference site selection and ecological category rationale.
- Actual observation dates, seasonal window, sampling effort, field protocol, assessor and method versions.
- Missing visits, site changes, QA/QC, calibration and uncertainty.
- Data-use terms for analysis, publication, redistribution and training.

Reconcile DWS site IDs with the Water for the Future headwater station, SUNCASA monitoring network and EcoFilter only through exact identifiers, coordinates, dates and operating records. Similar names or catchment membership are not enough.

## 6. Benchmark extensions

The crosswalk adds six adverse controls:
1. Annual report cadence confused with quarterly sampling.
2. REMP category promoted to SER recovery or causal impact.
3. Satellite greenness promoted as VEGRAI or fish/macroinvertebrate evidence.
4. A broad-catchment REMP station treated as a matched control without reach identity.
5. REMP method/version shift mistaken for ecological change.
6. Integrated Eco-status hides an unobserved SER attribute.

Benchmark direct-vs-proxy classification, indicator-gap disclosure, site/reach scope errors, method-shift false promotion and correct separation of monitoring versus reporting cadence.

## 7. Legacy Jukskei RHP evidence: a pre-SUNCASA baseline lead

The crosswalk now contains a separate legacy River Health Programme (RHP) record from DWS North West Regional Office, *River Health Programme in Crocodile West Marico WMA — Technical Report 3*, Contract WP 10592, final 31 July 2015 ([official DWS PDF](https://www.dws.gov.za/iwqs/rhp/state_of_rivers/Crocodile_West_Marico_WMA_technical_report_3_FINAL.pdf)). This is RHP-era evidence, not current REMP; DWS states that REMP replaced RHP in 2016.

The report identifies five Jukskei sub-quaternary reaches: `A21C-01269`, `A21C-01232`, `A21C-01215`, `A21C-01183`, and `A21C-01167`. The biological monitoring site is `A2JUKS-DIENR`, representing `A21C-01167`.

The report's catchment-level Jukskei summary gives ecological category D (highly modified, with large loss of habitat, biota and ecosystem functioning). At the monitored site:
- Fish PES/Ecological Category is reported as 55.9% / D and stable across 2013–2015.
- Macroinvertebrate MIRAI is reported D in 2013 and D/E in 2014; no 2015 macroinvertebrate sample was collected due to high flow, so the 2014 category was carried forward.
- Riparian vegetation is reported as D.
- Fish visits included field surveys and visual-only surveys during high flow; March 2015 visual work also cites sewage input posing a health risk.
- Desktop Google Earth Habitat Integrity assessment for sub-quaternary reaches was not sensitive enough to detect deterioration within an ecological category.

Treat this as **report-level historical comparator evidence**, not raw biological samples. One site's biota must not be generalized to all five reaches. A stable category does not establish that no within-category deterioration occurred. The machine-readable `legacy_jukskei_assessment` object preserves the report date, programme identity, reach/site IDs, indicator summaries, sampling gaps, pressures and admission requirements.

Required next evidence: original DWS RHP database rows, coordinates, field sheets, taxa lists, MIRAI calculations, fish/FROC records, riparian vegetation forms, habitat-integrity source imagery and method/version. Spatially reconcile this legacy site with current REMP, the 2021 Alexandra study, Water for the Future monitoring, SUNCASA intervention polygons and EcoFilter assets before comparing outcomes.

## 8. References

- DWS, [REMP background and reference condition](https://www.dws.gov.za/iwqs/rhp/rhp_background.aspx).
- DWS, [National State of Water Report 2024, §2.2.4](https://www.dws.gov.za/Projects/National%20State%20of%20Water%20Report/Documents/National%20State%20of%20Water%20Report%202024_Final%20%28002%29.pdf).
- DWS, [Rehabilitation Management Guidelines for Rivers, Version 3.6 (April 2024 draft)](https://www.dws.gov.za/wem/documents/RMG_Draft%20River%20Guidelines_16Apr2024_V3.6.pdf).
- DWS, [River Health indicators and indices](https://www.dws.gov.za/iwqs/rhp/indices.html).
- SER, [International Principles and Standards for Ecological Restoration, Third Edition (June 2026)](https://doi.org/10.1111/rec.70441).

## 9. Claim ceiling

This is an interoperability crosswalk. It does not assert that REMP data exist for Jukskei or Alexandra, that a REMP site overlaps a SUNCASA intervention, that any REMP ecological category improved, or that interventions caused any change.

