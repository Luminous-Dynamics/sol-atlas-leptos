# City heat and retrofit atlas — evidence profile v0.1

**Status:** domain contract and benchmark design. No Johannesburg thermal map has been ingested or validated by this document.

## 1. What the atlas is

A spatial projection of *qualified* outdoor exposure, building-cohort evidence, resident-defined thermal requirements, building-model trajectories, measured conditions and retrofit delivery capability.

It is not a climate model, EnergyPlus replacement, indoor-temperature prediction service, building register, population estimator, medical risk classifier or automatic retrofit approval.

The pipeline has separate evidence authorities:

`climate source -> spatial exposure -> cohort / resident context -> qualified building model -> thermal service trajectory -> retrofit option -> supply and installation -> measurement -> observed outcome`

Each arrow is a contract boundary. A downstream artefact cannot strengthen upstream evidence or change its spatial/temporal resolution.

## 2. Source/evidence classes

### Weather and climate

Allowed classes:
- `observed_station`: measured at a named instrument/site/time;
- `observed_reanalysis`: gridded reconstruction/assimilation with product and resolution stated;
- `typical_meteorological_year`: synthetic typical-year sequence for energy screening;
- `design_extreme`: a design weather condition with method/period stated;
- `observed_event`: actual heatwave/hot-night episode with date bounds;
- `future_projection`: climate-model/scenario/period output with ensemble/member and bias-correction status stated;
- `synthetic_fixture`: test data only.

These are not interchangeable. In particular:
- a TMY is not a heatwave;
- a design condition is not an observed event;
- a future projection is not a forecast for an exact date;
- a nearby station is not automatically representative of every neighbourhood;
- missing data are not zero exposure.

Climate.OneBuilding's 2026 TMYx release provides hourly building-simulation weather files, including recent 2011–2025 periods where available and ERA5-derived solar data. Use it as a candidate TMY source, with the exact location/product/date/digest/licence pinned. It is not by itself an extreme-event or future-climate dataset.

### Spatial exposure

A weather raster or station-derived surface must carry:
- source resolution and native spatial support;
- interpolation/downscaling method and validation;
- time aggregation and missing-data treatment;
- boundary/geography version and CRS;
- surface/air/indoor measurement type;
- uncertainty and permitted display precision.

An interpolation can create a continuous surface, but it does not create new measured observations. Show native/source resolution and model uncertainty.

### Population and socioeconomic context

Population or survey evidence retains:
- publisher, geography and boundary vintage;
- reference year, collection period, sample frame and weights;
- confidence intervals/standard errors/design effect when applicable;
- suppressed or unstable cells;
- explicit aggregation level and allowed join precision.

Stats SA's ward-level Small Area Population Estimates for 2022 use 2020 ward boundaries and raking/IPF against Census 2011/2022. Stats SA describes instability, variability, bias and undercount risk at very small areas. They remain ward-level estimates: do not assign them to building polygons or divide by building count without a separately validated downscaling method.

GCRO QoL7 2023/24 is a weighted, anonymised public-use survey. Use the technical report, questionnaire, weights and design. Do not geocode respondent-level records or interpret sample estimates as a census.

### Building cohort

A cohort is a classification supported by evidence (e.g. building typology or era) and a coverage report. Required fields:
- classification scheme and revision;
- source, date and mapping method;
- geographic support/precision;
- cohort coverage and unknown share;
- any validation sample and confusion matrix;
- permitted inferences and prohibited fields.

No roof colour, wall material, insulation, orientation, occupancy, tenure, dwelling condition or window-opening behavior may be invented from a coarse building footprint or land-cover class.

### Thermal model

Consume Symtropy #1848 and the EnergyPlus adapter #1900. A model receipt is admitted only if it binds:
- solver name/version/build/package digest;
- input model, geometry and construction revisions;
- weather profile ID/class/source period/digest;
- occupancy and internal gains schedule revision;
- operating/control assumptions;
- timestep, warm-up and convergence;
- selected outputs/units and missingness;
- validation profile and comparison dataset;
- calibration set and separate held-out set;
- error/residual metrics, uncertainty and applicability envelope.

