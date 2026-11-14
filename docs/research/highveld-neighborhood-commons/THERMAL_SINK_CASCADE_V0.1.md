# Highveld thermal sink cascade — water-store regeneration benchmark v0.1

**Study ID:** HIGHVELD-THERMAL-SINK-CASCADE-001  
**Parent:** HIGHVELD-PASSIVE-COOLING-WATER-001 / PR #741  
**Maturity:** PROTOCOL_ONLY_NOT_EXECUTED  
**Climate context:** Johannesburg / Highveld; site-specific weather package required

## 1. Core paradigm shift: a pool is a buffer, not an infinite heat sink

A water store can absorb heat because of its thermal capacity. It cannot absorb heat indefinitely. If a building repeatedly transfers heat into a pool or cistern without a qualified way to reject that heat, the store warms, the available temperature difference falls, heat-exchanger output declines and the apparent cooling benefit decays.

The system must therefore be designed as a thermal sink cascade:

1. Reduce building load first: solar control, shade, envelope, ventilation and night flushing.
2. Move heat only when a useful temperature difference and qualified heat-transfer path exist.
3. Buffer heat in water, PCM or another declared thermal store.
4. Regenerate that store through a separately qualified sink: radiative sky panel, dry-air cooler, ground loop or conditional evaporative rejection.
5. Use mechanical cooling only as a separately accounted, bounded backup.
6. Keep discretionary water and electricity within their own budgets; neither can be silently substituted for the other.

A pool/cistern may be an amenity, storage buffer and thermal interface. It is not automatically a heat sink, a cooling appliance or evidence of occupant comfort.

## 2. Research hypotheses

- **H1 — radiative regeneration:** a high-emissivity, low-solar-absorptance sky-facing radiator may cool a closed loop or storage store during suitable clear-sky periods with negligible operational water consumption. Benefit depends on cloud, atmospheric humidity, sky-view factor, wind, surface temperature and urban obstructions.
- **H2 — dry-air regeneration:** a dry cooler can reject heat when outdoor air is cool enough relative to the store/loop. It trades some fan/pump electricity for water independence.
- **H3 — ground regeneration:** a ground loop may provide a stable seasonal sink, but performance depends on site geology, moisture, loop geometry, bore spacing, thermal interference and long-term heat accumulation. Ground is not an infinite sink.
- **H4 — conditional evaporative regeneration:** evaporation may offer greater heat rejection in selected hot/dry windows, but must be water-budgeted, quality-qualified, humidity-aware and disabled when the water/rights/health gate fails.
- **H5 — layered cascade:** dispatch across radiative, dry-air and ground sinks may reduce peak electrical demand and water use while preserving comfort, but only if interfaces and common-cause failure modes are qualified.
- **H6 — no-water wins under scarcity:** a well-designed no-water passive building may dominate the water-store system on lifecycle burden, complexity and resilience under extended water interruption.

These are falsifiable hypotheses. No sink is ranked before matched testing.

## 3. Eight architecture arms

| Arm | Thermal-store / sink arrangement | Required falsification condition |
|---|---|---|
| NO_WATER_PASSIVE_BASELINE | Load reduction, shade, ventilation and thermal mass; no added water | Must be a competent baseline, not a deliberately weak control |
| WATER_STORE_NO_REGENERATION_NEGATIVE_CONTROL | Water store receives building heat but has no external regeneration | Must reveal store saturation and declining useful temperature headroom |
| WATER_STORE_RADIATIVE_SKY_REGENERATION | Closed loop rejects heat through a sky-facing radiative panel | Must lose performance under cloud, humid sky or poor sky view when the physics predicts it |
| WATER_STORE_DRY_AIR_REGENERATION | Dry cooler rejects heat to ambient air | Must account for fan/pump energy and failure under insufficient temperature approach |
| WATER_STORE_GROUND_LOOP_REGENERATION | Heat rejected to a qualified ground loop | Must report ground temperature drift and long-term thermal imbalance |
| WATER_STORE_CONDITIONAL_EVAPORATIVE_REGENERATION | Controlled evaporative heat rejection with metered water | Must block under water restriction, rights/quality uncertainty, reserve breach or humidity/health limit |
| WATER_STORE_LAYERED_SINK_CASCADE | Predeclared dispatch among radiative, dry-air, ground and limited evaporative sinks | Must show each path's independent contribution and cross-layer failure behaviour |
| INFINITE_SINK_ASSUMPTION_NEGATIVE_CONTROL | Deliberately invalid model treats the store/external sink as unlimited | Must be rejected; it is never an admissible design |

