# Port-City Foundry: Geographic Opportunity Discovery v0.1

**Status:** research specification; investigation hypotheses only  
**Parent:** [MARITIME-FOUNDRY-001 #370](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/370)  
**Work item:** [GEO-OPPORTUNITY-001 #377](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/377)  
**Date:** 2026-11-01

## 1. Purpose

Add a first-class geographic-discovery capability to Port-City Foundry. The system should answer:

> Where should we investigate a new maritime gateway, port upgrade, artificial harbour, barrier-island configuration, inland basin, waterway, dry port, rail corridor, or distributed logistics network—and where should we not build?

Do not confuse three different questions:

1. **Potential:** could a candidate geography plausibly support an infrastructure concept worth investigating?
2. **Feasibility:** does adequate site-specific evidence and qualified modelling establish that a defined design is physically, environmentally, legally and socially admissible?
3. **Investment merit:** does the admissible alternative outperform the best realistic counterfactual on financial, fiscal, economic, distributional and resilience grounds?

Global screening can only help with the first question. It cannot make the second or third decisions.

The search object should be a **coastal–hinterland opportunity system**, not a pin or square raster cell. It connects a marine-access envelope, candidate terminal/basin/canal configuration, inland production and demand catchments, multimodal transport graph, and evidence about ecology, hazards, geology, water, rights and communities.

## 2. Core paradigm shift: optimize the economic geography

A deep-water site can be a poor port if its hinterland is inaccessible. A difficult coastline can be worth investigating if an offshore terminal plus a reliable inland corridor reduces total delivered cost. An inland dry port, upgraded existing harbour, demand-management measure or no-build option may outperform any new marine structure.

Therefore search two spaces simultaneously:

### Natural-advantage discovery

- Naturally sheltered bays and coves.
- Estuaries and river mouths, with explicit sediment/flood/salinity/ecology context.
- Deep-water approaches and nearshore bathymetric gradients.
- Naturally protected anchorages and existing port assets.
- Navigable rivers, lakes and canals.
- Terrain and geology compatible with access corridors and terminal development.
- Existing road, rail, power, water, industrial and urban systems.

### Engineered-advantage discovery

- Exposed coastlines where a breakwater or offshore terminal could be investigated.
- Segmented barrier-island / multiple-inlet / lagoon configurations.
- Artificial offshore platforms and deep-water terminals.
- New canals and inland artificial basins.
- Port-plus-dry-port / rail portfolios.
- Distributed terminal and logistics-hub networks.

Each engineered option must carry the cost and evidence burden it creates: dredging, fill, structures, wave/sediment processes, maintenance, utilities, inland connections, rights, ecological effects and long-term adaptation. Do not treat an engineered opportunity as superior merely because it can be drawn in a digital twin.

## 3. Search architecture

### Stage 0 — define the service problem

A study starts with a declared problem statement:

- commodities / passengers and origin–destination demand;
- current routes, ports, transshipment and border crossings;
- vessel classes, draft / air-draft / turning envelope and service frequency;
- required throughput, service windows and reliability;
- time horizon, growth scenarios and climate horizon;
- affordability, finance and public-budget constraints;
- industrial / agricultural / population nodes to be served;
- non-compensable safety, environmental, water, land/water-rights and community constraints;
- jurisdiction-specific appraisal rules and price base.

Missing or weak demand evidence remains uncertain. The model must not invent demand to justify capacity.

### Stage 1 — build meaningful geography objects

- Derive coastline reaches, bays, estuary mouths, river mouths, waterway segments and inland corridors from versioned source geometry.
- Retain stable feature identities, source scale, coordinate reference, axis order, horizontal/vertical units, survey dates, accuracy and source lineage.
- Build an inland multimodal graph with port, terminal, rail yard, road link, waterway, industrial node, population centre and border-crossing nodes.
- Attach hinterland demand by time-dependent network access and generalized cost, not Euclidean distance.
- Use H3 or another hierarchical index only for coarse discovery / aggregation. A cell is not a cadastral parcel, harbour geometry, hydrographic survey or engineering unit.

### Stage 2 — generate candidate classes

Each opportunity object may generate one or more bounded concept hypotheses:

1. Natural sheltered harbour / bay.
2. Estuary / river-mouth port.
3. Exposed coast with engineered protection.
4. Offshore / deep-water terminal.
5. Segmented barrier-island / lagoon / multiple-inlet system.
6. Inland basin / canal connection.
7. River / lake / canal inland port.
8. Dry port / rail / road alternative.
9. Existing-port upgrade / brownfield expansion.
10. Distributed multi-port / terminal / hub network.
11. No-build / operational improvement.
12. Insufficient evidence / no marine case.

Candidate generation is not a feasibility judgement. It only identifies what could be investigated and what evidence is missing.

### Stage 3 — calculate coast–hinterland network value

Use a time-dependent multimodal graph and origin–destination demand to estimate:

- delivered cargo cost, not just port-side handling cost;
- first / last mile, transshipment and transfer costs;
- waiting, reliability, schedule variance and missed windows;
- inventory carrying cost, spoilage and empty repositioning;
- throughput and capacity constraints;
- border and customs friction;
- industrial and agricultural market access;
- resilience under disruption and seasonal closure;
- distribution of gains and costs across places and people.

Keep observed freight, reported terminal capacity, modelled OD flows, inferred catchments and hypothetical demand separate. Do not double count trade flows or count operator revenue, user price, payroll, tax, subsidy and social benefit as independent gains.

### Stage 4 — apply existing Foundry evidence gates

Compose, do not copy:

- [#65](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/65): geospatial asset catalog.
- [#115](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/115): maritime evidence projection.
- [#129](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/129): site-suitability / hard-gate policy.
- [#130](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/130) and [#140](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/140): GIS and conservative spatial overlay.
- [#131](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/131) and [#132](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/132): interval-aware Pareto analysis.
- [#133](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/133) and [#134](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/134): EVSI and spatial investigation design.
- [#135](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/135) and [#136](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/136): rights, freshness, coverage, datum, CRS and provenance.

A stale bathymetric cell, unknown vertical datum, insufficient coverage, unresolved rights, source restriction, missing ecological evidence or invalid geometry cannot silently become zero, safe, absent or eligible. The result should say NeedsEvidence, Unknown, Unsupported, Invalid or Ineligible as defined by the existing owners.

### Stage 5 — create paired portfolios

For every shortlisted geography, compare the same alternatives:

- no-build / operational change;
- existing-port upgrade;
- brownfield expansion;
- offshore terminal;
- barrier-island / artificial harbour;
- inland basin / canal;
- inland-waterway terminal;
- dry port / rail;
- distributed network.

A family may be excluded only with an explicit, inspectable reason. Preserve the alternatives that remain nondominated. Do not select a winner when evidence intervals overlap or stakeholder priorities make the result fragile.

### Stage 6 — uncertainty and adaptive investigation

Stress the search against demand, trade, capex, dredging, sediment supply, waves, surge, tides, river flow, sea level, energy price, financing, ecological constraints and regulatory scenarios.

Recommend the next investigation based on expected decision value: bathymetry, geotechnical work, ecological survey, tide / current measurements, freight-demand validation, rights verification, or community engagement. EVSI can rank discretionary studies, but cannot waive mandatory safety/legal/ecological investigations.

## 4. Data sources and fitness for use

### Global screening

**GEBCO_2026** is a global land/ocean terrain model at 15 arc-second spacing with a Type Identifier Grid. It is useful for coarse global screening. Its construction includes a base grid blending measured and estimated bathymetry with satellite-gravity-derived predictions, augmented with regional grids. Retain the TID and never treat every cell as local multibeam survey evidence or a navigation chart.  
Sources: https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid and https://www.gebco.net/data-products-gridded-bathymetry-data

### Water levels and storm surge

**Deltares GTSM** offers global tides, water-level and storm-surge context; the project page describes v4.1 at approximately 1.25 km coastal resolution. Use for regional screening / boundary context and scenario selection, not local port design.  
Source: https://www.deltares.nl/en/expertise/projects/global-modelling-of-tides-and-storm-surges

### Waves

**Copernicus Marine global wave reanalysis** provides broad ocean-wave context at approximately 0.2-degree scale. It cannot resolve harbour-scale diffraction, resonance, overtopping or local wave transformation.  
Source: https://data.marine.copernicus.eu/product/GLOBAL_MULTIYEAR_WAV_001_032/description

### Regional and local data

**EMODnet bathymetry** provides products and service metadata in covered European marine basins; it is not global coverage. For any real site, obtain appropriate national hydrographic surveys, multibeam / singlebeam metadata, uncertainty surfaces, survey age, tide reduction, vertical datum and permitted-use terms.  
Source: https://emodnet.ec.europa.eu/en/bathymetry

**IHO S-100** products (including S-101, S-102, S-104, S-111, S-124 and S-129) provide versioned interoperability profiles. Product edition / issue / update / coverage / datum / uncertainty and source status must remain explicit; conformance does not certify local navigation suitability.  
Source: https://iho.int/en/s-100-standards-in-force-0

### Port and network performance

**World Bank / S&P Global CPPI 2025** is a comparative operational benchmark centred on total container-vessel time in port. It is useful for existing-port diagnostics, but affected by wider shipping conditions and not a site-suitability or hinterland-development score.  
Source: https://www.worldbank.org/en/topic/transport/publication/cppi

**UNCTAD LSCI / PLSCI** provide country- and port-level liner-shipping connectivity context, not a full cargo-demand matrix.  
Source: https://unctadstat.unctad.org/insights/theme/111

Use national port authorities, rail / road operators, customs, trade and production statistics, water / energy agencies, land-use and environmental authorities, population and employment sources for the regional appraisal. Every source needs licence, geography, time, unit, CRS/datum, uncertainty, coverage and permitted-use review.

## 5. Research evidence that changes the design approach

A 24 September 2026 Nature Communications study used five years of shoreface-inclusive post-storm sediment observations and found sediment redistribution varied by barrier-island geomorphology. It reports that overwash fluxes were low relative to seaward fluxes and inlet-related landward transport in the studied system; post-storm recovery remained incomplete despite human intervention. The implication for Port-City Foundry is that coarse sediment assumptions and generic island shapes are not enough: morphology-specific sediment budgets and long-term field evidence are central to the benchmark.  
Source: https://www.nature.com/articles/s41467-026-77143-6

The World Bank's 28 August 2026 Africa integration agenda emphasizes that economic value depends on connected production networks, reduced trade friction, transport corridors, power, customs, standards and regional public goods—not ports in isolation.  
Source: https://www.worldbank.org/en/brief/2026/08/25/integrating-africa-from-threads-to-hubs

OECD et al. (23 February 2026) similarly argues that African corridors should be embedded in local production and territorial development plans, with geospatial data and local planning capacity supporting balanced development rather than standalone transit routes.  
Source: https://www.oecd.org/en/publications/corridors-trade-and-local-development-in-africa_660aa24e-en.html

## 6. Benchmark plan

Use frozen synthetic cases and time-causal retrospective public-reference cases, stratified by geography class. Historical backtests must use only the information available at the historical decision date. Do not leak post-construction knowledge into the candidate search. Hold out entire geographical regions or coast/harbour families rather than random adjacent tiles.

### Metrics

- Candidate retrieval recall@K, precision@K and nDCG, stratified by natural harbour, estuary, exposed coast, offshore, inland waterway and dry-port classes.
- Candidate-family coverage and reason-coded NeedsEvidence / Ineligible outcomes.
- Zero false-eligible outputs on hard-gate, rights, stale-source, datum, ecological-exclusion and incomplete-coverage negative controls.
- Network catchment accuracy against exact graph shortest-path / least-cost references; compare with Euclidean-nearest-port baseline.
- Robust Pareto stability, rank reversals and regret under demand, capex, O&M, discount rate, climate and preference perturbations.
- Investigation value and the number of expensive physical-model runs avoided without missing reference-feasible candidates.
- Runtime, memory, data volume, solver calls and human investigation burden under pinned compute budgets.
- Exact reproducibility from pinned source, transform, policy, candidate and solver manifests.

### Initial targets

- 100% expected disposition on planted hard-gate / evidence negative controls; zero false-safe and false-eligible.
- 100% recall on explicitly enumerated small synthetic reference cases (a contract test, not global geographic skill).
- At least 95% recall@K on a preregistered held-out public-reference candidate set, with precision, coverage and class stratification also reported.
- At least 30% fewer candidate evaluations or expensive solver calls than exhaustive generation on the same frozen search budget, with no loss of reference-feasible candidates and no hard-gate violations.
- Exact graph-distance / generalized-cost reference agreement on small network cases.
- Report all rank reversals and uncertainty; do not collapse uncertain sites into one forced rank.

The targets are hypotheses to test, not current achievements. A known port is not automatically a positive label for every nearby site, and a project that was built is not a causal proof of its economic return.

## 7. Southern African first study — no preselected winner

After the synthetic suite, build a source inventory for the Southern African coastal–inland production network. Potential reference nodes may include existing coastal gateways and inland corridors serving South Africa and neighbouring landlocked markets. They are study objects, not a preselected answer.

Compare:
- existing-port operations and expansion;
- existing rail / road corridors and inland terminals;
- inland production, agriculture, mining and manufacturing demand;
- border-crossing / customs and transfer friction;
- power, water, workforce and repair capacity;
- alternative coastal gateways and port redundancy;
- climate, sediment, flood, ecological and rights constraints.

Do not declare Durban, Richards Bay, Maputo, Ngqura, Walvis Bay, Beira or another location the preferred answer from global screening grids. Choose a study corridor only after data rights, local survey coverage, source quality, land/water-rights, environmental, demand and stakeholder coverage are audited.

## 8. Cross-repository ownership

- **Sol-Atlas:** search decomposition, evidence-aware spatial discovery, coast–hinterland graph projection, candidate portfolios and visual comparison. Compose existing Foundry policy/asset/provenance/Pareto/EVSI owners.
- **Symtropy:** deterministic scenario execution, transport/economic optimisation, ocean/estuary model profiles and versioned adapters to specialist solvers. Do not elevate game/rigid-body physics into hydrodynamics, sediment or geotechnical authority.
- **Symthaea:** translate service goals into candidate design hypotheses, explore alternatives and identify missing evidence. Do not invent rights, environmental status, demand or solver results.
- **Mycelix:** participant authority, commitments, logistics transactions, rights/consent provenance and construction decisions. A model output is not a permit or commitment.

## 9. Claim ceiling and non-goals

This work produces an auditable shortlist of investigation hypotheses. It does not certify buildability, navigation safety, structural stability, flood protection, environmental acceptability, legal rights, community consent, planning approval or investment bankability. Global sources screen; local measurements and qualified independent professional review establish engineering evidence. A rendered map marker is not a finding.

---

**Next execution step:** freeze the first source inventory and 30+ synthetic coast–hinterland candidate cases, then build the independent discovery verifier before connecting real geospatial datasets.


## 10. African port–hinterland redundancy is a first-class design variable

A 2026 *Transportation Research Part A* study, *Redundancies of African port hinterlands and their implications for new container port investments* (206, April 2026, 104920), examines the relationship between port accessibility, overlapping hinterlands and redundancy. Its reported results show that accessibility and redundancy do not move together in a simple way: Northern and Western Africa have high accessibility and redundancy in the study; African landlocked countries show low accessibility but relatively high redundancy; new port investments and rail upgrades can alter this trade-off. These are study-level findings, not local ground truth or a recommendation for any named site. https://doi.org/10.1016/j.tra.2026.104920

The opportunity engine must therefore optimize not only average accessibility but also **effective gateway redundancy**:

- Count independent feasible gateways and routes, not merely nearby port markers.
- Model common-mode failures: shared rail corridors, bridges, border crossings, floodplains, coastal hazards, power dependencies, chokepoints and carrier concentration.
- Measure access cost and capacity to multiple gateways under normal and disrupted conditions.
- Use network cut sets / edge-disjoint path analysis only when the graph edges have evidence-backed capacities and the independence assumptions are explicit.
- Compare a new port against alternative investments that strengthen existing gateways, add inland terminals, remove shared bottlenecks, or diversify border and rail access.
- Distinguish traffic diversion from newly served demand; do not claim that a second port creates new economic value merely because its throughput rises.

Add benchmark metrics: access and redundancy curves by hinterland; effective independent gateway count under declared failure scenarios; unmet demand and recovery time after common-mode vs independent failures; redundancy/access improvement per lifecycle dollar; diverted vs additional demand; and ranking stability as freight demand and failure correlations vary.

This should become a core component of GEO-OPPORTUNITY-001, not an ornamental "resilience score". A region with three ports behind the same single rail corridor may have lower true redundancy than a region with two gateways reached by genuinely independent routes.
