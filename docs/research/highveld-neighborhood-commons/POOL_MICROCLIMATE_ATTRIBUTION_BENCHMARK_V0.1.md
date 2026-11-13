# Pool and water-feature pedestrian microclimate attribution benchmark v0.1

**Study ID:** HIGHVELD-POOL-MICROCLIMATE-ATTRIBUTION-001  
**Parent:** HIGHVELD-POOL-WATER-THERMAL-001 / HIGHVELD-THERMAL-SERVICE-REFUGE-001  
**Maturity:** BENCHMARK_PROTOCOL_NOT_EXECUTED  
**Climate context:** Johannesburg / Highveld; no site or occupant measurements admitted

## 1. The causal question

A water feature can be cooler than the surrounding air and still fail to improve occupied conditions. A dry-bulb reduction can be offset by increased humidity, a change in wind, direct solar exposure or warmer mean radiant temperature. Shade can produce most of the benefit without water. A pool may buffer building heat while adding little pedestrian cooling.

This benchmark therefore asks:

> At matched geometry, shade, weather and occupancy boundary, what incremental change in occupied pedestrian thermal conditions is attributable to each water-feature treatment compared with a competent no-water dry-shade control?

It separates three distinct effects:
1. **Thermal storage/regeneration:** heat absorbed and rejected by pool or cistern water.
2. **Pedestrian microclimate:** local air, radiation, humidity and wind conditions around the feature.
3. **Building service:** indoor operative conditions and cooling energy in the building.

No one layer can stand in for another. A decrease in water temperature is not evidence of pedestrian cooling; outdoor microclimate change is not proof of indoor cooling or health benefit.

## 2. Six matched comparison arms

| Arm | Role | What it isolates |
|---|---|---|
| DRY_SHADE_CONTROL | Primary control | Shade/envelope effect without water |
| COVERED_POOL_BUFFER | Test | Thermal mass and surface/cover influence with evaporation strongly limited |
| SHADED_EXPOSED_POOL | Test | Exposed pool surface, radiative/sensible/latent exchange |
| SHADED_NO_SPRAY_RILL | Test | Shallow moving water without aerosol-generating spray |
| CLOSED_LOOP_INDIRECT_EVAP_ASSIST | Conditional test | Indirect heat rejection with hydraulic separation; rights/quality/maintenance gates apply |
| AEROSOL_GENERATING_FOUNTAIN_NEGATIVE_CONTROL | Negative control | Must not qualify without a separately admitted water-management/aerosol safety programme |

The primary comparison is water treatment versus the dry-shade control with equal shade geometry and comparable surface area/placement. Secondary comparisons may separately vary shade, orientation, depth, cover, circulation or feature geometry, but they must not be used as the primary water-effect estimate.

The aerosol-generating arm is a negative control by default, not a recommendation. A failed or unknown water-management plan, aerosol classification, water quality, treatment or maintenance state blocks its operation and any beneficial-performance claim.

## 3. Spatial transect and vertical measurement design

Each treatment and control plot uses the same station geometry:
- UPWIND_BACKGROUND — local incoming microclimate boundary;
- FEATURE_EDGE — immediate edge of the water/cover/structure;
- NEAR_FIELD_1M — near-field pedestrian exposure;
- DOWNWIND_3M — downwind effect and spatial decay;
- OCCUPIED_ZONE — predeclared seated/standing use location;
- MATCHED_DRY_CONTROL — matched no-water point with equivalent shade/material exposure.

Record air temperature and humidity at 0.1 m, 0.6 m, 1.1 m and 1.7 m where the experiment's mounting/safety layout permits. Use the same heights, sensor shields, aspiration, response times and calibration approach in treatment and control plots. The existing courtyard channels can be instantiated repeatedly by station/location/height; a single channel name does not imply one sensor or one location.

Measure mean-radiant/operative conditions with a documented globe/radiometer method, not air temperature alone. Record wind speed and direction at the relevant exposure height; a weather-station wind value is not automatically a pedestrian-level wind value.

## 4. Six Highveld weather regimes

1. HIGH_SOLAR_HOT_DRY — high short-wave load and lower humidity.
2. HOT_HUMID_LOW_EVAPORATIVE_POTENTIAL — humidity constrains evaporation.
3. HOT_CALM_LOW_WIND — weak convective exchange and possible moisture accumulation.
4. MULTIDAY_HEAT_WAVE_HOT_NIGHTS — cumulative heat storage and poor night recovery.
5. CLOUDY_HIGH_LONGWAVE — reduced radiative-sky rejection.
6. COLD_CLEAR_NIGHT_OVERCOOLING — night recovery and overcooling transition.

Actual scenarios require owner-admitted weather packages and a local on-site boundary measurement. Do not invent probabilities or treat a TMY as a specific observed event.

## 5. Experimental design and causal attribution

Use a preregistered paired block design:
- assign treatment/control geometry and station layout before collecting outcomes;
- randomize treatment order or use matched parallel plots where thermal carryover makes crossover invalid;
- precondition initial pool/store temperature and cover state, then record any mismatch rather than silently normalizing it;
- use day or multi-day weather blocks as independent replication units;
- treat minute-level samples as autocorrelated repeated measurements, not independent replicates;
- record wind direction and flag blocks where upwind/downwind station assignment changes;
- preserve raw time series and instrument calibration/clock receipts;
- freeze analysis window, missingness policy, exclusion rules and a practically meaningful effect threshold before inspecting held-out outcomes.

