# Johannesburg source register v0.1

This file is a research inventory of candidate sources, not an ingested or qualified local dataset. It records what is verifiably published and what still requires a rights, coverage, metadata and freshness check.

## Current source posture

- GCRO has a public catalogue across QoL survey data, administrative boundaries, public infrastructure, land cover/urban footprint, demographics and biodiversity. The portal itself notes that access varies by layer: some downloads are direct, others require following a landing/menu page or registration.
- GCRO QoL7 2023/24 is an anonymised public-use survey dataset under CC BY 4.0. It is a weighted household/adult survey, not building-level population truth.
- Stats SA published ward-level Small Area Population Estimates for 2022 using 2020 ward boundaries and raking/IPF. Stats SA itself warns of instability in small-area values; these estimates must not be silently downscaled to buildings.
- Johannesburg announced the ISDP/SPMS in August 2025. The announcement is not a licence grant or proof of anonymous API/download access for each layer.
- OSM is usable under ODbL with attribution and share-alike obligations; completeness and local positional correctness must be audited.
- No current official Johannesburg-area GTFS feed has been verified here. This is an explicit unresolved source gap, not a claim that no feed exists.
- The Johannesburg Development Agency announced completion of the Universal Design Access Plan (UDAP) on 2026-06-09. This adds a verified public method/plan-announcement source, not a verified machine-readable facility-access dataset; audit schedules, scores, risk register and full plan appendices remain unverified.

## Johannesburg municipal spatial-data path (verified public description)

The City of Johannesburg Corporate Geo-Informatics page provides a more actionable starting point than a generic municipal map reference:

- The City publishes a link to its **Spatial Data Inventory 2024**. The page says the inventory lists dataset descriptions, data type/format, creation date, data owner/contact and usage limitations. The inventory PDF itself has not yet been retrieved and parsed in this work; per-layer metadata remain unverified.
- The City says its external eServices Online Maps viewer is accessible without registering as an eServices client. Its user guidance says **some** map layers can be extracted as shapefile, KML or DXF. It also says CGIS is not the data custodian for all published layers and directs users to each layer's owner.
- The City's 2023 aerial product is reported to cover 1,644 km² and includes 15 cm pixel-resolution orthophotos, LiDAR with a minimum 4 points/m² and derived 25 cm contours. Orthophotos were made available on the Online Map Viewers from 2024-04-09. Orthophotos, contours and ground/non-ground LiDAR point data are available on request from CGIS Projects. Accuracy report, CRS/vertical datum, per-tile capture date and reuse terms must be confirmed before use.
- Contact points published by the City include cgisprojects@joburg.org.za (digital data/mapping) and ims@joburg.org.za (Online Maps/imagery queries). These are data-discovery routes, not permission to reuse a layer.

**Rules:** anonymous viewer access does not prove redistribution permission; a visible layer does not prove it is complete/current; orthophotos/LiDAR can support bounded geometry/massing analysis but cannot establish roof/wall materials, insulation, ownership, occupancy or indoor temperature.

Source: https://eservices.joburg.org.za/sites/cgismaps/Pages/default.aspx  
Extraction guidance: https://edit.eservices.joburg.org.za/Pages/OnlineMaps.aspx

## Johannesburg Universal Design Access Plan — completed 2026-06

The Johannesburg Development Agency (JDA) announced on **9 June 2026** that the Universal Design Access Plan for the City's Integrated Public Transport Network had been completed and finalised. The project ran from January 2025 to June 2026. The published description says its work included research, stakeholder engagement, facility audits, customer-satisfaction surveys and capacity-building, and that it assessed Rea Vaya, Metrobus and other operators across the full passenger journey.

The City/JDA's 19 August 2026 public explanation describes the plan as a roadmap covering trip planning and information, reaching a stop, boarding, travel, transfers and arrival. It identifies continuing barriers around pedestrian connections to stations, accessible passenger information, consistency across facilities, vehicle/platform interfaces, maintenance, staff training and integration between modes. The same public description says the plan would be considered through the City's Section 79 Committee and submitted to Council; the latest Council adoption status as of 9 November 2026 has not been independently verified here.

**Evidence state:** the official completion announcement and public descriptions are verified public sources. The full approved plan, appendices, audit tables, risk register, facility-level scores, station/stop/vehicle identifiers, survey instruments, methods and redistribution rights have not been obtained. Do not treat the announcement or roadmap as proof that an individual station, vehicle, route or neighbourhood is accessible.

### What to request before local validation

1. Final UDAP and appendices, technical audit framework, accessibility risk register and intervention schedule.
2. Facility/stop/vehicle inventory with stable IDs, operator/custodian, coordinates, audit date, source and revision.
3. Audit observations by travel-chain element: pedestrian approach, crossings, wayfinding, digital/printed information, fare purchase, boarding/vehicle/platform interface, onboard accessibility, transfer and destination access.
4. Survey instrument, sampling/response frame, response coverage, weighting, disaggregation, disclosure limits and rights.
5. Action register with responsible owner, committed funding, delivery milestone, maintenance obligations and independent verification method.
6. Council approval/adoption state, effective date and versioned implementation baseline.