Process exit code zero is not sufficient. A missing output, severe convergence warning, unsupported object, incomplete weather or out-of-domain condition blocks the affected claim.

### Physical measurements

A sensor record binds:
- instrument ID/type/serial or anonymized device token;
- calibration certificate/revision/validity;
- measured variable, units, accuracy/uncertainty and sampling interval;
- placement and height class, orientation/shielding and room type;
- install/remove timestamps;
- time synchronization and data-quality flags;
- consent/retention/privacy class;
- building/room subject token and authorized spatial aggregation.

A measured value is not automatically representative of the whole building or all occupants. A sensor reading cannot be called medical diagnosis or code compliance.

### Retrofit / delivery

An option and completed intervention must carry:
- exact building/cohort scope and baseline;
- design and BOM revision;
- material/product/lot identities and test/qualification status;
- installer/crew capability and authorization evidence;
- installation/inspection/commissioning receipt;
- cost, subsidies/finance assumptions, downtime and occupant disruption;
- expected and actual maintenance/replacement pathway;
- before/after measurement protocol and comparison scope;
- any displacement, rent, tenure or service-access safeguards.

A predicted retrofit is not an installed intervention. An installed measure without inspection is not an accepted outcome.

## 3. Claim levels

| Level | What can be said | Minimum supporting evidence |
|---|---|---|
| H0 — source inventory | A dataset or record is listed | Publisher, access method, retrieval date, rights state |
| H1 — coarse exposure | A named geography has a source-supported outdoor exposure summary | Weather class and digest, native resolution, valid spatial method, uncertainty |
| H2 — cohort context | A supported building/population cohort intersects a geography | Cohort/population source, vintage, validation and coverage, allowed spatial join |
| H3 — modelled thermal service | A declared model profile predicts a trajectory for a specified building/model scope | H2 context plus qualified solver receipt, exact weather/schedules/model scope, convergence and validation |
| H4 — measured indoor conditions | Sensors recorded conditions in named, privacy-protected rooms/buildings and periods | Calibration, placement, QC, consent and spatial aggregation limits |
| H5 — intervention effect | A retrofit changed measured outcomes relative to an admissible comparison | H4 pre/post, installed/accepted intervention, matched/staggered comparison, uncertainty and confounder plan |
| H6 — programme impact | A programme changed population-level outcomes | Sampling/causal design, displacement and distribution analysis, external review |

The map must show claim level. It must not visually blend H1 exposure and H4 measured indoor conditions as if they were equivalent. H6 programme-impact analysis is reserved and unsupported in v0.1; do not infer programme causality from mapped exposure, modelled trajectories or simple pre/post change.

## 4. Spatial and temporal joins

Every join request must declare:
- source layer identities/revisions;
- target geometry identity/revision;
- spatial predicate and CRS transform;
- source and target spatial support;
- time window and aggregation function;
- boundary crosswalk if geographies differ;
- missingness and uncertainty propagation;
- maximum claim resolution.

A ward-level population estimate joined to a 100 m heat grid remains a ward-level population context alongside a grid-level exposure surface; it does not become 100 m population. A TMY model joined to an observed heat event remains TMY model evidence, not event-specific prediction.

## 5. Climate and thermal benchmark strategy

The passive-cooling literature should motivate a measurement-first benchmark, not a giant simulation-only search. A randomized pilot published on 5 January 2026 in 40 rural African houses compared cool roofs, cross-ventilation, mat ceilings and controls. It reported lower daytime/nighttime heat index in the cool-roof arm, while the mat-ceiling arm had a day/night trade-off and cross-ventilation had no detected heat-index difference in that setting. This is useful for hypothesis generation, not a Johannesburg effect size or a universal ranking. The study supports a matched, instrumented test-cell/pilot approach in which local climate, housing type, installation quality, occupancy, hot nights and water/power availability are recorded.

