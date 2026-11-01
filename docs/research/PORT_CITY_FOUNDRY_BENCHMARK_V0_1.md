# Port-City Foundry Benchmark v0.1

**Status:** research specification; synthetic-first; not a design standard or engineering approval  
**Owner:** Sol-Atlas Built Environment Foundry  
**Tracking:** [MARITIME-FOUNDRY-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/370)  
**Cross-repository:** Symtropy, Symthaea, Mycelix  
**Version:** 0.1.0

## 1. Executive statement

Port-City Foundry is a benchmark and decision-support workstream for comparing complete maritime and inland-logistics infrastructure portfolios. It asks whether an existing port upgrade, offshore terminal, artificial barrier-island system, inland artificial basin, navigable canal, inland port, dry port, rail corridor, or distributed network produces the best long-term outcome for a defined region.

The central problem is not "can a port be built here?" It is:

> Which physically admissible, environmentally and socially acceptable, financially and economically defensible portfolio creates the greatest durable public value under uncertainty, compared with the best realistic alternative?

The Foundry should be able to discover a promising project configuration without presuming that new construction is the answer. A no-build, brownfield-upgrade, operational-improvement, demand-management, dry-port, or rail-first alternative must be allowed to win.

This is a systems-integration and benchmark programme. It is not a proposal to write a new coastal hydrodynamics engine in Rust, nor a claim that a generative optimizer can replace professional coastal, geotechnical, structural, navigation, ecological, economic, legal, or planning work.

## 2. Research gap and differentiation

Existing industry practice is already strong in important specialist areas:

- Deltares Delft3D FM covers hydrodynamics, hydrology, real-time control, morphodynamics, waves, water quality, and particle tracking for coastal, estuarine, river, rural, and urban contexts.
- XBeach is purpose-built for nearshore storm response and morphodynamic processes including dune erosion, overwash, inundation, scour, and beach change; it has analytical, laboratory, and field validation cases.
- GTSM provides global water-level/tide/surge context, with the current Deltares project page describing GTSMv4.1 and approximately 1.25 km coastal resolution.
- Port-performance and maritime-connectivity indicators already exist: World Bank/S&P Global's CPPI uses vessel time in port; UNCTAD's LSCI/PLSCI describe national and port-level liner-shipping connectivity.
- Established appraisal frameworks compare infrastructure costs and benefits; a port study must account for capacity, access, maintenance, demand, alternatives, and public value.

We should not claim superiority over these systems' validated physical solvers or over professional port-engineering workflows. Our opportunity is to integrate across a broader decision boundary:

1. Discover alternative geographies and infrastructure families from outcome requirements.
2. Compare maritime and non-maritime alternatives on the same door-to-door logistics and demand assumptions.
3. Couple physical-model results to lifecycle economics, resilience, ecological limits, rights, and distributional outcomes.
4. Preserve uncertainty and the Pareto frontier rather than forcing one weighted winner.
5. Identify which new investigation is most likely to change a decision.
6. Preserve independently auditable provenance from raw data through transformation, simulation, appraisal, review, and decision.

The differentiator is **whole-system design-space coverage, evidence integrity, robust comparison, and adaptive investment**, not an unsupported claim of better wave physics.

## 3. Ownership boundaries and existing work

Do not create parallel owners for capabilities that already exist.

### Sol-Atlas

This workstream composes:

- [#129](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/129): evidence-first site-suitability policy kernel.
- [#131](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/131) and [#132](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/132): robust Pareto analysis, sensitivity, and decision frontiers.
- [#133](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/133) and [#134](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/134): EVSI and spatial investigation planning.
- [#135](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/135) / [#136](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/136): source provenance, rights, freshness, CRS, resolution, and coverage.
- [#130](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/130) / [#140](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/140): GIS ingestion and conservative spatial overlay semantics.
- [#138](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/138): built-environment evidence graph and approval validation.
- [#65](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/65) / [#115](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/115): geospatial asset catalog and maritime data projection.

Sol-Atlas owns spatial candidate context, evidence-aware eligibility, the port-city alternative portfolio, multi-objective comparisons, investigation-value views, and visualization. It must not turn a rendered layer into an engineering finding.

### Symtropy

Reuse:

- [#1601](https://github.com/Luminous-Dynamics/symtropy/issues/1601): whole-life multimodal transport optimization and social cost.
- [#1612](https://github.com/Luminous-Dynamics/symtropy/issues/1612): deterministic maritime / floating-site scenario corpus.
- [#1634](https://github.com/Luminous-Dynamics/symtropy/issues/1634) and [#1644](https://github.com/Luminous-Dynamics/symtropy/issues/1644): source-pinned maritime receipts and IHO S-100 profiles.
- [#451](https://github.com/Luminous-Dynamics/symtropy/issues/451) and [#465](https://github.com/Luminous-Dynamics/symtropy/issues/465): estuary/coastal and coarse ocean profiles.
- [#1790](https://github.com/Luminous-Dynamics/symtropy/issues/1790): solver-neutral built-environment simulation adapters and validation boundaries.
- [#1172](https://github.com/Luminous-Dynamics/symtropy/issues/1172), [#1173](https://github.com/Luminous-Dynamics/symtropy/issues/1173), and [#1467](https://github.com/Luminous-Dynamics/symtropy/issues/1467): deterministic execution, model-suitability, and digital-twin boundaries.
- [#1858](https://github.com/Luminous-Dynamics/symtropy/issues/1858), [#1867](https://github.com/Luminous-Dynamics/symtropy/issues/1867), and [#1883](https://github.com/Luminous-Dynamics/symtropy/issues/1883): port-call scheduling, contingent recovery, and staged port-energy investment.

Symtropy owns deterministic simulation orchestration, replay, objective-vector evaluation, scenario execution, and optimization. Its rigid-body/game physics must not be presented as validated coastal flow, wave, sediment, geotechnical, or structural analysis.

### Symthaea

Reuse the Built Environment Foundry intent-to-design compiler [#7790](https://github.com/Luminous-Dynamics/symthaea/issues/7790), transport engineering and coverage audit [#6244](https://github.com/Luminous-Dynamics/symthaea/issues/6244) / [#6245](https://github.com/Luminous-Dynamics/symthaea/issues/6245), and marine-freight / dock co-design [#7444](https://github.com/Luminous-Dynamics/symthaea/issues/7444).

Symthaea may generate hypotheses, compile requirements into candidate design intent, compare evidence, and identify missing investigations. It does not certify its own candidate or elevate a simulation into physical truth.

### Mycelix

Reuse construction provenance and approval receipts [#5149](https://github.com/Luminous-Dynamics/mycelix/issues/5149), logistics commons [#4901](https://github.com/Luminous-Dynamics/mycelix/issues/4901), and production/dependency-network economics [#4814](https://github.com/Luminous-Dynamics/mycelix/issues/4814).

Mycelix owns identity, participant authority, commitments, rights/consent records, governance, provenance, and decision accountability. A Mycelix receipt is not a planning permission, land title, navigation authorization, or engineering certificate.

## 4. Alternative families

Each scenario must compare multiple portfolios against a realistic baseline. Candidate families are composable; a single project may combine an offshore terminal, a short protected channel, a mainland logistics hub, and inland rail.

| ID | Alternative family | Core design variables |
|---|---|---|
| ALT-00 | No-build / operational optimization | demand management, scheduling, berth/yard utilization, maintenance |
| ALT-01 | Existing port upgrade | berth, yard, access, handling, dredging, rail, shore power |
| ALT-02 | Brownfield expansion / reclamation | land footprint, dredge/fill balance, quay, access, sediment |
| ALT-03 | Offshore / deep-water terminal | terminal footprint, breakwater, causeway / bridge / tunnel / rail |
| ALT-04 | Segmented barrier-island system | island geometry, inlet spacing, tidal exchange, channels, sediment budget |
| ALT-05 | Inland basin / artificial bay | canal alignment, basin volume, locks/gates, pumping, water balance |
| ALT-06 | River / lake / canal inland port | channel class, seasonal navigability, draft, transfer facilities |
| ALT-07 | Dry port + inland corridor | rail/road capacity, terminal location, transfer time, customs / border friction |
| ALT-08 | Distributed port and hub network | portfolio of ports, redundancy, transshipment, inter-hub connections |

For each family the generator must expose a finite, bounded, versioned set of parameters. It must not invent design dimensions from a coarse global raster.

### Barrier-island hypotheses

Test segmented rather than only continuous barriers. Potential mechanisms include wave attenuation, changes in inlet currents, sediment trapping/redistribution, lagoon flushing, habitat creation, navigation access, and maintenance dredging. A design can only be credited with these co-benefits where a suitable model or measured evidence supports them.

Explicitly test adverse cases:

- concentrated tidal jets and scour at inlets;
- inlet shoaling and navigation-channel maintenance;
- downdrift erosion or sediment starvation;
- poor lagoon flushing, salinity change, and water-quality degradation;
- storm breaching, overwash, and island migration;
- induced flood levels or altered river/estuary backwater;
- loss or fragmentation of intertidal, wetland, fisheries, or coastal habitats.

### Inland bay / canal hypotheses

Distinguish navigation classes. A barge canal, small-vessel basin, and ocean-going container ship canal have materially different width, depth, turning, lock, bridge-clearance, and water-supply requirements.

Test:

- excavation and spoil / beneficial-use quantities;
- groundwater and aquifer interaction;
- lock/gate/pumping energy and reliability;
- saline intrusion and water-quality exchange;
- flood conveyance and drainage effects;
- sedimentation and maintenance;
- bridge/road/rail crossing impacts;
- vessel wake, bank protection, and operational constraints.

No inland bay is feasible merely because a geometric cut can connect it to the sea.

## 5. Candidate and evidence contract

The first machine-readable contract should be versioned and renderer-neutral. It should describe a candidate and its evidence without embedding one model's implicit assumptions.

### Candidate identity and geometry

- Stable candidate ID, alternative-family ID, parent-region ID, revision.
- Geometry type and canonical geometry digest.
- Horizontal CRS authority/code, axis order, coordinate units, projection, and declared transformations.
- Vertical datum, units, and transformation provenance wherever elevation or bathymetry is used.
- Distinct original geometry and each derived geometry; all transformations pin parent digest(s), software/version, parameters, and output digest.

### Site evidence

- Bathymetry/topography source, provider, edition/revision, source time, retrieval time, coverage, grid resolution, source-type metadata, uncertainty/accuracy, vertical datum, and rights/terms.
- Geotechnical, geophysical, ecological, hydrological, land-use, infrastructure, rights, and community evidence identities and jurisdictions.
- Observed / Derived / Curated / Scenario classification.
- Stale, unknown, disputed, restricted, partial, invalid, unsupported, and out-of-coverage states kept distinct.

### Engineering envelope

- Vessel class and declared design vessel envelope.
- Channel/berth depths and widths as intervals; turning/anchorage envelope; tide-window/under-keel clearance assumptions.
- Island, reclamation, breakwater, dredging, excavation, spoil, and fill quantities as sourced intervals.
- Construction stages, material quantities, logistics, access, utility capacities, and maintenance schedule.
- Model contexts of use, boundary conditions, mesh/grid, calibration/validation scope, numerical convergence, uncertainty, and unsupported processes.

### Network and urban envelope

- Time-dependent multimodal graph: port, terminal, dry port, rail yard, road link, inland waterway, industrial node, population centre.
- Capacity, time, cost, reliability, transfer, service frequency, seasonal availability, and source provenance on each edge.
- Commodity / passenger demand matrix with reference period, coverage, scenario assumptions, and uncertainty.
- Industrial/housing/utility capacity ranges, phasing, water and energy inputs, workforce/training assumptions, and access to essential services.

### Rights and social constraints

- Land/water rights evidence, jurisdiction, competent authority, source revision, and unresolved claim state.
- Community participation and consent processes where required; affected groups and distributional impacts.
- Protected ecology and non-compensable constraints represented as hard gates when required by law or policy.
- No automatic land acquisition, displacement, permit, or planning approval inference.

## 6. Multi-ledger economics

A single "economic score" is not sufficient. Maintain separately inspectable ledgers.

### 6.1 Financial / bankability ledger

Project cash flows: capital expenditure, financing, revenues, operating costs, maintenance, replacement, lifecycle renewals, residual value, and funding gap. Show currency, price base, tax treatment, inflation convention, financing structure, and assumptions.

### 6.2 Fiscal / public-budget ledger

Public capex, grants, guarantees, subsidies, tax receipts, public-service costs, contingent liabilities, and affordability of ongoing operations. Transfers should be visible but not mistaken for net social benefit.

### 6.3 Economic cost-benefit ledger

Count real resource opportunity costs and changes in generalized door-to-door freight/passenger cost:

- freight rates and handling where they reflect resource costs;
- transit time and delay variance;
- inventory carrying cost, spoilage, and missed delivery windows;
- transshipment, first/last mile, empty repositioning, and reliability;
- asset construction, land, survey, materials, equipment, operations, maintenance, dredging and disposal;
- lock/gate/pump energy and water use;
- climate adaptation, closure risk, renewal, and decommissioning;
- credible avoided flood damages and other co-benefits, without double-counting.

Where appropriate report NPV, BCR, EIRR, cost per delivered tonne / passenger, annualized lifecycle cost, and economic cost per tonne-kilometre. Discount rates, valuation conventions, appraisal periods, and shadow prices must be jurisdiction- and study-specific.

### 6.4 Environmental ledger

Report lifecycle greenhouse gases, local air pollution, sediment disturbance, dredging/disposal, habitat impact, water quality, salinity, freshwater demand, noise, land take, and ecological connectivity. Some of these are constraints or separate indicators, not monetary quantities to be traded off against a larger NPV.

### 6.5 Distributional ledger

Disaggregate costs/benefits by affected communities, workers, small businesses, rural and inland regions, users, taxpayers, and land/water rights-holders. Track displacement, accessibility, affordability, employment quality, local value-add and service reach. Gross job counts or payroll are not automatically net social benefits: account for labour opportunity cost, displacement, and whether jobs are additional and durable.

### 6.6 Resilience and real-options ledger

Compare expected and downside outcomes across climate, demand, trade, energy, financing, and construction scenarios. Include service continuity, recovery time, redundancy, adaptation triggers, staged investment, reversibility, and stranded-asset risk. Report robust metrics such as worst-case regret or CVaR only with clearly defined scenario weights and limitations.

Do not sum all these ledgers into one universal score. Keep mandatory safety, rights, ecological, water, and statutory conditions as non-compensable gates. Use robust Pareto analysis and show where stakeholder preferences change the order of options.

## 7. Optimization and decision methods

Start with a transparent hierarchy.

1. **Hard-gate evaluation:** apply source, data-rights, coverage, freshness, CRS, engineering, ecological, legal, navigation, and rights requirements before scoring.
2. **Candidate generation:** generate bounded, valid alternatives from declared parameter sets.
3. **Physical and network evaluation:** request only supported solver paths; unknown processes remain unknown.
4. **Economic ledger evaluation:** calculate each ledger with explicit units, price bases, timing, and source lineage.
5. **Robust multi-objective comparison:** preserve the nondominated frontier using intervals and scenario sets.
6. **Sensitivity:** vary construction costs, freight demand, growth, discount rates, storm/climate forcing, sediment coefficients, energy prices, and stakeholder preferences.
7. **Value of information:** select the next survey or investigation based on its potential to change the decision, while never waiving mandatory investigations.
8. **Adaptive investment:** compare one-shot megaprojects with modular stages and observation-triggered expansion / deferral / stop decisions.

### Algorithm baselines

Use the right method for the problem structure; compare under equal input, compute, and evaluation budgets:

- exhaustive enumeration for small fixtures (ground truth);
- deterministic weighted-sum baseline, with its limitations exposed;
- epsilon-constraint method;
- NSGA-II / NSGA-III or another declared evolutionary baseline;
- MOEA/D where objective decomposition is suitable;
- mixed-integer / linear optimization for network and capacity subproblems where assumptions support it;
- simple greedy and existing-planning baselines.

Report feasibility rate, hard-gate violation count, robust Pareto precision/recall, hypervolume (with a frozen reference point), epsilon indicator, regret to exhaustive or best-known reference, runtime, memory, function evaluations, seed variability, and stability under perturbation. A better hypervolume score does not excuse false feasibility or a worse safety/ecology outcome.

## 8. Benchmark suite and promotion gates

### PCF-GEO-001 — evidence-aware geographic screening

At least 30 frozen synthetic candidate-region cases: full/partial/unknown coverage, CRS and datum mismatches, source revision changes, coarse vs fine resolution, source rights, geometry invalidity, antimeridian, protected areas, land/water constraints, and uncertain boundaries.

**Gate:** 100% of planted hard-constraint and evidence-completeness negatives get the expected blocked / ineligible / NeedsEvidence / invalid disposition; zero false-safe or false-eligible outcomes.

### PCF-ALT-002 — alternative-family completeness

Cases cover ALT-00 through ALT-08, multi-family compositions, and one or more candidate families dominated by a non-build or upgrade alternative.

**Gate:** all configured alternatives are either evaluated or rejected with an explicit reason; no hidden default eliminates a family.

### PCF-OPT-003 — multi-objective optimization correctness

Small hand-computable portfolios with exact Pareto sets, ties, interval overlap, dominated points, missing data, and changed hard gates.

**Gate:** exact Pareto frontier on every hand-computable fixture; zero excluded candidate re-entry; order-independent canonical result.

### PCF-ECON-004 — social-cost accounting

Synthetic cases with transfers vs real resource costs, double-count traps, time/reliability/inventory changes, staged capex, maintenance/dredging, residual value, and distributional effects.

**Gate:** independent calculation mirror agrees within predeclared tolerance; no double-counted revenue, payroll, tax, subsidy, avoided delay, or user price; all currencies and price bases explicit.

### PCF-HYDRO-005 — coastal hydrodynamic interface

Analytical and published reference cases for tide propagation, steady and transient flow, estuary exchange, salinity/tracer conservation, wetting/drying, and water-level boundaries.

**Gate:** each quantity gets an independent per-case tolerance, conservation residual, boundary-condition record, and validation scope. A non-converged or unsupported case is not a pass.

### PCF-WAVE-006 — wave and storm response

Use model-specific analytical/laboratory/field reference cases for wave transformation, setup, overtopping/overwash, inundation, and structure interaction.

**Gate:** publish per-variable bias/RMSE or suitable process-specific metric; freeze the acceptance tolerance before scoring and keep calibration and holdout cases separate.

### PCF-SED-007 — sediment and morphology

Test bedload/suspended-load conservation, inlet/channel shoaling, scour, deposition, shoreline evolution, sediment budget, and dredging frequency/volume against appropriate reference data.

**Gate:** mass-balance residual and morphology errors reported separately; do not infer decadal island stability from a short storm run.

### PCF-NAV-008 — navigation and capacity

Test declared vessel envelope, under-keel/tide constraints, channel/turning envelope, berth compatibility, lock/gate restrictions, and throughput.

**Gate:** unknown datum, unsupported vessel class, missing channel survey, or failed clearance calculation cannot produce a navigable / safe disposition.

### PCF-NET-009 — regional logistics and network value

Use time-dependent freight OD, port connectivity, inland rail/water/road capacity, transfer, queues, reliability, and disruption.

**Gate:** compare total door-to-door generalized cost and delivered service, not only port throughput. Compare with existing-port and dry-port / rail baselines.

### PCF-CLIM-010 — climate and uncertainty

Perturb sea level, surge, waves, rainfall, river discharge, sediment supply, heat, demand, energy, and construction/financing cost.

**Gate:** report robust feasibility and rank reversals; no single forecast treated as certainty.

### PCF-ROBUST-011 — staged and adaptive capital

Compare one-shot, staged, modular, and reversible portfolios with demand and hazard observations over time.

**Gate:** quantify stranded capital, option value, regret, maintenance liabilities, service continuity, and explicit trigger logic.

### PCF-EVID-012 — reproducible evidence receipt

Pin input bytes, transformation receipts, geometry, policy, solver, model configuration, toolchain, objective definitions, random seeds, and result manifests.

**Gate:** same canonical manifest + same toolchain/policy reproduces the same canonical output; tampering, missing files, stale input, changed model identity, or unsupported semantics fail closed.

### PCF-IND-013 — industry baseline comparison

Compare with documented industry/academic workflows under identical scope, data quality, compute budgets, and evaluation rules.

**Gate:** publish exact baseline versions, hardware, solver settings, calibration/holdout split, costs, and limitations. No "better than SOTA" claim without a repeatable like-for-like result.

### Initial quantitative targets

- 100% planted hard-gate / critical-evidence negatives correctly blocked or marked indeterminate; zero false-eligible outputs.
- 100% deterministic replay for canonical policy and portfolio outputs under a pinned toolchain / numeric policy.
- 100% exact frontier match on hand-computable fixtures.
- <=1% objective gap versus exhaustive enumeration on small optimization fixtures, with no constraint violation.
- Report (do not conceal) Pareto precision/recall, hypervolume, epsilon indicator, regret, runtime, memory, and robustness for larger test cases.
- No threshold should be generalized from synthetic fixtures to real engineering suitability.

## 9. Reference data and fitness for use

### Global screening

**GEBCO_2026:** global topography/bathymetry at 15 arc-second spacing, with a Type Identifier Grid (TID) that distinguishes data-source types. The grid combines direct measurements and indirect/interpolated material. It is useful for coarse global screening, but its grid spacing and heterogeneous source support do not make it a local hydrographic survey or navigation chart. The TID must be retained in any screening result.  
Source: https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid

### Tide, surge, and sea-level scenarios

**Deltares GTSM:** global water-level / tide / storm-surge model. Deltares describes GTSMv4.1 as having about 1.25 km coastal resolution and reports a reduction in global-coast tide RMSE from 22.3 cm in v2.0 to 11.3 cm in v4.1 for the comparison described on its project page. Those are model-comparison metrics, not guaranteed error bounds at a proposed site. Use as regional context and scenario boundary evidence; refine with local measurements and site-appropriate modelling.  
Sources:
- https://www.deltares.nl/en/expertise/projects/global-modelling-of-tides-and-storm-surges
- https://publicwiki.deltares.nl/spaces/GTSM/pages/201033101/Model+description+and+development

**ECMWF/Copernicus global sea-level dataset:** describes tide, surge and sea-level-change time series generated with GTSM v3.0 at 10-minute time intervals for 1950–2050. Record the model generation/version and scenario provenance; do not merge it silently with GTSMv4.1 outputs.  
Source: https://www.ecmwf.int/en/forecasts/datasets/global-sea-level-change-time-series-1950-2050-derived-reanalysis-and-high-resolution-cmip6-climate-projections

### Waves

**Copernicus Marine Global Ocean Waves Reanalysis:** global 0.2° x 0.2° grid, hourly and monthly products, with temporal coverage listed from 1980 to 31 August 2026. Suitable for broad regional context and boundary forcing discovery; far too coarse to substitute for nearshore wave transformation, harbour resonance, overtopping, or local design conditions.  
Source: https://data.marine.copernicus.eu/product/GLOBAL_MULTIYEAR_WAV_001_032/description

### Regional bathymetry and marine evidence

**EMODnet Bathymetry:** European products include approximately 115 m grids in covered maritime basins, with survey-track metadata and access services. This is region-specific coverage, not a global resolution guarantee.  
Sources:
- https://emodnet.ec.europa.eu/en/bathymetry
- https://emodnet.ec.europa.eu/en/emodnet-web-service-documentation

For real project studies, require authoritative local hydrographic surveys, multibeam / singlebeam metadata, uncertainty layers, vertical datum, tide reductions, survey date, licence, and currentness. A planning bathymetry grid must not be treated as a chart for navigation.

### Maritime networks and existing port performance

- **UNCTAD LSCI / PLSCI:** country-level and port-level liner shipping connectivity. The indices are useful context for network integration; they are not a full port-demand model or a measure of physical feasibility. https://unctadstat.unctad.org/insights/theme/111
- **World Bank / S&P Global CPPI:** global comparative container-port performance based primarily on vessel time in port. Use to contextualize operational efficiency of existing container ports, not to rank new sites by itself. https://www.worldbank.org/en/topic/transport/publication/cppi
- **World Bank Ports PPP module:** a due-diligence reference for existing infrastructure, capacity, service levels, condition, financial performance, and upgrade needs. https://ppp.worldbank.org/transportation/ports-module
- **World Bank, Integrating Africa: From Threads to Hubs (25 August 2026):** regional value chains, reduction of trade frictions, interoperable systems, and corridor / regional public goods. This supports evaluating port proposals as regional production-and-logistics networks rather than isolated terminals. https://www.worldbank.org/en/brief/2026/08/25/integrating-africa-from-threads-to-hubs

## 10. Solver integration strategy

Use a Rust-first control plane for schemas, candidate enumeration, policy, objective accounting, orchestration, canonical receipts, and independent verification. Specialist numerical solvers remain external, version-pinned, and selected by context of use.

Candidate solver roles:

- **Delft3D FM:** regional hydrodynamics, tides, storm surge, estuarine/coastal flows, waves, morphology, sediment, and water quality where the chosen modules are appropriate and qualified.
- **XBeach:** storm-scale nearshore hydro- and morphodynamic response (e.g. overwash, erosion, inundation, scour); not a global/century-scale harbour model.
- **TELEMAC / SCHISM:** alternative specialist models for river-estuary-coast and unstructured-grid applications where model capability, available validation data, and licensing fit.
- **Separate geotechnical / structural / navigation authorities:** required for foundations, settlement, quay walls, breakwaters, locks, bridges, moorings, vessel manoeuvring, and under-keel clearance.

An adapter receipt should pin solver product/version/build digest, input files and digests, mesh/grid, coordinate and vertical datums, forcing/boundary data, parameter files, calibration data, validation data, convergence, numerical diagnostics, run environment, outputs/digests, uncertainty, limitations, and exact intended use.

A completed solver process is not equivalent to a validated result. A solver run, a model qualification, a project design review, and a statutory approval are separate states.

References:
- https://www.deltares.nl/en/software-and-data/products/delft3d-flexible-mesh-suite
- https://www.deltares.nl/en/software-and-data/products/xbeach
- https://github.com/schism-dev/schism

## 11. Benchmark protocol and comparison with industry

Every claimed performance improvement should use a pre-registered benchmark protocol:

1. Freeze problem statement, required outputs, constraints, objective directions, data, licences, and evidence cut-off.
2. Separate calibration, validation, and held-out scenarios by site/case family, not merely by random rows from one geometry.
3. Record every solver and optimizer version, configuration, hardware class, time budget, tolerance, seed, and failed run.
4. Compare against relevant baselines using the same input information and resource budget.
5. Publish raw machine-readable results, failure cases, uncertainty, and negative controls.
6. Re-run from a clean environment using immutable manifests.
7. Have a technically independent reviewer inspect benchmark design and the reference calculation.

### Industry comparison matrix

| Capability | Mature industry / research baseline | Foundry strategy |
|---|---|---|
| Coastal hydrodynamics | Delft3D FM, TELEMAC, SCHISM and other specialist models | Integrate; validate adapters; do not claim replacement |
| Storm-scale beach / dune processes | XBeach and physical/lab references | Use matched contexts of use; compare outputs and uncertainties |
| Bathymetric screening | GEBCO and regional hydrographic authorities | Preserve source TID, datum, uncertainty, coverage, and survey age |
| Port operating efficiency | CPPI, terminal KPIs, port-call data | Contextualize operations; never use as sole project-ranking target |
| Shipping connectivity | UNCTAD LSCI / PLSCI | Include network effects and inland connectivity |
| Project appraisal | Jurisdictional CBA / World Bank PPP due diligence / engineering studies | Multi-ledger appraisal with explicit counterfactual, uncertainty and distribution |
| Multi-objective optimization | Enumeration, epsilon-constraint, NSGA-II/III, MOEA/D, MILP where applicable | Compare equally; prove correctness on small exact cases; emphasize robust feasibility and regret |
| Provenance and public accountability | Project-specific audit / assurance processes | Typed, digest-pinned, cross-project evidence and governance records |

The path to "better than SOTA" is a sequence of independently demonstrated advantages, not a broad claim:
- broader alternatives considered at equal fidelity;
- fewer false-eligible candidates;
- less rank reversal under plausible uncertainty;
- lower lifecycle social cost or higher service / resilience at equivalent constraints;
- more efficient investigation planning;
- lower regret through staged capital;
- reproducible evidence and clearer public accountability.

If a reference solver outperforms our in-house simplification for a particular physical quantity, use the reference solver. A credible platform must be willing to lose a solver bake-off.

## 12. First implementation slices

1. **Contract and fixture layer:** Rust types for candidate-family, geometry/evidence references, objective ledgers, and claim ceilings. Add JSONL synthetic fixtures and canonical manifests.
2. **Independent verifier:** independently recompute hard-gate dispositions, economic-accounting identities, Pareto frontiers, and deterministic output digests.
3. **Portfolio composer:** compose existing Sol-Atlas site eligibility, robust Pareto, source freshness, spatial overlay, and EVSI rather than copying them.
4. **Symtropy exchange manifests:** request qualified model runs and consume outputs only under the returned solver/model-suitability profile.
5. **Reference reconstructions:** start with one reclaimed/offshore port-extension case and one inland port/waterway case, using public documented information; clearly separate reproduced geometry, assumptions, and missing data from verified facts.
6. **Regional pilot:** choose a Southern African corridor only after auditing official hydrographic, geotechnical, environmental, land/water rights, freight and infrastructure evidence. Do not preselect a build site from global screening grids.
7. **Human decision interface:** expose all competing portfolios, assumptions, unresolved requirements, affected parties, and investigation options; keep authority with competent institutions and rights-holders.

## 13. Non-goals and claim ceiling

This benchmark does not:

- produce construction-ready breakwater, island, canal, basin, lock, bridge, quay, foundation, or dredging designs;
- certify navigation, under-keel clearance, vessel manoeuvring, structural safety, geotechnical stability, or flood protection;
- provide an environmental/social impact assessment, land-title opinion, planning permission, or legal approval;
- convert public accessibility into a licence or permission to redistribute;
- infer safe absence from missing, coarse, stale, or incomplete data;
- turn synthetic fixtures or a visualization into real-world evidence;
- assume that new construction, artificial islands, or city growth is intrinsically beneficial.

Real-world decisions require local data, qualified independent engineering, environmental and social assessment, community participation, rights-holder processes, and statutory approvals.

## 14. References

1. GEBCO Bathymetric Compilation Group (2026), GEBCO_2026 Grid and TID: https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid
2. Deltares, Delft3D Flexible Mesh Suite: https://www.deltares.nl/en/software-and-data/products/delft3d-flexible-mesh-suite
3. Deltares, XBeach: https://www.deltares.nl/en/software-and-data/products/xbeach
4. Deltares, Global Modelling of Tides and Storm Surges: https://www.deltares.nl/en/expertise/projects/global-modelling-of-tides-and-storm-surges
5. ECMWF / Copernicus Climate Data Store, Global sea-level time series 1950–2050: https://www.ecmwf.int/en/forecasts/datasets/global-sea-level-change-time-series-1950-2050-derived-reanalysis-and-high-resolution-cmip6-climate-projections
6. Copernicus Marine, Global Ocean Waves Reanalysis: https://data.marine.copernicus.eu/product/GLOBAL_MULTIYEAR_WAV_001_032/description
7. EMODnet, Bathymetry and web services: https://emodnet.ec.europa.eu/en/bathymetry
8. UNCTADstat, Liner Shipping Connectivity Index and Port Liner Shipping Connectivity Index: https://unctadstat.unctad.org/insights/theme/111
9. World Bank / S&P Global, Container Port Performance Index: https://www.worldbank.org/en/topic/transport/publication/cppi
10. World Bank, Ports PPP Module: https://ppp.worldbank.org/transportation/ports-module
11. World Bank (25 August 2026), Integrating Africa: From Threads to Hubs: https://www.worldbank.org/en/brief/2026/08/25/integrating-africa-from-threads-to-hubs
12. SCHISM project: https://github.com/schism-dev/schism

---

**Status note:** This document is a proposed benchmark contract. No code implementation, physical simulation, site assessment, independent validation, or industry-performance comparison is claimed by this specification.


## 15. Geographic discovery and ex-post outcome extensions

The parent programme now has three explicitly separated follow-ons:

- [GEO-OPPORTUNITY-001 #377](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/377): discover coast–hinterland opportunity systems and compare natural, engineered, inland, dry-port and no-build alternatives. Its research note is [PORT_CITY_GEOGRAPHIC_OPPORTUNITY_DISCOVERY_V0_1.md](PORT_CITY_GEOGRAPHIC_OPPORTUNITY_DISCOVERY_V0_1.md).
- [PCF-CORPUS-001 #378](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/378): 30+ frozen synthetic alternatives and an independent verifier. Draft implementation: [PR #379](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/379).
- [PORT-OUTCOME-001 #380](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/380): ex-post causal outcome and forecast-calibration benchmark for completed, delayed, underused or canceled port/corridor projects.

The ex-post extension responds to a 2026 systematic review of 90 peer-reviewed port–hinterland studies. Its synthesis spans port choice, corridor competition, dry ports, governance, multimodal operations, digitalization, sustainability and resilience, and reports that much of the evidence remains model-based with limited causal identification. The Foundry should therefore distinguish modelled counterfactuals from measured outcomes and use pre-registered event-study, difference-in-differences, synthetic-control or other defensible designs where data and assumptions permit. Source: https://doi.org/10.3390/jmse14151434

The geographic extension also incorporates the 24 September 2026 *Nature Communications* barrier-island field study, which found morphology-dependent sediment redistribution and incomplete post-storm recovery in its studied system. Generic island geometries and unvalidated sediment assumptions must not be treated as stable designs. Source: https://www.nature.com/articles/s41467-026-77143-6

These additions expand the benchmark from "can we generate an alternative?" to:
1. can we discover a credible opportunity without missing important candidate classes?
2. can we preserve hard evidence and engineering boundaries while comparing alternatives?
3. can we determine after the project whether the forecasted regional value actually materialized?


## 16. Port–hinterland redundancy is not port count

Port-City Foundry must measure **effective independent access**, not how many port markers appear within a region. A hinterland with several gateways can still be dependent on a shared rail line, bridge, border crossing, power source, floodplain, carrier or chokepoint. Conversely, a landlocked region may have multiple viable port ranges even when all routes are relatively costly.

A 2026 *Transportation Research Part A* study of African port-hinterland redundancy reports trade-offs between accessibility and redundancy. It notes that terminal expansion can reduce redundancy for a core hinterland while increasing it for the periphery, and that rail connections can broaden access to ports while potentially reinforcing reliance on a single cost-efficient connection. Source: https://doi.org/10.1016/j.tra.2026.104920.

Sol-Atlas discovery should expose the source/coverage of access and failure-domain data. Symtropy should own the path, cut-set, capacity, disruption and network service benchmark under [MARITIME-REDUNDANCY-001 #3199](https://github.com/Luminous-Dynamics/symtropy/issues/3199), composing [TRANSPORT-OPT-001 #1601](https://github.com/Luminous-Dynamics/symtropy/issues/1601) and historical disruption replay [#1671](https://github.com/Luminous-Dynamics/symtropy/issues/1671).

Add to PCF-NET-009: accessibility and effective redundancy curves; edge-/node-/risk-domain-disjoint paths; min-cut and shared-bottleneck exposure; normal vs disrupted gateway reachability; delivered vs unmet demand; traffic diversion vs new demand; service recovery; and incremental resilience per lifecycle dollar. Do not assign probabilities to correlated failures without evidence. Treat failure-domain independence as Unknown when dependencies are not mapped.

The aim is not maximum redundancy at any cost. Compare the cost of independent access against the value of avoided disruptions and the option to divert cargo under stress. A new port should lose when upgrading a shared inland bottleneck or improving existing alternatives produces greater risk-adjusted public value.

## 17. Ex-post outcomes and forecast calibration

[PORT-OUTCOME-001 #380](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/380) adds a separate evidence track for comparing ex-ante project forecasts against later observed outcomes and defensible counterfactuals. A 2026 systematic review of 90 port–hinterland studies found 51/90 (56.7%) were model-based optimisation/simulation/analytical designs, while quasi-experimental identification was absent from the reviewed corpus. This is a published review's sample and search protocol, not a claim about every port study. Source: https://doi.org/10.3390/jmse14151434.

The Foundry's claimed advantage should therefore include outcome calibration and causal evidence where possible, not only better simulation or a larger candidate search. Historical forecasts must be frozen at their information cut-off; projects should be evaluated for additional network-wide service/value rather than traffic diverted from other gateways. Quasi-experimental methods require pre-registered outcomes, credible comparison groups, trend/placebo checks and transparent identification limits.
