# Johannesburg source register v0.1

This file is a research inventory of candidate sources, not an ingested or qualified local dataset. It records what is verifiably published and what still requires a rights, coverage, metadata and freshness check.

## Current source posture

- GCRO has a public catalogue across QoL survey data, administrative boundaries, public infrastructure, land cover/urban footprint, demographics and biodiversity. The portal itself notes that access varies by layer: some downloads are direct, others require following a landing/menu page or registration.
- GCRO QoL7 2023/24 is an anonymised public-use survey dataset under CC BY 4.0. It is a weighted household/adult survey, not building-level population truth.
- Stats SA published ward-level Small Area Population Estimates for 2022 using 2020 ward boundaries and raking/IPF. Stats SA itself warns of instability in small-area values; these estimates must not be silently downscaled to buildings.
- Johannesburg announced the ISDP/SPMS in August 2025. The announcement is not a licence grant or proof of anonymous API/download access for each layer.
- OSM is usable under ODbL with attribution and share-alike obligations; completeness and local positional correctness must be audited.
- No current official Johannesburg-area GTFS feed has been verified here. This is an explicit unresolved source gap, not a claim that no feed exists.

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
