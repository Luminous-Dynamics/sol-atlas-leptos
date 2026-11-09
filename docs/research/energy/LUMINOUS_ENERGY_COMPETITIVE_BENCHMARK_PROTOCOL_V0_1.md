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


## 9. Reproducible comparator version lock (checked 2026-11-09)

The exact machine-readable lock is `ENERGY_COMPETITOR_VERSION_LOCK_V0_1.json`. Version strings below are not enough on their own: each run must also pin package manifests, solver, inputs, model configuration and source snapshots.

| Comparator | Version to pin | Use in this programme | Current caution |
|---|---|---|---|
| NLR SAM | 2026.7.3 / SSC 308 | PV, PV-battery, CSP, geothermal and selected storage plant/component comparisons | Not a standalone battery CLI model; mapped battery channels and plant-loss boundaries must be explicit |
| REopt | API v3 stable / exact REopt.jl package and commit | Distributed DER sizing, facility/campus/community microgrid economics and resilience | The API is stable v3; pin the underlying REopt.jl package and solver for each run |
| PyPSA | v1.4.0 (2026-10-09) | Multi-carrier chronological optimization, unit commitment and network-constrained linear planning | Release notes identify behavior-changing fixes to rolling-horizon unit commitment and security-constrained optimization; use the current version, not a stale v1.2-era baseline |
| GenX.jl | v0.4.5 published release for reproducible comparator | Long-term capacity expansion / operations | The release list showed v0.4.5 while the repository README referred to v0.4.6; treat main/unreleased code as a separate comparator |
| NLR ATB | 2025 edition, latest published edition confirmed in this check | Technology cost/performance scenario assumptions | U.S. reference costs are not South African project prices; translate currency year, finance, logistics, taxes, grid and local supply assumptions explicitly |
| PGLib-OPF | v23.07 | Network OPF benchmark corpus | Primarily AC-OPF curated; DC-OPF is a separately declared formulation |
| IEC PV monitoring | IEC 61724-1:2021 Ed. 2.0 | PV sensor and performance-monitoring methodology | Edition 3.0 remains under development with forecast publication in 2027; do not claim draft compliance |
| IEC wind performance | IEC 61400-12-1:2022 Ed. 3.0 + COR1:2025 | Turbine power-curve and uncertainty measurement | Generic atlas curves are not a measured turbine performance test |
| SAWEM | Market Code Rev 2.3 (2026-02) | South African wholesale trading and settlement assumptions | A published code does not prove that a particular market participant, licence or project is operationally admitted |

PyPSA's v1.3.0 release added piecewise-linear marginal/capital-cost curves and piecewise efficiency/rate support, plus maintenance scheduling for Generator, Link and Process components. PyPSA v1.4.0 (2026-10-09) is the pinned current comparison and lists corrections affecting rolling-horizon unit commitment and security-constrained optimization. A benchmark must freeze the exact release so these changes are not mistaken for Luminous improvements.

NLR SAM's current downloads page identifies 2026.7.3 / SSC 308. Its battery-life documentation distinguishes cycle degradation from calendar degradation and documents chemistry-specific model options. Symthaea's simplified cycle-fade model must not claim equivalence to the detailed SAM ageing models without a parameter mapping and trajectory comparison.

GenX's release-list / repository-README discrepancy is material: freeze the published v0.4.5 tag for a release-level baseline. If v0.4.6 or main is used, record its exact immutable commit and label it unreleased/development.

## 10. South African resource input lock

Do not benchmark an optimizer on synthetic-perfect South African resource profiles and then describe the result as regional operational superiority.