The pool/cistern's role is recorded separately from the external heat-rejection path. Do not label a pool as a ground sink or count stored energy as rejected energy.

## 4. Matched experiment and scenario matrix

Every arm shares the same frozen:
- building geometry, construction, orientation, shading and courtyard sky-view factor;
- weather package, occupancy, internal gains, ventilation and comfort criteria;
- sensible/latent building-load trajectory;
- initial store volume, temperature profile/stratification and usable operating range;
- store and exchanger geometry, material/UA data, flow and control boundaries where comparable;
- source water rights, quality class, discretionary budget and protected household water floor;
- electrical budget, tariff/carbon boundary and pump/fan assumptions;
- instrument, calibration, uncertainty and data-rights package;
- independent energy/water balance oracle, model/configuration digest and controller information horizon.

Run these nine registered scenarios against all eight arms:

1. HOT_CLEAR_NIGHT — high daytime load, clear night with useful radiative potential.
2. CLOUDY_HUMID_NIGHT — reduced sky-window opportunity and constrained evaporation.
3. HOT_DRY_WATER_RESTRICTED — strong evaporative potential but discretionary water curtailed.
4. COLD_CLEAR_NIGHT — prevent overcooling, condensation and winter discomfort.
5. MULTIDAY_HEAT_WAVE_STORE_SATURATION — repeated heat input with insufficient recovery opportunity.
6. PUMP_OR_HEAT_EXCHANGER_FAULT — loss or degradation in the heat-transfer path.
7. SENSOR_DRIFT_OR_MISSINGNESS — biased/absent temperatures, flow or water-level signal.
8. WATER_QUALITY_OR_RIGHTS_WITHDRAWAL — water operation must fail closed while water-independent paths may continue.
9. GRID_AND_CONTROL_PARTITION — loss of external power or supervisory network with declared local safe control.

The manifest expands these into 72 deterministic arm/scenario registrations. This is a case inventory, not an event simulation. Symtropy must provide native event IDs, timestamps, paired weather/disturbance traces and thermal/service trajectories at execution.

## 5. Physics and conservation contract

### 5.1 Store energy

For a well-mixed first-order estimate:

\[
E_{store} = m c_p (T_{store}-T_{reference})
\]

For a stratified store, calculate layer-wise energy and report the profile; do not collapse it into one bulk temperature if the design depends on stratification. Sensible capacity is not useful cooling until a transfer path, acceptable temperature approach and occupied-zone outcome are shown.

### 5.2 Heat-exchanger limit

The heat transfer must be bounded by the measured/qualified exchanger and actual temperature difference. A convenient reporting form is:

\[
\dot Q = \dot m c_p (T_{out}-T_{in})
\]

and, where the selected exchanger model supports it, an effectiveness/UA or log-mean-temperature-difference calculation. The actual model and uncertainty must be pinned. No arbitrary fixed kW rating is inferred from pool volume alone.

### 5.3 Sink regeneration

For each sink, record heat rejected, input/output temperature, flow, auxiliary energy, availability window, limiting boundary condition and cumulative seasonal balance. Examples:
- radiative panel: sky-facing surface temperature, long-wave exchange, solar gain, sky view, cloud/atmospheric state, wind and loop transfer;
- dry cooler: inlet/outlet air and fluid temperatures, airflow, fan/pump electricity and approach temperature;
- ground loop: fluid temperatures, flow, ground response, bore/pipe geometry and seasonal drift;
- evaporative path: inlet/outlet air and water temperatures, flow, make-up water, evaporation, humidity/dew point, drift and treatment energy.

