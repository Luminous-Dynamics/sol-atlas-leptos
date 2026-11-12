# Radiative sky-sink validation benchmark v0.1

**Study ID:** HIGHVELD-RADIATIVE-SKY-SINK-001  
**Parent:** HIGHVELD-THERMAL-SINK-CASCADE-001  
**Maturity:** BENCHMARK_PROTOCOL_NOT_EXECUTED  
**Context:** Johannesburg / Highveld; no local panel or site measurements admitted

## 1. What is being benchmarked

The object under test is a **heat-rejection component and its coupled thermal service**, not a coating advertisement. A radiative panel can emit long-wave radiation through the atmospheric window. Whether it cools a building depends on solar absorption, atmospheric downwelling long-wave radiation, cloud, humidity, wind, panel temperature, sky-view factor, exchanger and loop losses, storage state, and the temperature level of the building-side load.

Keep five evidence levels distinct:

1. **Material:** solar reflectance, spectral emittance, weathering, soiling and aging.
2. **Exposed panel/component:** net heat rejection per area under measured sky, wind and surface boundary conditions.
3. **Loop/store:** heat transferred to/from a water or PCM store, parasitic pump power, usable temperature headroom and cycle closure.
4. **Building:** useful heat removed from an occupied zone and resulting comfort/energy response.
5. **Neighborhood:** outdoor comfort, sky obstructions, canyon/wind interactions, spatial access and distribution.

A material-level sub-ambient temperature or W/m² number cannot be promoted to a building cooling claim without the next evidence levels.

## 2. Published comparison envelope, not a Johannesburg performance target

A July 2026 review of passive radiative-cooling materials for extreme weather summarizes reported high-performing systems with solar reflectance around 0.95–0.98, mid-infrared emittance around 0.90–0.96, daytime sub-ambient temperature drops around 4.5–4.9 °C and net cooling powers around 93–117 W/m² under favorable clear-sky conditions. These values are an orientation point for material/component benchmarking; they are not a guaranteed product rating, a common single test condition or a building-level cooling result. Dust, humidity, wind, cloud, moisture ingress, UV aging and seasonal overcooling alter performance.

A November 2026 multi-scale built-environment review finds that sky access, urban form, atmospheric humidity, canyon geometry and inter-surface radiation materially reshape outcomes between material, building, neighborhood and city scales. It specifically cautions against extrapolating material/demo performance directly into urban energy or comfort benefits.

Consequently, a fair comparison requires the exact reported optical measurement method, solar spectrum, angle, surface temperature, ambient and sky conditions, wind, instrumentation, uncertainty, area definition and evidence type. Do not compare a material's peak clear-sky laboratory metric with a season-long courtyard average.