- GWA 4.0 is a 250 m microscale modeled wind-climate screening layer; it is not an observed time series.
- The WASA 30-year portal download is currently flagged unavailable in its 2026-09-01 notice pending regeneration. Do not claim to use it until the exact source files have been restored and hashed.
- The CSIR Agulhas-current field reference is a 2017 two-site study; it is not a coast-wide contemporary dataset.
- GSA South Africa country/region layers (last updated 2019, data through 2018) and 2026 global GIS layers (data through 2025) are different inputs and must not be merged under one dataset identity.
- Copernicus wave reanalysis and analysis/forecast products have different spatial and temporal characteristics. Keep reanalysis, near-real-time analysis and forecast horizons separate.
- The South African government's 2026-10-07 statement described a **proposed** first section 34 determination prioritizing 4,600 MW of BESS and 5,000 MW of gas-to-power. It said the first determination would make no provision for new wind or solar capacity, with a subsequent determination to address wind and solar including hybrid projects with storage and longer-term pumped storage. This is procurement sequencing, not a conclusion that wind/solar are technically unsuitable. It explicitly treats transmission, grid connection, fuel availability, delivered gas prices, supporting port/pipeline infrastructure and commissioning schedules as linked constraints. Do not model proposed capacity as already procured or connected.
- The 2026 GASIPPPP Bid Window 1 announcement reported four bids offering 2,830 MW against the 2,000 MW procurement target and noted that South Africa imports most gas requirements from Mozambique and has no LNG receiving terminal. Any gas-to-power scenario must therefore model actual fuel logistics and delivery risk rather than treating nameplate gas capacity as unconditionally firm.

These are input and policy context constraints, not statements that the cited projects, procurements or regional assets are built or operational.


## 11. Global renewable cost anchor — IRENA 2025 cohort

IRENA's *Renewable Power Generation Costs in 2025* (published July 2026) reports the following global weighted-average plant-level LCOE for projects commissioned in 2025. Values are in **2025 USD/MWh**.

| Technology | Global weighted-average LCOE |
|---|---:|
| Onshore wind | 33 |
| Solar PV | 44 |
| Hydropower | 62 |
| Offshore wind | 78 |
| Geothermal | 89 |
| Concentrating solar power | 115 |

IRENA reports that more than 90% of utility-scale renewable projects commissioned in 2025 delivered power below the cost of the cheapest new fossil-fuel plant built in their market. These are global cohort statistics, not South African project bids or guaranteed future costs.

**Critical comparison limitation:** IRENA explicitly cautions that conventional LCOE assumes generation at maximum load factor, condenses expenditure over time, and reports plant-level cost while excluding expenditures beyond the plant boundary such as transmission and distribution. It does not, by itself, capture the economic value of dispatchability or system services.

Therefore:

- retain LCOE as a plant-level technology cost metric;
- calculate delivered-service cost separately after storage losses, curtailment, transmission/distribution, reserve, balancing and grid constraints;
- compare firm capacity, effective load-carrying capability, outage service, location, ramping and flexibility independently;
- report storage as a separate asset/cost and avoid allocating its energy twice;
- use South African financing, currency year, taxes, labour, logistics, interconnection and resource evidence for any local estimate;
- do not claim a novel system beats solar/wind on LCOE unless technology, boundary, commissioning cohort, financing and system service are genuinely matched.

A useful Luminous result may be a portfolio with a higher plant-gate LCOE but lower total cost per unit of essential service delivered, lower outage risk or better industrial heat integration. That conclusion must be demonstrated with paired models and independently checked service receipts—not inferred from the technology labels.


## 12. Wind-model SOTA comparator: ETHOS.RESKit and the South African blind-holdout problem
The 2026-01-14 Nature Communications paper *Towards high resolution, validated and open global wind power assessments* (Peña-Sánchez et al.) is an important new reference for wind-power simulation.

The paper describes ETHOS.RESKit as a time-resolved GWA4 + ERA5 wind-power workflow and reports:
- Calibration using 213 global weather-mast locations in 25 countries and more than 8 million filtered hourly observations. The paper states that the mast-measurement geography includes South Africa.
- A separate turbine/farm power-validation set with more than 8 million hourly observations from 152 turbine/farm sites. The raw turbine power records are restricted by third-party agreements.
- On its reported hourly turbine/farm capacity-factor comparison (Table 2): relative mean error 0.056, Pearson correlation 0.844, RMSE 0.196 and Perkins skill score 0.90 for the calibrated workflow; the uncalibrated relative mean error is 0.292.
- A separate global/country comparison using IEA statistics. The abstract mentions a 0.6% mean error for annual aggregated energy output, while the detailed main text reports a 6.2% relative deviation between global average capacity factor 0.287 (model) and 0.306 (IEA). These are different cohorts/metrics; benchmark manifests must preserve the precise denominator, formula, temporal aggregation and source data rather than compressing them into one “accuracy” figure.
- More than 800 / 880 turbine model types and support for synthetic power curves. Pin the exact model release/archive before claiming the current turbine library size.

