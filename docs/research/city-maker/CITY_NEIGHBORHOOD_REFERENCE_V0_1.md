# CityMaker neighbourhood reference contract v0.1

**Purpose:** define one bounded, multi-system neighbourhood comparison without creating a new city simulator or duplicating the repositories that already own its physical, service, material and operational semantics.

**Status:** interface profile and benchmark design only. All numerical examples in future fixtures must be explicitly synthetic. No Johannesburg pilot area is selected by this contract.

## 1. The unit of comparison

A neighbourhood alternative is evaluated against a **shared service bundle over a declared horizon**.

The bundle is expressed as:
- resident-defined tasks and essential services;
- spatial/network accessibility requirements;
- required service capacity and time windows;
- housing/service quality and affordability floors;
- public-realm and ecological constraints;
- productive, material, utility and maintenance dependencies;
- construction and transition phases;
- outage/heat/transport/supply scenarios.

The comparator must not reduce this to a density, 15-minute threshold or single city-quality score.

## 2. Scope identity

Every comparison must bind one immutable scope identity:

| Dimension | Required binding |
|---|---|
| Study area | Stable geometry subject ID, exact revision/digest, source provenance, CRS and legal/use status |
| Geography | Boundary version, native source support, spatial resolution and permitted display resolution |
| Time | Common evaluation start/end, service time windows, construction phasing and scenario horizon |
| Population | Population-frame ID, source vintage, aggregation level, denominator, weighting/error metadata and known coverage |
| Functional unit | Explicit service/asset unit, denominator and period for every objective/constraint |
| Service bundle | Versioned requirements, groups, service categories, demand units and hard floors |
| Scenario family | Exact shared baseline/event traces and modelled/observed/synthetic classification |
| Decision authority | Resident/community participation and responsible statutory/professional/asset authorities, kept distinct |

Changing geography, population frame, functional unit, service requirements, horizon or scenario family changes the scope digest. Alternatives with different scope digests are NotComparable, not ranked.

## 3. Starter neighbourhood programme

The reference basket is configurable, not a requirement that every service be physically located inside the boundary.

Candidate domains:

1. **Homes and daily living:** accessible dwelling cohorts, shared facilities, basic services and housing affordability.
2. **Health and care:** primary care, emergency-response interface, childcare and support services.
3. **Education and skills:** early childhood, school, adult learning and vocational training.
4. **Food and commerce:** food retail/markets, optional urban agriculture, distribution and local commerce.
5. **Civic and social life:** community facilities, local governance/participation, cultural and shared spaces.
6. **Public realm and ecology:** parks, shade, recreation, safe routes, green-blue infrastructure, stormwater and biodiversity.
7. **Mobility and logistics:** walking, cycling, transit access, freight/delivery, emergency vehicle access and accessible crossings.
8. **Productive and repair capability:** workshop/maker/repair facilities, microfactory or panel-cell interface, storage, kitting and local supplier access.
9. **Circular material system:** deconstruction/reuse/salvage, testing, refurbishment, recycling and waste collection interfaces.
10. **Utilities and digital:** water, sanitation, stormwater, energy, thermal service, waste, communications and maintenance.
11. **Safety and emergency continuity:** fire/rescue, evacuation, medical response, incident communications and accessible assembly points.

An off-site service can be part of a neighbourhood service bundle if its evidence-backed network access, capacity and operating window are included. Do not force a service into an area to satisfy a checklist.

## 4. Morphology hypotheses

The first comparative study should include a maintain-existing baseline and avoid deciding a winner in advance.

| Alternative family | Hypothesis | Necessary counter-tests |
|---|---|---|
| Compact mixed-use blocks | Shorter daily trips and shared infrastructure may improve access and resource efficiency | Overcrowding, affordability, heat, open-space quality, service capacity and freight/noise conflict |
| Polycentric cluster | Multiple nodes may reduce single-point dependencies and improve distributed access | Duplicated facilities, low utilization, service quality, operating cost and transit connectivity |
| Courtyard/perimeter blocks | Shared open space and legible frontages may support social use and passive design | Sun/shade, ventilation, privacy, fire access, public/private access and disability requirements |
| Lower-rise incremental / retrofit-first | Reuse and phased upgrade may reduce disruption and capital demand | Hidden defects, service gaps, accessibility, hazardous materials and insufficient capacity |
| Civic/service hub with distributed satellites | Shared high-capacity services may improve specialist capability | Centralized failure, travel burden, queueing, affordability and last-mile access |
| Maintain-existing / minimal intervention | Existing assets may dominate costly redevelopment in some contexts | Deferred maintenance, current unmet needs, resilience and hidden risk |

