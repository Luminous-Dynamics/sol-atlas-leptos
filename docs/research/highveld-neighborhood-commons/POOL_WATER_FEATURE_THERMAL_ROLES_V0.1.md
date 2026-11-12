# Pool and water-feature thermal integration programme v0.1

**Study ID:** HIGHVELD-POOL-WATER-THERMAL-001  
**Parent:** HIGHVELD-PASSIVE-COOLING-WATER-001 / HIGHVELD-THERMAL-SINK-CASCADE-001  
**Maturity:** DESIGN_AND_BENCHMARK_CONTRACT_NOT_EXECUTED  
**Climate context:** Johannesburg / Highveld

## 1. Core design principle: distinguish storage, heat rejection and amenity

A pool, pond, rill, cistern or fountain can play several different roles. These roles must not be conflated:

1. **Thermal buffer:** water absorbs heat and increases thermal inertia. Its usable capacity is finite and bounded by a declared temperature/enthalpy range.
2. **Regenerating heat sink:** the stored heat is actually rejected to a qualified external path—radiative sky, dry-air exchanger, ground, or permitted evaporation. Without a path, the water warms and the available thermal headroom is consumed.
3. **Evaporative heat rejection:** evaporation removes latent heat but consumes water and adds moisture to the receiving air. It is not automatically useful occupied-zone cooling.
4. **Amenity/microclimate feature:** water may alter local radiant/sensible conditions, but an outdoor air-temperature result is not evidence of indoor comfort or useful building cooling.
5. **Hydronic working fluid:** a closed loop carries heat to/from an exchanger. It is not permission to mix pool, potable, greywater or reclaimed-water circuits.

A typical pool can be a useful peak-shifting thermal buffer or warm-side heat sink for a heat pump, if the temperature grade and safe pool operating range permit. It is not a free, infinite cold reservoir. If heat is transferred into the pool and no regeneration path exists, the pool temperature rises until the available heat-acceptance margin is exhausted.

For direct hydronic radiant cooling, verify the actual supply-water temperature, exchanger approach, room dew point/condensation constraints and load profile. Pool water that is too warm to accept heat at a useful temperature cannot be counted as useful cooling capacity.

## 2. Ten comparison archetypes

| Archetype | Intended thermal role | Key caveat |
|---|---|---|
| NO_WATER_PASSIVE_BASELINE | Primary no-water control | Competent shading/envelope/night ventilation baseline |
| COVERED_POOL_THERMAL_BUFFER | Finite sensible-heat storage | Cover reduces evaporation but can change convection and radiation; pool needs a regeneration path for repeated cycles |
| SHADED_POOL_WITH_CLOSED_LOOP_HX | Thermal buffer and warm-side heat-pump sink | Closed-loop separation, water temperature ceiling, HX approach and cycle closure required |
| OPEN_POOL_EVAPORATIVE_EXCHANGE | Latent heat exchange | Water consumption, humidity addition and actual useful cooling must be metered |
| SHADED_NO_SPRAY_RILL | Amenity and possible local microclimate effect | No indoor cooling claim without a measured building-side path; aerosol classification must be explicit |
| AEROSOL_GENERATING_FOUNTAIN | Amenity / possible evaporative effect | Aerosol and microbial water-management program required; not a default passive-cooling recommendation |
| CLOSED_LOOP_CISTERN_THERMAL_BATTERY | Finite thermal buffer | Cistern capacity and external regeneration must be qualified; source-water rights/quality are separate |
| POOL_HEAT_RECOVERY_FROM_COOLING_LOAD | Useful heat recovery for pool heating | Heat demand, pool setpoint/upper limit and overflow heat-rejection path must be metered |
| INDIRECT_EVAPORATIVE_HEAT_REJECTION | Conditional latent heat rejection through isolated exchanger | Water, approach temperature, drift/aerosols, treatment and water quality constraints |
| DIRECT_MISTING_SYSTEM | Direct evaporative cooling | Highest scrutiny: aerosols, potable/qualified water, microbial control, humidity and water entitlement |

The archetypes are alternatives for matched study. They are not a recommendation to build all of them. In water-stressed settings, start with the no-water passive baseline and closed-loop/dry/radiative paths before considering aerosol-generating or water-consuming equipment.

## 3. A critical cover/shade trade-off

A cover can reduce evaporation and water demand, but it also modifies convection and short-/long-wave radiation. EnergyPlus's indoor pool model represents those pathways separately, so the experiment should not use a single generic "cover efficiency" factor.

Compare at least:
- ALWAYS_UNCOVERED;
- ALWAYS_COVERED;
- DAY_SHADED_NIGHT_EXPOSED;
- DAY_SHADED_NIGHT_COVERED;
- ADAPTIVE_COVER_BY_WATER_BUDGET_AND_SKY_CONDITIONS.

