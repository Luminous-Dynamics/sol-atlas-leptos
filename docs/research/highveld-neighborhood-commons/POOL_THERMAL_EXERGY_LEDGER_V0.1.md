# Pool thermal exergy and temperature-grade ledger v0.1

**Study ID:** HIGHVELD-POOL-THERMAL-EXERGY-001  
**Parent:** HIGHVELD-POOL-PASSIVE-REGENERATION-LEDGER-001 / HIGHVELD-POOL-WATER-THERMAL-001  
**Maturity:** SYNTHETIC_EXERGY_ACCOUNTING_CONTRACT_NOT_PHYSICALLY_VALIDATED  
**Purpose:** distinguish sensible heat headroom from temperature-grade availability in a stratified water store.

## 1. Why kWh alone is insufficient

Two pools can contain the same thermal energy above a reference state but have different temperature-grade value. A pool near the outside air temperature may have little remaining cold-side exergy even though its nominal thermal capacity is large. A colder pool has more cold-side exergy, but may still be unable to accept a particular heat load if the heat-exchanger approach, flow rate or maximum water temperature prevents transfer.

The ledger therefore reports three distinct quantities:
1. **Sensible heat headroom:** idealized heat the water can absorb before reaching the owner/authority-set temperature ceiling, based on layer mass and heat capacity.
2. **Thermal exergy relative to a measured environment reference:** temperature-grade availability in the water layers relative to the outdoor reference temperature for that interval.
3. **Useful building-side cooling:** not calculated or claimed by this sidecar; it must come from a separate measured/qualified exchanger and occupied-zone path.

Exergy is a quality measure. It is not additional energy and must not be added to first-law heat balances.

## 2. Layer-resolved exergy

For each water layer \(i\), using an incompressible constant-specific-heat screening approximation:

\[
B_i = m_i c_p \left[(T_i-T_0)-T_0\ln\left(\frac{T_i}{T_0}\right)\right]
\]

where all temperatures are absolute Kelvin, \(m_i\) is layer water mass, \(c_p\) is the supplied specific heat and \(T_0\) is the measured environmental reference temperature. The result is non-negative; it is reported as:
- **cold-side exergy** when \(T_i < T_0\);
- **hot-side exergy** when \(T_i > T_0\).

The calculation is performed separately for each layer at interval start and end. A layer's mass is derived from pool water volume × supplied density × its measured/modelled mass fraction. Start and end layer fractions must each sum to one. Layer state, density, heat capacity and reference-temperature sources require evidence references.

For a fully mixed store the same formula can be applied to its single validated bulk state. This case does not assume full mixing: it uses three synthetic layer records. A future physical application requires the actual geometry and a defensible layer-mass/temperature mapping.

## 3. Sensible heat headroom

The idealized upper-bound sensible headroom to the declared water-temperature ceiling is:

\[
H = \sum_i m_i c_p \max(0, T_{\max}-T_i)
\]

with temperature differences in Kelvin. This is not a promise that the heat exchanger can deliver or absorb this amount. Actual useful capacity also depends on flow, heat-transfer approach, thermal stratification, control constraints and the building load.

The report labels this quantity as **idealized sensible headroom**, not deliverable cooling capacity.

## 4. Reference temperature and uncertainty

Each interval includes:
- environmental reference temperature and a measurement/method reference;
- water density and property-method reference;
- specific heat and property-method reference;
- layer-state evidence reference;
- exergy calculation uncertainty reference and expanded \(u_{95}\).

The environmental reference must correspond to the thermal boundary being assessed. In this synthetic fixture it is the outdoor dry-bulb reference at the pool exposure. If a study needs a building-side reference, it must define that boundary separately; do not silently substitute one reference for another.

The constant-\(c_p\), constant-density model is a screening calculation. Physical qualification must use temperature-appropriate water properties, measured calibration/uncertainty, and the selected property source. If the pool water chemistry materially changes the properties, pure-water defaults cannot be presumed.

## 5. Synthetic known-answer case and controls

The case uses three intervals corresponding to the pool passive-regeneration fixture:
- two hot periods where pool water is colder than the outdoor reference and therefore has cold-side exergy;
- a cool period where the pool is warmer than the outdoor reference and the ledger reports hot-side rather than cold-side exergy;
- three layers (top/mid/bottom) with separate start/end temperatures and mass fractions.

Known-answer tests verify:
- positive cold exergy below the reference and hot exergy above it;
- a warmer pool closer to the environmental reference has lower cold exergy for the same mass/property model;
- equal nominal heat capacity at different temperature grades produces different exergy;
- layer mass fractions that do not close to one are rejected;
- missing property/reference evidence, non-positive absolute temperature or temperature-limit breach cannot qualify.
- non-negative exergy across the tested temperature range and strict monotonic increase with temperature distance from the environmental reference, separately on cold and hot sides;
- linear scaling with water mass and supplied specific heat under the constant-property screening model;
- rejection of negative mass, non-positive specific heat, non-finite temperatures and absolute temperatures at or below zero Kelvin.

## 6. Cross-ledger identity and state reconciliation

The exergy sidecar now embeds the parent passive-regeneration case and checks it before calculating temperature-grade results. The parent case ID, thermal/water opening and closing state for every interval, usable store minimum/capacity, initial state, maximum temperature limit, cycle-closure requirement and closure tolerance must match. The interval order and identities must match exactly.

A mismatch fails the cross-ledger gate. It is not repaired by interpolation or by selecting the nearest interval. The environmental reference temperature is intentionally not copied from the parent: it is a separately measured boundary required for exergy, but its measurement method and uncertainty are independently required in this sidecar.

This makes the exergy sidecar an additional analysis of the same first-law trajectory, rather than a second, potentially inconsistent synthetic pool history.

## 7. How to use this with the existing system

- The pool passive-regeneration ledger remains the first-law and water-volume accounting authority.
- The exergy sidecar provides temperature-grade and idealized headroom metrics for the same kind of stratified water store.
- The sink-dispatch governor remains the operational eligibility gate and still checks measured temperature approach, flow, water entitlements and store headroom.
- EnergyPlus / Modelica / specialist exchangers remain the physics authorities for actual loop and building performance.

Do not use the exergy number by itself to dispatch a real heat exchanger or claim occupant cooling.

## 8. Acceptance targets and claim ceiling

- zero negative/non-finite property inputs accepted;
- zero layer mass-fraction gaps or double counting;
- zero environmental-reference substitution without a new declared boundary;
- 100% layer-state, property-method, reference-temperature and uncertainty traceability;
- exact agreement with an independent known-answer calculation within declared numerical tolerance;
- separately report cold exergy, hot exergy and idealized heat headroom;
- never add exergy to energy or water conservation totals.

**Status:** SYNTHETIC_EXERGY_ACCOUNTING_CONTRACT_NOT_PHYSICALLY_VALIDATED. No pool measurement, property certification, EnergyPlus/Modelica co-simulation, exchanger qualification or delivered cooling result is admitted.