The authors report South African mast measurements in the global calibration geography, but do not establish South African turbine-generation holdout validation. They also warn that the workflow is more appropriate for large-scale assessments than exact single locations; ERA5/GWA4 spatial representation, local terrain, diurnal and seasonal biases remain limitations. The paper's raw wind-farm database is commercially licensed and the turbine power records are restricted.

### Pinning

- Paper-specific reproduction: pin Zenodo archive DOI **10.5281/zenodo.17668776** and all archived files/checksums.
- Current software comparator: pin immutable GitHub tag **RESKit v0.5.0**, released 2026-07-10, commit `b41cf859fd619c5be717af84db5502b0ac4ca3e5`, plus exact Python environment and input data. This release is not assumed identical to the paper archive.
- Also compare baseline workflows such as Renewables.ninja, pyGRETA and atlite only with identical ERA5/GWA4 periods, turbine types, hub height, power curves, wake/availability losses, site set and evaluation metrics.

### South African evaluation rule

Do not blindly reuse a WASA mast for validation if it contributed to the ETHOS.RESKit calibration cohort. First recover the exact 213-mast source table/site IDs and match them to WASA mast identifiers, including instrument height, period, quality flags and data overlaps. Partition by site and time—not by random hourly rows—so correlated samples from one mast cannot leak across training and blind holdout.

Where South African masts overlap the calibration set, reserve other genuinely independent SAWS/CSIR/lidar campaigns, new measurement periods or turbine/farm telemetry as the blind set. If no independent turbine power series can be lawfully accessed, the result must remain `NEEDS_INDEPENDENT_POWER_DATA`; no local turbine-AEP superiority claim is permitted.

### Luminous research opportunity

A fair “better than SOTA” study should test four distinct outputs:
1. Resource field: GWA4 / WASA3 / ERA5 against independent wind-speed observations by height, season and terrain regime.
2. Turbine conversion: manufacturer curve and air-density correction, independently measured power curve where available.
3. Chronological generation: site/time-blocked capacity-factor and power-trajectory error, calibration, extreme/lull tail behavior, and uncertainty interval coverage.
4. Portfolio decision: delivered service, curtailment, storage sizing, grid constraints and lifecycle cost after using the resource model.

Do not fit a single global correction factor and claim universal local validation. Test a hierarchy of global, climate-region, terrain/regime and site-specific corrections with leakage-safe ablation, uncertainty propagation and minimum data sufficiency. A local correction is only eligible when independent data justify it and the calibration/holdout split is disclosed.


## 13. Hydropower benchmark lane: net energy recovery under protected water service
### Local reference ladder

| Reference | Appropriate use | Limits |
|---|---|---|
| WRC South African Hydropower Atlas TT 916/23 (2023) + SAHA E-Tool | Existing South African asset/resource discovery by hydro archetype | Preliminary criteria, heterogenous source layers, incomplete flow/head data, WWTW/WTW assumptions and pumped-storage overlap |
| DWS Water/Sanitation Climate Response Strategy (Jan 2025) | Confirms PAT and PRV energy-recovery as an explored South African option | Sector context, not a site-level asset register or measured performance data |
| 2026 WDN hydropower review | Compare conventional turbines, PAT, in-pipe harvesters, hydraulic/electric regulation and low-power telemetry use | Literature review; device/site measurements and manufacturer curves remain required |
| IEC 2026 pumped-storage trend report | Technology and standardization trends for storage | Trend report, not a project or component acceptance standard |
| NLR ATB hydropower / pumped storage | Reference cost/performance assumptions in the declared U.S. scenario | Not South African project costs; translate local works, water, finance, grid and environmental conditions |