The potential paradigm shift is a **time-varying cover**: reject solar gains during high-sun periods, then expose the pool at night only when the external sky/air/ground conditions make regeneration useful and water loss is acceptable. This is a testable hypothesis, not an established optimum. An opaque cover may prevent the very long-wave/evaporative exchange needed for regeneration; an exposed water surface may lose too much water. Compare the whole 24-hour and seasonal cycle.

A 2025 indoor-pool field study reported up to 70% reduction in evaporation for an unoccupied pool when a movable floor functioned as a cover. That result belongs to its measured indoor facility and operating conditions; it is not a universal outdoor-pool saving factor.


## 3.1 Pool heat recovery: a proven industry option that needs local qualification

Recovering building cooling condenser heat for pool heating is an established concept, not a new Highveld invention. A UC Davis / Western Cooling Efficiency Center field demonstration at a San Diego hotel measured average 5% electricity savings, peak reductions up to 12%, and 29% lower natural-gas pool-heating use over its four-week trial. A separate 2013 California feasibility analysis estimated 25–30% cooling-electricity savings and 30–35% peak-demand reduction for selected conditions; those are modelled estimates and are not the same evidence class as the field result.

The matched Highveld experiment compares:
- conventional building heat rejection to outdoor air plus conventional pool heating;
- controlled condenser-heat recovery to the pool through an isolated heat exchanger;
- heat recovery with a qualified overflow dry/radiative/ground sink when the pool has no remaining thermal headroom;
- heat recovery with the pool at its setpoint or upper operating limit, where no additional heat may be accepted.

Measure recovered heat, displaced pool-heater energy, cooling compressor electricity, condenser temperature/approach, pool temperature trajectory, auxiliary sink rejection, pump electricity and any change in water/air humidity loads. The pool is a useful heat demand only when heat is actually required and the upper water-temperature limit permits transfer. When the pool is warm or demand is absent, the system must revert to a qualified external rejection path. A pool cannot be counted as an infinite sink or continuously accept condenser heat.
 
**Heat-recovery control gates:** recovery may run only when there is a real pool-heating demand and a qualified isolated heat exchanger. The pool's maximum operating temperature must be set by the owner and applicable authority. When the pool has no thermal headroom, excess condenser heat must be rejected through a qualified external sink or the recovery path must be disabled. Required accounting includes recovered condenser heat, displaced pool-heater energy and fallback-sink rejection. A heat-recovery claim also requires measured displaced heater energy; recovered heat alone is not a savings claim.

**Evidence-class note:** the UC Davis WCEC hotel trial (San Diego, four weeks) reported average 5% electricity savings, peak reductions up to 12%, and 29% lower natural-gas pool-heating use. The 2013 California analysis estimated 25–30% cooling-electricity and 30–35% peak-demand reductions under selected modelled conditions. Those numbers remain external reference points, not Highveld targets or predictions. The field-trial and modelled estimates must not be combined as if they were one validation set.


