# South African source-readiness register for Highveld v0.1

**As-of date:** 12 November 2026. **Admission state:** discovery/context only. No raw source file has been admitted to the synthetic event corpus, and no local Highveld event trace is established by this register.

## Why this register exists

A useful research programme needs local grounding without confusing regional context, household survey responses, municipal notices, system telemetry and dwelling measurements. The sources below justify research questions and identify acquisition pathways; they are not interchangeable observations.

## Candidate evidence register

| Source | What it can support now | Scope / resolution limitation | Admission status and next gate |
|---|---|---|---|
| Statistics South Africa, General Household Survey 2025 | Population-survey context about reported water-service interruptions. The published 2025 report reports 32.5% for Gauteng on the measure concerning water interruptions lasting at least two days or more than 15 days in total; national value is 37.6%. [Stats SA GHS 2025, p. 34](https://www.statssa.gov.za/publications/P0318/P03182025.pdf) | A household survey estimate, not a dated event log, tap flow series, municipal hydraulic record or Highveld dwelling observation. It cannot time-align an interruption with a specific power outage or heat event. | CONTEXT_ONLY. Preserve survey year, questionnaire wording, population/denominator, weights, geography, uncertainty, release revision and licence/terms. Use only as population context; never as a per-event probability or individual household state. |
| Stats SA GHS 2025 microdata catalogue / DataFirst | Identifies the 2025 household/person data products and the water-interruption variables. [DataFirst catalogue](https://www.datafirst.uct.ac.za/dataportal/index.php/catalog/1144/variable/V360) | Variable metadata includes case counts; DataFirst explicitly warns that those counts are not population estimates. Microdata access and terms must be checked separately. | DISCOVERY_ONLY. Obtain exact approved extract, codebook, weighting/design metadata, access terms and digest before analysis. Do not use unweighted case counts as prevalence. |
| South African Government joint statement, 10 Sep 2026 | Qualitative context that national/provincial/City of Johannesburg actors discussed water-supply disruptions arising from intermittent power trips affecting Rand Water supply systems. [Government statement](https://www.gov.za/news/media-statements/government-measures-mitigate-water-supply-disruptions-resulting-power-trips) | A public statement describing a cross-service dependency and meeting; it is not a timestamped utility event series, household/tap interruption record, causal dataset or measured duration. | CONTEXT_ONLY. Use to prioritize a dependency question. Acquire primary utility source snapshots and exact geographic/time scope before admitting any observed event. |
| Eskom Data Portal | System-operator supply/demand and selected generation/outage context; requests can cover up to five years. [Eskom Data Portal](https://www.eskom.co.za/dataportal/) | National/system-level electricity context is not a City Power/Johannesburg feeder, street, customer or dwelling interruption. | DISCOVERY_ONLY for context. Request exact dataset, definitions, timezone, granularity, revision, source rights and coverage. No local-scope promotion. |
| Johannesburg Water notices and status boards | Official planned/unplanned supply-notice and status-board context. [Emergencies](https://www.johannesburgwater.co.za/emergencies/) · [Daily notices](https://www.johannesburgwater.co.za/daily-water-notices/) · [Interruptions](https://www.johannesburgwater.co.za/interruptions/) | A system/area notice does not prove a specific site's tap delivered or failed. A missing notice is not a normal-service observation. Daily resolution cannot establish hourly onset/end. | DISCOVERY_ONLY. Snapshot the actual source page or dataset, notice ID, area, issue/update time, event time, scope and exact rights. Link only at supported spatial/time resolution; preserve no_report. |
| SAWS / NCCIS / GreenBook / CSIR climate products | Potential source of weather observations, historical climate and future projections under separately documented products. [NCCIS climate data](https://nccis.environment.gov.za/climate-services/download-data) · [NCCIS projections](https://nccis.environment.gov.za/climate-services/climate-projections) | Product-specific spatial resolution, baseline, model ensemble/pathway, station/grid location, missingness, bias and rights differ. TMY, historical extremes and future climate projections are different evidence types. | NOT_ADMITTED. Sol-Atlas #611 remains the source-admission owner. Record product, station/grid, coordinates/elevation, temporal basis, QC, revision, representativeness, rights and digest. Do not relabel RCP as SSP or TMY as an event. |
| Mycelix #6025 building instruments / unoccupied test cell #4531 | Future route to site-/room-/tap-/component-level measurements after instrument and installation admission. | No instrument is physically qualified by being listed in the issue; no Highveld physical measurement is claimed here. | NOT_ADMITTED. Require channel passport, measurand/unit, reference method, calibration/uncertainty, placement, clock, raw-vs-derived, offline/fault states and consent/purpose. Run only under reviewed safe, unoccupied test procedures. |

## Evidence-class decision

For the current register:

- The Stats SA survey and government statement are **ContextOnly** for an event corpus: they do not identify compatible event windows at Highveld dwelling or service scope.
- Utility and climate portals are **discovery candidates**, not admitted ObservedJointEvent or ObservedPartialEvent records.
- All seven event traces currently in SYNTHETIC_CORPUS_V0.1.json remain SyntheticControlledEvent.
- No empirical joint heat/grid/water/repair trace has been assembled.
- No survey response has been converted into a timestamped outage.
- No regional/system event has been promoted to feeder, dwelling, room or tap scope.

## Research and acquisition sequence

1. **Water population context:** retrieve and preserve the official GHS 2025 release, methodology, questionnaire, weights/design and any lawful microdata access. Report uncertainty and denominator. Do not use survey prevalence as a hazard prior without a justified statistical model.
2. **Cross-service dependency:** retrieve primary source records for the 8–10 September 2026 power/water disruption episode from Rand Water, Johannesburg Water, City Power, Eskom and DWS where publicly available. Preserve each issuer's original scope; do not synthesize a joined event until source intervals, geography and independence are compatible.
3. **Local electricity and water:** seek a permissioned feeder/customer and tap/connection measurement pathway for a consenting unoccupied test site. Public notices and system metrics remain context.
4. **Climate:** use the exact NCCIS/SAWS/CSIR product admitted by #611, separate historical observations, typical-year data, design extremes and future projections.
5. **Physical outcomes:** use #6025 instrument admission and #4531 unoccupied test cell with reference methods, uncertainty and pre-registered procedures.
6. **Benchmark:** only after evidence admission, compare paired candidate arms through the existing #1848/#1849 trajectory and #700/#633 outcome owners.

## Non-negotiable anti-inference rules

- Survey prevalence is not an event timestamp or a per-household prediction.
- A government statement about power-related water disruption is not a matched event record.
- National generation adequacy is not a household/feeder outage.
- An area notice is not proof of tap delivery at a property.
- No report is not observed normal.
- Time overlap is not causation or common cause.
- TMY is not a historical extreme, and a future projection is not an observation.
- No source enters public fixtures unless its licence, permitted purpose and redistribution rights are clear.

## Claim ceiling

This register supports research prioritization and acquisition planning only. It does not establish observed Highveld joint events, their rate or probability, any dwelling's service state, thermal/air/water safety, health outcomes, climate representativeness or physical resilience.

## Additional primary-source acquisition leads (12 November 2026)

| Source | What it adds | Scope / limitation | Admission treatment |
|---|---|---|---|
| DWS, *Monthly Dashboard for a Water Secure Gauteng*, 31 May 2026 (Rev 02) | Monthly municipality-level water-use context, a 2014–April 2026 demand trend, four Rand Water strategic reservoir-system storage series since January 2025, and the 2026 planned-maintenance window. [DWS dashboard](https://www.dws.gov.za/documents/2026-05%20Gauteng%20Dashboard.pdf) | System/reservoir and municipal aggregates. The dashboard states that municipalities communicate localized supply outages separately. It does not establish a property's tap state, a specific feeder outage, or hourly household event intervals. | DISCOVERY_ONLY / SYSTEM_CONTEXT. Preserve PDF revision, as-of date, page, metric definition, aggregation, units and publication digest. Do not convert reservoir level or maintenance dates into a dwelling interruption. |
| Eskom Data Portal, weekly unplanned outages | Current-financial-year weekly trend and downloadable data file, useful for broad system context and source discovery. [Weekly unplanned outages](https://www.eskom.co.za/dataportal/outage-performance/weekly-unplanned-outages/) | Full-week trend, not local distribution feeder or household connection telemetry; the page describes spread and average weekly trend. | DISCOVERY_ONLY / SYSTEM_CONTEXT. Pin the download, fiscal-year boundary, outage definition, refresh date and geographic/system scope. No local-scope promotion. |
| Stats SA GHS 2025 publication package | The official release page lists the main report, key findings, presentation, addendum weighted tables and 2002–2025 selected-variable time series. [GHS 2025 release page](https://www.statssa.gov.za/?PPN=P0318&SCH=74563&page_id=1854) | Weighted published tables are still survey summaries; time-series tables are not event logs, feeder data or individual dwelling observations. | CONTEXT_ONLY. Preserve exact workbook/table revision, variable wording, denominator, weights, survey design and uncertainty before reusing. |

### Acquisition order refinement

1. Preserve the GHS 2025 report, addendum weighted tables, selected-variable time series, questionnaire and methodology as one versioned survey package.
2. Preserve each available monthly DWS Gauteng dashboard revision. Separate the four reservoir-system storage series, municipal demand and maintenance notes; do not join them as though they were tap-flow observations.
3. Snapshot the Eskom weekly unplanned-outage download with the fiscal-year definition and its data dictionary. Treat it as system context, not a City Power feeder outage series.
4. For the September 2026 Eikenhof episode, seek issuer-native records from Rand Water, City Power, Johannesburg Water, DWS and Eskom. Only build a joint trace after temporal coverage, locations, scope and clock definitions are independently reconciled.
