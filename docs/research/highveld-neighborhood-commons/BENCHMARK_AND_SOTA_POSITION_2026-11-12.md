# Highveld thermal-water cooling: SOTA comparison and executable benchmark plan

**Date:** 12 November 2026  
**Parent:** `NEIGHBORHOOD_RESILIENCE_COMMONS` / PR #741  
**Related tracking:** [#745 — passive cooling and water integration](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/745)  
**Maturity:** `RESEARCH_PLAN_NOT_EVALUATED`

## 1. Executive research conclusion

The opportunity is not to replace EnergyPlus, CFD, Modelica or building-control benchmark suites. It is to connect their specialist outputs into a reproducible, water-constrained, resident-service evidence chain:

`geometry + weather + water rights + source quality + thermal physics + controller + observed service + independent qualification`.

The differentiator is **cooling as a qualified service under a scarce-water and changing-climate constraint**, not a decorative water feature or a single simulated temperature reduction.

A pool or tank has thermal capacity; that fact alone does not establish a useful heat path to an occupied zone. Evaporated water removes heat at the evaporation interface; that does not establish the same number of kWh as useful room cooling. The benchmark must preserve these separate physical and service quantities.

## 2. Incumbent capability snapshot

### 2.1 EnergyPlus 26.2 — use the specialist physics

The current EnergyPlus 26.2 documentation provides a `SwimmingPool:Indoor` model integrated with the zone surface heat balance. It accounts for convection, evaporation, long- and short-wave radiation, conduction through the pool bottom, make-up water, heating and cover effects. Pool evaporation contributes to the zone moisture balance. This is valuable existing capability and should be used as a comparison/reference model when its assumptions fit.

The documented indoor-pool object is not a general outdoor courtyard-pool model:
- pool water is lumped into the inside face of a linked floor surface;
- the pool occupies the entire floor surface to which it is linked;
- that pool/floor surface cannot be part of a low-temperature radiant system or ventilated slab;
- the cover schedule can vary, but the separate cover heat-transfer reduction factors are fixed input values.

The v26.2 Input Output Reference lists `SwimmingPool:Indoor`; no `SwimmingPool:Outdoor` entry was found in the inspected object index. Treat that as a scoped documentation observation, not proof that no external extension or user-defined coupling exists.

EnergyPlus also provides `WaterUse:Storage`, `WaterUse:RainCollector`, `WaterUse:Connections`, direct/indirect evaporative coolers, `ZoneCoolTower:Shower`, heat exchangers and ground-heat-exchanger objects. This gives useful water collection, tank/account and HVAC couplings. A critical nuance: the documented `WaterUse:Storage` tank water temperature is currently schedule-driven; the docs state that it substitutes for a full thermal tank model and that several ambient/tank thermal fields are reserved for future use. For dynamic tank stratification, EnergyPlus separately provides mixed/stratified chilled-water thermal-storage objects. Do not treat the water-accounting tank as a validated pool/cistern thermal model.

**Our approach:** use EnergyPlus for the building/indoor-pool/control physics it supports; use an explicitly coupled outdoor-water/airflow model and an independently measured interface for the remaining system boundary. Do not fork its pool physics without a compelling, verified model gap.

References: [Indoor swimming pool engineering reference](https://energyplus.readthedocs.io/en/v26.2.0/guides/engineering-reference/17.3-indoor-swimming-pool.html); [Input Output Reference / EpJSON schema](https://energyplus.readthedocs.io/en/v26.2.0/guides/input-output-reference/index.html); [Water systems objects](https://energyplus.readthedocs.io/en/v26.2.0/guides/input-output-reference/1.56-group-water-systems.html); [water thermal tank reference](https://energyplus.readthedocs.io/en/v26.2.0/guides/engineering-reference/19.3-water-thermal-tanks-includes-water-heaters.html).

### 2.2 BOPTEST — controls benchmark, not a neighbourhood water twin

The public BOPTEST latest-release testcase page lists nine ready emulators, including BESTEST Air/Hydronic, hydronic heat-pump, two-zone apartment and multizone residential/office cases. These cases include dynamic models, embedded controls, weather, electricity prices and emissions data, with predefined REST API test scenarios.

That is a useful control-comparison baseline. The public case list does not expose a dedicated outdoor-pool/courtyard-water, household-water-entitlement, neighbourhood access or common-cause service case. This is a gap in the inspected public testcase list—not a claim that BOPTEST cannot be extended.

**Our approach:** align the controller comparison with BOPTEST principles and, where practical, package a custom water-coupled case or bridge rather than inventing incompatible metrics. Preserve exact model, controller, weather and test-scenario IDs.

Reference: [BOPTEST test cases](https://ibpsa.github.io/project1-boptest/testcases/).

### 2.3 South African regulatory and operating context

- SANS 10400-XA:2021 is identified in the 30 October 2026 Draft National Energy Efficiency Strategy as supporting the mandatory Part XA building-energy-code framework, with municipalities responsible for plan review and enforcement. The same gazette is a **draft** strategy, not itself an enacted amendment to SANS or the building regulations.
- The draft National Energy Efficiency Strategy 2025–35 proposes scheduled strengthening of SANS 10400-XA and expansion of Energy Performance Certification. Treat this as regulatory direction to monitor, not a current project compliance certificate.
- Johannesburg Water's Level 1 restrictions run from 1 September 2026 to 31 March 2027. The official customer notice prohibits pool filling and garden watering between 06:00 and 18:00; its emergency/customer page also explicitly includes water features in that restricted daytime list. Current restrictions and permissions must be checked again before operation.
- South Africa's 2015 national environmental-health norms include pool-quality monitoring and sampling recommendations for public/semi-public facilities. They distinguish pool use classes and include microbiological and chemical water monitoring. Obtain current local-authority/public-health interpretation before designing shared/community facilities; do not apply a public-pool sampling schedule as if it were a universal rule for every private ornamental feature.

References: [30 Oct 2026 Draft NEES 2025–35, Government Gazette 55638, Notice 4221](https://www.gov.za/sites/default/files/gcis_document/202610/55638gen4221.pdf); [Johannesburg Water Level 1 notice](https://www.johannesburgwater.co.za/blog/2026/09/15/level-1-water-restrictions-are-here-lets-make-every-drop-count/); [Johannesburg Water emergency/customer notice](https://scm.johannesburgwater.co.za/emergencies/); [National Environmental Health Norms and Standards, Gazette 39561 (2015)](https://www.gov.za/sites/default/files/gcis_document/201512/39561gon1229.pdf).

## 3. Corrected local-climate evidence and weather strategy

The previously referenced EPW.tools Johannesburg climate-trend page computes its 1981–2025 daily temperature series from NASA POWER / MERRA-2 at a 0.5° grid cell containing approximately `-26.14, 28.24`. It is **not ERA5**. We correct that source attribution here. The page reports a +0.84°C difference between the 1981–1990 and 2016–2025 annual means in that grid cell, and explicitly warns that the grid averages city and surroundings rather than representing an airport or neighbourhood thermometer. These are reanalysis/grid-cell calculations, not a station trend or a measured building load.

The same page notes that its EPW files use nearest station observations first and reanalysis only for gaps. That distinction and the file's station/gap metadata must travel with every input. A typical meteorological year alone is not adequate for a warming-life-cycle claim.

Required weather suite:
1. pinned typical-year EPW with source metadata and `.STAT`;
2. at least one recent actual meteorological year (AMY) with original weather-source metadata;
3. a hot-tail/historic extreme year, not selected after seeing candidate results;
4. a future-climate weather series for stated building-life horizons and scenario assumptions;
5. sensitivity to urban microclimate/shading/wind and missing-data treatment.

Where possible, compare reanalysis with quality-controlled South African Weather Service or other traceable station observations. Do not pass a coarse grid-cell mean off as local courtyard wind, humidity or temperature.

Reference: [EPW.tools Johannesburg method and trend](https://epw.tools/climate-johannesburg).

## 4. Paradigm shifts that could create a real advantage

### A. Thermal water service, not pool-as-ornament

Represent every water feature as a versioned physical service with:
- water source, quality, legal permission, rights and reserve floor;
- open surface, exposed area, depth, cover and wet/dry state;
- inlet/outlet temperatures, stratification, flow and heat-exchanger conductance;
- latent, sensible, radiative and conductive transfer as distinct terms;
- pump/filter/treatment power and duty;
- evaporation, splash, purge/backwash, leak and overflow;
- delivered cooling to a named occupied zone, with evidence and uncertainty.

The service claim must point to an actual heat path and measured/simulated occupant-zone response.

### B. Thermal cascade with exergy-aware dispatch

Do not ask only "how many kWh are stored?" Also ask whether the storage temperature is useful to the sink at the required time. A low-grade 20–25°C store may be useful for some heat-rejection or tempering tasks but cannot be treated as equivalent to chilled water at a lower supply temperature.

Explore this staged cascade:
1. use shade, envelope, solar control and night ventilation to reduce the load;
2. use high thermal mass and controlled night purge where climate permits;
3. use low-temperature hydronic loops only when the water/ground/sky sink is cooler enough and condensation limits are satisfied;
4. use covered water mass to buffer a qualified thermal loop;
5. enable direct evaporation only when wet-bulb depression, humidity, water rights and water budget justify it;
6. use mechanical cooling as a bounded top-up or resilience layer, not as an invisible baseline assumption.

Benchmark water storage against stratified chilled-water storage, ground exchange, PCM storage and a no-water passive design. Water's high heat capacity is not by itself proof it is the best storage medium.

### C. Water-aware controls with a hard entitlement invariant

Use a receding-horizon controller or robust model-predictive policy to choose among cover, charge, discharge, evaporative assist, bypass and safe shutdown. Inputs include forecast uncertainty, wet-bulb/dew point, heat-sink availability, water account balance, water quality, tariff/carbon signal, occupancy, equipment health and current restrictions.

Hard constraint: discretionary cooling cannot debit protected household water floors. The controller may choose a less-cool option or reject operation; it cannot invent water or silently cross an essential reserve. Unknown quality, rights, restriction status or instrument health fails closed.

### D. Climate-adaptive rather than one-weather-file design

Select and test control policies across multiple years and future climate scenarios. Evaluate distribution shift, forecast error, changed occupancy and component degradation. A design that wins only on one TMY file is not robust.

### E. Evidence-native digital thread

Every simulation and measurement should be linked to:
- geometry/configuration/BOM revision;
- weather and water-source dataset digests;
- engine/controller version and configuration;
- calibration and instrument certificates;
- assumptions, uncertainty and applicability domain;
- original observations, transformed inputs and derived outputs;
- independent replay result and claim ceiling.

This is where Sol-Atlas, Symtropy and Mycelix add value around specialist physics engines.

## 5. Benchmark ladder

### Stage 0 — analytical known-answer controls

Use the executable first-principles screen and hand-calculated cases to verify units, conservation and ceiling logic. It must never report the thermal capacity or latent heat at the water surface as useful cooling delivered to occupants.

Required exact cases:
- 1 m³ water × 1 K = approximately 1.163 kWh sensible storage capacity;
- 1 L evaporated × 2,450 kJ/kg latent heat = approximately 0.681 kWh absorbed at the evaporation interface;
- 1 kW pump × 1 hour = 1 kWh electricity;
- a balanced water account with inflow/outflow and closing volume;
- an over-budget case that blocks discretionary operation;
- missing source/quality/rights or instrument evidence that blocks qualification.

### Stage 1 — matched building simulation

At minimum compare:
- shade + orientation + cross/stack ventilation + night purge, no water;
- covered pool/cistern thermal mass without a claimed heat path;
- covered store + closed-loop heat exchanger + qualified nighttime sink;
- small shaded rill or wetted surface with controlled exposure;
- indirect/direct evaporative strategies where appropriate;
- water storage + PCM + ground-coupled variants;
- mechanical baseline and mixed-mode baseline.

Match geometry, occupancy, internal gains, weather, thermal comfort criteria, ventilation and control look-ahead. Report both energy/capacity-matched and water-budget-matched arms.

### Stage 2 — coupled outdoor microclimate

Use CFD or a qualified reduced-order airflow model for wind, buoyancy, courtyard aspect ratio, shade, surface temperatures and humidity transport. Validate with sensors or a controlled test cell. Then couple boundary conditions into the building model. A courtyard-average temperature drop cannot substitute for occupied-zone comfort.

### Stage 3 — hardware-in-the-loop / BOPTEST-style controls

Compare fixed schedules, rule-based control, model predictive control and robust/uncertainty-aware control on identical weather, forecast, plant and disturbance traces. Include pump failure, sensor drift, water curtailment, water-quality alarm, network loss and manual degraded operation. Score comfort, water, electricity, peak load, emissions, constraint violations, control actions and recovery.

### Stage 4 — instrumented field validation

Start with an instrumented small courtyard/test cell rather than an expensive pool retrofit. Use a paired no-water/shaded control and water-feature intervention. Log:
- calibrated air temperature/RH/dew point and globe/operative-temperature proxies;
- water temperatures at multiple depths, volume and makeup;
- flow and heat-exchanger inlet/outlet temperatures;
- solar radiation, wind direction/speed and rainfall;
- pump/filter/treatment electricity;
- water quality, maintenance events, occupancy and control state.

Run matched periods and a preregistered analysis. Separate measured outcomes from model-calibrated estimates. A real building pilot follows only after the test cell demonstrates water and thermal-account closure.

## 6. Proposed target metrics (not yet achieved)

| Dimension | Proposed research target |
|---|---|
| Household water floor | 0 unauthorized discretionary breaches |
| Water quality / rights | 0 operation with unknown or failed qualification |
| Water balance | residual within the combined, declared measurement uncertainty; no unmetered term silently set to zero |
| Thermal balance | residual within propagated model/instrument uncertainty for each held-out test |
| Useful cooling | never credited without a validated heat path and occupied-zone outcome |
| Model calibration | predeclared held-out bias/RMSE plus interval coverage and error by operating regime; no tuning on the holdout |
| Uncertainty | nominal 90%/95% predictive interval coverage assessed on held-out seasonal and stress conditions |
| Out-of-domain | 0 critical claims accepted outside the declared applicability domain |
| Controller | 0 hard-constraint violations; compare comfort/water/electricity Pareto sets |
| Robustness | disclose worst-case and tail performance under heat, drought, humid-hot and equipment faults |
| Audit | 100% results traceable to exact input, model, controller and instrument revisions |

Do not impose one universal R², RMSE or water-saving threshold across different building types. Pre-register building-specific acceptance margins, reference measurement uncertainties and service-floor consequences before inspecting the test results.

## 7. SOTA comparison and honest claim boundary

| Capability | Publicly documented incumbent baseline | Highveld opportunity |
|---|---|---|
| Indoor pool heat balance | EnergyPlus 26.2 `SwimmingPool:Indoor` | Reuse it where assumptions fit; qualify coupled building/commons outcomes |
| Rain capture and water-use accounting | EnergyPlus `WaterUse:RainCollector`, `WaterUse:Storage`, water meters | Add resident rights, reserved floors, source quality and auditable reuse authority |
| Stratified thermal storage | EnergyPlus mixed/stratified chilled-water tank and hot-water tank models | Compare a real pool/cistern heat-transfer path to dedicated TES/PCM/ground options |
| Evaporative equipment | EnergyPlus direct/indirect evaporative cooler and cool-tower objects | Outdoor courtyard geometry, humidity exposure, drought dispatch and field validation |
| Building controls benchmarking | BOPTEST v0.9.0, nine ready emulators and REST scenarios | Add a custom water-coupled apartment/courtyard case and resilience disturbances |
| Detailed outdoor airflow | CFD specialist tools | Link geometry, measured boundaries and uncertainty to indoor comfort and service claims |
| Regulatory compliance | SANS 10400-XA:2021 / municipal enforcement | Produce auditable inputs and M&V evidence; never replace the authority/certifier |

We do not currently have evidence that Highveld outperforms these physics/control engines. A defensible win would be a benchmark suite that couples them, preserves water/human constraints, exposes uncertainty and reproduces the evidence across an independent holdout.

## 8. Current local conditions and safety references

Johannesburg Water's Level 1 restriction dates are 1 September 2026–31 March 2027. Its main notice specifies daytime pool filling/garden watering restrictions; its emergency/customer page includes water features. Always recheck before operation.

South Africa's 2015 national environmental-health norms include public/semi-public pool water sampling, hygiene, sanitary condition and backflow safeguards. They are not a substitute for current municipal and specialist public-health signoff. Spray/mist and warm recirculating water need a separate aerosol/Legionella risk assessment; an ornamental feature should not become a poorly maintained aerosol generator.

The 30 Oct 2026 Draft NEES 2025–35 is a proposed strategy and explicitly not an enacted SANS amendment. It identifies SANS 10400-XA and municipal enforcement as part of the existing building-energy-code framework, while proposing future strengthening and wider EPC activity.

## 9. Immediate implementation sequence

1. Keep PR #736's candidate matrix frozen.
2. Land the topology and cooling manifest qualifiers in PR #741; wait for no runner to continue document/model-contract work.
3. Add the first-principles screening binary with synthetic known-answer inputs; ensure it emits separate storage, latent-interface and pump quantities and leaves useful zone cooling unknown.
4. Pin the exact EnergyPlus 26.2 objects and documented limitations in a model-interface manifest.
5. Obtain and hash a licensed/traceable Johannesburg EPW plus actual-year and future-climate candidates. Verify the EPW against nearby station data.
6. Create a minimal EnergyPlus baseline and indoor pool model; separately define an outdoor courtyard-water co-simulation interface.
7. Specify the independent water/energy-balance oracle and measurement uncertainty.
8. Build a paired courtyard test cell and preregister a field validation plan before claiming cooling savings.

**Status:** research and software contracts only. No actual EnergyPlus run, BOPTEST run, CFD result, field observation, cost result or SOTA superiority result has been produced by this packet.


## 10. Expanded industry/toolchain comparison (12 November 2026)

The engine boundary matters. These systems are complementary rather than interchangeable:

| System | Current public baseline observed | Best-fit responsibility in this programme | What it does not replace |
|---|---|---|---|
| EnergyPlus | 26.2.0 | Building/zone heat and moisture balance, supported indoor pool model, HVAC, water-use objects and thermal storage | Outdoor courtyard CFD, resident water rights, instrument qualification or household service truth |
| Modelica Buildings Library | 14.0.0, released 5 Oct 2026 | Dynamic hydronic plant, thermal storage, plant controls, district/building energy systems; recent release adds liquid-cooled data-centre components, EnergyPlus sizing coupling and TOUGH ground-response coupling | It is not automatically a measured Highveld courtyard-water model; exact library and Modelica solver versions must be pinned |
| OpenStudio | 3.11.0 SDK release, 30 Apr 2026 | Model authoring, workflow, measures and orchestration around EnergyPlus | It is not a separate physical solver or a second independent validation result when it invokes the same EnergyPlus engine |
| BOPTEST | v0.9.0, 18 Nov 2025; nine published ready test cases | Reproducible control benchmarking with REST interface, test scenarios and building emulators | It is not the current source of courtyard-water physics or community water-entitlement semantics |
| OpenFOAM Foundation | v14, 14 Jul 2026 | Outdoor airflow, buoyancy, surface exchange and CFD where a validated model is built | Not building-energy orchestration or resident-service assurance |
| OpenCFD OpenFOAM | v2606, 26 Jun 2026 | Alternative OpenFOAM distribution / CFD environment | Must not be treated as byte-for-byte or release-compatible with the Foundation distribution |
| Sol-Atlas / Symtropy / Mycelix | Luminous evidence/control integration under development | Geometry/configuration provenance; native time/event and service trajectories; water rights, consent, resource accounts and instrument admission | Does not replace any of the specialist physical solvers above |

Modelica Buildings Library 14.0.0 specifically adds liquid-cooled data-centre systems and TOUGH-based geothermal coupling, and extends EnergyPlus-linked thermal-zone sizing. This makes it an especially relevant plant/ground coupling comparator for thermal-water work. Its EnergyPlus-linked package documented on the release page uses `EnergyPlus_24_2_0`; do not imply the package is already a turnkey EnergyPlus 26.2 co-simulation. [LBNL Modelica Buildings release 14.0.0](https://simulationresearch.lbl.gov/modelica/library-release/2026/10/05/releases-11.1.2-12.1.2-13.0.1-14.0.0.html).

OpenStudio 3.11.0 is a current workflow/SDK layer and should be assessed as an authoring and orchestration option around EnergyPlus, not counted as an independent thermal solver result. [OpenStudio release page](https://openstudio.net/).

The two current OpenFOAM release families must be separately identified in every experiment: OpenFOAM Foundation v14 and OpenCFD v2606 have distinct release streams. [Foundation v14](https://openfoam.org/release/14/); [OpenCFD v2606](https://www.openfoam.com/download/release-history).

## 11. The defensible route to beating the state of the art

Do not compete on an isolated pool-temperature result or make a generic claim that Luminous can replace a specialist solver. Compete on a harder system-level benchmark:

1. **Matched physics:** same geometry, weather, occupancy, schedules and boundary conditions. Reuse each specialist engine in its intended domain.
2. **Independent cross-engine agreement:** compare EnergyPlus/Modelica/CFD outputs only at explicitly shared physical interfaces; report residuals, uncertainty and solver-boundary differences.
3. **Water-constrained control:** at the same declared water budget and comfort requirement, test whether the system reduces discretionary water and auxiliary electricity without increasing occupied-zone discomfort or humidity risk.
4. **Climate robustness:** evaluate multiple historical years, hot-tail conditions, future climate sequences, drought curtailment, humid-hot days, cold nights and equipment faults.
5. **Decision-grade uncertainty:** report held-out bias, calibrated interval coverage, false-feasible decisions at comfort/water thresholds and out-of-domain detection—not just R².
6. **Evidence replay:** another operator must be able to reconstruct the input package, run identities, transformations, water/energy balances and claim ceiling from digests and receipts.

A credible result that beats an incumbent on this benchmark would be a Pareto improvement at matched constraints (for example: equal or better comfort with less water and no greater auxiliary energy), or materially better robustness under the same disturbance envelope, with independent field validation. A faster run, a lower residual on an unmatched case or a prettier digital twin is not enough.

The multi-engine case matrix is registered separately in `MULTI_ENGINE_THERMAL_WATER_BENCHMARK_V0.1.md` and its JSON manifest. It remains `NOT_EXECUTED`.


## 12. Thermal storage: compare useful trajectory, not material or nameplate claims

A January 2026 review of building thermal energy storage identifies charge/discharge power and heat-exchanger capability as critical deployment constraints, alongside storage media such as PCM; it also notes that longer-duration thermochemical storage still has deployment challenges. A September 2026 PCM review summarizes case-specific reports of roughly 8–36.5% peak-load/heat-flux reduction and 10–30% annual energy savings in selected building comparisons. One separate 81%-of-peak-cooling-demand result came from a combined rooftop PCM + solar chimney + earth-to-air heat-exchanger CFD system, and must not be attributed to PCM alone. These are literature examples, not universal product guarantees or Johannesburg results.

EnergyPlus 26.2 includes ThermalStorage:PCM: the object is connected to plant and use loops, charges/melts and discharges/freezes PCM through a specified internal heat-exchanger effectiveness, and uses a PCM material with phase-change hysteresis. It is a useful specialist baseline. The Highveld store ledger should audit the resulting state trajectory and cycle closure, not recreate that phase physics.

Sources:
- [Nature Reviews: Increasing the sustainability of buildings by using thermal energy storage, 2 January 2026](https://www.nature.com/articles/s44359-025-00123-4)
- [Phase Change Materials in Energy Systems: Performance, Reliability, Life Cycle Evidence, and Design Guidelines, 12 September 2026](https://onlinelibrary.wiley.com/doi/10.1155/er/2036127)
- [EnergyPlus 26.2 Water Heaters and Thermal Storage reference: ThermalStorage:PCM](https://energyplus.readthedocs.io/en/v26.2.0/guides/input-output-reference/1.24-group-water-heaters.html)

Benchmark implication: compare usable kWh inside the operating temperature band, charge/discharge kW, heat rejected by each qualified path, parasitic electricity, water by source, degraded capacity, state-of-charge headroom and annual/seasonal cycle drift. A high latent-heat value or nominal tank capacity does not establish service continuity.

## 13. Thermal comfort and neighborhood service are different claims

ANSI/ASHRAE Standard 55-2023 uses standard and adaptive comfort methods and includes local discomfort from vertical air-temperature gradients. Its environmental factors include air temperature, thermal radiation, humidity and air speed, combined with activity and clothing; its stated population scope is healthy adults in indoor occupied spaces. The official ASHRAE addenda index currently lists addenda a and b to Standard 55-2023 dated 31 August 2026. Pin the exact edition/addenda and use the correct method at execution. Do not interpret comfort compliance as a heat-health safety guarantee.

A 2026 systematic review of urban heat vulnerability/mitigation found that many studies discuss mitigation without empirical testing and documented gaps in cooling access, energy burden, housing conditions, spatial scale and distributional outcomes. The Highveld service/refuge benchmark is designed to measure those gaps rather than assume that a mapped cooling feature benefits every household.

Sources:
- [ASHRAE Standard 55 – Thermal Environmental Conditions for Human Occupancy](https://www.ashrae.org/technical-resources/bookstore/standard-55-thermal-environmental-conditions-for-human-occupancy)
- [ASHRAE Standards Addenda](https://www.ashrae.org/technical-resources/standards-and-guidelines/standards-addenda)
- [Urban heat, vulnerability, and mitigation in U.S. cities: A systematic review, 2026](https://doi.org/10.1016/j.scsadv.2026.100077)
- [Passive radiative cooling in the built environment: A multi-scale review, November 2026](https://www.sciencedirect.com/science/article/pii/S2213138826006764)

## 14. New executable/research increments registered on 12 November 2026

The following work has been added to PR #741 without changing PR #736's frozen three-candidate architecture matrix:

| Increment | Registered scope | Current evidence ceiling |
|---|---|---|
| Thermal sink cascade | 8 architecture arms × 9 stress scenarios = 72 cases | Manifest and Rust structural qualifier; no thermal simulation |
| Sink dispatch governor | Fail-closed eligibility filter for radiative/dry/ground/evaporative paths | Synthetic eligibility tests; no actuation or equipment certification |
| Johannesburg weather package | SAWS/ARC/DWS/WRO/NOAA/reanalysis/EPW source-rights and representativeness register | No site selected, no paid/licensed data acquired, no weather observations admitted |
| Radiative sky-sink benchmark | 7 component/system arms × 7 weather regimes = 49 cases | Registered matrix; no panel or field test |
| Finite thermal-store ledger | Interval heat balance, state bounds, per-sink rejection and cycle closure | Synthetic known-answer ledger; no physical store validation |
| Thermal service and refuge benchmark | 7 building interventions × 8 operating scenarios + 4 refuge topologies × 6 failures = 80 cases | Registered matrix; no site, route, refuge or resident data admitted |
| Refuge availability governor | Site opening, thermal/ventilation/safe-water service, route accessibility, total/accessibility capacity and water reserves | Synthetic eligibility controls only; no real route/site/booking or occupant assignment |
| Courtyard instrument package | 35 template channels, including long-wave/radiative panel/loop sensors | Instrument identities/calibration remain unassigned; template only |

The thermal-store ledger reports peak input/rejection rate as well as energy, and marks each interval FAIL when its own balance, capacity or continuity checks fail. The thermal service/refuge qualifier requires the 80-case registration and preserves the distinction between ASHRAE comfort and local heat-health safety. Its refuge claim rules reject unknown dependencies, inaccessible routes and exhausted capacity.

## 15. What a defensible SOTA win would look like now

A credible Highveld result should demonstrate all of the following on the same frozen, held-out traces:

1. A specialist physics baseline (EnergyPlus/Modelica/validated external airflow model) with a documented configuration and applicable reference cases.
2. A competent passive no-water building baseline and a properly sized mechanical-cooling reference.
3. A finite thermal store whose charge/discharge power and seasonal state trajectory close within uncertainty.
4. A water/rights governor that never spends protected essential reserves and blocks unknown or prohibited source states.
5. Occupied-zone comfort evaluated with the correct standard/adaptive method, plus a separate authority-approved safety envelope where required.
6. Refuge availability based on live operating dependencies, capacity and accessible network routes—not proximity circles or straight-line distance.
7. Uncertainty-calibrated held-out results, per-household distributions, worst-case service gap, maintenance and lifecycle cost.
8. Exact replay by an independent operator.

The advancement is the coupled, evidence-carrying service layer around the best specialist engines. It is not a claim that a new Luminous model outperforms EnergyPlus at heat balance, Modelica at plant dynamics or CFD at outdoor flow. All new matrices remain NOT_EXECUTED until the required model, data-rights, instrument and independent-review gates are met.


## 16. Refuge network: static coverage is not service continuity

Recent literature indicates a shift from static cooling-centre coverage toward building-level exposure, access and time-varying supply/demand:
- A May 2025 geospatial framework ranked potential/available cooling centres in five U.S. cities using physical/social factors and accessibility/population coverage. It reported large gaps in walking access and energy-burden coverage; this is a siting/access baseline, not an operational capacity or outage replay.
- A September 2026 building-level heat-risk framework applied indoor thermal exposure and cooling-centre accessibility across 214,681 Phoenix residences and showed that building exposure and cooling-centre access can diverge spatially.
- A September 2026 state/process study explicitly evaluated heatwave-induced cooling-service supply/demand mismatch and functional degradation of fixed-location cooling supply.
- A 2026 urban-heat systematic review recommends operational twins linked to real-time sensing, outage risk, transit conditions and cooling-site occupancy, as well as interoperable benchmark datasets.

Highveld now has two distinct layers:
1. **Static snapshot feasibility:** aggregate max-flow across anonymous cohorts and shared refuges, using route accessibility, capacity and safe-water limits.
2. **Temporal service degradation:** evaluate ordered snapshots and integrate unmet-place-hours and accessible unmet-place-hours, while leaving native event IDs, clock and causal chronology with Symtropy.

These remain synthetic benchmarks. A credible win would show a better worst-cohort service ratio and lower unmet-place-hours under common held-out heat/outage/route/capacity traces—not simply more sites on a map.

Sources:
- [ORNL, Where to cool off: a geospatial framework for placement of cooling centers, May 2025](https://www.ornl.gov/publication/where-cool-geospatial-framework-placement-cooling-centers)
- [A two-dimensional framework for building-level heat risk assessment: Refining vulnerability identification and advancing equity, September 2026](https://www.sciencedirect.com/science/article/pii/S2212420926003535)
- [Urban heat resilience from a state–process perspective: Heatwave-induced cooling service supply–demand mismatch and hierarchical spatial allocation of greening, 15 September 2026](https://www.sciencedirect.com/science/article/pii/S2210670726004762)
- [Urban heat, vulnerability, and mitigation in U.S. cities: A systematic review, 2026](https://doi.org/10.1016/j.scsadv.2026.100077)

**Claim boundary:** no real refuge location, route, resident demand, operational capacity, outage or heatwave chronology is admitted by these new artifacts. No live allocation or emergency dispatch is implemented.


## 16. Pool and water-feature integration: role separation and specialist boundaries

EnergyPlus 26.2 already provides a SwimmingPool:Indoor model integrated into the building surface heat balance. It includes water-surface convection, evaporation, short-/long-wave radiation, bottom conduction, solar gain, heater/plant flow, makeup water, water-temperature change and effects of a scheduled cover on multiple transfer modes. Pool evaporation is also added to the zone moisture balance. Use this for indoor pools where its floor/surface assumptions fit; it is not a ready-made outdoor courtyard water-feature or neighborhood CFD model.

A 2025 IBPSA field paper measured an up-to-70% reduction in evaporation in one unoccupied Norwegian indoor pool when a movable floor acted as a cover. This is a site-specific result, not a universal saving factor. It is enough to motivate matched cover studies—not to assume that a pool cover gives the same benefit outdoors or improves thermal regeneration.

CDC's current decorative-fountain guidance says all decorative fountains produce aerosols, and recommends a water-management programme, cleaning/disinfection and monitoring of critical water parameters. ASHRAE's public water-system risk-management suite identifies Standard 188 and Guideline 12; Guideline 12 addresses decorative water features, cooling towers, misters, humidifiers and other devices. CDC also distinguishes typical swimming pools from hot tubs and aerosol devices: typical pools are usually too cool for Legionella growth, while hot/aerosol devices require their own assessment. The architecture should therefore classify the aerosol mechanism, actual water temperature, treatment, maintenance and exposure context rather than label every pool or fountain identically.

Sources:
- [EnergyPlus 26.2 Indoor Swimming Pool Engineering Reference](https://energyplus.readthedocs.io/en/v26.2.0/guides/engineering-reference/17.3-indoor-swimming-pool.html)
- [EnergyPlus 26.2 ThermalStorage:PCM reference](https://energyplus.readthedocs.io/en/v26.2.0/guides/input-output-reference/1.24-group-water-heaters.html)
- [IBPSA Building Simulation 2025: Quantifying the Influence of the Pool Cover on the Water Evaporation Rate](https://publications.ibpsa.org/conference/paper/?id=bs2025_1871)
- [CDC: Controlling Legionella in Decorative Fountains, 3 January 2025](https://www.cdc.gov/control-legionella/php/toolkit/decorative-fountain-module.html)
- [ASHRAE: Guidance for Water System Risk Management](https://www.ashrae.org/technical-resources/standards-and-guidelines/guidance-for-water-system-risk-management)
- [CDC: Evaluating Sources of Exposure](https://www.cdc.gov/investigate-legionella/php/public-health-strategy/evaluating-sources-of-exposure.html)

New pool/water research packet in PR #741:
- POOL_WATER_FEATURE_THERMAL_ROLES_V0.1.md
- POOL_WATER_FEATURE_THERMAL_MANIFEST_V0.1.json and schema
- highveld-pool-water-feature-qualifier structural/negative-control test binary
- 10 archetypes × 8 scenarios = 80 registered cases, including pool heat recovery from building cooling loads; five time-dependent cover/shading modes, including day shading/night exposure and adaptive cover.
- Contract: a pool is a finite thermal buffer; sustained cooling requires a named regeneration path. Evaporative removal is measured separately from useful occupied-zone cooling; water use, humidity, make-up, purge, rights, quality and aerosol risk remain explicit.

## 17. Multi-area refuge flow and shared-capacity audit

The existing highveld-refuge-network-feasibility binary is the network-level max-flow feasibility oracle, with a separate ordered supply/demand degradation sequence. It performs accessible-demand-first feasibility under shared site capacities and route links, while explicitly returning no person assignment or reservation.

The added highveld-refuge-shared-capacity-audit is intentionally narrower and independent: it validates a proposed anonymized group-to-site plan by re-summing every site's total, accessible and water use. It catches double-spending in submitted plans, per-group over-allocation, duplicate group/site rows, unknown sites and route-evidence failures. It reports separate service ratios/gaps; it is not a second allocation algorithm.

The benchmark target is zero false-feasible total/accessibility/water capacity, zero accepted unreachable routes and preservation of positive controls. A plan may be physically feasible but still have unmet demand; the audit must report that gap rather than hide it in an aggregate score.


### Pool heat recovery as a matched heat-use arm

The Highveld pool/water benchmark contains 10 archetypes × 8 scenarios = 80 registered cases. The tenth arm, POOL_HEAT_RECOVERY_FROM_COOLING_LOAD, follows an established industry pattern: a UC Davis WCEC hotel field trial reported average 5% electricity savings, peak reductions up to 12% and 29% lower pool-heating natural-gas use over four weeks in San Diego. The California feasibility study's 25–30% cooling-electricity / 30–35% peak reductions are modelled selected-condition estimates, not field performance.

The local benchmark must gate recovery by actual pool heating demand, an isolated exchanger and an owner/authority-defined pool temperature ceiling. When the pool is at its ceiling or there is no heating demand, excess condenser heat must move to a separately qualified fallback sink or recovery must stop. Count recovered heat, displaced pool-heater energy and fallback rejection separately. This is not an assumed Highveld saving.
