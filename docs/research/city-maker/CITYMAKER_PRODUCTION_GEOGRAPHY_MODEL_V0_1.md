# CityMaker production geography model v0.1

**Status:** integration/data contract proposal for CITY-PRODUCTION-GEOGRAPHY-001. This is not a location recommendation or industrial park certification.

## 1. Model purpose

Represent the spatial coupling between:
- productive capacity and its upstream dependencies;
- people, jobs and workforce access;
- freight, deliveries and reverse logistics;
- energy, water and shared utilities;
- material recovery and industrial symbiosis;
- land-use, environmental, public-space and community constraints.

The model is a **read-only projection and scenario boundary** over owner-native evidence. It does not create production inventory, physical flow accounting, routing, scheduling, site suitability, zoning authority, or community consent.

## 2. Graph model

### Nodes

Each node is typed and revision-bound:
- **ProductionSite:** factory, workshop, fabrication cell or industrial estate.
- **Supplier:** producer or provider of an input, component, tool or service.
- **RepairRecoverySite:** inspection, repair, remanufacturing, salvage, sorting or recycling facility.
- **LogisticsNode:** depot, consolidation point, terminal, loading bay or staging area.
- **WorkforceOrigin:** aggregate population/workforce zone, never individual household points.
- **ServiceNode:** housing, education, health, food, childcare or other worker-support service.
- **UtilityConnection:** declared electricity, water, wastewater, district heat or other utility interface.
- **LandParcel:** site candidate with explicit zoning/rights and environmental evidence.
- **DemandNode:** time-bounded construction, maintenance or product demand.
- **PublicRealm:** street, footway, curb, park or sensitive community interface.

Every node has:
- stable owner-native ID and revision;
- node type and scope;
- geometry plus CRS/spatial precision;
- source owner, record reference, retrieved-at and content digest;
- evidence state and uncertainty semantic;
- licence/rights state and allowed projection fields;
- validity interval and stale/revocation behavior.

### Edges

Edges represent typed relationships, not generic proximity:
- **supplies:** qualified product/grade/quantity with availability window;
- **depends_on:** production/equipment/tooling/maintenance dependency;
- **transforms:** owner-native process relation and input/output profile;
- **moves_by:** time-dependent network route/mode/vehicle profile;
- **serves_workforce:** group/mode/shift access evidence;
- **connects_utility:** utility provider and evidenced capacity/quality;
- **shares_resource:** declared shared input/output flow from Living-City;
- **recovers_from:** material or component return path and qualification state;
- **constrained_by:** zoning, environmental, flood, heat, rights or public-space constraint;
- **demands_from:** project/work-package demand with design revision and delivery window.

An edge's existence is not proof that the relationship is currently usable. Capacity, rights, currentness, grade, time, route and acceptance states remain separate.

## 3. Owner-native data boundary

| Domain | Authoritative owner | What the spatial model consumes | What it must not infer |
|---|---|---|---|
| Supplier/inventory/production dependencies | Mycelix CIV-ECON-005 #4814 | exact supplier, offer/inventory/lead-time/substitute/dependency receipt | local productive capacity from a map pin |
| Material/energy/water balance and distribution | Mycelix Living-City PR #5035 | flow/run receipts, units, boundaries and distributional outputs | resource savings from mapped proximity |
| Service and productive capability feasibility | Symtropy CityKit #3768 | exact capability/service capacity and feasibility receipts | productive closure or process capability independently |
| Panel manufacturing and accepted output | Symtropy #3773 / #3605 | accepted output, first-pass/final yield, labour and quality evidence | productivity from nominal machine cycle only |
| Material intervention and qualification | Symtropy #1856 / #1984; Mycelix #5149 | intervention/grade, lot/custody/inspection/revocation evidence | reusable/qualified material from a mapped stockpile |
| Worker and resident accessibility | Sol-Atlas #555 | mode/group/time-specific access result | access from straight-line distance alone |
| Material-loop/delivery projections | Sol-Atlas #569 / #573 | qualified location/route/time/catchment projection | stock allocation or accepted delivery |
| Place-based alternatives and comparison | This issue | site/cluster alternatives, shared geography, distributional consequences | permits, land title, consent or approvals |

A spatial projection must bind the exact upstream record, source revision, receipt digest, evaluation scope and claim ceiling. A producer's self-asserted status does not replace the independent owner verifier.

## 4. Alternative profiles

Every alternative shares one scope digest and demand baseline. Profile the following:
- centralized industrial estate;
- distributed local workshops/microfactories;
- polycentric specialist nodes;
- hybrid central production plus local finishing/repair;
- brownfield retrofit/reuse;
- maintain-existing/no-new-capacity baseline with imports and external suppliers retained.

Each profile specifies candidate sites, process/product families, capacity source receipts, commissioning phases, shared utility dependencies, freight/worker access, material loops and applicable hard constraints. No assumed local-content target.

## 5. Catchments and time

