# Multi-engine thermal-water benchmark v0.1

**Study ID:** `HIGHVELD-MULTI-ENGINE-THERMAL-WATER-001`  
**Parent:** `HIGHVELD-PASSIVE-COOLING-WATER-001`  
**Maturity:** `BENCHMARK_PREREGISTERED_NOT_EXECUTED`

## 1. Purpose

Establish a matched benchmark that uses specialist physics/control engines for the domains they already solve, then qualifies the combined design against resident comfort, water rights, water quality, water and energy conservation, uncertainty and physical measurements.

This is not an attempt to replace EnergyPlus, Modelica Buildings, OpenFOAM, OpenStudio or BOPTEST. The research contribution is the evidence-carrying bridge between them and the real service: a person experiences a safe, comfortable space while essential water remains protected.

## 2. Pinned engine roles

| Engine ID | Pin | Role | Benchmark use |
|---|---|---|---|
| `ENERGYPLUS_26_2_0` | EnergyPlus 26.2.0 | Building/zone heat and moisture balance | No-water baseline; supported indoor pool reference; plant and water-use objects |
| `MODELICA_BUILDINGS_14_0_0` | Buildings Library 14.0.0 + named Modelica environment | Dynamic hydronic systems, storage, controls and ground plant | Closed-loop pool/cistern store; plant-control counterfactual |
| `OPENSTUDIO_3_11_0` | OpenStudio SDK 3.11.0 | Model authoring/workflow layer around EnergyPlus | Reproducible model build/workflow; not counted as an independent physics result |
| `BOPTEST_0_9_0` | BOPTEST v0.9.0 | REST/control test harness | Same-trace control policy comparison or a clearly versioned custom case |
| `OPENFOAM_FOUNDATION_14` | OpenFOAM Foundation v14 | Outdoor CFD | Courtyard airflow, buoyancy and water-surface boundary model |
| `OPENFOAM_OPENCFD_2606` | OpenCFD OpenFOAM v2606 | Alternative CFD distribution | Separate sensitivity/replication arm; never silently treated as the Foundation binary |

Modelica 14.0.0's EnergyPlus-linked package must be version-pinned separately: its release notes identify `Buildings.ThermalZones.EnergyPlus_24_2_0`. No turnkey 26.2 coupling is assumed. OpenStudio is not independent from EnergyPlus where it launches the same EnergyPlus executable. The two OpenFOAM releases are different distributions; configuration and results cannot be assumed interchangeable.

## 3. Canonical scenario ladder

| Case | Question | Required engine(s) | Evidence state |
|---|---|---|---|
| `S0_ANALYTICAL_KNOWN_ANSWERS` | Do unit conversions, storage/latent/pump calculations and water balances reproduce finite known answers? | Rust screen + independent arithmetic oracle | Synthetic |
| `S1_INDOOR_POOL_REFERENCE` | Does the supported EnergyPlus indoor-pool model reproduce the selected reference/known-answer case within its assumptions? | EnergyPlus | Not run |
| `S2_NO_WATER_PASSIVE_BASELINE` | What is the comfort/energy response of the competent passive building with no added water? | EnergyPlus; Modelica where plant dynamics are relevant | Not run |
| `S3_OUTDOOR_COURTYARD_RILL` | What air-flow, humidity and occupied-zone changes occur around a small, shaded, low-splash feature? | OpenFOAM or validated reduced-order model + EnergyPlus coupling | Not run |
| `S4_CLOSED_LOOP_THERMAL_STORE` | Can a covered pool/cistern transfer useful heat through a measured loop and qualified heat sink? | Modelica Buildings + EnergyPlus boundary coupling | Not run |
| `S5_CONTROL_POLICY_COMPARISON` | How do fixed schedule, rules and MPC compare on the exact same weather/plant/disturbance trace? | BOPTEST-style REST harness with selected plant model | Not run |
| `S6_WATER_CURTAILMENT_AND_SOURCE_FAILURE` | Does operation preserve essential water floors and shut down when quality/rights/restrictions fail? | Highveld contracts + Mycelix; specialist plant models | Not run |
| `S7_HUMID_HOT_AND_COLD_NIGHT` | Does the design avoid humidity/condensation penalties and winter overcooling? | EnergyPlus + outdoor CFD/reduced model as applicable | Not run |
| `S8_COMPONENT_AND_SENSOR_FAULTS` | What happens under pump trip, sensor drift, flow-meter failure, network loss and blocked maintenance? | Symtropy native traces + plant/control test harness | Not run |
| `S9_PAIRED_FIELD_TEST_CELL` | Does the selected intervention create a measurable comfort benefit within water/energy and safety limits? | Paired test cell + instruments + independently replayed analysis | Not run |

## 4. Common frozen package