Therefore:
- reproduce published results in their own context before transferring;
- include hot nights and recovery, not only daytime peaks;
- evaluate packages (roof, shading, insulation, ventilation, openings, mass and controls) as interacting systems;
- include winter cold, indoor air quality, water and grid dependence;
- calibrate on one dataset and validate on a separate held-out dataset;
- measure as-built installation quality and actual resident operation;
- use the same weather episode and common assumptions when comparing alternatives.

## 6. Metrics — no blended heat-safety score

Keep these separate:
- outdoor exposure: heatwave days/nights, hot-night persistence, WBGT/heat-index/other selected metric as defined by its method;
- indoor thermal service: operative/air temperature distribution, humidity, overheating degree-hours/duration, adaptive-comfort exceedance where applicable;
- night recovery and next-day heat accumulation;
- indoor air quality and ventilation availability;
- heating/cooling energy, peak demand, outage reserve and water demand;
- affordability: capital, operating cost, maintenance and household utility burden;
- robustness: same performance under declared hot/cold/outage/weather uncertainty;
- implementation: qualified material supply, installer capacity, lead time, inspection and rework;
- distribution: group and cohort-specific exposure/service gaps, worst-served quantiles and unknown share;
- lifecycle: embodied impacts, service life, replacement and repairability.

Do not use a single threshold or one composite score as a medical or building-code safety determination. The applicable comfort/service profile and professional review are explicit.

## 7. Benchmark baselines

- South African building energy compliance: verify the official current SANS 10400-XA edition and text through SABS and competent local professionals before encoding requirements. The IEA policy entry still records the 2021 edition as in force, while 2026 trade/industry references mention a 2026 edition. This discrepancy must remain unresolved until primary authority is checked.
- Whole-building simulation: pin EnergyPlus through Symtropy #1900 and use the applicable, rights-cleared subset of ASHRAE Standard 140 for software capability. Standard 140 does not validate a Johannesburg dwelling or a retrofit package on its own.
- Weather: compare the exact site/date TMYx and observed event separately; include extreme and future scenarios only when source metadata support them.
- Field: compare calibrated pre/post indoor measurements and resident-defined service outcomes; simulated prediction is not measurement.
- Causal: use matched buildings or staggered rollout where ethically/practically appropriate; control for occupancy, weather, equipment and concurrent changes.

## 8. Initial adversarial cases

The first 24-case corpus is specified in [CITY-HEAT-RETROFIT-ATLAS-001 #593](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/593). Include, at minimum:
- TMY passed as heatwave;
- ward population promoted to building counts;
- survey sample treated as complete population;
- modeled temperature labeled measured;
- wrong weather/building revision;
- stale sensor calibration;
- non-converged EnergyPlus result;
- no overheating metric for an energy-only claim;
- a cool-roof day/night trade-off;
- insulation worsens hot-season performance;
- ventilation blocked by outdoor pollution/security;
- evaporative cooling exceeds water constraint;
- grid outage removes active cooling;
- mean improves while worst cohort declines;
- missing licence blocks redistribution;
- post-intervention observation invalidates model claim.

## 9. Rights, governance and safeguards

- Source licence/access state is per layer, not inferred from portal-level access.
- Keep respondent/household sensor data private by default; release only approved aggregated outputs.
- A mapped vulnerable area is not a list of vulnerable individuals.
- Participation and resident consent do not replace engineering or statutory approval.
- Do not use the atlas to penalize, insure, police or exclude residents or neighborhoods.
- Retrofit prioritization must report who benefits, who bears disruption, and whether rents/displacement may change.
- Preserve an appeal/correction process for residents and source owners.

## 10. Claim boundary

The v0.1 profile implements H0–H5 evidence admission and joining. H6 is explicitly reserved and returns Unsupported until a dedicated population-level causal design and governance contract is qualified. It does not establish any Johannesburg indoor heat map or retrofit effect. H3 claims require the exact qualified EnergyPlus/Symtropy receipt; H4 requires valid sensors; H5 requires a measured intervention comparison; H6 requires population-level causal evidence and governance safeguards.