### Match the engineering boundary

For conduit/PRS, compare PRV-only baseline versus turbine/PAT bypass using the exact same time-indexed upstream pressure, downstream minimum pressure, flow and water demand. For WTW/WWTW, compare the identical inlet/outlet hydraulic boundary and preserve treatment and effluent compliance. For rivers/weirs/canals, compare the same seasonal hydrograph and environmental-flow obligations.

Report independently:
- water-service pressure compliance, volume delivered, treatment/effluent performance, availability and restoration after faults;
- gross hydraulic power, turbine/generator efficiency map, auxiliary electricity, net exported/self-consumed kWh, and recovered kWh per cubic metre;
- flow/pressure variability, part-load efficiency, bypass duty, starts/stops, surge, cavitation, debris, sediment, corrosion, fouling, maintenance availability and spare-parts lead time;
- environmental flow, fish passage/ecological impact, water quality and downstream service;
- CAPEX/OPEX, civil works, interconnection, outage costs, lifecycle replacement and cost of useful delivered service.

### Reference physics

For a conduit or conventional head-driven turbine, the screening identity is (P=\rho g Q H_{net}\eta_{hyd}\eta_{gen}), where (Q) is measured flow and (H_{net}) includes conveyance losses. For hydrokinetic devices, kinetic resource is related to (\frac{1}{2}\rho A V^3) times a device-specific capture coefficient/efficiency; do not substitute static head or use mean current velocity without its time profile and swept area. Neither equation replaces a validated transient hydraulic model, turbine curve, or operational pressure check.

For a pumped-storage case, keep (E_{pump}\), (E_{generate}\), usable storage, hydraulic losses, electrical losses, round-trip efficiency and environmental/water constraints distinct. It is not new primary generation.

### Targeted benchmark controls

- Analytic power identity and dimensional/unit fixtures for head, pressure, flow, density and net output.
- Efficiency must not exceed the declared physical/equipment map or Carnot (when heat conversion is involved); no output at zero flow/head/velocity.
- Turbine/PAT off-design flow and head profiles must use the appropriate efficiency surface, not peak efficiency.
- Water pressure/flow service floors and environmental-flow constraints are hard feasibility gates, not soft penalties.
- Seeded water hammer, PRV/bypass failure, turbine trip, sensor failure, blockage, grid outage and unsafe restart must be detected or return indeterminate.
- Pumped-storage pairs must reject overlapping reservoir double counts, missing head/active volume, incomplete geology/hydrology, or a new reservoir without rights/environmental evidence.
- Compare no-build, PRV-only, PAT and purpose-built turbine under identical time series, financing, component lifetime, parasitic losses and service requirements.

Do not call any potential total a measured capacity. The WRC 2023 atlas explicitly states that its initial WWTW/WTW criteria relied on assumptions, hydrokinetic data were limited, and its 31,000 MW pumped-storage potential is overstated by overlapping candidate sites and unverified geology/hydrology/environment. Luminous should compete by improving source quality and decision confidence, not inflating the resource total.


### Hydraulic-machine standard references and water-use authorization

For the South African conduit-hydro vertical slice, benchmark to the published standard edition that matches the unit size and test boundary:

| Standard | Use | Boundary |
|---|---|---|
| IEC 60041:1991 | Field hydraulic-performance acceptance for turbines, pumps and pump-turbines | Field acceptance / contractual guarantees |
| IEC 60193:2019 | Model acceptance tests | Primarily >5 MW or >3 m reference diameter; smaller by agreement |
| IEC 61362:2024 | Governing-system requirements and performance specification | Not an acceptance test or water-service safety case |
| IEC 60545:2021 | Commissioning and operation | Operational handover and commissioning guidelines |
| IEC 62006:2010 | Small-hydro acceptance tests; safety, trial operation, reliability, cavitation/noise/vibration and performance classes | Published first edition; Edition 2 remains a development project as of 9 November 2026 |

Treat these standards as references for test plans, not proof that a candidate is compliant. A simulation can reproduce the equations but cannot issue a field or model acceptance certificate.

