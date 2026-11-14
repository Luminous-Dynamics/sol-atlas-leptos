# Multi-family Housing Benchmark v0.1

**Status:** synthetic-first benchmark protocol; no real apartment design or building performance is qualified.
**Parent:** [HOME-MULTIFAMILY-COMPILER-001 #840](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/840).
**Initial candidate:** `SYN-MF-COURTYARD-004`, a two-floor / four-unit synthetic building-level geometry fixture.

## 1. Research thesis

The objective is not to maximize dwelling count or produce more apartment renders. It is to improve **qualified resident outcomes per site, per lifecycle rand, and per unit of public infrastructure** without concealing the distribution of outcomes between apartments.

The decision object is nested:

`resident / household -> apartment -> floor -> building -> block / neighbourhood -> city services`

Every level must expose its own requirements, assets, ownership, maintenance, resource constraints, service trajectories and failure modes. A building-average result cannot compensate for a unit that fails a resident's hard requirement or essential service floor.

## 2. Johannesburg planning context

The City of Johannesburg's council-approved Spatial Development Framework 2040 (2024) supports a compact, polycentric city, mixed land use, infill and redevelopment, and proximity to jobs, public transport, social facilities and bulk infrastructure. It also explicitly states that densification alone does not make neighbourhoods liveable or sustainable.

Primary sources:
- City of Johannesburg SDF 2040: https://joburg.org.za/documents_/Documents/Johannesburg-Spatial-Development-Framework-2040.pdf
- City SDF page: https://joburg.org.za/documents_/Pages/Spatial-Development-Framework-2040.aspx
- UN-Habitat project description: https://unhabitat.org/spatial-development-framework-2040-city-of-johannesburg-metropolitan-municipality

The SDF is strategic planning context, not a parcel-specific development entitlement. Each actual site still requires exact zoning/land-use, building-line, height/FAR/coverage, title/servitude, environmental/heritage, bulk-service and authority review. No SDF density guideline should be hard-coded as a universal maximum, minimum or permission.

JOSHCO's published portfolio spans social/rental housing, greenfield and brownfield development, hostel redevelopment, inner-city conversions/refurbishment, communal/transitional housing and student accommodation. This is a useful local delivery and operating comparator; portfolio scale does not establish that one typology is suitable for every site or household.
- https://www.joshco.co.za/about-joshco/who-we-are/

## 3. Current industry baseline

The comparison must be specific and fair about product capability and maturity.

| Comparator | Publicly described capability | What it does not establish by itself |
|---|---|---|
| Autodesk Forma Building Layout Explorer | Autodesk's October 2026 announcement describes experimental multifamily layout exploration against unit mix, circulation and core placement. Autodesk characterizes current output as a starting point for exploration. | Resident acceptance, fire/accessibility compliance, construction release, real-site entitlement or whole-life service performance. |
| TestFit Site Planning / Site Solver | Multifamily and mixed-use site planning, unit mix, parking, setbacks, zoning/site constraints, feasibility metrics and Revit/CAD-style hand-off. | Exact current South African legal applicability, physical performance, professionally approved design or final cost certainty. |
| Finch | Residential/high-rise layout exploration, firm standards and plan libraries, unit distribution, BIM geometry and interoperability claims. | Independent validation of generated outputs, local project compliance, physical acceptance or resident outcomes. |
| Revit / Archicad + specialist tools | Detailed authoring and coordinated engineering workflows, subject to the selected toolchain and professional team. | Automatic preservation of every resident preference, cross-system evidence freshness or independently measured lifecycle outcome. |
| Human-led integrated architecture/engineering/QS | Professional judgment and context-specific coordination. | Zero omissions, low revision burden, transparent evidence dependency or reproducible performance without a measured process baseline. |
| Classical constraint / Pareto optimisers | Deterministic constraints, repeatable option search, transparent objectives and baselines. | Correct requirements, model assumptions, professional validity or source evidence unless explicitly supplied. |

Public product sources:
- Autodesk AU 2026: https://blogs.autodesk.com/forma/2026/10/14/what-autodesk-forma-announced-au-2026/
- TestFit Site Planning AI: https://www.testfit.io/product/site-planning-ai
- TestFit Site Solver: https://www.testfit.io/product/site-solver
- Finch product: https://www.finch3d.com/product

The test should compare equivalent deliverables, not a Luminous model against a commercial screenshot or a commercial feasibility diagram against a professionally reviewed BIM package.

## 4. Research evidence that changes the benchmark

### Daylight, view and privacy are distinct

A July 2026 *Building and Environment* study assessed 10,420 rooms across 35 New York apartment properties. It reports that view quality and daylight are affected by density, while visual privacy behaved largely independently of view in that dataset; the reported correlation between privacy and view was low. Its proposed resident-centric metrics and numerical thresholds are New York-specific and must not be treated as South African standards.

- Kim et al. (15 July 2026), DOI 10.1016/j.buildenv.2026.114680: https://www.sciencedirect.com/science/article/pii/S0360132326004853
- SimBuild 2026 in-situ metric-validation study: https://publications.ibpsa.org/conference/paper/?id=simbuild2026_1363

Design implication: keep daylight, glare, view and privacy as separate outcome vectors. Evaluate every occupied room/unit and report distribution and lower-tail results.

### High-rise water distribution is its own engineering problem

A review available online 27 October 2026 synthesizes 53 studies of water supply/distribution systems in high-rise residential buildings and identifies modelling, optimization and evidence gaps. It supports dedicated pressure, pumping, storage, energy, demand and outage analysis—not a generic building-water placeholder.

- Zhang et al. (2026), DOI 10.1016/j.jobe.2026.117679: https://www.sciencedirect.com/science/article/pii/S2352710226025015

Design implication: household water continuity must be assessed at actual delivery boundaries and by floor/pressure zone. A building-level tank volume does not establish delivered service.

### Thermal resilience is temporal

A June 2026 review argues that residential thermal resilience under climatic extremes and power outages must be treated as a time-dependent process rather than a single annual-energy number.

- Gortych et al. (25 June 2026), DOI 10.3390/en19133006: https://www.mdpi.com/1996-1073/19/13/3006

Design implication: compare indoor-temperature/service trajectories, recovery and loss of service under matched heat/outage scenarios; do not infer resilience from annual energy or modelled peak efficiency alone.

### Fire resilience remains a non-compensatory gate

Recent high-rise fire-resilience research emphasizes lifecycle management and interactions among facade systems, vertical pathways, governance, maintenance and emergency response. A visually connected route graph is nowhere near an evacuation or fire-safety assessment.

- Tang et al. (15 July 2026), DOI 10.1016/j.jobe.2026.117080: https://www.sciencedirect.com/science/article/pii/S2352710226019029
- Dong & Shi (13 April 2026), *Nature Cities*: https://www.nature.com/articles/s44284-026-00430-2

Design implication: fire strategy, compartmentation, escape, smoke control, facade performance, maintenance and responsible authority review remain separate specialist evidence gates.

## 5. Typology comparison set

At least six families should be represented, with a repair/retrofit and no-build option retained:

1. Walk-up / low-rise courtyard.
2. Mid-rise courtyard / perimeter block.
3. Slab.
4. Point tower.
5. Mixed-use transit/economic-node block.
6. Adaptive reuse / conversion.

Each is a candidate family, not a preferred solution. A site may justify one, several or none. Unit mix and height are not sufficient to describe a design: floor plate, access core, unit orientation, facade exposure, common spaces, vertical services, plant, loading/waste and maintenance logistics matter.

## 6. v0.1 synthetic fixture and claim ceiling

Artifacts:
- `multifamily-building.schema.json`
- `multifamily-building-synthetic-v0.1.json`

The first fixture describes a synthetic two-floor, four-unit building with floor-local integer-millimetre rectangles, corridors, a stair, a lift, shared courtyard/terrace spaces, declared everyday portals and vertical links.

P0 does not model internal apartment rooms, wall thickness, structure, shafts, detailed doors/windows, parking, site setbacks, multiple blocks, fire compartments or specialist engineering. Its connections are for everyday/shared/service topology, **never emergency egress**. A lift identity is not step-free qualification. A stair identity is not an evacuation strategy.

Independent fixture replay checks:
- JSON parse and local schema-reference resolution;
- exact unit/space/floor identities;
- per-floor rectangle containment and positive-area overlap;
- portal segments against shared room boundaries;
- declared vertical-core/floor identity;
- explicit UNKNOWN service topology/capacity and NOT_ASSESSED specialist registers.

These are fixture-integrity checks, not a full JSON Schema validator, Rust implementation run, architectural review or physical result.

## 7. Benchmark dimensions

Report separate vectors. Do not collapse these into one ranking.

### Urban / site
- dwellings and resident-defined capacity per net/gross hectare;
- floor-area ratio, site coverage, open-space allocation and public/private/shared realm;
- access to jobs, public transport, schools, health, food and daily services;
- bulk water, electricity, wastewater, stormwater, road, fire-service and waste capacity, each source-qualified;
- land acquisition, remediation, site preparation and infrastructure contribution.

### Apartment and building quality
- unit area and clear dimensions against exact resident-authored requirements;
- daylight/glare, view, visual privacy, airborne/impact acoustics and thermal exposure by occupied room;
- per-unit storage, laundry, cooking, work/care, mobility equipment and outdoor-space satisfaction;
- vertical circulation availability, step-free route evidence and lift/stair outage trajectories;
- maintenance access, replacement path, deliveries, waste and emergency service access.

### Service resilience
- water litres at actual fixture/service boundary by source/use class, pressure, outage minutes, reserve estimation error and recovery;
- electricity availability/quality, kWh, peak kW, critical-load coverage and outage minutes;
- useful hot-water/thermal service delivered, unmet demand, recovery, losses and safe fallback;
- ventilation flow, pollutant-specific occupied-zone endpoints, humidity, noise, filter loading and maintenance;
- per-unit essential-service breach minutes; report median, worst-decile and worst-unit, not only building average.

### Cost and equity
- monthly total housing cost per unit: rent/finance, levies/service charge, utilities, maintenance reserve, insurance and transport;
- lifecycle cost and replacement/repair exposure;
- cost distribution by unit/tenure and declared affordability cohorts where lawful and privacy-preserving;
- daylight, acoustic, privacy, access and service outcome distributions by unit, floor and orientation;
- shared-amenity access, usage, governance, hours, operating cost and whether any essential private service is being replaced.

### Construction and lifecycle
- whole-workflow installed cost and schedule;
- design/factory/site labour, setup, yield, scrap, transport/lift, weather downtime, inspection and rework;
- first-time-right and tolerance/nonconformance;
- as-built deviation and service commissioning completeness;
- adaptation/repair/replace/disassembly time and cost;
- embodied and operational carbon/water under explicit lifecycle scope.

## 8. Hard gates

- 100% exact unit/space/floor identity and fixture geometry parity.
- Zero false-positive geometry passes in frozen adversarial controls.
- 100% non-negotiable resident requirements traced to the affected unit and evidence owner.
- Zero unit-level hard failures concealed by building averages or premium-unit gains.
- Zero geometry, topology or IDS checks promoted to fire, accessibility, structure, planning, daylight, acoustic, thermal, utility-capacity or occupancy approval.
- Zero stale PASS after relevant unit/core/floor/envelope/utility/product/site/rule revisions.
- 100% shared resource conservation and explicit service-boundary accounting.
- No service capacity, cost, transport access or site entitlement filled with assumed defaults.
- No apartment layout selected solely because it maximizes unit count, parking or saleable area.

## 9. Research targets and improvement hypotheses

Targets below are **proposed experiments**, not standards or achieved performance. Freeze them only after pilot variance, comparator capability and assessor agreement are measured.

- **Intent / correctness:** 100% preservation of non-negotiables; zero unsupported requirement inference.
- **Search productivity:** target at least 20% more professionally reviewable non-dominated options per fixed expert-hour than the best matched baseline, without reducing defect recall or increasing false passes.
- **Revision productivity:** target at least 25% lower professional re-review effort for a fixed suite of benign and consequential changes, with zero stale relevant PASS results.
- **Cost-of-place:** test whether an option can reduce total monthly/lifecycle housing cost by at least 10% at matched resident requirements, service floors, location/access and quality. If the matched feasibility set cannot support this, publish the frontier rather than forcing the target.
- **Resilience:** test whether a design can reduce essential-service breach minutes by at least 20% under matched water/power/heat/outage scenarios without transferring shortages to other households or neighbours.
- **Manufacturing:** test whether DfMA reduces installed rework and schedule variance while retaining unit-level acoustic/fire/moisture/thermal interfaces. No universal prefabrication savings percentage is assumed.

All numeric hypotheses are subject to preregistration, sufficient independent samples and confidence/uncertainty reporting. Hard safety/rights/quality gates cannot be traded for speed, cost or yield.

## 10. Matched comparison protocol

Every method receives the same:
- resident brief and non-negotiables;
- surveyed site, parcel envelope and actual jurisdiction/rule edition;
- unit mix / household programme and common-space requirements;
- weather, climate and surrounding-building context;
- utility and transport capacity evidence;
- budget/cost boundaries, material supply assumptions and delivery method options;
- output requirements and professional review scope.

Pin software version, license/feature access, locale, libraries, prompt/settings, scripts, hardware, human time, compute, model versions and export configuration. Keep sealed tasks and blind professional review out of development/tuning.

Comparators:
1. Conventional integrated architect / engineer / quantity-surveyor team.
2. Best-of-breed commercial tools: Forma, TestFit, Finch and Revit/Archicad where equivalent functions are available.
3. Classical CP-SAT/MILP/constraint-based search and Pareto optimizers.
4. Repair/retrofit/adaptive reuse and no-build alternatives.
5. Luminous's own evidence-carrying workflow.

Report candidate options, time, review effort, requirement loss, independent defect recall/false blocks, IFC semantic loss, cost completeness and post-occupancy performance. Publish all failed attempts and exclusions.

## 11. Standards and authority boundaries

Use IFC 4.3.2.0 / ISO 16739-1:2024 for open built-asset exchange and IDS 1.0 for information requirements. IDS is limited to alphanumeric IFC information checks; it does not carry out the advanced geometric/physics/safety assessment.

- IFC: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/

Real project rules must be pinned to the exact applicable jurisdiction, edition, effective date, authority and use case. Professional fire, structural, geotechnical, building-services, accessibility, acoustic and environmental review remain separate. No benchmark score or software receipt is a permit.

## 12. Execution sequence

1. Stabilize the current single-home geometry PR #831 at its exact head.
2. Extend #833 / #832 to resident-authored unit programmes and whole-building evidence invalidation.
3. Implement a Rust multi-family contract qualifier and independent known-answer/adversarial corpus.
4. Add room-level geometry only after the building/floor/unit identity and topology contracts stabilize.
5. Add specialist result adapters for daylight/view/privacy, acoustics, thermal/ventilation, fire/accessibility, structural/foundation and utility performance.
6. Add IFC/IDS round-trip and semantic-loss receipts.
7. Compose #803/#814 DfMA delivery and #837 appliance/service interfaces for both private units and shared building systems.
8. Run matched commercial / human / classical-optimiser trials and blind professional review.
9. Progress to one surveyed and permissioned project only after independent review, controlled standards, service-capacity evidence and a competent delivery team.

## 13. Claim ceiling

This protocol, the schema and synthetic fixture do not establish a real site's development rights, infrastructure capacity, geotechnical conditions, structural or fire safety, accessibility, daylight/view/privacy/acoustic/thermal performance, product safety, affordability, resident satisfaction, cost savings, buildability or occupancy. No construction or operational action is authorized.