For a matched pair, estimate the incremental treatment effect using the paired change relative to the upwind/background and dry-control change. A simple block-level difference-in-differences is one auditable baseline:

\[
\Delta_{\mathrm{water}} =
(Y_{\mathrm{treatment,post}}-Y_{\mathrm{treatment,baseline}})
-
(Y_{\mathrm{dry-control,post}}-Y_{\mathrm{dry-control,baseline}})
\]

The primary outcome set includes operative or mean-radiant conditions and humidity/dew point, with dry-bulb temperature reported as a separate supporting variable. If wind/exposure is materially imbalanced, report the block as non-comparable or use a preregistered adjustment model; do not infer a water effect from unadjusted before/after temperatures.

Report paired block estimates, confidence/uncertainty intervals, between-day variation, spatial decay, time-of-day and weather-bin interactions. Use day-blocked or event-blocked holdouts; do not randomly split adjacent minute observations across train/test partitions.

## 6. Required measurements and outcomes

Pedestrian microclimate:
- dry-bulb air temperature and relative humidity/dew point at each station and height;
- wind speed/direction with sensor exposure/orientation;
- globe/mean-radiant/operative temperature with method and uncertainty;
- global/direct/diffuse solar and downwelling long-wave boundary where the claim needs it;
- surrounding surface and water surface temperatures;
- station distance, height, shade, horizon and local geometry digests.

Water/thermal:
- water temperature top/mid/bottom and pool/store energy state;
- evaporation, make-up, leakage, purge and water level;
- pump/recirculation energy and flow;
- heat rejected by long-wave, sensible, latent, ground and hydronic paths where measured/claimed;
- water source, rights, quality and protected reserve.

Building service:
- indoor operative temperature, humidity, air speed and occupied schedule where a building-side claim is made;
- HVAC/cooling electricity, peak demand and unmet load under matched operation;
- envelope, ventilation, shading and control digest.

Report separately:
- change in dry-bulb temperature;
- change in mean-radiant/operative condition;
- humidity/dew-point penalty or benefit;
- wind change and air-speed distribution;
- occupied-hour thermal exposure/discomfort under a selected applicable method;
- spatial reach and decay of the effect;
- water litres per occupied person-hour of measured benefit (with denominator clearly stated);
- pump/fan/treatment electricity;
- building-side useful heat removal and indoor response, only when independently measured.

Do not collapse these into a single "cooling percentage."

## 7. Negative controls and hard gates

The benchmark must detect:
- lower dry-bulb air temperature but worse humidity/operative conditions being mislabeled an overall comfort improvement;
- treatment plot receiving more shade or lower solar exposure than the dry control;
- treatment wind or upwind boundary differing without a preregistered adjustment;
- water evaporation or leakage missing and silently treated as zero;
- non-water shade benefits attributed to water;
- outdoor air-temperature effects promoted to indoor building cooling;
- pool heat absorption reported as sustained heat rejection without cycle closure;
- aerosol-generating operation with unknown or failed water safety programme;
- minutes treated as independent replicates;
- post-hoc weather-bin or station exclusion;
- missing station height, location, calibration, clock or sensor exposure metadata.

Acceptance targets:
- zero water-effect claims without matched dry-shade control;
- zero comfort claims from dry-bulb alone;
- zero indoor-cooling claims without a measured building-side path;
- zero unqualified water/aerosol operation;
- 100% treatment/control input and instrument traceability;
- 100% pre-registered block and holdout compliance;
- no essential-water reserve use for discretionary operation;
- preserve positive controls: where the no-water shade arm is best, the benchmark reports that outcome rather than forcing a water-feature win.

## 8. Comparison with current research and industry

Urban-heat literature frequently uses land-surface temperature, coarse spatial aggregation or simulation-only effects. These are useful for regional screening but do not resolve pedestrian exposure beside a particular pool/rill or identify the water feature's incremental contribution after shade and wind are controlled. A 2026 review of 50 U.S. urban-heat studies finds 68% discuss mitigation without empirical testing and flags misalignment between spatial scales, cooling access and lived exposure. A 2026 review of passive radiative cooling finds that sky view, urban form, humidity and canyon geometry alter outcomes between material, building and neighborhood scales.

Highveld should not claim that a pool or fountain cools a courtyard based on a single sensor or modeled water temperature. Its differentiator is the paired, spatially explicit and instrument-traceable attribution of marginal water effects, with water/thermal and indoor-building boundaries separately closed.

Sources:
- [Urban heat, vulnerability, and mitigation in U.S. cities: A systematic review (September 2026)](https://doi.org/10.1016/j.scsadv.2026.100077)
- [Passive radiative cooling in the built environment: A multi-scale review (November 2026)](https://www.sciencedirect.com/science/article/pii/S2213138826006764)
- [Thermal Resilience of Residential Buildings Under Climatic Extremes and Power Outages (June 2026)](https://www.mdpi.com/1996-1073/19/13/3006)

## 9. Claim ceiling

**Status:** BENCHMARK_PROTOCOL_NOT_EXECUTED. The 35-channel courtyard instrumentation file is a template only. No plot, water feature, instrument, treatment period, weather boundary, pedestrian comfort effect or indoor cooling result has been admitted. Any future study involving people requires separate participant/ethics and consent governance; cameras and identifiable movement tracking are not required.
