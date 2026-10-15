# Johannesburg Water data readiness and refinement plan v0.1

**Research date:** 2026-10-15  
**Repository:** `Luminous-Dynamics/sol-atlas-leptos`  
**Purpose:** add a first real, source-attributed public-data anchor without confusing aggregate reporting with hydraulic engineering inputs.

## What has been captured

The companion machine-readable bundle `sol-atlas-core/fixtures/water-engineering/johannesburg-water-public-observations-2024-25.v0.1.json` transcribes selected aggregate figures from Johannesburg Water's published **2024/2025 Water Quality Report for the Greater Johannesburg Water Supply System**:

- Reporting window: 1 July 2024–30 June 2025.
- 5,890 samples analysed and 79,025 tests undertaken.
- 99.4% of samples reported free of *E. coli*.
- 97.4% of samples reported below 1 NTU turbidity.
- Selected average constituent values and their reported reference thresholds are preserved exactly as transcribed, with notes rather than an assistant-generated compliance ruling.

Source notice published 23 March 2026: [Johannesburg Water — 2024/2025 Water Quality Report](https://www.news24.com/notices/notice/c58d8709-1721-451c-9b80-a7d966206c21). Johannesburg Water's official archive of monthly reports is [here](https://www.johannesburgwater.co.za/resource-centre/water-quality-reports/).

This is genuinely observed/published real-world data, but its granularity is **system-wide aggregate only**. It does not contain sampling-site coordinates or raw samples. Keep the mean E. coli constituent value separate from the reported percentage of samples free of E. coli: they are different summaries with different interpretation needs. The source reports colour in `Hz`; we preserve that unit label rather than silently correcting it.

The bundle also records the official [Pipe Replacement Programme](https://www.johannesburgwater.co.za/about-us/campaigns/pipe-replacement-programme/) targets of 428 km of water pipelines and 286 km of sewer pipelines over five years. These are plans, not completed works, and the public page does not provide pipe-level schedules or asset identifiers.

## Data acquisition register

| Source | Real data it can contribute | Access and limits | What we should do |
|---|---|---|---|
| Johannesburg Water monthly water-quality reports | Report-level test/sampling summaries and selected constituents | Official archive is public; current page lists monthly report links. Monthly PDFs still need careful extraction and period/schema checks. | Build a small importer with source URL, report period, file digest, table/field provenance, and manual spot-check. Preserve the original documents at source; do not republish them wholesale. |
| Johannesburg Water public notices and Daily Water Notices | Publicly announced outages, repair notices, service categories, and notice timing | Notices are changing public communications, not complete incident logs; announcements may not equal actual restoration times. | Capture notice ID, timestamp, published geography, service type, status and original wording. Separate observed notice from verified outage/repair outcome. |
| Johannesburg Water Pipe Replacement Programme | Portfolio targets and programme context | City-wide forward-looking targets only; no per-pipe inventory or completion receipts on this page. | Keep targets separate from realised progress and later reconcile with annual reports or authorised completion data. |
| DWS NIWIS Waste Water Quality | Wastewater effluent and monitoring compliance by province/WSA/WWTW where the system has reporting | Dashboard offers a 12-month view and monthly data refreshes. DWS explicitly cautions that non-reporting facilities may be absent, values are captured by WSAs/works, data have no warranty, and use is limited to academic/research/personal purposes; data may not be sold. | Start with the aggregate dashboard/export and retain the terms, extraction time, WSA/facility coverage, reporting completeness, and exact downloaded-file digest. Do not redistribute raw extracts or use commercially without permission. |
| DWS NIWIS Surface Water Flows / HYDSTRA | Historically observed and verified river flow, plus a distinct unverified real-time stream | DWS says real-time data are unverified and historical verified data can lag. Data have explicit attribution and use conditions. | Keep `verified_historical` and `unverified_realtime` as different evidence classes; never use the latter as a calibrated boundary without qualification. |
| DWS Resource Quality Information Services (RQIS) | River/reservoir water-quality station metadata and chemical/microbial time series | Some data appear as published snapshots; deeper raw historical data may require a request. Monitoring coverage and reporting gaps must be carried with records. | Identify Upper Vaal/Vaal sites relevant to the catchment, request suitable raw time-series, and log the request/permission rather than scraping around access restrictions. |
| Copernicus DEM GLO-30/GLO-90 | Public global digital surface elevation for early terrain/catchment screening | GLO-30/GLO-90 represent a surface including buildings and vegetation; source observations date mainly from 2011–2015. Not engineering survey data or a bare-earth drainage survey. | Use only as low-resolution screening; document tile, CRS, vertical datum/units and digest. Require finer verified topography and storm-drain survey before any design claim. |

DWS source documentation: [Waste Water Quality dashboard data and limitations](https://www.dws.gov.za/niwis2/Info/WasteWaterComplianceInfo), [Surface Water Flows methodology](https://www.dws.gov.za/niwis2/Info/SurfaceWaterFlowsInfo), and [RQIS data/reports](https://www.dwa.gov.za/iwqs/report.aspx). Copernicus references: [DEM collection](https://dataspace.copernicus.eu/explore-data/data-collections/copernicus-contributing-missions/collections-description/COP-DEM) and [GLO-30/GLO-90 COG access and licence overview](https://copernicus-dem-30m.s3.amazonaws.com/readme.html).

## How this can refine Symthaea / Mycelix / Symtropy / Sol-Atlas

1. **Sol-Atlas:** introduce a source-registered public-observation layer with period, spatial coverage, freshness, evidence class, access/rights, completeness and digest. Public aggregates should not be drawn as per-pipe conditions.
2. **Symthaea:** use those values to prioritise *questions* (sampling gaps, reliability indicators, performance changes); it should not infer pipe failure probability from city-wide water quality percentages.
3. **Symtropy:** use the data only at the physical scale and with the boundary conditions actually supported. DWS flow/rainfall observations and vetted terrain can inform catchment scenarios; aggregate drinking-water quality cannot calibrate drainage hydraulics.
4. **Mycelix:** preserve immutable attribution, who reviewed the extraction, permissions, deliberation and any later authorised action. Keep public reports distinct from restricted topology and utility telemetry.
5. **Hydraulic solvers:** EPANET needs authoritative topology, elevations, demands, pipe properties, operating rules and suitable pressure/flow measurements to calibrate a pressurised network. SWMM needs surveyed/credible catchments, drainage geometry and inverts, rainfall forcing, boundary/tailwater conditions, and observed depth/flow/overflow data. The aggregate water-quality bundle is not a substitute for either.

## What to request from an authorised utility partner

The next high-value dataset is a privacy- and security-reviewed, read-only and appropriately aggregated export containing:

- Asset IDs and pipe/conduit material, diameter, length, elevation/invert, install date and condition grade, with restricted topology access.
- Time-aligned pressure/flow/demand, reservoir/tank level, pump state and outage intervals for a defined study area.
- Geocoded or area-coded water bursts, sewer blockages/overflows, repair start/completion and verified post-repair performance.
- Rain-gauge/hyetograph and stormwater depth/flow/ponding observations for a catchment; surveyed inverts and outfalls for SWMM.
- Water-quality raw results with sample ID, sampled-at time, location/zone, method, detection limit, units and QA flags—shared under the relevant terms.
- Data dictionary, time zone, missingness conventions, calibration records, licence/permission, data owner and a named qualified engineering reviewer.

We should not publish exact critical-asset topology, detailed weaknesses, credentials, or raw operational telemetry. Start with a tiny, agreed pilot area, least-privilege access, and a defined retention policy.

## Acceptance gates

- Preserve the original source URL, publication date, observed period, access/use terms, extraction method and artifact SHA-256.
- Check schema and semantic consistency before admitting a record.
- Keep observed, reported, derived, simulated, planned, and human-verified claims distinct.
- Do not coerce missing data to zero or combine different denominators/units.
- Treat current dashboards as snapshots with an extraction timestamp; never imply downloaded data remain live.
- Keep all real network modelling read-only until an authorised owner and qualified professional approve the scope.

The current bundle is an initial real aggregate-data ingestion, not a hydraulic model calibration and not evidence of a projected savings amount.
