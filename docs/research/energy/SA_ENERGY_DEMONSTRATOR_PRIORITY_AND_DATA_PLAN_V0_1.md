# South African Energy Demonstrator Priority and Data-Acquisition Plan v0.1

**Status:** research prioritization, 9 November 2026. This is not a site selection, investment recommendation, plant design or operator authorization.

## Executive recommendation

The first South African demonstrator should begin with **retrospective qualification of an existing asset**, not a new greenfield energy plant. The strongest immediate path is to obtain original interval data and operator access for existing conduit PAT/hydropower installations, then construct a metered reference and blind operational benchmark.

In parallel, build a standard PV + battery + grid / backup baseline for the same useful service. A more complex generation architecture should beat that baseline on matched service and full system costs—not on nameplate capacity or theoretical resource potential.

Mine-water thermal should proceed as a measured heat-service investigation rather than an electricity-generation project. Marine current, wave, offshore wind, OTEC and geothermal electricity remain longer-horizon resource/engineering research until site measurement and survivability evidence improve.

## 1. Candidate ranking — ranked by evidence acquisition leverage, not theoretical resource size

| Priority | Candidate | Evidence currently in hand | Why it is valuable | Main missing evidence / gate |
|---|---|---|---|---|
| 1 | Leliefontein, Drakenstein bidirectional pump/PAT | Published South African full-scale installation since 2018; grid connection, bidirectional meter and SCADA described | Longest-running local candidate; tests pump-mode vs turbine-mode net import/export, controls, bypass and product lifecycle | Operator raw interval exports, meter units/calibration, SCADA mode chronology, current operational status and exact pump/turbine maps |
| 2 | Queenswood PAT pilot | WRC report gives five timed measured power/flow/head observations; linked 2014 paper describes same pilot | Small, concrete, local measured points for first map/ledger reproduction | Original notebooks/telemetry, instrument calibration/uncertainty, test interval, actual electric boundary, current status; do not count 2014 paper as second plant |
| 3 | Pierre van Ryneveld (Tshwane) conduit hydro pilot | Three field experiments; ~15 kW maximum capacity statement; cross-flow turbine and synchronous generator described | Useful controller and integration negative controls: frequency control, sudden load rejection, PRV hunting, variable flow/head and manual operation | Raw field logs, interval energy, head/flow chronology, grid protection, current operation and service records |
| 4 | New South African WWTW / WTW candidates | DWS/SAHA location catalogue and South African Hydropower Atlas screening | Potentially repeatable product-market channel across municipal water/wastewater assets | Exact owner/site, measured flow/head, operating chronology, effluent quality, outlet constraints, rights and environmental service |
| 5 | Mine-water thermal + nearby heat customer | CGS and CSIR/SANEDI research/context; international mine-water operating and storage examples | Could displace boiler/chiller energy as useful heat/cooling, possibly lower-risk than power generation | Mine-level flow/renewal, temperature, chemistry, pumping/treatment load, consumer heat profiles, pipe distance, reinjection and rights |
| 6 | PV + BESS / solar thermal industrial heat | Global resource atlas and cost references; local market/CSIR thermal context | Essential matched baseline and potentially first commercial product route | Site-specific hourly load, tariff, grid constraint, weather, roof/land, fire/grid approvals, measured system boundary |
| 7 | Wind / hybrid PV-wind | WASA3 and Global Wind Atlas planning layers | Useful hybrid/firmer portfolio comparison at selected inland/coastal sites | Mast/lidar measurements, terrain/RIX, wake, grid, EPC/O&M and geotechnical evidence |
| 8 | Marine current, wave, OTEC / offshore wind | DFFE planning/resource context, international pilots and cost evidence | Long-horizon differentiated research; Agulhas Current and wave are distinct from tidal range | Depth-resolved metocean measurements, bathymetry, device survivability, marine environmental approval, vessel/cable access and maintenance economics |
| 9 | Geothermal electricity / EGS | South African low-enthalpy prospectivity literature; Fervo commercial EGS external comparator | Useful exploration and international process comparator | Deep temperature/permeability/flow, drilling economics, water management and reservoir lifetime; no proven South African commercial power reservoir admitted |

The rankings represent **the next best evidence/engineering investment**, not a claim that one asset has more energy potential than another.

## 2. Workstream A — Leliefontein retrospective qualification

The IMESA 2021 publication describes a bidirectional municipal pumping/PAT station in Drakenstein, in operation since June 2018, with active-front-end variable-speed drives, actuated valves, bidirectional power metering, grid protection and SCADA.

The paper states estimated generation of 320 MWh/year and 9.9 GWh over 2018–2041. It also reports 271,236 MWh generated and 53,577 MWh consumed, which is incompatible with those annual/lifetime estimates. The cumulative units remain **unresolved**; none may be used as MWh or project yield until confirmed by the operator/authors.

### Minimum data request

1. Original bidirectional meter exports, all registers and import/export direction.
2. Timestamp convention/timezone, sample interval, clock quality and meter serial/calibration certificate.
3. Pump/turbine mode, valve position, bypass state, trip and availability timeline.
4. Flow, suction/discharge pressure, head, pump speed, generator speed and power/converter telemetry.
5. Auxiliary consumption, active-front-end losses, standby energy and grid outage records.
6. KSB pump model identity, turbine-mode curve/map, revisions, BEP assumptions and any independent test.
7. Maintenance, seal/bearing replacements, faults, protection events, water interruptions and current operational status.
8. CAPEX/OPEX, civil works, electrical upgrades, spares and actual installation/commissioning records.
9. Operator and municipal approval for data access, source use and any site visit.

### Reconciliation rules

