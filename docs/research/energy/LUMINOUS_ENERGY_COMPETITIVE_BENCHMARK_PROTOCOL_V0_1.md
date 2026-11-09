# Luminous Energy Engineering — Competitive Benchmark Protocol v0.1

**Status:** proposed benchmark protocol and research targets; no results claimed  
**Date:** 2026-11-09  
**Scope:** generation, storage, grid integration, industrial heat, siting, manufacturing, construction and lifecycle operation.

## 1. The competitive thesis

Do not try to beat every established package at its own specialty. REopt, PyPSA, GenX, SAM and specialist engineering tools already encode years of engineering and solver development. Replacing them wholesale would be the wrong first objective.

Luminous should build an evidence-carrying design and qualification loop around them:

1. Sol-Atlas establishes spatial/resource/site evidence and its limitations.
2. Symthaea proposes diverse architectures, identifies assumptions and selects falsifying experiments.
3. Sol-Atlas exact small-case oracles and specialist external models establish reference answers for supported subproblems.
4. Symtropy runs deterministic time-coupled component/system twins within declared physical scope.
5. Independent auditors recompute conservation, service, feasibility and comparison metrics from raw receipts.
6. Mycelix preserves evidence lineage, review, rights, ownership and approved decisions.
7. Physical prototypes, HIL and field measurements progressively replace synthetic/model-only assumptions.

The differentiator is **better evidenced decisions and safer transfer from concept to buildable service**, not the claim that one in-house optimizer is universally faster or more accurate than every industry tool.

## 2. Industry reference ladder

Pin exact release/commit, configuration, solver, data, licence and system boundary for every run.

