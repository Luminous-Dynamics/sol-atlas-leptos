# Pool passive thermal-buffer and regeneration ledger v0.1

**Study ID:** HIGHVELD-POOL-PASSIVE-REGENERATION-LEDGER-001  
**Parent:** HIGHVELD-POOL-WATER-THERMAL-001 / HIGHVELD-THERMAL-SINK-CASCADE-001  
**Maturity:** SYNTHETIC_CONSERVATION_CONTRACT_NOT_PHYSICALLY_VALIDATED  
**Purpose:** audit the finite thermal state, individual heat-rejection paths and pool-water mass balance over a cover/weather/operating sequence.

## 1. Why this is separate from the pool heat-recovery ledger

The pool heat-recovery ledger checks a heat-pump refrigeration circuit and the disposition of condenser heat. It does not establish whether an ordinary outdoor pool or water feature regenerates its thermal headroom passively across hot days and cool nights.

This ledger audits that distinct passive process:
- building-side heat received by pool water;
- solar heat absorbed by the pool;
- long-wave radiative exchange;
- sensible convective exchange with air;
- evaporative latent heat rejection;
- ground/conduction exchange;
- any isolated hydronic heat-exchanger path;
- water volume changes from evaporation, make-up, rain, leakage, overflow, backwash and withdrawals.

A pool is a finite buffer. Heat absorbed today must be rejected, transferred elsewhere or retained as higher stored thermal energy. A water feature is not automatically useful cooling for the building: air temperature, humidity, mean radiant temperature, occupant comfort and the building-side heat path are separate observations.

## 2. Thermal control volume and state basis

The pool energy state must come from layer-resolved water temperature/enthalpy, or a mixed-water assumption validated for the actual circulation regime. A bulk average is not sufficient when material stratification or stagnant zones exist.

The thermal ledger uses one declared reference state and a consistent enthalpy convention. For each interval:

\[
E_{end}=E_{start}+Q_{building}+Q_{solar}+Q_{other}+H_{liquid,in}
-Q_{LW}-Q_{sensible}-Q_{evap}-Q_{ground}-Q_{HX}-H_{liquid,out}
\]

The interval residual must be within the declared propagated expanded uncertainty. Heat flux terms remain separate; the tool does not infer an unmeasured flux by assigning the balance residual to it.

Evaporative latent heat is checked against measured/qualified evaporated water volume and the temperature-specific latent-heat method:

\[
Q_{evap,expected}=V_{evap}\,h_{fg}
\]

This is an independent cross-check, not an invitation to assume a universal kWh/L constant. The latent-heat value and its method are pinned in the case and require an admitted source/method for physical evaluation.

## 3. Independent water-volume balance

For each interval:

\[
V_{end}=V_{start}+V_{makeup}+V_{rain}
-V_{evap}-V_{withdrawal}-V_{leak}-V_{backwash}-V_{overflow}
\]

Water draw is a separate authority decision. Unknown or prohibited rights/quality states block any make-up draw. A protected essential-water reserve is not available for discretionary pool operation. Missing water measurements are UNKNOWN, never zero.

The sample uses no make-up water. Its volume falls only through synthetic evaporation so that the ledger tests the distinction between a thermal sink and a water-consuming operation.

## 4. Cover, shade and weather boundary

The case records the cover state for every interval. It does not apply a generic cover-efficiency multiplier. Compare cover/shade schedules under matched weather and geometry:
- uncovered;
- covered;
- shaded and exposed;
- shaded and covered;
- adaptive, where control logic and its input evidence are pinned.

The minimum outdoor boundary includes dry-bulb temperature, relative humidity, wind speed, global solar irradiance, downwelling long-wave radiation and sky-view factor. Add direct/diffuse solar, wind direction, precipitation and surrounding-surface temperatures where required by the selected model and claim.

This is important because pool heat loss is not a single evaporation coefficient. Recent experimental and modelling work reports strong dependence on wind, atmospheric state, cover, occupancy and season. Water-channel studies also report geometry/wind interactions and cases where added water features worsen daytime radiant conditions.

## 5. Synthetic known-answer case and negative controls

The fixture has three symbolic intervals with different cover states. It balances both the thermal energy store and water volume; the thermal store returns to its initial state at the end of the synthetic cycle.

Required controls:
- thermal energy residual outside u95;
- water-volume residual outside u95;
- evaporation mass versus latent heat mismatch;
- pool temperature above the externally declared limit;
- make-up water without qualified rights/quality and discretionary permission;
- make-up water above the declared discretionary budget;
- broken thermal-state continuity;
- broken water-volume continuity;
- cycle drift outside the declared tolerance;
- missing per-interval evidence reference.

The positive control must pass. A validator that rejects every input is not a successful safety gate.

## 6. Benchmarks and report fields

Report separately:
- finite thermal energy state and cycle drift (kWh);
- pool temperature and maximum layer temperature against the owner/authority ceiling;
- building heat input, solar absorption and other heat input;
- long-wave, sensible, evaporative, ground and hydronic heat rejection;
- interval energy residual and expanded uncertainty;
- opening/closing water volume;
- evaporated, make-up, rain, leak, withdrawal, backwash and overflow litres;
- evaporation latent-heat cross-check residual;
- heat rejected per cover mode and weather bin;
- source rights/quality status, cover-state provenance and evidence traceability.

Never use "pool cooling" as one scalar result. At minimum distinguish:
1. thermal buffering of building heat;
2. actual regeneration through named heat-rejection paths;
3. water consumed by that regeneration;
4. measured occupied-zone or outdoor-comfort response, which requires a separate evidence layer.

## 7. Current research comparison

- A March 2026 experimental/theoretical open solar PV/T pool study reports seasonal behavior changes and evaporative heat loss dominance in its tested system. This supports measuring evaporation and season-specific boundaries, not assuming the same proportion for Johannesburg.
- A March 2026 nighttime outdoor-pool study validates a reduced-order aggregate nighttime balance on 13 independent nighttime realizations, but its additive effective-coefficient structure limits independent identification of each flux. A Highveld field package should separately meter fluxes when claiming mechanism-specific effects.
- An August 2026 water-channel study uses validated CFD/wind-tunnel methods and finds wind and canyon configuration materially affect water-channel cooling; it also reports configurations with poorer daytime thermal comfort from long-wave radiation.
- A July 2026 blue/green/grey-space review identifies the need for cross-scale, climate-responsive evidence.

The useful Highveld advance is not to claim a new universal pool heat-transfer model. It is a reproducible, rights-aware, measured thermal-and-water ledger that exposes when apparent heat rejection merely consumes water, when a pool becomes saturated, or when a microclimate effect does not translate into occupant service.

## 8. Claim ceiling

**Status:** SYNTHETIC_CONSERVATION_CONTRACT_NOT_PHYSICALLY_VALIDATED. The committed case is arithmetic only. No pool, rill, cistern, pump, cover, heat exchanger, field sensor or Johannesburg weather observation has been measured or simulated by this ledger. No passive-cooling, water-saving, comfort or economic benefit is claimed.