South African legal gate: The National Water Act 36 of 1998 and DWS Water Use Authorization Process govern water-use authorization. The project must identify whether its activity falls under a licence, general authorization, existing lawful use or other applicable route, and preserve the Reserve / ecological and basic-human-needs constraints. Water-use authorization and environmental approvals are independent of grid connection and turbine performance.

Sources:
- National Water Act: https://www.gov.za/documents/national-water-act
- DWS authorization process: https://www.dws.gov.za/ewulaas/WUA.aspx
- IEC standards and current editions: links in `ENERGY_COMPETITOR_VERSION_LOCK_V0_1.json`

The programme's hydro scorecard must keep water service, treatment/effluent quality, environmental flow, rights/authorization, hydraulic performance, net energy and lifecycle economics as separate results and hard gates.


## 14. Water-distribution network PAT benchmark: EPANET + service-constrained co-optimization

### Immutable engine pins
- Contemporary network engine: OWA-EPANET **v2.3.5**, commit `444725fbf16ab28b0470ef0ed281dc4849d49583`, released 2026-02-20.
- Historical paper replay: OWA-EPANET **v2.2**, commit `4d8d82ddc260fce216af9321fc3d9a4646ac6827`.
- EPANET 2.3 adds FAVAD pressure-dependent leakage, PCV, independent pressure units and Flow Balance reporting. Version 2.3.5 fixes flow-unit conversion for Generic curves and mixed-unit behavior.
- Keep any Python orchestration package such as EPyT separately pinned; it is an interface, not the hydraulic solver. Production implementation should use the pinned OWA toolkit through an explicit Rust/C boundary where practical.

### Matched comparisons
1. Fixed PRV baseline.
2. Static PAT at a declared BEP point.
3. PAT selected using maximum, average and minimum discharge.
4. 2021 UKZN PRV/PAT placement/setting optimizer, only after exact source model recovery.
5. 2022 Cornubia five-PAT configuration reproduced using EPANET 2.2 and the study's original PRV-proxy/post-processing assumptions.
6. 2025 Jowitt WDN study, preserving its DDA vs PDA and one-to-five PAT scenario matrix.
7. 2026 Kalman filter + tracking MPC + actuator-settings-optimizer baseline.
8. Proposed Luminous robust/service-constrained MPC candidate with coordinated PRV/PAT control and leakage-aware pressure management.

### Co-simulation boundary
- EPANET owns multi-node network hydraulics, demand, pressure, leakage, valve and flow-balance calculations.
- A PAT-specific, evidence-backed hydraulic characteristic must reproduce the actual equipment pressure-drop/flow relationship. A generic head-loss curve/GPV representation is only valid after model- or field-test calibration and must not be described as native PAT physics.
- Symtropy #4386 owns the matched component conversion from net head/flow through turbine/PAT efficiency, generator and auxiliaries to net electrical output.
- A coupling iteration reconciles network pressure/flow with PAT settings. Non-convergence, map-domain violations or missing evidence return infeasible/indeterminate, never an extrapolated kW result.
- Pressure-dependent leakage is part of the water balance. Water saved by pressure control and electricity recovered by PATs are separate outcomes.
- Water hammer, surge, cavitation and protection remain specialist validation boundaries; steady EPANET runs do not certify them.

### Primary metrics
- Critical-node pressure: minimum margin, node-hours below minimum, worst pressure deficit and recovery time.
- Delivered service: demanded vs delivered volume, pressure-driven demand deficit and service availability.
- Leakage: kL/day and kL/month, pressure/leakage calibration residual, water-balance closure and uncertainty.
- Energy: hydraulic input, shaft output, gross electrical, auxiliaries, net AC/DC, net kWh per m³ through the PAT, map coverage and export/self-consumption.
- Controller: pressure tracking error, constraints violated, actuator movement/switching, starts/stops, chattering and fallback latency.
- Robustness: held-out demand chronology, sensor noise/dropout, pressure/flow bias, pipe-roughness uncertainty, leakage-law mismatch, reservoir level and PAT-map drift.
- Economics: energy value, avoided water-loss cost, lifecycle replacement/maintenance, outage and civil-work cost.
- Computation: worst-case solve time, wall-clock/update, memory, deterministic replay and abstention rate.