| Task | Appropriate external references | What they are good for | What not to claim |
|---|---|---|---|
| PV and solar-thermal plant modelling | [NLR System Advisor Model (SAM)](https://sam.nrel.gov/); [NLR 2025 ATB](https://atb.nlr.gov/electricity/2025/technologies) | PV, CSP, geothermal and other plant/component/financial reference workflows; technology cost/performance assumptions | A SAM annual plant model is not an AC network stability study or a site permit |
| PV performance measurement | IEC 61724-1:2021 | Monitoring classes, measurement and PV system performance analysis, including bifacial and soiling considerations | A synthetic PV trace is not IEC-compliant field monitoring |
| Wind power performance | IEC 61400-12-1:2022 + COR1:2025; IEC 61400-12-3:2022 | Power curve / energy production measurement and site-calibration procedure | A mean wind-speed raster is not a measured turbine power curve |
| Hydropower | NLR ATB 2025 hydropower / pumped storage; applicable IEC turbine acceptance standards (e.g. IEC 63461:2024 for Pelton model tests) | Technology cost/performance references and component/model acceptance where scope applies | A theoretical flow/head estimate is not a turbine acceptance test or water right |
| Geothermal | SAM 2026.7.3 / SSC 308; DOE Next-Generation Geothermal Liftoff and SUPERHOT programmes | Plant-model comparison and long-term development targets for EGS/superhot rock | DOE's $45/MWh enhanced-geothermal target for 2035 is a programme target, not a current South African project cost |
| Marine wave | IEC TS 62600-100:2024 with IEC TS 62600-101 resource characterization | Wave-converter performance assessment, resource transfer and uncertainty reporting | An Agulhas current observation is not a wave-power estimate |
| Distributed energy / microgrid sizing | [REopt API v3 / REopt.jl](https://developer.nlr.gov/docs/energy-optimization/reopt/v3/) | Techno-economic sizing and dispatch of DERs for buildings, campuses, communities and microgrids | Do not compare a one-site dispatch-only fixture to a full REopt planning case with different load, tariff, resilience and investment boundaries |
| Multi-carrier dispatch and network planning | [PyPSA](https://docs.pypsa.org/latest/user-guide/optimization/overview/) | Multi-carrier ED/UC, storage, LOPF/SCLOPF, capacity expansion, stochastic optimization and modelling-to-generate-alternatives | A network-free microgrid oracle is not an AC power-flow or transmission-planning competitor |
| Long-run generation / storage / transmission planning | [GenX](https://github.com/GenXProject/GenX.jl), NLR ReEDS | Capacity-expansion and operational planning at system scales | A small candidate generator or short-horizon dispatch model is not a system-wide planning replacement |
| Grid-forming resources | [ESIG 2025 grid-forming performance tests](https://www.esig.energy/reports-briefs/testing-the-performance-of-grid-forming-resources/) | Voltage-source behaviour test methods and system-strength metrics | Frequency/voltage claims based only on a steady-state energy ledger are invalid |
| Network optimization | PGLib-OPF; SimBench; MATPOWER / PowerModels / PyPSA matched cases | Network model import, OPF feasibility, network dispatch and chronology | A DC linearization is not a full non-linear AC-OPF validation |

### Current reference facts to carry into benchmark manifests

- NLR's 2025 ATB covers costs/performance for land-based and offshore wind, PV, CSP, geothermal, hydropower, PV-plus-battery, utility-scale battery and pumped storage. It is a source of assumptions and scenario ranges, not metered truth.
- NLR's 2025 utility battery ATB includes 2-, 4-, 6-, 8- and 10-hour LFP systems with bottom-up pack/inverter/BOS cost modelling for a 60 MW reference system.
- SAM release notes identify version **2026.7.3 / SSC 308**; geothermal model cost and power-block input options were updated in that release. The exact version must be pinned in any reproducible comparator run.
- REopt API **v3** is the stable API version and uses REopt.jl as its optimization engine.
- PyPSA's current documentation describes multi-carrier economic dispatch, storage, unit commitment, linear/network-constrained OPF, capacity expansion, stochastic optimization and near-optimal alternative exploration.
- IEC TS 62600-100:2024 defines a wave-energy-converter power-performance assessment method in conjunction with resource characterization. This is a suitable marine benchmark seam, not a universal performance guarantee.
- ESIG's September 2025 grid-forming report defines test methods and metrics for voltage-source behaviour.

## 3. Required benchmark layers

A benchmark result must not mix model correctness, optimization quality, physical validation and project viability.

### A. Input and resource fidelity

Report per input family:

- source identity, licence, version, snapshot digest, spatial/temporal extent and transformation;
- coverage, missingness, stale intervals, data latency and uncertainty;
- model-to-observation bias, MAE, RMSE, P50/P90 absolute error and uncertainty interval coverage;
- holdout design by site, season and operating regime.

**Targets:** 100% provenance/units/timebase/source-version completeness; zero silent missing-as-zero behaviour; zero mixed-version joins without a declared transformation. Freeze model-specific numerical tolerances before the blind holdout. Do not force one universal accuracy threshold on solar, wind, hydrology, geothermal and marine data.

### B. Physics and component fidelity

Report:

- per-stage energy, mass and carrier balance residuals;
- power, temperature, pressure, flow, state-of-charge, head and operating-envelope violations;
- component performance maps with temperature / load / speed / degradation conditions;
- net output after parasitic loads and auxiliary electricity;
- uncertainty and model-discrepancy against an independent reference.

**Targets:** exact recovery of hand-derived analytic fixtures; 100% rejection of seeded conservation and operating-envelope violations; scale-aware balance tolerances stated before evaluation. For reference-model comparisons, report differences rather than force numerical agreement where the physical models are not equivalent.

### C. Dispatch and optimization

For finite cases, use the Sol-Atlas exact bounded oracle as a reference. For larger cases, compare only matched formulations to REopt, PyPSA or GenX.

Report:
- objective recomputation, primal feasibility and independent audit result;
- solver status, incumbent, best bound, optimality gap, termination reason and time limit;
- essential / flexible unserved energy, longest critical interruption and recovery time;
- storage cycling, reserve shortfall, curtailment, imports/exports, emissions and cost;
- candidate count, runtime, peak memory, compute budget and deterministic replay.

**Targets:** zero false-feasible outputs on the finite hostile panel; exact objective equality on hand-computable fixtures; zero critical-service-floor violation for any case declared protected-feasible; no exact claim when a candidate cap or solver time limit truncates search. On matched larger MILP/LP cases, a practical research target is ≤1% reported optimality gap when a certified bound is available, with solver time and hardware reported. An approximate solution without a valid bound must be labelled approximate.

### D. Architecture search and learning

Compare Symthaea-generated designs against:
1. a human-readable hand-authored baseline;
2. a simple rule-based / greedy baseline;
3. a random/diverse candidate baseline;
4. an established optimizer on representable subproblems;
5. exact finite-model optima for small cases.

Give every method the same source data, forecast information, tuning opportunities, simulator calls, compute/time budget and permitted actions.

Report:
- physically valid candidate rate and hard-constraint rejection reasons;
- feasibility rate and service shortfall;
- Pareto hypervolume / coverage and objective-wise regret on frozen reference sets;
- candidate diversity, novelty (descriptive only), decision regret and compute-to-quality curve;
- calibration of uncertainty and value-of-information of selected experiments;
- ablation of reasoning/search, evidence-aware ranking, scenario selection and individual Symthaea mechanisms.

**Targets:** zero invalid candidates enter the ranked feasible set; recover frozen exact known answers; at least 10,000 deterministic candidate/scenario pairs as a future release goal (not a current result); demonstrate ≥10% improvement on a preregistered primary decision metric only where paired held-out results and uncertainty intervals support it, with no hard constraint, protected service, environmental or lifecycle regression. If HDC/CfC/FEP additions do not improve validity, regret, coverage or compute efficiency under ablation, simplify the system.

### E. Reliability and resilience

Test:
- low resource / multi-day lulls;
- cloud and forecast-error transients;
- cold/heat extremes and component derating;
- storage ageing / reserve boundary;
- grid loss, islanding, black start, reconnection and failed restart;
- common-cause failures and unavailable spares;
- stale/bad sensors, communication loss and model drift;
- repair delay, constrained access, fuel / water replenishment interruption.

Report EENS / unserved service by criticality, loss-of-load hours, longest interruption, reserve margin, black-start success only when tested, recovery trajectory and common-cause cut sets.

**Target:** no hidden critical-service failure inside a cheap aggregate score; zero false acceptance of hard service and safety constraints on the frozen test panel. Reliability probabilities require defensible failure/repair data, not merely the scenario count.

### F. Manufacturing, construction and lifecycle

Measure against the incumbent component and manufacturing process:

- BOM mass/cost/material criticality and supplier concentration;
- process energy, scrap, first-pass yield, defect/rework rate and inspection coverage;
- tooling/setup time, throughput, labour hours, bottleneck capacity and changeover;
- module interchangeability, field assembly time, lift/transport limits and civil works;
- commissioning failures, availability, mean time to detect/repair, spares lead time and maintenance access;
- embodied carbon, water use, land/sea impact, repairability, recovery/recycling and end-of-life cost.

**Targets:** every critical component has a versioned BOM/specification and traceable material/process evidence; no design is labelled buildable until access, foundations, heavy-lift, utilities, logistics and maintenance envelope have been checked. Cost/time/yield improvements must be demonstrated on a matched production run or a clearly labelled model; CAD or a digital twin is not factory evidence.

## 4. Technology-specific scorecards

| Technology | Primary engineering metrics | Important failure / external reference |
|---|---|---|
| Solar PV | net AC yield, performance ratio, temperature/soiling loss, clipping, degradation, availability, water/cleaning burden | IEC 61724-1 monitoring; SAM plant model; inverter/transformer and grid constraints |
| CSP | net electric output, solar-to-thermal conversion, storage dispatch, temperature grade, thermal losses, parasitics, water use | SAM / ATB; thermal-cycle component data and site DNI |
| Onshore/offshore wind | measured power curve, AEP uncertainty, capacity factor, wake/availability loss, structural fatigue, O&M weather access | IEC 61400-12-1 and 12-3; site met mast / lidar / metocean validation |
| Hydropower | turbine efficiency curve vs head/flow, net annual energy, ramp/dispatch, ecological flow, sediment, water loss and availability | IEC model/field acceptance where applicable; hydrology and licensed environmental/water limits |
| Pumped hydro | round-trip efficiency, usable storage MWh, head/flow, evaporation/seepage, start/ramp, lifetime, environmental impact | NLR ATB; measured pump-turbine performance and reservoir constraints |
| Geothermal | sustainable well productivity, net electric/thermal output, drilling rate/cost, well failure, corrosion/scaling, pump parasitics and decline | SAM; DOE EGS/SUPERHOT targets; temperature-depth-flow evidence |
| Wave/current/tidal | resource-to-wire capture efficiency, capture-width ratio, annual energy uncertainty, survival/extreme load, fatigue, mooring/cable availability and service-vessel access | IEC TS 62600 series; wave-buoy / ADCP / hindcast validation |
| Batteries | AC-to-AC RTE, usable capacity, power limits, thermal response, cycle/calendar degradation, safety and replacement cost | NREL SAM / ATB; cell and pack test evidence |
| Flow / iron-air / sodium-ion | usable energy, power/energy decoupling where relevant, RTE, degradation, electrolyte/material burden and lifecycle cost | Independent cycle tests and field duration; compare on identical duty cycles and system boundary |
| Thermal storage | delivered heat/cold by temperature grade, energy/exergy, standing and conversion loss, pump power, cycling, fouling and useful service | Analytic state references, SAM/STEP 1 where applicable, measured thermal cycles |
| Hydrogen / power-to-X | input electricity, conversion yield, compression/storage/transport, leakage, round-trip efficiency if reconverted, feedstock and end-use substitution | Plant-level LCA and process reference; do not compare chemical feedstock service to electricity storage as if identical |

## 5. How to beat the state of the art responsibly

The target is not one universal winner. The programme should seek improvements in the following sequence:

1. **Find a real weakness in the baseline.** Examples: excess curtailment because thermal demand is ignored; battery dispatch with incomplete degradation; plant designs that cannot connect to available grid capacity; a hydro resource estimate that ignores ecological flow; geothermal heat that is mislabeled as electricity; offshore equipment with no maintenance access plan.
2. **Build the smallest counterexample.** Freeze a synthetic/analytical fixture and a negative control that fails the incumbent or the proposed method in a specific, explainable way.
3. **Compare specialist tools on their home territory.** SAM for plant/component cases, REopt for distributed DER sizing/resilience, PyPSA for coupled network/market and chronological planning, GenX for capacity-expansion planning, PGLib-OPF for network formulation, and IEC test methods for component performance.
4. **Demonstrate an evidence-based advantage.** Better prediction error, less decision regret, lower delivered-service cost, less critical load shortfall, lower lifecycle material burden, faster construction, improved maintainability or better reliability—measured separately.
5. **Run ablations and disclose negative results.** If new reasoning or a new physical model does not improve the measured result, do not keep it solely because it is novel.
6. **Advance evidence stages.** Analytical → synthetic → source-calibrated → software-in-loop → processor/controller-in-loop → controller/power-HIL → lab/rig → pilot → multi-season field evidence.

A single simulated benchmark does not support a SOTA claim. An improvement must be reproducible on frozen data, independently recomputed, and survive a blind site/time holdout plus a meaningful physical validation stage appropriate to the claim.

## 6. Release gates

| Gate | Required evidence | Permitted claim |
|---|---|---|
| G0 — Contract | schema, units, provenance, negative controls | contract defined / structurally checked |
| G1 — Analytical | hand-derived known answers, conservation and exact finite oracle | analytically verified within declared model |
| G2 — Synthetic benchmark | frozen corpus, matched baseline, independent verifier | synthetic benchmark result |
| G3 — Source calibrated | pinned real datasets, calibration and blind holdout | model calibrated for declared domain |
| G4 — SIL / PIL / CHIL / HIL | real controller/software/hardware and test rig evidence | tested at named integration level |
| G5 — Physical pilot | calibrated sensors, safe procedure, site approval and measured operation | pilot measured for stated duration and envelope |
| G6 — Replicated operation | independent replication, full lifecycle and operating evidence | operating performance within demonstrated scope |

Never use "CI pending", "runner queued" or "branch exists" as evidence that any gate passed.

## 7. Cross-project next milestones

1. **Sol-Atlas:** source-qualified resource/site profile (#662) -> portfolio comparison (#293) -> producer connection evidence (#519) -> exact dispatch/network cases (#308/#332) -> uncertainty-aware operation (#312/#352).
2. **Symthaea:** candidate architecture generation (#8496) and deterministic grid scheduler qualification (#7248), with battery trajectory comparison against SAM (#7305).
3. **Symtropy:** multi-period thermal and physical twins (#1916/#2683), mine thermal storage (#4070), mine/brownfield reuse (#1951) and Energy Systems Studio contract (#2721).
4. **Mycelix:** provenance, data rights, ownership and approvals; a valid digest or signed receipt establishes lineage/identity, not physical truth.

## 8. Final rule

Optimize **useful, reliable service delivered over the entire asset lifecycle**—not nameplate MW, a single conversion efficiency, one average LCOE, or a novelty score. Keep cost, service, emissions, water, materials, land, reliability, equity, uncertainty and evidence quality visible as separate decision dimensions.