- Keep pump-mode grid import and turbine-mode export as separate time-aligned series.
- Reconcile to net annual import/export only after interval and register semantics are verified.
- Keep gross turbine electrical power, auxiliaries and net export separate.
- Reconcile hydraulic input `rho*g*Q*H`, turbine shaft, generator losses, auxiliary load and net electrical output.
- Missing, ambiguous or contradictory intervals remain unresolved; do not interpolate a whole operating history to make the annual estimate fit.
- The 2017 ASCE and 2021 IMESA papers may describe the same Leliefontein station; verify installation identity before counting independent evidence.

## 3. Workstream B — Queenswood and Pierre van Ryneveld replay

### Queenswood

The WRC KV 238/10 report gives five timed observations. It reports measured electrical power of approximately 2.32–4.52 kW at flows of 27.55–32.49 L/s and head drops of 25.04–30.94 m. The report says the unit was not optimized and the observations were initial trial runs. Its source table heading calls a column “output energy”, but the values are in watts; preserve the W unit and do not convert to kWh.

The 2014 Tshwane paper discusses the same Queenswood pilot. It is not an independent installation or replication.

First tasks: reconstruct the five operating points, meter boundary, head/flow locations, minimum-efficiency calculation and uncertainty; seek the original test notebooks; do not annualize five samples.

### Pierre van Ryneveld

The 2014 Tshwane paper documents three field experiments with a cross-flow turbine and synchronous generator, around 15 kW maximum depending on flow/head. It reports frequency-control problems, sudden load rejection, PRV hunting due to slow response, variable flow/head and manual operation.

Turn these into specific benchmark cases:
- sudden load rejection and overfrequency/overspeed response;
- pressure oscillation/PRV hunting;
- low and high water demand;
- loss of instrument signal;
- generator trip and safe bypass;
- manual fallback and restart;
- operator-authorized setpoint changes only.

## 4. Workstream C — standard PV/BESS and industrial heat baselines

For any candidate service site, establish:
- time-aligned electricity demand and critical/flexible split;
- heat demand by temperature grade and schedule;
- existing boiler/chiller/generator energy and auxiliary boundaries;
- import/export tariffs, demand charges, outage chronology and grid connection limits;
- solar irradiance / temperature and wind distributions with source date and uncertainty;
- space, structural, access, fire, water, noise and planning constraints.

Compare no-build, efficiency-only, grid + backup, PV, PV + BESS, wind, PV-wind-BESS and solar thermal / heat-pump alternatives. Use the existing Sol-Atlas #295–#309 pathway, heat-pump, storage, exergy, service, reliability and exact dispatch modules. Do not create a second LCOE or dispatch engine.

## 5. Workstream D — mine-water and industrial heat shortlist

A mine-water case needs a paired source and customer, not a mine location alone.

Minimum source:
- measured seasonal flow and replenishment;
- temperature at relevant levels and operating modes;
- pumping/treatment power and hydraulic head;
- chemistry, hardness, metals, corrosion, scaling and reinjection quality;
- water rights, remediation obligations and safe reinjection/discharge constraints.

Minimum customer:
- hourly heat/cooling loads, required temperatures, flow and return temperatures;
- existing boiler/chiller fuel, efficiency and operating costs;
- distance, elevation, network losses and backup needs;
- process continuity, maintenance windows and customer/site agreement.

Run direct heat exchange, heat pump, heat pump + storage, waste-heat recovery and solar-thermal hybrids against existing boiler/chiller. Treat mine water as a low-grade thermal resource unless measured evidence supports a different claim.

## 6. Shared metrics and acceptance

Every demonstrator should report these separate metric families:

**Service:** useful electricity/heat/cooling delivered by interval, critical service shortfall, outage duration, quality/temperature match, recovery time.

**Physics:** water/energy/thermal/exergy balances, map coverage, operating-domain violations, pressure and flow residuals.

**Net system:** gross conversion, auxiliaries, pumping/treatment, distribution loss, storage losses, curtailment and net import/export.

**Reliability:** availability, forced outages, bypass/trip modes, MTTR, starts/stops, fouling and degradation.

**Manufacturing/construction:** configuration/BOM revision, first-pass acceptance, rework/scrap, accepted output per crew-hour, tie-in duration, inspection and commissioning evidence.

**Economics/environment:** full CAPEX/OPEX, financing, replacement, maintenance, local supply-chain evidence, water use, carbon, materials, land/sea impacts and end-of-life.

No weighted score can compensate for a water-service, environmental, safety or essential-service breach. Unknown costs/impacts are not zero.

## 7. Next actions

1. Secure a scoped operator data-access request for Leliefontein; start with the bidirectional meter and SCADA export.
2. Request original Queenswood and Pierre van Ryneveld pilot records from the University of Pretoria, Tshwane and WRC/CSIR custodians.
3. Build a reproducible synthetic PAT reference dataset with explicit power/energy units and known-answer balances.
4. Build a matched South African PV/BESS baseline for the same hypothetical service boundary.
5. Identify one mine-water source/customer pair only when both sides have measured data and appropriate operator/authority access.
6. Use Symtropy #4386/#4401 for PAT component energy/uncertainty; Symtropy #4397/#4398 for network dynamics/parity; Symtropy #4420 for the product and rig; Sol-Atlas #295–#309 for portfolio/service accounting; Symthaea for proposal/experiment design; Mycelix for decision and inspection provenance.

## Claim ceiling

This shortlist ranks data-acquisition opportunities. It does not rank site suitability, declare current operating status, establish available energy, or authorize site interventions. A physical demonstrator requires operator access, competent site engineering, water-use/environmental and electrical approvals, independent inspection and commissioned measurement.