### Non-compensable gates
Zero hard service-floor violations on the frozen synthetic negative-control corpus; no unexplained flow-balance residual; no PAT branch output outside its equipment map; no leakage/flow counted twice; no controller win achieved by reducing delivered demand; no use of PRV proxy as measured PAT evidence; no credited generation while bypass-only or without available head/flow.

### Reproduction before superiority claims
Reproduce the 2022 Cornubia 2.2 model outputs on exact data (166.31 kW and 45.59 kL/month under its declared assumptions). Then reproduce the 2025 DDA/PDA Jowitt matrix and the 2026 multi-node RTC baseline where full inputs are obtainable. Treat missing network files or supplementary data as `NOT_REPRODUCIBLE_FROM_PUBLIC_ARTIFACT`. The 2021 and 2022 Cornubia studies share geography and are not independent replications until their input networks/demand data are crosswalked.

The candidate is **not “better than SOTA”** until it wins matched held-out network/time cases against these pinned baselines under equal sensor access, model calibration budget, computation budget and equipment evidence. Publish matched wins, losses and indeterminate cases.


## 15. Rust-native EPANET differential qualification

Two Rust integration paths are worth testing, but their roles must remain distinct:

- Safe wrapper `epanet 0.3.0` wraps the official OWA EPANET C API. It is an integration boundary for the reference engine, not an independent solver. The project handle is `Send` but not `Sync`; concurrent access to a single project requires synchronization. The lower-level `epanet-sys 2.3.0` crate builds OWA C through CMake/bindgen and requires a C compiler and libclang.
- `epanet-rs 0.2.3` is a Rust-native reimplementation. The published feature checklist marks INP, PDA, controls, emitters, and partial API compatibility as implemented; leakage, rules, energy and water-quality support are unchecked. Its maintainers claim up to 5× parallel extended-period speedup only for supported networks without tanks or controls. Treat that as an upstream claim to reproduce, not our measured result.

### Solver-routing policy

1. **OWA EPANET 2.3.5 remains the reference oracle** for benchmark acceptance, FAVAD pipe leakage, full feature support and any case whose behavior is not explicitly qualified on the Rust implementation.
2. Use the safe `epanet` wrapper if it simplifies Rust integration with the official OWA API; still pin the native OWA commit, build flags and exact library bytes.
3. Use `epanet-rs` only for feature subsets where differential tests establish parity. Initially target canonical no-tank/no-control networks, then sequential PDA / control cases if the exact behavior matches.
4. Do not route FAVAD/pipe-leakage cases to `epanet-rs` until its missing leakage capability is implemented upstream or an independently qualified adapter reproduces OWA link leakage and total water balance.
5. When a feature is unsupported, the case returns an explicit fallback-to-OWA or `UNSUPPORTED_BY_RUST_SOLVER` disposition; it never silently drops the feature.

### Benchmark gates

- Compare all node heads/pressures, link flows/headloss, delivered demands, deficit, leakage where applicable, control states, tanks and time chronology.
- Compare convergence state and mass/flow balance; a low aggregate RMS error cannot conceal one failed critical node.
- Reproduce upstream speedup only on the stated no-tanks/no-controls class and then quantify actual speedup on PAT/WDN cases. Do not extrapolate the 5× claim to FAVAD, controls, tank or PAT workloads.
- The Rust solver is a candidate scenario-ensemble accelerator, not a production replacement. A feature-specific parity receipt and deterministic run identity are prerequisites for every accelerated result.

External pins:
- `epanet` safe OWA wrapper 0.3.0: https://docs.rs/epanet/0.3.0/epanet/
- `epanet-sys` raw OWA API 2.3.0: https://docs.rs/epanet-sys/2.3.0/epanet_sys/
- `epanet-rs` native solver 0.2.3: https://docs.rs/crate/epanet-rs/0.2.3
- Upstream source revision reviewed 2026-11-09: https://github.com/Vitens/epanet-rs/tree/fee0d03b11ed71a6da7366f07290d6e44556add4

