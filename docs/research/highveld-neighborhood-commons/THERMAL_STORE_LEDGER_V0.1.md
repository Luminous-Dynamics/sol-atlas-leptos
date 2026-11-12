# Finite thermal-store trajectory ledger and cycle-closure benchmark v0.1

**Study ID:** HIGHVELD-THERMAL-STORE-LEDGER-001  
**Parent:** HIGHVELD-THERMAL-SINK-CASCADE-001  
**Maturity:** DETERMINISTIC_LEDGER_CONTRACT_NOT_PHYSICALLY_VALIDATED  
**Purpose:** independently check measured/replayed thermal-store energy trajectories, finite capacity, path-specific heat rejection, and seasonal cycle closure around specialist models.

## 1. Paradigm shift: thermal storage is a trajectory, not a nameplate capacity

Nameplate kWh and latent-heat enthalpy are not equivalent to useful cooling. The real service depends on:
- the temperature/enthalpy range in which a store can operate;
- charge and discharge power under real exchanger and flow constraints;
- stratification, phase-change hysteresis and incomplete cycling;
- how quickly the available sink regenerates the store;
- parasitic electricity and any water consumed;
- degradation, fouling and capacity retention;
- the occupied-zone demand and acceptable service floor.

The store should be represented as a time-ordered state trajectory. Every interval must reconcile start state + measured heat received - measured heat rejected = end state within propagated uncertainty. The state must remain within a declared usable range. A yearly/seasonal cycle must close within a predeclared tolerance; otherwise repeated operation has consumed thermal headroom or accumulated heat.

This is an independent accounting/qualification layer. It does not reproduce PCM phase physics, stratified tank dynamics, exchanger performance or fluid simulation.

## 2. Scope and energy boundary

The ledger's store_energy_kwh means the usable thermal energy state above a declared reference state, derived from admitted measurements or a qualified specialist model. For a stratified tank, it must represent a layer-resolved enthalpy sum or an independently validated equivalent. A single bulk temperature must not erase material stratification.

For each interval:

\[
E_{end}=E_{start}+Q_{building\rightarrow store}+Q_{other,in}
-\sum_j Q_{store\rightarrow sink_j}
\]

The per-interval residual is:

\[
r=E_{start}+Q_{in}-Q_{out}-E_{end}
\]

The absolute residual must be no greater than the interval's propagated expanded uncertainty (u95_kwh). Missing or unknown heat terms block qualification; they are never silently converted to zero.

Electrical parasitics (pumps, fans, valves, treatment) are recorded separately. Only the portion physically entering the store boundary as heat belongs in a qualified thermal input term. Electrical energy cannot be subtracted from thermal heat to create an arbitrary efficiency.

## 3. Input and output contract

The synthetic input case demonstrates:
- one finite store with a declared minimum/maximum usable energy range;
- initial energy state and cycle-closure tolerance;
- contiguous intervals with duration and start/end store energy;
- building heat into the store and other explicitly metered heat input;
- path-specific rejection through radiative, dry-air and ground sinks;
- separate parasitic electrical energy and water make-up;
- interval expanded uncertainty.

The Rust audit:
1. validates the structural contract and exact known sink identities;
2. rejects duplicate sink rows and negative/non-finite energy terms;
3. verifies interval continuity;
4. verifies energy balance against the supplied uncertainty;
5. rejects energy states outside the usable capacity;
6. checks seasonal cycle closure when required;
7. returns totals and the exact violation list.

The audit report is not a simulator output. It does not prove that its input measurements or model states are true.

## 4. Matched tests and falsification

The registered known-answer tests cover:
- balanced cycle with three intervals and distinct heat-rejection paths;
- capacity overflow under multi-day heat input;
- rejection greater than stored + incoming heat;
- interval energy residual outside uncertainty;
- interval discontinuity;
- cycle drift above the declared tolerance;
- duplicate or unknown sink path IDs;
- negative or non-finite energy terms.

The positive control must remain admissible. A test suite that rejects all inputs is not a valid safety gate.

## 5. Benchmark metrics

Report separately:
- usable measured/modelled capacity (kWh);
- maximum state of charge and minimum headroom;
- peak and time-integrated heat received (kW/kWh);
- heat rejected by each sink path (kW/kWh);
- charge/discharge power and temperature approach from the specialist engine/measurements;
- interval energy-balance residual and propagated uncertainty;
- cumulative seasonal store drift / cycle-closure error;
- parasitic electricity (kWh) and water make-up (L) by source class;
- unmet building load / occupied-zone discomfort from the system simulation or field measurement;
- degraded-capacity and incomplete-charge recovery across repeated cycles.

No single W/kg, Wh/L or peak kW figure is sufficient. Compare useful, cycle-available thermal service at matched volume, mass, cost, safety and operating temperature range.

## 6. Industry / SOTA position

- EnergyPlus 26.2 ThermalStorage:PCM: a plant-connected PCM tank that charges/melts and discharges/freezes through an internal heat exchanger with a user-specified effectiveness; its PCM material uses a phase-change hysteresis property. Use it for its documented model boundary and pin all inputs.
- EnergyPlus stratified/mixed chilled-water storage: use the applicable tank object where the physical store and flow assumptions fit.
- Modelica Buildings: use for dynamic plant/loop/control and storage couplings supported by the pinned library release and solver.
- Specialist PCM material and CFD models: retain phase kinetics, hysteresis, heat transfer, encapsulation and aging; verify them against material/component testing.
- Highveld: add independent finite-store accounting, water/rights constraints, coupled sink availability, per-household service consequences and evidence replay.

A defensible advance is not to claim that our ledger predicts PCM physics better. It is to identify false-feasible designs that pass on nameplate capacity but fail on charge/discharge limits, sink availability, state drift or cycle closure—and to demonstrate zero such failures on a finite, independent known-answer corpus and eventually on held-out field data.

## 7. Acceptance and claim ceiling

Hard acceptance:
- zero interval balance residuals beyond declared propagated uncertainty;
- zero store state outside declared usable bounds;
- zero unknown/duplicate sink paths accepted;
- zero hidden heat or water terms;
- zero missed seeded continuity/capacity/cycle negative controls;
- all valid positive controls preserved;
- exact provenance for model, measurements, uncertainty, inputs and transformations;
- 100% finite-oracle agreement.

**Status:** DETERMINISTIC_LEDGER_CONTRACT_NOT_PHYSICALLY_VALIDATED. The committed case is synthetic. No PCM, water tank, thermal store, sink path or building has been simulated or measured by this ledger.