A store that ends the annual/hot-season evaluation materially warmer than it began has not demonstrated repeatable storage capacity. Define a cycle-closure envelope from measurement uncertainty and declared operating range before examining results. Do not reset the store to a convenient starting temperature between consecutive heat-wave scenarios unless the reset itself is physically modelled and costed.

### 5.4 Energy and water balances

For the store/control volume:

\[
\Delta E_{store} = Q_{building\rightarrow store} - Q_{store\rightarrow sinks} + Q_{parasitic} + Q_{other}
\]

For water, separately meter opening volume, rain/source inflow, make-up by source class, transfers, evaporation, splash, purge/backwash, leakage, overflow and closing volume. Unknown terms remain unknown and prevent qualification if they materially affect the balance.

Never count:
- heat absorbed by a pool as heat rejected to the environment;
- latent heat at an evaporating surface as useful occupied-zone cooling;
- a model's ideal sink schedule as measured rejection capacity;
- pump/fan power as zero when the device is merely omitted from the model;
- a tank's nominal volume as fully usable storage independent of temperature limits and stratification.

## 6. Control paradigm: safety governor plus Pareto dispatcher

Use two strictly separated layers.

### Layer A — independent feasibility / entitlement governor

This layer is deterministic and fail-closed. It blocks the relevant action if:
- water source rights, water quality, restrictions or instrument state are unknown/failed;
- operation would debit protected essential water reserves;
- humidity, condensation, aerosol, water-quality, electrical or other safety bounds fail;
- sink availability, flow, temperature approach or store operating envelope is not qualified;
- model state is out of its declared applicability domain;
- network/clock/sensor integrity is insufficient for the proposed action.

No learned model or optimizer can override this governor.

### Layer B — bounded multi-objective dispatcher

Only options admitted by Layer A enter optimisation. Compare:
- comfort/discomfort degree-hours;
- cooling electricity and peak demand;
- water use by source class;
- thermal store headroom and seasonal drift;
- carbon/lifecycle cost;
- maintenance burden and availability.

Do not compress these into one weighted score by default. Return the Pareto set and the assumptions that create each trade-off. Residents/owners retain authority over discretionary water use and allocation. A perfect-forecast oracle may be reported as a non-deployable upper bound, never as the controller baseline.

Benchmark fixed schedule, rule-based control, MPC and robust/uncertainty-aware MPC on identical causal forecasts and disturbance traces. The robust controller must be compared with a simple safe rule, not only a weak fixed schedule. Safe local fallback must continue under network partition.

## 7. Benchmark gates and target metrics

### Non-negotiable gates

- zero essential-water-floor violations caused by discretionary cooling;
- zero operation with unknown/failed source rights, water quality or required safety qualification;
- zero unmodelled or unmetered heat/water terms silently treated as zero;
- zero false useful-cooling claims without heat path and occupied-zone evidence;
- zero accepted infinite-sink assumptions;
- zero missed predeclared sensor, pump, exchanger or grid fault controls;
- all positive controls remain admissible when their evidence supports operation;
- energy and water balances close within propagated instrument/model uncertainty;
- annual or seasonal store-cycle drift remains within a predeclared, physically justified envelope;
- 100% traceability to model, controller, geometry, weather, water and instrument versions.

### Comparative vectors

Report comfort, water, electricity, peak demand, emissions, heat rejected by path, store headroom, cycle drift, water losses, maintenance, fault-recovery time and lifecycle cost separately. A candidate wins only through a matched Pareto improvement or materially better worst-case resilience with equal or better hard-floor compliance.

Do not set one universal temperature-drop or R² threshold. Set building-specific comfort margins, heat-sink capacity, meter uncertainty and seasonal store-cycle requirements before seeing results.

## 8. Weather and source-data acquisition for Johannesburg