This is separately tracked in Symtropy [EPANET-RS-DIFF-001 #4398](https://github.com/Luminous-Dynamics/symtropy/issues/4398).

## 14. Stronger PAT benchmark ladder: component truth → full-scale operation → network and controller

A fair current comparator set must separate four evidence levels.

| Evidence level | Comparator | What it actually establishes | What it does not establish |
|---|---|---|---|
| Laboratory component map | Pugliese et al. 2016, two PAT test-rig characterizations | Experimental reverse-mode curves for two specific pump models | General pump-to-turbine conversion, WDN field performance |
| Laboratory control experiment | Parra et al. 2018 PAT/PRV pressure modulation | Variable-discharge critical-pressure tracking; up to 2.3 kW full-load, 40% maximum net system efficiency, up to 16% water savings in a German test case | South African field transfer or long-term reliability |
| Full-scale WDN operation | Muhammetoglu et al. 2018 Antalya PAT/PRV bypass | Approximately five months online flow/power/pressure monitoring at 130–300 m³/h; 0.7–8.4 kWh reported energy values for ~100 kPa reduction; 60% average efficiency | kW/annual energy from abstract (interval missing), multi-year availability, South African economics |
| Experimental fleet + modelled WDN | Manservigi et al. 2025, 16 experimental turbomachine curves in a 5,000-user Italian WDN | Matched hydraulic/electrical regulation comparison; 44.1% of available hydraulic energy recovered for selected candidate; case-specific LCOE/cost | 16 field installations or South African performance |
| South African modelled placement | Cornubia 2022 / UKZN 2021 | Local network topology and candidate-placement research | Measured PAT generation or independent replication |
| Modern simulated network control | Jowitt 2025; Galuppini et al. 2026 | DDA/PDA sensitivity and multi-node Kalman + MPC control architecture | Field-validated South African OT controller |

### Target metrics by evidence level

**Component map**
- Head, flow, torque/speed where available, hydraulic efficiency, generator/inverter losses and uncertainty.
- Curve prediction residual across the observed domain; no extrapolation to unmeasured head/flow.
- Best-efficiency-point distance and off-design efficiency distribution.
- Repeatability, calibration uncertainty and model-to-lab residual.

**Full-scale operation**
- Net AC/DC energy with explicit interval boundaries and auxiliaries.
- Flow, upstream/downstream pressure and bypass chronology.
- Service pressure compliance, pressure cycling, trip/fallback behaviour, availability, maintenance and meter uncertainty.
- Energy and water-balance residuals; reported kWh without interval duration cannot be normalized to kW.

**Network/control**
- Node-hours below minimum pressure, worst pressure deficit, delivered-demand deficit, leakage volume and water balance.
- PAT net kWh, hydraulic-energy recovery ratio, bypass fraction and off-design hours.
- Controller settling/overshoot, actuator travel, starts/stops, chattering, fail-safe latency, and constraint violations.
- Sensitivity to DDA versus PDA, FAVAD coefficients, demand forecast error, telemetry dropout, PAT map uncertainty and reservoir level.
- Matched LCOE / lifetime cost with water-service value, replacement, installation, O&M and tariff assumptions.

### Concrete Luminous research target

The target is not to beat the 44.1% energy-recovery figure by maximizing extraction. It is to beat a matched set of baselines on **net useful value while preserving the water service**. A credible result could have lower gross kWh yet outperform by preserving critical pressures, reducing leakage, avoiding bypass, improving metering uncertainty, decreasing controller wear, or delivering a better cost/service frontier.

Do not compare:
- 2.3 kW full-load output with a daily-energy or installed-capacity value;
- Antalya 0.7–8.4 kWh with kW without its interval;
- PAT turbine hydraulic efficiency with net system efficiency;
- 44.1% of available hydraulic energy with turbine-only efficiency;
- experimental pump curves with full-scale network energy as if they share a physical boundary;
- modelled Cornubia estimates with Antalya metered operation.

Sources and exact comparator pins are recorded in `ENERGY_COMPETITOR_VERSION_LOCK_V0_1.json`.