These are scenarios, not validated typology findings. The study must describe the assumptions for every option.

## 5. Owner-native evidence interfaces

The neighbourhood adapter consumes, validates identity/scope, spatially projects and reports—not recomputes—specialist outputs.

| Domain | Authority to consume | Required subject/evidence |
|---|---|---|
| Building/facility realization | Symthaea BUILT-ENV #6032, BIM #6036, site #6034 | BuildingProfile / IndustrialFacilityProfile / SiteInfrastructureProfile, design revision, realization status and exact source receipt |
| Operational service | Symthaea CIV-SERVICE #5949; Symtropy CityKit #3768 | Service status, capacity, operating window, dependencies, resource unit and exact subject receipt |
| Productive capability | Symthaea CIV-BOOT #5774; Symtropy #1834; Mycelix #4814 | Capability match, process/equipment/skill prerequisites, supplier/inventory/lead-time receipts |
| Physical/habitability | Symtropy #1790/#1848/#1900 | Solver/model scope, input digest, weather class, convergence, validation, trajectories and uncertainty |
| Materials/urban metabolism | Mycelix Living-City PR #5035; Symtropy #1856/#1984 | Physical flow balances, grade/functional unit, intervention results, material batch/custody and source revision |
| Construction productivity | Symtropy #3773/#3605; Symthaea #4795/#6040 | Accepted output, design/BOM/configuration, full labour denominator, metrology/inspection and commissioned workcell profile |
| Spatial accessibility | Sol-Atlas #555/#557; R5/OTP references | Network/mode/time route results, service windows/capacity, group requirements and route/source digest |
| Continuity and heat | Sol-Atlas #563/#574/#593/#595 | Exact phase timeline, weather class, building cohort/model/measurement claim level and intervention receipts |
| Alternatives/portfolio | Symtropy #2529; Sol-Atlas #131/#214 | Owner-native dominance certificate, VOI request, project dependencies, shared budget/workforce and sequence outcome |
| Custody/decision rights | Mycelix #5149 plus actual accountable parties | Lot/asset custody, inspection/approval/revocation and separately recorded community/legal decision |

The adapter may not turn a mapped asset into an operational service, a service point into adequate capacity, a material stock into qualified supply, a supplier offer into productive capability, or a simulation into physical measurement.

## 6. Cross-domain constraints that cannot be averaged away

At minimum, the neighbourhood model must be able to declare:
- rights/land-use and access permissions;
- building and infrastructure life-safety / egress requirements;
- accessible route and facility requirements for declared groups;
- essential service capacity and continuity floors;
- ecological/flood and public-space constraints;
- affordability/displacement safeguard conditions;
- water/energy/resource budgets and dependency requirements;
- construction access, temporary detours, laydown and emergency routes;
- source rights, privacy and minimum evidence-quality rules.

A known violation produces Infeasible/Blocked according to the owner contract. Missing or stale evidence produces Unknown. A scope/unit mismatch produces Unsupported or NotComparable. No high performance on another metric compensates for a hard floor violation.

## 7. Matched scenario protocol

Every morphology alternative receives the same:
- study area, population/service frame, horizon and service task bundle;
- source snapshots, weather/event traces and network revision;
- declared demand and functional units;
- utility, capacity and resource assumptions;
- construction phasing and delivery window;
- scenario events (heat, power, water, transit, route, supplier, facility and repair outages).

Observed events, typical weather, design extremes, future projections and synthetic fixtures stay distinct. Do not invent hazard probabilities or use TMY as heatwave data.

## 8. Metrics — report a vector

### Resident task and service
- known usable service opportunities by group, mode and time;
- capacity/demand ratio by category/time window;
- group share with no known usable service or capacity shortfall;
- p50/p90/worst-decile generalised travel cost;
- step-free, affordable and eligible access share;
- service continuity and stable recovery by phase.

