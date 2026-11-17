# Pool cover and shade schedule counterfactual benchmark v0.1

**Study ID:** HIGHVELD-POOL-COVER-SCHEDULE-001  
**Parent:** HIGHVELD-POOL-WATER-THERMAL-001 / HIGHVELD-POOL-PASSIVE-REGENERATION-LEDGER-001  
**Maturity:** BENCHMARK_PROTOCOL_NOT_EXECUTED  
**Climate context:** Johannesburg / Highveld; no site, pool or cover measurements admitted

## 1. Research question

Can a time-varying combination of pool cover and external shade preserve pool thermal utility and water while regenerating a finite thermal buffer at night—without worsening pedestrian radiant exposure, humidity or essential-water reserve?

The key paradigm shift is to treat the cover and shade as separate, time-varying controls over several exchange pathways, rather than assign one static "pool cover efficiency" coefficient.

A pool may be:
- a thermal buffer receiving building-side heat through an isolated heat exchanger;
- a heat-recovery sink for a heat pump when pool heating demand and headroom remain;
- a finite cold-side store whose temperature grade can be regenerated at night;
- an outdoor amenity with a separate pedestrian microclimate effect.

These are different service modes. A pool is not an infinite heat sink, and evaporation is not automatically useful cooling.

## 2. Four matched schedule arms

| Arm | Schedule | Purpose |
|---|---|---|
| ALWAYS_COVERED | Covered for entire study horizon | Water/heat retention reference |
| ALWAYS_OPEN | Exposed for entire horizon | Maximum exposure reference; water-loss and humidity counterfactual |
| DAY_SHADE_NIGHT_EXPOSE | Solar shade during declared day window; open to sky at night when qualified | Test passive night regeneration |
| ADAPTIVE_COVER_BY_BOUNDARY_AND_WATER | Switch among covered, shade-only and night-exposed states using qualified forecast/measurement, owner limits and uncertainty bounds | Main candidate policy |

The study must compare the same pool geometry, water depth/volume, cover and shade hardware, circulation, occupancy/activity, load, weather, building interface and operating horizon. If a schedule changes more than the cover/shade state, register a separate matched arm.

The adaptive arm is a policy hypothesis, not an established optimum. It must be allowed to lose to the fixed schedules.

## 3. Six stress scenarios

1. HOT_CLEAR_DAY — solar load and evaporation pressure.
2. CLEAR_COOL_NIGHT — candidate long-wave/sensible night regeneration.
3. CLOUDY_HUMID_NIGHT — reduced sky rejection and weak evaporative potential.
4. WINDY_DRY_NIGHT — increased sensible/latent exchange and potentially higher water loss.
5. WATER_CURTAILMENT — discretionary evaporation/makeup is constrained or prohibited.
6. HEAT_RECOVERY_DEMAND — pool is receiving condenser heat for a declared heating demand and should retain useful recovered heat.

The registered study is 4 schedule arms × 6 scenarios = 24 comparisons, before any repeated day-block or seasonal replication. A case is not a physical run.

## 4. Separate the four surface-exchange pathways

For an indoor swimming-pool model, EnergyPlus 26.2 supports a cover schedule and separate cover factors for evaporation, convection, short-wave radiation and long-wave radiation. The cover may also redirect blocked radiation into the zone balance. Use that documented model only for its indoor-pool boundary and pin the exact engine/input revision.

For an outdoor pool or water feature, do not blindly transfer the indoor-zone cover equations or coefficients. Outdoor cover/shade effects depend on the cover temperature, sky view, wind exposure, solar angle, long-wave exchange with surrounding surfaces, wetness, edge geometry and evaporation. Use specialist outdoor boundary modelling and field measurements.

Record each pathway separately:
- short-wave solar gain to water/cover;
- long-wave water/cover-to-sky and surrounding-surface exchange;
- sensible convection between water/cover and local air;
- latent evaporation and the associated water mass loss.

A single cover percentage cannot identify these four effects. Missing terms remain unknown, not zero.

## 5. Robust adaptive policy contract