References:
- [Passive radiative cooling materials for extreme weather (21 July 2026)](https://pmc.ncbi.nlm.nih.gov/articles/PMC13386544/)
- [Passive radiative cooling in the built environment: a multi-scale review (November 2026)](https://www.sciencedirect.com/science/article/pii/S2213138826006764)
- [Radiative sky cooling: bridging lab and applications (October 2025)](https://www.sciencedirect.com/science/article/pii/S1364032125006549)

## 3. Matched comparison arms

- **R0 — No added radiative panel:** same passive building, store and non-radiative heat-rejection options; panel area excluded or replaced with a declared reference surface.
- **R1 — Reference panel:** commercially or openly characterized high-emissivity panel with published optical test data and traceable build/revision.
- **R2 — Candidate panel:** candidate material/assembly with independently measured spectral properties; no supplier nominal alone.
- **R3 — Panel + thermal store:** R1/R2 coupled through a metered closed loop to a water/cistern/PCM store.
- **R4 — Layered sink cascade:** radiative panel dispatch combined with dry-air/ground/conditional evaporative paths under the separate governor contract.
- **R5 — Solar-absorbing negative control:** matched geometry/area but deliberately poor long-wave/solar properties to test the energy balance and detection method.
- **R6 — Infinite-sky negative control:** invalid boundary model assumes a permanently cold sky or unconstrained heat sink; the independent oracle must reject it.

For R0–R4, freeze panel area/orientation, insulation/backing, loop flow, exchanger, pump, controller horizon, store initial state and load trajectory. If an arm has different cost or area, report it as a separate budget/area-matched sensitivity rather than silently attributing the effect to material.

## 4. Required scenarios

The machine-readable registry defines seven boundary scenarios:
1. CLEAR_DRY_HIGH_SKY_VIEW
2. CLOUDY_HUMID_HIGH_LONGWAVE
3. HOT_DAY_HIGH_SOLAR_LOAD
4. WINDY_NIGHT_CONVECTIVE_COUPLING
5. SOILED_OR_WET_PANEL
6. PARTIAL_SKY_OBSTRUCTION_URBAN_CANYON
7. COLD_CLEAR_NIGHT_OVERCOOLING_RISK

The same panel is evaluated under a pinned TMY, recent actual year and predeclared hot-tail/future-climate sequences where data rights and model applicability allow. Local measurement is still required for the actual courtyard sky view, horizon, wind and long-wave boundary.

## 5. Measurement and equations

At minimum record:
- panel spectral solar reflectance and spectral long-wave emittance with measurement method, angular range, sample conditioning and calibration;
- panel area, tilt, azimuth, construction/backing/insulation, surface temperature and fouling/wetness state;
- global/direct/diffuse solar, downwelling long-wave radiation or qualified cloud/sky model, air temperature/RH/dew point, wind speed/direction and pressure;
- sky-view/horizon geometry and adjacent-surface temperatures;
- loop mass flow, fluid heat capacity, inlet/outlet temperatures, pressure drop, pump electricity and exchanger conditions;
- store volume, multi-depth temperature/stratification, usable temperature range and initial/final energy state;
- occupied-zone temperature/operative comfort, HVAC energy/peak and control state if a building-side benefit is claimed.

For the measured fluid loop, report heat transfer with sign and uncertainty:

\[
\dot Q_{\mathrm{loop}}=\dot m c_p (T_{\mathrm{out}}-T_{\mathrm{in}})
\]

Integrate over synchronized intervals. Report positive rejection from the store separately from useful cooling delivered to the zone. The latter requires a measured/qualified building-side heat path and occupied-zone response.

Panel net radiative flux is not the same as loop heat rejection. A panel can radiate while absorbing convective heat from air, solar input or backing conduction. Close the component energy balance with all material terms. Where not measured, show a bounded unknown rather than solving the residual by assigning it to radiation.

For the water/thermal store, require:
- heat received from the building;
- heat rejected by each external sink;
- storage energy change and stratification;
- pump/fan electricity and other parasitics;
- water loss/quality if a wet/evaporative assist is involved;
- seasonal/cycle closure and cumulative store temperature drift.

## 6. Metrics and targets

| Layer | Metrics | Qualification target |
|---|---|---|
| Material | solar reflectance, spectral emittance, aging/soiling retention | independently measured; uncertainty and method traceable |
| Panel | net heat rejection W/m², surface-to-ambient/sky temperature, peak and integrated rejection | measured/validated under all predeclared weather bins; no peak-only claim |
| Loop | kW transferred, pump/fan W, rejection-to-parasitic-electricity ratio, approach temperature | energy balance closes within propagated uncertainty |
| Store | usable capacity, charge/discharge efficiency, seasonal drift, end-of-cycle state | cycle closure inside a predeclared tolerance derived from instrument/model uncertainty |
| Building | useful zone cooling, operative-temperature/discomfort degree-hours, HVAC/peak electricity | improvement exceeds a predeclared practically important difference at matched boundary and does not worsen humidity/overcooling |
| Water | L by source class, evaporation, leakage, purge and reserve impact | zero protected essential-water floor breach; zero unqualified operation |
| Robustness | uncertainty coverage, OOD, fault response, degradation/soiling | no critical out-of-domain claim; 90%/95% interval coverage assessed on held-out regimes |
| Evidence | exact input/model/instrument/controller digests and replay | 100% traceability, 100% finite-oracle agreement |

The 90% and 95% interval levels are evaluation levels to report, not a claim that nominal coverage will automatically be met. Acceptance margins for thermal comfort, panel balance and cycle drift must be set for the site and instruments before holdout results are examined.

## 7. How to beat current SOTA without an unfair metric

Do not claim to beat the literature by exceeding 117 W/m² in a hand-selected clear-sky hour or by showing a colder panel. The meaningful contribution is a robust *system* result:
- match the reported material test method when comparing material properties;
- outperform or match a reference panel on net seasonal heat rejection under the same measured boundary conditions;
- preserve heat-rejection performance after dust/soiling/wetness and weather stress;
- deliver an independently validated occupied-zone improvement after loop, store, pump and controls are included;
- demonstrate seasonal heat-sink regeneration rather than store saturation;
- maintain the same comfort with less water/auxiliary electricity or improve comfort without sacrificing water/essential-service floors;
- demonstrate predictive interval coverage and correct fail-closed behaviour on held-out humid/cloudy/cold/fault conditions;
- reproduce the result from digested inputs and independent replay.

A result can be a Pareto improvement, or a documented region of applicability where the reference performs better. No scalar score may hide a failure of water, safety, comfort or seasonal store closure.

## 8. Industry/specialist boundaries

- **EnergyPlus 26.2:** building zone and supported indoor-pool/plant physics; do not pretend its indoor swimming-pool object is an outdoor urban-courtyard radiation solver.
- **Modelica Buildings 14.0:** plant, storage, loop and control dynamics where exact solver/library interfaces are pinned.
- **OpenStudio 3.11:** model authoring/workflow for EnergyPlus, not an independent second physics result.
- **BOPTEST 0.9:** controller test harness and same-trace comparison, not outdoor radiation or water-rights physics.
- **OpenFOAM Foundation v14 / OpenCFD v2606:** separate CFD distributions for airflow/urban geometry and coupled boundary work; each needs mesh/time-step and measurement qualification.
- **Material vendors/literature:** optical performance input/reference, never a substitute for site-scale field performance.

Highveld's differentiator is the evidence-carrying, water-entitlement-aware and seasonal-cycle-closed connection between these engines and occupant service, not replacement of their underlying physics.

## 9. Claim ceiling

**Status:** \`BENCHMARK_PROTOCOL_NOT_EXECUTED\`. No radiative panel has been selected or installed; no local panel optical test, heat rejection, occupied-zone cooling, field validation, EnergyPlus/Modelica/CFD co-simulation or comparative superiority result is claimed.