### Public realm and spatial form
- block permeability, severance, route detour and crossing evidence;
- public-space reach, verified public-use state, quality/comfort/shade and unknown share;
- jobs/service/repair access without conflating proximity and capacity;
- ecological/flood constraint footprint and uncertainty.

### Buildability and operations
- accepted functional outputs and bottlenecks;
- qualified and deliverable material supply by phase;
- component variants, logistics, storage/laydown and access conflicts;
- maintenance/repair time and local capability versus imports;
- utility/service dependencies, peak demand and outage behaviour.

### Affordability, rights and distribution
- housing/service cost per declared functional unit;
- distribution of benefits, access burdens, noise, route closures and heat burden;
- displacement/tenure safeguards and known unknowns;
- resident participation, dissent, appeal and decision-right coverage.

No composite score is the default. Any policy weight must be versioned, authorized and accompanied by sensitivity and distributional effects.

## 9. Standards and method crosswalk

### South African neighbourhood planning

Use the DHS/CSIR Neighbourhood Planning and Design Guide (Red Book) as the local planning-method baseline for layout, public open space, housing/social facilities, transport, water, sanitation, stormwater, solid waste, energy, safe communities and universal design. Map where the guide is applicable and record any deviation; do not treat guidance as a building approval or current law.

- https://www.dhs.gov.za/node/154
- https://www.csir.co.za/neighbourhood-planning-and-design-guide-red-book

### ISO city indicator family

- ISO 37120:2018 remains the published city-service/quality-of-life standard. Edition 3 has entered the Draft International Standard stage (ISO/DIS 37120 registered 2026-11-05). The draft is not a published international standard.
- ISO 37122:2019 smart-city indicators were confirmed current in 2024.
- ISO 37123:2019 resilient-city indicators were confirmed current in 2025.
- ISO 37124:2024 guides use of ISO 37120/37122/37123.

References:
- https://www.iso.org/standard/68498.html
- https://committee.iso.org/standard/90316.html?browse=tc
- https://www.iso.org/standard/69050.html
- https://www.iso.org/standard/70428.html
- https://www.iso.org/standard/80290.html

These are municipal indicator frameworks, not detailed neighbourhood design standards. Reproduce comparable indicators where definitions allow; label the finer network/capacity/phase results as CityMaker extensions.

## 10. Minimum v0.1 synthetic conformance cases

The parent issue #596 owns 28 cases, including reachable-but-undersized clinic, once-counted shared capacity, step-free route shortfall, unverified public-space status, ecological/flood hard constraint, affordability/displacement guard, freight/noise conflict, unqualified supplier, wrong material grade, phase mismatch, double-allocated water, TMY/heatwave confusion, ward-to-building promotion, stale transit, construction severance, fire-route assumptions, noncommissioned asset, wrong owner receipt and shared-resource portfolio conflict.

All fixtures are synthetic. The oracle must recompute dispositions from input facts and must not trust supplied expected outputs or candidate alternative status labels.

## 11. P1 Johannesburg pilot route

1. Finish rights/data readiness in Sol-Atlas #561.
2. Agree a bounded area with affected residents, landowners, custodians and relevant authorities; no site is selected by this contract.
3. Acquire only rights-cleared sources with explicit spatial/temporal support.
4. Establish resident-defined task/service requirements and dissent/appeal process.
5. Reproduce a selected Red Book and ISO city-indicator subset.
6. Consume owner-native building, service, utility, material, production, access, climate and habitability receipts.
7. Compare alternatives and do field audits, recording all unknowns and model/measurement disagreement.
8. Hand feasible project bundles to Sol-Atlas #214 for staged portfolio analysis.
9. Publish only privacy- and rights-reviewed aggregate evidence.

## 12. Claim boundary

This contract is a reference integration model, not a selected Johannesburg development proposal, local service-capacity survey, building approval, certified safe-neighbourhood design, community consensus or claim that a morphology is optimal.


## 13. Machine-readable profile

The companion JSON Schema is `CITY_NEIGHBORHOOD_EVALUATION_PROFILE_V0_1.schema.json`. It binds the study-area geometry, scope digest, time horizon, population frame, service requirements, alternatives, common scenario traces, non-compensatory constraints and owner-native evidence references. The schema makes cross-owner references explicit; it does not validate the truth or completeness of the referred evidence by itself.