A proper local package needs more than one typical-year EPW:
1. a pinned TMY/EPW for design comparisons;
2. at least one recent actual meteorological year;
3. a preregistered historic hot-tail year;
4. multi-day heat-wave and hot-night sequences;
5. future-climate time series for the stated asset life and emissions pathway;
6. site/courtyard wind, shade, cloud and humidity measurements for the field cell.

The South African Weather Service says its archives include hourly wind direction/speed, temperature, humidity, pressure and sunshine from 1950 onward. It also states that data has commercial value and is subject to data-use policy; obtain the required licence and do not redistribute source data outside its terms. The Water Research Observatory's dataset catalogue can help compare South African sources; it notes that SAWS/ARC station data may be fee-based and that gridded/reanalysis products need local validation. For any chosen site, compare gridded products with nearby quality-controlled station observations before using them as validation truth.

Candidate source acquisition:
- SAWS climate services and archived observations: https://testing.weathersa.co.za/home/aboutclimateatsaws
- Water Research Observatory climate/weather dataset finder: https://www.waterresearchobservatory.org/data-and-resources/climate-and-weather-data
- ARC Agrometeorology data portal: https://www.agroclimate.agric.za/WP/WP/About.php
- Johannesburg urban-heat vulnerability research: https://pmc.ncbi.nlm.nih.gov/articles/PMC9720904/
- Johannesburg convection-permitting climate-model evaluation: https://rmets.onlinelibrary.wiley.com/doi/full/10.1002/asl.1264

Do not treat a modelled urban grid cell as the courtyard's measured sky temperature, wind or humidity. Radiative-cooling availability must be measured or computed from a validated atmospheric/radiative boundary.

## 9. Industry and SOTA position

- EnergyPlus 26.2: reuse SwimmingPool:Indoor, plant, heat exchanger, evaporative and thermal-storage objects where the documented boundaries fit. The pool object is indoor/zone-coupled; a water-use storage accounting object is not a dynamic stratified outdoor pool model.
- Modelica Buildings 14.0: use for dynamic hydronic systems, heat-pump/plant controls and ground coupling where the pinned library/environment supports the intended loop. Record the exact Modelica compiler and solver.
- OpenStudio 3.11: use as a reproducible EnergyPlus model-authoring/workflow layer, not an independent physics vote.
- BOPTEST 0.9: use as a controller-benchmark pattern or custom test case; its published ready cases are building/HVAC emulators, not a native water-scarcity/thermal-sink neighbourhood case.
- OpenFOAM Foundation v14 / OpenCFD v2606: keep the distributions separate; use only for a defined external airflow/radiation/heat-transfer scope with mesh/time-step and measurement validation.
- Radiative cooling SOTA: the November 2026 multi-scale review finds that sky-view, urban form and humidity reshape performance between material, building, neighbourhood and city scales; material/demo results should not be extrapolated directly to urban benefit. The Highveld contribution must be full-scale/coupled measurement and seasonal sink-regeneration evidence, not a new coating R-value claim. Review: https://www.sciencedirect.com/science/article/pii/S2213138826006764

Highveld does not currently beat these engines at their core physics. Its potential advantage is the system-level, water-entitlement-aware, seasonally closed evidence chain around them.

## 10. Ownership, integration and status

- Sol-Atlas: geometry, sky-view and shade, configuration/BOM revision, sink/store topology and access.
- Symtropy: native event clock/identity, paired disturbances, thermal/resource/service trajectories, causal replay and cycle recovery.
- Mycelix: water rights/source permissions/quality evidence, essential reserves, allocation, instrument admission and audit receipts.
- Specialist engines: EnergyPlus, Modelica, CFD, hydraulic and radiative boundary physics.
- Independent oracle: conservation, cycle closure and finite known-answer qualification, not the candidate's own success predicate.

The JSON manifest is a deterministic case registry. The Rust qualifier checks structural invariants and enumerates cases; it is not a thermal solver, CFD model, controller or general JSON Schema runtime.

**Status:** PROTOCOL_ONLY_NOT_EXECUTED. No sink has been ranked, no EnergyPlus/Modelica/CFD simulation has been run, and no cooling, water-saving, lifecycle-cost or resilience superiority is claimed.
