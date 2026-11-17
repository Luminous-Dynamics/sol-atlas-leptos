# EnergyPlus 26.2 thermal-water object/interface map v0.1

**Scope:** map specialist EnergyPlus objects to the Highveld thermal-water study without duplicating or overclaiming their physics.  
**Pinned reference release:** EnergyPlus 26.2.0 documentation.  
**Maturity:** `INTERFACE_MAP_NOT_EXECUTED`; no IDF/EpJSON model has been run in this increment.

## 1. Engineering rule

Use a specialist object only for the physics and system boundary it actually represents. A water-accounting tank is not automatically a thermally stratified pool model. Latent heat at an outdoor water surface is not automatically delivered building cooling. Every object binding needs a version, exact input/output variables, units, schedule semantics, boundary conditions and validation case.

The Highveld stack should orchestrate the evidence around EnergyPlus, not silently implement a competing building heat-balance engine.

## 2. Object map

| EnergyPlus object / family | Use in this study | Important limitation / guard |
|---|---|---|
| `SwimmingPool:Indoor` | Model an indoor pool coupled to the linked zone surface and zone moisture/heat balances when its documented assumptions fit | The model is indoor-specific; the water mass is lumped at the linked floor surface, the pool is assumed to occupy that entire surface, and the surface cannot simultaneously serve as a low-temperature radiant system/ventilated slab. Do not use it as a generic outdoor pool or courtyard model |
| `WaterUse:RainCollector` | Represent a roof catchment/rain collection input to a water-use system | Collection volume is not evidence of water quality, rights, potable status or delivery to a household |
| `WaterUse:Storage` | Account for a water-use storage volume and schedule-driven stored-water temperature / water-use coupling | The documented thermal mode uses a scheduled temperature and is a simplified water-use storage representation. Do not substitute it for a dynamic, stratified pool/cistern thermal-store model |
| `WaterUse:Connections` | Connect water-use equipment, supplies, storage and end-use demands to the plant/water-use accounting boundary | The modelled connection/flow does not establish source rights, quality or a resident's received service |
| `ThermalStorage:ChilledWater:Stratified` | Candidate thermal-storage model where a stratified chilled-water tank is the actual design | It models a dedicated stratified thermal store; do not label an arbitrary pool as this object without a defensible hydraulic/thermal equivalence |
| `ThermalStorage:ChilledWater:Mixed` | Candidate for a well-mixed chilled-water storage sensitivity | Mixing assumption can erase temperature stratification important to an actual pool or cistern |
| `EvaporativeCooler:Direct:*` / `EvaporativeCooler:Indirect:*` | Compare packaged/direct or indirect evaporative air-side strategies where model object assumptions fit | Do not use these as a generic outdoor rill/fountain CFD model; account for air humidity, water use, fan/pump energy and outlet conditions |
| `ZoneCoolTower:Shower` | Explore a zone-coupled cool-tower/shower air path as a separate alternative | It represents a particular cooling-tower arrangement, not arbitrary courtyard surface evaporation |
| `HeatExchanger:FluidToFluid` | Couple a water loop to another plant loop through a declared heat-transfer interface | Must specify flow, setpoints/control, capacity/effectiveness and boundary conditions; object presence is not evidence of net heat rejection |
| Ground heat exchanger family | Candidate external sink/source if the design has a qualified ground loop | Requires site-specific ground, bore/pipe, moisture, spacing and long-term heat-balance evidence; not a free infinite sink |

The object names above are an integration shortlist. The exact 26.2 Input Output Reference / EpJSON schema and object-specific engineering-reference section must be pinned at model creation. Do not infer an undocumented capability from a family name.

## 3. Explicit gap: outdoor water-feature coupling

In the inspected 26.2 object index, `SwimmingPool:Indoor` is listed and no dedicated `SwimmingPool:Outdoor` object was found. This is a scoped observation about that index, not proof that external tools, user-defined components or co-simulation cannot represent outdoor water features.

For outdoor pool/rill/courtyard coupling:
1. Sol-Atlas freezes geometry, surfaces, shade, terrain, access and versioned component parameters.
2. An outdoor airflow/radiation/water-surface model (qualified CFD or validated reduced-order model) computes the external boundary response.
3. A declared co-simulation interface transfers only documented boundary variables to/from EnergyPlus.
4. The indoor building model computes zone loads and comfort response.
5. The coupled run records synchronisation, convergence, units, interpolation, conservation residuals and solver versions.
6. Instrumented data independently validates the coupled boundary and occupied-zone response.

Do not hide coupling lag or transfer errors inside a model residual. If the co-simulation interface does not close its energy/moisture balance within declared uncertainty, the run is not decision-qualified.

## 4. Model-binding packet required for every run

- EnergyPlus version/build, IDD/EpJSON schema digest and executable digest;
- exact object names, field names and schema revision;
- model/geometry/BIM revision and weather-file digest;
- schedules, timestep, warm-up, convergence settings and simulation period;
- initial water temperatures/stratification and makeup-water assumptions;
- surface construction, solar/longwave properties, wind exposure and zone connections;
- water and plant-loop topology, pumps, flow rates, heat exchanger performance and control;
- occupancy, internal gains, comfort criteria and ventilation;
- water source class/rights/quality and reserve floor outside EnergyPlus where relevant;
- raw outputs with units and time basis, post-processing transform version and input/output digests;
- declared validation/applicability domain and independent review.

## 5. Required model-selection tests

Before any design decision:
- reproduce the EnergyPlus indoor-pool reference example for a matching geometry and pool boundary;
- verify energy and moisture balance on known-answer reduced cases;
- test cover transitions, occupancy, evaporation and makeup-water response separately;
- compare the chosen thermal store with mixed and stratified tank alternatives where applicable;
- run mesh/time-step/model-fidelity sensitivity on the external courtyard airflow model;
- run a coupled energy/moisture exchange conservation check;
- preserve held-out test-cell periods for validation rather than calibrating on all data.

## 6. Ownership and claim ceiling

- **EnergyPlus:** building/zone and supported plant/pool object physics.
- **External CFD/reduced-order solver:** outdoor courtyard flow/radiation/water-surface physics.
- **Sol-Atlas:** geometry and model configuration/provenance.
- **Symtropy:** native time/event identity, matched traces, coupled trajectories and replay.
- **Mycelix:** water rights, consent, source quality, reserve/allocation and instrument admission.

This document is an object/interface selection contract, not an EnergyPlus model, simulation result, validation report or claim of superiority.