### How CityMorph should use the UDAP

Map the UDAP's audit and intervention outputs into the existing CityMorph access owner as owner-supplied observations and barrier evidence. Do not create a second routing/access model. Use its categories to propose additional audit dimensions and known-answer fixtures, but retain Unknown for absent facility/vehicle evidence and preserve rights, dates and source revisions. Cross-check route graph and stop data separately; a completed universal-design plan is not GTFS, an operating timetable, capacity evidence or delivered trips.

Sources:
- JDA completion release (9 June 2026): https://www.jda.org.za/jda-completes-landmark-universal-design-access-plan-to-advance-inclusive-public-transport-in-johannesburg/
- City/JDA description of remaining travel-chain barriers (19 August 2026): https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/JDAs-plan-puts-people-at-the-heart-of-Joburgs-public-transport.aspx
- City completion summary: https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/JDA-completes-universal-design-plan-for-public-transport.aspx
## GreenBook MetroView climate-risk source

GreenBook publicly describes MetroView as providing quantified spatial climate-risk information up to **ward level**, covering current and 2050 climate projections and components for hazard likelihood, exposure and vulnerability. The Johannesburg-specific link opens a JavaScript web application.

What is not yet verified: direct data export/API, underlying climate model/scenario/period/ensemble, boundary vintage, uncertainty, data update cadence and licence/redistribution. Treat this as a candidate ward-level context source only until its technical documentation and reuse rights are acquired. Never disaggregate its ward-level risk to individual buildings or households.

- Overview: https://greenbook.co.za/tool-landing-page.html
- Johannesburg profile application: https://joburg-riskprofile.greenbook.co.za/

## Exact Gauteng TMYx candidates (listed, not ingested)

The 2026 Gauteng listing includes two Johannesburg TMYx files for 2011–2025:

| Candidate | Listed filename | Listing size | State |
|---|---|---:|---|
| Johannesburg-Tambo International Airport (WMO 683680) | `ZAF_GT_Johannesburg-Tambo.Intl.AP.683680_TMYx.2011-2025.zip` | 433 KiB | Listed only; not downloaded or header-validated |
| Johannesburg Botanical Gardens (WMO 683610) | `ZAF_GT_Johannesburg.Botanical.Gardens.683610_TMYx.2011-2025.zip` | 385 KiB | Listed only; not downloaded or header-validated |

Retain both candidates for station-representativeness comparison. Do not assume the airport station represents dense built-up or valley microclimates, or that botanical-garden conditions represent all neighbourhoods. For each file, inspect EPW/STAT/COMMENT1 metadata, coordinates, elevation, period of record, missingness, time conventions, solar provenance and rights before selecting any reference profile. TMYx remains a typical-weather energy-screening source—not an observed heatwave, design extreme or future projection.

Sources:
- Gauteng/South Africa listing: https://climate.onebuilding.org/WMO_Region_1_Africa/ZAF_South_Africa/index.html
- TMYx sources and release notes: https://climate.onebuilding.org/sources/default.html

## Rules

1. No data file enters the pilot without a publisher, source URL, rights state, retrieved-at timestamp, version/effective date and content digest.
2. A web map is not automatically a redistributable dataset.
3. Survey estimates retain sample design, weights, uncertainty and disclosure restrictions.
4. Ward totals are not converted to building-level populations without a separately reviewed model.
5. Missing service hours, capacity, affordability, eligibility, step-free status and timetable data remain Unknown.
6. Do not infer a bus headway from a route line or stop point.
7. Preserve raw snapshots separately from normalized derivatives and make rebuilds deterministic.

## Source links

- GCRO portal: https://www.gcro.ac.za/outputs/dataportal/
- GCRO QoL7 / DataFirst: https://datafirst.uct.ac.za/dataportal/index.php/catalog/995
- Stats SA ward small area estimates: https://www.statssa.gov.za/?p=18967
- City of Johannesburg ISDP/SPMS announcement: https://joburg.org.za/media_/Newsroom/Pages/2025-News-Articles/Joburg-launches-SA%E2%80%99s-first-integrated-Spatial-Data-and-Planning-system.aspx
- OSM rights and attribution: https://www.openstreetmap.org/copyright
- GTFS publishing best practices: https://gtfs.org/documentation/schedule/schedule-best-practices/

See `JOHANNESBURG_SOURCE_REGISTER_V0_1.json` for machine-readable access and rights states.


## Construction and demolition material data: coverage warning

South African C&D material data are not a dependable complete inventory. A 2022 national estimate reported materially different 2017 totals using different methods (20.2 Mt potential generation by lifetime method; 10.8 Mt reaching disposal sites by per-capita method; 4.48 Mt in official statistics) and identified likely underreporting, including informal-sector reuse. These quantities have different definitions/boundaries and must not be combined or converted into present-day Johannesburg stock.