Every matched case must bind:
- geometry, orientation, shading, material and model revision digest;
- weather source, location/grid/station metadata, time basis, file digest and scenario classification;
- occupancy, internal gains, schedules, ventilation and comfort criteria;
- water budget, reserve floor, source class, source-rights and water-quality contract;
- plant topology, water inventory, pump/heat-exchanger parameters and control version;
- solver executable, library, environment, configuration and convergence/timestep settings;
- sensor registry, calibration evidence, uncertainty and validation split;
- independent thermal, moisture and water-balance oracle versions.

All arms must use the same external trace and information horizon. A TMY, AMY, extreme year and future-climate series are different scenario classes; do not splice or relabel them without an explicit transformation.

## 5. Interface variables and conservation

At each co-simulation boundary, specify the exchanged variables, units, sign convention, interval semantics, interpolation, synchronisation and missing-data behavior. Candidate variables include:
- zone/surface/ambient dry-bulb temperature and humidity ratio;
- radiant or operative-temperature proxies and solar/long-wave flux;
- water temperature, flow, volume, source and quality state;
- heat-exchanger inlet/outlet temperatures and mass flow;
- pump/filter electrical power;
- outdoor wind boundary, courtyard surface state and water evaporation estimate.

Compare only physically equivalent signals. For example, courtyard air temperature at a point is not directly interchangeable with zone operative temperature. Latent heat at the water interface is not useful zone cooling. Storage capacity is not cooling delivery.

At every boundary, report energy and moisture residuals. A non-convergent or non-conservative exchange is a failed model case, not a result to smooth away.

## 6. Benchmark metrics

Report separate vectors, with units, uncertainty and per-household/use-zone attribution:
- occupied-hour operative temperature, discomfort degree-hours and peak duration;
- relative humidity, dew point and condensation events;
- heat transfer by sensible, latent, radiative and conductive path;
- useful cooling to the occupied zone only where independently supported;
- water make-up by source, evaporation, splash, purge/backwash, leakage, overflow and closing balance;
- pump, filtration, treatment, control and backup electricity;
- water litres per useful cooling kWh only after useful cooling has been qualified;
- peak power, carbon intensity, maintenance labour, downtime and repairability;
- predictive bias, RMSE, interval coverage, calibration by regime and out-of-domain detection;
- comfort/water/energy constraint violations and safe degraded-mode behavior.

Do not create one universal cooling score that lets water consumption or household discomfort disappear into a weighted average.

## 7. Proposed research targets

These are target gates, not achieved performance:
- 0 unauthorized essential-water-floor violations;
- 0 operation on unknown/failed source rights, water quality or restriction state;
- water and energy balances close within propagated measurement/model uncertainty on held-out cases;
- 0 useful cooling credits without an independently qualified zone-side outcome;
- 0 critical out-of-domain decisions accepted;
- 90%/95% interval coverage assessed on held-out weather regimes, with calibration uncertainty reported;
- 0 hard safety/control constraint violations in the preregistered fault set;
- 100% of reported results trace to exact geometry, weather, solver, control, instrument and schema digests.

A benchmark win requires a Pareto improvement at matched water/comfort constraints, or materially better robustness under identical disturbances, with independent field evidence. Runtime alone, an unmatched simulated temperature drop or a high R² on training cases is not a win.

## 8. Validation sequence

1. Freeze S0 analytical controls and source manifests.
2. Reproduce S1 EnergyPlus indoor pool reference without changing object assumptions.
3. Build S2 no-water passive baseline and validate zone/energy balance.
4. Build S3 CFD/reduced-order courtyard case; calibrate on a development split and preserve a held-out field split.
5. Build S4 closed-loop hydronic store and compare dedicated thermal-storage alternatives.
6. Build S5 controller harness with fixed, rule-based and MPC policies using the same trace.
7. Execute S6–S8 negative and fault controls; unknown critical states fail closed.
8. Run S9 paired field test cell only after engineering, water-quality, safety and municipal approvals.

## 9. Owner boundaries and current state

- **Sol-Atlas:** geometry, model/BOM revision, spatial placement and provenance.
- **Symtropy:** event identity, clock/sequence, traces, co-simulation time alignment, trajectories and counterfactual replay.
- **Mycelix:** water rights, consent, source quality, allocation/reserves, instrument admission and commitments.
- **EnergyPlus / Modelica / OpenFOAM / BOPTEST:** their specialist physical/control roles.
- **Independent evaluator:** validation splits, balances, uncertainty and claim adjudication.

The manifest next to this document pins the initial tools and scenarios. Current state remains `BENCHMARK_PREREGISTERED_NOT_EXECUTED`. No EnergyPlus, Modelica, OpenStudio, BOPTEST or OpenFOAM simulation, physical field test, cooling benefit or superiority claim is reported.