A production catchment is not a simple radius. It must declare:
- product/grade and process family;
- required quantity and delivery window;
- allowed freight mode, vehicle/payload/size constraints;
- network and timetable revision;
- shift schedule and workforce group/mode profile;
- service/utility availability window;
- seasonal or disruption scenario;
- evidence freshness/uncertainty.

Compute or import specialist route results; compare only matched graph/feed/mode/time assumptions. Keep freight, worker and customer trips distinct, then analyze shared road, curb, public-space and time-window competition.

## 6. Feasibility gates

A site/cluster is not eligible merely because land and road access appear on a map. Required, non-compensatory evidence may include:
- legal permitted use, land access and title/lease status;
- environmental authorization and protected-area constraints;
- contamination and remediation status;
- flood/heat/air-quality exposure where relevant;
- grid connection, load profile and reliability;
- water, wastewater and fire-service capacity;
- freight vehicle access, turning/loading and operating-hour permission;
- worker access, safety and required support services;
- noise, emissions, waste and public-realm constraints;
- community consultation, rights and distributional safeguards.

A required missing or stale datum yields Unknown/Blocked, not an assumed pass. Statutory and professional approval remain outside this model.

## 7. Comparison metrics

Keep separate and disaggregated:
- qualified productive output by product/grade/period;
- local / regional-national / imported supply share;
- supplier concentration, common dependencies and substitute depth;
- capacity utilization, commissioning time and service interruption;
- inventory coverage and delivery lead-time distribution;
- freight vehicle-km, tonne-km, delivery windows and handling transfers;
- worker access time/cost by group, mode and shift;
- shared utility demand/shortfall by time and source;
- material/energy/water exchange actually qualified under common units/boundaries;
- noise/emissions/heat/flood burden distribution;
- public-space conflicts and curb occupancy;
- land/infrastructure/connection cost and fully burdened cost per accepted functional unit;
- repair/recovery capacity and time to resume operation;
- uncertainty, coverage, freshness and unresolved evidence.

No scalar industrial-sustainability score. Pareto comparisons are routed to owner-native frontier/certificate services under matched scope and declared uncertainty semantics.

## 8. Synthetic known-answer corpus v0.1

At least 24 cases should cover:
1. centralized capacity with a common road bottleneck;
2. distributed nodes with insufficient qualified throughput;
3. hybrid redundancy with extra transfer cost;
4. capable supplier with unavailable delivery window;
5. worker private-car access but no declared public/active access;
6. time-of-day freight/residential conflict;
7. insufficient utility capacity;
8. shared utility over-allocation;
9. grade/period mismatch for industrial by-product reuse;
10. nearby recovery site with incompatible process/grade;
11. zoning status Unknown;
12. brownfield contamination Unknown;
13. single maintenance provider for multiple critical sites;
14. imported tooling/spares bottleneck;
15. correlated transport outage;
16. high mean workforce access but poor access for one group;
17. freight optimization worsening pedestrian/public-space burden;
18. local production reducing freight but increasing source-weighted energy/water burden;
19. citywide flow balance with local catchment deficit;
20. factory commissioning after demand date;
21. sites sharing one upstream supplier (not independent redundancy);
22. mapped candidate site with no owner-native capability receipt;
23. land-rights constraint invalidating a candidate;
24. alternatives with different scope, functional unit or horizon (NotComparable).

All numeric values are synthetic and cannot be presented as Johannesburg facts.

## 9. Reproducibility and review

The future oracle must:
- consume input-only data and not trust expected outcome labels;
- preserve Unknown, Blocked, Unsupported, Infeasible and NotComparable separately;
- reject duplicate flow/capacity accounting and shared-dependency false redundancy;
- be invariant to input ordering after canonical normalization;
- emit exact scope, input, owner revision, corpus and source digests;
- record all alternative outcomes, including dominated and excluded candidates;
- never select/approve a site or trigger procurement, dispatch, actuation or land access.

## 10. Research references

- Bai et al. (2026), industry-city co-prosperity and cross-scale integration, review of 132 studies: https://doi.org/10.1016/j.eiar.2026.108349
- UNIDO / World Bank / GIZ, Eco-Industrial Park Framework v2.0, four categories of performance: park management, environmental, social and economic: https://www.unido.org/sites/default/files/files/2021-04/An%20international%20framework%20for%20eco-industrial%20parks%20v2.0.pdf
- UNIDO industrial park planning and performance monitoring: https://ipp.unido.org/industrial-parks-guidelines/phase-six-monitoring-and-evaluation
- Buijs et al. (2026), urban freight integrated with land use and public space: https://doi.org/10.1080/01441647.2026.2732021
- City Logistics 4.0 systematic review: https://doi.org/10.1007/s10479-025-06835-x
- Schuppler et al. (2026), circular-city case synthesis: https://doi.org/10.1007/s44498-026-00018-y

## Claim ceiling

The model and synthetic corpus can qualify data contracts and bounded scenario semantics only. They do not establish real supply capacity, legal site suitability, industrial-park status, job creation, community consent, environmental acceptability or an optimal production geography.