A November 2025 qualitative study in south-eastern Johannesburg interviewed 20 private-sector stakeholders; all 20 mentioned landfill use, and landfill was described as the cheapest option in those accounts. This is qualitative evidence, not a statistically representative citywide disposal rate or tonnage inventory.

A 2025 human-settlements report contains a Johannesburg deconstruction/reuse scenario: at 50% adoption it models 225.21 kt of salvaged materials/components and about 44% less waste otherwise sent to landfill or fly-tipped. This is a modeled scenario, not measured recoverable supply.

For material-loop mapping, an unrecorded facility or informal recovery channel is **Unknown**, not zero. A future pilot must include consent-based, non-extractive stakeholder engagement with informal salvagers, repairers, reuse yards and waste pickers, and must preserve the right not to disclose commercially or personally sensitive routes, locations, prices or identities. Any published spatial layer must use an aggregation/privacy policy and explicitly describe who and what is not represented.

Research references:
- https://journals.co.za/doi/10.17159/sajs.2022/12485
- https://doi.org/10.1080/03736245.2025.2587860
- https://www.circulareconomy.co.za/wp-content/uploads/2025/03/Human-Settlements_Report-CE-Opportunities_final.pdf


## Johannesburg-specific QoL7 evidence added (7 November 2026)

GCRO publishes a City of Johannesburg Municipal Report based on the 2023/24 Quality of Life Survey. It covers demographic composition, basic services, infrastructure conditions and public engagement. The underlying survey has 13,795 respondents across Gauteng overall, not 13,795 Johannesburg respondents; municipal comparisons must follow the survey's sample design and weighting reports.

Use the City report as survey-derived municipal context. It is not a street/building-level dataset, current 2026 service audit, facility point inventory or individual travel diary. Keep the public QoL dataset's CC BY 4.0 licence separate from the report's own reuse terms, which require their own check.

- City of Johannesburg Municipal Report: https://www.gcro.ac.za/outputs/data-briefs/detail/city-of-johannesburg-municipal-report-findings-from-the-gcros-quality-of-life-survey-7-202324/
- GCRO QoL7 technical and field documentation: https://www.gcro.ac.za/research/project/detail/quality-life-survey-7/
- GCRO QoL7 municipal benchmarking report: https://www.gcro.ac.za/outputs/data-briefs/detail/municipal-benchmarking-report-findings-from-the-gcros-quality-of-life-survey-7-202324/


## South African CDW systems review added (7 November 2026)

Amadi and Mahachi's 2025 state-of-the-art review reports high landfill dependence and under-reporting of South African construction-and-demolition waste, including missing informal recovery/reuse flows. It reports that 48% of C&D waste is deposited in scarce landfills and describes severe Johannesburg/Cape Town landfill-airspace pressure in the cited evidence.

These are review findings and source-context signals, **not current 2026 landfill capacity, Johannesburg project-level tonnage, or a qualified secondary-material inventory**. Before a local claim, inspect the primary studies and obtain current facility/municipal records. The CityMaterialLoops material-flow audit must count formal disposal, informal recovery/reuse, recycling, stockpiling and exported flows separately, with no duplicate batch allocation.

- Amadi & Mahachi (2025), *State-of-the-art review on construction and demolition waste: The South African context*: https://doi.org/10.1016/j.clwas.2025.100251


## Update — municipal GIS and observed transport evidence (2026-11-08)

The City of Johannesburg Corporate Geo-Informatics Online Maps viewer is publicly accessible without eServices registration. The City's help material says selected layers can be extracted as Shapefile/KML/DXF; the 2024 Spatial Data Inventory records dataset owner, format/date and usage limits and is reviewed annually. This materially improves source discoverability, but not all layers are open or current.

Freshness is explicitly heterogeneous: the Online Maps help page says cadastral data are weekly, the JRA roads layer was last updated in August 2019, 2023 aerial imagery was flown in May 2023, and most other data were captured over multiple years. A road layer from 2019 is not acceptable as current routing ground truth without a newer custodian-supplied snapshot or an explicit stale-data experiment.

The City also announced a 30–60 day electric-bus trial beginning 3 September 2026, with selected Metrobus/Rea Vaya buses integrated into Metrobus's fleet management and AVL system. This creates a potential request-only source for aggregate observed operations if the City publishes or authorizes release; it is not public GTFS and is not proof of route timetable availability. Metrobus's quarterly performance assessments are another public research source, but narrative reports must not be mistaken for machine-readable schedules.

Sources:
- https://eservices.joburg.org.za/pages/OnlineMaps.aspx
- https://eservices.joburg.org.za/sites/cgismaps/Pages/default.aspx
- https://mbus.joburg.org.za/about_strat_reports.aspx
- https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/City-puts-electric-buses-to-the-test-in-drive-towards-greener-public-transport.aspx
