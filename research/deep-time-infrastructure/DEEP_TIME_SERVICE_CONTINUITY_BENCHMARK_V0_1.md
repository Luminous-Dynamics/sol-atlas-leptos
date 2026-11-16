# Deep-Time Service Continuity Benchmark
## Protocol v0.1

**Status:** research protocol and 40-case specification; numeric corpus, independent executable oracle, and physical validation are pending.  
**Parent:** [DEEP-TIME-INFRASTRUCTURE-001 #886](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/886)  
**Composes:** Port-City Foundry #370/#372/#378; existing Symtropy hydrology, lock, sediment and transport owners; Symthaea adaptive-pathway and policy-robustness owners; Mycelix authority, rights and provenance owners.

## 1. Research thesis

The unit of long-term infrastructure design is not the asset. It is the **essential service and the network of options that can continue to provide it**.

A port, lock, bridge, canal, rail line, settlement or power station can be rebuilt, relocated, split or retired. A resilient region preserves access to freight, water, food, energy, healthcare, knowledge, mobility and ecological function while those physical components change.

This programme therefore benchmarks **service continuity under changing Earth systems**, not the ability to keep one fixed structure intact.

The proposed engine is a renderer-neutral *Adaptive Infrastructure Pathway Engine* (AIPE): it takes typed service requirements, a dependency graph, competing Earth-system scenarios, feasible adaptation actions, monitoring signals, rights and ecological constraints, then emits a contingent pathway policy and a reproducible evidence record. It is a decision-support contract, not an autonomous infrastructure authority.

## 2. Paradigm shifts

### 2.1 From asset durability to service continuity

Measure whether essential services survive, not whether a particular port or canal remains in place. Track:
- service availability and delivered capacity;
- cumulative unmet demand and longest interruption;
- time to restore service after disruption;
- number and independence of fallback routes;
- relocation, replacement and reconfiguration cost;
- ecological functions and distributional service floors.

A corridor can remain valuable as a service alignment even when every physical asset on it is replaced.

### 2.2 From prediction to contingent policy

A single forecast asks, "What should we build for year X?" An adaptive policy asks, "What do we do now, what do we monitor, and what action becomes necessary when an observable signal crosses a threshold?"

Represent a pathway as a directed policy graph:
- decision nodes consume only observations available at that time;
- action nodes carry duration, cost, capacity effect and prerequisites;
- monitoring nodes carry sampling cadence, detection delay and data quality;
- threshold nodes identify service limits that must not be crossed;
- branch conditions identify the observation that selects the next action.

Every trigger must include lead time. A trigger that fires after the adaptation action can no longer finish before the service threshold is not a successful adaptation strategy.

This composes established Dynamic Adaptive Policy Pathways (DAPP) practice rather than claiming to invent it. The 2024 New Zealand coastal guidance describes objectives, thresholds, alternative pathways, scenario stress tests, signals and triggers, and monitoring with adequate lead time. Its October 2026 update should be pinned in the benchmark bibliography. See [Ministry for the Environment guidance](https://environment.govt.nz/publications/coastal-hazards-and-climate-change-guidance/).

### 2.3 From a global map to source-qualified spatial evidence

Do not confuse grid spacing with local accuracy or engineering suitability.

GEBCO_2026 provides a global land/ocean terrain grid at 15 arc-second intervals and a Type Identifier Grid (TID). Its documentation says the grid combines sources of varying quality and coverage, includes predicted as well as measured bathymetry, and can contain shallow-water vertical-datum differences. GEBCO explicitly says the grid is not for navigation or other safety-at-sea use. Therefore:
- preserve the TID class and original-source lineage;
- keep horizontal CRS, vertical datum, epoch and transformations explicit;
- propagate vertical uncertainty into shoreline and route decisions;
- fail closed when datum conversion or source support is missing;
- treat global data as screening evidence, never as a substitute for local hydrographic survey.

Sources: [GEBCO_2026 Grid](https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid), [GEBCO grid development and TID](https://www.gebco.net/data-products/gridded-bathymetry-data/grid-development).

### 2.4 From engineering-only to coupled nature, sediment and water systems

Ports and inland waterways operate inside sediment budgets, watersheds, wetlands, estuaries and habitat networks. Nature-based and hybrid solutions must be first-class alternatives—not an afterthought after concrete designs have already won.

The 2025 World Bank / EcoShape ports guidance is a relevant new comparator. It presents nature-based solution families, enablers, business-case methods and cases from multiple regions. We should compare green, grey and hybrid alternatives with equal service requirements and explicit lifecycle, ecological and community outcomes. It is not evidence that a generic nature-based intervention will work at every site.

Sources: [EcoShape / World Bank guide, 9 July 2025](https://www.ecoshape.org/en/nature-based-solutions-for-ports-a-practical-guide-for-a-resilient-future/), [World Bank publication](https://ppp.worldbank.org/library/nature-based-solutions-ports-overview-nbs-implementation-practice-opportunities-and-challenges).

### 2.5 From weighted scores to constraints plus a Pareto set

Do not allow cheap construction, high throughput or economic return to compensate for:
- insufficient environmental flows;
- missing water rights or competent authority;
- unsafe geotechnical or navigation conditions;
- unqualified source data or model domains;
- severe habitat fragmentation or unmitigated ecological damage;
- unacceptable essential-service loss for an affected group;
- absent consent or legally required process.

Apply hard gates first. Only eligible alternatives enter multi-objective comparison. Preserve the Pareto set and explain trade-offs; never collapse non-compensable constraints into an arbitrary weighted sum.

### 2.6 From one solver to an evidence-qualified federation

No general-purpose engine should pretend to replace every specialist solver. Keep the model boundary explicit:
- Sol-Atlas owns source-qualified geography, alternative comparison and decision presentation.
- Symtropy owns physical context, hydrology, sediment, channel/lock operations and qualified model adapters.
- Symthaea generates candidate designs and adaptive policies, then tests them against frozen constraints and held-out scenarios.
- Mycelix carries authority, rights, consent, provenance and approval receipts.

A model output is admissible only for its declared context of use, calibration domain, data resolution, boundary conditions and validation evidence. TELEMAC publishes a validation manual and benchmark cases; use relevant analytical and laboratory cases as independent comparators, not as evidence that our own adapter is validated. See [TELEMAC-3D product and validation documentation](https://opentelemac.org/en/product/telemac3d).

### 2.7 Treat deep-time model disagreement as a benchmark signal

The current Late Cretaceous sea-level debate is a useful first adversarial model-family benchmark because its published reconstructions are methodologically different:

- **Paleogeographic flooding method (2022):** estimates a peak near 280 m above present around 80 Ma by measuring modern-land flooding in paleogeographic reconstructions and applying a modern global hypsometric slope. The paper explicitly discusses the sensitivity of the result to hypsometric slope and paleogeographic assumptions. [Paper](https://doi.org/10.1016/j.gr.2022.05.011).
- **Strontium-isotope / tectono-glacio-eustatic method (2022):** combines plate-tectonic and glaciation estimates, reporting Cretaceous highstands around 174–217 m and a Phanerozoic combined amplitude around 220 m. Its assumptions include ocean-floor-depth and continental-shelf-hypsometry treatments. [Paper](https://doi.org/10.1016/j.gr.2022.07.014).
- **Sequence-stratigraphic / basin synthesis (June 2026):** Xu and Jin compile 66 datasets, including 26 published curves and 40 stratigraphic or seismic sections across 38 continental-margin basins/regions. They propose a different Cenozoic history in which present global mean sea level is only about 40–55 m below the Late Cretaceous peak. This is a competing synthesis, not an established replacement. [Paper](https://doi.org/10.1016/j.earscirev.2026.105470).

Do not average the reported peaks or rank a reconstruction from one headline number. Preserve each method, input lineage, age model, reference frame, hypsometry, plate reconstruction, basin coverage, uncertainty and known limitations. Treat shared input curves or datasets as correlated evidence unless independence is demonstrated; a collection of 66 datasets is not automatically 66 independent confirmations.

The benchmark should test:
- whether the implementation preserves each reconstruction family without collapsing the input evidence;
- sensitivity to alternative paleogeographic grids, age assignments, basin coverage, dynamic topography and hypsometric assumptions;
- out-of-sample agreement with withheld stratigraphic/seismic sections where source data and permissions allow;
- uncertainty calibration and residual structure, not only peak-height error;
- whether a service-network decision remains acceptable across competing histories even when the inferred ancient shoreline differs materially.

This is a model-disagreement experiment at geological timescales. It cannot be translated into a present-day site flood map or an anthropogenic sea-level forecast.

## 3. Contract-level architecture

Minimum renderer-neutral study shape:

1. **Service requirements:** service IDs, essentiality, demand time series, service floor by population/group, continuity threshold and acceptable restoration time.
2. **Asset/dependency graph:** nodes and edges with capacities, dependencies, failure modes, common-cause groups, repair durations and relocation/replacement options.
3. **Scenario ensemble:** scenario-family ID, physical mechanism, time regime, boundary conditions, source lineage, uncertainty and probability status. No probability distribution is inferred from a list of scenarios.
4. **Alternative portfolio:** no-build, operational improvement, asset hardening, nature-based/hybrid intervention, route diversification, inland terminal, rail-first/dry-port, waterway, relocation and staged combinations where relevant.
5. **Hard constraints:** water rights, environmental flows, donor-basin limits, ecology, source quality, model context of use, consent, authority and group-level service floors.
6. **Actions and monitoring:** cost, duration, capacity, prerequisites, trigger, signal cadence, observation delay and action lead time.
7. **Outcome ledger:** service delivered, outage, unmet demand, recovery, water, energy, sediment/ecology, lifecycle cost, distribution and stranded assets.
8. **Evidence receipt:** immutable input digests, source versions, datum transforms, solver and configuration pins, scenario IDs, algorithm, random seed, run budget, verifier result and claim ceiling.

Every decision must be replayable from the evidence receipt. Every output must disclose what it did not model.

## 4. Forty-case qualification programme

The machine-readable companion register defines 40 initial test specifications across five groups:

| Group | Cases | Qualification objective |
|---|---:|---|
| Scenario semantics | 8 | Prevent category, time-horizon and sea-level reference-frame confusion |
| Water ledger | 8 | Exact accounting and hard rights/environmental-flow constraints |
| Service network | 8 | Cut sets, common-cause failures, capacity and recovery |
| Adaptive pathways | 8 | Trigger timing, observation delay, regret and policy validity |
| Evidence, rights and ecology | 8 | Datum/TID quality, uncertainty, legal authority, ecology and distribution |

**Maturity honesty:** these are 40 defined case specifications, not 40 fully frozen executable fixtures. The companion register marks the initial hand-computable water cases separately. The next implementation increment must add complete input values, expected output bytes/digests, an independent verifier and mutation tests before claiming a frozen 40-case corpus.

### 4.1 Exact reference cases

Initial hand-computable references include:
- navigation pool: 1,000 m3 initial storage, 125 m3 inflow, 105 m3 outflow; expected final storage 1,020 m3 and zero accounting residual;
- drought pool: 1,000 m3 initial storage, no inflow, 105 m3 outflow; expected 895 m3, below a 900 m3 minimum;
- environmental flow: 25 m3 requirement versus 20 m3 proposed release; reject;
- donor basin: 25 m3 requested transfer versus 20 m3 allocable surplus; reject;
- leakage: 8 m3 measured against 5 m3 limit; reject;
- lockage: four cycles at 12 m3 and 1.5 kWh each; 48 m3 water and 6 kWh energy;
- graph: a three-node single bridge and a two-path edge-disjoint control; exact cut/path results.

The numeric accounting fixtures are synthetic. They test implementation semantics only; they do not establish a real water supply or canal operating envelope.

## 5. Comparison protocol: fair tests against the field

Compare methods on the same case inputs, action universe, rights, scenario information, cost budget, time horizon and compute allowance.

### 5.1 Decision baselines

Required comparators:
- fixed master plan chosen before scenarios are revealed;
- status quo / no-build and operational-improvement baselines;
- simple greedy intervention selection;
- expert-authored pathway;
- DAPP-style trigger pathway;
- robust min-max policy;
- minimax-regret policy;
- stochastic expected-value policy only when calibrated probabilities exist;
- risk-sensitive/CVaR policy only when the probability and loss model is defensible;
- exhaustive exact oracle on small enumerable cases.

Use NSGA-II, NSGA-III, MOEA/D and SPEA2 for multi-objective search comparisons. These are algorithmic baselines, not a claim that one optimiser is inherently suitable for every decision problem. Report feasibility before Pareto quality, and publish the reference point, hypervolume method, budget and stopping criteria.

### 5.2 Specialist-tool comparators

| Domain | Industry/reference baseline | Our fair comparison |
|---|---|---|
| Coastal screening | NOAA Sea Level Rise Viewer; regional flood/coastal datasets | Source provenance, local datum, uncertainty and downstream infrastructure dependencies |
| Terrain/bathymetry | GEBCO_2026, Copernicus DEM, national LiDAR and hydrographic surveys | Data lineage, TID-aware quality gates, resolution/datum sensitivity |
| Port adaptation | PIANC guidance; World Bank Climate Risk and Ports | Matched service requirements, staged alternatives, whole-network dependencies |
| Nature-based ports | World Bank / EcoShape 2025 guide and cases | Green/grey/hybrid comparison with ecological, social and lifecycle outcomes |
| Coastal/hydraulic physics | Delft3D FM, XBeach, TELEMAC, SCHISM, USACE references | Independent benchmark residuals and context-of-use qualification |
| Deep-time Earth | GPlates, PALEOMAP and published plate/paleogeography reconstructions | Competing model families, time-indexed lineage and uncertainty-aware claim ceiling |
| Adaptive decisions | DAPP, robust decision making and decision-scaling literature | Exact small-case oracle, regret, trigger lead time and held-out scenario robustness |

A feature-count comparison is invalid. Specialist physical solvers should be treated as reference engines within their validated domains, not competitors to a cross-domain decision ledger.

### 5.3 Outcome scorecard

Report a scorecard, not one synthetic "resilience score":

- **Correctness:** exact-oracle agreement, invalid-policy rejection, false-safe and false-eligible rates.
- **Service:** essential-service coverage, cumulative unmet demand, longest outage, delivery capacity and recovery time.
- **Network:** minimum cut, independent gateway paths, common-cause exposure and fallback capacity.
- **Water/energy:** exact ledger residual, consumptive use, lockage, pumping energy and donor-basin depletion.
- **Ecology:** fish passage, sediment continuity, wetland/floodplain connectivity, habitat impact and ecological-flow compliance.
- **Distribution:** minimum service by affected group, displacement, affordability and burden allocation.
- **Lifecycle:** capex, O&M, renewal, relocation, decommissioning, stranded assets and uncertainty range.
- **Adaptability:** time-to-trigger, action lead time, reversible investment, pathway switch cost and option preservation.
- **Evidence:** claim-to-source traceability, datum/model validity and reproducible receipts.
- **Efficiency:** wall-clock p50/p95, memory, model evaluations, compute and energy where measured.

### 5.4 Proposed acceptance targets

These are hypotheses to test—not achieved results.

| Measure | Initial target |
|---|---:|
| Wrong scenario-family promotion | 0 |
| False-safe / false-eligible on frozen hard-gate negatives | 0 |
| Exact water-ledger accounting residual on hand fixtures | 0 m3 |
| Independent oracle agreement on exhaustive small cases | 100% |
| Deterministic replay for pinned synthetic cases | 100% |
| Unsupported branches or future-information leakage | 0 |
| Missing source/datum/rights treated as known-safe | 0 |
| Essential-service floor violations hidden by aggregate score | 0 |
| Source references for material consumed claims | 100% |
| Benchmark run completeness | All assigned cases retained, including failed/invalid/timeout cases |

For larger multi-objective problems, report hypervolume, epsilon indicator, feasible fraction, regret to known bounds, runtime and variance across seeds. Exact frontier agreement is required only where exhaustive enumeration is feasible.

## 6. How to make this genuinely better than current practice

The plausible differentiator is not a new coastal solver or a new weighted score. It is the **closed evidence-to-policy loop**:

1. Compile a service need and authority constraints into a typed study.
2. Generate genuinely different infrastructure portfolios, including no-build and nature-based alternatives.
3. Reject invalid data, rights, physical-domain and ecological assumptions before ranking.
4. Simulate only through appropriately qualified specialist models.
5. Generate an adaptive pathway with signals, triggers, observation delay and action lead time.
6. Attack the pathway with held-out and adversarial scenarios, including correlated failures.
7. Independently verify its exact small-case semantics and deterministic replay.
8. Issue a bounded evidence receipt showing why the policy is admissible, which futures it survives, which ones break it, and what should be monitored.

That produces an auditable decision system. It does not promise that an optimizer can find a universally best canal, city or port.

## 7. Implementation order

**Increment A — freeze exact semantics**
- Convert WAT-001 through WAT-007 into complete numeric JSON fixtures.
- Add independent standard-library water-ledger oracle; do not call production validator/scorer.
- Add graph cut/path known-answer fixtures and independent verifier.
- Add mutation tests for sign, units, duplicate IDs, missing releases and common-mode dependencies.

**Increment B — pathway policy oracle**
- Define a finite-state policy schema with strictly time-causal observations.
- Enumerate all feasible policies for small fixtures.
- Compare exact optimum against fixed, greedy, min-max and minimax-regret baselines.
- Return explicit bounded/incomplete status when search limits prevent proof of optimality.

**Increment C — held-out physical evidence**
- Pin selected published analytical/laboratory hydraulic and sediment references.
- Record solver version, mesh, boundary conditions, calibration, numerical tolerances and context of use.
- Compare independent models only on matched physical quantities and cases.
- Keep global screening data out of design-grade claims.

**Increment D — one regional demonstrator**
- Select only after source rights, local topographic/bathymetric surveys, water balances, ecological constraints and community process are qualified.
- Include a real no-build/operational-improvement baseline.
- Publish unsuccessful alternatives and uncertainty, not only the selected pathway.

**Increment E — public benchmark release**
- Freeze input corpus, exact versions and digests.
- Provide baseline implementations and resource budgets.
- Publish per-case outcomes and independent reproduction instructions.
- Maintain a separate scorecard for each physical, network, decision and evidence domain.

## 8. Claim ceiling

The 40-case register is a test specification. Passing synthetic tests will establish only the declared contract and oracle behavior. It will not establish:
- calibrated global or local probabilities;
- physical correctness outside tested contexts;
- actual water availability or legal entitlement;
- navigation safety, geotechnical suitability or ecological acceptability;
- community consent, financing, public value or construction approval;
- superiority over an industry tool without matched independent external validation.

No city, port, canal, water transfer or relocation is authorized or recommended by this benchmark.

## 9. Primary references

- GEBCO Bathymetric Compilation Group (2026), [GEBCO_2026 Grid](https://doi.org/10.5285/4f68d5c7-45eb-f999-e063-7086abc036fa); [grid documentation](https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid); [TID/grid development](https://www.gebco.net/data-products/gridded-bathymetry-data/grid-development).
- New Zealand Ministry for the Environment (2024; updated October 2026), [Coastal hazards and climate change guidance](https://environment.govt.nz/publications/coastal-hazards-and-climate-change-guidance/).
- EcoShape / World Bank (9 July 2025), [Nature-Based Solutions for Ports: A Practical Guide for a Resilient Future](https://www.ecoshape.org/en/nature-based-solutions-for-ports-a-practical-guide-for-a-resilient-future/).
- World Bank (2021), [Climate Risk and Ports: A Practical Guide on Strengthening Resilience](https://ppp.worldbank.org/library/climate-risk-and-ports-practical-guide-strengthening-resilience).
- openTELEMAC, [TELEMAC-3D and validation documentation](https://opentelemac.org/en/product/telemac3d).
- USACE, [EM 1110-2-1604: Hydraulic Design of Navigation Locks](https://www.publications.usace.army.mil/Portals/76/Publications/EngineerManuals/EM_1110-2-1604.pdf).
- PIANC, [Climate change adaptation for ports and waterways](https://www.pianc.org/climate-change/).

## Acceptance for protocol v0.1

- [x] Forty machine-readable case specifications exist across five qualification domains.
- [x] Seven hand-computable water-ledger oracles are declared.
- [x] Scenario-family integrity and long-horizon claim ceilings are explicit.
- [x] GEBCO_2026 TID/datum limitations are included.
- [x] DAPP and nature-based port guidance are included as established comparators.
- [ ] Full numeric fixture payloads and expected digests for all 40 cases.
- [ ] Independent executable oracle and adversarial mutation suite.
- [ ] Exact-head CI, with every case outcome accounted for.
- [ ] Independent hydraulic, sediment and external decision-policy validation.
