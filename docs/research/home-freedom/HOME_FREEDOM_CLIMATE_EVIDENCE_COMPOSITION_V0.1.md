# Home Freedom climate evidence composition contract v0.1

**Study:** HOME-FREEDOM-CLIMATE-EVIDENCE-001  
**Parent:** [Home Freedom Compiler #800](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/800)  
**Decision layer:** [Robust Pareto benchmark PR #931](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/931)  
**Climate source owner:** [City Heat Retrofit Atlas #593](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/593)  
**Maturity:** SYNTHETIC_CONTRACT  
**Execution:** NOT_INGESTED_NOT_SIMULATED  
**Version:** 0.1.0

## 1. Mission and boundary

Define the cross-owner contract that connects climate evidence members and time-aligned service-disruption events to Home Freedom decision scenarios. The contract composes owner-qualified references. It is not a climate engine, weather-data catalogue, physical twin, building simulator or metrology system.

This increment does not download, ingest, quality-control or license weather datasets; calculate climate hazards or return periods; infer courtyard microclimate from a distant station; run EnergyPlus; admit instruments or clocks; decide resident comfort thresholds or medical safety; or certify a design/building.

**Owner boundaries — consume, do not duplicate:**
- Sol-Atlas #593 owns Johannesburg heat/source readiness, spatial resolution, candidate climate records and climate evidence admission.
- Symtropy #1848 owns time-varying spatial resilience and habitability/service trajectories.
- Symtropy #1900 owns EnergyPlus 26.2.0 adapter qualification and solver-domain checks.
- Mycelix #6034 / PR #6040 owns instrument, calibration, installation and sample-level evidence.
- Mycelix #6054 owns stream identity, timebase, clock epochs, sequence and aggregation-input eligibility.
- Sol-Atlas #633 owns read-only in-use and post-occupancy comparability.
- Sol-Atlas #704/#705 owns existing semantic interoperability crosswalks.

Only opaque owner references and exact revisions/digests cross these boundaries. Downstream consumers must not reinterpret upstream admission states.

## 2. Paradigm shift: an ensemble of evidence

A climate input is typed by what it is and what it can support. These classes are not interchangeable:

| Evidence class | Intended role | Prohibited inference |
|---|---|---|
| Typical meteorological year (TMY/TMYx) | Representative annual screening and relative design comparison | Heatwave, worst-case, future-climate or site-microclimate qualification |
| Observed station history | Actual historical episodes within documented station scope | Continuous or representative site truth without station, instrument, QC and representativeness review |
| Reanalysis | Spatially complete comparator and gap/context analysis | Direct local observation or parcel measurement |
| Observed extreme episode | Replay of a specified historical event | A return-period/design extreme unless the selection method supports that claim |
| Future projection | Explicit model/ensemble/pathway/period exploration | Deterministic forecast or unqualified single future |
| On-site microclimate | Parcel, room or courtyard observations from admitted instruments/installations | Site truth when instrument, placement, clock, coverage or consent is unresolved |
| Synthetic trace | Software contract and adversarial tests | Any real weather, climate, building or resident outcome |

Never merge these types into one generic weather-file field. When sources are compared, preserve them as parallel members. Aggregation is a separately qualified operation with a named method, scope, dependence assumptions and output uncertainty.

### Source research and limitations

- **ERA5 hourly single levels:** Copernicus provides global hourly reanalysis; the standard atmospheric grid is 0.25 degrees and a point time-series may use the nearest grid point. It is assimilation-based reanalysis, not a measurement at the requested parcel. Preserve exact product, grid point, coordinates, retrieval, coverage and processing.
- **ERA5-Land:** global hourly land reanalysis at 0.1 degrees (about 9 km), driven by ERA5 atmospheric forcing. It offers finer land-grid detail but does not become a local station or courtyard observation.
- **NOAA ISD / GHCNh transition:** ISD is the legacy global hourly/synoptic station record compiled from many contributing sources, with quality-control flags and uneven coverage. NCEI now describes Global Historical Climatology Network hourly (GHCNh) as the next-generation hourly/synoptic dataset that replaces ISD. At acquisition time, compare the relevant GHCNh and legacy ISD station inventories, variable availability, period-of-record, QC and transition status; do not assume the records are byte- or semantics-identical. Preserve original station IDs, inventory/history evidence, flags and missingness.
- **SAWS:** the South African Weather Service is the relevant national climate-data owner for local station context. Its climate-services page reports hourly wind, temperature, humidity, pressure and sunshine observations from 1950 onward for its archive, with station coverage varying. SAWS states accessed information is not for resale and commercial data provision may carry additional fees; obtain the applicable data policy, station history, quality information and written reuse scope before acquisition or publication.
- **TMY:** a typical year is assembled for typical-weather analysis, often by selecting representative months rather than preserving one continuous observed episode. Useful for annual screening; unsuitable as a heatwave/extreme replay.
- **On-site microclimate:** must reference Mycelix's exact instrument passport, installation, sample and stream-timebase receipts. An airport-to-courtyard adjustment without an independently qualified method is not a measurement.
- **Future climate:** each member requires explicit model, ensemble member, pathway/scenario, projection period, downscaling and bias-correction method where used. Preserve individual ensemble members; do not hide spread in one average.

The current Climate.OneBuilding South Africa catalogue lists 2011–2025 TMYx entries for both Johannesburg-Tambo International Airport (WMO 683680) and Johannesburg Botanical Gardens (WMO 683610), alongside rolling 2004–2018, 2007–2021 and 2009–2023 windows and an all-available-period file. Its source description says TMYx combines hourly station meteorology from the legacy NCEI/NOAA ISD with gridded ERA5 solar-radiation data. The source guide says the selected source year for each month is recorded in EPW COMMENT 1 and the STAT file; a 2011–2025 TMYx is therefore a 15-year selection pool, not a continuous 15-year event series. Preserve that source construction lineage and every selected-month year.

The catalogue entries are now confirmed as current candidate listings, but the ZIP download did not complete in this work session, so no EPW/STAT header, coordinates/elevation, month-source years, variable coverage, digest or embedded station metadata have been inspected. Licence/redistribution and representativeness also remain unresolved. Neither file is admitted by this contract.

## 3. Evidence member contract

The machine-readable schema requires each climate member to carry:

1. **Identity and source:** immutable member ID; evidence class; publisher/product; exact source locator and version/period; retrieval time; exact content digest only when bytes are acquired; rights state and rights-record reference.
2. **Spatial support:** point/grid/area/site support, coordinates where known, elevation, grid spacing and representativeness disposition. Nominal grid resolution is not effective site accuracy.
3. **Temporal support:** UTC event/coverage bounds, sampling interval, time basis and explicit coverage state. Local civil time is presentation metadata, not universal ordering authority.
4. **Quality:** QC state, missingness, station-history and instrument-history review, and explicit imputation policy. Missing data is not zero and cannot silently become a continuous trace.
5. **Uncertainty:** separately typed measurement, sampling, spatial-representativeness, reanalysis, model-discrepancy, projection-spread and missingness components. Confidence, prediction, measurement and engineering bounds are distinct semantics.
6. **Rights and purpose ceiling:** licence and redistribution/commercial-use state, plus the purposes the member can support.
7. **Class-specific metadata:** TMY selection period/method; station/history/QC references; reanalysis product/grid metadata; extreme episode selector/source member; projection model/member/pathway/period; or microclimate instrument/install/sample/timebase receipts.

A member marked CANDIDATE_ONLY, OWNER_REVIEW_REQUIRED, UNKNOWN, rights-restricted or out-of-domain must not be promoted by a downstream consumer. ADMITTED_FOR_SCREENING and ADMITTED_FOR_MODEL_INPUT are different purpose-specific states.

## 4. Episode selection and compound scenarios

An episode record must be reproducible from its parent source-member ID and exact digest; event-class definition/version; selection-rule reference and threshold/quantile basis; source coverage; UTC start/end and duration; missingness/QC/timebase treatment; and owner/reviewer disposition.

Do not hard-code one universal heatwave or hot-night threshold into the compiler. The appropriate definition depends on climate context, source quality, building/use scope and the service or comfort question. The method must be selected and reviewed by climate and building-science owners.

Compound scenarios combine a climate member with owner-native service-disruption traces (grid outage, water interruption, air-quality-constrained ventilation, equipment failure or maintenance delay). Each event retains its owner, exact event reference and timebase. Cross-stream alignment requires a qualified Mycelix #6054 receipt. Unknown alignment means the compound episode is not admitted as aligned.

Composition modes:
- SINGLE_MEMBER: one source member for one narrowly defined scenario.
- PARALLEL_NOT_POOLED: compare station and reanalysis members side-by-side.
- MEMBERWISE_STRESS_SET: retain discrete event or projection members as separate mandatory scenarios.
- APPROVED_STATISTICAL_AGGREGATION: allowed only with a named method and explicit dependence/correlation and output-uncertainty contract.

Default is parallel, not pooled. Statistical aggregation without a named method is invalid.

## 5. Binding to robust decision PR #931

PR #931 v0.1 already accepts an opaque exogenous_trace_id per scenario. This increment does not change that closed decision-input contract. The integration boundary is:

```text
climate evidence members (Sol-Atlas #593)
        + owner-native service event traces
        + exact timebase/sequence receipts (Mycelix #6054)
        + composition method and revision
                    |
                    v
       canonical scenario composition receipt
                    |
                    v
          opaque exogenous_trace_id
                    |
                    v
        Home Freedom robust decision #931
```

For empirical integration, the opaque trace identity must resolve to an immutable composition receipt binding sorted exact member IDs and content digests; source class/version/rights/admission; time windows, units, timebases, QC/missingness and uncertainty types; service event IDs and alignment method; composition/episode selection revision; and model-input revision/invalidation dependencies.

Synthetic labels in v0.1 are identifiers only, not hashes or source-data receipts. A changed member, rights state, QC decision, episode boundary, timebase mapping, service event or composition method invalidates dependent results, not unrelated evidence.

## 6. Adversarial P0 corpus

1. TMY offered as heatwave/extreme event.
2. TMY months treated as one continuous observed episode.
3. Station moved/re-sited but history not reviewed.
4. Station instrument or reporting method changed mid-series.
5. QC flags dropped or downgraded during export.
6. Missing intervals silently filled with zeros or unqualified interpolation.
7. Station used outside spatial/temporal representativeness.
8. Nearest ERA5 grid point presented as a parcel measurement.
9. ERA5-Land presented as direct on-site observation.
10. ERA5 and ERA5-Land treated as independent observations without method review.
11. Future projection missing model/member/pathway/period.
12. Projection ensemble average hides an adverse member.
13. Episode lacks reproducible selector or source digest.
14. Hot-night sequence truncated by local-time/day-boundary conversion.
15. Cold snap omitted because only heat scenarios were selected.
16. Grid and water traces aligned by local wall-clock labels without a timebase receipt.
17. Reboot/clock correction creates duplicate or reordered intervals.
18. Compound scenario uses climate and outage records with disjoint coverage.
19. On-site sensor lacks calibration, installation geometry or stream qualification.
20. Expired/moved/replaced instrument still supports current microclimate evidence.
21. Rights are unknown or prohibit intended commercial/redistribution use.
22. Source digest changes but old model output remains current.
23. One candidate is evaluated under a more favourable scenario set than another.
24. Statistical aggregation selected without named method/dependence contract.
25. Measurement, model and projection uncertainty collapsed into one unsupported interval.

## 7. Benchmark ladder and performance targets

Targets below are proposed gates, not achieved results.

| Stage | Benchmark | Proposed target |
|---|---|---|
| P0 contract | Schema + frozen known-answer + hostile mutations | 100% independent schema/fixture agreement; zero false admissions; preserve all positive controls |
| P0 replay | Permutation, identity, invalidation and timebase controls | 100% exact deterministic replay; zero stale dependent claims after mutation |
| P1 source readiness | Station, reanalysis, TMY, projection and on-site strata | 100% provenance/rights/quality/time/spatial fields complete before purpose admission |
| P2 building model | Held-out episodes and version-pinned EnergyPlus outputs | Error and interval coverage by archetype, season, event family and source class; zero out-of-domain robust pass |
| P3 physical test cell | Independent sensors and controlled hot/cold/outage episodes | Declared uncertainty coverage demonstrated against independent measurements; report repeatability, lag and missingness |
| P4 occupied outcomes | Consent-based post-occupancy comparison | Household-resolved service vectors and uncertainty; no group average masks an essential-service failure |
| P2 workflow stretch | Matched baseline vs integrated evidence workflow | Hypothesis: 20% lower median time to decision-ready Pareto set with equal input maturity, human effort and compute, and no quality/hard-gate regression |

Measure a service vector rather than one thermal score: operative/air temperature and occupied hours outside resident-approved bands; degree-hours and event duration with threshold semantics; hot-night retention and overnight recovery; cold-period service deficit; ventilation under outdoor air-quality constraints; essential water/electricity availability; peak demand, outage survival and recovery time; repair burden; and per-scenario/per-household worst-stratum outcomes.

ASHRAE 55 may inform a declared comfort evaluation, but is not a universal medical safety threshold. Scope, occupancy, clothing/metabolic assumptions, air speed, humidity and applicability must be explicit. A zero observed false-pass count is not proof of zero population risk; publish denominators and confidence bounds by failure family.

## 8. Industry comparison: compete at the composition boundary

Do not try to replace Autodesk Forma's early site/daylight workflows, TestFit's site configuration/development analysis, Finch3D's constraint-driven layout generation, EnergyPlus/OpenStudio building-energy simulation or BOPTEST's standardized control benchmarking.

Use version-pinned adapters and matched evaluations. ASHRAE Standard 140 is a software evaluation/verification method; passing it does not validate a particular building, local weather file, site or occupied outcome. BOPTEST evaluates control strategies on declared emulators; it is not a whole-home design oracle.

The differentiator to prove is preservation of resident hard constraints, source/authority lineage, climate-member identity, scenario comparability, dependency invalidation, uncertainty/out-of-domain abstention and resident-owned Pareto trade-offs through as-built and occupied outcomes. This is not a current claim of SOTA superiority.

## 9. Current evidence status

As of 17 November 2026: no Highveld weather extraction has been admitted; SAWS access, histories and rights need direct confirmation; the two Johannesburg TMYx items remain uninspected candidates; no parcel survey, station review, licence receipt, on-site instrument, EnergyPlus Highveld run, physical test-cell trace or occupied outcome is admitted. PR #931 software tests are separate from all physical/weather evidence states.

## 10. Source register

- Copernicus ERA5 hourly single-level time-series: https://cds.climate.copernicus.eu/datasets/reanalysis-era5-single-levels-timeseries
- Copernicus ERA5-Land hourly: https://cds.climate.copernicus.eu/datasets/reanalysis-era5-land
- Climate.OneBuilding South Africa TMYx catalogue: https://www.climate.onebuilding.org/WMO_Region_1_Africa/ZAF_South_Africa/index.html
- Climate.OneBuilding TMYx source/method notes: https://www.climate.onebuilding.org/sources/
- NOAA Integrated Surface Database: https://www.ncei.noaa.gov/products/land-based-station/integrated-surface-database
- NOAA Global Historical Climatology Network hourly (GHCNh): https://www.ncei.noaa.gov/products/global-historical-climatology-network-hourly
- NOAA station histories / HOMR: https://www.ncei.noaa.gov/products/land-based-station/station-histories
- South African Weather Service climate-data services: https://testing.weathersa.co.za/home/aboutclimateatsaws
- EnergyPlus auxiliary programs and weather-data processing: https://energyplus.readthedocs.io/en/latest/auxiliary-programs/auxiliary-programs.html
- ASHRAE building energy simulation and Standard 140 resources: https://www.ashrae.org/technical-resources/bookstore/ashrae-building-energy-simulation-and-performance-modeling-resources
- BOPTEST: https://ibpsa.github.io/project1-boptest/

## 11. Claim ceiling

This is a synthetic cross-owner composition contract. No weather has been downloaded, no climate episode replayed, no local climate qualified, no building simulation run, no instrument admitted and no dwelling outcome measured. It makes no claim of site suitability, thermal safety, future resilience, affordability, buildability, certification or SOTA superiority.