References:
- [UC Davis Western Cooling Efficiency Center: hotel pool heat-recovery field demonstration](https://wcec.ucdavis.edu/swimming-pools-as-heatsinks-for-air-conditioners/)
- [ETCC / SDG&E project and report record](https://www.etcc-ca.com/reports/waste-heat-recovery-air-conditioner-swimming-pool-heating)
- [Harrington & Modera: California pool heat-sink feasibility analysis](https://www.sciencedirect.com/science/article/abs/pii/S0378778812006871)

## 4. Evaporation, moisture and health constraints

Evaporative heat rejection should be reported as:
- measured evaporation/make-up water, bleed, drift, leakage and purge separately;
- latent heat associated with measured evaporation;
- source class, rights, water-quality evidence and treatment;
- receiving-air humidity/dew-point change and ventilation/exhaust path;
- useful heat removed from the building or occupied zone, separately from heat removed from pool water;
- treatment, pump, fan and dehumidification electricity.

Do not equate an estimated evaporation rate with useful cooling delivered to people. Indoor pool evaporation adds moisture to the zone and can increase ventilation/dehumidification loads.

Decorative fountains, sprays, cascades and water walls require explicit aerosol assessment. CDC's decorative-fountain guidance says all decorative fountains produce aerosols and calls for a water-management programme, cleaning/disinfection and monitoring of critical water parameters. ASHRAE's water-system risk-management suite includes Standard 188 and Guideline 12; the latter discusses decorative features, cooling towers, misters, humidifiers and other systems. The authority having jurisdiction and qualified water-management professional determine applicable requirements.

Never admit untreated greywater to a spray/mist/aerosol feature. Reuse water must have a source-specific treatment and end-use qualification; "greywater" is not itself a quality class. Do not place aerosol-generating fountains where local health authority guidance prohibits or where exposed populations cannot be protected. A feature with unknown aerosol state or unknown water-management status is unavailable for an aerosol-cooling claim.

A typical swimming pool is not identical to a decorative fountain or hot tub. CDC notes that Legionella is unlikely to grow in typical swimming pools because the water is usually too cold, while warm pools/hot tubs and aerosol-generating features need separate consideration. Never transfer one device's risk classification to another without the actual operating temperatures, aerosol mechanism and water-management boundary.

## 5. EnergyPlus and specialist-engine comparison

EnergyPlus 26.2 has a documented SwimmingPool:Indoor model embedded in the surface heat balance. It represents pool water-surface convection, evaporation, radiation, bottom conduction, solar gains, heating plant, make-up water and water-temperature change; it can represent a scheduled cover fraction and its effects on several heat-transfer modes. It also adds pool evaporation to the zone moisture balance.

Use this as the indoor-pool specialist baseline. It is not a ready-made outdoor courtyard/urban-canyon water-feature solver. Outdoor microclimate and aerosol transport need site-representative measurement and/or qualified airflow/radiation models with explicit limitations. EnergyPlus does not validate Legionella control or water rights.

EnergyPlus 26.2's ThermalStorage:PCM is a separate useful storage comparator. Modelica Buildings is appropriate for dynamic plant/storage loops where its pinned library and solver match the model. Highveld should compare the same load/weather/store trajectories through specialist engines and then use the independent finite-store ledger to audit trajectory closure—not replace their physics.

## 6. Matched 72-case benchmark

Ten archetypes × eight scenarios = 80 registered comparisons:
1. HIGH_SOLAR_HOT_DRY;
2. HOT_HUMID_LOW_EVAPORATIVE_POTENTIAL;
3. EXTREME_WATER_CURTAILMENT;
4. MULTIDAY_HEAT_WAVE_AND_HOT_NIGHTS;
5. GRID_OUTAGE_AND_PUMP_LOSS;
6. COLD_CLEAR_NIGHT_AND_OVERCOOLING;
7. BIOFILM_OR_DISINFECTANT_CONTROL_FAILURE;
8. STAGNATION_MAINTENANCE_OR_WATER_QUALITY_FAILURE.

Each case pins geometry, surface area/depth, cover schedule, shade/sky-view, occupancy/activity, pool/setpoint temperature, heat exchanger, flow, water source, weather, ventilation, building load, plant/control, instrument and model digests. Capacity- and budget-matched comparisons are separate arms.

## 7. Required measurements and metrics

Thermal:
- pool/cistern depth-resolved temperature and usable enthalpy range;
- store energy at start/end of each interval;
- heat transferred from the building through the exchanger;
- heat rejected by each named sink path;
- pool surface solar, long-wave, convective and latent exchanges where the claim requires them;
- exchanger inlet/outlet temperatures, flow and pressure drop;
- pool/zone operative conditions, humidity/dew point and air speed;
- 24-hour and seasonal cycle closure, maximum water temperature and remaining thermal headroom.

Water and health:
- make-up water, evaporation, bleed, drift, leakage, purge and initial fill;
- potable/non-potable source identity, rights, water-quality and treatment evidence;
- aerosol-generation class and water-management-plan status;
- cleaning/disinfection, biofilm/sediment, disinfectant residual and temperature records as required by the adopted programme;
- maintenance and safe shutdown state.

Performance:
- useful building heat removed (kWh) and peak (kW);
- pool/cistern heat absorbed and rejected by path (kWh);
- store headroom, regeneration time and cycle drift;
- litres per useful kWh of delivered building cooling;
- additional humidity/dehumidification burden;
- pump/fan/treatment electricity;
- occupant comfort and outdoor microclimate outcomes as separate metrics;
- lifecycle cost, maintenance burden and water-source opportunity cost.

## 8. SOTA targets and honest win conditions

Targets:
- zero claims of sustained cooling from a finite pool/store without a qualified regeneration path;
- zero protected essential-water floor breaches;
- zero aerosol operation with unknown/failed water-management status;
- zero untreated greywater in aerosol-generating paths;
- zero useful indoor-cooling claims based solely on outdoor air-temperature or pool-water-temperature change;
- zero over-temperature/condensation violations under authority- and design-approved limits;
- energy and water balances close within propagated measurement/model uncertainty;
- 100% replayable geometry, control, water and instrument provenance.

A meaningful win is a matched reduction in peak indoor heat or discomfort hours with lower water and auxiliary electricity, while preserving pool/service limits and closing the seasonal thermal cycle. Another legitimate win may be showing that a covered pool is only a buffer and that a dry/radiative or ground path is more robust under drought. Do not force a water-feature solution where a no-water or closed-loop design performs better.

**Status:** DESIGN_AND_BENCHMARK_CONTRACT_NOT_EXECUTED. No pool, fountain, water-quality programme, water rights, site measurement, EnergyPlus model run or physical cooling result is admitted.