The adaptive schedule can recommend night exposure for regeneration only when all required inputs are qualified:
- pool temperature is above the owner-declared regeneration target and below its owner/authority maximum;
- sky-view and weather boundary are representative for the cover/pool location;
- forecast or measured net heat rejection has a positive lower uncertainty bound;
- forecast evaporation plus its uncertainty does not exceed the admitted discretionary water budget;
- water rights, quality, treatment and maintenance states are qualified;
- the cover/shade mechanism and local manual override are available;
- the decision does not conflict with an active pool heat-recovery demand.

Formally, for a declared expanded uncertainty bound:

\[
Q_{\mathrm{reject,lower}} = \hat Q_{\mathrm{reject}} - U_{95,Q} > 0
\]

\[
V_{\mathrm{evap,upper}} = \hat V_{\mathrm{evap}} + U_{95,V}
\leq V_{\mathrm{discretionary,budget}}
\]

These are case-specific robust decision gates, not universal heat-rejection thresholds. Use a compatible uncertainty convention for the forecast/model. Correlated error terms must not be treated as independent without justification.

If any required forecast, rights, quality, cover, sky-view or temperature input is unknown, the adaptive schedule may not claim regeneration eligibility. It emits a hold/review state, not a best guess. No policy output directly actuates the cover.

### Lexicographic decision priorities

1. Pool/user safety, hygiene, water rights and protected essential-water reserves.
2. Owner/authority-defined pool temperature and equipment limits.
3. Active heat-recovery demand and finite store headroom.
4. Robust thermal regeneration and water conservation.
5. Pedestrian amenity and measured microclimate benefit.

Do not optimize these by a single weighted score that can trade away a water-rights or safety breach.

## 6. Metrics

Report separately for every arm and scenario:
- total evaporation litres and makeup litres by water source;
- thermal energy received/rejected by short-wave, long-wave, sensible, latent, ground and hydronic paths;
- pool temperature profile, layer stratification, cold/hot exergy and remaining usable headroom;
- net regeneration kWh and next-day heat-acceptance capacity;
- recovered pool heat and independently measured displaced heater energy;
- cover/shade actuator electricity, maintenance and cycle count;
- local air temperature, humidity/dew point, wind and MRT/globe at occupied heights;
- cover/heat-path failures, time in hold/review state and unmet thermal service;
- uncertainty bounds and balance residuals.

A schedule wins only if it produces a matched Pareto improvement in thermal-service availability, water consumption, auxiliary energy, maintenance and pedestrian conditions—or a transparent trade-off with no hard-floor violation.

## 7. Benchmark acceptance targets

- zero water-rights, quality, protected-reserve or pool-temperature-limit violations;
- zero night-regeneration recommendations when the lower uncertainty bound of heat rejection is non-positive;
- zero exposure recommendations when upper-bound evaporation exceeds the discretionary budget;
- zero accepted cover schedules with missing pathway factors/evidence where the selected model requires them;
- zero indoor-pool equations silently applied as outdoor validation;
- 100% exact replay of cover schedule, weather boundary, pool state, water/energy ledgers and uncertainty method;
- preserve positive controls where the qualified clear-night regeneration case is eligible and the heat-recovery retention case remains covered;
- report worst-case and season-long outcomes, not just one clear night.

## 8. State-of-the-art comparison

EnergyPlus 26.2 supplies a useful indoor-pool heat-balance model with separate convection, evaporation, radiation, bottom conduction, makeup water and plant heating. The cover schedule separately modifies several of these pathways. A 2025 indoor-pool study of a movable floor/cover measured up to 70% evaporation reduction for an unoccupied pool at its specific Norwegian facility. Neither result is a universal outdoor-pool coefficient.

Outdoor water-channel studies find that wind and canyon configuration can dominate local response and that multiple channels can worsen daytime long-wave conditions. Therefore the adaptive-cover benchmark must measure the actual outdoor exposure and be permitted to report no benefit or a negative result.

Highveld's potential contribution is the cross-layer policy evidence: finite heat-store state, four-pathway cover accounting, weather/uncertainty-aware schedule qualification, water rights, measured pedestrian outcomes and independent replay. This is complementary to the specialist physics engine, not a replacement.

## 9. Claim ceiling

**Status:** BENCHMARK_PROTOCOL_NOT_EXECUTED. The 24 cases are registered only. No cover schedule has been selected, optimized, installed or measured; no outdoor pool result, water saving, energy saving or pedestrian-comfort benefit is claimed.
